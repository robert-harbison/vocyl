use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use tauri::Manager;

use crate::{
    models::get_kokoro_model_dir,
    settings::SettingsCache,
    stt::emit_overlay_state,
};

// ── Kokoro engine cache ───────────────────────────────────────────────────────

struct KokoroHandle {
    tts: sherpa_onnx::OfflineTts,
    /// The model dir this handle was built from. Rebuilt if settings change.
    model_dir: PathBuf,
}

// ── Managed state ─────────────────────────────────────────────────────────────

pub struct TtsState {
    pub is_playing: Arc<AtomicBool>,
    pub stop_flag: Arc<AtomicBool>,
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

// ── TTS speak ─────────────────────────────────────────────────────────────────

/// Start speaking `text` using the Kokoro engine.
pub fn start_speaking<R: tauri::Runtime>(app: &tauri::AppHandle<R>, text: String) {
    log::info!("TTS: starting synthesis ({} chars)", text.len());
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

        let result = speak_kokoro(&app_handle, &text, &settings, stop_flag).await;

        if let Err(e) = result {
            log::error!("TTS: synthesis error: {}", e);
            emit_overlay_state(&app_handle, "error");
            tokio::time::sleep(Duration::from_secs(2)).await;
        } else {
            log::info!("TTS: synthesis complete");
        }

        is_playing.store(false, Ordering::SeqCst);

        if let Some(overlay) = app_handle.get_webview_window("overlay") {
            let _ = overlay.hide();
        }
    });
}

/// Stop ongoing TTS playback.
pub fn stop_speaking<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    log::info!("TTS: stop requested");
    app.state::<TtsState>().stop_flag.store(true, Ordering::SeqCst);
}

/// Pre-warm the Kokoro engine so the first call doesn't pay the model-load cost.
pub fn prewarm_kokoro<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let app = app.clone();
    std::thread::spawn(move || {
        let model_dir = match get_kokoro_model_dir(&app) {
            Ok(d) => d,
            Err(e) => {
                log::debug!("TTS: pre-warm skipped — Kokoro not yet installed: {}", e);
                return;
            }
        };
        let kokoro_arc = app.state::<TtsState>().kokoro.clone();
        let mut guard = kokoro_arc.lock().unwrap();
        if guard.is_some() {
            log::debug!("TTS: Kokoro already loaded, skipping pre-warm");
            return;
        }
        log::info!("TTS: pre-warming Kokoro model…");
        if let Ok(tts) = create_kokoro_engine(&model_dir, num_threads()) {
            *guard = Some(KokoroHandle { tts, model_dir });
            log::info!("TTS: Kokoro pre-warm complete");
        } else {
            log::warn!("TTS: Kokoro pre-warm failed — model loads on first use instead");
        }
    });
}

// ── Shared audio output helper ────────────────────────────────────────────────

fn resolve_output_device(name: Option<&str>) -> Result<cpal::Device, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let device = match name {
        Some(n) => {
            let found = host
                .output_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().ok().as_deref() == Some(n));
            match found {
                Some(d) => d,
                None => {
                    log::warn!("TTS: output device '{}' not found, using default", n);
                    host.default_output_device()
                        .ok_or_else(|| "No output device found".to_string())?
                }
            }
        }
        None => host
            .default_output_device()
            .ok_or_else(|| "No output device found".to_string())?,
    };
    Ok(device)
}

// ── Sentence splitting ────────────────────────────────────────────────────────

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

    sentences
        .into_iter()
        .flat_map(|s| if s.len() > 200 { split_at_clause(&s) } else { vec![s] })
        .filter(|s| !s.trim().is_empty())
        .collect()
}

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

// ── Kokoro native engine (sherpa-onnx) ────────────────────────────────────────

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
        max_num_sentences: 1,
        ..Default::default()
    }
}

