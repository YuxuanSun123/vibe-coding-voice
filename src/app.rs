use crate::hotkeys::{HotkeyController, build_hotkey, install_event_handler};
use crate::services::{
    begin_input, check_local_model, clear_result, finish_input, prepare_recording_overlay_host,
    run_practice_flow,
};
use crate::state::{DeliveryTarget, InputMode, InputState, NativeAppState};
use crate::tray::TrayController;
use arboard::Clipboard;
use eframe::egui::{
    self, Align, Align2, Button, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, RichText,
    Sense, Stroke, StrokeKind, TextEdit, Vec2,
};
use global_hotkey::hotkey::Code;
use std::time::{Duration, Instant};

const ACCENT: Color32 = Color32::from_rgb(196, 66, 28);
const SUCCESS: Color32 = Color32::from_rgb(76, 134, 75);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(42, 38, 32);
const TEXT_MUTED: Color32 = Color32::from_rgb(132, 124, 110);
const PANEL_BG: Color32 = Color32::from_rgb(245, 242, 236);
const CARD_BG: Color32 = Color32::from_rgb(253, 251, 246);
const BORDER: Color32 = Color32::from_rgb(225, 220, 208);
const INK: Color32 = Color32::from_rgb(21, 20, 15);
const PROCESSING: Color32 = Color32::from_rgb(196, 140, 55);

#[derive(Clone, Copy)]
enum ToastKind {
    Error,
}

#[derive(Clone)]
struct ToastDraft {
    message: String,
    kind: ToastKind,
}

struct ToastState {
    message: String,
    kind: ToastKind,
    started_at: f64,
}

pub struct VoiceInputNativeApp {
    pub state: NativeAppState,
    hotkey_controller: Option<HotkeyController>,
    tray_controller: Option<TrayController>,
    settings_open: bool,
    recording_started_at: Option<Instant>,
    pending_toast: Option<ToastDraft>,
    toast: Option<ToastState>,
}

impl Default for VoiceInputNativeApp {
    fn default() -> Self {
        let mut app = Self {
            state: NativeAppState::default(),
            hotkey_controller: None,
            tray_controller: None,
            settings_open: false,
            recording_started_at: None,
            pending_toast: None,
            toast: None,
        };
        app.state.input_mode = InputMode::CodeEdit;
        app.initialize_shortcut();
        app.initialize_tray();
        app.initialize_overlay_host();
        app
    }
}

impl eframe::App for VoiceInputNativeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_event_handler(ctx);
        ctx.request_repaint_after(Duration::from_millis(16));

        if let Some(pending) = self.pending_toast.take() {
            self.show_toast_now(ctx, pending.kind, pending.message);
        }

        self.capture_shortcut_if_needed(ctx);
        self.handle_global_hotkey(ctx);

        egui::TopBottomPanel::top("top_bar")
            .exact_height(58.0)
            .frame(Frame::default().fill(PANEL_BG).inner_margin(Margin::symmetric(16, 10)))
            .show(ctx, |ui| {
                self.show_top_bar(ui, ctx);
            });

        egui::TopBottomPanel::bottom("result_panel")
            .exact_height(176.0)
            .frame(Frame::default().fill(PANEL_BG).inner_margin(Margin::symmetric(16, 16)))
            .show(ctx, |ui| {
                self.show_result_panel(ui, ctx);
            });

        egui::CentralPanel::default()
            .frame(Frame::default().fill(PANEL_BG).inner_margin(Margin::symmetric(16, 12)))
            .show(ctx, |ui| {
                self.show_recording_panel(ui, ctx);
            });

        self.show_settings_window(ctx);
        self.show_toast(ctx);
    }
}

