// Platform-specific keyboard and clipboard implementations live in the submodules.
// This file contains only the shared flow; there are no #[cfg] blocks here.

use arboard::Clipboard;
use enigo::{Enigo, Settings};
use std::time::Duration;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use self::windows::{release_modifiers, send_copy, send_paste, POST_PASTE_MS, POST_RESTORE_MS, PRE_PASTE_MS};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use self::macos::{release_modifiers, send_copy, send_paste, POST_PASTE_MS, POST_RESTORE_MS, PRE_PASTE_MS};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use self::linux::{release_modifiers, send_copy, send_paste, POST_PASTE_MS, POST_RESTORE_MS, PRE_PASTE_MS};

const PRE_COPY_MS: u64 = 50;
const POST_COPY_MS: u64 = 200;

/// Inject `text` at the OS cursor by placing it on the clipboard and sending
/// the platform paste shortcut. Saves and restores the original clipboard.
pub fn inject_text(text: &str) -> Result<(), String> {
    log::info!("Injecting text ({} chars) at cursor", text.len());

    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;
    let saved = clipboard.get_text().ok();
    log::debug!("Saved clipboard ({} chars)", saved.as_deref().unwrap_or("").len());

    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to set clipboard: {e}"))?;

    if PRE_PASTE_MS > 0 {
        std::thread::sleep(Duration::from_millis(PRE_PASTE_MS));
    }

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Input injector error: {e:?}"))?;
    send_paste(&mut enigo)?;

    std::thread::sleep(Duration::from_millis(POST_PASTE_MS));

    if let Some(ref original) = saved {
        if let Err(e) = clipboard.set_text(original.as_str()) {
            log::warn!("Failed to restore clipboard: {}", e);
        } else {
            if POST_RESTORE_MS > 0 {
                std::thread::sleep(Duration::from_millis(POST_RESTORE_MS));
            }
            log::debug!("Clipboard restored");
        }
    }

    log::info!("Text injected successfully");
    Ok(())
}

/// Send the platform copy shortcut and return whatever text landed on the clipboard.
/// Clears the clipboard first so an empty result is distinguishable from a stale value.
/// Saves and restores the original clipboard contents.
pub fn get_selected_text() -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let saved = clipboard.get_text().ok();

    // Clear before the copy shortcut: if nothing is selected, the shortcut is a
    // no-op and the clipboard stays empty, giving us a reliable "nothing selected" signal.
    clipboard.set_text("").map_err(|e| e.to_string())?;

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Input handler error: {e:?}"))?;

    release_modifiers(&mut enigo);

    std::thread::sleep(Duration::from_millis(PRE_COPY_MS));
    send_copy(&mut enigo)?;
    std::thread::sleep(Duration::from_millis(POST_COPY_MS));

    let selected = clipboard.get_text().unwrap_or_default();
    log::debug!("Clipboard after copy: {} chars", selected.len());

    if let Some(old) = saved {
        if !old.is_empty() {
            clipboard.set_text(old).ok();
        }
    }

    Ok(selected)
}
