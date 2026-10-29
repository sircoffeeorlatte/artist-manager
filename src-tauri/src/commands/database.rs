use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

const CATEGORIES: [(&str, &str); 3] = [
    ("irl", "IRL"),
    ("anime", "Аниме"),
    ("3d", "3D"),
];
const KINDS: [(&str, &str); 3] = [
    ("photo", "Фото"),
    ("video", "Видео"),
    ("archive", "Архивы"),
];
static INDEX_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexedMediaRecord {
    pub id: String,
    pub kind: String,
    pub original_name: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub extension: String,
    pub size_bytes: u64,
    pub created_at: Option<u64>,
    pub modified_at: Option<u64>,
    pub imported_at: Option<u64>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub creator_tag_id: Option<String>,
    pub is_demo: bool,
    pub preview_only: bool,
    pub status: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaIndexStatus {
    pub database_path: String,
    pub library_root: String,
    pub is_scanning: bool,
    pub last_started_at: Option<u64>,
    pub last_completed_at: Option<u64>,
    pub last_full_scan_at: Option<u64>,
    pub last_seen_file_count: usize,
    pub available_count: usize,
    pub missing_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexResult {
    pub scanned_files: usize,
    pub added_files: usize,
    pub updated_files: usize,
    pub missing_files: usize,
    pub completed_at: u64,
}

#[derive(Debug)]
struct ScannedMedia {
    relative_path: String,
    kind: String,
    category: String,
    original_name: String,
    extension: String,
    mime_type: String,
    size_bytes: u64,
    created_at: Option<u64>,
    modified_at: Option<u64>,
}

#[derive(Debug, Clone)]
struct ExistingMedia {
    size_bytes: u64,
    modified_at: Option<u64>,
    status: String,
}

fn now_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn to_db_error(error: impl std::fmt::Display) -> String {
    format!("Ошибка SQLite: {error}")
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir()
        .map_err(|error| format!("Не удалось определить папку данных приложения: {error}"))?;
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Не удалось создать папку данных приложения {}: {error}", dir.display()))?;
    Ok(dir.join("media-library.sqlite3"))
}

fn open_database(app: &AppHandle) -> Result<(Connection, PathBuf), String> {
    let path = database_path(app)?;
    let connection = Connection::open(&path).map_err(to_db_error)?;
    connection.busy_timeout(Duration::from_secs(8)).map_err(to_db_error)?;
    connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;")
        .map_err(to_db_error)?;
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );"
    ).map_err(to_db_error)?;
    let current_version: i64 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |row| row.get(0),
    ).map_err(to_db_error)?;
    if current_version < 3 {
        connection.execute_batch(include_str!("../../../database/migrations/0003_runtime_sqlite_index.sql"))
            .map_err(|error| format!("Не удалось применить миграцию SQLite 0003: {error}"))?;
        connection.execute(
            "INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (3, ?1)",
            params![now_seconds() as i64],
        ).map_err(to_db_error)?;
    }
    connection.pragma_update(None, "user_version", current_version.max(3)).map_err(to_db_error)?;
    connection.execute(
        "INSERT OR IGNORE INTO index_state (id, library_root, is_scanning, last_seen_file_count, last_missing_count) VALUES (1, '', 0, 0, 0)",
        [],
    ).map_err(to_db_error)?;
    Ok((connection, path))
}

pub fn initialize_database(app: &AppHandle) -> Result<String, String> {
    let (connection, path) = open_database(app)?;
    connection.execute_batch("PRAGMA journal_mode = WAL;").map_err(to_db_error)?;
    // A crash can leave the previous run marked as scanning. Clear that flag on app startup.
    connection.execute(
        "UPDATE index_state SET is_scanning = 0,
            last_error = 'Индексация была прервана предыдущим запуском'
         WHERE id = 1 AND is_scanning = 1",
        [],
    ).map_err(to_db_error)?;
    Ok(path.to_string_lossy().to_string())
}

