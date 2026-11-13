use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{CableConnection, ChainMode, ModularNode, NodeType, SignalNodeStatus};
use crate::app::LunulaApp;
use crate::audio::NodeSafetyStatus;
use crate::ui::components::draw_five_finger_hand;
use crate::ui::patchbay::types::*;

impl LunulaApp {
    // Reset layout of nodes and center canvas view with spacious, non-overlapping 2-deck studio design
    pub fn reset_patchbay_layout(&mut self, canvas_width: f32, canvas_height: f32) {
        self.patchbay_pan = Vec2::ZERO;
        let node_width = 198.0;
        let gap_x = 42.0;
        let needed_w = 4.0 * node_width + 3.0 * gap_x + 60.0;

        // Auto-fit zoom if the canvas is narrower than the full 4-node highway chain
        if canvas_width > 200.0 && canvas_width < needed_w {
            self.patchbay_zoom = (canvas_width / needed_w).clamp(0.65, 1.0);
        } else {
            self.patchbay_zoom = 1.0;
        }

        let effective_w = if self.patchbay_zoom > 0.01 { canvas_width / self.patchbay_zoom } else { canvas_width };
        let margin_x = ((effective_w - (4.0 * node_width + 3.0 * gap_x)) * 0.5).max(30.0);
        let top_y = (canvas_height * 0.18).clamp(40.0, 70.0);
        let bot_y = top_y + 145.0;

        for node in self.nodes.iter_mut() {
            match node.id {
                1 => node.pos = Pos2::new(margin_x, top_y),
                2 => node.pos = Pos2::new(margin_x + (node_width + gap_x), top_y),
                3 => node.pos = Pos2::new(margin_x + (node_width + gap_x) * 1.5, bot_y),
                4 => node.pos = Pos2::new(margin_x + (node_width + gap_x) * 2.0, top_y),
                5 => node.pos = Pos2::new(margin_x + (node_width + gap_x) * 3.0, top_y),
                _ => {}
            }
        }
    }

    // Add a new modular filter node to the canvas
    pub fn add_modular_filter(&mut self, node_type: NodeType) {
        // Block modules that are currently Coming Soon
        if matches!(node_type, NodeType::VolumeAttenuator | NodeType::BassGuard) {
            return;
        }

        let id = self.next_node_id;
        self.next_node_id += 1;

        let (title, sub1, sub2) = match &node_type {
            NodeType::TubeWarmer => ("🔥 TUBE WARMER".to_string(), "Warmth 35%".to_string(), "Triode 12AX7".to_string()),
            NodeType::StereoWidener => ("🌐 STEREO WIDENER".to_string(), "Haas Effect 3D".to_string(), "+35% Soundstage".to_string()),
            NodeType::BassGuard => ("🛡️ BASS GUARD".to_string(), "Sub-sonic Limiter".to_string(), "Protects Woofer".to_string()),
            NodeType::VolumeAttenuator => ("🔉 ATTENUATOR".to_string(), "Volume -6 dB".to_string(), "Passive Pad".to_string()),
            NodeType::LowPassFilter => ("🌊 LOW-PASS FILTER".to_string(), "Cutoff 800 Hz".to_string(), "Slope 12dB • Q 0.71".to_string()),
            NodeType::HighPassFilter => ("🏔️ HIGH-PASS FILTER".to_string(), "Cutoff 80 Hz".to_string(), "Slope 12dB • Q 0.71".to_string()),
            NodeType::BandPassFilter => ("🎯 BAND-PASS FILTER".to_string(), "Center 1.0 kHz".to_string(), "Slope 12dB • Q 1.41".to_string()),
            NodeType::Crossover => ("✂️ 2-WAY CROSSOVER".to_string(), "X-Over 500 Hz".to_string(), "LR4 (24 dB/oct)".to_string()),
            NodeType::Mixer => ("🎚️ MIXER / SUMMING".to_string(), "Mix: 50% A • 50% B".to_string(), "Dual-Bus Gerbang".to_string()),
            NodeType::Splitter => ("🔀 SPLITTER / Y-BRANCH".to_string(), "Dual-Out (Pin A & B)".to_string(), "Dual-Bus Gerbang".to_string()),
            NodeType::SavedEqPreset(name) => ("🎛️ EQ PRESET".to_string(), name.clone(), "Klik untuk Buka".to_string()),
            NodeType::VolumeBooster => ("⚡ VOLUME BOOSTER".to_string(), "+6.0 dB Gain".to_string(), "Pure Clean Amp".to_string()),
            NodeType::AnalogPreamp => ("📻 ANALOG PREAMP".to_string(), "0.0 dB Drive".to_string(), "Class-A Warmth".to_string()),
            NodeType::DynamicLoudness => ("🌙 DYNAMIC LOUDNESS".to_string(), "Loudness 50%".to_string(), "ISO 226 Contour".to_string()),
            NodeType::SteppedAttenuator => ("🪜 STEPPED ATTENUATOR".to_string(), "-12 dB".to_string(), "64-Step Resistor".to_string()),
            NodeType::OpticalLeveler => ("🎚️ OPTICAL LEVELER".to_string(), "Peak Red 30%".to_string(), "LA-2A Opto".to_string()),
            NodeType::StereoBalance => ("⚖️ STEREO BALANCE".to_string(), "Center 0.0 dB".to_string(), "Pan & Trim".to_string()),
            NodeType::TimeAlignment => ("⏱️ TIME & PHASE ALIGN".to_string(), "Delay 0 µs".to_string(), "Phase 0° (Normal)".to_string()),
            NodeType::StereoMatrix => ("🌐 STEREO MATRIX".to_string(), "Stereo Direct".to_string(), "Bauer / MS / Swap".to_string()),
            _ => ("AUDIO FILTER".to_string(), "DSP Unit".to_string(), "Active".to_string()),
        };

        // Spawn in visible center of canvas relative to pan
        let spawn_x = 280.0 - self.patchbay_pan.x + (id as f32 * 16.0) % 80.0;
        let spawn_y = 120.0 - self.patchbay_pan.y + (id as f32 * 20.0) % 80.0;
        let param_val = match &node_type {
            NodeType::LowPassFilter => 800.0,
            NodeType::HighPassFilter => 80.0,
            NodeType::BandPassFilter => 1000.0,
            NodeType::Crossover => 500.0,
            NodeType::Mixer => 0.5,
            NodeType::Splitter => 0.5,
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
            NodeType::TimeAlignment => 0.0,
            NodeType::StereoMatrix => 0.0,
            _ => 0.0,
        };
        let new_node = ModularNode {
            id,
            node_type: node_type.clone(),
            title,
            sub1,
            sub2,
            pos: Pos2::new(spawn_x, spawn_y),
            is_removable: true,
            param_val,
        };

        // For Mixer/Splitter: pre-register independent satellite world-coords at spawn
        if matches!(node_type, NodeType::Mixer | NodeType::Splitter) {
            if node_type == NodeType::Mixer {
                self.satellite_positions.insert((id, 0), Pos2::new(spawn_x - 90.0, spawn_y - 10.0));
                self.satellite_positions.insert((id, 1), Pos2::new(spawn_x - 90.0, spawn_y + 44.0));
            } else {
                self.satellite_positions.insert((id, 0), Pos2::new(spawn_x + 155.0, spawn_y - 10.0));
                self.satellite_positions.insert((id, 1), Pos2::new(spawn_x + 155.0, spawn_y + 44.0));
            }
        }
        self.nodes.push(new_node);
        self.selected_node = Some(id);
        self.invalidate_chain_cache();
        self.sync_signal_chain_to_engine();
    }

    // Interactive Modular Patchbay Canvas with Pan, Drag, and Connect/Disconnect Cables
    pub fn render_modular_patchbay_canvas(&mut self, ui: &mut egui::Ui, canvas_size: Vec2) {
        let (canvas_rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
        let base_painter = ui.painter().with_clip_rect(canvas_rect);

        // Canvas Background (drawn unscaled to fill exact canvas_rect)
        base_painter.rect_filled(canvas_rect, 10.0, Color32::from_rgb(248, 250, 252));
        base_painter.rect_stroke(canvas_rect, 10.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

        let is_modal_blocking = self.is_modal_or_overlay_active();
        let raw_pointer_pos = response.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.latest_pos()));

        // Panning with Mouse Wheel / Trackpad Scroll OR Zoom with Ctrl + Scroll
        let mut is_ctrl = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
        let pinch_zoom: f32 = ui.input(|i| i.zoom_delta());
        let smooth_scroll = ui.input(|i| i.smooth_scroll_delta);

        let mut raw_wheel_delta = Vec2::ZERO;
        let mut has_raw_wheel = false;
        ui.input(|i| {
            for ev in &i.raw.events {
                if let egui::Event::MouseWheel { delta, modifiers, .. } = ev {
                    raw_wheel_delta += *delta;
                    has_raw_wheel = true;
                    if modifiers.ctrl || modifiers.command {
                        is_ctrl = true;
                    }
                }
            }
        });
        let is_interacting_with_tutorial = if self.show_patchbay_tutorial || self.active_filter_tutorial.is_some() {
            let padding = 12.0;
            let card_w = 440.0f32.min(canvas_rect.width() - 24.0);
            let card_h = 510.0f32.min(canvas_rect.height() - 24.0);
            let r = Rect::from_min_size(
                Pos2::new(canvas_rect.max.x - card_w - padding, canvas_rect.min.y + padding),
                Vec2::new(card_w, card_h),
            );
            raw_pointer_pos.map_or(false, |p| r.contains(p))
        } else {
            false
        };

        let is_interacting_with_saved_eq_popup = if self.show_saved_eq_palette_popup {
            let screen_rect = ui.ctx().input(|i| i.raw.screen_rect).unwrap_or_else(|| ui.clip_rect());
            let popup_w = 400.0;
            let popup_h = 440.0;
            let (sidebar_left, anchor_y) = if let Some(a) = self.saved_eq_popup_anchor {
                (a.x, a.y)
            } else {
                (screen_rect.max.x - 300.0, screen_rect.center().y)
            };
            let mut x = sidebar_left - popup_w - 12.0;
            if x < screen_rect.min.x + 12.0 {
                x = screen_rect.min.x + 12.0;
            }
            let min_y = screen_rect.min.y + 44.0;
            let max_y = (screen_rect.max.y - popup_h - 16.0).max(min_y);
            let ideal_y = anchor_y - popup_h * 0.5;
            let y = ideal_y.clamp(min_y, max_y);
            let popup_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(popup_w, popup_h));
            raw_pointer_pos.map_or(false, |p| popup_rect.contains(p))
        } else {
            false
        };

