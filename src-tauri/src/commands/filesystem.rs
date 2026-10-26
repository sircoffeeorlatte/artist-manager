use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

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
    let settings_dir = app.path().app_local_data_dir()
        .map_err(|error| format!("Не удалось определить папку настроек приложения: {error}"))?;
    fs::create_dir_all(&settings_dir)
        .map_err(|error| format!("Не удалось создать папку настроек приложения: {error}"))?;
    fs::write(settings_dir.join("library_root.txt"), root.to_string_lossy().as_bytes())
        .map_err(|error| format!("Не удалось сохранить путь медиатеки: {error}"))
}

#[tauri::command]
pub fn register_library_root(app: tauri::AppHandle, root_path: String) -> Result<(), String> {
    let root = PathBuf::from(root_path.trim());
    if !root.is_dir() { return Err("Папка медиатеки не найдена. Сначала создай структуру в настройках.".into()); }
    app.asset_protocol_scope().allow_directory(&root, true)
        .map_err(|error| format!("Не удалось разрешить просмотр файлов медиатеки: {error}"))?;
    Ok(())
}

#[tauri::command]
pub fn import_media_files(
    source_paths: Vec<String>,
    root_path: String,
    category: String,
    kind: String,
    mode: String,
) -> Result<ImportResult, String> {
    if !["photo", "video"].contains(&kind.as_str()) { return Err("На этом этапе импорт поддерживает фото и видео.".into()); }
    if !["copy", "move"].contains(&mode.as_str()) { return Err("Неизвестный режим импорта.".into()); }
    let category_name = category_folder(&category)?;
    let kind_name = kind_folder(&kind)?;
    let root = PathBuf::from(root_path.trim());
    if !root.is_dir() { return Err("Папка медиатеки не найдена. Открой настройки и создай её.".into()); }
    ensure_structure(&root)?;
    let destination_dir = root.join(category_name).join(kind_name);
    let mut imported = Vec::new();
    let mut warnings = Vec::new();

    for source_string in source_paths {
        let source = PathBuf::from(&source_string);
        if !source.is_file() {
            warnings.push(format!("Пропущено: источник не является файлом — {}", source.display()));
            continue;
        }
        let extension = extension_for(&source);
        if !supports_kind(&extension, &kind) {
            warnings.push(format!("Пропущен неподдерживаемый для этой галереи формат: {}", source.file_name().and_then(|v| v.to_str()).unwrap_or("unknown")));
            continue;
        }
        let file_name = source.file_name().and_then(|v| v.to_str()).unwrap_or("unnamed-file").to_string();
        let source_canonical = fs::canonicalize(&source).ok();
        let same_parent = source.parent().and_then(|parent| fs::canonicalize(parent).ok()) == fs::canonicalize(&destination_dir).ok();
        if same_parent {
            if let Ok(item) = to_media_item(&source, &root, Some(&kind)) { imported.push(item); }
            continue;
        }
        let destination = unique_destination(&destination_dir, &file_name);
        match fs::copy(&source, &destination) {
            Ok(_) => {
                let source_size = fs::metadata(&source).map(|m| m.len()).unwrap_or(u64::MAX);
                let destination_size = fs::metadata(&destination).map(|m| m.len()).unwrap_or(u64::MIN);
                if source_size != destination_size {
                    let _ = fs::remove_file(&destination);
                    warnings.push(format!("Проверка размера не прошла, файл оставлен на месте: {file_name}"));
                    continue;
                }
                if mode == "move" {
                    // Delete the original only after a successful copy and byte-size verification.
                    if let Some(original) = source_canonical.as_ref() {
                        if fs::remove_file(original).is_err() {
                            warnings.push(format!("Копия сохранена, но исходный файл не удалось удалить: {file_name}"));
                        }
                    } else if fs::remove_file(&source).is_err() {
                        warnings.push(format!("Копия сохранена, но исходный файл не удалось удалить: {file_name}"));
                    }
                }
                match to_media_item(&destination, &root, Some(&kind)) {
                    Ok(item) => imported.push(item),
                    Err(error) => warnings.push(error),
                }
            }
            Err(error) => warnings.push(format!("Не удалось импортировать {file_name}: {error}")),
        }
    }
    Ok(ImportResult { imported, warnings })
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
