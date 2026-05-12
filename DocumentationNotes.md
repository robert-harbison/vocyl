# Documentation Notes

This document contains points that will need to be properly documented for the end user.

## Settings & Data

Settings are stored in a JSON file under the OS application-config directory:

| Platform    | Path                                                              |
| ----------- | ----------------------------------------------------------------- |
| **Windows** | `%APPDATA%\com.sparkinex.vocyl\settings.json`                     |
| **macOS**   | `~/Library/Application Support/com.sparkinex.vocyl/settings.json` |
| **Linux**   | `~/.config/com.sparkinex.vocyl/settings.json`                     |

Additional application data (e.g. cached files) will be stored in the same directory alongside `settings.json`.
