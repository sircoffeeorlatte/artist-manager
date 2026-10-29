mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Err(error) = commands::database::initialize_database(app.handle()) {
                eprintln!("Не удалось инициализировать SQLite: {error}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::filesystem::create_library_structure,
            commands::filesystem::get_saved_library_root,
            commands::filesystem::save_library_root,
            commands::filesystem::register_library_root,
            commands::filesystem::import_media_files,
            commands::filesystem::recover_interrupted_imports,
            commands::filesystem::list_library_media,
            commands::database::refresh_media_index,
            commands::database::get_media_index_status,
            commands::database::list_indexed_media,
            commands::database::set_media_tags,
            commands::wallpapers::list_wallpapers,
            commands::wallpapers::import_wallpaper,
            commands::config::load_user_config,
            commands::config::save_user_config,
            commands::config::get_user_config_path,
            commands::config::open_folder_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска приложения Tauri");
}
