use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::NodeType;

pub fn get_rect_perimeter_pos(r: Rect, dist: f32) -> Pos2 {
    let w = r.width();
    let h = r.height();
    let p = 2.0 * (w + h);
    let d = (dist % p + p) % p;
    if d < w {
        Pos2::new(r.min.x + d, r.min.y)
    } else if d < w + h {
        Pos2::new(r.max.x, r.min.y + (d - w))
    } else if d < 2.0 * w + h {
        Pos2::new(r.max.x - (d - (w + h)), r.max.y)
    } else {
        Pos2::new(r.min.x, r.max.y - (d - (2.0 * w + h)))
    }
}

pub fn render_routing_help_button(
    ui: &mut egui::Ui,
    title: &str,
    node_type: NodeType,
    active_tutorial: &mut Option<NodeType>,
    tutorial_step: &mut usize,
) {
    let btn = ui.add(
        egui::Button::new(
            egui::RichText::new("? Panduan")
                .size(9.5)
                .strong()
                .color(Color32::from_rgb(14, 116, 144))
        )
        .fill(Color32::from_rgb(240, 249, 255))
        .stroke(Stroke::new(1.0, Color32::from_rgb(186, 230, 253)))
        .corner_radius(6.0)
    );
    if btn.clicked() {
        *active_tutorial = Some(node_type);
        *tutorial_step = 0;
    }
    btn.on_hover_text(format!("Klik untuk membuka panduan slide & routing {}", title));
}

#[inline]
pub fn get_node_size(node_type: &NodeType) -> Vec2 {
    match node_type {
        NodeType::TubeWarmer => Vec2::new(228.0, 102.0),
        _ => Vec2::new(198.0, 84.0),
    }
}

pub fn get_interactive_slider_bounds_and_val(node_type: &NodeType, world_rect: Rect, node_width: f32, pos_x: f32) -> Option<(Rect, f32)> {
    match node_type {
        NodeType::LowPassFilter => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 20.0, world_rect.min.y + 24.0), Vec2::new(node_width - 40.0, 24.0));
            let track_min_x = world_rect.min.x + 24.0;
            let track_w = (node_width - 48.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            let val = (50.0 * (400.0_f32).powf(norm)).clamp(50.0, 20000.0);
            Some((hit, val))
        }
        NodeType::HighPassFilter => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 20.0, world_rect.min.y + 24.0), Vec2::new(node_width - 40.0, 24.0));
            let track_min_x = world_rect.min.x + 24.0;
            let track_w = (node_width - 48.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            let val = (20.0 * (1000.0_f32).powf(norm)).clamp(20.0, 20000.0);
            Some((hit, val))
        }
        NodeType::BandPassFilter => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 20.0, world_rect.min.y + 24.0), Vec2::new(node_width - 40.0, 24.0));
            let track_min_x = world_rect.min.x + 24.0;
            let track_w = (node_width - 48.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            let val = (20.0 * (1000.0_f32).powf(norm)).clamp(20.0, 20000.0);
            Some((hit, val))
        }
        NodeType::Crossover => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 20.0, world_rect.min.y + 24.0), Vec2::new(node_width - 40.0, 24.0));
            let track_min_x = world_rect.min.x + 24.0;
            let track_w = (node_width - 48.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            let val = (40.0 * (200.0_f32).powf(norm)).clamp(40.0, 8000.0);
            Some((hit, val))
        }
        NodeType::Mixer => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 26.0, world_rect.min.y + 34.0), Vec2::new(node_width - 52.0, 32.0));
            let track_min_x = world_rect.min.x + 30.0;
            let track_w = (node_width - 60.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, norm))
        }
        NodeType::TubeWarmer => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 86.0, world_rect.min.y + 30.0), Vec2::new(node_width - 110.0, 56.0));
            let track_min_x = world_rect.min.x + 92.0;
            let track_w = (node_width - 120.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::StereoWidener => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 24.0, world_rect.min.y + 34.0), Vec2::new(node_width - 48.0, 32.0));
            let track_min_x = world_rect.min.x + 28.0;
            let track_w = (node_width - 56.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::VolumeBooster => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 24.0, world_rect.min.y + 34.0), Vec2::new(node_width - 48.0, 32.0));
            let track_min_x = world_rect.min.x + 28.0;
            let track_w = (node_width - 56.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 24.0).clamp(0.0, 24.0)))
        }
        NodeType::AnalogPreamp => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 24.0, world_rect.min.y + 34.0), Vec2::new(node_width - 48.0, 32.0));
            let track_min_x = world_rect.min.x + 30.0;
            let track_w = (node_width - 60.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (-24.0 + norm * 36.0).clamp(-24.0, 12.0)))
        }
        NodeType::DynamicLoudness => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 24.0, world_rect.min.y + 34.0), Vec2::new(node_width - 48.0, 32.0));
            let track_min_x = world_rect.min.x + 28.0;
            let track_w = (node_width - 56.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::SteppedAttenuator => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 22.0, world_rect.min.y + 34.0), Vec2::new(node_width - 76.0, 32.0));
            let track_min_x = world_rect.min.x + 26.0;
            let track_w = (node_width - 84.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (-60.0 + norm * 60.0).clamp(-60.0, 0.0).round()))
        }
        NodeType::OpticalLeveler => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 24.0, world_rect.min.y + 34.0), Vec2::new(node_width - 48.0, 32.0));
            let track_min_x = world_rect.min.x + 28.0;
            let track_w = (node_width - 56.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::StereoBalance => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 22.0, world_rect.min.y + 34.0), Vec2::new(node_width - 44.0, 32.0));
            let track_min_x = world_rect.min.x + 26.0;
            let track_w = (node_width - 52.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (-100.0 + norm * 200.0).clamp(-100.0, 100.0)))
        }
        _ => None,
    }
}

