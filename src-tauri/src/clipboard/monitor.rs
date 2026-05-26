use arboard::Clipboard;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

use crate::db::models;

pub fn start_monitoring(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut clipboard = Clipboard::new().unwrap();
        let mut last_content: Option<String> = None;

        loop {
            if let Ok(text) = clipboard.get_text() {
                let should_insert = match &last_content {
                    Some(last) => &text != last,
                    None => true,
                };

                if should_insert && !text.is_empty() {
                    let preview = text.chars().take(120).collect::<String>();
                    let item = models::insert_clipboard_item(&text, "text", &preview);
                    if let Ok(item) = item {
                        let _ = app.emit("clipboard-new-item", &item);
                    }
                    last_content = Some(text);
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    });
}
