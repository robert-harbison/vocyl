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
pub const SETTINGS_SCHEMA_VERSION: u32 = 2;

// ── Binary download URLs (Windows x64) ───────────────────────────────────────

pub const WHISPER_WIN_URL: &str =
    "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.4/whisper-bin-x64.zip";

pub const PIPER_WIN_URL: &str =
    "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip";

/// Sherpa-ONNX static CLI package (tar.bz2) — contains sherpa-onnx-offline-tts.exe (headless CLI).
pub const SHERPA_TTS_WIN_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.1/sherpa-onnx-v1.13.1-win-x64-static-MT-Release.tar.bz2";

/// Kokoro English v0.19 model package (tar.bz2, includes model, voices, tokens, espeak-ng-data).
pub const KOKORO_PKG_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-en-v0_19.tar.bz2";