fn canonical_root(root_path: &str) -> Result<PathBuf, String> {
    if root_path.trim().is_empty() {
        return Err("Сначала выбери корневую папку медиатеки в настройках.".into());
    }
    let root = PathBuf::from(root_path.trim());
    if !root.is_dir() {
        return Err(format!("Папка медиатеки недоступна: {}. Проверь диск и путь в настройках.", root.display()));
    }
    fs::canonicalize(&root).map_err(|error| format!("Не удалось определить путь медиатеки: {error}"))
}

fn supported_kind(extension: &str) -> Option<&'static str> {
    match extension {
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tif" | "tiff" | "avif" | "heic" => Some("photo"),
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "m4v" | "mpg" | "mpeg" | "wmv" | "flv" | "3gp" | "ogv" | "gif" => Some("video"),
        "zip" | "7z" | "rar" | "tar" | "gz" | "bz2" | "xz" => Some("archive"),
        _ => None,
    }
}

fn mime_for(extension: &str, kind: &str) -> String {
    match (kind, extension) {
        ("photo", "jpg" | "jpeg") => "image/jpeg".to_string(),
        ("photo", "png") => "image/png".to_string(),
        ("photo", "webp") => "image/webp".to_string(),
        ("photo", "gif") => "image/gif".to_string(),
        ("photo", "avif") => "image/avif".to_string(),
        ("photo", "bmp") => "image/bmp".to_string(),
        ("photo", "tif" | "tiff") => "image/tiff".to_string(),
        ("photo", "heic") => "image/heic".to_string(),
        ("video", "gif") => "image/gif".to_string(),
        ("video", "mp4" | "m4v") => "video/mp4".to_string(),
        ("video", "webm") => "video/webm".to_string(),
        ("video", "mkv") => "video/x-matroska".to_string(),
        ("video", "avi") => "video/x-msvideo".to_string(),
        ("video", "mov") => "video/quicktime".to_string(),
        ("video", "wmv") => "video/x-ms-wmv".to_string(),
        ("video", "ogv") => "video/ogg".to_string(),
        ("archive", "zip") => "application/zip".to_string(),
        ("archive", "7z") => "application/x-7z-compressed".to_string(),
        ("archive", "rar") => "application/vnd.rar".to_string(),
        ("archive", "tar") => "application/x-tar".to_string(),
        ("archive", "gz") => "application/gzip".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

fn epoch_seconds(value: std::io::Result<SystemTime>) -> Option<u64> {
    value.ok()?.duration_since(UNIX_EPOCH).ok().map(|duration| duration.as_secs())
}

fn scan_folder(
    root: &Path,
    folder: &Path,
    category: &str,
    kind: &str,
    collected: &mut Vec<ScannedMedia>,
) -> Result<(), String> {
    let entries = fs::read_dir(folder)
        .map_err(|error| format!("Не удалось прочитать папку {}: {error}", folder.display()))?;

    for entry_result in entries {
        let entry = entry_result.map_err(|error| format!("Не удалось прочитать запись в {}: {error}", folder.display()))?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| format!("Не удалось определить тип {}: {error}", path.display()))?;
        // Do not follow symbolic links. A linked directory could lead outside the library or loop back into it.
        if file_type.is_dir() {
            scan_folder(root, &path, category, kind, collected)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
        if supported_kind(&extension) != Some(kind) || path.file_name().and_then(|name| name.to_str()).unwrap_or("").ends_with(".partial") {
            continue;
        }
        let metadata = fs::metadata(&path).map_err(|error| format!("Не удалось прочитать метаданные {}: {error}", path.display()))?;
        let relative_path = path.strip_prefix(root)
            .map_err(|error| format!("Путь {} не находится внутри медиатеки: {error}", path.display()))?
            .to_string_lossy().replace('\\', "/");
        let original_name = path.file_name().and_then(|value| value.to_str()).unwrap_or("unnamed").to_string();
        collected.push(ScannedMedia {
            relative_path,
            kind: kind.to_string(),
            category: category.to_string(),
            original_name,
            extension: format!(".{extension}"),
            mime_type: mime_for(&extension, kind),
            size_bytes: metadata.len(),
            created_at: epoch_seconds(metadata.created()),
            modified_at: epoch_seconds(metadata.modified()),
        });
    }
    Ok(())
}

fn scan_library(root: &Path) -> Result<Vec<ScannedMedia>, String> {
    let mut collected = Vec::new();
    let mut known_folders_found = 0_usize;
    for (category, category_folder) in CATEGORIES {
        for (kind, kind_folder) in KINDS {
            let folder = root.join(category_folder).join(kind_folder);
            if folder.is_dir() {
                known_folders_found += 1;
                scan_folder(root, &folder, category, kind, &mut collected)?;
            } else if folder.exists() {
                return Err(format!("Ожидаемая папка медиатеки не является каталогом: {}", folder.display()));
            }
        }
    }
    if known_folders_found == 0 {
        return Err(format!(
            "В папке {} не найдена структура медиатеки. Чтобы не отметить все записи как отсутствующие, индексация остановлена. Проверь корневой путь или создай структуру в настройках.",
            root.display()
        ));
    }
    collected.sort_by(|a, b| a.relative_path.to_lowercase().cmp(&b.relative_path.to_lowercase()));
    Ok(collected)
}

fn read_status_from_connection(connection: &Connection, db_path: &Path) -> Result<MediaIndexStatus, String> {
    let row = connection.query_row(
        "SELECT library_root, is_scanning, last_started_at, last_completed_at, last_full_scan_at,
                last_seen_file_count, last_missing_count, last_error
         FROM index_state WHERE id = 1",
        [],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)? != 0,
            row.get::<_, Option<i64>>(2)?,
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, i64>(5)?.max(0) as usize,
            row.get::<_, i64>(6)?.max(0) as usize,
            row.get::<_, Option<String>>(7)?,
        )),
    ).optional().map_err(to_db_error)?;

    let (library_root, is_scanning, started, completed, full_scan, last_seen, missing, last_error) =
        row.unwrap_or_else(|| (String::new(), false, None, None, None, 0, 0, None));
    let available_count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM media_files WHERE status <> 'missing'", [], |row| row.get(0),
    ).map_err(to_db_error)?;
    Ok(MediaIndexStatus {
        database_path: db_path.to_string_lossy().to_string(),
        library_root,
        is_scanning,
        last_started_at: started.and_then(|value| u64::try_from(value).ok()),
        last_completed_at: completed.and_then(|value| u64::try_from(value).ok()),
        last_full_scan_at: full_scan.and_then(|value| u64::try_from(value).ok()),
        last_seen_file_count: last_seen,
        available_count: available_count.max(0) as usize,
        missing_count: missing,
        last_error,
    })
}

