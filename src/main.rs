mod app;
mod hotkeys;
mod sensevoice;
mod services;
mod state;
mod tray;

use app::VoiceInputNativeApp;
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily};
use std::fs;
use std::sync::Arc;

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let candidates = [
        r"c:\Windows\Fonts\simhei.ttf",
        r"c:\Windows\Fonts\msyh.ttc",
        r"c:\Windows\Fonts\msyhl.ttc",
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

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = Color32::from_rgb(245, 242, 236);
    visuals.panel_fill = Color32::from_rgb(245, 242, 236);
    visuals.extreme_bg_color = Color32::from_rgb(253, 251, 246);
    visuals.faint_bg_color = Color32::from_rgb(239, 235, 226);
    visuals.override_text_color = Some(Color32::from_rgb(42, 38, 32));
    visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(225, 220, 208);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(253, 251, 246);
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(225, 220, 208));
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(239, 235, 226);
    visuals.widgets.active.bg_fill = Color32::from_rgb(21, 20, 15);
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(196, 66, 28, 32);
    visuals.selection.stroke = egui::Stroke::new(1.0, Color32::from_rgb(196, 66, 28));
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(0);
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
    ctx.set_style(style);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Vibe Coding Voice")
            .with_inner_size([560.0, 480.0])
            .with_min_inner_size([480.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Vibe Coding Voice",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            configure_visuals(&cc.egui_ctx);
            Ok(Box::new(VoiceInputNativeApp::default()))
        }),
    )
}
