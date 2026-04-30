use crate::sensevoice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    CodeEdit,
    DirectPrompt,
    BugReport,
    CommitMessage,
    TerminalCommand,
    DocPolish,
}

impl InputMode {
    pub const ALL: [InputMode; 6] = [
        InputMode::CodeEdit,
        InputMode::DirectPrompt,
        InputMode::BugReport,
        InputMode::CommitMessage,
        InputMode::TerminalCommand,
        InputMode::DocPolish,
    ];

    pub fn label(self) -> &'static str {
        match self {
            InputMode::CodeEdit => "改代码",
            InputMode::DirectPrompt => "直接提示词",
            InputMode::BugReport => "报错求助",
            InputMode::CommitMessage => "提交说明",
            InputMode::TerminalCommand => "终端命令",
            InputMode::DocPolish => "技术说明",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryTarget {
    Cursor,
    VsCodeChat,
    CopilotChat,
    GenericInput,
}

impl DeliveryTarget {
    pub const ALL: [DeliveryTarget; 4] = [
        DeliveryTarget::Cursor,
        DeliveryTarget::VsCodeChat,
        DeliveryTarget::CopilotChat,
        DeliveryTarget::GenericInput,
    ];

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

impl InputState {
    pub fn label(self) -> &'static str {
        match self {
            InputState::Idle => "待机",
            InputState::Recording => "录音中",
            InputState::Processing => "处理中",
            InputState::Success => "成功",
            InputState::Error => "错误",
        }
    }
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
    pub debug_clicks: u32,
    pub auto_paste: bool,
    pub local_model_dir: String,
    pub local_model_ready: bool,
    pub local_model_status: String,
    pub local_model_summary: String,
    pub last_recording_info: String,
}

impl Default for NativeAppState {
    fn default() -> Self {
        Self {
            shortcut: "ctrl+z".to_string(),
            shortcut_registered: false,
            shortcut_recording: false,
            shortcut_status: "快捷键尚未注册。".to_string(),
            input_mode: InputMode::CodeEdit,
            delivery_target: DeliveryTarget::Cursor,
            input_state: InputState::Idle,
            status_message: "原生输入法已启动，下一步接全局快捷键、录音和自动投送。".to_string(),
            raw_text: String::new(),
            delivered_text: String::new(),
            practice_text: "帮我把这个登录流程加上 refresh token，并补齐错误处理和测试。"
                .to_string(),
            debug_clicks: 0,
            auto_paste: true,
            local_model_dir: sensevoice::default_model_dir()
                .to_string_lossy()
                .into_owned(),
            local_model_ready: false,
            local_model_status: "尚未检查 SenseVoice 本地模型。".to_string(),
            local_model_summary: "当前默认指向官方 sherpa-onnx SenseVoice int8 目录；如果你切回旧 FunASR 风格目录，程序也会尝试把 `tokens.json` 转成 `tokens.txt` 再探测加载。".to_string(),
            last_recording_info: "尚未开始真实录音。".to_string(),
        }
    }
}
