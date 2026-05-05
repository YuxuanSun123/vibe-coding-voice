use crate::sensevoice;
use crate::state::{DeliveryTarget, InputMode, InputState, NativeAppState};
use anyhow::{Context, Result, anyhow};
use arboard::Clipboard;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetForegroundWindow, PostMessageW, SetForegroundWindow, WM_CLOSE,
};

const TARGET_SAMPLE_RATE: u32 = 16_000;

struct RecordingSession {
    _stream: Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    started_at: Instant,
}

struct RecordingCapture {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    duration_secs: f32,
}

#[derive(Clone)]
pub struct FinishInputSnapshot {
    input_mode: InputMode,
    delivery_target: DeliveryTarget,
    auto_paste: bool,
    local_model_dir: String,
    local_model_ready: bool,
    local_model_status: String,
    local_model_summary: String,
}

pub struct FinishInputWork {
    normalized_samples: Vec<f32>,
    last_recording_info: String,
    snapshot: FinishInputSnapshot,
}

pub struct FinishInputResult {
    pub input_state: InputState,
    pub status_message: String,
    pub raw_text: Option<String>,
    pub delivered_text: Option<String>,
    pub local_model_ready: bool,
    pub local_model_status: String,
    pub local_model_summary: String,
    pub last_recording_info: String,
}

struct OverlayUiState {
    visible: bool,
    mode: String,
    level: f32,
    started_at_ms: u64,
    last_level_flush: Option<Instant>,
}

fn recording_slot() -> &'static Mutex<Option<RecordingSession>> {
    static RECORDING: OnceLock<Mutex<Option<RecordingSession>>> = OnceLock::new();
    RECORDING.get_or_init(|| Mutex::new(None))
}

fn overlay_slot() -> &'static Mutex<Option<Child>> {
    static OVERLAY: OnceLock<Mutex<Option<Child>>> = OnceLock::new();
    OVERLAY.get_or_init(|| Mutex::new(None))
}

fn overlay_state_path() -> PathBuf {
    std::env::temp_dir().join("vibe-coding-voice-overlay-state.json")
}

fn delivery_target_slot() -> &'static Mutex<Option<isize>> {
    static DELIVERY_TARGET: OnceLock<Mutex<Option<isize>>> = OnceLock::new();
    DELIVERY_TARGET.get_or_init(|| Mutex::new(None))
}

fn overlay_ui_slot() -> &'static Mutex<OverlayUiState> {
    static OVERLAY_UI: OnceLock<Mutex<OverlayUiState>> = OnceLock::new();
    OVERLAY_UI.get_or_init(|| {
        Mutex::new(OverlayUiState {
            visible: false,
            mode: "idle".to_string(),
            level: 0.0,
            started_at_ms: unix_now_ms(),
            last_level_flush: None,
        })
    })
}

pub fn prepare_recording_overlay_host() -> Result<()> {
    close_stale_overlay_host();
    set_overlay_state(false, "idle", 0.0, true)?;
    ensure_overlay_host_running()?;
    set_overlay_state(false, "idle", 0.0, true)?;
    Ok(())
}

pub fn run_practice_flow(state: &mut NativeAppState) {
    state.input_state = InputState::Processing;
    state.status_message = "正在演练原生输入法链路...".to_string();

    state.raw_text = normalize_text(&state.practice_text);
    state.delivered_text = transform_text(state.input_mode, &state.raw_text);
    let delivery_message =
        deliver_output(state).unwrap_or_else(|error| format!("投送失败: {error:#}"));
    state.input_state = InputState::Success;
    state.status_message = format!(
        "原生输入法演练已完成，目标为 {}。{}",
        state.delivery_target.label(),
        delivery_message
    );
}

pub fn begin_input(state: &mut NativeAppState) {
    match start_recording() {
        Ok(info) => {
            state.input_state = InputState::Recording;
            state.last_recording_info = info;

            if let Err(error) = show_recording_overlay("listening") {
                state.status_message = format!("录音已开始，但浮层启动失败: {error:#}");
            } else {
                state.status_message =
                    "已经开始真实录音。说完后再次按快捷键或点“结束并发送”即可完成。".to_string();
            }
        }
        Err(error) => {
            state.input_state = InputState::Error;
            state.status_message = format!("启动录音失败: {error:#}");
            state.last_recording_info = "录音没有启动。".to_string();
        }
    }
}

