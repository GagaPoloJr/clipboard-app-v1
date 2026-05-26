mod clipboard;
mod commands;
mod db;
mod paste;

use std::sync::OnceLock;
use std::time::Instant;
use std::sync::Mutex;
use db::models::Database;
use tauri::{LogicalPosition, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
static LAST_TOGGLE: Mutex<Option<Instant>> = Mutex::new(None);

fn position_window_top_right(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let scale_factor = monitor.scale_factor();
        let monitor_size = monitor.size();
        let monitor_logical = monitor_size.to_logical::<f64>(scale_factor);
        if let Ok(window_size) = window.outer_size() {
            let window_logical = window_size.to_logical::<f64>(scale_factor);
            let x = (monitor_logical.width - window_logical.width - 16.0).max(0.0);
            let y = 32.0;
            let _ = window.set_position(LogicalPosition::new(x, y));
        }
    }
}

fn show_window(handle: &tauri::AppHandle) {
    let mut last = LAST_TOGGLE.lock().unwrap();
    let now = Instant::now();
    if let Some(prev) = *last {
        if now.duration_since(prev).as_millis() < 300 {
            eprintln!("[SHOW] debounce skipped: too soon");
            return;
        }
    }
    *last = Some(now);
    drop(last);

    if let Some(window) = handle.get_webview_window("main") {
        let visible = window.is_visible();
        eprintln!("[SHOW] is_visible() = {:?}", visible);
        
        #[cfg(target_os = "macos")]
        if let Err(e) = handle.show() {
            eprintln!("[SHOW] handle.show() error: {:?}", e);
        }

        match visible {
            Ok(true) => {
                eprintln!("[SHOW] already visible, re-focusing");
                let _ = window.set_focus();
            }
            Ok(false) => {
                eprintln!("[SHOW] showing window");
                position_window_top_right(&window);
                let _ = window.show();
                let _ = window.set_focus();
            }
            _ => {}
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, shortcut, event| {
                    eprintln!("[SHORTCUT] handler fired: shortcut={:?} state={:?}", shortcut, event.state);
                    if event.state == ShortcutState::Pressed {
                        if let Some(handle) = APP_HANDLE.get() {
                            show_window(handle);
                        } else {
                            eprintln!("[SHORTCUT] APP_HANDLE not set yet!");
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let _ = APP_HANDLE.set(app.handle().clone());

            eprintln!("[SETUP] registering shortcut Cmd+Shift+V");
            app.global_shortcut().register(
                Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV),
            )?;
            eprintln!("[SETUP] shortcut registered OK");

            let show_item = MenuItemBuilder::with_id("toggle", "Show/Hide").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "toggle" => {
                            show_window(app);
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_window(tray.app_handle());
                    }
                })
                .build(app)?;

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
            commands::copy_to_clipboard,
            commands::copy_and_paste,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
