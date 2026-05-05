# Model Setup Guide

Vibe Coding Voice Native does not bundle speech recognition models. You need to download a SenseVoice-compatible model yourself and point the app to that local directory.

The current public app supports the SenseVoice-compatible path only. Whisper.cpp and Qwen3-ASR are not enabled in the UI or runtime yet.

## Recommended Model

The most tested model layout is the sherpa-onnx SenseVoice INT8 package:

```text
sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Official reference: [sherpa-onnx SenseVoice pre-trained models](https://k2-fsa.github.io/sherpa/onnx/sense-voice/pretrained.html).

Expected files:

```text
sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
├── model.int8.onnx
└── tokens.txt
```

This model family is suitable for Mandarin Chinese, English, Japanese, Korean, and Cantonese workflows. It is a practical default for mixed Chinese/English developer dictation.

## Download

From the repository directory, run:

```powershell
mkdir ..\official-models
cd ..\official-models
curl.exe -L -O https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
tar -xjf sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
cd ..\vibe-coding-voice
```

The app's default model path is:

```text
../official-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

For example:

```text
workspace/
├── vibe-coding-voice/
└── official-models/
    └── sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
        ├── model.int8.onnx
        └── tokens.txt
```

If you place the model somewhere else, open the app settings and choose that folder manually.

## Configure The App

1. Start the app.
2. Click the settings icon in the top-right corner.
3. In `模型目录`, either paste the model folder path or click `选择文件夹`.
4. Select the folder that directly contains `model.int8.onnx` and `tokens.txt`.
5. Click `检查 SenseVoice`.
6. If the model is valid, the settings page shows the detected model file, token count, source format, and loading strategy.

Correct:

```text
C:\models\sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Incorrect:

```text
C:\models
```

## Legacy FunASR-Style Directory

The app also has a compatibility path for older export layouts:

```text
your-model-dir/
├── model.onnx
├── config.yaml
└── tokens.json
```

When this layout is detected, the app attempts to generate a `tokens.txt` file next to `tokens.json`.

This does not guarantee the model can be loaded. Some ONNX exports do not include metadata such as `vocab_size`, which `transcribe-rs` needs. If that happens, use a sherpa-onnx compatible export or convert the model again with the required metadata.

## Model License Notes

Model files are large and are intentionally excluded from this repository.

Before using or redistributing any model, check:

- The model license
- Whether commercial use is allowed
- Whether redistribution is allowed
- Whether converted ONNX artifacts can be shared
- Whether attribution is required

The MIT license in this repository only applies to this project's source code. It does not grant rights to third-party model weights.

If the downloaded model directory includes its own `LICENSE` or `README.md`, keep those files with the model and review them before redistribution or commercial use.

## Troubleshooting

### The app cannot find `model.int8.onnx` or `model.onnx`

Check that the configured path points to the model folder itself, not its parent folder.

### The app says `tokens.json` is missing

You are probably using the legacy compatibility path. Either add `tokens.json` to that directory or switch to the recommended sherpa-onnx layout with `tokens.txt`.

### The app says ONNX metadata is missing

The model may be a raw FunASR export. Use a sherpa-onnx compatible SenseVoice export, or convert the model with the metadata required by `transcribe-rs`.

### Transcription is slow

Use an INT8 model when possible. CPU speed, audio length, and model size all affect latency.

## Files That Should Not Be Committed

The repository ignores common model and audio artifacts:

```text
*.onnx
*.wav
*.mp3
*.flac
*.m4a
official-models/
models/
```

Keep model files and private recordings out of pull requests.
