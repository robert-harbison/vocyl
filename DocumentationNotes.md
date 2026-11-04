# Documentation Notes

This document contains points that will need to be properly documented for the end user.

## Settings & Data

Settings are stored in a JSON file under the OS application-config directory:

| Platform    | Path                                                              |
| ----------- | ----------------------------------------------------------------- |
| **Windows** | `%APPDATA%\com.sparkinex.vocyl\settings.json`                     |
| **macOS**   | `~/Library/Application Support/com.sparkinex.vocyl/settings.json` |
| **Linux**   | `~/.config/com.sparkinex.vocyl/settings.json`                     |

Downloaded models and runtime binaries are stored in the application data directory:

| Platform    | Path                                                 |
| ----------- | ---------------------------------------------------- |
| **Windows** | `%APPDATA%\com.sparkinex.vocyl\`                     |
| **macOS**   | `~/Library/Application Support/com.sparkinex.vocyl/` |
| **Linux**   | `~/.local/share/com.sparkinex.vocyl/`                |

Subdirectories within the data folder:

| Path          | Contents                                      |
| ------------- | --------------------------------------------- |
| `models/tts/` | TTS voice models (Kokoro package)             |
| `models/stt/` | STT models (`ggml-*.bin` Whisper model files) |

## Logs

Application logs are written to a rotating log file in the OS application-log directory.
Both Rust backend and JavaScript frontend messages are written to the same file.

| Platform    | Path                                                |
| ----------- | --------------------------------------------------- |
| **Windows** | `%LOCALAPPDATA%\com.sparkinex.vocyl\logs\vocyl.log` |
| **macOS**   | `~/Library/Logs/vocyl/vocyl.log`                    |
| **Linux**   | `~/.local/share/com.sparkinex.vocyl/logs/vocyl.log` |

Log level is `INFO` by default. The log file is automatically rotated by `tauri-plugin-log`.

# Default Shortcuts

| Action                | Windows/Linux | macOS        |
| --------------------- | ------------- | ------------ |
| STT (Push-to-talk)    | `Alt + A`     | `Option + A` |
| TTS (Speak / Overlay) | `Alt + S`     | `Option + S` |
