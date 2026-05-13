use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tauri::Manager;

use enigo::Keyboard;

use crate::{
    models::{get_kokoro_model_dir, get_piper_bin, get_sherpa_tts_bin, model_dir},
    settings::SettingsCache,
    stt::emit_overlay_state,
};

// ── Managed state ─────────────────────────────────────────────────────────────

pub struct TtsState {
    pub is_playing: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
}

impl TtsState {
    pub fn new() -> Self {
        Self {
            is_playing: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ── Get selected text ─────────────────────────────────────────────────────────

/// Simulate Ctrl+C to copy the current selection, read the clipboard, then
/// restore the previous clipboard content.
pub fn get_selected_text() -> Result<String, String> {
    use arboard::Clipboard;
    use enigo::{Direction, Enigo, Key, Settings};

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let saved = clipboard.get_text().ok();

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create input handler: {e:?}"))?;

    // Release Shift/Alt that may still be physically held from the global
    // shortcut combo (e.g. Ctrl+Shift+F9). Without this the simulated Ctrl+C
    // would arrive as Ctrl+Shift+C which most apps do not treat as "copy".
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();

    // Brief pause so modifier-release events settle before we send Ctrl+C
    std::thread::sleep(Duration::from_millis(50));

    enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::C, Direction::Click).map_err(|e| format!("{e:?}"))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;

    // Give the target app time to update the clipboard
    std::thread::sleep(Duration::from_millis(200));

    let selected = clipboard.get_text().unwrap_or_default();

    // Restore old clipboard content
    if let Some(old) = saved {
        clipboard.set_text(old).ok();
    }

    Ok(selected)
}

// ── TTS speak ─────────────────────────────────────────────────────────────────

/// Start speaking the given text using the engine configured in settings.
/// Emits `overlay-state` events and hides the overlay when done.
pub fn start_speaking<R: tauri::Runtime>(app: &tauri::AppHandle<R>, text: String) {
    let tts = app.state::<TtsState>();
    tts.is_playing.store(true, Ordering::SeqCst);
    tts.stop_flag.store(false, Ordering::SeqCst);

    let stop_flag = tts.stop_flag.clone();
    let is_playing = tts.is_playing.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let settings = app_handle
            .state::<SettingsCache>()
            .0
            .lock()
            .unwrap()
            .clone();

        let result = match settings.tts_engine.as_str() {
            "piper" => speak_piper(&app_handle, &text, &settings, stop_flag.clone()).await,
            "kokoro" => speak_kokoro(&app_handle, &text, &settings, stop_flag.clone()).await,
            _ => Err(format!("Unknown TTS engine: {}", settings.tts_engine)),
        };

        if let Err(e) = result {
            eprintln!("[vocyl] TTS error: {e}");
            emit_overlay_state(&app_handle, "error");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        is_playing.store(false, Ordering::SeqCst);

        if let Some(overlay) = app_handle.get_webview_window("overlay") {
            let _ = overlay.hide();
        }
    });
}

/// Stop ongoing TTS playback.
pub fn stop_speaking<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let tts = app.state::<TtsState>();
    tts.stop_flag.store(true, Ordering::SeqCst);
}

// ── Piper subprocess ──────────────────────────────────────────────────────────