        let is_interacting_with_clipping_popup = if self.show_clipping_settings_popup {
            let popup_w = 390.0;
            let popup_h = 320.0;
            let x = (canvas_rect.max.x - popup_w - 16.0).max(canvas_rect.min.x + 8.0);
            let y = canvas_rect.min.y + 16.0;
            let popup_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(popup_w, popup_h));
            raw_pointer_pos.map_or(false, |p| popup_rect.contains(p))
        } else {
            false
        };

        let is_interacting_with_popup = is_interacting_with_saved_eq_popup || is_interacting_with_clipping_popup;

        let is_canvas_blocked = is_modal_blocking || is_interacting_with_tutorial || is_interacting_with_popup;

        // Dynamic Zoom (Ctrl + Scroll / Pinch) vs Pan (Normal Scroll)
        let zoom_scroll = if has_raw_wheel {
            raw_wheel_delta.y + raw_wheel_delta.x
        } else if is_ctrl && (smooth_scroll.y.abs() > 0.0 || smooth_scroll.x.abs() > 0.0) {
            smooth_scroll.y + smooth_scroll.x
        } else {
            0.0
        };

        let has_zoom = (is_ctrl && zoom_scroll.abs() > 0.0) || (pinch_zoom - 1.0).abs() > 0.001;

        if is_ctrl {
            // When Ctrl is held, ONLY perform zoom. Never let scroll leak into canvas panning!
            if response.hovered() && !is_canvas_blocked && has_zoom {
                let factor = if (pinch_zoom - 1.0).abs() > 0.001 {
                    pinch_zoom.clamp(0.90, 1.10)
                } else if zoom_scroll > 0.0 {
                    // Zoom In: Gentle ~8% step per wheel notch (120 units)
                    1.0 + (zoom_scroll.abs() / 120.0).clamp(0.25, 2.0) * 0.08
                } else if zoom_scroll < 0.0 {
                    // Zoom Out: Gentle ~8% step per wheel notch
                    1.0 / (1.0 + (zoom_scroll.abs() / 120.0).clamp(0.25, 2.0) * 0.08)
                } else {
                    1.0
                };

                let old_zoom = self.patchbay_zoom;
                let new_zoom = (old_zoom * factor).clamp(0.45, 1.8);
                if (new_zoom - old_zoom).abs() > 0.0001 {
                    if let Some(mpos) = raw_pointer_pos {
                        let m_rel = mpos - canvas_rect.min;
                        self.patchbay_pan = self.patchbay_pan + m_rel * (1.0 / new_zoom - 1.0 / old_zoom);
                        self.patchbay_pan.x = self.patchbay_pan.x.clamp(-3500.0, 3500.0);
                        self.patchbay_pan.y = self.patchbay_pan.y.clamp(-2500.0, 2500.0);
                    }
                    self.patchbay_zoom = new_zoom;
                    ui.ctx().request_repaint();
                }
            }
        } else {
            // Normal scroll (without Ctrl) pans the canvas
            let pan_scroll = if has_raw_wheel {
                raw_wheel_delta
            } else {
                smooth_scroll
            };
            if response.hovered() && !is_canvas_blocked && (pan_scroll.x.abs() > 0.0 || pan_scroll.y.abs() > 0.0) {
                self.patchbay_pan += pan_scroll / self.patchbay_zoom;
                self.patchbay_pan.x = self.patchbay_pan.x.clamp(-3500.0, 3500.0);
                self.patchbay_pan.y = self.patchbay_pan.y.clamp(-2500.0, 2500.0);
            }
        }

        // TSTransform is applied AFTER all shapes are drawn via transform_layer_shapes() at the
        // end of this function.  This means every shape is drawn in pre-transform screen-space coords:
        //   drawn_pos = canvas_rect.min + patchbay_pan + node.pos
        // After transform_layer_shapes(ts), egui re-scales those positions to:
        //   final_screen = drawn_pos * zoom + canvas_rect.min * (1 - zoom)
        //                = canvas_rect.min + (patchbay_pan + node.pos) * zoom  ✓
        //
        // Because the painter clips shapes in pre-transform coords, virt_clip_rect must be large
        // enough to contain every possible drawn_pos regardless of pan or zoom value.
        // We use the full (unpanned) canvas_rect as the base and expand generously by the maximum
        // possible pan extent plus a fixed node-size budget — simple and correct at any zoom level.
        let canvas_layer_id = egui::LayerId::new(egui::Order::Middle, ui.scope_id().with("patchbay_canvas_world"));
        let ts = egui::emath::TSTransform::new(
            canvas_rect.min.to_vec2() * (1.0 - self.patchbay_zoom),
            self.patchbay_zoom,
        );
        // Painter clip rect is in pre-transform screen coords.
        // When transform_layer_shapes(ts) runs at the end of the frame, egui transforms all shape
        // clip rects by: shape.clip_rect = ts * shape.clip_rect.
        // Therefore, setting the pre-transform clip rect to ts.inverse() * canvas_rect via set_clip_rect()
        // (which bypasses with_clip_rect's self.clip_rect intersection) guarantees that post-transform
        // on-screen clip rect is EXACTLY canvas_rect at any zoom level, preventing horizontal/vertical clipping!
        let mut painter = ui.ctx().layer_painter(canvas_layer_id);
        painter.set_clip_rect(ts.inverse() * canvas_rect);

        // Virtual pointer coordinates mapped through inverse TSTransform
        let mouse_pos = if is_canvas_blocked { None } else { raw_pointer_pos.map(|p| ts.inverse() * p) };

        // Dynamic scrolling dot grid on unscaled base canvas
        let grid_size = (28.0 * self.patchbay_zoom).clamp(14.0, 56.0);
        let eff_pan_x = self.patchbay_pan.x * self.patchbay_zoom;
        let eff_pan_y = self.patchbay_pan.y * self.patchbay_zoom;
        let offset_x = (eff_pan_x % grid_size + grid_size) % grid_size;
        let offset_y = (eff_pan_y % grid_size + grid_size) % grid_size;
        let mut gx = offset_x - grid_size;
        while gx <= canvas_size.x + grid_size {
            let mut gy = offset_y - grid_size;
            while gy <= canvas_size.y + grid_size {
                base_painter.circle_filled(
                    Pos2::new(canvas_rect.min.x + gx, canvas_rect.min.y + gy),
                    (1.0 * self.patchbay_zoom).clamp(0.7, 1.4),
                    Color32::from_rgb(203, 213, 225),
                );
                gy += grid_size;
            }
            gx += grid_size;
        }

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
        let mut auto_fix_node: Option<usize> = None;
        let mut resampler_setting_change: Option<crate::models::ResamplerSettings> = None;

        let pointer_pos = if is_canvas_blocked { None } else { ui.input(|i| i.pointer.latest_pos()).map(|p| ts.inverse() * p).or(mouse_pos) };
        let is_left_pressed = !is_canvas_blocked && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));

        // Check if cursor is hovering over any node rect
        let is_hovering_any_node = self.nodes.iter().any(|node| {
            let n_size = get_node_size(&node.node_type);
            let r = Rect::from_min_size(
                Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                    canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                ),
                n_size,
            );
            pointer_pos.map_or(false, |p| r.contains(p))
        });

        // 0a. Check hover & active drag on interactive node sliders, draggable headers, and whole cards
        let mut is_hovering_slider = false;
        let mut is_hovering_header = false;
        if !is_canvas_blocked {
            for node in &self.nodes {
                let n_size = get_node_size(&node.node_type);
                let world_rect = Rect::from_min_size(
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                        canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                    ),
                    n_size,
                );
                let has_slider = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, n_size.x, 0.0).is_some();
                if let Some((s_hit, _)) = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, n_size.x, 0.0) {
                    if pointer_pos.map_or(false, |p| s_hit.contains(p)) {
                        is_hovering_slider = true;
                        break;
                    }
                }
                // SteppedAttenuator DIM button hover
                if node.node_type == NodeType::SteppedAttenuator {
                    let dim_hit = Rect::from_min_size(Pos2::new(world_rect.max.x - 48.0, world_rect.min.y + 36.0), Vec2::new(34.0, 20.0));
                    if pointer_pos.map_or(false, |p| dim_hit.contains(p)) {
                        is_hovering_slider = true;
                        break;
                    }
                }
                // App Resampler Quality & Profile Badges hover
                if node.node_type == NodeType::Driver {
                    let b1_hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 18.0, world_rect.min.y + 58.0), Vec2::new(54.0, 19.0));
                    let b2_hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 76.0, world_rect.min.y + 58.0), Vec2::new(58.0, 19.0));
                    if pointer_pos.map_or(false, |p| b1_hit.contains(p) || b2_hit.contains(p)) {
                        is_hovering_slider = true;
                        break;
                    }
                }
                if !has_slider {
                    // Slider-less module: whole card is a drag handle!
                    if pointer_pos.map_or(false, |p| world_rect.contains(p)) {
                        is_hovering_header = true;
                    }
                } else {
                    // Slider module: top 28px header is drag handle
                    let header_rect = Rect::from_min_size(world_rect.min, Vec2::new(n_size.x, 28.0));
                    if pointer_pos.map_or(false, |p| header_rect.contains(p)) {
                        is_hovering_header = true;
                    }
                }
            }
        }
        if self.dragging_node.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if self.dragging_slider_node.is_some() || (is_hovering_slider && !self.is_panning_canvas) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        } else if is_hovering_header && !self.is_panning_canvas && self.connecting_from_node.is_none() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        // 0b. Mouse press down directly on slider track or thumb
        let mut clicked_slider = false;
        if is_left_pressed && !is_canvas_blocked {
            if let Some(pos) = pointer_pos {
                for node in &self.nodes {
                    let n_size = get_node_size(&node.node_type);
                    let world_rect = Rect::from_min_size(
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                            canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                        ),
                        n_size,
                    );
                    // Check DIM button toggle on Stepped Attenuator
                    if node.node_type == NodeType::SteppedAttenuator {
                        let dim_hit = Rect::from_min_size(Pos2::new(world_rect.max.x - 48.0, world_rect.min.y + 36.0), Vec2::new(34.0, 20.0));
                        if dim_hit.contains(pos) {
                            let new_val = if (node.param_val - (-20.0)).abs() < 0.5 { 0.0 } else { -20.0 };
                            param_change = Some((node.id, new_val));
                            self.selected_node = Some(node.id);
                            clicked_slider = true;
                            break;
                        }
                    }
                    // Check Sinc Quality & Profile Badges on App Resampler
                    if node.node_type == NodeType::Driver {
                        let b1_hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 18.0, world_rect.min.y + 58.0), Vec2::new(54.0, 19.0));
                        let b2_hit = Rect::from_min_size(Pos2::new(world_rect.min.x + 76.0, world_rect.min.y + 58.0), Vec2::new(58.0, 19.0));
                        if b1_hit.contains(pos) {
                            let next_q = match self.resampler_settings.quality {
                                crate::models::SincQuality::Fast => crate::models::SincQuality::Studio,
                                crate::models::SincQuality::Studio => crate::models::SincQuality::Extreme,
                                crate::models::SincQuality::Extreme => crate::models::SincQuality::Fast,
                            };
                            let mut new_settings = self.resampler_settings;
                            new_settings.quality = next_q;
                            resampler_setting_change = Some(new_settings);
                            clicked_slider = true;
                            break;
                        } else if b2_hit.contains(pos) {
                            let next_p = match self.resampler_settings.profile {
                                crate::models::RollOffProfile::Sharp => crate::models::RollOffProfile::Smooth,
                                crate::models::RollOffProfile::Smooth => crate::models::RollOffProfile::Nyquist,
                                crate::models::RollOffProfile::Nyquist => crate::models::RollOffProfile::Sharp,
                            };
                            let mut new_settings = self.resampler_settings;
                            new_settings.profile = next_p;
                            resampler_setting_change = Some(new_settings);
                            clicked_slider = true;
                            break;
                        }
                    }
                    if let Some((s_hit, val)) = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, n_size.x, pos.x) {
                        if s_hit.contains(pos) {
                            self.dragging_slider_node = Some(node.id);
                            self.dragging_node = None;
                            self.is_panning_canvas = false;
                            self.connecting_from_node = None;
                            self.connecting_from_port = 0;
                            self.connecting_is_input_drag = false;
                            self.selected_node = Some(node.id);
                            param_change = Some((node.id, val));
                            clicked_slider = true;
                            break;
                        }
                    }
                }
            }
        }

        if let Some(new_settings) = resampler_setting_change {
            self.set_resampler_settings(new_settings);
            self.selected_node = Some(4);
        }

        // 0c. Continuous slider dragging updates
        if is_left_down {
            if let Some(drag_node_id) = self.dragging_slider_node {
                if let Some(pos) = pointer_pos {
                    if let Some(node) = self.nodes.iter().find(|n| n.id == drag_node_id) {
                        let n_size = get_node_size(&node.node_type);
                        let world_rect = Rect::from_min_size(
                            Pos2::new(
                                canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                            ),
                            n_size,
                        );
                        if let Some((_, val)) = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, n_size.x, pos.x) {
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
        if right_or_middle_pressed && response.hovered() && !is_hovering_any_node && !is_canvas_blocked {
            self.is_panning_canvas = true;
            self.connecting_from_node = None;
            self.connecting_from_port = 0;
            self.dragging_node = None;
        }

        if (is_right_down || is_middle_down) && self.is_panning_canvas {
            let delta = ui.input(|i| i.pointer.delta()) / self.patchbay_zoom;
            self.patchbay_pan += delta;
        }

        if !is_left_down && !is_right_down && !is_middle_down {
            self.is_panning_canvas = false;
        }

        // 1. Detect clicking on Pins to create cables OR clicking nodes to drag OR empty space to pan (Left Click)
        if self.dragging_slider_node.is_some() || clicked_slider {
            // Slider active, ignore canvas drag
        } else if response.drag_started() && is_left_down && !is_right_down && !is_middle_down && !is_canvas_blocked {
            if let Some(pos) = mouse_pos {
                // Check if user clicked a Satellite Box body to drag the satellite independently
                let mut hit_satellite_body = false;
                for node in &self.nodes {
                    if node.node_type == NodeType::Splitter || node.node_type == NodeType::Mixer {
                        for port in 0..2 {
                            let sat_world = self.get_satellite_pos(node.id, port, node.pos, &node.node_type);
                            let sat_screen = Pos2::new(
                                canvas_rect.min.x + self.patchbay_pan.x + sat_world.x,
                                canvas_rect.min.y + self.patchbay_pan.y + sat_world.y,
                            );
                            let sat_rect = Rect::from_min_size(sat_screen, Vec2::new(52.0, 34.0));
                            if sat_rect.contains(pos) {
                                let pin_pos = if node.node_type == NodeType::Mixer {
                                    Pos2::new(sat_screen.x, sat_screen.y + 17.0)
                                } else {
                                    Pos2::new(sat_screen.x + 52.0, sat_screen.y + 17.0)
                                };
                                if pos.distance(pin_pos) > 16.0 {
                                    self.dragging_satellite = Some((node.id, port));
                                    self.dragging_node = None;
                                    self.selected_node = Some(node.id);
                                    hit_satellite_body = true;
                                    break;
                                }
                            }
                        }
                    }
                    if hit_satellite_body { break; }
                }
                // Check if user clicked an Output Pin (Right dot) OR an Input Pin (Left dot)
                for node in &self.nodes {
                    let n_size = get_node_size(&node.node_type);
                    let world_rect = Rect::from_min_size(
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                            canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                        ),
                        n_size,
                    );

                    // 1. Output Pin (Right dot) - Except Endpoint
                    if node.node_type != NodeType::Endpoint {
                        let hit_out_pin = if node.node_type == NodeType::Splitter {
                            let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                            let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                            let pin_a = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x + 52.0, canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y + 17.0);
                            let pin_b = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x + 52.0, canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y + 17.0);
                            if pos.distance(pin_a) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 0;
                                self.connecting_is_input_drag = false;
                                true
                            } else if pos.distance(pin_b) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 1;
                                self.connecting_is_input_drag = false;
                                true
                            } else {
                                false
                            }
                        } else if node.node_type == NodeType::Crossover {
                            let pin_a = Pos2::new(world_rect.max.x, world_rect.center().y - 14.0);
                            let pin_b = Pos2::new(world_rect.max.x, world_rect.center().y + 14.0);
                            if pos.distance(pin_a) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 0;
                                self.connecting_is_input_drag = false;
                                true
                            } else if pos.distance(pin_b) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 1;
                                self.connecting_is_input_drag = false;
                                true
                            } else {
                                false
                            }
                        } else {
                            let out_pin_pos = Pos2::new(world_rect.max.x, world_rect.center().y);
                            if pos.distance(out_pin_pos) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 0;
                                self.connecting_is_input_drag = false;
                                true
                            } else {
                                false
                            }
                        };
                        if hit_out_pin {
                            break;
                        }
                    }

                    // 2. Input Pin (Left dot) - Except Source (Enables Reverse Dragging!)
                    if node.node_type != NodeType::Source {
                        let hit_in_pin = if node.node_type == NodeType::Mixer {
                            let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                            let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                            let pin_a = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x, canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y + 17.0);
                            let pin_b = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x, canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y + 17.0);
                            if pos.distance(pin_a) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 0;
                                self.connecting_is_input_drag = true;
                                true
                            } else if pos.distance(pin_b) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 1;
                                self.connecting_is_input_drag = true;
                                true
                            } else {
                                false
                            }
                        } else {
                            let in_pin_pos = Pos2::new(world_rect.min.x, world_rect.center().y);
                            if pos.distance(in_pin_pos) <= 20.0 {
                                self.connecting_from_node = Some(node.id);
                                self.connecting_from_port = 0;
                                self.connecting_is_input_drag = true;
                                true
                            } else {
                                false
                            }
                        };
                        if hit_in_pin {
                            break;
                        }
                    }
                }

                // If not starting cable, check if clicked on a node card
                if self.connecting_from_node.is_none() {
                    let mut clicked_on_any_node = false;
                    for (i, node) in self.nodes.iter().enumerate().rev() {
                        let n_size = get_node_size(&node.node_type);
                        let world_rect = Rect::from_min_size(
                            Pos2::new(
                                canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                            ),
                            n_size,
                        );
                        if world_rect.contains(pos) {
                            clicked_on_any_node = true;
                            self.selected_node = Some(node.id);
                            self.advance_studio_state.selected_cable_idx = None;

                            let has_slider = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, n_size.x, pos.x).is_some();
                            let is_slider_hit = get_interactive_slider_bounds_and_val(&node.node_type, world_rect, n_size.x, pos.x)
                                .map_or(false, |(s_hit, _)| s_hit.contains(pos));

                            // SteppedAttenuator DIM button hit
                            let is_dim_hit = if node.node_type == NodeType::SteppedAttenuator {
                                let dim_hit = Rect::from_min_size(Pos2::new(world_rect.max.x - 48.0, world_rect.min.y + 36.0), Vec2::new(34.0, 20.0));
                                dim_hit.contains(pos)
                            } else {
                                false
                            };

                            if !has_slider {
                                // Slider-less module: Entire body is a drag handle!
                                self.dragging_node = Some(i);
                            } else {
                                // Module with sliders/knobs: Header (top 28px) is drag handle
                                let header_rect = Rect::from_min_size(world_rect.min, Vec2::new(n_size.x, 28.0));
                                if header_rect.contains(pos) && !is_slider_hit && !is_dim_hit {
                                    self.dragging_node = Some(i);
                                }
                            }
                            break;
                        }
                    }

                    // Panning canvas ONLY if clicked on empty background outside all nodes and sliders
                    if !clicked_on_any_node && self.dragging_node.is_none() && self.dragging_slider_node.is_none() {
                        self.is_panning_canvas = true;
                        self.selected_node = None;
                        self.advance_studio_state.selected_cable_idx = None;
                    }
                }
            }
        }

        // 2. Handle Node Dragging, Satellite Dragging, OR Canvas Background Panning (Left Click)
        if is_mouse_down && is_left_down && self.connecting_from_node.is_none() && self.dragging_slider_node.is_none() {
            let raw_delta = ui.input(|i| i.pointer.delta());
            let delta = raw_delta / self.patchbay_zoom;
            if let Some((node_id, port)) = self.dragging_satellite {
                let node_pos = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.pos).unwrap_or(Pos2::ZERO);
                let node_type = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.node_type.clone()).unwrap_or(NodeType::Mixer);
                let cur = self.get_satellite_pos(node_id, port, node_pos, &node_type);
                self.satellite_positions.insert((node_id, port), cur + delta);
            } else if let Some(drag_idx) = self.dragging_node {
                let node = &mut self.nodes[drag_idx];
                node.pos += delta;
            } else if self.is_panning_canvas {
                self.patchbay_pan += delta;
            }
        }

        // 3. Handle Cable Connection Release (Supports Bidirectional Dragging, DAG Branching & Prevents Cycles)
        if response.drag_stopped() {
            if let (Some(from_id), Some(pos)) = (self.connecting_from_node, mouse_pos) {
                if !self.connecting_is_input_drag {
                    // Standard Drag: From Output Pin to an Input Pin
                    for node in &self.nodes {
                        if node.id != from_id && node.node_type != NodeType::Source {
                            let n_size = get_node_size(&node.node_type);
                            let world_rect = Rect::from_min_size(
                                Pos2::new(
                                    canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                    canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                                ),
                                n_size,
                            );
                            let is_mixer = node.node_type == NodeType::Mixer;
                            let (hit_in_pin, target_port) = if is_mixer {
                                let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                                let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                                let pin_a = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x, canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y + 17.0);
                                let pin_b = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x, canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y + 17.0);
                                if pos.distance(pin_a) <= 20.0 {
                                    (true, 0)
                                } else if pos.distance(pin_b) <= 20.0 {
                                    (true, 1)
                                } else {
                                    (false, 0)
                                }
                            } else {
                                let in_pin_pos = Pos2::new(world_rect.min.x, world_rect.center().y);
                                (pos.distance(in_pin_pos) <= 20.0, 0)
                            };

                            if hit_in_pin {
                                let from_port = self.connecting_from_port;
                                let already_connected = self.connections.iter().any(|c| {
                                    c.from_node == from_id && c.from_port == from_port && c.to_node == node.id && c.to_port == target_port
                                });
                                let would_create_cycle = self.can_reach_node(node.id, from_id);

                                if !already_connected && !would_create_cycle {
                                    if !is_mixer {
                                        self.connections.retain(|c| c.to_node != node.id);
                                    } else {
                                        self.connections.retain(|c| !(c.to_node == node.id && c.to_port == target_port));
                                    }

                                    let is_from_splitter = self.nodes.iter().find(|n| n.id == from_id).map_or(false, |n| n.node_type == NodeType::Splitter || n.node_type == NodeType::Crossover);
                                    if !is_from_splitter {
                                        self.connections.retain(|c| c.from_node != from_id);
                                    } else {
                                        self.connections.retain(|c| !(c.from_node == from_id && c.from_port == from_port));
                                    }

                                    self.connections.push(CableConnection::with_ports(
                                        from_id,
                                        from_port,
                                        node.id,
                                        target_port,
                                    ));
                                    topology_changed = true;
                                }
                                break;
                            }
                        }
                    }
                } else {
                    // Reverse Drag: From Input Pin to an Output Pin
                    for node in &self.nodes {
                        if node.id != from_id && node.node_type != NodeType::Endpoint {
                            let n_size = get_node_size(&node.node_type);
                            let world_rect = Rect::from_min_size(
                                Pos2::new(
                                    canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                                    canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                                ),
                                n_size,
                            );
                            let is_splitter = node.node_type == NodeType::Splitter || node.node_type == NodeType::Crossover;
                            let (hit_out_pin, target_out_port) = if node.node_type == NodeType::Splitter {
                                let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                                let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                                let pin_a = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x + 52.0, canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y + 17.0);
                                let pin_b = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x + 52.0, canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y + 17.0);
                                if pos.distance(pin_a) <= 20.0 {
                                    (true, 0)
                                } else if pos.distance(pin_b) <= 20.0 {
                                    (true, 1)
                                } else {
                                    (false, 0)
                                }
                            } else if node.node_type == NodeType::Crossover {
                                let pin_a = Pos2::new(world_rect.max.x, world_rect.center().y - 14.0);
                                let pin_b = Pos2::new(world_rect.max.x, world_rect.center().y + 14.0);
                                if pos.distance(pin_a) <= 20.0 {
                                    (true, 0)
                                } else if pos.distance(pin_b) <= 20.0 {
                                    (true, 1)
                                } else if pos.distance(Pos2::new(world_rect.max.x, world_rect.center().y)) <= 20.0 {
                                    if pos.y < world_rect.center().y { (true, 0) } else { (true, 1) }
                                } else {
                                    (false, 0)
                                }
                            } else {
                                let out_pin_pos = Pos2::new(world_rect.max.x, world_rect.center().y);
                                (pos.distance(out_pin_pos) <= 20.0, 0)
                            };

                            if hit_out_pin {
                                let from_in_port = self.connecting_from_port;
                                let already_connected = self.connections.iter().any(|c| {
                                    c.from_node == node.id && c.from_port == target_out_port && c.to_node == from_id && c.to_port == from_in_port
                                });
                                let would_create_cycle = self.can_reach_node(from_id, node.id);

                                if !already_connected && !would_create_cycle {
                                    // Replace incoming cable on from_id's input port
                                    let is_from_mixer = self.nodes.iter().find(|n| n.id == from_id).map_or(false, |n| n.node_type == NodeType::Mixer);
                                    if !is_from_mixer {
                                        self.connections.retain(|c| c.to_node != from_id);
                                    } else {
                                        self.connections.retain(|c| !(c.to_node == from_id && c.to_port == from_in_port));
                                    }

                                    // Replace outgoing cable on node.id's output port
                                    if !is_splitter {
                                        self.connections.retain(|c| c.from_node != node.id);
                                    } else {
                                        self.connections.retain(|c| !(c.from_node == node.id && c.from_port == target_out_port));
                                    }

                                    self.connections.push(CableConnection::with_ports(
                                        node.id,
                                        target_out_port,
                                        from_id,
                                        from_in_port,
                                    ));
                                    topology_changed = true;
                                }
                                break;
                            }
                        }
                    }
                }
            }
            self.connecting_from_node = None;
            self.connecting_from_port = 0;
            self.connecting_is_input_drag = false;
            self.dragging_node = None;
            self.dragging_satellite = None;
            self.is_panning_canvas = false;
        }

        // Cursor State Feedback:
        // Default arrow normally, custom 5-finger open hand cursor ONLY when holding right-click to pan!
        if !is_canvas_blocked {
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
        }

        // Evaluate Signal Chain reachability
        let chain_eval = self.evaluate_signal_chain();

        // Real-time static headroom & clipping risk analysis (Abstract Interpretation)
        let static_report = self.get_static_analysis();

        let is_fallback = self.signal_path_report.as_ref().map_or(false, |r| {
            r.requested_mode == ChainMode::BitPerfect && (r.actual_mode == ChainMode::OsResampled || r.is_resampling)
        });
        let is_true_bp_highway = self.is_true_bit_perfect_highway();

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
                let from_size = get_node_size(&from_n.node_type);
                let to_size = get_node_size(&to_n.node_type);
                let p_start = if from_n.node_type == NodeType::Splitter {
                    let sat_world = self.get_satellite_pos(from_n.id, conn.from_port, from_n.pos, &from_n.node_type);
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + sat_world.x + 52.0,
                        canvas_rect.min.y + self.patchbay_pan.y + sat_world.y + 17.0,
                    )
                } else if from_n.node_type == NodeType::Crossover {
                    let pin_offset_y = if conn.from_port == 1 { 14.0 } else { -14.0 };
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + from_size.x,
                        canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + from_size.y * 0.5 + pin_offset_y,
                    )
                } else {
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + from_size.x,
                        canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + from_size.y * 0.5,
                    )
                };
                let p_end = if to_n.node_type == NodeType::Mixer {
                    let sat_world = self.get_satellite_pos(to_n.id, conn.to_port, to_n.pos, &to_n.node_type);
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + sat_world.x,
                        canvas_rect.min.y + self.patchbay_pan.y + sat_world.y + 17.0,
                    )
                } else {
                    Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + to_n.pos.x,
                        canvas_rect.min.y + self.patchbay_pan.y + to_n.pos.y + to_size.y * 0.5,
                    )
                };

                let dx = (p_end.x - p_start.x).abs().max(40.0) * 0.5;
                let (cp1, cp2) = (
                    Pos2::new(p_start.x + dx, p_start.y),
                    Pos2::new(p_end.x - dx, p_end.y),
                );

                let steps = 24;
                let mut points = Vec::with_capacity(steps + 1);
                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                    let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                    points.push(Pos2::new(x, y));
                }

                let is_flowing_cable = chain_eval.flowing_cables.contains(conn);

                // Check if user clicked or right-clicked on cable to disconnect
                let mid = points[12];
                let is_hover_cut = mouse_pos.map_or(false, |p| p.distance(mid) <= 10.0);
                let is_hover_cable = mouse_pos.map_or(false, |p| points.iter().any(|pt| pt.distance(p) <= 6.0) || p.distance(mid) <= 14.0);

                if is_hover_cut && (response.clicked() || response.secondary_clicked()) {
                    connection_to_remove = Some(conn_idx);
                } else if is_hover_cable && response.clicked() {
                    self.advance_studio_state.selected_cable_idx = Some(conn_idx);
                    self.selected_node = None;
                }

                let is_selected_cable = self.advance_studio_state.selected_cable_idx == Some(conn_idx);

                let is_bypassed_resampler_cable = is_true_bp_highway
                    && (from_n.node_type == NodeType::Driver || to_n.node_type == NodeType::Driver);
                let is_effect_cable = ((from_n.node_type.is_dsp_or_effect() || to_n.node_type.is_dsp_or_effect())
                    && (self.chain_mode == ChainMode::BitPerfect || self.chain_mode == ChainMode::OsResampled))
                    || is_bypassed_resampler_cable;

                let is_hot_cable = static_report.is_edge_hot(conn.from_node as u32, conn.from_port, conn.to_node as u32, conn.to_port)
                    && self.chain_mode == ChainMode::DspActive;

                let cable_color = if is_selected_cable {
                    Color32::from_rgb(52, 211, 153) // Emerald green when selected for probe
                } else if is_hover_cable {
                    Color32::from_rgb(239, 68, 68) // Red on hover to cut
                } else if is_effect_cable {
                    Color32::from_rgb(203, 213, 225) // Muted slate gray when connected to bypassed effect
                } else if is_hot_cable && is_flowing_cable {
                    Color32::from_rgb(239, 68, 68) // Crimson red for hot overdriven signal (> 0 dBFS)
                } else if is_flowing_cable {
                    theme_color
                } else {
                    Color32::from_rgb(239, 68, 68) // Red when severed/dead end
                };

                // Drop shadow (single batched polyline)
                let shadow_pts: Vec<Pos2> = points.iter().map(|p| *p + Vec2::new(0.0, 1.5)).collect();
                painter.add(egui::Shape::line(shadow_pts, Stroke::new(3.0, Color32::from_black_alpha(18))));

                // Selected cable outer neon halo or hot cable warning aura
                if is_selected_cable {
                    painter.add(egui::Shape::line(points.clone(), Stroke::new(5.5, Color32::from_rgba_premultiplied(52, 211, 153, 90))));
                } else if is_hot_cable && is_flowing_cable && !is_effect_cable {
                    let pulse = (ui.input(|i| i.time) * 4.0).sin().abs() as f32;
                    let alpha = (60.0 + pulse * 50.0) as u8;
                    painter.add(egui::Shape::line(points.clone(), Stroke::new(5.5, Color32::from_rgba_premultiplied(239, 68, 68, alpha))));
                }

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
                    let particle_glow = if is_hot_cable {
                        Color32::from_rgba_premultiplied(254, 202, 202, 230) // Hot white-pinkish red
                    } else if is_fallback {
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
                        let inner_dot = if is_hot_cable { Color32::from_rgb(255, 180, 180) } else { Color32::WHITE };
                        painter.circle_filled(pt, 2.0, inner_dot);
                    }
                }


                // Cut hint bubble on hover or hot signal badge
                if !is_canvas_blocked && is_hover_cable {
                    if is_hot_cable && is_flowing_cable {
                        let cable_tip = ui.interact(Rect::from_center_size(ts.mul_pos(mid), Vec2::splat(24.0)), ui.scope_id().with(("cable_tip", conn_idx)), egui::Sense::hover());
                        cable_tip.show_tooltip_text("⚡ KABEL PANAS (> 0 dBFS)\nSinyal melampaui batas full-scale audio.\nKlik untuk memilih kabel, klik tanda ✕ untuk memutus.");
                    }
                    painter.circle_filled(mid, 9.5, Color32::from_rgb(239, 68, 68));
                    painter.text(mid, egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(10.5), Color32::WHITE);
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                } else if is_hot_cable && is_flowing_cable && !is_effect_cable {
                    // Hot signal warning badge on cable midpoint
                    painter.circle_filled(mid, 8.5, Color32::from_rgb(239, 68, 68));
                    painter.circle_stroke(mid, 8.5, Stroke::new(1.2, Color32::WHITE));
                    painter.text(mid, egui::Align2::CENTER_CENTER, "⚡", egui::FontId::proportional(9.0), Color32::WHITE);
                } else if !is_flowing_cable {
                    // Warning icon at midpoint of dead cable
                    painter.circle_filled(mid, 8.5, Color32::from_rgb(239, 68, 68));
                    painter.text(mid, egui::Align2::CENTER_CENTER, "!", egui::FontId::proportional(10.5), Color32::WHITE);
                }
            }
        }

        // 4c. Bit-Perfect Direct Jump Highway (RAM Buffer Node 2 ──▶ Output DAC Node 5)
        // ONLY when truly Bit-Perfect 1:1 (`is_true_bp_highway`), signal jumps directly from Node 2 (RAM BUFFER)
        // to Node 5 (OUTPUT DAC), bypassing both modular DSP effects and Node 4 (APP RESAMPLER).
        // If fallback occurred (`!is_true_bp_highway`), signal bypasses modular DSP effects into Node 4 (APP RESAMPLER)
        // because Rubato Sinc is actively adapting the sample rate!
        let is_bypass_mode = self.chain_mode == ChainMode::BitPerfect || self.chain_mode == ChainMode::OsResampled;
        if is_bypass_mode && chain_eval.is_complete {
            let node2 = self.nodes.iter().find(|n| n.id == 2 || n.node_type == NodeType::Buffer);
            let target_node = if is_true_bp_highway {
                self.nodes.iter().find(|n| n.id == 5 || n.node_type == NodeType::Endpoint)
            } else {
                self.nodes.iter().find(|n| n.id == 4 || n.node_type == NodeType::Driver)
            };
            if let (Some(n2), Some(n_target)) = (node2, target_node) {
                let has_direct_conn = self.connections.iter().any(|c| c.from_node == n2.id && c.to_node == n_target.id);
                if !has_direct_conn {
                    let n2_size = get_node_size(&n2.node_type);
                    let target_size = get_node_size(&n_target.node_type);
                    let p_start = Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + n2.pos.x + n2_size.x,
                        canvas_rect.min.y + self.patchbay_pan.y + n2.pos.y + n2_size.y * 0.5,
                    );
                    let p_end = Pos2::new(
                        canvas_rect.min.x + self.patchbay_pan.x + n_target.pos.x,
                        canvas_rect.min.y + self.patchbay_pan.y + n_target.pos.y + target_size.y * 0.5,
                    );

                    let dx = (p_end.x - p_start.x).abs().max(40.0) * 0.5;
                    let (cp1, cp2) = (
                        Pos2::new(p_start.x + dx, p_start.y),
                        Pos2::new(p_end.x - dx, p_end.y),
                    );

                    let steps = 24;
                    let mut points = Vec::with_capacity(steps + 1);
                    for step in 0..=steps {
                        let t = step as f32 / steps as f32;
                        let u = 1.0 - t;
                        let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                        let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                        points.push(Pos2::new(x, y));
                    }

                    let (bypass_col, halo_col, badge_text, badge_bg, badge_stroke, badge_text_col) = if is_true_bp_highway {
                        (
                            Color32::from_rgb(16, 185, 129),
                            Color32::from_rgba_premultiplied(52, 211, 153, 90),
                            "🛡️ BYPASS BIT-PERFECT 1:1",
                            Color32::from_rgb(236, 253, 245),
                            Color32::from_rgb(16, 185, 129),
                            Color32::from_rgb(4, 120, 87),
                        )
                    } else {
                        (
                            Color32::from_rgb(245, 158, 11),
                            Color32::from_rgba_premultiplied(251, 191, 36, 90),
                            "⚡ FALLBACK TO RESAMPLER",
                            Color32::from_rgb(254, 243, 199),
                            Color32::from_rgb(245, 158, 11),
                            Color32::from_rgb(180, 83, 9),
                        )
                    };

                    let mid = points[12];

                    // Drop shadow
                    let shadow_pts: Vec<Pos2> = points.iter().map(|p| *p + Vec2::new(0.0, 1.5)).collect();
                    painter.add(egui::Shape::line(shadow_pts, Stroke::new(3.0, Color32::from_black_alpha(18))));

                    // Glowing neon halo
                    painter.add(egui::Shape::line(points.clone(), Stroke::new(5.0, halo_col)));

                    // Main vibrant bypass cable
                    painter.add(egui::Shape::line(points.clone(), Stroke::new(2.8, bypass_col)));

                    // Animated live audio data particles running along direct bypass
                    if self.is_playing {
                        let particle_glow = if self.chain_mode == ChainMode::BitPerfect {
                            Color32::from_rgba_premultiplied(167, 243, 208, 220)
                        } else {
                            Color32::from_rgba_premultiplied(253, 230, 138, 220)
                        };
                        for k in 0..3 {
                            let t_part = ((self.visualizer_anim_phase * 0.75 + k as f32 * 0.33) % 1.0 + 1.0) % 1.0;
                            let p_idx = (t_part * 24.0).min(23.0) as usize;
                            let pt = points[p_idx];
                            painter.circle_filled(pt, 4.0, particle_glow);
                            painter.circle_filled(pt, 2.0, Color32::WHITE);
                        }
                    }

                    // Sleek Pill Badge at midpoint
                    let badge_rect = Rect::from_center_size(mid, Vec2::new(168.0, 22.0));
                    let is_hover_bypass = mouse_pos.map_or(false, |p| badge_rect.contains(p) || points.iter().any(|pt| pt.distance(p) <= 8.0));

                    let curr_badge_bg = if is_hover_bypass {
                        Color32::from_rgb(209, 250, 229)
                    } else {
                        badge_bg
                    };
                    painter.rect_filled(badge_rect, 11.0, curr_badge_bg);
                    painter.rect_stroke(badge_rect, 11.0, Stroke::new(1.0, badge_stroke), egui::StrokeKind::Outside);
                    painter.text(
                        mid,
                        egui::Align2::CENTER_CENTER,
                        badge_text,
                        egui::FontId::proportional(10.0),
                        badge_text_col,
                    );

                    if is_hover_bypass {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        if response.clicked() {
                            switch_mode = Some(ChainMode::DspActive);
                        }
                    }
                }
            }
        }

        // 4b. Render Broken Gaps between Standard Default Nodes with Click-to-Connect
        // Only rendered if the signal chain is incomplete AND no alternative path exists!
        // If from_id can already reach to_id (e.g. through an EQ preset or modular filter),
        // the connection is fully satisfied and NO red dashed line is drawn!
        if !chain_eval.is_complete {
            let standard_pairs: Vec<(usize, usize)> = if is_true_bp_highway {
                vec![(1, 2), (2, 5)]
            } else {
                vec![(1, 2), (2, 4), (4, 5)]
            };
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
                        let from_size = get_node_size(&from_n.node_type);
                        let to_size = get_node_size(&to_n.node_type);
                        let p_start = Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + from_size.x,
                            canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + from_size.y * 0.5,
                        );
                        let p_end = Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + to_n.pos.x,
                            canvas_rect.min.y + self.patchbay_pan.y + to_n.pos.y + to_size.y * 0.5,
                        );

                        let dx = (p_end.x - p_start.x).abs() * 0.5;
                        let (cp1, cp2) = (Pos2::new(p_start.x + dx, p_start.y), Pos2::new(p_end.x - dx, p_end.y));

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
                        let is_hover_gap = mouse_pos.map_or(false, |p| p.distance(mid_p) <= 14.0);
                        if is_hover_gap {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            if response.clicked() {
                                connection_to_add = Some((from_id, to_id));
                            }
                        }

                        let btn_rad = if is_hover_gap { 11.0 } else { 9.5 };
                        painter.circle_filled(mid_p, btn_rad, Color32::from_rgb(239, 68, 68));
                        painter.circle_stroke(mid_p, btn_rad, Stroke::new(1.5, Color32::WHITE));
                        painter.text(mid_p, egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(10.5), Color32::WHITE);
                    }
                }
            }
        }

        // Remove disconnected cable
        if let Some(idx) = connection_to_remove {
            self.connections.remove(idx);
            if self.advance_studio_state.selected_cable_idx == Some(idx) {
                self.advance_studio_state.selected_cable_idx = None;
            } else if let Some(sel) = self.advance_studio_state.selected_cable_idx {
                if sel > idx {
                    self.advance_studio_state.selected_cable_idx = Some(sel - 1);
                }
            }
            topology_changed = true;
        }

        // 5. Draw Live Rubber-Band Cable while Dragging (Forward or Reverse)
        if let (Some(from_id), Some(pos)) = (self.connecting_from_node, mouse_pos) {
            if let Some(from_n) = self.nodes.iter().find(|n| n.id == from_id) {
                let from_size = get_node_size(&from_n.node_type);
                let (p_start, p_end) = if self.connecting_is_input_drag {
                    // Reverse Drag: Originates from Input Pin (Left), target cursor searching for Output Pin
                    let p_in = if from_n.node_type == NodeType::Mixer {
                        let sat_world = self.get_satellite_pos(from_n.id, self.connecting_from_port, from_n.pos, &from_n.node_type);
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + sat_world.x,
                            canvas_rect.min.y + self.patchbay_pan.y + sat_world.y + 17.0,
                        )
                    } else {
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x,
                            canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + from_size.y * 0.5,
                        )
                    };
                    (pos, p_in)
                } else {
                    // Standard Drag: Originates from Output Pin (Right), target cursor searching for Input Pin
                    let p_out = if from_n.node_type == NodeType::Splitter {
                        let sat_world = self.get_satellite_pos(from_n.id, self.connecting_from_port, from_n.pos, &from_n.node_type);
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + sat_world.x + 52.0,
                            canvas_rect.min.y + self.patchbay_pan.y + sat_world.y + 17.0,
                        )
                    } else if from_n.node_type == NodeType::Crossover {
                        let pin_offset_y = if self.connecting_from_port == 1 { 14.0 } else { -14.0 };
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + from_size.x,
                            canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + from_size.y * 0.5 + pin_offset_y,
                        )
                    } else {
                        Pos2::new(
                            canvas_rect.min.x + self.patchbay_pan.x + from_n.pos.x + from_size.x,
                            canvas_rect.min.y + self.patchbay_pan.y + from_n.pos.y + from_size.y * 0.5,
                        )
                    };
                    (p_out, pos)
                };

                let dx = (p_end.x - p_start.x).abs().max(40.0) * 0.5;
                let cp1 = Pos2::new(p_start.x + dx, p_start.y);
                let cp2 = Pos2::new(p_end.x - dx, p_end.y);

                let steps = 22;
                let mut points = Vec::with_capacity(steps + 1);
                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                    let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                    points.push(Pos2::new(x, y));
                }

                // Drop shadow
                let shadow_pts: Vec<Pos2> = points.iter().map(|p| *p + Vec2::new(0.0, 2.0)).collect();
                painter.add(egui::Shape::line(shadow_pts, Stroke::new(3.5, Color32::from_black_alpha(24))));

                // Radiant neon wire
                painter.add(egui::Shape::line(points, Stroke::new(2.4, Color32::from_rgb(14, 165, 233))));

                // Glowing cursor plug tip
                let cursor_tip = if self.connecting_is_input_drag { p_start } else { p_end };
                painter.circle_filled(cursor_tip, 8.0, Color32::from_rgba_premultiplied(14, 165, 233, 80));
                painter.circle_filled(cursor_tip, 5.5, Color32::from_rgb(14, 165, 233));
                painter.circle_filled(cursor_tip, 2.5, Color32::WHITE);
                ui.ctx().request_repaint();
            }
        }

        // 6. Render Nodes & Input/Output Patch Pins with Dynamic Outline Animation
        let mut navigate_to_eq_preset: Option<String> = None;
        let mut navigate_to_main_eq = false;

        for (i, node) in self.nodes.iter().enumerate() {
            let is_active = self.selected_node == Some(node.id);
            let is_dragging = self.dragging_node == Some(i);

            let n_size = get_node_size(&node.node_type);
            let node_width = n_size.x;
            let raw_node_rect = Rect::from_min_size(
                Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + node.pos.x,
                    canvas_rect.min.y + self.patchbay_pan.y + node.pos.y,
                ),
                n_size,
            );

            let is_hovered = mouse_pos.map_or(false, |p| raw_node_rect.contains(p));
            let hover_anim = ui.ctx().animate_bool(ui.scope_id().with(("node_hover", node.id)), is_hovered);
            let node_rect = raw_node_rect.translate(Vec2::new(0.0, -hover_anim * 2.5));

            // Is this node part of the active flowing audio signal?
            let is_flowing = chain_eval.node_status(node.id) == SignalNodeStatus::Flowing;

            // Static analysis status for this node
            let node_rep = static_report.get_node_report(node.id as u32);
            let is_culprit_node = node_rep.map_or(false, |r| r.is_culprit) && self.chain_mode == ChainMode::DspActive;
            let is_overdriven_victim = node_rep.map_or(false, |r| matches!(r.status, NodeSafetyStatus::OverdrivenDownstream)) && self.chain_mode == ChainMode::DspActive;

            if is_active {
                painter.rect_stroke(node_rect.expand(1.5), 10.0, Stroke::new(1.5, Color32::from_rgb(129, 140, 248)), egui::StrokeKind::Outside);
            }

            // Dedicated interaction responses: separated IDs so context menu and hover tooltip never collide layers in egui
            let screen_node_rect = ts.mul_rect(node_rect);
            let node_resp = ui.interact(screen_node_rect, ui.scope_id().with(("patch_node_ctx", node.id)), egui::Sense::click());
            let tooltip_resp = ui.interact(screen_node_rect, ui.scope_id().with(("patch_node_tip", node.id)), egui::Sense::hover());

            // Static analysis diagnostic tooltip on hover (strictly suppressed if canvas is blocked, dragging, or context menu is open)
            if !is_canvas_blocked && is_hovered && !node_resp.is_pointer_button_down_on() {
                if is_culprit_node {
                    if let Some(r) = node_rep {
                        let recs = if !r.recommendations.is_empty() {
                            format!("\n\nSaran Solusi:\n• {}", r.recommendations.join("\n• "))
                        } else {
                            String::new()
                        };
                        tooltip_resp.show_tooltip_text(format!(
                            "💥 ASAL-USUL CLIPPING (CULPRIT)\n{}\nPeak Amplitudo: {:.2} dBFS (Defisit Headroom: +{:.2} dB){}",
                            r.reason, r.peak_dbfs, r.headroom_deficit_db, recs
                        ));
                    }
                } else if is_overdriven_victim {
                    if let Some(r) = node_rep {
                        tooltip_resp.show_tooltip_text(format!(
                            "⚠️ OVERDRIVEN DOWNSTREAM (KORBAN CLIPPING)\n{}\nPeak Amplitudo Masuk: {:.2} dBFS\nModul ini menerima sinyal terdistorsi dari modul upstream.",
                            r.reason, r.peak_dbfs
                        ));
                    }
                } else if node.node_type == NodeType::Source {
                    if let Some(r) = node_rep {
                        let peak_mag = r.output_interval.peak_magnitude();
                        let peak_db = r.output_interval.peak_dbfs();
                        let raw_headroom = -peak_db.min(0.0);
                        let headroom_db = if raw_headroom.abs() < 0.001 { 0.0 } else { raw_headroom.max(0.0) };
                        let track_name = static_report.context_track_title.as_deref().unwrap_or(&self.current_track.title);
                        let guard_status_text = if !self.clipping_detection_enabled {
                            "Headroom Guard: Nonaktif (Peringatan visual dibungkam)".to_string()
                        } else if self.clipping_threshold_db < -0.05 {
                            format!("Headroom Guard: Aktif [Strict {:+.1} dBFS]", self.clipping_threshold_db)
                        } else if self.clipping_threshold_db > 0.05 {
                            format!("Headroom Guard: Aktif [Relaxed {:+.1} dBFS]", self.clipping_threshold_db)
                        } else {
                            "Headroom Guard: Aktif [Standar 0.0 dBFS]".to_string()
                        };
                        tooltip_resp.show_tooltip_text(format!(
                            "🎛️ FILE SOURCE (KONTEKS AUDIO)\nLagu Aktif: {}\nAmplitudo Puncak: {:.3} ({:.2} dBFS)\nHeadroom Aman Tersisa: +{:.2} dB ke 0 dBFS\n{}\n\nStatus: Analisis statis mengukur headroom berdasarkan konteks lagu ini.",
                            track_name, peak_mag, peak_db, headroom_db, guard_status_text
                        ));
                    }
                }
            }
            let is_released_on_node = !is_canvas_blocked
                && ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary))
                && is_hovered
                && self.connecting_from_node.is_none()
                && !ui.input(|i| i.pointer.is_decidedly_dragging());
            let is_node_clicked = !is_canvas_blocked && (node_resp.clicked() || (response.clicked() && is_hovered) || is_released_on_node);
            if is_node_clicked {
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

            if !is_canvas_blocked {
                node_resp.context_menu(|ui| {
                ui.label(egui::RichText::new(&node.title).strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                ui.label(egui::RichText::new(&node.sub1).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                ui.separator();
                if is_culprit_node {
                    if ui.button(egui::RichText::new("🪄 Auto-Fix Headroom (Koreksi Otomatis)").strong().color(Color32::from_rgb(16, 185, 129))).clicked() {
                        auto_fix_node = Some(node.id);
                        ui.close();
                    }
                    ui.separator();
                }
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
            }

            // Dynamic Soft Shadow (expands & deepens when hovered / lifted)
            painter.rect_filled(
                node_rect.translate(Vec2::new(0.0, 2.5 + hover_anim * 2.5)),
                10.0,
                Color32::from_black_alpha((14.0 + hover_anim * 14.0) as u8),
            );

            let theme_color = if is_fallback {
                Color32::from_rgb(245, 158, 11)
            } else {
                match self.chain_mode {
                    ChainMode::BitPerfect => Color32::from_rgb(16, 185, 129),
                    ChainMode::DspActive => Color32::from_rgb(99, 102, 241),
                    ChainMode::OsResampled => Color32::from_rgb(245, 158, 11),
                }
            };

            let is_bypassed_resampler = node.node_type == NodeType::Driver && is_true_bp_highway;
            let is_bypassed_effect = (node.node_type.is_dsp_or_effect()
                && (self.chain_mode == ChainMode::BitPerfect || self.chain_mode == ChainMode::OsResampled))
                || is_bypassed_resampler;

            // Floating Honest Banner above Node 4 (APP RESAMPLER) whenever fallback, exclusive rejection, or Rubato Sinc rate adaptation is active!
            if node.node_type == NodeType::Driver {
                let is_hw_adapted = self.signal_path_report.as_ref().map_or(false, |r| r.is_hardware_rate_adapted());
                let is_ex_rejected = self.signal_path_report.as_ref().map_or(false, |r| r.is_exclusive_rejected());
                let is_resamp_active = self.signal_path_report.as_ref().map_or(false, |r| r.is_resampling);

                if is_fallback || is_ex_rejected || is_hw_adapted || (self.chain_mode != ChainMode::BitPerfect && is_resamp_active) {
                    let (float_txt, float_bg, float_stroke, float_fg) = if is_fallback || is_ex_rejected {
                        (
                            "⚠️ FALLBACK ADAPTED BY RUBATO SINC",
                            Color32::from_rgb(254, 243, 199),
                            Color32::from_rgb(245, 158, 11),
                            Color32::from_rgb(180, 83, 9),
                        )
                    } else if is_hw_adapted {
                        (
                            "⚡ HARDWARE ADAPTED BY RUBATO SINC",
                            Color32::from_rgb(224, 242, 254),
                            Color32::from_rgb(14, 165, 233),
                            Color32::from_rgb(3, 105, 161),
                        )
                    } else {
                        (
                            "🔄 ADAPTED BY RUBATO SINC",
                            Color32::from_rgb(238, 242, 255),
                            Color32::from_rgb(99, 102, 241),
                            Color32::from_rgb(67, 56, 202),
                        )
                    };
                    let float_rect = Rect::from_center_size(
                        Pos2::new(node_rect.center().x, node_rect.min.y - 14.0),
                        Vec2::new(222.0, 20.0),
                    );
                    painter.rect_filled(
                        float_rect.translate(Vec2::new(0.0, 1.5)),
                        10.0,
                        Color32::from_black_alpha(18),
                    );
                    painter.rect_filled(float_rect, 10.0, float_bg);
                    painter.rect_stroke(float_rect, 10.0, Stroke::new(1.2, float_stroke), egui::StrokeKind::Outside);
                    painter.text(
                        float_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        float_txt,
                        egui::FontId::proportional(9.5),
                        float_fg,
                    );
                }
            }

            let bg = if is_dragging {
                Color32::from_rgb(241, 245, 249)
            } else if is_bypassed_effect {
                Color32::from_rgba_premultiplied(248, 250, 252, 230)
            } else if !is_flowing {
                Color32::from_rgb(255, 248, 248)
            } else {
                Color32::WHITE
            };

            let is_portal_chassis = node.node_type == NodeType::Splitter || node.node_type == NodeType::Mixer;
            let is_pass_filter = matches!(node.node_type, NodeType::LowPassFilter | NodeType::HighPassFilter | NodeType::BandPassFilter);
            let is_custom_chassis = is_portal_chassis || is_pass_filter;

            if !is_custom_chassis {
                painter.rect_filled(node_rect, 10.0, bg);

                // Subtle Header Bar styling with grip dots
                let header_rect = Rect::from_min_size(node_rect.min, Vec2::new(n_size.x, 26.0));
                let header_bg = if is_hovered {
                    Color32::from_rgba_premultiplied(theme_color.r(), theme_color.g(), theme_color.b(), (16.0 + hover_anim * 20.0) as u8)
                } else {
                    Color32::from_rgb(248, 250, 252)
                };
                painter.rect_filled(header_rect, 10.0, header_bg);
                painter.rect_filled(Rect::from_min_size(Pos2::new(header_rect.min.x, header_rect.max.y - 6.0), Vec2::new(n_size.x, 6.0)), 0.0, header_bg);
                painter.line_segment(
                    [Pos2::new(node_rect.min.x, header_rect.max.y), Pos2::new(node_rect.max.x, header_rect.max.y)],
                    Stroke::new(0.8, Color32::from_rgb(226, 232, 240)),
                );

                // Tactile grip dots on header left
                let grip_x = node_rect.min.x + 10.0;
                let grip_cy = header_rect.center().y;
                let grip_col = if is_hovered { theme_color } else { Color32::from_rgb(203, 213, 225) };
                for gr_r in 0..2 {
                    for gr_c in 0..2 {
                        painter.circle_filled(
                            Pos2::new(grip_x + gr_c as f32 * 3.5, grip_cy - 2.5 + gr_r as f32 * 5.0),
                            1.0,
                            grip_col,
                        );
                    }
                }

                // Dynamic Outline Styling & Animation
                if is_bypassed_effect {
                    // Option B: Effect is BYPASSED / DILEWATI — calm muted slate border & direct dashed wire inside
                    painter.rect_stroke(node_rect, 10.0, Stroke::new(1.2, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                    
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

                    // Click on bypassed node to re-engage Modular DSP mode
                    if is_node_clicked {
                        switch_mode = Some(ChainMode::DspActive);
                    }
                } else if is_flowing {
                    let border_color = if is_culprit_node {
                        Color32::from_rgb(239, 68, 68) // Crimson hazard for clipping origin
                    } else if is_overdriven_victim {
                        Color32::from_rgb(245, 158, 11) // Amber warning for downstream victim
                    } else {
                        theme_color
                    };

                    if self.is_playing {
                        // Pulsing luminous aura matching theme or hazard
                        let aura_alpha = (60.0 + 35.0 * (self.visualizer_anim_phase * 4.0).sin()) as u8;
                        painter.rect_stroke(
                            node_rect.expand(2.0),
                            10.0,
                            Stroke::new(1.8, Color32::from_rgba_premultiplied(border_color.r(), border_color.g(), border_color.b(), aura_alpha / 2)),
                            egui::StrokeKind::Outside,
                        );

                        // Animated laser beacons moving along the rectangle's perimeter
                        let p_len = 2.0 * (node_rect.width() + node_rect.height());
                        let travel_dist = (self.visualizer_anim_phase * 140.0 + (node.id as f32) * 55.0) % p_len;
                        let b1 = get_rect_perimeter_pos(node_rect, travel_dist);
                        let b2 = get_rect_perimeter_pos(node_rect, travel_dist + p_len * 0.5);

                        let (b1_col, b2_col) = if is_culprit_node {
                            (Color32::from_rgb(239, 68, 68), Color32::from_rgb(252, 165, 165))
                        } else if is_overdriven_victim {
                            (Color32::from_rgb(245, 158, 11), Color32::from_rgb(253, 230, 138))
                        } else {
                            match self.chain_mode {
                                ChainMode::BitPerfect => (Color32::from_rgb(52, 211, 153), Color32::from_rgb(34, 211, 238)),
                                ChainMode::DspActive => (Color32::from_rgb(129, 140, 248), Color32::from_rgb(192, 132, 252)),
                                ChainMode::OsResampled => (Color32::from_rgb(251, 191, 36), Color32::from_rgb(245, 158, 11)),
                            }
                        };

                        painter.circle_filled(b1, 3.8, b1_col);
                        painter.circle_filled(b1, 1.8, Color32::WHITE);
                        painter.circle_filled(b2, 3.8, b2_col);
                        painter.circle_filled(b2, 1.8, Color32::WHITE);

                        // Flowing border
                        let stroke_w = 1.6 + 0.4 * (self.visualizer_anim_phase * 3.5).sin();
                        painter.rect_stroke(node_rect, 8.0, Stroke::new(stroke_w, border_color), egui::StrokeKind::Outside);
                    } else {
                        // Ready / Paused: Crisp theme border
                        let stroke_w = if is_culprit_node { 1.8 } else { 1.3 };
                        painter.rect_stroke(node_rect, 8.0, Stroke::new(stroke_w, border_color), egui::StrokeKind::Outside);
                    }
                } else {
                    // BROKEN / NO SIGNAL: Pulsing Crimson Alert
                    let warn_t = (ui.input(|i| i.time) * 4.0).sin().abs() as f32;
                    let stroke_w = 1.5 + warn_t * 0.8;
                    painter.rect_stroke(node_rect, 8.0, Stroke::new(stroke_w, Color32::from_rgb(239, 68, 68)), egui::StrokeKind::Outside);
                    painter.rect_stroke(node_rect.expand(2.0), 10.0, Stroke::new(1.5, Color32::from_rgba_premultiplied(239, 68, 68, 35)), egui::StrokeKind::Outside);
                }
            }

            // =========================================================================
            // PASS FILTER REACTORS (LPF, HPF, BPF): BESPOKE CHASSIS & FULL ANIMATION
            // =========================================================================
            if is_pass_filter {
                self.render_pass_filter_node(
                    ui,
                    &painter,
                    node,
                    node_rect,
                    node_width,
                    &mut param_change,
                    &mut switch_mode,
                    is_node_clicked,
                    is_hovered,
                    is_dragging,
                    hover_anim,
                    is_flowing,
                    is_bypassed_effect,
                    is_culprit_node,
                    is_overdriven_victim,
                    theme_color,
                );
            }

            // 2. Title Text (Foreground: high contrast, rendered ON TOP of wave background)
            if !is_custom_chassis {
                let title_col = if is_bypassed_effect {
                    Color32::from_rgb(100, 116, 139)
                } else {
                    Color32::from_rgb(15, 23, 42)
                };
                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 13.0),
                    egui::Align2::CENTER_CENTER,
                    &node.title,
                    egui::FontId::proportional(12.5),
                    title_col,
                );
            }

            // Culprit or Overdriven Header Badge
            if is_culprit_node {
                let deficit_db = node_rep.map_or(0.0, |r| r.headroom_deficit_db);
                let badge_text = format!("💥 +{:.1}dB", deficit_db);
                let badge_rect = Rect::from_center_size(
                    Pos2::new(node_rect.max.x - 28.0, node_rect.min.y + 13.0),
                    Vec2::new(48.0, 15.0),
                );
                painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(254, 226, 226));
                painter.rect_stroke(badge_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(239, 68, 68)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_text,
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(185, 28, 28),
                );
            } else if is_overdriven_victim {
                let badge_text = "⚠️ OVERDRIVE";
                let badge_rect = Rect::from_center_size(
                    Pos2::new(node_rect.max.x - 36.0, node_rect.min.y + 13.0),
                    Vec2::new(64.0, 15.0),
                );
                painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(254, 243, 199));
                painter.rect_stroke(badge_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(245, 158, 11)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_text,
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(180, 83, 9),
                );
            }

            if node.node_type == NodeType::Crossover {
                let default_f = 500.0;
                let freq = if node.param_val > 10.0 { node.param_val } else { default_f };
                let active_freq = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(freq);

                let (accent_col, track_col, stroke_col, label_prefix) = (Color32::from_rgb(168, 85, 247), Color32::from_rgb(250, 245, 255), Color32::from_rgb(147, 51, 234), "X-Over");

                // Interactive Mini Cutoff Slider & High-Contrast Dual Telemetry Badges
                if !is_bypassed_effect {
                    let slider_track_rect = Rect::from_min_size(
                        Pos2::new(node_rect.min.x + 24.0, node_rect.min.y + 35.0),
                        Vec2::new((node_width - 48.0).max(10.0), 5.0),
                    );

                    let fill_ratio = ((active_freq / 20.0).log10() / (1000.0_f32).log10()).clamp(0.0, 1.0);
                    let thumb_x = slider_track_rect.min.x + slider_track_rect.width() * fill_ratio;
                    let cy = slider_track_rect.center().y;

                    painter.rect_filled(
                        Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(slider_track_rect.width(), 4.0)),
                        2.0,
                        track_col,
                    );
                    painter.rect_filled(
                        Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(thumb_x - slider_track_rect.min.x, 4.0)),
                        2.0,
                        accent_col,
                    );
                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.0, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.0, Stroke::new(1.8, stroke_col));

                    let readout = if active_freq >= 1000.0 {
                        format!("{} {:.1}k", label_prefix, active_freq / 1000.0)
                    } else {
                        format!("{} {:.0}Hz", label_prefix, active_freq)
                    };

                    let slope_str = if node.sub2.contains("48dB") {
                        "48dB"
                    } else if node.sub2.contains("24dB") {
                        "24dB"
                    } else {
                        "12dB"
                    };

                    let badge_y = node_rect.min.y + 61.0;
                    let badge_w = (node_width - 34.0) * 0.48;
                    let badge_h = 20.0;

                    // Left Badge: Frequency Readout (Luminous & high-contrast)
                    let b_left_rect = Rect::from_center_size(
                        Pos2::new(node_rect.center().x - badge_w * 0.52, badge_y),
                        Vec2::new(badge_w, badge_h),
                    );
                    painter.rect_filled(b_left_rect, 3.5, track_col);
                    painter.rect_stroke(b_left_rect, 3.5, Stroke::new(1.2, stroke_col), egui::StrokeKind::Outside);
                    painter.text(
                        b_left_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(10.5),
                        stroke_col,
                    );

                    // Right Badge: Slope & Quality Telemetry (Crisp White on Dark Pill)
                    let b_right_rect = Rect::from_center_size(
                        Pos2::new(node_rect.center().x + badge_w * 0.52, badge_y),
                        Vec2::new(badge_w, badge_h),
                    );
                    painter.rect_filled(b_right_rect, 3.5, Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(b_right_rect, 3.5, Stroke::new(1.0, Color32::from_rgb(71, 85, 105)), egui::StrokeKind::Outside);

                    let tele_sub2 = if node.node_type == NodeType::Crossover {
                        format!("LR4 {}", slope_str)
                    } else if let Some(idx) = node.sub2.find("Q ") {
                        let q_str = node.sub2[idx + 2..].split_whitespace().next().unwrap_or("0.7");
                        format!("{} • Q{}", slope_str, q_str)
                    } else {
                        format!("Slope {}", slope_str)
                    };

                    painter.text(
                        b_right_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        tele_sub2,
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(241, 245, 249),
                    );
                } else {
                    let b_rect = Rect::from_center_size(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 44.0),
                        Vec2::new(154.0, 22.0),
                    );
                    painter.rect_filled(b_rect, 4.0, Color32::from_rgb(241, 245, 249));
                    painter.rect_stroke(b_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                    painter.text(
                        b_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bit-Perfect)",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(100, 116, 139),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 66.0),
                        egui::Align2::CENTER_CENTER,
                        "Passthrough 1:1 Stream",
                        egui::FontId::proportional(10.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::Mixer {
                let active_mix = node.param_val.clamp(0.0, 1.0);
                let weight_a = 1.0 - active_mix;
                let weight_b = active_mix;

                let conn_a = self.connections.iter().any(|c| c.to_node == node.id && c.to_port == 0);
                let conn_b = self.connections.iter().any(|c| c.to_node == node.id && c.to_port == 1);
                let is_active_audio = self.is_playing && is_flowing && (conn_a || conn_b);

                let instance_idx = self.nodes.iter()
                    .filter(|n| n.node_type == NodeType::Mixer)
                    .position(|n| n.id == node.id)
                    .unwrap_or(0);
                let theme = get_instance_color_theme(instance_idx);
                let col_a = theme.col_a;
                let col_b = theme.col_b;
                let pal_a = theme.pal_a;
                let pal_b = theme.pal_b;
                let lerp_rgb = |c1: Color32, c2: Color32, k: f32| -> Color32 {
                    let t = k.clamp(0.0, 1.0);
                    Color32::from_rgb(
                        (c1.r() as f32 * (1.0 - t) + c2.r() as f32 * t).round() as u8,
                        (c1.g() as f32 * (1.0 - t) + c2.g() as f32 * t).round() as u8,
                        (c1.b() as f32 * (1.0 - t) + c2.b() as f32 * t).round() as u8,
                    )
                };
                let col_mixed = if conn_a && conn_b { lerp_rgb(col_a, col_b, active_mix) }
                    else if conn_a { col_a } else if conn_b { col_b }
                    else { Color32::from_rgb(168, 85, 247) };

                let t = ui.input(|i| i.time) as f32;
                let pulse = (t * 3.5).sin().abs();

                // Compute Satellite World & Screen Coordinates
                let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                let sat_a_screen = Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x,
                    canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y,
                );
                let sat_b_screen = Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x,
                    canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y,
                );
                let sat_a_rect = Rect::from_min_size(sat_a_screen, Vec2::new(52.0, 34.0));
                let sat_b_rect = Rect::from_min_size(sat_b_screen, Vec2::new(52.0, 34.0));

                let is_sat_a_hovered = !is_canvas_blocked && raw_pointer_pos.map_or(false, |p| sat_a_rect.contains(p));
                let is_sat_b_hovered = !is_canvas_blocked && raw_pointer_pos.map_or(false, |p| sat_b_rect.contains(p));
                let is_sat_a_dragged = self.dragging_satellite == Some((node.id, 0));
                let is_sat_b_dragged = self.dragging_satellite == Some((node.id, 1));
                let is_gate_hovered = is_hovered;
                let is_gate_selected = self.selected_node == Some(node.id);
                let is_gate_dragged = self.dragging_node == Some(node.id);

                let is_family_active = is_gate_hovered || is_gate_selected || is_gate_dragged || is_sat_a_hovered || is_sat_b_hovered || is_sat_a_dragged || is_sat_b_dragged;

                let vortex_pos = Pos2::new(node_rect.min.x + 18.0, node_rect.min.y + 44.0);

                // =========================================================================
                // 1. GERBANG UTAMA (MODERN FUSION RECEPTOR CONSOLE) - 140 x 78 px
                // =========================================================================
                painter.rect_filled(node_rect.translate(Vec2::new(0.0, 3.0)), 10.0, Color32::from_black_alpha(16));

                if is_family_active {
                    painter.rect_stroke(
                        node_rect.expand(2.0),
                        11.0,
                        Stroke::new(2.0, Color32::from_rgba_unmultiplied(col_mixed.r(), col_mixed.g(), col_mixed.b(), 70)),
                        egui::StrokeKind::Outside,
                    );
                }

                painter.rect_filled(node_rect, 10.0, Color32::WHITE);

                let header_rect = Rect::from_min_size(node_rect.min, Vec2::new(node_rect.width(), 22.0));
                let header_bg = if is_family_active {
                    Color32::from_rgba_unmultiplied(col_mixed.r(), col_mixed.g(), col_mixed.b(), 18)
                } else {
                    Color32::from_rgb(248, 250, 252)
                };
                painter.rect_filled(header_rect, 10.0, header_bg);
                painter.rect_filled(Rect::from_min_size(Pos2::new(header_rect.min.x, header_rect.max.y - 6.0), Vec2::new(node_rect.width(), 6.0)), 0.0, header_bg);
                painter.line_segment(
                    [Pos2::new(node_rect.min.x, header_rect.max.y), Pos2::new(node_rect.max.x, header_rect.max.y)],
                    Stroke::new(0.8, Color32::from_rgb(241, 245, 249)),
                );

                let border_col = if is_family_active {
                    col_mixed
                } else if is_active_audio {
                    Color32::from_rgba_unmultiplied(col_mixed.r(), col_mixed.g(), col_mixed.b(), 200)
                } else {
                    Color32::from_rgb(226, 232, 240)
                };
                painter.rect_stroke(node_rect, 10.0, Stroke::new(if is_family_active { 1.8 } else { 1.2 }, border_col), egui::StrokeKind::Outside);

                // 4 Corner Allen Bolts
                for &(dx, dy) in &[(5.5, 5.5), (node_rect.width() - 5.5, 5.5), (5.5, node_rect.height() - 5.5), (node_rect.width() - 5.5, node_rect.height() - 5.5)] {
                    let bp = Pos2::new(node_rect.min.x + dx, node_rect.min.y + dy);
                    painter.circle_filled(bp, 1.2, Color32::from_rgb(226, 232, 240));
                }

                // Left: Title Badge Pill
                let title_pill = Rect::from_min_size(Pos2::new(node_rect.min.x + 8.0, node_rect.min.y + 4.0), Vec2::new(76.0, 14.0));
                painter.rect_filled(title_pill, 4.0, Color32::from_rgb(241, 245, 249));
                painter.text(
                    Pos2::new(title_pill.min.x + 4.0, title_pill.center().y),
                    egui::Align2::LEFT_CENTER,
                    "🎚️ MIXER",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(15, 23, 42),
                );
                painter.text(
                    Pos2::new(title_pill.max.x - 4.0, title_pill.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!("CH-0{}", theme.instance_num),
                    egui::FontId::monospace(7.2),
                    Color32::from_rgb(100, 116, 139),
                );

                // Right: Status LED
                let led_pos = Pos2::new(node_rect.max.x - 12.0, node_rect.min.y + 11.0);
                if is_active_audio {
                    let led_pulse = (t * 5.0).sin().abs();
                    let halo_alpha = (60.0 + 90.0 * led_pulse) as u8;
                    painter.circle_stroke(led_pos, 4.2, Stroke::new(1.0, Color32::from_rgba_unmultiplied(34, 197, 94, halo_alpha)));
                    painter.circle_filled(led_pos, 2.2, Color32::from_rgb(34, 197, 94));
                    painter.circle_filled(led_pos, 1.0, Color32::WHITE);
                } else if conn_a || conn_b {
                    painter.circle_filled(led_pos, 2.0, Color32::from_rgb(52, 211, 153));
                } else {
                    painter.circle_filled(led_pos, 2.0, Color32::from_rgb(203, 213, 225));
                }

                // Left: Fusion Core Vortex Aperture (Obsidian Chamber with Orbiting Quantum Plasma)
                painter.circle_filled(vortex_pos, 10.0, Color32::from_rgb(15, 23, 42));
                let vortex_alpha = if is_active_audio { (180.0 + 75.0 * pulse) as u8 } else { 90 };
                painter.circle_stroke(
                    vortex_pos,
                    9.5,
                    Stroke::new(1.2, Color32::from_rgba_unmultiplied(col_mixed.r(), col_mixed.g(), col_mixed.b(), vortex_alpha)),
                );

                let spin_spd = if is_active_audio { 4.5 } else { 1.2 };
                let ang = t * spin_spd;
                let r_orb = 4.2;
                let orb_a = Pos2::new(vortex_pos.x + ang.cos() * r_orb, vortex_pos.y + ang.sin() * r_orb);
                let orb_b = Pos2::new(vortex_pos.x - ang.cos() * r_orb, vortex_pos.y - ang.sin() * r_orb);

                painter.circle_filled(orb_a, 2.0, col_a);
                painter.circle_stroke(orb_a, 3.2, Stroke::new(0.8, Color32::from_rgba_unmultiplied(col_a.r(), col_a.g(), col_a.b(), 120)));
                painter.circle_filled(orb_b, 2.0, col_b);
                painter.circle_stroke(orb_b, 3.2, Stroke::new(0.8, Color32::from_rgba_unmultiplied(col_b.r(), col_b.g(), col_b.b(), 120)));
                painter.circle_filled(vortex_pos, 1.8, Color32::WHITE);

                if is_active_audio && conn_a && conn_b && weight_a > 0.05 && weight_b > 0.05 {
                    for ring in 0..2 {
                        let ring_r = 5.0 + ((t * 2.5 + ring as f32 * 0.5).fract()) * 16.0;
                        let ring_alpha = ((1.0 - ring_r / 21.0) * 160.0) as u8;
                        painter.circle_stroke(
                            vortex_pos,
                            ring_r,
                            Stroke::new(1.2, Color32::from_rgba_unmultiplied(col_mixed.r(), col_mixed.g(), col_mixed.b(), ring_alpha)),
                        );
                    }
                }

                // Crossfader Slider Track (Precision Studio Fader)
                let track_min_x = node_rect.min.x + 36.0;
                let track_max_x = node_rect.max.x - 22.0;
                let track_y = node_rect.min.y + 44.0;
                let track_w = track_max_x - track_min_x;
                let knob_x = track_min_x + active_mix * track_w;

                let track_rect = Rect::from_min_size(Pos2::new(track_min_x, track_y - 2.5), Vec2::new(track_w, 5.0));
                painter.rect_filled(track_rect, 2.5, Color32::from_rgb(241, 245, 249));
                painter.rect_stroke(track_rect, 2.5, Stroke::new(0.8, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

                let cnx = track_min_x + track_w * 0.5;
                painter.line_segment([Pos2::new(cnx, track_y - 4.5), Pos2::new(cnx, track_y + 4.5)],
                    Stroke::new(1.0, Color32::from_rgb(148, 163, 184)));

                if (knob_x - cnx).abs() > 1.0 {
                    let (fs, fe) = if knob_x > cnx { (cnx, knob_x) } else { (knob_x, cnx) };
                    let fc = if knob_x > cnx { col_b } else { col_a };
                    painter.rect_filled(Rect::from_min_max(Pos2::new(fs, track_y - 2.0), Pos2::new(fe, track_y + 2.0)), 2.0,
                        Color32::from_rgba_unmultiplied(fc.r(), fc.g(), fc.b(), 180));
                }

                let knob_pos = Pos2::new(knob_x, track_y);
                painter.circle_filled(knob_pos + Vec2::new(0.0, 1.2), 5.2, Color32::from_black_alpha(25));
                painter.circle_filled(knob_pos, 5.2, Color32::WHITE);
                painter.circle_stroke(knob_pos, 5.2, Stroke::new(1.6, col_mixed));
                painter.circle_filled(knob_pos, 2.0, col_mixed);

                painter.text(Pos2::new(track_min_x, track_y + 11.0), egui::Align2::LEFT_TOP,
                    format!("{:.0}% A", weight_a * 100.0), egui::FontId::monospace(7.8), col_a);
                painter.text(Pos2::new(track_max_x, track_y + 11.0), egui::Align2::RIGHT_TOP,
                    format!("{:.0}% B", weight_b * 100.0), egui::FontId::monospace(7.8), col_b);

                painter.text(Pos2::new(node_rect.center().x, node_rect.max.y - 6.0), egui::Align2::CENTER_BOTTOM,
                    "SUMMING BUS", egui::FontId::monospace(7.2), Color32::from_rgb(148, 163, 184));

                // =========================================================================
                // 2. KOTAK MESIN SATELIT (INPUT POD A & B) & TRANSMISSION
                // =========================================================================
                for port in 0..2u8 {
                    let sat_rect = if port == 0 { sat_a_rect } else { sat_b_rect };
                    let is_conn = if port == 0 { conn_a } else { conn_b };
                    let pod_col = if port == 0 { col_a } else { col_b };
                    let pal = if port == 0 { &pal_a } else { &pal_b };
                    let is_pod_dragged = if port == 0 { is_sat_a_dragged } else { is_sat_b_dragged };
                    let is_pod_hovered = if port == 0 { is_sat_a_hovered } else { is_sat_b_hovered };
                    let weight = if port == 0 { weight_a } else { weight_b };

                    if is_pod_hovered && !is_canvas_blocked {
                        let screen_sat_rect = ts.mul_rect(sat_rect);
                        let sat_tip = ui.interact(screen_sat_rect, ui.scope_id().with(("sat_tip_mix", node.id, port)), egui::Sense::hover());
                        sat_tip.show_tooltip_text(format!(
                            "📡 Satelit Input {} // 🎚️ MIXER #{}\n• Klik tahan & geser untuk memindah posisi\n• Tarik kabel ke port lingkaran kiri untuk menyambung",
                            if port == 0 { "A" } else { "B" }, theme.instance_num
                        ));
                    }

                    // Shadow + outer glow
                    let shadow_y = if is_pod_dragged { 4.0 } else if is_pod_hovered { 2.5 } else { 1.8 };
                    painter.rect_filled(sat_rect.translate(Vec2::new(0.0, shadow_y)), 7.0,
                        Color32::from_black_alpha(if is_pod_dragged { 32 } else if is_pod_hovered { 20 } else { 12 }));

                    if is_family_active || is_pod_hovered {
                        painter.rect_stroke(
                            sat_rect.expand(1.5),
                            8.0,
                            Stroke::new(1.5, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), if is_pod_dragged || is_pod_hovered { 180 } else { 80 })),
                            egui::StrokeKind::Outside,
                        );
                    }

                    // Body
                    painter.rect_filled(sat_rect, 7.0, Color32::WHITE);
                    let pod_border = if is_pod_dragged || is_pod_hovered {
                        pod_col
                    } else if is_conn {
                        Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 180)
                    } else {
                        Color32::from_rgb(226, 232, 240)
                    };
                    painter.rect_stroke(sat_rect, 7.0, Stroke::new(if is_pod_dragged || is_pod_hovered { 1.6 } else { 1.2 }, pod_border), egui::StrokeKind::Outside);

                    // Micro LED Status Indicator
                    let sat_led = Pos2::new(sat_rect.min.x + 8.5, sat_rect.min.y + 7.5);
                    if is_active_audio && is_conn && weight > 0.01 {
                        let l_pulse = (t * 6.0 + port as f32 * 1.57).sin().abs();
                        painter.circle_stroke(sat_led, 3.5, Stroke::new(1.0, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), (100.0 + 120.0 * l_pulse) as u8)));
                        painter.circle_filled(sat_led, 2.0, pod_col);
                        painter.circle_filled(sat_led, 0.9, Color32::WHITE);
                    } else if is_conn {
                        painter.circle_filled(sat_led, 1.8, pod_col);
                    } else {
                        painter.circle_filled(sat_led, 1.8, Color32::from_rgb(203, 213, 225));
                    }

                    // Tactile Grip Dots (Top Right)
                    for gx in [-3.5f32, 0.0, 3.5] {
                        painter.line_segment(
                            [Pos2::new(sat_rect.max.x - 10.0 + gx, sat_rect.min.y + 3.0),
                             Pos2::new(sat_rect.max.x - 10.0 + gx, sat_rect.min.y + 5.5)],
                            Stroke::new(0.8, Color32::from_rgb(226, 232, 240)),
                        );
                    }

                    // Modern Badge Chip
                    let badge_rect = Rect::from_center_size(Pos2::new(sat_rect.center().x + 2.0, sat_rect.center().y - 2.0), Vec2::new(32.0, 13.0));
                    painter.rect_filled(badge_rect, 3.0, Color32::from_rgb(248, 250, 252));
                    painter.text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        if port == 0 { "IN A" } else { "IN B" },
                        egui::FontId::monospace(7.8),
                        if is_conn { pod_col } else { Color32::from_rgb(100, 116, 139) },
                    );

                    // Channel Sub-code Text (e.g. #1.A)
                    painter.text(
                        Pos2::new(sat_rect.center().x + 2.0, sat_rect.max.y - 4.5),
                        egui::Align2::CENTER_BOTTOM,
                        format!("#{}.{}", theme.instance_num, if port == 0 { "A" } else { "B" }),
                        egui::FontId::monospace(6.8),
                        if is_conn { Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 180) } else { Color32::from_rgb(148, 163, 184) },
                    );

                    // Left Cable Dock Pin (Chrome Ring + Illuminated Socket)
                    let pin_pos = Pos2::new(sat_rect.min.x + 3.0, sat_rect.center().y);
                    painter.circle_stroke(pin_pos, 4.5, Stroke::new(1.2, Color32::from_rgb(203, 213, 225)));
                    if is_conn {
                        painter.circle_filled(pin_pos, 3.0, pod_col);
                        painter.circle_filled(pin_pos, 1.2, Color32::WHITE);
                    } else {
                        painter.circle_filled(pin_pos, 2.5, Color32::from_rgb(241, 245, 249));
                    }

                    // === RIGHT APERTURE: "DOORWAY LIGHT SPILL" (Scaled by slider weight) ===
                    let mouth_x = sat_rect.max.x;
                    let mouth_cy = sat_rect.center().y;
                    let aperture_active = is_conn && weight > 0.01;
                    let base_alpha = if is_active_audio && aperture_active {
                        ((160.0 + 80.0 * (t * 6.0 + port as f32 * 1.57).sin()) * weight).clamp(0.0, 255.0) as u8
                    } else if aperture_active {
                        (45.0 * weight).clamp(0.0, 255.0) as u8
                    } else {
                        0u8
                    };

                    // Slit line
                    if base_alpha > 0 {
                        painter.line_segment(
                            [Pos2::new(mouth_x - 1.5, mouth_cy - 7.5), Pos2::new(mouth_x - 1.5, mouth_cy + 7.5)],
                            Stroke::new(2.5, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), base_alpha)));

                        // Doorway light fan (scaled by weight)
                        let scale_factor = 0.35 + 0.65 * weight;
                        let fan_colors = [base_alpha, (base_alpha as u16 * 60 / 100) as u8,
                            (base_alpha as u16 * 35 / 100) as u8, (base_alpha as u16 * 18 / 100) as u8];
                        let fan_widths = [3.0f32 * scale_factor, 7.0 * scale_factor, 13.0 * scale_factor, 20.0 * scale_factor];
                        let fan_depths = [4.0f32 * scale_factor, 9.0 * scale_factor, 16.0 * scale_factor, 24.0 * scale_factor];
                        for fi in 0..4 {
                            let fa = fan_colors[fi];
                            let hw = fan_widths[fi];
                            let depth = fan_depths[fi];
                            let tri = vec![
                                Pos2::new(mouth_x, mouth_cy - 2.0),
                                Pos2::new(mouth_x, mouth_cy + 2.0),
                                Pos2::new(mouth_x + depth, mouth_cy + hw),
                                Pos2::new(mouth_x + depth, mouth_cy - hw),
                            ];
                            painter.add(egui::Shape::convex_polygon(
                                tri,
                                Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), fa),
                                Stroke::NONE,
                            ));
                        }
                        painter.circle_filled(Pos2::new(mouth_x - 1.0, mouth_cy), 2.0, Color32::WHITE);
                    }

                    // === TRANSMISSION DYNAMICS (Pod -> Gate Vortex) ===
                    let p_pod_exit = Pos2::new(mouth_x + 2.0, mouth_cy);
                    let p_target = vortex_pos;
                    let raw_dx = p_target.x - p_pod_exit.x;
                    let raw_dy = (p_target.y - p_pod_exit.y).abs();
                    let dist = p_pod_exit.distance(p_target);
                    let is_direct = raw_dx > 10.0 && raw_dy <= raw_dx * 2.5 && dist <= 260.0;

                    if is_direct {
                        // DIRECT FLUID TELEMETRY STREAM (Color blends as particles approach fusion core)
                        let bezier_eval = |p0: Pos2, p3: Pos2, u: f32| -> Pos2 {
                            let cp1 = Pos2::new(p0.x + raw_dx * 0.4, p0.y);
                            let cp2 = Pos2::new(p3.x - raw_dx * 0.4, p3.y);
                            let v = 1.0 - u;
                            Pos2::new(
                                v*v*v*p0.x + 3.0*v*v*u*cp1.x + 3.0*v*u*u*cp2.x + u*u*u*p3.x,
                                v*v*v*p0.y + 3.0*v*v*u*cp1.y + 3.0*v*u*u*cp2.y + u*u*u*p3.y,
                            )
                        };

                        if is_active_audio && is_conn && weight > 0.01 {
                            // Active: particle count proportional to weight
                            let count = ((36.0 * weight).round() as usize).max(4);
                            let phi = 0.6180339887f32;
                            for i in 0..count {
                                let seed = i as f32 * phi;
                                let speed = 0.42 + (seed.fract() * 0.28);
                                let u = ((i as f32 / count as f32) + t * speed).fract();
                                let u_prev = (u - 0.045).max(0.0);

                                // Lateral spread & wave turbulence
                                let spread = ((i as f32 * 1.91).sin()) * (2.0 + 4.5 * weight) * (0.3 + 0.7 * (u * (1.0 - u) * 4.0));
                                let wave = (u * 11.0 - t * 4.8 + seed * 6.28).sin() * 1.4;
                                let total_offset = spread + wave;

                                let bp = bezier_eval(p_pod_exit, p_target, u);
                                let bp_prv = bezier_eval(p_pod_exit, p_target, u_prev);

                                let pos = Pos2::new(bp.x, bp.y + total_offset);
                                let prv = Pos2::new(bp_prv.x, bp_prv.y + total_offset);

                                // Dynamic color blending near fusion point:
                                // As u approaches 1.0, stream A and stream B blend colors!
                                let blend_k = ((u - 0.45) / 0.55).clamp(0.0, 1.0).powf(1.3);
                                let p_col_base = pal[i % 8];
                                let p_col = lerp_rgb(p_col_base, col_mixed, blend_k);
                                let r = (1.1 + (seed * 2.9).fract() * 1.2) * (0.7 + 0.5 * weight);

                                // Comet trail & core
                                painter.line_segment([prv, pos],
                                    Stroke::new(r * 0.9, Color32::from_rgba_unmultiplied(p_col.r(), p_col.g(), p_col.b(), 90)));
                                painter.circle_filled(pos, r + 1.1, Color32::from_rgba_unmultiplied(p_col.r(), p_col.g(), p_col.b(), 35));
                                painter.circle_filled(pos, r, p_col);
                            }
                        } else if is_conn && weight > 0.01 {
                            // Idle: faint motes
                            let idle_count = ((6.0 * weight).round() as usize).max(2);
                            for i in 0..idle_count {
                                let seed = i as f32 * 0.618;
                                let u = ((i as f32 / idle_count as f32) + t * 0.14).fract();
                                let bp = bezier_eval(p_pod_exit, p_target, u);
                                let wave = (u * 8.0 - t * 2.0 + seed * 6.28).sin() * 1.5;
                                let p_col = pal[i % 8];
                                painter.circle_filled(Pos2::new(bp.x, bp.y + wave), 1.5,
                                    Color32::from_rgba_unmultiplied(p_col.r(), p_col.g(), p_col.b(), 90));
                            }
                        }
                    } else if is_conn && weight > 0.01 {
                        // =================================================================
                        // WARP: BLACK HOLE (In-Portal) -> WHITE HOLE (Out-Portal)
                        // Spawns farther away from mouths (~65px)
                        // =================================================================
                        let p_bh = Pos2::new(p_pod_exit.x + 65.0, p_pod_exit.y);
                        let p_wh = Pos2::new(p_target.x - 65.0, p_target.y + if port == 0 { -10.0 } else { 10.0 });

                        // Subspace Einstein-Rosen bridge
                        let link_alpha = if is_active_audio { 60 } else { 25 };
                        for step in 0..10 {
                            let u = step as f32 / 9.0;
                            let lp = p_bh + (p_wh - p_bh) * u;
                            let wave_off = (u * 6.0 + t * 3.0).sin() * 1.5;
                            painter.circle_filled(Pos2::new(lp.x, lp.y + wave_off), 1.1,
                                Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), link_alpha));
                        }

                        // --- 1. BLACK HOLE (Gravitational Inward Swirl) ---
                        // Event horizon
                        painter.circle_filled(p_bh, 7.5, Color32::from_rgb(4, 6, 14));
                        // Photon sphere / gravitational lensing
                        painter.circle_stroke(p_bh, 8.0, Stroke::new(1.8, Color32::from_rgb(255, 255, 255)));
                        // Glowing accretion disk
                        painter.circle_stroke(p_bh, 12.0 + 1.5 * pulse, Stroke::new(2.2,
                            Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 180)));
                        painter.circle_stroke(p_bh, 16.0 + 2.5 * pulse, Stroke::new(1.4,
                            Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 90)));
                        // Orbiting accretion photons
                        for k in 0..4 {
                            let ang = t * 8.0 + k as f32 * 1.57;
                            let r_orb = 11.5 + 2.0 * (k as f32 * 0.7).sin();
                            painter.circle_filled(Pos2::new(p_bh.x + r_orb * ang.cos(), p_bh.y + r_orb * ang.sin()), 1.4, Color32::WHITE);
                        }

                        // --- 2. WHITE HOLE (Radiant Singularity Outward Swirl) ---
                        // Luminous quasar core
                        painter.circle_filled(p_wh, 5.0, Color32::WHITE);
                        painter.circle_stroke(p_wh, 7.5 + pulse * 1.5, Stroke::new(2.2,
                            Color32::from_rgba_unmultiplied(255, 255, 255, 220)));
                        painter.circle_filled(p_wh, 14.0, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 40));
                        // Ejection shockwave rings
                        let wave_r = 6.0 + ((t * 2.8).fract()) * 18.0;
                        painter.circle_stroke(p_wh, wave_r, Stroke::new(1.2,
                            Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), ((1.0 - wave_r / 24.0) * 180.0) as u8)));

                        // --- PARTICLES: Spiral into Black Hole ---
                        if is_active_audio {
                            let count = ((24.0 * weight).round() as usize).max(4);
                            let phi = 0.6180339887f32;
                            for i in 0..count {
                                let seed = i as f32 * phi;
                                let speed = 0.45 + (seed.fract() * 0.25);
                                let u = ((i as f32 / count as f32) + t * speed).fract();
                                let p_col = pal[i % 8];

                                let (pos, r, col) = if u < 0.65 {
                                    // Approach straight line towards Black Hole perimeter
                                    let tau = u / 0.65;
                                    let target_entry = Pos2::new(p_bh.x - 18.0, p_bh.y);
                                    let pt = p_pod_exit + (target_entry - p_pod_exit) * tau;
                                    let wave = (tau * 10.0 - t * 4.0 + seed * 6.28).sin() * 1.2;
                                    (Pos2::new(pt.x, pt.y + wave), 1.5, p_col)
                                } else {
                                    // Spiral into Black Hole singularity (inward spiral)
                                    let tau = (u - 0.65) / 0.35; // 0..1
                                    let spiral_r = 18.0 * (1.0 - tau).powf(1.2);
                                    let spiral_ang = std::f32::consts::PI + tau * 3.5 * std::f32::consts::PI + seed * 2.0;
                                    let pt = Pos2::new(p_bh.x + spiral_r * spiral_ang.cos(), p_bh.y + spiral_r * spiral_ang.sin());
                                    let heated_col = lerp_rgb(p_col, Color32::WHITE, tau);
                                    (pt, 1.4 * (1.0 - 0.5 * tau), heated_col)
                                };
                                painter.circle_filled(pos, r + 0.8, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 50));
                                painter.circle_filled(pos, r, col);
                            }

                            // --- PARTICLES: Spiral out of White Hole -> Gate Vortex ---
                            for i in 0..count {
                                let seed = i as f32 * phi + 0.35;
                                let speed = 0.45 + (seed.fract() * 0.25);
                                let u = ((i as f32 / count as f32) + t * speed + 0.5).fract();

                                let (pos, r, col) = if u < 0.35 {
                                    // Spiral out of White Hole (outward spiral)
                                    let tau = u / 0.35; // 0..1
                                    let spiral_r = 18.0 * tau.powf(0.85);
                                    let spiral_ang = tau * 3.2 * std::f32::consts::PI + seed * 2.0;
                                    let pt = Pos2::new(p_wh.x + spiral_r * spiral_ang.cos(), p_wh.y + spiral_r * spiral_ang.sin());
                                    let cool_col = lerp_rgb(Color32::WHITE, pod_col, tau);
                                    (pt, 1.2 + 0.3 * tau, cool_col)
                                } else {
                                    // Fly from White Hole perimeter towards Fusion Vortex
                                    let tau = (u - 0.35) / 0.65;
                                    let exit_origin = Pos2::new(p_wh.x + 18.0, p_wh.y);
                                    let pt = exit_origin + (p_target - exit_origin) * tau;
                                    let wave = (tau * 10.0 - t * 4.0 + seed * 6.28).sin() * 1.2;
                                    // Color blends with mixed fusion as it reaches vortex
                                    let blended = lerp_rgb(pod_col, col_mixed, tau.powf(1.4));
                                    (Pos2::new(pt.x, pt.y + wave), 1.5, blended)
                                };
                                painter.circle_filled(pos, r + 0.8, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 50));
                                painter.circle_filled(pos, r, col);
                            }
                        }
                    }

                    ui.ctx().request_repaint();
                }
            } else if node.node_type == NodeType::Splitter {
                let active_split = if node.param_val <= 0.001 { 0.5 } else { node.param_val.clamp(0.0, 1.0) };
                let pct_a = 1.0 - active_split;
                let pct_b = active_split;

                let conn_in = self.connections.iter().any(|c| c.to_node == node.id);
                let conn_out_a = self.connections.iter().any(|c| c.from_node == node.id && c.from_port == 0);
                let conn_out_b = self.connections.iter().any(|c| c.from_node == node.id && c.from_port == 1);
                let is_active_audio = self.is_playing && is_flowing && conn_in;

                let instance_idx = self.nodes.iter()
                    .filter(|n| n.node_type == NodeType::Splitter)
                    .position(|n| n.id == node.id)
                    .unwrap_or(0);
                let theme = get_instance_color_theme(instance_idx);
                let col_a = theme.col_a;
                let col_b = theme.col_b;
                let col_hulu = theme.col_primary;
                let pal_a = theme.pal_a;
                let pal_b = theme.pal_b;
                let lerp_rgb = |c1: Color32, c2: Color32, k: f32| -> Color32 {
                    let t = k.clamp(0.0, 1.0);
                    Color32::from_rgb(
                        (c1.r() as f32 * (1.0 - t) + c2.r() as f32 * t).round() as u8,
                        (c1.g() as f32 * (1.0 - t) + c2.g() as f32 * t).round() as u8,
                        (c1.b() as f32 * (1.0 - t) + c2.b() as f32 * t).round() as u8,
                    )
                };

                let t = ui.input(|i| i.time) as f32;
                let pulse = (t * 3.5).sin().abs();

                // Compute Satellite World & Screen Coordinates
                let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                let sat_a_screen = Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x,
                    canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y,
                );
                let sat_b_screen = Pos2::new(
                    canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x,
                    canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y,
                );
                let sat_a_rect = Rect::from_min_size(sat_a_screen, Vec2::new(52.0, 34.0));
                let sat_b_rect = Rect::from_min_size(sat_b_screen, Vec2::new(52.0, 34.0));

                let is_sat_a_hovered = !is_canvas_blocked && raw_pointer_pos.map_or(false, |p| sat_a_rect.contains(p));
                let is_sat_b_hovered = !is_canvas_blocked && raw_pointer_pos.map_or(false, |p| sat_b_rect.contains(p));
                let is_sat_a_dragged = self.dragging_satellite == Some((node.id, 0));
                let is_sat_b_dragged = self.dragging_satellite == Some((node.id, 1));
                let is_gate_hovered = is_hovered;
                let is_gate_selected = self.selected_node == Some(node.id);
                let is_gate_dragged = self.dragging_node == Some(node.id);

                let is_family_active = is_gate_hovered || is_gate_selected || is_gate_dragged || is_sat_a_hovered || is_sat_b_hovered || is_sat_a_dragged || is_sat_b_dragged;

                let emitter_pos = Pos2::new(node_rect.max.x - 15.0, node_rect.min.y + 44.0);

                // =========================================================================
                // 1. GERBANG UTAMA (MODERN QUANTUM EMITTER CONSOLE) - 140 x 78 px
                // =========================================================================
                painter.rect_filled(node_rect.translate(Vec2::new(0.0, 3.0)), 10.0, Color32::from_black_alpha(16));

                if is_family_active {
                    painter.rect_stroke(
                        node_rect.expand(2.0),
                        11.0,
                        Stroke::new(2.0, Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), 70)),
                        egui::StrokeKind::Outside,
                    );
                }

                painter.rect_filled(node_rect, 10.0, Color32::WHITE);

                let header_rect = Rect::from_min_size(node_rect.min, Vec2::new(node_rect.width(), 22.0));
                let header_bg = if is_family_active {
                    Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), 18)
                } else {
                    Color32::from_rgb(248, 250, 252)
                };
                painter.rect_filled(header_rect, 10.0, header_bg);
                painter.rect_filled(Rect::from_min_size(Pos2::new(header_rect.min.x, header_rect.max.y - 6.0), Vec2::new(node_rect.width(), 6.0)), 0.0, header_bg);
                painter.line_segment(
                    [Pos2::new(node_rect.min.x, header_rect.max.y), Pos2::new(node_rect.max.x, header_rect.max.y)],
                    Stroke::new(0.8, Color32::from_rgb(241, 245, 249)),
                );

                let border_col = if is_family_active {
                    col_hulu
                } else if is_active_audio {
                    Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), 200)
                } else {
                    Color32::from_rgb(226, 232, 240)
                };
                painter.rect_stroke(node_rect, 10.0, Stroke::new(if is_family_active { 1.8 } else { 1.2 }, border_col), egui::StrokeKind::Outside);

                // 4 Corner Allen Bolts
                for &(dx, dy) in &[(5.5, 5.5), (node_rect.width() - 5.5, 5.5), (5.5, node_rect.height() - 5.5), (node_rect.width() - 5.5, node_rect.height() - 5.5)] {
                    let bp = Pos2::new(node_rect.min.x + dx, node_rect.min.y + dy);
                    painter.circle_filled(bp, 1.2, Color32::from_rgb(226, 232, 240));
                }

                // Left: Title Badge Pill
                let title_pill = Rect::from_min_size(Pos2::new(node_rect.min.x + 8.0, node_rect.min.y + 4.0), Vec2::new(82.0, 14.0));
                painter.rect_filled(title_pill, 4.0, Color32::from_rgb(241, 245, 249));
                painter.text(
                    Pos2::new(title_pill.min.x + 4.0, title_pill.center().y),
                    egui::Align2::LEFT_CENTER,
                    "🔀 SPLITTER",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(15, 23, 42),
                );
                painter.text(
                    Pos2::new(title_pill.max.x - 4.0, title_pill.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!("CH-0{}", theme.instance_num),
                    egui::FontId::monospace(7.2),
                    Color32::from_rgb(100, 116, 139),
                );

                // Right: Status LED
                let led_pos = Pos2::new(node_rect.max.x - 12.0, node_rect.min.y + 11.0);
                if is_active_audio {
                    let led_pulse = (t * 5.0).sin().abs();
                    let halo_alpha = (60.0 + 90.0 * led_pulse) as u8;
                    painter.circle_stroke(led_pos, 4.2, Stroke::new(1.0, Color32::from_rgba_unmultiplied(34, 197, 94, halo_alpha)));
                    painter.circle_filled(led_pos, 2.2, Color32::from_rgb(34, 197, 94));
                    painter.circle_filled(led_pos, 1.0, Color32::WHITE);
                } else if conn_in {
                    painter.circle_filled(led_pos, 2.0, Color32::from_rgb(52, 211, 153));
                } else {
                    painter.circle_filled(led_pos, 2.0, Color32::from_rgb(203, 213, 225));
                }

                // Input Hulu Pin on Left
                let in_hulu = Pos2::new(node_rect.min.x + 3.0, node_rect.min.y + 44.0);
                painter.circle_stroke(in_hulu, 4.5, Stroke::new(1.2, Color32::from_rgb(203, 213, 225)));
                if conn_in {
                    painter.circle_filled(in_hulu, 3.0, col_hulu);
                    painter.circle_filled(in_hulu, 1.2, Color32::WHITE);
                } else {
                    painter.circle_filled(in_hulu, 2.5, Color32::from_rgb(241, 245, 249));
                }

                // Internal Waveguide Light Beam from Input to Emitter
                let beam_alpha = if is_active_audio { (140.0 + 80.0 * pulse) as u8 } else if conn_in { 60 } else { 20 };
                painter.line_segment(
                    [Pos2::new(in_hulu.x + 5.0, in_hulu.y), emitter_pos],
                    Stroke::new(1.2, Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), beam_alpha)),
                );

                // Right: Quantum Emitter Nozzle & Prism Waveguide
                painter.circle_filled(emitter_pos, 10.0, Color32::from_rgb(15, 23, 42));
                let emit_alpha = if is_active_audio && (conn_out_a || conn_out_b) { (180.0 + 75.0 * pulse) as u8 } else { 90 };
                painter.circle_stroke(
                    emitter_pos,
                    9.5,
                    Stroke::new(1.2, Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), emit_alpha)),
                );

                let spin_spd = if is_active_audio { 4.5 } else { 1.2 };
                let ang = t * spin_spd;
                let r_orb = 4.2;
                let orb_a = Pos2::new(emitter_pos.x + ang.cos() * r_orb, emitter_pos.y + ang.sin() * r_orb);
                let orb_b = Pos2::new(emitter_pos.x - ang.cos() * r_orb, emitter_pos.y - ang.sin() * r_orb);

                painter.circle_filled(orb_a, 2.0, col_a);
                painter.circle_stroke(orb_a, 3.2, Stroke::new(0.8, Color32::from_rgba_unmultiplied(col_a.r(), col_a.g(), col_a.b(), 120)));
                painter.circle_filled(orb_b, 2.0, col_b);
                painter.circle_stroke(orb_b, 3.2, Stroke::new(0.8, Color32::from_rgba_unmultiplied(col_b.r(), col_b.g(), col_b.b(), 120)));
                painter.circle_filled(emitter_pos, 1.8, Color32::WHITE);

                if is_active_audio && (conn_out_a || conn_out_b) {
                    for ring in 0..2 {
                        let ring_r = 5.0 + ((t * 2.5 + ring as f32 * 0.5).fract()) * 16.0;
                        let ring_alpha = ((1.0 - ring_r / 21.0) * 160.0) as u8;
                        painter.circle_stroke(
                            emitter_pos,
                            ring_r,
                            Stroke::new(1.2, Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), ring_alpha)),
                        );
                    }
                }

                // Split Ratio Fader Slider (Precision Studio Fader)
                let track_min_x = node_rect.min.x + 22.0;
                let track_max_x = node_rect.max.x - 36.0;
                let track_y = node_rect.min.y + 44.0;
                let track_w = track_max_x - track_min_x;
                let knob_x = track_min_x + active_split * track_w;
                let cur_mix_col = lerp_rgb(col_a, col_b, active_split);

                let track_rect = Rect::from_min_size(Pos2::new(track_min_x, track_y - 2.5), Vec2::new(track_w, 5.0));
                painter.rect_filled(track_rect, 2.5, Color32::from_rgb(241, 245, 249));
                painter.rect_stroke(track_rect, 2.5, Stroke::new(0.8, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

                let cnx = track_min_x + track_w * 0.5;
                painter.line_segment([Pos2::new(cnx, track_y - 4.5), Pos2::new(cnx, track_y + 4.5)],
                    Stroke::new(1.0, Color32::from_rgb(148, 163, 184)));

                if (knob_x - cnx).abs() > 1.0 {
                    let (fs, fe) = if knob_x > cnx { (cnx, knob_x) } else { (knob_x, cnx) };
                    let fc = if knob_x > cnx { col_b } else { col_a };
                    painter.rect_filled(Rect::from_min_max(Pos2::new(fs, track_y - 2.0), Pos2::new(fe, track_y + 2.0)), 2.0,
                        Color32::from_rgba_unmultiplied(fc.r(), fc.g(), fc.b(), 180));
                }

                let knob_pos2 = Pos2::new(knob_x, track_y);
                painter.circle_filled(knob_pos2 + Vec2::new(0.0, 1.2), 5.2, Color32::from_black_alpha(25));
                painter.circle_filled(knob_pos2, 5.2, Color32::WHITE);
                painter.circle_stroke(knob_pos2, 5.2, Stroke::new(1.6, cur_mix_col));
                painter.circle_filled(knob_pos2, 2.0, cur_mix_col);

                painter.text(Pos2::new(track_min_x, track_y + 11.0), egui::Align2::LEFT_TOP,
                    format!("{:.0}% A", pct_a * 100.0), egui::FontId::monospace(7.8), col_a);
                painter.text(Pos2::new(track_max_x, track_y + 11.0), egui::Align2::RIGHT_TOP,
                    format!("{:.0}% B", pct_b * 100.0), egui::FontId::monospace(7.8), col_b);

                painter.text(Pos2::new(node_rect.center().x, node_rect.max.y - 6.0), egui::Align2::CENTER_BOTTOM,
                    "BRANCH EMITTER", egui::FontId::monospace(7.2), Color32::from_rgb(148, 163, 184));

                // =========================================================================
                // 2. KOTAK MESIN SATELIT (OUTPUT POD A & B) & TRANSMISSION
                // =========================================================================
                for port in 0..2u8 {
                    let sat_rect = if port == 0 { sat_a_rect } else { sat_b_rect };
                    let is_out_connected = if port == 0 { conn_out_a } else { conn_out_b };
                    let pod_col = if port == 0 { col_a } else { col_b };
                    let pal = if port == 0 { &pal_a } else { &pal_b };
                    let is_pod_dragged = if port == 0 { is_sat_a_dragged } else { is_sat_b_dragged };
                    let is_pod_hovered = if port == 0 { is_sat_a_hovered } else { is_sat_b_hovered };
                    let split_pct = if port == 0 { pct_a } else { pct_b };

                    if is_pod_hovered && !is_canvas_blocked {
                        let screen_sat_rect = ts.mul_rect(sat_rect);
                        let sat_tip = ui.interact(screen_sat_rect, ui.scope_id().with(("sat_tip_spl", node.id, port)), egui::Sense::hover());
                        sat_tip.show_tooltip_text(format!(
                            "📡 Satelit Output {} // 🔀 SPLITTER #{}\n• Klik tahan & geser untuk memindah posisi\n• Tarik kabel dari port lingkaran kanan ke modul lain",
                            if port == 0 { "A" } else { "B" }, theme.instance_num
                        ));
                    }

                    let shadow_y = if is_pod_dragged { 4.0 } else if is_pod_hovered { 2.5 } else { 1.8 };
                    painter.rect_filled(sat_rect.translate(Vec2::new(0.0, shadow_y)), 7.0,
                        Color32::from_black_alpha(if is_pod_dragged { 32 } else if is_pod_hovered { 20 } else { 12 }));

                    if is_family_active || is_pod_hovered {
                        painter.rect_stroke(
                            sat_rect.expand(1.5),
                            8.0,
                            Stroke::new(1.5, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), if is_pod_dragged || is_pod_hovered { 180 } else { 80 })),
                            egui::StrokeKind::Outside,
                        );
                    }

                    painter.rect_filled(sat_rect, 7.0, Color32::WHITE);
                    let pod_border_c = if is_pod_dragged || is_pod_hovered {
                        pod_col
                    } else if is_out_connected {
                        Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 180)
                    } else {
                        Color32::from_rgb(226, 232, 240)
                    };
                    painter.rect_stroke(sat_rect, 7.0, Stroke::new(if is_pod_dragged || is_pod_hovered { 1.6 } else { 1.2 }, pod_border_c), egui::StrokeKind::Outside);

                    // Micro LED Status Indicator (Top Right)
                    let sat_led = Pos2::new(sat_rect.max.x - 8.5, sat_rect.min.y + 7.5);
                    if is_active_audio && is_out_connected && split_pct > 0.01 {
                        let l_pulse = (t * 6.0 + port as f32 * 1.57).sin().abs();
                        painter.circle_stroke(sat_led, 3.5, Stroke::new(1.0, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), (100.0 + 120.0 * l_pulse) as u8)));
                        painter.circle_filled(sat_led, 2.0, pod_col);
                        painter.circle_filled(sat_led, 0.9, Color32::WHITE);
                    } else if is_out_connected {
                        painter.circle_filled(sat_led, 1.8, pod_col);
                    } else {
                        painter.circle_filled(sat_led, 1.8, Color32::from_rgb(203, 213, 225));
                    }

                    // Tactile Grip Dots (Top Left)
                    for gx in [-3.5f32, 0.0, 3.5] {
                        painter.line_segment(
                            [Pos2::new(sat_rect.min.x + 10.0 + gx, sat_rect.min.y + 3.0),
                             Pos2::new(sat_rect.min.x + 10.0 + gx, sat_rect.min.y + 5.5)],
                            Stroke::new(0.8, Color32::from_rgb(226, 232, 240)),
                        );
                    }

                    // Modern Badge Chip
                    let badge_rect = Rect::from_center_size(Pos2::new(sat_rect.center().x - 2.0, sat_rect.center().y - 2.0), Vec2::new(34.0, 13.0));
                    painter.rect_filled(badge_rect, 3.0, Color32::from_rgb(248, 250, 252));
                    painter.text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        if port == 0 { "OUT A" } else { "OUT B" },
                        egui::FontId::monospace(7.8),
                        if is_out_connected { pod_col } else { Color32::from_rgb(100, 116, 139) },
                    );

                    painter.text(
                        Pos2::new(sat_rect.center().x - 2.0, sat_rect.max.y - 4.5),
                        egui::Align2::CENTER_BOTTOM,
                        format!("#{}.{}", theme.instance_num, if port == 0 { "A" } else { "B" }),
                        egui::FontId::monospace(6.8),
                        if is_out_connected { Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 180) } else { Color32::from_rgb(148, 163, 184) },
                    );

                    // Right Cable Dock Pin (Chrome Ring + Illuminated Socket)
                    let pin_pos = Pos2::new(sat_rect.max.x - 3.0, sat_rect.center().y);
                    painter.circle_stroke(pin_pos, 4.5, Stroke::new(1.2, Color32::from_rgb(203, 213, 225)));
                    if is_out_connected {
                        painter.circle_filled(pin_pos, 3.0, pod_col);
                        painter.circle_filled(pin_pos, 1.2, Color32::WHITE);
                    } else {
                        painter.circle_filled(pin_pos, 2.5, Color32::from_rgb(241, 245, 249));
                    }

                    // === LEFT APERTURE: "DOORWAY LIGHT SPILL" (Active ONLY if connected & split_pct > 0) ===
                    let mouth_x = sat_rect.min.x;
                    let mouth_cy = sat_rect.center().y;
                    let has_flow = is_out_connected && is_active_audio && split_pct > 0.01;
                    let base_alpha = if has_flow {
                        ((160.0 + 80.0 * (t * 6.0 + port as f32 * 1.57).sin()) * split_pct).clamp(0.0, 255.0) as u8
                    } else {
                        0u8
                    };

                    if base_alpha > 0 {
                        // Slit line
                        painter.line_segment(
                            [Pos2::new(mouth_x + 1.5, mouth_cy - 7.5), Pos2::new(mouth_x + 1.5, mouth_cy + 7.5)],
                            Stroke::new(2.5, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), base_alpha)));

                        // Doorway light fan fanning LEFTWARD
                        let scale_factor = 0.35 + 0.65 * split_pct;
                        let fan_colors = [base_alpha, (base_alpha as u16 * 60 / 100) as u8,
                            (base_alpha as u16 * 35 / 100) as u8, (base_alpha as u16 * 18 / 100) as u8];
                        let fan_widths = [3.0f32 * scale_factor, 7.0 * scale_factor, 13.0 * scale_factor, 20.0 * scale_factor];
                        let fan_depths = [4.0f32 * scale_factor, 9.0 * scale_factor, 16.0 * scale_factor, 24.0 * scale_factor];
                        for fi in 0..4 {
                            let fa = fan_colors[fi];
                            let hw = fan_widths[fi];
                            let depth = fan_depths[fi];
                            let tri = vec![
                                Pos2::new(mouth_x, mouth_cy - 2.0),
                                Pos2::new(mouth_x, mouth_cy + 2.0),
                                Pos2::new(mouth_x - depth, mouth_cy + hw),
                                Pos2::new(mouth_x - depth, mouth_cy - hw),
                            ];
                            painter.add(egui::Shape::convex_polygon(
                                tri,
                                Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), fa),
                                Stroke::NONE,
                            ));
                        }
                        painter.circle_filled(Pos2::new(mouth_x + 1.0, mouth_cy), 2.0, Color32::WHITE);
                    }

                    // === TRANSMISSION DYNAMICS (Emitter -> Pod) ===
                    // ONLY emit particles if this satellite out is connected!
                    if is_out_connected && split_pct > 0.01 {
                        let p_emitter = emitter_pos;
                        let p_pod_entry = Pos2::new(mouth_x - 2.0, mouth_cy);
                        let raw_dx = p_pod_entry.x - p_emitter.x;
                        let raw_dy = (p_pod_entry.y - p_emitter.y).abs();
                        let dist = p_emitter.distance(p_pod_entry);
                        let is_direct = raw_dx > 10.0 && raw_dy <= raw_dx * 2.5 && dist <= 260.0;

                        let bezier_eval_sp = |p0: Pos2, p3: Pos2, u: f32| -> Pos2 {
                            let cp1 = Pos2::new(p0.x + raw_dx * 0.4, p0.y);
                            let cp2 = Pos2::new(p3.x - raw_dx * 0.4, p3.y);
                            let v = 1.0 - u;
                            Pos2::new(
                                v*v*v*p0.x + 3.0*v*v*u*cp1.x + 3.0*v*u*u*cp2.x + u*u*u*p3.x,
                                v*v*v*p0.y + 3.0*v*v*u*cp1.y + 3.0*v*u*u*cp2.y + u*u*u*p3.y,
                            )
                        };

                        if is_direct {
                            if is_active_audio {
                                let count = ((36.0 * split_pct).round() as usize).max(4);
                                let phi = 0.6180339887f32;
                                for i in 0..count {
                                    let seed = i as f32 * phi;
                                    let speed = 0.42 + (seed.fract() * 0.28);
                                    let u = ((i as f32 / count as f32) + t * speed).fract();
                                    let u_prev = (u - 0.045).max(0.0);

                                    let spread = ((i as f32 * 1.91).sin()) * (2.0 + 4.5 * split_pct) * (0.3 + 0.7 * (u * (1.0 - u) * 4.0));
                                    let wave = (u * 11.0 - t * 4.8 + seed * 6.28).sin() * 1.4;

                                    let bp = bezier_eval_sp(p_emitter, p_pod_entry, u);
                                    let bp_prv = bezier_eval_sp(p_emitter, p_pod_entry, u_prev);

                                    let pos = Pos2::new(bp.x, bp.y + spread + wave);
                                    let prv = Pos2::new(bp_prv.x, bp_prv.y + spread + wave);

                                    // Particle transitions from Hulu color to branch color
                                    let p_col_base = pal[i % 8];
                                    let p_col = lerp_rgb(col_hulu, p_col_base, (u * 1.4).clamp(0.0, 1.0));
                                    let r = (1.1 + (seed * 2.9).fract() * 1.2) * (0.7 + 0.5 * split_pct);

                                    painter.line_segment([prv, pos],
                                        Stroke::new(r * 0.9, Color32::from_rgba_unmultiplied(p_col.r(), p_col.g(), p_col.b(), 90)));
                                    painter.circle_filled(pos, r + 1.1, Color32::from_rgba_unmultiplied(p_col.r(), p_col.g(), p_col.b(), 35));
                                    painter.circle_filled(pos, r, p_col);
                                }
                            } else {
                                let idle_count = ((6.0 * split_pct).round() as usize).max(2);
                                for i in 0..idle_count {
                                    let seed = i as f32 * 0.618;
                                    let u = ((i as f32 / idle_count as f32) + t * 0.14).fract();
                                    let bp = bezier_eval_sp(p_emitter, p_pod_entry, u);
                                    let wave = (u * 8.0 - t * 2.0 + seed * 6.28).sin() * 1.5;
                                    let p_col = pal[i % 8];
                                    painter.circle_filled(Pos2::new(bp.x, bp.y + wave), 1.5,
                                        Color32::from_rgba_unmultiplied(p_col.r(), p_col.g(), p_col.b(), 90));
                                }
                            }
                        } else {
                            // =============================================================
                            // WARP: BLACK HOLE (In-Portal at Emitter) -> WHITE HOLE (at Pod)
                            // Spawns farther away from mouths (~65px)
                            // =============================================================
                            let p_bh = Pos2::new(p_emitter.x + 65.0, p_emitter.y + if port == 0 { -10.0 } else { 10.0 });
                            let p_wh = Pos2::new(p_pod_entry.x - 65.0, p_pod_entry.y);

                            // Subspace Einstein-Rosen bridge
                            let link_alpha = if is_active_audio { 60 } else { 25 };
                            for step in 0..10 {
                                let u = step as f32 / 9.0;
                                let lp = p_bh + (p_wh - p_bh) * u;
                                let wave_off = (u * 6.0 + t * 3.0).sin() * 1.5;
                                painter.circle_filled(Pos2::new(lp.x, lp.y + wave_off), 1.1,
                                    Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), link_alpha));
                            }

                            // --- 1. BLACK HOLE (at Emitter side) ---
                            painter.circle_filled(p_bh, 7.5, Color32::from_rgb(4, 6, 14));
                            painter.circle_stroke(p_bh, 8.0, Stroke::new(1.8, Color32::from_rgb(255, 255, 255)));
                            painter.circle_stroke(p_bh, 12.0 + 1.5 * pulse, Stroke::new(2.2,
                                Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), 180)));
                            painter.circle_stroke(p_bh, 16.0 + 2.5 * pulse, Stroke::new(1.4,
                                Color32::from_rgba_unmultiplied(col_hulu.r(), col_hulu.g(), col_hulu.b(), 90)));
                            for k in 0..4 {
                                let ang = t * 8.0 + k as f32 * 1.57;
                                let r_orb = 11.5 + 2.0 * (k as f32 * 0.7).sin();
                                painter.circle_filled(Pos2::new(p_bh.x + r_orb * ang.cos(), p_bh.y + r_orb * ang.sin()), 1.4, Color32::WHITE);
                            }

                            // --- 2. WHITE HOLE (at Pod side) ---
                            painter.circle_filled(p_wh, 5.0, Color32::WHITE);
                            painter.circle_stroke(p_wh, 7.5 + pulse * 1.5, Stroke::new(2.2,
                                Color32::from_rgba_unmultiplied(255, 255, 255, 220)));
                            painter.circle_filled(p_wh, 14.0, Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), 40));
                            let wave_r = 6.0 + ((t * 2.8).fract()) * 18.0;
                            painter.circle_stroke(p_wh, wave_r, Stroke::new(1.2,
                                Color32::from_rgba_unmultiplied(pod_col.r(), pod_col.g(), pod_col.b(), ((1.0 - wave_r / 24.0) * 180.0) as u8)));

                            // --- PARTICLES: Emitter -> Spiral into Black Hole ---
                            if is_active_audio {
                                let count = ((24.0 * split_pct).round() as usize).max(4);
                                let phi = 0.6180339887f32;
                                for i in 0..count {
                                    let seed = i as f32 * phi;
                                    let speed = 0.45 + (seed.fract() * 0.25);
                                    let u = ((i as f32 / count as f32) + t * speed).fract();

                                    let (pos, r, col) = if u < 0.65 {
                                        let tau = u / 0.65;
                                        let target_entry = Pos2::new(p_bh.x - 18.0, p_bh.y);
                                        let pt = p_emitter + (target_entry - p_emitter) * tau;
                                        let wave = (tau * 10.0 - t * 4.0 + seed * 6.28).sin() * 1.2;
                                        (Pos2::new(pt.x, pt.y + wave), 1.5, col_hulu)
                                    } else {
                                        let tau = (u - 0.65) / 0.35;
                                        let spiral_r = 18.0 * (1.0 - tau).powf(1.2);
                                        let spiral_ang = std::f32::consts::PI + tau * 3.5 * std::f32::consts::PI + seed * 2.0;
                                        let pt = Pos2::new(p_bh.x + spiral_r * spiral_ang.cos(), p_bh.y + spiral_r * spiral_ang.sin());
                                        let heated_col = lerp_rgb(col_hulu, Color32::WHITE, tau);
                                        (pt, 1.4 * (1.0 - 0.5 * tau), heated_col)
                                    };
                                    painter.circle_filled(pos, r + 0.8, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 50));
                                    painter.circle_filled(pos, r, col);
                                }

                                // --- PARTICLES: Spiral out of White Hole -> Satellite Pod ---
                                for i in 0..count {
                                    let seed = i as f32 * phi + 0.35;
                                    let speed = 0.45 + (seed.fract() * 0.25);
                                    let u = ((i as f32 / count as f32) + t * speed + 0.5).fract();

                                    let (pos, r, col) = if u < 0.35 {
                                        let tau = u / 0.35;
                                        let spiral_r = 18.0 * tau.powf(0.85);
                                        let spiral_ang = tau * 3.2 * std::f32::consts::PI + seed * 2.0;
                                        let pt = Pos2::new(p_wh.x + spiral_r * spiral_ang.cos(), p_wh.y + spiral_r * spiral_ang.sin());
                                        let cool_col = lerp_rgb(Color32::WHITE, pod_col, tau);
                                        (pt, 1.2 + 0.3 * tau, cool_col)
                                    } else {
                                        let tau = (u - 0.35) / 0.65;
                                        let exit_origin = Pos2::new(p_wh.x + 18.0, p_wh.y);
                                        let pt = exit_origin + (p_pod_entry - exit_origin) * tau;
                                        let wave = (tau * 10.0 - t * 4.0 + seed * 6.28).sin() * 1.2;
                                        (Pos2::new(pt.x, pt.y + wave), 1.5, pod_col)
                                    };
                                    painter.circle_filled(pos, r + 0.8, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 50));
                                    painter.circle_filled(pos, r, col);
                                }
                            }
                        }
                    }

                    ui.ctx().request_repaint();
                }
            } else if node.node_type == NodeType::VolumeBooster {
                let gain = (node.param_val).clamp(0.0, 24.0);
                let active_gain = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(gain);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 44.0;
                    let badge_rect = Rect::from_center_size(Pos2::new(node_rect.min.x + 24.0, cy), Vec2::new(26.0, 22.0));
                    painter.rect_filled(badge_rect, 4.0, Color32::from_rgb(30, 27, 75));
                    painter.text(badge_rect.center(), egui::Align2::CENTER_CENTER, "⚡", egui::FontId::proportional(12.5), Color32::from_rgb(250, 204, 21));

                    let track_min_x = node_rect.min.x + 50.0;
                    let track_w = (node_width - 76.0).max(10.0);
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
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.0),
                        thrust_color,
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::AnalogPreamp {
                let gain = (node.param_val).clamp(-24.0, 12.0);
                let active_gain = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(gain);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 44.0;
                    let dial_center = Pos2::new(node_rect.min.x + 28.0, cy);
                    let dial_radius = 13.0;
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

                    let track_min_x = node_rect.min.x + 50.0;
                    let track_w = (node_width - 76.0).max(10.0);
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
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(194, 65, 12),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::DynamicLoudness {
                let depth = (node.param_val).clamp(0.0, 100.0);
                let active_depth = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(depth);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 44.0;
                    let moon_pos = Pos2::new(node_rect.min.x + 24.0, cy);
                    painter.text(moon_pos, egui::Align2::CENTER_CENTER, "🌙", egui::FontId::proportional(13.5), Color32::from_rgb(245, 158, 11));

                    let crt_w = (node_width - 56.0).max(10.0);
                    let crt_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 38.0, cy - 10.0), Vec2::new(crt_w, 20.0));
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
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(217, 119, 6),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::SteppedAttenuator {
                let db = (node.param_val).clamp(-60.0, 0.0);
                let active_db = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(db);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 44.0;
                    let dial_center = Pos2::new(node_rect.min.x + 26.0, cy);
                    let dial_r = 11.5;
                    painter.circle_filled(dial_center, dial_r, Color32::from_rgb(241, 245, 249));
                    painter.circle_stroke(dial_center, dial_r, Stroke::new(1.2, Color32::from_rgb(148, 163, 184)));
                    for d in 0..8 {
                        let ang = d as f32 * (std::f32::consts::PI * 2.0 / 8.0);
                        let dot_p = Pos2::new(dial_center.x + ang.cos() * (dial_r - 2.5), dial_center.y + ang.sin() * (dial_r - 2.5));
                        painter.circle_filled(dot_p, 1.0, Color32::from_rgb(100, 116, 139));
                    }

                    let nixie_w = 56.0;
                    let nixie_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 46.0, cy - 10.0), Vec2::new(nixie_w, 20.0));
                    painter.rect_filled(nixie_rect, 3.0, Color32::from_rgb(15, 23, 42));
                    painter.rect_stroke(nixie_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(6, 182, 212)), egui::StrokeKind::Outside);
                    painter.text(nixie_rect.center(), egui::Align2::CENTER_CENTER, format!("{:.0} dB", active_db), egui::FontId::monospace(11.0), Color32::from_rgb(34, 211, 238));

                    let is_dim = (active_db - (-20.0)).abs() < 0.5;
                    let dim_rect = Rect::from_min_size(Pos2::new(node_rect.max.x - 48.0, cy - 10.0), Vec2::new(36.0, 20.0));
                    let (dim_bg, dim_fg) = if is_dim {
                        (Color32::from_rgb(6, 182, 212), Color32::WHITE)
                    } else {
                        (Color32::from_rgb(241, 245, 249), Color32::from_rgb(100, 116, 139))
                    };
                    painter.rect_filled(dim_rect, 3.0, dim_bg);
                    painter.rect_stroke(dim_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                    painter.text(dim_rect.center(), egui::Align2::CENTER_CENTER, "DIM", egui::FontId::proportional(10.0), dim_fg);

                    let step_num = (((active_db - (-60.0)) / 60.0) * 63.0).round() as i32 + 1;
                    let readout = format!("Step {}/64 • Resistor Ladder", step_num);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(14, 116, 144),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::OpticalLeveler {
                let peak_red = (node.param_val).clamp(0.0, 100.0);
                let active_red = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(peak_red);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 44.0;
                    let bulb_center = Pos2::new(node_rect.min.x + 26.0, cy);
                    let norm = active_red / 100.0;
                    let glow_alpha = (100.0 + norm * 155.0) as u8;
                    painter.circle_filled(bulb_center, 8.5, Color32::from_rgba_premultiplied(234, 88, 12, glow_alpha));
                    painter.circle_filled(bulb_center, 5.0, Color32::from_rgba_premultiplied(254, 215, 170, 220));
                    painter.circle_stroke(bulb_center, 8.5, Stroke::new(1.0, Color32::from_rgb(194, 65, 12)));

                    let vu_w = (node_width - 56.0).max(10.0);
                    let vu_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 42.0, cy - 10.0), Vec2::new(vu_w, 20.0));
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
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(180, 83, 9),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::StereoBalance {
                let bal = (node.param_val).clamp(-100.0, 100.0);
                let active_bal = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(bal);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 44.0;
                    let track_min_x = node_rect.min.x + 26.0;
                    let track_w = (node_width - 52.0).max(10.0);
                    let norm = ((active_bal - (-100.0)) / 200.0).clamp(0.0, 1.0);
                    let thumb_x = track_min_x + track_w * norm;

                    let l_col = if active_bal < -5.0 { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(148, 163, 184) };
                    painter.text(Pos2::new(track_min_x - 6.0, cy), egui::Align2::RIGHT_CENTER, "L", egui::FontId::proportional(11.0), l_col);

                    let r_col = if active_bal > 5.0 { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(148, 163, 184) };
                    painter.text(Pos2::new(track_min_x + track_w + 6.0, cy), egui::Align2::LEFT_CENTER, "R", egui::FontId::proportional(11.0), r_col);

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
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(79, 70, 229),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::TubeWarmer {
                let warmth = (node.param_val).clamp(0.0, 100.0);
                let active_warmth = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(warmth);
                let norm = (active_warmth / 100.0).clamp(0.0, 1.0);
                let val_int = active_warmth.round().clamp(0.0, 100.0) as u32;

                // 3 SLENDER NIXIE TUBES (Authentic desktop Nixie clock style with glowing cathode digits)
                let tube_w = 20.0;
                let tube_h = 49.0;
                let tube_spacing = 3.0;
                let tubes_start_x = node_rect.min.x + 8.0;
                let tube_top_y = node_rect.min.y + 21.0;
                let tube_bottom_y = tube_top_y + tube_h;

                // Slender PCB / chassis mounting plate beneath the 3 tubes
                let pcb_rect = Rect::from_min_size(Pos2::new(tubes_start_x - 1.5, tube_bottom_y - 2.5), Vec2::new(69.0, 5.0));
                painter.rect_filled(pcb_rect, 1.5, Color32::from_rgb(18, 22, 32));
                painter.rect_stroke(pcb_rect, 1.5, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Outside);

                // Digit extraction for the 3 tubes: Hundreds, Tens, Units
                let digits = [(val_int / 100) % 10, (val_int / 10) % 10, val_int % 10];

                for idx in 0..3 {
                    let tube_x = tubes_start_x + idx as f32 * (tube_w + tube_spacing);
                    let glass_rect = Rect::from_min_size(Pos2::new(tube_x, tube_top_y + 2.0), Vec2::new(tube_w, tube_h - 5.5));
                    let digit_char = char::from_digit(digits[idx], 10).unwrap_or('0');
                    let ch_str = digit_char.to_string();
                    let digit_pos = Pos2::new(glass_rect.center().x, glass_rect.center().y - 1.0);
                    let font_id = egui::FontId::monospace(16.5);

                    if !is_bypassed_effect {
                        // 1. Ceramic Socket & Gold Pin Leads
                        let socket_rect = Rect::from_min_size(Pos2::new(tube_x + 2.0, tube_bottom_y - 4.5), Vec2::new(tube_w - 4.0, 4.5));
                        painter.rect_filled(socket_rect, 1.0, Color32::from_rgb(24, 28, 42));
                        painter.rect_stroke(socket_rect, 1.0, Stroke::new(0.8, Color32::from_rgb(15, 18, 28)), egui::StrokeKind::Outside);
                        painter.line_segment([Pos2::new(tube_x + 5.0, tube_bottom_y), Pos2::new(tube_x + 5.0, tube_bottom_y + 2.5)], Stroke::new(1.0, Color32::from_rgb(217, 119, 6)));
                        painter.line_segment([Pos2::new(tube_x + tube_w - 5.0, tube_bottom_y), Pos2::new(tube_x + tube_w - 5.0, tube_bottom_y + 2.5)], Stroke::new(1.0, Color32::from_rgb(217, 119, 6)));

                        // 2. Royal Blue LED Base Underglow (distinctive desktop Nixie clock feature)
                        let led_pos = Pos2::new(tube_x + tube_w * 0.5, tube_bottom_y - 2.5);
                        painter.circle_filled(led_pos, 5.5, Color32::from_rgba_premultiplied(37, 99, 235, 110));
                        painter.circle_filled(led_pos, 2.0, Color32::from_rgb(96, 165, 250));
                        let wash_rect = Rect::from_min_size(Pos2::new(tube_x + 1.5, tube_bottom_y - 12.0), Vec2::new(tube_w - 3.0, 9.0));
                        painter.rect_filled(wash_rect, 2.0, Color32::from_rgba_premultiplied(30, 64, 175, 45));

                        // 3. Deep Dark Glass Interior (Pure dark background, zero orange blob!)
                        painter.rect_filled(glass_rect, 7.0, Color32::from_rgb(13, 14, 22));

                        // 4. Glass Evacuation Tip / Nipple at top
                        let tip_x = glass_rect.center().x;
                        painter.line_segment([Pos2::new(tip_x, tube_top_y + 2.5), Pos2::new(tip_x, tube_top_y - 1.5)], Stroke::new(1.4, Color32::from_rgba_premultiplied(203, 213, 225, 140)));
                        painter.circle_filled(Pos2::new(tip_x, tube_top_y - 1.5), 1.0, Color32::from_rgba_premultiplied(226, 232, 240, 160));

                        // 5. Fine Anode Honeycomb Wire Mesh Grid & Mica Spacers
                        let mesh_col = Color32::from_rgba_premultiplied(100, 116, 139, 24);
                        for y_step in 1..6 {
                            let my = glass_rect.min.y + y_step as f32 * 6.5;
                            painter.line_segment([Pos2::new(glass_rect.min.x + 2.5, my), Pos2::new(glass_rect.max.x - 2.5, my)], Stroke::new(0.6, mesh_col));
                        }
                        for x_step in 1..3 {
                            let mx = glass_rect.min.x + x_step as f32 * 6.6;
                            painter.line_segment([Pos2::new(mx, glass_rect.min.y + 6.0), Pos2::new(mx, glass_rect.max.y - 4.0)], Stroke::new(0.6, mesh_col));
                        }
                        painter.line_segment(
                            [Pos2::new(glass_rect.min.x + 3.0, glass_rect.min.y + 6.0), Pos2::new(glass_rect.max.x - 3.0, glass_rect.min.y + 6.0)],
                            Stroke::new(0.8, Color32::from_rgb(45, 52, 68)),
                        );
                        painter.line_segment(
                            [Pos2::new(glass_rect.min.x + 3.0, glass_rect.max.y - 4.0), Pos2::new(glass_rect.max.x - 3.0, glass_rect.max.y - 4.0)],
                            Stroke::new(0.8, Color32::from_rgb(45, 52, 68)),
                        );

                        // 6. Micro-Flicker / Plasma Ionization Animation
                        let flicker = if self.is_playing && is_flowing {
                            let t = ui.input(|i| i.time) + idx as f64 * 0.45;
                            ui.ctx().request_repaint();
                            0.94 + 0.06 * (((t * 22.0).sin().abs() * 0.6) + ((t * 8.5).cos().abs() * 0.4)) as f32
                        } else {
                            1.0
                        };
                        let glow_alpha = ((70.0 + norm * 185.0) * flicker).clamp(50.0, 255.0) as u8;
                        let core_alpha = ((90.0 + norm * 165.0) * flicker).clamp(70.0, 255.0) as u8;

                        // 7. Stacked Unlit Cathode Wire Outline (dim ghost digit '8' for authentic depth)
                        painter.text(digit_pos, egui::Align2::CENTER_CENTER, "8", font_id.clone(), Color32::from_rgba_premultiplied(45, 52, 70, 40));

                        // 8. Sharp Glowing Cathode Numerals (NO background blob, razor-sharp filament text)
                        // A. Tight neon sheath halo
                        let halo_col = Color32::from_rgba_premultiplied(234, 88, 12, (glow_alpha as f32 * 0.35) as u8);
                        painter.text(digit_pos + Vec2::new(-0.7, 0.0), egui::Align2::CENTER_CENTER, &ch_str, font_id.clone(), halo_col);
                        painter.text(digit_pos + Vec2::new(0.7, 0.0), egui::Align2::CENTER_CENTER, &ch_str, font_id.clone(), halo_col);
                        painter.text(digit_pos + Vec2::new(0.0, -0.7), egui::Align2::CENTER_CENTER, &ch_str, font_id.clone(), halo_col);
                        painter.text(digit_pos + Vec2::new(0.0, 0.7), egui::Align2::CENTER_CENTER, &ch_str, font_id.clone(), halo_col);

                        // B. Vivid Neon Orange cathode body
                        painter.text(
                            digit_pos,
                            egui::Align2::CENTER_CENTER,
                            &ch_str,
                            font_id.clone(),
                            Color32::from_rgba_premultiplied(255, 145, 25, glow_alpha),
                        );

                        // C. Incandescent Hot White-Amber core filament
                        painter.text(
                            digit_pos,
                            egui::Align2::CENTER_CENTER,
                            &ch_str,
                            egui::FontId::monospace(15.5),
                            Color32::from_rgba_premultiplied(255, 248, 225, (core_alpha as f32 * 0.95) as u8),
                        );

                        // 9. Glass Specular Highlights
                        painter.line_segment(
                            [Pos2::new(glass_rect.min.x + 2.2, glass_rect.min.y + 6.0), Pos2::new(glass_rect.min.x + 2.2, glass_rect.max.y - 6.0)],
                            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 50)),
                        );
                        painter.line_segment(
                            [Pos2::new(glass_rect.min.x + 3.5, glass_rect.min.y + 3.0), Pos2::new(glass_rect.center().x - 1.5, glass_rect.min.y + 1.5)],
                            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 75)),
                        );
                        painter.rect_stroke(glass_rect, 7.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(148, 163, 184, 75)), egui::StrokeKind::Outside);
                    } else {
                        // Extinguished / Bypassed State for Tube idx
                        let socket_rect = Rect::from_min_size(Pos2::new(tube_x + 2.0, tube_bottom_y - 4.5), Vec2::new(tube_w - 4.0, 4.5));
                        painter.rect_filled(socket_rect, 1.0, Color32::from_rgb(30, 36, 48));
                        painter.rect_filled(glass_rect, 7.0, Color32::from_rgb(22, 25, 34));

                        let tip_x = glass_rect.center().x;
                        painter.line_segment([Pos2::new(tip_x, tube_top_y + 2.5), Pos2::new(tip_x, tube_top_y - 1.5)], Stroke::new(1.2, Color32::from_rgba_premultiplied(100, 116, 139, 80)));

                        // Cold dormant cathode wire
                        painter.text(digit_pos, egui::Align2::CENTER_CENTER, &ch_str, font_id.clone(), Color32::from_rgb(100, 116, 139));
                        painter.rect_stroke(glass_rect, 7.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(100, 116, 139, 50)), egui::StrokeKind::Outside);
                    }
                }

                // Right Controls: Subtitle, Slider & Readout
                let track_min_x = node_rect.min.x + 84.0;
                let track_w = (node_rect.width() - 94.0).max(10.0);

                if !is_bypassed_effect {
                    let slider_y = node_rect.min.y + 44.0;
                    let thumb_x = track_min_x + track_w * norm;

                    painter.text(
                        Pos2::new(track_min_x, node_rect.min.y + 26.0),
                        egui::Align2::LEFT_CENTER,
                        "TRIODE 12AX7",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(180, 83, 9),
                    );

                    let track_rect = Rect::from_min_size(Pos2::new(track_min_x, slider_y - 2.5), Vec2::new(track_w, 5.0));
                    painter.rect_filled(track_rect, 2.5, Color32::from_rgb(254, 243, 199));
                    let fill_w = (thumb_x - track_min_x).max(0.0);
                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, slider_y - 2.5), Vec2::new(fill_w, 5.0)), 2.5, Color32::from_rgb(249, 115, 22));
                    painter.circle_filled(Pos2::new(thumb_x, slider_y), 5.5, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, slider_y), 5.5, Stroke::new(1.8, Color32::from_rgb(234, 88, 12)));

                    let readout = format!("Warmth {:.0}%", active_warmth);
                    painter.text(
                        Pos2::new(track_min_x + track_w * 0.5, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.5),
                        Color32::from_rgb(194, 65, 12),
                    );
                } else {
                    let right_cx = track_min_x + track_w * 0.5;
                    painter.text(
                        Pos2::new(right_cx, node_rect.min.y + 42.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(right_cx, node_rect.min.y + 60.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::StereoWidener {
                let widener = (node.param_val).clamp(0.0, 100.0);
                let active_widener = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(widener);

                if !is_bypassed_effect {
                    let cy = node_rect.min.y + 44.0;
                    let norm = active_widener / 100.0;

                    // 3D Spatial Soundstage Icon / Expansion Visualizer
                    let center_x = node_rect.min.x + 26.0;
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
                    let track_min_x = node_rect.min.x + 46.0;
                    let track_w = (node_width - 68.0).max(10.0);
                    let thumb_x = track_min_x + track_w * norm;

                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(track_w, 5.0)), 2.5, Color32::from_rgb(238, 242, 255));
                    painter.rect_filled(Rect::from_min_size(Pos2::new(track_min_x, cy - 2.5), Vec2::new(thumb_x - track_min_x, 5.0)), 2.5, Color32::from_rgb(99, 102, 241));
                    painter.circle_filled(Pos2::new(thumb_x, cy), 5.5, Color32::WHITE);
                    painter.circle_stroke(Pos2::new(thumb_x, cy), 5.5, Stroke::new(1.8, Color32::from_rgb(79, 70, 229)));

                    let readout = format!("Soundstage +{:.0}% • Haas 3D", active_widener);
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        readout,
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(67, 56, 202),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Bypassed)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "Nonaktif di Bit-Perfect",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::Source {
                // 🎛️ FILE SOURCE: Real-time Song Progress Tracking Tape Spools & Format Badge
                let cy = node_rect.min.y + 44.0;
                let r_l_cx = node_rect.min.x + 30.0;
                let r_r_cx = node_rect.max.x - 30.0;
                let r_rad = 11.5;

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
                let badge_rect = Rect::from_center_size(Pos2::new(node_rect.center().x, cy - 7.0), Vec2::new(84.0, 18.0));
                painter.rect_filled(badge_rect, 3.0, Color32::from_rgb(238, 242, 255));
                painter.rect_stroke(badge_rect, 3.0, Stroke::new(0.8, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                let badge_txt = if !node.sub1.is_empty() { &node.sub1 } else { "FLAC Lossless" };
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_txt,
                    egui::FontId::proportional(10.5),
                    Color32::from_rgb(67, 56, 202),
                );

                let sub2_txt = if !node.sub2.is_empty() { &node.sub2 } else { "Lossless Master" };
                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(100, 116, 139),
                );
            } else if node.node_type == NodeType::Buffer {
                // ⚡ RAM BUFFER: Segmented 8-Block Low-Latency Cache Bank
                let seg_w = 15.0;
                let seg_h = 16.0;
                let seg_gap = 4.0;
                let total_w = 8.0 * seg_w + 7.0 * seg_gap;
                let start_x = node_rect.center().x - total_w * 0.5;
                let cy = node_rect.min.y + 43.0;

                for s in 0..8 {
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
                    Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(14, 116, 144),
                );
            } else if node.node_type == NodeType::Dsp {
                // 🎛️ DSP ENGINE: 9 Mini Dancing Graphic Spectrum Bars
                if !is_bypassed_effect {
                    let bar_count = 9;
                    let bar_w = 11.0;
                    let bar_gap = 4.5;
                    let total_w = bar_count as f32 * bar_w + (bar_count - 1) as f32 * bar_gap;
                    let start_x = node_rect.center().x - total_w * 0.5;
                    let base_y = node_rect.min.y + 53.0;
                    let max_h = 22.0;

                    for b in 0..bar_count {
                        let bx = start_x + b as f32 * (bar_w + bar_gap);
                        let spec_val = self.spectrum_bars[(b * 48 / bar_count).min(47)];
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
                        Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                        egui::Align2::CENTER_CENTER,
                        "64-bit Master DSP Bus",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(99, 102, 241),
                    );
                } else {
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 38.0),
                        egui::Align2::CENTER_CENTER,
                        "DILEWATI (Direct Wire)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                    painter.text(
                        Pos2::new(node_rect.center().x, node_rect.min.y + 56.0),
                        egui::Align2::CENTER_CENTER,
                        "0.0 dB Pure Master",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            } else if node.node_type == NodeType::Driver {
                // 📡 APP RESAMPLER: Real-Time Sinc/PCM Oscilloscope Screen
                let scope_rect = Rect::from_min_size(
                    Pos2::new(node_rect.min.x + 18.0, node_rect.min.y + 28.0),
                    Vec2::new(node_width - 36.0, 25.0),
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
                let pts_count = 36;
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
                        // When idle: Plot Sinc Kernel Impulse Response Curve
                        let x = (frac - 0.5) * 6.28318;
                        let cutoff = self.resampler_settings.profile.cutoff() as f32;
                        let sinc_val = if x.abs() < 1e-4 {
                            1.0
                        } else {
                            (x * cutoff * 2.2).sin() / (x * cutoff * 2.2)
                        };
                        let win = (0.5 * (1.0 + (frac * std::f32::consts::PI * 2.0).cos())).clamp(0.0, 1.0);
                        sc_cy - (sinc_val * win) * (scope_rect.height() * 0.40)
                    };
                    trace_pts.push(Pos2::new(px, py));
                }

                // Phosphor Bloom Glow underneath (softer, wider stroke)
                let glow_col = if is_active_audio {
                    Color32::from_rgba_unmultiplied(52, 211, 153, 50)
                } else {
                    Color32::from_rgba_unmultiplied(56, 189, 248, 35)
                };
                for w in trace_pts.windows(2) {
                    trace_painter.line_segment([w[0], w[1]], Stroke::new(2.8, glow_col));
                }
                // High-intensity sharp beam trace on top
                let beam_color = if is_active_audio {
                    Color32::from_rgb(52, 211, 153)
                } else {
                    Color32::from_rgb(56, 189, 248)
                };
                for w in trace_pts.windows(2) {
                    trace_painter.line_segment([w[0], w[1]], Stroke::new(1.3, beam_color));
                }

                // Micro badge in top-right
                let (badge_txt, badge_col) = if is_bypassed_resampler {
                    ("BYPASSED (1:1)", Color32::from_rgb(148, 163, 184))
                } else if is_active_audio {
                    ("LIVE", Color32::from_rgb(52, 211, 153))
                } else {
                    (self.resampler_settings.quality.short_name(), Color32::from_rgb(56, 189, 248))
                };
                painter.text(
                    Pos2::new(scope_rect.max.x - 3.5, scope_rect.min.y + 2.0),
                    egui::Align2::RIGHT_TOP,
                    badge_txt,
                    egui::FontId::proportional(8.0),
                    badge_col,
                );

                // Interactive Badges & Rate Readout below scope
                let b1_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 18.0, node_rect.min.y + 57.0), Vec2::new(52.0, 19.0));
                let b2_rect = Rect::from_min_size(Pos2::new(node_rect.min.x + 74.0, node_rect.min.y + 57.0), Vec2::new(56.0, 19.0));

                let b1_hov = mouse_pos.map_or(false, |p| b1_rect.contains(p));
                let b2_hov = mouse_pos.map_or(false, |p| b2_rect.contains(p));

                // Badge 1: Sinc Quality
                painter.rect_filled(b1_rect, 3.0, if b1_hov { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(15, 23, 42) });
                painter.rect_stroke(b1_rect, 3.0, Stroke::new(1.0, if b1_hov { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(51, 65, 85) }), egui::StrokeKind::Inside);
                painter.text(b1_rect.center(), egui::Align2::CENTER_CENTER, self.resampler_settings.quality.short_name(), egui::FontId::proportional(9.5), Color32::from_rgb(199, 210, 254));

                // Badge 2: Roll-Off Profile
                painter.rect_filled(b2_rect, 3.0, if b2_hov { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(15, 23, 42) });
                painter.rect_stroke(b2_rect, 3.0, Stroke::new(1.0, if b2_hov { Color32::from_rgb(16, 185, 129) } else { Color32::from_rgb(51, 65, 85) }), egui::StrokeKind::Inside);
                painter.text(b2_rect.center(), egui::Align2::CENTER_CENTER, self.resampler_settings.profile.short_name(), egui::FontId::proportional(9.5), Color32::from_rgb(167, 243, 208));

                // Right text: hardware rate (shows "44.1k ──▶ 48.0k" when Hardware Adapted in WASAPI Exclusive Modular FX)
                let rate_text = self.signal_path_report.as_ref().map_or("48.0k".to_string(), |r| {
                    if r.is_hardware_rate_adapted() {
                        format!("{:.1}k➔{:.1}k", r.source_rate as f32 / 1000.0, r.device_rate as f32 / 1000.0)
                    } else {
                        format!("{:.1}k", r.device_rate as f32 / 1000.0)
                    }
                });
                painter.text(
                    Pos2::new(node_rect.max.x - 18.0, node_rect.min.y + 66.5),
                    egui::Align2::RIGHT_CENTER,
                    rate_text,
                    egui::FontId::monospace(9.5),
                    Color32::from_rgb(148, 163, 184),
                );
            } else if node.node_type == NodeType::Endpoint {
                // 🔊 OUTPUT DAC: Precision Studio Stereo Meter (L & R) with Peak-Hold & Clip Warning
                let dac_rect = Rect::from_min_size(
                    Pos2::new(node_rect.min.x + 18.0, node_rect.min.y + 32.0),
                    Vec2::new(node_width - 36.0, 24.0),
                );
                // Sleek obsidian chassis window
                painter.rect_filled(dac_rect, 3.0, Color32::from_rgb(10, 15, 29));
                painter.rect_stroke(dac_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Outside);

                if self.is_playing && is_flowing {
                    ui.ctx().request_repaint();
                }

                // Render Channel L (top row) and Channel R (bottom row)
                let row_h = 8.5;
                let ch_y_l = dac_rect.min.y + 2.0;
                let ch_y_r = dac_rect.min.y + 13.0;

                let meter_x = dac_rect.min.x + 15.0;
                let meter_w = dac_rect.width() - 27.0; // Room for L/R label on left, clip LED on right
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
                        egui::FontId::proportional(9.5),
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
                    Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(217, 119, 6),
                );
            } else if let NodeType::SavedEqPreset(ref name) = node.node_type {
                // 🎛️ EQ PRESET: Mini Dynamic Frequency Curve CRT Screen
                let screen_rect = Rect::from_min_size(
                    Pos2::new(node_rect.min.x + 20.0, node_rect.min.y + 32.0),
                    Vec2::new(node_width - 40.0, 24.0),
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
                    Pos2::new(node_rect.center().x, node_rect.min.y + 68.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(11.0),
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
                    Pos2::new(node_rect.center().x, node_rect.min.y + 36.0),
                    egui::Align2::CENTER_CENTER,
                    &display_sub1,
                    egui::FontId::proportional(12.0),
                    sub1_col,
                );

                painter.text(
                    Pos2::new(node_rect.center().x, node_rect.min.y + 54.0),
                    egui::Align2::CENTER_CENTER,
                    sub2_txt,
                    egui::FontId::proportional(11.0),
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

            let pulse = (self.visualizer_anim_phase * 6.5).sin() * 0.5 + 0.5;

            // Input Pin(s) (Left) - Except Source node
            if node.node_type != NodeType::Source {
                let is_input_receptive = if let Some(from_id) = self.connecting_from_node {
                    if !self.connecting_is_input_drag {
                        node.id != from_id && !self.can_reach_node(node.id, from_id)
                    } else {
                        false
                    }
                } else {
                    false
                };

                let draw_single_pin = |pin_pos: Pos2, is_hover: bool, is_receptive: bool, is_hot: bool, label: Option<&str>| {
                    if is_receptive {
                        let beacon_r = 7.0 + pulse * 3.5;
                        let beacon_alpha = (60.0 + pulse * 130.0) as u8;
                        painter.circle_filled(pin_pos, beacon_r, Color32::from_rgba_premultiplied(14, 165, 233, beacon_alpha));
                    }

                    if is_hot {
                        let hot_pulse = (ui.input(|i| i.time) * 4.0).sin().abs() as f32;
                        painter.circle_stroke(pin_pos, 7.5 + hot_pulse * 1.5, Stroke::new(1.4, Color32::from_rgba_premultiplied(239, 68, 68, 180)));
                    }

                    let is_snapped = is_receptive && is_hover;
                    let (r, col) = if is_snapped {
                        (8.0, Color32::from_rgb(14, 165, 233))
                    } else if is_hover {
                        (6.5, Color32::from_rgb(14, 165, 233))
                    } else if is_receptive {
                        (5.2, Color32::from_rgb(56, 189, 248))
                    } else if is_bypassed_effect {
                        (4.5, Color32::from_rgb(203, 213, 225))
                    } else if is_hot {
                        (4.5, Color32::from_rgb(239, 68, 68))
                    } else if is_flowing {
                        (4.5, theme_color)
                    } else {
                        (4.5, Color32::from_rgb(239, 68, 68))
                    };

                    if is_hover || is_snapped {
                        painter.circle_filled(pin_pos, r + 4.5, Color32::from_rgba_premultiplied(14, 165, 233, 90));
                    }

                    painter.circle_filled(pin_pos, r, col);
                    painter.circle_stroke(pin_pos, r, Stroke::new(1.6, Color32::WHITE));
                    if is_hover || is_snapped {
                        painter.circle_filled(pin_pos, 2.8, Color32::WHITE);
                    }

                    if !is_canvas_blocked && is_hover && is_hot {
                        let pin_tip = ui.interact(Rect::from_center_size(ts.mul_pos(pin_pos), Vec2::splat(18.0)), ui.scope_id().with(("pin_overdriven_tip", node.id, pin_pos.x as i32, pin_pos.y as i32)), egui::Sense::hover());
                        pin_tip.show_tooltip_text("⚠️ PORT INPUT OVERDRIVEN\nSinyal yang masuk ke modul ini melebihi 0 dBFS.");
                    }

                    if let Some(lbl) = label {
                        painter.text(
                            Pos2::new(pin_pos.x + 9.0, pin_pos.y),
                            egui::Align2::LEFT_CENTER,
                            lbl,
                            egui::FontId::proportional(10.5),
                            Color32::from_rgb(100, 116, 139),
                        );
                    }
                };

                if is_input_receptive {
                    ui.ctx().request_repaint();
                }

                if node.node_type == NodeType::Mixer {
                    let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                    let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                    let pin_a = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x, canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y + 17.0);
                    let pin_b = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x, canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y + 17.0);
                    let is_hover_a = mouse_pos.map_or(false, |p| p.distance(pin_a) <= 18.0);
                    let is_hover_b = mouse_pos.map_or(false, |p| p.distance(pin_b) <= 18.0);
                    if is_hover_a || is_hover_b {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    let is_hot_a = static_report.edge_reports.iter().any(|e| e.to_node == node.id as u32 && e.to_port == 0 && e.is_hot) && self.chain_mode == ChainMode::DspActive;
                    let is_hot_b = static_report.edge_reports.iter().any(|e| e.to_node == node.id as u32 && e.to_port == 1 && e.is_hot) && self.chain_mode == ChainMode::DspActive;
                    draw_single_pin(pin_a, is_hover_a, is_input_receptive, is_hot_a, Some("A"));
                    draw_single_pin(pin_b, is_hover_b, is_input_receptive, is_hot_b, Some("B"));
                } else {
                    let in_pin = Pos2::new(node_rect.min.x, node_rect.center().y);
                    let is_hover_in = mouse_pos.map_or(false, |p| p.distance(in_pin) <= 18.0);
                    if is_hover_in {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    let is_hot_in = static_report.edge_reports.iter().any(|e| e.to_node == node.id as u32 && e.to_port == 0 && e.is_hot) && self.chain_mode == ChainMode::DspActive;
                    draw_single_pin(in_pin, is_hover_in, is_input_receptive, is_hot_in, None);
                }
            }

            // Output Pin(s) (Right) - Except Endpoint node
            if node.node_type != NodeType::Endpoint {
                let is_output_receptive = if let Some(from_id) = self.connecting_from_node {
                    if self.connecting_is_input_drag {
                        node.id != from_id && !self.can_reach_node(from_id, node.id)
                    } else {
                        false
                    }
                } else {
                    false
                };

                let draw_single_out_pin = |pin_pos: Pos2, is_hover: bool, is_receptive: bool, label: Option<&str>| {
                    if is_receptive {
                        let beacon_r = 7.0 + pulse * 3.5;
                        let beacon_alpha = (60.0 + pulse * 130.0) as u8;
                        painter.circle_filled(pin_pos, beacon_r, Color32::from_rgba_premultiplied(14, 165, 233, beacon_alpha));
                    }

                    let is_snapped = is_receptive && is_hover;
                    let (r, col) = if is_snapped {
                        (8.0, Color32::from_rgb(14, 165, 233))
                    } else if is_hover {
                        (6.5, Color32::from_rgb(14, 165, 233))
                    } else if is_receptive {
                        (5.2, Color32::from_rgb(56, 189, 248))
                    } else if is_bypassed_effect {
                        (4.5, Color32::from_rgb(203, 213, 225))
                    } else if is_flowing {
                        (4.5, theme_color)
                    } else {
                        (4.5, Color32::from_rgb(239, 68, 68))
                    };

                    if is_hover || is_snapped {
                        painter.circle_filled(pin_pos, r + 4.5, Color32::from_rgba_premultiplied(14, 165, 233, 90));
                    }

                    painter.circle_filled(pin_pos, r, col);
                    painter.circle_stroke(pin_pos, r, Stroke::new(1.6, Color32::WHITE));
                    if is_hover || is_snapped {
                        painter.circle_filled(pin_pos, 2.8, Color32::WHITE);
                    }

                    if let Some(lbl) = label {
                        painter.text(
                            Pos2::new(pin_pos.x - 9.0, pin_pos.y),
                            egui::Align2::RIGHT_CENTER,
                            lbl,
                            egui::FontId::proportional(10.5),
                            Color32::from_rgb(100, 116, 139),
                        );
                    }
                };

                if is_output_receptive {
                    ui.ctx().request_repaint();
                }

                if node.node_type == NodeType::Splitter {
                    let sat_a_world = self.get_satellite_pos(node.id, 0, node.pos, &node.node_type);
                    let sat_b_world = self.get_satellite_pos(node.id, 1, node.pos, &node.node_type);
                    let pin_a = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_a_world.x + 52.0, canvas_rect.min.y + self.patchbay_pan.y + sat_a_world.y + 17.0);
                    let pin_b = Pos2::new(canvas_rect.min.x + self.patchbay_pan.x + sat_b_world.x + 52.0, canvas_rect.min.y + self.patchbay_pan.y + sat_b_world.y + 17.0);
                    let is_hover_a = mouse_pos.map_or(false, |p| p.distance(pin_a) <= 18.0);
                    let is_hover_b = mouse_pos.map_or(false, |p| p.distance(pin_b) <= 18.0);
                    if is_hover_a || is_hover_b {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    draw_single_out_pin(pin_a, is_hover_a, is_output_receptive, Some("A"));
                    draw_single_out_pin(pin_b, is_hover_b, is_output_receptive, Some("B"));
                } else if node.node_type == NodeType::Crossover {
                    let pin_low = Pos2::new(node_rect.max.x, node_rect.center().y - 14.0);
                    let pin_high = Pos2::new(node_rect.max.x, node_rect.center().y + 14.0);
                    let is_hover_low = mouse_pos.map_or(false, |p| p.distance(pin_low) <= 18.0);
                    let is_hover_high = mouse_pos.map_or(false, |p| p.distance(pin_high) <= 18.0);
                    if is_hover_low || is_hover_high {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    draw_single_out_pin(pin_low, is_hover_low, is_output_receptive, Some("LOW"));
                    draw_single_out_pin(pin_high, is_hover_high, is_output_receptive, Some("HIGH"));
                } else {
                    let out_pin = Pos2::new(node_rect.max.x, node_rect.center().y);
                    let is_hover_out = mouse_pos.map_or(false, |p| p.distance(out_pin) <= 18.0);
                    if is_hover_out {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                    }
                    draw_single_out_pin(out_pin, is_hover_out, is_output_receptive, None);
                }
            }
        }

        // Apply real-time node parameter adjustments (e.g. LowPassFilter cutoff or Mixer crossfader)
        if let Some((node_id, new_val)) = param_change {
            self.update_node_parameter(node_id, new_val);
        }

        if let Some(node_id) = auto_fix_node {
            self.apply_auto_fix_for_node(node_id);
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

        // 7. Interactive Bottom Status Pill Message & Colors
        let (pill_bg, pill_stroke, status_icon, status_msg, text_col) = if chain_eval.is_complete {
            if self.is_playing {
                if is_fallback {
                    let src_rate = self.signal_path_report.as_ref().map_or(44.1, |r| r.source_rate as f32 / 1000.0);
                    let dev_rate = self.signal_path_report.as_ref().map_or(48.0, |r| r.device_rate as f32 / 1000.0);
                    let is_ex_rej = self.signal_path_report.as_ref().map_or(false, |r| r.is_exclusive_rejected());
                    let fallback_msg = if is_ex_rej {
                        format!("DAC Hardware tidak mendukung clock asli {:.1} kHz dalam Exclusive mode — dialihkan jujur ke Shared ({:.1} kHz)", src_rate, dev_rate)
                    } else {
                        format!("FALLBACK KE OS RESAMPLED — DAC Hardware tidak mendukung sample rate {:.1} kHz, dialihkan ke {:.1} kHz", src_rate, dev_rate)
                    };
                    (
                        Color32::from_rgba_premultiplied(254, 243, 199, 230),
                        Stroke::new(1.0, Color32::from_rgb(253, 230, 138)),
                        "⚠️",
                        fallback_msg,
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
                                sub_text.to_string(),
                                Color32::from_rgb(6, 95, 70),
                            )
                        }
                        ChainMode::DspActive => {
                            let flowing_fx: Vec<&str> = self.nodes.iter()
                                .filter(|n| n.node_type.is_dsp_or_effect() && chain_eval.flowing_nodes.contains(&n.id))
                                .map(|n| n.title.as_str())
                                .collect();

                            if !flowing_fx.is_empty() {
                                let fx_summary = if flowing_fx.len() <= 3 {
                                    flowing_fx.join(" • ")
                                } else {
                                    format!("{} Modul ({})", flowing_fx.len(), flowing_fx[..2].join(" • ") + " • ...")
                                };
                                if !static_report.is_chain_safe {
                                    let context_suffix = match (&static_report.context_track_title, static_report.context_peak_dbfs) {
                                        (Some(title), Some(peak_db)) => format!(" [Lagu: '{}' • Peak {:.1} dBFS]", title, peak_db),
                                        (Some(title), None) => format!(" [Lagu: '{}']", title),
                                        _ => " [Ref Konservatif 0 dBFS]".to_string(),
                                    };
                                    (
                                        Color32::from_rgba_premultiplied(254, 242, 242, 240),
                                        Stroke::new(1.2, Color32::from_rgb(252, 165, 165)),
                                        "⚠️",
                                        format!("PERINGATAN HEADROOM (+{:.1} dBFS) — {} modul berisiko clipping ({}){}! Kurangi gain atau pasang attenuator.",
                                            static_report.max_headroom_deficit_db,
                                            static_report.culprit_nodes.len(),
                                            fx_summary,
                                            context_suffix
                                        ),
                                        Color32::from_rgb(185, 28, 28),
                                    )
                                } else {
                                    let adapt_suffix = self.signal_path_report.as_ref().filter(|r| r.is_hardware_rate_adapted()).map_or(String::new(), |r| {
                                        format!(" • {:.1}k ──▶ {:.1}k (Hardware Adapted • Exclusive Locked)", r.source_rate as f32 / 1000.0, r.device_rate as f32 / 1000.0)
                                    });
                                    let context_suffix = match (&static_report.context_track_title, static_report.context_peak_dbfs) {
                                        (Some(title), Some(peak_db)) if peak_db < -0.01 => format!("{} [Konteks: '{}' • Headroom +{:.1} dB]", adapt_suffix, title, -peak_db),
                                        (Some(title), Some(peak_db)) => format!("{} [Konteks: '{}' • Peak {:.1} dBFS]", adapt_suffix, title, peak_db),
                                        (Some(title), None) => format!("{} [Konteks: '{}']", adapt_suffix, title),
                                        _ => adapt_suffix,
                                    };
                                    (
                                        Color32::from_rgba_premultiplied(238, 242, 255, 230),
                                        Stroke::new(1.0, Color32::from_rgb(199, 210, 254)),
                                        "🟣",
                                        format!("JALUR MODULAR AKTIF (64-BIT FLOAT • HEADROOM AMAN) — Memproses: {}{}", fx_summary, context_suffix),
                                        Color32::from_rgb(67, 56, 202),
                                    )
                                }
                            } else {
                                (
                                    Color32::from_rgba_premultiplied(236, 253, 245, 230),
                                    Stroke::new(1.0, Color32::from_rgb(167, 243, 208)),
                                    "🟢",
                                    "JALUR AUDIO DIRECT — Alur Passthrough 1:1 Bersih (Belum ada modul efek yang disisipkan)".to_string(),
                                    Color32::from_rgb(6, 95, 70),
                                )
                            }
                        }
                        ChainMode::OsResampled => (
                            Color32::from_rgba_premultiplied(254, 243, 199, 230),
                            Stroke::new(1.0, Color32::from_rgb(253, 230, 138)),
                            "🟡",
                            "JALUR AUDIO AKTIF (OS RESAMPLED) — Resampling ke 48.0 kHz via Rubato Sinc SRC Shared Mode".to_string(),
                            Color32::from_rgb(146, 64, 14),
                        ),
                    }
                }
            } else {
                if self.chain_mode == ChainMode::DspActive && !static_report.is_chain_safe {
                    let context_suffix = match (&static_report.context_track_title, static_report.context_peak_dbfs) {
                        (Some(title), Some(peak_db)) => format!(" pada lagu '{}' (Peak {:.1} dBFS)", title, peak_db),
                        (Some(title), None) => format!(" pada lagu '{}'", title),
                        _ => String::new(),
                    };
                    (
                        Color32::from_rgba_premultiplied(254, 242, 242, 240),
                        Stroke::new(1.2, Color32::from_rgb(252, 165, 165)),
                        "⚠️",
                        format!("PERINGATAN HEADROOM (+{:.1} dBFS) — Terdeteksi potensi clipping pada {} modul{} sebelum playback dimulai.",
                            static_report.max_headroom_deficit_db,
                            static_report.culprit_nodes.len(),
                            context_suffix
                        ),
                        Color32::from_rgb(185, 28, 28),
                    )
                } else {
                    (
                        Color32::from_rgba_premultiplied(241, 245, 249, 230),
                        Stroke::new(1.0, Color32::from_rgb(226, 232, 240)),
                        "⚪",
                        "SIGNAL CHAIN LENGKAP — Rantai audio terhubung sempurna (Tekan Play untuk memutar)".to_string(),
                        Color32::from_rgb(71, 85, 105),
                    )
                }
            }
        } else {
            (
                Color32::from_rgba_premultiplied(254, 242, 242, 240),
                Stroke::new(1.2, Color32::from_rgb(252, 165, 165)),
                "🔴",
                "JALUR SINYAL TERPUTUS — Audio dibisukan otomatis. Klik tanda ✕ merah untuk menyambung!".to_string(),
                Color32::from_rgb(185, 28, 28),
            )
        };

        // Apply 2D TSTransform on all canvas world layer shapes (nodes, cables, satellites, particles)
        ui.ctx().transform_layer_shapes(canvas_layer_id, ts);

        // Dedicated Foreground HUD Painter (drawn on Order::Foreground so HUD is ALWAYS on top of nodes)
        let hud_layer_id = egui::LayerId::new(egui::Order::Foreground, ui.scope_id().with("patchbay_canvas_hud"));
        let mut hud_painter = ui.ctx().layer_painter(hud_layer_id);
        hud_painter.set_clip_rect(canvas_rect);

        // Floating Zoom Indicator Pill (bottom-left)
        let zoom_pct = (self.patchbay_zoom * 100.0).round() as i32;
        let pill_h = 26.0;
        let pill_y = canvas_rect.max.y - 38.0;
        let pill_w = 138.0;
        let pill_rect = Rect::from_min_size(
            Pos2::new(canvas_rect.min.x + 12.0, pill_y),
            Vec2::new(pill_w, pill_h),
        );
        let pill_resp = ui.interact(pill_rect, ui.scope_id().with("patchbay_zoom_pill"), egui::Sense::click());
        let is_pill_hover = pill_resp.hovered();
        hud_painter.rect_filled(
            pill_rect,
            13.0,
            if is_pill_hover { Color32::from_rgb(241, 245, 249) } else { Color32::from_rgba_unmultiplied(255, 255, 255, 235) },
        );
        hud_painter.rect_stroke(
            pill_rect,
            13.0,
            Stroke::new(1.0, Color32::from_rgb(203, 213, 225)),
            egui::StrokeKind::Outside,
        );
        let zoom_text = format!("🔍 {}% • Ctrl+Scroll", zoom_pct);
        hud_painter.text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            zoom_text,
            egui::FontId::proportional(10.5),
            Color32::from_rgb(71, 85, 105),
        );
        if pill_resp.clicked() {
            self.patchbay_zoom = 1.0;
        }
        if is_pill_hover {
            pill_resp.show_tooltip_text("Klik untuk reset zoom ke 100%\nGunakan Ctrl + Scroll untuk Zoom In / Out");
        }

        // Floating Status Banner / Flag (positioned to the right of the zoom pill with 8px gap)
        let banner_start_x = pill_rect.max.x + 8.0;
        let banner_width = (canvas_rect.max.x - 12.0 - banner_start_x).max(120.0);
        let banner_rect = Rect::from_min_size(
            Pos2::new(banner_start_x, pill_y),
            Vec2::new(banner_width, pill_h),
        );

        hud_painter.rect_filled(banner_rect, 13.0, pill_bg);
        hud_painter.rect_stroke(banner_rect, 13.0, pill_stroke, egui::StrokeKind::Outside);

        let show_fix_button = !static_report.is_chain_safe && !static_report.culprit_nodes.is_empty();
        let text_align_pos = if show_fix_button {
            Pos2::new(banner_rect.min.x + 16.0, banner_rect.center().y)
        } else {
            banner_rect.center()
        };
        let text_align = if show_fix_button {
            egui::Align2::LEFT_CENTER
        } else {
            egui::Align2::CENTER_CENTER
        };

        hud_painter.text(
            text_align_pos,
            text_align,
            format!("{} {}", status_icon, status_msg),
            egui::FontId::proportional(12.0),
            text_col,
        );

        if show_fix_button && !is_canvas_blocked {
            let fix_btn_w = 126.0;
            let fix_btn_rect = Rect::from_min_size(
                Pos2::new(banner_rect.max.x - fix_btn_w - 6.0, banner_rect.min.y + 3.0),
                Vec2::new(fix_btn_w, 20.0),
            );
            let fix_resp = ui.interact(fix_btn_rect, ui.scope_id().with("banner_auto_fix_btn"), egui::Sense::click());
            let is_fix_hovered = fix_resp.hovered();
            let btn_bg = if is_fix_hovered {
                Color32::from_rgb(220, 38, 38)
            } else {
                Color32::from_rgb(239, 68, 68)
            };
            hud_painter.rect_filled(fix_btn_rect, 4.0, btn_bg);
            hud_painter.text(
                fix_btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                "🪄 Auto-Fix Semua",
                egui::FontId::proportional(11.5),
                Color32::WHITE,
            );
            if fix_resp.on_hover_text("Otomatis sesuaikan gain modul-modul biang kerok agar berada dalam margin headroom aman").clicked() {
                self.apply_all_headroom_auto_fixes();
            }
        }

        // Add reconnected cable
        if let Some((from_id, to_id)) = connection_to_add {
            if !self.connections.iter().any(|c| c.from_node == from_id && c.to_node == to_id) {
                self.connections.push(CableConnection::new(from_id, to_id));
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
            let has_flowing_dsp = chain_eval.flowing_nodes.iter().any(|&id| {
                self.nodes.iter().find(|n| n.id == id).map_or(false, |n| n.node_type.is_dsp_or_effect())
            });
            if has_flowing_dsp && self.chain_mode != ChainMode::DspActive {
                self.set_chain_mode(ChainMode::DspActive);
            } else {
                self.sync_signal_chain_to_engine();
            }
        }

        // Empty Canvas Context Menu: Right-Click on empty canvas background to quickly add modules
        if !is_hovering_any_node && !is_canvas_blocked {
            response.context_menu(|ui| {
                ui.label(egui::RichText::new("➕ TAMBAH MODUL STUDIO").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                ui.separator();
                if ui.button("🌊 Low-Pass Filter (LPF)").clicked() {
                    self.add_modular_filter(NodeType::LowPassFilter);
                    ui.close();
                }
                if ui.button("🏔️ High-Pass Filter (HPF)").clicked() {
                    self.add_modular_filter(NodeType::HighPassFilter);
                    ui.close();
                }
                if ui.button("🎯 Band-Pass Filter (BPF)").clicked() {
                    self.add_modular_filter(NodeType::BandPassFilter);
                    ui.close();
                }
                if ui.button("✂️ 2-Way Crossover (LR4 24dB)").clicked() {
                    self.add_modular_filter(NodeType::Crossover);
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
                ui.add_enabled_ui(false, |ui| {
                    let _ = ui.button("🔒 Volume Attenuator (Coming Soon)");
                });
                ui.add_enabled_ui(false, |ui| {
                    let _ = ui.button("🔒 Bass Guard Limiter (Coming Soon)");
                });
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
                if ui.button("🔄 Reset Posisi & Zoom Canvas").clicked() {
                    self.reset_patchbay_layout(canvas_rect.width(), canvas_rect.height());
                    ui.close();
                }
            });
        }


        // Render floating beginner visual tutorial module in top-right of canvas
        self.render_patchbay_tutorial_card(ui, canvas_rect);

        // Render filter-specific routing tutorial module
        self.render_filter_tutorial_card(ui, canvas_rect);

        // Render floating headroom guard & clipping detection settings popup
        self.render_clipping_settings_popup(ui, canvas_rect);
    }

    // =========================================================================
    // BESPOKE PASS FILTER REACTORS (LPF, HPF, BPF) - CUSTOM CHASSIS & ANIMATION
    // =========================================================================
    pub fn render_pass_filter_node(
        &self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        node: &crate::models::ModularNode,
        node_rect: Rect,
        node_width: f32,
        param_change: &mut Option<(usize, f32)>,
        switch_mode: &mut Option<crate::models::ChainMode>,
        is_node_clicked: bool,
        is_hovered: bool,
        is_dragging: bool,
        hover_anim: f32,
        is_flowing: bool,
        is_bypassed_effect: bool,
        is_culprit_node: bool,
        is_overdriven_victim: bool,
        theme_color: Color32,
    ) {
        let t = ui.input(|i| i.time) as f32;

        if is_bypassed_effect && is_node_clicked {
            *switch_mode = Some(crate::models::ChainMode::DspActive);
        }

        let lerp_rgb = |c1: Color32, c2: Color32, k: f32| -> Color32 {
            let k = k.clamp(0.0, 1.0);
            Color32::from_rgb(
                (c1.r() as f32 + (c2.r() as f32 - c1.r() as f32) * k) as u8,
                (c1.g() as f32 + (c2.g() as f32 - c1.g() as f32) * k) as u8,
                (c1.b() as f32 + (c2.b() as f32 - c1.b() as f32) * k) as u8,
            )
        };

        match node.node_type {
            NodeType::LowPassFilter => {
                self.render_lpf_hydro_sieve(
                    ui, painter, node, node_rect, node_width, param_change,
                    is_hovered, is_dragging, hover_anim, is_flowing, is_bypassed_effect,
                    is_culprit_node, is_overdriven_victim, theme_color, t, lerp_rgb,
                );
            }
            NodeType::HighPassFilter => {
                self.render_hpf_plasma_prism(
                    ui, painter, node, node_rect, node_width, param_change,
                    is_hovered, is_dragging, hover_anim, is_flowing, is_bypassed_effect,
                    is_culprit_node, is_overdriven_victim, theme_color, t, lerp_rgb,
                );
            }
            NodeType::BandPassFilter => {
                self.render_bpf_cyclotron(
                    ui, painter, node, node_rect, node_width, param_change,
                    is_hovered, is_dragging, hover_anim, is_flowing, is_bypassed_effect,
                    is_culprit_node, is_overdriven_victim, theme_color, t, lerp_rgb,
                );
            }
            _ => {}
        }
    }

    // --- 1. LOW-PASS FILTER: HYDRO-TECTONIC SIEVE ---
    fn render_lpf_hydro_sieve(
        &self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        node: &crate::models::ModularNode,
        node_rect: Rect,
        node_width: f32,
        param_change: &mut Option<(usize, f32)>,
        is_hovered: bool,
        is_dragging: bool,
        _hover_anim: f32,
        is_flowing: bool,
        is_bypassed_effect: bool,
        is_culprit_node: bool,
        is_overdriven_victim: bool,
        _theme_color: Color32,
        t: f32,
        _lerp_rgb: impl Fn(Color32, Color32, f32) -> Color32,
    ) {
        let cutoff = if node.param_val > 50.0 { node.param_val } else { 800.0 };
        let active_cutoff = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(cutoff);

        // 1. Asymmetric Rolloff Polygon Chassis (High on left, slopes down on right)
        let poly = vec![
            Pos2::new(node_rect.min.x, node_rect.min.y + 8.0),
            Pos2::new(node_rect.min.x + 8.0, node_rect.min.y),
            Pos2::new(node_rect.min.x + 104.0, node_rect.min.y),
            Pos2::new(node_rect.min.x + 142.0, node_rect.min.y + 8.0),
            Pos2::new(node_rect.max.x - 16.0, node_rect.min.y + 18.0),
            Pos2::new(node_rect.max.x, node_rect.min.y + 24.0),
            Pos2::new(node_rect.max.x, node_rect.max.y - 8.0),
            Pos2::new(node_rect.max.x - 8.0, node_rect.max.y),
            Pos2::new(node_rect.min.x + 8.0, node_rect.max.y),
            Pos2::new(node_rect.min.x, node_rect.max.y - 8.0),
        ];

        let bg = if is_dragging {
            Color32::from_rgb(241, 245, 249)
        } else if is_bypassed_effect {
            Color32::from_rgba_premultiplied(248, 250, 252, 230)
        } else if !is_flowing {
            Color32::from_rgb(255, 248, 248)
        } else {
            Color32::WHITE
        };

        // Fill chassis body
        painter.add(egui::Shape::convex_polygon(poly.clone(), bg, Stroke::NONE));

        let border_col = if is_culprit_node {
            Color32::from_rgb(239, 68, 68)
        } else if is_overdriven_victim {
            Color32::from_rgb(245, 158, 11)
        } else if is_bypassed_effect {
            Color32::from_rgb(203, 213, 225)
        } else if is_flowing {
            Color32::from_rgb(14, 165, 233)
        } else {
            Color32::from_rgb(239, 68, 68)
        };

        // Perimeter Stroke & Outer Aura
        if is_flowing && self.is_playing && !is_bypassed_effect {
            let aura_alpha = (50.0 + 30.0 * (self.visualizer_anim_phase * 4.0).sin()) as u8;
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(3.0, Color32::from_rgba_unmultiplied(border_col.r(), border_col.g(), border_col.b(), aura_alpha / 2)));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(3.0, Color32::from_rgba_unmultiplied(border_col.r(), border_col.g(), border_col.b(), aura_alpha / 2)));

            // Traveling laser beacons on chassis perimeter
            let mut total_len = 0.0f32;
            for i in 0..poly.len() {
                total_len += poly[i].distance(poly[(i + 1) % poly.len()]);
            }
            let travel_dist = (self.visualizer_anim_phase * 130.0 + (node.id as f32) * 50.0) % total_len;
            let eval_poly = |dist: f32| -> Pos2 {
                let mut acc = 0.0f32;
                let d = (dist % total_len + total_len) % total_len;
                for i in 0..poly.len() {
                    let next_i = (i + 1) % poly.len();
                    let seg_len = poly[i].distance(poly[next_i]);
                    if acc + seg_len >= d {
                        let frac = if seg_len > 0.001 { (d - acc) / seg_len } else { 0.0 };
                        return poly[i].lerp(poly[next_i], frac);
                    }
                    acc += seg_len;
                }
                poly[0]
            };
            let b1 = eval_poly(travel_dist);
            let b2 = eval_poly(travel_dist + total_len * 0.5);
            painter.circle_filled(b1, 3.8, border_col);
            painter.circle_filled(b1, 1.8, Color32::WHITE);
            painter.circle_filled(b2, 3.8, Color32::from_rgb(56, 189, 248));
            painter.circle_filled(b2, 1.8, Color32::WHITE);

            let stroke_w = 1.6 + 0.4 * (self.visualizer_anim_phase * 3.5).sin();
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(stroke_w, border_col));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(stroke_w, border_col));
        } else {
            let stroke_w = if is_hovered { 1.8 } else { 1.2 };
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(stroke_w, border_col));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(stroke_w, border_col));
        }

        // Corner Allen Bolts
        for &(bx, by) in &[(node_rect.min.x + 6.0, node_rect.min.y + 6.0),
                          (node_rect.min.x + 98.0, node_rect.min.y + 6.0),
                          (node_rect.min.x + 6.0, node_rect.max.y - 6.0),
                          (node_rect.max.x - 6.0, node_rect.max.y - 6.0)] {
            painter.circle_filled(Pos2::new(bx, by), 1.2, Color32::from_rgb(203, 213, 225));
        }

        // 2. Top Header Pill: "🌊 LPF SIEVE"
        let title_pill = Rect::from_min_size(Pos2::new(node_rect.min.x + 10.0, node_rect.min.y + 4.0), Vec2::new(88.0, 15.0));
        painter.rect_filled(title_pill, 4.0, Color32::from_rgb(240, 249, 255));
        painter.rect_stroke(title_pill, 4.0, Stroke::new(0.8, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);
        painter.text(
            Pos2::new(title_pill.min.x + 4.0, title_pill.center().y),
            egui::Align2::LEFT_CENTER,
            "🌊 LPF SIEVE",
            egui::FontId::proportional(8.5),
            Color32::from_rgb(14, 116, 144),
        );
        painter.text(
            Pos2::new(title_pill.max.x - 4.0, title_pill.center().y),
            egui::Align2::RIGHT_CENTER,
            format!("#{}", node.id),
            egui::FontId::monospace(7.5),
            Color32::from_rgb(100, 116, 139),
        );

        // Status LED
        let led_pos = Pos2::new(node_rect.min.x + 106.0, node_rect.min.y + 11.0);
        if is_flowing && self.is_playing && !is_bypassed_effect {
            let led_pulse = (t * 5.0).sin().abs();
            painter.circle_stroke(led_pos, 4.2, Stroke::new(1.0, Color32::from_rgba_unmultiplied(14, 165, 233, (60.0 + 90.0 * led_pulse) as u8)));
            painter.circle_filled(led_pos, 2.2, Color32::from_rgb(14, 165, 233));
            painter.circle_filled(led_pos, 1.0, Color32::WHITE);
        } else if is_flowing && !is_bypassed_effect {
            painter.circle_filled(led_pos, 2.0, Color32::from_rgb(56, 189, 248));
        } else {
            painter.circle_filled(led_pos, 2.0, Color32::from_rgb(203, 213, 225));
        }

        // 3. Treble Dissipation Vents in the sloped cutout (Top-Right)
        let v_base_x = node_rect.min.x + 128.0;
        let v_base_y = node_rect.min.y + 5.0;
        for vi in 0..4 {
            let vx = v_base_x + vi as f32 * 13.0;
            let vy = v_base_y + vi as f32 * 4.2;
            painter.line_segment(
                [Pos2::new(vx, vy), Pos2::new(vx + 8.0, vy + 4.5)],
                Stroke::new(1.2, Color32::from_rgb(203, 213, 225)),
            );

            // Animated steam/mist particles escaping when audio is active
            if is_flowing && self.is_playing && !is_bypassed_effect {
                let steam_t = (t * 2.5 + vi as f32 * 0.45).fract();
                let steam_x = vx + 4.0 + (steam_t * 6.28).sin() * 2.0;
                let steam_y = vy - steam_t * 9.0;
                let steam_alpha = ((1.0 - steam_t) * 110.0) as u8;
                painter.circle_filled(Pos2::new(steam_x, steam_y), 1.2 + steam_t * 1.5, Color32::from_rgba_unmultiplied(186, 230, 253, steam_alpha));
            }
        }

        // 4. Hydro-Acoustic Bottom Reservoir & Animated Liquid Waves
        let res_rect = Rect::from_min_max(
            Pos2::new(node_rect.min.x + 2.0, node_rect.min.y + 44.0),
            Pos2::new(node_rect.max.x - 2.0, node_rect.max.y - 2.0),
        );
        let wave_painter = painter.with_clip_rect(res_rect);

        if is_flowing && self.is_playing && !is_bypassed_effect {
            let norm_cutoff = ((active_cutoff / 100.0).log10() / 2.0).clamp(0.0, 1.0);
            let water_depth = 0.55 - norm_cutoff * 0.22;
            let base_y = node_rect.max.y - (node_rect.height() * water_depth);

            // Back wave layer
            let mut back_poly = Vec::new();
            back_poly.push(Pos2::new(node_rect.min.x, node_rect.max.y));
            let steps = 14;
            for s in 0..=steps {
                let fx = node_rect.min.x + (node_rect.width() * (s as f32 / steps as f32));
                let wave_offset = ((fx * 0.08 + t * 3.2).sin()) * 1.8;
                back_poly.push(Pos2::new(fx, base_y + wave_offset));
            }
            back_poly.push(Pos2::new(node_rect.max.x, node_rect.max.y));
            wave_painter.add(egui::Shape::convex_polygon(back_poly, Color32::from_rgba_premultiplied(14, 165, 233, 40), Stroke::NONE));

            // Front wave layer
            let mut front_poly = Vec::new();
            front_poly.push(Pos2::new(node_rect.min.x, node_rect.max.y));
            for s in 0..=steps {
                let fx = node_rect.min.x + (node_rect.width() * (s as f32 / steps as f32));
                let wave_offset = (((fx * 0.09 - t * 4.0) + 1.8).sin()) * 1.6;
                front_poly.push(Pos2::new(fx, base_y + wave_offset + 0.8));
            }
            front_poly.push(Pos2::new(node_rect.max.x, node_rect.max.y));
            wave_painter.add(egui::Shape::convex_polygon(front_poly, Color32::from_rgba_premultiplied(6, 182, 212, 70), Stroke::NONE));

            // Rising bubbles
            for bi in 0..3 {
                let b_seed = bi as f32 * 1.4;
                let b_y = node_rect.max.y - (((t * (16.0 + bi as f32 * 4.0) + b_seed * 10.0)) % (node_rect.height() * 0.40));
                let b_x = node_rect.min.x + 22.0 + bi as f32 * 45.0 + (t * 2.0 + b_seed).sin() * 3.0;
                wave_painter.circle_stroke(Pos2::new(b_x, b_y), 1.5, Stroke::new(0.8, Color32::from_rgba_premultiplied(255, 255, 255, 180)));
            }

            ui.ctx().request_repaint();
        }

        // 5. Sluice Gate (Cutoff Sieve Barrier) & Slider Track
        let slider_track_rect = Rect::from_min_size(
            Pos2::new(node_rect.min.x + 24.0, node_rect.min.y + 36.0),
            Vec2::new((node_width - 48.0).max(10.0), 5.0),
        );
        let fill_ratio = ((active_cutoff / 50.0).ln() / (400.0_f32).ln()).clamp(0.0, 1.0);
        let thumb_x = slider_track_rect.min.x + slider_track_rect.width() * fill_ratio;
        let cy = slider_track_rect.center().y;

        if !is_bypassed_effect {
            // Sieve Barrier Line (Glowing laser divider)
            if is_flowing && self.is_playing {
                painter.line_segment(
                    [Pos2::new(thumb_x, node_rect.min.y + 24.0), Pos2::new(thumb_x, node_rect.max.y - 18.0)],
                    Stroke::new(1.4, Color32::from_rgba_unmultiplied(14, 165, 233, 140)),
                );
                // Whirlpool ripple at water surface
                let wp_r = 3.5 + (t * 4.5).sin().abs() * 2.5;
                painter.circle_stroke(
                    Pos2::new(thumb_x, node_rect.min.y + 46.0),
                    wp_r,
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(56, 189, 248, 160)),
                );
            }

            // Hydraulic Slider Track
            painter.rect_filled(Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(slider_track_rect.width(), 4.0)), 2.0, Color32::from_rgb(224, 242, 254));
            painter.rect_filled(Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(thumb_x - slider_track_rect.min.x, 4.0)), 2.0, Color32::from_rgb(14, 165, 233));

            // Sluice Gate Knob (Hydraulic wheel)
            painter.circle_filled(Pos2::new(thumb_x, cy + 1.0), 5.5, Color32::from_black_alpha(20));
            painter.circle_filled(Pos2::new(thumb_x, cy), 5.2, Color32::WHITE);
            painter.circle_stroke(Pos2::new(thumb_x, cy), 5.2, Stroke::new(1.8, Color32::from_rgb(2, 132, 199)));
            painter.circle_filled(Pos2::new(thumb_x, cy), 2.0, Color32::from_rgb(14, 165, 233));

            // 6. Dual Telemetry Badges at bottom
            let readout = if active_cutoff >= 1000.0 {
                format!("Cutoff {:.1}k", active_cutoff / 1000.0)
            } else {
                format!("Cutoff {:.0}Hz", active_cutoff)
            };

            let slope_str = if node.sub2.contains("48dB") { "48dB" } else if node.sub2.contains("24dB") { "24dB" } else { "12dB" };
            let q_str = if let Some(idx) = node.sub2.find("Q ") {
                node.sub2[idx + 2..].split_whitespace().next().unwrap_or("0.71")
            } else {
                "0.71"
            };

            let badge_y = node_rect.min.y + 63.0;
            let badge_w = (node_width - 34.0) * 0.48;
            let badge_h = 19.0;

            // Left: Cutoff Readout
            let b_left = Rect::from_center_size(Pos2::new(node_rect.center().x - badge_w * 0.52, badge_y), Vec2::new(badge_w, badge_h));
            painter.rect_filled(b_left, 3.5, Color32::from_rgb(224, 242, 254));
            painter.rect_stroke(b_left, 3.5, Stroke::new(1.0, Color32::from_rgb(2, 132, 199)), egui::StrokeKind::Outside);
            painter.text(b_left.center(), egui::Align2::CENTER_CENTER, readout, egui::FontId::proportional(10.0), Color32::from_rgb(2, 132, 199));

            // Right: Slope & Q Readout
            let b_right = Rect::from_center_size(Pos2::new(node_rect.center().x + badge_w * 0.52, badge_y), Vec2::new(badge_w, badge_h));
            painter.rect_filled(b_right, 3.5, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(b_right, 3.5, Stroke::new(0.8, Color32::from_rgb(71, 85, 105)), egui::StrokeKind::Outside);
            painter.text(b_right.center(), egui::Align2::CENTER_CENTER, format!("{} • Q{}", slope_str, q_str), egui::FontId::proportional(9.5), Color32::from_rgb(241, 245, 249));
        } else {
            // Dormant / Bypassed State
            let b_rect = Rect::from_center_size(Pos2::new(node_rect.center().x, node_rect.min.y + 44.0), Vec2::new(154.0, 22.0));
            painter.rect_filled(b_rect, 4.0, Color32::from_rgb(241, 245, 249));
            painter.rect_stroke(b_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
            painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, "DILEWATI (Bit-Perfect)", egui::FontId::proportional(11.0), Color32::from_rgb(100, 116, 139));
            painter.text(Pos2::new(node_rect.center().x, node_rect.min.y + 66.0), egui::Align2::CENTER_CENTER, "Passthrough 1:1 Stream", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        }
    }

    // --- 2. HIGH-PASS FILTER: AERO-PLASMA PRISM ---
    fn render_hpf_plasma_prism(
        &self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        node: &crate::models::ModularNode,
        node_rect: Rect,
        node_width: f32,
        param_change: &mut Option<(usize, f32)>,
        is_hovered: bool,
        is_dragging: bool,
        _hover_anim: f32,
        is_flowing: bool,
        is_bypassed_effect: bool,
        is_culprit_node: bool,
        is_overdriven_victim: bool,
        _theme_color: Color32,
        t: f32,
        _lerp_rgb: impl Fn(Color32, Color32, f32) -> Color32,
    ) {
        let cutoff = if node.param_val > 10.0 { node.param_val } else { 80.0 };
        let active_cutoff = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(cutoff);

        // 1. Inverse Wedge Polygon Chassis with 45° cut on bottom-left (Sub Drain)
        let poly = vec![
            Pos2::new(node_rect.min.x, node_rect.min.y + 8.0),
            Pos2::new(node_rect.min.x + 8.0, node_rect.min.y),
            Pos2::new(node_rect.max.x - 8.0, node_rect.min.y),
            Pos2::new(node_rect.max.x, node_rect.min.y + 8.0),
            Pos2::new(node_rect.max.x, node_rect.max.y - 8.0),
            Pos2::new(node_rect.max.x - 8.0, node_rect.max.y),
            Pos2::new(node_rect.min.x + 38.0, node_rect.max.y),
            Pos2::new(node_rect.min.x, node_rect.max.y - 28.0),
        ];

        let bg = if is_dragging {
            Color32::from_rgb(241, 245, 249)
        } else if is_bypassed_effect {
            Color32::from_rgba_premultiplied(248, 250, 252, 230)
        } else if !is_flowing {
            Color32::from_rgb(255, 248, 248)
        } else {
            Color32::WHITE
        };

        painter.add(egui::Shape::convex_polygon(poly.clone(), bg, Stroke::NONE));

        let border_col = if is_culprit_node {
            Color32::from_rgb(239, 68, 68)
        } else if is_overdriven_victim {
            Color32::from_rgb(245, 158, 11)
        } else if is_bypassed_effect {
            Color32::from_rgb(203, 213, 225)
        } else if is_flowing {
            Color32::from_rgb(56, 189, 248)
        } else {
            Color32::from_rgb(239, 68, 68)
        };

        // Perimeter Stroke & Outer Aura
        if is_flowing && self.is_playing && !is_bypassed_effect {
            let aura_alpha = (50.0 + 30.0 * (self.visualizer_anim_phase * 4.0).sin()) as u8;
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(3.0, Color32::from_rgba_unmultiplied(border_col.r(), border_col.g(), border_col.b(), aura_alpha / 2)));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(3.0, Color32::from_rgba_unmultiplied(border_col.r(), border_col.g(), border_col.b(), aura_alpha / 2)));

            // Traveling laser beacons on chassis perimeter
            let mut total_len = 0.0f32;
            for i in 0..poly.len() {
                total_len += poly[i].distance(poly[(i + 1) % poly.len()]);
            }
            let travel_dist = (self.visualizer_anim_phase * 130.0 + (node.id as f32) * 50.0) % total_len;
            let eval_poly = |dist: f32| -> Pos2 {
                let mut acc = 0.0f32;
                let d = (dist % total_len + total_len) % total_len;
                for i in 0..poly.len() {
                    let next_i = (i + 1) % poly.len();
                    let seg_len = poly[i].distance(poly[next_i]);
                    if acc + seg_len >= d {
                        let frac = if seg_len > 0.001 { (d - acc) / seg_len } else { 0.0 };
                        return poly[i].lerp(poly[next_i], frac);
                    }
                    acc += seg_len;
                }
                poly[0]
            };
            let b1 = eval_poly(travel_dist);
            let b2 = eval_poly(travel_dist + total_len * 0.5);
            painter.circle_filled(b1, 3.8, border_col);
            painter.circle_filled(b1, 1.8, Color32::WHITE);
            painter.circle_filled(b2, 3.8, Color32::from_rgb(147, 197, 253));
            painter.circle_filled(b2, 1.8, Color32::WHITE);

            let stroke_w = 1.6 + 0.4 * (self.visualizer_anim_phase * 3.5).sin();
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(stroke_w, border_col));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(stroke_w, border_col));
        } else {
            let stroke_w = if is_hovered { 1.8 } else { 1.2 };
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(stroke_w, border_col));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(stroke_w, border_col));
        }

        // Corner Bolts
        for &(bx, by) in &[(node_rect.min.x + 6.0, node_rect.min.y + 6.0),
                          (node_rect.max.x - 6.0, node_rect.min.y + 6.0),
                          (node_rect.max.x - 6.0, node_rect.max.y - 6.0),
                          (node_rect.min.x + 40.0, node_rect.max.y - 6.0)] {
            painter.circle_filled(Pos2::new(bx, by), 1.2, Color32::from_rgb(203, 213, 225));
        }

        // 2. Header Badge Pill: "🏔️ HPF PRISM"
        let title_pill = Rect::from_min_size(Pos2::new(node_rect.min.x + 10.0, node_rect.min.y + 4.0), Vec2::new(88.0, 15.0));
        painter.rect_filled(title_pill, 4.0, Color32::from_rgb(240, 249, 255));
        painter.rect_stroke(title_pill, 4.0, Stroke::new(0.8, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);
        painter.text(
            Pos2::new(title_pill.min.x + 4.0, title_pill.center().y),
            egui::Align2::LEFT_CENTER,
            "🏔️ HPF PRISM",
            egui::FontId::proportional(8.5),
            Color32::from_rgb(3, 105, 161),
        );
        painter.text(
            Pos2::new(title_pill.max.x - 4.0, title_pill.center().y),
            egui::Align2::RIGHT_CENTER,
            format!("#{}", node.id),
            egui::FontId::monospace(7.5),
            Color32::from_rgb(100, 116, 139),
        );

        // Status LED
        let led_pos = Pos2::new(node_rect.max.x - 12.0, node_rect.min.y + 11.0);
        if is_flowing && self.is_playing && !is_bypassed_effect {
            let led_pulse = (t * 5.0).sin().abs();
            painter.circle_stroke(led_pos, 4.2, Stroke::new(1.0, Color32::from_rgba_unmultiplied(56, 189, 248, (60.0 + 90.0 * led_pulse) as u8)));
            painter.circle_filled(led_pos, 2.2, Color32::from_rgb(56, 189, 248));
            painter.circle_filled(led_pos, 1.0, Color32::WHITE);
        } else if is_flowing && !is_bypassed_effect {
            painter.circle_filled(led_pos, 2.0, Color32::from_rgb(56, 189, 248));
        } else {
            painter.circle_filled(led_pos, 2.0, Color32::from_rgb(203, 213, 225));
        }

        // 3. Sub-Rumble Drain Void (Bottom-Left Cutaway Zone)
        let drain_center = Pos2::new(node_rect.min.x + 18.0, node_rect.max.y - 12.0);
        painter.circle_filled(drain_center, 8.5, Color32::from_rgb(15, 23, 42));
        painter.circle_stroke(drain_center, 8.5, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)));

        if is_flowing && self.is_playing && !is_bypassed_effect {
            // Dark amber/crimson motes spiraling into the drain
            for pi in 0..4 {
                let p_tau = (t * 2.2 + pi as f32 * 0.25).fract();
                let spiral_r = 7.0 * (1.0 - p_tau);
                let spiral_ang = t * 6.0 + pi as f32 * 1.57;
                let pt = Pos2::new(drain_center.x + spiral_r * spiral_ang.cos(), drain_center.y + spiral_r * spiral_ang.sin());
                painter.circle_filled(pt, 1.1, Color32::from_rgb(239, 68, 68));
            }
            painter.circle_filled(drain_center, 1.5, Color32::WHITE);
        } else {
            painter.circle_filled(drain_center, 1.5, Color32::from_rgb(71, 85, 105));
        }

        // 4. Plasma Prism Accelerator Spine & Interactive Slider
        let slider_track_rect = Rect::from_min_size(
            Pos2::new(node_rect.min.x + 24.0, node_rect.min.y + 36.0),
            Vec2::new((node_width - 48.0).max(10.0), 5.0),
        );
        let fill_ratio = ((active_cutoff / 20.0).ln() / (1000.0_f32).ln()).clamp(0.0, 1.0);
        let thumb_x = slider_track_rect.min.x + slider_track_rect.width() * fill_ratio;
        let cy = slider_track_rect.center().y;

        if !is_bypassed_effect {
            // Prism Spine Refraction Grid (Above slider)
            for gi in 0..7 {
                let gx = node_rect.min.x + 36.0 + gi as f32 * 18.0;
                painter.line_segment(
                    [Pos2::new(gx, node_rect.min.y + 22.0), Pos2::new(gx + 4.0, node_rect.min.y + 30.0)],
                    Stroke::new(0.8, Color32::from_rgb(224, 242, 254)),
                );
            }

            // Electric plasma arcs along the passband (from thumb_x to output pin)
            if is_flowing && self.is_playing {
                let arc_end_x = node_rect.max.x - 4.0;
                if thumb_x < arc_end_x {
                    let segs = 6;
                    let seg_w = (arc_end_x - thumb_x) / segs as f32;
                    let mut prev_pt = Pos2::new(thumb_x, cy - 8.0);
                    for s in 1..=segs {
                        let cur_x = thumb_x + s as f32 * seg_w;
                        let jitter = ((t * 22.0 + s as f32 * 3.1).sin()) * 2.5;
                        let cur_pt = Pos2::new(cur_x, cy - 8.0 + jitter);
                        painter.line_segment([prev_pt, cur_pt], Stroke::new(1.1, Color32::from_rgb(147, 197, 253)));
                        prev_pt = cur_pt;
                    }
                }
                ui.ctx().request_repaint();
            }

            // Precision Slider Track
            painter.rect_filled(Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(slider_track_rect.width(), 4.0)), 2.0, Color32::from_rgb(224, 242, 254));
            painter.rect_filled(Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(thumb_x - slider_track_rect.min.x, 4.0)), 2.0, Color32::from_rgb(56, 189, 248));

            // Plasma Needle Thumb
            painter.circle_filled(Pos2::new(thumb_x, cy + 1.0), 5.5, Color32::from_black_alpha(20));
            painter.circle_filled(Pos2::new(thumb_x, cy), 5.2, Color32::WHITE);
            painter.circle_stroke(Pos2::new(thumb_x, cy), 5.2, Stroke::new(1.8, Color32::from_rgb(2, 132, 199)));
            painter.circle_filled(Pos2::new(thumb_x, cy), 2.0, Color32::from_rgb(56, 189, 248));

            // Telemetry Badges
            let readout = if active_cutoff >= 1000.0 {
                format!("Cutoff {:.1}k", active_cutoff / 1000.0)
            } else {
                format!("Cutoff {:.0}Hz", active_cutoff)
            };

            let slope_str = if node.sub2.contains("48dB") { "48dB" } else if node.sub2.contains("24dB") { "24dB" } else { "12dB" };
            let q_str = if let Some(idx) = node.sub2.find("Q ") {
                node.sub2[idx + 2..].split_whitespace().next().unwrap_or("0.71")
            } else {
                "0.71"
            };

            let badge_y = node_rect.min.y + 63.0;
            let badge_w = (node_width - 34.0) * 0.48;
            let badge_h = 19.0;

            // Left: Cutoff
            let b_left = Rect::from_center_size(Pos2::new(node_rect.center().x - badge_w * 0.52, badge_y), Vec2::new(badge_w, badge_h));
            painter.rect_filled(b_left, 3.5, Color32::from_rgb(224, 242, 254));
            painter.rect_stroke(b_left, 3.5, Stroke::new(1.0, Color32::from_rgb(2, 132, 199)), egui::StrokeKind::Outside);
            painter.text(b_left.center(), egui::Align2::CENTER_CENTER, readout, egui::FontId::proportional(10.0), Color32::from_rgb(2, 132, 199));

            // Right: Slope & Q
            let b_right = Rect::from_center_size(Pos2::new(node_rect.center().x + badge_w * 0.52, badge_y), Vec2::new(badge_w, badge_h));
            painter.rect_filled(b_right, 3.5, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(b_right, 3.5, Stroke::new(0.8, Color32::from_rgb(71, 85, 105)), egui::StrokeKind::Outside);
            painter.text(b_right.center(), egui::Align2::CENTER_CENTER, format!("{} • Q{}", slope_str, q_str), egui::FontId::proportional(9.5), Color32::from_rgb(241, 245, 249));
        } else {
            let b_rect = Rect::from_center_size(Pos2::new(node_rect.center().x, node_rect.min.y + 44.0), Vec2::new(154.0, 22.0));
            painter.rect_filled(b_rect, 4.0, Color32::from_rgb(241, 245, 249));
            painter.rect_stroke(b_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
            painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, "DILEWATI (Bit-Perfect)", egui::FontId::proportional(11.0), Color32::from_rgb(100, 116, 139));
            painter.text(Pos2::new(node_rect.center().x, node_rect.min.y + 66.0), egui::Align2::CENTER_CENTER, "Passthrough 1:1 Stream", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        }
    }

    // --- 3. BAND-PASS FILTER: CENTRIFUGAL CYCLOTRON ---
    fn render_bpf_cyclotron(
        &self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        node: &crate::models::ModularNode,
        node_rect: Rect,
        node_width: f32,
        param_change: &mut Option<(usize, f32)>,
        is_hovered: bool,
        is_dragging: bool,
        _hover_anim: f32,
        is_flowing: bool,
        is_bypassed_effect: bool,
        is_culprit_node: bool,
        is_overdriven_victim: bool,
        _theme_color: Color32,
        t: f32,
        _lerp_rgb: impl Fn(Color32, Color32, f32) -> Color32,
    ) {
        let freq = if node.param_val > 10.0 { node.param_val } else { 1000.0 };
        let active_freq = param_change.filter(|(id, _)| *id == node.id).map(|(_, v)| v).unwrap_or(freq);

        // 1. Symmetrical Hexagonal / Bell-Prism Polygon Chassis
        let poly = vec![
            Pos2::new(node_rect.min.x + 16.0, node_rect.min.y + 5.0),
            Pos2::new(node_rect.center().x, node_rect.min.y), // Crest of bell curve
            Pos2::new(node_rect.max.x - 16.0, node_rect.min.y + 5.0),
            Pos2::new(node_rect.max.x, node_rect.center().y), // Output pin apex
            Pos2::new(node_rect.max.x - 16.0, node_rect.max.y - 5.0),
            Pos2::new(node_rect.center().x, node_rect.max.y),
            Pos2::new(node_rect.min.x + 16.0, node_rect.max.y - 5.0),
            Pos2::new(node_rect.min.x, node_rect.center().y), // Input pin apex
        ];

        let bg = if is_dragging {
            Color32::from_rgb(241, 245, 249)
        } else if is_bypassed_effect {
            Color32::from_rgba_premultiplied(248, 250, 252, 230)
        } else if !is_flowing {
            Color32::from_rgb(255, 248, 248)
        } else {
            Color32::WHITE
        };

        painter.add(egui::Shape::convex_polygon(poly.clone(), bg, Stroke::NONE));

        let border_col = if is_culprit_node {
            Color32::from_rgb(239, 68, 68)
        } else if is_overdriven_victim {
            Color32::from_rgb(245, 158, 11)
        } else if is_bypassed_effect {
            Color32::from_rgb(203, 213, 225)
        } else if is_flowing {
            Color32::from_rgb(245, 158, 11)
        } else {
            Color32::from_rgb(239, 68, 68)
        };

        // Perimeter Stroke & Outer Aura
        if is_flowing && self.is_playing && !is_bypassed_effect {
            let aura_alpha = (50.0 + 30.0 * (self.visualizer_anim_phase * 4.0).sin()) as u8;
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(3.0, Color32::from_rgba_unmultiplied(border_col.r(), border_col.g(), border_col.b(), aura_alpha / 2)));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(3.0, Color32::from_rgba_unmultiplied(border_col.r(), border_col.g(), border_col.b(), aura_alpha / 2)));

            // Traveling laser beacons on chassis perimeter
            let mut total_len = 0.0f32;
            for i in 0..poly.len() {
                total_len += poly[i].distance(poly[(i + 1) % poly.len()]);
            }
            let travel_dist = (self.visualizer_anim_phase * 130.0 + (node.id as f32) * 50.0) % total_len;
            let eval_poly = |dist: f32| -> Pos2 {
                let mut acc = 0.0f32;
                let d = (dist % total_len + total_len) % total_len;
                for i in 0..poly.len() {
                    let next_i = (i + 1) % poly.len();
                    let seg_len = poly[i].distance(poly[next_i]);
                    if acc + seg_len >= d {
                        let frac = if seg_len > 0.001 { (d - acc) / seg_len } else { 0.0 };
                        return poly[i].lerp(poly[next_i], frac);
                    }
                    acc += seg_len;
                }
                poly[0]
            };
            let b1 = eval_poly(travel_dist);
            let b2 = eval_poly(travel_dist + total_len * 0.5);
            painter.circle_filled(b1, 3.8, border_col);
            painter.circle_filled(b1, 1.8, Color32::WHITE);
            painter.circle_filled(b2, 3.8, Color32::from_rgb(251, 191, 36));
            painter.circle_filled(b2, 1.8, Color32::WHITE);

            let stroke_w = 1.6 + 0.4 * (self.visualizer_anim_phase * 3.5).sin();
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(stroke_w, border_col));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(stroke_w, border_col));
        } else {
            let stroke_w = if is_hovered { 1.8 } else { 1.2 };
            for w in poly.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(stroke_w, border_col));
            }
            painter.line_segment([poly[poly.len() - 1], poly[0]], Stroke::new(stroke_w, border_col));
        }

        // Corner Bolts
        for &(bx, by) in &[(node_rect.min.x + 18.0, node_rect.min.y + 8.0),
                          (node_rect.max.x - 18.0, node_rect.min.y + 8.0),
                          (node_rect.max.x - 18.0, node_rect.max.y - 8.0),
                          (node_rect.min.x + 18.0, node_rect.max.y - 8.0)] {
            painter.circle_filled(Pos2::new(bx, by), 1.2, Color32::from_rgb(203, 213, 225));
        }

        // 2. Header Badge Pill: "🎯 BPF CYCLO"
        let title_pill = Rect::from_min_size(Pos2::new(node_rect.min.x + 18.0, node_rect.min.y + 5.0), Vec2::new(88.0, 14.0));
        painter.rect_filled(title_pill, 4.0, Color32::from_rgb(254, 243, 199));
        painter.rect_stroke(title_pill, 4.0, Stroke::new(0.8, Color32::from_rgb(253, 224, 71)), egui::StrokeKind::Outside);
        painter.text(
            Pos2::new(title_pill.min.x + 4.0, title_pill.center().y),
            egui::Align2::LEFT_CENTER,
            "🎯 BPF CYCLO",
            egui::FontId::proportional(8.5),
            Color32::from_rgb(180, 83, 9),
        );
        painter.text(
            Pos2::new(title_pill.max.x - 4.0, title_pill.center().y),
            egui::Align2::RIGHT_CENTER,
            format!("#{}", node.id),
            egui::FontId::monospace(7.5),
            Color32::from_rgb(100, 116, 139),
        );

        // Status LED
        let led_pos = Pos2::new(node_rect.max.x - 22.0, node_rect.min.y + 11.0);
        if is_flowing && self.is_playing && !is_bypassed_effect {
            let led_pulse = (t * 5.0).sin().abs();
            painter.circle_stroke(led_pos, 4.2, Stroke::new(1.0, Color32::from_rgba_unmultiplied(245, 158, 11, (60.0 + 90.0 * led_pulse) as u8)));
            painter.circle_filled(led_pos, 2.2, Color32::from_rgb(245, 158, 11));
            painter.circle_filled(led_pos, 1.0, Color32::WHITE);
        } else if is_flowing && !is_bypassed_effect {
            painter.circle_filled(led_pos, 2.0, Color32::from_rgb(245, 158, 11));
        } else {
            painter.circle_filled(led_pos, 2.0, Color32::from_rgb(203, 213, 225));
        }

        // 3. Dual Ejection Exhaust Vents (Top-Left and Bottom-Left Flanks)
        for vi in 0..2 {
            let vx = node_rect.min.x + 24.0 + vi as f32 * 8.0;
            painter.line_segment([Pos2::new(vx, node_rect.min.y + 20.0), Pos2::new(vx + 4.0, node_rect.min.y + 25.0)], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
            painter.line_segment([Pos2::new(vx, node_rect.max.y - 20.0), Pos2::new(vx + 4.0, node_rect.max.y - 25.0)], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
        }

        // 4. Centrifugal Cyclotron Core & Interactive Slider
        let slider_track_rect = Rect::from_min_size(
            Pos2::new(node_rect.min.x + 24.0, node_rect.min.y + 36.0),
            Vec2::new((node_width - 48.0).max(10.0), 5.0),
        );
        let fill_ratio = ((active_freq / 20.0).ln() / (1000.0_f32).ln()).clamp(0.0, 1.0);
        let thumb_x = slider_track_rect.min.x + slider_track_rect.width() * fill_ratio;
        let cy = slider_track_rect.center().y;

        if !is_bypassed_effect {
            // Cyclotron Magnetic Ring Core at thumb_x
            let core_center = Pos2::new(thumb_x, cy);
            painter.circle_filled(core_center, 9.5, Color32::from_rgb(15, 23, 42));
            painter.circle_stroke(core_center, 9.5, Stroke::new(1.2, Color32::from_rgb(245, 158, 11)));

            if is_flowing && self.is_playing {
                // Dual counter-rotating plasma orbits
                let r_orb = 5.2;
                let ang1 = t * 5.2;
                let ang2 = -t * 3.8;
                let orb1 = Pos2::new(core_center.x + ang1.cos() * r_orb, core_center.y + ang1.sin() * r_orb);
                let orb2 = Pos2::new(core_center.x + ang2.cos() * r_orb, core_center.y + ang2.sin() * r_orb);

                painter.circle_filled(orb1, 1.8, Color32::from_rgb(251, 191, 36));
                painter.circle_filled(orb2, 1.8, Color32::from_rgb(245, 158, 11));

                // Pulsing central focus spark
                let spark_pulse = (t * 6.0).sin().abs();
                painter.circle_filled(core_center, 1.6 + spark_pulse * 0.8, Color32::WHITE);

                ui.ctx().request_repaint();
            } else {
                painter.circle_filled(core_center, 1.8, Color32::from_rgb(245, 158, 11));
            }

            // Amber Slider Track
            painter.rect_filled(Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(slider_track_rect.width(), 4.0)), 2.0, Color32::from_rgb(254, 243, 199));
            painter.rect_filled(Rect::from_min_size(Pos2::new(slider_track_rect.min.x, cy - 2.0), Vec2::new(thumb_x - slider_track_rect.min.x, 4.0)), 2.0, Color32::from_rgb(245, 158, 11));

            // Outer dial ring around cyclotron
            painter.circle_stroke(core_center, 11.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(245, 158, 11, 140)));

            // Telemetry Badges
            let readout = if active_freq >= 1000.0 {
                format!("Center {:.1}k", active_freq / 1000.0)
            } else {
                format!("Center {:.0}Hz", active_freq)
            };

            let slope_str = if node.sub2.contains("48dB") { "48dB" } else if node.sub2.contains("24dB") { "24dB" } else { "12dB" };
            let q_str = if let Some(idx) = node.sub2.find("Q ") {
                node.sub2[idx + 2..].split_whitespace().next().unwrap_or("1.41")
            } else {
                "1.41"
            };

            let badge_y = node_rect.min.y + 63.0;
            let badge_w = (node_width - 34.0) * 0.48;
            let badge_h = 19.0;

            // Left: Center Readout
            let b_left = Rect::from_center_size(Pos2::new(node_rect.center().x - badge_w * 0.52, badge_y), Vec2::new(badge_w, badge_h));
            painter.rect_filled(b_left, 3.5, Color32::from_rgb(254, 243, 199));
            painter.rect_stroke(b_left, 3.5, Stroke::new(1.0, Color32::from_rgb(217, 119, 6)), egui::StrokeKind::Outside);
            painter.text(b_left.center(), egui::Align2::CENTER_CENTER, readout, egui::FontId::proportional(10.0), Color32::from_rgb(217, 119, 6));

            // Right: Bandwidth & Q
            let b_right = Rect::from_center_size(Pos2::new(node_rect.center().x + badge_w * 0.52, badge_y), Vec2::new(badge_w, badge_h));
            painter.rect_filled(b_right, 3.5, Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(b_right, 3.5, Stroke::new(0.8, Color32::from_rgb(71, 85, 105)), egui::StrokeKind::Outside);
            painter.text(b_right.center(), egui::Align2::CENTER_CENTER, format!("{} • Q{}", slope_str, q_str), egui::FontId::proportional(9.5), Color32::from_rgb(241, 245, 249));
        } else {
            let b_rect = Rect::from_center_size(Pos2::new(node_rect.center().x, node_rect.min.y + 44.0), Vec2::new(154.0, 22.0));
            painter.rect_filled(b_rect, 4.0, Color32::from_rgb(241, 245, 249));
            painter.rect_stroke(b_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
            painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, "DILEWATI (Bit-Perfect)", egui::FontId::proportional(11.0), Color32::from_rgb(100, 116, 139));
            painter.text(Pos2::new(node_rect.center().x, node_rect.min.y + 66.0), egui::Align2::CENTER_CENTER, "Passthrough 1:1 Stream", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
        }
    }

}