fn create_kokoro_engine(
    model_dir: &PathBuf,
    num_threads: i32,
) -> Result<sherpa_onnx::OfflineTts, String> {
    // On Windows, try DirectML (GPU acceleration) first, then fall back to CPU.
    #[cfg(target_os = "windows")]
    if let Some(tts) = sherpa_onnx::OfflineTts::create(
        &build_kokoro_config(model_dir, num_threads, "directml"),
    ) {
        log::info!("TTS: Kokoro engine initialised on DirectML (GPU)");
        return Ok(tts);
    }

    #[cfg(target_os = "windows")]
    log::warn!("TTS: DirectML unavailable, falling back to CPU");

    sherpa_onnx::OfflineTts::create(&build_kokoro_config(model_dir, num_threads, "cpu"))
        .ok_or_else(|| {
            let msg = "Failed to initialise Kokoro TTS engine. Check the model files.".to_string();
            log::error!("TTS: {}", msg);
            msg
        })
}

async fn speak_kokoro<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    text: &str,
    settings: &crate::settings::VocylSettings,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let model_dir = get_kokoro_model_dir(app)?;
    let voice = settings.tts_voice.as_deref().unwrap_or("af");
    let sid = kokoro_sid(voice);
    log::info!("TTS: Kokoro voice '{}' (sid={})", voice, sid);
    let output_device = settings.output_device_id.clone();
    let kokoro_arc = app.state::<TtsState>().kokoro.clone();
    let n_threads = num_threads();

    let sentences = split_sentences(text);
    if sentences.is_empty() {
        log::info!("TTS: no sentences to speak");
        return Ok(());
    }
    log::info!("TTS: synthesising {} sentence(s)", sentences.len());

    let (tx, rx) = tokio::sync::mpsc::channel::<(Vec<f32>, u32)>(2);
    let stop_synth = stop_flag.clone();
    let stop_play = stop_flag.clone();

    // ── Synthesiser ───────────────────────────────────────────────────────────
    let synth_task = tokio::task::spawn_blocking(move || -> Result<(), String> {
        {
            let mut guard = kokoro_arc.lock().unwrap();
            let needs_reload = guard
                .as_ref()
                .map(|h| h.model_dir != model_dir)
                .unwrap_or(true);
            if needs_reload {
                log::info!("TTS: loading Kokoro model…");
                let tts = create_kokoro_engine(&model_dir, n_threads)?;
                *guard = Some(KokoroHandle { tts, model_dir: model_dir.clone() });
                log::info!("TTS: Kokoro model loaded and cached");
            }
        }

        let gen_config = sherpa_onnx::GenerationConfig {
            sid,
            speed: 1.0,
            ..Default::default()
        };

        for sentence in sentences {
            if stop_synth.load(Ordering::SeqCst) {
                log::info!("TTS: synthesis cancelled");
                break;
            }

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
                        Some(move |_samples: &[f32], _progress: f32| -> bool {
                            !stop_cb.load(Ordering::SeqCst)
                        }),
                    )
                    .map(|a| (a.samples().to_vec(), a.sample_rate() as u32))
            };

            if let Some(data) = audio_data {
                if tx.blocking_send(data).is_err() {
                    log::info!("TTS: player channel closed, stopping synthesis");
                    break;
                }
            }
        }
        Ok(())
    });

    // ── Player ────────────────────────────────────────────────────────────────
    let play_task = tokio::task::spawn_blocking(move || -> Result<(), String> {
        use rodio::buffer::SamplesBuffer;

        let device = resolve_output_device(output_device.as_deref())?;
        let (_stream, handle) =
            rodio::OutputStream::try_from_device(&device).map_err(|e| e.to_string())?;
        let sink = rodio::Sink::try_new(&handle).map_err(|e| e.to_string())?;

        let mut rx = rx;
        while let Some((samples, sample_rate)) = rx.blocking_recv() {
            if stop_play.load(Ordering::SeqCst) {
                log::info!("TTS: playback cancelled");
                break;
            }
            sink.append(SamplesBuffer::new(1, sample_rate, samples));
        }

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

// ── Utilities ─────────────────────────────────────────────────────────────────

fn num_threads() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4) as i32
}
