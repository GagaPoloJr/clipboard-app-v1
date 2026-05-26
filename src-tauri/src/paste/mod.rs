use std::process::Command;
use std::thread;
use std::time::Duration;

use base64::Engine;

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

pub fn get_clipboard_html() -> Option<String> {
    let output = Command::new("osascript")
        .args([
            "-e",
            r#"use framework "AppKit""#,
            "-e",
            r#"set pb to current application's NSPasteboard's generalPasteboard()"#,
            "-e",
            r#"set htmlStr to pb's stringForType:"public.html""#,
            "-e",
            r#"if htmlStr is not missing value then return htmlStr as text"#,
            "-e",
            r#"return ""#,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!("[HTML] osascript failed: {}", String::from_utf8_lossy(&output.stderr));
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let s = stdout.trim().to_string();
    if s.is_empty() {
        eprintln!("[HTML] osascript returned empty (no HTML on clipboard)");
        None
    } else {
        eprintln!("[HTML] detected HTML content ({} bytes)", s.len());
        Some(s)
    }
}

pub fn set_clipboard_html(plain: &str, html: &str) -> Result<(), String> {
    let plain_b64 = base64::engine::general_purpose::STANDARD.encode(plain);
    let html_b64 = base64::engine::general_purpose::STANDARD.encode(html);

    let script = format!(
        "use framework \"AppKit\"\n\
         set pb to current application's NSPasteboard's generalPasteboard()\n\
         pb's clearContents()\n\
         set plainStr to (do shell script \"echo {} | base64 -d\")\n\
         set htmlStr to (do shell script \"echo {} | base64 -d\")\n\
         set theItem to current application's NSPasteboardItem's alloc()'s init()\n\
         theItem's setString:plainStr forType:\"public.utf8-plain-text\"\n\
         theItem's setString:htmlStr forType:\"public.html\"\n\
         pb's writeObjects:{{theItem}}",
        plain_b64, html_b64
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
