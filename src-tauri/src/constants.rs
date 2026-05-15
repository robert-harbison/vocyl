/// Default push-to-talk shortcut (overridden by saved settings).
pub const DEFAULT_PTT_SHORTCUT: &str = "CmdOrCtrl+Shift+F8";

/// Default text-to-speech shortcut (overridden by saved settings).
pub const DEFAULT_TTS_SHORTCUT: &str = "CmdOrCtrl+Shift+F9";

/// Width / height of the floating overlay pill (logical pixels).
pub const OVERLAY_W: f64 = 280.0;
pub const OVERLAY_H: f64 = 64.0;

/// Distance from the top of the screen (logical pixels).
pub const OVERLAY_Y: f64 = 24.0;

/// Bump this whenever a breaking change is made to VocylSettings.
pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

// ── Piper binary download URLs (rhasspy/piper 2023.11.14-2) ───────────────────

const PIPER_WIN_URL: &str =
    "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip";

const PIPER_LINUX_X64_URL: &str =
    "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz";

const PIPER_LINUX_ARM64_URL: &str =
    "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_aarch64.tar.gz";

const PIPER_MAC_X64_URL: &str =
    "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_macos_x64.tar.gz";

const PIPER_MAC_ARM64_URL: &str =
    "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_macos_aarch64.tar.gz";

/// Returns the Piper download URL for the current OS and CPU architecture,
/// or `None` if no pre-built binary is available for this platform.
pub fn piper_url() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", _) => Some(PIPER_WIN_URL),
        ("linux", "x86_64") => Some(PIPER_LINUX_X64_URL),
        ("linux", "aarch64") => Some(PIPER_LINUX_ARM64_URL),
        ("macos", "x86_64") => Some(PIPER_MAC_X64_URL),
        ("macos", "aarch64") => Some(PIPER_MAC_ARM64_URL),
        _ => None,
    }
}

// ── Kokoro model package (platform-independent, tar.bz2) ─────────────────────

/// Kokoro English v0.19 model package (tar.bz2, includes model, voices, tokens, espeak-ng-data).
pub const KOKORO_PKG_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-en-v0_19.tar.bz2";

// ── Whisper STT binary (Windows only; Linux/macOS use native whisper-rs) ─────

/// whisper.cpp CLI release — only published for Windows by the upstream project.
/// On Linux/macOS the `whisper-rs` crate is compiled into the binary instead.
#[cfg(target_os = "windows")]
pub const WHISPER_WIN_URL: &str =
    "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.4/whisper-bin-x64.zip";

// ── Sherpa-ONNX legacy binary (Windows x64 only, kept for existing installs) ─

/// Sherpa-ONNX static CLI package (tar.bz2) — legacy; Kokoro now uses the native Rust crate.
pub const SHERPA_TTS_WIN_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.1/sherpa-onnx-v1.13.1-win-x64-static-MT-Release.tar.bz2";
