use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};

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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LibraryMediaItem {
    id: String,
    kind: String,
    original_name: String,
    relative_path: String,
    absolute_path: String,
    extension: String,
    size_bytes: u64,
    created_at: Option<u64>,
    modified_at: Option<u64>,
    imported_at: Option<u64>,
    category: Option<String>,
    tags: Vec<String>,
    creator_tag_id: Option<String>,
    is_demo: bool,
    preview_only: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    imported: Vec<LibraryMediaItem>,
    warnings: Vec<String>,
}

fn time_seconds(value: std::io::Result<SystemTime>) -> Option<u64> {
    value.ok()?.duration_since(UNIX_EPOCH).ok().map(|duration| duration.as_secs())
}

fn category_folder(category: &str) -> Result<&'static str, String> {
    CATEGORIES.iter().find(|(key, _)| *key == category).map(|(_, folder)| *folder)
        .ok_or_else(|| format!("Неизвестная категория: {category}"))
}

fn kind_folder(kind: &str) -> Result<&'static str, String> {
    KINDS.iter().find(|(key, _)| *key == kind).map(|(_, folder)| *folder)
        .ok_or_else(|| format!("Неизвестный тип файла: {kind}"))
}

fn ensure_structure(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|error| format!("Не удалось создать папку медиатеки: {error}"))?;
    fs::create_dir_all(root.join("wallpapers")).map_err(|error| format!("Не удалось создать каталог обоев {}: {error}", root.join("wallpapers").display()))?;
    for (_, category_name) in CATEGORIES {
        for (_, kind_name) in KINDS {
            fs::create_dir_all(root.join(category_name).join(kind_name))
                .map_err(|error| format!("Не удалось создать каталог {}: {error}", root.join(category_name).join(kind_name).display()))?;
        }
    }
    Ok(())
}

fn extension_for(path: &Path) -> String {
    path.extension().and_then(|extension| extension.to_str()).unwrap_or("").to_lowercase()
}

fn supports_kind(extension: &str, kind: &str) -> bool {
    match kind {
        "photo" => ["jpg", "jpeg", "png", "webp", "bmp", "tif", "tiff", "avif", "heic"].contains(&extension),
        // GIFs belong to the animated/video gallery in this project.
        "video" => ["mp4", "mkv", "avi", "mov", "webm", "m4v", "mpg", "mpeg", "wmv", "flv", "3gp", "ogv", "gif"].contains(&extension),
        "archive" => ["zip", "7z", "rar", "tar", "gz", "bz2", "xz"].contains(&extension),
        _ => false,
    }
}

fn category_key(folder: &str) -> Option<&'static str> {
    CATEGORIES.iter().find(|(_, label)| *label == folder).map(|(key, _)| *key)
}

fn to_media_item(path: &Path, root: &Path, forced_kind: Option<&str>) -> Result<LibraryMediaItem, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("Не удалось прочитать {}: {error}", path.display()))?;
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("unnamed").to_string();
    let extension = path.extension().and_then(|value| value.to_str()).map(|value| format!(".{}", value.to_lowercase())).unwrap_or_default();
    let relative_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
    let components: Vec<&str> = relative_path.split('/').collect();
    let category = components.first().and_then(|folder| category_key(folder)).map(str::to_string);
    let kind = forced_kind.map(str::to_string).or_else(|| {
        components.get(1).and_then(|folder| KINDS.iter().find(|(_, label)| label == folder).map(|(key, _)| key.to_string()))
    }).unwrap_or_else(|| "video".to_string());
    let imported_at = SystemTime::now().duration_since(UNIX_EPOCH).ok().map(|duration| duration.as_secs());
    Ok(LibraryMediaItem {
        id: relative_path.clone(),
        kind,
        original_name: name,
        relative_path,
        absolute_path: path.to_string_lossy().to_string(),
        extension,
        size_bytes: metadata.len(),
        created_at: time_seconds(metadata.created()),
        modified_at: time_seconds(metadata.modified()),
        imported_at,
        category,
        tags: Vec::new(),
        creator_tag_id: None,
        is_demo: false,
        preview_only: false,
    })
}

