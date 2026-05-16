//! macOS text bridge — Accessibility API primary, clipboard fallback.
//!
//! # Selection reading
//! Reads the `AXSelectedText` attribute of the focused UI element via the
//! macOS Accessibility API (`ApplicationServices` framework). This requires
//! that the user has granted Accessibility permission to Vocyl in
//! System Settings → Privacy & Security → Accessibility.
//!
//! Falls back to clipboard + Cmd+C when:
//! - Permission has not been granted (`AXIsProcessTrusted` returns false).
//! - The focused element does not expose `AXSelectedText`.
//! - Any AX API call returns an error.
//!
//! # Text insertion
//! Clipboard paste (Cmd+V) is used. `AXValue` write via AX API replaces the
//! entire field value, which is incorrect for cursor-position insertion, so
//! clipboard paste is the right primitive for PTT-style dictation.
//!
//! # Memory management
//! All `CFTypeRef`/`AXUIElementRef` values returned from AX "Create" functions
//! are wrapped in `OwnedCFRef` which calls `CFRelease` on drop.

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

use enigo::{Direction, Enigo, Key, Keyboard};

use crate::text_bridge::{
    TextBridge, TextBridgeCapabilities, TextBridgeError,
    clipboard_fallback::{FallbackConfig, get_selection_via_copy, insert_via_paste},
};

// ── C types ───────────────────────────────────────────────────────────────────

type CFTypeRef = *const c_void;
type AXUIElementRef = *const c_void;
type AXError = i32;

const AX_SUCCESS: AXError = 0;
const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

// ── macOS framework bindings ──────────────────────────────────────────────────

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFTypeRef, // CFStringRef
        value: *mut CFTypeRef,
    ) -> AXError;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFStringCreateWithBytes(
        allocator: *const c_void,
        bytes: *const u8,
        num_bytes: isize,
        encoding: u32,
        is_external_representation: u8,
    ) -> CFTypeRef; // returns CFStringRef (Create rule)
    fn CFStringGetCStringPtr(str: CFTypeRef, encoding: u32) -> *const c_char;
    fn CFStringGetCString(
        str: CFTypeRef,
        buffer: *mut c_char,
        buffer_size: isize,
        encoding: u32,
    ) -> bool;
    fn CFStringGetLength(str: CFTypeRef) -> isize;
}

// ── RAII wrapper for CF Create-rule references ────────────────────────────────

struct OwnedCFRef(CFTypeRef);

impl OwnedCFRef {
    fn null() -> Self {
        Self(std::ptr::null())
    }
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
    fn as_raw(&self) -> CFTypeRef {
        self.0
    }
}

impl Drop for OwnedCFRef {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) }
        }
    }
}

// ── CF string helpers ─────────────────────────────────────────────────────────

/// Create a CFStringRef from a Rust str. Returns an OwnedCFRef that releases on drop.
fn rust_to_cfstring(s: &str) -> OwnedCFRef {
    let bytes = s.as_bytes();
    let cf = unsafe {
        CFStringCreateWithBytes(
            std::ptr::null(),
            bytes.as_ptr(),
            bytes.len() as isize,
            CF_STRING_ENCODING_UTF8,
            0,
        )
    };
    OwnedCFRef(cf)
}

/// Convert a CFStringRef to a Rust String. Returns None if the value is null
/// or cannot be decoded as UTF-8.
///
/// # Safety
/// `cf_str` must be a valid CFStringRef or null.
unsafe fn cfstring_to_rust(cf_str: CFTypeRef) -> Option<String> {
    if cf_str.is_null() {
        return None;
    }
    // Fast path: direct UTF-8 pointer (common for ASCII strings).
    let ptr = CFStringGetCStringPtr(cf_str, CF_STRING_ENCODING_UTF8);
    if !ptr.is_null() {
        return CStr::from_ptr(ptr).to_str().ok().map(str::to_owned);
    }
    // Slow path: copy via CFStringGetCString.
    let char_len = CFStringGetLength(cf_str);
    if char_len <= 0 {
        return Some(String::new());
    }
    // UTF-8 is at most 4 bytes per Unicode code point (CFStringGetLength returns UTF-16 units).
    let buf_size = (char_len as usize) * 4 + 1;
    let mut buf: Vec<u8> = vec![0; buf_size];
    let ok = CFStringGetCString(
        cf_str,
        buf.as_mut_ptr() as *mut c_char,
        buf_size as isize,
        CF_STRING_ENCODING_UTF8,
    );
    if ok {
        CStr::from_ptr(buf.as_ptr() as *const c_char)
            .to_str()
            .ok()
            .map(str::to_owned)
    } else {
        None
    }
}

// ── Accessibility helpers ─────────────────────────────────────────────────────

/// Copy an AX attribute value from `element` into an `OwnedCFRef`.
/// Returns a null OwnedCFRef when the attribute is unavailable.
unsafe fn ax_copy_attribute(element: AXUIElementRef, attribute: &str) -> OwnedCFRef {
    let attr_cf = rust_to_cfstring(attribute);
    if attr_cf.is_null() {
        return OwnedCFRef::null();
    }
    let mut value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(element, attr_cf.as_raw(), &mut value);
    if err == AX_SUCCESS && !value.is_null() {
        OwnedCFRef(value)
    } else {
        OwnedCFRef::null()
    }
}

