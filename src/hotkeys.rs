use anyhow::{Context, Result, bail};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

pub struct HotkeyController {
    manager: GlobalHotKeyManager,
    registered: Option<HotKey>,
}

impl HotkeyController {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new().context("创建全局快捷键管理器失败。")?;
        Ok(Self {
            manager,
            registered: None,
        })
    }

    pub fn register_from_string(&mut self, shortcut: &str) -> Result<String> {
        let hotkey: HotKey = shortcut
            .parse()
            .map_err(|error| anyhow::anyhow!("快捷键格式无效: {}", error))?;
        self.register(hotkey)
    }

    pub fn register(&mut self, hotkey: HotKey) -> Result<String> {
        let previous = self.registered.take();

        if let Some(prev) = previous {
            self.manager
                .unregister(prev)
                .with_context(|| format!("取消旧快捷键 `{}` 失败。", format_hotkey(prev)))?;
        }

        if let Err(error) = self.manager.register(hotkey) {
            if let Some(prev) = previous {
                let _ = self.manager.register(prev);
                self.registered = Some(prev);
            }
            bail!("注册快捷键 `{}` 失败: {}", format_hotkey(hotkey), error);
        }

        self.registered = Some(hotkey);
        Ok(format_hotkey(hotkey))
    }

    pub fn poll_triggered(&self) -> bool {
        let Some(active) = self.registered else {
            return false;
        };

        let mut triggered = false;
        while let Some(event) = pop_pending_event() {
            if event.id == active.id() && event.state == HotKeyState::Pressed {
                triggered = true;
            }
        }
        triggered
    }
}

pub fn install_event_handler(ctx: &egui::Context) {
    static INSTALLED: OnceLock<()> = OnceLock::new();

    if INSTALLED.get().is_some() {
        return;
    }

    let ctx = ctx.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |event| {
        if let Ok(mut queue) = pending_events().lock() {
            queue.push_back(event);
        }
        ctx.request_repaint();
    }));
    let _ = INSTALLED.set(());
}

pub fn build_hotkey(modifiers: egui::Modifiers, key: egui::Key) -> Result<HotKey> {
    let code = map_egui_key(key)?;
    let mut mods = Modifiers::empty();

    if modifiers.ctrl || modifiers.command {
        mods |= Modifiers::CONTROL;
    }
    if modifiers.alt {
        mods |= Modifiers::ALT;
    }
    if modifiers.shift {
        mods |= Modifiers::SHIFT;
    }
    if modifiers.mac_cmd {
        mods |= Modifiers::SUPER;
    }

    Ok(HotKey::new(Some(mods), code))
}

pub fn format_hotkey(hotkey: HotKey) -> String {
    let mut parts = Vec::new();

    if hotkey.mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if hotkey.mods.contains(Modifiers::ALT) {
        parts.push("Alt");
    }
    if hotkey.mods.contains(Modifiers::SHIFT) {
        parts.push("Shift");
    }
    if hotkey.mods.contains(Modifiers::SUPER) {
        parts.push("Win");
    }

    parts.push(format_key(hotkey.key));
    parts.join("+")
}

fn pending_events() -> &'static Mutex<VecDeque<GlobalHotKeyEvent>> {
    static EVENTS: OnceLock<Mutex<VecDeque<GlobalHotKeyEvent>>> = OnceLock::new();
    EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn pop_pending_event() -> Option<GlobalHotKeyEvent> {
    pending_events().lock().ok()?.pop_front()
}

