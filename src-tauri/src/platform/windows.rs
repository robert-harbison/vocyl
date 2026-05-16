use enigo::{Direction, Enigo, Key, Keyboard};

// Clipboard is owned by a window message on Windows — no background thread
// delay needed before paste, but a brief post-paste wait lets the target consume.
pub const PRE_PASTE_MS: u64 = 0;
pub const POST_PASTE_MS: u64 = 150;
pub const POST_RESTORE_MS: u64 = 0;

pub fn send_paste(enigo: &mut Enigo) -> Result<(), String> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

// Key::C (virtual key code) rather than Key::Unicode('c'): enigo's WinAPI backend
// sends WM_CHAR for Unicode keys, which most apps ignore for shortcut handling.
// Key::C → VK_C is what Ctrl+C actually requires.
pub fn send_copy(enigo: &mut Enigo) -> Result<(), String> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::C, Direction::Click).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub fn release_modifiers(enigo: &mut Enigo) {
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
}