pub fn begin_finish_input(state: &mut NativeAppState) -> Result<FinishInputWork> {
    if state.input_state != InputState::Recording {
        return Err(anyhow!("当前没有正在进行的口述。"));
    }

    state.input_state = InputState::Processing;
    state.status_message = "正在停止录音并送入 SenseVoice...".to_string();
    let _ = show_recording_overlay("processing");

    let capture = match stop_recording() {
        Ok(capture) => capture,
        Err(error) => {
            hide_recording_overlay();
            return Err(error);
        }
    };

    let last_recording_info = format!(
        "录音完成，原始采样率 {} Hz，通道 {}，时长约 {:.2} 秒。",
        capture.sample_rate, capture.channels, capture.duration_secs
    );
    state.last_recording_info = last_recording_info.clone();

    let normalized_samples =
        resample_to_target(capture.samples, capture.sample_rate, capture.channels);
    if normalized_samples.is_empty() {
        hide_recording_overlay();
        return Err(anyhow!("录到的音频为空，请再试一次。"));
    }

    Ok(FinishInputWork {
        normalized_samples,
        last_recording_info,
        snapshot: FinishInputSnapshot {
            input_mode: state.input_mode,
            delivery_target: state.delivery_target,
            auto_paste: state.auto_paste,
            local_model_dir: state.local_model_dir.clone(),
            local_model_ready: state.local_model_ready,
            local_model_status: state.local_model_status.clone(),
            local_model_summary: state.local_model_summary.clone(),
        },
    })
}

pub fn run_finish_input_work(work: FinishInputWork) -> FinishInputResult {
    let FinishInputWork {
        normalized_samples,
        last_recording_info,
        snapshot,
    } = work;

    let mut local_model_ready = snapshot.local_model_ready;
    let mut local_model_status = snapshot.local_model_status.clone();
    let mut local_model_summary = snapshot.local_model_summary.clone();

    let result = (|| -> Result<(String, String, usize, String)> {
        if !local_model_ready {
            let probe =
                sensevoice::prepare_and_probe(std::path::Path::new(&snapshot.local_model_dir))?;
            local_model_ready = true;
            local_model_status = "SenseVoice 已可被 Rust provider 加载。".to_string();
            local_model_summary = sensevoice::format_probe_summary(&probe);
        }

        let transcript = sensevoice::transcribe_audio(
            std::path::Path::new(&snapshot.local_model_dir),
            normalized_samples,
        )?;
        let raw_text = normalize_text(&transcript.text);
        let delivered_text = transform_text(snapshot.input_mode, &raw_text);
        let delivery_message = deliver_output_from_snapshot(&snapshot, delivered_text.trim())?;
        Ok((
            raw_text,
            delivered_text,
            transcript.segment_count,
            delivery_message,
        ))
    })();

    match result {
        Ok((raw_text, delivered_text, segments, delivery_message)) => {
            hide_recording_overlay();
            FinishInputResult {
                input_state: InputState::Success,
                status_message: format!(
                    "SenseVoice 转写完成，已识别 {} 个字符，切出了 {} 段。{}",
                    raw_text.chars().count(),
                    segments,
                    delivery_message
                ),
                raw_text: Some(raw_text),
                delivered_text: Some(delivered_text),
                local_model_ready,
                local_model_status,
                local_model_summary,
                last_recording_info,
            }
        }
        Err(error) => {
            let _ = show_recording_overlay("error");
            thread::sleep(std::time::Duration::from_millis(900));
            hide_recording_overlay();
            FinishInputResult {
                input_state: InputState::Error,
                status_message: format!("SenseVoice 转写失败: {error:#}"),
                raw_text: None,
                delivered_text: None,
                local_model_ready,
                local_model_status,
                local_model_summary,
                last_recording_info,
            }
        }
    }
}