/// Read the selected text from the currently focused UI element via AX APIs.
/// Returns None when nothing is selected or the element does not expose
/// `AXSelectedText`.
fn ax_get_selected_text() -> Result<Option<String>, TextBridgeError> {
    unsafe { ax_get_selected_text_unsafe() }
}

unsafe fn ax_get_selected_text_unsafe() -> Result<Option<String>, TextBridgeError> {
    let system_wide = AXUIElementCreateSystemWide();
    if system_wide.is_null() {
        return Err(TextBridgeError::NativeApiUnavailable {
            reason: "AXUIElementCreateSystemWide returned null".into(),
        });
    }
    let _system_wide_guard = OwnedCFRef(system_wide);

    // Focused application element
    let focused_app = ax_copy_attribute(system_wide, "AXFocusedApplication");
    if focused_app.is_null() {
        log::debug!("AX: no focused application");
        return Ok(None);
    }

    // Focused UI element within the application
    let focused_elem = ax_copy_attribute(focused_app.as_raw(), "AXFocusedUIElement");
    if focused_elem.is_null() {
        log::debug!("AX: no focused UI element");
        return Ok(None);
    }

    // AXSelectedText — the text the user has highlighted
    let selected_text_ref = ax_copy_attribute(focused_elem.as_raw(), "AXSelectedText");
    if selected_text_ref.is_null() {
        log::debug!("AX: AXSelectedText attribute unavailable");
        return Ok(None);
    }

    // Verify the returned value is actually a CFString before casting
    if CFGetTypeID(selected_text_ref.as_raw()) != CFStringGetTypeID() {
        log::debug!("AX: AXSelectedText is not a CFString");
        return Ok(None);
    }

    let text = cfstring_to_rust(selected_text_ref.as_raw());
    Ok(text.filter(|s| !s.is_empty()))
}

// ── Key simulation helpers ────────────────────────────────────────────────────

fn mac_send_paste(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Meta, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Meta, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn mac_send_copy(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Meta, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Meta, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn mac_release_modifiers(enigo: &mut Enigo) {
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
    // Release Meta to prevent it from being held through the copy shortcut
    // in sandboxed or system-prompt apps.
    enigo.key(Key::Meta, Direction::Release).ok();
}

fn macos_fallback_config() -> FallbackConfig {
    FallbackConfig {
        send_paste: mac_send_paste,
        send_copy: mac_send_copy,
        release_modifiers: mac_release_modifiers,
        // Brief settle between clipboard-write and key-event prevents races
        // in sandboxed apps. Post-paste wait lets the target consume the content.
        pre_paste_ms: 50,
        post_paste_ms: 150,
        post_restore_ms: 0,
    }
}

// ── TextBridge implementation ─────────────────────────────────────────────────

pub struct MacosTextBridge;

// Safety: MacosTextBridge holds no state; AX objects are created per-call.
unsafe impl Send for MacosTextBridge {}
unsafe impl Sync for MacosTextBridge {}

impl TextBridge for MacosTextBridge {
    fn capabilities(&self) -> TextBridgeCapabilities {
        let ax_trusted = unsafe { AXIsProcessTrusted() };
        TextBridgeCapabilities {
            can_read_selection_native: ax_trusted,
            can_insert_native: false, // Clipboard paste is correct for cursor-position insert
            can_replace_selection_native: false,
            uses_clipboard_fallback: true,
            requires_accessibility_permission: !ax_trusted,
            wayland_limited: false,
            backend_name: "macos-ax",
        }
    }

    fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError> {
        let trusted = unsafe { AXIsProcessTrusted() };
        if trusted {
            match ax_get_selected_text() {
                Ok(result) => {
                    log::debug!(
                        "AX selection: {} chars",
                        result.as_deref().map_or(0, str::len)
                    );
                    return Ok(result);
                }
                Err(e) => {
                    log::warn!("AX get_selected_text failed ({e}), using clipboard fallback");
                }
            }
        } else {
            log::warn!(
                "Accessibility permission not granted — using clipboard fallback. \
                 Grant access in System Settings → Privacy & Security → Accessibility."
            );
        }
        get_selection_via_copy(&macos_fallback_config())
    }

    fn insert_text(&self, text: &str) -> Result<(), TextBridgeError> {
        insert_via_paste(text, &macos_fallback_config())
    }

    fn replace_selection(&self, text: &str) -> Result<(), TextBridgeError> {
        insert_via_paste(text, &macos_fallback_config())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_bridge::TextBridge;

    #[test]
    fn capabilities_backend_name() {
        let bridge = MacosTextBridge;
        assert_eq!(bridge.capabilities().backend_name, "macos-ax");
    }

    #[test]
    fn capabilities_not_wayland() {
        let bridge = MacosTextBridge;
        assert!(!bridge.capabilities().wayland_limited);
    }
}