pub fn get_status_internal(app: &AppHandle) -> Result<MediaIndexStatus, String> {
    let (connection, path) = open_database(app)?;
    read_status_from_connection(&connection, &path)
}

pub fn ensure_indexed(app: &AppHandle, root_path: &str) -> Result<(), String> {
    let root = canonical_root(root_path)?;
    let current_root = root.to_string_lossy().to_string();
    let mut status = get_status_internal(app)?;

    // If another page is doing a full index, wait for that operation rather than
    // failing navigation simply because the shared index is temporarily busy.
    if status.is_scanning {
        let guard = INDEX_MUTEX.lock().map_err(|_| "Не удалось дождаться завершения индексации.".to_string())?;
        // No active indexer can own this mutex now. If the DB still says scanning,
        // recover a stale flag left by an interrupted transaction/process.
        status = get_status_internal(app)?;
        if status.is_scanning {
            let (connection, _) = open_database(app)?;
            connection.execute(
                "UPDATE index_state SET is_scanning = 0, last_error = 'Индексация была прервана предыдущим запуском' WHERE id = 1",
                [],
            ).map_err(to_db_error)?;
            status = get_status_internal(app)?;
        }
        drop(guard);
    }
    if status.is_scanning {
        return Err("Индекс медиатеки всё ещё обновляется. Повтори запрос через несколько секунд.".into());
    }
    let root_changed = !same_path(&status.library_root, &current_root);
    let interrupted_previous_run = status.last_error.as_deref() == Some("Индексация была прервана предыдущим запуском");
    if status.last_error.is_some() && !root_changed && !interrupted_previous_run {
        return Err(format!(
            "Последняя индексация завершилась ошибкой: {}. Повтори индексацию кнопкой «Переиндексировать» на странице «База файлов».",
            status.last_error.as_deref().unwrap_or("неизвестная ошибка")
        ));
    }
    if status.last_completed_at.is_none() || root_changed || interrupted_previous_run {
        index_library_internal(app.clone(), current_root)?;
    }
    Ok(())
}

