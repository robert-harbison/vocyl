use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use tauri::{Emitter, Manager};

use crate::{
    models::model_dir,
    settings::SettingsCache,
};

// ── Managed state ─────────────────────────────────────────────────────────────

pub struct SttState {
    pub is_recording: Arc<AtomicBool>,
    pub samples: Arc<Mutex<Vec<f32>>>,
    pub sample_rate: Arc<Mutex<u32>>,
    pub channels: Arc<Mutex<u16>>,
}

impl SttState {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(16000)),
            channels: Arc::new(Mutex::new(1)),
        }
    }
}

// ── Recording ─────────────────────────────────────────────────────────────────

/// Start recording on a background thread. Returns immediately.
/// Emits `overlay-state: "listening"` on success.
pub fn start_recording<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    device_name: Option<String>,
) {
    log::info!("STT: starting recording (device={:?})", device_name);
    let is_recording = app.state::<SttState>().is_recording.clone();
    let samples = app.state::<SttState>().samples.clone();
    let sample_rate_cell = app.state::<SttState>().sample_rate.clone();
    let channels_cell = app.state::<SttState>().channels.clone();
    let app_handle = app.clone();

    std::thread::spawn(move || {
        let host = cpal::default_host();

        let device = match device_name.as_deref() {
            Some(name) => {
                let found = host
                    .input_devices()
                    .ok()
                    .and_then(|mut devices| devices.find(|d| d.name().ok().as_deref() == Some(name)));
                match found {
                    Some(d) => {
                        log::info!("STT: using requested input device '{}'", name);
                        d
                    }
                    None => {
                        log::warn!("STT: requested device '{}' not found, falling back to default", name);
                        match host.default_input_device() {
                            Some(d) => d,
                            None => {
                                log::error!("STT: no input device available");
                                emit_overlay_state(&app_handle, "error");
                                return;
                            }
                        }
                    }
                }
            }
            None => match host.default_input_device() {
                Some(d) => {
                    log::info!("STT: using default input device");
                    d
                }
                None => {
                    log::error!("STT: no default input device found");
                    emit_overlay_state(&app_handle, "error");
                    return;
                }
            },
        };

        let config = match device.default_input_config() {
            Ok(c) => {
                log::info!(
                    "STT: input config - sample_rate={} channels={} format={:?}",
                    c.sample_rate().0,
                    c.channels(),
                    c.sample_format()
                );
                c
            }
            Err(e) => {
                log::error!("STT: failed to get input config: {}", e);
                emit_overlay_state(&app_handle, "error");
                return;
            }
        };

        *sample_rate_cell.lock().unwrap() = config.sample_rate().0;
        *channels_cell.lock().unwrap() = config.channels();
        samples.lock().unwrap().clear();
        is_recording.store(true, Ordering::SeqCst);

        let sample_fmt = config.sample_format();
        let stream_cfg: cpal::StreamConfig = config.into();

        let samples_f32 = samples.clone();
        let samples_i16 = samples.clone();
        let is_rec_f32 = is_recording.clone();
        let is_rec_i16 = is_recording.clone();
        let is_rec_loop = is_recording.clone();

        let err_fn = |e: cpal::StreamError| log::error!("STT: audio stream error: {:?}", e);

        let stream = match sample_fmt {
            SampleFormat::F32 => device.build_input_stream(
                &stream_cfg,
                move |data: &[f32], _| {
                    if is_rec_f32.load(Ordering::SeqCst) {
                        samples_f32.lock().unwrap().extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &stream_cfg,
                move |data: &[i16], _| {
                    if is_rec_i16.load(Ordering::SeqCst) {
                        let converted: Vec<f32> =
                            data.iter().map(|&s| s as f32 / 32768.0).collect();
                        samples_i16.lock().unwrap().extend(converted);
                    }
                },
                err_fn,
                None,
            ),
            _ => {
                log::error!("STT: unsupported sample format {:?}", sample_fmt);
                emit_overlay_state(&app_handle, "error");
                return;
            }
        };

        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                log::error!("STT: failed to build input stream: {:?}", e);
                emit_overlay_state(&app_handle, "error");
                return;
            }
        };

        if stream.play().is_err() {
            log::error!("STT: failed to start audio stream");
            emit_overlay_state(&app_handle, "error");
            return;
        }

        log::info!("STT: recording started");
        emit_overlay_state(&app_handle, "listening");

        while is_rec_loop.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(10));
        }

        drop(stream);
        log::info!("STT: recording stopped");
    });
}

// ── Transcription ─────────────────────────────────────────────────────────────

