# Third-Party Licenses

Vocyl incorporates or interacts with the following third-party components.
This document provides license information and attribution as required by
Apache-2.0 and as good practice for open-source software.

> **Key for "Use type":**
>
> - **Compiled in** — statically or dynamically linked into the Vocyl binary
> - **Runtime download** — downloaded at the user's request, NOT bundled in the installer
> - **Subprocess** — executed as a separate OS process; not linked into Vocyl in any way

---

## Rust Dependencies

### Tauri

- **Repository:** https://github.com/tauri-apps/tauri
- **Version used:** 2.x
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in
- **Copyright:** 2019-2024 Tauri Programme within The Commons Conservancy

```
MIT License

Copyright 2019-2024 Tauri Programme within The Commons Conservancy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

### tauri-plugin-single-instance, tauri-plugin-opener, tauri-plugin-global-shortcut, tauri-plugin-log

- **Repository:** https://github.com/tauri-apps/plugins-workspace
- **Version used:** 2.x
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in
- **Copyright:** 2019-2024 Tauri Programme within The Commons Conservancy

_(License text same as Tauri above)_

---

### cpal (Cross-Platform Audio Library)

- **Repository:** https://github.com/RustAudio/cpal
- **License:** Apache-2.0
- **Use type:** Compiled in

```
Apache License, Version 2.0
https://www.apache.org/licenses/LICENSE-2.0
```

---

### rodio

- **Repository:** https://github.com/RustAudio/rodio
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in

---

### reqwest

- **Repository:** https://github.com/seanmonstar/reqwest
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in
- **Copyright:** Sean McArthur

---

### tokio

- **Repository:** https://github.com/tokio-rs/tokio
- **License:** MIT
- **Use type:** Compiled in
- **Copyright:** Tokio Contributors

---

### futures-util

- **Repository:** https://github.com/rust-lang/futures-rs
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in

---

### serde

- **Repository:** https://github.com/serde-rs/serde
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in
- **Copyright:** 2016 Erick Tryzelaar and David Tolnay

---

### serde_json

- **Repository:** https://github.com/serde-rs/json
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in

---

### log

- **Repository:** https://github.com/rust-lang/log
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in
- **Copyright:** The Rust Project Developers

---

### enigo

- **Repository:** https://github.com/enigo-rs/enigo
- **License:** MIT
- **Use type:** Compiled in

---

### arboard

- **Repository:** https://github.com/1Password/arboard
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in
- **Copyright:** 1Password

---

### hound

- **Repository:** https://github.com/ruuda/hound
- **License:** Apache-2.0
- **Use type:** Compiled in (Windows only, for WAV writing)
- **Copyright:** 2015 Ruud van Asseldonk

---

### zip

- **Repository:** https://github.com/zip-rs/zip2
- **License:** MIT
- **Use type:** Compiled in

---

### bzip2

- **Repository:** https://github.com/alexcrichton/bzip2-rs
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in

---

### tar

- **Repository:** https://github.com/alexcrichton/tar-rs
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in

---

### flate2

- **Repository:** https://github.com/rust-lang/flate2-rs
- **License:** MIT OR Apache-2.0
- **Use type:** Compiled in

---

### sherpa-onnx (Rust crate)

- **Repository:** https://github.com/k2-fsa/sherpa-onnx
- **License:** Apache-2.0
- **Use type:** Compiled in (provides Kokoro TTS inference via ONNX Runtime)
- **Copyright:** 2023-2024 k2-fsa / Next-gen Kaldi

```
Apache License, Version 2.0
https://www.apache.org/licenses/LICENSE-2.0
```

---

### whisper-rs (Linux/macOS only)

- **Repository:** https://github.com/tazz4843/whisper-rs
- **License:** MIT
- **Use type:** Compiled in (non-Windows only; wraps whisper.cpp via bindgen)

---

## External Binaries / Data Downloaded at Runtime

> These components are **NOT** bundled with Vocyl. They are downloaded
> from the internet at the user's explicit request. Users who download
> them are subject to the respective licenses.

---

### whisper.cpp (Windows: whisper-cli.exe subprocess)

- **Repository:** https://github.com/ggml-org/whisper.cpp
- **License:** MIT
- **Use type:** Runtime download + Subprocess (Windows only)
- **Copyright:** 2022 Georgi Gerganov

```
MIT License

Copyright (c) 2022-2024 Georgi Gerganov

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

### Whisper GGML Models

- **Source:** https://huggingface.co/ggerganov/whisper.cpp
- **License:** MIT (original OpenAI Whisper models)
- **Use type:** Runtime download (models loaded by whisper-rs or whisper-cli.exe)
- **Copyright:** 2022 OpenAI

The GGML-converted whisper models follow the same MIT license as the original
OpenAI Whisper models. They are NOT bundled with or compiled into Vocyl.

---

### Kokoro TTS Model (kokoro-en-v0_19)

