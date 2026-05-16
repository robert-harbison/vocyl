use std::path::PathBuf;

use futures_util::StreamExt;
use tauri::{Emitter, Manager};
use tokio::io::AsyncWriteExt;

use crate::constants::KOKORO_PKG_URL;

// ── Path helpers ──────────────────────────────────────────────────────────────

pub fn model_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>, dir: &str) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(data_dir.join("models").join(dir))
}

// ── Generic model file commands ───────────────────────────────────────────────

#[tauri::command]
pub fn list_downloaded_files(app: tauri::AppHandle, dir: String) -> Result<Vec<String>, String> {
    log::debug!("Listing downloaded files in dir='{}'", dir);
    let path = model_dir(&app, &dir)?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let files: Vec<String> = std::fs::read_dir(&path)
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

    log::debug!("Found {} file(s) in '{}'", files.len(), dir);
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
        log::info!("File already exists, skipping download: {}/{}", dir, filename);
        let _ = app.emit(
            "download-complete",
            serde_json::json!({ "dir": &dir, "filename": &filename }),
        );
        return Ok(());
    }

    log::info!("Starting download: {}/{} from {}", dir, filename, url);
    let tmp_path = dest_dir.join(format!("{filename}.tmp"));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            log::error!("HTTP request failed for {}: {}", url, e);
            format!("HTTP request failed: {e}")
        })?;

    if !response.status().is_success() {
        log::error!("HTTP {} for URL: {}", response.status(), url);
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

    log::info!("Download complete: {}/{} ({} bytes)", dir, filename, downloaded);
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
        log::info!("Deleting file: {}/{}", dir, filename);
        std::fs::remove_file(&path).map_err(|e| {
            log::error!("Failed to delete {}/{}: {}", dir, filename, e);
            e.to_string()
        })?;
        log::info!("Deleted: {}/{}", dir, filename);
    } else {
        log::debug!("delete_file: {}/{} does not exist, nothing to do", dir, filename);
    }
    Ok(())
}

// ── Shared HTTP download helper ───────────────────────────────────────────────

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
        .map_err(|e| {
            log::error!("HTTP request failed for {}: {}", url, e);
            format!("HTTP request failed: {e}")
        })?;

    if !response.status().is_success() {
        log::error!("HTTP {} downloading {}", response.status(), url);
        return Err(format!("HTTP {}: {url}", response.status()));
    }

    let total = response.content_length();
    log::info!(
        "Download started: {} ({} bytes total)",
        progress_label,
        total.map(|n| n.to_string()).unwrap_or_else(|| "unknown".to_string())
    );
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

    log::info!("Download stream complete: {} ({} bytes)", progress_label, downloaded);
    Ok(())
}

// ── Kokoro model package ──────────────────────────────────────────────────────

pub fn get_kokoro_model_dir<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let dir = model_dir(app, "tts")?.join("kokoro-en-v0_19");
    if dir.join("model.onnx").exists() && dir.join("voices.bin").exists() {
        return Ok(dir);
    }
    Err("Kokoro model not installed. Download it in Settings → Text-to-Speech.".to_string())
}

#[tauri::command]
pub fn is_kokoro_installed(app: tauri::AppHandle) -> bool {
    let installed = get_kokoro_model_dir(&app).is_ok();
    log::debug!("is_kokoro_installed → {}", installed);
    installed
}

#[tauri::command]
pub async fn download_kokoro_package(app: tauri::AppHandle) -> Result<(), String> {
    let dest = model_dir(&app, "tts")?;
    tokio::fs::create_dir_all(&dest).await.map_err(|e| e.to_string())?;

    if get_kokoro_model_dir(&app).is_ok() {
        log::info!("Kokoro model already installed, skipping download");
        let _ = app.emit("download-complete", serde_json::json!({ "dir": "tts", "filename": "kokoro" }));
        return Ok(());
    }

    log::info!("Starting Kokoro package download from {}", KOKORO_PKG_URL);
    let tmp_path = dest.join("kokoro.tar.bz2.tmp");
    stream_download(&app, KOKORO_PKG_URL, &tmp_path, "tts", "kokoro").await?;

    log::info!("Kokoro download complete, extracting…");
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

    log::info!("Kokoro package installed successfully");
    let _ = app.emit("download-complete", serde_json::json!({ "dir": "tts", "filename": "kokoro" }));
    Ok(())
}