pub fn check_local_model(state: &mut NativeAppState) {
    state.local_model_ready = false;
    state.status_message = "正在检查 SenseVoice 本地模型...".to_string();

    match sensevoice::prepare_and_probe(std::path::Path::new(&state.local_model_dir)) {
        Ok(probe) => {
            state.local_model_ready = true;
            state.local_model_status =
                "SenseVoice 已可被 Rust provider 加载。下一步只差真实录音链路。".to_string();
            state.local_model_summary = sensevoice::format_probe_summary(&probe);
            state.status_message =
                "SenseVoice 本地模型检查通过，已生成兼容词表并完成加载探测。".to_string();
        }
        Err(error) => {
            state.local_model_status = format!("SenseVoice 检查失败: {error:#}");
            state.local_model_summary = "当前目录至少要有 `model.onnx`、`config.yaml`、`tokens.json`。如果已经齐全但仍失败，通常说明这个 ONNX 缺少 `transcribe-rs` 需要的 metadata，需要转换成 sherpa 兼容版本，或后续改走 FunASR runtime。".to_string();
            state.input_state = InputState::Error;
            state.status_message =
                "SenseVoice 本地模型暂时不可用，请先看下方错误信息。".to_string();
        }
    }
}

fn start_recording() -> Result<String> {
    let slot = recording_slot();
    let mut guard = slot
        .lock()
        .map_err(|_| anyhow!("录音状态锁已损坏，建议重启程序。"))?;

    if guard.is_some() {
        return Err(anyhow!("当前已经在录音中。"));
    }

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow!("没有找到可用的默认麦克风。"))?;
    let device_name = device.name().unwrap_or_else(|_| "未知设备".to_string());
    let supported_config = device
        .default_input_config()
        .context("读取默认麦克风配置失败。")?;
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.config();

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let error_label = device_name.clone();
    let stream = build_input_stream(
        &device,
        &config,
        sample_format,
        Arc::clone(&buffer),
        error_label,
    )?;
    stream.play().context("启动麦克风采集失败。")?;
    remember_delivery_target_window();

    *guard = Some(RecordingSession {
        _stream: stream,
        buffer,
        sample_rate: config.sample_rate.0,
        channels: config.channels,
        started_at: Instant::now(),
    });

    Ok(format!(
        "正在使用默认麦克风 `{}` 录音，输入配置为 {} Hz / {} 通道。",
        device_name, config.sample_rate.0, config.channels
    ))
}

fn stop_recording() -> Result<RecordingCapture> {
    let slot = recording_slot();
    let mut guard = slot
        .lock()
        .map_err(|_| anyhow!("录音状态锁已损坏，建议重启程序。"))?;
    let session = guard
        .take()
        .ok_or_else(|| anyhow!("当前没有可停止的录音会话。"))?;

    let elapsed = session.started_at.elapsed().as_secs_f32();
    let samples = session
        .buffer
        .lock()
        .map_err(|_| anyhow!("录音缓冲区锁已损坏。"))?
        .clone();

    Ok(RecordingCapture {
        samples,
        sample_rate: session.sample_rate,
        channels: session.channels,
        duration_secs: elapsed,
    })
}

fn build_input_stream(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    buffer: Arc<Mutex<Vec<f32>>>,
    device_name: String,
) -> Result<Stream> {
    let err_fn = move |error| {
        eprintln!("输入设备 `{}` 录音出错: {}", device_name, error);
    };

    match sample_format {
        SampleFormat::F32 => device
            .build_input_stream(
                config,
                move |data: &[f32], _| push_samples_f32(&buffer, data),
                err_fn,
                None,
            )
            .context("创建 f32 录音流失败。"),
        SampleFormat::I16 => device
            .build_input_stream(
                config,
                move |data: &[i16], _| {
                    let converted: Vec<f32> = data
                        .iter()
                        .map(|sample| *sample as f32 / i16::MAX as f32)
                        .collect();
                    push_samples_f32(&buffer, &converted);
                },
                err_fn,
                None,
            )
            .context("创建 i16 录音流失败。"),
        SampleFormat::U16 => device
            .build_input_stream(
                config,
                move |data: &[u16], _| {
                    let converted: Vec<f32> = data
                        .iter()
                        .map(|sample| (*sample as f32 / u16::MAX as f32) * 2.0 - 1.0)
                        .collect();
                    push_samples_f32(&buffer, &converted);
                },
                err_fn,
                None,
            )
            .context("创建 u16 录音流失败。"),
        other => Err(anyhow!("暂不支持的采样格式: {:?}", other)),
    }
}

