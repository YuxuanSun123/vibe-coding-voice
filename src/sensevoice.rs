use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use transcribe_rs::{
    TranscriptionEngine,
    engines::sense_voice::{
        Language, SenseVoiceEngine, SenseVoiceInferenceParams, SenseVoiceModelParams,
    },
};

const DEFAULT_SENSEVOICE_MODEL_DIR_NAME: &str =
    "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17";

#[derive(Debug, Clone)]
pub struct SenseVoiceProbe {
    pub model_dir: PathBuf,
    pub model_file: PathBuf,
    pub tokens_txt_file: PathBuf,
    pub tokens_count: usize,
    pub source_format: &'static str,
    pub load_strategy: &'static str,
}

#[derive(Debug, Clone)]
pub struct SenseVoiceTranscript {
    pub text: String,
    pub segment_count: usize,
}

pub fn default_model_dir() -> PathBuf {
    let release_default = current_exe_dir()
        .join("models")
        .join(DEFAULT_SENSEVOICE_MODEL_DIR_NAME);
    if release_default.exists() {
        return release_default;
    }

    let release_official_models = current_exe_dir()
        .join("official-models")
        .join(DEFAULT_SENSEVOICE_MODEL_DIR_NAME);
    if release_official_models.exists() {
        return release_official_models;
    }

    development_model_dir()
}

pub fn prepare_and_probe(model_dir: &Path) -> Result<SenseVoiceProbe> {
    let model_dir = canonicalize_or_original(model_dir);
    let tokens_txt_file = model_dir.join("tokens.txt");
    let official_int8 = model_dir.join("model.int8.onnx");
    let official_fp32 = model_dir.join("model.onnx");
    let legacy_config = model_dir.join("config.yaml");
    let legacy_tokens_json = model_dir.join("tokens.json");

    if official_int8.exists() && tokens_txt_file.exists() {
        let tokens_count = count_tokens_from_txt(&tokens_txt_file)?;
        try_load_model(&model_dir, SenseVoiceModelParams::int8())?;

        return Ok(SenseVoiceProbe {
            model_dir,
            model_file: official_int8,
            tokens_txt_file,
            tokens_count,
            source_format: "官方 sherpa-onnx SenseVoice int8",
            load_strategy: "INT8 (`model.int8.onnx`) + 直接使用 `tokens.txt`",
        });
    }

    ensure_file_exists(&official_fp32)?;
    ensure_file_exists(&legacy_config)?;
    ensure_file_exists(&legacy_tokens_json)?;

    let tokens = load_tokens_from_json(&legacy_tokens_json)?;
    if tokens.is_empty() {
        bail!("`tokens.json` 为空，无法生成 `tokens.txt`。");
    }

    write_tokens_txt(&tokens_txt_file, &tokens)?;
    try_load_model(&model_dir, SenseVoiceModelParams::fp32())?;

    Ok(SenseVoiceProbe {
        model_dir,
        model_file: official_fp32,
        tokens_txt_file,
        tokens_count: tokens.len(),
        source_format: "旧 FunASR 风格目录",
        load_strategy: "FP32 (`model.onnx`) + 自动生成 `tokens.txt`",
    })
}

pub fn format_probe_summary(probe: &SenseVoiceProbe) -> String {
    [
        format!("模型目录: {}", probe.model_dir.display()),
        format!("模型文件: {}", probe.model_file.display()),
        format!("目录类型: {}", probe.source_format),
        format!("兼容词表: {}", probe.tokens_txt_file.display()),
        format!("token 数量: {}", probe.tokens_count),
        format!("加载策略: {}", probe.load_strategy),
    ]
    .join("\n")
}

pub fn transcribe_audio(model_dir: &Path, samples: Vec<f32>) -> Result<SenseVoiceTranscript> {
    let model_dir = canonicalize_or_original(model_dir);
    let params = detect_model_params(&model_dir)?;
    let mut engine = SenseVoiceEngine::new();
    engine
        .load_model_with_params(&model_dir, params)
        .map_err(|error| anyhow::anyhow!("加载 SenseVoice 失败: {}", error))?;

    let result = engine
        .transcribe_samples(
            samples,
            Some(SenseVoiceInferenceParams {
                language: Language::Auto,
                use_itn: true,
            }),
        )
        .map_err(|error| anyhow::anyhow!("SenseVoice 转写失败: {}", error))?;

    Ok(SenseVoiceTranscript {
        text: result.text.trim().to_string(),
        segment_count: result.segments.map(|segments| segments.len()).unwrap_or(0),
    })
}

fn ensure_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("缺少文件: {}", path.display());
    }

    Ok(())
}

fn load_tokens_from_json(path: &Path) -> Result<Vec<String>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("读取 `{}` 失败。", path.display()))?;
    let tokens: Vec<String> = serde_json::from_str(&content)
        .with_context(|| format!("解析 `{}` 失败，预期为字符串数组。", path.display()))?;
    Ok(tokens)
}

fn count_tokens_from_txt(path: &Path) -> Result<usize> {
    let content =
        fs::read_to_string(path).with_context(|| format!("读取 `{}` 失败。", path.display()))?;
    Ok(content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count())
}

fn write_tokens_txt(path: &Path, tokens: &[String]) -> Result<()> {
    let mut content = String::with_capacity(tokens.len() * 12);
    for (idx, token) in tokens.iter().enumerate() {
        content.push_str(token);
        content.push(' ');
        content.push_str(&idx.to_string());
        content.push('\n');
    }

    let should_write = match fs::read_to_string(path) {
        Ok(existing) => existing != content,
        Err(_) => true,
    };

    if should_write {
        fs::write(path, content).with_context(|| format!("写入 `{}` 失败。", path.display()))?;
    }

    Ok(())
}

fn try_load_model(model_dir: &Path, params: SenseVoiceModelParams) -> Result<()> {
    let mut engine = SenseVoiceEngine::new();
    engine
        .load_model_with_params(model_dir, params)
        .map_err(|error| {
            let error_text = error.to_string();
            if error_text.contains("Missing required metadata key: vocab_size") {
                anyhow::anyhow!(
                    "已生成 `tokens.txt`，但当前 `model.onnx` 缺少 `vocab_size` 等 ONNX metadata。\
这说明 `{}` 里的模型更接近 FunASR 原始导出，不是 `transcribe-rs` 当前可直接加载的 SenseVoice 目录。\
下一步需要把它转换成 sherpa/transcribe-rs 兼容导出，或改走 FunASR 官方 runtime。",
                    model_dir.display()
                )
            } else {
                anyhow::anyhow!(
                    "`transcribe-rs` 无法加载 `{}`: {}",
                    model_dir.display(),
                    error_text
                )
            }
        })?;
    engine.unload_model();
    Ok(())
}

fn detect_model_params(model_dir: &Path) -> Result<SenseVoiceModelParams> {
    let int8_path = model_dir.join("model.int8.onnx");
    let fp32_path = model_dir.join("model.onnx");

    if int8_path.exists() {
        Ok(SenseVoiceModelParams::int8())
    } else if fp32_path.exists() {
        Ok(SenseVoiceModelParams::fp32())
    } else {
        bail!(
            "在 `{}` 下找不到 `model.int8.onnx` 或 `model.onnx`。",
            model_dir.display()
        );
    }
}

fn canonicalize_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn current_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn development_model_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap_or(manifest_dir.as_path())
        .join("official-models")
        .join(DEFAULT_SENSEVOICE_MODEL_DIR_NAME)
}
