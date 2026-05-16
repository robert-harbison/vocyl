//! Windows text bridge — UI Automation primary, clipboard fallback.
//!
//! # UI Automation via raw COM vtable calls
//!
//! The `windows` crate version pinned by `enigo` (0.58) does not expose
//! `Win32_UI_UIAutomation` bindings. Rather than adding a second version of
//! the crate, this module calls UIAutomation COM interfaces directly through
//! their vtables using raw unsafe FFI. The vtable offsets are fixed by the
//! Windows SDK ABI and are verified against `UIAutomationClient.h`:
//!
//! | Interface                     | Method              | vtable index |
//! |-------------------------------|---------------------|--------------|
//! | IUIAutomation                 | GetFocusedElement   | 8            |
//! | IUIAutomationElement          | GetCurrentPattern   | 16           |
//! | IUIAutomationTextPattern      | GetSelection        | 5            |
//! | IUIAutomationTextRangeArray   | get_Length          | 3            |
//! | IUIAutomationTextRangeArray   | GetElement          | 4            |
//! | IUIAutomationTextRange        | GetText             | 12           |
//!
//! # Selection reading
//! UIAutomation reads selected text without touching the clipboard, returning
//! `None` when there is genuinely no selection. Falls back to clipboard + Ctrl+C
//! when UIA is unavailable (element lacks TextPattern support) or errors.
//!
//! # Text insertion
//! Clipboard paste (Ctrl+V) is used. `IUIAutomationValuePattern::SetValue`
//! replaces the entire field — incorrect for PTT dictation. Clipboard paste
//! naturally inserts at the cursor or replaces the current selection.

use std::os::raw::c_void;

use enigo::{Direction, Enigo, Key, Keyboard};

use crate::text_bridge::{
    TextBridge, TextBridgeCapabilities, TextBridgeError,
    clipboard_fallback::{FallbackConfig, get_selection_via_copy, insert_via_paste},
};

// ── COM types ─────────────────────────────────────────────────────────────────

type HRESULT = i32;
type BSTR = *mut u16;

// UIAutomation pattern ID for text (constant from Windows SDK)
const UIA_TEXT_PATTERN_ID: i32 = 10014;

/// COM GUID — matches the Windows SDK `GUID` struct layout.
#[repr(C)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

// CLSID_CUIAutomation {FF48DBA4-60EF-4201-AA87-54103EEF594E}
const CLSID_CUIAUTOMATION: GUID = GUID {
    data1: 0xFF48_DBA4,
    data2: 0x60EF,
    data3: 0x4201,
    data4: [0xAA, 0x87, 0x54, 0x10, 0x3E, 0xEF, 0x59, 0x4E],
};
// IID_IUIAutomation {30CBE57D-D9D0-452A-AB13-7AC5AC4825EE}
const IID_IUIAUTOMATION: GUID = GUID {
    data1: 0x30CB_E57D,
    data2: 0xD9D0,
    data3: 0x452A,
    data4: [0xAB, 0x13, 0x7A, 0xC5, 0xAC, 0x48, 0x25, 0xEE],
};
// IID_IUIAutomationTextPattern {32EFA5BA-1CCB-4BE1-93A0-3BD03B4C53A6}
const IID_IUIAUTOMATION_TEXT_PATTERN: GUID = GUID {
    data1: 0x32EF_A5BA,
    data2: 0x1CCB,
    data3: 0x4BE1,
    data4: [0x93, 0xA0, 0x3B, 0xD0, 0x3B, 0x4C, 0x53, 0xA6],
};

// ── System library declarations ───────────────────────────────────────────────

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, co_init: u32) -> HRESULT;
    fn CoCreateInstance(
        rclsid: *const GUID,
        outer: *mut c_void,
        cls_ctx: u32,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}

#[link(name = "oleaut32")]
extern "system" {
    fn SysFreeString(bstr: BSTR);
}

// ── RAII COM pointer ──────────────────────────────────────────────────────────

struct ComPtr(*mut c_void);

