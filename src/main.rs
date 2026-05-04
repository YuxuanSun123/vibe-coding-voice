mod app;
mod hotkeys;
mod sensevoice;
mod services;
mod state;
mod tray;

use app::VoiceInputNativeApp;
use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::fs;
use std::sync::Arc;

const WINDOW_TITLE: &str = "Vibe Coding Voice";
const WINDOW_INNER_SIZE: [f32; 2] = [560.0, 520.0];
const WINDOW_MIN_INNER_SIZE: [f32; 2] = [500.0, 440.0];
const FONT_CANDIDATES: [&str; 3] = [
    r"c:\Windows\Fonts\msyh.ttc",
    r"c:\Windows\Fonts\msyhl.ttc",
    r"c:\Windows\Fonts\simhei.ttf",
];

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    for path in FONT_CANDIDATES {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };

        fonts.font_data.insert(
            "windows_cjk".to_string(),
            Arc::new(FontData::from_owned(bytes)),
        );

        if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
            family.insert(0, "windows_cjk".to_string());
        }
        if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
            family.insert(0, "windows_cjk".to_string());
        }

        ctx.set_fonts(fonts);
        return;
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(WINDOW_INNER_SIZE)
            .with_min_inner_size(WINDOW_MIN_INNER_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            // 视觉主题在 app 内根据 ThemeMode 切换，这里不再写死
            Ok(Box::new(VoiceInputNativeApp::default()))
        }),
    )
}
