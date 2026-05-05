# Contributing

Thanks for helping improve Vibe Coding Voice Native.

## Project Stage

The project is in alpha. Small, focused pull requests are easiest to review. Please avoid broad rewrites unless an issue or discussion has already agreed on the direction.

## Development Setup

Requirements:

- Windows
- Rust stable toolchain with edition 2024 support
- PowerShell
- A microphone
- Optional: a local SenseVoice-compatible model directory for transcription testing

Run the app:

```powershell
cargo run --bin vibe-coding-voice-native
```

Run checks:

```powershell
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
```

## Pull Request Checklist

Before opening a pull request:

- Keep the change scoped to one topic.
- Run formatting and checks.
- Update `README.md` when setup, behavior, or user-facing workflows change.
- Do not commit model files, recordings, generated binaries, local paths, or secrets.
- Mention the Windows version and model directory shape you tested when relevant.

## Issue Guidelines

For bugs, include:

- Windows version
- App version or commit SHA
- Steps to reproduce
- Expected behavior
- Actual behavior
- Model directory shape, without uploading proprietary model files
- Screenshots or short screen recordings when UI behavior matters

For feature requests, explain the workflow you want to improve and why the current behavior is not enough.

## Code Style

- Follow existing Rust and egui patterns in the repository.
- Prefer small functions over large UI rewrites.
- Keep platform-specific behavior explicit.
- Avoid adding network behavior unless it is clearly documented and optional.