impl ComPtr {
    fn as_raw(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for ComPtr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { com_release(self.0) }
        }
    }
}

// ── Raw COM vtable helpers ────────────────────────────────────────────────────

/// Read the function pointer at `vtable_idx` from `obj`'s vtable.
/// # Safety
/// `obj` must be a valid COM object pointer.
unsafe fn vtable_fn(obj: *mut c_void, vtable_idx: usize) -> *const c_void {
    let vtable = *(obj as *const *const *const c_void);
    *vtable.add(vtable_idx)
}

// IUnknown::QueryInterface (vtable[0])
unsafe fn com_query_interface(obj: *mut c_void, iid: &GUID) -> Option<ComPtr> {
    type F = unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT;
    let f: F = std::mem::transmute(vtable_fn(obj, 0));
    let mut out: *mut c_void = std::ptr::null_mut();
    if f(obj, iid, &mut out) >= 0 && !out.is_null() {
        Some(ComPtr(out))
    } else {
        None
    }
}

// IUnknown::Release (vtable[2])
unsafe fn com_release(obj: *mut c_void) {
    type F = unsafe extern "system" fn(*mut c_void) -> u32;
    let f: F = std::mem::transmute(vtable_fn(obj, 2));
    f(obj);
}

// IUIAutomation::GetFocusedElement (vtable[8])
// Vtable offsets in IUIAutomation: [CompareElements(3), CompareRuntimeIds(4),
// GetRootElement(5), GetElementFromHandle(6), GetElementFromPoint(7),
// GetFocusedElement(8)]
unsafe fn uia_get_focused_element(uia: *mut c_void) -> Option<ComPtr> {
    type F = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT;
    let f: F = std::mem::transmute(vtable_fn(uia, 8));
    let mut out: *mut c_void = std::ptr::null_mut();
    if f(uia, &mut out) >= 0 && !out.is_null() {
        Some(ComPtr(out))
    } else {
        None
    }
}

// IUIAutomationElement::GetCurrentPattern (vtable[16])
// vtable positions: SetFocus(3)..BuildUpdatedCache(9)..GetCachedPropertyValueEx(13)..
//   GetCurrentPatternAs(14)..GetCachedPatternAs(15)..GetCurrentPattern(16)
unsafe fn uia_element_get_pattern(elem: *mut c_void, pattern_id: i32) -> Option<ComPtr> {
    type F = unsafe extern "system" fn(*mut c_void, i32, *mut *mut c_void) -> HRESULT;
    let f: F = std::mem::transmute(vtable_fn(elem, 16));
    let mut out: *mut c_void = std::ptr::null_mut();
    if f(elem, pattern_id, &mut out) >= 0 && !out.is_null() {
        Some(ComPtr(out))
    } else {
        None
    }
}

// IUIAutomationTextPattern::GetSelection (vtable[5])
// vtable positions: RangeFromPoint(3), RangeFromChild(4), GetSelection(5)
unsafe fn uia_text_pattern_get_selection(tp: *mut c_void) -> Option<ComPtr> {
    type F = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT;
    let f: F = std::mem::transmute(vtable_fn(tp, 5));
    let mut out: *mut c_void = std::ptr::null_mut();
    if f(tp, &mut out) >= 0 && !out.is_null() {
        Some(ComPtr(out))
    } else {
        None
    }
}

// IUIAutomationTextRangeArray::get_Length (vtable[3])
unsafe fn uia_range_array_length(arr: *mut c_void) -> i32 {
    type F = unsafe extern "system" fn(*mut c_void, *mut i32) -> HRESULT;
    let f: F = std::mem::transmute(vtable_fn(arr, 3));
    let mut len: i32 = 0;
    f(arr, &mut len);
    len
}

// IUIAutomationTextRangeArray::GetElement (vtable[4])
unsafe fn uia_range_array_get_element(arr: *mut c_void, index: i32) -> Option<ComPtr> {
    type F = unsafe extern "system" fn(*mut c_void, i32, *mut *mut c_void) -> HRESULT;
    let f: F = std::mem::transmute(vtable_fn(arr, 4));
    let mut out: *mut c_void = std::ptr::null_mut();
    if f(arr, index, &mut out) >= 0 && !out.is_null() {
        Some(ComPtr(out))
    } else {
        None
    }
}

