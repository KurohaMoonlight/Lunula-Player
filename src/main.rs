#![windows_subsystem = "windows"]

use eframe::egui;
use egui::{Color32, Stroke};
use lunulaplayer::app::LunulaApp;

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/icon_256.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1240.0, 820.0])
        .with_min_inner_size([1040.0, 680.0])
        .with_title("LunulaPlayer 🌙 - Audiophile Music Player");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "LunulaPlayer",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = Color32::from_rgb(248, 250, 252);
            visuals.window_fill = Color32::WHITE;
            visuals.override_text_color = Some(Color32::from_rgb(15, 23, 42));
            visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(51, 65, 85);
            visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(30, 41, 59);
            visuals.widgets.hovered.fg_stroke.color = Color32::from_rgb(15, 23, 42);
            visuals.widgets.active.fg_stroke.color = Color32::BLACK;
            visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(203, 213, 225));
            cc.egui_ctx.set_visuals(visuals);

            let mut fonts = egui::FontDefinitions::default();
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
                fonts.font_data.insert(
                    "segoe_ui".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "segoe_ui".to_owned());
            }
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\malgun.ttf") {
                fonts.font_data.insert(
                    "malgun".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("malgun".to_owned());
            }
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\consola.ttf") {
                fonts.font_data.insert(
                    "consola".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, "consola".to_owned());
            }
            cc.egui_ctx.set_fonts(fonts);

            let mut style = (*cc.egui_ctx.style_of(egui::Theme::Light)).clone();
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::new(18.5, egui::FontFamily::Proportional)),
                (egui::TextStyle::Body, egui::FontId::new(13.5, egui::FontFamily::Proportional)),
                (egui::TextStyle::Button, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Small, egui::FontId::new(11.5, egui::FontFamily::Proportional)),
                (egui::TextStyle::Monospace, egui::FontId::new(12.5, egui::FontFamily::Monospace)),
            ].into();
            let style_arc = std::sync::Arc::new(style);
            cc.egui_ctx.set_style_of(egui::Theme::Light, style_arc.clone());
            cc.egui_ctx.set_style_of(egui::Theme::Dark, style_arc);

            Ok(stacker::grow(32 * 1024 * 1024, || {
                Box::new(LunulaApp::new(cc))
            }))
        }),
    )
}