impl VoiceInputNativeApp {
    fn initialize_shortcut(&mut self) {
        match HotkeyController::new() {
            Ok(mut controller) => match controller.register_from_string(&self.state.shortcut) {
                Ok(display) => {
                    self.state.shortcut = display.clone();
                    self.state.shortcut_registered = true;
                    self.state.shortcut_status = format!("已注册全局快捷键 `{display}`。");
                    self.hotkey_controller = Some(controller);
                }
                Err(error) => {
                    self.state.shortcut_registered = false;
                    self.state.shortcut_status =
                        format!("快捷键初始化失败，请重新设置: {error:#}");
                    self.hotkey_controller = Some(controller);
                    self.queue_error_toast(self.state.shortcut_status.clone());
                }
            },
            Err(error) => {
                self.state.shortcut_registered = false;
                self.state.shortcut_status = format!("系统级快捷键不可用: {error:#}");
                self.queue_error_toast(self.state.shortcut_status.clone());
            }
        }
    }

    fn initialize_tray(&mut self) {
        match TrayController::install() {
            Ok(controller) => {
                self.tray_controller = Some(controller);
            }
            Err(error) => {
                self.state.status_message = format!(
                    "{} 托盘初始化失败: {error:#}",
                    self.state.status_message
                );
                self.queue_error_toast(format!("托盘初始化失败: {error:#}"));
            }
        }
    }

    fn initialize_overlay_host(&mut self) {
        if let Err(error) = prepare_recording_overlay_host() {
            self.state.status_message = format!(
                "{} 录音浮层预热失败: {error:#}",
                self.state.status_message
            );
            self.queue_error_toast(format!("录音浮层预热失败: {error:#}"));
        }
    }

    fn show_top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let (status_text, dot_color) = self.status_chip(ctx);

