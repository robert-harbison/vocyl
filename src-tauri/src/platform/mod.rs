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

/// Read whatever text the user currently has selected and return it.
///
/// On Linux, the PRIMARY selection (updated automatically as the user highlights
/// text) is tried first — this sidesteps the arboard X11 background-thread
/// ownership race that causes the CLIPBOARD approach to return "" even when
/// text is selected.  Falls back to a Ctrl+C approach using fresh Clipboard
/// instances to avoid the same race on the CLIPBOARD selection.
///
/// On macOS and Windows, a single Clipboard instance is reused throughout
/// because NSPasteboard and the WinAPI clipboard are not subject to the same
/// background-thread ownership model.
pub fn get_selected_text() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    return get_selected_text_linux();

    #[cfg(not(target_os = "linux"))]
    get_selected_text_via_copy()
}

/// macOS / Windows: save clipboard, clear, Ctrl/Cmd+C, read back, restore.
/// Reusing one Clipboard instance is safe here (no background-thread race).
#[cfg(not(target_os = "linux"))]
fn get_selected_text_via_copy() -> Result<String, String> {
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

/// Linux: try PRIMARY selection first; fall back to Ctrl+C with fresh instances.
#[cfg(target_os = "linux")]
fn get_selected_text_linux() -> Result<String, String> {
    // On X11 and Wayland, highlighted text is placed in the PRIMARY selection
    // automatically.  Reading it requires no Ctrl+C and has no race condition.
    if let Some(text) = linux::try_get_primary_selection() {
        log::debug!("Clipboard after copy: {} chars (primary selection)", text.len());
        return Ok(text);
    }

    // PRIMARY was empty (nothing highlighted).  Fall back to Ctrl+C on the
    // CLIPBOARD selection, using fresh Clipboard instances each time to avoid
    // the arboard background-thread race on X11:
    //
    // Bug: arboard's set_text("") spawns a background thread that *owns* the
    // X11 CLIPBOARD selection.  When the focused app handles Ctrl+C it claims
    // ownership, but arboard may not have processed the SelectionClear event
    // before get_text() is called on the same instance, so it returns the
    // stale "" it stored rather than querying the new owner.
    //
    // Fix: drop the Clipboard that owns "" before creating a fresh one for
    // reading; the fresh instance has no cached owner state and will correctly
    // send XConvertSelection to the focused app.

    let saved = {
        let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
        let saved = cb.get_text().ok();
        cb.set_text("").map_err(|e| e.to_string())?;
        saved
        // cb drops here — arboard's background thread exits, releasing X11
        // CLIPBOARD ownership before Ctrl+C is sent.
    };

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Input handler error: {e:?}"))?;
    release_modifiers(&mut enigo);
    std::thread::sleep(Duration::from_millis(PRE_COPY_MS));
    send_copy(&mut enigo)?;
    std::thread::sleep(Duration::from_millis(POST_COPY_MS));

    // Fresh instance: queries the current X11 CLIPBOARD owner (the focused app)
    // without any stale background-thread state.
    let selected = {
        let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
        cb.get_text().unwrap_or_default()
    };
    log::debug!("Clipboard after copy: {} chars (ctrl-c fallback)", selected.len());

    if let Some(old) = saved {
        if !old.is_empty() {
            if let Ok(mut cb) = Clipboard::new() {
                cb.set_text(old).ok();
            }
        }
    }

    Ok(selected)
}
