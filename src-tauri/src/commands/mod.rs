use std::borrow::Cow;
use std::io::Cursor;

use base64::Engine;
use png::ColorType;
use tauri::{AppHandle, Manager, State};

use crate::db::models::{Database, ClipboardItem};

#[tauri::command]
pub fn get_history(
    db: State<Database>,
    search: Option<String>,
) -> Result<Vec<ClipboardItem>, String> {
    db.get_all(search.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_item(db: State<Database>, id: String) -> Result<(), String> {
    db.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_pin(db: State<Database>, id: String) -> Result<bool, String> {
    db.toggle_pin(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_history(db: State<Database>) -> Result<(), String> {
    db.clear_unpinned().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_to_clipboard(db: State<Database>, id: String) -> Result<(), String> {
    let items = db.get_all(None).map_err(|e| e.to_string())?;
    let item = items.into_iter().find(|i| i.id == id)
        .ok_or_else(|| "Item not found".to_string())?;

    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;

    if item.content_type == "image" {
        let png_bytes = base64::engine::general_purpose::STANDARD
            .decode(&item.content)
            .map_err(|e| e.to_string())?;

        let decoder = png::Decoder::new(Cursor::new(&png_bytes));
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;

        let w = reader.info().width as usize;
        let h = reader.info().height as usize;
        let is_rgba = matches!(reader.info().color_type, ColorType::Rgba);

        let mut rgba = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut rgba).map_err(|e| e.to_string())?;

        if is_rgba {
            clipboard
                .set_image(arboard::ImageData {
                    width: w,
                    height: h,
                    bytes: Cow::Owned(rgba),
                })
                .map_err(|e| e.to_string())?;
        }
    } else {
        {
            let mut ignore = crate::clipboard::monitor::IGNORE_TEXT.lock().unwrap();
            *ignore = Some(item.content.clone());
        }
        clipboard.set_text(item.content).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn copy_and_paste(
    app: AppHandle,
    db: State<Database>,
    id: String,
) -> Result<(), String> {
    let items = db.get_all(None).map_err(|e| e.to_string())?;
    let item = items.into_iter().find(|i| i.id == id)
        .ok_or_else(|| "Item not found".to_string())?;

    if item.content_type == "image" {
        return copy_to_clipboard(db, id);
    }

    {
        let mut ignore = crate::clipboard::monitor::IGNORE_TEXT.lock().unwrap();
        *ignore = Some(item.content.clone());
    }

    crate::paste::copy_and_paste(&item.content)?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    Ok(())
}
