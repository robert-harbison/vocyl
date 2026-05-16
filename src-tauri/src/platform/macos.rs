use enigo::{Direction, Enigo, Key, Keyboard};

// Brief settle between clipboard-write and key-event prevents races in sandboxed
// apps. Post-paste wait lets the target consume the content.
pub const PRE_PASTE_MS: u64 = 50;
pub const POST_PASTE_MS: u64 = 150;
pub const POST_RESTORE_MS: u64 = 0;

pub fn send_paste(enigo: &mut Enigo) -> Result<(), String> {
    enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub fn send_copy(enigo: &mut Enigo) -> Result<(), String> {
    enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub fn release_modifiers(enigo: &mut Enigo) {
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
    enigo.key(Key::Meta, Direction::Release).ok();
}