- **Source:** https://github.com/k2-fsa/sherpa-onnx/releases (model package)
- **Model weights/voices/tokens:** Apache-2.0
- **Use type:** Runtime download (model files loaded by sherpa-onnx at runtime)

**⚠️ Important — espeak-ng data files (GPL-3.0):**

The Kokoro model package (`kokoro-en-v0_19.tar.bz2`) includes espeak-ng
phonemizer data (`espeak-ng-data/`). The espeak-ng project is licensed
under **GPL-3.0-or-later**.

- **espeak-ng repository:** https://github.com/espeak-ng/espeak-ng
- **espeak-ng license:** GPL-3.0-or-later
- **Copyright:** 2015-2022 David Beaumont and the espeak-ng contributors

The espeak-ng data files are downloaded as part of the Kokoro model package.
They are NOT compiled into Vocyl. They are loaded at runtime by the
sherpa-onnx library during text-to-speech synthesis. Users who download the
Kokoro model package receive GPL-3.0-licensed data as part of that package.

Full text of the GPL-3.0 license: https://www.gnu.org/licenses/gpl-3.0.html

---

### Piper TTS (optional, subprocess only)

- **Repository:** https://github.com/OHF-Voice/piper1-gpl
- **License:** ⚠️ **GPL-3.0-or-later**
- **Use type:** Runtime download + Subprocess (completely separate OS process)
- **Copyright:** Michael Hansen and OHF-Voice contributors

**Important licensing note:**

Piper is licensed under the **GNU General Public License v3.0 or later
(GPL-3.0-or-later)**. Vocyl downloads the Piper binary at the user's
explicit request and invokes it as a **completely separate operating system
process**, communicating only through standard input and temporary WAV files.
Vocyl does not link against, embed, or incorporate any part of Piper's source
code or compiled code.

Under the GPL, when the user downloads and runs the Piper binary, they receive
all rights under GPL-3.0 including the right to the source code. The complete
source code for Piper is available at:

https://github.com/OHF-Voice/piper1-gpl

Full text of the GPL-3.0 license: https://www.gnu.org/licenses/gpl-3.0.html

---

## JavaScript / Frontend Dependencies

All frontend dependencies are MIT licensed and fully compatible with
Apache-2.0. They are used only at build time (bundled into the frontend
assets) with no runtime linking implications.

| Package                   | Version | License           | Repository                                         |
| ------------------------- | ------- | ----------------- | -------------------------------------------------- |
| react                     | ^19     | MIT               | https://github.com/facebook/react                  |
| react-dom                 | ^19     | MIT               | https://github.com/facebook/react                  |
| react-hook-form           | ^7      | MIT               | https://github.com/react-hook-form/react-hook-form |
| tailwindcss               | ^4      | MIT               | https://github.com/tailwindlabs/tailwindcss        |
| @tailwindcss/vite         | ^4      | MIT               | https://github.com/tailwindlabs/tailwindcss        |
| clsx                      | ^2      | MIT               | https://github.com/lukeed/clsx                     |
| @tauri-apps/api           | ^2      | MIT OR Apache-2.0 | https://github.com/tauri-apps/tauri                |
| @tauri-apps/plugin-log    | ^2      | MIT OR Apache-2.0 | https://github.com/tauri-apps/plugins-workspace    |
| @tauri-apps/plugin-opener | ^2      | MIT OR Apache-2.0 | https://github.com/tauri-apps/plugins-workspace    |

---

## License Compatibility Summary

| Component          | License           | Compiled into Vocyl binary? | Notes                                 |
| ------------------ | ----------------- | --------------------------- | ------------------------------------- |
| Vocyl itself       | Apache-2.0        | —                           |                                       |
| All Rust crates    | MIT or Apache-2.0 | Yes                         | ✅ Compatible                         |
| all JS deps        | MIT               | Build-time bundle           | ✅ Compatible                         |
| whisper-cli.exe    | MIT               | No (subprocess, Windows)    | ✅ Compatible                         |
| Whisper models     | MIT               | No (runtime data)           | ✅ Compatible                         |
| sherpa-onnx        | Apache-2.0        | Yes                         | ✅ Compatible                         |
| Kokoro model files | Apache-2.0        | No (runtime data)           | ✅ Compatible                         |
| espeak-ng-data     | **GPL-3.0**       | No (runtime data)           | ⚠️ Data only, not linked              |
| Piper binary       | **GPL-3.0**       | No (subprocess)             | ⚠️ Separate process, arm's-length IPC |

**Vocyl's Apache-2.0 license is maintained** because the GPL-3.0 components
(Piper binary and espeak-ng data files) are neither compiled into nor linked
with the Vocyl binary. They are separate programs and data files downloaded
and managed by the user. Vocyl communicates with Piper exclusively through
subprocess IPC (stdin and temp files) which constitutes "mere aggregation"
under GPL-3.0 Section 5.

However, users who download Piper or the Kokoro model package (which includes
espeak-ng data) receive GPL-3.0-licensed software and are subject to those
terms.