fn push_samples_f32(buffer: &Arc<Mutex<Vec<f32>>>, data: &[f32]) {
    if let Ok(mut guard) = buffer.lock() {
        guard.extend_from_slice(data);
    }
    update_overlay_level(level_from_samples(data));
}

fn resample_to_target(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Vec<f32> {
    let mono = downmix_to_mono(samples, channels);
    if mono.is_empty() {
        return mono;
    }

    if sample_rate == TARGET_SAMPLE_RATE {
        return mono;
    }

    linear_resample(&mono, sample_rate, TARGET_SAMPLE_RATE)
}

fn downmix_to_mono(samples: Vec<f32>, channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples;
    }

    let channels = channels as usize;
    let mut mono = Vec::with_capacity(samples.len() / channels.max(1));
    for frame in samples.chunks(channels) {
        let sum: f32 = frame.iter().copied().sum();
        mono.push(sum / frame.len() as f32);
    }
    mono
}

fn linear_resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if samples.len() < 2 || from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = to_rate as f64 / from_rate as f64;
    let target_len = ((samples.len() as f64) * ratio).round().max(1.0) as usize;
    let mut output = Vec::with_capacity(target_len);

    for idx in 0..target_len {
        let src_pos = idx as f64 / ratio;
        let left = src_pos.floor() as usize;
        let right = (left + 1).min(samples.len() - 1);
        let frac = (src_pos - left as f64) as f32;
        let sample = samples[left] * (1.0 - frac) + samples[right] * frac;
        output.push(sample);
    }

    output
}

fn deliver_output(state: &NativeAppState) -> Result<String> {
    deliver_output_from_snapshot(
        &FinishInputSnapshot {
            input_mode: state.input_mode,
            delivery_target: state.delivery_target,
            auto_paste: state.auto_paste,
            local_model_dir: state.local_model_dir.clone(),
            local_model_ready: state.local_model_ready,
            local_model_status: state.local_model_status.clone(),
            local_model_summary: state.local_model_summary.clone(),
        },
        state.delivered_text.trim(),
    )
}

fn deliver_output_from_snapshot(snapshot: &FinishInputSnapshot, text: &str) -> Result<String> {
    if text.is_empty() {
        return Ok("没有可投送的文本。".to_string());
    }

    if !snapshot.auto_paste {
        return Ok("已生成投送文本，但自动粘贴已关闭。".to_string());
    }

    match snapshot.delivery_target {
        DeliveryTarget::GenericInput => {
            hide_recording_overlay();
            write_clipboard(text)?;
            focus_delivery_target_window();
            simulate_paste_shortcut()?;
            Ok("已复制到剪贴板，并尝试粘贴到当前输入框。".to_string())
        }
        DeliveryTarget::Cursor | DeliveryTarget::VsCodeChat | DeliveryTarget::CopilotChat => {
            hide_recording_overlay();
            write_clipboard(text)?;
            focus_delivery_target_window();
            simulate_paste_shortcut()?;
            Ok(format!(
                "已复制到剪贴板，并尝试粘贴到 {}。",
                snapshot.delivery_target.label()
            ))
        }
    }
}

fn write_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("打开系统剪贴板失败。")?;
    clipboard
        .set_text(text.to_string())
        .context("写入系统剪贴板失败。")?;
    Ok(())
}

fn simulate_paste_shortcut() -> Result<()> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|error| anyhow!("初始化输入模拟失败: {error}"))?;

    // Give Windows a brief moment to observe the updated clipboard before Ctrl+V.
    thread::sleep(std::time::Duration::from_millis(80));

    #[cfg(target_os = "macos")]
    {
        enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|error| anyhow!("按下 Meta 失败: {error}"))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|error| anyhow!("发送 V 失败: {error}"))?;
        enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|error| anyhow!("释放 Meta 失败: {error}"))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|error| anyhow!("按下 Ctrl 失败: {error}"))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|error| anyhow!("发送 V 失败: {error}"))?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|error| anyhow!("释放 Ctrl 失败: {error}"))?;
    }

    Ok(())
}

