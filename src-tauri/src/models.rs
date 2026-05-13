use std::path::PathBuf;

use futures_util::StreamExt;
use tauri::{Emitter, Manager};
use tokio::io::AsyncWriteExt;

use crate::constants::{KOKORO_PKG_URL, SHERPA_TTS_WIN_URL, piper_url};
#[cfg(target_os = "windows")]
use crate::constants::WHISPER_WIN_URL;

// ── Path helpers ──────────────────────────────────────────────────────────────

pub fn model_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>, dir: &str) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("models").join(dir))
}

pub fn binary_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>, subdir: &str) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("binaries").join(subdir))
}

/// Platform-specific name for the Piper executable.
fn piper_exe() -> &'static str {
    if cfg!(windows) { "piper.exe" } else { "piper" }
}

pub fn get_piper_bin<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = binary_dir(app, "piper")?;
    let exe = piper_exe();

    // Direct placement (e.g. extracted flat into the piper/ dir)
    let candidate = dir.join(exe);
    if candidate.exists() {
        return Ok(candidate);
    }
    // One level of nesting (e.g. piper_linux_x86_64.tar.gz → piper/piper)
    let nested = dir.join("piper").join(exe);
    if nested.exists() {
        return Ok(nested);
    }
    Err("Piper runtime not installed. Download it in Settings → Text-to-Speech.".to_string())
}

// ── Generic model file commands ───────────────────────────────────────────────

#[tauri::command]
pub fn list_downloaded_files(app: tauri::AppHandle, dir: String) -> Result<Vec<String>, String> {
    let path = model_dir(&app, &dir)?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let files = std::fs::read_dir(&path)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_file() {
                Some(entry.file_name().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(files)
}

#[tauri::command]
pub async fn download_file(
    app: tauri::AppHandle,
    dir: String,
    filename: String,
    url: String,
) -> Result<(), String> {
    let dest_dir = model_dir(&app, &dir)?;
    tokio::fs::create_dir_all(&dest_dir)
        .await
        .map_err(|e| e.to_string())?;

    let dest_path = dest_dir.join(&filename);
    if dest_path.exists() {
        let _ = app.emit(
            "download-complete",
            serde_json::json!({ "dir": &dir, "filename": &filename }),
        );
        return Ok(());
    }

    let tmp_path = dest_dir.join(format!("{filename}.tmp"));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {url}", response.status()));
    }

    let total = response.content_length();
    let mut downloaded = 0u64;

    {
        let mut file = tokio::fs::File::create(&tmp_path)
            .await
            .map_err(|e| e.to_string())?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;

            let progress = total
                .map(|t| if t > 0 { (downloaded as f64 / t as f64 * 100.0) as u32 } else { 0 })
                .unwrap_or(0);

            let _ = app.emit(
                "download-progress",
                serde_json::json!({
                    "dir": &dir,
                    "filename": &filename,
                    "downloaded": downloaded,
                    "total": total.unwrap_or(0),
                    "progress": progress,
                }),
            );
        }
    }

    tokio::fs::rename(&tmp_path, &dest_path)
        .await
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        "download-complete",
        serde_json::json!({ "dir": &dir, "filename": &filename }),
    );

    Ok(())
}

#[tauri::command]
pub fn delete_file(app: tauri::AppHandle, dir: String, filename: String) -> Result<(), String> {
    let path = model_dir(&app, &dir)?.join(&filename);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── Binary installation checks ────────────────────────────────────────────────

#[tauri::command]
pub fn is_piper_installed(app: tauri::AppHandle) -> bool {
    get_piper_bin(&app).is_ok()
}

/// Windows: look for the downloaded whisper-cli.exe.
/// Linux/macOS: whisper-rs is compiled into the binary, so always installed.
#[tauri::command]
pub fn is_whisper_installed(app: tauri::AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    { get_whisper_bin(&app).is_ok() }

    #[cfg(not(target_os = "windows"))]
    { let _ = app; true }
}

/// Returns the path to the whisper-cli executable on Windows.
/// Only called on Windows; the function is not compiled on other platforms.
#[cfg(target_os = "windows")]
pub fn get_whisper_bin<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = binary_dir(app, "whisper")?;

    for name in &["whisper-cli.exe", "main.exe"] {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // One level of nesting (zip extracts as whisper-bin-x64/whisper-cli.exe)
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                for name in &["whisper-cli.exe", "main.exe"] {
                    let candidate = entry.path().join(name);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }
    }

    Err("Whisper runtime not installed. Download it in Settings → Speech-to-Text.".to_string())
}

