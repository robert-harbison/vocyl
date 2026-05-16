// All keyboard and clipboard operations that differ by OS live here.
// The rest of the codebase is free of #[cfg] scatter for these concerns.

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::time::Duration;

// ── Platform timing ───────────────────────────────────────────────────────────

// Linux (X11/XWayland): arboard owns the CLIPBOARD selection via a background
// thread. The thread must be alive and responsive before SelectionRequest
// events arrive, so we sleep between writing and key-send (PRE_PASTE_MS), and
// again after the paste to keep ownership alive while the target reads it
// (POST_PASTE_MS). After restoring we give arboard time to re-claim (POST_RESTORE_MS).
#[cfg(target_os = "linux")]
const PRE_PASTE_MS: u64 = 100;
#[cfg(target_os = "linux")]
const POST_PASTE_MS: u64 = 250;
#[cfg(target_os = "linux")]
const POST_RESTORE_MS: u64 = 50;

// macOS: a brief settle between clipboard-write and key-event prevents races in
// sandboxed apps. Post-paste wait lets the target consume the content.
#[cfg(target_os = "macos")]
const PRE_PASTE_MS: u64 = 50;
#[cfg(target_os = "macos")]
const POST_PASTE_MS: u64 = 150;
#[cfg(target_os = "macos")]
const POST_RESTORE_MS: u64 = 0;

// Windows: clipboard is owned by a window message; no background thread delay needed.
#[cfg(target_os = "windows")]
const PRE_PASTE_MS: u64 = 0;
#[cfg(target_os = "windows")]
const POST_PASTE_MS: u64 = 150;
#[cfg(target_os = "windows")]
const POST_RESTORE_MS: u64 = 0;

// After releasing modifiers from the global shortcut, wait before sending the
// copy key so the OS treats them as separate events.
const PRE_COPY_MS: u64 = 50;
// After Ctrl/Cmd+C, wait for the target app to write its selection to clipboard.
const POST_COPY_MS: u64 = 200;

// ── Platform key helpers ──────────────────────────────────────────────────────

// Ctrl+V (Windows/Linux) or Cmd+V (macOS).
fn send_paste(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("{e:?}"))?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    }
    Ok(())
}

// Ctrl+C (Linux) / VK_C (Windows) / Cmd+C (macOS).
// Windows uses Key::C (virtual key code) rather than Key::Unicode('c') because
// enigo's WinAPI backend sends WM_CHAR for Unicode keys, which most apps ignore
// for shortcut handling. Key::C → VK_C is what Ctrl+C actually requires.
fn send_copy(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::C, Direction::Click).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("{e:?}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    }
    Ok(())
}

// ── Public API ────────────────────────────────────────────────────────────────

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

    // Release any modifiers that may still be physically held from the global shortcut.
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
    #[cfg(target_os = "macos")]
    enigo.key(Key::Meta, Direction::Release).ok();

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
