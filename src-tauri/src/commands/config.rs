use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static CONFIG_IO_LOCK: Mutex<()> = Mutex::new(());
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppearanceConfig {
    pub accent_hue: u16,
    pub scale: u8,
    pub background_color: String,
    pub wallpaper_path: String,
    pub theme: String,
    pub visual_mode: String,
}

impl Default for AppearanceConfig {
    fn default() -> Self { Self { accent_hue: 210, scale: 100, background_color: "#101114".to_string(), wallpaper_path: String::new(), theme: "dark".to_string(), visual_mode: "liquid-glass".to_string() } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct VideoConfig {
    pub volume: f64,
    pub muted: bool,
    pub tiktok_muted: bool,
}

impl Default for VideoConfig {
    fn default() -> Self { Self { volume: 1.0, muted: false, tiktok_muted: true } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GridColumns {
    pub photo: u8,
    pub video: u8,
    pub archive: u8,
}

impl Default for GridColumns {
    fn default() -> Self { Self { photo: 3, video: 3, archive: 3 } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UserConfig {
    pub version: u32,
    pub appearance: AppearanceConfig,
    pub video: VideoConfig,
    pub grid_columns: GridColumns,
    pub video_gallery_mode: String,
    pub startup_page: String,
    pub library_root: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            version: 1,
            appearance: AppearanceConfig::default(),
            video: VideoConfig::default(),
            grid_columns: GridColumns::default(),
            video_gallery_mode: "grid".to_string(),
            startup_page: "/".to_string(),
            library_root: String::new(),
        }
    }
}

pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir()
        .map_err(|error| format!("Не удалось определить папку конфигурации приложения: {error}"))?;
    Ok(directory.join("user-config.json"))
}

fn read_user_config_file_unlocked(app: &AppHandle) -> Result<Option<UserConfig>, String> {
    let path = config_path(app)?;
    match fs::read_to_string(&path) {
        Ok(text) => {
            let parsed: UserConfig = serde_json::from_str(&text)
                .map_err(|error| format!("Файл конфигурации повреждён ({}): {error}", path.display()))?;
            Ok(Some(normalize_config(parsed)))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Не удалось прочитать файл конфигурации ({}): {error}", path.display())),
    }
}

fn write_user_config_file_unlocked(app: &AppHandle, config: &UserConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let directory = path.parent().ok_or_else(|| "Не удалось определить папку конфигурации.".to_string())?;
    fs::create_dir_all(directory)
        .map_err(|error| format!("Не удалось создать папку конфигурации {}: {error}", directory.display()))?;
    let normalized = normalize_config(config.clone());
    let json = serde_json::to_vec_pretty(&normalized)
        .map_err(|error| format!("Не удалось сериализовать конфигурацию: {error}"))?;
    let temporary = directory.join("user-config.json.tmp");
    fs::write(&temporary, json)
        .map_err(|error| format!("Не удалось записать временный конфиг: {error}"))?;
    // Replace old config only after the complete temporary file has been written.
    if path.exists() {
        let backup = directory.join("user-config.json.bak");
        let _ = fs::remove_file(&backup);
        fs::rename(&path, &backup)
            .map_err(|error| format!("Не удалось подготовить замену конфигурации: {error}"))?;
        if let Err(error) = fs::rename(&temporary, &path) {
            let _ = fs::rename(&backup, &path);
            let _ = fs::remove_file(&temporary);
            return Err(format!("Не удалось сохранить новую конфигурацию: {error}"));
        }
        let _ = fs::remove_file(backup);
    } else {
        fs::rename(&temporary, &path)
            .map_err(|error| format!("Не удалось завершить создание конфигурации: {error}"))?;
    }
    Ok(())
}

pub fn read_user_config_file(app: &AppHandle) -> Result<Option<UserConfig>, String> {
    let _guard = CONFIG_IO_LOCK.lock()
        .map_err(|_| "Заблокирован доступ к файлу конфигурации.".to_string())?;
    read_user_config_file_unlocked(app)
}

pub fn write_user_config_file(app: &AppHandle, config: &UserConfig) -> Result<(), String> {
    let _guard = CONFIG_IO_LOCK.lock()
        .map_err(|_| "Заблокирован доступ к файлу конфигурации.".to_string())?;
    write_user_config_file_unlocked(app, config)
}

/// Atomically updates the library path without discarding other user preferences.
pub fn update_library_root_config(app: &AppHandle, library_root: String) -> Result<(), String> {
    let _guard = CONFIG_IO_LOCK.lock()
        .map_err(|_| "Заблокирован доступ к файлу конфигурации.".to_string())?;
    let mut config = read_user_config_file_unlocked(app)?.unwrap_or_else(UserConfig::default);
    config.library_root = library_root;
    write_user_config_file_unlocked(app, &config)
}

fn normalize_config(mut config: UserConfig) -> UserConfig {
    config.version = 1;
    config.appearance.accent_hue = config.appearance.accent_hue.min(360);
    let background = config.appearance.background_color.trim().to_ascii_lowercase();
    let valid_background = background.len() == 7
        && background.starts_with('#')
        && background[1..].chars().all(|character| character.is_ascii_hexdigit());
    config.appearance.background_color = if valid_background { background } else { "#101114".to_string() };
    let wallpaper_path = config.appearance.wallpaper_path.trim().replace('\\', "/");
    let valid_wallpaper = wallpaper_path.starts_with("wallpapers/") && !wallpaper_path.split('/').any(|part| part == "." || part == "..");
    config.appearance.wallpaper_path = if valid_wallpaper { wallpaper_path } else { String::new() };
    if config.appearance.theme != "light" { config.appearance.theme = "dark".to_string(); }
    if config.appearance.visual_mode != "standard" { config.appearance.visual_mode = "liquid-glass".to_string(); }
    config.appearance.scale = (((config.appearance.scale.clamp(80, 120) as f64) / 5.0).round() as u8) * 5;
    config.video.volume = if config.video.volume.is_finite() {
        config.video.volume.clamp(0.0, 1.0)
    } else { 1.0 };
    config.grid_columns.photo = config.grid_columns.photo.clamp(2, 7);
    config.grid_columns.video = config.grid_columns.video.clamp(2, 7);
    config.grid_columns.archive = config.grid_columns.archive.clamp(2, 7);
    if config.video_gallery_mode != "tiktok" { config.video_gallery_mode = "grid".to_string(); }
    if !["/", "/gallery/videos", "/gallery/photos", "/gallery/archives", "/database", "/settings", "/import"]
        .contains(&config.startup_page.as_str()) {
        config.startup_page = "/".to_string();
    }
    config.library_root = config.library_root.trim().to_string();
    config
}

#[tauri::command]
pub fn load_user_config(app: AppHandle) -> Result<Option<UserConfig>, String> {
    read_user_config_file(&app)
}

#[tauri::command]
pub fn save_user_config(app: AppHandle, config: UserConfig) -> Result<(), String> {
    write_user_config_file(&app, &config)
}

#[tauri::command]
pub fn get_user_config_path(app: AppHandle) -> Result<String, String> {
    Ok(config_path(&app)?.to_string_lossy().to_string())
}
