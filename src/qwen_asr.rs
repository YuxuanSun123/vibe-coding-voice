use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};
use tungstenite::client::IntoClientRequest;
use tungstenite::{Message, connect};

const MAX_APPEND_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone)]
pub struct QwenAsrConfig {
    pub api_key: String,
    pub model: String,
    pub url: String,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct QwenAsrTranscript {
    pub text: String,
}

pub fn validate_config(config: &QwenAsrConfig) -> Result<()> {
    if config.api_key.trim().is_empty() {
        bail!("请先填写 DashScope API Key。");
    }
    if config.model.trim().is_empty() {
        bail!("请先填写 Qwen-ASR 模型名。");
    }
    if config.url.trim().is_empty() {
        bail!("请先填写 Qwen-ASR WebSocket 地址。");
    }
    if !config.url.trim().starts_with("wss://") {
        bail!("Qwen-ASR WebSocket 地址需要以 wss:// 开头。");
    }
    Ok(())
}

pub fn transcribe_audio(config: &QwenAsrConfig, samples: Vec<f32>) -> Result<QwenAsrTranscript> {
    validate_config(config)?;
    if samples.is_empty() {
        bail!("录音音频为空。");
    }

    let pcm = samples_to_pcm16_le(&samples);
    let base_url = config.url.trim().trim_end_matches('/');
    let separator = if base_url.contains('?') { "&" } else { "?" };
    let endpoint = format!("{base_url}{separator}model={}", config.model.trim());
    let mut request = endpoint
        .as_str()
        .into_client_request()
        .with_context(|| format!("创建 Qwen-ASR WebSocket 请求失败: {endpoint}"))?;
    request.headers_mut().insert(
        "Authorization",
        format!("bearer {}", config.api_key.trim())
            .parse()
            .context("生成 DashScope Authorization header 失败")?,
    );

    let (mut socket, _) = connect(request).context("连接 Qwen-ASR Realtime 失败")?;
    send_json(
        &mut socket,
        json!({
            "event_id": event_id("session"),
            "type": "session.update",
            "session": {
                "input_audio_format": "pcm",
                "sample_rate": 16000,
                "input_audio_transcription": {
                    "language": config.language.trim()
                },
                "turn_detection": null
            }
        }),
    )?;

    for chunk in pcm.chunks(MAX_APPEND_BYTES) {
        send_json(
            &mut socket,
            json!({
                "event_id": event_id("audio"),
                "type": "input_audio_buffer.append",
                "audio": general_purpose::STANDARD.encode(chunk)
            }),
        )?;
    }

    send_json(
        &mut socket,
        json!({
            "event_id": event_id("commit"),
            "type": "input_audio_buffer.commit"
        }),
    )?;
    send_json(
        &mut socket,
        json!({
            "event_id": event_id("finish"),
            "type": "session.finish"
        }),
    )?;

    let mut final_text = String::new();
    loop {
        let message = socket.read().context("读取 Qwen-ASR 响应失败")?;
        let Message::Text(text) = message else {
            continue;
        };
        let payload: Value = serde_json::from_str(&text)
            .with_context(|| format!("解析 Qwen-ASR 响应失败: {text}"))?;
        let event_type = payload
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();

        match event_type {
            "error" => bail!(format_qwen_error(&payload)),
            "conversation.item.input_audio_transcription.completed" => {
                if let Some(transcript) = payload.get("transcript").and_then(Value::as_str) {
                    final_text = transcript.trim().to_string();
                }
            }
            "conversation.item.input_audio_transcription.failed" => {
                bail!(format_qwen_error(&payload));
            }
            "session.finished" => break,
            _ => {}
        }
    }

    let _ = socket.close(None);
    if final_text.is_empty() {
        bail!("Qwen-ASR 没有返回最终识别文本。");
    }

    Ok(QwenAsrTranscript { text: final_text })
}

fn send_json(
    socket: &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
    payload: Value,
) -> Result<()> {
    socket
        .send(Message::Text(payload.to_string().into()))
        .context("发送 Qwen-ASR WebSocket 事件失败")
}

fn samples_to_pcm16_le(samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn format_qwen_error(payload: &Value) -> String {
    let error = payload.get("error").unwrap_or(payload);
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("未知错误");
    let param = error.get("param").and_then(Value::as_str).unwrap_or("");
    if param.is_empty() {
        format!("Qwen-ASR 返回错误: {code}: {message}")
    } else {
        format!("Qwen-ASR 返回错误: {code}: {message} ({param})")
    }
}

fn event_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{prefix}_{millis}")
}
