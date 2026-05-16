use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

#[derive(Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub fn list_input_devices() -> Result<Vec<AudioDevice>, String> {
    log::info!("Enumerating input audio devices");
    // On Linux, prefer PipeWire/PulseAudio enumeration for human-readable names.
    #[cfg(target_os = "linux")]
    if let Some(devices) = pactl_list_devices(true) {
        log::info!("Found {} input device(s) via pactl", devices.len());
        return Ok(devices);
    }

    let host = cpal::default_host();
    let devices = host.input_devices().map_err(|e| e.to_string())?;

    let result: Vec<AudioDevice> = devices
        .filter_map(|d| {
            let id = d.name().ok()?;
            if !is_useful_alsa_device(&id) {
                return None;
            }
            let name = friendly_alsa_name(&id);
            Some(AudioDevice { id, name })
        })
        .collect();

    log::info!("Found {} input device(s)", result.len());
    Ok(result)
}

#[tauri::command]
pub fn list_output_devices() -> Result<Vec<AudioDevice>, String> {
    log::info!("Enumerating output audio devices");
    // On Linux, prefer PipeWire/PulseAudio enumeration for human-readable names.
    #[cfg(target_os = "linux")]
    if let Some(devices) = pactl_list_devices(false) {
        log::info!("Found {} output device(s) via pactl", devices.len());
        return Ok(devices);
    }

    let host = cpal::default_host();
    let devices = host.output_devices().map_err(|e| e.to_string())?;

    let result: Vec<AudioDevice> = devices
        .filter_map(|d| {
            let id = d.name().ok()?;
            if !is_useful_alsa_device(&id) {
                return None;
            }
            let name = friendly_alsa_name(&id);
            Some(AudioDevice { id, name })
        })
        .collect();

    log::info!("Found {} output device(s)", result.len());
    Ok(result)
}

// ── Linux / PipeWire / PulseAudio ─────────────────────────────────────────────

/// Use `pactl list sinks` (output) or `pactl list sources` (input) to enumerate
/// devices with their human-readable descriptions. This works on both PipeWire
/// (via its PulseAudio compatibility layer) and classic PulseAudio.
///
/// Returns `None` if `pactl` is not available so callers can fall back to ALSA.
#[cfg(target_os = "linux")]
fn pactl_list_devices(input: bool) -> Option<Vec<AudioDevice>> {
    let kind = if input { "sources" } else { "sinks" };
    let output = std::process::Command::new("pactl")
        .args(["list", kind])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut devices = parse_pactl_output(&text, input);
    if devices.is_empty() {
        return None;
    }
    // Prepend a "use system default" sentinel so the user can always revert.
    devices.insert(0, AudioDevice {
        id: String::new(),
        name: "Default (System)".to_string(),
    });
    Some(devices)
}

/// Parse the output of `pactl list sinks` or `pactl list sources`.
/// Extracts `Name:` (used as device ID) and `Description:` (display label).
/// Monitor sources (e.g. loopback of an output) are excluded from the input list.
#[cfg(target_os = "linux")]
fn parse_pactl_output(text: &str, input: bool) -> Vec<AudioDevice> {
    let mut devices: Vec<AudioDevice> = Vec::new();
    let mut cur_name: Option<String> = None;
    let mut cur_desc: Option<String> = None;

    let flush = |name: Option<String>, desc: Option<String>, devices: &mut Vec<AudioDevice>| {
        if let (Some(id), Some(display)) = (name, desc) {
            // Skip monitor sources — they capture the output of a sink, not a real mic.
            if input && id.ends_with(".monitor") {
                return;
            }
            devices.push(AudioDevice { id, name: display });
        }
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Name:") {
            flush(cur_name.take(), cur_desc.take(), &mut devices);
            cur_name = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Description:") {
            cur_desc = Some(rest.trim().to_string());
        }
    }
    flush(cur_name, cur_desc, &mut devices);

    devices
}

// ── ALSA fallback (non-Linux paths and Linux without pactl) ───────────────────

/// Skip raw-hardware and plugin devices that are exclusive-access or serve no
/// purpose in a GUI picker (hw:, plughw:, dmix:, surround channels, etc.).
fn is_useful_alsa_device(name: &str) -> bool {
    #[cfg(not(target_os = "linux"))]
    let _ = name;
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
fn friendly_alsa_name(name: &str) -> String {
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