fn unique_destination(directory: &Path, file_name: &str) -> PathBuf {
    let initial = directory.join(file_name);
    if !initial.exists() { return initial; }
    let source = Path::new(file_name);
    let stem = source.file_stem().and_then(|value| value.to_str()).unwrap_or("file");
    let extension = source.extension().and_then(|value| value.to_str());
    for number in 1..100_000_u32 {
        let candidate_name = match extension {
            Some(extension) if !extension.is_empty() => format!("{stem} ({number}).{extension}"),
            _ => format!("{stem} ({number})"),
        };
        let candidate = directory.join(candidate_name);
        if !candidate.exists() { return candidate; }
    }
    directory.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(), file_name))
}

#[tauri::command]
pub fn create_library_structure(root_path: String) -> Result<String, String> {
    if root_path.trim().is_empty() { return Err("Путь к медиатеке не указан.".into()); }
    let root = PathBuf::from(root_path.trim());
    if !root.is_absolute() { return Err("Нужен абсолютный путь, например D:\\MediaLibrary.".into()); }
    ensure_structure(&root)?;
    // Keep the normal user-facing path rather than Windows' optional \\?\ device prefix.
    Ok(root.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_saved_library_root(app: tauri::AppHandle) -> Result<String, String> {
    if let Some(config) = super::config::read_user_config_file(&app)? {
        if !config.library_root.trim().is_empty() {
            return Ok(config.library_root);
        }
    }

    // Legacy fallback for builds which stored only library_root.txt.
    let settings_dir = app.path().app_local_data_dir()
        .map_err(|error| format!("Не удалось определить папку настроек приложения: {error}"))?;
    let settings_file = settings_dir.join("library_root.txt");
    match fs::read_to_string(settings_file) {
        Ok(path) => Ok(path.trim().to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!("Не удалось прочитать настройки медиатеки: {error}")),
    }
}

#[tauri::command]
pub fn save_library_root(app: tauri::AppHandle, root_path: String) -> Result<(), String> {
    let root = PathBuf::from(root_path.trim());
    if !root.is_dir() {
        return Err("Нельзя сохранить путь: папка медиатеки не существует.".into());
    }

    super::config::update_library_root_config(&app, root.to_string_lossy().to_string())?;

    // Keep the legacy file updated for compatibility with an older build rolled back by the user.
    let settings_dir = app.path().app_local_data_dir()
        .map_err(|error| format!("Не удалось определить папку настроек приложения: {error}"))?;
    fs::create_dir_all(&settings_dir)
        .map_err(|error| format!("Не удалось создать папку настроек приложения: {error}"))?;
    fs::write(settings_dir.join("library_root.txt"), root.to_string_lossy().as_bytes())
        .map_err(|error| format!("Не удалось обновить резервный путь медиатеки: {error}"))
}

#[tauri::command]
pub fn register_library_root(app: tauri::AppHandle, root_path: String) -> Result<(), String> {
    let root = PathBuf::from(root_path.trim());
    if !root.is_dir() { return Err("Папка медиатеки не найдена. Сначала создай структуру в настройках.".into()); }
    app.asset_protocol_scope().allow_directory(&root, true)
        .map_err(|error| format!("Не удалось разрешить просмотр файлов медиатеки: {error}"))?;
    Ok(())
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgress {
    operation_id: String,
    overall_percent: u8,
    file_index: usize,
    total_files: usize,
    current_file: String,
    bytes_copied: u64,
    file_size: u64,
    completed_files: usize,
    file_succeeded: Option<bool>,
    status: String,
}

fn progress_percent(completed_bytes: u64, total_bytes: u64, completed_files: usize, total_files: usize) -> u8 {
    if total_files == 0 {
        return 100;
    }
    if total_bytes > 0 {
        return ((completed_bytes as f64 / total_bytes as f64) * 100.0).floor().clamp(0.0, 100.0) as u8;
    }
    ((completed_files as f64 / total_files as f64) * 100.0).floor().clamp(0.0, 100.0) as u8
}

fn emit_import_progress(app: &tauri::AppHandle, progress: &ImportProgress) {
    let _ = app.emit("import-progress", progress);
}

fn copy_file_with_progress(
    app: &tauri::AppHandle,
    operation_id: &str,
    source: &Path,
    temporary_destination: &Path,
    file_size: u64,
    completed_bytes_before: u64,
    total_bytes: u64,
    file_index: usize,
    total_files: usize,
    current_file: &str,
    completed_files: usize,
) -> std::io::Result<u64> {
    let mut input = File::open(source)?;
    // A unique hidden staging file keeps half-written data out of the gallery.
    let mut output = OpenOptions::new().write(true).create_new(true).open(temporary_destination)?;
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut bytes_copied = 0_u64;
    let mut last_emit = Instant::now();
    let mut last_percent = u8::MAX;

    emit_import_progress(app, &ImportProgress {
        operation_id: operation_id.to_string(),
        overall_percent: progress_percent(completed_bytes_before, total_bytes, completed_files, total_files),
        file_index,
        total_files,
        current_file: current_file.to_string(),
        bytes_copied: 0,
        file_size,
        completed_files,
        file_succeeded: None,
        status: "copying".to_string(),
    });

    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 { break; }
        output.write_all(&buffer[..read])?;
        bytes_copied = bytes_copied.saturating_add(read as u64);

        let overall_percent = progress_percent(
            completed_bytes_before.saturating_add(bytes_copied),
            total_bytes,
            completed_files,
            total_files,
        ).min(99);
        if overall_percent != last_percent || last_emit.elapsed() >= Duration::from_millis(120) {
            emit_import_progress(app, &ImportProgress {
                operation_id: operation_id.to_string(),
                overall_percent,
                file_index,
                total_files,
                current_file: current_file.to_string(),
                bytes_copied,
                file_size,
                completed_files,
                file_succeeded: None,
                status: "copying".to_string(),
            });
            last_percent = overall_percent;
            last_emit = Instant::now();
        }
    }
    output.flush()?;
    // Flush file contents to the OS before verifying and atomically promoting the staged file.
    output.sync_all()?;
    Ok(bytes_copied)
}

fn safe_operation_id(value: &str) -> String {
    let sanitized: String = value.chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '-' || *character == '_')
        .take(48)
        .collect();
    if sanitized.is_empty() { "unknown".to_string() } else { sanitized }
}

#[tauri::command]
pub fn import_media_files(
    app: tauri::AppHandle,
    source_paths: Vec<String>,
    root_path: String,
    category: String,
    kind: String,
    mode: String,
    operation_id: String,
) -> Result<ImportResult, String> {
    if !["photo", "video", "archive"].contains(&kind.as_str()) {
        return Err("Неизвестный тип импорта.".into());
    }
    if !["copy", "move"].contains(&mode.as_str()) { return Err("Неизвестный режим импорта.".into()); }
    let category_name = category_folder(&category)?;
    let kind_name = kind_folder(&kind)?;
    let root = PathBuf::from(root_path.trim());
    if !root.is_dir() { return Err("Папка медиатеки не найдена. Открой настройки и создай её.".into()); }
    ensure_structure(&root)?;
    let destination_dir = root.join(category_name).join(kind_name);
    let operation_id = safe_operation_id(&operation_id);
    let mut imported = Vec::new();
    let mut warnings = Vec::new();

    // Snapshot source paths and sizes. A source may still disappear afterwards; that is handled per-file.
    let mut candidates: Vec<(PathBuf, u64)> = Vec::new();
    for source_string in source_paths {
        let source = PathBuf::from(&source_string);
        if !source.is_file() {
            warnings.push(format!("Пропущено: исходный файл не найден или недоступен — {}", source.display()));
            continue;
        }
        let extension = extension_for(&source);
        if !supports_kind(&extension, &kind) {
            warnings.push(format!("Пропущен неподдерживаемый для этого типа формат: {}", source.file_name().and_then(|v| v.to_str()).unwrap_or("unknown")));
            continue;
        }
        match fs::metadata(&source) {
            Ok(metadata) => candidates.push((source, metadata.len())),
            Err(error) => warnings.push(format!("Не удалось прочитать размер файла {}: {error}", source.display())),
        }
    }

    let total_files = candidates.len();
    let total_bytes = candidates.iter().fold(0_u64, |total, (_, size)| total.saturating_add(*size));
    emit_import_progress(&app, &ImportProgress {
        operation_id: operation_id.clone(),
        overall_percent: if total_files == 0 { 100 } else { 0 },
        file_index: 0,
        total_files,
        current_file: String::new(),
        bytes_copied: 0,
        file_size: total_bytes,
        completed_files: 0,
        file_succeeded: None,
        status: "started".to_string(),
    });

    let mut completed_bytes = 0_u64;
    let mut completed_files = 0_usize;
    for (index, (source, expected_size)) in candidates.iter().enumerate() {
        let file_name = source.file_name().and_then(|v| v.to_str()).unwrap_or("unnamed-file").to_string();
        let source_canonical = fs::canonicalize(source).ok();
        let same_parent = source.parent().and_then(|parent| fs::canonicalize(parent).ok()) == fs::canonicalize(&destination_dir).ok();
        let mut file_verified = false;

        if same_parent {
            match to_media_item(source, &root, Some(&kind)) {
                Ok(item) => { imported.push(item); file_verified = true; },
                Err(error) => warnings.push(error),
            }
        } else {
            let destination = unique_destination(&destination_dir, &file_name);
            let temporary_destination = destination_dir.join(format!(".media-library-{operation_id}-{index}.partial"));
            match copy_file_with_progress(
                &app,
                &operation_id,
                source,
                &temporary_destination,
                *expected_size,
                completed_bytes,
                total_bytes,
                index + 1,
                total_files,
                &file_name,
                completed_files,
            ) {
                Ok(bytes_copied) => {
                    let source_size_now = fs::metadata(source).ok().map(|metadata| metadata.len());
                    let temporary_size = fs::metadata(&temporary_destination).ok().map(|metadata| metadata.len());
                    let size_matches = bytes_copied == *expected_size
                        && temporary_size == Some(*expected_size)
                        && source_size_now.map(|size| size == *expected_size).unwrap_or(true);

                    if !size_matches {
                        let _ = fs::remove_file(&temporary_destination);
                        warnings.push(format!("Проверка размера не прошла; неполная копия удалена, оригинал не тронут: {file_name}"));
                    } else if let Err(error) = fs::rename(&temporary_destination, &destination) {
                        let _ = fs::remove_file(&temporary_destination);
                        warnings.push(format!("Копия прочитана, но не удалось завершить запись в медиатеку ({file_name}): {error}. Исходник оставлен."));
                    } else {
                        file_verified = true;
                        if source_size_now.is_none() {
                            warnings.push(format!("Исходный файл исчез во время копирования; проверенная копия сохранена в медиатеке: {file_name}"));
                        } else if mode == "move" {
                            let original = source_canonical.as_deref().unwrap_or(source.as_path());
                            match fs::remove_file(original) {
                                Ok(()) => {},
                                Err(error) if error.kind() == std::io::ErrorKind::NotFound => warnings.push(format!("Копия сохранена, но исходник уже исчез до удаления: {file_name}")),
                                Err(error) => warnings.push(format!("Копия сохранена, но исходник не удалось удалить: {file_name}: {error}")),
                            }
                        }
                        match to_media_item(&destination, &root, Some(&kind)) {
                            Ok(item) => imported.push(item),
                            Err(error) => warnings.push(error),
                        }
                    }
                }
                Err(error) => {
                    // If the application is interrupted before this branch runs, startup recovery removes the .partial file.
                    let _ = fs::remove_file(&temporary_destination);
                    warnings.push(format!("Не удалось импортировать {file_name}: {error}. Исходник не удалялся."));
                }
            }
        }

        completed_bytes = completed_bytes.saturating_add(*expected_size);
        completed_files += 1;
        emit_import_progress(&app, &ImportProgress {
            operation_id: operation_id.clone(),
            overall_percent: progress_percent(completed_bytes, total_bytes, completed_files, total_files).min(99),
            file_index: index + 1,
            total_files,
            current_file: file_name,
            bytes_copied: if file_verified { *expected_size } else { 0 },
            file_size: *expected_size,
            completed_files,
            file_succeeded: Some(file_verified),
            status: "file-complete".to_string(),
        });
    }

    emit_import_progress(&app, &ImportProgress {
        operation_id,
        overall_percent: 100,
        file_index: total_files,
        total_files,
        current_file: String::new(),
        bytes_copied: total_bytes,
        file_size: total_bytes,
        completed_files,
        file_succeeded: None,
        status: "finished".to_string(),
    });
    Ok(ImportResult { imported, warnings })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRecoveryResult {
    removed: Vec<String>,
    failures: Vec<String>,
}

/// Removes only app-owned staging files. Finalized media and source files are never touched.
#[tauri::command]
pub fn recover_interrupted_imports(app: tauri::AppHandle) -> Result<ImportRecoveryResult, String> {
    let saved_root = get_saved_library_root(app)?;
    if saved_root.trim().is_empty() { return Ok(ImportRecoveryResult { removed: Vec::new(), failures: Vec::new() }); }
    let root = PathBuf::from(saved_root);
    if !root.is_dir() { return Ok(ImportRecoveryResult { removed: Vec::new(), failures: Vec::new() }); }

    let mut removed = Vec::new();
    let mut failures = Vec::new();
    for (_, category_name) in CATEGORIES {
        for (_, kind_name) in KINDS {
            let directory = root.join(category_name).join(kind_name);
            let entries = match fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    failures.push(format!("{}: {error}", directory.display()));
                    continue;
                }
            };
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => { failures.push(format!("Не удалось прочитать каталог {}: {error}", directory.display())); continue; }
                };
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if !path.is_file() || !name.starts_with(".media-library-") || !name.ends_with(".partial") { continue; }
                let relative = path.strip_prefix(&root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
                match fs::remove_file(&path) {
                    Ok(()) => removed.push(relative),
                    Err(error) => failures.push(format!("{relative}: {error}")),
                }
            }
        }
    }
    Ok(ImportRecoveryResult { removed, failures })
}

