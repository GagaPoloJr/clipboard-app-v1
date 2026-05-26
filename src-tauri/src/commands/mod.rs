use tauri::{AppHandle, Manager, State};
use crate::db::models::{Database, ClipboardItem};
use crate::paste;

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
    clipboard.set_text(item.content).map_err(|e| e.to_string())?;

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

    paste::copy_and_paste(&item.content)?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    Ok(())
}
