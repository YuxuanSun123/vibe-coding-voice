# Vibe Coding Voice Native

Vibe Coding Voice Native is a local-first Windows voice input tool for developers. It lets you start and stop dictation with a global shortcut, transcribe speech with a local SenseVoice-compatible model, and send the result back to the active input field.

The app is built with Rust and egui. It focuses on Chinese developer dictation, lightweight native UI, global hotkeys, local microphone capture, a recording overlay, and a SenseVoice transcription path that can run without sending audio to a remote service.

## Status

This project is in alpha. It is useful for local experimentation and daily dogfooding, but the model setup, input focus recovery, packaging, and release workflow are still being polished.

## Preview

![Home preview](docs/assets/home-preview.png)

![Dictation flow preview](docs/assets/demo-flow.gif)

The first image shows the main app layout. The GIF is a lightweight documentation preview of the dictation flow.

Currently supported:

- Windows
- Local microphone input
- Local SenseVoice-compatible transcription
- Global shortcut-driven recording
- Copy and send actions for recognized text
- Optional automatic paste into the active input field
- Tray and recording overlay basics

Not yet guaranteed:

- Production-grade packaging and auto-update
- Cross-platform support
- Complete installer or model downloader
- Stable extension APIs

## Features

- Native desktop UI built with `eframe` / `egui`
- Global shortcut to start and finish recording
- Local microphone capture through `cpal`
- SenseVoice-compatible model probing and transcription through `transcribe-rs`
- Result editor, copy button, send button, and auto-paste toggle
- Recording and processing overlay implemented with PowerShell
- Windows tray integration
- Chinese font loading to avoid broken CJK rendering

## Privacy And Security

- Audio is intended to be processed locally by the configured model.
- The app does not intentionally upload recordings or transcripts to a remote service.
- Clipboard and keyboard simulation are used for copy, paste, and send workflows.
- The recording overlay is launched through `recording-overlay.ps1`.
- Review the source and dependencies before using this with sensitive code, credentials, or private documents.

## Requirements

- Windows 10 or later
- Rust stable toolchain with edition 2024 support
- PowerShell
- A working microphone
- A local SenseVoice-compatible model directory

## Quick Start

Clone the repository and run the app:

```powershell
git clone https://github.com/YuxuanSun123/vibe-coding-voice.git
cd vibe-coding-voice
cargo run --bin vibe-coding-voice-native
```

You can also use the helper script:

```powershell
.\run-native.ps1
```

Run checks during development:

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
```

## Model Setup

The repository does not include model files. You need to prepare a local SenseVoice-compatible model yourself.

See [docs/MODELS.md](docs/MODELS.md) for detailed download, directory layout, compatibility, and troubleshooting notes.

By default, the app looks for this sibling directory:

```text
../official-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

The preferred directory shape is:

```text
official-models/
└─ sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
   ├─ model.int8.onnx
   └─ tokens.txt
```

The app also has a compatibility path for older FunASR-style directories:

```text
your-model-dir/
├─ model.onnx
├─ config.yaml
└─ tokens.json
```

In that case, the app attempts to generate a compatible `tokens.txt`. Some ONNX exports may still fail if they do not include the metadata required by `transcribe-rs`.

Model licenses are separate from this repository's source license. Check the license and redistribution terms for any model you download or convert.

## Project Structure

```text
.
├─ src/
│  ├─ app.rs                 # Main UI, settings page, recording panel, result panel
│  ├─ hotkeys.rs             # Global shortcut registration and event handling
│  ├─ main.rs                # Window, fonts, app initialization
│  ├─ sensevoice.rs          # Model probing, compatibility helpers, transcription
│  ├─ services.rs            # Recording, transcription, delivery, overlay orchestration
│  ├─ state.rs               # App state and defaults
│  └─ tray.rs                # Tray integration
├─ recording-overlay.ps1     # Lightweight recording status overlay
├─ run-native.ps1            # Local run helper
├─ Cargo.toml
├─ Cargo.lock
└─ README.md
```

## Development Notes

Important entry points:

- `src/main.rs`: app bootstrap, fonts, native window configuration
- `src/app.rs`: visual layout, controls, pages, and interaction state
- `src/services.rs`: recording pipeline, transcription flow, and text delivery
- `src/sensevoice.rs`: model directory detection and transcription engine loading
- `recording-overlay.ps1`: standalone overlay process used while recording or processing

Before opening a pull request, run:

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
```

## Roadmap

- Improve release packaging for non-developer users
- Add screenshots and short demo GIFs
- Add clearer model setup diagnostics
- Improve focus restoration after recording
- Add automated release builds
- Explore cross-platform support after the Windows workflow is stable

## Contributing

Contributions are welcome while the project is still young. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening issues or pull requests.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
