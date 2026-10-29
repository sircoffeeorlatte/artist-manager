mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::filesystem::create_library_structure,
            commands::filesystem::get_saved_library_root,
            commands::filesystem::save_library_root,
            commands::filesystem::register_library_root,
            commands::filesystem::import_media_files,
            commands::filesystem::recover_interrupted_imports,
            commands::filesystem::list_library_media,
            commands::wallpapers::list_wallpapers,
            commands::wallpapers::import_wallpaper,
            commands::config::load_user_config,
            commands::config::save_user_config,
            commands::config::get_user_config_path,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска приложения Tauri");
}
