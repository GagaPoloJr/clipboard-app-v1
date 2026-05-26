mod clipboard;
mod commands;
mod db;
mod paste;

use db::models::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            let db_path = app_dir.join("clipboard.db");
            let db = Database::new(db_path.to_str().unwrap())
                .expect("failed to initialize database");

            app.manage(db);

            let handle = app.handle().clone();
            clipboard::monitor::start_monitoring(handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_history,
            commands::delete_item,
            commands::toggle_pin,
            commands::clear_history,
            commands::copy_and_paste,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
