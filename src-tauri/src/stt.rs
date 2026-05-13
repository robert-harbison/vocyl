use std::{
    path::PathBuf,
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
use enigo::Keyboard;
use tauri::{Emitter, Manager};

use crate::{
    models::{get_whisper_bin, model_dir},
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
/// Emits `overlay-state: "listening"` via app handle on success.
pub fn start_recording<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    device_name: Option<String>,
) {
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
                    Some(d) => d,
                    None => match host.default_input_device() {
                        Some(d) => d,
                        None => {
                            emit_overlay_state(&app_handle, "error");
                            return;
                        }
                    },
                }
            }
            None => match host.default_input_device() {
                Some(d) => d,
                None => {
                    emit_overlay_state(&app_handle, "error");
                    return;
                }
            },
        };

        let config = match device.default_input_config() {
            Ok(c) => c,
            Err(_) => {
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

        let err_fn = |e: cpal::StreamError| eprintln!("[vocyl] audio stream error: {e:?}");

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
                emit_overlay_state(&app_handle, "error");
                return;
            }
        };

        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[vocyl] failed to build input stream: {e:?}");
                emit_overlay_state(&app_handle, "error");
                return;
            }
        };

        if stream.play().is_err() {
            emit_overlay_state(&app_handle, "error");
            return;
        }

        emit_overlay_state(&app_handle, "listening");

        while is_rec_loop.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(10));
        }

        drop(stream);
    });
}

/// Stop recording and transcribe. Returns transcript text, or error string.
pub async fn stop_and_transcribe<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<String, String> {
    // Signal recording thread to stop
    app.state::<SttState>()
        .is_recording
        .store(false, Ordering::SeqCst);

    // Give the recording thread a moment to finish writing samples
    tokio::time::sleep(Duration::from_millis(80)).await;

    let samples = app.state::<SttState>().samples.lock().unwrap().clone();
    let sample_rate = *app.state::<SttState>().sample_rate.lock().unwrap();
    let channels = *app.state::<SttState>().channels.lock().unwrap();

    if samples.is_empty() {
        return Ok(String::new());
    }

    // Write to temp WAV
    let wav_path = write_temp_wav(&samples, sample_rate, channels)?;

    // Look up active STT model
    let model_size = app
        .state::<SettingsCache>()
        .0
        .lock()
        .unwrap()
        .active_stt_model
        .clone()
        .ok_or("No STT model selected. Choose one in Settings → Speech-to-Text.")?;

    let filename = format!("ggml-{model_size}.bin");
    let model_path = model_dir(app, "stt")?.join(&filename);

    if !model_path.exists() {
        return Err(format!("STT model '{filename}' not found. Download it in Settings → Speech-to-Text."));
    }

    let whisper_bin = get_whisper_bin(app)?;

    // Run whisper-cli
    let output = tokio::process::Command::new(&whisper_bin)
        .args([
            "-m",
            model_path.to_str().unwrap_or_default(),
            "-f",
            wav_path.to_str().unwrap_or_default(),
            "-nt",
            "-l",
            "en",
            "-t",
            "4",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run whisper: {e}"))?;

    std::fs::remove_file(&wav_path).ok();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Whisper exited with error: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let transcript = stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(transcript)
}

// ── Text injection ────────────────────────────────────────────────────────────

pub fn inject_text(text: &str) -> Result<(), String> {
    use enigo::{Enigo, Settings};
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create input injector: {e:?}"))?;
    // Keyboard trait is imported at the top of the file
    enigo
        .text(text)
        .map_err(|e| format!("Text injection failed: {e:?}"))?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn write_temp_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!(
        "vocyl_stt_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
    for &s in samples {
        writer.write_sample(s).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;

    Ok(path)
}

pub fn emit_overlay_state<R: tauri::Runtime>(app: &tauri::AppHandle<R>, state: &str) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.emit("overlay-state", state);
    }
}
