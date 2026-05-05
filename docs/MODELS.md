# Model Setup Guide

Vibe Coding Voice Native does not bundle speech recognition models. You need to download or export a SenseVoice-compatible model yourself and point the app to that local directory.

## Recommended Model Shape

The most tested path is a sherpa-onnx style SenseVoice directory:

```text
official-models/
└─ sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
   ├─ model.int8.onnx
   └─ tokens.txt
```

By default, the app looks for this directory next to the repository:

```text
../official-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

For example:

```text
workspace/
├─ vibe-coding-voice-native/
└─ official-models/
   └─ sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
      ├─ model.int8.onnx
      └─ tokens.txt
```

## Legacy FunASR-Style Directory

The app also has a compatibility path for older export layouts:

```text
your-model-dir/
├─ model.onnx
├─ config.yaml
└─ tokens.json
```

When this layout is detected, the app attempts to generate a `tokens.txt` file next to `tokens.json`.

This does not guarantee the model can be loaded. Some ONNX exports do not include metadata such as `vocab_size`, which `transcribe-rs` needs. If that happens, use a sherpa-onnx compatible export or convert the model again with the required metadata.

## Download And License Notes

Model files are large and are intentionally excluded from this repository.

Before using or redistributing any model, check:

- The model license
- Whether commercial use is allowed
- Whether redistribution is allowed
- Whether converted ONNX artifacts can be shared
- Whether attribution is required

The MIT license in this repository only applies to this project's source code. It does not grant rights to third-party model weights.

## Configure The App

1. Place the model directory somewhere outside the repository, or in a sibling `official-models` directory.
2. Start the app.
3. Open settings.
4. Set the model directory if the default path is not correct.
5. Click the model check button.

If loading succeeds, the settings page will show the detected model file, token count, source format, and loading strategy.

## Troubleshooting

### The app cannot find `model.int8.onnx` or `model.onnx`

Check that the configured directory points to the model folder itself, not its parent folder.

Correct:

```text
C:\models\sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

Incorrect:

```text
C:\models
```

### The app says `tokens.json` is missing

You are probably using the legacy compatibility path. Either add `tokens.json` to that directory or switch to the recommended sherpa-onnx style layout with `tokens.txt`.

### The app says ONNX metadata is missing

The model may be a raw FunASR export. Use a sherpa-onnx compatible SenseVoice export, or convert the model with the metadata required by `transcribe-rs`.

### Transcription is slow

Use an int8 model when possible. CPU speed, audio length, and model size all affect latency.

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
