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

impl LunulaApp {
    // Reset layout of nodes and center canvas view
    pub fn reset_patchbay_layout(&mut self, canvas_width: f32, canvas_height: f32) {
        self.patchbay_pan = Vec2::ZERO;
        let node_width = 136.0;
        let margin = 20.0;
        let available = (canvas_width - margin * 2.0 - 5.0 * node_width).max(12.0 * 4.0);
        let spacing = (available / 4.0).clamp(16.0, 44.0);
        let center_y = (canvas_height * 0.40 - 27.0).max(36.0);

        for (i, node) in self.nodes.iter_mut().take(5).enumerate() {
            node.pos = Pos2::new(margin + i as f32 * (node_width + spacing), center_y);
        }
    }

    // Add a new modular filter node to the canvas
    pub fn add_modular_filter(&mut self, node_type: NodeType) {
        let id = self.next_node_id;
        self.next_node_id += 1;

        let (title, sub1, sub2) = match &node_type {
            NodeType::TubeWarmer => ("TUBE WARMER".to_string(), "2nd Harmonic".to_string(), "Analog Warmth".to_string()),
            NodeType::StereoWidener => ("STEREO WIDENER".to_string(), "Haas Effect 3D".to_string(), "+35% Soundstage".to_string()),
            NodeType::BassGuard => ("BASS GUARD".to_string(), "Sub-sonic Limiter".to_string(), "Protects Woofer".to_string()),
            NodeType::VolumeAttenuator => ("ATTENUATOR".to_string(), "Volume -6 dB".to_string(), "Passive Pad".to_string()),
            NodeType::SavedEqPreset(name) => ("EQ PRESET".to_string(), name.clone(), "Biquad Peaking".to_string()),
            _ => ("AUDIO FILTER".to_string(), "DSP Unit".to_string(), "Active".to_string()),
        };

        // Spawn in visible center of canvas relative to pan
        let spawn_x = 280.0 - self.patchbay_pan.x + (id as f32 * 16.0) % 80.0;
        let spawn_y = 120.0 - self.patchbay_pan.y + (id as f32 * 20.0) % 80.0;
        let new_node = ModularNode {
            id,
            node_type,
            title,
            sub1,
            sub2,
            pos: Pos2::new(spawn_x, spawn_y),
            is_removable: true,
        };

        self.nodes.push(new_node);
        self.selected_node = Some(id);
    }