        ui.horizontal(|ui| {
            let chip = Frame::default()
                .fill(INK)
                .corner_radius(CornerRadius::same(24))
                .inner_margin(Margin::symmetric(14, 8));
            chip.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 5.0, dot_color);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(status_text)
                            .size(15.0)
                            .color(Color32::from_rgb(247, 241, 227))
                            .strong(),
                    );
                });
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let gear = Button::new(RichText::new("⚙").size(24.0).color(TEXT_MUTED))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::same(14))
                    .min_size(Vec2::splat(28.0));
                if ui.add(gear).clicked() {
                    self.settings_open = true;
                }
            });
        });
    }

    fn show_recording_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let available = ui.available_height();
        let reserved = 220.0;
        let top_space = ((available - reserved) * 0.5).max(8.0);
        ui.add_space(top_space);

        ui.vertical_centered(|ui| {
            let button_size = Vec2::splat(160.0);
            let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());
            let painter = ui.painter_at(rect);
            let center = rect.center();
            let is_recording = self.state.input_state == InputState::Recording;
            let is_busy = matches!(self.state.input_state, InputState::Processing);

            let pulse_driver = ctx.animate_value_with_time(
                Id::new("record_button_phase"),
                ctx.input(|input| input.time) as f32,
                0.12,
            );
            let active = ctx.animate_bool_with_time(Id::new("record_button_active"), is_recording, 0.18);
            let phase = (pulse_driver * 0.85).fract();
            if active > 0.01 {
                for offset in [0.0_f32, 0.5_f32] {
                    let progress = (phase + offset).fract();
                    let radius = 48.0 * (1.0 + 0.6 * progress);
                    let alpha = (90.0 * (1.0 - progress) * active).round() as u8;
                    if alpha > 0 {
                        painter.circle_stroke(
                            center,
                            radius,
                            Stroke::new(
                                2.0,
                                Color32::from_rgba_unmultiplied(
                                    ACCENT.r(),
                                    ACCENT.g(),
                                    ACCENT.b(),
                                    alpha,
                                ),
                            ),
                        );
                    }
                }
            }

            let fill = if is_recording { ACCENT } else { INK };
            painter.circle_filled(center, 48.0, fill);
            self.paint_mic_icon(&painter, center, Color32::from_rgb(248, 244, 238));

            if response.clicked() && !is_busy {
                self.toggle_recording(ctx);
            }

            ui.add_space(12.0);
            self.show_shortcut_hint(ui);
            ui.add_space(24.0);

            if is_recording {
                ui.label(
                    RichText::new(self.recording_elapsed_label())
                        .font(FontId::monospace(24.0))
                        .color(TEXT_PRIMARY),
                );
            } else if self.state.input_state == InputState::Processing {
                ui.label(
                    RichText::new("识别中…")
                        .font(FontId::monospace(20.0))
                        .color(TEXT_MUTED),
                );
            } else {
                ui.add_space(28.0);
            }
        });
    }

    fn show_shortcut_hint(&self, ui: &mut egui::Ui) {
        if self.state.shortcut_recording {
            ui.label(
                RichText::new("按下新的组合键…")
                    .size(11.0)
                    .color(TEXT_MUTED),
            );
            return;
        }

        if self.state.input_state == InputState::Recording {
            ui.label(
                RichText::new("再按一次结束")
                    .size(11.0)
                    .color(TEXT_MUTED),
            );
            return;
        }

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.label(RichText::new("按").size(11.0).color(TEXT_MUTED));
            for part in self.state.shortcut.split('+') {
                let key_frame = Frame::default()
                    .fill(CARD_BG)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(8, 4));
                key_frame.show(ui, |ui| {
                    ui.label(
                        RichText::new(part)
                            .size(11.0)
                            .color(TEXT_PRIMARY)
                            .family(egui::FontFamily::Monospace),
                    );
                });
            }
            ui.label(RichText::new("开始口述").size(11.0).color(TEXT_MUTED));
        });
    }

    fn show_result_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let line_y = ui.min_rect().top();
        ui.painter().hline(
            ui.min_rect().x_range(),
            line_y,
            Stroke::new(1.0, BORDER),
        );

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let chip = Button::new(
                RichText::new(format!("→ {}", self.state.delivery_target.label()))
                    .size(13.0)
                    .color(Color32::from_rgb(248, 244, 238))
                    .strong(),
            )
            .fill(INK)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(15))
            .min_size(Vec2::new(92.0, 30.0));
            if ui.add(chip).clicked() {
                self.cycle_delivery_target();
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let send = Button::new(
                    RichText::new("发送")
                        .size(12.0)
                        .color(Color32::from_rgb(248, 244, 238))
                        .strong(),
                )
                .fill(INK)
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::same(10))
                .min_size(Vec2::new(58.0, 30.0));
                if ui.add(send).clicked() {
                    self.handle_send(ctx);
                }

                let clear = Button::new(RichText::new("清空").size(12.0).color(TEXT_PRIMARY))
                    .fill(CARD_BG)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::new(58.0, 30.0));
                if ui.add(clear).clicked() {
                    clear_result(&mut self.state);
                    self.recording_started_at = None;
                }

                let copy = Button::new(RichText::new("复制").size(12.0).color(TEXT_PRIMARY))
                    .fill(CARD_BG)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::new(58.0, 30.0));
                if ui.add(copy).clicked() {
                    self.copy_visible_text(ctx);
                }
            });
        });

        ui.add_space(10.0);
        let editor_frame = Frame::default()
            .fill(CARD_BG)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(12));
        editor_frame.show(ui, |ui| {
            let editor = TextEdit::multiline(&mut self.state.raw_text)
                .hint_text("刚刚识别的文字会出现在这里…")
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .frame(false);
            let response = ui.add_sized([ui.available_width(), 60.0], editor);
            if response.changed() {
                self.sync_hidden_delivery_text();
            }
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("识别完自动粘贴到目标")
                    .size(11.0)
                    .color(TEXT_MUTED),
            );
            ui.add_space(6.0);
            let toggle = ui.toggle_value(&mut self.state.auto_paste, "");
            if toggle.changed() {
                self.sync_hidden_delivery_text();
            }
        });
    }

    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.settings_open {
            return;
        }

        let mut open = self.settings_open;
        egui::Window::new("设置")
            .open(&mut open)
            .default_width(420.0)
            .min_width(380.0)
            .collapsible(false)
            .resizable(true)
            .frame(
                Frame::default()
                    .fill(CARD_BG)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(16))
                    .inner_margin(Margin::same(16)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("全局快捷键").size(14.0).strong());
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let capture_label = if self.state.shortcut_recording {
                        "按下新的组合键..."
                    } else {
                        &self.state.shortcut
                    };
                    ui.label(RichText::new(capture_label).family(egui::FontFamily::Monospace));

                    let button_text = if self.state.shortcut_recording {
                        "取消"
                    } else {
                        "更改快捷键"
                    };
                    if ui.button(button_text).clicked() {
                        self.toggle_shortcut_recording();
                    }
                });

                ui.separator();
                ui.label(RichText::new("投送目标").size(14.0).strong());
                egui::ComboBox::from_id_salt("settings_delivery_target")
                    .selected_text(self.state.delivery_target.label())
                    .show_ui(ui, |ui| {
                        for target in DeliveryTarget::ALL {
                            ui.selectable_value(&mut self.state.delivery_target, target, target.label());
                        }
                    });

                ui.separator();
                ui.label(RichText::new("意图模式").size(14.0).strong());
                egui::ComboBox::from_id_salt("settings_input_mode")
                    .selected_text(self.state.input_mode.label())
                    .show_ui(ui, |ui| {
                        for mode in InputMode::ALL {
                            ui.selectable_value(&mut self.state.input_mode, mode, mode.label());
                        }
                    });

                ui.separator();
                ui.label(RichText::new("本地模型").size(14.0).strong());
                ui.add_space(4.0);
                ui.label(RichText::new("模型目录").size(12.0).color(TEXT_MUTED));
                ui.text_edit_singleline(&mut self.state.local_model_dir);
                ui.add_space(6.0);
                if ui.button("检查 SenseVoice").clicked() {
                    check_local_model(&mut self.state);
                    if !self.state.local_model_ready {
                        self.show_toast_now(ctx, ToastKind::Error, self.state.local_model_status.clone());
                    }
                }
                ui.add_space(6.0);
                ui.label(
                    RichText::new(if self.state.local_model_ready { "模型已就绪" } else { "模型未就绪" })
                        .size(12.0)
                        .color(if self.state.local_model_ready { SUCCESS } else { ACCENT }),
                );
                ui.add_space(6.0);
                ui.label(RichText::new("状态").size(12.0).color(TEXT_MUTED));
                ui.label(RichText::new(&self.state.local_model_status).size(12.0));
                ui.add_space(4.0);
                ui.label(RichText::new("摘要").size(12.0).color(TEXT_MUTED));
                ui.label(RichText::new(&self.state.local_model_summary).size(12.0));
            });
        self.settings_open = open;
    }

    fn show_toast(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };

        let elapsed = (ctx.input(|input| input.time) - toast.started_at) as f32;
        if elapsed >= 3.0 {
            self.toast = None;
            return;
        }

        let fade = if elapsed > 2.4 {
            (1.0 - ((elapsed - 2.4) / 0.6)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let alpha = (235.0 * fade).round() as u8;
        let bg = match toast.kind {
            ToastKind::Error => Color32::from_rgba_unmultiplied(75, 19, 13, alpha),
        };

        egui::Area::new(Id::new("voice_input_toast"))
            .anchor(Align2::RIGHT_TOP, [-18.0, 18.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                Frame::default()
                    .fill(bg)
                    .stroke(Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(196, 66, 28, (190.0 * fade).round() as u8),
                    ))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_max_width(320.0);
                        ui.label(
                            RichText::new(&toast.message)
                                .size(12.0)
                                .color(Color32::from_rgb(250, 239, 233)),
                        );
                    });
            });
    }

    fn status_chip(&self, ctx: &egui::Context) -> (&'static str, Color32) {
        match self.state.input_state {
            InputState::Idle | InputState::Success => ("准备就绪", SUCCESS),
            InputState::Recording => {
                let phase = ((ctx.input(|input| input.time) * 4.0).sin() * 0.5 + 0.5) as f32;
                let alpha = (140.0 + phase * 115.0).round() as u8;
                (
                    "录音中",
                    Color32::from_rgba_unmultiplied(ACCENT.r(), ACCENT.g(), ACCENT.b(), alpha),
                )
            }
            InputState::Processing => ("识别中", PROCESSING),
            InputState::Error => ("出错了", ACCENT),
        }
    }

    fn toggle_recording(&mut self, ctx: &egui::Context) {
        if self.state.input_state == InputState::Recording {
            finish_input(&mut self.state);
        } else {
            begin_input(&mut self.state);
        }
        self.sync_after_action(ctx);
    }

    fn copy_visible_text(&mut self, ctx: &egui::Context) {
        let text = self.state.raw_text.trim();
        if text.is_empty() {
            self.show_toast_now(ctx, ToastKind::Error, "当前没有可复制的识别文本。");
            return;
        }

        match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text.to_string())) {
            Ok(()) => {}
            Err(error) => {
                self.show_toast_now(ctx, ToastKind::Error, format!("复制失败: {error}"));
            }
        }
    }

    fn handle_send(&mut self, ctx: &egui::Context) {
        if self.state.raw_text.trim().is_empty() {
            self.show_toast_now(ctx, ToastKind::Error, "当前没有可发送的识别文本。");
            return;
        }

        self.state.practice_text = self.state.raw_text.clone();
        run_practice_flow(&mut self.state);
        self.sync_hidden_delivery_text();
        if self.state.status_message.contains("投送失败") {
            self.show_toast_now(ctx, ToastKind::Error, self.state.status_message.clone());
        }
    }

    fn cycle_delivery_target(&mut self) {
        let current = DeliveryTarget::ALL
            .iter()
            .position(|target| *target == self.state.delivery_target)
            .unwrap_or(0);
        let next = (current + 1) % DeliveryTarget::ALL.len();
        self.state.delivery_target = DeliveryTarget::ALL[next];
    }

    fn recording_elapsed_label(&self) -> String {
        let Some(started_at) = self.recording_started_at else {
            return "0:00".to_string();
        };

        let elapsed = started_at.elapsed().as_secs();
        let minutes = elapsed / 60;
        let seconds = elapsed % 60;
        format!("{minutes}:{seconds:02}")
    }

    fn paint_mic_icon(&self, painter: &egui::Painter, center: egui::Pos2, color: Color32) {
        let stroke = Stroke::new(4.0, color);
        let body = egui::Rect::from_center_size(center + egui::vec2(0.0, -10.0), egui::vec2(20.0, 28.0));
        painter.rect_stroke(body, CornerRadius::same(10), stroke, StrokeKind::Inside);
        painter.line_segment(
            [center + egui::vec2(-16.0, 0.0), center + egui::vec2(-16.0, 4.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(16.0, 0.0), center + egui::vec2(16.0, 4.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(-16.0, 4.0), center + egui::vec2(-8.0, 16.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(16.0, 4.0), center + egui::vec2(8.0, 16.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(0.0, 18.0), center + egui::vec2(0.0, 28.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(-12.0, 30.0), center + egui::vec2(12.0, 30.0)],
            stroke,
        );
    }

    fn capture_shortcut_if_needed(&mut self, ctx: &egui::Context) {
        if !self.state.shortcut_recording {
            return;
        }

        let captured = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } => Some((*key, *modifiers)),
                _ => None,
            })
        });

        let Some((key, modifiers)) = captured else {
            return;
        };

        if key == egui::Key::Escape && !modifiers.alt && !modifiers.ctrl && !modifiers.shift {
            self.state.shortcut_recording = false;
            self.state.shortcut_status = "已取消快捷键录入。".to_string();
            return;
        }

        let hotkey = match build_hotkey(modifiers, key) {
            Ok(hotkey) => hotkey,
            Err(error) => {
                self.state.shortcut_status = format!("这个按键暂时不支持: {error:#}");
                self.show_toast_now(ctx, ToastKind::Error, self.state.shortcut_status.clone());
                return;
            }
        };

        if hotkey.mods.is_empty()
            && !matches!(
                hotkey.key,
                Code::F1
                    | Code::F2
                    | Code::F3
                    | Code::F4
                    | Code::F5
                    | Code::F6
                    | Code::F7
                    | Code::F8
                    | Code::F9
                    | Code::F10
                    | Code::F11
                    | Code::F12
            )
        {
            self.state.shortcut_status =
                "建议至少带一个 Ctrl / Alt / Shift，或直接使用 F1-F12。".to_string();
            self.show_toast_now(ctx, ToastKind::Error, self.state.shortcut_status.clone());
            return;
        }

        match self.apply_shortcut(hotkey) {
            Ok(()) => {
                self.state.shortcut_recording = false;
            }
            Err(error) => {
                self.state.shortcut_status = format!("{error:#}");
                self.show_toast_now(ctx, ToastKind::Error, self.state.shortcut_status.clone());
            }
        }
    }

    fn apply_shortcut(&mut self, hotkey: global_hotkey::hotkey::HotKey) -> anyhow::Result<()> {
        let controller = self
            .hotkey_controller
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("全局快捷键管理器还没有初始化成功。"))?;

        let display = controller.register(hotkey)?;
        self.state.shortcut = display.clone();
        self.state.shortcut_registered = true;
        self.state.shortcut_status = format!("已注册新的全局快捷键 `{display}`。");
        Ok(())
    }

    fn handle_global_hotkey(&mut self, ctx: &egui::Context) {
        let Some(controller) = self.hotkey_controller.as_ref() else {
            return;
        };
        if self.state.shortcut_recording || !controller.poll_triggered() {
            return;
        }

        self.state.status_message = format!("检测到全局快捷键 `{}`。", self.state.shortcut);
        self.toggle_recording(ctx);
    }

    fn toggle_shortcut_recording(&mut self) {
        self.state.shortcut_recording = !self.state.shortcut_recording;
        if self.state.shortcut_recording {
            self.state.shortcut_status =
                "快捷键录入中。请直接按下新的组合键，按 Esc 可取消。".to_string();
        } else if self.state.shortcut_registered {
            self.state.shortcut_status = format!("已保留当前快捷键 `{}`。", self.state.shortcut);
        } else {
            self.state.shortcut_status = "已取消快捷键录入。".to_string();
        }
    }

    fn sync_after_action(&mut self, ctx: &egui::Context) {
        if self.state.input_state == InputState::Recording {
            self.recording_started_at = Some(Instant::now());
        } else {
            self.recording_started_at = None;
        }

        if self.state.input_state == InputState::Error {
            self.show_toast_now(ctx, ToastKind::Error, self.state.status_message.clone());
        }

        if !self.state.raw_text.is_empty() {
            self.sync_hidden_delivery_text();
        }
    }

    fn sync_hidden_delivery_text(&mut self) {
        self.state.delivered_text = transform_text(self.state.input_mode, &self.state.raw_text);
    }

    fn queue_error_toast(&mut self, message: impl Into<String>) {
        self.pending_toast = Some(ToastDraft {
            message: message.into(),
            kind: ToastKind::Error,
        });
    }

    fn show_toast_now(&mut self, ctx: &egui::Context, kind: ToastKind, message: impl Into<String>) {
        self.toast = Some(ToastState {
            message: message.into(),
            kind,
            started_at: ctx.input(|input| input.time),
        });
    }
}

fn transform_text(mode: InputMode, input: &str) -> String {
    match mode {
        InputMode::CodeEdit => [
            "请基于当前代码上下文执行以下修改:",
            &format!("- 需求: {input}"),
            "- 先理解现有实现",
            "- 保持改动范围尽量小",
            "- 给出必要的关键说明",
        ]
        .join("\n"),
        InputMode::DirectPrompt => input.to_string(),
        InputMode::BugReport => ["问题描述:", input, "", "复现步骤:", "1. 待补充", "2. 待补充"].join("\n"),
        InputMode::CommitMessage => format!("feat: {input}"),
        InputMode::TerminalCommand => [
            "请生成或说明终端命令，要求如下:",
            &format!("- 目标: {input}"),
            "- 优先给出可直接执行的命令",
        ]
        .join("\n"),
        InputMode::DocPolish => format!("请将以下口述整理为清晰的技术说明:\n{input}"),
    }
}
