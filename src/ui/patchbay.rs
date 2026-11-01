use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{CableConnection, ChainMode, ModularNode, NodeType, SignalNodeStatus};
use crate::app::LunulaApp;
use crate::ui::components::{draw_five_finger_hand, render_card_frame};

fn get_rect_perimeter_pos(r: Rect, dist: f32) -> Pos2 {
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

fn render_routing_help_button(
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

fn get_interactive_slider_bounds_and_val(node_type: &NodeType, world_rect: Rect, node_width: f32, pos_x: f32) -> Option<(Rect, f32)> {
    match node_type {
        NodeType::LowPassFilter => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 8.0, world_rect.min.y + 19.0), Vec2::new(node_width - 16.0, 22.0));
            let track_min_x = world_rect.min.x + 14.0;
            let track_w = (node_width - 28.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            let val = (100.0 * (100.0_f32).powf(norm)).clamp(100.0, 10000.0);
            Some((hit, val))
        }
        NodeType::Mixer => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 8.0, world_rect.min.y + 19.0), Vec2::new(node_width - 16.0, 22.0));
            let track_min_x = world_rect.min.x + 24.0;
            let track_w = (node_width - 48.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, norm))
        }
        NodeType::TubeWarmer => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 40.0, world_rect.min.y + 19.0), Vec2::new(node_width - 48.0, 22.0));
            let track_min_x = world_rect.min.x + 42.0;
            let track_w = (node_width - 54.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::StereoWidener => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 34.0, world_rect.min.y + 19.0), Vec2::new(node_width - 42.0, 22.0));
            let track_min_x = world_rect.min.x + 36.0;
            let track_w = (node_width - 50.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::VolumeBooster => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 34.0, world_rect.min.y + 19.0), Vec2::new(node_width - 42.0, 22.0));
            let track_min_x = world_rect.min.x + 36.0;
            let track_w = (node_width - 50.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 24.0).clamp(0.0, 24.0)))
        }
        NodeType::AnalogPreamp => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 34.0, world_rect.min.y + 19.0), Vec2::new(node_width - 50.0, 22.0));
            let track_min_x = world_rect.min.x + 42.0;
            let track_w = (node_width - 66.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (-24.0 + norm * 36.0).clamp(-24.0, 12.0)))
        }
        NodeType::DynamicLoudness => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 8.0, world_rect.min.y + 19.0), Vec2::new(node_width - 16.0, 22.0));
            let track_min_x = world_rect.min.x + 32.0;
            let track_w = (node_width - 44.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::SteppedAttenuator => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 8.0, world_rect.min.y + 19.0), Vec2::new(node_width - 48.0, 22.0));
            let track_min_x = world_rect.min.x + 14.0;
            let track_w = (node_width - 56.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (-60.0 + norm * 60.0).clamp(-60.0, 0.0).round()))
        }
        NodeType::OpticalLeveler => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 32.0, world_rect.min.y + 19.0), Vec2::new(node_width - 40.0, 22.0));
            let track_min_x = world_rect.min.x + 36.0;
            let track_w = (node_width - 48.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (norm * 100.0).clamp(0.0, 100.0)))
        }
        NodeType::StereoBalance => {
            let hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 8.0, world_rect.min.y + 19.0), Vec2::new(node_width - 16.0, 22.0));
            let track_min_x = world_rect.min.x + 20.0;
            let track_w = (node_width - 40.0).max(10.0);
            let norm = ((pos_x - track_min_x) / track_w).clamp(0.0, 1.0);
            Some((hit, (-100.0 + norm * 200.0).clamp(-100.0, 100.0)))
        }
        _ => None,
    }
}

impl LunulaApp {
    // Reset layout of nodes and center canvas view
    pub fn reset_patchbay_layout(&mut self, canvas_width: f32, canvas_height: f32) {
        self.patchbay_pan = Vec2::ZERO;
        let node_width = 144.0;
        let margin = 20.0;
        let available = (canvas_width - margin * 2.0 - 5.0 * node_width).max(12.0 * 4.0);
        let spacing = (available / 4.0).clamp(16.0, 44.0);
        let center_y = (canvas_height * 0.40 - 29.0).max(36.0);

        for (i, node) in self.nodes.iter_mut().take(5).enumerate() {
            node.pos = Pos2::new(margin + i as f32 * (node_width + spacing), center_y);
        }
    }

    // Add a new modular filter node to the canvas
    pub fn add_modular_filter(&mut self, node_type: NodeType) {
        let id = self.next_node_id;
        self.next_node_id += 1;

        let (title, sub1, sub2) = match &node_type {
            NodeType::TubeWarmer => ("TUBE WARMER".to_string(), "Warmth 35%".to_string(), "Triode 12AX7".to_string()),
            NodeType::StereoWidener => ("STEREO WIDENER".to_string(), "Haas Effect 3D".to_string(), "+35% Soundstage".to_string()),
            NodeType::BassGuard => ("BASS GUARD".to_string(), "Sub-sonic Limiter".to_string(), "Protects Woofer".to_string()),
            NodeType::VolumeAttenuator => ("ATTENUATOR".to_string(), "Volume -6 dB".to_string(), "Passive Pad".to_string()),
            NodeType::LowPassFilter => ("LOW-PASS FILTER".to_string(), "Cutoff 800 Hz".to_string(), "Muffled / Underwater".to_string()),
            NodeType::Mixer => ("MIXER / SUMMING".to_string(), "Mix: 50% A • 50% B".to_string(), "Dual-Bus Gerbang".to_string()),
            NodeType::Splitter => ("SPLITTER / Y-BRANCH".to_string(), "Dual-Out (Pin A & B)".to_string(), "Dual-Bus Gerbang".to_string()),
            NodeType::SavedEqPreset(name) => ("🎛️ EQ PRESET".to_string(), name.clone(), "Klik untuk Buka".to_string()),
            NodeType::VolumeBooster => ("VOLUME BOOSTER".to_string(), "+6.0 dB Gain".to_string(), "Pure Clean Amp".to_string()),
            NodeType::AnalogPreamp => ("ANALOG PREAMP".to_string(), "0.0 dB Drive".to_string(), "Class-A Warmth".to_string()),
            NodeType::DynamicLoudness => ("DYNAMIC LOUDNESS".to_string(), "Loudness 50%".to_string(), "ISO 226 Contour".to_string()),
            NodeType::SteppedAttenuator => ("STEPPED ATTENUATOR".to_string(), "-12 dB".to_string(), "64-Step Resistor".to_string()),
            NodeType::OpticalLeveler => ("OPTICAL LEVELER".to_string(), "Peak Red 30%".to_string(), "LA-2A Opto".to_string()),
            NodeType::StereoBalance => ("STEREO BALANCE".to_string(), "Center 0.0 dB".to_string(), "Pan & Trim".to_string()),
            _ => ("AUDIO FILTER".to_string(), "DSP Unit".to_string(), "Active".to_string()),
        };

        // Spawn in visible center of canvas relative to pan
        let spawn_x = 280.0 - self.patchbay_pan.x + (id as f32 * 16.0) % 80.0;
        let spawn_y = 120.0 - self.patchbay_pan.y + (id as f32 * 20.0) % 80.0;
        let param_val = match &node_type {
            NodeType::LowPassFilter => 800.0,
            NodeType::Mixer => 0.5,
            NodeType::StereoWidener => 35.0,
            NodeType::VolumeAttenuator => -6.02,
            NodeType::TubeWarmer => 35.0,
            NodeType::BassGuard => 24.0,
            NodeType::VolumeBooster => 6.0,
            NodeType::AnalogPreamp => 0.0,
            NodeType::DynamicLoudness => 50.0,
            NodeType::SteppedAttenuator => -12.0,
            NodeType::OpticalLeveler => 30.0,
            NodeType::StereoBalance => 0.0,
            _ => 0.0,
        };
        let new_node = ModularNode {
            id,
            node_type,
            title,
            sub1,
            sub2,
            pos: Pos2::new(spawn_x, spawn_y),
            is_removable: true,
            param_val,
        };

        self.nodes.push(new_node);
        self.selected_node = Some(id);
        self.invalidate_chain_cache();
        if self.chain_mode != ChainMode::DspActive {
            self.set_chain_mode(ChainMode::DspActive);
        } else {
            self.sync_signal_chain_to_engine();
        }
    }