#[tauri::command]
pub fn list_library_media(root_path: String, kind: String) -> Result<Vec<LibraryMediaItem>, String> {
    if !["photo", "video", "archive"].contains(&kind.as_str()) { return Err("Неизвестный тип медиа.".into()); }
    let root = PathBuf::from(root_path.trim());
    if !root.is_dir() { return Ok(Vec::new()); }
    let directory_name = kind_folder(&kind)?;
    let mut items = Vec::new();
    for (_, category_name) in CATEGORIES {
        let folder = root.join(category_name).join(directory_name);
        let entries = match fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("Не удалось прочитать {}: {error}", folder.display())),
        };
        for entry in entries {
            let entry = match entry { Ok(entry) => entry, Err(error) => { eprintln!("Не удалось прочитать запись каталога: {error}"); continue; } };
            let path = entry.path();
            if !path.is_file() { continue; }
            let extension = extension_for(&path);
            if !supports_kind(&extension, &kind) { continue; }
            match to_media_item(&path, &root, Some(&kind)) {
                Ok(item) => items.push(item),
                Err(error) => eprintln!("{error}"),
            }
        }
    }
    items.sort_by(|a, b| b.modified_at.cmp(&a.modified_at).then_with(|| a.original_name.to_lowercase().cmp(&b.original_name.to_lowercase())));
    Ok(items)
}
