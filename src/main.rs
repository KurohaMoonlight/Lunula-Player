#![windows_subsystem = "windows"]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

#[cfg(windows)]
fn show_crash_dialog(msg: &str) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    let title: Vec<u16> = std::ffi::OsStr::new("LunulaPlayer - Fatal Error Occurred\0")
        .encode_wide()
        .collect();
    let text: Vec<u16> = std::ffi::OsStr::new(&format!("{}\0", msg))
        .encode_wide()
        .collect();

    unsafe {
        MessageBoxW(0 as _, text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
    }
}

#[cfg(not(windows))]
fn show_crash_dialog(_msg: &str) {}

fn main() -> eframe::Result {
    // Install panic hook for rock-solid stability & crash reporting
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".into());
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        let err_msg = format!(
            "LunulaPlayer has encountered an unexpected error:\n\nMessage: {}\nLocation: {}\n\nA crash report has been saved to lunula_crash.log.",
            payload, location
        );
        eprintln!("[FATAL PANIC] {}", err_msg);

        if let Ok(mut file) = std::fs::File::create("lunula_crash.log") {
            use std::io::Write;
            let _ = writeln!(file, "=== LUNULAPLAYER CRASH REPORT ===");
            let _ = writeln!(file, "Timestamp: {:?}", std::time::SystemTime::now());
            let _ = writeln!(file, "Error: {}", payload);
            let _ = writeln!(file, "Location: {}", location);
            let _ = writeln!(
                file,
                "Thread: {:?}",
                std::thread::current().name().unwrap_or("unnamed")
            );
        }

        show_crash_dialog(&err_msg);
    }));

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1240.0, 820.0])
        .with_min_inner_size([1040.0, 680.0])
        .with_title("LunulaPlayer 🌙 - Audiophile Music Player");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        system_font_fallback: false,
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
            if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
                fonts.font_data.insert(
                    "segoe_ui_emoji".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("segoe_ui_emoji".to_owned());
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