fn same_path(left: &str, right: &str) -> bool {
    let normalize = |value: &str| value.trim_end_matches(|c: char| c == '\\' || c == '/').replace('/', "\\").to_lowercase();
    normalize(left) == normalize(right)
}

pub fn index_library_internal(app: AppHandle, root_path: String) -> Result<IndexResult, String> {
    let _lock = INDEX_MUTEX.lock().map_err(|_| "Не удалось получить блокировку индексатора.".to_string())?;
    let root = canonical_root(&root_path)?;
    let root_string = root.to_string_lossy().to_string();
    let started_at = now_seconds();
    {
        let (connection, _) = open_database(&app)?;
        connection.execute(
            "UPDATE index_state SET library_root = ?1, is_scanning = 1, last_started_at = ?2, last_error = NULL WHERE id = 1",
            params![root_string, started_at as i64],
        ).map_err(to_db_error)?;
    }

    // All work after setting is_scanning runs behind this result handler. If any
    // scan, SQL statement, or commit fails, restore a visible non-busy state.
    let result = apply_scan_transaction(&app, &root, &root_string);
    if let Err(error) = &result {
        if let Ok((connection, _)) = open_database(&app) {
            let _ = connection.execute(
                "UPDATE index_state SET is_scanning = 0, last_error = ?1 WHERE id = 1",
                params![error],
            );
        }
    }
    result
}

fn apply_scan_transaction(app: &AppHandle, root: &Path, root_string: &str) -> Result<IndexResult, String> {
    let scanned = scan_library(root)?;
    let (mut connection, _) = open_database(app)?;
    let transaction = connection.transaction().map_err(to_db_error)?;
    let mut previous: HashMap<String, ExistingMedia> = HashMap::new();
    {
        let mut statement = transaction.prepare("SELECT relative_path, size_bytes, modified_at_source, status FROM media_files")
            .map_err(to_db_error)?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?.max(0) as u64,
                row.get::<_, Option<i64>>(2)?.and_then(|value| u64::try_from(value).ok()),
                row.get::<_, String>(3)?,
            ))
        }).map_err(to_db_error)?;
        for row in rows {
            let (path, size_bytes, modified_at, status) = row.map_err(to_db_error)?;
            previous.insert(path, ExistingMedia { size_bytes, modified_at, status });
        }
    }

    let mut added_files = 0_usize;
    let mut updated_files = 0_usize;
    let mut seen_paths = std::collections::HashSet::with_capacity(scanned.len());

    for item in &scanned {
        seen_paths.insert(item.relative_path.clone());
        match previous.get(&item.relative_path) {
            None => added_files += 1,
            Some(old) if old.status == "missing" || old.size_bytes != item.size_bytes || old.modified_at != item.modified_at => updated_files += 1,
            Some(_) => {},
        }
        let imported_at = now_seconds();
        transaction.execute(
            "INSERT INTO media_files (
                id, kind, original_name, relative_path, source_path, extension, mime_type,
                size_bytes, created_at_source, modified_at_source, imported_at,
                content_category, sha256, metadata_json, status
             ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11, NULL, '{}', 'available')
             ON CONFLICT(relative_path) DO UPDATE SET
                kind = excluded.kind,
                original_name = excluded.original_name,
                extension = excluded.extension,
                mime_type = excluded.mime_type,
                size_bytes = excluded.size_bytes,
                created_at_source = excluded.created_at_source,
                modified_at_source = excluded.modified_at_source,
                content_category = excluded.content_category,
                status = CASE WHEN media_files.status = 'needs_review' THEN 'needs_review' ELSE 'available' END",
            params![
                item.relative_path, item.kind, item.original_name, item.relative_path,
                item.extension, item.mime_type, item.size_bytes as i64,
                item.created_at.map(|value| value as i64), item.modified_at.map(|value| value as i64),
                imported_at as i64, item.category,
            ],
        ).map_err(to_db_error)?;
    }

    let missing_paths: Vec<String> = previous.iter()
        .filter(|(path, old)| old.status != "missing" && !seen_paths.contains(*path))
        .map(|(path, _)| path.clone())
        .collect();
    for path in &missing_paths {
        transaction.execute("UPDATE media_files SET status = 'missing' WHERE relative_path = ?1", params![path])
            .map_err(to_db_error)?;
    }
    let missing_files = missing_paths.len();
    let completed_at = now_seconds();
    let missing_count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM media_files WHERE status = 'missing'", [], |row| row.get(0),
    ).map_err(to_db_error)?;
    transaction.execute(
        "UPDATE index_state SET library_root = ?1, is_scanning = 0, last_completed_at = ?2,
            last_full_scan_at = ?2, last_seen_file_count = ?3, last_missing_count = ?4, last_error = NULL WHERE id = 1",
        params![root_string, completed_at as i64, scanned.len() as i64, missing_count],
    ).map_err(to_db_error)?;
    transaction.execute(
        "INSERT INTO app_settings(key, value) VALUES('database.schema_version', '1')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value", [],
    ).map_err(to_db_error)?;
    transaction.commit().map_err(to_db_error)?;

    Ok(IndexResult { scanned_files: scanned.len(), added_files, updated_files, missing_files, completed_at })
}

