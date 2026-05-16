use enigo::{Direction, Enigo, Key, Keyboard};

// arboard owns the CLIPBOARD selection via a background thread. Sleep before
// key-send (PRE_PASTE_MS) so the thread is alive before SelectionRequest events
// arrive. POST_PASTE_MS keeps ownership alive while the target reads.
// POST_RESTORE_MS gives arboard time to re-claim after restoring the original.
pub const PRE_PASTE_MS: u64 = 100;
pub const POST_PASTE_MS: u64 = 250;
pub const POST_RESTORE_MS: u64 = 50;

pub fn send_paste(enigo: &mut Enigo) -> Result<(), String> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub fn send_copy(enigo: &mut Enigo) -> Result<(), String> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub fn release_modifiers(enigo: &mut Enigo) {
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
}