/// Windows: download and extract the official whisper-cli.exe ZIP.
/// Linux/macOS: no-op — whisper-rs is compiled into the binary.
#[tauri::command]
pub async fn download_whisper(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let dest = binary_dir(&app, "whisper")?;
        download_and_extract_zip(&app, WHISPER_WIN_URL, dest, "whisper").await
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Built into the app — nothing to download.
        let _ = app.emit(
            "download-complete",
            serde_json::json!({ "dir": "binaries", "filename": "whisper" }),
        );
        Ok(())
    }
}

// ── ZIP download + extract ────────────────────────────────────────────────────

async fn download_and_extract_zip(
    app: &tauri::AppHandle,
    url: &str,
    extract_to: PathBuf,
    progress_label: &str,
) -> Result<(), String> {
    tokio::fs::create_dir_all(&extract_to)
        .await
        .map_err(|e| e.to_string())?;

    let tmp_zip = extract_to.join("__download.zip.tmp");

    stream_download(app, url, &tmp_zip, "binaries", progress_label).await?;

    let zip_path = tmp_zip.clone();
    let out_dir = extract_to.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let data = std::fs::read(&zip_path).map_err(|e| e.to_string())?;
        let cursor = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let out_path = out_dir.join(entry.name());

            if entry.is_dir() {
                std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut outfile = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = outfile.metadata().unwrap().permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&out_path, perms).ok();
                }
            }
        }

        std::fs::remove_file(&zip_path).ok();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    let _ = app.emit(
        "download-complete",
        serde_json::json!({ "dir": "binaries", "filename": progress_label }),
    );

    Ok(())
}

// ── tar.gz download + extract ─────────────────────────────────────────────────

async fn download_and_extract_targz(
    app: &tauri::AppHandle,
    url: &str,
    extract_to: PathBuf,
    progress_label: &str,
) -> Result<(), String> {
    tokio::fs::create_dir_all(&extract_to)
        .await
        .map_err(|e| e.to_string())?;

    let tmp = extract_to.join("__download.tar.gz.tmp");

    stream_download(app, url, &tmp, "binaries", progress_label).await?;

    let tmp2 = tmp.clone();
    let out_dir = extract_to.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let file = std::fs::File::open(&tmp2).map_err(|e| e.to_string())?;
        let mut archive = Archive::new(GzDecoder::new(file));
        // tar::Archive::unpack preserves Unix permissions from the archive headers.
        archive.unpack(&out_dir).map_err(|e| e.to_string())?;
        std::fs::remove_file(&tmp2).ok();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    let _ = app.emit(
        "download-complete",
        serde_json::json!({ "dir": "binaries", "filename": progress_label }),
    );

    Ok(())
}

// ── tar.bz2 download + extract ────────────────────────────────────────────────

async fn download_and_extract_tarbz2(
    app: &tauri::AppHandle,
    url: &str,
    extract_to: PathBuf,
    progress_label: &str,
) -> Result<(), String> {
    tokio::fs::create_dir_all(&extract_to)
        .await
        .map_err(|e| e.to_string())?;

    let tmp = extract_to.join("__download.tar.bz2.tmp");

    stream_download(app, url, &tmp, "binaries", progress_label).await?;

    let tmp2 = tmp.clone();
    let out_dir = extract_to.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        use bzip2::read::BzDecoder;
        use tar::Archive;

        let file = std::fs::File::open(&tmp2).map_err(|e| e.to_string())?;
        let mut archive = Archive::new(BzDecoder::new(file));
        archive.unpack(&out_dir).map_err(|e| e.to_string())?;
        std::fs::remove_file(&tmp2).ok();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    let _ = app.emit(
        "download-complete",
        serde_json::json!({ "dir": "binaries", "filename": progress_label }),
    );

    Ok(())
}

// ── Shared HTTP download helper ───────────────────────────────────────────────

