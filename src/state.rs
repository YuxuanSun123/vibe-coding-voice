use crate::sensevoice;

const DEFAULT_QWEN_MODEL: &str = "qwen3-asr-flash-realtime";
const DEFAULT_QWEN_URL: &str = "wss://dashscope.aliyuncs.com/api-ws/v1/realtime";
const DEFAULT_QWEN_LANGUAGE: &str = "zh";
const DEFAULT_QWEN_STATUS: &str = "尚未配置 Qwen-ASR Realtime。";

const DEFAULT_SHORTCUT: &str = "ctrl+z";
const DEFAULT_SHORTCUT_STATUS: &str = "快捷键尚未注册。";
const DEFAULT_STATUS_MESSAGE: &str = "原生输入法已启动，下一步接全局快捷键、录音和自动投送。";
const DEFAULT_PRACTICE_TEXT: &str = "帮我把这个登录流程加上 refresh token，并补齐错误处理和测试。";
const DEFAULT_MODEL_STATUS: &str = "尚未检查 SenseVoice 本地模型。";
const DEFAULT_MODEL_SUMMARY: &str = "当前默认指向官方 sherpa-onnx SenseVoice int8 目录；如果你切回旧 FunASR 风格目录，程序也会尝试把 `tokens.json` 转成 `tokens.txt` 再探测加载。";
const DEFAULT_RECORDING_INFO: &str = "尚未开始真实录音。";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    CodeEdit,
    DirectPrompt,
}

impl InputMode {
    pub const ALL: [InputMode; 2] = [InputMode::DirectPrompt, InputMode::CodeEdit];

    pub fn label(self) -> &'static str {
        match self {
            InputMode::CodeEdit => "改代码",
            InputMode::DirectPrompt => "原文输出",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProvider {
    Local,
    OnlineQwen,
}

impl ModelProvider {
    pub const ALL: [ModelProvider; 2] = [ModelProvider::Local, ModelProvider::OnlineQwen];

    pub fn label(self) -> &'static str {
        match self {
            ModelProvider::Local => "本地模型",
            ModelProvider::OnlineQwen => "在线模型",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryTarget {
    Cursor,
    #[allow(dead_code)]
    VsCodeChat,
    #[allow(dead_code)]
    CopilotChat,
    GenericInput,
}

impl DeliveryTarget {
    pub fn label(self) -> &'static str {
        match self {
            DeliveryTarget::Cursor => "Cursor",
            DeliveryTarget::VsCodeChat => "VS Code Chat",
            DeliveryTarget::CopilotChat => "Copilot Chat",
            DeliveryTarget::GenericInput => "当前输入框",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    Idle,
    Recording,
    Processing,
    Success,
    Error,
}

pub struct NativeAppState {
    pub shortcut: String,
    pub shortcut_registered: bool,
    pub shortcut_recording: bool,
    pub shortcut_status: String,
    pub input_mode: InputMode,
    pub delivery_target: DeliveryTarget,
    pub input_state: InputState,
    pub status_message: String,
    pub raw_text: String,
    pub delivered_text: String,
    pub practice_text: String,
    pub auto_paste: bool,
    pub model_provider: ModelProvider,
    pub local_model_dir: String,
    pub local_model_ready: bool,
    pub local_model_status: String,
    pub local_model_summary: String,
    pub qwen_api_key: String,
    pub qwen_model: String,
    pub qwen_url: String,
    pub qwen_language: String,
    pub qwen_ready: bool,
    pub qwen_status: String,
    pub last_recording_info: String,
}

impl Default for NativeAppState {
    fn default() -> Self {
        Self {
            shortcut: DEFAULT_SHORTCUT.to_string(),
            shortcut_registered: false,
            shortcut_recording: false,
            shortcut_status: DEFAULT_SHORTCUT_STATUS.to_string(),
            input_mode: InputMode::DirectPrompt,
            delivery_target: DeliveryTarget::Cursor,
            input_state: InputState::Idle,
            status_message: DEFAULT_STATUS_MESSAGE.to_string(),
            raw_text: String::new(),
            delivered_text: String::new(),
            practice_text: DEFAULT_PRACTICE_TEXT.to_string(),
            auto_paste: true,
            model_provider: ModelProvider::Local,
            local_model_dir: sensevoice::default_model_dir()
                .to_string_lossy()
                .into_owned(),
            local_model_ready: false,
            local_model_status: DEFAULT_MODEL_STATUS.to_string(),
            local_model_summary: DEFAULT_MODEL_SUMMARY.to_string(),
            qwen_api_key: String::new(),
            qwen_model: DEFAULT_QWEN_MODEL.to_string(),
            qwen_url: DEFAULT_QWEN_URL.to_string(),
            qwen_language: DEFAULT_QWEN_LANGUAGE.to_string(),
            qwen_ready: false,
            qwen_status: DEFAULT_QWEN_STATUS.to_string(),
            last_recording_info: DEFAULT_RECORDING_INFO.to_string(),
        }
    }
}