    // Interactive Modular Patchbay Canvas with Pan, Drag, and Connect/Disconnect Cables
    pub fn render_modular_patchbay_canvas(&mut self, ui: &mut egui::Ui, canvas_size: Vec2) {
        let (canvas_rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
        let painter = ui.painter().with_clip_rect(canvas_rect);

        // Canvas Background
        painter.rect_filled(canvas_rect, 10.0, Color32::from_rgb(248, 250, 252));
        painter.rect_stroke(canvas_rect, 10.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

        let mouse_pos = response.interact_pointer_pos();

        // Panning with Mouse Wheel / Trackpad Scroll
        let scroll = ui.input(|i| i.smooth_scroll_delta);
        let is_interacting_with_tutorial = if self.show_patchbay_tutorial || self.active_filter_tutorial.is_some() {
            let padding = 12.0;
            let card_w = 400.0f32.min(canvas_rect.width() - 24.0);
            let card_h = 440.0f32.min(canvas_rect.height() - 24.0);
            let r = Rect::from_min_size(
                Pos2::new(canvas_rect.max.x - card_w - padding, canvas_rect.min.y + padding),
                Vec2::new(card_w, card_h),
            );
            mouse_pos.map_or(false, |p| r.contains(p))
        } else {
            false
        };

        if response.hovered() && !is_interacting_with_tutorial && (scroll.x.abs() > 0.0 || scroll.y.abs() > 0.0) {
            self.patchbay_pan += scroll;
        }

        // Dynamic scrolling dot grid
        let grid_size = 28.0;
        let offset_x = (self.patchbay_pan.x % grid_size + grid_size) % grid_size;
        let offset_y = (self.patchbay_pan.y % grid_size + grid_size) % grid_size;
        let mut gx = offset_x - grid_size;
        while gx <= canvas_size.x + grid_size {
            let mut gy = offset_y - grid_size;
            while gy <= canvas_size.y + grid_size {
                painter.circle_filled(
                    Pos2::new(canvas_rect.min.x + gx, canvas_rect.min.y + gy),
                    1.0,
                    Color32::from_rgb(203, 213, 225),
                );
                gy += grid_size;
            }
            gx += grid_size;
        }

        let node_width = 144.0;
        let node_height = 58.0;
        let pin_radius = 5.0;

        let is_mouse_down = ui.input(|i| i.pointer.is_decidedly_dragging());
        let is_left_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        let is_right_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Secondary));
        let is_middle_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle));
        let right_or_middle_pressed = ui.input(|i| {
            i.pointer.button_pressed(egui::PointerButton::Secondary)
                || i.pointer.button_pressed(egui::PointerButton::Middle)
        });

        let mut node_to_delete: Option<usize> = None;
        let mut connection_to_remove: Option<usize> = None;
        let mut connection_to_add: Option<(usize, usize)> = None;
        let mut disconnect_node: Option<usize> = None;
        let mut toggle_dsp = false;
        let mut switch_mode: Option<ChainMode> = None;
        let mut topology_changed = false;
        let mut param_change: Option<(usize, f32)> = None;

        let pointer_pos = ui.input(|i| i.pointer.latest_pos()).or(mouse_pos);
        let is_left_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));

        // Check if cursor is hovering over any node rect
        let is_hovering_any_node = self.nodes.iter().any(|node| {
            let r = Rect::from_min_size(
                Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                    canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                ),
                Vec2::new(node_width, node_height),
            );
            pointer_pos.map_or(false, |p| r.contains(p))
        });

        // 0a. Check hover & active drag on interactive node sliders
        let mut is_hovering_slider = false;
        if !is_interacting_with_tutorial {
            for node in &self.nodes {
                let world_rect = Rect::from_min_size(
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                        canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                    ),
                    Vec2::new(node_width, node_height),
                );
                if let Some((s_hit, _)) = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, node_width, 0.0) {
                    if pointer_pos.map_or(false, |p| s_hit.contains(p)) {
                        is_hovering_slider = true;
                        break;
                    }
                }
                // SteppedAttenuator DIM button hover
                if node.node_type == NodeType::SteppedAttenuator {
                    let dim_hit = Rect::from_min_size(Pos2::new(world_rect.max.x - 38.0, world_rect.min.y + 20.0), Vec2::new(30.0, 16.0));
                    if pointer_pos.map_or(false, |p| dim_hit.contains(p)) {
                        is_hovering_slider = true;
                        break;
                    }
                }
            }
        }
        if is_hovering_slider && !self.is_panning_canvas && self.dragging_node.is_none() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // 0b. Mouse press down directly on slider track or thumb
        let mut clicked_slider = false;
        if is_left_pressed && !is_interacting_with_tutorial {
            if let Some(pos) = pointer_pos {
                for node in &self.nodes {
                    let world_rect = Rect::from_min_size(
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                            canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                        ),
                        Vec2::new(node_width, node_height),
                    );
                    // Check DIM button toggle on Stepped Attenuator
                    if node.node_type == NodeType::SteppedAttenuator {
                        let dim_hit = Rect::from_min_size(Pos2::new(world_rect.max.x - 38.0, world_rect.min.y + 20.0), Vec2::new(30.0, 16.0));
                        if dim_hit.contains(pos) {
                            let new_val = if (node.param_val - (-20.0)).abs() < 0.5 { 0.0 } else { -20.0 };
                            param_change = Some((node.id, new_val));
                            self.selected_node = Some(node.id);
                            clicked_slider = true;
                            break;
                        }
                    }
                    if let Some((s_hit, val)) = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, node_width, pos.x) {
                        if s_hit.contains(pos) {
                            self.dragging_slider_node = Some(node.id);
                            self.dragging_node = None;
                            self.is_panning_canvas = false;
                            self.connecting_from_node = None;
                            self.selected_node = Some(node.id);
                            param_change = Some((node.id, val));
                            clicked_slider = true;
                            break;
                        }
                    }
                }
            }
        }

        // 0c. Continuous slider dragging updates
        if is_left_down {
            if let Some(drag_node_id) = self.dragging_slider_node {
                if let Some(pos) = pointer_pos {
                    if let Some(node) = self.nodes.iter().find(|n| n.id == drag_node_id) {
                        let world_rect = Rect::from_min_size(
                            Pos2::new(
                                canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                            ),
                            Vec2::new(node_width, node_height),
                        );
                        if let Some((_, val)) = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, node_width, pos.x) {
                            param_change = Some((drag_node_id, val));
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }
        } else {
            self.dragging_slider_node = None;
        }

        // 0d. Right-Click or Middle-Click Hold to Pan Canvas (when on empty background canvas)
        if right_or_middle_pressed && response.hovered() && !is_hovering_any_node && !is_interacting_with_tutorial {
            self.is_panning_canvas = true;
            self.connecting_from_node = None;
            self.dragging_node = None;
        }

        if (is_right_down || is_middle_down) && self.is_panning_canvas {
            let delta = ui.input(|i| i.pointer.delta());
            self.patchbay_pan += delta;
        }

        if !is_left_down && !is_right_down && !is_middle_down {
            self.is_panning_canvas = false;
        }

        // 1. Detect clicking on Pins to create cables OR clicking nodes to drag OR empty space to pan (Left Click)
        if self.dragging_slider_node.is_some() || clicked_slider {
            // Slider active, ignore canvas drag
        } else if response.drag_started() && is_left_down && !is_right_down && !is_middle_down && !is_interacting_with_tutorial {
            if let Some(pos) = mouse_pos {
                // Check if user clicked an Output Pin (Right dot)
                for node in &self.nodes {
                    let world_rect = Rect::from_min_size(
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                            canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                        ),
                        Vec2::new(node_width, node_height),
                    );
                    let hit_out_pin = if node.node_type == NodeType::Splitter {
                        let pin_a = Pos2::new(world_rect.max.x, world_rect.center().y - 12.0);
                        let pin_b = Pos2::new(world_rect.max.x, world_rect.center().y + 12.0);
                        pos.distance(pin_a) <= 12.0 || pos.distance(pin_b) <= 12.0
                    } else {
                        let out_pin_pos = Pos2::new(world_rect.max.x, world_rect.center().y);
                        pos.distance(out_pin_pos) <= 12.0
                    };
                    if hit_out_pin {
                        self.connecting_from_node = Some(node.id);
                        break;
                    }
                }

                // If not starting cable, check if clicked on a node card
                if self.connecting_from_node.is_none() {
                    for (i, node) in self.nodes.iter().enumerate().rev() {
                        let world_rect = Rect::from_min_size(
                            Pos2::new(
                                canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                            ),
                            Vec2::new(node_width, node_height),
                        );
                        if world_rect.contains(pos) {
                            if node.node_type == NodeType::LowPassFilter || node.node_type == NodeType::Mixer {
                                let s_rect = Rect::from_min_size(
                                    Pos2::new(world_rect.min.x + 8.0, world_rect.min.y + 19.0),
                                    Vec2::new(world_rect.width() - 16.0, 20.0),
                                );
                                if s_rect.contains(pos) {
                                    self.selected_node = Some(node.id);
                                    break;
                                }
                            }
                            self.dragging_node = Some(i);
                            self.selected_node = Some(node.id);
                            break;
                        }
                    }

                    // If clicked on empty canvas background, also allow panning canvas
                    if self.dragging_node.is_none() && self.dragging_slider_node.is_none() {
                        self.is_panning_canvas = true;
                    }
                }
            }
        }

        // 2. Handle Node Dragging OR Canvas Background Panning (Left Click)
        if is_mouse_down && is_left_down && self.connecting_from_node.is_none() && self.dragging_slider_node.is_none() {
            let delta = ui.input(|i| i.pointer.delta());
            if let Some(drag_idx) = self.dragging_node {
                let node = &mut self.nodes[drag_idx];
                node.pos += delta;
            } else if self.is_panning_canvas {
                self.patchbay_pan += delta;
            }
        }

        // 3. Handle Cable Connection Release on an Input Pin (Supports DAG Branching & Prevents Cycles)
        if response.drag_stopped() {
            if let (Some(from_id), Some(pos)) = (self.connecting_from_node, mouse_pos) {
                for node in &self.nodes {
                    // Valid connection target:
                    // 1. Not connecting to itself (node.id != from_id)
                    // 2. Source node cannot receive input (Source has no input pin)
                    // 3. DAG cycle prevention: node cannot already reach from_id (would create feedback loop)
                    if node.id != from_id && node.node_type != NodeType::Source {
                        let world_rect = Rect::from_min_size(
                            Pos2::new(
                                canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                            ),
                            Vec2::new(node_width, node_height),
                        );
                        let is_mixer = node.node_type == NodeType::Mixer;
                        let hit_in_pin = if is_mixer {
                            let pin_a = Pos2::new(world_rect.min.x, world_rect.center().y - 12.0);
                            let pin_b = Pos2::new(world_rect.min.x, world_rect.center().y + 12.0);
                            pos.distance(pin_a) <= 18.0 || pos.distance(pin_b) <= 18.0 || pos.distance(Pos2::new(world_rect.min.x, world_rect.center().y)) <= 18.0
                        } else {
                            let in_pin_pos = Pos2::new(world_rect.min.x, world_rect.center().y);
                            pos.distance(in_pin_pos) <= 18.0
                        };

                        if hit_in_pin {
                            let already_connected = self.connections.iter().any(|c| c.from_node == from_id && c.to_node == node.id);
                            let would_create_cycle = self.can_reach_node(node.id, from_id);

                            if !already_connected && !would_create_cycle {
                                if !is_mixer {
                                    // 1-Input-1-Cable Rule: Normal nodes automatically replace old cable with new cable
                                    self.connections.retain(|c| c.to_node != node.id);
                                } else {
                                    // Mixer Gerbang: Allows max 2 inputs (Input A & Input B). If 2 already present, replace the second
                                    let incoming_count = self.connections.iter().filter(|c| c.to_node == node.id).count();
                                    if incoming_count >= 2 {
                                        if let Some(pos_last) = self.connections.iter().rposition(|c| c.to_node == node.id) {
                                            self.connections.remove(pos_last);
                                        }
                                    }
                                }

                                // Outgoing Cable Rule:
                                // Normal nodes have exactly 1 output pin -> new outgoing cable automatically replaces old outgoing cable.
                                // Splitter nodes have 2 output pins (Pin A & Pin B) allowing up to 2 simultaneous outgoing cables.
                                let is_from_splitter = self.nodes.iter().find(|n| n.id == from_id).map_or(false, |n| n.node_type == NodeType::Splitter);
                                if !is_from_splitter {
                                    self.connections.retain(|c| c.from_node != from_id);
                                } else {
                                    let outgoing_count = self.connections.iter().filter(|c| c.from_node == from_id).count();
                                    if outgoing_count >= 2 {
                                        if let Some(pos_last) = self.connections.iter().rposition(|c| c.from_node == from_id) {
                                            self.connections.remove(pos_last);
                                        }
                                    }
                                }

                                self.connections.push(CableConnection {
                                    from_node: from_id,
                                    to_node: node.id,
                                });
                                topology_changed = true;
                            }
                            break;
                        }
                    }
                }
            }
            self.connecting_from_node = None;
            self.dragging_node = None;
            self.is_panning_canvas = false;
        }

        // Cursor State Feedback:
        // Default arrow normally, custom 5-finger open hand cursor ONLY when holding right-click to pan!
        let is_holding_pan = (is_right_down || is_middle_down) && self.is_panning_canvas;
        if is_holding_pan {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            if let Some(pos) = mouse_pos {
                let top_painter = ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Tooltip,
                    ui.scope_id().with("five_finger_hand_cursor"),
                ));
                draw_five_finger_hand(&top_painter, pos);
            }
        } else if self.connecting_from_node.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        } else if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        }

        // Evaluate Signal Chain reachability
        let chain_eval = self.evaluate_signal_chain();

        let is_fallback = self.signal_path_report.as_ref().map_or(false, |r| {
            r.requested_mode == ChainMode::BitPerfect && r.actual_mode == ChainMode::OsResampled
        });

        let theme_color = if is_fallback {
            Color32::from_rgb(245, 158, 11)
        } else {
            match self.chain_mode {
                ChainMode::BitPerfect => Color32::from_rgb(16, 185, 129),
                ChainMode::DspActive => Color32::from_rgb(99, 102, 241),
                ChainMode::OsResampled => Color32::from_rgb(245, 158, 11),
            }
        };

        // 4. Render Established Patchbay Cables (with click to disconnect)
        for (conn_idx, conn) in self.connections.iter().enumerate() {
            let from_node = self.nodes.iter().find(|n| n.id == conn.from_node);
            let to_node = self.nodes.iter().find(|n| n.id == conn.to_node);

            if let (Some(from_n), Some(to_n)) = (from_node, to_node) {
                let p_start = if from_n.node_type == NodeType::Splitter {
                    let outgoing_conns: Vec<usize> = self.connections.iter().filter(|c| c.from_node == from_n.id).map(|c| c.to_node).collect();
                    let is_second = outgoing_conns.get(1) == Some(&conn.to_node);
                    let pin_offset_y = if is_second { 12.0 } else { -12.0 };
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + node_width,
                        canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + node_height * 0.5 + pin_offset_y,
                    )
                } else {
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + node_width,
                        canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + node_height * 0.5,
                    )
                };
                let p_end = if to_n.node_type == NodeType::Mixer {
                    let incoming_conns: Vec<usize> = self.connections.iter().filter(|c| c.to_node == to_n.id).map(|c| c.from_node).collect();
                    let is_second = incoming_conns.get(1) == Some(&conn.from_node);
                    let pin_offset_y = if is_second { 12.0 } else { -12.0 };
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + to_n.pos.x,
                        canvas_rect.min.y + self.patchbay_pan.y + to_n.pos.y + node_height * 0.5 + pin_offset_y,
                    )
                } else {
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + to_n.pos.x,
                        canvas_rect.min.y + self.patchbay_pan.y + to_n.pos.y + node_height * 0.5,
                    )
                };

                let is_bypass_jumper = conn.from_node == 2 && conn.to_node == 4;
                let (cp1, cp2) = if is_bypass_jumper {
                    // Option B: Arch upward over DSP Engine node cleanly
                    let node3_top = self.nodes.iter().find(|n| n.id == 3).map(|n| n.pos.y).unwrap_or(from_n.pos.y);
                    let top_y = p_start.y.min(p_end.y).min(canvas_rect.min.y + self.patchbay_pan.y + node3_top);
                    let arch_cp_y = top_y - 20.0;
                    let dx = (p_end.x - p_start.x).abs() * 0.5;
                    (
                        Pos2::new(p_start.x + dx * 0.45, arch_cp_y),
                        Pos2::new(p_end.x - dx * 0.45, arch_cp_y),
                    )
                } else {
                    let dx = (p_end.x - p_start.x).abs().max(40.0) * 0.5;
                    (
                        Pos2::new(p_start.x + dx, p_start.y),
                        Pos2::new(p_end.x - dx, p_end.y),
                    )
                };

                let steps = 24;
                let mut points = Vec::with_capacity(steps + 1);
                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                    let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                    points.push(Pos2::new(x, y));
                }

                let is_flowing_cable = chain_eval.flowing_cables.contains(&(conn.from_node, conn.to_node));

                // Check if user clicked or right-clicked on cable to disconnect
                let mid = points[12];
                let is_hover_cable = mouse_pos.map_or(false, |p| p.distance(mid) <= 14.0 || points.iter().any(|pt| pt.distance(p) <= 6.0));

                if is_hover_cable && (response.clicked() || response.secondary_clicked()) {
                    connection_to_remove = Some(conn_idx);
                }

                let is_effect_cable = (from_n.node_type.is_dsp_or_effect() || to_n.node_type.is_dsp_or_effect())
                    && (self.chain_mode == ChainMode::BitPerfect || self.chain_mode == ChainMode::OsResampled);

                let cable_color = if is_hover_cable {
                    Color32::from_rgb(239, 68, 68) // Red on hover to cut
                } else if is_effect_cable {
                    Color32::from_rgb(203, 213, 225) // Muted slate gray when connected to bypassed effect
                } else if is_flowing_cable {
                    theme_color
                } else {
                    Color32::from_rgb(239, 68, 68) // Red when severed/dead end
                };

                // Drop shadow (single batched polyline)
                let shadow_pts: Vec<Pos2> = points.iter().map(|p| *p + Vec2::new(0.0, 1.5)).collect();
                painter.add(egui::Shape::line(shadow_pts, Stroke::new(3.0, Color32::from_black_alpha(18))));

                // Cable path: draw dashed line if connected to a bypassed effect, or single batched polyline
                let stroke_w = if is_hover_cable { 2.8 } else if is_flowing_cable { 2.3 } else { 1.8 };
                if is_effect_cable {
                    for (i, seg) in points.windows(2).enumerate() {
                        if i % 2 == 0 {
                            painter.line_segment([seg[0], seg[1]], Stroke::new(stroke_w, cable_color));
                        }
                    }
                } else {
                    painter.add(egui::Shape::line(points.clone(), Stroke::new(stroke_w, cable_color)));
                }

                // If flowing and song is playing: draw live animated audio data particles traveling along the cable!
                // Only on active flowing paths, NOT through bypassed effects in Bit-Perfect or OsResampled!
                if is_flowing_cable && !is_effect_cable && self.is_playing {
                    let particle_glow = if is_fallback {
                        Color32::from_rgba_premultiplied(253, 230, 138, 200)
                    } else {
                        match self.chain_mode {
                            ChainMode::BitPerfect => Color32::from_rgba_premultiplied(167, 243, 208, 200),
                            ChainMode::DspActive => Color32::from_rgba_premultiplied(199, 210, 254, 200),
                            ChainMode::OsResampled => Color32::from_rgba_premultiplied(253, 230, 138, 200),
                        }
                    };
                    for k in 0..3 {
                        let t_part = ((self.visualizer_anim_phase * 0.75 + k as f32 * 0.33) % 1.0 + 1.0) % 1.0;
                        let p_idx = (t_part * 24.0).min(23.0) as usize;
                        let pt = points[p_idx];
                        painter.circle_filled(pt, 4.0, particle_glow);
                        painter.circle_filled(pt, 2.0, Color32::WHITE);
                    }
                }

                // Option B: DIRECT BYPASS badge on arching jumper wire
                if is_bypass_jumper && is_flowing_cable {
                    let (badge_text, badge_bg) = if is_fallback {
                        ("⚠️ FALLBACK 48kHz", Color32::from_rgba_premultiplied(217, 119, 6, 240))
                    } else if self.chain_mode == ChainMode::OsResampled {
                        ("⚡ 48kHz OS MIXER", Color32::from_rgba_premultiplied(245, 158, 11, 235))
                    } else {
                        ("⚡ DIRECT BYPASS", Color32::from_rgba_premultiplied(16, 185, 129, 235))
                    };
                    let badge_rect = Rect::from_center_size(mid - Vec2::new(0.0, 9.0), Vec2::new(104.0, 15.0));
                    painter.rect_filled(badge_rect, 4.0, badge_bg);
                    painter.rect_stroke(badge_rect, 4.0, Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Outside);
                    painter.text(badge_rect.center(), egui::Align2::CENTER_CENTER, badge_text, egui::FontId::proportional(8.5), Color32::WHITE);
                }

                // Cut hint bubble on hover
                if is_hover_cable {
                    painter.circle_filled(mid, 8.0, Color32::from_rgb(239, 68, 68));
                    painter.text(mid, egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(9.0), Color32::WHITE);
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                } else if !is_flowing_cable {
                    // Warning icon at midpoint of dead cable
                    painter.circle_filled(mid, 7.0, Color32::from_rgb(239, 68, 68));
                    painter.text(mid, egui::Align2::CENTER_CENTER, "!", egui::FontId::proportional(9.0), Color32::WHITE);
                }
            }
        }

        // 4b. Render Broken Gaps between Standard Default Nodes with Click-to-Connect
        // Only rendered if the signal chain is incomplete AND no alternative path exists!
        // If from_id can already reach to_id (e.g. through an EQ preset or modular filter),
        // the connection is fully satisfied and NO red dashed line is drawn!
        if !chain_eval.is_complete {
            let standard_pairs: Vec<(usize, usize)> = vec![(1, 2), (2, 4), (4, 5)];
            for (from_id, to_id) in standard_pairs {
                // If from_id can already reach to_id through modular filters, it is NOT broken!
                if self.can_reach_node(from_id, to_id) {
                    continue;
                }

                let has_conn = self.connections.iter().any(|c| c.from_node == from_id && c.to_node == to_id);
                if !has_conn {
                    let from_node = self.nodes.iter().find(|n| n.id == from_id);
                    let to_node = self.nodes.iter().find(|n| n.id == to_id);
                    if let (Some(from_n), Some(to_n)) = (from_node, to_node) {
                        let p_start = Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + node_width,
                            canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + node_height * 0.5,
                        );
                        let p_end = Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + to_n.pos.x,
                            canvas_rect.min.y + self.patchbay_pan.y + to_n.pos.y + node_height * 0.5,
                        );

                        let is_jumper_gap = from_id == 2 && to_id == 4;
                        let (cp1, cp2) = if is_jumper_gap {
                            let node3_top = self.nodes.iter().find(|n| n.id == 3).map(|n| n.pos.y).unwrap_or(from_n.pos.y);
                            let top_y = p_start.y.min(p_end.y).min(canvas_rect.min.y + self.patchbay_pan.y + node3_top);
                            let arch_cp_y = top_y - 20.0;
                            let dx = (p_end.x - p_start.x).abs() * 0.5;
                            (Pos2::new(p_start.x + dx * 0.45, arch_cp_y), Pos2::new(p_end.x - dx * 0.45, arch_cp_y))
                        } else {
                            let dx = (p_end.x - p_start.x).abs() * 0.5;
                            (Pos2::new(p_start.x + dx, p_start.y), Pos2::new(p_end.x - dx, p_end.y))
                        };

                        let steps = 18;
                        let mut points = Vec::with_capacity(steps + 1);
                        for step in 0..=steps {
                            let t = step as f32 / steps as f32;
                            let u = 1.0 - t;
                            let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                            let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                            points.push(Pos2::new(x, y));
                        }

                        // Red dashed line
                        for (idx, seg) in points.windows(2).enumerate() {
                            if idx % 2 == 0 {
                                painter.line_segment([seg[0], seg[1]], Stroke::new(1.8, Color32::from_rgb(239, 68, 68)));
                            }
                        }

                        let mid_p = points[steps / 2];
                        let is_hover_gap = mouse_pos.map_or(false, |p| p.distance(mid_p) <= 13.0);
                        if is_hover_gap {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            if response.clicked() {
                                connection_to_add = Some((from_id, to_id));
                            }
                        }

                        let btn_rad = if is_hover_gap { 10.0 } else { 8.5 };
                        painter.circle_filled(mid_p, btn_rad, Color32::from_rgb(239, 68, 68));
                        painter.circle_stroke(mid_p, btn_rad, Stroke::new(1.5, Color32::WHITE));
                        painter.text(mid_p, egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(9.5), Color32::WHITE);
                    }
                }
            }
        }

        // Remove disconnected cable
        if let Some(idx) = connection_to_remove {
            self.connections.remove(idx);
            topology_changed = true;
        }

        // 5. Draw Live Rubber-Band Cable while Dragging
        if let (Some(from_id), Some(pos)) = (self.connecting_from_node, mouse_pos) {
            if let Some(from_n) = self.nodes.iter().find(|n| n.id == from_id) {
                let p_start = if from_n.node_type == NodeType::Splitter {
                    let outgoing_count = self.connections.iter().filter(|c| c.from_node == from_n.id).count();
                    let pin_offset_y = if outgoing_count == 0 { -12.0 } else { 12.0 };
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + node_width,
                        canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + node_height * 0.5 + pin_offset_y,
                    )
                } else {
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + node_width,
                        canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + node_height * 0.5,
                    )
                };
                let p_end = pos;

                let dx = (p_end.x - p_start.x).abs().max(40.0) * 0.5;
                let cp1 = Pos2::new(p_start.x + dx, p_start.y);
                let cp2 = Pos2::new(p_end.x - dx, p_end.y);

                let steps = 20;
                let mut points = Vec::with_capacity(steps + 1);
                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                    let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                    points.push(Pos2::new(x, y));
                }

                painter.add(egui::Shape::line(points, Stroke::new(2.5, Color32::from_rgb(14, 165, 233))));

                painter.circle_filled(p_end, 5.0, Color32::from_rgb(14, 165, 233));
            }
        }

        // 6. Render Nodes & Input/Output Patch Pins with Dynamic Outline Animation
        let mut navigate_to_eq_preset: Option<String> = None;
        let mut navigate_to_main_eq = false;

        for (i, node) in self.nodes.iter().enumerate() {
            let is_active = self.selected_node == Some(node.id);
            let is_dragging = self.dragging_node == Some(i);

            let node_rect = Rect::from_min_size(
                Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                    canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                ),
                Vec2::new(node_width, node_height),
            );

            let is_hovered = mouse_pos.map_or(false, |p| node_rect.contains(p));
            let hover_anim = ui.ctx().animate_bool(ui.scope_id().with(("node_hover", node.id)), is_hovered);

            // Is this node part of the active flowing audio signal?
            let is_flowing = chain_eval.node_status(node.id) == SignalNodeStatus::Flowing;

            if is_active {
                painter.rect_stroke(node_rect.expand(1.2), 9.0, Stroke::new(1.2, Color32::from_rgb(129, 140, 248)), egui::StrokeKind::Outside);
            }

            // Context menu on node
            let node_resp = ui.interact(node_rect, ui.scope_id().with(("patch_node_ctx", node.id)), egui::Sense::click());
            if node_resp.clicked() {
                match &node.node_type {
                    NodeType::SavedEqPreset(name) => {
                        navigate_to_eq_preset = Some(name.clone());
                    }
                    NodeType::Dsp => {
                        navigate_to_main_eq = true;
                    }
                    _ => {}
                }
            }

            node_resp.context_menu(|ui| {
                ui.label(egui::RichText::new(&node.title).strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                ui.label(egui::RichText::new(&node.sub1).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                ui.separator();
                if let NodeType::SavedEqPreset(ref name) = node.node_type {
                    if ui.button("🎛️ Buka & Edit di Tab Equalizer").clicked() {
                        navigate_to_eq_preset = Some(name.clone());
                        ui.close();
                    }
                    ui.separator();
                } else if node.node_type == NodeType::Dsp {
                    if ui.button("🎛️ Buka DSP di Tab Equalizer").clicked() {
                        navigate_to_main_eq = true;
                        ui.close();
                    }
                    ui.separator();
                }
                if node.is_removable {
                    if ui.button("🗑️ Hapus Filter / Modul Ini").clicked() {
                        node_to_delete = Some(node.id);
                        ui.close();
                    }
                }
                if ui.button("✂️ Putuskan Semua Kabel Terhubung").clicked() {
                    disconnect_node = Some(node.id);
                    ui.close();
                }
                if node.node_type.is_dsp_or_effect() {
                    let label = if self.chain_mode == ChainMode::DspActive {
                        "⚡ Bypass Efek & DSP (Direct Bit-Perfect)"
                    } else {
                        "🎛️ Aktifkan DSP Processing (Efek Aktif)"
                    };
                    if ui.button(label).clicked() {
                        toggle_dsp = true;
                        ui.close();
                    }
                }
            });

            // Shadow
            painter.rect_filled(node_rect.translate(Vec2::new(0.0, 2.0 + hover_anim * 1.5)), 8.0, Color32::from_black_alpha(14));

            let theme_color = if is_fallback {
                Color32::from_rgb(245, 158, 11)
            } else {
                match self.chain_mode {
                    ChainMode::BitPerfect => Color32::from_rgb(16, 185, 129),
                    ChainMode::DspActive => Color32::from_rgb(99, 102, 241),
                    ChainMode::OsResampled => Color32::from_rgb(245, 158, 11),
                }
            };

            let is_bypassed_effect = node.node_type.is_dsp_or_effect()
                && (self.chain_mode == ChainMode::BitPerfect || self.chain_mode == ChainMode::OsResampled);

            let bg = if is_dragging {
                Color32::from_rgb(241, 245, 249)
            } else if is_bypassed_effect {
                Color32::from_rgba_premultiplied(248, 250, 252, 220)
            } else if !is_flowing {
                Color32::from_rgb(255, 245, 245)
            } else {
                Color32::WHITE
            };

            painter.rect_filled(node_rect, 8.0, bg);

            // Dynamic Outline Styling & Animation
            if is_bypassed_effect {
                // Option B: Effect is BYPASSED / DILEWATI — calm muted slate border & direct dashed wire inside
                painter.rect_stroke(node_rect, 8.0, Stroke::new(1.2, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                
                // Dashed direct jumper wire through the center of effect
                let pin_l = Pos2::new(node_rect.min.x + 8.0, node_rect.center().y);
                let pin_r = Pos2::new(node_rect.max.x - 8.0, node_rect.center().y);
                for d in 0..10 {
                    if d % 2 == 0 {
                        let p1 = pin_l.lerp(pin_r, d as f32 / 10.0);
                        let p2 = pin_l.lerp(pin_r, (d + 1) as f32 / 10.0);
                        painter.line_segment([p1, p2], Stroke::new(1.2, Color32::from_rgb(203, 213, 225)));
                    }
                }

                // DILEWATI badge on the top of the node
                let badge_rect = Rect::from_min_size(node_rect.min + Vec2::new(6.0, 4.0), Vec2::new(44.0, 12.0));
                painter.rect_filled(badge_rect, 3.0, Color32::from_rgb(226, 232, 240));
                painter.text(badge_rect.center(), egui::Align2::CENTER_CENTER, "DILEWATI", egui::FontId::proportional(7.5), Color32::from_rgb(100, 116, 139));

                if node_resp.clicked() {
                    switch_mode = Some(ChainMode::DspActive);
                }
            } else if is_flowing {
                if self.is_playing {
                    // Pulsing luminous aura matching theme
                    let aura_alpha = (60.0 + 35.0 * (self.visualizer_anim_phase * 4.0).sin()) as u8;
                    painter.rect_stroke(
                        node_rect.expand(2.0),
                        10.0,
                        Stroke::new(1.8, Color32::from_rgba_premultiplied(theme_color.r(), theme_color.g(), theme_color.b(), aura_alpha / 2)),
                        egui::StrokeKind::Outside,
                    );

                    // Animated laser beacons moving along the rectangle's perimeter
                    let p_len = 2.0 * (node_width + node_height);
                    let travel_dist = (self.visualizer_anim_phase * 140.0 + (node.id as f32) * 55.0) % p_len;
                    let b1 = get_rect_perimeter_pos(node_rect, travel_dist);
                    let b2 = get_rect_perimeter_pos(node_rect, travel_dist + p_len * 0.5);

                    let (b1_col, b2_col) = match self.chain_mode {
                        ChainMode::BitPerfect => (Color32::from_rgb(52, 211, 153), Color32::from_rgb(34, 211, 238)),
                        ChainMode::DspActive => (Color32::from_rgb(129, 140, 248), Color32::from_rgb(192, 132, 252)),
                        ChainMode::OsResampled => (Color32::from_rgb(251, 191, 36), Color32::from_rgb(245, 158, 11)),
                    };

                    painter.circle_filled(b1, 3.8, b1_col);
                    painter.circle_filled(b1, 1.8, Color32::WHITE);
                    painter.circle_filled(b2, 3.8, b2_col);
                    painter.circle_filled(b2, 1.8, Color32::WHITE);

                    // Flowing border
                    let stroke_w = 1.6 + 0.4 * (self.visualizer_anim_phase * 3.5).sin();
                    painter.rect_stroke(node_rect, 8.0, Stroke::new(stroke_w, theme_color), egui::StrokeKind::Outside);
                } else {
                    // Ready / Paused: Crisp theme border
                    painter.rect_stroke(node_rect, 8.0, Stroke::new(1.3, theme_color), egui::StrokeKind::Outside);
                }
            } else {
                // BROKEN / NO SIGNAL: Pulsing Crimson Alert
                let warn_t = (ui.input(|i| i.time) * 4.0).sin().abs() as f32;
                let stroke_w = 1.5 + warn_t * 0.8;
                painter.rect_stroke(node_rect, 8.0, Stroke::new(stroke_w, Color32::from_rgb(239, 68, 68)), egui::StrokeKind::Outside);
                painter.rect_stroke(node_rect.expand(2.0), 10.0, Stroke::new(1.5, Color32::from_rgba_premultiplied(239, 68, 68, 35)), egui::StrokeKind::Outside);
            }

            // 1. Dynamic Liquid Wave Animation for Low-Pass Filter (Drawn in background BEFORE text/slider)
            if node.node_type == NodeType::LowPassFilter {
                let cutoff = if node.param_val > 50.0 { node.param_val } else { 800.0 };
                let active_cutoff = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(cutoff);

                if self.is_playing && is_flowing && !is_bypassed_effect {
                    let wave_painter = painter.with_clip_rect(node_rect);
                    let time = ui.input(|i| i.time);

                    // Water depth: keep water level strictly in the lower area so title is NEVER submerged!
                    let norm_cutoff = ((active_cutoff / 100.0).log10() / 2.0).clamp(0.0, 1.0);
                    let water_depth = 0.50 - norm_cutoff * 0.25; // depth 0.25 (10kHz) to 0.50 (100Hz)
                    let base_y = node_rect.max.y - (node_rect.height() * water_depth);

                    let w_speed = 3.2;
                    let w_amp = 1.8;

                    // Back wave layer (translucent cyan)
                    let mut back_poly = Vec::new();
                    back_poly.push(Pos2::new(node_rect.min.x, node_rect.max.y));
                    let steps = 14;
                    for s in 0..=steps {
                        let fx = node_rect.min.x + (node_rect.width() * (s as f32 / steps as f32));
                        let wave_offset = ((fx * 0.08 + (time * w_speed) as f32).sin()) * w_amp;
                        back_poly.push(Pos2::new(fx, base_y + wave_offset));
                    }
                    back_poly.push(Pos2::new(node_rect.max.x, node_rect.max.y));
                    wave_painter.add(egui::Shape::convex_polygon(
                        back_poly,
                        Color32::from_rgba_premultiplied(14, 165, 233, 40),
                        Stroke::NONE,
                    ));

                    // Front wave layer (vibrant teal/cyan)
                    let mut front_poly = Vec::new();
                    front_poly.push(Pos2::new(node_rect.min.x, node_rect.max.y));
                    for s in 0..=steps {
                        let fx = node_rect.min.x + (node_rect.width() * (s as f32 / steps as f32));
                        let wave_offset = (((fx * 0.09 - (time * (w_speed * 1.25)) as f32) + 1.8).sin()) * (w_amp * 0.9);
                        front_poly.push(Pos2::new(fx, base_y + wave_offset + 0.8));
                    }
                    front_poly.push(Pos2::new(node_rect.max.x, node_rect.max.y));
                    wave_painter.add(egui::Shape::convex_polygon(
                        front_poly,
                        Color32::from_rgba_premultiplied(6, 182, 212, 70),
                        Stroke::NONE,
                    ));

                    // Rising bubbles when audio is flowing & playing
                    let b1_y = node_rect.max.y - (((time * 16.0) as f32) % (node_rect.height() * 0.45));
                    let b2_y = node_rect.max.y - ((((time * 22.0) + 7.0) as f32) % (node_rect.height() * 0.45));
                    wave_painter.circle_stroke(
                        Pos2::new(node_rect.min.x + 20.0, b1_y),
                        1.5,
                        Stroke::new(0.8, Color32::from_rgba_premultiplied(255, 255, 255, 180)),
                    );
                    wave_painter.circle_stroke(
                        Pos2::new(node_rect.max.x - 24.0, b2_y),
                        2.0,
                        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 160)),
                    );
                    ui.ctx().request_repaint();
                }
            }

            // 2. Title Text (Foreground: high contrast, rendered ON TOP of wave background)
            let title_col = if is_bypassed_effect {
                Color32::from_rgb(100, 116, 139)
            } else {
                Color32::from_rgb(15, 23, 42)
            };
            painter.text(
                Pos2::new(node_rect.center().x, node_rect.min.y + 11.5),
                egui::Align2::CENTER_CENTER,
                &node.title,
                egui::FontId::proportional(10.5),
                title_col,
            );

            if node.node_type == NodeType::LowPassFilter {
                let cutoff = if node.param_val > 50.0 { node.param_val } else { 800.0 };
                let active_cutoff = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(cutoff);

                // Interactive Mini Cutoff Slider
                if !is_bypassed_effect {
                    let slider_track_rect = Rect::from_min_size(
                        Pos2::new(node_rect.min.x + 14.0, node_rect.min.y + 24.0),
                        Vec2::new(node_width - 28.0, 5.0),
                    );

                    let fill_ratio = ((active_cutoff / 100.0).log10() / 2.0).clamp(0.0, 1.0);
                    let thumb_x = slider_track_rect.min.x + slider_track_rect.width() * fill_ratio;
                    let cy = slider_track_rect.center().y;

                    painter.rect_filled(
                        Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(slider_track_rect.width(), 4.0)),
                        2.0,
                        Color32::from_rgb(224, 242, 254),
                    );
                    painter.rect_filled(
                        Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(thumb_x - slider_track_rect.min.x, 4.0)),
                        2.0,
                        Color32::from_rgb(14, 165, 233),
                    );
                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.0, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.0, Stroke::new(1.8, Color32::from_rgb(2, 132, 199)));

                    let readout = if active_cutoff >= 1000.0 {
                        format!("Cutoff {:.1} kHz", active_cutoff / 1000.0)
                    } else {
                        format!("Cutoff {:.0} Hz", active_cutoff)
                    };
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 40.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.5),
                        Color32::from_rgb(2, 132, 199),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::Mixer {
                let mix = (node.param_val).clamp(0.0, 1.0);
                let active_mix = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(mix);

                if !is_bypassed_effect {
                    let slider_track_rect = Rect::from_min_size(
                        Pos2::new(node_rect.min.x + 24.0, node_rect.min.y + 24.0),
                        Vec2::new(node_width - 48.0, 5.0),
                    );
                    let thumb_x = slider_track_rect.min.x + slider_track_rect.width() * active_mix;
                    let cy = slider_track_rect.center().y;

                    // Track background
                    painter.rect_filled(
                        Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(slider_track_rect.width(), 4.0)),
                        2.0,
                        Color32::from_rgb(243, 232, 255),
                    );
                    // Center notch at 50%
                    let center_x = slider_track_rect.center().x;
                    painter.line_segment([Pos2::new(center_x, cy - 4.0), Pos2::new(center_x, cy + 4.0)], Stroke::new(1.2, Color32::from_rgb(192, 132, 252)));

                    // Active fill from center
                    let (fill_start, fill_w) = if thumb_x >= center_x {
                        (center_x, thumb_x - center_x)
                    } else {
                        (thumb_x, center_x - thumb_x)
                    };
                    painter.rect_filled(
                        Rect::from_min_size(Pos2::new(fill_start, cy - 2.0), Vec2::new(fill_w, 4.0)),
                        2.0,
                        Color32::from_rgb(147, 51, 234),
                    );

                    // Thumb knob
                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.0, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.0, Stroke::new(1.8, Color32::from_rgb(126, 34, 206)));

                    // A / B labels beside track
                    painter.text(Pos2::new(slider_track_rect.min.x - 4.0, cy), egui::Align2::RIGHT_CENTER, "A", egui::FontId::proportional(8.5), Color32::from_rgb(126, 34, 206));
                    painter.text(Pos2::new(slider_track_rect.max.x + 4.0, cy), egui::Align2::LEFT_CENTER, "B", egui::FontId::proportional(8.5), Color32::from_rgb(126, 34, 206));

                    // Readout
                    let a_pct = (1.0 - active_mix) * 100.0;
                    let b_pct = active_mix * 100.0;
                    let readout = format!("A: {:.0}% • B: {:.0}%", a_pct, b_pct);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 40.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(126, 34, 206),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::Splitter {
                if !is_bypassed_effect {
                    let cy = node_rect.center().y;
                    let p_in = Pos2::new(node_rect.min.x + 20.0, cy);
                    let p_fork = Pos2::new(node_rect.center().x - 4.0, cy);
                    let p_out_a = Pos2::new(node_rect.max.x - 20.0, cy - 12.0);
                    let p_out_b = Pos2::new(node_rect.max.x - 20.0, cy + 12.0);

                    let branch_stroke = Stroke::new(1.8, Color32::from_rgb(245, 158, 11));
                    painter.line_segment([p_in, p_fork], branch_stroke);
                    painter.line_segment([p_fork, p_out_a], branch_stroke);
                    painter.line_segment([p_fork, p_out_b], branch_stroke);

                    // Fork dot
                    painter.circle_filled(p_fork, 3.0, Color32::from_rgb(217, 119, 6));

                    // Animated pulsing dots flowing along the fork if playing
                    if self.is_playing && is_flowing {
                        let t = (self.visualizer_anim_phase * 3.0).fract();
                        let d_fork = p_in.lerp(p_fork, t);
                        painter.circle_filled(d_fork, 2.0, Color32::WHITE);
                        let d_a = p_fork.lerp(p_out_a, t);
                        painter.circle_filled(d_a, 2.0, Color32::WHITE);
                        let d_b = p_fork.lerp(p_out_b, t);
                        painter.circle_filled(d_b, 2.0, Color32::WHITE);
                    }

                    // Readout text below fork
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 40.5),
                        egui::Align2::CENTER_CENTER,
                        "1 In ➔ Out A & B",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(217, 119, 6),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::VolumeBooster {
                let gain = (node.param_val).clamp(0.0, 24.0);
                let active_gain = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(gain);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let badge_rect = Rect::from_center_size(Pos2::new(node_rect.min.x + 20.0, cy), Vec2::new(22.0, 18.0));
                    painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(30, 27, 75));
                    painter.text(badge_rect.center(), egui::Align2::CENTER_CENTER, "⚡", egui::FontId::proportional(11.0), Color32::from_rgb(250, 204, 21));

                    let track_min_x = node_rect.min.x + 36.0;
                    let track_w = (node_width - 50.0).max(10.0);
                    let norm = (active_gain / 24.0).clamp(0.0, 1.0);
                    let fill_w = track_w * norm;
                    let thumb_x = track_min_x + fill_w;

                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 3.0), Vec2::new(track_w, 6.0)), 3.0, Color32::from_rgb(243, 232, 255));
                    let thrust_color = if norm > 0.75 {
                        Color32::from_rgb(239, 68, 68)
                    } else if norm > 0.4 {
                        Color32::from_rgb(139, 92, 246)
                    } else {
                        Color32::from_rgb(6, 182, 212)
                    };
                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 3.0), Vec2::new(fill_w, 6.0)), 3.0, thrust_color);

                    let led_pos = Pos2::new(track_min_x + track_w + 6.0, cy);
                    let led_color = if norm > 0.75 { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(34, 197, 94) };
                    painter.circle_filled(led_pos, 2.5, led_color);

                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.5, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.5, Stroke::new(1.8, thrust_color));

                    if self.is_playing && is_flowing {
                        let t = ui.input(|i| i.time);
                        let spark_x = track_min_x + ((t * 40.0) as f32 % fill_w.max(1.0));
                        painter.circle_filled(Pos2::new(spark_x, cy - 5.0), 1.2, Color32::from_rgb(250, 204, 21));
                        ui.ctx().request_repaint();
                    }

                    let readout = format!("+{:.1} dB Boost • Clean Amp", active_gain);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        thrust_color,
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::AnalogPreamp {
                let gain = (node.param_val).clamp(-24.0, 12.0);
                let active_gain = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(gain);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let dial_center = Pos2::new(node_rect.min.x + 22.0, cy);
                    let dial_radius = 11.5;
                    painter.circle_filled(dial_center, dial_radius, Color32::from_rgb(226, 232, 240));
                    painter.circle_stroke(dial_center, dial_radius, Stroke::new(1.5, Color32::from_rgb(148, 163, 184)));

                    let norm = ((active_gain - (-24.0)) / 36.0).clamp(0.0, 1.0);
                    let angle = std::f32::consts::PI * 0.75 + norm * (std::f32::consts::PI * 1.5);
                    let pointer_len = dial_radius - 2.5;
                    let p_tip = Pos2::new(
                        dial_center.x + angle.cos() * pointer_len,
                        dial_center.y + angle.sin() * pointer_len,
                    );
                    painter.line_segment([dial_center, p_tip], Stroke::new(2.2, Color32::from_rgb(234, 88, 12)));
                    painter.circle_filled(dial_center, 2.5, Color32::from_rgb(249, 115, 22));

                    let track_min_x = node_rect.min.x + 42.0;
                    let track_w = (node_width - 66.0).max(10.0);
                    let thumb_x = track_min_x + track_w * norm;

                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(track_w, 5.0)), 2.5, Color32::from_rgb(254, 243, 199));
                    let detent_x = track_min_x + track_w * (24.0 / 36.0);
                    painter.line_segment([Pos2::new(detent_x, cy - 4.5), Pos2::new(detent_x, cy + 4.5)], Stroke::new(1.2, Color32::from_rgb(217, 119, 6)));

                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.0, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.0, Stroke::new(1.8, Color32::from_rgb(234, 88, 12)));

                    let led_x = node_rect.max.x - 12.0;
                    let leds = [
                        (-12.0, Color32::from_rgb(34, 197, 94)),
                        (-3.0, Color32::from_rgb(34, 197, 94)),
                        (0.0, Color32::from_rgb(234, 179, 8)),
                        (6.0, Color32::from_rgb(239, 68, 68)),
                    ];
                    for (idx, (thresh, active_col)) in leds.iter().enumerate() {
                        let ly = cy + 9.0 - (idx as f32 * 6.0);
                        let is_on = active_gain >= *thresh;
                        let col = if is_on { *active_col } else { Color32::from_rgb(226, 232, 240) };
                        painter.circle_filled(Pos2::new(led_x, ly), 2.0, col);
                    }

                    let readout = format!("{:+0.1} dB • Class-A SAT", active_gain);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(194, 65, 12),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::DynamicLoudness {
                let depth = (node.param_val).clamp(0.0, 100.0);
                let active_depth = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(depth);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let moon_pos = Pos2::new(node_rect.min.x + 18.0, cy);
                    painter.text(moon_pos, egui::Align2::CENTER_CENTER, "🌙", egui::FontId::proportional(12.0), Color32::from_rgb(245, 158, 11));

                    let crt_w = (node_width - 44.0).max(10.0);
                    let crt_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 32.0, cy - 9.0), Vec2::new(crt_w, 18.0));
                    painter.rect_filled(crt_rect, 3.0, Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(crt_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), egui::StrokeKind::Outside);

                    let norm = active_depth / 100.0;
                    let c_painter = painter.with_clip_rect(crt_rect);
                    let mut curve_pts = Vec::new();
                    let steps = 12;
                    let y_mid = crt_rect.center().y;
                    for s in 0..=steps {
                        let t = s as f32 / steps as f32;
                        let x = crt_rect.min.x + crt_rect.width() * t;
                        let lift = if t < 0.3 {
                            (1.0 - t / 0.3) * 5.0 * norm
                        } else if t > 0.7 {
                            ((t - 0.7) / 0.3) * 3.5 * norm
                        } else {
                            0.0
                        };
                        let y = y_mid - lift + 1.0;
                        curve_pts.push(Pos2::new(x, y));
                    }
                    for i in 0..curve_pts.len() - 1 {
                        c_painter.line_segment([curve_pts[i], curve_pts[i + 1]], Stroke::new(1.5, Color32::from_rgb(251, 191, 36)));
                    }

                    let thumb_x = crt_rect.min.x + crt_rect.width() * norm;
                    c_painter.circle_filled(Pos2::new(thumb_x, crt_rect.max.y - 2.0), 2.5, Color32::WHITE);

                    let readout = format!("Loudness {:.0}% • ISO 226 Contour", active_depth);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(217, 119, 6),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::SteppedAttenuator {
                let db = (node.param_val).clamp(-60.0, 0.0);
                let active_db = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(db);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let dial_center = Pos2::new(node_rect.min.x + 20.0, cy);
                    let dial_r = 10.0;
                    painter.circle_filled(dial_center, dial_r, Color32::from_rgb(241, 245, 249));
                    painter.circle_stroke(dial_center, dial_r, Stroke::new(1.2, Color32::from_rgb(148, 163, 184)));
                    for d in 0..8 {
                        let ang = d as f32 * (std::f32::consts::PI * 2.0 / 8.0);
                        let dot_p = Pos2::new(dial_center.x + ang.cos() * (dial_r - 2.5), dial_center.y + ang.sin() * (dial_r - 2.5));
                        painter.circle_filled(dot_p, 1.0, Color32::from_rgb(100, 116, 139));
                    }

                    let nixie_w = 48.0;
                    let nixie_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 36.0, cy - 8.0), Vec2::new(nixie_w, 16.0));
                    painter.rect_filled(nixie_rect, 3.0, Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(nixie_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(6, 182, 212)), egui::StrokeKind::Outside);
                    painter.text(nixie_rect.center(), egui::Align2::CENTER_CENTER, format!("{:.0} dB", active_db), egui::FontId::monospace(9.5), Color32::from_rgb(34, 211, 238));

                    let is_dim = (active_db - (-20.0)).abs() < 0.5;
                    let dim_rect = Rect::from_min_size(Pos2::new(node_rect.max.x - 38.0, cy - 8.0), Vec2::new(30.0, 16.0));
                    let (dim_bg, dim_fg) = if is_dim {
                        (Color32::from_rgb(6, 182, 212), Color32::WHITE)
                    } else {
                        (Color32::from_rgb(241, 245, 249), Color32::from_rgb(100, 116, 139))
                    };
                    painter.rect_filled(dim_rect, 3.0, dim_bg);
                    painter.rect_stroke(dim_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                    painter.text(dim_rect.center(), egui::Align2::CENTER_CENTER, "DIM", egui::FontId::proportional(8.5), dim_fg);

                    let step_num = (((active_db - (-60.0)) / 60.0) * 63.0).round() as i32 + 1;
                    let readout = format!("Step {}/64 • Resistor Ladder", step_num);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(14, 116, 144),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::OpticalLeveler {
                let peak_red = (node.param_val).clamp(0.0, 100.0);
                let active_red = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(peak_red);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let bulb_center = Pos2::new(node_rect.min.x + 20.0, cy);
                    let norm = active_red / 100.0;
                    let glow_alpha = (100.0 + norm * 155.0) as u8;
                    painter.circle_filled(bulb_center, 8.5, Color32::from_rgba_premultiplied(234, 88, 12, glow_alpha));
                    painter.circle_filled(bulb_center, 5.0, Color32::from_rgba_premultiplied(254, 215, 170, 220));
                    painter.circle_stroke(bulb_center, 8.5, Stroke::new(1.0, Color32::from_rgb(194, 65, 12)));

                    let vu_w = (node_width - 48.0).max(10.0);
                    let vu_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 36.0, cy - 9.0), Vec2::new(vu_w, 18.0));
                    painter.rect_filled(vu_rect, 3.0, Color32::from_rgb(254, 243, 199));
                    painter.rect_stroke(vu_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(217, 119, 6)), egui::StrokeKind::Outside);

                    let vu_painter = painter.with_clip_rect(vu_rect);
                    for t in 0..=4 {
                        let tx = vu_rect.min.x + vu_rect.width() * (t as f32 / 4.0);
                        vu_painter.line_segment([Pos2::new(tx, vu_rect.min.y), Pos2::new(tx, vu_rect.min.y + 3.0)], Stroke::new(0.8, Color32::from_rgb(120, 53, 15)));
                    }

                    let needle_deflect = if self.is_playing {
                        let t = ui.input(|i| i.time);
                        norm * 0.7 + (t * 6.0).sin().abs() as f32 * norm * 0.3
                    } else {
                        norm
                    };
                    let needle_pivot = Pos2::new(vu_rect.center().x, vu_rect.max.y + 6.0);
                    let n_ang = -std::f32::consts::PI * 0.5 - (0.5 - needle_deflect) * 0.8;
                    let n_tip = Pos2::new(needle_pivot.x + n_ang.cos() * 22.0, needle_pivot.y + n_ang.sin() * 22.0);
                    vu_painter.line_segment([needle_pivot, n_tip], Stroke::new(1.4, Color32::from_rgb(185, 28, 28)));

                    let readout = format!("Peak Red {:.0}% • LA-2A Opto", active_red);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(180, 83, 9),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::StereoBalance {
                let bal = (node.param_val).clamp(-100.0, 100.0);
                let active_bal = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(bal);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let track_min_x = node_rect.min.x + 20.0;
                    let track_w = (node_width - 40.0).max(10.0);
                    let norm = ((active_bal - (-100.0)) / 200.0).clamp(0.0, 1.0);
                    let thumb_x = track_min_x + track_w * norm;

                    let l_col = if active_bal < -5.0 { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(148, 163, 184) };
                    painter.text(Pos2::new(track_min_x - 5.0, cy), egui::Align2::RIGHT_CENTER, "L", egui::FontId::proportional(9.0), l_col);

                    let r_col = if active_bal > 5.0 { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(148, 163, 184) };
                    painter.text(Pos2::new(track_min_x + track_w + 5.0, cy), egui::Align2::LEFT_CENTER, "R", egui::FontId::proportional(9.0), r_col);

                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(track_w, 5.0)), 2.5, Color32::from_rgb(238, 242, 255));
                    let center_x = track_min_x + track_w * 0.5;
                    painter.line_segment([Pos2::new(center_x, cy - 5.0), Pos2::new(center_x, cy + 5.0)], Stroke::new(1.3, Color32::from_rgb(99, 102, 241)));

                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.5, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.5, Stroke::new(1.8, Color32::from_rgb(79, 70, 229)));

                    let readout = if active_bal.abs() < 1.0 {
                        "Center (0.0 dB)".to_string()
                    } else if active_bal < 0.0 {
                        format!("L {:.0}% / R {:.0}%", 50.0 + active_bal.abs() * 0.5, 50.0 - active_bal.abs() * 0.5)
                    } else {
                        format!("L {:.0}% / R {:.0}%", 50.0 - active_bal * 0.5, 50.0 + active_bal * 0.5)
                    };
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(79, 70, 229),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::TubeWarmer {
                let warmth = (node.param_val).clamp(0.0, 100.0);
                let active_warmth = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(warmth);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let tube_rect = Rect::from_center_size(Pos2::new(node_rect.min.x + 22.0, cy), Vec2::new(18.0, 24.0));
                    painter.rect_filled(tube_rect, 6.0, Color32::from_rgb(30, 27, 75));
                    painter.rect_stroke(tube_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(79, 70, 229)), egui::StrokeKind::Outside);

                    let norm = active_warmth / 100.0;
                    let glow_pulse = if self.is_playing {
                        let t = ui.input(|i| i.time);
                        0.8 + 0.2 * (t * 4.0).sin().abs() as f32
                    } else {
                        1.0
                    };
                    let glow_alpha = ((120.0 + norm * 135.0) * glow_pulse).clamp(0.0, 255.0) as u8;
                    let f1 = Pos2::new(tube_rect.center().x - 3.5, tube_rect.center().y);
                    let f2 = Pos2::new(tube_rect.center().x + 3.5, tube_rect.center().y);
                    painter.circle_filled(f1, 2.5, Color32::from_rgba_premultiplied(249, 115, 22, glow_alpha));
                    painter.circle_filled(f2, 2.5, Color32::from_rgba_premultiplied(251, 146, 60, glow_alpha));

                    let track_min_x = node_rect.min.x + 42.0;
                    let track_w = (node_width - 54.0).max(10.0);
                    let thumb_x = track_min_x + track_w * norm;

                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(track_w, 5.0)), 2.5, Color32::from_rgb(255, 237, 213));
                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(thumb_x - track_min_x, 5.0)), 2.5, Color32::from_rgb(249, 115, 22));
                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.5, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.5, Stroke::new(1.8, Color32::from_rgb(234, 88, 12)));

                    let readout = format!("Warmth {:.0}% • Triode 12AX7", active_warmth);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(194, 65, 12),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::StereoWidener {
                let widener = (node.param_val).clamp(0.0, 100.0);
                let active_widener = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(widener);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 28.0;
                    let norm = active_widener / 100.0;

                    // 3D Spatial Soundstage Icon / Expansion Visualizer
                    let center_x = node_rect.min.x + 22.0;
                    let center_y = cy;
                    let pulse = if self.is_playing && is_flowing {
                        let t = ui.input(|i| i.time);
                        0.8 + 0.2 * (t * 4.0).sin().abs() as f32
                    } else {
                        1.0
                    };

                    for arc_i in 1..=3 {
                        let arc_rad = arc_i as f32 * 3.4 * pulse;
                        let alpha = ((60.0 + arc_i as f32 * 45.0) * pulse).clamp(0.0, 255.0) as u8;
                        let col = Color32::from_rgba_premultiplied(99, 102, 241, alpha);
                        painter.circle_stroke(Pos2::new(center_x, center_y), arc_rad, Stroke::new(1.0, col));
                    }
                    painter.circle_filled(Pos2::new(center_x, center_y), 2.5, Color32::from_rgb(129, 140, 248));

                    // Interactive Soundstage Slider
                    let track_min_x = node_rect.min.x + 40.0;
                    let track_w = (node_width - 54.0).max(10.0);
                    let thumb_x = track_min_x + track_w * norm;

                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(track_w, 5.0)), 2.5, Color32::from_rgb(238, 242, 255));
                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(thumb_x - track_min_x, 5.0)), 2.5, Color32::from_rgb(99, 102, 241));
                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.5, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.5, Stroke::new(1.8, Color32::from_rgb(79, 70, 229)));

                    let readout = format!("Soundstage +{:.0}% • Haas 3D", active_widener);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.5),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(67, 56, 202),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::Source {
                // 🎛️ FILE SOURCE: Real-time Song Progress Tracking Tape Spools & Format Badge
                let cy = node_rect.min.y + 33.0;
                let r_l_cx = node_rect.min.x + 24.0;
                let r_r_cx = node_rect.max.x - 24.0;
                let r_rad = 9.0;

                let reel_angle = if self.is_playing && is_flowing {
                    ui.ctx().request_repaint();
                    self.visualizer_anim_phase * 4.5
                } else {
                    0.0
                };

                // Track playback progress (0.0 to 1.0)
                let progress = if self.current_track.duration > 0.0 {
                    (self.current_time / self.current_track.duration).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                // Tape pack radius: Left reel shrinks as tape unwinds, Right reel grows as tape winds up
                let tape_pack_l = r_rad * (0.35 + 0.65 * (1.0 - progress));
                let tape_pack_r = r_rad * (0.35 + 0.65 * progress);

                // Tape guide line connecting the reels
                painter.line_segment(
                    [Pos2::new(r_l_cx, cy + r_rad), Pos2::new(r_r_cx, cy + r_rad)],
                    Stroke::new(1.5, Color32::from_rgb(100, 116, 139)),
                );

                // Dynamic traveling magnetic read head (moves from Left Reel to Right Reel according to song progress)
                let head_min_x = r_l_cx + 4.0;
                let head_max_x = r_r_cx - 4.0;
                let head_x = head_min_x + (head_max_x - head_min_x) * progress;

                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(head_x, cy + r_rad), Vec2::new(7.0, 5.0)),
                    1.5,
                    Color32::from_rgb(30, 41, 59),
                );
                // Active indicator dot on the magnetic read head
                let head_dot_col = if self.is_playing && is_flowing { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(148, 163, 184) };
                painter.circle_filled(Pos2::new(head_x, cy + r_rad), 1.2, head_dot_col);

                // Left Supply Reel (with tape pack)
                painter.circle_filled(Pos2::new(r_l_cx, cy), tape_pack_l, Color32::from_rgb(71, 85, 105));
                painter.circle_filled(Pos2::new(r_l_cx, cy), r_rad * 0.35, Color32::from_rgb(241, 245, 249));
                painter.circle_stroke(Pos2::new(r_l_cx, cy), r_rad, Stroke::new(1.2, Color32::from_rgb(148, 163, 184)));
                painter.circle_filled(Pos2::new(r_l_cx, cy), 2.5, Color32::from_rgb(100, 116, 139));
                for spk in 0..3 {
                    let ang = reel_angle + spk as f32 * (std::f32::consts::TAU / 3.0);
                    let ex = r_l_cx + ang.cos() * (r_rad - 1.5);
                    let ey = cy + ang.sin() * (r_rad - 1.5);
                    painter.line_segment([Pos2::new(r_l_cx, cy), Pos2::new(ex, ey)], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
                }

                // Right Take-Up Reel (with tape pack)
                painter.circle_filled(Pos2::new(r_r_cx, cy), tape_pack_r, Color32::from_rgb(71, 85, 105));
                painter.circle_filled(Pos2::new(r_r_cx, cy), r_rad * 0.35, Color32::from_rgb(241, 245, 249));
                painter.circle_stroke(Pos2::new(r_r_cx, cy), r_rad, Stroke::new(1.2, Color32::from_rgb(148, 163, 184)));
                painter.circle_filled(Pos2::new(r_r_cx, cy), 2.5, Color32::from_rgb(100, 116, 139));
                for spk in 0..3 {
                    let ang = reel_angle + spk as f32 * (std::f32::consts::TAU / 3.0);
                    let ex = r_r_cx + ang.cos() * (r_rad - 1.5);
                    let ey = cy + ang.sin() * (r_rad - 1.5);
                    painter.line_segment([Pos2::new(r_r_cx, cy), Pos2::new(ex, ey)], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
                }

                // Center format badge
                let badge_rect = Rect::from_center_size(Pos2::new(node_rect.center().x, cy - 6.0), Vec2::new(64.0, 13.0));
                painter.rect_filled(badge_rect, 3.0, Color32::from_rgb(238, 242, 255));
                painter.rect_stroke(badge_rect, 3.0, Stroke::new(0.8, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                let badge_txt = if !node.sub1.is_empty() { &node.sub1 } else { "FLAC Lossless" };
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_txt,
                    egui::FontId::proportional(8.0),
                    Color32::from_rgb(67, 56, 202),
                );

                let sub2_txt = if !node.sub2.is_empty() { &node.sub2 } else { "Lossless Master" };
                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 49.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(100, 116, 139),
                );
            } else if node.node_type == NodeType::Buffer {
                // ⚡ RAM BUFFER: Segmented 6-Block Low-Latency Cache Bank
                let seg_w = 13.0;
                let seg_h = 12.0;
                let seg_gap = 3.5;
                let total_w = 6.0 * seg_w + 5.0 * seg_gap;
                let start_x = node_rect.center().x - total_w * 0.5;
                let cy = node_rect.min.y + 27.0;

                for s in 0..6 {
                    let sx = start_x + s as f32 * (seg_w + seg_gap);
                    let s_rect = Rect::from_min_size(Pos2::new(sx, cy - seg_h * 0.5), Vec2::new(seg_w, seg_h));

                    let is_lit = is_flowing;
                    let pulse = if self.is_playing && is_flowing {
                        ui.ctx().request_repaint();
                        (self.visualizer_anim_phase * 5.0 + s as f32 * 0.9).sin() * 0.5 + 0.5
                    } else {
                        0.6
                    };

                    let col = if !is_flowing {
                        Color32::from_rgb(254, 202, 202)
                    } else if is_lit {
                        let r = (20.0 + 14.0 * pulse) as u8;
                        let g = (180.0 + 55.0 * pulse) as u8;
                        let b = (220.0 + 35.0 * pulse) as u8;
                        Color32::from_rgb(r, g, b)
                    } else {
                        Color32::from_rgb(226, 232, 240)
                    };

                    painter.rect_filled(s_rect, 2.0, col);
                    painter.rect_stroke(s_rect, 2.0, Stroke::new(0.8, Color32::from_rgb(14, 165, 233)), egui::StrokeKind::Outside);
                }

                let sub2_txt = if !node.sub2.is_empty() { &node.sub2 } else { "100% Buffered • 0 Disk I/O" };
                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 48.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(14, 116, 144),
                );
            } else if node.node_type == NodeType::Dsp {
                // 🎛️ DSP ENGINE: 7 Mini Dancing Graphic Spectrum Bars
                if !is_bypassed_effect {
                    let bar_count = 7;
                    let bar_w = 8.0;
                    let bar_gap = 4.0;
                    let total_w = bar_count as f32 * bar_w + (bar_count - 1) as f32 * bar_gap;
                    let start_x = node_rect.center().x - total_w * 0.5;
                    let base_y = node_rect.min.y + 39.0;
                    let max_h = 16.0;

                    for b in 0..bar_count {
                        let bx = start_x + b as f32 * (bar_w + bar_gap);
                        let spec_val = self.spectrum_bars[b * 6];
                        let active_h = if self.is_playing && is_flowing {
                            ui.ctx().request_repaint();
                            let wave_bounce = ((self.visualizer_anim_phase * 6.0 + b as f32 * 1.1).sin() * 0.35 + 0.35).abs();
                            (spec_val * 1.5).max(wave_bounce).clamp(0.15, 1.0) * max_h
                        } else {
                            3.5
                        };

                        let b_rect = Rect::from_min_size(Pos2::new(bx, base_y - active_h), Vec2::new(bar_w, active_h));
                        painter.rect_filled(b_rect, 1.5, Color32::from_rgb(129, 140, 248));
                        painter.line_segment(
                            [Pos2::new(bx, base_y - active_h - 1.5), Pos2::new(bx + bar_w, base_y - active_h - 1.5)],
                            Stroke::new(1.0, Color32::from_rgb(192, 132, 252)),
                        );
                    }

                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 48.0),
                        egui::Align2::CENTER_CENTER,
                        "64-bit Master DSP Bus",
                        egui::FontId::proportional(8.5),
                        Color32::from_rgb(99, 102, 241),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Direct Wire)",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                        egui::Align2::CENTER_CENTER,
                        "0.0 dB Pure Master",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::Driver {
                // 📡 APP RESAMPLER: Real-Time Sinc/PCM Oscilloscope Screen
                let scope_rect = Rect::from_min_size(
                    Pos2::new(node_rect.min.x + 14.0, node_rect.min.y + 19.0),
                    Vec2::new(node_width - 28.0, 21.0),
                );
                // Oscilloscope dark CRT glass bezel
                painter.rect_filled(scope_rect, 3.0, Color32::from_rgb(11, 19, 43));
                painter.rect_stroke(scope_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Outside);

                let sc_cy = scope_rect.center().y;

                // Oscilloscope grid reticle (horizontal center line and vertical division markers)
                painter.line_segment(
                    [Pos2::new(scope_rect.min.x + 2.0, sc_cy), Pos2::new(scope_rect.max.x - 2.0, sc_cy)],
                    Stroke::new(0.6, Color32::from_rgb(30, 41, 59)),
                );
                for g_idx in 1..4 {
                    let gx = scope_rect.min.x + scope_rect.width() * (g_idx as f32 * 0.25);
                    painter.line_segment(
                        [Pos2::new(gx, scope_rect.min.y + 2.0), Pos2::new(gx, scope_rect.max.y - 2.0)],
                        Stroke::new(0.5, Color32::from_rgba_unmultiplied(51, 65, 85, 90)),
                    );
                }

                let trace_painter = painter.with_clip_rect(scope_rect);
                let pts_count = 32;
                let mut trace_pts = Vec::with_capacity(pts_count);

                let is_active_audio = self.is_playing && is_flowing;
                if is_active_audio {
                    ui.ctx().request_repaint();
                }

                for p_idx in 0..pts_count {
                    let frac = p_idx as f32 / (pts_count - 1) as f32;
                    let px = scope_rect.min.x + 2.0 + (scope_rect.width() - 4.0) * frac;

                    let py = if is_active_audio {
                        let sample_idx = (p_idx * 48 / pts_count).min(47);
                        let sample = self.live_pcm_waveform[sample_idx];
                        let amp = (sample * 1.5).clamp(-0.95, 0.95);
                        sc_cy - amp * (scope_rect.height() * 0.44)
                    } else {
                        sc_cy
                    };
                    trace_pts.push(Pos2::new(px, py));
                }

                // Phosphor Bloom Glow underneath (softer, wider stroke)
                if is_active_audio {
                    for w in trace_pts.windows(2) {
                        trace_painter.line_segment([w[0], w[1]], Stroke::new(2.8, Color32::from_rgba_unmultiplied(52, 211, 153, 50)));
                    }
                }
                // High-intensity sharp beam trace on top
                let beam_color = if is_active_audio { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(51, 65, 85) };
                for w in trace_pts.windows(2) {
                    trace_painter.line_segment([w[0], w[1]], Stroke::new(1.3, beam_color));
                }

                // Micro "LIVE OSC" badge in top-right
                if is_active_audio {
                    painter.text(
                        Pos2::new(scope_rect.max.x - 3.5, scope_rect.min.y + 2.0),
                        egui::Align2::RIGHT_TOP,
                        "LIVE",
                        egui::FontId::proportional(6.0),
                        Color32::from_rgb(52, 211, 153),
                    );
                }

                let sub2_txt = if !node.sub2.is_empty() { &node.sub2 } else { "Rubato Sinc • 48.0 kHz" };
                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 49.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(16, 185, 129),
                );
            } else if node.node_type == NodeType::Endpoint {
                // 🔊 OUTPUT DAC: Precision Studio Stereo Meter (L & R) with Peak-Hold & Clip Warning
                let dac_rect = Rect::from_min_size(
                    Pos2::new(node_rect.min.x + 12.0, node_rect.min.y + 19.0),
                    Vec2::new(node_width - 24.0, 22.0),
                );
                // Sleek obsidian chassis window
                painter.rect_filled(dac_rect, 3.0, Color32::from_rgb(10, 15, 29));
                painter.rect_stroke(dac_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Outside);

                if self.is_playing && is_flowing {
                    ui.ctx().request_repaint();
                }

                // Render Channel L (top row) and Channel R (bottom row)
                let row_h = 7.5;
                let ch_y_l = dac_rect.min.y + 2.5;
                let ch_y_r = dac_rect.min.y + 12.0;

                let meter_x = dac_rect.min.x + 13.0;
                let meter_w = dac_rect.width() - 25.0; // Room for L/R label on left, clip LED on right
                let seg_count = 14;
                let seg_w = (meter_w - (seg_count - 1) as f32 * 1.5) / seg_count as f32;

                for (ch_idx, &ch_y) in [ch_y_l, ch_y_r].iter().enumerate() {
                    let is_left = ch_idx == 0;
                    let level = if is_left { self.dac_meter_l } else { self.dac_meter_r };
                    let peak_hold = if is_left { self.dac_peak_hold_l } else { self.dac_peak_hold_r };
                    let is_clipped = if is_left { self.dac_clip_l } else { self.dac_clip_r };

                    // Channel badge ("L" or "R")
                    let lbl_col = if is_left { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(168, 85, 247) };
                    painter.text(
                        Pos2::new(dac_rect.min.x + 3.0, ch_y + row_h * 0.5),
                        egui::Align2::LEFT_CENTER,
                        if is_left { "L" } else { "R" },
                        egui::FontId::proportional(7.5),
                        lbl_col,
                    );

                    // Multi-Segment Meter Bar
                    let active_segs = (level * seg_count as f32).round() as usize;
                    for s in 0..seg_count {
                        let sx = meter_x + s as f32 * (seg_w + 1.5);
                        let s_rect = Rect::from_min_size(Pos2::new(sx, ch_y), Vec2::new(seg_w, row_h));

                        let is_seg_lit = s < active_segs && (self.is_playing && is_flowing);
                        let seg_frac = s as f32 / seg_count as f32;

                        let col = if is_seg_lit {
                            if seg_frac < 0.65 {
                                Color32::from_rgb(16, 185, 129) // Green (-18 to -6 dB)
                            } else if seg_frac < 0.88 {
                                Color32::from_rgb(245, 158, 11) // Amber (-6 to -1 dB)
                            } else {
                                Color32::from_rgb(239, 68, 68)  // Red Peak (0 dBFS)
                            }
                        } else {
                            Color32::from_rgb(20, 27, 45) // Inactive dark slot
                        };

                        painter.rect_filled(s_rect, 1.0, col);
                    }

                    // Floating Peak-Hold Needle / Marker
                    if self.is_playing && is_flowing && peak_hold > 0.05 {
                        let ph_x = meter_x + (meter_w - 2.0) * peak_hold.clamp(0.0, 1.0);
                        let ph_col = if peak_hold > 0.88 {
                            Color32::from_rgb(254, 202, 202)
                        } else if peak_hold > 0.65 {
                            Color32::from_rgb(254, 240, 138)
                        } else {
                            Color32::from_rgb(167, 243, 208)
                        };
                        painter.line_segment(
                            [Pos2::new(ph_x, ch_y), Pos2::new(ph_x, ch_y + row_h)],
                            Stroke::new(1.5, ph_col),
                        );
                    }

                    // 0 dBFS Instant Clip Warning LED
                    let led_cx = dac_rect.max.x - 5.5;
                    let led_cy = ch_y + row_h * 0.5;
                    let (clip_fill, clip_glow) = if is_clipped && is_flowing {
                        (Color32::from_rgb(239, 68, 68), true)
                    } else {
                        (Color32::from_rgb(30, 41, 59), false)
                    };
                    if clip_glow {
                        painter.circle_filled(Pos2::new(led_cx, led_cy), 3.5, Color32::from_rgba_unmultiplied(239, 68, 68, 80));
                    }
                    painter.circle_filled(Pos2::new(led_cx, led_cy), 2.0, clip_fill);
                }

                let sub2_txt = if !node.sub2.is_empty() { &node.sub2 } else { "32-bit Float PCM Direct" };
                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 49.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(217, 119, 6),
                );
            } else if let NodeType::SavedEqPreset(ref name) = node.node_type {
                // 🎛️ EQ PRESET: Mini Dynamic Frequency Curve CRT Screen
                let screen_rect = Rect::from_min_size(
                    Pos2::new(node_rect.min.x + 14.0, node_rect.min.y + 19.0),
                    Vec2::new(node_width - 28.0, 19.0),
                );
                painter.rect_filled(screen_rect, 3.0, Color32::from_rgb(15, 23, 42));
                painter.rect_stroke(screen_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), egui::StrokeKind::Outside);

                // Zero line (0 dB)
                let sc_cy = screen_rect.center().y;
                painter.line_segment(
                    [Pos2::new(screen_rect.min.x + 2.0, sc_cy), Pos2::new(screen_rect.max.x - 2.0, sc_cy)],
                    Stroke::new(0.6, Color32::from_rgb(30, 41, 59)),
                );

                // Render dynamic EQ curve across the screen width
                let curve_painter = painter.with_clip_rect(screen_rect);
                let pts_count = 24;
                let mut curve_pts = Vec::new();
                let is_active_live = &self.active_eq.name == name;
                let bands = if is_active_live {
                    &self.active_eq.bands
                } else if let Some(p) = self.eq_presets.iter().find(|p| &p.name == name) {
                    &p.bands
                } else {
                    &self.active_eq.bands
                };

                for p_idx in 0..=pts_count {
                    let frac = p_idx as f64 / pts_count as f64;
                    // Log frequency mapping from 20 Hz to 20000 Hz
                    let freq = 20.0 * (1000.0_f64).powf(frac);
                    let db = crate::ui::equalizer::compute_total_curve_db(bands, freq, 44100.0, 0.0, !is_flowing);
                    let px = screen_rect.min.x + screen_rect.width() * frac as f32;
                    let py = (sc_cy - (db as f32 / 12.0) * 8.0).clamp(screen_rect.min.y + 1.0, screen_rect.max.y - 1.0);
                    curve_pts.push(Pos2::new(px, py));
                }

                let curve_stroke_color = if !is_flowing {
                    Color32::from_rgb(148, 163, 184)
                } else {
                    Color32::from_rgb(168, 85, 247) // Purple EQ curve
                };

                for w in curve_pts.windows(2) {
                    curve_painter.line_segment([w[0], w[1]], Stroke::new(1.4, curve_stroke_color));
                }

                // Preset name and live status
                let sub2_txt = if is_flowing {
                    if is_active_live { "Live Tuning • Aktif" } else { "Preset • Aktif" }
                } else {
                    "Klik untuk Buka EQ"
                };
                let sub2_col = if is_flowing {
                    Color32::from_rgb(168, 85, 247)
                } else {
                    Color32::from_rgb(100, 116, 139)
                };

                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 48.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(8.5),
                    sub2_col,
                );
            } else {
                // Sublabels: Dynamic based on bypassed vs flowing vs broken
                let (sub1_txt, sub1_col, sub2_txt, sub2_col) = if is_bypassed_effect {
                    if node.node_type == NodeType::Dsp {
                        ("DILEWATI (Direct Wire)", Color32::from_rgb(148, 163, 184), "0.0 dB Pure Master", Color32::from_rgb(148, 163, 184))
                    } else if let NodeType::SavedEqPreset(_) = node.node_type {
                        (&node.sub1[..], Color32::from_rgb(71, 85, 105), "DILEWATI (Bypass)", Color32::from_rgb(148, 163, 184))
                    } else {
                        ("DILEWATI (Bypassed)", Color32::from_rgb(148, 163, 184), "Nonaktif di Bit-Perfect", Color32::from_rgb(148, 163, 184))
                    }
                } else if is_flowing {
                    (&node.sub1[..], theme_color, &node.sub2[..], Color32::from_rgb(100, 116, 139))
                } else {
                    ("NO SIGNAL", Color32::from_rgb(239, 68, 68), "Terputus", Color32::from_rgb(220, 38, 38))
                };

                if is_hovered && matches!(node.node_type, NodeType::SavedEqPreset(_) | NodeType::Dsp) {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                let display_sub1 = if sub1_txt.len() > 17 {
                    format!("{}...", &sub1_txt[..15])
                } else {
                    sub1_txt.to_string()
                };

                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                    egui::Align2::CENTER_CENTER,
                    &display_sub1,
                    egui::FontId::proportional(10.0),
                    sub1_col,
                );

                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 41.5),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(9.0),
                    sub2_col,
                );
            }

            // Status Indicator Dot (Top-Right)
            let status_dot_pos = Pos2::new(node_rect.max.x - 8.0, node_rect.min.y + 8.0);
            if is_bypassed_effect {
                painter.circle_filled(status_dot_pos, 3.0, Color32::from_rgb(203, 213, 225));
            } else if is_flowing {
                if self.is_playing {
                    painter.circle_filled(status_dot_pos, 3.5, theme_color);
                } else {
                    painter.circle_filled(status_dot_pos, 3.0, theme_color);
                }
            } else {
                painter.circle_filled(status_dot_pos, 4.0, Color32::from_rgb(239, 68, 68));
            }

            // Input Pin (Left) - Except Source node
            if node.node_type != NodeType::Source {
                if node.node_type == NodeType::Mixer {
                    let pin_a = Pos2::new(node_rect.min.x, node_rect.center().y - 12.0);
                    let pin_b = Pos2::new(node_rect.min.x, node_rect.center().y + 12.0);
                    let is_hover_a = mouse_pos.map_or(false, |p| p.distance(pin_a) <= 10.0);
                    let is_hover_b = mouse_pos.map_or(false, |p| p.distance(pin_b) <= 10.0);
                    if is_hover_a || is_hover_b {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    let col_a = if is_hover_a { Color32::from_rgb(14, 165, 233) } else if is_flowing { theme_color } else { Color32::from_rgb(239, 68, 68) };
                    let col_b = if is_hover_b { Color32::from_rgb(14, 165, 233) } else if is_flowing { theme_color } else { Color32::from_rgb(239, 68, 68) };

                    painter.circle_filled(pin_a, pin_radius, col_a);
                    painter.circle_stroke(pin_a, pin_radius, Stroke::new(1.5, Color32::WHITE));
                    painter.text(Pos2::new(pin_a.x + 8.0, pin_a.y), egui::Align2::LEFT_CENTER, "A", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

                    painter.circle_filled(pin_b, pin_radius, col_b);
                    painter.circle_stroke(pin_b, pin_radius, Stroke::new(1.5, Color32::WHITE));
                    painter.text(Pos2::new(pin_b.x + 8.0, pin_b.y), egui::Align2::LEFT_CENTER, "B", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));
                } else {
                    let in_pin = Pos2::new(node_rect.min.x, node_rect.center().y);
                    let is_hover_in = mouse_pos.map_or(false, |p| p.distance(in_pin) <= 10.0);
                    if is_hover_in {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    let pin_col = if is_hover_in {
                        Color32::from_rgb(14, 165, 233)
                    } else if is_bypassed_effect {
                        Color32::from_rgb(203, 213, 225)
                    } else if is_flowing {
                        theme_color
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };
                    painter.circle_filled(in_pin, pin_radius, pin_col);
                    painter.circle_stroke(in_pin, pin_radius, Stroke::new(1.5, Color32::WHITE));
                }
            }

            // Output Pin (Right) - Except Endpoint node
            if node.node_type != NodeType::Endpoint {
                if node.node_type == NodeType::Splitter {
                    let pin_a = Pos2::new(node_rect.max.x, node_rect.center().y - 12.0);
                    let pin_b = Pos2::new(node_rect.max.x, node_rect.center().y + 12.0);
                    let is_hover_a = mouse_pos.map_or(false, |p| p.distance(pin_a) <= 10.0);
                    let is_hover_b = mouse_pos.map_or(false, |p| p.distance(pin_b) <= 10.0);
                    if is_hover_a || is_hover_b {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    let col_a = if is_hover_a { Color32::from_rgb(14, 165, 233) } else if is_flowing { theme_color } else { Color32::from_rgb(239, 68, 68) };
                    let col_b = if is_hover_b { Color32::from_rgb(14, 165, 233) } else if is_flowing { theme_color } else { Color32::from_rgb(239, 68, 68) };

                    painter.circle_filled(pin_a, pin_radius, col_a);
                    painter.circle_stroke(pin_a, pin_radius, Stroke::new(1.5, Color32::WHITE));
                    painter.text(Pos2::new(pin_a.x - 8.0, pin_a.y), egui::Align2::RIGHT_CENTER, "A", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

                    painter.circle_filled(pin_b, pin_radius, col_b);
                    painter.circle_stroke(pin_b, pin_radius, Stroke::new(1.5, Color32::WHITE));
                    painter.text(Pos2::new(pin_b.x - 8.0, pin_b.y), egui::Align2::RIGHT_CENTER, "B", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));
                } else {
                    let out_pin = Pos2::new(node_rect.max.x, node_rect.center().y);
                    let is_hover_out = mouse_pos.map_or(false, |p| p.distance(out_pin) <= 10.0);
                    if is_hover_out {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    let pin_col = if is_hover_out {
                        Color32::from_rgb(14, 165, 233)
                    } else if is_bypassed_effect {
                        Color32::from_rgb(203, 213, 225)
                    } else if is_flowing {
                        theme_color
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };
                    painter.circle_filled(out_pin, pin_radius, pin_col);
                    painter.circle_stroke(out_pin, pin_radius, Stroke::new(1.5, Color32::WHITE));
                }
            }
        }

        // Apply real-time node parameter adjustments (e.g. LowPassFilter cutoff or Mixer crossfader)
        if let Some((node_id, new_val)) = param_change {
            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                n.param_val = new_val;
                match n.node_type {
                    NodeType::LowPassFilter => {
                        n.sub1 = if new_val >= 1000.0 {
                            format!("Cutoff {:.1} kHz", new_val / 1000.0)
                        } else {
                            format!("Cutoff {:.0} Hz", new_val)
                        };
                    }
                    NodeType::Mixer => {
                        let a_pct = (1.0 - new_val) * 100.0;
                        let b_pct = new_val * 100.0;
                        n.sub1 = format!("Mix: {:.0}% A • {:.0}% B", a_pct, b_pct);
                    }
                    NodeType::TubeWarmer => {
                        n.sub1 = format!("Warmth {:.0}%", new_val);
                    }
                    NodeType::StereoWidener => {
                        n.sub1 = format!("+{:.0}% Soundstage", new_val);
                    }
                    NodeType::VolumeBooster => {
                        n.sub1 = format!("+{:.1} dB Gain", new_val);
                    }
                    NodeType::AnalogPreamp => {
                        n.sub1 = format!("{:+0.1} dB Drive", new_val);
                    }
                    NodeType::DynamicLoudness => {
                        n.sub1 = format!("Loudness {:.0}%", new_val);
                    }
                    NodeType::SteppedAttenuator => {
                        n.sub1 = format!("{:.0} dB", new_val);
                    }
                    NodeType::OpticalLeveler => {
                        n.sub1 = format!("Peak Red {:.0}%", new_val);
                    }
                    NodeType::StereoBalance => {
                        if new_val.abs() < 1.0 {
                            n.sub1 = "Center 0.0 dB".to_string();
                        } else if new_val < 0.0 {
                            n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 + new_val.abs() * 0.5, 50.0 - new_val.abs() * 0.5);
                        } else {
                            n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 - new_val * 0.5, 50.0 + new_val * 0.5);
                        }
                    }
                    _ => {}
                }
            }
            if self.chain_mode != ChainMode::DspActive {
                self.set_chain_mode(ChainMode::DspActive);
            }
            if let Some(ref engine) = self.audio_engine {
                engine.set_node_param(node_id as u32, 0, new_val);
            }
        }

        if let Some(ref name) = navigate_to_eq_preset {
            if let Some(preset) = self.eq_presets.iter().find(|p| &p.name == name).cloned() {
                self.active_eq = preset;
                self.eq_is_dirty = false;
                self.sync_parametric_to_legacy_eq();
            }
            self.current_tab = crate::models::AppTab::Equalizer;
        } else if navigate_to_main_eq {
            self.current_tab = crate::models::AppTab::Equalizer;
        }

        // 7. Interactive Bottom Status Pill
        let banner_rect = Rect::from_min_size(
            Pos2::new(canvas_rect.min.x + 12.0, canvas_rect.max.y - 25.0),
            Vec2::new(canvas_rect.width() - 24.0, 19.0),
        );
        let (pill_bg, pill_stroke, status_icon, status_msg, text_col) = if chain_eval.is_complete {
            if self.is_playing {
                if is_fallback {
                    (
                        Color32::from_rgba_premultiplied(254, 243, 199, 230),
                        Stroke::new(1.0, Color32::from_rgb(253, 230, 138)),
                        "⚠️",
                        "FALLBACK KE OS RESAMPLED — Hardware tidak mendukung sample rate asli file, dialihkan ke 48.0 kHz",
                        Color32::from_rgb(146, 64, 14),
                    )
                } else {
                    match self.chain_mode {
                        ChainMode::BitPerfect => {
                            let is_ex = self.signal_path_report.as_ref().map_or(false, |r| r.is_exclusive);
                            let sub_text = if is_ex {
                                "JALUR AUDIO AKTIF (BIT-PERFECT) — WASAPI Exclusive Mode (Direct Bitstream), bypass DSP & OS mixer"
                            } else {
                                "JALUR AUDIO AKTIF (BIT-PERFECT) — Shared Mode Passthrough (1:1 Native Rate), bypass DSP & Volume"
                            };
                            (
                                Color32::from_rgba_premultiplied(236, 253, 245, 230),
                                Stroke::new(1.0, Color32::from_rgb(167, 243, 208)),
                                "🟢",
                                sub_text,
                                Color32::from_rgb(6, 95, 70),
                            )
                        }
                        ChainMode::DspActive => (
                            Color32::from_rgba_premultiplied(238, 242, 255, 230),
                            Stroke::new(1.0, Color32::from_rgb(199, 210, 254)),
                            "🟣",
                            "JALUR AUDIO AKTIF (DSP ACTIVE) — 10-Band Peaking EQ dalam presisi 64-bit Float (f64) dengan Headroom Guard",
                            Color32::from_rgb(67, 56, 202),
                        ),
                        ChainMode::OsResampled => (
                            Color32::from_rgba_premultiplied(254, 243, 199, 230),
                            Stroke::new(1.0, Color32::from_rgb(253, 230, 138)),
                            "🟡",
                            "JALUR AUDIO AKTIF (OS RESAMPLED) — Resampling ke 48.0 kHz via Rubato Sinc SRC Shared Mode",
                            Color32::from_rgb(146, 64, 14),
                        ),
                    }
                }
            } else {
                (
                    Color32::from_rgba_premultiplied(241, 245, 249, 230),
                    Stroke::new(1.0, Color32::from_rgb(226, 232, 240)),
                    "⚪",
                    "SIGNAL CHAIN LENGKAP — Rantai audio terhubung sempurna (Tekan Play untuk memutar)",
                    Color32::from_rgb(71, 85, 105),
                )
            }
        } else {
            (
                Color32::from_rgba_premultiplied(254, 242, 242, 240),
                Stroke::new(1.2, Color32::from_rgb(252, 165, 165)),
                "🔴",
                "JALUR SINYAL TERPUTUS — Audio dibisukan otomatis. Klik tanda ✕ merah untuk menyambung!",
                Color32::from_rgb(185, 28, 28),
            )
        };

        painter.rect_filled(banner_rect, 6.0, pill_bg);
        painter.rect_stroke(banner_rect, 6.0, pill_stroke, egui::StrokeKind::Outside);
        painter.text(
            banner_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{} {}", status_icon, status_msg),
            egui::FontId::proportional(10.5),
            text_col,
        );

        // Add reconnected cable
        if let Some((from_id, to_id)) = connection_to_add {
            if !self.connections.iter().any(|c| c.from_node == from_id && c.to_node == to_id) {
                self.connections.push(CableConnection {
                    from_node: from_id,
                    to_node: to_id,
                });
                topology_changed = true;
            }
        }

        // Disconnect all cables connected to this node
        if let Some(d_id) = disconnect_node {
            self.connections.retain(|c| c.from_node != d_id && c.to_node != d_id);
            topology_changed = true;
        }

        // Mode switching (via DSP card click or toggle)
        if toggle_dsp {
            let next_mode = match self.chain_mode {
                ChainMode::DspActive => ChainMode::BitPerfect,
                _ => ChainMode::DspActive,
            };
            self.set_chain_mode(next_mode);
        } else if let Some(m) = switch_mode {
            self.set_chain_mode(m);
        }

        // Delete node and its connections
        if let Some(del_id) = node_to_delete {
            self.nodes.retain(|n| n.id != del_id);
            self.connections.retain(|c| !node_to_delete.is_some_and(|d| c.from_node == d || c.to_node == d));
            if self.selected_node == Some(del_id) {
                self.selected_node = None;
            }
            topology_changed = true;
        }

        if topology_changed {
            self.invalidate_chain_cache();
            let chain_eval = self.evaluate_signal_chain();
            let has_flowing_custom_nodes = self.nodes.iter().any(|n| n.is_removable && chain_eval.flowing_nodes.contains(&n.id));
            if has_flowing_custom_nodes && self.chain_mode != ChainMode::DspActive {
                self.set_chain_mode(ChainMode::DspActive);
            } else {
                self.sync_signal_chain_to_engine();
            }
        }

        // Empty Canvas Context Menu: Right-Click on empty canvas background to quickly add modules
        if !is_hovering_any_node {
            response.context_menu(|ui| {
                ui.label(egui::RichText::new("➕ TAMBAH MODUL STUDIO").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                ui.separator();
                if ui.button("🌊 Low-Pass Filter").clicked() {
                    self.add_modular_filter(NodeType::LowPassFilter);
                    ui.close();
                }
                if ui.button("🎚️ Mixer / Summing Bus (Gerbang Paralel)").clicked() {
                    self.add_modular_filter(NodeType::Mixer);
                    ui.close();
                }
                if ui.button("🔀 Splitter / Y-Branch Bus (Gerbang Cabang)").clicked() {
                    self.add_modular_filter(NodeType::Splitter);
                    ui.close();
                }
                if ui.button("🔥 Tube Warmer (Analog)").clicked() {
                    self.add_modular_filter(NodeType::TubeWarmer);
                    ui.close();
                }
                if ui.button("🌐 Stereo 3D Widener").clicked() {
                    self.add_modular_filter(NodeType::StereoWidener);
                    ui.close();
                }
                if ui.button("🔉 Volume Attenuator (-6dB)").clicked() {
                    self.add_modular_filter(NodeType::VolumeAttenuator);
                    ui.close();
                }
                ui.separator();
                ui.label(egui::RichText::new("🔊 Manipulasi Volume & Gain:").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                if ui.button("⚡ Volume Booster (0 s/d +24dB)").clicked() {
                    self.add_modular_filter(NodeType::VolumeBooster);
                    ui.close();
                }
                if ui.button("🎛️ Analog Preamp & Trim (-24 s/d +12dB)").clicked() {
                    self.add_modular_filter(NodeType::AnalogPreamp);
                    ui.close();
                }
                if ui.button("🌙 Dynamic Loudness (ISO 226 Contour)").clicked() {
                    self.add_modular_filter(NodeType::DynamicLoudness);
                    ui.close();
                }
                if ui.button("🔘 Stepped Attenuator (64-Step Resistor)").clicked() {
                    self.add_modular_filter(NodeType::SteppedAttenuator);
                    ui.close();
                }
                if ui.button("🛡️ Optical Leveler (LA-2A Opto)").clicked() {
                    self.add_modular_filter(NodeType::OpticalLeveler);
                    ui.close();
                }
                if ui.button("⚖️ Stereo Balance & Trim (Pan Stage)").clicked() {
                    self.add_modular_filter(NodeType::StereoBalance);
                    ui.close();
                }
                ui.separator();
                if ui.button("🔄 Reset Posisi Canvas").clicked() {
                    self.patchbay_pan = Pos2::ZERO.to_vec2();
                    ui.close();
                }
            });
        }

        // Render floating beginner visual tutorial module in top-right of canvas
        self.render_patchbay_tutorial_card(ui, canvas_rect);

        // Render filter-specific routing tutorial module
        self.render_filter_tutorial_card(ui, canvas_rect);
    }

    // Right Sidebar Filter & FX Toolbox Palette
    pub fn render_fx_sidebar_palette(&mut self, ui: &mut egui::Ui) {
        let mut delete_filter_type: Option<NodeType> = None;
        let mut delete_all_saved_eq_nodes = false;

        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("🎛️ FILTER & FX PALETTE").strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                ui.label(egui::RichText::new("Klik kiri untuk menambah • Klik kanan untuk opsi hapus").size(11.0).color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Filter 0: Low-Pass Filter (Featured Test Filter)
                    let lpf_nodes: Vec<(usize, f32)> = self.nodes.iter()
                        .filter(|n| n.node_type == NodeType::LowPassFilter)
                        .map(|n| (n.id, n.param_val))
                        .collect();
                    let has_lpf = !lpf_nodes.is_empty();

                    let active_lpf_idx = lpf_nodes.iter()
                        .position(|(id, _)| Some(*id) == self.selected_node)
                        .unwrap_or(0);

                    let (target_id, current_lpf_cutoff) = if has_lpf {
                        let (id, cut) = lpf_nodes[active_lpf_idx];
                        (Some(id), cut)
                    } else {
                        (None, 800.0)
                    };

                    let cutoff_str = if current_lpf_cutoff >= 1000.0 {
                        format!("{:.1} kHz", current_lpf_cutoff / 1000.0)
                    } else {
                        format!("{:.0} Hz", current_lpf_cutoff)
                    };

                    let mut lpf_sidebar_drag_change: Option<f32> = None;
                    let mut switch_to_lpf_id: Option<usize> = None;

                    let g_lpf = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🌊 Low-Pass Filter").strong().size(12.5).color(Color32::from_rgb(14, 116, 144)));
                                ui.label(egui::RichText::new(&cutoff_str).strong().size(10.5).color(Color32::from_rgb(2, 132, 199)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Low-Pass Filter",
                                        NodeType::LowPassFilter,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });

                            // Opsi B: Tab Selector Pills if multiple LPF cards exist on canvas
                            if lpf_nodes.len() > 1 {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Card Aktif:").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    for (i, (id, cut)) in lpf_nodes.iter().enumerate() {
                                        let is_sel = i == active_lpf_idx;
                                        let lbl = if *cut >= 1000.0 {
                                            format!("#{}: {:.1}k", i + 1, cut / 1000.0)
                                        } else {
                                            format!("#{}: {:.0}Hz", i + 1, cut)
                                        };
                                        let btn = if is_sel {
                                            ui.add(egui::Button::new(egui::RichText::new(lbl).size(9.0).strong().color(Color32::WHITE)).fill(Color32::from_rgb(14, 165, 233)))
                                        } else {
                                            ui.button(egui::RichText::new(lbl).size(9.0).color(Color32::from_rgb(71, 85, 105)))
                                        };
                                        if btn.clicked() {
                                            switch_to_lpf_id = Some(*id);
                                        }
                                    }
                                });
                            }

                            ui.label(egui::RichText::new(format!(
                                "Memotong frekuensi treble di atas {}. Suara vokal dan instrumen tinggi teredam tebal (efek 'underwater / muffled') untuk uji coba signal chain.",
                                cutoff_str
                            )).size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            // Visual mini frequency response graph (synchronized with canvas slider & interactive)
                            let (graph_rect, graph_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), egui::Sense::click_and_drag());
                            let g_painter = ui.painter().with_clip_rect(graph_rect);
                            g_painter.rect_filled(graph_rect, 4.0, Color32::from_rgb(240, 249, 255));
                            g_painter.rect_stroke(graph_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);

                            let x_start = graph_rect.min.x + 8.0;
                            let x_end = graph_rect.max.x - 8.0;
                            let usable_w = (x_end - x_start).max(10.0);

                            // Interactivity: Dragging / Clicking on mini sidebar graph also adjusts cutoff of active LPF
                            if (graph_resp.clicked() || graph_resp.dragged()) && has_lpf {
                                if let Some(pos) = graph_resp.interact_pointer_pos() {
                                    let norm = ((pos.x - x_start) / usable_w).clamp(0.0, 1.0);
                                    let new_cutoff = (100.0 * (100.0_f32).powf(norm)).clamp(100.0, 10000.0);
                                    lpf_sidebar_drag_change = Some(new_cutoff);
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    ui.ctx().request_repaint();
                                }
                            }

                            let y_top = graph_rect.min.y + 7.0;
                            let y_bottom = graph_rect.max.y - 7.0;

                            // Opsi A: Draw Secondary (Ghost) Curve(s) for other LPF cards in canvas
                            for (i, (_id, other_cut)) in lpf_nodes.iter().enumerate() {
                                if i != active_lpf_idx {
                                    let ghost_norm = ((other_cut / 100.0).log10() / 2.0).clamp(0.05, 0.95);
                                    let g_x_cut = x_start + usable_w * ghost_norm;

                                    let mut g_pts = Vec::new();
                                    g_pts.push(Pos2::new(x_start, y_top));
                                    if g_x_cut - 8.0 > x_start {
                                        g_pts.push(Pos2::new(g_x_cut - 8.0, y_top));
                                    }
                                    g_pts.push(Pos2::new(g_x_cut, y_top + 3.0));
                                    let drop_x = (g_x_cut + 14.0).min(x_end);
                                    g_pts.push(Pos2::new(drop_x, y_bottom));
                                    g_pts.push(Pos2::new(x_end, y_bottom));

                                    for seg in 0..g_pts.len() - 1 {
                                        g_painter.line_segment(
                                            [g_pts[seg], g_pts[seg + 1]],
                                            Stroke::new(1.3, Color32::from_rgba_premultiplied(148, 163, 184, 160)),
                                        );
                                    }

                                    let g_marker = Pos2::new(g_x_cut, y_top + 3.0);
                                    g_painter.circle_filled(g_marker, 2.5, Color32::from_rgb(148, 163, 184));
                                    let ghost_lbl = if *other_cut >= 1000.0 {
                                        format!("#{}: {:.1}k", i + 1, other_cut / 1000.0)
                                    } else {
                                        format!("#{}: {:.0}Hz", i + 1, other_cut)
                                    };
                                    g_painter.text(
                                        Pos2::new(g_x_cut, y_bottom - 2.0),
                                        egui::Align2::CENTER_BOTTOM,
                                        ghost_lbl,
                                        egui::FontId::proportional(7.5),
                                        Color32::from_rgb(148, 163, 184),
                                    );
                                }
                            }

                            // Dynamic cutoff X for Primary Active Curve
                            let norm_cut = ((current_lpf_cutoff / 100.0).log10() / 2.0).clamp(0.05, 0.95);
                            let x_cut = x_start + usable_w * norm_cut;

                            let mut curve_pts = Vec::new();
                            curve_pts.push(Pos2::new(x_start, y_top));
                            if x_cut - 8.0 > x_start {
                                curve_pts.push(Pos2::new(x_cut - 8.0, y_top));
                            }
                            curve_pts.push(Pos2::new(x_cut, y_top + 3.0));
                            let drop_x = (x_cut + 14.0).min(x_end);
                            curve_pts.push(Pos2::new(drop_x, y_bottom));
                            curve_pts.push(Pos2::new(x_end, y_bottom));

                            for i in 0..curve_pts.len() - 1 {
                                g_painter.line_segment([curve_pts[i], curve_pts[i + 1]], Stroke::new(2.2, Color32::from_rgb(2, 132, 199)));
                            }

                            // Animated pulsing cutoff marker dot when audio is playing
                            let time = ui.input(|i| i.time);
                            let pulse = if self.is_playing {
                                ui.ctx().request_repaint();
                                (time * 5.0).sin().abs() as f32
                            } else {
                                0.0
                            };

                            let marker_pos = Pos2::new(x_cut, y_top + 3.0);
                            let marker_r = 3.5 + pulse * 1.5;
                            if self.is_playing {
                                g_painter.circle_filled(marker_pos, marker_r + 2.0, Color32::from_rgba_premultiplied(14, 165, 233, (100.0 + pulse * 90.0) as u8));
                            }
                            g_painter.circle_filled(marker_pos, marker_r, Color32::from_rgb(2, 132, 199));
                            g_painter.circle_stroke(marker_pos, marker_r, Stroke::new(1.0, Color32::WHITE));

                            // Axis edge labels
                            g_painter.text(Pos2::new(x_start + 2.0, y_bottom - 1.0), egui::Align2::LEFT_BOTTOM, "20 Hz", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                            g_painter.text(Pos2::new(x_end - 2.0, y_bottom - 1.0), egui::Align2::RIGHT_BOTTOM, "20 kHz", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));

                            // Knee label: dynamic placement to avoid collision with edge text
                            let label_x = x_cut.clamp(x_start + 30.0, x_end - 34.0);
                            let (label_y, align) = if norm_cut > 0.65 {
                                (y_top + 1.0, egui::Align2::RIGHT_TOP)
                            } else {
                                (y_top - 1.0, egui::Align2::CENTER_BOTTOM)
                            };
                            let active_prefix = if lpf_nodes.len() > 1 { format!("#{} ", active_lpf_idx + 1) } else { String::new() };
                            g_painter.text(
                                Pos2::new(label_x, label_y),
                                align,
                                format!("{}Cutoff {}", active_prefix, cutoff_str),
                                egui::FontId::proportional(8.5),
                                Color32::from_rgb(2, 132, 199),
                            );

                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::LowPassFilter);
                                }
                                if has_lpf {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::LowPassFilter);
                                    }
                                }
                            });
                        });
                    });
                    g_lpf.response.context_menu(|ui| {
                        ui.label(egui::RichText::new(format!("🌊 Low-Pass Filter ({})", cutoff_str)).strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::LowPassFilter);
                            ui.close();
                        }
                        if has_lpf {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::LowPassFilter);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_sel) = switch_to_lpf_id {
                        self.selected_node = Some(new_sel);
                    }

                    if let Some(new_val) = lpf_sidebar_drag_change {
                        if let Some(tid) = target_id {
                            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == tid) {
                                n.param_val = new_val;
                                n.sub1 = if new_val >= 1000.0 {
                                    format!("Cutoff {:.1} kHz", new_val / 1000.0)
                                } else {
                                    format!("Cutoff {:.0} Hz", new_val)
                                };
                            }
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(tid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter: Mixer / Summing Bus (Gerbang Paralel)
                    let mixer_node = self.nodes.iter().find(|n| n.node_type == NodeType::Mixer);
                    let has_mixer = mixer_node.is_some();
                    let current_mix = mixer_node.map(|n| n.param_val).unwrap_or(0.5);
                    let mut mixer_sidebar_drag_change: Option<f32> = None;

                    let g_mix = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎚️ Mixer / Summing Bus").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                let mix_txt = format!("{:.0}% / {:.0}%", (1.0 - current_mix) * 100.0, current_mix * 100.0);
                                ui.label(egui::RichText::new(mix_txt).strong().size(10.5).color(Color32::from_rgb(147, 51, 234)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Mixer / Summing Bus",
                                        NodeType::Mixer,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Gerbang penggabung dua jalur sinyal paralel (Dry & Wet) dengan slider crossfader. Mengamankan headroom sinyal dari clipping.").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            // Mini crossfader visualization
                            let (m_rect, m_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), egui::Sense::click_and_drag());
                            let m_painter = ui.painter().with_clip_rect(m_rect);
                            m_painter.rect_filled(m_rect, 4.0, Color32::from_rgb(250, 245, 255));
                            m_painter.rect_stroke(m_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(233, 213, 255)), egui::StrokeKind::Outside);

                            let mx_start = m_rect.min.x + 20.0;
                            let mx_end = m_rect.max.x - 20.0;
                            let mx_w = (mx_end - mx_start).max(10.0);
                            let m_cy = m_rect.center().y;

                            if (m_resp.clicked() || m_resp.dragged()) && has_mixer {
                                if let Some(pos) = m_resp.interact_pointer_pos() {
                                    let norm = ((pos.x - mx_start) / mx_w).clamp(0.0, 1.0);
                                    mixer_sidebar_drag_change = Some(norm);
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    ui.ctx().request_repaint();
                                }
                            }

                            // Labels A & B
                            m_painter.text(Pos2::new(mx_start - 4.0, m_cy), egui::Align2::RIGHT_CENTER, "A", egui::FontId::proportional(9.0), Color32::from_rgb(126, 34, 206));
                            m_painter.text(Pos2::new(mx_end + 4.0, m_cy), egui::Align2::LEFT_CENTER, "B", egui::FontId::proportional(9.0), Color32::from_rgb(126, 34, 206));

                            // Slider track with center notch
                            m_painter.rect_filled(Rect::from_min_size(Pos2::new(mx_start, m_cy - 2.0), Vec2::new(mx_w, 4.0)), 2.0, Color32::from_rgb(243, 232, 255));
                            m_painter.line_segment([Pos2::new(mx_start + mx_w * 0.5, m_cy - 4.0), Pos2::new(mx_start + mx_w * 0.5, m_cy + 4.0)], Stroke::new(1.2, Color32::from_rgb(192, 132, 252)));

                            let thumb_x = mx_start + mx_w * current_mix;
                            m_painter.circle_filled(Pos2::new(thumb_x, m_cy), 4.5, Color32::WHITE);
                            m_painter.circle_stroke(Pos2::new(thumb_x, m_cy), 4.5, Stroke::new(1.6, Color32::from_rgb(147, 51, 234)));

                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::Mixer);
                                }
                                if has_mixer {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::Mixer);
                                    }
                                }
                            });
                        });
                    });
                    g_mix.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🎚️ Mixer / Summing Bus").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::Mixer);
                            ui.close();
                        }
                        if has_mixer {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::Mixer);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_mix) = mixer_sidebar_drag_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::Mixer) {
                            n.param_val = new_mix;
                            let a_pct = (1.0 - new_mix) * 100.0;
                            let b_pct = new_mix * 100.0;
                            n.sub1 = format!("Mix: {:.0}% A • {:.0}% B", a_pct, b_pct);
                            let mid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(mid as u32, 0, new_mix);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Module: Splitter / Y-Branch Bus
                    let has_splitter = self.nodes.iter().any(|n| n.node_type == NodeType::Splitter);
                    let g_split = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔀 Splitter / Y-Branch").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Splitter / Y-Branch",
                                        NodeType::Splitter,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Gerbang pembagi 1 input ke 2 output (Pin A & Pin B) tanpa kehilangan kualitas sinyal (lossless duplication).").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            // Mini Y-branch visualization
                            let (s_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), egui::Sense::hover());
                            let s_painter = ui.painter().with_clip_rect(s_rect);
                            s_painter.rect_filled(s_rect, 4.0, Color32::from_rgb(255, 251, 235));
                            s_painter.rect_stroke(s_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(253, 230, 138)), egui::StrokeKind::Outside);

                            let sx_in = Pos2::new(s_rect.min.x + 35.0, s_rect.center().y);
                            let sx_mid = Pos2::new(s_rect.center().x, s_rect.center().y);
                            let sx_out_a = Pos2::new(s_rect.max.x - 35.0, s_rect.center().y - 6.0);
                            let sx_out_b = Pos2::new(s_rect.max.x - 35.0, s_rect.center().y + 6.0);

                            s_painter.line_segment([sx_in, sx_mid], Stroke::new(1.8, Color32::from_rgb(245, 158, 11)));
                            s_painter.line_segment([sx_mid, sx_out_a], Stroke::new(1.8, Color32::from_rgb(245, 158, 11)));
                            s_painter.line_segment([sx_mid, sx_out_b], Stroke::new(1.8, Color32::from_rgb(245, 158, 11)));
                            s_painter.circle_filled(sx_mid, 2.5, Color32::from_rgb(217, 119, 6));

                            s_painter.text(Pos2::new(sx_in.x - 4.0, sx_in.y), egui::Align2::RIGHT_CENTER, "In", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));
                            s_painter.text(Pos2::new(sx_out_a.x + 4.0, sx_out_a.y), egui::Align2::LEFT_CENTER, "A", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));
                            s_painter.text(Pos2::new(sx_out_b.x + 4.0, sx_out_b.y), egui::Align2::LEFT_CENTER, "B", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));

                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::Splitter);
                                }
                                if has_splitter {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::Splitter);
                                    }
                                }
                            });
                        });
                    });
                    g_split.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔀 Splitter / Y-Branch Bus").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::Splitter);
                            ui.close();
                        }
                        if has_splitter {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::Splitter);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Filter 1: Tube Warmer (Analog Saturation)
                    let tube_node = self.nodes.iter().find(|n| n.node_type == NodeType::TubeWarmer);
                    let has_tube = tube_node.is_some();
                    let current_tube_warmth = tube_node.map(|n| n.param_val).unwrap_or(35.0);
                    let mut tube_change: Option<f32> = None;

                    let g1 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔥 Tube Warmer (Analog)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("Warmth {:.0}%", current_tube_warmth)).strong().size(10.5).color(Color32::from_rgb(249, 115, 22)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Tube Warmer (Analog)",
                                        NodeType::TubeWarmer,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Harmonik triode 12AX7 vintage untuk nada hangat, vokal tebal, dan saturasi analog musikal.").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            if has_tube {
                                let mut val = current_tube_warmth;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("%").show_value(false)).changed() {
                                    tube_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("20% Mild", 20.0), ("35% Triode", 35.0), ("50% Rich", 50.0), ("75% Hot", 75.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            tube_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::TubeWarmer);
                                }
                                if has_tube {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::TubeWarmer);
                                    }
                                }
                            });
                        });
                    });
                    g1.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔥 Tube Warmer (Analog)").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::TubeWarmer);
                            ui.close();
                        }
                        if has_tube {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::TubeWarmer);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = tube_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::TubeWarmer) {
                            n.param_val = new_val;
                            n.sub1 = format!("Warmth {:.0}%", new_val);
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter 2: Spatial Stereo Widener (Active Haas 3D)
                    let widener_node = self.nodes.iter().find(|n| n.node_type == NodeType::StereoWidener);
                    let has_widener = widener_node.is_some();
                    let current_widener_val = widener_node.map(|n| n.param_val).unwrap_or(35.0);
                    let mut widener_change: Option<f32> = None;

                    let g2 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🌐 Stereo 3D Widener").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("+{:.0}% Width", current_widener_val)).strong().size(10.5).color(Color32::from_rgb(99, 102, 241)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Stereo 3D Widener",
                                        NodeType::StereoWidener,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Memperluas dimensi stereo & soundstage 3D spasial secara alami menggunakan pemisahan Mid/Side Haas.").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            if has_widener {
                                let mut val = current_widener_val;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("%").show_value(false)).changed() {
                                    widener_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("+15% Natural", 15.0), ("+35% Haas", 35.0), ("+60% Wide", 60.0), ("+100% 3D", 100.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            widener_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::StereoWidener);
                                }
                                if has_widener {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::StereoWidener);
                                    }
                                }
                            });
                        });
                    });
                    g2.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🌐 Stereo 3D Widener").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::StereoWidener);
                            ui.close();
                        }
                        if has_widener {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::StereoWidener);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = widener_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::StereoWidener) {
                            n.param_val = new_val;
                            n.sub1 = format!("+{:.0}% Soundstage", new_val);
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter 3: Bass Guard Limiter (Coming Soon)
                    let has_bass = self.nodes.iter().any(|n| n.node_type == NodeType::BassGuard);
                    let g3 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🛡️ Bass Guard & Limiter").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("🚧 Coming Soon").size(10.0).strong().color(Color32::from_rgb(217, 119, 6)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Bass Guard & Limiter",
                                        NodeType::BassGuard,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Mencegah distorsi sub-bass berlebih dan melindungi driver headphone (Mesin DSP sedang dalam pengembangan).").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("🚧 Coming Soon").strong().color(Color32::from_rgb(180, 83, 9))).on_hover_text("Mesin DSP filter ini sedang dalam antrean pengembangan").clicked() {
                                    self.add_modular_filter(NodeType::BassGuard);
                                }
                                if has_bass {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::BassGuard);
                                    }
                                }
                            });
                        });
                    });
                    g3.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🛡️ Bass Guard & Limiter 🚧 Coming Soon").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::BassGuard);
                            ui.close();
                        }
                        if has_bass {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::BassGuard);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Filter 4: Volume Attenuator (Low Volume Pad) (Coming Soon)
                    let has_vol = self.nodes.iter().any(|n| n.node_type == NodeType::VolumeAttenuator);
                    let g4 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔉 Volume Attenuator (-6dB)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("🚧 Coming Soon").size(10.0).strong().color(Color32::from_rgb(217, 119, 6)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Volume Attenuator (-6dB)",
                                        NodeType::VolumeAttenuator,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Peredam gain pasif untuk malam hari (Gunakan Stepped Attenuator di suite bawah untuk kontrol aktif).").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("🚧 Coming Soon").strong().color(Color32::from_rgb(180, 83, 9))).on_hover_text("Mesin DSP filter ini sedang dalam antrean pengembangan").clicked() {
                                    self.add_modular_filter(NodeType::VolumeAttenuator);
                                }
                                if has_vol {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::VolumeAttenuator);
                                    }
                                }
                            });
                        });
                    });
                    g4.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔉 Volume Attenuator (-6dB) 🚧 Coming Soon").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::VolumeAttenuator);
                            ui.close();
                        }
                        if has_vol {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::VolumeAttenuator);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Volume Suite Section Header
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔊 MANIPULASI VOLUME & GAIN (6 FILTER)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                    });
                    ui.label(egui::RichText::new("Gain staging presisi, saturasi Class-A, kontur ISO 226, attenuator pasif, opto leveling, dan stereo balance.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                    ui.add_space(6.0);

                    // 1. Pure Volume Booster
                    let vb_node = self.nodes.iter().find(|n| n.node_type == NodeType::VolumeBooster);
                    let has_vb = vb_node.is_some();
                    let current_vb_gain = vb_node.map(|n| n.param_val).unwrap_or(6.0);
                    let mut vb_change: Option<f32> = None;

                    let g_vb = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚡ Pure Volume Booster").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("+{:.1} dB", current_vb_gain)).strong().size(10.5).color(Color32::from_rgb(139, 92, 246)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Volume Booster",
                                        NodeType::VolumeBooster,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Penguat gain bersih 0 s/d +24dB berpresisi f64 tanpa saturasi pewarnaan nada.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_vb {
                                let mut val = current_vb_gain;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=24.0).suffix(" dB").show_value(false)).changed() {
                                    vb_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("+3dB", 3.0), ("+6dB", 6.0), ("+12dB", 12.0), ("+18dB", 18.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            vb_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::VolumeBooster);
                                }
                                if has_vb {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::VolumeBooster);
                                    }
                                }
                            });
                        });
                    });
                    g_vb.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("⚡ Pure Volume Booster").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::VolumeBooster);
                            ui.close();
                        }
                        if has_vb {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::VolumeBooster);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = vb_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::VolumeBooster) {
                            n.param_val = new_val;
                            n.sub1 = format!("+{:.1} dB Gain", new_val);
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 2. Analog Preamp & Trim
                    let preamp_node = self.nodes.iter().find(|n| n.node_type == NodeType::AnalogPreamp);
                    let has_preamp = preamp_node.is_some();
                    let current_preamp_gain = preamp_node.map(|n| n.param_val).unwrap_or(0.0);
                    let mut preamp_change: Option<f32> = None;

                    let g_preamp = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎛️ Analog Preamp & Trim").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:+0.1} dB", current_preamp_gain)).strong().size(10.5).color(Color32::from_rgb(234, 88, 12)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Analog Preamp",
                                        NodeType::AnalogPreamp,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Gain staging studio -24 s/d +12dB dengan saturasi trafo analog Class-A halus.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_preamp {
                                let mut val = current_preamp_gain;
                                if ui.add(egui::Slider::new(&mut val, -24.0..=12.0).suffix(" dB").show_value(false)).changed() {
                                    preamp_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("-6dB Trim", -6.0), ("0dB Unity", 0.0), ("+3dB Warm", 3.0), ("+6dB Sat", 6.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            preamp_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::AnalogPreamp);
                                }
                                if has_preamp {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::AnalogPreamp);
                                    }
                                }
                            });
                        });
                    });
                    g_preamp.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🎛️ Analog Preamp & Trim").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::AnalogPreamp);
                            ui.close();
                        }
                        if has_preamp {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::AnalogPreamp);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = preamp_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::AnalogPreamp) {
                            n.param_val = new_val;
                            n.sub1 = format!("{:+0.1} dB Drive", new_val);
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 3. Dynamic Loudness
                    let loud_node = self.nodes.iter().find(|n| n.node_type == NodeType::DynamicLoudness);
                    let has_loudness = loud_node.is_some();
                    let current_loud_depth = loud_node.map(|n| n.param_val).unwrap_or(50.0);
                    let mut loud_change: Option<f32> = None;

                    let g_loud = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🌙 Dynamic Loudness").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:.0}%", current_loud_depth)).strong().size(10.5).color(Color32::from_rgb(245, 158, 11)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Dynamic Loudness",
                                        NodeType::DynamicLoudness,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Kompensasi kontur equal-loudness ISO 226 untuk bass dan treble penuh di volume rendah.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_loudness {
                                let mut val = current_loud_depth;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("%").show_value(false)).changed() {
                                    loud_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("25% Mild", 25.0), ("50% Night", 50.0), ("75% Deep", 75.0), ("100% Full", 100.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            loud_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::DynamicLoudness);
                                }
                                if has_loudness {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::DynamicLoudness);
                                    }
                                }
                            });
                        });
                    });
                    g_loud.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🌙 Dynamic Loudness (ISO 226)").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::DynamicLoudness);
                            ui.close();
                        }
                        if has_loudness {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::DynamicLoudness);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = loud_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::DynamicLoudness) {
                            n.param_val = new_val;
                            n.sub1 = format!("Loudness {:.0}%", new_val);
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 4. Audiophile Stepped Attenuator
                    let att_node = self.nodes.iter().find(|n| n.node_type == NodeType::SteppedAttenuator);
                    let has_att = att_node.is_some();
                    let current_att_db = att_node.map(|n| n.param_val).unwrap_or(-12.0);
                    let mut att_change: Option<f32> = None;

                    let g_att = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔘 Stepped Attenuator").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:.0} dB", current_att_db)).strong().size(10.5).color(Color32::from_rgb(6, 182, 212)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Stepped Attenuator",
                                        NodeType::SteppedAttenuator,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Resistor ladder 64-step passive attenuator (-60 s/d 0dB) dengan tombol instan DIM -20dB.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_att {
                                let mut val = current_att_db;
                                if ui.add(egui::Slider::new(&mut val, -60.0..=0.0).suffix(" dB").step_by(1.0).show_value(false)).changed() {
                                    att_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("0dB Max", 0.0), ("-12dB", -12.0), ("DIM -20dB", -20.0), ("-36dB", -36.0)] {
                                        let btn = if (current_att_db - p_val).abs() < 0.5 {
                                            ui.add(egui::Button::new(egui::RichText::new(lbl).size(9.0).color(Color32::WHITE)).fill(Color32::from_rgb(6, 182, 212)))
                                        } else {
                                            ui.button(egui::RichText::new(lbl).size(9.0))
                                        };
                                        if btn.clicked() {
                                            att_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::SteppedAttenuator);
                                }
                                if has_att {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::SteppedAttenuator);
                                    }
                                }
                            });
                        });
                    });
                    g_att.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔘 Stepped Attenuator").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::SteppedAttenuator);
                            ui.close();
                        }
                        if has_att {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::SteppedAttenuator);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = att_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::SteppedAttenuator) {
                            n.param_val = new_val;
                            n.sub1 = format!("{:.0} dB", new_val);
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 5. Studio Optical Leveler (LA-2A Opto)
                    let opt_node = self.nodes.iter().find(|n| n.node_type == NodeType::OpticalLeveler);
                    let has_opt = opt_node.is_some();
                    let current_opt_red = opt_node.map(|n| n.param_val).unwrap_or(30.0);
                    let mut opt_change: Option<f32> = None;

                    let g_opt = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🛡️ Studio Optical Leveler").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:.0}%", current_opt_red)).strong().size(10.5).color(Color32::from_rgb(180, 83, 9)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Optical Leveler",
                                        NodeType::OpticalLeveler,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Leveler optocoupler bergaya LA-2A vintage dengan pelepasan 2-tahap halus tanpa pumping.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_opt {
                                let mut val = current_opt_red;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("% Red").show_value(false)).changed() {
                                    opt_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("20% Light", 20.0), ("40% Classic", 40.0), ("65% Heavy", 65.0), ("85% Max", 85.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            opt_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::OpticalLeveler);
                                }
                                if has_opt {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::OpticalLeveler);
                                    }
                                }
                            });
                        });
                    });
                    g_opt.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🛡️ Studio Optical Leveler").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::OpticalLeveler);
                            ui.close();
                        }
                        if has_opt {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::OpticalLeveler);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = opt_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::OpticalLeveler) {
                            n.param_val = new_val;
                            n.sub1 = format!("Peak Red {:.0}%", new_val);
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 6. Dual-Channel Stereo Balance & Trim
                    let bal_node = self.nodes.iter().find(|n| n.node_type == NodeType::StereoBalance);
                    let has_bal = bal_node.is_some();
                    let current_bal = bal_node.map(|n| n.param_val).unwrap_or(0.0);
                    let mut bal_change: Option<f32> = None;

                    let g_bal = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚖️ Stereo Balance & Trim").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                let bal_lbl = if current_bal.abs() < 1.0 {
                                    "Center".to_string()
                                } else if current_bal < 0.0 {
                                    format!("L {:.0}%", current_bal.abs())
                                } else {
                                    format!("R {:.0}%", current_bal)
                                };
                                ui.label(egui::RichText::new(bal_lbl).strong().size(10.5).color(Color32::from_rgb(79, 70, 229)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Stereo Balance",
                                        NodeType::StereoBalance,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Panning kurva daya konstan sinus/kosinus dari -100% (Kiri) ke +100% (Kanan).").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_bal {
                                let mut val = current_bal;
                                if ui.add(egui::Slider::new(&mut val, -100.0..=100.0).suffix("%").show_value(false)).changed() {
                                    bal_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("◀ L 50%", -50.0), ("● Center", 0.0), ("R 50% ▶", 50.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            bal_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::StereoBalance);
                                }
                                if has_bal {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::StereoBalance);
                                    }
                                }
                            });
                        });
                    });
                    g_bal.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("⚖️ Stereo Balance & Trim").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::StereoBalance);
                            ui.close();
                        }
                        if has_bal {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::StereoBalance);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = bal_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::StereoBalance) {
                            n.param_val = new_val;
                            if new_val.abs() < 1.0 {
                                n.sub1 = "Center 0.0 dB".to_string();
                            } else if new_val < 0.0 {
                                n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 + new_val.abs() * 0.5, 50.0 - new_val.abs() * 0.5);
                            } else {
                                n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 - new_val * 0.5, 50.0 + new_val * 0.5);
                            }
                            let nid = n.id;
                            if self.chain_mode != ChainMode::DspActive {
                                self.set_chain_mode(ChainMode::DspActive);
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter 5: Equalizer Presets Card (Matches Filter 1-4 anatomy perfectly)
                    let has_any_eq_node = self.nodes.iter().any(|n| matches!(n.node_type, NodeType::SavedEqPreset(_)));
                    let is_open = self.show_saved_eq_palette_popup;
                    let sidebar_min_x = ui.max_rect().min.x;

                    let mut btn_clicked = false;
                    let g5 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎛️ Equalizer Presets").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("🚧 Coming Soon").size(10.0).strong().color(Color32::from_rgb(217, 119, 6)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Equalizer Presets",
                                        NodeType::SavedEqPreset("Default".to_string()),
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Koleksi kurva frekuensi siap pakai untuk vokal, bass, dan akustik (Coming Soon).").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                let btn_label = if is_open { "🎛️ Tutup Preset" } else { "➕ Tambah ke Chain" };
                                let btn_resp = ui.button(egui::RichText::new(btn_label).strong());
                                if btn_resp.clicked() {
                                    btn_clicked = true;
                                    self.show_saved_eq_palette_popup = !self.show_saved_eq_palette_popup;
                                    let click_y = ui.input(|i| i.pointer.interact_pos()).map(|p| p.y).unwrap_or(btn_resp.rect.center().y);
                                    self.saved_eq_popup_anchor = Some(Pos2::new(sidebar_min_x, click_y));
                                }
                                if has_any_eq_node {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus semua modul EQ Preset dari canvas").clicked() {
                                        delete_all_saved_eq_nodes = true;
                                    }
                                }
                            });
                        });
                    });

                    let card_resp = ui.interact(g5.response.rect, ui.scope_id().with("card_eq_presets_click"), egui::Sense::click());
                    if card_resp.clicked() && !btn_clicked {
                        self.show_saved_eq_palette_popup = !self.show_saved_eq_palette_popup;
                        let click_y = ui.input(|i| i.pointer.interact_pos()).map(|p| p.y).unwrap_or(g5.response.rect.center().y);
                        self.saved_eq_popup_anchor = Some(Pos2::new(sidebar_min_x, click_y));
                    }

                    g5.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🎛️ Equalizer Presets").strong());
                        ui.separator();
                        if ui.button("🎛️ Buka / Tutup Koleksi Preset").clicked() {
                            self.show_saved_eq_palette_popup = !self.show_saved_eq_palette_popup;
                            let click_y = ui.input(|i| i.pointer.interact_pos()).map(|p| p.y).unwrap_or(g5.response.rect.center().y);
                            self.saved_eq_popup_anchor = Some(Pos2::new(sidebar_min_x, click_y));
                            ui.close();
                        }
                        if has_any_eq_node {
                            if ui.button("🗑️ Hapus Semua EQ Preset dari Canvas").clicked() {
                                delete_all_saved_eq_nodes = true;
                                ui.close();
                            }
                        }
                    });
                });
            });
        });

        // Execute batch filter deletion
        if let Some(ref nt) = delete_filter_type {
            let ids: Vec<usize> = self.nodes.iter().filter(|n| &n.node_type == nt && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            if let Some(sel) = self.selected_node {
                if ids.contains(&sel) {
                    self.selected_node = None;
                }
            }
        }

        if delete_all_saved_eq_nodes {
            let ids: Vec<usize> = self.nodes.iter().filter(|n| matches!(n.node_type, NodeType::SavedEqPreset(_)) && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            if let Some(sel) = self.selected_node {
                if ids.contains(&sel) {
                    self.selected_node = None;
                }
            }
        }

        if delete_filter_type.is_some() || delete_all_saved_eq_nodes {
            self.invalidate_chain_cache();
            self.sync_signal_chain_to_engine();
        }

        self.render_saved_eq_floating_popup(ui);
    }

    /// Floating popup widget displaying all saved equalizers.
    /// Positioned intelligently relative to cursor / anchor edge so it never overlaps or gets clipped.
    pub fn render_saved_eq_floating_popup(&mut self, ui: &mut egui::Ui) {
        if !self.show_saved_eq_palette_popup {
            return;
        }

        let screen_rect = ui.ctx().input(|i| i.raw.screen_rect).unwrap_or_else(|| ui.clip_rect());
        let popup_w = 400.0;
        let popup_h = 440.0;

        // Position calculation:
        // Positioned cleanly to the LEFT of the sidebar palette with a 12px gap,
        // so it NEVER overlaps or covers the sidebar, buttons, or any filter card!
        let (sidebar_left, anchor_y) = if let Some(a) = self.saved_eq_popup_anchor {
            (a.x, a.y)
        } else {
            (screen_rect.max.x - 300.0, screen_rect.center().y)
        };

        let mut x = sidebar_left - popup_w - 12.0;
        if x < screen_rect.min.x + 12.0 {
            x = screen_rect.min.x + 12.0;
        }

        // Align vertically around anchor_y, clamped to stay safely inside screen bounds
        let min_y = screen_rect.min.y + 44.0;
        let max_y = (screen_rect.max.y - popup_h - 16.0).max(min_y);
        let ideal_y = anchor_y - popup_h * 0.5;
        let y = ideal_y.clamp(min_y, max_y);

        let popup_pos = Pos2::new(x, y);

        let mut close_popup = false;
        let mut open_preset: Option<String> = None;
        let mut add_preset_to_canvas: Option<String> = None;
        let mut delete_preset_name: Option<String> = None;

        let popup_id = ui.scope_id().with("saved_eq_floating_popup_area");
        let area_resp = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.5, Color32::from_rgb(147, 51, 234)))
                    .corner_radius(12.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width(popup_w - 28.0);
                        ui.set_max_height(popup_h - 28.0);

                        // Header Bar
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🎛️").size(15.0));
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("Koleksi Preset Equalizer")
                                        .strong()
                                        .size(13.5)
                                        .color(Color32::from_rgb(15, 23, 42))
                                );
                                ui.label(
                                    egui::RichText::new("Pilih preset untuk dipasang ke canvas patchbay atau diedit di EQ.")
                                        .size(10.5)
                                        .color(Color32::from_rgb(100, 116, 139))
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::new(
                                    egui::RichText::new("✖").strong().size(11.0).color(Color32::from_rgb(100, 116, 139))
                                )
                                .fill(Color32::from_rgb(241, 245, 249))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                .corner_radius(6.0);

                                if ui.add_sized([24.0, 24.0], close_btn).on_hover_text("Tutup jendela preset").clicked() {
                                    close_popup = true;
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        egui::ScrollArea::vertical()
                            .max_height(popup_h - 110.0)
                            .show(ui, |ui| {
                                for preset in &self.eq_presets {
                                    let is_on_canvas = self.nodes.iter().any(|n| n.node_type == NodeType::SavedEqPreset(preset.name.clone()));
                                    let chain_eval = self.evaluate_signal_chain();
                                    let is_flowing_in_chain = self.chain_mode == ChainMode::DspActive && is_on_canvas && self.nodes.iter().any(|n| {
                                        n.node_type == NodeType::SavedEqPreset(preset.name.clone()) && chain_eval.node_status(n.id) == SignalNodeStatus::Flowing
                                    });

                                    let card_bg = if is_flowing_in_chain {
                                        Color32::from_rgb(240, 253, 244)
                                    } else {
                                        Color32::from_rgb(255, 255, 255)
                                    };

                                    let border_stroke = if is_flowing_in_chain {
                                        Stroke::new(1.2, Color32::from_rgb(34, 197, 94))
                                    } else {
                                        Stroke::new(1.0, Color32::from_rgb(226, 232, 240))
                                    };

                                    egui::Frame::new()
                                        .fill(card_bg)
                                        .stroke(border_stroke)
                                        .corner_radius(10.0)
                                        .inner_margin(egui::Margin::symmetric(12, 10))
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                // ROW 1: Preset Name & Badges & Delete Button
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(&preset.name)
                                                            .strong()
                                                            .size(12.5)
                                                            .color(Color32::from_rgb(15, 23, 42))
                                                    );

                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        // Delete Button: only for custom user presets (factory presets are protected)
                                                        let is_factory = crate::models::ParametricEqPreset::default_presets().iter().any(|dp| dp.name == preset.name);
                                                        if !is_factory && self.eq_presets.len() > 1 {
                                                            let del_btn = egui::Button::new(
                                                                egui::RichText::new("🗑").size(10.5).color(Color32::from_rgb(239, 68, 68))
                                                            )
                                                            .fill(Color32::from_rgb(254, 242, 242))
                                                            .stroke(Stroke::new(0.8, Color32::from_rgb(254, 202, 202)))
                                                            .corner_radius(5.0);

                                                            if ui.add_sized([22.0, 20.0], del_btn).on_hover_text("Hapus preset kustom ini").clicked() {
                                                                delete_preset_name = Some(preset.name.clone());
                                                            }
                                                        }

                                                        // Badges: ONLY displayed if actually on canvas
                                                        if is_flowing_in_chain {
                                                            egui::Frame::new()
                                                                .fill(Color32::from_rgb(220, 252, 231))
                                                                .corner_radius(4.0)
                                                                .inner_margin(egui::Margin::symmetric(6, 2))
                                                                .show(ui, |ui| {
                                                                    ui.label(egui::RichText::new("⚡ Aktif di Chain").size(9.5).strong().color(Color32::from_rgb(22, 163, 74)));
                                                                });
                                                        } else if is_on_canvas {
                                                            egui::Frame::new()
                                                                .fill(Color32::from_rgb(243, 232, 255))
                                                                .corner_radius(4.0)
                                                                .inner_margin(egui::Margin::symmetric(6, 2))
                                                                .show(ui, |ui| {
                                                                    ui.label(egui::RichText::new("✓ Di Canvas").size(9.5).strong().color(Color32::from_rgb(147, 51, 234)));
                                                                });
                                                        }
                                                    });
                                                });

                                                ui.add_space(3.0);

                                                // ROW 2: Subtitle Info (Band Count, Preamp, Range)
                                                let preamp_str = if preset.preamp_mode == crate::models::PreampMode::Auto {
                                                    "Auto Preamp".to_string()
                                                } else {
                                                    format!("{:+.1} dB", preset.manual_preamp_db)
                                                };
                                                ui.label(
                                                    egui::RichText::new(format!("{} Band Parametrik  •  {}  •  20 Hz – 20 kHz", preset.bands.len(), preamp_str))
                                                        .size(10.5)
                                                        .color(Color32::from_rgb(100, 116, 139))
                                                );

                                                ui.add_space(8.0);

                                                // ROW 3: Dedicated Action Buttons (Full breathing room, never overlap!)
                                                ui.horizontal(|ui| {
                                                    // Button 1: Buka & Edit di Tab Equalizer
                                                    let btn_open = egui::Button::new(
                                                        egui::RichText::new("🎛️ Buka di EQ").size(11.0).strong().color(Color32::from_rgb(30, 41, 59))
                                                    )
                                                    .fill(Color32::from_rgb(241, 245, 249))
                                                    .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                                    .corner_radius(6.0);

                                                    if ui.add_sized([135.0, 24.0], btn_open)
                                                        .on_hover_text("Buka preset ini di Tab Equalizer untuk melihat kurva respon frekuensi dan mengedit band")
                                                        .clicked()
                                                    {
                                                        open_preset = Some(preset.name.clone());
                                                    }

                                                    ui.add_space(6.0);

                                                    // Button 2: Pasang ke Canvas / Sudah di Canvas
                                                    if !is_on_canvas {
                                                        let btn_canvas = egui::Button::new(
                                                            egui::RichText::new("➕ Pasang ke Canvas").size(11.0).strong().color(Color32::from_rgb(14, 165, 233))
                                                        )
                                                        .fill(Color32::from_rgb(240, 249, 255))
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(186, 230, 253)))
                                                        .corner_radius(6.0);

                                                        if ui.add_sized([145.0, 24.0], btn_canvas)
                                                            .on_hover_text("Pasang modul filter equalizer ini ke dalam diagram patchbay")
                                                            .clicked()
                                                        {
                                                            add_preset_to_canvas = Some(preset.name.clone());
                                                        }
                                                    } else {
                                                        let btn_canvas_on = egui::Button::new(
                                                            egui::RichText::new("✓ Terpasang di Canvas").size(11.0).color(Color32::from_rgb(147, 51, 234))
                                                        )
                                                        .fill(Color32::from_rgb(250, 245, 255))
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(233, 213, 255)))
                                                        .corner_radius(6.0);

                                                        ui.add_sized([145.0, 24.0], btn_canvas_on)
                                                            .on_hover_text("Modul preset equalizer ini sudah aktif di kanvas Signal Chain Studio");
                                                    }
                                                });
                                            });
                                        });

                                    ui.add_space(6.0);
                                }
                            });
                    });
            });

        // Close on clicking outside the popup frame
        if ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let rect = area_resp.response.rect;
                let is_inside_popup = rect.contains(pos);
                // Also ignore if clicking near the sidebar trigger button so toggle works smoothly
                let is_near_trigger = self.saved_eq_popup_anchor.map_or(false, |a| {
                    (pos.x >= a.x && pos.x <= a.x + 350.0) && (pos.y - a.y).abs() <= 24.0
                });
                if !is_inside_popup && !is_near_trigger {
                    close_popup = true;
                }
            }
        }

        if close_popup {
            self.show_saved_eq_palette_popup = false;
        }

        if let Some(name) = add_preset_to_canvas {
            self.add_modular_filter(NodeType::SavedEqPreset(name));
        }

        if let Some(name) = open_preset {
            if let Some(p) = self.eq_presets.iter().find(|p| p.name == name).cloned() {
                self.active_eq = p;
                self.eq_is_dirty = false;
                self.sync_parametric_to_legacy_eq();
                self.current_tab = crate::models::AppTab::Equalizer;
                self.show_saved_eq_palette_popup = false;
            }
        }

        if let Some(name) = delete_preset_name {
            self.eq_presets.retain(|p| p.name != name);
            let nt = NodeType::SavedEqPreset(name);
            let ids: Vec<usize> = self.nodes.iter().filter(|n| n.node_type == nt && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            self.save_library_state();
        }
    }

    /// Beginner visual tutorial module positioned in top-right of the Signal Chain Studio canvas
    pub fn render_patchbay_tutorial_card(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        if !self.show_patchbay_tutorial {
            return;
        }

        let padding = 12.0;

        // Full Tutorial Card
        let card_w = 360.0f32.min(canvas_rect.width() - 24.0);
        let card_h = 410.0f32.min(canvas_rect.height() - 24.0);
        let card_pos = Pos2::new(
            canvas_rect.max.x - card_w - padding,
            canvas_rect.min.y + padding,
        );

        let area_id = ui.scope_id().with("patchbay_tutorial_card_area");
        egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(card_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.5, Color32::from_rgb(147, 51, 234)))
                    .corner_radius(12.0)
                    .inner_margin(12.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 0,
                        color: Color32::from_black_alpha(28),
                    })
                    .show(ui, |ui| {
                        ui.set_width(card_w - 24.0);
                        ui.set_max_height(card_h - 24.0);

                        // Header Bar
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📖").size(15.0));
                            ui.label(
                                egui::RichText::new("PANDUAN SIGNAL CHAIN")
                                    .strong()
                                    .size(12.5)
                                    .color(Color32::from_rgb(15, 23, 42)),
                            );

                            let step_num = (self.patchbay_tutorial_step % 5) + 1;
                            ui.label(
                                egui::RichText::new(format!("Langkah {}/5", step_num))
                                    .size(10.5)
                                    .strong()
                                    .color(Color32::from_rgb(147, 51, 234)),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("✕").size(12.0).strong().color(Color32::from_rgb(148, 163, 184)))
                                    .on_hover_text("Tutup tutorial")
                                    .clicked()
                                {
                                    self.show_patchbay_tutorial = false;
                                }
                            });
                        });

                        ui.add_space(6.0);

                        // Step Selection Tabs
                        ui.horizontal(|ui| {
                            let steps = [
                                "1. Aliran",
                                "2. Kabel",
                                "3. Pasang Filter",
                                "4. Mute Aman",
                                "5. Gerbang Mixer",
                            ];
                            for (idx, name) in steps.iter().enumerate() {
                                let is_active = self.patchbay_tutorial_step == idx;
                                let btn = if is_active {
                                    ui.button(egui::RichText::new(*name).size(10.0).strong().color(Color32::WHITE))
                                } else {
                                    ui.button(egui::RichText::new(*name).size(10.0).color(Color32::from_rgb(71, 85, 105)))
                                };
                                if btn.clicked() {
                                    self.patchbay_tutorial_step = idx;
                                }
                            }
                        });

                        ui.add_space(6.0);

                        // Visual Diagram / Illustration Box (Vector graphics rendered with painter)
                        let diag_h = 115.0;
                        let (diag_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), diag_h), egui::Sense::hover());
                        let p = ui.painter().with_clip_rect(diag_rect);
                        p.rect_filled(diag_rect, 8.0, Color32::from_rgb(248, 250, 252));
                        p.rect_stroke(diag_rect, 8.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

                        match self.patchbay_tutorial_step % 5 {
                            0 => {
                                // Step 1: Konsep Aliran Audio (Source -> Filter -> Speaker)
                                let cy = diag_rect.center().y;
                                let n_w = 78.0;
                                let n_h = 36.0;

                                let r1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 10.0, cy - 18.0), Vec2::new(n_w, n_h));
                                let r2 = Rect::from_min_size(Pos2::new(diag_rect.center().x - n_w * 0.5, cy - 18.0), Vec2::new(n_w, n_h));
                                let r3 = Rect::from_min_size(Pos2::new(diag_rect.max.x - n_w - 10.0, cy - 18.0), Vec2::new(n_w, n_h));

                                // Box 1: File Lagu
                                p.rect_filled(r1, 6.0, Color32::from_rgb(238, 242, 255));
                                p.rect_stroke(r1, 6.0, Stroke::new(1.2, Color32::from_rgb(99, 102, 241)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(r1.center().x, r1.min.y + 11.0), egui::Align2::CENTER_CENTER, "🎵 File Lagu", egui::FontId::proportional(10.0), Color32::from_rgb(30, 41, 59));
                                p.text(Pos2::new(r1.center().x, r1.min.y + 24.0), egui::Align2::CENTER_CENTER, "(Sumber)", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

                                // Box 2: Filter Efek
                                p.rect_filled(r2, 6.0, Color32::from_rgb(245, 243, 255));
                                p.rect_stroke(r2, 6.0, Stroke::new(1.2, Color32::from_rgb(147, 51, 234)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(r2.center().x, r2.min.y + 11.0), egui::Align2::CENTER_CENTER, "🎛️ Filter Efek", egui::FontId::proportional(10.0), Color32::from_rgb(30, 41, 59));
                                p.text(Pos2::new(r2.center().x, r2.min.y + 24.0), egui::Align2::CENTER_CENTER, "(Low-Pass dll)", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

                                // Box 3: Speaker DAC
                                p.rect_filled(r3, 6.0, Color32::from_rgb(236, 253, 245));
                                p.rect_stroke(r3, 6.0, Stroke::new(1.2, Color32::from_rgb(16, 185, 129)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(r3.center().x, r3.min.y + 11.0), egui::Align2::CENTER_CENTER, "🔊 Speaker", egui::FontId::proportional(10.0), Color32::from_rgb(30, 41, 59));
                                p.text(Pos2::new(r3.center().x, r3.min.y + 24.0), egui::Align2::CENTER_CENTER, "(Output DAC)", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

                                // Connecting green flowing cables
                                let p1 = Pos2::new(r1.max.x, cy);
                                let p2 = Pos2::new(r2.min.x, cy);
                                let p3 = Pos2::new(r2.max.x, cy);
                                let p4 = Pos2::new(r3.min.x, cy);

                                p.line_segment([p1, p2], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));
                                p.line_segment([p3, p4], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));

                                // Flow direction dots / pulse
                                p.circle_filled(Pos2::new((p1.x + p2.x) * 0.5, cy), 3.0, Color32::from_rgb(52, 211, 153));
                                p.circle_filled(Pos2::new((p3.x + p4.x) * 0.5, cy), 3.0, Color32::from_rgb(52, 211, 153));

                                p.text(
                                    Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                    egui::Align2::CENTER_CENTER,
                                    "▶ Sinyal mengalir dari Kiri ke Kanan tanpa delay",
                                    egui::FontId::proportional(9.0),
                                    Color32::from_rgb(100, 116, 139),
                                );
                            }
                            1 => {
                                // Step 2: Sambung & Putus Kabel
                                let cy = diag_rect.center().y - 8.0;
                                let n_w = 90.0;
                                let n_h = 42.0;

                                let r1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 20.0, cy - 21.0), Vec2::new(n_w, n_h));
                                let r2 = Rect::from_min_size(Pos2::new(diag_rect.max.x - n_w - 20.0, cy - 21.0), Vec2::new(n_w, n_h));

                                p.rect_filled(r1, 6.0, Color32::WHITE);
                                p.rect_stroke(r1, 6.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(r1.center().x - 6.0, r1.center().y), egui::Align2::CENTER_CENTER, "Modul A", egui::FontId::proportional(10.5), Color32::from_rgb(30, 41, 59));

                                p.rect_filled(r2, 6.0, Color32::WHITE);
                                p.rect_stroke(r2, 6.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(r2.center().x + 6.0, r2.center().y), egui::Align2::CENTER_CENTER, "Modul B", egui::FontId::proportional(10.5), Color32::from_rgb(30, 41, 59));

                                let pin_out = Pos2::new(r1.max.x, cy);
                                let pin_in = Pos2::new(r2.min.x, cy);

                                // Pin circles
                                p.circle_filled(pin_out, 5.0, Color32::from_rgb(99, 102, 241));
                                p.circle_stroke(pin_out, 8.0, Stroke::new(1.2, Color32::from_rgb(165, 180, 252)));

                                p.circle_filled(pin_in, 5.0, Color32::from_rgb(99, 102, 241));
                                p.circle_stroke(pin_in, 8.0, Stroke::new(1.2, Color32::from_rgb(165, 180, 252)));

                                // Bezier cable
                                let cp1 = Pos2::new(pin_out.x + 35.0, pin_out.y + 20.0);
                                let cp2 = Pos2::new(pin_in.x - 35.0, pin_in.y + 20.0);
                                let shape = egui::epaint::CubicBezierShape::from_points_stroke(
                                    [pin_out, cp1, cp2, pin_in],
                                    false,
                                    Color32::TRANSPARENT,
                                    Stroke::new(2.5, Color32::from_rgb(99, 102, 241)),
                                );
                                p.add(shape);

                                // Mouse pointer drag hint
                                let mid_p = Pos2::new((pin_out.x + pin_in.x) * 0.5, cy + 15.0);
                                p.text(Pos2::new(pin_out.x - 4.0, pin_out.y - 14.0), egui::Align2::CENTER_CENTER, "Pin Output", egui::FontId::proportional(8.5), Color32::from_rgb(99, 102, 241));
                                p.text(Pos2::new(pin_in.x + 4.0, pin_in.y - 14.0), egui::Align2::CENTER_CENTER, "Pin Input", egui::FontId::proportional(8.5), Color32::from_rgb(99, 102, 241));
                                p.text(Pos2::new(mid_p.x, diag_rect.max.y - 12.0), egui::Align2::CENTER_CENTER, "👆 Tarik garis dari kanan ke kiri • Klik kabel untuk memutus", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));
                            }
                            2 => {
                                // Step 3: Memasang Low-Pass Filter
                                let cy = diag_rect.center().y - 10.0;
                                let r_lpf = Rect::from_min_size(Pos2::new(diag_rect.center().x - 70.0, cy - 20.0), Vec2::new(140.0, 44.0));

                                p.rect_filled(r_lpf, 6.0, Color32::from_rgb(240, 249, 255));
                                p.rect_stroke(r_lpf, 6.0, Stroke::new(1.5, Color32::from_rgb(14, 165, 233)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(r_lpf.center().x, r_lpf.min.y + 12.0), egui::Align2::CENTER_CENTER, "🌊 LOW-PASS FILTER", egui::FontId::proportional(10.0), Color32::from_rgb(14, 116, 144));
                                p.text(Pos2::new(r_lpf.center().x, r_lpf.min.y + 26.0), egui::Align2::CENTER_CENTER, "Cutoff 800 Hz (Muffled)", egui::FontId::proportional(8.5), Color32::from_rgb(2, 132, 199));

                                // Mini curve line inside card
                                let c_x1 = r_lpf.min.x + 12.0;
                                let c_x2 = r_lpf.center().x;
                                let c_x3 = r_lpf.max.x - 12.0;
                                let c_y1 = r_lpf.max.y - 6.0;
                                let c_y2 = r_lpf.max.y - 12.0;
                                p.line_segment([Pos2::new(c_x1, c_y2), Pos2::new(c_x2, c_y2)], Stroke::new(1.8, Color32::from_rgb(2, 132, 199)));
                                p.line_segment([Pos2::new(c_x2, c_y2), Pos2::new(c_x3, c_y1)], Stroke::new(1.8, Color32::from_rgb(2, 132, 199)));

                                p.text(
                                    Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                    egui::Align2::CENTER_CENTER,
                                    "✨ Tambah dari palette kanan ➔ Selipkan di rantai kabel",
                                    egui::FontId::proportional(9.0),
                                    Color32::from_rgb(14, 116, 144),
                                );
                            }
                            3 => {
                                // Step 4: Proteksi Mute Saat Putus
                                let half_w = (diag_rect.width() - 32.0) * 0.5;
                                let b1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 10.0, diag_rect.min.y + 10.0), Vec2::new(half_w, diag_h - 20.0));
                                let b2 = Rect::from_min_size(Pos2::new(diag_rect.center().x + 6.0, diag_rect.min.y + 10.0), Vec2::new(half_w, diag_h - 20.0));

                                // Box 1: Tersambung Normal
                                p.rect_filled(b1, 6.0, Color32::from_rgb(236, 253, 245));
                                p.rect_stroke(b1, 6.0, Stroke::new(1.0, Color32::from_rgb(167, 243, 208)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(b1.center().x, b1.min.y + 14.0), egui::Align2::CENTER_CENTER, "✓ Kabel Utuh", egui::FontId::proportional(10.0), Color32::from_rgb(6, 95, 70));
                                p.line_segment([Pos2::new(b1.min.x + 12.0, b1.center().y), Pos2::new(b1.max.x - 12.0, b1.center().y)], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));
                                p.text(Pos2::new(b1.center().x, b1.max.y - 14.0), egui::Align2::CENTER_CENTER, "🔊 Suara Normal", egui::FontId::proportional(9.0), Color32::from_rgb(5, 150, 105));

                                // Box 2: Terputus / Mute
                                p.rect_filled(b2, 6.0, Color32::from_rgb(254, 242, 242));
                                p.rect_stroke(b2, 6.0, Stroke::new(1.0, Color32::from_rgb(254, 202, 202)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(b2.center().x, b2.min.y + 14.0), egui::Align2::CENTER_CENTER, "⚠️ Kabel Putus", egui::FontId::proportional(10.0), Color32::from_rgb(153, 27, 27));
                                // Dashed line
                                let cx = b2.center().x;
                                let cy = b2.center().y;
                                p.line_segment([Pos2::new(b2.min.x + 12.0, cy), Pos2::new(cx - 8.0, cy)], Stroke::new(2.0, Color32::from_rgb(239, 68, 68)));
                                p.text(Pos2::new(cx, cy), egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(11.0), Color32::from_rgb(239, 68, 68));
                                p.line_segment([Pos2::new(cx + 8.0, cy), Pos2::new(b2.max.x - 12.0, cy)], Stroke::new(2.0, Color32::from_rgb(239, 68, 68)));
                                p.text(Pos2::new(b2.center().x, b2.max.y - 14.0), egui::Align2::CENTER_CENTER, "🔇 Hening Otomatis", egui::FontId::proportional(9.0), Color32::from_rgb(220, 38, 38));
                            }
                            _ => {
                                // Step 5: Percabangan & Gerbang Mixer
                                let cy = diag_rect.center().y - 4.0;
                                let n_w = 64.0;
                                let n_h = 28.0;

                                // Modul A (Atas - Dry) & Modul B (Bawah - Wet/Filter)
                                let r_a = Rect::from_min_size(Pos2::new(diag_rect.min.x + 10.0, cy - 36.0), Vec2::new(n_w, n_h));
                                let r_b = Rect::from_min_size(Pos2::new(diag_rect.min.x + 10.0, cy + 8.0), Vec2::new(n_w, n_h));

                                p.rect_filled(r_a, 5.0, Color32::from_rgb(238, 242, 255));
                                p.rect_stroke(r_a, 5.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                                p.text(r_a.center(), egui::Align2::CENTER_CENTER, "Jalur A (Dry)", egui::FontId::proportional(8.0), Color32::from_rgb(67, 56, 202));

                                p.rect_filled(r_b, 5.0, Color32::from_rgb(240, 249, 255));
                                p.rect_stroke(r_b, 5.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);
                                p.text(r_b.center(), egui::Align2::CENTER_CENTER, "Jalur B (Filter)", egui::FontId::proportional(8.0), Color32::from_rgb(2, 132, 199));

                                // Mixer Card (Tengah)
                                let r_mix = Rect::from_min_size(Pos2::new(diag_rect.center().x - 48.0, cy - 28.0), Vec2::new(96.0, 56.0));
                                p.rect_filled(r_mix, 6.0, Color32::from_rgb(250, 245, 255));
                                p.rect_stroke(r_mix, 6.0, Stroke::new(1.5, Color32::from_rgb(147, 51, 234)), egui::StrokeKind::Outside);
                                p.text(Pos2::new(r_mix.center().x, r_mix.min.y + 11.0), egui::Align2::CENTER_CENTER, "🎚️ MIXER (Summing)", egui::FontId::proportional(8.5), Color32::from_rgb(126, 34, 206));

                                // Input pin markers on Mixer
                                let pin_a_pos = Pos2::new(r_mix.min.x, r_mix.center().y - 10.0);
                                let pin_b_pos = Pos2::new(r_mix.min.x, r_mix.center().y + 10.0);
                                p.circle_filled(pin_a_pos, 4.0, Color32::from_rgb(147, 51, 234));
                                p.circle_filled(pin_b_pos, 4.0, Color32::from_rgb(147, 51, 234));
                                p.text(Pos2::new(pin_a_pos.x + 7.0, pin_a_pos.y), egui::Align2::LEFT_CENTER, "Pin A", egui::FontId::proportional(7.5), Color32::from_rgb(126, 34, 206));
                                p.text(Pos2::new(pin_b_pos.x + 7.0, pin_b_pos.y), egui::Align2::LEFT_CENTER, "Pin B", egui::FontId::proportional(7.5), Color32::from_rgb(126, 34, 206));

                                // Mini fader bar in Mixer
                                let fader_y = r_mix.max.y - 10.0;
                                p.line_segment([Pos2::new(r_mix.min.x + 18.0, fader_y), Pos2::new(r_mix.max.x - 18.0, fader_y)], Stroke::new(2.0, Color32::from_rgb(233, 213, 255)));
                                p.circle_filled(Pos2::new(r_mix.center().x, fader_y), 3.5, Color32::from_rgb(147, 51, 234));

                                // Output DAC (Kanan)
                                let r_dac = Rect::from_min_size(Pos2::new(diag_rect.max.x - n_w - 10.0, cy - 14.0), Vec2::new(n_w, n_h));
                                p.rect_filled(r_dac, 5.0, Color32::from_rgb(236, 253, 245));
                                p.rect_stroke(r_dac, 5.0, Stroke::new(1.0, Color32::from_rgb(167, 243, 208)), egui::StrokeKind::Outside);
                                p.text(r_dac.center(), egui::Align2::CENTER_CENTER, "Output DAC", egui::FontId::proportional(8.5), Color32::from_rgb(6, 95, 70));

                                // Cables
                                p.line_segment([Pos2::new(r_a.max.x, r_a.center().y), pin_a_pos], Stroke::new(1.8, Color32::from_rgb(147, 51, 234)));
                                p.line_segment([Pos2::new(r_b.max.x, r_b.center().y), pin_b_pos], Stroke::new(1.8, Color32::from_rgb(147, 51, 234)));
                                p.line_segment([Pos2::new(r_mix.max.x, r_mix.center().y), Pos2::new(r_dac.min.x, r_dac.center().y)], Stroke::new(2.2, Color32::from_rgb(16, 185, 129)));

                                p.text(
                                    Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                    egui::Align2::CENTER_CENTER,
                                    "✨ 1 Pin biasa = 1 Kabel. Untuk 2 cabang paralel, satukan lewat Gerbang Mixer!",
                                    egui::FontId::proportional(8.5),
                                    Color32::from_rgb(126, 34, 206),
                                );
                            }
                        }

                        ui.add_space(8.0);

                        // Explanatory Text Content
                        match self.patchbay_tutorial_step % 5 {
                            0 => {
                                ui.label(egui::RichText::new("Bagaimana Audio Mengalir?").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Audio mengalir seperti kabel fisik panggung dari FILE SOURCE (kiri) menuju OUTPUT DAC (kanan).\n• Setiap modul di tengahnya menyaring atau memproses suara secara real-time.\n• Anda bisa menyisipkan efek sebanyak apa pun sesuai selera.").size(10.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            1 => {
                                ui.label(egui::RichText::new("Menyambung & Memutus Kabel").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Menyambung: Klik & tahan bulatan pin kanan (Output) sebuah modul, lalu tarik kabelnya ke pin kiri (Input) modul tujuan.\n• Memutus: Cukup klik langsung pada garis kabel yang ingin dilepas.\n• Geser Kanvas: Tahan klik kanan atau roda mouse lalu geser.").size(10.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            2 => {
                                ui.label(egui::RichText::new("Uji Coba: Low-Pass Filter").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Ingin mendengar perbedaannya langsung? Klik '➕ Tambah ke Chain' pada kartu Low-Pass Filter di sidebar kanan.\n• Sambungkan kabel dari RAM Buffer ke Low-Pass Filter, lalu dari filter ke Output Engine.\n• Suara vokal & cymbal langsung teredam tebal (efek underwater)!").size(10.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            3 => {
                                ui.label(egui::RichText::new("Keamanan Audio (Auto-Mute)").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Jika kabel terputus di tengah jalan, LunulaPlayer otomatis membisukan audio (silent mute).\n• Ini sengaja dirancang agar telinga dan headphone Anda aman dari suara ledakan/letupan keras (pop/click).\n• Sambungkan kembali kabel sampai hijau untuk melanjutkan musik.").size(10.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            _ => {
                                ui.label(egui::RichText::new("Percabangan (Splitter) & Penggabung (Mixer)").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Aturan 1 Kabel: Modul biasa hanya mengizinkan 1 kabel masuk dan 1 kabel keluar. Menarik kabel baru otomatis menggantikan kabel lama.\n• Membagi Jalur (Splitter / Y-Branch): Gunakan modul '🔀 Splitter' untuk menduplikasi sinyal menjadi 2 cabang (Pin A & Pin B) secara lossless tanpa kehilangan kualitas.\n• Menggabungkan Jalur (Mixer / Summing): Sambungkan kedua cabang ke Pin A & Pin B milik '🎚️ Mixer' untuk mencampur suara Dry & Wet dengan slider crossfader presisi!").size(10.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Footer Navigation
                        ui.horizontal(|ui| {
                            let can_prev = self.patchbay_tutorial_step > 0;
                            if ui.add_enabled(can_prev, egui::Button::new("◀ Sebelumnya")).clicked() {
                                if self.patchbay_tutorial_step > 0 {
                                    self.patchbay_tutorial_step -= 1;
                                }
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if self.patchbay_tutorial_step >= 4 {
                                    if ui.button(egui::RichText::new("Selesai ✓ Tutup").strong().color(Color32::from_rgb(16, 185, 129))).clicked() {
                                        self.show_patchbay_tutorial = false;
                                    }
                                } else {
                                    if ui.button(egui::RichText::new("Lanjut ▶").strong()).clicked() {
                                        self.patchbay_tutorial_step += 1;
                                    }
                                }
                            });
                        });
                    });
            });
    }

    /// Dedicated Slide-Based Interactive Filter & FX Tutorial Modal with rich diagrams & in-depth guides
    pub fn render_filter_tutorial_card(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let filter_type = match &self.active_filter_tutorial {
            Some(t) => t.clone(),
            None => return,
        };

        let padding = 12.0;
        let card_w = 430.0f32.min(canvas_rect.width() - 24.0);
        let card_h = 495.0f32.min(canvas_rect.height() - 24.0);
        let card_pos = Pos2::new(
            canvas_rect.max.x - card_w - padding,
            canvas_rect.min.y + padding,
        );

        let area_id = ui.scope_id().with("filter_tutorial_card_area");
        egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(card_pos)
            .show(ui.ctx(), |ui| {
                let theme_color = match &filter_type {
                    NodeType::LowPassFilter => Color32::from_rgb(14, 165, 233),
                    NodeType::TubeWarmer => Color32::from_rgb(249, 115, 22),
                    NodeType::StereoWidener => Color32::from_rgb(99, 102, 241),
                    NodeType::BassGuard => Color32::from_rgb(16, 185, 129),
                    NodeType::VolumeAttenuator => Color32::from_rgb(245, 158, 11),
                    NodeType::SavedEqPreset(_) => Color32::from_rgb(168, 85, 247),
                    NodeType::Mixer => Color32::from_rgb(147, 51, 234),
                    NodeType::Splitter => Color32::from_rgb(245, 158, 11),
                    NodeType::VolumeBooster => Color32::from_rgb(139, 92, 246),
                    NodeType::AnalogPreamp => Color32::from_rgb(234, 88, 12),
                    NodeType::DynamicLoudness => Color32::from_rgb(245, 158, 11),
                    NodeType::SteppedAttenuator => Color32::from_rgb(6, 182, 212),
                    NodeType::OpticalLeveler => Color32::from_rgb(180, 83, 9),
                    NodeType::StereoBalance => Color32::from_rgb(79, 70, 229),
                    _ => Color32::from_rgb(14, 165, 233),
                };

                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.5, theme_color))
                    .corner_radius(12.0)
                    .inner_margin(13.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 6],
                        blur: 20,
                        spread: 0,
                        color: Color32::from_black_alpha(32),
                    })
                    .show(ui, |ui| {
                        ui.set_width(card_w - 26.0);
                        ui.set_max_height(card_h - 26.0);

                        let step = self.filter_tutorial_step % 4;

                        let (icon, filter_title, is_added) = match &filter_type {
                            NodeType::LowPassFilter => ("🌊", "LOW-PASS FILTER", self.nodes.iter().any(|n| n.node_type == NodeType::LowPassFilter)),
                            NodeType::TubeWarmer => ("🔥", "TUBE WARMER", self.nodes.iter().any(|n| n.node_type == NodeType::TubeWarmer)),
                            NodeType::StereoWidener => ("🌐", "STEREO 3D WIDENER", self.nodes.iter().any(|n| n.node_type == NodeType::StereoWidener)),
                            NodeType::BassGuard => ("🛡️", "BASS GUARD & LIMITER", self.nodes.iter().any(|n| n.node_type == NodeType::BassGuard)),
                            NodeType::VolumeAttenuator => ("🔉", "VOLUME ATTENUATOR (-6dB)", self.nodes.iter().any(|n| n.node_type == NodeType::VolumeAttenuator)),
                            NodeType::SavedEqPreset(name) => ("🎛️", "EQUALIZER PRESET", self.nodes.iter().any(|n| matches!(&n.node_type, NodeType::SavedEqPreset(p) if p == name))),
                            NodeType::Mixer => ("🎚️", "MIXER / SUMMING BUS", self.nodes.iter().any(|n| n.node_type == NodeType::Mixer)),
                            NodeType::Splitter => ("🔀", "SPLITTER / Y-BRANCH BUS", self.nodes.iter().any(|n| n.node_type == NodeType::Splitter)),
                            NodeType::VolumeBooster => ("⚡", "VOLUME BOOSTER (+24dB)", self.nodes.iter().any(|n| n.node_type == NodeType::VolumeBooster)),
                            NodeType::AnalogPreamp => ("🎛️", "ANALOG PREAMP & TRIM", self.nodes.iter().any(|n| n.node_type == NodeType::AnalogPreamp)),
                            NodeType::DynamicLoudness => ("🌙", "DYNAMIC LOUDNESS (ISO 226)", self.nodes.iter().any(|n| n.node_type == NodeType::DynamicLoudness)),
                            NodeType::SteppedAttenuator => ("🔘", "STEPPED ATTENUATOR (64-STEP)", self.nodes.iter().any(|n| n.node_type == NodeType::SteppedAttenuator)),
                            NodeType::OpticalLeveler => ("🛡️", "OPTICAL LEVELER (LA-2A)", self.nodes.iter().any(|n| n.node_type == NodeType::OpticalLeveler)),
                            NodeType::StereoBalance => ("⚖️", "STEREO BALANCE & TRIM", self.nodes.iter().any(|n| n.node_type == NodeType::StereoBalance)),
                            _ => ("🎛️", "FILTER MODULAR", true),
                        };

                        // 1. Header Bar
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).size(16.0));
                            ui.label(
                                egui::RichText::new(format!("PANDUAN: {}", filter_title))
                                    .strong()
                                    .size(12.0)
                                    .color(Color32::from_rgb(15, 23, 42)),
                            );

                            ui.label(
                                egui::RichText::new(format!("Slide {}/4", step + 1))
                                    .strong()
                                    .size(10.5)
                                    .color(theme_color),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("✕").size(12.5).strong().color(Color32::from_rgb(148, 163, 184)))
                                    .on_hover_text("Tutup tutorial")
                                    .clicked()
                                {
                                    self.active_filter_tutorial = None;
                                }
                            });
                        });

                        ui.add_space(4.0);

                        // 2. Interactive Slide Selection Tabs
                        ui.horizontal(|ui| {
                            let slide_titles = [
                                "1. Karakter Suara",
                                "2. Posisi Sah ✅",
                                "3. Posisi Terlarang 🚫",
                                "4. Mode & Tips ⚡",
                            ];
                            for (idx, title) in slide_titles.iter().enumerate() {
                                let is_active = step == idx;
                                let btn = if is_active {
                                    ui.add(egui::Button::new(egui::RichText::new(*title).size(9.5).strong().color(Color32::WHITE)).fill(theme_color))
                                } else {
                                    ui.button(egui::RichText::new(*title).size(9.5).color(Color32::from_rgb(71, 85, 105)))
                                };
                                if btn.clicked() {
                                    self.filter_tutorial_step = idx;
                                }
                            }
                        });

                        ui.add_space(6.0);

                        // 3. Tailored Visual Diagram Box for each slide (Height: 120px)
                        let diag_h = 120.0;
                        let (diag_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), diag_h), egui::Sense::hover());
                        let p = ui.painter().with_clip_rect(diag_rect);
                        p.rect_filled(diag_rect, 8.0, Color32::from_rgb(248, 250, 252));
                        p.rect_stroke(diag_rect, 8.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

                        match step {
                            0 => {
                                // Slide 1: Karakter Suara & Kurva Akustik
                                if filter_type == NodeType::LowPassFilter {
                                    let x_start = diag_rect.min.x + 14.0;
                                    let x_cut = diag_rect.min.x + diag_rect.width() * 0.55;
                                    let x_end = diag_rect.max.x - 14.0;
                                    let y_pass = diag_rect.min.y + 24.0;
                                    let y_mute = diag_rect.max.y - 20.0;

                                    // Shaded passband area
                                    let mut shade_poly = Vec::new();
                                    shade_poly.push(Pos2::new(x_start, y_mute));
                                    shade_poly.push(Pos2::new(x_start, y_pass));
                                    shade_poly.push(Pos2::new(x_cut, y_pass));
                                    shade_poly.push(Pos2::new(x_cut + 24.0, y_mute));
                                    shade_poly.push(Pos2::new(x_end, y_mute));
                                    p.add(egui::Shape::convex_polygon(shade_poly, Color32::from_rgba_premultiplied(14, 165, 233, 35), Stroke::NONE));

                                    // Flat line in passband
                                    p.line_segment([Pos2::new(x_start, y_pass), Pos2::new(x_cut, y_pass)], Stroke::new(2.5, Color32::from_rgb(14, 165, 233)));
                                    // Rolloff slope curve (12dB/octave)
                                    let cp1 = Pos2::new(x_cut + 8.0, y_pass);
                                    let cp2 = Pos2::new(x_cut + 16.0, y_mute);
                                    let p_end = Pos2::new(x_end, y_mute);
                                    let curve = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [Pos2::new(x_cut, y_pass), cp1, cp2, p_end],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.5, Color32::from_rgb(14, 165, 233)),
                                    );
                                    p.add(curve);

                                    // Cutoff Marker Dot & Vertical Guideline
                                    p.line_segment([Pos2::new(x_cut, y_pass), Pos2::new(x_cut, y_mute)], Stroke::new(1.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)).color));
                                    p.circle_filled(Pos2::new(x_cut, y_pass), 4.0, Color32::from_rgb(2, 132, 199));
                                    p.circle_stroke(Pos2::new(x_cut, y_pass), 4.0, Stroke::new(1.5, Color32::WHITE));

                                    // Dynamic wave ripple if playing
                                    if self.is_playing {
                                        let t = ui.input(|i| i.time);
                                        for step_x in 0..8 {
                                            let rx = x_start + step_x as f32 * ((x_cut - x_start) / 8.0);
                                            let ry = y_pass + (rx * 0.1 + (t * 5.0) as f32).sin() * 2.5;
                                            p.circle_filled(Pos2::new(rx, ry), 1.2, Color32::from_rgb(56, 189, 248));
                                        }
                                    }

                                    // Axis Frequency Labels
                                    p.text(Pos2::new(x_start, y_mute + 4.0), egui::Align2::LEFT_TOP, "20 Hz (Bass Lolos ✅)", egui::FontId::proportional(8.5), Color32::from_rgb(16, 185, 129));
                                    p.text(Pos2::new(x_cut, y_pass - 12.0), egui::Align2::CENTER_BOTTOM, "Cutoff 800 Hz", egui::FontId::proportional(9.0), Color32::from_rgb(2, 132, 199));
                                    p.text(Pos2::new(x_end, y_mute + 4.0), egui::Align2::RIGHT_TOP, "20 kHz (Treble Muted 🔇)", egui::FontId::proportional(8.5), Color32::from_rgb(239, 68, 68));
                                } else if filter_type == NodeType::Mixer {
                                    let cy = diag_rect.center().y - 8.0;
                                    let bar_w = diag_rect.width() - 48.0;
                                    let bar_x = diag_rect.min.x + 24.0;
                                    let bar_r = Rect::from_min_size(Pos2::new(bar_x, cy - 8.0), Vec2::new(bar_w, 16.0));

                                    // Gradient-like dual background (Purple A to Indigo B)
                                    p.rect_filled(Rect::from_min_size(Pos2::new(bar_x, cy - 8.0), Vec2::new(bar_w * 0.5, 16.0)), 4.0, Color32::from_rgb(243, 232, 255));
                                    p.rect_filled(Rect::from_min_size(Pos2::new(bar_x + bar_w * 0.5, cy - 8.0), Vec2::new(bar_w * 0.5, 16.0)), 4.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(bar_r, 4.0, Stroke::new(1.0, Color32::from_rgb(216, 180, 254)), egui::StrokeKind::Outside);

                                    // Center balance notch
                                    p.line_segment([Pos2::new(bar_x + bar_w * 0.5, cy - 11.0), Pos2::new(bar_x + bar_w * 0.5, cy + 11.0)], Stroke::new(1.5, Color32::from_rgb(147, 51, 234)));

                                    // Thumb knob
                                    let thumb_x = bar_x + bar_w * 0.5;
                                    p.circle_filled(Pos2::new(thumb_x, cy), 6.5, Color32::WHITE);
                                    p.circle_stroke(Pos2::new(thumb_x, cy), 6.5, Stroke::new(2.0, Color32::from_rgb(147, 51, 234)));

                                    p.text(Pos2::new(bar_x, cy - 14.0), egui::Align2::LEFT_BOTTOM, "Channel A (Dry / Suara Asli)", egui::FontId::proportional(9.0), Color32::from_rgb(126, 34, 206));
                                    p.text(Pos2::new(bar_x + bar_w, cy - 14.0), egui::Align2::RIGHT_BOTTOM, "Channel B (Wet / Efek)", egui::FontId::proportional(9.0), Color32::from_rgb(79, 70, 229));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "⚖️ Constant-Power Blending: Daya audio seimbang tanpa lonjakan clipping",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(126, 34, 206),
                                    );
                                } else if filter_type == NodeType::Splitter {
                                    let cy = diag_rect.center().y - 6.0;
                                    let p_in = Pos2::new(diag_rect.min.x + 36.0, cy);
                                    let p_fork = Pos2::new(diag_rect.center().x - 20.0, cy);
                                    let p_a = Pos2::new(diag_rect.max.x - 45.0, cy - 24.0);
                                    let p_b = Pos2::new(diag_rect.max.x - 45.0, cy + 24.0);

                                    p.line_segment([p_in, p_fork], Stroke::new(2.6, theme_color));
                                    p.line_segment([p_fork, p_a], Stroke::new(2.6, theme_color));
                                    p.line_segment([p_fork, p_b], Stroke::new(2.6, theme_color));
                                    p.circle_filled(p_fork, 4.0, Color32::from_rgb(217, 119, 6));

                                    p.circle_filled(p_in, 4.5, Color32::from_rgb(99, 102, 241));
                                    p.text(Pos2::new(p_in.x - 8.0, cy), egui::Align2::RIGHT_CENTER, "Input PCM (64-bit)", egui::FontId::proportional(9.0), Color32::from_rgb(71, 85, 105));

                                    p.circle_filled(p_a, 4.5, Color32::from_rgb(16, 185, 129));
                                    p.text(Pos2::new(p_a.x + 8.0, p_a.y), egui::Align2::LEFT_CENTER, "Out Pin A (100% Klon)", egui::FontId::proportional(9.0), Color32::from_rgb(16, 185, 129));

                                    p.circle_filled(p_b, 4.5, Color32::from_rgb(14, 165, 233));
                                    p.text(Pos2::new(p_b.x + 8.0, p_b.y), egui::Align2::LEFT_CENTER, "Out Pin B (100% Klon)", egui::FontId::proportional(9.0), Color32::from_rgb(14, 165, 233));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🔀 Lossless Duplication: Kedua output menerima aliran audio bitstream identik tanpa latensi",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(217, 119, 6),
                                    );
                                } else if filter_type == NodeType::VolumeBooster {
                                    let x_start = diag_rect.min.x + 24.0;
                                    let x_end = diag_rect.max.x - 24.0;
                                    let y_bot = diag_rect.max.y - 24.0;
                                    let y_top = diag_rect.min.y + 22.0;

                                    // Unity gain baseline (dashed)
                                    p.line_segment([Pos2::new(x_start, y_bot - 8.0), Pos2::new(x_end, y_bot - 8.0)], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
                                    p.text(Pos2::new(x_start, y_bot - 14.0), egui::Align2::LEFT_BOTTOM, "0.0 dB (Unity)", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));

                                    // Linear clean amplification ramp
                                    let p_ramp_start = Pos2::new(x_start, y_bot - 8.0);
                                    let p_ramp_end = Pos2::new(x_end, y_top + 4.0);
                                    p.line_segment([p_ramp_start, p_ramp_end], Stroke::new(2.5, theme_color));
                                    p.circle_filled(p_ramp_end, 4.0, Color32::from_rgb(139, 92, 246));
                                    p.text(Pos2::new(x_end, y_top - 2.0), egui::Align2::RIGHT_BOTTOM, "+24.0 dB Max (Uncolored)", egui::FontId::proportional(8.5), Color32::from_rgb(139, 92, 246));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "⚡ Pure Uncolored Clean Amp: Penguatan sinyal linier murni tanpa distorsi",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(124, 58, 237),
                                    );
                                } else if filter_type == NodeType::AnalogPreamp {
                                    let x_start = diag_rect.min.x + 24.0;
                                    let x_mid = diag_rect.center().x;
                                    let x_end = diag_rect.max.x - 24.0;
                                    let y_mid = diag_rect.center().y - 6.0;

                                    // Class-A soft saturation transfer curve (tanh curve)
                                    let mut pts = Vec::new();
                                    for i in 0..=20 {
                                        let t = (i as f32 / 20.0) * 2.0 - 1.0;
                                        let x = x_mid + t * (x_end - x_mid);
                                        let y = y_mid - (t * 1.5).tanh() * 28.0;
                                        pts.push(Pos2::new(x, y));
                                    }
                                    for seg in 0..pts.len() - 1 {
                                        p.line_segment([pts[seg], pts[seg + 1]], Stroke::new(2.2, theme_color));
                                    }

                                    p.text(Pos2::new(x_start, y_mid + 20.0), egui::Align2::LEFT_TOP, "-24 dB Clean Trim", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                                    p.text(Pos2::new(x_mid, y_mid + 16.0), egui::Align2::CENTER_TOP, "0 dB Detent", egui::FontId::proportional(8.0), Color32::from_rgb(217, 119, 6));
                                    p.text(Pos2::new(x_end, y_mid - 24.0), egui::Align2::RIGHT_BOTTOM, "+12 dB Class-A Saturation", egui::FontId::proportional(8.5), Color32::from_rgb(234, 88, 12));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🎛️ Transformer Iron Core Warmth: Lembut meredam transient dengan harmonik musikal",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(194, 65, 12),
                                    );
                                } else if filter_type == NodeType::DynamicLoudness {
                                    let x_start = diag_rect.min.x + 20.0;
                                    let x_mid = diag_rect.center().x;
                                    let x_end = diag_rect.max.x - 20.0;
                                    let y_mid = diag_rect.center().y - 4.0;

                                    // ISO 226 Fletcher-Munson Smiley Curve
                                    let mut pts = Vec::new();
                                    for i in 0..=24 {
                                        let t = i as f32 / 24.0;
                                        let x = x_start + t * (x_end - x_start);
                                        let lift = if t < 0.3 {
                                            (1.0 - t / 0.3) * 16.0
                                        } else if t > 0.7 {
                                            ((t - 0.7) / 0.3) * 11.0
                                        } else {
                                            0.0
                                        };
                                        let y = y_mid - lift;
                                        pts.push(Pos2::new(x, y));
                                    }
                                    for seg in 0..pts.len() - 1 {
                                        p.line_segment([pts[seg], pts[seg + 1]], Stroke::new(2.4, theme_color));
                                    }

                                    p.text(Pos2::new(x_start, y_mid - 18.0), egui::Align2::LEFT_BOTTOM, "+8.5 dB Sub-Bass (<120Hz)", egui::FontId::proportional(8.0), Color32::from_rgb(217, 119, 6));
                                    p.text(Pos2::new(x_mid, y_mid + 8.0), egui::Align2::CENTER_TOP, "Mid 1kHz (Flat 0dB)", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                                    p.text(Pos2::new(x_end, y_mid - 13.0), egui::Align2::RIGHT_BOTTOM, "+5.0 dB Air (>9kHz)", egui::FontId::proportional(8.0), Color32::from_rgb(217, 119, 6));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🌙 Equal-Loudness Contour: Mengangkat bass & treble di malam hari / volume rendah",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(217, 119, 6),
                                    );
                                } else if filter_type == NodeType::SteppedAttenuator {
                                    let x_start = diag_rect.min.x + 20.0;
                                    let x_end = diag_rect.max.x - 20.0;
                                    let y_top = diag_rect.min.y + 24.0;
                                    let y_bot = diag_rect.max.y - 28.0;

                                    // Discrete stair-step descent (64-step illustration)
                                    let num_steps = 12;
                                    let step_w = (x_end - x_start) / num_steps as f32;
                                    let step_h = (y_bot - y_top) / num_steps as f32;
                                    for s in 0..num_steps {
                                        let sx = x_start + s as f32 * step_w;
                                        let sy = y_top + s as f32 * step_h;
                                        p.line_segment([Pos2::new(sx, sy), Pos2::new(sx + step_w, sy)], Stroke::new(2.0, theme_color));
                                        if s < num_steps - 1 {
                                            p.line_segment([Pos2::new(sx + step_w, sy), Pos2::new(sx + step_w, sy + step_h)], Stroke::new(2.0, theme_color));
                                        }
                                    }

                                    // DIM -20dB marker
                                    let dim_x = x_start + 4.0 * step_w;
                                    let dim_y = y_top + 4.0 * step_h;
                                    p.circle_filled(Pos2::new(dim_x, dim_y), 4.0, Color32::from_rgb(6, 182, 212));
                                    p.text(Pos2::new(dim_x + 8.0, dim_y), egui::Align2::LEFT_CENTER, "DIM -20 dB Quick Drop", egui::FontId::proportional(8.5), Color32::from_rgb(14, 116, 144));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🔘 64-Step Discrete Ladder: Redaman pasif presisi tinggi tanpa kebocoran bit",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(14, 116, 144),
                                    );
                                } else if filter_type == NodeType::OpticalLeveler {
                                    let x_start = diag_rect.min.x + 24.0;
                                    let x_end = diag_rect.max.x - 24.0;
                                    let y_top = diag_rect.min.y + 22.0;
                                    let y_bot = diag_rect.max.y - 26.0;

                                    // Fast Stage 1 release curve (first 50% in 60ms) + Slow Stage 2 release (smooth tail)
                                    let x_stage1 = x_start + (x_end - x_start) * 0.35;
                                    let y_stage1 = y_top + (y_bot - y_top) * 0.55;

                                    let cp1 = Pos2::new(x_start + 20.0, y_top + 2.0);
                                    let cp2 = Pos2::new(x_stage1 - 10.0, y_stage1);
                                    let curve1 = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [Pos2::new(x_start, y_top), cp1, cp2, Pos2::new(x_stage1, y_stage1)],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.4, theme_color),
                                    );
                                    p.add(curve1);

                                    let cp3 = Pos2::new(x_stage1 + 30.0, y_stage1 + 5.0);
                                    let cp4 = Pos2::new(x_end - 20.0, y_bot);
                                    let curve2 = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [Pos2::new(x_stage1, y_stage1), cp3, cp4, Pos2::new(x_end, y_bot)],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.4, Color32::from_rgb(217, 119, 6)),
                                    );
                                    p.add(curve2);

                                    p.circle_filled(Pos2::new(x_stage1, y_stage1), 3.5, Color32::from_rgb(180, 83, 9));
                                    p.text(Pos2::new(x_start + 10.0, y_top - 2.0), egui::Align2::LEFT_BOTTOM, "Tahap 1: Cepat 60ms", egui::FontId::proportional(8.0), theme_color);
                                    p.text(Pos2::new(x_end, y_stage1), egui::Align2::RIGHT_CENTER, "Tahap 2: Pelepasan Halus 1-5 Detik", egui::FontId::proportional(8.0), Color32::from_rgb(180, 83, 9));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🛡️ Dual-Stage Optical Release: Leveling alami instrumen & vokal bebas efek pumping",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(180, 83, 9),
                                    );
                                } else if filter_type == NodeType::StereoBalance {
                                    let x_start = diag_rect.min.x + 24.0;
                                    let x_mid = diag_rect.center().x;
                                    let x_end = diag_rect.max.x - 24.0;
                                    let y_mid = diag_rect.center().y - 6.0;

                                    // Dual Sine / Cosine Constant-Power Panning Curves
                                    let mut l_pts = Vec::new();
                                    let mut r_pts = Vec::new();
                                    for i in 0..=20 {
                                        let t = i as f32 / 20.0;
                                        let x = x_start + t * (x_end - x_start);
                                        // Left channel power: cos(t * pi / 2)
                                        let l_y = y_mid + 16.0 - (t * std::f32::consts::PI * 0.5).cos() * 26.0;
                                        // Right channel power: sin(t * pi / 2)
                                        let r_y = y_mid + 16.0 - (t * std::f32::consts::PI * 0.5).sin() * 26.0;
                                        l_pts.push(Pos2::new(x, l_y));
                                        r_pts.push(Pos2::new(x, r_y));
                                    }
                                    for seg in 0..l_pts.len() - 1 {
                                        p.line_segment([l_pts[seg], l_pts[seg + 1]], Stroke::new(2.2, Color32::from_rgb(99, 102, 241)));
                                        p.line_segment([r_pts[seg], r_pts[seg + 1]], Stroke::new(2.2, Color32::from_rgb(168, 85, 247)));
                                    }

                                    // Center detent point
                                    p.circle_filled(Pos2::new(x_mid, y_mid - 2.5), 3.5, Color32::WHITE);
                                    p.circle_stroke(Pos2::new(x_mid, y_mid - 2.5), 3.5, Stroke::new(1.5, theme_color));
                                    p.text(Pos2::new(x_start, y_mid - 12.0), egui::Align2::LEFT_BOTTOM, "Left Channel", egui::FontId::proportional(8.5), Color32::from_rgb(79, 70, 229));
                                    p.text(Pos2::new(x_mid, y_mid + 16.0), egui::Align2::CENTER_TOP, "Center: Power Konstan", egui::FontId::proportional(8.0), Color32::from_rgb(100, 116, 139));
                                    p.text(Pos2::new(x_end, y_mid - 12.0), egui::Align2::RIGHT_BOTTOM, "Right Channel", egui::FontId::proportional(8.5), Color32::from_rgb(147, 51, 234));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "⚖️ Constant-Power Pan: Daya akustik konstan tanpa penurunan volume di tengah",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(79, 70, 229),
                                    );
                                } else {
                                    let cy = diag_rect.center().y;
                                    p.text(Pos2::new(diag_rect.center().x, cy - 10.0), egui::Align2::CENTER_CENTER, "🔊 KARAKTER SUARA MODUL", egui::FontId::proportional(11.0), theme_color);
                                    p.text(Pos2::new(diag_rect.center().x, cy + 10.0), egui::Align2::CENTER_CENTER, "Memodifikasi profil frekuensi, saturasi, atau panggung stereo", egui::FontId::proportional(9.0), Color32::from_rgb(100, 116, 139));
                                }
                            }
                            1 => {
                                // Slide 2: Posisi Sah & Alur Sinyal Lengkap
                                if filter_type == NodeType::Mixer {
                                    let cy = diag_rect.center().y - 8.0;
                                    let b_w = 64.0;
                                    let b_h = 28.0;

                                    let r1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 10.0, cy - 30.0), Vec2::new(b_w, b_h));
                                    let r2 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 10.0, cy + 8.0), Vec2::new(b_w, b_h));
                                    let r_m = Rect::from_min_size(Pos2::new(diag_rect.center().x - 45.0, cy - 20.0), Vec2::new(90.0, 42.0));
                                    let r3 = Rect::from_min_size(Pos2::new(diag_rect.max.x - b_w - 10.0, cy - 14.0), Vec2::new(b_w, b_h));

                                    p.rect_filled(r1, 5.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(r1, 5.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                                    p.text(r1.center(), egui::Align2::CENTER_CENTER, "RAM / DSP (A)", egui::FontId::proportional(8.0), Color32::from_rgb(67, 56, 202));

                                    p.rect_filled(r2, 5.0, Color32::from_rgb(240, 249, 255));
                                    p.rect_stroke(r2, 5.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);
                                    p.text(r2.center(), egui::Align2::CENTER_CENTER, "Filter / LPF (B)", egui::FontId::proportional(8.0), Color32::from_rgb(2, 132, 199));

                                    p.rect_filled(r_m, 6.0, Color32::from_rgb(250, 245, 255));
                                    p.rect_stroke(r_m, 6.0, Stroke::new(1.5, theme_color), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r_m.center().x, r_m.center().y - 5.0), egui::Align2::CENTER_CENTER, "🎚️ MIXER BUS", egui::FontId::proportional(8.5), theme_color);
                                    p.text(Pos2::new(r_m.center().x, r_m.center().y + 8.0), egui::Align2::CENTER_CENTER, "✅ SUMMING SAH", egui::FontId::proportional(7.0), Color32::from_rgb(16, 185, 129));

                                    p.rect_filled(r3, 5.0, Color32::from_rgb(236, 253, 245));
                                    p.rect_stroke(r3, 5.0, Stroke::new(1.0, Color32::from_rgb(167, 243, 208)), egui::StrokeKind::Outside);
                                    p.text(r3.center(), egui::Align2::CENTER_CENTER, "Output DAC", egui::FontId::proportional(8.5), Color32::from_rgb(6, 95, 70));

                                    // Connecting cables
                                    p.line_segment([Pos2::new(r1.max.x, r1.center().y), Pos2::new(r_m.min.x, r_m.center().y - 10.0)], Stroke::new(2.2, Color32::from_rgb(147, 51, 234)));
                                    p.line_segment([Pos2::new(r2.max.x, r2.center().y), Pos2::new(r_m.min.x, r_m.center().y + 10.0)], Stroke::new(2.2, Color32::from_rgb(147, 51, 234)));
                                    p.line_segment([Pos2::new(r_m.max.x, r_m.center().y), Pos2::new(r3.min.x, r3.center().y)], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "✨ Masukkan 2 cabang sinyal ke Pin A & B, lalu teruskan ke DAC",
                                        egui::FontId::proportional(9.0),
                                        Color32::from_rgb(16, 185, 129),
                                    );
                                } else if filter_type == NodeType::Splitter {
                                    let cy = diag_rect.center().y - 8.0;
                                    let b_w = 54.0;
                                    let b_h = 24.0;

                                    let r_src = Rect::from_min_size(Pos2::new(diag_rect.min.x + 8.0, cy - 12.0), Vec2::new(b_w, b_h));
                                    let r_split = Rect::from_min_size(Pos2::new(diag_rect.min.x + 72.0, cy - 16.0), Vec2::new(64.0, 32.0));
                                    let r_fx = Rect::from_min_size(Pos2::new(diag_rect.center().x + 12.0, cy + 14.0), Vec2::new(b_w, b_h));
                                    let r_mix = Rect::from_min_size(Pos2::new(diag_rect.max.x - 126.0, cy - 16.0), Vec2::new(58.0, 32.0));
                                    let r_dac = Rect::from_min_size(Pos2::new(diag_rect.max.x - 52.0, cy - 12.0), Vec2::new(44.0, b_h));

                                    p.rect_filled(r_src, 4.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(r_src, 4.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                                    p.text(r_src.center(), egui::Align2::CENTER_CENTER, "Resampler", egui::FontId::proportional(8.0), Color32::from_rgb(67, 56, 202));

                                    p.rect_filled(r_split, 5.0, Color32::from_rgb(255, 251, 235));
                                    p.rect_stroke(r_split, 5.0, Stroke::new(1.5, theme_color), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r_split.center().x, r_split.center().y - 4.0), egui::Align2::CENTER_CENTER, "🔀 SPLITTER", egui::FontId::proportional(7.5), theme_color);
                                    p.text(Pos2::new(r_split.center().x, r_split.center().y + 7.0), egui::Align2::CENTER_CENTER, "Out A • Out B", egui::FontId::proportional(6.5), Color32::from_rgb(180, 83, 9));

                                    p.rect_filled(r_fx, 4.0, Color32::from_rgb(240, 249, 255));
                                    p.rect_stroke(r_fx, 4.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);
                                    p.text(r_fx.center(), egui::Align2::CENTER_CENTER, "Filter (LPF)", egui::FontId::proportional(7.5), Color32::from_rgb(2, 132, 199));

                                    p.rect_filled(r_mix, 5.0, Color32::from_rgb(250, 245, 255));
                                    p.rect_stroke(r_mix, 5.0, Stroke::new(1.2, Color32::from_rgb(147, 51, 234)), egui::StrokeKind::Outside);
                                    p.text(r_mix.center(), egui::Align2::CENTER_CENTER, "🎚️ MIXER", egui::FontId::proportional(7.5), Color32::from_rgb(147, 51, 234));

                                    p.rect_filled(r_dac, 4.0, Color32::from_rgb(236, 253, 245));
                                    p.rect_stroke(r_dac, 4.0, Stroke::new(1.0, Color32::from_rgb(167, 243, 208)), egui::StrokeKind::Outside);
                                    p.text(r_dac.center(), egui::Align2::CENTER_CENTER, "DAC", egui::FontId::proportional(8.0), Color32::from_rgb(6, 95, 70));

                                    // Diamond Cables
                                    p.line_segment([Pos2::new(r_src.max.x, r_src.center().y), Pos2::new(r_split.min.x, r_split.center().y)], Stroke::new(2.0, Color32::from_rgb(99, 102, 241)));
                                    // Upper Dry branch: Pin A to Mixer In A
                                    p.line_segment([Pos2::new(r_split.max.x, r_split.center().y - 6.0), Pos2::new(r_mix.min.x, r_mix.center().y - 6.0)], Stroke::new(2.0, Color32::from_rgb(16, 185, 129)));
                                    // Lower Wet branch: Pin B to LPF, then LPF to Mixer In B
                                    p.line_segment([Pos2::new(r_split.max.x, r_split.center().y + 6.0), Pos2::new(r_fx.min.x, r_fx.center().y)], Stroke::new(2.0, theme_color));
                                    p.line_segment([Pos2::new(r_fx.max.x, r_fx.center().y), Pos2::new(r_mix.min.x, r_mix.center().y + 6.0)], Stroke::new(2.0, Color32::from_rgb(14, 165, 233)));
                                    // Mixer to DAC
                                    p.line_segment([Pos2::new(r_mix.max.x, r_mix.center().y), Pos2::new(r_dac.min.x, r_dac.center().y)], Stroke::new(2.2, Color32::from_rgb(16, 185, 129)));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💎 Diamond Loop: Pin A (Dry) & Pin B (LPF) menyatu kembali di Mixer",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(16, 185, 129),
                                    );
                                } else {
                                    let cy = diag_rect.center().y - 10.0;
                                    let b_w = 66.0;
                                    let b_h = 32.0;

                                    let r1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 8.0, cy - b_h * 0.5), Vec2::new(b_w, b_h));
                                    let r2 = Rect::from_min_size(Pos2::new(diag_rect.center().x - b_w * 0.5, cy - b_h * 0.5), Vec2::new(b_w, b_h));
                                    let r3 = Rect::from_min_size(Pos2::new(diag_rect.max.x - b_w - 8.0, cy - b_h * 0.5), Vec2::new(b_w, b_h));

                                    // Box 1: RAM Buffer
                                    p.rect_filled(r1, 5.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(r1, 5.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                                    p.text(r1.center(), egui::Align2::CENTER_CENTER, "RAM Buffer", egui::FontId::proportional(8.5), Color32::from_rgb(67, 56, 202));

                                    // Box 2: Filter Efek (Highlighted)
                                    p.rect_filled(r2, 5.0, Color32::from_rgb(240, 249, 255));
                                    p.rect_stroke(r2, 5.0, Stroke::new(1.8, theme_color), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r2.center().x, r2.center().y - 4.0), egui::Align2::CENTER_CENTER, filter_title, egui::FontId::proportional(7.5), theme_color);
                                    p.text(Pos2::new(r2.center().x, r2.center().y + 6.0), egui::Align2::CENTER_CENTER, "✅ POSISI SAH", egui::FontId::proportional(7.0), Color32::from_rgb(16, 185, 129));

                                    // Box 3: Output DAC
                                    p.rect_filled(r3, 5.0, Color32::from_rgb(236, 253, 245));
                                    p.rect_stroke(r3, 5.0, Stroke::new(1.0, Color32::from_rgb(167, 243, 208)), egui::StrokeKind::Outside);
                                    p.text(r3.center(), egui::Align2::CENTER_CENTER, "Output DAC", egui::FontId::proportional(8.5), Color32::from_rgb(6, 95, 70));

                                    // Flowing Green Cables
                                    p.line_segment([Pos2::new(r1.max.x, cy), Pos2::new(r2.min.x, cy)], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));
                                    p.line_segment([Pos2::new(r2.max.x, cy), Pos2::new(r3.min.x, cy)], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));

                                    p.circle_filled(Pos2::new((r1.max.x + r2.min.x) * 0.5, cy), 3.0, Color32::from_rgb(52, 211, 153));
                                    p.circle_filled(Pos2::new((r2.max.x + r3.min.x) * 0.5, cy), 3.0, Color32::from_rgb(52, 211, 153));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "✨ Pasang di antara RAM Buffer dan DSP Engine, atau sesudah DSP",
                                        egui::FontId::proportional(9.0),
                                        Color32::from_rgb(16, 185, 129),
                                    );
                                }
                            }
                            2 => {
                                // Slide 3: Posisi Terlarang & Bahaya Routing
                                if filter_type == NodeType::Mixer {
                                    let cy = diag_rect.center().y - 12.0;
                                    let half_w = (diag_rect.width() - 36.0) * 0.5;
                                    let b1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 12.0, cy - 22.0), Vec2::new(half_w, 44.0));
                                    let b2 = Rect::from_min_size(Pos2::new(diag_rect.center().x + 6.0, cy - 22.0), Vec2::new(half_w, 44.0));

                                    // Bad Box 1: Output dicolok ke Input sendiri (Feedback Loop)
                                    p.rect_filled(b1, 6.0, Color32::from_rgb(254, 242, 242));
                                    p.rect_stroke(b1, 6.0, Stroke::new(1.2, Color32::from_rgb(252, 165, 165)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(b1.center().x, b1.min.y + 13.0), egui::Align2::CENTER_CENTER, "🚫 Feedback Loop", egui::FontId::proportional(9.5), Color32::from_rgb(185, 28, 28));
                                    p.text(Pos2::new(b1.center().x, b1.min.y + 27.0), egui::Align2::CENTER_CENTER, "Output dicolok ke Pin Input sendiri!", egui::FontId::proportional(7.5), Color32::from_rgb(153, 27, 27));

                                    // Bad Box 2: Sesudah DAC
                                    p.rect_filled(b2, 6.0, Color32::from_rgb(254, 242, 242));
                                    p.rect_stroke(b2, 6.0, Stroke::new(1.2, Color32::from_rgb(252, 165, 165)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(b2.center().x, b2.min.y + 13.0), egui::Align2::CENTER_CENTER, "🚫 Setelah Output DAC", egui::FontId::proportional(9.5), Color32::from_rgb(185, 28, 28));
                                    p.text(Pos2::new(b2.center().x, b2.min.y + 27.0), egui::Align2::CENTER_CENTER, "DAC adalah hardware fisik!", egui::FontId::proportional(7.5), Color32::from_rgb(153, 27, 27));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🔇 LunulaPlayer auto-mute jika loop terdeteksi untuk melindungi speaker",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(239, 68, 68),
                                    );
                                } else {
                                    let cy = diag_rect.center().y - 12.0;
                                    let half_w = (diag_rect.width() - 36.0) * 0.5;
                                    let b1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 12.0, cy - 22.0), Vec2::new(half_w, 44.0));
                                    let b2 = Rect::from_min_size(Pos2::new(diag_rect.center().x + 6.0, cy - 22.0), Vec2::new(half_w, 44.0));

                                    // Bad Box 1: Sebelum Source
                                    p.rect_filled(b1, 6.0, Color32::from_rgb(254, 242, 242));
                                    p.rect_stroke(b1, 6.0, Stroke::new(1.2, Color32::from_rgb(252, 165, 165)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(b1.center().x, b1.min.y + 13.0), egui::Align2::CENTER_CENTER, "🚫 Sebelum File Source", egui::FontId::proportional(9.5), Color32::from_rgb(185, 28, 28));
                                    p.text(Pos2::new(b1.center().x, b1.min.y + 27.0), egui::Align2::CENTER_CENTER, "File belum didecode ke PCM!", egui::FontId::proportional(8.0), Color32::from_rgb(153, 27, 27));

                                    // Bad Box 2: Sesudah DAC
                                    p.rect_filled(b2, 6.0, Color32::from_rgb(254, 242, 242));
                                    p.rect_stroke(b2, 6.0, Stroke::new(1.2, Color32::from_rgb(252, 165, 165)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(b2.center().x, b2.min.y + 13.0), egui::Align2::CENTER_CENTER, "🚫 Setelah Output DAC", egui::FontId::proportional(9.5), Color32::from_rgb(185, 28, 28));
                                    p.text(Pos2::new(b2.center().x, b2.min.y + 27.0), egui::Align2::CENTER_CENTER, "DAC adalah hardware fisik!", egui::FontId::proportional(8.0), Color32::from_rgb(153, 27, 27));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🔇 LunulaPlayer auto-mute jika kabel putus/salah demi keamanan telinga",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(239, 68, 68),
                                    );
                                }
                            }
                            _ => {
                                // Slide 4: Mode Bit-Perfect vs DSP & Live Slider Control
                                if filter_type == NodeType::Mixer {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;

                                    let r_box = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 18.0), Vec2::new(line_w, 48.0));
                                    p.rect_filled(r_box, 4.0, Color32::from_rgb(250, 245, 255));
                                    p.rect_stroke(r_box, 4.0, Stroke::new(1.0, Color32::from_rgb(233, 213, 255)), egui::StrokeKind::Outside);

                                    p.text(Pos2::new(r_box.min.x + 10.0, r_box.min.y + 12.0), egui::Align2::LEFT_CENTER, "◀ 0% (100% Dry A)", egui::FontId::proportional(8.5), Color32::from_rgb(126, 34, 206));
                                    p.text(Pos2::new(r_box.center().x, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "● 50% / 50% (Center Mix)", egui::FontId::proportional(8.5), Color32::from_rgb(147, 51, 234));
                                    p.text(Pos2::new(r_box.max.x - 10.0, r_box.min.y + 12.0), egui::Align2::RIGHT_CENTER, "100% (Full Wet B) ▶", egui::FontId::proportional(8.5), Color32::from_rgb(79, 70, 229));

                                    // Slider illustration
                                    let s_y = r_box.min.y + 32.0;
                                    let s_w = line_w - 40.0;
                                    let s_x = r_box.min.x + 20.0;
                                    p.line_segment([Pos2::new(s_x, s_y), Pos2::new(s_x + s_w, s_y)], Stroke::new(2.5, Color32::from_rgb(216, 180, 254)));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.5, s_y), 5.0, Color32::from_rgb(147, 51, 234));
                                    p.circle_stroke(Pos2::new(s_x + s_w * 0.5, s_y), 5.0, Stroke::new(1.5, Color32::WHITE));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Geser slider langsung pada kartu kanvas atau sidebar kanan secara live",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(100, 116, 139),
                                    );
                                } else if filter_type == NodeType::Splitter {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;

                                    let r_box = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 18.0), Vec2::new(line_w, 48.0));
                                    p.rect_filled(r_box, 4.0, Color32::from_rgb(255, 251, 235));
                                    p.rect_stroke(r_box, 4.0, Stroke::new(1.0, Color32::from_rgb(253, 230, 138)), egui::StrokeKind::Outside);

                                    p.text(Pos2::new(r_box.min.x + 10.0, r_box.min.y + 12.0), egui::Align2::LEFT_CENTER, "⚡ 100% Zero Latency", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));
                                    p.text(Pos2::new(r_box.center().x, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "💎 Bit-Accurate Clone", egui::FontId::proportional(8.5), Color32::from_rgb(217, 119, 6));
                                    p.text(Pos2::new(r_box.max.x - 10.0, r_box.min.y + 12.0), egui::Align2::RIGHT_CENTER, "🔒 Anti Phase-Cancellation", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));

                                    let s_y = r_box.min.y + 32.0;
                                    let s_w = line_w - 40.0;
                                    let s_x = r_box.min.x + 20.0;
                                    p.line_segment([Pos2::new(s_x, s_y), Pos2::new(s_x + s_w, s_y)], Stroke::new(2.5, Color32::from_rgb(253, 230, 138)));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.25, s_y), 4.5, Color32::from_rgb(245, 158, 11));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.75, s_y), 4.5, Color32::from_rgb(147, 51, 234));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Pasang bersama Mixer untuk kendali penuh Dry/Wet parallel processing",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(100, 116, 139),
                                    );
                                } else {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;

                                    // Lane 1: DSP Active Mode
                                    let r_dsp = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 20.0), Vec2::new(line_w, 24.0));
                                    p.rect_filled(r_dsp, 4.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(r_dsp, 4.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r_dsp.min.x + 8.0, r_dsp.center().y), egui::Align2::LEFT_CENTER, "🟣 Mode DSP Active", egui::FontId::proportional(9.0), Color32::from_rgb(67, 56, 202));
                                    p.text(Pos2::new(r_dsp.max.x - 8.0, r_dsp.center().y), egui::Align2::RIGHT_CENTER, "Filter Aktif 64-bit • Slider Real-Time", egui::FontId::proportional(8.5), Color32::from_rgb(79, 70, 229));

                                    // Lane 2: Bit-Perfect Mode
                                    let r_bp = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy + 10.0), Vec2::new(line_w, 24.0));
                                    p.rect_filled(r_bp, 4.0, Color32::from_rgb(241, 245, 249));
                                    p.rect_stroke(r_bp, 4.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r_bp.min.x + 8.0, r_bp.center().y), egui::Align2::LEFT_CENTER, "🟢 Mode Bit-Perfect", egui::FontId::proportional(9.0), Color32::from_rgb(6, 95, 70));
                                    p.text(Pos2::new(r_bp.max.x - 8.0, r_bp.center().y), egui::Align2::RIGHT_CENTER, "⚡ Otomatis DILEWATI (Direct Wire 1:1)", egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Klik kartu DSP ENGINE pada kanvas untuk berganti mode seketika",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(100, 116, 139),
                                    );
                                }
                            }
                        }

                        ui.add_space(8.0);

                        // 4. In-Depth Text Content for Active Slide
                        egui::ScrollArea::vertical().max_height(210.0).show(ui, |ui| {
                            match step {
                                0 => {
                                    // Slide 1 text
                                    if filter_type == NodeType::LowPassFilter {
                                        ui.label(egui::RichText::new("Apa itu Low-Pass Filter (LPF)?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Low-Pass Filter adalah filter frekuensi biquad berpresisi 64-bit float (f64) yang meloloskan frekuensi nada rendah (bass) dan memotong frekuensi nada tinggi (treble) di atas titik Cutoff yang Anda tentukan.").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Efek Akustik & Karakter Suara:").strong().size(11.0).color(Color32::from_rgb(14, 116, 144)));
                                        ui.label(egui::RichText::new("• Efek 'Underwater / Muffled': Suara musik terdengar seperti didengarkan dari dalam air atau dari balik pintu tebal.\n• Menghilangkan Desis & Harshness: Menyaring suara cymbal tajam, desis kaset tua (tape hiss), atau distorsi treble yang menusuk telinga.\n• Suasana Lo-Fi Santai: Sangat cocok untuk mendengarkan lagu saat malam hari, relaksasi, atau membaca buku.").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Nilai Cutoff:").strong().size(11.0).color(Color32::from_rgb(2, 132, 199)));
                                        ui.label(egui::RichText::new("• 100 Hz - 300 Hz: Hanya getaran sub-bass murni yang terdengar (vokal hilang total).\n• 500 Hz - 1.2 kHz: Suara vokal teredam berat, bass hangat mendominasi.\n• 3 kHz - 6 kHz: Nuansa piringan hitam klasik, vokal tetap jelas tanpa 's' tajam.\n• 10 kHz: Pemotongan frekuensi ultrasonik, menjaga kehangatan analog sutra.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Apa itu Mixer / Summing Bus?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Mixer / Summing Bus adalah gerbang perantara (hub) cerdas yang menggabungkan dua aliran audio terpisah (Pin A dan Pin B) menjadi satu jalur output utuh.").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Fungsi Utama & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(126, 34, 206)));
                                        ui.label(egui::RichText::new("• Parallel Processing (Dry/Wet): Mencampur suara murni tanpa efek (Dry) di Jalur A dengan suara yang disaring filter (Wet) di Jalur B. Anda dapat mengatur ketebalan efek tanpa merusak kejelasan vokal aslinya.\n• Constant-Power Crossfading: Menggunakan algoritma kurva daya konstan sehingga tidak ada penurunan volume di titik tengah (50/50).\n• Anti-Clipping Headroom: Mencegah distorsi digital saat dua sinyal dijumlahkan bersamaan.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Apa itu Splitter / Y-Branch Bus?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Splitter / Y-Branch Bus adalah modul pembagi sinyal audio cerdas yang menduplikasi 1 input audio menjadi 2 output mandiri (Pin Out A & Pin Out B) dengan teknologi Lossless 64-bit PCM Duplication.").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Fungsi Utama & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(217, 119, 6)));
                                        ui.label(egui::RichText::new("• Pasangan Sejati Modul Mixer: Memungkinkan routing Dry/Wet paralel tanpa merusak kabel sinyal utama.\n• Zero Phase Delay (Fase Sinkron): Kedua pin memancarkan sampel bitstream yang sinkron mikrodetik yang sama, mencegah distorsi fase (comb filtering) saat disatukan kembali di Mixer.\n• Bit-Accurate Clone: Tidak ada penurunan gain maupun penurunan dynamic range sinyal.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::VolumeBooster {
                                        ui.label(egui::RichText::new("Apa itu Pure Volume Booster?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Pure Volume Booster adalah modul penguat gain murni berpresisi 64-bit float (f64) dari 0.0 dB hingga +24.0 dB tanpa pewarnaan atau degradasi akustik.").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(139, 92, 246)));
                                        ui.label(egui::RichText::new("• 100% Transparent Amp: Penguatan sinyal murni tanpa saturasi trafo atau perubahan respon frekuensi.\n• Soft-Limit Guard: Dilengkapi algoritma proteksi batas lembut di ambang digital peak agar tidak terjadi distorsi keras (harsh digital clipping).\n• Sangat Ideal untuk Lagu Rekaman Rendah: Mengangkat rekaman vinyl atau master audio klasik yang terlalu pelan.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::AnalogPreamp {
                                        ui.label(egui::RichText::new("Apa itu Analog Preamp & Trim?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Analog Preamp & Trim adalah tahap penguatan gain studio (-24.0 dB s/d +12.0 dB) yang mereplikasi saturasi trafo analog diskrit Class-A.").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(234, 88, 12)));
                                        ui.label(egui::RichText::new("• Transformer Core Saturation: Saat gain dinaikkan di atas 0 dB, gelombang transient instrumen dan vokal dibulatkan dengan kehangatan harmonik musikal.\n• Input Trim Attenuation: Menurunkan level gain hingga -24 dB untuk rekaman modern yang terlalu panas guna memulihkan headroom DAC.\n• Indikator Headroom Ladder: 4-LED meter visual memberi tahu status aman sinyal.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::DynamicLoudness {
                                        ui.label(egui::RichText::new("Apa itu Dynamic Loudness (ISO 226)?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Dynamic Loudness menerapkan kontur kurva equal-loudness standar internasional ISO 226 (Fletcher-Munson) untuk mengompensasi kelemahan sensitivitas pendengaran manusia pada volume rendah.").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("• Bass & Treble Tetap Terdengar Penuh: Mengangkat bass (<120 Hz hingga +8.5 dB) dan treble (>9 kHz hingga +5.0 dB) saat mendengarkan musik di malam hari.\n• Mode Santai Malam Hari: Nikmati musik favorit dengan detail kaya tanpa harus menyetel volume speaker keras-keras.\n• Kendali Kedalaman (Depth 0-100%): Sesuaikan kekuatan kompensasi kontur sesuai headphone atau speaker Anda.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::SteppedAttenuator {
                                        ui.label(egui::RichText::new("Apa itu Stepped Attenuator?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Stepped Attenuator mereplikasi rotary attenuator pasif 64-langkah berbasis resistor ladder presisi tinggi (rentang -60.0 dB hingga 0.0 dB).").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(6, 182, 212)));
                                        ui.label(egui::RichText::new("• Resolusi Diskrit 1.0 dB: Menghilangkan distorsi resolusi dan ketidakseimbangan channel yang sering terjadi pada potensiometer analog murah.\n• Tombol Cepat [DIM -20dB]: Redam volume seketika sebesar 20 dB saat Anda perlu berbicara atau menerima panggilan telepon.\n• Zero Jitter & Bit-Accurate: Sinyal diturunkan dengan perkalian f64 bit-sempurna tanpa truncation noise.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::OpticalLeveler {
                                        ui.label(egui::RichText::new("Apa itu Studio Optical Leveler?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Studio Optical Leveler adalah dynamic leveler legendaris berbasis sel photocell optocoupler (terinspirasi dari kompresor studio Teletronix LA-2A).").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(180, 83, 9)));
                                        ui.label(egui::RichText::new("• Pelepasan 2-Tahap Alami (Two-Stage Release): Pelepasan cepat 50% awal dalam 60 ms, disusul pelepasan lambat bertahap selama beberapa detik. Sangat transparan tanpa efek napas/pumping.\n• Menjinakkan Dinamika Vokal & Akustik: Meratakan lonjakan vokal mendadak sehingga vokal terdengar konsisten dan intim di telinga.\n• Indikator Jarum VU Meter: Jarum meter bergerak secara visual menunjukkan reduksi puncak sinyal.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::StereoBalance {
                                        ui.label(egui::RichText::new("Apa itu Stereo Balance & Trim?").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Stereo Balance & Trim adalah kendali posisi panorama panggung suara stereo menggunakan kurva panning Constant-Power sinus dan kosinus (-100% Kiri s/d +100% Kanan).").size(10.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(11.0).color(Color32::from_rgb(79, 70, 229)));
                                        ui.label(egui::RichText::new("• Daya Akustik Konstan (Constant-Power Panning): Tidak ada penurunan energi suara saat instrumen berada di tengah (Center detent 0 dB).\n• Kompensasi Pendengaran & Speaker Ruangan: Koreksi ketidakseimbangan posisi duduk pendengar atau perbedaan sensitivitas telinga kiri dan kanan.\n• Snapping Center Detent: Slider mudah dikembalikan ke titik imbang tengah 0.0 dB.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else {
                                        ui.label(egui::RichText::new("Karakter Suara Modul").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Modul ini dirancang khusus untuk memproses karakter tonal, kedalaman analog, atau batas keamanan sinyal audio Anda secara real-time.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                                    }
                                }
                                1 => {
                                    // Slide 2 text
                                    if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Di Mana Mixer Boleh Disambungkan?").strong().size(11.5).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Gerbang Penggabung Akhir (Sebelum DAC / Resampler) [Rekomendasi Utama]:\n   Tarik cabang 1 langsung dari DSP ke Pin A (Dry), dan cabang 2 melalui Low-Pass Filter ke Pin B (Wet). Output Mixer disambungkan ke Output DAC.\n\n2. Di Antara Filter Modular (Sub-Bus Mixing):\n   Menggabungkan keluaran dua filter berbeda (misal LPF dan Tube Warmer) sebelum keduanya diteruskan ke Stereo Widener.").size(10.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Fleksibilitas Input:").strong().size(10.5).color(Color32::from_rgb(126, 34, 206)));
                                        ui.label(egui::RichText::new("Jika Anda hanya menyambungkan 1 kabel ke Pin A (Pin B kosong), Mixer otomatis mengalirkan sinyal Channel A tanpa error.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Di Mana Splitter Boleh Disambungkan?").strong().size(11.5).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Percabangan Awal (Diamond Loop) [Rekomendasi Utama]:\n   Colokkan kabel dari App Resampler atau RAM Buffer ke Input Splitter. Tarik Pin Out A langsung ke Mixer (Pin A), dan Pin Out B ke Low-Pass Filter lalu ke Mixer (Pin B). Output Mixer teruskan ke DAC.\n\n2. Multi-Filter Bus Routing:\n   Membagi sinyal untuk diproses dua filter berbeda secara independen sebelum disatukan kembali.").size(10.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Aturan 2 Output:").strong().size(10.5).color(Color32::from_rgb(217, 119, 6)));
                                        ui.label(egui::RichText::new("Berbeda dari kartu biasa yang hanya mengizinkan 1 kabel keluar, Splitter mengizinkan 2 kabel aktif keluar sekaligus (Pin A dan Pin B).").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else {
                                        ui.label(egui::RichText::new("Di Mana Sinyal Boleh Disambungkan?").strong().size(11.5).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Antara RAM Buffer dan DSP Engine (Rekomendasi Utama):\n   Memotong nada tinggi langsung dari hasil decode mentah sebelum sinyal masuk ke 10-band equalizer.\n\n2. Sesudah DSP Engine (Sebelum App Resampler):\n   Menghaluskan puncak frekuensi jika equalizer melakukan boost treble berlebih.\n\n3. Berurutan (In-Series) Sebelum Efek Lain:\n   Pasang sebelum Tube Warmer atau Stereo Widener agar efek analog memproses sinyal yang sudah bersih.").size(10.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Jaminan Kemurnian:").strong().size(10.5).color(Color32::from_rgb(2, 132, 199)));
                                        ui.label(egui::RichText::new("Semua sampel audio dalam RAM Buffer diproses dengan matematika 64-bit float, sehingga penyisipan filter tidak menimbulkan noise lantai maupun distorsi bit.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    }
                                }
                                2 => {
                                    // Slide 3 text
                                    if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Mixer").strong().size(11.5).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Feedback Loop:\n   Jangan menarik kabel dari pin Output Mixer kembali ke pin Input A atau Input B milik Mixer itu sendiri. Hal ini memicu loop tak berujung yang berbahaya bagi speaker.\n\n2. 🚫 Dilarang Sebelum File Source / Sesudah DAC:\n   Mixer adalah pemroses sinyal digital 64-bit float di software, sehingga wajib berada di antara File Source (decoder) dan Output DAC fisik.\n\n3. 🚫 Duplikasi Jalur Identik:\n   Menyambungkan kabel yang sama persis ke Pin A dan Pin B tanpa filter hanya akan melipatgandakan volume tanpa memberikan variasi suara.").size(10.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(10.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika loop terdeteksi atau kabel putus di tengah jalan, sistem secara otomatis membisukan suara untuk melindungi telinga Anda.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Splitter").strong().size(11.5).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Feedback Loop:\n   Jangan menarik kabel dari pin Output A atau B kembali ke pin Input Splitter itu sendiri.\n\n2. 🚫 Dilarang Sebelum File Source / Sesudah DAC:\n   Splitter bekerja pada domain PCM 64-bit di software, sehingga wajib berada di antara decoder berkas dan Output DAC fisik.\n\n3. 🚫 Jangan Menggantung Output Tanpa Tujuan:\n   Jika hanya 1 jalur yang dibutuhkan, gunakan kabel sambungan langsung tanpa Splitter agar kanvas tetap rapi.").size(10.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(10.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika loop terdeteksi atau kabel putus di tengah jalan, sistem secara otomatis membisukan suara untuk melindungi telinga dan speaker Anda.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Bahaya Routing").strong().size(11.5).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Sebelum File Source:\n   File Source membaca bit data mentah berkas dari SSD/HDD (FLAC, MP3, DSD). Filter membutuhkan data PCM yang sudah didecode, sehingga tidak bisa ditaruh sebelum decoder.\n\n2. 🚫 Dilarang Sesudah Output DAC:\n   Output DAC adalah terminal hardware fisik terakhir di laptop/PC Anda (chip ESS Sabre, Realtek, USB DAC) yang langsung mengirim gelombang analog ke kabel jack speaker. Tidak ada modul software yang bisa dipasang sesudahnya.\n\n3. 🚫 Hindari Percabangan Balik (Feedback Loop):\n   Jangan menyambungkan pin keluaran (Output) kembali ke pin masukan (Input) modul yang sama, karena akan menciptakan putaran suara melengking (feedback loop).").size(10.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(10.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika kabel terputus di tengah jalan, LunulaPlayer otomatis membisukan sinyal (silent mute) untuk melindungi speaker dan telinga Anda dari suara ledakan keras (pop/click).").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    }
                                }
                                _ => {
                                    // Slide 4 text
                                    if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Mengatur Crossfader & Mode Audio").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Crossfader Real-Time:\n  Tarik slider langsung di atas kartu kanvas atau di sidebar kanan dari 0% (Channel A murni) hingga 100% (Channel B murni). Nilai diperbarui seketika tanpa latency.\n\n• Posisi Slider Populer:\n  - 0% B: Suara asli tanpa efek (Dry murni).\n  - 30% - 50% B: Suara asli tetap dominan, dengan sentuhan hangat/muffled dari filter B.\n  - 50% / 50%: Campuran seimbang sempurna (Constant-Power Crossfade).\n  - 100% B: Suara efek total (Wet murni).\n\n• Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, seluruh rantai pemrosesan termasuk Mixer dilewati (direct stream 1:1) ke DAC hardware.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Karakteristik Operasional & Mode Audio").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Otomatis & Bebas Penyetelan:\n  Splitter tidak membutuhkan slider atau kenop karena bertugas menduplikasi sinyal 100% secara akurat dan transparan.\n\n• Pasangan Sempurna dengan Mixer (Diamond Loop):\n  - Splitter: Gerbang Pembagi (1 In ➔ 2 Out)\n  - Mixer: Gerbang Penggabung (2 In ➔ 1 Out)\n  Kombinasi keduanya menciptakan arsitektur parallel processing profesional standar studio rekaman!\n\n• Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, seluruh modul termasuk Splitter dilewati (direct stream 1:1) ke DAC hardware.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::VolumeBooster {
                                        ui.label(egui::RichText::new("Mengatur Volume Booster & Mode Audio").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Real-Time (0 s/d +24 dB):\n  Tarik fader langsung di kartu kanvas atau sidebar kanan. Gain bertambah mulus tanpa bunyi 'zipper click'.\n\n• Tombol Cepat Preset (+3dB, +6dB, +12dB, +18dB):\n  Gunakan preset cepat untuk memilih penguatan standar tanpa perlu menggeser manual.\n\n• Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, booster otomatis dilewati demi integritas 1:1 data stream.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::AnalogPreamp {
                                        ui.label(egui::RichText::new("Mengatur Preamp & Saturasi Class-A").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Dial Putar & Slider (-24 s/d +12 dB):\n  Putar dial aluminium atau gunakan slider trim. Titik tengah 0 dB merupakan detent netral (unity gain).\n\n• Preset Studio Cepat:\n  - -6dB Trim: Mengamankan lagu modern dari inter-sample peak.\n  - 0dB Unity: Penyangga linier murni.\n  - +3dB / +6dB: Mengaktifkan saturasi trafo analog Class-A untuk vokal hangat.\n\n• Mode Bit-Perfect:\n  Preamp otomatis di-bypass saat Bit-Perfect diaktifkan.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::DynamicLoudness {
                                        ui.label(egui::RichText::new("Mengatur Kontur Loudness ISO 226").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Kedalaman (0% s/d 100%):\n  Atur intensitas senyum kurva frekuensi dari 0% (datar) hingga 100% (penuh).\n\n• Rekomendasi Penggunaan:\n  - 25% Mild: Untuk speaker desktop jarak dekat (near-field).\n  - 50% Night: Rekomendasi standar mendengarkan musik santai di malam hari.\n  - 75% - 100%: Untuk headphone berkarakter dingin/tipis atau rekaman lama yang minim bass.\n\n• Mode Bit-Perfect:\n  Kompensasi loudness otomatis dilewati saat beralih ke Bit-Perfect.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::SteppedAttenuator {
                                        ui.label(egui::RichText::new("Mengatur Stepped Attenuator & DIM").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Stepped Slider (-60 dB s/d 0 dB):\n  Melompat presisi 1.0 dB per step, meniru klik mekanis resistor ladder audiophile.\n\n• Tombol Cepat [DIM -20dB]:\n  Klik tombol DIM langsung di kartu kanvas atau sidebar untuk menurunkan level ke -20 dB seketika, dan klik lagi untuk kembali ke volume awal.\n\n• Mode Bit-Perfect:\n  Saat mode Bit-Perfect aktif, seluruh peredaman digital dilewati murni ke DAC.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::OpticalLeveler {
                                        ui.label(egui::RichText::new("Mengatur Peak Reduction LA-2A").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Peak Reduction (0% s/d 100%):\n  Mengatur ambang sensitivitas kompresi optik. Semakin tinggi persentase, semakin intim dan rata dinamika suaranya.\n\n• Rekomendasi Preset:\n  - 20% Light: Leveling transparan untuk musik orkestra dan jazz.\n  - 40% Classic: Karakter LA-2A klasik untuk vokal pop dan akustik.\n  - 65% Heavy: Mengunci vokal di depan panggung suara (in-your-face vocal).\n\n• Mode Bit-Perfect:\n  Leveler otomatis di-bypass saat beralih ke Bit-Perfect.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::StereoBalance {
                                        ui.label(egui::RichText::new("Mengatur Panning Stereo & Center Detent").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Panorama (-100% Kiri s/d +100% Kanan):\n  Menggeser panggung suara dengan kurva daya konstan tanpa perubahan level akustik total.\n\n• Tombol Instan Center:\n  Klik tombol '● Center' di sidebar untuk mengembalikan posisi instrumen ke titik tengah 0.0 dB secara instan.\n\n• Mode Bit-Perfect:\n  Panning stereo dilewati murni saat mode Bit-Perfect diaktifkan.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else {
                                        ui.label(egui::RichText::new("Mengatur Slider & Mode Audio").strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Geser Slider Cutoff:\n  Anda dapat menggeser slider langsung di atas kartu kanvas dari 100 Hz hingga 10 kHz. Perubahan frekuensi terjadi seketika tanpa jeda dan tanpa perlu me-rebuild graph sinyal.\n\n• Perilaku pada Mode Bit-Perfect:\n  Mode Bit-Perfect mewajibkan data lagu dikirim ke DAC murni 1:1 tanpa modifikasi 1 bit pun. Oleh karena itu, saat mode Bit-Perfect aktif, seluruh filter kustom otomatis DILEWATI (bypassed) melalui kawat jumper abu-abu langsung.\n\n• Cara Mengaktifkan Efek:\n  Cukup klik kartu DSP ENGINE pada kanvas atau tekan tombol toggle mode ke 'DSP Active'. Filter akan seketika kembali memproses suara.").size(10.0).color(Color32::from_rgb(51, 65, 85)));
                                    }
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // 5. Footer Carousel Navigation & Action Buttons
                        let mut add_to_canvas = false;
                        ui.horizontal(|ui| {
                            // Previous Slide button
                            let can_prev = step > 0;
                            if ui.add_enabled(can_prev, egui::Button::new("◀ Sebelumnya")).clicked() {
                                if self.filter_tutorial_step > 0 {
                                    self.filter_tutorial_step -= 1;
                                }
                            }

                            // Dot carousel indicators (clickable)
                            ui.add_space(4.0);
                            for d in 0..4 {
                                let is_current = step == d;
                                let dot_text = if is_current { "●" } else { "○" };
                                let dot_col = if is_current { theme_color } else { Color32::from_rgb(203, 213, 225) };
                                if ui.add(egui::Button::new(egui::RichText::new(dot_text).size(12.0).color(dot_col)).frame(false)).clicked() {
                                    self.filter_tutorial_step = d;
                                }
                            }
                            ui.add_space(4.0);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if step >= 3 {
                                    if ui.button(egui::RichText::new("Selesai ✓ Tutup").strong().color(Color32::from_rgb(16, 185, 129))).clicked() {
                                        self.active_filter_tutorial = None;
                                    }
                                } else {
                                    if ui.button(egui::RichText::new("Lanjut ▶").strong()).clicked() {
                                        self.filter_tutorial_step += 1;
                                    }
                                }

                                if !is_added {
                                    if ui.button(egui::RichText::new("➕ Pasang ke Kanvas").strong().color(theme_color)).clicked() {
                                        add_to_canvas = true;
                                    }
                                } else {
                                    ui.label(egui::RichText::new("✓ Terpasang").size(10.0).color(Color32::from_rgb(16, 185, 129)));
                                }
                            });
                        });

                        if add_to_canvas {
                            self.add_modular_filter(filter_type);
                            self.active_filter_tutorial = None;
                        }
                    });
            });
    }
}
