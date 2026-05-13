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
    models::{get_kokoro_model_dir, get_piper_bin, model_dir},
    settings::SettingsCache,
    stt::emit_overlay_state,
};

// ── Kokoro engine cache ───────────────────────────────────────────────────────

/// Holds the loaded Kokoro TTS engine so the ONNX model is only read from
/// disk once. Subsequent TTS calls skip model loading entirely and go straight
/// to inference (~0.5–1 s for a sentence vs ~3–5 s cold-start).
///
/// `sherpa_onnx::OfflineTts` already implements `Send`, so this struct is
/// `Send + Sync` automatically and safe to hold in Tauri managed state.
struct KokoroHandle {
    tts: sherpa_onnx::OfflineTts,
    /// The model dir this handle was built from. If settings point to a
    /// different directory the handle is rebuilt on next use.
    model_dir: PathBuf,
}

// ── Managed state ─────────────────────────────────────────────────────────────

pub struct TtsState {
    pub is_playing: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
    /// Cached Kokoro engine — `None` until first use, then kept alive for the
    /// lifetime of the app (or until the model directory changes).
    kokoro: Arc<std::sync::Mutex<Option<KokoroHandle>>>,
}

impl TtsState {
    pub fn new() -> Self {
        Self {
            is_playing: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            kokoro: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

// ── Get selected text ─────────────────────────────────────────────────────────

/// Simulate the platform copy shortcut on the current selection, read the
/// clipboard, then restore the previous clipboard content.
pub fn get_selected_text() -> Result<String, String> {
    use arboard::Clipboard;
    use enigo::{Direction, Enigo, Key, Settings};

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let saved = clipboard.get_text().ok();

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create input handler: {e:?}"))?;

    // Release modifiers that may still be held from the global shortcut combo.
    enigo.key(Key::Shift, Direction::Release).ok();
    enigo.key(Key::Alt, Direction::Release).ok();
    #[cfg(target_os = "macos")]
    enigo.key(Key::Meta, Direction::Release).ok();

    // Brief pause so modifier-release events settle before we send the copy key.
    std::thread::sleep(Duration::from_millis(50));

    // macOS uses Cmd+C; Linux and Windows use Ctrl+C.
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::C, Direction::Click).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("{e:?}"))?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::C, Direction::Click).map_err(|e| format!("{e:?}"))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("{e:?}"))?;
    }

    // Give the target app time to update the clipboard.
    std::thread::sleep(Duration::from_millis(200));

    let selected = clipboard.get_text().unwrap_or_default();

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

/// Pre-warm the Kokoro engine in the background so the first TTS call is fast.
/// Call this once from app setup after managing `TtsState`.
pub fn prewarm_kokoro<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let app = app.clone();
    std::thread::spawn(move || {
        let settings = app.state::<SettingsCache>().0.lock().unwrap().clone();
        if settings.tts_engine != "kokoro" {
            return;
        }
        let model_dir = match get_kokoro_model_dir(&app) {
            Ok(d) => d,
            Err(_) => return, // model not installed yet — skip
        };
        let kokoro_arc = app.state::<TtsState>().kokoro.clone();
        let mut guard = kokoro_arc.lock().unwrap();
        if guard.is_some() {
            return; // already loaded
        }
        eprintln!("[vocyl] Pre-warming Kokoro model…");
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get().min(8))
            .unwrap_or(4) as i32;
        if let Ok(tts) = create_kokoro_engine(&model_dir, num_threads) {
            *guard = Some(KokoroHandle { tts, model_dir });
            eprintln!("[vocyl] Kokoro pre-warm complete.");
        }
    });
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

    let tmp_wav = std::env::temp_dir().join(format!(
        "vocyl_tts_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

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

// ── Sentence splitting ────────────────────────────────────────────────────────

/// Split text into synthesis-sized chunks.
/// Primary splits at sentence endings; long chunks are further divided at
/// commas/semicolons so no single chunk causes a multi-second wait.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?' | '\n') {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    // Further split any chunk that is very long (no sentence punctuation)
    sentences
        .into_iter()
        .flat_map(|s| {
            if s.len() > 200 {
                split_at_clause(&s)
            } else {
                vec![s]
            }
        })
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// Split a long string at clause boundaries (comma, semicolon, colon).
fn split_at_clause(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, ',' | ';' | ':') && current.len() > 80 {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                chunks.push(trimmed);
            }
            current.clear();
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        chunks.push(trimmed);
    }
    chunks
}

// ── Kokoro native engine (sherpa-onnx Rust crate) ─────────────────────────────

/// Maps a Kokoro voice name to the speaker ID used by kokoro-en-v0_19.
fn kokoro_sid(voice: &str) -> i32 {
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

/// Build an [`OfflineTtsConfig`] for the Kokoro model at `model_dir`.
fn build_kokoro_config(
    model_dir: &PathBuf,
    num_threads: i32,
    provider: &str,
) -> sherpa_onnx::OfflineTtsConfig {
    sherpa_onnx::OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            kokoro: sherpa_onnx::OfflineTtsKokoroModelConfig {
                model: Some(model_dir.join("model.onnx").to_string_lossy().to_string()),
                voices: Some(model_dir.join("voices.bin").to_string_lossy().to_string()),
                tokens: Some(model_dir.join("tokens.txt").to_string_lossy().to_string()),
                data_dir: Some(
                    model_dir.join("espeak-ng-data").to_string_lossy().to_string(),
                ),
                ..Default::default()
            },
            num_threads,
            debug: false,
            provider: Some(provider.to_string()),
            ..Default::default()
        },
        // Kokoro only supports max_num_sentences = 1.
        max_num_sentences: 1,
        ..Default::default()
    }
}

