use arboard::Clipboard;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{sleep, Duration};

use crate::db::models::{self, Database};

const MAX_HISTORY: usize = 500;

use std::sync::Mutex;

pub static IGNORE_TEXT: Mutex<Option<String>> = Mutex::new(None);

pub fn start_monitoring(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let db = app.state::<Database>();
        let mut clipboard = Clipboard::new().unwrap();
        let mut last_content: Option<String> = None;

        loop {
            if let Ok(text) = clipboard.get_text() {
                let mut ignore_matched = false;
                {
                    let mut ignore_text = IGNORE_TEXT.lock().unwrap();
                    if let Some(ignored) = ignore_text.as_ref() {
                        if *ignored == text {
                            ignore_matched = true;
                            *ignore_text = None; // Reset after ignoring once
                        }
                    }
                }

                if ignore_matched {
                    last_content = Some(text.clone());
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }

                let should_insert = match &last_content {
                    Some(last) => &text != last,
                    None => true,
                };

                if should_insert && !text.is_empty() {
                    let preview = get_preview(&text);
                    if let Ok(item) = models::create_clipboard_item(&text, "text", &preview) {
                        if let Ok(inserted_item) = db.insert(&item) {
                            let _ = db.purge_oldest(MAX_HISTORY);
                            let _ = app.emit("clipboard-new-item", &inserted_item);
                        }
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
