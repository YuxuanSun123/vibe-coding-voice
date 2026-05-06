# Vibe Coding Voice Native

[简体中文说明](README.md)

Vibe Coding Voice Native is a local-first Windows voice input tool for developers. It records speech from your microphone, transcribes it with a local SenseVoice-compatible model, and sends the result back to the active input field.

The app is built with Rust and egui. It is designed around Chinese and mixed Chinese/English developer dictation, global hotkeys, a small native UI, local microphone capture, a recording overlay, and a local transcription path that does not intentionally upload audio to a remote service.

## Status

This project is in alpha. It is useful for local experimentation and daily dogfooding, but model setup, input focus recovery, packaging, and release workflow are still being polished.

## Preview

![Home preview](docs/assets/home-preview.png)

![Dictation flow preview](docs/assets/demo-flow.gif)

The first image shows the main app layout. The GIF is a lightweight documentation preview of the dictation flow.

## Currently Supported

- Windows 10 or later
- Local microphone input
- Local SenseVoice-compatible transcription
- Online Qwen-ASR Realtime transcription
- Global shortcut-driven recording
- Two output modes: original text and code-edit prompt
- Copy and send actions for recognized text
- Optional automatic paste into the active input field
- Tray and recording overlay basics
- Manual local model directory selection from the settings page

## Not Yet Guaranteed

- Production-grade packaging and auto-update
- Cross-platform support
- Complete installer or in-app model downloader
- Stable extension APIs
- Local Whisper.cpp backend

Experimental work on additional ASR backends may happen later. The current public build supports a local SenseVoice-compatible model path and an online Qwen-ASR Realtime path.

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

- In local mode, audio is intended to be processed locally by the configured model.
- In online mode, recorded audio is sent to the configured DashScope/Qwen-ASR Realtime service.
- Clipboard and keyboard simulation are used for copy, paste, and send workflows.
- The recording overlay is launched through `recording-overlay.ps1`.
- Review the source, dependencies, and model license before using this with sensitive code, credentials, or private documents.

## Requirements

- Windows 10 or later
- Rust stable toolchain with edition 2024 support
- PowerShell
- A working microphone
- A local SenseVoice-compatible model directory
- A DashScope API Key if you use the online model path

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

Model files are not included in this repository. You need to download a local SenseVoice-compatible model and point the app to that folder.

Recommended model:

```text
sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Official model reference: [sherpa-onnx SenseVoice pre-trained models](https://k2-fsa.github.io/sherpa/onnx/sense-voice/pretrained.html).

Release builds first look for a model folder next to the executable:

```text
models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Recommended directory shape:

```text
vibe-coding-voice-native-windows-v0.1.0-alpha.1/
├── vibe-coding-voice-native.exe
└── models/
    └── sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
        ├── model.int8.onnx
        └── tokens.txt
```

When running from source, you can also place the model in a sibling `official-models` directory:

```text
official-models/
└── sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
    ├── model.int8.onnx
    └── tokens.txt
```

Download example:

```powershell
mkdir ..\official-models
cd ..\official-models
curl.exe -L -O https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
tar -xjf sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
cd ..\vibe-coding-voice
```

Then open the app:

1. Click the settings icon.
2. Confirm the model directory, or click `选择文件夹` to choose the downloaded model folder.
3. Click `检查 SenseVoice`.
4. Return to the home page and start recording.

See [docs/MODELS.md](docs/MODELS.md) for detailed download, directory layout, compatibility, and troubleshooting notes.

## Online Model

The settings page can switch between `本地模型` and `在线模型`. For the online path, configure:

- DashScope API Key
- Model name, default `qwen3-asr-flash-realtime`
- WebSocket URL, default `wss://dashscope.aliyuncs.com/api-ws/v1/realtime`
- Language, default `zh`

The online path uses Qwen-ASR Realtime in Manual mode. After recording finishes, the app sends 16 kHz PCM audio over WebSocket, waits for the final transcript, and then copies or delivers the resulting text.

## Model Licenses

The MIT license in this repository applies to this project's source code. It does not apply to third-party model weights.

Before using, redistributing, or packaging a model, check:

- The model license
- The `LICENSE` file inside the downloaded model directory, if present
- Whether commercial use is allowed
- Whether redistribution is allowed
- Whether attribution is required
- Whether converted ONNX artifacts can be shared

## Third-Party Notices

This project uses open-source Rust crates and local model/runtime ecosystems. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for dependency and model notice guidance.

## Project Structure

```text
.
├── src/
│   ├── app.rs                 # Main UI, settings page, recording panel, result panel
│   ├── hotkeys.rs             # Global shortcut registration and event handling
│   ├── main.rs                # Window, fonts, app initialization
│   ├── sensevoice.rs          # Model probing, compatibility helpers, transcription
│   ├── services.rs            # Recording, transcription, delivery, overlay orchestration
│   ├── state.rs               # App state and defaults
│   └── tray.rs                # Tray integration
├── assets/                    # App-owned bundled UI assets
├── docs/
├── recording-overlay.ps1      # Lightweight recording status overlay
├── run-native.ps1             # Local run helper
├── Cargo.toml
├── Cargo.lock
└── README.md
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
cargo test
```

## Roadmap

- Improve release packaging for non-developer users
- Add clearer model setup diagnostics
- Improve focus restoration after recording
- Add automated release builds
- Explore cross-platform support after the Windows workflow is stable
- Revisit optional ASR backends after the default SenseVoice workflow is stable

## Contributing

Contributions are welcome while the project is still young. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening issues or pull requests.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
