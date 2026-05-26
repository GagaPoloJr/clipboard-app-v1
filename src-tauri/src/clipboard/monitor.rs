use arboard::Clipboard;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{sleep, Duration};

use crate::db::models::{self, Database};

const MAX_HISTORY: usize = 500;

pub fn start_monitoring(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let db = app.state::<Database>();
        let mut clipboard = Clipboard::new().unwrap();
        let mut last_content: Option<String> = None;

        loop {
            if let Ok(text) = clipboard.get_text() {
                let should_insert = match &last_content {
                    Some(last) => &text != last,
                    None => true,
                };

                if should_insert && !text.is_empty() {
                    let preview = get_preview(&text);
                    if let Ok(item) = models::create_clipboard_item(&text, "text", &preview) {
                        let item_clone = item.clone();
                        let _ = db.insert(&item);
                        let _ = db.purge_oldest(MAX_HISTORY);
                        let _ = app.emit("clipboard-new-item", &item_clone);
                    }
                    last_content = Some(text);
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    });
}

fn get_preview(text: &str) -> String {
    text.chars().take(120).collect()
}
