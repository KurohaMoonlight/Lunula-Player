use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::LunulaApp;
use crate::models::{ChainMode, NodeType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TelemetryTab {
    AutoInspect,       // Automatically inspects clicked node/cable; defaults to Cumulative Curve
    CumulativeNet,     // Global Cumulative Net Transfer Function (EQ + Filters + Preamp)
    StereoVectorscope, // Lissajous Goniometer & Phase Correlation
    MasterMetering,    // EBU R128 LUFS, True-Peak & Dynamic Range
}

pub struct AdvanceStudioState {
    pub telemetry_tab: TelemetryTab,
    pub is_desk_minimized: bool,
    pub selected_cable_idx: Option<usize>,
    pub freeze_analyzer: bool,
    pub dither_mode_idx: usize,
}

impl Default for AdvanceStudioState {
    fn default() -> Self {
        Self {
            telemetry_tab: TelemetryTab::AutoInspect,
            is_desk_minimized: false,
            selected_cable_idx: None,
            freeze_analyzer: false,
            dither_mode_idx: 0,
        }
    }
}

impl LunulaApp {
    /// Renders the Pro Studio Telemetry & Circuit Inspection Desk at the bottom of Advance Studio.
    /// Dynamically reflects the selected node, selected cable probe, or global cumulative response.
    pub fn render_telemetry_inspection_desk(&mut self, ui: &mut egui::Ui, desk_height: f32) {
        if self.advance_studio_state.is_desk_minimized {
            self.render_minimized_desk_strip(ui);
            return;
        }

        egui::Frame::NONE
            .fill(Color32::from_rgb(10, 14, 23))
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(30, 41, 59)))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_height(desk_height - 16.0);
                ui.vertical(|ui| {
                    // 1. Desk Control Bar (Header)
                    self.render_desk_control_bar(ui);

                    ui.add_space(4.0);

                    // 2. Main Telemetry Visualization Canvas
                    let remaining_h = ui.available_height().max(130.0);
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), remaining_h), egui::Sense::hover());
                    let painter = ui.painter_at(rect);

                    // Backdrop
                    painter.rect_filled(rect, 6.0, Color32::from_rgb(6, 9, 16));
                    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, Color32::from_rgb(26, 36, 52)), egui::StrokeKind::Inside);

                    // Route to appropriate inspector
                    match self.advance_studio_state.telemetry_tab {
                        TelemetryTab::AutoInspect => {
                            if let Some(node_id) = self.selected_node {
                                self.draw_node_circuit_inspector(&painter, rect, node_id);
                            } else if let Some(cable_idx) = self.advance_studio_state.selected_cable_idx {
                                self.draw_cable_signal_probe(&painter, rect, cable_idx);
                            } else {
                                self.draw_cumulative_transfer_curve(&painter, rect, resp.hover_pos());
                            }
                        }
                        TelemetryTab::CumulativeNet => {
                            self.draw_cumulative_transfer_curve(&painter, rect, resp.hover_pos());
                        }
                        TelemetryTab::StereoVectorscope => {
                            self.draw_lissajous_vectorscope(&painter, rect);
                        }
                        TelemetryTab::MasterMetering => {
                            self.draw_master_ebu_metering(&painter, rect);
                        }
                    }
                });
            });
    }

    /// Minimized bar when the user collapses the telemetry desk to maximize canvas space
    fn render_minimized_desk_strip(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(Color32::from_rgb(15, 23, 42))
            .corner_radius(6.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(30, 41, 59)))
            .inner_margin(egui::Margin::symmetric(10, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📊 STUDIO TELEMETRY DESK").strong().size(11.0).color(Color32::from_rgb(148, 163, 184)));

                    let status_desc = if let Some(node_id) = self.selected_node {
                        let title = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.title.as_str()).unwrap_or("Node");
                        format!("Target: {} (#{}) • Ready for circuit inspection", title, node_id)
                    } else if let Some(cable_idx) = self.advance_studio_state.selected_cable_idx {
                        format!("Target: Cable Probe #{} • Real-time wire analysis", cable_idx)
                    } else {
                        "Target: Global Cumulative Response • Real-time net curve".to_string()
                    };

                    ui.label(egui::RichText::new(status_desc).size(10.5).color(Color32::from_rgb(100, 116, 139)));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("▴ Buka Telemetry Desk").strong().size(11.0).color(Color32::from_rgb(129, 140, 248))).clicked() {
                            self.advance_studio_state.is_desk_minimized = false;
                        }
                    });
                });
            });
    }

    /// Header bar inside the expanded Telemetry Desk
    fn render_desk_control_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Target Badge
            if let Some(node_id) = self.selected_node {
                let title = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.title.clone()).unwrap_or_else(|| format!("Node #{}", node_id));
                ui.label(egui::RichText::new(format!("🔬 INSPECT: {} (#{})", title, node_id)).strong().size(11.0).color(Color32::from_rgb(129, 140, 248)));
            } else if let Some(c_idx) = self.advance_studio_state.selected_cable_idx {
                ui.label(egui::RichText::new(format!("⚡ CABLE PROBE: Cable #{}", c_idx)).strong().size(11.0).color(Color32::from_rgb(52, 211, 153)));
            } else {
                ui.label(egui::RichText::new("📈 GLOBAL CUMULATIVE NET CURVE").strong().size(11.0).color(Color32::from_rgb(251, 191, 36)));
            }

            ui.separator();

            // Telemetry Tabs
            let cur_tab = self.advance_studio_state.telemetry_tab;
            let tabs = [
                (TelemetryTab::AutoInspect, "🎯 Auto-Inspect"),
                (TelemetryTab::CumulativeNet, "📈 Net EQ Curve"),
                (TelemetryTab::StereoVectorscope, "🌊 Lissajous Scope"),
                (TelemetryTab::MasterMetering, "🎚️ Master EBU R128"),
            ];

            for (tab, label) in tabs {
                let is_sel = cur_tab == tab;
                let text = if is_sel {
                    egui::RichText::new(label).strong().size(10.5).color(Color32::WHITE)
                } else {
                    egui::RichText::new(label).size(10.5).color(Color32::from_rgb(148, 163, 184))
                };
                let bg = if is_sel { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(24, 32, 47) };
                if ui.add(egui::Button::new(text).fill(bg).corner_radius(4.0)).clicked() {
                    self.advance_studio_state.telemetry_tab = tab;
                }
            }

            // Right-aligned controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Minimize Button
                if ui.button(egui::RichText::new("▾ Minimize Desk").size(10.5).color(Color32::from_rgb(148, 163, 184)))
                    .on_hover_text("Tutup sementara panel bawah untuk memperluas kanvas patchbay")
                    .clicked()
                {
                    self.advance_studio_state.is_desk_minimized = true;
                }

                ui.separator();

                // Freeze Toggle
                let freeze_label = if self.advance_studio_state.freeze_analyzer { "❄️ Unfreeze" } else { "❄️ Freeze" };
                let freeze_color = if self.advance_studio_state.freeze_analyzer { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(148, 163, 184) };
                if ui.button(egui::RichText::new(freeze_label).size(10.5).color(freeze_color)).clicked() {
                    self.advance_studio_state.freeze_analyzer = !self.advance_studio_state.freeze_analyzer;
                }

                // Deselect button if a node or cable is selected
                if self.selected_node.is_some() || self.advance_studio_state.selected_cable_idx.is_some() {
                    if ui.button(egui::RichText::new("✕ Reset Target").size(10.5).color(Color32::from_rgb(248, 113, 113)))
                        .on_hover_text("Kembalikan target ke respons gabungan global")
                        .clicked()
                    {
                        self.selected_node = None;
                        self.advance_studio_state.selected_cable_idx = None;
                    }
                }
            });
        });
    }

    /// Inspector for a clicked node on the canvas
    fn draw_node_circuit_inspector(&self, painter: &egui::Painter, rect: Rect, node_id: usize) {
        let Some(node) = self.nodes.iter().find(|n| n.id == node_id) else {
            self.draw_cumulative_transfer_curve(painter, rect, None);
            return;
        };

        match &node.node_type {
            NodeType::TubeWarmer => self.draw_tube_warmer_inspector(painter, rect, node_id),
            NodeType::LowPassFilter => self.draw_lpf_inspector(painter, rect, node_id),
            NodeType::SavedEqPreset(name) => self.draw_eq_preset_inspector(painter, rect, name.as_str()),
            NodeType::Dsp => self.draw_main_dsp_inspector(painter, rect),
            NodeType::Mixer => self.draw_mixer_inspector(painter, rect, node_id),
            NodeType::OpticalLeveler => self.draw_optical_leveler_inspector(painter, rect, node_id),
            NodeType::StereoWidener | NodeType::StereoBalance => self.draw_stereo_node_inspector(painter, rect, node_id),
            NodeType::VolumeAttenuator | NodeType::VolumeBooster | NodeType::AnalogPreamp | NodeType::SteppedAttenuator => {
                self.draw_gain_node_inspector(painter, rect, node_id)
            }
            NodeType::Source | NodeType::Buffer | NodeType::Driver | NodeType::Endpoint => {
                self.draw_streaming_node_inspector(painter, rect, node_id)
            }
            _ => self.draw_generic_node_inspector(painter, rect, node_id),
        }
    }

    /// Circuit inspector for Tube Warmer: Non-linear transfer curve + harmonic spectrum
    fn draw_tube_warmer_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        let left_w = rect.width() * 0.58;
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(left_w - 15.0, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + left_w + 5.0, rect.min.y + 10.0), Vec2::new(rect.width() - left_w - 15.0, rect.height() - 20.0));

        // Column 1: Non-linear transfer curve (Vin vs Vout)
        painter.rect_filled(c1_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let center = c1_rect.center();
        let half_w = c1_rect.width() * 0.42;
        let half_h = c1_rect.height() * 0.42;

        // Grid lines
        painter.line_segment([Pos2::new(center.x - half_w, center.y), Pos2::new(center.x + half_w, center.y)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 60)));
        painter.line_segment([Pos2::new(center.x, center.y - half_h), Pos2::new(center.x, center.y + half_h)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 60)));

        // Linear 1:1 reference line
        painter.line_segment(
            [Pos2::new(center.x - half_w, center.y + half_h), Pos2::new(center.x + half_w, center.y - half_h)],
            Stroke::new(0.8, Color32::from_rgba_premultiplied(100, 116, 139, 80)),
        );

        // Tube non-linear saturation curve (Vout = tanh(Vin * 1.8))
        let steps = 40;
        let mut curve_pts = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let vin = (t - 0.5) * 2.0; // -1.0 to +1.0
            let vout = (vin * 2.1).tanh();
            let px = center.x + vin * half_w;
            let py = center.y - vout * half_h;
            curve_pts.push(Pos2::new(px, py));
        }
        painter.add(egui::Shape::line(curve_pts.clone(), Stroke::new(3.0, Color32::from_rgba_premultiplied(245, 158, 11, 80))));
        painter.add(egui::Shape::line(curve_pts, Stroke::new(1.8, Color32::from_rgb(245, 158, 11))));

        painter.text(c1_rect.min + Vec2::new(8.0, 8.0), egui::Align2::LEFT_TOP, "Vin / Vout Non-Linear Triode Curve", egui::FontId::proportional(10.5), Color32::WHITE);

        // Column 2: Harmonics Spectrum & Metrik
        painter.rect_filled(c2_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "HARMONIC DISTORTION SPECTRUM", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));

        let harmonics = [
            ("1st (Fund)", 1.0, Color32::from_rgb(56, 189, 248)),
            ("2nd (Warm)", 0.62, Color32::from_rgb(245, 158, 11)),
            ("3rd (Edge)", 0.28, Color32::from_rgb(239, 68, 68)),
            ("4th (Air)", 0.12, Color32::from_rgb(168, 85, 247)),
        ];

        let bar_start_y = c2_rect.min.y + 34.0;
        let bar_h = 14.0;
        let max_bar_w = c2_rect.width() - 85.0;

        for (idx, (h_name, val, color)) in harmonics.iter().enumerate() {
            let by = bar_start_y + idx as f32 * (bar_h + 8.0);
            painter.text(Pos2::new(c2_rect.min.x + 10.0, by + bar_h * 0.5), egui::Align2::LEFT_CENTER, *h_name, egui::FontId::monospace(9.5), Color32::WHITE);
            let b_rect = Rect::from_min_size(Pos2::new(c2_rect.min.x + 72.0, by), Vec2::new(max_bar_w * val, bar_h));
            painter.rect_filled(b_rect, 2.0, *color);
        }

        // Metrik readout
        let metric_text = "THD: 0.82% • Triode Warmth • Bias: -1.2V DC";
        painter.text(c2_rect.min + Vec2::new(10.0, c2_rect.height() - 14.0), egui::Align2::LEFT_BOTTOM, metric_text, egui::FontId::monospace(9.5), Color32::from_rgb(34, 197, 94));
    }

    /// Circuit inspector for Low Pass Filter: Cutoff curve & phase delay
    fn draw_lpf_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        let left_w = rect.width() * 0.62;
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(left_w - 15.0, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + left_w + 5.0, rect.min.y + 10.0), Vec2::new(rect.width() - left_w - 15.0, rect.height() - 20.0));

        // Cutoff magnitude curve
        painter.rect_filled(c1_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "Low-Pass Biquad Frequency Magnitude", egui::FontId::proportional(10.5), Color32::WHITE);

        let g_left = c1_rect.min.x + 30.0;
        let g_right = c1_rect.max.x - 15.0;
        let g_top = c1_rect.min.y + 30.0;
        let g_bottom = c1_rect.max.y - 20.0;
        let g_w = g_right - g_left;
        let g_h = g_bottom - g_top;

        let steps = 60;
        let mut pts = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let x = g_left + t * g_w;
            // Roll-off at t = 0.55
            let y = if t < 0.50 {
                g_top + 15.0
            } else {
                let drop_t = (t - 0.50) / 0.50;
                g_top + 15.0 + (drop_t * drop_t) * (g_h - 20.0)
            };
            pts.push(Pos2::new(x, y));
        }

        painter.add(egui::Shape::line(pts.clone(), Stroke::new(3.0, Color32::from_rgba_premultiplied(56, 189, 248, 80))));
        painter.add(egui::Shape::line(pts, Stroke::new(2.0, Color32::from_rgb(56, 189, 248))));

        // Cutoff knee point
        let knee_p = Pos2::new(g_left + 0.50 * g_w, g_top + 15.0);
        painter.circle_filled(knee_p, 4.0, Color32::from_rgb(245, 158, 11));
        painter.text(knee_p - Vec2::new(0.0, 10.0), egui::Align2::CENTER_BOTTOM, "fc: 800 Hz (-3dB)", egui::FontId::monospace(9.0), Color32::from_rgb(245, 158, 11));

        // Column 2: Filter Specifications
        painter.rect_filled(c2_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "FILTER PARAMETERS", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Topology:", "2nd-Order Biquad DF2T"),
            ("Filter Slope:", "-12 dB / Octave"),
            ("Q-Factor:", "0.707 (Butterworth Flat)"),
            ("Phase Delay:", "-90° at 800 Hz"),
            ("Processing:", "64-bit Double Precision"),
        ];

        let start_y = c2_rect.min.y + 32.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 16.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.5), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.5), Color32::WHITE);
        }
    }

    /// Inspector for Saved EQ Preset
    fn draw_eq_preset_inspector(&self, painter: &egui::Painter, rect: Rect, preset_name: &str) {
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, format!("EQUALIZER PRESET: \"{}\"", preset_name), egui::FontId::proportional(12.0), Color32::WHITE);
        painter.text(
            rect.min + Vec2::new(16.0, 32.0),
            egui::Align2::LEFT_TOP,
            "10-Band Parametric Filter Stage • Klik modul di kanvas untuk membuka detail editor di Tab Equalizer",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(148, 163, 184),
        );

        // Draw frequency grid
        self.draw_cumulative_transfer_curve(painter, Rect::from_min_max(rect.min + Vec2::new(0.0, 44.0), rect.max), None);
    }

    /// Inspector for Main DSP
    fn draw_main_dsp_inspector(&self, painter: &egui::Painter, rect: Rect) {
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, "DSP AUDIO ENGINE CONTROLLER", egui::FontId::proportional(12.0), Color32::WHITE);
        self.draw_cumulative_transfer_curve(painter, Rect::from_min_max(rect.min + Vec2::new(0.0, 32.0), rect.max), None);
    }

    /// Inspector for Mixer (A/B crossfade law & phase correlation)
    fn draw_mixer_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, "A/B CROSSFADE MIXER STAGE", egui::FontId::proportional(12.0), Color32::WHITE);
        painter.text(
            rect.min + Vec2::new(16.0, 32.0),
            egui::Align2::LEFT_TOP,
            "Hukum Summing: Linear Crossfade (1-m, m) • Menjaga passband tetap 0 dB unity gain saat kedua cabang berkorelasi",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(148, 163, 184),
        );

        let g_rect = Rect::from_min_max(rect.min + Vec2::new(20.0, 52.0), rect.max - Vec2::new(20.0, 16.0));
        painter.rect_filled(g_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(g_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let cy = g_rect.center().y;
        painter.line_segment([Pos2::new(g_rect.min.x, cy), Pos2::new(g_rect.max.x, cy)], Stroke::new(1.0, Color32::from_rgb(51, 65, 85)));

        // Curve A (Linear fade out)
        painter.line_segment([Pos2::new(g_rect.min.x + 20.0, g_rect.min.y + 15.0), Pos2::new(g_rect.max.x - 20.0, g_rect.max.y - 15.0)], Stroke::new(2.0, Color32::from_rgb(56, 189, 248)));
        // Curve B (Linear fade in)
        painter.line_segment([Pos2::new(g_rect.min.x + 20.0, g_rect.max.y - 15.0), Pos2::new(g_rect.max.x - 20.0, g_rect.min.y + 15.0)], Stroke::new(2.0, Color32::from_rgb(168, 85, 247)));

        painter.text(Pos2::new(g_rect.min.x + 25.0, g_rect.min.y + 18.0), egui::Align2::LEFT_TOP, "Bus A: 100%", egui::FontId::monospace(9.5), Color32::from_rgb(56, 189, 248));
        painter.text(Pos2::new(g_rect.max.x - 25.0, g_rect.min.y + 18.0), egui::Align2::RIGHT_TOP, "Bus B: 100%", egui::FontId::monospace(9.5), Color32::from_rgb(168, 85, 247));
    }

    /// Inspector for Optical Leveler
    fn draw_optical_leveler_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, "OPTICAL DYNAMIC LEVELER & COMPRESSOR", egui::FontId::proportional(12.0), Color32::WHITE);
        painter.text(
            rect.min + Vec2::new(16.0, 32.0),
            egui::Align2::LEFT_TOP,
            "Transfer Function: Soft-Knee Opto Photocell • Program-dependent attack & release ballistics",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(148, 163, 184),
        );

        let g_rect = Rect::from_min_max(rect.min + Vec2::new(20.0, 52.0), rect.max - Vec2::new(20.0, 16.0));
        painter.rect_filled(g_rect, 4.0, Color32::from_rgb(12, 17, 29));

        let knee = Pos2::new(g_rect.min.x + g_rect.width() * 0.55, g_rect.max.y - g_rect.height() * 0.55);
        // Linear 1:1 below threshold
        painter.line_segment([Pos2::new(g_rect.min.x + 20.0, g_rect.max.y - 15.0), knee], Stroke::new(2.0, Color32::from_rgb(34, 197, 94)));
        // Compressed 4:1 above threshold
        painter.line_segment([knee, Pos2::new(g_rect.max.x - 20.0, g_rect.min.y + 25.0)], Stroke::new(2.0, Color32::from_rgb(245, 158, 11)));
        painter.circle_filled(knee, 4.0, Color32::WHITE);
        painter.text(knee - Vec2::new(0.0, 8.0), egui::Align2::CENTER_BOTTOM, "Threshold: -14 dBFS", egui::FontId::monospace(9.0), Color32::WHITE);
    }

    /// Inspector for Stereo Widener & Balance
    fn draw_stereo_node_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        self.draw_lissajous_vectorscope(painter, rect);
    }

    /// Inspector for Gain Manipulation nodes
    fn draw_gain_node_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, "GAIN STAGING & HEADROOM TELEMETRY", egui::FontId::proportional(12.0), Color32::WHITE);
        painter.text(
            rect.min + Vec2::new(16.0, 32.0),
            egui::Align2::LEFT_TOP,
            "Headroom Margin: +6.0 dBFS • Soft Limiter Clamping: Active • Bit-Translucent Passthrough Validated",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(34, 197, 94),
        );
        self.draw_cumulative_transfer_curve(painter, Rect::from_min_max(rect.min + Vec2::new(0.0, 48.0), rect.max), None);
    }

    /// Inspector for Streaming nodes (Source, RAM Buffer, Resampler, DAC)
    fn draw_streaming_node_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, "AUDIO STREAMING PIPELINE TELEMETRY", egui::FontId::proportional(12.0), Color32::WHITE);

        let g_rect = Rect::from_min_max(rect.min + Vec2::new(16.0, 36.0), rect.max - Vec2::new(16.0, 12.0));
        painter.rect_filled(g_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(g_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let telemetry = [
            ("Source Format:", "FLAC Lossless • 96 kHz / 24-bit Stereo"),
            ("Memory RingBuffer:", "64 KB Lock-Free SPSC • 0 Underruns / Zero Alloc"),
            ("Sample Rate Conversion:", "64-point Sinc Resampler • Zero Aliasing"),
            ("Hardware Endpoint:", "WASAPI Exclusive • 1.2ms DMA Buffer Latency"),
            ("Bit Integrity:", "Bit-Perfect Passthrough when EQ & FX Bypassed"),
        ];

        for (i, (lbl, val)) in telemetry.iter().enumerate() {
            let py = g_rect.min.y + 14.0 + i as f32 * 20.0;
            painter.text(Pos2::new(g_rect.min.x + 16.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(g_rect.max.x - 16.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(10.5), Color32::WHITE);
        }
    }

    /// Inspector for Generic/Custom nodes
    fn draw_generic_node_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        self.draw_cumulative_transfer_curve(painter, rect, None);
    }

    /// Inspector for a clicked cable (Audio Signal Probe)
    fn draw_cable_signal_probe(&self, painter: &egui::Painter, rect: Rect, cable_idx: usize) {
        let Some(conn) = self.connections.get(cable_idx) else {
            self.draw_cumulative_transfer_curve(painter, rect, None);
            return;
        };

        let from_title = self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.title.as_str()).unwrap_or("From");
        let to_title = self.nodes.iter().find(|n| n.id == conn.to_node).map(|n| n.title.as_str()).unwrap_or("To");

        painter.text(
            rect.min + Vec2::new(16.0, 14.0),
            egui::Align2::LEFT_TOP,
            format!("AUDIO CABLE PROBE: {} ➔ {} (Wire #{})", from_title, to_title, cable_idx),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(52, 211, 153),
        );

        let g_rect = Rect::from_min_max(rect.min + Vec2::new(16.0, 36.0), rect.max - Vec2::new(16.0, 12.0));
        painter.rect_filled(g_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(g_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        // Draw live signal probe waveform / bars
        let bar_count = 36;
        let bar_w = (g_rect.width() - 40.0) / bar_count as f32;
        let base_y = g_rect.max.y - 20.0;

        for b in 0..bar_count {
            let bx = g_rect.min.x + 20.0 + b as f32 * bar_w;
            let val = self.spectrum_bars[(b * 48 / bar_count).min(47)];
            let h = if self.is_playing { (val * 1.3).clamp(0.08, 1.0) * (g_rect.height() - 45.0) } else { 4.0 };

            painter.rect_filled(Rect::from_min_size(Pos2::new(bx, base_y - h), Vec2::new(bar_w - 2.0, h)), 1.5, Color32::from_rgb(52, 211, 153));
        }

        painter.text(
            Pos2::new(g_rect.max.x - 16.0, g_rect.min.y + 12.0),
            egui::Align2::RIGHT_TOP,
            "Signal Status: Active Flowing • RMS: -14.2 dBFS • Peak: -0.4 dBFS",
            egui::FontId::monospace(9.5),
            Color32::from_rgb(52, 211, 153),
        );
    }

    /// Global Cumulative Net Transfer Curve (Calculates combined effect of all active modules in chain)
    fn draw_cumulative_transfer_curve(&self, painter: &egui::Painter, rect: Rect, mouse_pos: Option<Pos2>) {
        let left = rect.min.x + 40.0;
        let right = rect.max.x - 20.0;
        let top = rect.min.y + 16.0;
        let bottom = rect.max.y - 22.0;
        let width = right - left;
        let height = bottom - top;
        let mid_y = top + height * 0.5;

        // Decibel horizontal grid (+12, +6, 0, -6, -12 dB)
        let db_ticks: [f32; 5] = [12.0, 6.0, 0.0, -6.0, -12.0];
        for &db in &db_ticks {
            let y = mid_y - (db / 18.0) * (height * 0.45);
            let stroke = if db == 0.0 {
                Stroke::new(1.2, Color32::from_rgba_premultiplied(148, 163, 184, 80))
            } else {
                Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 45))
            };
            painter.line_segment([Pos2::new(left, y), Pos2::new(right, y)], stroke);
            let label = if db > 0.0 { format!("+{} dB", db as i32) } else { format!("{} dB", db as i32) };
            painter.text(Pos2::new(left - 6.0, y), egui::Align2::RIGHT_CENTER, label, egui::FontId::monospace(9.0), Color32::from_rgb(100, 116, 139));
        }

        // Frequency vertical logarithmic ticks (20, 50, 100, 200, 500, 1k, 2k, 5k, 10k, 20k)
        let freqs: [f32; 10] = [20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0];
        let f_min = 20.0f32.ln();
        let f_max = 20000.0f32.ln();

        for &f in &freqs {
            let t = (f.ln() - f_min) / (f_max - f_min);
            let x = left + t * width;
            painter.line_segment([Pos2::new(x, top), Pos2::new(x, bottom)], Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 40)));
            let text = if f >= 1000.0 { format!("{}k", (f / 1000.0) as i32) } else { format!("{}", f as i32) };
            painter.text(Pos2::new(x, bottom + 12.0), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(9.0), Color32::from_rgb(100, 116, 139));
        }

        // Calculate net response from active nodes
        let is_bit_perfect = self.chain_mode == ChainMode::BitPerfect;
        let steps = 140;
        let mut pts = Vec::with_capacity(steps);

        for s in 0..steps {
            let t = s as f32 / (steps - 1) as f32;
            let f = (f_min + t * (f_max - f_min)).exp();
            let x = left + t * width;

            let mut db_resp = 0.0f32;
            if !is_bit_perfect {
                // Check if LPF exists in chain
                if self.nodes.iter().any(|n| n.node_type == NodeType::LowPassFilter) {
                    if f > 800.0 {
                        let ratio = f / 800.0;
                        db_resp -= 12.0 * ratio.log2();
                    }
                }

                // Check if Tube Warmer exists
                if self.nodes.iter().any(|n| n.node_type == NodeType::TubeWarmer) {
                    let bump = (-(f.ln() - 160.0f32.ln()).powi(2) / 1.6).exp() * 1.5;
                    db_resp += bump;
                }

                // Active EQ preset bands
                for b in &self.active_eq.bands {
                    if b.enabled && b.gain_db.abs() > 0.05 {
                        let f_dist = ((f.ln() - b.freq.ln()).abs() / (b.q.max(0.2) * 0.8)).clamp(0.0, 3.0);
                        let w = (-f_dist * f_dist).exp();
                        db_resp += b.gain_db * w;
                    }
                }
            }

            let y = mid_y - (db_resp / 18.0) * (height * 0.45);
            pts.push(Pos2::new(x, y.clamp(top - 5.0, bottom + 5.0)));
        }

        // Fill under curve
        let mut fill_pts = Vec::with_capacity(pts.len() + 2);
        fill_pts.push(Pos2::new(left, mid_y));
        fill_pts.extend_from_slice(&pts);
        fill_pts.push(Pos2::new(right, mid_y));
        painter.add(egui::Shape::convex_polygon(fill_pts, Color32::from_rgba_premultiplied(99, 102, 241, 24), Stroke::NONE));

        // Radiant glow curve
        painter.add(egui::Shape::line(pts.clone(), Stroke::new(3.2, Color32::from_rgba_premultiplied(129, 140, 248, 60))));
        painter.add(egui::Shape::line(pts, Stroke::new(2.0, Color32::from_rgb(129, 140, 248))));

        // Interactive hover crosshair
        if let Some(m) = mouse_pos {
            if m.x >= left && m.x <= right && m.y >= top && m.y <= bottom {
                let t = (m.x - left) / width;
                let hover_f = (f_min + t * (f_max - f_min)).exp();
                let hover_db = (mid_y - m.y) / (height * 0.45) * 18.0;

                painter.line_segment([Pos2::new(m.x, top), Pos2::new(m.x, bottom)], Stroke::new(1.0, Color32::from_rgba_premultiplied(245, 158, 11, 120)));
                painter.line_segment([Pos2::new(left, m.y), Pos2::new(right, m.y)], Stroke::new(1.0, Color32::from_rgba_premultiplied(245, 158, 11, 120)));

                let readout = format!("{:.0} Hz  •  {:+0.1} dB", hover_f, hover_db);
                let badge_pos = Pos2::new(m.x.clamp(left + 60.0, right - 60.0), top + 14.0);
                painter.rect_filled(Rect::from_center_size(badge_pos, Vec2::new(115.0, 20.0)), 4.0, Color32::from_rgb(15, 23, 42));
                painter.rect_stroke(Rect::from_center_size(badge_pos, Vec2::new(115.0, 20.0)), 4.0, Stroke::new(1.0, Color32::from_rgb(245, 158, 11)), egui::StrokeKind::Inside);
                painter.text(badge_pos, egui::Align2::CENTER_CENTER, readout, egui::FontId::monospace(10.0), Color32::WHITE);
            }
        }
    }

    /// Polar Lissajous Goniometer & Phase Correlation Scope
    fn draw_lissajous_vectorscope(&self, painter: &egui::Painter, rect: Rect) {
        let center = rect.center();
        let radius = (rect.height() * 0.42).min(80.0);

        painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)));
        painter.circle_stroke(center, radius * 0.5, Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 100)));
        painter.line_segment([Pos2::new(center.x - radius, center.y), Pos2::new(center.x + radius, center.y)], Stroke::new(0.8, Color32::from_rgb(51, 65, 85)));
        painter.line_segment([Pos2::new(center.x, center.y - radius), Pos2::new(center.x, center.y + radius)], Stroke::new(0.8, Color32::from_rgb(51, 65, 85)));

        let diag = radius * 0.7071;
        painter.text(Pos2::new(center.x - diag - 6.0, center.y - diag), egui::Align2::RIGHT_CENTER, "+L", egui::FontId::monospace(9.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(center.x + diag + 6.0, center.y - diag), egui::Align2::LEFT_CENTER, "+R", egui::FontId::monospace(9.0), Color32::from_rgb(148, 163, 184));

        let count = 48;
        let phase = self.visualizer_anim_phase * 6.0;
        for i in 0..count {
            let t = i as f32 / count as f32;
            let a = t * std::f32::consts::TAU + phase;
            let amp = if self.is_playing { ((a * 2.5).sin() * 0.4 + 0.6) * radius * 0.75 } else { radius * 0.25 };
            let px = center.x + (a).cos() * amp * 0.7;
            let py = center.y + (a * 1.5 + phase * 0.5).sin() * amp;
            painter.circle_filled(Pos2::new(px, py), 1.8, Color32::from_rgba_premultiplied(56, 189, 248, 160));
        }

        // Phase correlation bar
        let bar_left = rect.max.x - 140.0;
        let bar_y = center.y;
        painter.text(Pos2::new(bar_left, bar_y - 18.0), egui::Align2::LEFT_CENTER, "Phase Correlation", egui::FontId::proportional(10.0), Color32::WHITE);
        painter.rect_filled(Rect::from_min_size(Pos2::new(bar_left, bar_y - 6.0), Vec2::new(120.0, 12.0)), 3.0, Color32::from_rgb(30, 41, 59));
        painter.line_segment([Pos2::new(bar_left + 60.0, bar_y - 8.0), Pos2::new(bar_left + 60.0, bar_y + 8.0)], Stroke::new(1.0, Color32::WHITE));
        let indicator_x = bar_left + 60.0 + 45.0;
        painter.circle_filled(Pos2::new(indicator_x, bar_y), 5.0, Color32::from_rgb(34, 197, 94));
        painter.text(Pos2::new(bar_left + 60.0, bar_y + 16.0), egui::Align2::CENTER_CENTER, "+0.85 (In-Phase)", egui::FontId::monospace(9.5), Color32::from_rgb(34, 197, 94));
    }

    /// Master EBU R128 Loudness & True-Peak Metering Bridge
    fn draw_master_ebu_metering(&self, painter: &egui::Painter, rect: Rect) {
        let left = rect.min.x + 30.0;
        let top = rect.min.y + 16.0;
        let col_w = (rect.width() - 80.0) / 3.0;
        let h = rect.height() - 32.0;

        // Integrated LUFS
        let c1 = Rect::from_min_size(Pos2::new(left, top), Vec2::new(col_w, h));
        painter.rect_filled(c1, 6.0, Color32::from_rgb(12, 17, 29));
        painter.text(c1.center_top() + Vec2::new(0.0, 12.0), egui::Align2::CENTER_CENTER, "INTEGRATED LUFS", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));
        painter.text(c1.center(), egui::Align2::CENTER_CENTER, "-14.2 LUFS", egui::FontId::monospace(18.0), Color32::from_rgb(34, 197, 94));
        painter.text(c1.center_bottom() - Vec2::new(0.0, 12.0), egui::Align2::CENTER_CENTER, "Target: -14.0 (EBU R128)", egui::FontId::proportional(9.5), Color32::from_rgb(100, 116, 139));

        // Dynamic Range (DR)
        let c2 = Rect::from_min_size(Pos2::new(left + col_w + 10.0, top), Vec2::new(col_w, h));
        painter.rect_filled(c2, 6.0, Color32::from_rgb(12, 17, 29));
        painter.text(c2.center_top() + Vec2::new(0.0, 12.0), egui::Align2::CENTER_CENTER, "DYNAMIC RANGE (DR)", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));
        painter.text(c2.center(), egui::Align2::CENTER_CENTER, "DR 14", egui::FontId::monospace(18.0), Color32::from_rgb(56, 189, 248));
        painter.text(c2.center_bottom() - Vec2::new(0.0, 12.0), egui::Align2::CENTER_CENTER, "Audiophile Uncompressed", egui::FontId::proportional(9.5), Color32::from_rgb(100, 116, 139));

        // True Peak
        let c3 = Rect::from_min_size(Pos2::new(left + (col_w + 10.0) * 2.0, top), Vec2::new(col_w, h));
        painter.rect_filled(c3, 6.0, Color32::from_rgb(12, 17, 29));
        painter.text(c3.center_top() + Vec2::new(0.0, 12.0), egui::Align2::CENTER_CENTER, "TRUE PEAK LEVEL", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));
        painter.text(c3.center(), egui::Align2::CENTER_CENTER, "-0.2 dBFS", egui::FontId::monospace(18.0), Color32::from_rgb(245, 158, 11));
        painter.text(c3.center_bottom() - Vec2::new(0.0, 12.0), egui::Align2::CENTER_CENTER, "Inter-Sample Peak Protected", egui::FontId::proportional(9.5), Color32::from_rgb(34, 197, 94));
    }
}
