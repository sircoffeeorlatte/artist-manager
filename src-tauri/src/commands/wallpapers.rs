use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "avif", "gif"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "webm", "m4v", "mpg", "mpeg", "wmv", "flv", "3gp", "ogv"];

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperItem {
    name: String,
    relative_path: String,
    absolute_path: String,
    extension: String,
    kind: String,
    size_bytes: u64,
    modified_at: Option<u64>,
}

fn normalized_root(root_path: &str) -> Result<PathBuf, String> {
    if root_path.trim().is_empty() {
        return Err("Путь к медиатеке не указан.".into());
    }
    let root = PathBuf::from(root_path.trim());
    if !root.is_absolute() || !root.is_dir() {
        return Err(format!("Папка медиатеки недоступна: {}", root.display()));
    }
    Ok(root)
}

fn extension(path: &Path) -> String {
    path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase()
}

fn kind_for_extension(extension: &str) -> Option<&'static str> {
    if IMAGE_EXTENSIONS.contains(&extension) { Some("image") }
    else if VIDEO_EXTENSIONS.contains(&extension) { Some("video") }
    else { None }
}

fn unique_destination(directory: &Path, file_name: &str) -> PathBuf {
    let initial = directory.join(file_name);
    if !initial.exists() { return initial; }
    let source = Path::new(file_name);
    let stem = source.file_stem().and_then(|value| value.to_str()).unwrap_or("wallpaper");
    let ext = source.extension().and_then(|value| value.to_str());
    for number in 1..100_000_u32 {
        let name = match ext {
            Some(ext) if !ext.is_empty() => format!("{stem} ({number}).{ext}"),
            _ => format!("{stem} ({number})"),
        };
        let candidate = directory.join(name);
        if !candidate.exists() { return candidate; }
    }
    directory.join(format!("wallpaper-{}-{file_name}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()))
}

fn to_item(path: &Path, root: &Path) -> Result<WallpaperItem, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("Не удалось прочитать обои {}: {error}", path.display()))?;
    if !metadata.is_file() { return Err("Выбранный объект не является файлом.".into()); }
    let ext = extension(path);
    let kind = kind_for_extension(&ext).ok_or_else(|| format!("Формат .{ext} не поддерживается для обоев."))?;
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("wallpaper").to_string();
    let relative_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
    let modified_at = metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok()).map(|duration| duration.as_secs());
    Ok(WallpaperItem {
        name,
        relative_path,
        absolute_path: path.to_string_lossy().to_string(),
        extension: format!(".{ext}"),
        kind: kind.to_string(),
        size_bytes: metadata.len(),
        modified_at,
    })
}

fn wallpaper_directory(root: &Path) -> Result<PathBuf, String> {
    let directory = root.join("wallpapers");
    fs::create_dir_all(&directory).map_err(|error| format!("Не удалось создать папку обоев {}: {error}", directory.display()))?;
    Ok(directory)
}

#[tauri::command]
pub fn list_wallpapers(app: AppHandle, root_path: String) -> Result<Vec<WallpaperItem>, String> {
    let root = normalized_root(&root_path)?;
    super::filesystem::register_library_root(app, root.to_string_lossy().to_string())?;
    let directory = wallpaper_directory(&root)?;
    let entries = fs::read_dir(&directory).map_err(|error| format!("Не удалось прочитать папку обоев: {error}"))?;
    let mut wallpapers = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("Не удалось прочитать запись папки обоев: {error}"))?;
        let path = entry.path();
        if !path.is_file() || kind_for_extension(&extension(&path)).is_none() { continue; }
        match to_item(&path, &root) {
            Ok(item) => wallpapers.push(item),
            Err(_) => continue,
        }
    }
    wallpapers.sort_by(|left, right| right.modified_at.cmp(&left.modified_at).then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase())));
    Ok(wallpapers)
}

#[tauri::command]
pub fn import_wallpaper(app: AppHandle, source_path: String, root_path: String) -> Result<WallpaperItem, String> {
    let root = normalized_root(&root_path)?;
    super::filesystem::register_library_root(app, root.to_string_lossy().to_string())?;
    let source = PathBuf::from(source_path.trim());
    if !source.is_absolute() || !source.is_file() {
        return Err(format!("Исходный файл не найден: {}", source.display()));
    }
    let ext = extension(&source);
    if kind_for_extension(&ext).is_none() {
        return Err(format!("Формат .{ext} не поддерживается. Выбери обычное изображение или видео."));
    }
    let directory = wallpaper_directory(&root)?;
    let source_canonical = source.canonicalize().map_err(|error| format!("Не удалось открыть исходный файл: {error}"))?;
    let directory_canonical = directory.canonicalize().map_err(|error| format!("Не удалось открыть папку обоев: {error}"))?;
    if source_canonical.starts_with(&directory_canonical) {
        return to_item(&source, &root);
    }

    let source_name = source.file_name().and_then(|value| value.to_str()).ok_or_else(|| "Не удалось определить имя файла обоев.".to_string())?;
    let destination = unique_destination(&directory, source_name);
    let destination_name = destination.file_name().and_then(|value| value.to_str()).unwrap_or("wallpaper");
    let temporary = unique_destination(&directory, &format!(".{destination_name}.partial"));
    if let Err(error) = fs::copy(&source_canonical, &temporary) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("Не удалось скопировать обои. Проверь доступность исходного и целевого диска: {error}"));
    }
    let source_size = match fs::metadata(&source_canonical) {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            return Err(format!("Исходный файл исчез во время копирования: {error}"));
        }
    };
    let copied_size = match fs::metadata(&temporary) {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            return Err(format!("Не удалось проверить временную копию: {error}"));
        }
    };
    if source_size != copied_size {
        let _ = fs::remove_file(&temporary);
        return Err("Проверка копии не пройдена: размер отличается от исходного файла.".into());
    }
    if let Err(error) = fs::rename(&temporary, &destination) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("Не удалось завершить сохранение обоев: {error}"));
    }
    to_item(&destination, &root)
}
