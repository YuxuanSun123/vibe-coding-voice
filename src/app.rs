use crate::hotkeys::{HotkeyController, build_hotkey, install_event_handler};
use crate::services::{
    begin_input, check_local_model, finish_input, prepare_recording_overlay_host,
    run_practice_flow,
};
use crate::state::{DeliveryTarget, InputMode, InputState, NativeAppState};
use crate::tray::TrayController;
use arboard::Clipboard;
use eframe::egui::{
    self, Align, Align2, Button, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin,
    RichText, Sense, Stroke, StrokeKind, TextEdit, Vec2,
};
use global_hotkey::hotkey::Code;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    Paper, // 纸感米白
    Ink,   // 深色 IDE
}

struct Palette {
    accent: Color32,
    success: Color32,
    processing: Color32,
    text_primary: Color32,
    text_muted: Color32,
    panel_bg: Color32,
    card_bg: Color32,
    border: Color32,
    divider: Color32,
    ink: Color32,        // 主操作按钮底色
    ink_text: Color32,   // 主操作按钮文字
}

impl Palette {
    fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Paper => Self {
                accent: Color32::from_rgb(196, 66, 28),
                success: Color32::from_rgb(76, 134, 75),
                processing: Color32::from_rgb(196, 140, 55),
                text_primary: Color32::from_rgb(42, 38, 32),
                text_muted: Color32::from_rgb(132, 124, 110),
                panel_bg: Color32::from_rgb(245, 242, 236),
                card_bg: Color32::from_rgb(253, 251, 246),
                border: Color32::from_rgb(225, 220, 208),
                divider: Color32::from_rgb(225, 220, 208),
                ink: Color32::from_rgb(21, 20, 15),
                ink_text: Color32::from_rgb(248, 244, 238),
            },
            ThemeMode::Ink => Self {
                accent: Color32::from_rgb(255, 107, 94),
                success: Color32::from_rgb(109, 211, 160),
                processing: Color32::from_rgb(245, 176, 66),
                text_primary: Color32::from_rgb(216, 218, 224),
                text_muted: Color32::from_rgb(138, 142, 152),
                panel_bg: Color32::from_rgb(28, 29, 34),
                card_bg: Color32::from_rgb(37, 39, 45),
                border: Color32::from_rgb(52, 55, 63),
                divider: Color32::from_rgb(52, 55, 63),
                ink: Color32::from_rgb(99, 136, 245),
                ink_text: Color32::WHITE,
            },
        }
    }

    fn apply_visuals(&self, ctx: &egui::Context, mode: ThemeMode) {
        let mut visuals = if mode == ThemeMode::Paper {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.window_fill = self.panel_bg;
        visuals.panel_fill = self.panel_bg;
        visuals.extreme_bg_color = self.card_bg;
        visuals.faint_bg_color = self.panel_bg;
        visuals.override_text_color = Some(self.text_primary);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border);
        visuals.widgets.inactive.bg_fill = self.card_bg;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border);
        visuals.widgets.hovered.bg_fill = self.panel_bg;
        visuals.widgets.active.bg_fill = self.card_bg;
        visuals.selection.bg_fill =
            Color32::from_rgba_unmultiplied(self.accent.r(), self.accent.g(), self.accent.b(), 32);
        visuals.selection.stroke = Stroke::new(1.0, self.accent);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = Margin::same(0);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
        ctx.set_style(style);
    }
}

struct LayoutTokens {
    top_bar_height: f32,
    panel_margin: Margin,
    top_bar_margin: Margin,
    result_panel_margin: Margin,
    status_chip_padding: Margin,
    status_chip_radius: u8,
    record_button_size: f32,
    record_button_radius: f32,
    record_pulse_width: f32,
    record_top_padding_min: f32,
    record_hint_gap: f32,
    record_status_gap: f32,
    key_chip_padding: Margin,
    key_chip_radius: u8,
    section_gap: f32,
    small_gap: f32,
    divider_inset: f32,
    bottom_divider_top_gap: f32,
    editor_height: f32,
    editor_rows: usize,
    button_height: f32,
    primary_button_width: f32,
    secondary_button_width: f32,
    button_radius: u8,
}

