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

// ── Piper binary download URLs ────────────────────────────────────────────────
//
// ⚠️  LICENSE NOTICE — GPL-3.0-or-later
//
// Piper TTS is licensed under the GNU General Public License v3.0 or later
// (GPL-3.0-or-later), available at: https://www.gnu.org/licenses/gpl-3.0.html
// Source repository: https://github.com/OHF-Voice/piper1-gpl
//
// Vocyl downloads the Piper binary at the USER'S EXPLICIT REQUEST and runs it
// as a COMPLETELY SEPARATE OS PROCESS.  Vocyl does not link against, embed, or
// distribute any part of Piper.  Communication occurs only through stdin and
// temporary WAV files.  Vocyl's own Apache-2.0 license is therefore unaffected.
//
// TODO(VOC-12): Update these URLs to OHF-Voice/piper1-gpl release artifacts before
// public release.  Check https://github.com/OHF-Voice/piper1-gpl/releases
// for the correct version tag and filenames, then replace the rhasspy URLs below.
// This is a BREAKING CHANGE — the new repo uses a different archive layout.

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
///
/// The returned URL points to the OHF-Voice/piper1-gpl release (GPL-3.0).
/// The binary is run as a separate subprocess; it is never linked into Vocyl.
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

// ── Whisper STT binary (Windows only; Linux/macOS use native whisper-rs) ─────
//
// whisper.cpp is MIT licensed. Source: https://github.com/ggml-org/whisper.cpp
// The pre-built whisper-cli.exe is downloaded at the user's request and run as
// a completely separate subprocess. It is NOT compiled into or linked with Vocyl.

/// whisper.cpp CLI release — only published for Windows by the upstream project.
/// On Linux/macOS the `whisper-rs` crate is compiled into the binary instead.
#[cfg(target_os = "windows")]
pub const WHISPER_WIN_URL: &str =
    "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.4/whisper-bin-x64.zip";

// ── Sherpa-ONNX legacy binary (Windows x64 only, kept for existing installs) ─

/// Sherpa-ONNX static CLI package (tar.bz2) — legacy; Kokoro now uses the native Rust crate.
pub const SHERPA_TTS_WIN_URL: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.1/sherpa-onnx-v1.13.1-win-x64-static-MT-Release.tar.bz2";
