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
            let id = d.name().ok()?;
            if !is_useful_device(&id) {
                return None;
            }
            let name = friendly_device_name(&id);
            Some(AudioDevice { id, name })
        })
        .collect())
}

#[tauri::command]
pub fn list_output_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let devices = host.output_devices().map_err(|e| e.to_string())?;

    Ok(devices
        .filter_map(|d| {
            let id = d.name().ok()?;
            if !is_useful_device(&id) {
                return None;
            }
            let name = friendly_device_name(&id);
            Some(AudioDevice { id, name })
        })
        .collect())
}

/// On Linux/ALSA, skip raw-hardware and plugin devices that are exclusive-access
/// or serve no purpose in a GUI picker (hw:, plughw:, dmix:, surround channels, etc.).
fn is_useful_device(name: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        const SKIP: &[&str] = &[
            "hw:", "plughw:", "dmix:", "dsnoop:", "dshare:",
            "front:", "rear:", "center_lfe:", "side:",
            "surround40:", "surround41:", "surround50:", "surround51:", "surround71:",
            "iec958:", "modem:",
            "samplerate:", "speexrate:", "upmix:", "vdownmix:",
        ];
        if SKIP.iter().any(|prefix| name.starts_with(prefix)) {
            return false;
        }
    }
    true
}

/// Convert an ALSA device identifier into a human-readable display name.
/// On Windows/macOS, cpal already provides friendly names so the raw value is returned.
fn friendly_device_name(name: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        // ALSA names encode the card name in a CARD= parameter, e.g.:
        //   "sysdefault:CARD=HDA_Intel_PCH,DEV=0" → "HDA Intel PCH (Default)"
        //   "default:CARD=Generic"                 → "Generic (Default)"
        if let Some(idx) = name.find("CARD=") {
            let after = &name[idx + 5..];
            let card_raw = after.split(',').next().unwrap_or(after);
            let card = card_raw.replace('_', " ");
            let prefix = name.split(':').next().unwrap_or("");
            return match prefix {
                "default" | "sysdefault" => format!("{} (Default)", card.trim()),
                _ => card.trim().to_string(),
            };
        }
        let mapped = match name {
            "default"  => Some("Default Device"),
            "pulse"    => Some("PulseAudio"),
            "pipewire" => Some("PipeWire"),
            "jack"     => Some("JACK"),
            _          => None,
        };
        if let Some(s) = mapped {
            return s.to_string();
        }
    }
    name.to_string()
}