impl Default for LayoutTokens {
    fn default() -> Self {
        Self {
            top_bar_height: 30.0,
            panel_margin: Margin::symmetric(16, 8),
            top_bar_margin: Margin::symmetric(12, 2),
            result_panel_margin: Margin::symmetric(16, 7),
            status_chip_padding: Margin::symmetric(10, 4),
            status_chip_radius: 24,
            record_button_size: 144.0,
            record_button_radius: 44.0,
            record_pulse_width: 2.0,
            record_top_padding_min: 12.0,
            record_hint_gap: 10.0,
            record_status_gap: 16.0,
            key_chip_padding: Margin::symmetric(8, 4),
            key_chip_radius: 6,
            section_gap: 4.0,
            small_gap: 3.0,
            divider_inset: 0.0,
            bottom_divider_top_gap: 8.0,
            editor_height: 40.0,
            editor_rows: 2,
            button_height: 30.0,
            primary_button_width: 72.0,
            secondary_button_width: 64.0,
            button_radius: 12,
        }
    }
}

impl LayoutTokens {
    fn recording_content_height(&self, input_state: InputState) -> f32 {
        let status_height = if matches!(input_state, InputState::Recording | InputState::Processing)
        {
            28.0
        } else {
            22.0
        };
        self.record_button_size + self.record_hint_gap + 40.0 + self.record_status_gap + status_height
    }
}

struct Theme {
    mode: ThemeMode,
    palette: Palette,
    layout: LayoutTokens,
}

impl Theme {
    fn for_mode(mode: ThemeMode) -> Self {
        Self {
            mode,
            palette: Palette::for_mode(mode),
            layout: LayoutTokens::default(),
        }
    }

    fn apply_visuals(&self, ctx: &egui::Context) {
        self.palette.apply_visuals(ctx, self.mode);
    }
}

struct TopBar;

impl TopBar {
    fn show(
        app: &mut VoiceInputNativeApp,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
    ) {
        app.show_top_bar(ui, ctx, theme);
    }
}

struct RecordingPanel;

impl RecordingPanel {
    fn show(
        app: &mut VoiceInputNativeApp,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
    ) {
        app.show_recording_panel(ui, ctx, theme);
    }
}

struct ResultPanel;

impl ResultPanel {
    fn show(
        app: &mut VoiceInputNativeApp,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
    ) {
        app.show_result_panel(ui, ctx, theme);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppPage {
    Home,
    Settings,
}

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
    current_page: AppPage,
    recording_started_at: Option<Instant>,
    pending_toast: Option<ToastDraft>,
    toast: Option<ToastState>,
    theme: ThemeMode,
    theme_applied: bool,
}

impl Default for VoiceInputNativeApp {
    fn default() -> Self {
        let mut app = Self {
            state: NativeAppState::default(),
            hotkey_controller: None,
            tray_controller: None,
            current_page: AppPage::Home,
            recording_started_at: None,
            pending_toast: None,
            toast: None,
            theme: ThemeMode::Paper,
            theme_applied: false,
        };
        app.state.input_mode = InputMode::CodeEdit;
        app.state.delivery_target = DeliveryTarget::GenericInput;
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

        let theme = Theme::for_mode(self.theme);
        if !self.theme_applied {
            theme.apply_visuals(ctx);
            self.theme_applied = true;
        }

        if let Some(pending) = self.pending_toast.take() {
            self.show_toast_now(ctx, pending.kind, pending.message);
        }

        self.capture_shortcut_if_needed(ctx);
        self.handle_global_hotkey(ctx);

        if self.current_page == AppPage::Home {
            egui::TopBottomPanel::bottom("result_panel")
                .resizable(false)
                .show_separator_line(false)
                .frame(
                    Frame::default()
                        .fill(theme.palette.panel_bg)
                        .inner_margin(theme.layout.result_panel_margin),
                )
                .show(ctx, |ui| {
                    ResultPanel::show(self, ui, ctx, &theme);
                });
        }

        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(theme.palette.panel_bg)
                    .inner_margin(theme.layout.panel_margin),
            )
            .show(ctx, |ui| {
                let top_bar_inner_height = (theme.layout.top_bar_height
                    - f32::from(theme.layout.top_bar_margin.top)
                    - f32::from(theme.layout.top_bar_margin.bottom))
                .max(0.0);
                let top_bar_response = ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), theme.layout.top_bar_height),
                    Layout::top_down(Align::Min),
                    |ui| {
                        Frame::default()
                            .fill(theme.palette.panel_bg)
                            .inner_margin(theme.layout.top_bar_margin)
                            .show(ui, |ui| {
                                ui.set_min_height(top_bar_inner_height);
                                TopBar::show(self, ui, ctx, &theme);
                            });
                    },
                );
                let divider_y = top_bar_response.response.rect.bottom();
                let divider_left = ui.max_rect().left() - f32::from(theme.layout.panel_margin.left)
                    + theme.layout.divider_inset;
                let divider_right = ui.max_rect().right()
                    + f32::from(theme.layout.panel_margin.right)
                    - theme.layout.divider_inset;
                ui.painter().line_segment(
                    [egui::pos2(divider_left, divider_y), egui::pos2(divider_right, divider_y)],
                    Stroke::new(1.0, theme.palette.divider),
                );
                ui.add_space(theme.layout.section_gap);
                match self.current_page {
                    AppPage::Home => RecordingPanel::show(self, ui, ctx, &theme),
                    AppPage::Settings => self.show_settings_page(ui, ctx, &theme),
                }
            });

        self.show_toast(ctx, &theme);
    }
}

