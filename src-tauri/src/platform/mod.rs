// Platform-specific text injection and selection reading.
// All implementation lives in the text_bridge module; this file is a thin
// adapter that preserves the existing call-site signatures used by overlay.rs.

/// Inject `text` at the OS cursor. Called by the PTT handler after transcription.
pub fn inject_text(text: &str) -> Result<(), String> {
    crate::text_bridge::insert_text(text).map_err(|e| e.to_string())
}

/// Read the user's currently selected text.
/// Returns an empty string when nothing is selected (for backward compatibility
/// with callers that check `t.trim().is_empty()`).
pub fn get_selected_text() -> Result<String, String> {
    crate::text_bridge::get_selected_text()
        .map(|opt| opt.unwrap_or_default())
        .map_err(|e| e.to_string())
}
