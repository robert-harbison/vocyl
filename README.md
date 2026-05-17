<br>

<div style="text-align: center;">
  <img src="src/assets/Vocyl_Logo.png" alt="Vocyl Logo" width="200"/>
</div>

<br>

# Vocyl

Vocyl is a work-in-progress desktop application for fully local, offline text-to-speech and speech-to-text.

The goal of Vocyl is to provide fast, high-quality voice tooling that runs entirely on the user's machine without requiring cloud services or subscriptions.

Planned platform support:

- Windows
- Linux (Wayland support is best effort)
- macOS

## Project Status

Vocyl is currently in very early development and is not ready for production use.

Public releases are not yet available. Once the project reaches a more stable state, installation instructions, documentation, and prebuilt binaries will be added here.

## License

Vocyl is licensed under the [Apache License, Version 2.0](LICENSE).

### Third-party components

Vocyl integrates with several third-party open-source components. Full license details are in [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) and [NOTICE](NOTICE).

**Compiled into the Vocyl binary** (MIT or Apache-2.0, fully compatible):
[Tauri](https://tauri.app), [cpal](https://github.com/RustAudio/cpal), [rodio](https://github.com/RustAudio/rodio), [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx), [whisper-rs](https://github.com/tazz4843/whisper-rs) (Linux/macOS), [tokio](https://tokio.rs), [reqwest](https://github.com/seanmonstar/reqwest), [serde](https://serde.rs), [enigo](https://github.com/enigo-rs/enigo), [arboard](https://github.com/1Password/arboard), [hound](https://github.com/ruuda/hound), and others.

**Downloaded at the user's request (separate from the Vocyl binary):**

| Component                                                  | License     | Notes                                                                               |
| ---------------------------------------------------------- | ----------- | ----------------------------------------------------------------------------------- |
| [whisper.cpp CLI](https://github.com/ggml-org/whisper.cpp) | MIT         | Windows only; run as a separate subprocess.                                         |
| Kokoro model (model/voices/tokens)                         | Apache-2.0  | ONNX model weights.                                                                 |
| espeak-ng data (bundled in Kokoro package)                 | **GPL-3.0** | Phonemizer data files. Source: [espeak-ng](https://github.com/espeak-ng/espeak-ng). |
| Whisper GGML models                                        | MIT         | Original models by OpenAI.                                                          |

> **Note on GPL-3.0 components:** Espeak-ng phonemizer data files
> (bundled in the Kokoro model package) are GPL-3.0 licensed. They are never
> compiled into or linked with the Vocyl binary. Vocyl's Apache-2.0 license applies only to the Vocyl
> source code and binary. Users who download the Kokoro model package
> receive GPL-3.0-licensed software and are subject to those terms.
> Source code for both is freely available at the links above.