impl VoiceInputNativeApp {
    fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            ThemeMode::Paper => ThemeMode::Ink,
            ThemeMode::Ink => ThemeMode::Paper,
        };
        self.theme_applied = false;
    }

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
                self.state.status_message =
                    format!("{} 托盘初始化失败: {error:#}", self.state.status_message);
                self.queue_error_toast(format!("托盘初始化失败: {error:#}"));
            }
        }
    }

    fn initialize_overlay_host(&mut self) {
        if let Err(error) = prepare_recording_overlay_host() {
            self.state.status_message =
                format!("{} 录音浮层预热失败: {error:#}", self.state.status_message);
            self.queue_error_toast(format!("录音浮层预热失败: {error:#}"));
        }
    }

    fn show_top_bar(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
    ) {
        if self.current_page == AppPage::Settings {
            ui.horizontal(|ui| {
                let back = Button::new(
                    RichText::new("← 返回")
                        .size(13.0)
                        .color(theme.palette.text_primary)
                        .strong(),
                )
                .fill(theme.palette.card_bg)
                .stroke(Stroke::new(1.0, theme.palette.border))
                .corner_radius(CornerRadius::same(12))
                .min_size(Vec2::new(78.0, 30.0));
                if ui.add(back).clicked() {
                    self.current_page = AppPage::Home;
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("设置")
                        .size(16.0)
                        .color(theme.palette.text_primary)
                        .strong(),
                );
            });
            return;
        }

        let (status_text, dot_color) = self.status_chip(ctx, theme);

        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 24.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                let chip = Frame::default()
                    .fill(theme.palette.ink)
                    .corner_radius(CornerRadius::same(theme.layout.status_chip_radius))
                    .inner_margin(theme.layout.status_chip_padding);
                chip.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 5.0, dot_color);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(status_text)
                                .size(13.0)
                                .color(theme.palette.ink_text)
                                .strong(),
                        );
                    });
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let gear = Button::new(
                        RichText::new("⚙").size(18.0).color(theme.palette.text_muted),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::same(10))
                    .min_size(Vec2::splat(20.0));
                    if ui.add(gear).clicked() {
                        self.current_page = AppPage::Settings;
                    }
                });
            },
        );

    }

    fn show_recording_panel(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
    ) {
        let top_space = ((ui.available_height()
            - theme.layout.recording_content_height(self.state.input_state))
            * 0.5)
            .max(theme.layout.record_top_padding_min);
        ui.add_space(top_space);

        ui.vertical_centered(|ui| {
            let button_size = Vec2::splat(theme.layout.record_button_size);
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
            let active = ctx.animate_bool_with_time(
                Id::new("record_button_active"),
                is_recording,
                0.18,
            );
            let phase = (pulse_driver * 0.85).fract();
            if active > 0.01 {
                for offset in [0.0_f32, 0.5_f32] {
                    let progress = (phase + offset).fract();
                    let radius = theme.layout.record_button_radius * (1.0 + 0.6 * progress);
                    let alpha = (90.0 * (1.0 - progress) * active).round() as u8;
                    if alpha > 0 {
                        painter.circle_stroke(
                            center,
                            radius,
                            Stroke::new(
                                theme.layout.record_pulse_width,
                                Color32::from_rgba_unmultiplied(
                                    theme.palette.accent.r(),
                                    theme.palette.accent.g(),
                                    theme.palette.accent.b(),
                                    alpha,
                                ),
                            ),
                        );
                    }
                }
            }

            let fill = if is_recording {
                theme.palette.accent
            } else {
                theme.palette.ink
            };
            painter.circle_filled(center, theme.layout.record_button_radius, fill);
            self.paint_mic_icon(&painter, center, theme.palette.ink_text);

            if response.clicked() && !is_busy {
                self.toggle_recording(ctx);
            }

            ui.add_space(theme.layout.record_hint_gap);
            self.show_shortcut_hint(ui, theme);
            ui.add_space(theme.layout.record_status_gap);

            if is_recording {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(self.recording_elapsed_label())
                            .font(FontId::monospace(20.0))
                            .color(theme.palette.text_primary),
                    );
                });
            } else if self.state.input_state == InputState::Processing {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("识别中…")
                            .font(FontId::monospace(17.0))
                            .color(theme.palette.text_muted),
                    );
                });
            } else {
                ui.add_space(22.0);
            }
        });
    }

    fn show_shortcut_hint(&self, ui: &mut egui::Ui, theme: &Theme) {
        if self.state.shortcut_recording {
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), 20.0),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("按下新的组合键…")
                                .size(11.0)
                                .color(theme.palette.text_muted),
                        );
                    });
                },
            );
            return;
        }

        if self.state.input_state == InputState::Recording {
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), 20.0),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            RichText::new("再按一次结束")
                                .size(11.0)
                                .color(theme.palette.text_muted),
                        );
                    });
                },
            );
            return;
        }

        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 34.0),
            Layout::top_down(Align::Center),
            |ui| {
                let row_width = self.shortcut_row_width(ui, theme);
                let left_space = ((ui.available_width() - row_width) * 0.5).max(0.0);
                ui.horizontal(|ui| {
                    if left_space > 0.0 {
                        ui.add_space(left_space);
                    }
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.allocate_ui_with_layout(
                        Vec2::new(self.shortcut_text_width(ui, theme, "按"), 32.0),
                        Layout::top_down(Align::Center),
                        |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("按")
                                        .size(11.0)
                                        .color(theme.palette.text_muted),
                                );
                            });
                        },
                    );
                    for part in self.state.shortcut.split('+') {
                        let key_frame = Frame::default()
                            .fill(theme.palette.card_bg)
                            .stroke(Stroke::new(1.0, theme.palette.border))
                            .corner_radius(CornerRadius::same(theme.layout.key_chip_radius))
                            .inner_margin(theme.layout.key_chip_padding);
                        key_frame.show(ui, |ui| {
                            ui.label(
                                RichText::new(part)
                                    .size(11.0)
                                    .color(theme.palette.text_primary)
                                    .family(egui::FontFamily::Monospace),
                            );
                        });
                    }
                    ui.allocate_ui_with_layout(
                        Vec2::new(self.shortcut_text_width(ui, theme, "开始口述"), 32.0),
                        Layout::top_down(Align::Center),
                        |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("开始口述")
                                        .size(11.0)
                                        .color(theme.palette.text_muted),
                                );
                            });
                        },
                    );
                });
            },
        );
        ui.add_space(theme.layout.small_gap);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 18.0),
            Layout::top_down(Align::Center),
            |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("或点击麦克风按钮")
                            .size(11.0)
                            .color(theme.palette.text_muted),
                    );
                });
            },
        );
    }

    fn shortcut_row_width(&self, ui: &egui::Ui, theme: &Theme) -> f32 {
        let spacing = 6.0;
        let mut width = self.shortcut_text_width(ui, theme, "按")
            + self.shortcut_text_width(ui, theme, "开始口述");

        for part in self.state.shortcut.split('+') {
            width += self.shortcut_key_width(ui, theme, part)
                + f32::from(theme.layout.key_chip_padding.left)
                + f32::from(theme.layout.key_chip_padding.right);
        }

        let item_count = 2 + self.state.shortcut.split('+').count();
        width + spacing * (item_count.saturating_sub(1) as f32)
    }

    fn shortcut_text_width(&self, ui: &egui::Ui, theme: &Theme, value: &str) -> f32 {
        ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(
                    value.to_owned(),
                    FontId::proportional(11.0),
                    theme.palette.text_primary,
                )
                .size()
                .x
        })
    }

    fn shortcut_key_width(&self, ui: &egui::Ui, theme: &Theme, value: &str) -> f32 {
        ui.fonts(|fonts| {
            fonts
                .layout_no_wrap(
                    value.to_owned(),
                    FontId::monospace(11.0),
                    theme.palette.text_primary,
                )
                .size()
                .x
        })
    }

    fn show_result_panel(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        theme: &Theme,
    ) {
        let divider_y = ui.max_rect().top() + theme.layout.bottom_divider_top_gap;
        let divider_left = ui.max_rect().left() - f32::from(theme.layout.result_panel_margin.left)
            + theme.layout.divider_inset;
        let divider_right = ui.max_rect().right()
            + f32::from(theme.layout.result_panel_margin.right)
            - theme.layout.divider_inset;
        ui.painter().line_segment(
            [egui::pos2(divider_left, divider_y), egui::pos2(divider_right, divider_y)],
            Stroke::new(1.0, theme.palette.divider),
        );
        self.state.delivery_target = DeliveryTarget::GenericInput;
        let control_band_height = theme.layout.bottom_divider_top_gap
            + theme.layout.section_gap
            + 32.0
            + theme.layout.small_gap;
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), control_band_height),
            Layout::right_to_left(Align::Center),
            |ui| {
                egui::ComboBox::from_id_salt("inline_input_mode")
                    .selected_text(self.state.input_mode.label())
                    .show_ui(ui, |ui| {
                        for mode in InputMode::ALL {
                            ui.selectable_value(&mut self.state.input_mode, mode, mode.label());
                        }
                    });
            },
        );
        let editor_frame = Frame::default()
            .fill(theme.palette.card_bg)
            .stroke(Stroke::new(1.0, theme.palette.border))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(10));
        editor_frame.show(ui, |ui| {
            let editor = TextEdit::multiline(&mut self.state.raw_text)
                .hint_text("刚刚识别的文字会出现在这里…")
                .desired_rows(theme.layout.editor_rows)
                .desired_width(f32::INFINITY)
                .frame(false);
            let response =
                ui.add_sized([ui.available_width(), theme.layout.editor_height], editor);
            if response.changed() {
                self.sync_hidden_delivery_text();
            }
        });

        ui.add_space(theme.layout.section_gap);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let checkbox = ui.checkbox(
                &mut self.state.auto_paste,
                RichText::new("识别后自动粘贴")
                    .size(11.0)
                    .color(theme.palette.text_muted),
            );
            if checkbox.changed() {
                self.sync_hidden_delivery_text();
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = theme.layout.section_gap;
                let send = Button::new(
                    RichText::new("发送")
                        .size(12.5)
                        .color(theme.palette.ink_text)
                        .strong(),
                )
                .fill(theme.palette.ink)
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::same(theme.layout.button_radius))
                .min_size(Vec2::new(
                    theme.layout.primary_button_width,
                    theme.layout.button_height,
                ));
                if ui.add(send).clicked() {
                    self.handle_send(ctx);
                }

                let copy = Button::new(
                    RichText::new("复制")
                        .size(12.0)
                        .color(theme.palette.text_primary),
                )
                .fill(theme.palette.card_bg)
                .stroke(Stroke::new(1.0, theme.palette.border))
                .corner_radius(CornerRadius::same(theme.layout.button_radius))
                .min_size(Vec2::new(
                    theme.layout.secondary_button_width,
                    theme.layout.button_height,
                ));
                if ui.add(copy).clicked() {
                    self.copy_visible_text(ctx);
                }
            });
        });
    }

    fn show_settings_page(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, theme: &Theme) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            Frame::default()
                .fill(theme.palette.card_bg)
                .stroke(Stroke::new(1.0, theme.palette.border))
                .corner_radius(CornerRadius::same(16))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new("外观").size(14.0).strong());
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.theme == ThemeMode::Paper, "纸感米白")
                            .clicked()
                        {
                            if self.theme != ThemeMode::Paper {
                                self.toggle_theme();
                            }
                        }
                        if ui
                            .selectable_label(self.theme == ThemeMode::Ink, "深色 IDE")
                            .clicked()
                        {
                            if self.theme != ThemeMode::Ink {
                                self.toggle_theme();
                            }
                        }
                    });

                    ui.separator();
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
                    ui.label(RichText::new("本地模型").size(14.0).strong());
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("模型目录")
                            .size(12.0)
                            .color(theme.palette.text_muted),
                    );
                    ui.text_edit_singleline(&mut self.state.local_model_dir);
                    ui.add_space(6.0);
                    if ui.button("检查 SenseVoice").clicked() {
                        check_local_model(&mut self.state);
                        if !self.state.local_model_ready {
                            self.show_toast_now(
                                ctx,
                                ToastKind::Error,
                                self.state.local_model_status.clone(),
                            );
                        }
                    }
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(if self.state.local_model_ready {
                            "模型已就绪"
                        } else {
                            "模型未就绪"
                        })
                        .size(12.0)
                        .color(if self.state.local_model_ready {
                            theme.palette.success
                        } else {
                            theme.palette.accent
                        }),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("状态")
                            .size(12.0)
                            .color(theme.palette.text_muted),
                    );
                    ui.label(RichText::new(&self.state.local_model_status).size(12.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("摘要")
                            .size(12.0)
                            .color(theme.palette.text_muted),
                    );
                    ui.label(RichText::new(&self.state.local_model_summary).size(12.0));
                });
        });
    }

    fn show_toast(&mut self, ctx: &egui::Context, theme: &Theme) {
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
                        Color32::from_rgba_unmultiplied(
                            theme.palette.accent.r(),
                            theme.palette.accent.g(),
                            theme.palette.accent.b(),
                            (190.0 * fade).round() as u8,
                        ),
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

    fn status_chip(&self, ctx: &egui::Context, theme: &Theme) -> (&'static str, Color32) {
        match self.state.input_state {
            InputState::Idle | InputState::Success => ("准备就绪", theme.palette.success),
            InputState::Recording => {
                let phase = ((ctx.input(|input| input.time) * 4.0).sin() * 0.5 + 0.5) as f32;
                let alpha = (140.0 + phase * 115.0).round() as u8;
                (
                    "录音中",
                    Color32::from_rgba_unmultiplied(
                        theme.palette.accent.r(),
                        theme.palette.accent.g(),
                        theme.palette.accent.b(),
                        alpha,
                    ),
                )
            }
            InputState::Processing => ("识别中", theme.palette.processing),
            InputState::Error => ("出错了", theme.palette.accent),
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
        let stroke = Stroke::new(3.6, color);
        let body = egui::Rect::from_center_size(
            center + egui::vec2(0.0, -8.0),
            egui::vec2(16.0, 25.0),
        );
        painter.rect_stroke(body, CornerRadius::same(8), stroke, StrokeKind::Inside);
        painter.line_segment(
            [center + egui::vec2(-13.0, -2.0), center + egui::vec2(-13.0, 4.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(13.0, -2.0), center + egui::vec2(13.0, 4.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(-13.0, 4.0), center + egui::vec2(-6.0, 12.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(13.0, 4.0), center + egui::vec2(6.0, 12.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(0.0, 14.0), center + egui::vec2(0.0, 23.0)],
            stroke,
        );
        painter.line_segment(
            [center + egui::vec2(-10.0, 25.0), center + egui::vec2(10.0, 25.0)],
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
        InputMode::BugReport => [
            "问题描述:",
            input,
            "",
            "复现步骤:",
            "1. 待补充",
            "2. 待补充",
        ]
        .join("\n"),
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
