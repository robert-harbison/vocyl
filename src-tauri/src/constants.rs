/// Push-to-talk shortcut. Change this to whatever key you prefer.
/// Valid modifiers: Ctrl, Shift, Alt, Super/Meta, CmdOrCtrl
pub const PTT_SHORTCUT: &str = "CmdOrCtrl+Shift+Space";

/// Width / height of the floating overlay pill (logical pixels).
pub const OVERLAY_W: f64 = 280.0;
pub const OVERLAY_H: f64 = 64.0;

/// Distance from the top of the screen (logical pixels).
pub const OVERLAY_Y: f64 = 24.0;

/// Bump this whenever a breaking change is made to VocylSettings.
/// A future migration function can inspect the stored value and upgrade it.
pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
