/// Default push-to-talk shortcut (overridden by saved settings).
/// Alt on Windows/Linux, Option on macOS.
pub const DEFAULT_PTT_SHORTCUT: &str = "Alt+A";

/// Default text-to-speech shortcut (overridden by saved settings).
/// Alt on Windows/Linux, Option on macOS.
pub const DEFAULT_TTS_SHORTCUT: &str = "Alt+S";

/// Width / height of the floating overlay pill (logical pixels).
pub const OVERLAY_W: f64 = 280.0;
pub const OVERLAY_H: f64 = 64.0;

/// Distance from the top of the screen (logical pixels).
pub const OVERLAY_Y: f64 = 24.0;

/// Bump this whenever a breaking change is made to VocylSettings.
pub const SETTINGS_SCHEMA_VERSION: u32 = 2;

// ── Kokoro model package (platform-independent, tar.bz2) ─────────────────────
//
// NOTICE: The Kokoro model package includes espeak-ng phonemizer data files
// (espeak-ng-data/), which are licensed under GPL-3.0-or-later.
// The model weights, voice embeddings, and token list are Apache-2.0.
// Source for espeak-ng: https://github.com/espeak-ng/espeak-ng
// These files are downloaded at the user's request and are NOT compiled into
// or linked with the Vocyl binary.

/// Kokoro English v0.19 model package (tar.bz2, includes model, voices, tokens, espeak-ng-data).
pub const KOKORO_PKG_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-en-v0_19.tar.bz2";