// IUIAutomationTextRange::GetText (vtable[12])
// vtable positions: Clone(3)..GetEnclosingElement(11)..GetText(12)
unsafe fn uia_range_get_text(range: *mut c_void) -> Option<String> {
    type F = unsafe extern "system" fn(*mut c_void, i32, *mut BSTR) -> HRESULT;
    let f: F = std::mem::transmute(vtable_fn(range, 12));
    let mut bstr: BSTR = std::ptr::null_mut();
    let hr = f(range, -1 /* no limit */, &mut bstr);
    if hr >= 0 && !bstr.is_null() {
        let text = bstr_to_string(bstr);
        SysFreeString(bstr);
        text
    } else {
        None
    }
}

/// Convert a Windows BSTR to a Rust String.
/// BSTR layout: 4-byte length (in bytes) at bstr[-4], then null-terminated UTF-16.
unsafe fn bstr_to_string(bstr: BSTR) -> Option<String> {
    if bstr.is_null() {
        return None;
    }
    let byte_len = *((bstr as *const u8).sub(4) as *const u32) as usize;
    if byte_len == 0 {
        return Some(String::new());
    }
    let char_len = byte_len / 2;
    let slice = std::slice::from_raw_parts(bstr, char_len);
    Some(String::from_utf16_lossy(slice))
}

// ── UIAutomation selected-text reader ─────────────────────────────────────────

/// Read the selected text in the focused element via UI Automation.
///
/// Returns `Err` when UIA is structurally unavailable for the focused element
/// (no focused element, element lacks TextPattern support) — callers should
/// fall back to clipboard. Returns `Ok(None)` only when TextPattern IS
/// supported and confirms there is no active selection.
fn uia_get_selected_text() -> Result<Option<String>, String> {
    unsafe { uia_get_selected_text_inner() }
}

unsafe fn uia_get_selected_text_inner() -> Result<Option<String>, String> {
    // COINIT_MULTITHREADED = 0x0; ignore already-initialised states.
    let _ = CoInitializeEx(std::ptr::null_mut(), 0x0);

    let mut uia_raw: *mut c_void = std::ptr::null_mut();
    let hr = CoCreateInstance(
        &CLSID_CUIAUTOMATION,
        std::ptr::null_mut(),
        0x1, // CLSCTX_INPROC_SERVER
        &IID_IUIAUTOMATION,
        &mut uia_raw,
    );
    if hr < 0 || uia_raw.is_null() {
        return Err(format!("CoCreateInstance IUIAutomation: HRESULT {hr:#010x}"));
    }
    let uia = ComPtr(uia_raw);

    // Structural failures → Err so the clipboard fallback triggers. These are
    // normal for browsers, Electron apps, and legacy Win32 controls that don't
    // expose IUIAutomationTextPattern.
    let elem = uia_get_focused_element(uia.as_raw())
        .ok_or_else(|| "no focused element".to_string())?;

    // GetCurrentPattern returns IUnknown — QueryInterface to IUIAutomationTextPattern.
    let pattern_unk = uia_element_get_pattern(elem.as_raw(), UIA_TEXT_PATTERN_ID)
        .ok_or_else(|| "element has no TextPattern support".to_string())?;
    let text_pattern = com_query_interface(pattern_unk.as_raw(), &IID_IUIAUTOMATION_TEXT_PATTERN)
        .ok_or_else(|| "QueryInterface for IUIAutomationTextPattern failed".to_string())?;

    // TextPattern IS supported from here — Ok(None) means authoritative "no selection".
    let Some(ranges) = uia_text_pattern_get_selection(text_pattern.as_raw()) else {
        log::debug!("UIA: GetSelection returned null");
        return Ok(None);
    };

    let count = uia_range_array_length(ranges.as_raw());
    if count <= 0 {
        log::debug!("UIA: no selection ranges");
        return Ok(None);
    }

    let mut text = String::new();
    for i in 0..count {
        if let Some(range) = uia_range_array_get_element(ranges.as_raw(), i) {
            if let Some(t) = uia_range_get_text(range.as_raw()) {
                text.push_str(&t);
            }
        }
    }

    Ok(if text.is_empty() { None } else { Some(text) })
}

