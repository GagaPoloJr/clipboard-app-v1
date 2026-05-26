use arboard::Clipboard;
use base64::Engine;
use png::{ColorType, Encoder};
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
        let mut last_text: Option<String> = None;
        let mut last_image_bytes: Option<Vec<u8>> = None;

        loop {
            if let Ok(image_data) = clipboard.get_image() {
                let raw_bytes = image_data.bytes.into_owned();

                // Compare raw bytes to prevent expensive PNG encoding every 500ms
                let is_new_image = match &last_image_bytes {
                    Some(last) => last != &raw_bytes,
                    None => true,
                };

                if is_new_image {
                    let w = image_data.width as u32;
                    let h = image_data.height as u32;

                    let mut png_bytes = Vec::new();
                    {
                        let mut encoder = Encoder::new(&mut png_bytes, w, h);
                        encoder.set_color(ColorType::Rgba);
                        encoder.set_depth(png::BitDepth::Eight);
                        if let Ok(mut writer) = encoder.write_header() {
                            let _ = writer.write_image_data(&raw_bytes);
                        }
                    }

                    if !png_bytes.is_empty() {
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                        let preview = format!("[Image {}x{}]", w, h);

                        if let Ok(item) = models::create_clipboard_item(&b64, "image", &preview) {
                            if let Ok(inserted_item) = db.insert(&item) {
                                let _ = db.purge_oldest(MAX_HISTORY);
                                let _ = app.emit("clipboard-new-item", &inserted_item);
                            }
                        }
                    }
                }

                last_image_bytes = Some(raw_bytes);
            } else if let Ok(text) = clipboard.get_text() {
                let mut ignore_matched = false;
                {
                    let mut ignore_text = IGNORE_TEXT.lock().unwrap();
                    if let Some(ignored) = ignore_text.as_ref() {
                        if *ignored == text {
                            ignore_matched = true;
                            *ignore_text = None;
                        }
                    }
                }

                if ignore_matched {
                    last_text = Some(text.clone());
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }

                let should_insert = match &last_text {
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
                    last_text = Some(text);
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    });
}

fn get_preview(text: &str) -> String {
    text.chars().take(120).collect()
}
