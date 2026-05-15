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
use tauri::{Emitter, Manager};

use crate::{
    models::model_dir,
    settings::SettingsCache,
};

// Windows-only: subprocess path needs the binary location and hound for WAV writing.
#[cfg(target_os = "windows")]
use crate::models::get_whisper_bin;

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
                        None => { emit_overlay_state(&app_handle, "error"); return; }
                    },
                }
            }
            None => match host.default_input_device() {
                Some(d) => d,
                None => { emit_overlay_state(&app_handle, "error"); return; }
            },
        };

        let config = match device.default_input_config() {
            Ok(c) => c,
            Err(_) => { emit_overlay_state(&app_handle, "error"); return; }
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
            _ => { emit_overlay_state(&app_handle, "error"); return; }
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

// ── Transcription ─────────────────────────────────────────────────────────────

/// Stop recording and transcribe. Platform dispatch:
/// - Windows  → subprocess (whisper-cli.exe)
/// - Linux/macOS → native whisper-rs compiled into the binary
pub async fn stop_and_transcribe<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<String, String> {
    app.state::<SttState>()
        .is_recording
        .store(false, Ordering::SeqCst);

    // Give the recording thread a moment to finish writing samples.
    tokio::time::sleep(Duration::from_millis(80)).await;

    let samples = app.state::<SttState>().samples.lock().unwrap().clone();
    let sample_rate = *app.state::<SttState>().sample_rate.lock().unwrap();
    let channels = *app.state::<SttState>().channels.lock().unwrap();

    if samples.is_empty() {
        return Ok(String::new());
    }

    let model_size = app
        .state::<SettingsCache>()
        .0
        .lock()
        .unwrap()
        .active_stt_model
        .clone()
        .ok_or("No STT model selected. Choose one in Settings → Speech-to-Text.")?;

    let model_path = model_dir(app, "stt")?.join(format!("ggml-{model_size}.bin"));
    if !model_path.exists() {
        return Err(format!(
            "STT model 'ggml-{model_size}.bin' not found. Download it in Settings → Speech-to-Text."
        ));
    }

    // ── Windows: run whisper-cli.exe subprocess ───────────────────────────────
    #[cfg(target_os = "windows")]
    {
        let whisper_bin = get_whisper_bin(app)?;
        return transcribe_subprocess(whisper_bin, model_path, samples, sample_rate, channels)
            .await;
    }

    // ── Linux / macOS: native whisper-rs compiled into the binary ─────────────
    #[cfg(not(target_os = "windows"))]
    transcribe_native(model_path, samples, sample_rate, channels).await
}

// ── Windows subprocess path ───────────────────────────────────────────────────

#[cfg(target_os = "windows")]
async fn transcribe_subprocess(
    whisper_bin: PathBuf,
    model_path: PathBuf,
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
) -> Result<String, String> {
    // Mix to mono then resample to 16 kHz — whisper.cpp's internal resampler
    // produces garbage (hallucinated dots) when given raw multi-channel /
    // high-sample-rate audio on Windows.
    let mono: Vec<f32> = if channels > 1 {
        samples
            .chunks_exact(channels as usize)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };
    let mut audio_16k = if sample_rate != 16_000 {
        resample_linear(&mono, sample_rate, 16_000)
    } else {
        mono
    };

    // Prepend 300 ms of silence. Whisper's first segment often hallucinates
    // consonant repetitions (e.g. "nnnnn") when real speech starts at sample 0
    // because the model expects a brief quiet onset. The padding costs nothing
    // in transcription accuracy and reliably fixes the first-word corruption.
    let pad = vec![0.0f32; (16_000.0 * 0.3) as usize];
    audio_16k.splice(0..0, pad);

    let wav_path = write_temp_wav(&audio_16k)?;

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
        .filter(|l| !l.starts_with("whisper_") && !l.starts_with("ggml_") && !l.starts_with("main:"))
        .filter(|l| !(l.starts_with('[') && l.ends_with(']')))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(transcript)
}

#[cfg(target_os = "windows")]
fn write_temp_wav(samples: &[f32]) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!(
        "vocyl_stt_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
    for &s in samples {
        // Clamp to [-1, 1] before converting to avoid i16 overflow on hot samples.
        let s = s.clamp(-1.0, 1.0);
        let pcm = (s * 32767.0) as i16;
        writer.write_sample(pcm).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;

    Ok(path)
}

// ── Linux / macOS native path (whisper-rs 0.16) ───────────────────────────────

#[cfg(not(target_os = "windows"))]
async fn transcribe_native(
    model_path: PathBuf,
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
        // Mix multi-channel audio down to mono.
        let mono: Vec<f32> = if channels > 1 {
            samples
                .chunks_exact(channels as usize)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                .collect()
        } else {
            samples
        };

        // Whisper requires exactly 16 kHz. Resample if the capture device used a different rate.
        let mut audio = if sample_rate != 16_000 {
            resample_linear(&mono, sample_rate, 16_000)
        } else {
            mono
        };

        // Prepend 300 ms of silence — fixes first-word hallucination at segment 0.
        let pad = vec![0.0f32; (16_000.0 * 0.3) as usize];
        audio.splice(0..0, pad);

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

/// Linear resampler for voice audio. Adequate quality for speech; avoids an
/// extra dependency. Whisper.cpp requires exactly 16 kHz mono f32 input.
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

// ── Text injection ────────────────────────────────────────────────────────────

pub fn inject_text(text: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // enigo's text() on Windows corrupts longer strings via SendInput/unicode.
        // Clipboard + Ctrl+V is what every other Windows STT tool uses and is reliable.
        use arboard::Clipboard;
        use enigo::{Direction, Enigo, Key, Keyboard, Settings};

        let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;
        clipboard.set_text(text).map_err(|e| format!("Failed to copy transcript to clipboard: {e}"))?;

        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create input injector: {e:?}"))?;
        enigo.key(Key::Control, Direction::Press)
            .map_err(|e| format!("Key injection failed: {e:?}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| format!("Key injection failed: {e:?}"))?;
        enigo.key(Key::Control, Direction::Release)
            .map_err(|e| format!("Key injection failed: {e:?}"))?;

        return Ok(());
    }

    // Linux/X11: enigo.text() uses XSendEvent which most apps ignore.
    // Clipboard + Ctrl+V is reliable across all X11 apps.
    #[cfg(target_os = "linux")]
    {
        use arboard::Clipboard;
        use enigo::{Direction, Enigo, Key, Keyboard, Settings};

        let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;
        clipboard.set_text(text).map_err(|e| format!("Failed to copy transcript to clipboard: {e}"))?;

        std::thread::sleep(std::time::Duration::from_millis(50));

        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create input injector: {e:?}"))?;
        enigo.key(Key::Control, Direction::Press)
            .map_err(|e| format!("Key injection failed: {e:?}"))?;
        enigo.key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| format!("Key injection failed: {e:?}"))?;
        enigo.key(Key::Control, Direction::Release)
            .map_err(|e| format!("Key injection failed: {e:?}"))?;

        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        use enigo::{Enigo, Keyboard, Settings};
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create input injector: {e:?}"))?;
        enigo
            .text(text)
            .map_err(|e| format!("Text injection failed: {e:?}"))?;
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn emit_overlay_state<R: tauri::Runtime>(app: &tauri::AppHandle<R>, state: &str) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.emit("overlay-state", state);
    }
}