/// Create an [`OfflineTts`] engine.
/// On Windows, DirectML (GPU) is tried first and falls back to CPU.
/// On macOS and Linux, CPU is used directly.
fn create_kokoro_engine(
    model_dir: &PathBuf,
    num_threads: i32,
) -> Result<sherpa_onnx::OfflineTts, String> {
    #[cfg(target_os = "windows")]
    {
        if let Some(tts) = sherpa_onnx::OfflineTts::create(
            &build_kokoro_config(model_dir, num_threads, "directml"),
        ) {
            return Ok(tts);
        }
        eprintln!("[vocyl] DirectML unavailable, falling back to CPU.");
    }

    sherpa_onnx::OfflineTts::create(&build_kokoro_config(model_dir, num_threads, "cpu"))
        .ok_or_else(|| "Failed to initialise Kokoro TTS engine. Check the model files.".to_string())
}

async fn speak_kokoro<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    text: &str,
    settings: &crate::settings::VocylSettings,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let model_dir = get_kokoro_model_dir(app)?;
    let sid = kokoro_sid(settings.tts_voice.as_deref().unwrap_or("af"));
    let output_device = settings.output_device_id.clone();
    let kokoro_arc = app.state::<TtsState>().kokoro.clone();

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4) as i32;

    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return Ok(());
    }

    // Channel capacity 2: synthesizer stays one sentence ahead of the player.
    let (tx, rx) = tokio::sync::mpsc::channel::<(Vec<f32>, u32)>(2);

    let stop_synth = stop_flag.clone();
    let stop_play = stop_flag.clone();

    // ── Synthesizer ───────────────────────────────────────────────────────────
    // Runs on the blocking thread pool. Ensures the model is loaded, then
    // synthesizes each sentence and sends (samples, sample_rate) to the player.
    let synth_task = tokio::task::spawn_blocking(move || -> Result<(), String> {
        // Load / validate cached model (mutex held only for this block).
        {
            let mut guard = kokoro_arc.lock().unwrap();
            let needs_reload = guard
                .as_ref()
                .map(|h| h.model_dir != model_dir)
                .unwrap_or(true);
            if needs_reload {
                eprintln!("[vocyl] Loading Kokoro model (one-time cost)…");
                let tts = create_kokoro_engine(&model_dir, num_threads)?;
                *guard = Some(KokoroHandle { tts, model_dir: model_dir.clone() });
                eprintln!("[vocyl] Kokoro model loaded and cached.");
            }
        }

        let gen_config = sherpa_onnx::GenerationConfig {
            sid,
            speed: 1.0,
            ..Default::default()
        };

        for sentence in sentences {
            if stop_synth.load(Ordering::SeqCst) {
                break;
            }

            // Lock only during inference so the mutex isn't held while the
            // channel send blocks (waiting for the player to consume).
            let audio_data = {
                let guard = kokoro_arc.lock().unwrap();
                let stop_cb = stop_synth.clone();
                guard
                    .as_ref()
                    .unwrap()
                    .tts
                    .generate_with_config(
                        &sentence,
                        &gen_config,
                        // Callback lets the user cancel synthesis mid-sentence.
                        Some(move |_samples: &[f32], _progress: f32| -> bool {
                            !stop_cb.load(Ordering::SeqCst)
                        }),
                    )
                    .map(|a| (a.samples().to_vec(), a.sample_rate() as u32))
            };

            if let Some(data) = audio_data {
                if tx.blocking_send(data).is_err() {
                    break; // player dropped — user stopped
                }
            }
        }
        Ok(())
    });

    // ── Player ────────────────────────────────────────────────────────────────
    // Opens the audio device once, then appends each sentence to the sink as
    // it arrives — rodio plays them gaplessly in sequence.
    let play_task = tokio::task::spawn_blocking(move || -> Result<(), String> {
        use cpal::traits::{DeviceTrait, HostTrait};
        use rodio::buffer::SamplesBuffer;

        let host = cpal::default_host();
        let device = match output_device.as_deref() {
            Some(name) => host
                .output_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().ok().as_deref() == Some(name))
                .unwrap_or_else(|| host.default_output_device().expect("no output device")),
            None => host.default_output_device().ok_or("No output device found")?,
        };

        let (_stream, handle) =
            rodio::OutputStream::try_from_device(&device).map_err(|e| e.to_string())?;
        let sink = rodio::Sink::try_new(&handle).map_err(|e| e.to_string())?;

        let mut rx = rx;
        while let Some((samples, sample_rate)) = rx.blocking_recv() {
            if stop_play.load(Ordering::SeqCst) {
                break;
            }
            // Append each sentence — rodio queues them for seamless playback.
            sink.append(SamplesBuffer::new(1, sample_rate, samples));
        }

        // Drain whatever is still queued in the sink.
        while !sink.empty() && !stop_play.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(50));
        }
        sink.stop();
        Ok(())
    });

    let (s, p) = tokio::join!(synth_task, play_task);
    s.map_err(|e| e.to_string())??;
    p.map_err(|e| e.to_string())??;
    Ok(())
}

// ── Audio playback ────────────────────────────────────────────────────────────

/// Play a WAV file from disk (used by the Piper engine).
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
        None => host.default_output_device().ok_or("No output device found")?,
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