// ============================================================================
// 🌊 RIVER STREAM GRAPHICS PALETTE & GEOMETRY HELPERS (Splitter & Summing)
// ============================================================================

/// Palette warna representasi aliran sungai Lunula:
/// • Dry (Jalur A): Warm Orange (Suara murni/kering/asli tanpa efek)
/// • Wet (Jalur B): Aqua Cyan (Suara basah dengan modulasi/filter)
/// • Batas Tanggul: Slate Grey (Batas headroom 0 dBFS pada DAC)
/// • Air Meluber: Crimson Red (Indikator clipping/luapan digital audio)
pub const RIVER_COLOR_DRY: Color32 = Color32::from_rgb(251, 146, 60);      // #FB923C Warm Orange
pub const RIVER_COLOR_WET: Color32 = Color32::from_rgb(56, 189, 248);      // #38BDF8 Aqua Cyan
pub const RIVER_COLOR_BANK: Color32 = Color32::from_rgb(71, 85, 105);      // #475569 Slate Grey
pub const RIVER_COLOR_OVERFLOW: Color32 = Color32::from_rgb(239, 68, 68);  // #EF4444 Crimson Alert

/// Blending dua warna sungai secara linier (sRGB interpolation).
#[inline]
pub fn blend_river_colors(color_a: Color32, color_b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = (color_a.r() as f32 * (1.0 - t) + color_b.r() as f32 * t).round() as u8;
    let g = (color_a.g() as f32 * (1.0 - t) + color_b.g() as f32 * t).round() as u8;
    let b = (color_a.b() as f32 * (1.0 - t) + color_b.b() as f32 * t).round() as u8;
    let a = (color_a.a() as f32 * (1.0 - t) + color_b.a() as f32 * t).round() as u8;
    Color32::from_rgba_premultiplied(r, g, b, a)
}

/// Struktur data status luapan / overflow sungai audio
#[derive(Clone, Copy, Debug)]
pub struct RiverOverflowState {
    pub is_overflow: bool,
    pub overflow_factor: f32, // 0.0 jika di dalam tanggul, > 0.0 jika meluap
    pub nominal_level: f32,   // Perkiraan amplitudo (0.0 .. 1.0+)
}

/// Menghitung apakah debit summing melebihi batas tanggul (0 dBFS headroom).
#[inline]
pub fn evaluate_summing_overflow(
    weight_a: f32,
    weight_b: f32,
    live_peak: f32,
) -> RiverOverflowState {
    let combined_gain = weight_a + weight_b;
    let effective_peak = if live_peak > 0.01 {
        live_peak * combined_gain
    } else {
        combined_gain * 0.8
    };

    let is_overflow = effective_peak > 1.0;
    let overflow_factor = if is_overflow {
        (effective_peak - 1.0).clamp(0.0, 1.5)
    } else {
        0.0
    };

    RiverOverflowState {
        is_overflow,
        overflow_factor,
        nominal_level: effective_peak,
    }
}