/// Stream-downloads `url` to `dest`, emitting `download-progress` events.
/// `event_dir` is included in the event payload so the frontend can match
/// the right progress bar (e.g. `"binaries"` for executables, `"tts"` for models).
/// Does NOT emit `download-complete` — callers do that after extraction.
async fn stream_download(
    app: &tauri::AppHandle,
    url: &str,
    dest: &PathBuf,
    event_dir: &str,
    progress_label: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {url}", response.status()));
    }

    let total = response.content_length();
    let mut downloaded = 0u64;

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let progress = total
            .map(|t| if t > 0 { (downloaded as f64 / t as f64 * 100.0) as u32 } else { 0 })
            .unwrap_or(0);

        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "dir": event_dir,
                "filename": progress_label,
                "downloaded": downloaded,
                "total": total.unwrap_or(0),
                "progress": progress,
            }),
        );
    }

    Ok(())
}

// ── Piper binary download ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn download_piper(app: tauri::AppHandle) -> Result<(), String> {
    let url = piper_url().ok_or_else(|| {
        format!(
            "Piper has no pre-built binary for {} ({}). Use Kokoro TTS instead.",
            std::env::consts::OS,
            std::env::consts::ARCH,
        )
    })?;

    let dest = binary_dir(&app, "piper")?;

    if url.ends_with(".zip") {
        download_and_extract_zip(&app, url, dest, "piper").await
    } else {
        download_and_extract_targz(&app, url, dest, "piper").await
    }
}

// ── Sherpa-ONNX legacy binary (Windows only, kept for existing installs) ──────

#[allow(dead_code)]
pub fn get_sherpa_tts_bin<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = binary_dir(app, "sherpa")?;
    let exe = if cfg!(windows) { "sherpa-onnx-offline-tts.exe" } else { "sherpa-onnx-offline-tts" };
    find_exe_recursive(&dir, exe)
        .ok_or_else(|| "Sherpa-ONNX TTS not installed.".to_string())
}

fn find_exe_recursive(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    let direct = dir.join(name);
    if direct.exists() {
        return Some(direct);
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(found) = find_exe_recursive(&entry.path(), name) {
                    return Some(found);
                }
            }
        }
    }
    None
}

#[tauri::command]
pub fn is_sherpa_tts_installed(app: tauri::AppHandle) -> bool {
    get_sherpa_tts_bin(&app).is_ok()
}

// ── Kokoro model package (tar.bz2) ────────────────────────────────────────────

pub fn get_kokoro_model_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = model_dir(app, "tts")?.join("kokoro-en-v0_19");
    if dir.join("model.onnx").exists() && dir.join("voices.bin").exists() {
        return Ok(dir);
    }
    Err("Kokoro model not installed. Download it in Settings → Text-to-Speech.".to_string())
}

#[tauri::command]
pub fn is_kokoro_installed(app: tauri::AppHandle) -> bool {
    get_kokoro_model_dir(&app).is_ok()
}

#[tauri::command]
pub async fn download_kokoro_package(app: tauri::AppHandle) -> Result<(), String> {
    let dest = model_dir(&app, "tts")?;
    tokio::fs::create_dir_all(&dest).await.map_err(|e| e.to_string())?;

    if get_kokoro_model_dir(&app).is_ok() {
        let _ = app.emit("download-complete", serde_json::json!({ "dir": "tts", "filename": "kokoro" }));
        return Ok(());
    }

    let tmp_path = dest.join("kokoro.tar.bz2.tmp");
    stream_download(&app, KOKORO_PKG_URL, &tmp_path, "tts", "kokoro").await?;

    let dest_dir = dest.clone();
    let tmp = tmp_path.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        use bzip2::read::BzDecoder;
        use tar::Archive;

        let file = std::fs::File::open(&tmp).map_err(|e| e.to_string())?;
        let mut archive = Archive::new(BzDecoder::new(file));
        archive.unpack(&dest_dir).map_err(|e| e.to_string())?;
        std::fs::remove_file(&tmp).ok();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    let _ = app.emit("download-complete", serde_json::json!({ "dir": "tts", "filename": "kokoro" }));
    Ok(())
}

// ── Sherpa-ONNX TTS binary download (legacy, Windows only) ───────────────────

#[tauri::command]
#[allow(dead_code)]
pub async fn download_sherpa_tts(app: tauri::AppHandle) -> Result<(), String> {
    let dest = binary_dir(&app, "sherpa")?;

    if get_sherpa_tts_bin(&app).is_ok() {
        let _ = app.emit("download-complete", serde_json::json!({ "dir": "binaries", "filename": "sherpa" }));
        return Ok(());
    }

    download_and_extract_tarbz2(&app, SHERPA_TTS_WIN_URL, dest, "sherpa").await
}
