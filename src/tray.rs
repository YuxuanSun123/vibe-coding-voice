use anyhow::{Context, Result};
use eframe::egui::IconData;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct TrayController {
    _tray_icon: TrayIcon,
}

impl TrayController {
    pub fn install() -> Result<Self> {
        let icon = load_tray_icon().context("创建托盘图标失败。")?;
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Vibe Coding Voice")
            .with_icon(icon)
            .build()
            .context("初始化系统托盘失败。")?;

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }
}

fn load_tray_icon() -> Result<Icon> {
    let icon_data = build_icon_data();
    Icon::from_rgba(
        icon_data.rgba.clone(),
        icon_data.width,
        icon_data.height,
    )
    .context("生成托盘 RGBA 图标失败。")
}

fn build_icon_data() -> IconData {
    let size = 32;
    let mut rgba = vec![0_u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let border = x < 3 || y < 3 || x >= size - 3 || y >= size - 3;
            let active_band = (10..22).contains(&x) && (8..24).contains(&y);
            let color = if border {
                [53, 132, 228, 255]
            } else if active_band {
                [118, 185, 255, 255]
            } else {
                [20, 24, 31, 230]
            };
            rgba[idx..idx + 4].copy_from_slice(&color);
        }
    }

    IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}