/// Stop recording and transcribe using the native whisper-rs library.
pub async fn stop_and_transcribe<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<String, String> {
    log::info!("STT: stopping recording and starting transcription");
    app.state::<SttState>()
        .is_recording
        .store(false, Ordering::SeqCst);

    // Give the recording thread a moment to finish writing samples.
    tokio::time::sleep(Duration::from_millis(80)).await;

    let samples = app.state::<SttState>().samples.lock().unwrap().clone();
    let sample_rate = *app.state::<SttState>().sample_rate.lock().unwrap();
    let channels = *app.state::<SttState>().channels.lock().unwrap();

    log::info!(
        "STT: captured {} samples at {} Hz ({} channel(s))",
        samples.len(),
        sample_rate,
        channels
    );

    if samples.is_empty() {
        log::info!("STT: no samples captured, skipping transcription");
        return Ok(String::new());
    }

    let model_size = app
        .state::<SettingsCache>()
        .0
        .lock()
        .unwrap()
        .active_stt_model
        .clone()
        .ok_or_else(|| {
            log::warn!("STT: no model selected");
            "No STT model selected. Choose one in Settings → Speech-to-Text.".to_string()
        })?;

    let model_path = model_dir(app, "stt")?.join(format!("ggml-{model_size}.bin"));
    if !model_path.exists() {
        log::error!("STT: model file not found: {}", model_path.display());
        return Err(format!(
            "STT model 'ggml-{model_size}.bin' not found. Download it in Settings → Speech-to-Text."
        ));
    }

    log::info!("STT: transcribing with model '{}'", model_size);
    let result = transcribe(model_path, samples, sample_rate, channels).await;
    match &result {
        Ok(text) => log::info!("STT: transcription complete ({} chars)", text.len()),
        Err(e) => log::error!("STT: transcription failed: {}", e),
    }
    result
}

// ── whisper-rs inference ──────────────────────────────────────────────────────

async fn transcribe(
    model_path: std::path::PathBuf,
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        let audio = prepare_audio_for_whisper(samples, sample_rate, channels);

        let ctx = whisper_rs::WhisperContext::new_with_params(
            model_path.to_str().unwrap_or_default(),
            whisper_rs::WhisperContextParameters::default(),
        )
        .map_err(|e| format!("Failed to load Whisper model: {e}"))?;

        let mut state = ctx
            .create_state()
            .map_err(|e| format!("Whisper state error: {e}"))?;

        let mut params = whisper_rs::FullParams::new(
            whisper_rs::SamplingStrategy::Greedy { best_of: 0 },
        );
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_n_threads(4);

        state
            .full(params, &audio)
            .map_err(|e| format!("Whisper inference failed: {e}"))?;

        let n = state.full_n_segments();
        let mut parts: Vec<String> = Vec::with_capacity(n as usize);
        for i in 0..n {
            if let Some(seg) = state.get_segment(i) {
                if let Ok(text) = seg.to_str_lossy() {
                    let t = text.trim().to_string();
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
            }
        }
        Ok(parts.join(" "))
    })
    .await
    .map_err(|e| e.to_string())?
}

// ── Audio preparation ─────────────────────────────────────────────────────────

/// Mix to mono, resample to 16 kHz, and prepend 300 ms of silence.
/// Whisper requires exactly 16 kHz mono f32. The silence padding prevents
/// first-word hallucination that occurs when real speech starts at sample 0.
fn prepare_audio_for_whisper(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Vec<f32> {
    let mono: Vec<f32> = if channels > 1 {
        samples
            .chunks_exact(channels as usize)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };

    let mut audio = if sample_rate != 16_000 {
        log::debug!("STT: resampling from {} Hz to 16 kHz", sample_rate);
        resample_linear(&mono, sample_rate, 16_000)
    } else {
        mono
    };

    let pad = vec![0.0f32; (16_000.0 * 0.3) as usize];
    audio.splice(0..0, pad);
    audio
}

// ── Linear resampler ──────────────────────────────────────────────────────────

/// Linear resampler adequate for speech. Whisper requires exactly 16 kHz mono f32.
fn resample_linear(samples: &[f32], from_hz: u32, to_hz: u32) -> Vec<f32> {
    let ratio = from_hz as f64 / to_hz as f64;
    let out_len = (samples.len() as f64 / ratio).round() as usize;
    (0..out_len)
        .map(|i| {
            let pos = i as f64 * ratio;
            let lo = pos as usize;
            let frac = (pos - lo as f64) as f32;
            let a = samples.get(lo).copied().unwrap_or(0.0);
            let b = samples.get(lo + 1).copied().unwrap_or(a);
            a + (b - a) * frac
        })
        .collect()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn emit_overlay_state<R: tauri::Runtime>(app: &tauri::AppHandle<R>, state: &str) {
    log::debug!("Overlay state → {}", state);
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.emit("overlay-state", state);
    }
}
