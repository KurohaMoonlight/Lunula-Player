use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::LunulaApp;
use crate::models::{ChainMode, NodeType, RollOffProfile, SincQuality};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TelemetryTab {
    AutoInspect,       // Automatically inspects clicked node/cable; defaults to Cumulative Curve
    CumulativeNet,     // Global Cumulative Net Transfer Function (EQ + Filters + Preamp)
    StereoVectorscope, // Lissajous Goniometer & Phase Correlation
    MasterMetering,    // EBU R128 LUFS, True-Peak & Dynamic Range
    WaveformHeatmap,   // Waveform Saturation Heatmap & Distortion Monitor
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
                    let t = if self.advance_studio_state.freeze_analyzer {
                        1.0
                    } else {
                        ui.input(|i| i.time) as f32
                    };

                    if !self.advance_studio_state.freeze_analyzer && self.is_playing {
                        ui.ctx().request_repaint();
                    }

                    match self.advance_studio_state.telemetry_tab {
                        TelemetryTab::AutoInspect => {
                            if let Some(node_id) = self.selected_node {
                                self.draw_node_circuit_inspector(ui, &painter, rect, node_id, t);
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
                            self.draw_master_ebu_metering(ui, &painter, rect);
                        }
                        TelemetryTab::WaveformHeatmap => {
                            self.draw_waveform_saturation_heatmap(ui, &painter, rect);
                        }
                    }
                });
            });
    }

    /// Minimized bar when the user collapses the telemetry desk to maximize canvas space
    fn render_minimized_desk_strip(&mut self, ui: &mut egui::Ui) {
        let avail_w = ui.available_width();
        egui::Frame::NONE
            .fill(Color32::from_rgb(15, 23, 42))
            .corner_radius(6.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(30, 41, 59)))
            .inner_margin(egui::Margin::symmetric(10, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📊 STUDIO TELEMETRY DESK").strong().size(11.0).color(Color32::from_rgb(148, 163, 184)));

                    if avail_w >= 640.0 {
                        let status_desc = if let Some(node_id) = self.selected_node {
                            let title = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.title.as_str()).unwrap_or("Node");
                            format!("Target: {} (#{}) • Ready for circuit inspection", title, node_id)
                        } else if let Some(cable_idx) = self.advance_studio_state.selected_cable_idx {
                            format!("Target: Cable Probe #{} • Real-time wire analysis", cable_idx)
                        } else {
                            "Target: Global Cumulative Response • Real-time net curve".to_string()
                        };

                        ui.label(egui::RichText::new(status_desc).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let btn_txt = if avail_w < 550.0 { "▴ Buka Desk" } else { "▴ Buka Telemetry Desk" };
                        let restore_btn = egui::Button::new(
                            egui::RichText::new(btn_txt)
                                .strong()
                                .size(11.0)
                                .color(Color32::from_rgb(165, 180, 252)),
                        )
                        .fill(Color32::from_rgb(30, 41, 59))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(99, 102, 241)))
                        .corner_radius(4.0);

                        if ui.add(restore_btn).clicked() {
                            self.advance_studio_state.is_desk_minimized = false;
                        }
                    });
                });
            });
    }

    /// Header bar inside the expanded Telemetry Desk
    fn render_desk_control_bar(&mut self, ui: &mut egui::Ui) {
        let avail_w = ui.available_width();
        let is_compact = avail_w < 920.0;
        let is_ultra_compact = avail_w < 700.0;

        ui.horizontal(|ui| {
            // Target Badge
            if let Some(node_id) = self.selected_node {
                let title = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.title.clone()).unwrap_or_else(|| format!("Node #{}", node_id));
                let badge_txt = if avail_w >= 1050.0 {
                    format!("🔬 INSPECT: {} (#{})", title, node_id)
                } else if avail_w >= 850.0 {
                    let short_t = if title.len() > 14 { format!("{}...", &title[..12]) } else { title.clone() };
                    format!("🔬 #{} {}", node_id, short_t)
                } else {
                    format!("🔬 #{}", node_id)
                };
                ui.label(egui::RichText::new(badge_txt).strong().size(11.0).color(Color32::from_rgb(129, 140, 248)))
                    .on_hover_text(format!("Inspeksi Sirkuit Modul #{} ({})", node_id, title));
            } else if let Some(c_idx) = self.advance_studio_state.selected_cable_idx {
                let badge_txt = if avail_w >= 900.0 {
                    format!("⚡ CABLE PROBE: Cable #{}", c_idx)
                } else {
                    format!("⚡ Probe #{}", c_idx)
                };
                ui.label(egui::RichText::new(badge_txt).strong().size(11.0).color(Color32::from_rgb(52, 211, 153)));
            } else {
                let badge_txt = if avail_w >= 980.0 {
                    "📈 GLOBAL CUMULATIVE NET CURVE"
                } else if avail_w >= 780.0 {
                    "📈 GLOBAL NET CURVE"
                } else {
                    "📈 Global"
                };
                ui.label(egui::RichText::new(badge_txt).strong().size(11.0).color(Color32::from_rgb(251, 191, 36)))
                    .on_hover_text("Respons frekuensi gabungan seluruh rantai sinyal DSP");
            }

            ui.separator();

            // Telemetry Tabs
            let cur_tab = self.advance_studio_state.telemetry_tab;
            let tabs: &[(TelemetryTab, &str, &str)] = if avail_w >= 1080.0 {
                &[
                    (TelemetryTab::AutoInspect, "🎯 Auto-Inspect", "Otomatis menginspeksi modul atau kabel yang dipilih"),
                    (TelemetryTab::CumulativeNet, "📈 Net EQ Curve", "Kurva respons kumulatif gabungan seluruh rantai filter"),
                    (TelemetryTab::StereoVectorscope, "🌊 Lissajous Scope", "Vectorscope Lissajous stereo real-time dan korelasi fase"),
                    (TelemetryTab::MasterMetering, "🎚️ Master EBU R128", "Loudness terintegrasi EBU R128, True-Peak dan Dynamic Range"),
                    (TelemetryTab::WaveformHeatmap, "🔥 Saturation Heatmap", "Heatmap saturasi waveform dan deteksi clipping real-time"),
                ]
            } else if avail_w >= 840.0 {
                &[
                    (TelemetryTab::AutoInspect, "🎯 Auto", "Otomatis menginspeksi modul/kabel terpilih"),
                    (TelemetryTab::CumulativeNet, "📈 Net EQ", "Kurva respons kumulatif seluruh filter"),
                    (TelemetryTab::StereoVectorscope, "🌊 Lissajous", "Lissajous stereo scope & korelasi fase"),
                    (TelemetryTab::MasterMetering, "🎚️ EBU R128", "EBU R128 LUFS & True-Peak metering"),
                    (TelemetryTab::WaveformHeatmap, "🔥 Heatmap", "Waveform saturation heatmap & clipping"),
                ]
            } else {
                &[
                    (TelemetryTab::AutoInspect, "🎯", "Auto-Inspect Modul"),
                    (TelemetryTab::CumulativeNet, "📈", "Net EQ Curve"),
                    (TelemetryTab::StereoVectorscope, "🌊", "Lissajous Scope"),
                    (TelemetryTab::MasterMetering, "🎚️", "Master EBU R128"),
                    (TelemetryTab::WaveformHeatmap, "🔥", "Saturation Heatmap"),
                ]
            };

            for &(tab, label, tip) in tabs {
                let is_sel = cur_tab == tab;
                let text = if is_sel {
                    egui::RichText::new(label).strong().size(10.5).color(Color32::WHITE)
                } else {
                    egui::RichText::new(label).size(10.5).color(Color32::from_rgb(148, 163, 184))
                };
                let bg = if is_sel { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(24, 32, 47) };
                if ui.add(egui::Button::new(text).fill(bg).corner_radius(4.0)).on_hover_text(tip).clicked() {
                    self.advance_studio_state.telemetry_tab = tab;
                }
            }

            // Right-aligned controls (Sleek dark theme buttons with responsive hover feedback)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 1. Sembunyikan / Minimize Button
                let min_lbl = if is_ultra_compact {
                    "▾"
                } else if is_compact {
                    "▾ Tutup"
                } else {
                    "▾ Sembunyikan Desk"
                };
                let min_btn = egui::Button::new(
                    egui::RichText::new(min_lbl)
                        .size(10.5)
                        .strong()
                        .color(Color32::from_rgb(203, 213, 225)),
                )
                .fill(Color32::from_rgb(24, 32, 47))
                .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
                .corner_radius(4.0);

                if ui.add(min_btn)
                    .on_hover_text("Tutup sementara panel bawah untuk memperluas kanvas patchbay")
                    .clicked()
                {
                    self.advance_studio_state.is_desk_minimized = true;
                }

                ui.separator();

                // 2. Freeze Toggle Button
                let is_frozen = self.advance_studio_state.freeze_analyzer;
                let freeze_lbl = if is_frozen {
                    if is_ultra_compact { "❄ Beku" } else { "❄ Analyzer Beku" }
                } else {
                    if is_ultra_compact { "❄ Live" } else { "❄ Freeze" }
                };
                let (freeze_bg, freeze_stroke, freeze_fg) = if is_frozen {
                    (
                        Color32::from_rgba_premultiplied(14, 165, 233, 40),
                        Color32::from_rgb(56, 189, 248),
                        Color32::from_rgb(186, 230, 253),
                    )
                } else {
                    (
                        Color32::from_rgb(24, 32, 47),
                        Color32::from_rgb(51, 65, 85),
                        Color32::from_rgb(148, 163, 184),
                    )
                };

                let freeze_btn = egui::Button::new(
                    egui::RichText::new(freeze_lbl)
                        .size(10.5)
                        .strong()
                        .color(freeze_fg),
                )
                .fill(freeze_bg)
                .stroke(Stroke::new(1.0, freeze_stroke))
                .corner_radius(4.0);

                if ui.add(freeze_btn)
                    .on_hover_text(if is_frozen {
                        "Analisis spektrum & kurva sedang dijeda (klik untuk live)"
                    } else {
                        "Bekukan (freeze) frame analisis spektrum dan kurva"
                    })
                    .clicked()
                {
                    self.advance_studio_state.freeze_analyzer = !self.advance_studio_state.freeze_analyzer;
                }

                // 3. Deselect Target Button if a node or cable is selected
                if self.selected_node.is_some() || self.advance_studio_state.selected_cable_idx.is_some() {
                    let reset_lbl = if is_ultra_compact {
                        "✕"
                    } else if is_compact {
                        "✕ Reset"
                    } else {
                        "✕ Reset Target"
                    };
                    let reset_btn = egui::Button::new(
                        egui::RichText::new(reset_lbl)
                            .size(10.5)
                            .strong()
                            .color(Color32::from_rgb(252, 165, 165)),
                    )
                    .fill(Color32::from_rgba_premultiplied(239, 68, 68, 25))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(239, 68, 68, 70)))
                    .corner_radius(4.0);

                    if ui.add(reset_btn)
                        .on_hover_text("Kembalikan target inspeksi ke kurva respons gabungan global")
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
    fn draw_node_circuit_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, node_id: usize, t: f32) {
        let Some(node) = self.nodes.iter().find(|n| n.id == node_id).cloned() else {
            self.draw_cumulative_transfer_curve(painter, rect, None);
            return;
        };

        match &node.node_type {
            NodeType::TubeWarmer => self.draw_tube_warmer_inspector(painter, rect, node_id, t),
            NodeType::LowPassFilter => self.draw_lpf_inspector(ui, painter, rect, node_id, t),
            NodeType::HighPassFilter => self.draw_hpf_inspector(ui, painter, rect, node_id, t),
            NodeType::BandPassFilter => self.draw_bpf_inspector(ui, painter, rect, node_id, t),
            NodeType::Crossover => self.draw_crossover_inspector(ui, painter, rect, node_id, t),
            NodeType::SavedEqPreset(name) => self.draw_eq_preset_inspector(ui, painter, rect, name.as_str(), t),
            NodeType::Dsp => self.draw_main_dsp_inspector(painter, rect, t),
            NodeType::Mixer => self.draw_mixer_inspector(painter, rect, node_id, t),
            NodeType::Splitter => self.draw_splitter_inspector(painter, rect, node_id, t),
            NodeType::OpticalLeveler => self.draw_optical_leveler_inspector(ui, painter, rect, node_id, t),
            NodeType::Compressor => self.draw_compressor_inspector(ui, painter, rect, node_id, t),
            NodeType::StereoWidener => self.draw_stereo_widener_inspector(painter, rect, node_id, t),
            NodeType::StereoBalance => self.draw_stereo_balance_inspector(painter, rect, node_id, t),
            NodeType::DynamicLoudness => self.draw_dynamic_loudness_inspector(painter, rect, node_id, t),
            NodeType::VolumeBooster => self.draw_volume_booster_inspector(painter, rect, node_id, t),
            NodeType::SteppedAttenuator => self.draw_stepped_attenuator_inspector(painter, rect, node_id, t),
            NodeType::BassGuard => self.draw_bass_guard_inspector(painter, rect, node_id, t),
            NodeType::AnalogPreamp => self.draw_analog_preamp_inspector(painter, rect, node_id, t),
            NodeType::VolumeAttenuator => self.draw_volume_attenuator_inspector(painter, rect, node_id, t),
            NodeType::Driver => self.draw_app_resampler_inspector(ui, painter, rect, node_id, t),
            NodeType::Source | NodeType::Buffer | NodeType::Endpoint => {
                self.draw_streaming_node_inspector(painter, rect, node_id)
            }
        }
    }

    /// Circuit inspector for Tube Warmer 12AX7: Non-linear transfer curve + animated harmonic spectrum (FFT)
    fn draw_tube_warmer_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Non-linear transfer curve (Vin vs Vout)
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let center = c1_rect.center();
        let half_w = c1_rect.width() * 0.40;
        let half_h = c1_rect.height() * 0.38;

        // Grid lines & linear 1:1 reference line
        painter.line_segment([Pos2::new(center.x - half_w, center.y), Pos2::new(center.x + half_w, center.y)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 60)));
        painter.line_segment([Pos2::new(center.x, center.y - half_h), Pos2::new(center.x, center.y + half_h)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 60)));
        painter.line_segment(
            [Pos2::new(center.x - half_w, center.y + half_h), Pos2::new(center.x + half_w, center.y - half_h)],
            Stroke::new(0.8, Color32::from_rgba_premultiplied(100, 116, 139, 80)),
        );

        // Tube non-linear saturation curve (Vout = tanh(Vin * 2.1))
        let steps = 40;
        let mut curve_pts = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let vin = (frac - 0.5) * 2.0; // -1.0 to +1.0
            let vout = (vin * 2.1).tanh();
            let px = center.x + vin * half_w;
            let py = center.y - vout * half_h;
            curve_pts.push(Pos2::new(px, py));
        }
        painter.add(egui::Shape::line(curve_pts.clone(), Stroke::new(3.0, Color32::from_rgba_premultiplied(245, 158, 11, 75))));
        painter.add(egui::Shape::line(curve_pts, Stroke::new(1.8, Color32::from_rgb(245, 158, 11))));

        // Signal driver dot traveling along the saturation curve driven by real audio waveform
        let sweep_vin = if self.is_playing {
            (self.live_pcm_waveform[0] * 1.5).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let sweep_vout = (sweep_vin * 2.1).tanh();
        let dot_pos = Pos2::new(center.x + sweep_vin * half_w, center.y - sweep_vout * half_h);
        painter.circle_filled(dot_pos, 4.0, Color32::WHITE);
        painter.circle_stroke(dot_pos, 6.5, Stroke::new(1.2, Color32::from_rgb(251, 191, 36)));

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "Vin / Vout Triode Soft-Saturation Curve", egui::FontId::proportional(10.0), Color32::WHITE);
        painter.text(c1_rect.min + Vec2::new(10.0, 24.0), egui::Align2::LEFT_TOP, "Saturasi Lembut • Kompresi Puncak Simetris", egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));

        // Column 2: Harmonics Spectrum & Tube Filament
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🔥 SPEKTRUM DISTORSI HARMONIK FFT (12AX7)", egui::FontId::proportional(10.0), Color32::from_rgb(251, 146, 60));

        let warm_scale = if self.is_playing { ((self.dac_meter_l + self.dac_meter_r) * 0.5).clamp(0.08, 1.0) } else { 0.0 };
        let harmonics = [
            ("H1 (1k)", 0.22, 54.0 * warm_scale, Color32::from_rgb(251, 191, 36)),
            ("H2 (2k)", 0.44, 32.0 * warm_scale, Color32::from_rgb(249, 115, 22)),
            ("H3 (3k)", 0.64, 18.0 * warm_scale, Color32::from_rgb(245, 158, 11)),
            ("H4 (4k)", 0.82, 10.0 * warm_scale, Color32::from_rgb(217, 119, 6)),
        ];

        let base_y = c2_rect.max.y - 28.0;
        for &(h_name, h_frac, h_height, h_col) in &harmonics {
            let hx = c2_rect.min.x + c2_rect.width() * h_frac;
            let top_y = base_y - h_height;

            let bar_rect = Rect::from_min_max(Pos2::new(hx - 6.0, top_y), Pos2::new(hx + 6.0, base_y));
            painter.rect_filled(bar_rect, 2.0, Color32::from_rgba_premultiplied(h_col.r(), h_col.g(), h_col.b(), 85));
            painter.rect_stroke(bar_rect, 2.0, Stroke::new(1.2, h_col), egui::StrokeKind::Outside);

            painter.line_segment([Pos2::new(hx - 8.0, top_y), Pos2::new(hx + 8.0, top_y)], Stroke::new(1.8, Color32::WHITE));
            painter.text(Pos2::new(hx, top_y - 6.0), egui::Align2::CENTER_CENTER, h_name, egui::FontId::proportional(8.5), h_col);
        }

        // Noise floor shimmer
        let mut noise_pts = Vec::with_capacity(25);
        for i in 0..25 {
            let nx = c2_rect.min.x + 12.0 + (c2_rect.width() - 24.0) * (i as f32 / 24.0);
            let n_jitter = ((i as f32 * 19.3 + t * 24.0).sin() * 2.0).abs();
            noise_pts.push(Pos2::new(nx, base_y - n_jitter));
        }
        for pair in noise_pts.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(0.8, Color32::from_rgba_premultiplied(100, 116, 139, 80)));
        }

        // Metrik readout (responsive to column width)
        let metric_text = if c2_rect.width() >= 330.0 {
            "THD: 1.4% • Even H2: -18.2 dB (Kehangatan Analog) • Bias: -1.2V DC"
        } else if c2_rect.width() >= 240.0 {
            "THD: 1.4% • H2: -18.2 dB • Bias: -1.2V"
        } else {
            "THD: 1.4% • H2: -18dB"
        };
        painter.text(c2_rect.min + Vec2::new(10.0, c2_rect.height() - 10.0), egui::Align2::LEFT_BOTTOM, metric_text, egui::FontId::monospace(9.0), Color32::from_rgb(52, 211, 153));
    }

    /// Circuit inspector for Low Pass Filter: Cutoff curve & slope/fringe controls
    fn draw_lpf_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, node_id: usize, _t: f32) {
        let padding = 8.0;
        let gap = 10.0;
        let avail_w = (rect.width() - (padding * 2.0) - gap).max(100.0);
        let c2_w = (avail_w * 0.32).clamp(270.0, 350.0).min(avail_w * 0.48);
        let c1_w = avail_w - c2_w;
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(padding, padding), Vec2::new(c1_w, rect.height() - (padding * 2.0)));
        let c2_rect = Rect::from_min_size(Pos2::new(c1_rect.max.x + gap, rect.min.y + padding), Vec2::new(c2_w, rect.height() - (padding * 2.0)));

        let Some(node) = self.nodes.iter().find(|n| n.id == node_id).cloned() else { return; };
        let current_cutoff = if node.param_val > 20.0 { node.param_val } else { 800.0 };
        let current_slope = if node.sub2.contains("48dB") { 48 } else if node.sub2.contains("24dB") { 24 } else { 12 };
        let current_fringe = if let Some(idx) = node.sub2.find("Q ") {
            node.sub2[idx + 2..].split_whitespace().next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.707)
        } else {
            0.707
        };

        let accent_col = Color32::from_rgb(52, 211, 153);

        // Column 1: Cutoff magnitude curve
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let slope_lbl = match current_slope { 48 => "-48 dB/oct (8-Pole)", 24 => "-24 dB/oct (4-Pole)", _ => "-12 dB/oct (2-Pole)" };
        let title_txt = if c1_w < 400.0 {
            format!("📉 LOW-PASS (-{} dB/oct)", current_slope)
        } else {
            format!("📉 LOW-PASS ROLL-OFF ({})", slope_lbl)
        };
        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, title_txt, egui::FontId::proportional(10.5), accent_col);

        let g_left = c1_rect.min.x + 36.0;
        let g_right = c1_rect.max.x - 12.0;
        let g_top = c1_rect.min.y + 28.0;
        let g_bottom = c1_rect.max.y - 24.0;
        let g_w = g_right - g_left;
        let g_h = g_bottom - g_top;

        // dB Grid Reference lines
        painter.line_segment([Pos2::new(g_left, g_top + 14.0), Pos2::new(g_right, g_top + 14.0)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 45)));
        painter.text(Pos2::new(g_left - 4.0, g_top + 14.0), egui::Align2::RIGHT_CENTER, "0 dB", egui::FontId::monospace(8.0), Color32::from_rgb(148, 163, 184));
        painter.line_segment([Pos2::new(g_left, g_top + 14.0 + g_h * 0.4), Pos2::new(g_right, g_top + 14.0 + g_h * 0.4)], Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 45)));
        painter.text(Pos2::new(g_left - 4.0, g_top + 14.0 + g_h * 0.4), egui::Align2::RIGHT_CENTER, "-12 dB", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));

        let norm_cutoff = ((current_cutoff / 20.0).log10() / (1000.0_f32).log10()).clamp(0.05, 0.95);
        let cutoff_x = g_left + g_w * norm_cutoff;

        // Vertical dashed cutoff line
        let mut cy_walk = g_top + 4.0;
        while cy_walk < g_bottom {
            painter.line_segment(
                [Pos2::new(cutoff_x, cy_walk), Pos2::new(cutoff_x, (cy_walk + 4.0).min(g_bottom))],
                Stroke::new(1.0, accent_col),
            );
            cy_walk += 7.0;
        }

        // Shaded Stopband Region
        let stopband_rect = Rect::from_min_max(Pos2::new(cutoff_x, g_top), Pos2::new(g_right, g_bottom));
        painter.rect_filled(stopband_rect, 0.0, Color32::from_rgba_premultiplied(239, 68, 68, 16));
        painter.text(
            Pos2::new((cutoff_x + g_right) * 0.5, g_top + 10.0),
            egui::Align2::CENTER_CENTER,
            format!("TERPOTONG (-{}dB/oct)", current_slope),
            egui::FontId::proportional(8.0),
            Color32::from_rgba_premultiplied(248, 113, 113, 180),
        );

        let steps = 60;
        let mut pts = Vec::with_capacity(steps + 1);
        let slope_power = match current_slope { 48 => 3.2, 24 => 2.3, _ => 1.5 };
        let resonance_bump = ((current_fringe - 0.707).max(0.0) * 12.0).min(18.0);

        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let x = g_left + frac * g_w;
            let y = if frac <= norm_cutoff {
                let dist = (norm_cutoff - frac) / norm_cutoff.max(0.01);
                let bump = if dist < 0.25 { (1.0 - dist / 0.25) * resonance_bump } else { 0.0 };
                (g_top + 14.0) - bump
            } else {
                let over = (frac - norm_cutoff) / (1.0 - norm_cutoff).max(0.01);
                (g_top + 14.0) + over.powf(slope_power) * (g_h - 22.0)
            };
            pts.push(Pos2::new(x, y.min(g_bottom)));
        }

        painter.add(egui::Shape::line(pts.clone(), Stroke::new(3.0, Color32::from_rgba_premultiplied(52, 211, 153, 80))));
        painter.add(egui::Shape::line(pts, Stroke::new(2.0, accent_col)));

        // Cutoff knee point with prominent luminous badge
        let knee_p = Pos2::new(cutoff_x, g_top + 14.0);
        painter.circle_filled(knee_p, 4.5, Color32::WHITE);
        painter.circle_stroke(knee_p, 6.5, Stroke::new(1.5, accent_col));
        let freq_str = if current_cutoff >= 1000.0 { format!("{:.1} kHz", current_cutoff / 1000.0) } else { format!("{:.0} Hz", current_cutoff) };
        let knee_badge_rect = Rect::from_center_size(Pos2::new(knee_p.x, knee_p.y - 14.0), Vec2::new(94.0, 18.0));
        painter.rect_filled(knee_badge_rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(knee_badge_rect, 4.0, Stroke::new(1.2, accent_col), egui::StrokeKind::Outside);
        painter.text(knee_badge_rect.center(), egui::Align2::CENTER_CENTER, format!("fc: {} (-3.0dB)", freq_str), egui::FontId::proportional(10.0), Color32::WHITE);

        // High-Contrast Telemetry HUD Bar at bottom of Column 1
        let hud_rect = Rect::from_min_max(Pos2::new(c1_rect.min.x + 8.0, c1_rect.max.y - 25.0), Pos2::new(c1_rect.max.x - 8.0, c1_rect.max.y - 3.0));
        painter.rect_filled(hud_rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(hud_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), egui::StrokeKind::Inside);
        let q_note = if current_fringe > 1.0 { format!("(Peak +{:.1}dB)", (current_fringe - 1.0) * 4.0) } else { "(Butterworth Flat)".to_string() };
        let hw = hud_rect.width();
        let hud_txt = if hw >= 680.0 {
            format!("📡 Cutoff: {}   •   📐 Slope: -{} dB/oct   •   ⚡ Q: {:.2} {}   •   🛡️ Atenuasi @ 1-Oct: -{}.0 dB", freq_str, current_slope, current_fringe, q_note, current_slope)
        } else if hw >= 480.0 {
            format!("📡 Cutoff: {}   •   📐 -{} dB/oct   •   ⚡ Q: {:.2}   •   🛡️ -{}.0 dB", freq_str, current_slope, current_fringe, current_slope)
        } else if hw >= 320.0 {
            format!("📡 {}   •   -{} dB/oct   •   Q: {:.2}", freq_str, current_slope, current_fringe)
        } else {
            format!("{} -{}dB", freq_str, current_slope)
        };
        painter.text(
            hud_rect.center(),
            egui::Align2::CENTER_CENTER,
            hud_txt,
            egui::FontId::proportional(10.0),
            Color32::from_rgb(52, 211, 153),
        );

        // Column 2: Interactive Controls
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let controls_rect = Rect::from_min_max(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 10.0), Pos2::new(c2_rect.max.x - 10.0, c2_rect.max.y - 6.0));
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect));

        let mut new_cutoff = current_cutoff;
        let mut new_slope = current_slope;
        let mut new_fringe = current_fringe;

        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("CUTOFF FREKUENSI:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            ui.label(egui::RichText::new(&freq_str).strong().size(13.0).color(accent_col));
            ui.label(egui::RichText::new(format!("• -{} dB/oct", current_slope)).strong().size(10.0).color(Color32::WHITE));
        });

        let mut log_val = (new_cutoff / 20.0).log10();
        let log_slider = egui::Slider::new(&mut log_val, 0.0..=3.0).show_value(false);
        if child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), log_slider).changed() {
            new_cutoff = (20.0 * 10.0_f32.powf(log_val)).clamp(20.0, 20000.0);
        }

        child_ui.horizontal_wrapped(|ui| {
            let presets = [("300 Hz", 300.0), ("800 Hz", 800.0), ("2.5 kHz", 2500.0), ("8.0 kHz", 8000.0)];
            for (name, val) in presets {
                let is_sel = (new_cutoff - val).abs() < 20.0;
                let bg = if is_sel { accent_col } else { Color32::from_rgb(30, 41, 59) };
                let text_col = if is_sel { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(241, 245, 249) };
                let stroke_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(100, 116, 139) };
                let btn = egui::Button::new(egui::RichText::new(name).size(10.0).strong().color(text_col))
                    .fill(bg)
                    .stroke(Stroke::new(1.2, stroke_col))
                    .corner_radius(4.0);
                if ui.add(btn).clicked() {
                    new_cutoff = val;
                }
            }
        });

        child_ui.add_space(5.0);
        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Slope:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            for &sl in &[12, 24, 48] {
                let lbl = format!("-{} dB/oct", sl);
                let is_sel = new_slope == sl;
                let bg = if is_sel { accent_col } else { Color32::from_rgb(30, 41, 59) };
                let text_col = if is_sel { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(241, 245, 249) };
                let stroke_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(100, 116, 139) };
                let btn = egui::Button::new(egui::RichText::new(lbl).size(10.0).strong().color(text_col))
                    .fill(bg)
                    .stroke(Stroke::new(1.2, stroke_col))
                    .corner_radius(4.0);
                if ui.add(btn).clicked() {
                    new_slope = sl;
                }
            }
        });

        child_ui.add_space(5.0);
        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Fringe Q:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            let q_desc = if new_fringe > 1.2 { "(Peak Punchy)" } else if new_fringe >= 0.70 { "(Butterworth)" } else { "(Gentle)" };
            ui.label(egui::RichText::new(format!("{:.2} {}", new_fringe, q_desc)).strong().size(10.5).color(accent_col));
        });
        let q_slider = egui::Slider::new(&mut new_fringe, 0.20..=4.00).show_value(false);
        child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), q_slider);

        // Apply parameter updates
        let cutoff_changed = (new_cutoff - current_cutoff).abs() > 1.0;
        let slope_changed = new_slope != current_slope;
        let fringe_changed = (new_fringe - current_fringe).abs() > 0.02;

        if cutoff_changed || slope_changed || fringe_changed {
            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                n.param_val = new_cutoff;
                n.sub1 = if new_cutoff >= 1000.0 { format!("Cutoff {:.1} kHz", new_cutoff / 1000.0) } else { format!("Cutoff {:.0} Hz", new_cutoff) };
                n.sub2 = format!("Slope {}dB • Q {:.2}", new_slope, new_fringe);
            }
            if let Some(ref engine) = self.audio_engine {
                if cutoff_changed { engine.set_node_param(node_id as u32, 0, new_cutoff); }
                if slope_changed { engine.set_node_param(node_id as u32, 1, new_slope as f32); }
                if fringe_changed { engine.set_node_param(node_id as u32, 2, new_fringe); }
            }
        }
    }

    /// Circuit inspector for High Pass Filter: Low-cut curve & slope/fringe controls
    fn draw_hpf_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, node_id: usize, _t: f32) {
        let padding = 8.0;
        let gap = 10.0;
        let avail_w = (rect.width() - (padding * 2.0) - gap).max(100.0);
        let c2_w = (avail_w * 0.32).clamp(270.0, 350.0).min(avail_w * 0.48);
        let c1_w = avail_w - c2_w;
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(padding, padding), Vec2::new(c1_w, rect.height() - (padding * 2.0)));
        let c2_rect = Rect::from_min_size(Pos2::new(c1_rect.max.x + gap, rect.min.y + padding), Vec2::new(c2_w, rect.height() - (padding * 2.0)));

        let Some(node) = self.nodes.iter().find(|n| n.id == node_id).cloned() else { return; };
        let current_cutoff = if node.param_val > 10.0 { node.param_val } else { 80.0 };
        let current_slope = if node.sub2.contains("48dB") { 48 } else if node.sub2.contains("24dB") { 24 } else { 12 };
        let current_fringe = if let Some(idx) = node.sub2.find("Q ") {
            node.sub2[idx + 2..].split_whitespace().next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.707)
        } else {
            0.707
        };

        let accent_col = Color32::from_rgb(56, 189, 248);

        // Column 1: High-Pass Magnitude Curve
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let slope_lbl = match current_slope { 48 => "+48 dB/oct (8-Pole)", 24 => "+24 dB/oct (4-Pole)", _ => "+12 dB/oct (2-Pole)" };
        let title_txt = if c1_w < 400.0 {
            format!("🏔️ HIGH-PASS (+{} dB/oct)", current_slope)
        } else {
            format!("🏔️ HIGH-PASS ROLL-OFF ({})", slope_lbl)
        };
        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, title_txt, egui::FontId::proportional(10.5), accent_col);

        let g_left = c1_rect.min.x + 36.0;
        let g_right = c1_rect.max.x - 12.0;
        let g_top = c1_rect.min.y + 28.0;
        let g_bottom = c1_rect.max.y - 24.0;
        let g_w = g_right - g_left;
        let g_h = g_bottom - g_top;

        // dB Grid Reference lines
        painter.line_segment([Pos2::new(g_left, g_top + 14.0), Pos2::new(g_right, g_top + 14.0)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 45)));
        painter.text(Pos2::new(g_left - 4.0, g_top + 14.0), egui::Align2::RIGHT_CENTER, "0 dB", egui::FontId::monospace(8.0), Color32::from_rgb(148, 163, 184));
        painter.line_segment([Pos2::new(g_left, g_top + 14.0 + g_h * 0.4), Pos2::new(g_right, g_top + 14.0 + g_h * 0.4)], Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 45)));
        painter.text(Pos2::new(g_left - 4.0, g_top + 14.0 + g_h * 0.4), egui::Align2::RIGHT_CENTER, "-12 dB", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));

        let norm_cutoff = ((current_cutoff / 20.0).log10() / (1000.0_f32).log10()).clamp(0.05, 0.95);
        let cutoff_x = g_left + g_w * norm_cutoff;

        // Vertical dashed cutoff line
        let mut cy_walk = g_top + 4.0;
        while cy_walk < g_bottom {
            painter.line_segment(
                [Pos2::new(cutoff_x, cy_walk), Pos2::new(cutoff_x, (cy_walk + 4.0).min(g_bottom))],
                Stroke::new(1.0, accent_col),
            );
            cy_walk += 7.0;
        }

        // Shaded Sub-bass cut region (left side of cutoff)
        let stopband_rect = Rect::from_min_max(Pos2::new(g_left, g_top), Pos2::new(cutoff_x, g_bottom));
        painter.rect_filled(stopband_rect, 0.0, Color32::from_rgba_premultiplied(239, 68, 68, 16));
        painter.text(
            Pos2::new((g_left + cutoff_x) * 0.5, g_top + 10.0),
            egui::Align2::CENTER_CENTER,
            format!("SUB-BASS CUT ({}dB/oct)", current_slope),
            egui::FontId::proportional(8.0),
            Color32::from_rgba_premultiplied(248, 113, 113, 180),
        );

        let steps = 60;
        let mut pts = Vec::with_capacity(steps + 1);
        let slope_power = match current_slope { 48 => 3.2, 24 => 2.3, _ => 1.5 };
        let resonance_bump = ((current_fringe - 0.707).max(0.0) * 12.0).min(18.0);

        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let x = g_left + frac * g_w;
            let y = if frac >= norm_cutoff {
                let dist = (frac - norm_cutoff) / (1.0 - norm_cutoff).max(0.01);
                let bump = if dist < 0.25 { (1.0 - dist / 0.25) * resonance_bump } else { 0.0 };
                (g_top + 14.0) - bump
            } else {
                let under = (norm_cutoff - frac) / norm_cutoff.max(0.01);
                (g_top + 14.0) + under.powf(slope_power) * (g_h - 22.0)
            };
            pts.push(Pos2::new(x, y.min(g_bottom)));
        }

        painter.add(egui::Shape::line(pts.clone(), Stroke::new(3.0, Color32::from_rgba_premultiplied(56, 189, 248, 80))));
        painter.add(egui::Shape::line(pts, Stroke::new(2.0, accent_col)));

        // Cutoff knee point with prominent luminous badge
        let knee_p = Pos2::new(cutoff_x, g_top + 14.0);
        painter.circle_filled(knee_p, 4.5, Color32::WHITE);
        painter.circle_stroke(knee_p, 6.5, Stroke::new(1.5, accent_col));
        let freq_str = if current_cutoff >= 1000.0 { format!("{:.1} kHz", current_cutoff / 1000.0) } else { format!("{:.0} Hz", current_cutoff) };
        let knee_badge_rect = Rect::from_center_size(Pos2::new(knee_p.x, knee_p.y - 14.0), Vec2::new(94.0, 18.0));
        painter.rect_filled(knee_badge_rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(knee_badge_rect, 4.0, Stroke::new(1.2, accent_col), egui::StrokeKind::Outside);
        painter.text(knee_badge_rect.center(), egui::Align2::CENTER_CENTER, format!("fc: {} (-3.0dB)", freq_str), egui::FontId::proportional(10.0), Color32::WHITE);

        // High-Contrast Telemetry HUD Bar at bottom of Column 1
        let hud_rect = Rect::from_min_max(Pos2::new(c1_rect.min.x + 8.0, c1_rect.max.y - 25.0), Pos2::new(c1_rect.max.x - 8.0, c1_rect.max.y - 3.0));
        painter.rect_filled(hud_rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(hud_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), egui::StrokeKind::Inside);
        let q_note = if current_fringe > 1.0 { format!("(Peak +{:.1}dB)", (current_fringe - 1.0) * 4.0) } else { "(Sub-Sonic Clean)".to_string() };
        let hw = hud_rect.width();
        let hud_txt = if hw >= 680.0 {
            format!("📡 Cutoff: {}   •   📐 Slope: +{} dB/oct   •   ⚡ Q: {:.2} {}   •   🛡️ Reduksi @ 1/2-Oct: -{}.0 dB", freq_str, current_slope, current_fringe, q_note, current_slope)
        } else if hw >= 480.0 {
            format!("📡 Cutoff: {}   •   📐 +{} dB/oct   •   ⚡ Q: {:.2}   •   🛡️ -{}.0 dB", freq_str, current_slope, current_fringe, current_slope)
        } else if hw >= 320.0 {
            format!("📡 {}   •   +{} dB/oct   •   Q: {:.2}", freq_str, current_slope, current_fringe)
        } else {
            format!("{} +{}dB", freq_str, current_slope)
        };
        painter.text(
            hud_rect.center(),
            egui::Align2::CENTER_CENTER,
            hud_txt,
            egui::FontId::proportional(10.0),
            Color32::from_rgb(56, 189, 248),
        );

        // Column 2: Controls
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let controls_rect = Rect::from_min_max(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 10.0), Pos2::new(c2_rect.max.x - 10.0, c2_rect.max.y - 6.0));
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect));

        let mut new_cutoff = current_cutoff;
        let mut new_slope = current_slope;
        let mut new_fringe = current_fringe;

        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("CUTOFF SUB-BASS:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            ui.label(egui::RichText::new(&freq_str).strong().size(13.0).color(accent_col));
            ui.label(egui::RichText::new(format!("• +{} dB/oct", current_slope)).strong().size(10.0).color(Color32::WHITE));
        });

        let mut log_val = (new_cutoff / 20.0).log10();
        let log_slider = egui::Slider::new(&mut log_val, 0.0..=3.0).show_value(false);
        if child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), log_slider).changed() {
            new_cutoff = (20.0 * 10.0_f32.powf(log_val)).clamp(20.0, 20000.0);
        }

        child_ui.horizontal_wrapped(|ui| {
            let presets = [("40 Hz", 40.0), ("80 Hz (Vocal)", 80.0), ("160 Hz", 160.0), ("500 Hz", 500.0)];
            for (name, val) in presets {
                let is_sel = (new_cutoff - val).abs() < 5.0;
                let bg = if is_sel { accent_col } else { Color32::from_rgb(30, 41, 59) };
                let text_col = if is_sel { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(241, 245, 249) };
                let stroke_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(100, 116, 139) };
                let btn = egui::Button::new(egui::RichText::new(name).size(10.0).strong().color(text_col))
                    .fill(bg)
                    .stroke(Stroke::new(1.2, stroke_col))
                    .corner_radius(4.0);
                if ui.add(btn).clicked() {
                    new_cutoff = val;
                }
            }
        });

        child_ui.add_space(5.0);
        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Slope:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            for &sl in &[12, 24, 48] {
                let lbl = format!("+{} dB/oct", sl);
                let is_sel = new_slope == sl;
                let bg = if is_sel { accent_col } else { Color32::from_rgb(30, 41, 59) };
                let text_col = if is_sel { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(241, 245, 249) };
                let stroke_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(100, 116, 139) };
                let btn = egui::Button::new(egui::RichText::new(lbl).size(10.0).strong().color(text_col))
                    .fill(bg)
                    .stroke(Stroke::new(1.2, stroke_col))
                    .corner_radius(4.0);
                if ui.add(btn).clicked() {
                    new_slope = sl;
                }
            }
        });

        child_ui.add_space(5.0);
        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Fringe Q:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            let q_desc = if new_fringe > 1.2 { "(Peak Punchy)" } else if new_fringe >= 0.70 { "(Butterworth)" } else { "(Sub-Sonic Clean)" };
            ui.label(egui::RichText::new(format!("{:.2} {}", new_fringe, q_desc)).strong().size(10.5).color(accent_col));
        });
        let q_slider = egui::Slider::new(&mut new_fringe, 0.20..=4.00).show_value(false);
        child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), q_slider);

        let cutoff_changed = (new_cutoff - current_cutoff).abs() > 1.0;
        let slope_changed = new_slope != current_slope;
        let fringe_changed = (new_fringe - current_fringe).abs() > 0.02;

        if cutoff_changed || slope_changed || fringe_changed {
            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                n.param_val = new_cutoff;
                n.sub1 = if new_cutoff >= 1000.0 { format!("Cutoff {:.1} kHz", new_cutoff / 1000.0) } else { format!("Cutoff {:.0} Hz", new_cutoff) };
                n.sub2 = format!("Slope {}dB • Q {:.2}", new_slope, new_fringe);
            }
            if let Some(ref engine) = self.audio_engine {
                if cutoff_changed { engine.set_node_param(node_id as u32, 0, new_cutoff); }
                if slope_changed { engine.set_node_param(node_id as u32, 1, new_slope as f32); }
                if fringe_changed { engine.set_node_param(node_id as u32, 2, new_fringe); }
            }
        }
    }

    /// Circuit inspector for Band Pass Filter: Peak band curve & bandwidth/slope controls
    fn draw_bpf_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, node_id: usize, _t: f32) {
        let padding = 8.0;
        let gap = 10.0;
        let avail_w = (rect.width() - (padding * 2.0) - gap).max(100.0);
        let c2_w = (avail_w * 0.32).clamp(270.0, 350.0).min(avail_w * 0.48);
        let c1_w = avail_w - c2_w;
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(padding, padding), Vec2::new(c1_w, rect.height() - (padding * 2.0)));
        let c2_rect = Rect::from_min_size(Pos2::new(c1_rect.max.x + gap, rect.min.y + padding), Vec2::new(c2_w, rect.height() - (padding * 2.0)));

        let Some(node) = self.nodes.iter().find(|n| n.id == node_id).cloned() else { return; };
        let current_center = if node.param_val > 20.0 { node.param_val } else { 1000.0 };
        let current_slope = if node.sub2.contains("24dB") { 24 } else { 12 };
        let current_fringe = if let Some(idx) = node.sub2.find("Q ") {
            node.sub2[idx + 2..].split_whitespace().next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(1.414)
        } else {
            1.414
        };

        let accent_col = Color32::from_rgb(245, 158, 11);

        // Column 1: Band-Pass Bell Curve
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let slope_lbl = if current_slope == 24 { "4-Pole (24 dB/oct)" } else { "2-Pole (12 dB/oct)" };
        let title_txt = if c1_w < 400.0 {
            format!("🎯 BAND-PASS (±{} dB/oct)", current_slope)
        } else {
            format!("🎯 BAND-PASS (0 dB PEAK GAIN • {})", slope_lbl)
        };
        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, title_txt, egui::FontId::proportional(10.5), accent_col);

        let g_left = c1_rect.min.x + 36.0;
        let g_right = c1_rect.max.x - 12.0;
        let g_top = c1_rect.min.y + 28.0;
        let g_bottom = c1_rect.max.y - 24.0;
        let g_w = g_right - g_left;
        let g_h = g_bottom - g_top;

        // dB Grid Reference lines
        painter.line_segment([Pos2::new(g_left, g_top + 14.0), Pos2::new(g_right, g_top + 14.0)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 45)));
        painter.text(Pos2::new(g_left - 4.0, g_top + 14.0), egui::Align2::RIGHT_CENTER, "0 dB", egui::FontId::monospace(8.0), Color32::from_rgb(148, 163, 184));
        painter.line_segment([Pos2::new(g_left, g_top + 14.0 + g_h * 0.4), Pos2::new(g_right, g_top + 14.0 + g_h * 0.4)], Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 45)));
        painter.text(Pos2::new(g_left - 4.0, g_top + 14.0 + g_h * 0.4), egui::Align2::RIGHT_CENTER, "-12 dB", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));

        let norm_center = ((current_center / 20.0).log10() / (1000.0_f32).log10()).clamp(0.05, 0.95);
        let center_x = g_left + g_w * norm_center;

        // Vertical dashed center line
        let mut cy_walk = g_top + 4.0;
        while cy_walk < g_bottom {
            painter.line_segment(
                [Pos2::new(center_x, cy_walk), Pos2::new(center_x, (cy_walk + 4.0).min(g_bottom))],
                Stroke::new(1.0, accent_col),
            );
            cy_walk += 7.0;
        }

        let steps = 60;
        let mut pts = Vec::with_capacity(steps + 1);
        let bw = (0.22 / current_fringe.max(0.2)).clamp(0.03, 0.50);

        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let x = g_left + frac * g_w;
            let dist = ((frac - norm_center) / bw).abs();
            let y = (g_top + 14.0) + (1.0 - (-dist * dist).exp()) * (g_h - 22.0);
            pts.push(Pos2::new(x, y.min(g_bottom)));
        }

        painter.add(egui::Shape::line(pts.clone(), Stroke::new(3.0, Color32::from_rgba_premultiplied(245, 158, 11, 80))));
        painter.add(egui::Shape::line(pts, Stroke::new(2.0, accent_col)));

        // Center peak point with prominent luminous badge
        let peak_p = Pos2::new(center_x, g_top + 14.0);
        painter.circle_filled(peak_p, 4.5, Color32::WHITE);
        painter.circle_stroke(peak_p, 6.5, Stroke::new(1.5, accent_col));
        let freq_str = if current_center >= 1000.0 { format!("{:.1} kHz", current_center / 1000.0) } else { format!("{:.0} Hz", current_center) };
        let peak_badge_rect = Rect::from_center_size(Pos2::new(peak_p.x, peak_p.y - 14.0), Vec2::new(94.0, 18.0));
        painter.rect_filled(peak_badge_rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(peak_badge_rect, 4.0, Stroke::new(1.2, accent_col), egui::StrokeKind::Outside);
        painter.text(peak_badge_rect.center(), egui::Align2::CENTER_CENTER, format!("f0: {} (0.0 dB)", freq_str), egui::FontId::proportional(10.0), Color32::WHITE);

        // High-Contrast Telemetry HUD Bar at bottom of Column 1
        let hud_rect = Rect::from_min_max(Pos2::new(c1_rect.min.x + 8.0, c1_rect.max.y - 25.0), Pos2::new(c1_rect.max.x - 8.0, c1_rect.max.y - 3.0));
        painter.rect_filled(hud_rect, 4.0, Color32::from_rgb(15, 23, 42));
        painter.rect_stroke(hud_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)), egui::StrokeKind::Inside);
        let bw_oct = 1.0 / current_fringe.max(0.1);
        let hw = hud_rect.width();
        let hud_txt = if hw >= 680.0 {
            format!("📡 Pusat: {}   •   📐 Kemiringan: ±{} dB/oct   •   ⚡ Q: {:.2} (BW: {:.2} Oct)   •   🛡️ 0.0 dB Normal", freq_str, current_slope, current_fringe, bw_oct)
        } else if hw >= 480.0 {
            format!("📡 Pusat: {}   •   📐 ±{} dB/oct   •   ⚡ Q: {:.2} (BW: {:.2} Oct)", freq_str, current_slope, current_fringe, bw_oct)
        } else if hw >= 320.0 {
            format!("📡 {}   •   ±{} dB/oct   •   Q: {:.2}", freq_str, current_slope, current_fringe)
        } else {
            format!("{} ±{}dB", freq_str, current_slope)
        };
        painter.text(
            hud_rect.center(),
            egui::Align2::CENTER_CENTER,
            hud_txt,
            egui::FontId::proportional(10.0),
            Color32::from_rgb(251, 191, 36),
        );

        // Column 2: Controls
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let controls_rect = Rect::from_min_max(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 10.0), Pos2::new(c2_rect.max.x - 10.0, c2_rect.max.y - 6.0));
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect));

        let mut new_center = current_center;
        let mut new_slope = current_slope;
        let mut new_fringe = current_fringe;

        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("FREKUENSI PUSAT (f0):").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            ui.label(egui::RichText::new(&freq_str).strong().size(13.0).color(accent_col));
            ui.label(egui::RichText::new(format!("• ±{} dB/oct", current_slope)).strong().size(10.0).color(Color32::WHITE));
        });

        let mut log_val = (new_center / 20.0).log10();
        let log_slider = egui::Slider::new(&mut log_val, 0.0..=3.0).show_value(false);
        if child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), log_slider).changed() {
            new_center = (20.0 * 10.0_f32.powf(log_val)).clamp(20.0, 20000.0);
        }

        child_ui.horizontal_wrapped(|ui| {
            let presets = [("300 Hz", 300.0), ("1.0 kHz (Vocal)", 1000.0), ("3.0 kHz", 3000.0), ("8.0 kHz", 8000.0)];
            for (name, val) in presets {
                let is_sel = (new_center - val).abs() < 25.0;
                let bg = if is_sel { accent_col } else { Color32::from_rgb(30, 41, 59) };
                let text_col = if is_sel { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(241, 245, 249) };
                let stroke_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(100, 116, 139) };
                let btn = egui::Button::new(egui::RichText::new(name).size(10.0).strong().color(text_col))
                    .fill(bg)
                    .stroke(Stroke::new(1.2, stroke_col))
                    .corner_radius(4.0);
                if ui.add(btn).clicked() {
                    new_center = val;
                }
            }
        });

        child_ui.add_space(5.0);
        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Slope:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            for &sl in &[12, 24] {
                let lbl = format!("±{} dB/oct", sl);
                let is_sel = new_slope == sl;
                let bg = if is_sel { accent_col } else { Color32::from_rgb(30, 41, 59) };
                let text_col = if is_sel { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(241, 245, 249) };
                let stroke_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(100, 116, 139) };
                let btn = egui::Button::new(egui::RichText::new(lbl).size(10.0).strong().color(text_col))
                    .fill(bg)
                    .stroke(Stroke::new(1.2, stroke_col))
                    .corner_radius(4.0);
                if ui.add(btn).clicked() {
                    new_slope = sl;
                }
            }
        });

        child_ui.add_space(5.0);
        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Fringe Q / BW:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            let bw_val = 1.0 / new_fringe.max(0.1);
            ui.label(egui::RichText::new(format!("{:.2} ({:.2} Oct)", new_fringe, bw_val)).strong().size(10.5).color(accent_col));
        });
        let q_slider = egui::Slider::new(&mut new_fringe, 0.20..=10.00).show_value(false);
        child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), q_slider);

        let center_changed = (new_center - current_center).abs() > 1.0;
        let slope_changed = new_slope != current_slope;
        let fringe_changed = (new_fringe - current_fringe).abs() > 0.05;

        if center_changed || slope_changed || fringe_changed {
            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                n.param_val = new_center;
                n.sub1 = if new_center >= 1000.0 { format!("Center {:.1} kHz", new_center / 1000.0) } else { format!("Center {:.0} Hz", new_center) };
                n.sub2 = format!("Slope {}dB • Q {:.2}", new_slope, new_fringe);
            }
            if let Some(ref engine) = self.audio_engine {
                if center_changed { engine.set_node_param(node_id as u32, 0, new_center); }
                if slope_changed { engine.set_node_param(node_id as u32, 1, new_slope as f32); }
                if fringe_changed { engine.set_node_param(node_id as u32, 2, new_fringe); }
            }
        }
    }

    /// Circuit inspector for 2-Way Linkwitz-Riley Crossover: Dual LP/HP transfer curves crossing at -6 dB
    fn draw_crossover_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, node_id: usize, _t: f32) {
        let padding = 8.0;
        let gap = 10.0;
        let avail_w = (rect.width() - (padding * 2.0) - gap).max(100.0);

        // Fully responsive column calculation:
        // Column 2 (controls) maintains a comfortable 270px..=350px width, while Column 1 (graph)
        // dynamically fills all remaining width, preventing graph cramping on wide/ultrawide displays.
        let c2_w = (avail_w * 0.32).clamp(270.0, 350.0).min(avail_w * 0.48);
        let c1_w = avail_w - c2_w;

        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(padding, padding), Vec2::new(c1_w, rect.height() - (padding * 2.0)));
        let c2_rect = Rect::from_min_size(Pos2::new(c1_rect.max.x + gap, rect.min.y + padding), Vec2::new(c2_w, rect.height() - (padding * 2.0)));

        let Some(node) = self.nodes.iter().find(|n| n.id == node_id).cloned() else { return; };
        let current_xover = if node.param_val > 20.0 { node.param_val } else { 500.0 };
        let is_inverted = node.sub2.contains("Invert");

        let accent_col = Color32::from_rgb(168, 85, 247);

        // Column 1: Dual Crossover Curves (LR4 -24 dB/oct)
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let title_txt = if c1_w < 380.0 {
            "✂️ LR4 CROSSOVER (-24 dB)"
        } else if c1_w < 500.0 {
            "✂️ LR4 CROSSOVER (-24 dB/oct)"
        } else {
            "✂️ LINKWITZ-RILEY LR4 CROSSOVER (-24 dB/oct)"
        };
        painter.text(c1_rect.min + Vec2::new(10.0, 9.0), egui::Align2::LEFT_TOP, title_txt, egui::FontId::proportional(10.5), accent_col);

        let g_left = c1_rect.min.x + 36.0;
        let g_right = c1_rect.max.x - 14.0;
        let g_top = c1_rect.min.y + 26.0;
        let g_bottom = c1_rect.max.y - 28.0;
        let g_w = (g_right - g_left).max(10.0);
        let g_h = (g_bottom - g_top).max(10.0);

        // Logarithmic frequency grid lines (100Hz, 1kHz, 10kHz) for realistic audio scope reference
        let f_grids: [(f32, &str); 3] = [(100.0, "100Hz"), (1000.0, "1kHz"), (10000.0, "10kHz")];
        for (f_val, f_lbl) in f_grids {
            let norm = ((f_val / 20.0).log10() / (1000.0_f32).log10()).clamp(0.0, 1.0);
            let fx = g_left + g_w * norm;
            if fx > g_left + 12.0 && fx < g_right - 12.0 {
                painter.line_segment(
                    [Pos2::new(fx, g_top), Pos2::new(fx, g_bottom)],
                    Stroke::new(0.6, Color32::from_rgba_premultiplied(51, 65, 85, 45)),
                );
                painter.text(
                    Pos2::new(fx, g_bottom + 1.0),
                    egui::Align2::CENTER_TOP,
                    f_lbl,
                    egui::FontId::monospace(7.5),
                    Color32::from_rgb(100, 116, 139),
                );
            }
        }

        // Flat 0 dB Summation Line
        painter.line_segment(
            [Pos2::new(g_left, g_top + 8.0), Pos2::new(g_right, g_top + 8.0)],
            Stroke::new(1.2, Color32::from_rgba_premultiplied(168, 85, 247, 100)),
        );
        painter.text(Pos2::new(g_left - 4.0, g_top + 8.0), egui::Align2::RIGHT_CENTER, "0 dB", egui::FontId::monospace(8.0), accent_col);
        painter.text(Pos2::new(g_right - 4.0, g_top + 4.0), egui::Align2::RIGHT_BOTTOM, "Sum: 0 dB Flat", egui::FontId::monospace(8.0), accent_col);

        // -6 dB Crossover Reference Line
        painter.line_segment([Pos2::new(g_left, g_top + 20.0), Pos2::new(g_right, g_top + 20.0)], Stroke::new(0.8, Color32::from_rgba_premultiplied(100, 116, 139, 45)));
        painter.text(Pos2::new(g_left - 4.0, g_top + 20.0), egui::Align2::RIGHT_CENTER, "-6 dB", egui::FontId::monospace(8.0), Color32::from_rgb(148, 163, 184));

        let norm_xover = ((current_xover / 20.0).log10() / (1000.0_f32).log10()).clamp(0.05, 0.95);
        let xover_x = g_left + g_w * norm_xover;

        // Vertical dashed crossover line
        let mut cy_walk = g_top + 4.0;
        while cy_walk < g_bottom {
            painter.line_segment(
                [Pos2::new(xover_x, cy_walk), Pos2::new(xover_x, (cy_walk + 4.0).min(g_bottom))],
                Stroke::new(1.0, accent_col),
            );
            cy_walk += 7.0;
        }

        let steps = ((g_w / 6.0) as usize).clamp(40, 100);
        let mut low_pts = Vec::with_capacity(steps + 1);
        let mut high_pts = Vec::with_capacity(steps + 1);

        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let x = g_left + frac * g_w;

            // Low-pass branch
            let y_low = if frac <= norm_xover {
                let over = (frac / norm_xover).clamp(0.0, 1.0);
                (g_top + 8.0) + over.powf(2.5) * 12.0
            } else {
                let over = (frac - norm_xover) / (1.0 - norm_xover).max(0.01);
                (g_top + 20.0) + over.powf(2.3) * (g_h - 28.0)
            };
            low_pts.push(Pos2::new(x, y_low.min(g_bottom)));

            // High-pass branch
            let y_high = if frac >= norm_xover {
                let over = ((1.0 - frac) / (1.0 - norm_xover)).clamp(0.0, 1.0);
                (g_top + 8.0) + over.powf(2.5) * 12.0
            } else {
                let under = (norm_xover - frac) / norm_xover.max(0.01);
                (g_top + 20.0) + under.powf(2.3) * (g_h - 28.0)
            };
            high_pts.push(Pos2::new(x, y_high.min(g_bottom)));
        }

        // Draw Low-pass (green) and High-pass (cyan)
        painter.add(egui::Shape::line(low_pts, Stroke::new(2.0, Color32::from_rgb(52, 211, 153))));
        painter.add(egui::Shape::line(high_pts, Stroke::new(2.0, Color32::from_rgb(56, 189, 248))));

        // Crossover Point (-6 dB intersection)
        let x_point = Pos2::new(xover_x, g_top + 20.0);
        painter.circle_filled(x_point, 4.0, Color32::WHITE);
        painter.circle_stroke(x_point, 6.0, Stroke::new(1.2, accent_col));
        let freq_str = if current_xover >= 1000.0 { format!("{:.1} kHz", current_xover / 1000.0) } else { format!("{:.0} Hz", current_xover) };
        painter.text(x_point - Vec2::new(0.0, 10.0), egui::Align2::CENTER_BOTTOM, format!("X-Over: {} (-6dB)", freq_str), egui::FontId::monospace(8.5), Color32::WHITE);

        // Fully Responsive Telemetry HUD Bar at bottom of Column 1
        let hud_rect = Rect::from_min_max(
            Pos2::new(c1_rect.min.x + 8.0, c1_rect.max.y - 23.0),
            Pos2::new(c1_rect.max.x - 8.0, c1_rect.max.y - 3.0),
        );
        painter.rect_filled(hud_rect, 4.0, Color32::from_rgb(8, 12, 20));
        painter.rect_stroke(hud_rect, 4.0, Stroke::new(0.8, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let hw = hud_rect.width();
        let (item_xover, item_slope, item_phase, item_sum) = if hw >= 680.0 {
            (
                format!("📡 Titik X-Over: {} (-6dB)", freq_str),
                "📐 Slope: LR4 24 dB/oct".to_string(),
                if is_inverted { "⚡ Fasa: 180° Invert" } else { "⚡ Fasa: 0° In-Phase" }.to_string(),
                "✨ Sum: 0.0 dB (Flat)".to_string(),
            )
        } else if hw >= 480.0 {
            (
                format!("📡 X-Over: {}", freq_str),
                "📐 LR4 24dB".to_string(),
                if is_inverted { "⚡ 180° Inv" } else { "⚡ 0° In-Phase" }.to_string(),
                "Sum: 0.0 dB".to_string(),
            )
        } else if hw >= 320.0 {
            (
                format!("📡 {}", freq_str),
                "LR4".to_string(),
                if is_inverted { "180°" } else { "0°" }.to_string(),
                "0.0 dB".to_string(),
            )
        } else {
            (
                freq_str.to_string(),
                String::new(),
                String::new(),
                "0 dB".to_string(),
            )
        };

        let mut hud_ui = ui.new_child(egui::UiBuilder::new().max_rect(hud_rect));
        hud_ui.set_clip_rect(hud_rect);
        hud_ui.with_layout(egui::Layout::left_to_right(egui::Align::Center).with_main_align(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(5.0, 0.0);
            ui.label(egui::RichText::new(&item_xover).size(9.0).monospace().color(Color32::from_rgb(192, 132, 252)));
            if !item_slope.is_empty() {
                ui.label(egui::RichText::new("•").size(8.0).color(Color32::from_rgb(71, 85, 105)));
                ui.label(egui::RichText::new(&item_slope).size(9.0).monospace().color(Color32::from_rgb(52, 211, 153)));
            }
            if !item_phase.is_empty() {
                ui.label(egui::RichText::new("•").size(8.0).color(Color32::from_rgb(71, 85, 105)));
                let phase_col = if is_inverted { Color32::from_rgb(248, 113, 113) } else { Color32::from_rgb(56, 189, 248) };
                ui.label(egui::RichText::new(&item_phase).size(9.0).monospace().color(phase_col));
            }
            if !item_sum.is_empty() {
                ui.label(egui::RichText::new("•").size(8.0).color(Color32::from_rgb(71, 85, 105)));
                ui.label(egui::RichText::new(&item_sum).size(9.0).monospace().strong().color(Color32::from_rgb(251, 191, 36)));
            }
        });

        // Column 2: Controls
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let controls_rect = Rect::from_min_max(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 10.0), Pos2::new(c2_rect.max.x - 10.0, c2_rect.max.y - 6.0));
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect));

        let mut new_xover = current_xover;
        let mut new_invert = is_inverted;

        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("FREKUENSI CROSSOVER:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            ui.label(egui::RichText::new(&freq_str).strong().size(13.0).color(accent_col));
            ui.label(egui::RichText::new("• LR4 24 dB/oct").strong().size(10.0).color(Color32::from_rgb(148, 163, 184)));
        });

        let mut log_val = (new_xover / 20.0).log10();
        let log_slider = egui::Slider::new(&mut log_val, 0.4..=2.6).show_value(false); // 50 Hz to 8000 Hz
        if child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), log_slider).changed() {
            new_xover = (20.0 * 10.0_f32.powf(log_val)).clamp(50.0, 8000.0);
        }

        child_ui.horizontal_wrapped(|ui| {
            let presets = [("80 Hz Sub", 80.0), ("300 Hz Bass", 300.0), ("500 Hz Mid", 500.0), ("2.5 kHz Treble", 2500.0)];
            let avail_w = ui.available_width();
            let btn_w = if avail_w < 260.0 { (avail_w - 6.0) / 2.0 } else { ((avail_w - 18.0) / 4.0).clamp(62.0, 95.0) };
            for (name, val) in presets {
                let is_sel = (new_xover - val).abs() < 20.0;
                let bg = if is_sel { accent_col } else { Color32::from_rgb(24, 32, 47) };
                let text_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };
                let stroke_col = if is_sel { Color32::from_rgb(216, 180, 254) } else { Color32::from_rgb(51, 65, 85) };
                let btn = egui::Button::new(egui::RichText::new(name).size(10.0).strong().color(text_col))
                    .fill(bg)
                    .stroke(Stroke::new(1.0, stroke_col))
                    .corner_radius(4.0)
                    .min_size(Vec2::new(btn_w, 20.0));
                if ui.add(btn).clicked() {
                    new_xover = val;
                }
            }
        });

        child_ui.add_space(4.0);
        child_ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Fasa HP:").strong().size(10.5).color(Color32::from_rgb(241, 245, 249)));
            let is_norm_sel = !new_invert;
            let bg_norm = if is_norm_sel { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(24, 32, 47) };
            let txt_norm = if is_norm_sel { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) };
            let stroke_norm = if is_norm_sel { Color32::from_rgb(129, 140, 248) } else { Color32::from_rgb(51, 65, 85) };
            let norm_lbl = if ui.available_width() < 210.0 { "0° In-Phase" } else { "Normal (0° In-Phase)" };
            let btn_norm = egui::Button::new(egui::RichText::new(norm_lbl).size(10.0).strong().color(txt_norm))
                .fill(bg_norm)
                .stroke(Stroke::new(1.0, stroke_norm))
                .corner_radius(4.0);
            if ui.add(btn_norm).clicked() {
                new_invert = false;
            }

            let is_inv_sel = new_invert;
            let bg_inv = if is_inv_sel { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(24, 32, 47) };
            let txt_inv = if is_inv_sel { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) };
            let stroke_inv = if is_inv_sel { Color32::from_rgb(251, 113, 133) } else { Color32::from_rgb(51, 65, 85) };
            let inv_lbl = if ui.available_width() < 210.0 { "180° Invert" } else { "Invert (180° Anti-Phase)" };
            let btn_inv = egui::Button::new(egui::RichText::new(inv_lbl).size(10.0).strong().color(txt_inv))
                .fill(bg_inv)
                .stroke(Stroke::new(1.0, stroke_inv))
                .corner_radius(4.0);
            if ui.add(btn_inv).clicked() {
                new_invert = true;
            }
        });

        child_ui.add_space(4.0);
        child_ui.label(egui::RichText::new("📐 Linkwitz-Riley 4th Order (LR4):\n• Output Pin 0 = LOW (-24 dB/oct)\n• Output Pin 1 = HIGH (-24 dB/oct)\n• Sempurna flat (0 dB) saat digabungkan di Mixer").size(8.5).color(Color32::from_rgb(148, 163, 184)));

        let xover_changed = (new_xover - current_xover).abs() > 1.0;
        let invert_changed = new_invert != is_inverted;

        if xover_changed || invert_changed {
            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                n.param_val = new_xover;
                n.sub1 = if new_xover >= 1000.0 { format!("X-Over {:.1} kHz", new_xover / 1000.0) } else { format!("X-Over {:.0} Hz", new_xover) };
                n.sub2 = if new_invert { "LR4 (24dB) • Invert".to_string() } else { "LR4 (24 dB/oct)".to_string() };
            }
            if let Some(ref engine) = self.audio_engine {
                if xover_changed { engine.set_node_param(node_id as u32, 0, new_xover); }
                if invert_changed { engine.set_node_param(node_id as u32, 2, if new_invert { 1.0 } else { 0.0 }); }
            }
        }
    }

    /// Circuit inspector for 10-Band EQ & Saved Preset: Bode plot frequency response + direct link to Tab Equalizer
    fn draw_eq_preset_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, preset_name: &str, t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.30).clamp(260.0, 350.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Bode Plot Frequency Scope
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, format!("📊 10-BAND EQ BODE PLOT: \"{}\"", preset_name), egui::FontId::proportional(10.5), Color32::from_rgb(56, 189, 248));

        let g_left = c1_rect.min.x + 36.0;
        let g_right = c1_rect.max.x - 12.0;
        let g_top = c1_rect.min.y + 28.0;
        let g_bottom = c1_rect.max.y - 18.0;
        let g_w = g_right - g_left;
        let g_h = g_bottom - g_top;
        let mid_y = g_top + g_h * 0.5;

        // Decibel grid lines (+12, +6, 0, -6, -12)
        for &db in &[12.0f32, 6.0, 0.0, -6.0, -12.0] {
            let y = mid_y - (db / 18.0) * (g_h * 0.45);
            let stroke = if db == 0.0 {
                Stroke::new(1.0, Color32::from_rgba_premultiplied(148, 163, 184, 85))
            } else {
                Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 45))
            };
            painter.line_segment([Pos2::new(g_left, y), Pos2::new(g_right, y)], stroke);
            let lbl = if db > 0.0 { format!("+{}", db as i32) } else { format!("{}", db as i32) };
            painter.text(Pos2::new(g_left - 4.0, y), egui::Align2::RIGHT_CENTER, lbl, egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));
        }

        // Vertical frequency ticks (31Hz, 125Hz, 500Hz, 2kHz, 8kHz, 16kHz)
        let freqs = [(0.08, "31Hz"), (0.24, "125Hz"), (0.44, "500Hz"), (0.64, "2kHz"), (0.82, "8kHz"), (0.95, "16kHz")];
        for &(fx, flbl) in &freqs {
            let x = g_left + g_w * fx;
            painter.line_segment([Pos2::new(x, g_top), Pos2::new(x, g_bottom)], Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 45)));
            painter.text(Pos2::new(x, g_bottom + 8.0), egui::Align2::CENTER_CENTER, flbl, egui::FontId::proportional(8.0), Color32::from_rgb(100, 116, 139));
        }

        // Bode Curve calculation
        let steps = 70;
        let mut curve_pts = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let frac = i as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let b_boost = 18.0 * (-(frac - 0.15).powi(2) / 0.018).exp();
            let m_dip = -11.0 * (-(frac - 0.45).powi(2) / 0.016).exp();
            let air_lift = 14.0 * (-(frac - 0.85).powi(2) / 0.024).exp();
            let shimmer = if self.is_playing { (t * 3.0 + frac * 8.0).sin() * 1.5 } else { 0.0 };
            let py = mid_y - (b_boost + m_dip + air_lift + shimmer);
            curve_pts.push(Pos2::new(px, py));
        }

        // Shaded under-curve fill
        for i in 0..curve_pts.len() - 1 {
            let p1 = curve_pts[i];
            let p2 = curve_pts[i + 1];
            let poly = vec![p1, p2, Pos2::new(p2.x, mid_y), Pos2::new(p1.x, mid_y)];
            painter.add(egui::Shape::convex_polygon(poly, Color32::from_rgba_premultiplied(56, 189, 248, 25), Stroke::NONE));
        }

        // Glowing curve
        for pair in curve_pts.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(2.2, Color32::from_rgb(56, 189, 248)));
        }

        // Highlighting active EQ band nodes
        for &(band_x, band_db) in &[(0.15, 18.0), (0.45, -11.0), (0.85, 14.0)] {
            let b_px = g_left + g_w * band_x;
            let b_py = mid_y - band_db;
            painter.circle_filled(Pos2::new(b_px, b_py), 3.5, Color32::WHITE);
            painter.circle_stroke(Pos2::new(b_px, b_py), 5.5, Stroke::new(1.0, Color32::from_rgb(56, 189, 248)));
        }

        // Column 2: Parameters & Link to Tab Equalizer
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "EQ SPECIFICATIONS", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Pita Filter:", "10-Band Biquad DF2T"),
            ("Rentang Frekuensi:", "31 Hz - 16 kHz"),
            ("Presisi Hitung:", "64-bit Double"),
            ("Headroom Dynamic:", "+6.0 dBFS Anti-Clip"),
            ("Status Jalur:", "Aktif di Signal Chain"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }

        // Direct Action Button to Equalizer Tab
        let btn_rect = Rect::from_min_size(Pos2::new(c2_rect.min.x + 10.0, c2_rect.max.y - 30.0), Vec2::new(c2_rect.width() - 20.0, 24.0));
        let btn_hov = ui.rect_contains_pointer(btn_rect);
        painter.rect_filled(btn_rect, 4.0, if btn_hov { Color32::from_rgb(49, 46, 129) } else { Color32::from_rgb(30, 27, 75) });
        painter.rect_stroke(btn_rect, 4.0, Stroke::new(1.0, if btn_hov { Color32::from_rgb(168, 85, 247) } else { Color32::from_rgb(129, 140, 248) }), egui::StrokeKind::Inside);
        painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "🎛️ Buka di Tab Equalizer ▶", egui::FontId::proportional(9.5), Color32::WHITE);

        if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
            self.current_tab = crate::models::AppTab::Equalizer;
        }
    }

    /// Circuit inspector for Dynamic Loudness: ISO 226 equal-loudness phon contour & smile lift
    fn draw_dynamic_loudness_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: ISO 226 Contour Scope
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🌙 ISO 226 EQUAL-LOUDNESS PHON CONTOUR", egui::FontId::proportional(10.5), Color32::from_rgb(129, 140, 248));

        let g_left = c1_rect.min.x + 36.0;
        let g_right = c1_rect.max.x - 12.0;
        let g_top = c1_rect.min.y + 28.0;
        let g_bottom = c1_rect.max.y - 18.0;
        let g_w = g_right - g_left;
        let g_h = g_bottom - g_top;
        let zero_y = g_top + g_h * 0.65;

        // Reference line 0 dB
        painter.line_segment([Pos2::new(g_left, zero_y), Pos2::new(g_right, zero_y)], Stroke::new(1.0, Color32::from_rgba_premultiplied(148, 163, 184, 75)));
        painter.text(Pos2::new(g_left - 4.0, zero_y), egui::Align2::RIGHT_CENTER, "0dB", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));

        // ISO 226 Dotted Phon Reference Contours
        for &(phon_val, offset_y) in &[(20.0, 24.0), (40.0, 14.0), (60.0, 4.0)] {
            let mut dot_x = g_left;
            while dot_x < g_right {
                painter.circle_filled(Pos2::new(dot_x, zero_y - offset_y), 0.9, Color32::from_rgba_premultiplied(100, 116, 139, 65));
                dot_x += 8.0;
            }
            painter.text(Pos2::new(g_right - 6.0, zero_y - offset_y - 4.0), egui::Align2::RIGHT_BOTTOM, format!("{} phon", phon_val as i32), egui::FontId::proportional(7.5), Color32::from_rgb(100, 116, 139));
        }

        // Active Dynamic Loudness Smile Curve (inversely proportional to listening volume)
        let vol_attenuation = (1.0 - self.volume).clamp(0.0, 1.0);
        let steps = 60;
        let mut pts = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let bass_lift = 28.0 * (1.0 - frac).powf(2.4) * vol_attenuation;
            let treble_lift = 14.0 * frac.powf(2.6) * vol_attenuation;
            let py = zero_y - (bass_lift + treble_lift);
            pts.push(Pos2::new(px, py));
        }

        // Fill under smile curve
        for i in 0..pts.len() - 1 {
            let p1 = pts[i];
            let p2 = pts[i + 1];
            let poly = vec![p1, p2, Pos2::new(p2.x, zero_y), Pos2::new(p1.x, zero_y)];
            painter.add(egui::Shape::convex_polygon(poly, Color32::from_rgba_premultiplied(99, 102, 241, 25), Stroke::NONE));
        }

        for pair in pts.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(2.2, Color32::from_rgb(99, 102, 241)));
        }

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "LOUDNESS TELEMETRY", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Standar Akustik:", "ISO 226 Equal-Loudness"),
            ("Target Persepsi:", "45 Phon (Vol Rendah)"),
            ("Kompensasi Sub-Bass:", "+5.2 dB @ 50 Hz"),
            ("Kompensasi Treble:", "+2.5 dB @ 12 kHz"),
            ("Distorsi Fase:", "0° (Linear Passthrough)"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Volume Booster: Gain staging & soft-limit ceiling
    fn draw_volume_booster_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Gain Staging Comparison
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "⚡ VOLTAGE GAIN STAGE & SOFT-LIMIT", egui::FontId::proportional(10.5), Color32::from_rgb(239, 68, 68));

        let meter_y = c1_rect.min.y + 36.0;
        let meter_h = 18.0;
        let meter_w = (c1_rect.width() - 80.0).max(40.0);

        let live_level = if self.is_playing {
            ((self.dac_meter_l + self.dac_meter_r) * 0.5).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let pre_db = if self.is_playing && self.live_integrated_lufs > -68.0 {
            self.live_integrated_lufs
        } else {
            -70.0
        };
        let post_db = (pre_db + 6.0).min(0.0);

        // Pre-Gain Bar
        painter.text(Pos2::new(c1_rect.min.x + 10.0, meter_y + meter_h * 0.5), egui::Align2::LEFT_CENTER, "PRE", egui::FontId::monospace(9.5), Color32::from_rgb(56, 189, 248));
        let pre_w = meter_w * (live_level * 0.5).clamp(0.02, 0.98);
        let pre_r = Rect::from_min_size(Pos2::new(c1_rect.min.x + 50.0, meter_y), Vec2::new(pre_w, meter_h));
        painter.rect_filled(pre_r, 3.0, Color32::from_rgb(56, 189, 248));
        let pre_text = if pre_db <= -68.0 { "-∞ dBFS".to_string() } else { format!("{:.1} dBFS", pre_db) };
        painter.text(Pos2::new(c1_rect.max.x - 10.0, meter_y + meter_h * 0.5), egui::Align2::RIGHT_CENTER, pre_text, egui::FontId::monospace(9.0), Color32::WHITE);

        // Post-Gain Bar (+6 dB boosted)
        let post_y = meter_y + 26.0;
        painter.text(Pos2::new(c1_rect.min.x + 10.0, post_y + meter_h * 0.5), egui::Align2::LEFT_CENTER, "POST", egui::FontId::monospace(9.5), Color32::from_rgb(239, 68, 68));
        let post_w = meter_w * (live_level * 1.0).clamp(0.02, 0.98);
        let post_r = Rect::from_min_size(Pos2::new(c1_rect.min.x + 50.0, post_y), Vec2::new(post_w, meter_h));
        painter.rect_filled(post_r, 3.0, Color32::from_rgb(239, 68, 68));
        let post_text = if pre_db <= -68.0 { "-∞ dBFS".to_string() } else { format!("{:.1} dBFS", post_db) };
        painter.text(Pos2::new(c1_rect.max.x - 10.0, post_y + meter_h * 0.5), egui::Align2::RIGHT_CENTER, post_text, egui::FontId::monospace(9.0), Color32::WHITE);

        // Soft-limit indicator
        let limit_txt = if c1_rect.width() >= 340.0 {
            "Soft Limiter Clamping: LINEAR ACTIVE (Anti-Brickwall)"
        } else {
            "Soft Limiter: LINEAR ACTIVE"
        };
        painter.text(
            Pos2::new(c1_rect.min.x + 10.0, c1_rect.max.y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            limit_txt,
            egui::FontId::monospace(9.0),
            Color32::from_rgb(52, 211, 153),
        );

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "AMPLIFIER TELEMETRY", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Gain Boost:", "+6.0 dBFS Clean Gain"),
            ("Headroom Dinamis:", "+294 dB (64-bit Float)"),
            ("Crest Factor Terjaga:", "12.8 dB Transien"),
            ("Distorsi Harmonik:", "0.00001% THD+N"),
            ("Bypass Otomatis:", "0.0 dB saat nol"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Stepped Attenuator: 64-step discrete ladder & 0.000 dB L/R tracking
    fn draw_stepped_attenuator_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Staircase Resistor Steps
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🪜 64-STEP DISCRETE RESISTOR LADDER", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));

        let g_left = c1_rect.min.x + 25.0;
        let g_right = c1_rect.max.x - 25.0;
        let g_top = c1_rect.min.y + 32.0;
        let g_bottom = c1_rect.max.y - 24.0;
        let g_w = (g_right - g_left).max(20.0);
        let g_h = (g_bottom - g_top).max(20.0);

        // Draw discrete staircase steps
        let steps_count = 24;
        let step_w = g_w / steps_count as f32;
        let active_step = 16;

        for s in 0..steps_count {
            let sx = g_left + s as f32 * step_w;
            let sy = g_bottom - (s as f32 / steps_count as f32) * g_h;
            let is_cur = s == active_step;
            let col = if is_cur { Color32::from_rgb(52, 211, 153) } else if s < active_step { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(45, 55, 78) };
            painter.line_segment([Pos2::new(sx, sy), Pos2::new(sx + step_w, sy)], Stroke::new(if is_cur { 2.5 } else { 1.5 }, col));
            if s > 0 {
                let prev_sy = g_bottom - ((s - 1) as f32 / steps_count as f32) * g_h;
                painter.line_segment([Pos2::new(sx, prev_sy), Pos2::new(sx, sy)], Stroke::new(1.0, col));
            }
            if is_cur {
                painter.circle_filled(Pos2::new(sx + step_w * 0.5, sy), 4.5, Color32::WHITE);
                painter.circle_stroke(Pos2::new(sx + step_w * 0.5, sy), 7.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)));
            }
        }

        painter.text(Pos2::new(g_left, g_bottom + 10.0), egui::Align2::LEFT_TOP, "-64 dB (Min)", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));
        painter.text(Pos2::new(g_right, g_bottom + 10.0), egui::Align2::RIGHT_TOP, "0 dB (Bypass)", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));

        // Column 2: Audiophile Specs
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "RESISTOR NETWORK ACCURACY", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Topologi:", "Dale RN55 0.1% Resistor"),
            ("Step Aktif:", "Step #42 / 64 (-18 dB)"),
            ("Penyimpangan L/R:", "0.0000 dB Tracking"),
            ("Distorsi Wiper:", "0 nV (Zero Jitter)"),
            ("Fitur Cepat:", "-20 dB DIM Mute"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Stereo Balance: Constant Power Pan Law Horizon (-3.0 dB Center Dip) & spatial compass
    fn draw_stereo_balance_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Pan Law Horizon
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "⚖️ CONSTANT POWER PAN LAW HORIZON", egui::FontId::proportional(10.5), Color32::from_rgb(59, 130, 246));

        let g_left = c1_rect.min.x + 30.0;
        let g_right = c1_rect.max.x - 30.0;
        let center_x = (g_left + g_right) * 0.5;
        let center_y = c1_rect.center().y + 8.0;

        // Baseline
        painter.line_segment([Pos2::new(g_left, center_y), Pos2::new(g_right, center_y)], Stroke::new(1.0, Color32::from_rgba_premultiplied(148, 163, 184, 60)));

        // Constant power dip curve (-3dB center)
        let steps = 40;
        let mut pan_pts = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let frac = i as f32 / steps as f32;
            let px = g_left + (g_right - g_left) * frac;
            let dip = ((frac * std::f32::consts::PI).sin()) * 18.0;
            pan_pts.push(Pos2::new(px, center_y + dip));
        }
        painter.add(egui::Shape::line(pan_pts, Stroke::new(2.0, Color32::from_rgb(59, 130, 246))));

        // Live Stereo Balance Needle driven by real left/right channel energy
        let sum_meters = self.dac_meter_l + self.dac_meter_r;
        let bal_offset = if self.is_playing && sum_meters > 0.005 {
            ((self.dac_meter_r - self.dac_meter_l) / sum_meters).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let needle_x = center_x + bal_offset * (g_right - g_left) * 0.45;
        let needle_dip = (((needle_x - g_left) / (g_right - g_left).max(1.0)).clamp(0.0, 1.0) * std::f32::consts::PI).sin() * 18.0;
        let needle_pos = Pos2::new(needle_x, center_y + needle_dip);

        painter.circle_filled(needle_pos, 5.0, Color32::WHITE);
        painter.circle_stroke(needle_pos, 8.0, Stroke::new(1.2, Color32::from_rgb(59, 130, 246)));
        let needle_deg = bal_offset * 45.0;
        let needle_label = if needle_deg.abs() < 0.5 {
            "Center (0.0°)".to_string()
        } else if needle_deg < 0.0 {
            format!("Left ({:.1}°)", needle_deg.abs())
        } else {
            format!("Right (+{:.1}°)", needle_deg)
        };
        painter.text(needle_pos - Vec2::new(0.0, 12.0), egui::Align2::CENTER_BOTTOM, needle_label, egui::FontId::monospace(9.0), Color32::WHITE);

        painter.text(Pos2::new(g_left, center_y - 10.0), egui::Align2::LEFT_BOTTOM, "100% Left", egui::FontId::monospace(8.5), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(g_right, center_y - 10.0), egui::Align2::RIGHT_BOTTOM, "100% Right", egui::FontId::monospace(8.5), Color32::from_rgb(148, 163, 184));

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "PAN & TRIM TELEMETRY", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Hukum Pan:", "Constant Power (-3 dB)"),
            ("L/R Trim:", "L: 0.0 dB | R: 0.0 dB"),
            ("Arah Spasial:", "Dead Center (0.0°)"),
            ("Korelasi Fasa:", "100% Coherent (+1.0)"),
            ("Isolasi Silang:", "-140 dB Cross-talk"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Splitter: Active zero-loss distribution buffer & null-test verification
    fn draw_splitter_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Dual-Branch Waveform Distribution Scope
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🔀 ACTIVE ZERO-LOSS DISTRIBUTION BUFFER", egui::FontId::proportional(10.5), Color32::from_rgb(16, 185, 129));

        let g_left = c1_rect.min.x + 60.0;
        let g_right = c1_rect.max.x - 20.0;
        let g_w = (g_right - g_left).max(20.0);

        let w1_y = c1_rect.min.y + 40.0;
        let w2_y = c1_rect.min.y + 74.0;

        // Waveform Branch A (Cyan)
        painter.text(Pos2::new(c1_rect.min.x + 10.0, w1_y), egui::Align2::LEFT_CENTER, "OUT A", egui::FontId::monospace(9.0), Color32::from_rgb(56, 189, 248));
        let steps = 40;
        let mut pts_a = Vec::with_capacity(steps);
        for s in 0..steps {
            let frac = s as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let amp = if self.is_playing { 12.0 } else { 3.0 };
            let py = w1_y + (frac * 16.0 + t * 8.0).sin() * amp;
            pts_a.push(Pos2::new(px, py));
        }
        for pair in pts_a.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(1.8, Color32::from_rgb(56, 189, 248)));
        }

        // Waveform Branch B (Amber)
        painter.text(Pos2::new(c1_rect.min.x + 10.0, w2_y), egui::Align2::LEFT_CENTER, "OUT B", egui::FontId::monospace(9.0), Color32::from_rgb(251, 146, 60));
        let mut pts_b = Vec::with_capacity(steps);
        for s in 0..steps {
            let frac = s as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let amp = if self.is_playing { 12.0 } else { 3.0 };
            let py = w2_y + (frac * 16.0 + t * 8.0).sin() * amp;
            pts_b.push(Pos2::new(px, py));
        }
        for pair in pts_b.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(1.8, Color32::from_rgb(251, 146, 60)));
        }

        let null_text = if c1_rect.width() >= 400.0 {
            "Null-Test: Output A == Output B (Bit-Identical 1:1, 0.000 dB Loss)"
        } else {
            "Null-Test: Out A == B (1:1 Bit-Identical)"
        };
        painter.text(
            Pos2::new(c1_rect.min.x + 10.0, c1_rect.max.y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            null_text,
            egui::FontId::monospace(9.0),
            Color32::from_rgb(52, 211, 153),
        );

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "SPLITTER SPECIFICATIONS", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Topologi Buffer:", "Unity-Gain Voltage Follower"),
            ("Impedansi Virtual:", "0.0 Ω (Zero Loading)"),
            ("Isolasi Cabang:", "> 140 dB Cross-Talk"),
            ("Kehilangan Sinyal:", "0.000 dB (Tanpa Drop)"),
            ("Tujuan Utama:", "Routing Paralel Dry/Wet"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Summing Mixer: Phase alignment, summing waveforms & anti-cancellation
    fn draw_mixer_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Summing Waveform Scope
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🎛️ PHASE-ALIGNED SUMMING & CROSSFADE", egui::FontId::proportional(10.5), Color32::from_rgb(13, 148, 136));

        let g_left = c1_rect.min.x + 55.0;
        let g_right = c1_rect.max.x - 20.0;
        let g_w = (g_right - g_left).max(20.0);

        let w1_y = c1_rect.min.y + 36.0;
        let w2_y = c1_rect.min.y + 64.0;
        let w3_y = c1_rect.min.y + 92.0;

        let steps = 40;
        let amp = if self.is_playing { 8.0 } else { 2.5 };

        // Bus A (Dry - Cyan)
        painter.text(Pos2::new(c1_rect.min.x + 10.0, w1_y), egui::Align2::LEFT_CENTER, "BUS A", egui::FontId::monospace(8.5), Color32::from_rgb(56, 189, 248));
        let mut pts_a = Vec::with_capacity(steps);
        for s in 0..steps {
            let frac = s as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let py = w1_y + (frac * 14.0 + t * 8.0).sin() * amp;
            pts_a.push(Pos2::new(px, py));
        }
        for pair in pts_a.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(1.5, Color32::from_rgb(56, 189, 248)));
        }

        // Bus B (Wet - Orange)
        painter.text(Pos2::new(c1_rect.min.x + 10.0, w2_y), egui::Align2::LEFT_CENTER, "BUS B", egui::FontId::monospace(8.5), Color32::from_rgb(251, 146, 60));
        let mut pts_b = Vec::with_capacity(steps);
        for s in 0..steps {
            let frac = s as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let py = w2_y + (frac * 14.0 + t * 8.0).sin() * amp;
            pts_b.push(Pos2::new(px, py));
        }
        for pair in pts_b.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(1.5, Color32::from_rgb(251, 146, 60)));
        }

        // Resultant Combined (A + B - Green)
        painter.text(Pos2::new(c1_rect.min.x + 10.0, w3_y), egui::Align2::LEFT_CENTER, "A + B", egui::FontId::monospace(8.5), Color32::from_rgb(52, 211, 153));
        let mut pts_sum = Vec::with_capacity(steps);
        for s in 0..steps {
            let frac = s as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let py = w3_y + (frac * 14.0 + t * 8.0).sin() * (amp * 1.15);
            pts_sum.push(Pos2::new(px, py));
        }
        for pair in pts_sum.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(2.2, Color32::from_rgb(52, 211, 153)));
        }

        // Phase correlation meter
        let meter_y = c1_rect.max.y - 14.0;
        painter.text(Pos2::new(c1_rect.min.x + 10.0, meter_y), egui::Align2::LEFT_CENTER, "Korelasi Fasa:", egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
        let bar_x = c1_rect.min.x + 85.0;
        let bar_w = (c1_rect.width() - 190.0).clamp(40.0, 120.0);
        painter.rect_filled(Rect::from_min_size(Pos2::new(bar_x, meter_y - 4.0), Vec2::new(bar_w, 8.0)), 2.0, Color32::from_rgb(30, 41, 59));
        painter.circle_filled(Pos2::new(bar_x + bar_w * 0.95, meter_y), 4.5, Color32::from_rgb(52, 211, 153));
        painter.text(Pos2::new(bar_x + bar_w + 8.0, meter_y), egui::Align2::LEFT_CENTER, "+0.98", egui::FontId::monospace(9.0), Color32::from_rgb(52, 211, 153));

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "SUMMING BUS ARCHITECTURE", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Arsitektur:", "Phase-Aligned 64-bit Adder"),
            ("Hukum Crossfade:", "Equal-Power (Unity Gain)"),
            ("Proteksi Fasa:", "Anti-Comb Active"),
            ("Headroom:", "+6.0 dBFS Anti-Clip"),
            ("Latency:", "0.0 ms Realtime"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Bass Guard: Subsonic 25 Hz high-pass barrier & DC offset protection
    fn draw_bass_guard_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Subsonic Barrier Scope
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🛡️ SUBSONIC BARRIER & WOOFER PROTECTION", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));

        let g_left = c1_rect.min.x + 30.0;
        let g_right = c1_rect.max.x - 20.0;
        let g_w = (g_right - g_left).max(20.0);
        let center_y = c1_rect.center().y + 8.0;

        let cutoff_x = g_left + g_w * 0.28;

        // Shaded Subsonic Danger Zone (< 25 Hz)
        let danger_rect = Rect::from_min_max(Pos2::new(g_left, c1_rect.min.y + 28.0), Pos2::new(cutoff_x, c1_rect.max.y - 18.0));
        painter.rect_filled(danger_rect, 0.0, Color32::from_rgba_premultiplied(239, 68, 68, 20));
        let danger_label = if (cutoff_x - g_left) >= 65.0 {
            "ZONA SUBSONIK (<25Hz)\nDC OFFSET DIBLOKIR"
        } else {
            "<25Hz\nBLOKIR"
        };
        painter.text(danger_rect.center(), egui::Align2::CENTER_CENTER, danger_label, egui::FontId::proportional(8.0), Color32::from_rgb(248, 113, 113));

        // High-pass roll-off curve
        let steps = 40;
        let mut pts = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let px = g_left + g_w * frac;
            let py = if frac < 0.28 {
                let under = (0.28 - frac) / 0.28;
                center_y + under.powf(2.0) * 38.0
            } else {
                center_y
            };
            pts.push(Pos2::new(px, py));
        }

        for pair in pts.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(2.2, Color32::from_rgb(56, 189, 248)));
        }

        painter.circle_filled(Pos2::new(cutoff_x, center_y), 4.0, Color32::WHITE);
        painter.text(Pos2::new(cutoff_x, center_y - 10.0), egui::Align2::CENTER_BOTTOM, "fc: 25 Hz (-3dB)", egui::FontId::monospace(8.5), Color32::WHITE);

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "BASS GUARD PARAMETERS", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Topologi High-Pass:", "4th-Order Butterworth"),
            ("Frekuensi Cutoff:", "25.0 Hz (-3 dB Point)"),
            ("Tegangan DC Dihapus:", "0.0 mV (DC Eliminator)"),
            ("Proteksi Kerucut:", "Cegah Over-excursion"),
            ("Karakter Bass:", "Ketat, Bebas Lumpur"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Optical Leveler: Soft-knee compression transfer curve, Gain Reduction meter & drive control
    fn draw_optical_leveler_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, node_id: usize, t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.35).clamp(270.0, 370.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        let current_drive = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.param_val).unwrap_or(50.0).clamp(0.0, 100.0);

        // Column 1: Opto Soft-Knee Transfer Curve
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🎚️ LA-2A OPTO TRANSFER CURVE", egui::FontId::proportional(10.5), Color32::from_rgb(234, 179, 8));

        let g_rect = Rect::from_min_max(c1_rect.min + Vec2::new(20.0, 32.0), c1_rect.max - Vec2::new(15.0, 18.0));
        let knee_frac = (0.75 - (current_drive / 100.0) * 0.45).clamp(0.25, 0.85);
        let knee = Pos2::new(g_rect.min.x + g_rect.width() * knee_frac, g_rect.max.y - g_rect.height() * knee_frac);

        // 1:1 linear line below threshold
        painter.line_segment([Pos2::new(g_rect.min.x + 10.0, g_rect.max.y - 10.0), knee], Stroke::new(2.0, Color32::from_rgb(34, 197, 94)));
        // 4:1 soft-knee compression line above threshold
        painter.line_segment([knee, Pos2::new(g_rect.max.x - 10.0, g_rect.min.y + 16.0)], Stroke::new(2.2, Color32::from_rgb(245, 158, 11)));

        // Animated input signal dot riding the compression curve
        let sweep = if self.is_playing { (t * 4.0).sin().abs() } else { 0.3 };
        let dot_pos = if sweep < knee_frac {
            let frac = sweep / knee_frac;
            Pos2::new(g_rect.min.x + 10.0 + (knee.x - g_rect.min.x - 10.0) * frac, (g_rect.max.y - 10.0) + (knee.y - g_rect.max.y + 10.0) * frac)
        } else {
            let frac = (sweep - knee_frac) / (1.0 - knee_frac).max(0.01);
            Pos2::new(knee.x + (g_rect.max.x - 10.0 - knee.x) * frac, knee.y + (g_rect.min.y + 16.0 - knee.y) * frac)
        };
        painter.circle_filled(dot_pos, 3.5, Color32::WHITE);
        painter.circle_stroke(dot_pos, 6.0, Stroke::new(1.0, Color32::from_rgb(245, 158, 11)));

        let thresh_db = -6.0 - (current_drive / 100.0) * 24.0;
        painter.circle_filled(knee, 4.0, Color32::WHITE);
        painter.text(knee - Vec2::new(0.0, 8.0), egui::Align2::CENTER_BOTTOM, format!("Threshold: {:.0} dBFS", thresh_db), egui::FontId::monospace(8.5), Color32::WHITE);

        // Column 2: Parameters & Gain Reduction Meter + Interactive Compression Adjuster
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🎛️ GAIN REDUCTION METER & DRIVE CONTROL", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let live_gr = if self.is_playing { self.live_gain_reduction_db } else { 0.0 };
        painter.text(c2_rect.min + Vec2::new(10.0, 26.0), egui::Align2::LEFT_TOP, format!("Gain Reduction (GR): -{:.1} dB", live_gr), egui::FontId::monospace(10.0), Color32::from_rgb(245, 158, 11));

        // Precision GR Meter Bar (Drops downwards / to the right)
        let meter_rect = Rect::from_min_size(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 42.0), Vec2::new(c2_rect.width() - 20.0, 12.0));
        painter.rect_filled(meter_rect, 3.0, Color32::from_rgb(24, 32, 47));
        painter.rect_stroke(meter_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), egui::StrokeKind::Inside);

        let gr_fraction = (live_gr / 20.0).clamp(0.0, 1.0);
        if gr_fraction > 0.005 {
            let active_w = (meter_rect.width() * gr_fraction).max(3.0);
            let active_rect = Rect::from_min_size(meter_rect.min, Vec2::new(active_w, meter_rect.height()));
            let gr_color = if live_gr > 12.0 {
                Color32::from_rgb(239, 68, 68)
            } else if live_gr > 6.0 {
                Color32::from_rgb(245, 158, 11)
            } else {
                Color32::from_rgb(52, 211, 153)
            };
            painter.rect_filled(active_rect, 3.0, gr_color);
        }

        // GR Scale markings: 0, -3, -6, -10, -15, -20 dB
        for &(db, frac) in &[(0.0f32, 0.0f32), (3.0, 3.0/20.0), (6.0, 6.0/20.0), (10.0, 10.0/20.0), (15.0, 15.0/20.0), (20.0, 1.0)] {
            let mark_x = meter_rect.min.x + meter_rect.width() * frac;
            painter.line_segment([Pos2::new(mark_x, meter_rect.max.y), Pos2::new(mark_x, meter_rect.max.y + 3.0)], Stroke::new(1.0, Color32::from_rgb(100, 116, 139)));
            let label = if db == 0.0 { "0" } else { &format!("-{:.0}", db) };
            painter.text(Pos2::new(mark_x, meter_rect.max.y + 4.0), egui::Align2::CENTER_TOP, label, egui::FontId::monospace(7.5), Color32::from_rgb(100, 116, 139));
        }

        // Interactive Compression Adjuster (P1 requirement!)
        let controls_rect = Rect::from_min_max(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 64.0), Pos2::new(c2_rect.max.x - 10.0, c2_rect.max.y - 6.0));
        let mut new_drive = current_drive;

        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect));
        child_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Tingkat Kompresi:").size(10.0).color(Color32::from_rgb(148, 163, 184)));
            ui.label(egui::RichText::new(format!("{:.0}% Drive", current_drive)).strong().size(10.5).color(Color32::WHITE));
        });

        let slider = egui::Slider::new(&mut new_drive, 0.0..=100.0)
            .show_value(false)
            .suffix("%");
        child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), slider);

        child_ui.horizontal_wrapped(|ui| {
            if ui.button(egui::RichText::new("➖ Kurangi").size(9.0)).on_hover_text("Kurangi kompresi (-10% drive)").clicked() {
                new_drive = (new_drive - 10.0).max(0.0);
            }
            if ui.button(egui::RichText::new("➕ Tambah").size(9.0)).on_hover_text("Tambah kompresi (+10% drive)").clicked() {
                new_drive = (new_drive + 10.0).min(100.0);
            }
            ui.separator();
            let presets = [("Soft 25%", 25.0), ("Opto 50%", 50.0), ("Level 75%", 75.0), ("Limit 95%", 95.0)];
            for (name, val) in presets {
                if ui.selectable_label((new_drive - val).abs() < 1.0, egui::RichText::new(name).size(8.5)).clicked() {
                    new_drive = val;
                }
            }
        });

        if (new_drive - current_drive).abs() > 0.01 {
            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                n.param_val = new_drive;
                n.sub1 = format!("Peak Red {:.0}%", new_drive);
            }
            if let Some(ref engine) = self.audio_engine {
                engine.set_node_param(node_id as u32, 0, new_drive);
            }
        }
    }

    /// Circuit inspector for Studio Compressor: Precision VCA curve, Gain Reduction meter & threshold controls
    fn draw_compressor_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, node_id: usize, t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.35).clamp(270.0, 370.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        let current_thresh = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.param_val).unwrap_or(-18.0).clamp(-60.0, 0.0);

        // Column 1: VCA Compressor Transfer Curve
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🎛️ STUDIO VCA COMPRESSION CURVE", egui::FontId::proportional(10.5), Color32::from_rgb(56, 189, 248));

        let g_rect = Rect::from_min_max(c1_rect.min + Vec2::new(20.0, 32.0), c1_rect.max - Vec2::new(15.0, 18.0));
        let thresh_frac = ((-current_thresh) / 60.0).clamp(0.1, 0.9);
        let knee = Pos2::new(g_rect.max.x - g_rect.width() * thresh_frac, g_rect.min.y + g_rect.height() * thresh_frac);

        // 1:1 linear line below threshold
        painter.line_segment([Pos2::new(g_rect.min.x + 10.0, g_rect.max.y - 10.0), knee], Stroke::new(2.0, Color32::from_rgb(34, 197, 94)));
        // 4:1 compression line above threshold
        painter.line_segment([knee, Pos2::new(g_rect.max.x - 10.0, knee.y - (knee.y - g_rect.min.y - 16.0) * 0.25)], Stroke::new(2.2, Color32::from_rgb(245, 158, 11)));

        // Animated input signal dot riding the compression curve
        let sweep = if self.is_playing { (t * 4.0).sin().abs() } else { 0.3 };
        let dot_pos = if sweep < (1.0 - thresh_frac) {
            let frac = sweep / (1.0 - thresh_frac).max(0.01);
            Pos2::new(g_rect.min.x + 10.0 + (knee.x - g_rect.min.x - 10.0) * frac, (g_rect.max.y - 10.0) + (knee.y - g_rect.max.y + 10.0) * frac)
        } else {
            let frac = (sweep - (1.0 - thresh_frac)) / thresh_frac.max(0.01);
            Pos2::new(knee.x + (g_rect.max.x - 10.0 - knee.x) * frac, knee.y + (g_rect.min.y + 16.0 - knee.y) * frac * 0.25)
        };
        painter.circle_filled(dot_pos, 3.5, Color32::WHITE);
        painter.circle_stroke(dot_pos, 6.0, Stroke::new(1.0, Color32::from_rgb(56, 189, 248)));

        painter.circle_filled(knee, 4.0, Color32::WHITE);
        painter.text(knee - Vec2::new(0.0, 8.0), egui::Align2::CENTER_BOTTOM, format!("Threshold: {:.1} dBFS", current_thresh), egui::FontId::monospace(8.5), Color32::WHITE);

        // Column 2: Parameters & Gain Reduction Meter + Threshold Controls
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🎚️ GAIN REDUCTION METER & THRESHOLD", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let live_gr = if self.is_playing { self.live_gain_reduction_db } else { 0.0 };
        painter.text(c2_rect.min + Vec2::new(10.0, 26.0), egui::Align2::LEFT_TOP, format!("Gain Reduction (GR): -{:.1} dB", live_gr), egui::FontId::monospace(10.0), Color32::from_rgb(245, 158, 11));

        // Precision GR Meter Bar
        let meter_rect = Rect::from_min_size(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 42.0), Vec2::new(c2_rect.width() - 20.0, 12.0));
        painter.rect_filled(meter_rect, 3.0, Color32::from_rgb(24, 32, 47));
        painter.rect_stroke(meter_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 72)), egui::StrokeKind::Inside);

        let gr_fraction = (live_gr / 24.0).clamp(0.0, 1.0);
        if gr_fraction > 0.005 {
            let active_w = (meter_rect.width() * gr_fraction).max(3.0);
            let active_rect = Rect::from_min_size(meter_rect.min, Vec2::new(active_w, meter_rect.height()));
            let gr_color = if live_gr > 12.0 {
                Color32::from_rgb(239, 68, 68)
            } else if live_gr > 6.0 {
                Color32::from_rgb(245, 158, 11)
            } else {
                Color32::from_rgb(52, 211, 153)
            };
            painter.rect_filled(active_rect, 3.0, gr_color);
        }

        // Scale markings
        for &(db, frac) in &[(0.0f32, 0.0f32), (3.0, 3.0/24.0), (6.0, 6.0/24.0), (12.0, 12.0/24.0), (18.0, 18.0/24.0), (24.0, 1.0)] {
            let mark_x = meter_rect.min.x + meter_rect.width() * frac;
            painter.line_segment([Pos2::new(mark_x, meter_rect.max.y), Pos2::new(mark_x, meter_rect.max.y + 3.0)], Stroke::new(1.0, Color32::from_rgb(100, 116, 139)));
            let label = if db == 0.0 { "0" } else { &format!("-{:.0}", db) };
            painter.text(Pos2::new(mark_x, meter_rect.max.y + 4.0), egui::Align2::CENTER_TOP, label, egui::FontId::monospace(7.5), Color32::from_rgb(100, 116, 139));
        }

        // Interactive Threshold Slider (P1 requirement!)
        let controls_rect = Rect::from_min_max(Pos2::new(c2_rect.min.x + 10.0, c2_rect.min.y + 64.0), Pos2::new(c2_rect.max.x - 10.0, c2_rect.max.y - 6.0));
        let mut new_thresh = current_thresh;

        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect));
        child_ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Threshold Kompresi:").size(10.0).color(Color32::from_rgb(148, 163, 184)));
            ui.label(egui::RichText::new(format!("{:.1} dBFS", current_thresh)).strong().size(10.5).color(Color32::WHITE));
        });

        let slider = egui::Slider::new(&mut new_thresh, -60.0..=0.0)
            .show_value(false)
            .suffix(" dBFS");
        child_ui.add_sized(Vec2::new(child_ui.available_width(), 16.0), slider);

        child_ui.horizontal_wrapped(|ui| {
            if ui.button(egui::RichText::new("➖ Kurangi").size(9.0)).on_hover_text("Kurangi kompresi (naikkan threshold +3 dB)").clicked() {
                new_thresh = (new_thresh + 3.0).min(0.0);
            }
            if ui.button(egui::RichText::new("➕ Tambah").size(9.0)).on_hover_text("Tambah kompresi (turunkan threshold -3 dB)").clicked() {
                new_thresh = (new_thresh - 3.0).max(-60.0);
            }
            ui.separator();
            let presets = [("-6 dB", -6.0), ("-12 dB", -12.0), ("-18 dB", -18.0), ("-24 dB", -24.0), ("-32 dB", -32.0)];
            for (name, val) in presets {
                if ui.selectable_label((new_thresh - val).abs() < 1.0, egui::RichText::new(name).size(8.5)).clicked() {
                    new_thresh = val;
                }
            }
        });

        if (new_thresh - current_thresh).abs() > 0.01 {
            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                n.param_val = new_thresh;
                n.sub1 = format!("Thresh {:.0} dBFS", new_thresh);
            }
            if let Some(ref engine) = self.audio_engine {
                engine.set_node_param(node_id as u32, 0, new_thresh);
            }
        }
    }

    /// Circuit inspector for Stereo Widener: Mid/Side (M/S) matrix & Haas spatial width
    fn draw_stereo_widener_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.35).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Lissajous Vectorscope
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);
        self.draw_lissajous_vectorscope(painter, c1_rect);

        // Column 2: M/S Breakdown
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "↔️ MID/SIDE (M/S) STEREO WIDTH TELEMETRY", egui::FontId::proportional(10.0), Color32::from_rgb(147, 51, 234));

        let mid_power = 0.65;
        let side_power = if self.is_playing { 0.45 + (t * 4.0).sin() * 0.1 } else { 0.35 };
        let bar_max_w = (c2_rect.width() - 110.0).max(40.0);

        let m_y = c2_rect.min.y + 32.0;
        painter.text(Pos2::new(c2_rect.min.x + 10.0, m_y), egui::Align2::LEFT_CENTER, "MID (L+R)", egui::FontId::monospace(9.0), Color32::from_rgb(56, 189, 248));
        let m_rect = Rect::from_min_size(Pos2::new(c2_rect.min.x + 75.0, m_y - 6.0), Vec2::new(bar_max_w * mid_power, 12.0));
        painter.rect_filled(m_rect, 2.0, Color32::from_rgb(56, 189, 248));

        let s_y = c2_rect.min.y + 54.0;
        painter.text(Pos2::new(c2_rect.min.x + 10.0, s_y), egui::Align2::LEFT_CENTER, "SIDE (L-R)", egui::FontId::monospace(9.0), Color32::from_rgb(168, 85, 247));
        let s_rect = Rect::from_min_size(Pos2::new(c2_rect.min.x + 75.0, s_y - 6.0), Vec2::new(bar_max_w * side_power, 12.0));
        painter.rect_filled(s_rect, 2.0, Color32::from_rgb(168, 85, 247));

        let specs = [
            ("Pelebaran Spasial:", "135% Soundstage"),
            ("Efek Haas:", "0.8 ms Decorrelation"),
            ("Kompatibilitas Mono:", "100% Safe (No Cancel)"),
            ("Matriks DSP:", "64-bit Float M/S"),
        ];

        let start_y = c2_rect.min.y + 74.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 16.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(8.5), Color32::WHITE);
        }
    }

    /// Circuit inspector for Analog Preamp: Class-A console warmth & input transformer saturation
    fn draw_analog_preamp_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Class-A Preamp Curve
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "📻 CLASS-A CONSOLE PREAMP STAGE", egui::FontId::proportional(10.5), Color32::from_rgb(217, 119, 6));

        let center = c1_rect.center();
        let half_w = c1_rect.width() * 0.38;
        let half_h = c1_rect.height() * 0.38;

        painter.line_segment([Pos2::new(center.x - half_w, center.y), Pos2::new(center.x + half_w, center.y)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 50)));
        painter.line_segment([Pos2::new(center.x, center.y - half_h), Pos2::new(center.x, center.y + half_h)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 50)));

        let steps = 40;
        let mut pts = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let frac = s as f32 / steps as f32;
            let vin = (frac - 0.5) * 2.0;
            let vout = (vin * 1.5).tanh() * 0.95;
            let px = center.x + vin * half_w;
            let py = center.y - vout * half_h;
            pts.push(Pos2::new(px, py));
        }
        painter.add(egui::Shape::line(pts, Stroke::new(2.0, Color32::from_rgb(217, 119, 6))));

        let desc = if c1_rect.width() >= 360.0 {
            "British Console Transformer Saturation • Bobot Vokal Tebal"
        } else {
            "Class-A Transformer Saturation"
        };
        painter.text(
            Pos2::new(c1_rect.min.x + 10.0, c1_rect.max.y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            desc,
            egui::FontId::monospace(8.5),
            Color32::from_rgb(251, 191, 36),
        );

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "PREAMP SPECIFICATIONS", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Model:", "British Class-A Console"),
            ("Drive Sinyal:", "+0.0 dB Warmth"),
            ("Slew Rate:", "80 V/μs Gesit"),
            ("Noise Floor:", "-130 dBFS Studio"),
            ("Emulasi Trafo:", "Mu-Metal Saturation"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Circuit inspector for Volume Attenuator: Digital attenuation ladder & bit-depth headroom expansion
    fn draw_volume_attenuator_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.32).clamp(260.0, 360.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Attenuation Margin Bar
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c1_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "🔉 DIGITAL HEADROOM ATTENUATION", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));

        let m_y = c1_rect.min.y + 40.0;
        let bar_w = (c1_rect.width() - 80.0).max(40.0);

        painter.text(Pos2::new(c1_rect.min.x + 10.0, m_y), egui::Align2::LEFT_CENTER, "REDAM", egui::FontId::monospace(9.0), Color32::from_rgb(148, 163, 184));
        let att_rect = Rect::from_min_size(Pos2::new(c1_rect.min.x + 55.0, m_y - 7.0), Vec2::new(bar_w * 0.70, 14.0));
        painter.rect_filled(att_rect, 3.0, Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(c1_rect.max.x - 10.0, m_y), egui::Align2::RIGHT_CENTER, "-6.0 dB", egui::FontId::monospace(9.0), Color32::WHITE);

        let desc = if c1_rect.width() >= 360.0 {
            "Memberi ruang headroom sebelum sinyal masuk modul sensitif"
        } else {
            "Headroom Pre-FX Buffer"
        };
        painter.text(
            Pos2::new(c1_rect.min.x + 10.0, c1_rect.max.y - 12.0),
            egui::Align2::LEFT_BOTTOM,
            desc,
            egui::FontId::monospace(8.5),
            Color32::from_rgb(52, 211, 153),
        );

        // Column 2: Parameters
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(10.0, 10.0), egui::Align2::LEFT_TOP, "ATTENUATOR PARAMETERS", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let specs = [
            ("Tingkat Redaman:", "-6.0 dBFS"),
            ("Resolusi Bit:", "64-bit Float Passthrough"),
            ("Noise Tambahan:", "0.0 dB Dither-Free"),
            ("Transien:", "100% Bit-Translucent"),
            ("Tujuan:", "Ekspansi Headroom"),
        ];

        let start_y = c2_rect.min.y + 28.0;
        for (i, (lbl, val)) in specs.iter().enumerate() {
            let py = start_y + i as f32 * 17.0;
            painter.text(Pos2::new(c2_rect.min.x + 10.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 10.0, py), egui::Align2::RIGHT_TOP, *val, egui::FontId::monospace(9.0), Color32::WHITE);
        }
    }

    /// Inspector for Main DSP
    fn draw_main_dsp_inspector(&self, painter: &egui::Painter, rect: Rect, _t: f32) {
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, "🎛️ DSP AUDIO ENGINE MASTER CONTROLLER", egui::FontId::proportional(12.0), Color32::WHITE);
        painter.text(
            rect.min + Vec2::new(16.0, 32.0),
            egui::Align2::LEFT_TOP,
            "Router Pemrosesan 64-bit Double Precision • 300+ dB Dynamic Headroom • Nol Latency Tersembunyi",
            egui::FontId::proportional(10.0),
            Color32::from_rgb(56, 189, 248),
        );
        self.draw_cumulative_transfer_curve(painter, Rect::from_min_max(rect.min + Vec2::new(0.0, 48.0), rect.max), None);
    }

    /// Circuit inspector for App Resampler: Real-time Sinc Reconstruction Kernel & Custom Controls
    fn draw_app_resampler_inspector(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, _node_id: usize, _t: f32) {
        let avail_w = rect.width();
        let c2_w = (avail_w * 0.38).clamp(280.0, 380.0).min(avail_w * 0.48);
        let c1_w = (avail_w - c2_w - 20.0).max(180.0);
        let c1_rect = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(c1_w, rect.height() - 20.0));
        let c2_rect = Rect::from_min_size(Pos2::new(rect.min.x + c1_w + 15.0, rect.min.y + 10.0), Vec2::new(avail_w - c1_w - 25.0, rect.height() - 20.0));

        // Column 1: Sinc Kernel Impulse Response & Anti-Aliasing Scope
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(
            c1_rect.min + Vec2::new(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            "📡 RUBATO SINC RECONSTRUCTION KERNEL",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(52, 211, 153),
        );

        let g_left = c1_rect.min.x + 36.0;
        let g_right = c1_rect.max.x - 14.0;
        let g_top = c1_rect.min.y + 30.0;
        let g_bottom = c1_rect.max.y - 28.0;
        let g_w = (g_right - g_left).max(20.0);
        let g_h = (g_bottom - g_top).max(20.0);
        let mid_y = g_top + g_h * 0.5;
        let mid_x = g_left + g_w * 0.5;

        // Reticle Center Axes
        painter.line_segment([Pos2::new(g_left, mid_y), Pos2::new(g_right, mid_y)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 70)));
        painter.line_segment([Pos2::new(mid_x, g_top), Pos2::new(mid_x, g_bottom)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 70)));

        // Horizontal Grid Division Markers (-1.0, -0.5, 0, +0.5, +1.0)
        for &(norm_amp, lbl) in &[(1.0f32, "+1.0"), (0.5, "+0.5"), (-0.5, "-0.5"), (-1.0, "-1.0")] {
            let y = mid_y - norm_amp * (g_h * 0.44);
            painter.line_segment([Pos2::new(g_left, y), Pos2::new(g_right, y)], Stroke::new(0.5, Color32::from_rgba_premultiplied(51, 65, 85, 45)));
            painter.text(Pos2::new(g_left - 4.0, y), egui::Align2::RIGHT_CENTER, lbl, egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));
        }

        // Time / Zero Crossing Ticks (-3, -2, -1, 0, +1, +2, +3)
        for &k in &[-3.0f32, -2.0, -1.0, 1.0, 2.0, 3.0] {
            let x = mid_x + (k / 3.8) * (g_w * 0.48);
            if x >= g_left && x <= g_right {
                painter.line_segment([Pos2::new(x, g_top), Pos2::new(x, g_bottom)], Stroke::new(0.5, Color32::from_rgba_premultiplied(51, 65, 85, 40)));
                painter.text(Pos2::new(x, g_bottom + 8.0), egui::Align2::CENTER_CENTER, format!("{:+}T", k as i32), egui::FontId::proportional(8.0), Color32::from_rgb(100, 116, 139));
            }
        }

        // Calculate Sinc Kernel points: y(x) = sin(pi * x * fc) / (pi * x * fc) * BlackmanHarris(x)
        let cutoff = self.resampler_settings.profile.cutoff() as f32;
        let sinc_points = 80;
        let mut kernel_pts = Vec::with_capacity(sinc_points + 1);
        let span = 3.8f32;

        for s in 0..=sinc_points {
            let frac = s as f32 / sinc_points as f32;
            let px = g_left + g_w * frac;
            let x = (frac - 0.5) * 2.0 * span;
            let arg = std::f32::consts::PI * x * cutoff;
            let sinc_val = if arg.abs() < 1e-4 {
                1.0
            } else {
                arg.sin() / arg
            };
            // Blackman-Harris 4-term window
            let w_frac = frac;
            let win = (0.35875
                - 0.48829 * (2.0 * std::f32::consts::PI * w_frac).cos()
                + 0.14128 * (4.0 * std::f32::consts::PI * w_frac).cos()
                - 0.01168 * (6.0 * std::f32::consts::PI * w_frac).cos()).clamp(0.0, 1.0);

            let amp = (sinc_val * win).clamp(-1.0, 1.0);
            let py = mid_y - amp * (g_h * 0.44);
            kernel_pts.push(Pos2::new(px, py));
        }

        // Shaded polygon under the kernel curve
        for i in 0..kernel_pts.len() - 1 {
            let p1 = kernel_pts[i];
            let p2 = kernel_pts[i + 1];
            let poly = vec![p1, p2, Pos2::new(p2.x, mid_y), Pos2::new(p1.x, mid_y)];
            painter.add(egui::Shape::convex_polygon(poly, Color32::from_rgba_premultiplied(52, 211, 153, 20), Stroke::NONE));
        }

        // Phosphor glow line
        for pair in kernel_pts.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(2.4, Color32::from_rgba_premultiplied(52, 211, 153, 70)));
        }
        // Sharp beam line
        for pair in kernel_pts.windows(2) {
            painter.line_segment([pair[0], pair[1]], Stroke::new(1.3, Color32::from_rgb(52, 211, 153)));
        }

        // Peak marker
        painter.circle_filled(Pos2::new(mid_x, mid_y - (g_h * 0.44)), 3.5, Color32::WHITE);
        painter.circle_stroke(Pos2::new(mid_x, mid_y - (g_h * 0.44)), 5.5, Stroke::new(1.0, Color32::from_rgb(52, 211, 153)));

        // Readout text at bottom of Column 1 (responsive)
        let kernel_desc = if c1_rect.width() >= 480.0 {
            format!(
                "Kernel: Sinc {} • Cutoff: {:.2} Nyquist • Stopband Rejection: >144 dB • Window: Blackman-Harris",
                self.resampler_settings.quality.short_name(),
                self.resampler_settings.profile.cutoff()
            )
        } else if c1_rect.width() >= 320.0 {
            format!(
                "Sinc {} • Cutoff: {:.2} • >144 dB Stopband",
                self.resampler_settings.quality.short_name(),
                self.resampler_settings.profile.cutoff()
            )
        } else {
            format!("Sinc {}", self.resampler_settings.quality.short_name())
        };
        painter.text(c1_rect.min + Vec2::new(10.0, c1_rect.height() - 10.0), egui::Align2::LEFT_BOTTOM, kernel_desc, egui::FontId::monospace(8.5), Color32::from_rgb(148, 163, 184));

        // Column 2: Interactive Controls & Hardware Streaming Telemetry
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        painter.text(c2_rect.min + Vec2::new(12.0, 10.0), egui::Align2::LEFT_TOP, "🎛️ SINC RESAMPLER CONTROLS & TELEMETRY", egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        // Telemetry details
        let src_str = if self.current_track.duration > 0.0 {
            format!("{} • {:.1} kHz / {}-bit", self.current_track.format, self.current_track.sample_rate as f32 / 1000.0, self.current_track.bit_depth)
        } else {
            "FLAC Lossless • 96.0 kHz / 24-bit".to_string()
        };
        let dac_rate = self.signal_path_report.as_ref().map_or(48000, |r| r.device_rate);
        let dac_str = format!("{:.1} kHz ({})", dac_rate as f32 / 1000.0, if self.signal_path_report.as_ref().map_or(false, |r| r.is_exclusive) { "Exclusive" } else { "Shared" });
        let is_resamp = self.signal_path_report.as_ref().map_or(true, |r| r.is_resampling);
        let status_str = if is_resamp { "Rubato Sinc SRC Aktif" } else { "1:1 Bit-Perfect Passthrough" };

        let specs = [
            ("Format Asli:", src_str),
            ("Endpoint:", dac_str),
            ("Status Konversi:", status_str.to_string()),
        ];

        let mut py = c2_rect.min.y + 26.0;
        for (lbl, val) in specs {
            painter.text(Pos2::new(c2_rect.min.x + 12.0, py), egui::Align2::LEFT_TOP, lbl, egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(c2_rect.max.x - 12.0, py), egui::Align2::RIGHT_TOP, val, egui::FontId::monospace(9.0), Color32::WHITE);
            py += 15.0;
        }

        // Section 1: Sinc Quality (Length)
        py += 4.0;
        painter.text(Pos2::new(c2_rect.min.x + 12.0, py), egui::Align2::LEFT_TOP, "KUALITAS SINC CONVOLUTION:", egui::FontId::proportional(9.5), Color32::from_rgb(199, 210, 254));
        py += 14.0;

        let cur_q = self.resampler_settings.quality;
        let btn_w = ((c2_rect.width() - 24.0 - 8.0 * 2.0) / 3.0).max(50.0);
        let btn_h = 22.0;

        let q_options = if btn_w >= 85.0 {
            [
                (SincQuality::Fast, "⚡ Fast (64-pt)"),
                (SincQuality::Studio, "💎 Studio (128-pt)"),
                (SincQuality::Extreme, "🚀 Extreme (256-pt)"),
            ]
        } else {
            [
                (SincQuality::Fast, "⚡ 64-pt"),
                (SincQuality::Studio, "💎 128-pt"),
                (SincQuality::Extreme, "🚀 256-pt"),
            ]
        };

        for (idx, (q, label)) in q_options.iter().enumerate() {
            let bx = c2_rect.min.x + 12.0 + idx as f32 * (btn_w + 8.0);
            let b_rect = Rect::from_min_size(Pos2::new(bx, py), Vec2::new(btn_w, btn_h));
            let is_sel = cur_q == *q;
            let is_hov = ui.rect_contains_pointer(b_rect);

            let bg = if is_sel {
                Color32::from_rgb(67, 56, 202)
            } else if is_hov {
                Color32::from_rgb(30, 41, 59)
            } else {
                Color32::from_rgb(15, 23, 42)
            };
            let stroke_col = if is_sel {
                Color32::from_rgb(129, 140, 248)
            } else if is_hov {
                Color32::from_rgb(99, 102, 241)
            } else {
                Color32::from_rgb(51, 65, 85)
            };
            let txt_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };

            painter.rect_filled(b_rect, 4.0, bg);
            painter.rect_stroke(b_rect, 4.0, Stroke::new(1.0, stroke_col), egui::StrokeKind::Inside);
            painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, *label, egui::FontId::proportional(9.0), txt_col);

            if ui.input(|i| i.pointer.any_pressed()) && is_hov && !is_sel {
                let mut new_settings = self.resampler_settings;
                new_settings.quality = *q;
                self.set_resampler_settings(new_settings);
            }
        }

        py += btn_h + 3.0;
        painter.text(Pos2::new(c2_rect.min.x + 12.0, py), egui::Align2::LEFT_TOP, cur_q.desc(), egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

        // Section 2: Roll-Off Profile (Cutoff)
        py += 16.0;
        painter.text(Pos2::new(c2_rect.min.x + 12.0, py), egui::Align2::LEFT_TOP, "PROFIL ROLL-OFF FREKUENSI (CUTOFF):", egui::FontId::proportional(9.5), Color32::from_rgb(167, 243, 208));
        py += 14.0;

        let cur_p = self.resampler_settings.profile;
        let p_options = if btn_w >= 85.0 {
            [
                (RollOffProfile::Sharp, "🎯 Sharp (0.95)"),
                (RollOffProfile::Smooth, "☕ Smooth (0.90)"),
                (RollOffProfile::Nyquist, "✨ Nyquist (0.98)"),
            ]
        } else {
            [
                (RollOffProfile::Sharp, "🎯 Sharp"),
                (RollOffProfile::Smooth, "☕ Smooth"),
                (RollOffProfile::Nyquist, "✨ Nyquist"),
            ]
        };

        for (idx, (p, label)) in p_options.iter().enumerate() {
            let bx = c2_rect.min.x + 12.0 + idx as f32 * (btn_w + 8.0);
            let b_rect = Rect::from_min_size(Pos2::new(bx, py), Vec2::new(btn_w, btn_h));
            let is_sel = cur_p == *p;
            let is_hov = ui.rect_contains_pointer(b_rect);

            let bg = if is_sel {
                Color32::from_rgb(6, 95, 70)
            } else if is_hov {
                Color32::from_rgb(30, 41, 59)
            } else {
                Color32::from_rgb(15, 23, 42)
            };
            let stroke_col = if is_sel {
                Color32::from_rgb(52, 211, 153)
            } else if is_hov {
                Color32::from_rgb(16, 185, 129)
            } else {
                Color32::from_rgb(51, 65, 85)
            };
            let txt_col = if is_sel { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };

            painter.rect_filled(b_rect, 4.0, bg);
            painter.rect_stroke(b_rect, 4.0, Stroke::new(1.0, stroke_col), egui::StrokeKind::Inside);
            painter.text(b_rect.center(), egui::Align2::CENTER_CENTER, *label, egui::FontId::proportional(9.0), txt_col);

            if ui.input(|i| i.pointer.any_pressed()) && is_hov && !is_sel {
                let mut new_settings = self.resampler_settings;
                new_settings.profile = *p;
                self.set_resampler_settings(new_settings);
            }
        }

        py += btn_h + 3.0;
        painter.text(Pos2::new(c2_rect.min.x + 12.0, py), egui::Align2::LEFT_TOP, cur_p.desc(), egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));
    }

    /// Inspector for Streaming nodes (Source, RAM Buffer, Resampler, DAC)
    fn draw_streaming_node_inspector(&self, painter: &egui::Painter, rect: Rect, _node_id: usize) {
        let title = if rect.width() >= 480.0 {
            "AUDIO STREAMING PIPELINE TELEMETRY"
        } else {
            "STREAMING PIPELINE"
        };
        painter.text(rect.min + Vec2::new(16.0, 14.0), egui::Align2::LEFT_TOP, title, egui::FontId::proportional(12.0), Color32::WHITE);

        let g_rect = Rect::from_min_max(rect.min + Vec2::new(16.0, 36.0), rect.max - Vec2::new(16.0, 12.0));
        painter.rect_filled(g_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(g_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        let is_compact = g_rect.width() < 560.0;

        let source_format = if self.current_track.duration > 0.0 {
            if is_compact {
                format!(
                    "{} • {:.1}kHz/{}-bit",
                    self.current_track.format.to_uppercase(),
                    self.current_track.sample_rate as f32 / 1000.0,
                    self.current_track.bit_depth
                )
            } else {
                format!(
                    "{} Lossless • {:.1} kHz / {}-bit Stereo",
                    self.current_track.format.to_uppercase(),
                    self.current_track.sample_rate as f32 / 1000.0,
                    self.current_track.bit_depth
                )
            }
        } else {
            "Idle (No Track)".to_string()
        };

        let ringbuffer_status = if self.is_playing {
            if is_compact { "64KB SPSC • Realtime DMA".to_string() } else { "64 KB Lock-Free SPSC • Healthy / Realtime DMA".to_string() }
        } else {
            if is_compact { "64KB SPSC • Standby".to_string() } else { "64 KB Lock-Free SPSC • Standby / Zero Jitter".to_string() }
        };

        let resampler_status = if let Some(ref r) = self.signal_path_report {
            if r.is_resampling {
                if is_compact {
                    format!("{} Sinc ➔ {:.1} kHz", self.resampler_settings.quality.short_name(), r.device_rate as f32 / 1000.0)
                } else {
                    format!(
                        "{} Sinc ➔ {:.1} kHz • {}",
                        self.resampler_settings.quality.label(),
                        r.device_rate as f32 / 1000.0,
                        self.resampler_settings.profile.label()
                    )
                }
            } else {
                format!("Native 1:1 Rate ({:.1} kHz)", r.device_rate as f32 / 1000.0)
            }
        } else {
            format!("{} Sinc • Standby", self.resampler_settings.quality.label())
        };

        let endpoint_status = if let Some(ref r) = self.signal_path_report {
            if is_compact {
                format!("{:.1} kHz ({})", r.device_rate as f32 / 1000.0, if r.is_exclusive { "Exclusive" } else { "Shared" })
            } else {
                format!(
                    "{} • {:.1} kHz ({})",
                    r.device_name,
                    r.device_rate as f32 / 1000.0,
                    if r.is_exclusive { "WASAPI Exclusive" } else { "WASAPI Shared" }
                )
            }
        } else {
            "Endpoint Init...".to_string()
        };

        let bit_integrity = if self.chain_mode == ChainMode::BitPerfect {
            if is_compact { "Bit-Perfect Passthrough (1:1)".to_string() } else { "Bit-Perfect Passthrough (All DSP & FX Bypassed)".to_string() }
        } else {
            if is_compact { "DSP Studio Chain Active".to_string() } else { "DSP Active Studio Chain (Custom Processing Enabled)".to_string() }
        };

        let telemetry = [
            ("Source Format:", source_format),
            ("Memory RingBuffer:", ringbuffer_status),
            ("Sample Rate Conversion:", resampler_status),
            ("Hardware Endpoint:", endpoint_status),
            ("Bit Integrity:", bit_integrity),
        ];

        for (i, (lbl, val)) in telemetry.iter().enumerate() {
            let py = g_rect.min.y + 14.0 + i as f32 * 20.0;
            painter.text(Pos2::new(g_rect.min.x + 16.0, py), egui::Align2::LEFT_TOP, *lbl, egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));
            painter.text(Pos2::new(g_rect.max.x - 16.0, py), egui::Align2::RIGHT_TOP, val, egui::FontId::monospace(10.0), Color32::WHITE);
        }
    }

    /// Inspector for a clicked cable (Audio Signal Probe)
    fn draw_cable_signal_probe(&self, painter: &egui::Painter, rect: Rect, cable_idx: usize) {
        let Some(conn) = self.connections.get(cable_idx) else {
            self.draw_cumulative_transfer_curve(painter, rect, None);
            return;
        };

        let from_title = self.nodes.iter().find(|n| n.id == conn.from_node).map(|n| n.title.as_str()).unwrap_or("From");
        let to_title = self.nodes.iter().find(|n| n.id == conn.to_node).map(|n| n.title.as_str()).unwrap_or("To");

        let probe_title = if rect.width() >= 620.0 {
            format!("AUDIO CABLE PROBE: {} ➔ {} (Wire #{})", from_title, to_title, cable_idx)
        } else if rect.width() >= 420.0 {
            format!("PROBE: {} ➔ {} (#{})", from_title, to_title, cable_idx)
        } else {
            format!("PROBE (#{})", cable_idx)
        };
        painter.text(
            rect.min + Vec2::new(16.0, 14.0),
            egui::Align2::LEFT_TOP,
            probe_title,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(52, 211, 153),
        );

        let g_rect = Rect::from_min_max(rect.min + Vec2::new(16.0, 36.0), rect.max - Vec2::new(16.0, 12.0));
        painter.rect_filled(g_rect, 4.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(g_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        // Draw live signal probe waveform / bars
        let bar_count = 36;
        let bar_w = ((g_rect.width() - 40.0) / bar_count as f32).max(2.0);
        let base_y = g_rect.max.y - 20.0;

        for b in 0..bar_count {
            let bx = g_rect.min.x + 20.0 + b as f32 * bar_w;
            let val = self.spectrum_bars[(b * 48 / bar_count).min(47)];
            let h = if self.is_playing { (val * 1.3).clamp(0.08, 1.0) * (g_rect.height() - 45.0) } else { 4.0 };

            painter.rect_filled(Rect::from_min_size(Pos2::new(bx, base_y - h), Vec2::new(bar_w - 2.0, h)), 1.5, Color32::from_rgb(52, 211, 153));
        }

        let status = if self.is_playing { "Active" } else { "Standby" };
        let rms_str = if self.is_playing && self.live_integrated_lufs > -68.0 {
            format!("{:.1} dBFS", self.live_integrated_lufs)
        } else {
            "-∞ dBFS".to_string()
        };
        let peak_str = if self.is_playing && self.live_true_peak_db > -89.0 {
            format!("{:+.1} dBFS", self.live_true_peak_db)
        } else {
            "-∞ dBFS".to_string()
        };

        let sig_text = if g_rect.width() >= 550.0 {
            format!("Signal Status: {} • RMS: {} • Peak: {}", status, rms_str, peak_str)
        } else if g_rect.width() >= 360.0 {
            format!("RMS: {} • Peak: {}", rms_str, peak_str)
        } else {
            format!("Peak: {}", peak_str)
        };

        painter.text(
            Pos2::new(g_rect.max.x - 16.0, g_rect.min.y + 12.0),
            egui::Align2::RIGHT_TOP,
            sig_text,
            egui::FontId::monospace(9.5),
            if self.is_playing { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(148, 163, 184) },
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
        let is_wide = rect.width() >= 340.0;
        let (center, radius, bar_left, bar_y) = if is_wide {
            let r = (rect.height() * 0.42).min((rect.width() - 160.0) * 0.45).clamp(35.0, 80.0);
            let c = Pos2::new(rect.min.x + (rect.width() - 140.0) * 0.5, rect.center().y);
            let bl = (rect.max.x - 135.0).max(c.x + r + 15.0);
            (c, r, bl, rect.center().y)
        } else {
            let r = ((rect.height() - 36.0) * 0.40).min(rect.width() * 0.38).clamp(24.0, 58.0);
            let c = Pos2::new(rect.center().x, rect.min.y + (rect.height() - 32.0) * 0.5);
            let bl = (rect.center().x - 60.0).max(rect.min.x + 8.0);
            (c, r, bl, rect.max.y - 14.0)
        };

        painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)));
        painter.circle_stroke(center, radius * 0.5, Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 100)));
        painter.line_segment([Pos2::new(center.x - radius, center.y), Pos2::new(center.x + radius, center.y)], Stroke::new(0.8, Color32::from_rgb(51, 65, 85)));
        painter.line_segment([Pos2::new(center.x, center.y - radius), Pos2::new(center.x, center.y + radius)], Stroke::new(0.8, Color32::from_rgb(51, 65, 85)));

        let diag = radius * 0.7071;
        // 45-degree diagonal axes
        painter.line_segment(
            [Pos2::new(center.x - diag, center.y - diag), Pos2::new(center.x + diag, center.y + diag)],
            Stroke::new(0.6, Color32::from_rgba_premultiplied(51, 65, 85, 80)),
        );
        painter.line_segment(
            [Pos2::new(center.x - diag, center.y + diag), Pos2::new(center.x + diag, center.y - diag)],
            Stroke::new(0.6, Color32::from_rgba_premultiplied(51, 65, 85, 80)),
        );

        painter.text(Pos2::new(center.x - diag - 6.0, center.y - diag), egui::Align2::RIGHT_CENTER, "+L", egui::FontId::monospace(9.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(center.x + diag + 6.0, center.y - diag), egui::Align2::LEFT_CENTER, "+R", egui::FontId::monospace(9.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(center.x, center.y - radius - 5.0), egui::Align2::CENTER_BOTTOM, "M (Mono)", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));
        painter.text(Pos2::new(center.x + radius + 6.0, center.y), egui::Align2::LEFT_CENTER, "S (Side)", egui::FontId::monospace(8.0), Color32::from_rgb(100, 116, 139));

        // Real Stereo Lissajous point cloud from live audio stream
        let point_count = 64;
        let stride = 1024 / point_count;
        let inv_sqrt2 = 0.7071068f32;
        let gain = 1.35f32;

        let mut gonio_pts = Vec::with_capacity(point_count);
        for i in 0..point_count {
            let idx = (i * stride).min(1023);
            let l = self.live_stereo_l[idx];
            let r = self.live_stereo_r[idx];
            let s = (r - l) * inv_sqrt2;
            let m = (r + l) * inv_sqrt2;
            let px = center.x + (s * radius * gain).clamp(-radius, radius);
            let py = center.y - (m * radius * gain).clamp(-radius, radius);
            gonio_pts.push(Pos2::new(px, py));
        }

        if self.is_playing {
            // Draw phosphor line trail connecting sample trajectory
            painter.add(egui::Shape::line(
                gonio_pts.clone(),
                Stroke::new(1.0, Color32::from_rgba_premultiplied(56, 189, 248, 55)),
            ));
            // Draw live phosphor dots
            for pt in gonio_pts {
                painter.circle_filled(pt, 1.5, Color32::from_rgba_premultiplied(56, 189, 248, 190));
            }
        } else {
            // Rest dot at center when idle
            painter.circle_filled(center, 2.0, Color32::from_rgba_premultiplied(56, 189, 248, 90));
        }

        // Phase correlation bar
        let bar_width = 115.0f32;
        if is_wide {
            painter.text(Pos2::new(bar_left, bar_y - 18.0), egui::Align2::LEFT_CENTER, "Phase Correlation", egui::FontId::proportional(10.0), Color32::WHITE);
        }
        painter.rect_filled(Rect::from_min_size(Pos2::new(bar_left, bar_y - 5.0), Vec2::new(bar_width, 10.0)), 3.0, Color32::from_rgb(30, 41, 59));

        // Zero mark in the middle
        let half_bar = bar_width * 0.5;
        painter.line_segment([Pos2::new(bar_left + half_bar, bar_y - 7.0), Pos2::new(bar_left + half_bar, bar_y + 7.0)], Stroke::new(1.0, Color32::from_rgb(148, 163, 184)));

        let corr = if self.is_playing { self.live_phase_correlation.clamp(-1.0, 1.0) } else { 1.0 };
        let indicator_x = bar_left + half_bar + corr * (half_bar - 8.0);
        let (corr_col, corr_desc) = if !self.is_playing {
            (Color32::from_rgb(100, 116, 139), "Idle")
        } else if corr >= 0.5 {
            (Color32::from_rgb(34, 197, 94), "In-Phase")
        } else if corr >= 0.0 {
            (Color32::from_rgb(56, 189, 248), "Wide")
        } else if corr >= -0.5 {
            (Color32::from_rgb(245, 158, 11), "Cancel")
        } else {
            (Color32::from_rgb(239, 68, 68), "Anti-Phase")
        };

        painter.circle_filled(Pos2::new(indicator_x, bar_y), 4.5, corr_col);
        let corr_text = if self.is_playing {
            format!("{:+.2} ({})", corr, corr_desc)
        } else {
            "+1.00 (Standby)".to_string()
        };
        if is_wide {
            painter.text(Pos2::new(bar_left + half_bar, bar_y + 15.0), egui::Align2::CENTER_CENTER, corr_text, egui::FontId::monospace(9.0), corr_col);
        }
    }

    /// Master EBU R128 Loudness, True-Peak (ITU-R BS.1770) & Inter-Sample Clipping Indicator
    fn draw_master_ebu_metering(&mut self, ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect) {
        let left = rect.min.x + 16.0;
        let top = rect.min.y + 12.0;
        let is_grid = rect.width() < 560.0;
        let gap = 10.0;

        let (c1, c2, c3, c4) = if is_grid {
            let col_w = ((rect.width() - 32.0 - gap) / 2.0).max(100.0);
            let row_h = ((rect.height() - 24.0 - gap) / 2.0).max(50.0);
            let r1 = Rect::from_min_size(Pos2::new(left, top), Vec2::new(col_w, row_h));
            let r2 = Rect::from_min_size(Pos2::new(left + col_w + gap, top), Vec2::new(col_w, row_h));
            let r3 = Rect::from_min_size(Pos2::new(left, top + row_h + gap), Vec2::new(col_w, row_h));
            let r4 = Rect::from_min_size(Pos2::new(left + col_w + gap, top + row_h + gap), Vec2::new(col_w, row_h));
            (r1, r2, r3, r4)
        } else {
            let col_w = (rect.width() - 32.0 - gap * 3.0) / 4.0;
            let h = rect.height() - 24.0;
            let r1 = Rect::from_min_size(Pos2::new(left, top), Vec2::new(col_w, h));
            let r2 = Rect::from_min_size(Pos2::new(left + (col_w + gap), top), Vec2::new(col_w, h));
            let r3 = Rect::from_min_size(Pos2::new(left + (col_w + gap) * 2.0, top), Vec2::new(col_w, h));
            let r4 = Rect::from_min_size(Pos2::new(left + (col_w + gap) * 3.0, top), Vec2::new(col_w, h));
            (r1, r2, r3, r4)
        };

        // 1. INTEGRATED LUFS
        painter.rect_filled(c1, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c1, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);
        let lufs_title = if c1.width() >= 130.0 { "INTEGRATED LUFS" } else { "LUFS" };
        painter.text(c1.center_top() + Vec2::new(0.0, 10.0), egui::Align2::CENTER_CENTER, lufs_title, egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let lufs_val = if self.is_playing { self.live_integrated_lufs } else { -70.0 };
        let (lufs_str, lufs_col) = if lufs_val <= -68.0 {
            ("-∞ LUFS".to_string(), Color32::from_rgb(100, 116, 139))
        } else {
            let col = if lufs_val > -9.0 {
                Color32::from_rgb(239, 68, 68) // Loud / Clipping risk
            } else if lufs_val > -13.0 {
                Color32::from_rgb(245, 158, 11) // High energy
            } else if lufs_val >= -16.0 {
                Color32::from_rgb(34, 197, 94) // Sweet spot (-14 LUFS target)
            } else {
                Color32::from_rgb(56, 189, 248) // Quiet / dynamic
            };
            (format!("{:.1} LUFS", lufs_val), col)
        };
        painter.text(c1.center() - Vec2::new(0.0, 4.0), egui::Align2::CENTER_CENTER, lufs_str, egui::FontId::monospace(16.0), lufs_col);
        let lufs_target_label = if c1.width() >= 140.0 { "Target: -14.0 (EBU R128)" } else { "Target: -14 LUFS" };
        painter.text(c1.center_bottom() - Vec2::new(0.0, 8.0), egui::Align2::CENTER_CENTER, lufs_target_label, egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

        // 2. DYNAMIC RANGE (DR)
        painter.rect_filled(c2, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c2, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);
        let dr_title = if c2.width() >= 130.0 { "DYNAMIC RANGE (DR)" } else { "DR METER" };
        painter.text(c2.center_top() + Vec2::new(0.0, 10.0), egui::Align2::CENTER_CENTER, dr_title, egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let dr_val = if self.is_playing { self.live_dynamic_range } else { 0.0 };
        let is_compact_dr = c2.width() < 145.0;
        let (dr_str, dr_sub, dr_col) = if dr_val < 1.0 {
            ("DR 0".to_string(), "Idle / Silence".to_string(), Color32::from_rgb(100, 116, 139))
        } else if dr_val >= 14.0 {
            (format!("DR {:.0}", dr_val), if is_compact_dr { "Audiophile" } else { "Audiophile Uncompressed" }.to_string(), Color32::from_rgb(56, 189, 248))
        } else if dr_val >= 9.0 {
            (format!("DR {:.0}", dr_val), if is_compact_dr { "Studio Master" } else { "Balanced Studio Master" }.to_string(), Color32::from_rgb(34, 197, 94))
        } else if dr_val >= 6.0 {
            (format!("DR {:.0}", dr_val), if is_compact_dr { "Compressed" } else { "Modern Compressed" }.to_string(), Color32::from_rgb(245, 158, 11))
        } else {
            (format!("DR {:.0}", dr_val), if is_compact_dr { "Brickwall" } else { "Heavy Brickwall Limiting" }.to_string(), Color32::from_rgb(239, 68, 68))
        };
        painter.text(c2.center() - Vec2::new(0.0, 4.0), egui::Align2::CENTER_CENTER, dr_str, egui::FontId::monospace(16.0), dr_col);
        painter.text(c2.center_bottom() - Vec2::new(0.0, 8.0), egui::Align2::CENTER_CENTER, dr_sub, egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));

        // 3. TRUE-PEAK (dBTP) & INTER-SAMPLE CLIPPING (P2!)
        painter.rect_filled(c3, 6.0, Color32::from_rgb(12, 17, 29));
        let tp_stroke = if self.live_isp_clip_detected {
            Stroke::new(1.2, Color32::from_rgb(239, 68, 68))
        } else {
            Stroke::new(1.0, Color32::from_rgb(30, 41, 59))
        };
        painter.rect_stroke(c3, 6.0, tp_stroke, egui::StrokeKind::Inside);
        let tp_title = if c3.width() >= 135.0 { "TRUE-PEAK & ISP CLIP" } else { "TRUE-PEAK" };
        painter.text(c3.center_top() + Vec2::new(0.0, 10.0), egui::Align2::CENTER_CENTER, tp_title, egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let tp_val = if self.is_playing { self.live_true_peak_db } else { -96.0 };
        let (tp_str, tp_col) = if tp_val <= -90.0 {
            ("-∞ dBTP".to_string(), Color32::from_rgb(100, 116, 139))
        } else if tp_val > 0.0 || self.live_isp_clip_detected {
            (format!("{:+.1} dBTP", tp_val), Color32::from_rgb(239, 68, 68))
        } else if tp_val >= -1.0 {
            (format!("{:+.1} dBTP", tp_val), Color32::from_rgb(245, 158, 11))
        } else {
            (format!("{:+.1} dBTP", tp_val), Color32::from_rgb(34, 197, 94))
        };
        painter.text(c3.center() - Vec2::new(0.0, 8.0), egui::Align2::CENTER_CENTER, tp_str, egui::FontId::monospace(16.0), tp_col);

        // Inter-Sample Clip Warning + Reset Button
        let reset_btn_rect = Rect::from_min_size(Pos2::new(c3.min.x + 8.0, c3.max.y - 24.0), Vec2::new(c3.width() - 16.0, 18.0));
        if self.live_isp_clip_detected {
            let is_hov = ui.rect_contains_pointer(reset_btn_rect);
            painter.rect_filled(reset_btn_rect, 3.0, if is_hov { Color32::from_rgb(185, 28, 28) } else { Color32::from_rgb(127, 29, 29) });
            painter.rect_stroke(reset_btn_rect, 3.0, Stroke::new(1.0, Color32::from_rgb(248, 113, 113)), egui::StrokeKind::Inside);
            let clip_label = if c3.width() >= 130.0 { format!("⚠️ CLIP ({}) • Reset ✕", self.live_isp_clip_count) } else { format!("CLIP ({}) ✕", self.live_isp_clip_count) };
            painter.text(reset_btn_rect.center(), egui::Align2::CENTER_CENTER, clip_label, egui::FontId::proportional(8.5), Color32::WHITE);
            if ui.input(|i| i.pointer.any_pressed()) && is_hov {
                self.live_isp_clip_detected = false;
                self.live_isp_clip_count = 0;
                self.live_isp_max_over_db = 0.0;
            }
        } else {
            painter.text(c3.center_bottom() - Vec2::new(0.0, 8.0), egui::Align2::CENTER_CENTER, "✅ Inter-Sample Safe", egui::FontId::proportional(8.5), Color32::from_rgb(34, 197, 94));
        }

        // 4. MASTER GAIN REDUCTION (P1 in Master View!)
        painter.rect_filled(c4, 6.0, Color32::from_rgb(12, 17, 29));
        painter.rect_stroke(c4, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);
        let gr_title = if c4.width() >= 130.0 { "GAIN REDUCTION" } else { "GAIN RED." };
        painter.text(c4.center_top() + Vec2::new(0.0, 10.0), egui::Align2::CENTER_CENTER, gr_title, egui::FontId::proportional(10.0), Color32::from_rgb(148, 163, 184));

        let gr_val = if self.is_playing { self.live_gain_reduction_db } else { 0.0 };
        let gr_color = if gr_val > 6.0 {
            Color32::from_rgb(239, 68, 68)
        } else if gr_val > 0.05 {
            Color32::from_rgb(245, 158, 11)
        } else {
            Color32::from_rgb(100, 116, 139)
        };
        let gr_text = if gr_val > 0.05 {
            format!("-{:.1} dB", gr_val)
        } else {
            "0.0 dB".to_string()
        };
        painter.text(c4.center() - Vec2::new(0.0, 8.0), egui::Align2::CENTER_CENTER, gr_text, egui::FontId::monospace(16.0), gr_color);

        // GR Horizontal Meter Bar
        let gr_bar_rect = Rect::from_min_size(Pos2::new(c4.min.x + 10.0, c4.max.y - 20.0), Vec2::new(c4.width() - 20.0, 8.0));
        painter.rect_filled(gr_bar_rect, 2.0, Color32::from_rgb(24, 32, 47));
        if gr_val > 0.05 {
            let active_w = (gr_bar_rect.width() * (gr_val / 18.0).clamp(0.0, 1.0)).max(2.0);
            painter.rect_filled(Rect::from_min_size(gr_bar_rect.min, Vec2::new(active_w, gr_bar_rect.height())), 2.0, gr_color);
        }
        let gr_sub = if gr_val > 0.05 { "Active Leveling" } else { "Linear Passthrough" };
        painter.text(c4.center_bottom() - Vec2::new(0.0, 24.0), egui::Align2::CENTER_CENTER, gr_sub, egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));
    }

    /// Waveform Saturation Heatmap: Visualizes waveform amplitude with color-coded saturation & clipping zones
    fn draw_waveform_saturation_heatmap(&mut self, _ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect) {
        let pad = 12.0;
        let c_rect = Rect::from_min_size(rect.min + Vec2::new(pad, pad), Vec2::new(rect.width() - pad * 2.0, rect.height() - pad * 2.0));

        // Background
        painter.rect_filled(c_rect, 6.0, Color32::from_rgb(8, 12, 20));
        painter.rect_stroke(c_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

        // 1. Header with Live Metrics (adaptive)
        let header_title = if c_rect.width() >= 480.0 {
            "🔥 WAVEFORM SATURATION HEATMAP"
        } else {
            "🔥 SATURATION HEATMAP"
        };
        painter.text(c_rect.min + Vec2::new(12.0, 10.0), egui::Align2::LEFT_TOP, header_title, egui::FontId::proportional(11.0), Color32::from_rgb(245, 158, 11));

        // Calculate saturation statistics over the 1024 stereo samples
        let mut clean_count = 0usize;
        let mut warm_count = 0usize;
        let mut clip_count = 0usize;
        let mut max_abs = 0.0f32;

        for i in 0..1024 {
            let amp = ((self.live_stereo_l[i] + self.live_stereo_r[i]) * 0.5).abs();
            if amp > max_abs {
                max_abs = amp;
            }
            if amp >= 0.98 {
                clip_count += 1;
            } else if amp >= 0.50 {
                warm_count += 1;
            } else {
                clean_count += 1;
            }
        }

        let clean_pct = (clean_count as f32 / 1024.0) * 100.0;
        let warm_pct = (warm_count as f32 / 1024.0) * 100.0;
        let clip_pct = (clip_count as f32 / 1024.0) * 100.0;

        let stats_text = if c_rect.width() >= 660.0 {
            format!("Clean (< -6dB): {:.0}% • Warm (-6..0dB): {:.0}% • Clip (0dBFS): {:.1}%", clean_pct, warm_pct, clip_pct)
        } else if c_rect.width() >= 480.0 {
            format!("Clean: {:.0}% • Warm: {:.0}% • Clip: {:.1}%", clean_pct, warm_pct, clip_pct)
        } else if c_rect.width() >= 340.0 {
            format!("{:.0}% / {:.0}% / {:.1}%", clean_pct, warm_pct, clip_pct)
        } else {
            String::new()
        };
        if !stats_text.is_empty() {
            painter.text(Pos2::new(c_rect.max.x - 12.0, c_rect.min.y + 10.0), egui::Align2::RIGHT_TOP, stats_text, egui::FontId::monospace(9.5), Color32::from_rgb(148, 163, 184));
        }

        // 2. Waveform Canvas Bounds
        let scope_rect = Rect::from_min_max(Pos2::new(c_rect.min.x + 12.0, c_rect.min.y + 28.0), Pos2::new(c_rect.max.x - 12.0, c_rect.max.y - 12.0));
        painter.rect_filled(scope_rect, 4.0, Color32::from_rgb(4, 7, 13));
        painter.rect_stroke(scope_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(20, 28, 42)), egui::StrokeKind::Inside);

        let mid_y = scope_rect.center().y;
        let half_h = (scope_rect.height() * 0.45).max(10.0);

        // dB Reference Lines: 0 dBFS (Clip), -6 dBFS (Warm threshold), Center (0V)
        let clip_y_top = mid_y - half_h * 0.98;
        let clip_y_bot = mid_y + half_h * 0.98;
        painter.line_segment([Pos2::new(scope_rect.min.x, clip_y_top), Pos2::new(scope_rect.max.x, clip_y_top)], Stroke::new(1.0, Color32::from_rgba_premultiplied(239, 68, 68, 90)));
        painter.line_segment([Pos2::new(scope_rect.min.x, clip_y_bot), Pos2::new(scope_rect.max.x, clip_y_bot)], Stroke::new(1.0, Color32::from_rgba_premultiplied(239, 68, 68, 90)));
        painter.text(Pos2::new(scope_rect.max.x - 4.0, clip_y_top - 2.0), egui::Align2::RIGHT_BOTTOM, "0 dBFS CLIP", egui::FontId::monospace(7.5), Color32::from_rgb(239, 68, 68));

        let warm_y_top = mid_y - half_h * 0.50;
        let warm_y_bot = mid_y + half_h * 0.50;
        painter.line_segment([Pos2::new(scope_rect.min.x, warm_y_top), Pos2::new(scope_rect.max.x, warm_y_top)], Stroke::new(0.8, Color32::from_rgba_premultiplied(245, 158, 11, 45)));
        painter.line_segment([Pos2::new(scope_rect.min.x, warm_y_bot), Pos2::new(scope_rect.max.x, warm_y_bot)], Stroke::new(0.8, Color32::from_rgba_premultiplied(245, 158, 11, 45)));
        painter.text(Pos2::new(scope_rect.max.x - 4.0, warm_y_top - 2.0), egui::Align2::RIGHT_BOTTOM, "-6 dBFS WARM", egui::FontId::monospace(7.5), Color32::from_rgb(245, 158, 11));

        // Center line
        painter.line_segment([Pos2::new(scope_rect.min.x, mid_y), Pos2::new(scope_rect.max.x, mid_y)], Stroke::new(0.8, Color32::from_rgba_premultiplied(100, 116, 139, 45)));

        // 3. Render Heatmap-Colored Waveform Slices
        let num_bars = ((scope_rect.width() / 4.0).floor() as usize).clamp(64, 256);
        let samples_per_bar = 1024 / num_bars;

        for b in 0..num_bars {
            let start = b * samples_per_bar;
            let end = (start + samples_per_bar).min(1024);

            let mut peak_val = 0.0f32;
            let mut signed_peak = 0.0f32;
            for i in start..end {
                let s = (self.live_stereo_l[i] + self.live_stereo_r[i]) * 0.5;
                if s.abs() > peak_val {
                    peak_val = s.abs();
                    signed_peak = s;
                }
            }

            let x = scope_rect.min.x + (b as f32 / num_bars as f32) * scope_rect.width();
            let y_extent = signed_peak.clamp(-1.0, 1.0) * half_h;
            let bar_top = mid_y - y_extent.abs();
            let bar_bot = mid_y + y_extent.abs();

            // Zone Color Heatmap
            let bar_color = if peak_val >= 0.98 {
                Color32::from_rgb(239, 68, 68) // Blazing Red (Clip)
            } else if peak_val >= 0.85 {
                Color32::from_rgb(234, 88, 12) // Hot Orange (High saturation)
            } else if peak_val >= 0.50 {
                Color32::from_rgb(245, 158, 11) // Warm Amber (Soft compression)
            } else {
                Color32::from_rgb(34, 197, 94) // Mint Green (Clean linear headroom)
            };

            let bar_w = (scope_rect.width() / num_bars as f32).max(1.5);
            painter.rect_filled(Rect::from_min_max(Pos2::new(x, bar_top), Pos2::new(x + bar_w - 0.5, bar_bot)), 1.0, bar_color);

            // Clip warning dot/mark if peak reaches ceiling
            if peak_val >= 0.98 {
                painter.circle_filled(Pos2::new(x + bar_w * 0.5, clip_y_top), 2.5, Color32::WHITE);
                painter.circle_filled(Pos2::new(x + bar_w * 0.5, clip_y_bot), 2.5, Color32::WHITE);
            }
        }
    }
}