fn query_records(app: &AppHandle, root_path: &str, kind: Option<&str>, include_missing: bool) -> Result<Vec<IndexedMediaRecord>, String> {
    let _canonical_root = canonical_root(root_path)?;
    // Keep the configured path style for file-protocol URLs; only the index uses canonical root paths internally.
    let root = PathBuf::from(root_path.trim());
    let (connection, _) = open_database(app)?;
    let mut statement = connection.prepare(
        "SELECT mf.id, mf.kind, mf.original_name, mf.relative_path, mf.extension, mf.size_bytes,
                mf.created_at_source, mf.modified_at_source, mf.imported_at, mf.content_category,
                mf.status, COALESCE(GROUP_CONCAT(CASE WHEN t.tag_type = 'general' THEN t.name END, char(31)), ''),
                MAX(CASE WHEN t.tag_type = 'creator' THEN CAST(t.id AS TEXT) ELSE NULL END)
         FROM media_files mf
         LEFT JOIN media_file_tags mft ON mft.media_file_id = mf.id
         LEFT JOIN tags t ON t.id = mft.tag_id
         WHERE (?1 IS NULL OR mf.kind = ?1) AND (?2 = 1 OR mf.status <> 'missing')
         GROUP BY mf.id
         ORDER BY COALESCE(mf.modified_at_source, 0) DESC, lower(mf.original_name) ASC",
    ).map_err(to_db_error)?;
    let rows = statement.query_map(params![kind, if include_missing { 1 } else { 0 }], |row| {
        let relative_path: String = row.get(3)?;
        let tags_joined: String = row.get(11)?;
        let creator_tag_id: Option<String> = row.get(12)?;
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            relative_path,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?.max(0) as u64,
            row.get::<_, Option<i64>>(6)?,
            row.get::<_, Option<i64>>(7)?,
            row.get::<_, Option<i64>>(8)?,
            row.get::<_, Option<String>>(9)?,
            row.get::<_, String>(10)?,
            tags_joined,
            creator_tag_id,
        ))
    }).map_err(to_db_error)?;

    let mut result = Vec::new();
    for row in rows {
        let (id, kind, original_name, relative_path, extension, size_bytes, created, modified, imported, category, status, tags_joined, creator_tag_id) = row.map_err(to_db_error)?;
        let mut absolute = root.clone();
        for part in relative_path.split('/') {
            if !part.is_empty() && part != "." && part != ".." { absolute.push(part); }
        }
        result.push(IndexedMediaRecord {
            id, kind, original_name, relative_path, absolute_path: absolute.to_string_lossy().to_string(),
            extension, size_bytes,
            created_at: created.and_then(|value| u64::try_from(value).ok()),
            modified_at: modified.and_then(|value| u64::try_from(value).ok()),
            imported_at: imported.and_then(|value| u64::try_from(value).ok()),
            category, tags: if tags_joined.is_empty() { Vec::new() } else { tags_joined.split('\u{1f}').map(str::to_string).collect() },
            creator_tag_id, is_demo: false, preview_only: false, status,
        });
    }
    Ok(result)
}