/// Menghasilkan titik-titik kurva Bézier kubik yang mulus untuk sungai horizontal.
pub fn generate_river_bezier_points(
    start: Pos2,
    end: Pos2,
    steps: usize,
) -> Vec<Pos2> {
    let dx = (end.x - start.x).abs().max(10.0) * 0.5;
    let cp1 = Pos2::new(start.x + dx, start.y);
    let cp2 = Pos2::new(end.x - dx, end.y);
    let mut pts = Vec::with_capacity(steps + 1);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let u = 1.0 - t;
        let x = u * u * u * start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * end.x;
        let y = u * u * u * start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * end.y;
        pts.push(Pos2::new(x, y));
    }
    pts
}

/// Menggambar satu aliran pita sungai (river stream ribbon) lengkap dengan
/// batas tanggul, riak air (ripples), dan efek luapan merah saat clipping.
pub fn draw_river_stream(
    painter: &egui::Painter,
    points: &[Pos2],
    stream_width: f32,
    stream_color: Color32,
    overflow_state: RiverOverflowState,
    anim_phase: f32,
    is_playing: bool,
) {
    if points.len() < 2 {
        return;
    }

    // 1. Jika terindikasi meluber: gambar semburat luapan air merah di luar tanggul
    if overflow_state.is_overflow {
        let spill_width = stream_width + 4.0 + overflow_state.overflow_factor * 8.0;
        let spill_color = Color32::from_rgba_premultiplied(
            RIVER_COLOR_OVERFLOW.r(),
            RIVER_COLOR_OVERFLOW.g(),
            RIVER_COLOR_OVERFLOW.b(),
            75,
        );
        painter.add(egui::Shape::line(points.to_vec(), Stroke::new(spill_width, spill_color)));
    }

    // 2. Aliran utama air sungai
    let active_color = if overflow_state.is_overflow {
        blend_river_colors(stream_color, RIVER_COLOR_OVERFLOW, (overflow_state.overflow_factor * 0.8).clamp(0.25, 0.85))
    } else {
        stream_color
    };
    painter.add(egui::Shape::line(points.to_vec(), Stroke::new(stream_width, active_color)));

    // 3. Efek riak air halus (water ripples) saat musik berputar (ultra-lightweight)
    if is_playing {
        let t = anim_phase.fract();
        let idx = ((points.len() - 1) as f32 * t).round() as usize;
        if let Some(&p) = points.get(idx) {
            painter.circle_filled(
                p,
                (stream_width * 0.35).clamp(1.5, 3.5),
                Color32::from_rgba_premultiplied(255, 255, 255, 175),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_river_palette_and_blending() {
        // At mix = 0.0 (Dry murni), should be exactly Dry (Orange)
        let dry = blend_river_colors(RIVER_COLOR_DRY, RIVER_COLOR_WET, 0.0);
        assert_eq!(dry, RIVER_COLOR_DRY);

        // At mix = 1.0 (Wet murni), should be exactly Wet (Cyan)
        let wet = blend_river_colors(RIVER_COLOR_DRY, RIVER_COLOR_WET, 1.0);
        assert_eq!(wet, RIVER_COLOR_WET);

        // At mix = 0.5, components should be strictly in between
        let mid = blend_river_colors(RIVER_COLOR_DRY, RIVER_COLOR_WET, 0.5);
        assert!(mid.r() <= RIVER_COLOR_DRY.r() && mid.r() >= RIVER_COLOR_WET.r());
        assert!(mid.b() >= RIVER_COLOR_DRY.b() && mid.b() <= RIVER_COLOR_WET.b());
    }

    #[test]
    fn test_river_overflow_detection() {
        // Safe condition: low peak and balanced gains
        let safe = evaluate_summing_overflow(0.5, 0.5, 0.8);
        assert!(!safe.is_overflow);
        assert_eq!(safe.overflow_factor, 0.0);

        // Dangerous summing: both channels near 0 dBFS (total combined > 1.0)
        let clipped = evaluate_summing_overflow(0.9, 0.8, 1.0);
        assert!(clipped.is_overflow);
        assert!(clipped.overflow_factor > 0.5);
        assert!(clipped.nominal_level > 1.0);
    }

    #[test]
    fn test_river_bezier_generation() {
        let p_start = Pos2::new(10.0, 20.0);
        let p_end = Pos2::new(100.0, 50.0);
        let pts = generate_river_bezier_points(p_start, p_end, 16);

        assert_eq!(pts.len(), 17);
        assert_eq!(pts.first().unwrap().x, 10.0);
        assert_eq!(pts.first().unwrap().y, 20.0);
        assert_eq!(pts.last().unwrap().x, 100.0);
        assert_eq!(pts.last().unwrap().y, 50.0);
    }
}



