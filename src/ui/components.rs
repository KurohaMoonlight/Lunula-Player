use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

pub fn format_time(sec: f32) -> String {
    let m = (sec as u32) / 60;
    let s = (sec as u32) % 60;
    format!("{:02}:{:02}", m, s)
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        ((1.0 - t) * a.r() as f32 + t * b.r() as f32) as u8,
        ((1.0 - t) * a.g() as f32 + t * b.g() as f32) as u8,
        ((1.0 - t) * a.b() as f32 + t * b.b() as f32) as u8,
        ((1.0 - t) * a.a() as f32 + t * b.a() as f32) as u8,
    )
}

pub fn render_card_frame<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
        .corner_radius(12.0)
        .inner_margin(16.0)
        .show(ui, add_contents)
}

// Render an individual modern Nordic stat card
pub fn render_stat_box(ui: &mut egui::Ui, icon: &str, label: &str, value: &str, sub: &str, accent_color: Color32) {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(135.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).size(14.0));
                    ui.label(egui::RichText::new(label).strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                });
                ui.add_space(3.0);
                ui.label(egui::RichText::new(value).strong().size(16.5).color(accent_color));
                ui.add_space(1.0);
                ui.label(egui::RichText::new(sub).size(11.0).color(Color32::from_rgb(100, 116, 139)));
            });
        });
}

// Draw stylized audiophile vinyl sleeve cover
pub fn draw_album_art(painter: &egui::Painter, rect: Rect, title: &str, is_hires: bool) {
    painter.rect_filled(rect, 8.0, Color32::from_rgb(15, 23, 42)); // Matte dark sleeve
    let center = rect.center();
    let vinyl_radius = (rect.height() * 0.44).min(rect.width() * 0.36);

    // Groove rings
    painter.circle_filled(center, vinyl_radius, Color32::from_rgb(24, 24, 27));
    painter.circle_stroke(center, vinyl_radius * 0.88, Stroke::new(1.0, Color32::from_rgb(39, 39, 42)));
    painter.circle_stroke(center, vinyl_radius * 0.72, Stroke::new(1.0, Color32::from_rgb(45, 45, 48)));
    painter.circle_stroke(center, vinyl_radius * 0.56, Stroke::new(1.0, Color32::from_rgb(39, 39, 42)));

    // Center spindle label
    let label_color = if is_hires {
        Color32::from_rgb(16, 185, 129) // Emerald
    } else {
        Color32::from_rgb(14, 165, 233) // Sky
    };
    painter.circle_filled(center, vinyl_radius * 0.36, label_color);
    painter.circle_filled(center, vinyl_radius * 0.12, Color32::from_rgb(15, 23, 42));

    // Initial on label
    let initial = title.chars().next().unwrap_or('♫').to_uppercase().to_string();
    painter.text(
        Pos2::new(center.x, center.y - 1.0),
        egui::Align2::CENTER_CENTER,
        initial,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );

    if is_hires {
        let badge_rect = Rect::from_min_size(
            Pos2::new(rect.max.x - 52.0, rect.min.y + 6.0),
            Vec2::new(46.0, 16.0),
        );
        painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(6, 78, 59));
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "HI-RES",
            egui::FontId::proportional(9.0),
            Color32::from_rgb(110, 231, 183),
        );
    }
}

// Draw artist avatar with initials
pub fn draw_artist_avatar(painter: &egui::Painter, center: Pos2, radius: f32, name: &str) {
    let mut hash: u64 = 5381;
    for b in name.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u64);
    }
    let colors = [
        Color32::from_rgb(14, 165, 233),  // Sky
        Color32::from_rgb(99, 102, 241),  // Indigo
        Color32::from_rgb(16, 185, 129),  // Emerald
        Color32::from_rgb(245, 158, 11),  // Amber
        Color32::from_rgb(236, 72, 153),  // Pink
        Color32::from_rgb(139, 92, 246),  // Purple
    ];
    let col = colors[(hash % colors.len() as u64) as usize];

    painter.circle_filled(center, radius, col);
    painter.circle_stroke(center, radius, Stroke::new(1.5, Color32::WHITE));

    let parts: Vec<&str> = name.split_whitespace().collect();
    let initials = if parts.len() >= 2 {
        let c1 = parts[0].chars().next().unwrap_or('A');
        let c2 = parts[1].chars().next().unwrap_or(' ');
        format!("{}{}", c1, c2).to_uppercase()
    } else {
        name.chars().take(2).collect::<String>().to_uppercase()
    };

    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        initials,
        egui::FontId::proportional(radius * 0.75),
        Color32::WHITE,
    );
}

// Renders a clean 5-finger open hand cursor (Thumb + 4 fingers + palm)
pub fn draw_five_finger_hand(painter: &egui::Painter, pos: Pos2) {
    let shadow_color = Color32::from_black_alpha(45);
    let border_color = Color32::from_rgb(15, 23, 42);
    let fill_color = Color32::WHITE;

    let shadow_offset = Vec2::new(1.2, 1.8);

    // 5 fingers: Thumb + 4 fingers
    let fingers = [
        (Pos2::new(-3.0, 3.5), Pos2::new(-8.5, -0.5), 2.2),
        (Pos2::new(-3.0, 0.0), Pos2::new(-3.0, -8.5), 2.0),
        (Pos2::new(0.2, 0.0), Pos2::new(0.2, -11.5), 2.0),
        (Pos2::new(3.4, 0.0), Pos2::new(3.4, -9.0), 2.0),
        (Pos2::new(6.4, 1.5), Pos2::new(6.4, -6.5), 1.8),
    ];

    let palm_size = Vec2::new(13.0, 11.0);

    // 1. Drop Shadow pass
    let p_shadow = pos + shadow_offset;
    for (start, end, r) in &fingers {
        painter.line_segment(
            [p_shadow + start.to_vec2(), p_shadow + end.to_vec2()],
            Stroke::new(*r * 2.0 + 2.0, shadow_color),
        );
    }
    let shadow_palm = Rect::from_center_size(p_shadow + Vec2::new(1.0, 4.5), palm_size);
    painter.rect_filled(shadow_palm, 4.0, shadow_color);

    // 2. Dark Outline pass
    for (start, end, r) in &fingers {
        painter.line_segment(
            [pos + start.to_vec2(), pos + end.to_vec2()],
            Stroke::new(*r * 2.0 + 2.4, border_color),
        );
    }
    let palm_rect = Rect::from_center_size(pos + Vec2::new(1.0, 4.5), palm_size);
    painter.rect_stroke(palm_rect, 4.0, Stroke::new(1.4, border_color), egui::StrokeKind::Outside);

    // 3. Crisp White Fill pass
    for (start, end, r) in &fingers {
        painter.line_segment(
            [pos + start.to_vec2(), pos + end.to_vec2()],
            Stroke::new(*r * 2.0, fill_color),
        );
    }
    painter.rect_filled(palm_rect, 4.0, fill_color);
}

pub fn open_file_in_explorer(path_str: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,{}", path_str))
            .spawn();
    }
}