// ── Key simulation helpers ────────────────────────────────────────────────────

// Windows requires Key::C (virtual key code VK_C) for Ctrl+C, not
// Key::Unicode('c'). enigo's WinAPI backend sends WM_CHAR for Unicode keys,
// which most apps ignore for shortcut handling.
fn win_send_paste(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn win_send_copy(enigo: &mut Enigo) -> Result<(), TextBridgeError> {
    enigo.key(Key::Control, Direction::Press).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::C, Direction::Click).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| TextBridgeError::InputSimulationError(format!("{e:?}")))?;
    Ok(())
}

fn win_release_modifiers(enigo: &mut Enigo) {
    enigo.key(Key::Alt, Direction::Release).ok();
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Control, Direction::Release).ok();
    enigo.key(Key::Meta, Direction::Release).ok();
}

fn windows_fallback_config() -> FallbackConfig {
    FallbackConfig {
        send_paste: win_send_paste,
        send_copy: win_send_copy,
        release_modifiers: win_release_modifiers,
        // Windows clipboard is message-based — no setup delay needed.
        // Brief post-paste wait lets the target app consume the paste message.
        pre_paste_ms: 0,
        post_paste_ms: 150,
        post_restore_ms: 0,
    }
}

// ── TextBridge implementation ─────────────────────────────────────────────────

pub struct WindowsTextBridge;

// Safety: WindowsTextBridge is stateless; COM objects are created per-call.
unsafe impl Send for WindowsTextBridge {}
unsafe impl Sync for WindowsTextBridge {}

impl TextBridge for WindowsTextBridge {
    fn capabilities(&self) -> TextBridgeCapabilities {
        TextBridgeCapabilities {
            can_read_selection_native: true,
            // Clipboard paste is the correct API for insert-at-cursor; UIA
            // ValuePattern replaces the entire field which is wrong for PTT.
            can_insert_native: false,
            can_replace_selection_native: false,
            uses_clipboard_fallback: true,
            requires_accessibility_permission: false,
            wayland_limited: false,
            backend_name: "windows-uia",
        }
    }

    fn get_selected_text(&self) -> Result<Option<String>, TextBridgeError> {
        match uia_get_selected_text() {
            Ok(result) => {
                log::debug!(
                    "UIA selection: {} chars",
                    result.as_deref().map_or(0, str::len)
                );
                Ok(result)
            }
            Err(e) => {
                log::debug!("UIA unavailable ({e}), using clipboard fallback");
                get_selection_via_copy(&windows_fallback_config())
            }
        }
    }

    fn insert_text(&self, text: &str) -> Result<(), TextBridgeError> {
        insert_via_paste(text, &windows_fallback_config())
    }

    fn replace_selection(&self, text: &str) -> Result<(), TextBridgeError> {
        // Clipboard paste naturally replaces any active selection.
        insert_via_paste(text, &windows_fallback_config())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_bridge::TextBridge;

    #[test]
    fn capabilities_are_correct() {
        let bridge = WindowsTextBridge;
        let caps = bridge.capabilities();
        assert_eq!(caps.backend_name, "windows-uia");
        assert!(caps.can_read_selection_native);
        assert!(!caps.requires_accessibility_permission);
        assert!(!caps.wayland_limited);
        assert!(caps.uses_clipboard_fallback);
    }

    #[test]
    fn bstr_empty_is_empty_string() {
        // Test that a null BSTR byte count gives empty string
        // We can't allocate a real BSTR without Windows, but we test the
        // null-pointer branch:
        let result = unsafe { bstr_to_string(std::ptr::null_mut()) };
        assert!(result.is_none());
    }
}