fn normalize_text(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn show_recording_overlay(mode: &str) -> Result<()> {
    ensure_overlay_host_running()?;
    set_overlay_state(true, mode, 0.0, true)
}

fn hide_recording_overlay() {
    let _ = set_overlay_state(false, "idle", 0.0, true);
}

fn ensure_overlay_host_running() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let slot = overlay_slot();
        let mut guard = slot
            .lock()
            .map_err(|_| anyhow!("浮层状态锁已损坏，建议重启程序。"))?;

        let already_running = match guard.as_mut() {
            Some(child) => child.try_wait().ok().flatten().is_none(),
            None => false,
        };
        if already_running {
            return Ok(());
        }

        *guard = None;
        let overlay_script =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("recording-overlay.ps1");
        let state_file = overlay_state_path();
        let child = Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-STA")
            .arg("-File")
            .arg(&overlay_script)
            .arg("-StateFile")
            .arg(&state_file)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("启动录音浮层失败: {}", overlay_script.display()))?;

        *guard = Some(child);
        Ok(())
    }
}

fn close_stale_overlay_host() {
    #[cfg(target_os = "windows")]
    {
        if let Ok(mut guard) = overlay_slot().lock()
            && let Some(mut child) = guard.take()
        {
            let _ = child.kill();
            let _ = child.wait();
        }

        let title: Vec<u16> = "Vibe Coding Voice Overlay"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if !hwnd.is_null() {
            unsafe {
                let _ = PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
    }
}

fn set_overlay_state(visible: bool, mode: &str, level: f32, reset_timer: bool) -> Result<()> {
    let state = overlay_ui_slot();
    let mut guard = state
        .lock()
        .map_err(|_| anyhow!("浮层 UI 状态锁已损坏，建议重启程序。"))?;
    guard.visible = visible;
    guard.mode = mode.to_string();
    guard.level = level;
    if reset_timer {
        guard.started_at_ms = unix_now_ms();
    }
    guard.last_level_flush = Some(Instant::now());
    flush_overlay_state(&guard)
}

fn flush_overlay_state(state: &OverlayUiState) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = state;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let path = overlay_state_path();
        let payload = json!({
            "visible": state.visible,
            "mode": state.mode,
            "level": state.level,
            "started_at_ms": state.started_at_ms,
        });
        fs::write(&path, payload.to_string())
            .with_context(|| format!("写入录音浮层状态失败: {}", path.display()))?;
        Ok(())
    }
}

fn update_overlay_level(level: f32) {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = level;
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let state = overlay_ui_slot();
        let Ok(mut guard) = state.lock() else {
            return;
        };
        if !guard.visible || guard.mode != "listening" {
            return;
        }

        guard.level = (guard.level * 0.7 + level * 0.3).clamp(0.0, 1.0);

        let now = Instant::now();
        if let Some(last) = guard.last_level_flush
            && now.duration_since(last) < Duration::from_millis(50)
        {
            return;
        }
        guard.last_level_flush = Some(now);
        let _ = flush_overlay_state(&guard);
    }
}

fn level_from_samples(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }

    let energy = data.iter().map(|sample| sample * sample).sum::<f32>() / data.len() as f32;
    (energy.sqrt() * 4.0).clamp(0.0, 1.0)
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn remember_delivery_target_window() {
    #[cfg(target_os = "windows")]
    {
        let hwnd = unsafe { GetForegroundWindow() };
        if !hwnd.is_null()
            && let Ok(mut guard) = delivery_target_slot().lock()
        {
            *guard = Some(hwnd as isize);
        }
    }
}

fn focus_delivery_target_window() {
    #[cfg(target_os = "windows")]
    {
        let hwnd = delivery_target_slot().lock().ok().and_then(|guard| *guard);
        if let Some(hwnd) = hwnd {
            unsafe {
                let _ = SetForegroundWindow(hwnd as _);
            }
            thread::sleep(Duration::from_millis(70));
        }
    }
}

fn transform_text(mode: InputMode, input: &str) -> String {
    match mode {
        InputMode::CodeEdit => [
            "请基于当前项目和代码上下文完成以下修改:",
            &format!("- 需求: {input}"),
            "- 先理解现有实现和相关依赖关系",
            "- 优先做最小必要改动，不要无关重构",
            "- 保持原有代码风格和命名习惯",
            "- 修改后说明改了哪些文件、做了什么",
        ]
        .join("\n"),
        InputMode::DirectPrompt => input.to_string(),
    }
}