async fn speak_piper<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    text: &str,
    settings: &crate::settings::VocylSettings,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let piper_bin = get_piper_bin(app)?;

    let voice_id = settings
        .active_tts_model_id
        .as_deref()
        .ok_or("No TTS voice selected. Choose one in Settings → Text-to-Speech.")?;

    let model_filename = format!("{voice_id}.onnx");
    let model_path = model_dir(app, "tts")?.join(&model_filename);

    if !model_path.exists() {
        return Err(format!(
            "Piper voice model '{model_filename}' not found. Download it in Settings → Text-to-Speech."
        ));
    }

    // Write to a temp WAV file via piper
    let tmp_wav = std::env::temp_dir().join(format!(
        "vocyl_tts_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    // Write text to a temp input file (avoids stdin encoding issues on Windows)
    let tmp_txt = tmp_wav.with_extension("txt");
    std::fs::write(&tmp_txt, text).map_err(|e| e.to_string())?;

    let mut child = tokio::process::Command::new(&piper_bin)
        .args([
            "--model",
            model_path.to_str().unwrap_or_default(),
            "--output-file",
            tmp_wav.to_str().unwrap_or_default(),
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start piper: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(text.as_bytes()).await.map_err(|e| e.to_string())?;
        stdin.shutdown().await.ok();
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;

    std::fs::remove_file(&tmp_txt).ok();

    if !status.success() {
        return Err("Piper synthesis failed".to_string());
    }

    if !tmp_wav.exists() {
        return Err("Piper did not produce audio output".to_string());
    }

    // Play the WAV file on a blocking thread
    let output_device = settings.output_device_id.clone();
    let wav_path = tmp_wav.clone();

    tokio::task::spawn_blocking(move || {
        let result = play_wav(&wav_path, output_device, stop_flag);
        std::fs::remove_file(&wav_path).ok();
        result
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(())
}

// ── Kokoro subprocess (sherpa-onnx) ───────────────────────────────────────────

/// Maps a Kokoro voice name to the speaker ID used by kokoro-en-v0_19.
fn kokoro_sid(voice: &str) -> u32 {
    match voice {
        "af" | "af_heart" => 0,
        "af_bella" => 1,
        "af_nicole" => 2,
        "af_sarah" => 3,
        "af_sky" => 4,
        "am_adam" => 5,
        "am_michael" => 6,
        "bf_emma" => 7,
        "bf_isabella" => 8,
        "bm_george" => 9,
        "bm_lewis" => 10,
        _ => 0,
    }
}

async fn speak_kokoro<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    text: &str,
    settings: &crate::settings::VocylSettings,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let sherpa_bin = get_sherpa_tts_bin(app)?;
    let model_dir = get_kokoro_model_dir(app)?;

    let voice = settings.tts_voice.as_deref().unwrap_or("af");
    let sid = kokoro_sid(voice);

    let tmp_wav = std::env::temp_dir().join(format!(
        "vocyl_tts_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    // Build args
    let model_arg = format!("--kokoro-model={}", model_dir.join("model.onnx").to_str().unwrap_or_default());
    let voices_arg = format!("--kokoro-voices={}", model_dir.join("voices.bin").to_str().unwrap_or_default());
    let tokens_arg = format!("--kokoro-tokens={}", model_dir.join("tokens.txt").to_str().unwrap_or_default());
    let data_arg = format!("--kokoro-data-dir={}", model_dir.join("espeak-ng-data").to_str().unwrap_or_default());
    let sid_arg = format!("--sid={sid}");
    let out_arg = format!("--output-filename={}", tmp_wav.to_str().unwrap_or_default());

    let mut args = vec![
        "--debug=0".to_string(),
        model_arg,
        voices_arg,
        tokens_arg,
        data_arg,
        "--num-threads=2".to_string(),
        sid_arg,
        out_arg,
    ];

    // Include lexicon files if present
    if let Ok(entries) = std::fs::read_dir(&model_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("lexicon") && name.ends_with(".txt") {
                args.push(format!("--kokoro-lexicon={}", entry.path().to_str().unwrap_or_default()));
            }
        }
    }

    args.push(text.to_string());

    let mut cmd = tokio::process::Command::new(&sherpa_bin);
    cmd.args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    // Prevent the sherpa-onnx GUI window from appearing on Windows
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW

    let status = cmd
        .status()
        .await
        .map_err(|e| format!("Failed to start sherpa-onnx: {e}"))?;

    if !status.success() {
        return Err("Kokoro synthesis failed. Check that the model was downloaded correctly.".to_string());
    }

    if !tmp_wav.exists() {
        return Err("Sherpa-ONNX did not produce audio output.".to_string());
    }

    let output_device = settings.output_device_id.clone();
    let wav_path = tmp_wav.clone();

    tokio::task::spawn_blocking(move || {
        let result = play_wav(&wav_path, output_device, stop_flag);
        std::fs::remove_file(&wav_path).ok();
        result
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(())
}

// ── WAV playback ──────────────────────────────────────────────────────────────

fn play_wav(
    path: &PathBuf,
    device_name: Option<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();

    let device = match device_name.as_deref() {
        Some(name) => host
            .output_devices()
            .map_err(|e| e.to_string())?
            .find(|d| d.name().ok().as_deref() == Some(name))
            .unwrap_or_else(|| host.default_output_device().expect("no output device")),
        None => host
            .default_output_device()
            .ok_or("No output device found")?,
    };

    let (_stream, handle) =
        rodio::OutputStream::try_from_device(&device).map_err(|e| e.to_string())?;

    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let source = rodio::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| e.to_string())?;

    let sink = rodio::Sink::try_new(&handle).map_err(|e| e.to_string())?;
    sink.append(source);

    while !sink.empty() && !stop_flag.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(50));
    }

    sink.stop();
    Ok(())
}