pub fn query_media_internal(app: &AppHandle, root_path: &str, kind: Option<&str>, include_missing: bool) -> Result<Vec<IndexedMediaRecord>, String> {
    query_records(app, root_path, kind, include_missing)
}

#[tauri::command]
pub async fn refresh_media_index(app: AppHandle, root_path: String) -> Result<IndexResult, String> {
    tauri::async_runtime::spawn_blocking(move || index_library_internal(app, root_path))
        .await
        .map_err(|error| format!("Задача индексации завершилась аварийно: {error}"))?
}

#[tauri::command]
pub fn get_media_index_status(app: AppHandle) -> Result<MediaIndexStatus, String> {
    get_status_internal(&app)
}

#[tauri::command]
pub async fn list_indexed_media(
    app: AppHandle,
    root_path: String,
    kind: Option<String>,
    include_missing: Option<bool>,
) -> Result<Vec<IndexedMediaRecord>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ensure_indexed(&app, &root_path)?;
        query_records(&app, &root_path, kind.as_deref(), include_missing.unwrap_or(false))
    }).await.map_err(|error| format!("Не удалось прочитать SQLite-индекс: {error}"))?
}

#[tauri::command]
pub fn set_media_tags(app: AppHandle, root_path: String, relative_path: String, tags: Vec<String>) -> Result<(), String> {
    ensure_indexed(&app, &root_path)?;
    let normalized = relative_path.trim().replace('\\', "/").trim_start_matches('/').to_string();
    if normalized.split('/').any(|part| part == "..") {
        return Err("Относительный путь содержит недопустимый сегмент.".into());
    }
    let (mut connection, _) = open_database(&app)?;
    let transaction = connection.transaction().map_err(to_db_error)?;
    let file_id: Option<String> = transaction.query_row(
        "SELECT id FROM media_files WHERE relative_path = ?1", params![normalized], |row| row.get(0),
    ).optional().map_err(to_db_error)?;
    let file_id = file_id.ok_or_else(|| "Файл не найден в индексе. Обнови индекс и повтори попытку.".to_string())?;
    transaction.execute(
        "DELETE FROM media_file_tags WHERE media_file_id = ?1 AND tag_id IN (SELECT id FROM tags WHERE tag_type = 'general')",
        params![file_id],
    ).map_err(to_db_error)?;
    for raw_tag in tags {
        let name = raw_tag.trim();
        if name.is_empty() { continue; }
        if name.chars().count() > 80 { return Err(format!("Тег слишком длинный (максимум 80 символов): {}…", name.chars().take(20).collect::<String>())); }
        transaction.execute("INSERT OR IGNORE INTO tags(name, tag_type) VALUES (?1, 'general')", params![name])
            .map_err(to_db_error)?;
        let tag_id: i64 = transaction.query_row(
            "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE AND tag_type = 'general'", params![name], |row| row.get(0),
        ).map_err(to_db_error)?;
        transaction.execute("INSERT OR IGNORE INTO media_file_tags(media_file_id, tag_id) VALUES (?1, ?2)", params![file_id, tag_id])
            .map_err(to_db_error)?;
    }
    transaction.commit().map_err(to_db_error)
}
