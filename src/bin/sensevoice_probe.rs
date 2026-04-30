#[path = "../sensevoice.rs"]
mod sensevoice;

use anyhow::Result;
use std::path::PathBuf;
use transcribe_rs::audio::read_wav_samples;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    let model_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(sensevoice::default_model_dir);
    let wav_path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| model_dir.join("test_wavs").join("zh.wav"));

    let probe = sensevoice::prepare_and_probe(&model_dir)?;

    println!("SenseVoice 模型检查通过。");
    println!("{}", sensevoice::format_probe_summary(&probe));
    if wav_path.exists() {
        let samples = read_wav_samples(&wav_path)
            .map_err(|error| anyhow::anyhow!("读取测试 wav 失败: {}", error))?;
        let transcript = sensevoice::transcribe_audio(&model_dir, samples)?;
        println!("测试音频: {}", wav_path.display());
        println!("测试转写: {}", transcript.text);
        println!("分段数量: {}", transcript.segment_count);
    }

    Ok(())
}