    // Interactive Modular Patchbay Canvas with Pan, Drag, and Connect/Disconnect Cables
    pub fn render_modular_patchbay_canvas(&mut self, ui: &mut egui::Ui, canvas_size: Vec2) {
        let (canvas_rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
        let painter = ui.painter().with_clip_rect(canvas_rect);

        // Canvas Background
        painter.rect_filled(canvas_rect, 10.0, Color32::from_rgb(248, 250, 252));
        painter.rect_stroke(canvas_rect, 10.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

        // Panning with Mouse Wheel / Trackpad Scroll
        let scroll = ui.input(|i| i.smooth_scroll_delta);
        if response.hovered() && (scroll.x.abs() > 0.0 || scroll.y.abs() > 0.0) {
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

        let node_width = 136.0;
        let node_height = 54.0;
        let pin_radius = 5.0;

        let mouse_pos = response.interact_pointer_pos();
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

        // Check if cursor is hovering over any node rect
        let is_hovering_any_node = self.nodes.iter().any(|node| {
            let r = Rect::from_min_size(
                Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                    canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                ),
                Vec2::new(node_width, node_height),
            );
            mouse_pos.map_or(false, |p| r.contains(p))
        });

        // 0. Right-Click or Middle-Click Hold to Pan Canvas (when on empty background canvas)
        if right_or_middle_pressed && response.hovered() && !is_hovering_any_node {
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
        if response.drag_started() && is_left_down && !is_right_down && !is_middle_down {
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
                    let out_pin_pos = Pos2::new(world_rect.max.x, world_rect.center().y);
                    if pos.distance(out_pin_pos) <= 12.0 {
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
                            self.dragging_node = Some(i);
                            self.selected_node = Some(node.id);
                            break;
                        }
                    }

                    // If clicked on empty canvas background, also allow panning canvas
                    if self.dragging_node.is_none() {
                        self.is_panning_canvas = true;
                    }
                }
            }
        }

        // 2. Handle Node Dragging OR Canvas Background Panning (Left Click)
        if is_mouse_down && is_left_down && self.connecting_from_node.is_none() {
            let delta = ui.input(|i| i.pointer.delta());
            if let Some(drag_idx) = self.dragging_node {
                let node = &mut self.nodes[drag_idx];
                node.pos += delta;
            } else if self.is_panning_canvas {
                self.patchbay_pan += delta;
            }
        }

        // 3. Handle Cable Connection Release on an Input Pin
        if response.drag_stopped() {
            if let (Some(from_id), Some(pos)) = (self.connecting_from_node, mouse_pos) {
                for node in &self.nodes {
                    if node.id != from_id {
                        let world_rect = Rect::from_min_size(
                            Pos2::new(
                                canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                            ),
                            Vec2::new(node_width, node_height),
                        );
                        let in_pin_pos = Pos2::new(world_rect.min.x, world_rect.center().y);
                        if pos.distance(in_pin_pos) <= 16.0 {
                            // Check if connection already exists
                            let already_connected = self.connections.iter().any(|c| c.from_node == from_id && c.to_node == node.id);
                            if !already_connected {
                                self.connections.push(CableConnection {
                                    from_node: from_id,
                                    to_node: node.id,
                                });
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

        let _glow_color = match self.chain_mode {
            ChainMode::BitPerfect => Color32::from_rgb(16, 185, 129),
            ChainMode::DspActive => Color32::from_rgb(99, 102, 241),
            ChainMode::OsResampled => Color32::from_rgb(245, 158, 11),
        };

        // 4. Render Established Patchbay Cables (with click to disconnect)
        for (conn_idx, conn) in self.connections.iter().enumerate() {
            let from_node = self.nodes.iter().find(|n| n.id == conn.from_node);
            let to_node = self.nodes.iter().find(|n| n.id == conn.to_node);

            if let (Some(from_n), Some(to_n)) = (from_node, to_node) {
                let p_start = Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + node_width,
                    canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + node_height * 0.5,
                );
                let p_end = Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + to_n.pos.x,
                    canvas_rect.min.y + self.patchbay_pan.y + to_n.pos.y + node_height * 0.5,
                );

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

                let theme_color = match self.chain_mode {
                    ChainMode::BitPerfect => Color32::from_rgb(16, 185, 129),
                    ChainMode::DspActive => Color32::from_rgb(99, 102, 241),
                    ChainMode::OsResampled => Color32::from_rgb(245, 158, 11),
                };

                let cable_color = if is_hover_cable {
                    Color32::from_rgb(239, 68, 68) // Red on hover to cut
                } else if is_flowing_cable {
                    theme_color
                } else {
                    Color32::from_rgb(239, 68, 68) // Red when severed/dead end
                };

                // Drop shadow
                for seg in points.windows(2) {
                    painter.line_segment([seg[0] + Vec2::new(0.0, 1.5), seg[1] + Vec2::new(0.0, 1.5)], Stroke::new(3.0, Color32::from_black_alpha(18)));
                }

                // Cable path
                let stroke_w = if is_hover_cable { 2.8 } else if is_flowing_cable { 2.3 } else { 1.8 };
                for seg in points.windows(2) {
                    painter.line_segment([seg[0], seg[1]], Stroke::new(stroke_w, cable_color));
                }

                // If flowing and song is playing: draw live animated audio data particles traveling along the cable!
                if is_flowing_cable && self.is_playing {
                    let particle_glow = match self.chain_mode {
                        ChainMode::BitPerfect => Color32::from_rgba_premultiplied(167, 243, 208, 200),
                        ChainMode::DspActive => Color32::from_rgba_premultiplied(199, 210, 254, 200),
                        ChainMode::OsResampled => Color32::from_rgba_premultiplied(253, 230, 138, 200),
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
                    let badge_text = if self.chain_mode == ChainMode::OsResampled {
                        "⚡ 48kHz OS MIXER"
                    } else {
                        "⚡ DIRECT BYPASS"
                    };
                    let badge_rect = Rect::from_center_size(mid - Vec2::new(0.0, 9.0), Vec2::new(94.0, 15.0));
                    painter.rect_filled(badge_rect, 4.0, Color32::from_rgba_premultiplied(16, 185, 129, 235));
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
        let standard_pairs: Vec<(usize, usize)> = match self.chain_mode {
            ChainMode::BitPerfect | ChainMode::OsResampled => vec![(1, 2), (2, 4), (4, 5)],
            ChainMode::DspActive => vec![(1, 2), (2, 3), (3, 4), (4, 5)],
        };
        for (from_id, to_id) in standard_pairs {
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

        // Remove disconnected cable
        if let Some(idx) = connection_to_remove {
            self.connections.remove(idx);
        }

        // 5. Draw Live Rubber-Band Cable while Dragging
        if let (Some(from_id), Some(pos)) = (self.connecting_from_node, mouse_pos) {
            if let Some(from_n) = self.nodes.iter().find(|n| n.id == from_id) {
                let p_start = Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + node_width,
                    canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + node_height * 0.5,
                );
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

                for seg in points.windows(2) {
                    painter.line_segment([seg[0], seg[1]], Stroke::new(2.5, Color32::from_rgb(14, 165, 233)));
                }

                painter.circle_filled(p_end, 5.0, Color32::from_rgb(14, 165, 233));
            }
        }

        // 6. Render Nodes & Input/Output Patch Pins with Dynamic Outline Animation
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
            node_resp.context_menu(|ui| {
                ui.label(egui::RichText::new(&node.title).strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                ui.label(egui::RichText::new(&node.sub1).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                ui.separator();
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
                if node.node_type == NodeType::Dsp {
                    let label = if self.chain_mode == ChainMode::DspActive {
                        "⚡ Bypass DSP (Direct Bit-Perfect)"
                    } else {
                        "⚡ Aktifkan DSP Processing"
                    };
                    if ui.button(label).clicked() {
                        toggle_dsp = true;
                        ui.close();
                    }
                }
            });

            // Shadow
            painter.rect_filled(node_rect.translate(Vec2::new(0.0, 2.0 + hover_anim * 1.5)), 8.0, Color32::from_black_alpha(14));

            let theme_color = match self.chain_mode {
                ChainMode::BitPerfect => Color32::from_rgb(16, 185, 129),
                ChainMode::DspActive => Color32::from_rgb(99, 102, 241),
                ChainMode::OsResampled => Color32::from_rgb(245, 158, 11),
            };

            let is_bypassed_dsp = node.node_type == NodeType::Dsp
                && (self.chain_mode == ChainMode::BitPerfect || self.chain_mode == ChainMode::OsResampled);

            let bg = if is_dragging {
                Color32::from_rgb(241, 245, 249)
            } else if is_bypassed_dsp {
                Color32::from_rgba_premultiplied(248, 250, 252, 220)
            } else if !is_flowing {
                Color32::from_rgb(255, 245, 245)
            } else {
                Color32::WHITE
            };

            painter.rect_filled(node_rect, 8.0, bg);

            // Dynamic Outline Styling & Animation
            if is_bypassed_dsp {
                // Option B: DSP is BYPASSED — calm muted slate border & direct dashed wire inside
                painter.rect_stroke(node_rect, 8.0, Stroke::new(1.2, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                
                // Dashed direct jumper wire through the center of DSP
                let pin_l = Pos2::new(node_rect.min.x + 8.0, node_rect.center().y);
                let pin_r = Pos2::new(node_rect.max.x - 8.0, node_rect.center().y);
                for d in 0..10 {
                    if d % 2 == 0 {
                        let p1 = pin_l.lerp(pin_r, d as f32 / 10.0);
                        let p2 = pin_l.lerp(pin_r, (d + 1) as f32 / 10.0);
                        painter.line_segment([p1, p2], Stroke::new(1.2, Color32::from_rgb(203, 213, 225)));
                    }
                }
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

            // Title
            let title_col = if is_bypassed_dsp {
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

            // Sublabels: Dynamic based on bypassed vs flowing vs broken
            let (sub1_txt, sub1_col, sub2_txt, sub2_col) = if is_bypassed_dsp {
                ("BYPASSED (Direct Wire)", Color32::from_rgb(148, 163, 184), "0.0 dB Pure Master", Color32::from_rgb(148, 163, 184))
            } else if is_flowing {
                (&node.sub1[..], theme_color, &node.sub2[..], Color32::from_rgb(100, 116, 139))
            } else {
                ("NO SIGNAL", Color32::from_rgb(239, 68, 68), "Terputus", Color32::from_rgb(220, 38, 38))
            };

            painter.text(
                Pos2::new(node_rect.center().x, node_rect.min.y + 27.5),
                egui::Align2::CENTER_CENTER,
                sub1_txt,
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

            // Status Indicator Dot (Top-Right)
            let status_dot_pos = Pos2::new(node_rect.max.x - 8.0, node_rect.min.y + 8.0);
            if is_bypassed_dsp {
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
                let in_pin = Pos2::new(node_rect.min.x, node_rect.center().y);
                let is_hover_in = mouse_pos.map_or(false, |p| p.distance(in_pin) <= 10.0);
                if is_hover_in {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                }
                let pin_col = if is_hover_in {
                    Color32::from_rgb(14, 165, 233)
                } else if is_bypassed_dsp {
                    Color32::from_rgb(203, 213, 225)
                } else if is_flowing {
                    theme_color
                } else {
                    Color32::from_rgb(239, 68, 68)
                };
                painter.circle_filled(in_pin, pin_radius, pin_col);
                painter.circle_stroke(in_pin, pin_radius, Stroke::new(1.5, Color32::WHITE));
            }

            // Output Pin (Right) - Except Endpoint node
            if node.node_type != NodeType::Endpoint {
                let out_pin = Pos2::new(node_rect.max.x, node_rect.center().y);
                let is_hover_out = mouse_pos.map_or(false, |p| p.distance(out_pin) <= 10.0);
                if is_hover_out {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                }
                let pin_col = if is_hover_out {
                    Color32::from_rgb(14, 165, 233)
                } else if is_bypassed_dsp {
                    Color32::from_rgb(203, 213, 225)
                } else if is_flowing {
                    theme_color
                } else {
                    Color32::from_rgb(239, 68, 68)
                };
                painter.circle_filled(out_pin, pin_radius, pin_col);
                painter.circle_stroke(out_pin, pin_radius, Stroke::new(1.5, Color32::WHITE));
            }

            // Remove Button for Custom Filter Nodes
            if node.is_removable {
                let del_btn_center = Pos2::new(node_rect.max.x - 9.0, node_rect.min.y + 9.0);
                let is_hover_del = mouse_pos.map_or(false, |p| p.distance(del_btn_center) <= 8.0);
                if is_hover_del {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    if response.clicked() {
                        node_to_delete = Some(node.id);
                    }
                }
                let del_color = if is_hover_del { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(203, 213, 225) };
                painter.circle_filled(del_btn_center, 6.0, del_color);
                painter.text(del_btn_center, egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(8.0), Color32::WHITE);
            }
        }

        // 7. Interactive Bottom Status Pill
        let banner_rect = Rect::from_min_size(
            Pos2::new(canvas_rect.min.x + 12.0, canvas_rect.max.y - 25.0),
            Vec2::new(canvas_rect.width() - 24.0, 19.0),
        );
        let (pill_bg, pill_stroke, status_icon, status_msg, text_col) = if chain_eval.is_complete {
            if self.is_playing {
                match self.chain_mode {
                    ChainMode::BitPerfect => (
                        Color32::from_rgba_premultiplied(236, 253, 245, 230),
                        Stroke::new(1.0, Color32::from_rgb(167, 243, 208)),
                        "🟢",
                        "JALUR AUDIO AKTIF (BIT-PERFECT) — Melompati DSP, sinyal mengalir murni bit-per-bit ke DAC",
                        Color32::from_rgb(6, 95, 70),
                    ),
                    ChainMode::DspActive => (
                        Color32::from_rgba_premultiplied(238, 242, 255, 230),
                        Stroke::new(1.0, Color32::from_rgb(199, 210, 254)),
                        "🟣",
                        "JALUR AUDIO AKTIF (DSP ACTIVE) — 10-Band Peaking EQ & FX aktif dalam presisi 32-bit Float",
                        Color32::from_rgb(67, 56, 202),
                    ),
                    ChainMode::OsResampled => (
                        Color32::from_rgba_premultiplied(254, 243, 199, 230),
                        Stroke::new(1.0, Color32::from_rgb(253, 230, 138)),
                        "🟡",
                        "JALUR AUDIO AKTIF (OS RESAMPLED) — Resampling ke 48.0 kHz via Windows Audio Mixer Shared",
                        Color32::from_rgb(146, 64, 14),
                    ),
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
            }
        }

        // Disconnect all cables connected to this node
        if let Some(d_id) = disconnect_node {
            self.connections.retain(|c| c.from_node != d_id && c.to_node != d_id);
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
        }
    }

    // Right Sidebar Filter & FX Toolbox Palette
    pub fn render_fx_sidebar_palette(&mut self, ui: &mut egui::Ui) {
        let mut delete_filter_type: Option<NodeType> = None;

        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("🎛️ FILTER & FX PALETTE").strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                ui.label(egui::RichText::new("Klik kiri untuk menambah • Klik kanan untuk opsi hapus").size(11.0).color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Filter 1: Tube Warmer
                    let has_tube = self.nodes.iter().any(|n| n.node_type == NodeType::TubeWarmer);
                    let g1 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("🔥 Tube Warmer (Analog)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                            ui.label(egui::RichText::new("Harmonik ganjil-genap lampu tabung vintage untuk suara vokal tebal.").size(10.5).color(Color32::from_rgb(100, 116, 139)));
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
                    ui.add_space(8.0);

                    // Filter 2: Spatial Stereo Widener
                    let has_widener = self.nodes.iter().any(|n| n.node_type == NodeType::StereoWidener);
                    let g2 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("🌐 Stereo 3D Widener").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                            ui.label(egui::RichText::new("Memperlebar soundstage instrumen menggunakan algoritma Haas delay.").size(10.5).color(Color32::from_rgb(100, 116, 139)));
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
                    ui.add_space(8.0);

                    // Filter 3: Bass Guard Limiter
                    let has_bass = self.nodes.iter().any(|n| n.node_type == NodeType::BassGuard);
                    let g3 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("🛡️ Bass Guard & Limiter").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                            ui.label(egui::RichText::new("Mencegah distorsi sub-bass berlebih dan melindungi driver headphone.").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
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
                        ui.label(egui::RichText::new("🛡️ Bass Guard & Limiter").strong());
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

                    // Filter 4: Volume Attenuator (Low Volume Pad)
                    let has_vol = self.nodes.iter().any(|n| n.node_type == NodeType::VolumeAttenuator);
                    let g4 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("🔉 Volume Attenuator (-6dB)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                            ui.label(egui::RichText::new("Peredam gain pasif untuk mendengarkan di malam hari (Late-night low vol).").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
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
                        ui.label(egui::RichText::new("🔉 Volume Attenuator (-6dB)").strong());
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

                    // Filter 5: Saved EQ Presets
                    ui.label(egui::RichText::new("SAVED EQUALIZERS").strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));
                    ui.add_space(4.0);

                    let presets = ["Vocal Clarity EQ", "Bass Boost Matrix", "Air & Sparkle EQ"];
                    for preset in presets {
                        let preset_type = NodeType::SavedEqPreset(preset.to_string());
                        let has_eq = self.nodes.iter().any(|n| n.node_type == preset_type);

                        let row = ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(preset).strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if has_eq {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus preset ini dari canvas").clicked() {
                                        delete_filter_type = Some(preset_type.clone());
                                    }
                                }
                                if ui.button(egui::RichText::new("➕").strong()).on_hover_text("Pasang ke diagram").clicked() {
                                    self.add_modular_filter(preset_type.clone());
                                }
                            });
                        });
                        row.response.context_menu(|ui| {
                            ui.label(egui::RichText::new(preset).strong());
                            ui.separator();
                            if ui.button("➕ Pasang ke Diagram").clicked() {
                                self.add_modular_filter(preset_type.clone());
                                ui.close();
                            }
                            if has_eq {
                                if ui.button("🗑️ Hapus dari Canvas").clicked() {
                                    delete_filter_type = Some(preset_type.clone());
                                    ui.close();
                                }
                            }
                        });
                        ui.separator();
                    }
                });
            });
        });

        // Execute batch filter deletion
        if let Some(nt) = delete_filter_type {
            let ids: Vec<usize> = self.nodes.iter().filter(|n| n.node_type == nt && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            if let Some(sel) = self.selected_node {
                if ids.contains(&sel) {
                    self.selected_node = None;
                }
            }
        }
    }
}
