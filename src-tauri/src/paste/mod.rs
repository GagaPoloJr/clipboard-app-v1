use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn copy_and_paste(content: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(content).map_err(|e| e.to_string())?;

    thread::sleep(Duration::from_millis(100));

    Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to keystroke \"v\" using command down",
        ])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}
