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

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let candidates = [
        r"c:\Windows\Fonts\msyh.ttc",
        r"c:\Windows\Fonts\msyhl.ttc",
        r"c:\Windows\Fonts\simhei.ttf",
    ];

    for path in candidates {
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
            .with_title("Vibe Coding Voice")
            .with_inner_size([560.0, 520.0])
            .with_min_inner_size([500.0, 440.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vibe Coding Voice",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            // 视觉主题在 app 内根据 ThemeMode 切换，这里不再写死
            Ok(Box::new(VoiceInputNativeApp::default()))
        }),
    )
}