fn map_egui_key(key: egui::Key) -> Result<Code> {
    use egui::Key as EKey;
    use global_hotkey::hotkey::Code as GCode;

    let code = match key {
        EKey::A => GCode::KeyA,
        EKey::B => GCode::KeyB,
        EKey::C => GCode::KeyC,
        EKey::D => GCode::KeyD,
        EKey::E => GCode::KeyE,
        EKey::F => GCode::KeyF,
        EKey::G => GCode::KeyG,
        EKey::H => GCode::KeyH,
        EKey::I => GCode::KeyI,
        EKey::J => GCode::KeyJ,
        EKey::K => GCode::KeyK,
        EKey::L => GCode::KeyL,
        EKey::M => GCode::KeyM,
        EKey::N => GCode::KeyN,
        EKey::O => GCode::KeyO,
        EKey::P => GCode::KeyP,
        EKey::Q => GCode::KeyQ,
        EKey::R => GCode::KeyR,
        EKey::S => GCode::KeyS,
        EKey::T => GCode::KeyT,
        EKey::U => GCode::KeyU,
        EKey::V => GCode::KeyV,
        EKey::W => GCode::KeyW,
        EKey::X => GCode::KeyX,
        EKey::Y => GCode::KeyY,
        EKey::Z => GCode::KeyZ,
        EKey::Num0 => GCode::Digit0,
        EKey::Num1 => GCode::Digit1,
        EKey::Num2 => GCode::Digit2,
        EKey::Num3 => GCode::Digit3,
        EKey::Num4 => GCode::Digit4,
        EKey::Num5 => GCode::Digit5,
        EKey::Num6 => GCode::Digit6,
        EKey::Num7 => GCode::Digit7,
        EKey::Num8 => GCode::Digit8,
        EKey::Num9 => GCode::Digit9,
        EKey::F1 => GCode::F1,
        EKey::F2 => GCode::F2,
        EKey::F3 => GCode::F3,
        EKey::F4 => GCode::F4,
        EKey::F5 => GCode::F5,
        EKey::F6 => GCode::F6,
        EKey::F7 => GCode::F7,
        EKey::F8 => GCode::F8,
        EKey::F9 => GCode::F9,
        EKey::F10 => GCode::F10,
        EKey::F11 => GCode::F11,
        EKey::F12 => GCode::F12,
        EKey::Space => GCode::Space,
        EKey::Enter => GCode::Enter,
        EKey::Tab => GCode::Tab,
        EKey::Escape => GCode::Escape,
        EKey::Backspace => GCode::Backspace,
        EKey::Delete => GCode::Delete,
        EKey::Insert => GCode::Insert,
        EKey::Home => GCode::Home,
        EKey::End => GCode::End,
        EKey::PageUp => GCode::PageUp,
        EKey::PageDown => GCode::PageDown,
        EKey::ArrowUp => GCode::ArrowUp,
        EKey::ArrowDown => GCode::ArrowDown,
        EKey::ArrowLeft => GCode::ArrowLeft,
        EKey::ArrowRight => GCode::ArrowRight,
        _ => bail!("当前只支持字母、数字、功能键、方向键和常见控制键。"),
    };

    Ok(code)
}

fn format_key(code: Code) -> &'static str {
    use global_hotkey::hotkey::Code::*;

    match code {
        KeyA => "A",
        KeyB => "B",
        KeyC => "C",
        KeyD => "D",
        KeyE => "E",
        KeyF => "F",
        KeyG => "G",
        KeyH => "H",
        KeyI => "I",
        KeyJ => "J",
        KeyK => "K",
        KeyL => "L",
        KeyM => "M",
        KeyN => "N",
        KeyO => "O",
        KeyP => "P",
        KeyQ => "Q",
        KeyR => "R",
        KeyS => "S",
        KeyT => "T",
        KeyU => "U",
        KeyV => "V",
        KeyW => "W",
        KeyX => "X",
        KeyY => "Y",
        KeyZ => "Z",
        Digit0 => "0",
        Digit1 => "1",
        Digit2 => "2",
        Digit3 => "3",
        Digit4 => "4",
        Digit5 => "5",
        Digit6 => "6",
        Digit7 => "7",
        Digit8 => "8",
        Digit9 => "9",
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        Space => "Space",
        Enter => "Enter",
        Tab => "Tab",
        Escape => "Esc",
        Backspace => "Backspace",
        Delete => "Delete",
        Insert => "Insert",
        Home => "Home",
        End => "End",
        PageUp => "PageUp",
        PageDown => "PageDown",
        ArrowUp => "Up",
        ArrowDown => "Down",
        ArrowLeft => "Left",
        ArrowRight => "Right",
        _ => "Unknown",
    }
}
