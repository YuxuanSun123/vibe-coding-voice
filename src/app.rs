use crate::hotkeys::{HotkeyController, build_hotkey, install_event_handler};
use crate::services::{
    begin_input, check_local_model, clear_result, finish_input, prepare_recording_overlay_host,
    run_practice_flow,
};
use crate::state::{DeliveryTarget, InputMode, NativeAppState};
use crate::tray::TrayController;
use eframe::egui;
use global_hotkey::hotkey::Code;
use std::time::Duration;

pub struct VoiceInputNativeApp {
    pub state: NativeAppState,
    hotkey_controller: Option<HotkeyController>,
    tray_controller: Option<TrayController>,
}

impl Default for VoiceInputNativeApp {
    fn default() -> Self {
        let mut app = Self {
            state: NativeAppState::default(),
            hotkey_controller: None,
            tray_controller: None,
        };
        app.initialize_shortcut();
        app.initialize_tray();
        app.initialize_overlay_host();
        app
    }
}

impl eframe::App for VoiceInputNativeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_event_handler(ctx);
        ctx.request_repaint_after(Duration::from_millis(50));
        self.capture_shortcut_if_needed(ctx);
        self.handle_global_hotkey();

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("Vibe Coding Voice");
                ui.label("纯桌面输入法骨架");
                ui.separator();

                ui.label(format!("状态: {}", self.state.input_state.label()));
                ui.label(format!("快捷键: {}", self.state.shortcut));
                ui.label(format!("目标: {}", self.state.delivery_target.label()));
                ui.separator();

                ui.label("状态消息");
                ui.monospace(&self.state.status_message);
                ui.separator();
                ui.label("快捷键状态");
                ui.monospace(&self.state.shortcut_status);
                ui.separator();
                ui.label("最近一次录音");
                ui.monospace(&self.state.last_recording_info);
                ui.separator();

                if ui.button("测试原生按钮事件").clicked() {
                    self.state.debug_clicks += 1;
                    self.state.status_message =
                        format!("原生点击事件正常，累计点击 {} 次。", self.state.debug_clicks);
                }

                ui.label(format!("调试点击: {}", self.state.debug_clicks));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("一次语音输入");
            ui.label("这个版本是纯桌面程序，不再依赖 WebView。先把输入法主流程、状态和模块骨架搭起来。");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("开始口述").clicked() {
                    begin_input(&mut self.state);
                }

                if ui.button("结束并发送").clicked() {
                    finish_input(&mut self.state);
                }

                if ui.button("演练输入链路").clicked() {
                    run_practice_flow(&mut self.state);
                }

                if ui.button("清空结果").clicked() {
                    clear_result(&mut self.state);
                }
            });

            ui.add_space(12.0);

            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    ui.label("快捷键");
                    ui.horizontal(|ui| {
                        let capture_label = if self.state.shortcut_recording {
                            "按下新的组合键..."
                        } else {
                            &self.state.shortcut
                        };
                        ui.monospace(capture_label);

                        let button_text = if self.state.shortcut_recording {
                            "取消"
                        } else {
                            "更改快捷键"
                        };
                        if ui.button(button_text).clicked() {
                            self.toggle_shortcut_recording();
                        }
                    });
                    ui.end_row();

                    ui.label("投送目标");
                    egui::ComboBox::from_id_salt("delivery_target")
                        .selected_text(self.state.delivery_target.label())
                        .show_ui(ui, |ui| {
                            for target in DeliveryTarget::ALL {
                                ui.selectable_value(
                                    &mut self.state.delivery_target,
                                    target,
                                    target.label(),
                                );
                            }
                        });
                    ui.end_row();

                    ui.label("意图模式");
                    egui::ComboBox::from_id_salt("input_mode")
                        .selected_text(self.state.input_mode.label())
                        .show_ui(ui, |ui| {
                            for mode in InputMode::ALL {
                                ui.selectable_value(&mut self.state.input_mode, mode, mode.label());
                            }
                        });
                    ui.end_row();
                });

            ui.add_space(8.0);
            ui.checkbox(&mut self.state.auto_paste, "自动粘贴到目标输入框");

            ui.add_space(12.0);
            ui.group(|ui| {
                ui.heading("本地模型");
                ui.label("先把 SenseVoice Small 目录检查和兼容层接上，再往后串录音。");
                ui.add_space(6.0);
                ui.label("模型目录");
                ui.text_edit_singleline(&mut self.state.local_model_dir);

                ui.horizontal(|ui| {
                    if ui.button("检查 SenseVoice").clicked() {
                        check_local_model(&mut self.state);
                    }

                    let status_label = if self.state.local_model_ready {
                        "已就绪"
                    } else {
                        "未就绪"
                    };
                    ui.label(format!("模型状态: {status_label}"));
                });

                ui.label("模型检查结果");
                ui.monospace(&self.state.local_model_status);
                ui.add_space(4.0);
                ui.label("兼容层摘要");
                ui.monospace(&self.state.local_model_summary);
            });

            ui.add_space(12.0);
            ui.label("口述练习文本");
            ui.add(
                egui::TextEdit::multiline(&mut self.state.practice_text)
                    .desired_rows(5)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(12.0);
            ui.columns(2, |columns| {
                columns[0].group(|ui| {
                    ui.label("识别文本");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.raw_text)
                            .desired_rows(10)
                            .desired_width(f32::INFINITY),
                    );
                });

                columns[1].group(|ui| {
                    ui.label("投送文本");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.delivered_text)
                            .desired_rows(10)
                            .desired_width(f32::INFINITY),
                    );
                });
            });

            ui.add_space(12.0);
            ui.label("下一步");
            ui.label("- 补更稳的全局快捷键冲突提示");
            ui.label("- 做快捷键持久化");
            ui.label("- 托盘菜单和最小化到托盘");
        });
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
                }
            },
            Err(error) => {
                self.state.shortcut_registered = false;
                self.state.shortcut_status = format!("系统级快捷键不可用: {error:#}");
            }
        }
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
            }
        }
    }

    fn initialize_overlay_host(&mut self) {
        if let Err(error) = prepare_recording_overlay_host() {
            self.state.status_message = format!(
                "{} 录音浮层预热失败: {error:#}",
                self.state.status_message
            );
        }
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
                return;
            }
        };

        if hotkey.mods.is_empty() && !matches!(hotkey.key, Code::F1 | Code::F2 | Code::F3 | Code::F4 | Code::F5 | Code::F6 | Code::F7 | Code::F8 | Code::F9 | Code::F10 | Code::F11 | Code::F12) {
            self.state.shortcut_status =
                "建议至少带一个 Ctrl / Alt / Shift，或直接使用 F1-F12。".to_string();
            return;
        }

        match self.apply_shortcut(hotkey) {
            Ok(()) => {
                self.state.shortcut_recording = false;
            }
            Err(error) => {
                self.state.shortcut_status = format!("{error:#}");
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

    fn handle_global_hotkey(&mut self) {
        let Some(controller) = self.hotkey_controller.as_ref() else {
            return;
        };
        if self.state.shortcut_recording || !controller.poll_triggered() {
            return;
        }

        self.state.status_message = format!("检测到全局快捷键 `{}`。", self.state.shortcut);
        if self.state.input_state == crate::state::InputState::Recording {
            finish_input(&mut self.state);
        } else {
            begin_input(&mut self.state);
        }
    }
}
