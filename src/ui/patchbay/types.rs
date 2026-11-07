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
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 22.0, world_rect.min.y + 34.0), Vec2::new(node_width - 44.0, 32.0));
            let track_min_x = world_rect.min.x + 26.0;
            let track_w = (node_width - 52.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            let val = (50.0 * (400.0_f32).powf(norm)).clamp(50.0, 20000.0);
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

