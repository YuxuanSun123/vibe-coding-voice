# Third-Party Notices

This file summarizes third-party components and model-related notice boundaries for Vibe Coding Voice Native.

## Source Code License

The source code in this repository is licensed under the MIT License. See [LICENSE](LICENSE).

## Rust Dependencies

This project depends on open-source Rust crates listed in `Cargo.toml` and locked in `Cargo.lock`, including:

- `anyhow` for error handling
- `eframe` / `egui` for the native UI
- `cpal` for microphone capture
- `transcribe-rs` for the SenseVoice transcription path
- `ort` and ONNX Runtime binaries for local ONNX model execution
- `tungstenite`, `native-tls`, and `base64` for the Qwen-ASR Realtime WebSocket path
- `global-hotkey` for global shortcut registration
- `tray-icon` for Windows tray integration
- `arboard` and `enigo` for clipboard and keyboard delivery workflows
- `serde_json` for JSON parsing and serialization
- `windows-sys` for Windows API calls

Each dependency is governed by its own license. Check the crate metadata from crates.io or run a license audit tool such as `cargo-deny` before redistributing binaries in a stricter compliance environment.

## Local PowerShell Overlay

`recording-overlay.ps1` is part of this project. It is included in release bundles because the desktop app launches it to show recording and processing status.

## App Assets And Documentation Media

Files under `assets/` and `docs/assets/` are project-owned UI/documentation assets unless a file explicitly states otherwise. They are included only to support the app UI and README documentation.

## Speech Recognition Models

Model weights are not included in this repository and are not covered by this repository's MIT License.

The recommended SenseVoice-compatible model is downloaded separately from the sherpa-onnx model releases:

```text
sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Reference: [sherpa-onnx SenseVoice pre-trained models](https://k2-fsa.github.io/sherpa/onnx/sense-voice/pretrained.html).

Before using, redistributing, or bundling any model, review that model's license and terms. In particular, confirm:

- the contents of the downloaded model's `LICENSE` and `README.md`, if present
- whether commercial use is permitted
- whether redistribution is permitted
- whether attribution is required
- whether converted or quantized artifacts can be shared

## Online Speech Recognition Services

When users choose the online Qwen-ASR Realtime path, audio is sent to the configured DashScope WebSocket endpoint. Users are responsible for their own API key, account terms, service region, data handling requirements, and usage costs.

## Not Bundled

The repository intentionally excludes:

- ONNX model files
- local model directories
- microphone recordings
- generated audio files
- environment files and logs

See `.gitignore` for the current exclusion list.
