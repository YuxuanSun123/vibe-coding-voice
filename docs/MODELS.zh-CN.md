# 模型安装指南

Vibe Coding Voice Native 不随仓库打包语音识别模型。你需要自行下载 SenseVoice 兼容模型，并在应用设置里指向本地模型目录。

当前公开版本只启用了 SenseVoice 兼容路径。Whisper.cpp 和 Qwen3-ASR 暂未在界面和运行时启用。

## 推荐模型

目前测试最多的是 sherpa-onnx 的 SenseVoice INT8 模型包：

```text
sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

官方说明：[sherpa-onnx SenseVoice pre-trained models](https://k2-fsa.github.io/sherpa/onnx/sense-voice/pretrained.html)。

模型文件夹里应该直接包含：

```text
sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
  model.int8.onnx
  tokens.txt
```

这个模型族适合普通话、英文、日语、韩语和粤语场景。对中文和中英混合的编程口述来说，它是一个比较实用的默认选择。

## 下载

在仓库目录中运行：

```powershell
mkdir ..\official-models
cd ..\official-models
curl.exe -L -O https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
tar -xjf sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
cd ..\vibe-coding-voice
```

应用默认模型路径是：

```text
../official-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

例如：

```text
workspace/
  vibe-coding-voice/
  official-models/
    sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/
      model.int8.onnx
      tokens.txt
```

如果你把模型放在其他位置，可以在应用设置里手动选择该文件夹。

## 在应用里配置

1. 启动应用。
2. 点击右上角设置图标。
3. 在 `模型目录` 中粘贴路径，或点击 `选择文件夹`。
4. 选择直接包含 `model.int8.onnx` 和 `tokens.txt` 的文件夹。
5. 点击 `检查 SenseVoice`。
6. 如果模型有效，设置页会显示模型文件、token 数量、来源格式和加载策略。

正确示例：

```text
C:\models\sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17
```

错误示例：

```text
C:\models
```

## 旧 FunASR 风格目录

应用也保留了旧导出格式的兼容路径：

```text
your-model-dir/
  model.onnx
  config.yaml
  tokens.json
```

检测到这种结构时，应用会尝试在 `tokens.json` 旁边生成 `tokens.txt`。

这不保证模型一定能加载。有些 ONNX 导出缺少 `transcribe-rs` 需要的元数据，比如 `vocab_size`。如果遇到这种情况，建议换用 sherpa-onnx 兼容导出，或重新转换模型并保留必要元数据。

## 模型许可证说明

模型文件通常很大，因此不会放进这个仓库。

下载、使用或分发任何模型前，请确认：

- 模型许可证
- 商业使用是否允许
- 再分发是否允许
- 是否需要署名
- 转换或量化后的文件是否可以分享

本仓库的 MIT 许可证只适用于项目源码，不授予第三方模型权重的使用权。

如果下载后的模型目录中包含 `LICENSE` 或 `README.md`，请保留这些文件，并在再分发或商业使用前仔细阅读。

## 常见问题

### 应用找不到 `model.int8.onnx` 或 `model.onnx`

请确认配置路径指向模型文件夹本身，而不是它的上一级目录。

### 应用提示缺少 `tokens.json`

你可能正在使用旧兼容路径。可以补上 `tokens.json`，也可以改用推荐的 sherpa-onnx 目录结构，也就是使用 `tokens.txt`。

### 应用提示 ONNX 元数据缺失

模型可能是原始 FunASR 导出。请使用 sherpa-onnx 兼容的 SenseVoice 导出，或重新转换模型并保留所需 metadata。

### 转写很慢

优先使用 INT8 模型。CPU 性能、音频长度和模型大小都会影响延迟。

## 不应该提交到仓库的文件

不要提交这些内容：

```text
official-models/
models/
*.onnx
*.wav
*.mp3
```

`.gitignore` 已经排除了常见模型和音频文件。
