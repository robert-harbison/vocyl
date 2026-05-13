use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

#[derive(Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub fn list_input_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let devices = host.input_devices().map_err(|e| e.to_string())?;

    Ok(devices
        .filter_map(|d| {
            let name = d.name().ok()?;
            Some(AudioDevice {
                id: name.clone(),
                name,
            })
        })
        .collect())
}

#[tauri::command]
pub fn list_output_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let devices = host.output_devices().map_err(|e| e.to_string())?;

    Ok(devices
        .filter_map(|d| {
            let name = d.name().ok()?;
            Some(AudioDevice {
                id: name.clone(),
                name,
            })
        })
        .collect())
}
