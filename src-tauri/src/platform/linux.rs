use arboard::Clipboard;
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

/// Try to read the X11/Wayland PRIMARY selection (the "highlighted text" buffer).
///
/// On X11 and Wayland, any text the user drags over is placed in PRIMARY
/// automatically — no Ctrl+C needed.  This avoids the arboard CLIPBOARD
/// background-thread race condition entirely.
///
/// Returns `None` if the selection is empty or unavailable.
pub fn try_get_primary_selection() -> Option<String> {
    // Attempt arboard's native PRIMARY selection API (arboard >= 3.3.0).
    if let Some(text) = try_primary_via_arboard() {
        return Some(text);
    }

    // arboard failed (e.g. compositor doesn't support zwp-primary-selection,
    // or an older X11 quirk).  Try external tools as a subprocess fallback.
    try_primary_via_subprocess()
}

/// Use arboard's built-in LinuxClipboardKind::Primary API.
fn try_primary_via_arboard() -> Option<String> {
    use arboard::GetExtLinux;

    let mut clipboard = Clipboard::new().ok()?;
    match clipboard
        .get()
        .clipboard(arboard::LinuxClipboardKind::Primary)
        .text()
    {
        Ok(text) if !text.is_empty() => {
            log::debug!("primary selection via arboard: {} chars", text.len());
            Some(text)
        }
        Ok(_) => None,
        Err(e) => {
            log::debug!("arboard primary unavailable: {e}");
            None
        }
    }
}

/// Subprocess fallback: try wl-paste, xsel, xclip in order.
fn try_primary_via_subprocess() -> Option<String> {
    // wl-paste: Wayland clipboard utility (also works on some XWayland sessions)
    if let Some(t) = run_cmd("wl-paste", &["--primary", "--no-newline"]) {
        return Some(t);
    }
    // xsel: common X11 tool
    if let Some(t) = run_cmd("xsel", &["--output", "--primary"]) {
        return Some(t);
    }
    // xclip: alternative X11 tool
    if let Some(t) = run_cmd("xclip", &["-out", "-selection", "primary"]) {
        return Some(t);
    }
    None
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(cmd).args(args).output().ok()?;
    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        if !text.is_empty() {
            log::debug!("{cmd}: primary {} chars", text.len());
            return Some(text);
        }
    }
    None
}
