use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::LunulaApp;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdvanceGraphMode {
    CumulativeCurve,   // Net Cumulative EQ/Filter Transfer Curve
    DualSpectrumRta,   // Pre vs Post Real-time FFT Spectrum
    StereoVectorscope, // Lissajous Goniometer & Phase Correlation
    MasterMetering,    // EBU R128 LUFS, True-Peak & Dynamic Range
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdvanceRackTab {
    RackUnits,     // Modular Hardware Channel Strips
    SignalMatrix,  // 6x6 Pro Audio Routing Crossbar
    MasterSection, // Calibration, Clock & Dither
}

pub struct AdvanceStudioState {
    pub graph_mode: AdvanceGraphMode,
    pub rack_tab: AdvanceRackTab,
    pub selected_rack_unit: usize,
    pub freeze_graph: bool,
    pub show_grid_db: bool,
    pub bypass_all_fx: bool,
    pub ab_state: bool, // false = A (Dry), true = B (Wet)
    pub matrix_routes: [[bool; 6]; 6],

    // Rack Unit 1: Tube Preamp
    pub tube_power: bool,
    pub tube_drive_db: f32,
    pub tube_warmth: f32,
    pub tube_triode_mode: bool,

    // Rack Unit 2: Precision Dynamic EQ
    pub eq_power: bool,
    pub eq_low_gain: f32,
    pub eq_mid_gain: f32,
    pub eq_high_gain: f32,
    pub eq_air_band: bool,

    // Rack Unit 3: Optical Compressor
    pub comp_power: bool,
    pub comp_thresh_db: f32,
    pub comp_ratio_idx: usize,
    pub comp_makeup_db: f32,

    // Rack Unit 4: Stereo Imager
    pub stereo_power: bool,
    pub stereo_width_pct: f32,
    pub bass_mono_hz: f32,

    // Rack Unit 5: Master Limiter
    pub limiter_power: bool,
    pub ceiling_db: f32,
    pub oversampling_4x: bool,
    pub soft_clip: bool,

    // Calibration
    pub dither_mode_idx: usize,
}

impl Default for AdvanceStudioState {
    fn default() -> Self {
        let mut matrix_routes = [[false; 6]; 6];
        for i in 0..6 {
            matrix_routes[i][i] = true;
        }

        Self {
            graph_mode: AdvanceGraphMode::CumulativeCurve,
            rack_tab: AdvanceRackTab::RackUnits,
            selected_rack_unit: 0,
            freeze_graph: false,
            show_grid_db: true,
            bypass_all_fx: false,
            ab_state: false,
            matrix_routes,

            tube_power: true,
            tube_drive_db: 4.5,
            tube_warmth: 65.0,
            tube_triode_mode: true,

            eq_power: true,
            eq_low_gain: 2.0,
            eq_mid_gain: -1.0,
            eq_high_gain: 3.5,
            eq_air_band: true,

            comp_power: true,
            comp_thresh_db: -14.0,
            comp_ratio_idx: 1, // 4:1
            comp_makeup_db: 2.5,

            stereo_power: true,
            stereo_width_pct: 125.0,
            bass_mono_hz: 120.0,

            limiter_power: true,
            ceiling_db: -0.1,
            oversampling_4x: true,
            soft_clip: true,

            dither_mode_idx: 0,
        }
    }
}

impl LunulaApp {
    /// Renders the Advance Studio GUI (High-resolution Dual Graph Analyzer + Pro Hardware Rack).
    /// Pure GUI Mockup / Prototype mode for experimentation.
    pub fn render_advance_studio(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 1. Top Status & Signal Flow Strip
            self.render_advance_status_strip(ui);

            ui.add_space(8.0);

            // 2. High-Resolution Graph Analyzer Section (Height ~230px)
            self.render_advance_graph_section(ui);

            ui.add_space(8.0);

            // 3. Lower Pro Rack & Matrix Section (Scrollable or Fitted)
            self.render_advance_rack_section(ui);
        });
    }

    /// Status and Signal Flow summary banner
    fn render_advance_status_strip(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(Color32::from_rgb(15, 23, 42))
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(30, 41, 59)))
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left: Live Engine Indicator
                    let is_active = self.is_playing;
                    let dot_color = if is_active {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(148, 163, 184)
                    };
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.5, dot_color);
                    if is_active {
                        ui.painter().circle_stroke(dot_rect.center(), 6.5, Stroke::new(1.0, Color32::from_rgba_premultiplied(34, 197, 94, 90)));
                    }

                    ui.label(
                        egui::RichText::new("ADVANCE DSP CONSOLE")
                            .strong()
                            .size(11.5)
                            .color(Color32::WHITE),
                    );

                    ui.separator();

                    // Signal Flow breadcrumbs
                    let flow_text = "Source (96k/24b) ➔ Tube Stage ➔ Parametric EQ ➔ Dynamics ➔ Spatializer ➔ DAC Out";
                    ui.label(
                        egui::RichText::new(flow_text)
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );

                    // Right-aligned controls
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Prototype Tag
                        ui.label(
                            egui::RichText::new("🔬 GUI MOCKUP / PREVIEW")
                                .size(10.0)
                                .strong()
                                .color(Color32::from_rgb(245, 158, 11)),
                        );

                        ui.separator();

                        // Freeze button
                        let freeze_btn = if self.advance_studio_state.freeze_graph {
                            egui::RichText::new("❄️ Unfreeze").size(11.0).color(Color32::from_rgb(56, 189, 248))
                        } else {
                            egui::RichText::new("❄️ Freeze").size(11.0).color(Color32::from_rgb(148, 163, 184))
                        };
                        if ui.button(freeze_btn).on_hover_text("Bekukan grafik analisis real-time").clicked() {
                            self.advance_studio_state.freeze_graph = !self.advance_studio_state.freeze_graph;
                        }

                        // Bypass All
                        let bypass_btn = if self.advance_studio_state.bypass_all_fx {
                            egui::RichText::new("🚫 Bypass ACTIVE").size(11.0).strong().color(Color32::from_rgb(239, 68, 68))
                        } else {
                            egui::RichText::new("⚡ FX Engaged").size(11.0).strong().color(Color32::from_rgb(34, 197, 94))
                        };
                        if ui.button(bypass_btn).on_hover_text("Bypass seluruh modul efek sekaligus").clicked() {
                            self.advance_studio_state.bypass_all_fx = !self.advance_studio_state.bypass_all_fx;
                        }

                        // A/B Compare Toggle
                        let ab_text = if self.advance_studio_state.ab_state {
                            egui::RichText::new("B: WET").size(11.0).strong().color(Color32::from_rgb(168, 85, 247))
                        } else {
                            egui::RichText::new("A: DRY").size(11.0).strong().color(Color32::from_rgb(59, 130, 246))
                        };
                        if ui.button(ab_text).on_hover_text("Bandingkan instan antara suara asli (A) vs hasil olahan efek (B)").clicked() {
                            self.advance_studio_state.ab_state = !self.advance_studio_state.ab_state;
                        }
                        ui.label(egui::RichText::new("Compare:").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                    });
                });
            });
    }

    /// Section A: High-Resolution Dual Graph Analyzer
    fn render_advance_graph_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(Color32::from_rgb(11, 15, 25))
            .corner_radius(10.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(30, 41, 59)))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Header: Graph Mode Selector Pills
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("AUDIO ANALYZER")
                                .strong()
                                .size(11.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );

                        ui.add_space(8.0);

                        let current_mode = self.advance_studio_state.graph_mode;
                        let modes = [
                            (AdvanceGraphMode::CumulativeCurve, "📈 Cumulative Curve (Net EQ)"),
                            (AdvanceGraphMode::DualSpectrumRta, "📊 Pre vs Post Spectrum"),
                            (AdvanceGraphMode::StereoVectorscope, "🌊 Stereo Vectorscope"),
                            (AdvanceGraphMode::MasterMetering, "🎚️ Master Metering Bridge"),
                        ];

                        for (mode, label) in modes {
                            let is_sel = current_mode == mode;
                            let text = if is_sel {
                                egui::RichText::new(label).strong().size(11.0).color(Color32::WHITE)
                            } else {
                                egui::RichText::new(label).size(11.0).color(Color32::from_rgb(148, 163, 184))
                            };

                            let bg_color = if is_sel {
                                Color32::from_rgb(79, 70, 229)
                            } else {
                                Color32::from_rgb(15, 23, 42)
                            };

                            let btn = egui::Button::new(text)
                                .fill(bg_color)
                                .corner_radius(5.0)
                                .stroke(Stroke::new(0.8, if is_sel { Color32::from_rgb(129, 140, 248) } else { Color32::from_rgb(51, 65, 85) }));

                            if ui.add(btn).clicked() {
                                self.advance_studio_state.graph_mode = mode;
                            }
                        }

                        // Right details
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("20 Hz — 20 kHz • Precision Logarithmic Grid")
                                    .size(10.0)
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                        });
                    });

                    ui.add_space(6.0);

                    // Canvas rendering
                    let canvas_height = 200.0;
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), canvas_height), egui::Sense::hover());
                    let painter = ui.painter_at(rect);

                    // Grid backdrop
                    painter.rect_filled(rect, 6.0, Color32::from_rgb(6, 9, 18));
                    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Inside);

                    match self.advance_studio_state.graph_mode {
                        AdvanceGraphMode::CumulativeCurve => {
                            self.draw_cumulative_curve_graph(&painter, rect, resp.hover_pos());
                        }
                        AdvanceGraphMode::DualSpectrumRta => {
                            self.draw_dual_spectrum_rta(&painter, rect);
                        }
                        AdvanceGraphMode::StereoVectorscope => {
                            self.draw_stereo_vectorscope(&painter, rect);
                        }
                        AdvanceGraphMode::MasterMetering => {
                            self.draw_master_metering_bridge(&painter, rect);
                        }
                    }
                });
            });
    }

    /// Renders the Net Cumulative Transfer Curve Graph
    fn draw_cumulative_curve_graph(&self, painter: &egui::Painter, rect: Rect, mouse_pos: Option<Pos2>) {
        let st = &self.advance_studio_state;
        let left = rect.min.x + 40.0;
        let right = rect.max.x - 20.0;
        let top = rect.min.y + 16.0;
        let bottom = rect.max.y - 24.0;
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
                Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 50))
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

        // Compute cumulative response points
        let steps = 140;
        let mut pts = Vec::with_capacity(steps);
        for s in 0..steps {
            let t = s as f32 / (steps - 1) as f32;
            let f = (f_min + t * (f_max - f_min)).exp();
            let x = left + t * width;

            // Theoretical response: Low shelf + mid bell + high shelf + tube warmth + limiter ceiling
            let mut db_resp = 0.0f32;
            if !st.bypass_all_fx {
                // Low shelf
                if st.eq_power {
                    let low_t = (1.0 - (f / 250.0).clamp(0.0, 1.0)).powi(2);
                    db_resp += st.eq_low_gain * low_t;

                    // Mid peak around 1kHz
                    let mid_dist = ((f.ln() - 1000.0f32.ln()).abs() / 0.8).clamp(0.0, 3.0);
                    let mid_t = (-mid_dist * mid_dist).exp();
                    db_resp += st.eq_mid_gain * mid_t;

                    // High shelf
                    let high_t = ((f - 3000.0) / 7000.0).clamp(0.0, 1.0);
                    db_resp += st.eq_high_gain * high_t;

                    // Air band boost
                    if st.eq_air_band && f >= 10000.0 {
                        db_resp += 1.8 * ((f - 10000.0) / 10000.0);
                    }
                }

                // Tube warmth (low-mid analog bump)
                if st.tube_power {
                    let tube_bump = (-(f.ln() - 180.0f32.ln()).powi(2) / 1.5).exp() * (st.tube_drive_db * 0.15);
                    db_resp += tube_bump;
                }

                // Compressor gentle attenuation
                if st.comp_power && db_resp > 2.0 {
                    db_resp = 2.0 + (db_resp - 2.0) * 0.5;
                }
            }

            let y = mid_y - (db_resp / 18.0) * (height * 0.45);
            pts.push(Pos2::new(x, y));
        }

        // Shaded area under curve
        let mut fill_pts = Vec::with_capacity(pts.len() + 2);
        fill_pts.push(Pos2::new(left, mid_y));
        fill_pts.extend_from_slice(&pts);
        fill_pts.push(Pos2::new(right, mid_y));
        painter.add(egui::Shape::convex_polygon(fill_pts, Color32::from_rgba_premultiplied(99, 102, 241, 28), Stroke::NONE));

        // Radiant glow curve
        painter.add(egui::Shape::line(pts.clone(), Stroke::new(3.5, Color32::from_rgba_premultiplied(129, 140, 248, 60))));
        painter.add(egui::Shape::line(pts, Stroke::new(2.0, Color32::from_rgb(129, 140, 248))));

        // Interactive hover crosshair
        if let Some(m) = mouse_pos {
            if m.x >= left && m.x <= right && m.y >= top && m.y <= bottom {
                let t = (m.x - left) / width;
                let hover_f = (f_min + t * (f_max - f_min)).exp();
                let hover_db = (mid_y - m.y) / (height * 0.45) * 18.0;

                // Crosshair lines
                painter.line_segment([Pos2::new(m.x, top), Pos2::new(m.x, bottom)], Stroke::new(1.0, Color32::from_rgba_premultiplied(245, 158, 11, 140)));
                painter.line_segment([Pos2::new(left, m.y), Pos2::new(right, m.y)], Stroke::new(1.0, Color32::from_rgba_premultiplied(245, 158, 11, 140)));

                // Value readout badge
                let readout = format!("{:.0} Hz  •  {:+0.1} dB", hover_f, hover_db);
                let badge_pos = Pos2::new(m.x.clamp(left + 60.0, right - 60.0), top + 14.0);
                painter.rect_filled(Rect::from_center_size(badge_pos, Vec2::new(115.0, 20.0)), 4.0, Color32::from_rgb(15, 23, 42));
                painter.rect_stroke(Rect::from_center_size(badge_pos, Vec2::new(115.0, 20.0)), 4.0, Stroke::new(1.0, Color32::from_rgb(245, 158, 11)), egui::StrokeKind::Inside);
                painter.text(badge_pos, egui::Align2::CENTER_CENTER, readout, egui::FontId::monospace(10.0), Color32::WHITE);
            }
        }
    }

    /// Renders Dual Spectrum RTA (Pre vs Post DSP)
    fn draw_dual_spectrum_rta(&self, painter: &egui::Painter, rect: Rect) {
        let left = rect.min.x + 35.0;
        let right = rect.max.x - 20.0;
        let top = rect.min.y + 16.0;
        let bottom = rect.max.y - 24.0;
        let width = right - left;
        let height = bottom - top;

        let bar_count = 48;
        let bar_w = (width / bar_count as f32) - 1.5;

        // Draw Pre-DSP bars in cool slate-cyan
        for b in 0..bar_count {
            let bx = left + b as f32 * (bar_w + 1.5);
            let raw_val = self.spectrum_bars[b];
            let active_val = if self.is_playing {
                raw_val.max(((self.visualizer_anim_phase * 5.0 + b as f32 * 0.5).sin() * 0.3 + 0.3).abs())
            } else {
                0.08
            };

            let h_pre = active_val * height * 0.85;
            let bar_rect = Rect::from_min_size(Pos2::new(bx, bottom - h_pre), Vec2::new(bar_w, h_pre));
            painter.rect_filled(bar_rect, 1.5, Color32::from_rgba_premultiplied(59, 130, 246, 70));

            // Post-DSP curve overlay (higher gain at highs & lows due to EQ)
            let post_factor = if b < 12 { 1.25 } else if b > 36 { 1.35 } else { 0.95 };
            let h_post = (h_pre * post_factor).min(height);
            let post_top = Pos2::new(bx + bar_w * 0.5, bottom - h_post);

            painter.circle_filled(post_top, 2.5, Color32::from_rgb(168, 85, 247));
        }

        // Legend
        let leg_pos = Pos2::new(right - 180.0, top + 10.0);
        painter.rect_filled(Rect::from_min_size(leg_pos, Vec2::new(170.0, 22.0)), 4.0, Color32::from_rgba_premultiplied(15, 23, 42, 220));
        painter.rect_filled(Rect::from_min_size(leg_pos + Vec2::new(10.0, 6.0), Vec2::new(10.0, 10.0)), 2.0, Color32::from_rgb(59, 130, 246));
        painter.text(leg_pos + Vec2::new(26.0, 11.0), egui::Align2::LEFT_CENTER, "Pre-DSP", egui::FontId::proportional(10.0), Color32::WHITE);

        painter.rect_filled(Rect::from_min_size(leg_pos + Vec2::new(95.0, 6.0), Vec2::new(10.0, 10.0)), 2.0, Color32::from_rgb(168, 85, 247));
        painter.text(leg_pos + Vec2::new(111.0, 11.0), egui::Align2::LEFT_CENTER, "Post-DSP", egui::FontId::proportional(10.0), Color32::WHITE);
    }

    /// Renders Stereo Vectorscope & Goniometer (Lissajous)
    fn draw_stereo_vectorscope(&self, painter: &egui::Painter, rect: Rect) {
        let center = rect.center();
        let radius = (rect.height() * 0.4).min(80.0);

        // Circular polar grid
        painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_rgb(51, 65, 85)));
        painter.circle_stroke(center, radius * 0.5, Stroke::new(0.8, Color32::from_rgba_premultiplied(51, 65, 85, 120)));
        painter.line_segment([Pos2::new(center.x - radius, center.y), Pos2::new(center.x + radius, center.y)], Stroke::new(0.8, Color32::from_rgb(51, 65, 85)));
        painter.line_segment([Pos2::new(center.x, center.y - radius), Pos2::new(center.x, center.y + radius)], Stroke::new(0.8, Color32::from_rgb(51, 65, 85)));

        // Diagonal M/S axes (L and R axes rotated 45 deg)
        let diag_offset = radius * 0.7071;
        painter.line_segment([Pos2::new(center.x - diag_offset, center.y - diag_offset), Pos2::new(center.x + diag_offset, center.y + diag_offset)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 60)));
        painter.line_segment([Pos2::new(center.x - diag_offset, center.y + diag_offset), Pos2::new(center.x + diag_offset, center.y - diag_offset)], Stroke::new(0.8, Color32::from_rgba_premultiplied(148, 163, 184, 60)));

        painter.text(Pos2::new(center.x - diag_offset - 8.0, center.y - diag_offset), egui::Align2::RIGHT_CENTER, "+L", egui::FontId::monospace(9.0), Color32::from_rgb(148, 163, 184));
        painter.text(Pos2::new(center.x + diag_offset + 8.0, center.y - diag_offset), egui::Align2::LEFT_CENTER, "+R", egui::FontId::monospace(9.0), Color32::from_rgb(148, 163, 184));

        // Animated Lissajous audio scatter points
        let count = 48;
        let phase = self.visualizer_anim_phase * 6.0;
        for i in 0..count {
            let t = i as f32 / count as f32;
            let a = t * std::f32::consts::TAU + phase;
            let amp = if self.is_playing {
                ((a * 3.0).sin() * 0.4 + 0.6) * radius * 0.75
            } else {
                radius * 0.25
            };
            let px = center.x + (a).cos() * amp * 0.7;
            let py = center.y + (a * 1.5 + phase * 0.5).sin() * amp;
            painter.circle_filled(Pos2::new(px, py), 1.8, Color32::from_rgba_premultiplied(56, 189, 248, 160));
        }

        // Phase correlation meter bar on the right
        let bar_left = rect.max.x - 140.0;
        let bar_y = center.y;
        painter.text(Pos2::new(bar_left, bar_y - 20.0), egui::Align2::LEFT_CENTER, "Phase Correlation", egui::FontId::proportional(10.5), Color32::WHITE);
        painter.rect_filled(Rect::from_min_size(Pos2::new(bar_left, bar_y - 6.0), Vec2::new(120.0, 12.0)), 3.0, Color32::from_rgb(30, 41, 59));
        // Center zero line
        painter.line_segment([Pos2::new(bar_left + 60.0, bar_y - 8.0), Pos2::new(bar_left + 60.0, bar_y + 8.0)], Stroke::new(1.0, Color32::WHITE));
        // Indicator (+0.85 in-phase)
        let indicator_x = bar_left + 60.0 + 45.0;
        painter.circle_filled(Pos2::new(indicator_x, bar_y), 5.0, Color32::from_rgb(34, 197, 94));
        painter.text(Pos2::new(bar_left + 60.0, bar_y + 16.0), egui::Align2::CENTER_CENTER, "+0.85 (In-Phase)", egui::FontId::monospace(9.5), Color32::from_rgb(34, 197, 94));
    }

    /// Renders Master Metering Bridge (EBU R128 LUFS & True-Peak)
    fn draw_master_metering_bridge(&self, painter: &egui::Painter, rect: Rect) {
        let left = rect.min.x + 30.0;
        let top = rect.min.y + 20.0;

        // 3 Column Metering: [Integrated LUFS] [Short-Term LUFS] [True Peak L/R]
        let col_w = (rect.width() - 80.0) / 3.0;

        // Col 1: Integrated Loudness
        let c1_rect = Rect::from_min_size(Pos2::new(left, top), Vec2::new(col_w, rect.height() - 40.0));
        painter.rect_filled(c1_rect, 6.0, Color32::from_rgb(15, 23, 42));
        painter.text(c1_rect.center_top() + Vec2::new(0.0, 14.0), egui::Align2::CENTER_CENTER, "INTEGRATED LUFS", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));
        painter.text(c1_rect.center() + Vec2::new(0.0, 5.0), egui::Align2::CENTER_CENTER, "-14.2 LUFS", egui::FontId::monospace(18.0), Color32::from_rgb(34, 197, 94));
        painter.text(c1_rect.center_bottom() - Vec2::new(0.0, 14.0), egui::Align2::CENTER_CENTER, "Target: -14.0 LUFS (EBU R128)", egui::FontId::proportional(9.5), Color32::from_rgb(100, 116, 139));

        // Col 2: Dynamic Range & Momentary
        let c2_rect = Rect::from_min_size(Pos2::new(left + col_w + 10.0, top), Vec2::new(col_w, rect.height() - 40.0));
        painter.rect_filled(c2_rect, 6.0, Color32::from_rgb(15, 23, 42));
        painter.text(c2_rect.center_top() + Vec2::new(0.0, 14.0), egui::Align2::CENTER_CENTER, "DYNAMIC RANGE (DR)", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));
        painter.text(c2_rect.center() + Vec2::new(0.0, 5.0), egui::Align2::CENTER_CENTER, "DR 14", egui::FontId::monospace(18.0), Color32::from_rgb(56, 189, 248));
        painter.text(c2_rect.center_bottom() - Vec2::new(0.0, 14.0), egui::Align2::CENTER_CENTER, "Audiophile High Dynamic Range", egui::FontId::proportional(9.5), Color32::from_rgb(100, 116, 139));

        // Col 3: True Peak L/R Ladder
        let c3_rect = Rect::from_min_size(Pos2::new(left + (col_w + 10.0) * 2.0, top), Vec2::new(col_w, rect.height() - 40.0));
        painter.rect_filled(c3_rect, 6.0, Color32::from_rgb(15, 23, 42));
        painter.text(c3_rect.center_top() + Vec2::new(0.0, 14.0), egui::Align2::CENTER_CENTER, "TRUE PEAK LEVEL", egui::FontId::proportional(10.5), Color32::from_rgb(148, 163, 184));
        painter.text(c3_rect.center() + Vec2::new(0.0, 5.0), egui::Align2::CENTER_CENTER, "-0.2 dBFS", egui::FontId::monospace(18.0), Color32::from_rgb(245, 158, 11));
        painter.text(c3_rect.center_bottom() - Vec2::new(0.0, 14.0), egui::Align2::CENTER_CENTER, "Inter-Sample Peak Protected", egui::FontId::proportional(9.5), Color32::from_rgb(34, 197, 94));
    }

    /// Section B: Pro Rackmount Units & Matrix Strip
    fn render_advance_rack_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(Color32::from_rgb(15, 23, 42))
            .corner_radius(10.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(30, 41, 59)))
            .inner_margin(egui::Margin::symmetric(14, 12))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Rack Tabs
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("PRO HARDWARE RACK")
                                .strong()
                                .size(11.5)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );

                        ui.add_space(10.0);

                        let cur_tab = self.advance_studio_state.rack_tab;
                        let tabs = [
                            (AdvanceRackTab::RackUnits, "🎛️ Channel Strips (5 Units)"),
                            (AdvanceRackTab::SignalMatrix, "🔀 6x6 Audio Matrix Crossbar"),
                            (AdvanceRackTab::MasterSection, "⚙️ Clock Calibration & Dither"),
                        ];

                        for (tab, label) in tabs {
                            let is_sel = cur_tab == tab;
                            let text = if is_sel {
                                egui::RichText::new(label).strong().size(11.0).color(Color32::WHITE)
                            } else {
                                egui::RichText::new(label).size(11.0).color(Color32::from_rgb(148, 163, 184))
                            };
                            let bg = if is_sel { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(30, 41, 59) };

                            if ui.add(egui::Button::new(text).fill(bg).corner_radius(4.0)).clicked() {
                                self.advance_studio_state.rack_tab = tab;
                            }
                        }
                    });

                    ui.separator();
                    ui.add_space(6.0);

                    // Render selected tab
                    match self.advance_studio_state.rack_tab {
                        AdvanceRackTab::RackUnits => self.render_rack_units(ui),
                        AdvanceRackTab::SignalMatrix => self.render_signal_matrix(ui),
                        AdvanceRackTab::MasterSection => self.render_master_calibration(ui),
                    }
                });
            });
    }

    /// Tab 1: Render 5 Hardware Rack Units
    fn render_rack_units(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Unit 1: Tube Preamp
                    self.render_unit_tube_preamp(ui);
                    ui.add_space(8.0);

                    // Unit 2: Dynamic EQ
                    self.render_unit_dynamic_eq(ui);
                    ui.add_space(8.0);

                    // Unit 3: Optical Compressor
                    self.render_unit_optical_comp(ui);
                    ui.add_space(8.0);

                    // Unit 4: Stereo Spatializer
                    self.render_unit_stereo_spatializer(ui);
                    ui.add_space(8.0);

                    // Unit 5: Master Limiter
                    self.render_unit_master_limiter(ui);
                });
            });
    }

    fn render_unit_tube_preamp(&mut self, ui: &mut egui::Ui) {
        let st = &mut self.advance_studio_state;
        egui::Frame::NONE
            .fill(Color32::from_rgb(24, 30, 44))
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(180.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut st.tube_power, "");
                        ui.label(egui::RichText::new("01 • TUBE PREAMP").strong().size(11.0).color(Color32::from_rgb(245, 158, 11)));
                    });
                    ui.separator();

                    ui.label(egui::RichText::new(format!("Drive: {:+0.1} dB", st.tube_drive_db)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.tube_drive_db, 0.0..=24.0).show_value(false));

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("Warmth: {:.0}%", st.tube_warmth)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.tube_warmth, 0.0..=100.0).show_value(false));

                    ui.add_space(6.0);
                    let mode_str = if st.tube_triode_mode { "Triode (Smooth)" } else { "Pentode (Punchy)" };
                    if ui.button(egui::RichText::new(mode_str).size(10.0)).clicked() {
                        st.tube_triode_mode = !st.tube_triode_mode;
                    }
                });
            });
    }

    fn render_unit_dynamic_eq(&mut self, ui: &mut egui::Ui) {
        let st = &mut self.advance_studio_state;
        egui::Frame::NONE
            .fill(Color32::from_rgb(24, 30, 44))
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(180.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut st.eq_power, "");
                        ui.label(egui::RichText::new("02 • DYNAMIC EQ").strong().size(11.0).color(Color32::from_rgb(56, 189, 248)));
                    });
                    ui.separator();

                    ui.label(egui::RichText::new(format!("Low: {:+0.1} dB", st.eq_low_gain)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.eq_low_gain, -12.0..=12.0).show_value(false));

                    ui.label(egui::RichText::new(format!("Mid: {:+0.1} dB", st.eq_mid_gain)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.eq_mid_gain, -12.0..=12.0).show_value(false));

                    ui.label(egui::RichText::new(format!("High: {:+0.1} dB", st.eq_high_gain)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.eq_high_gain, -12.0..=12.0).show_value(false));

                    ui.checkbox(&mut st.eq_air_band, egui::RichText::new("Air Band (16k)").size(10.0));
                });
            });
    }

    fn render_unit_optical_comp(&mut self, ui: &mut egui::Ui) {
        let st = &mut self.advance_studio_state;
        egui::Frame::NONE
            .fill(Color32::from_rgb(24, 30, 44))
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(180.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut st.comp_power, "");
                        ui.label(egui::RichText::new("03 • OPTICAL COMP").strong().size(11.0).color(Color32::from_rgb(234, 179, 8)));
                    });
                    ui.separator();

                    ui.label(egui::RichText::new(format!("Threshold: {:+0.1} dB", st.comp_thresh_db)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.comp_thresh_db, -40.0..=0.0).show_value(false));

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("Makeup: {:+0.1} dB", st.comp_makeup_db)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.comp_makeup_db, 0.0..=12.0).show_value(false));

                    ui.add_space(6.0);
                    let ratios = ["2:1", "4:1", "8:1", "Limit"];
                    ui.horizontal(|ui| {
                        for (idx, r) in ratios.iter().enumerate() {
                            let is_sel = st.comp_ratio_idx == idx;
                            if ui.selectable_label(is_sel, egui::RichText::new(*r).size(9.5)).clicked() {
                                st.comp_ratio_idx = idx;
                            }
                        }
                    });
                });
            });
    }

    fn render_unit_stereo_spatializer(&mut self, ui: &mut egui::Ui) {
        let st = &mut self.advance_studio_state;
        egui::Frame::NONE
            .fill(Color32::from_rgb(24, 30, 44))
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(180.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut st.stereo_power, "");
                        ui.label(egui::RichText::new("04 • SPATIALIZER").strong().size(11.0).color(Color32::from_rgb(168, 85, 247)));
                    });
                    ui.separator();

                    ui.label(egui::RichText::new(format!("Width: {:.0}%", st.stereo_width_pct)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.stereo_width_pct, 0.0..=200.0).show_value(false));

                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("Bass Mono: {:.0} Hz", st.bass_mono_hz)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.bass_mono_hz, 60.0..=250.0).show_value(false));

                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Center Sub-Bass • Wide Air").size(9.5).color(Color32::from_rgb(148, 163, 184)));
                });
            });
    }

    fn render_unit_master_limiter(&mut self, ui: &mut egui::Ui) {
        let st = &mut self.advance_studio_state;
        egui::Frame::NONE
            .fill(Color32::from_rgb(24, 30, 44))
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(180.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut st.limiter_power, "");
                        ui.label(egui::RichText::new("05 • MASTER LIMITER").strong().size(11.0).color(Color32::from_rgb(239, 68, 68)));
                    });
                    ui.separator();

                    ui.label(egui::RichText::new(format!("Ceiling: {:+0.2} dBFS", st.ceiling_db)).size(10.5).color(Color32::WHITE));
                    ui.add(egui::Slider::new(&mut st.ceiling_db, -3.0..=0.0).show_value(false));

                    ui.add_space(6.0);
                    ui.checkbox(&mut st.oversampling_4x, egui::RichText::new("4x Oversampling").size(10.0));
                    ui.checkbox(&mut st.soft_clip, egui::RichText::new("Analog Soft Knee").size(10.0));
                });
            });
    }

    /// Tab 2: Render 6x6 Pro Audio Matrix Crossbar
    fn render_signal_matrix(&mut self, ui: &mut egui::Ui) {
        let st = &mut self.advance_studio_state;
        let labels = ["Source", "Preamp", "Dyn EQ", "Comp", "Spatial", "DAC Out"];

        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Klik titik perpotongan (crosspoint) untuk menghubungkan bus routing audio:").size(11.0).color(Color32::from_rgb(148, 163, 184)));
            ui.add_space(8.0);

            // Matrix Table
            egui::Grid::new("advance_studio_matrix_grid")
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("");
                    for col in &labels {
                        ui.label(egui::RichText::new(*col).strong().size(10.5).color(Color32::WHITE));
                    }
                    ui.end_row();

                    for row in 0..6 {
                        ui.label(egui::RichText::new(labels[row]).strong().size(10.5).color(Color32::WHITE));
                        for col in 0..6 {
                            let is_connected = st.matrix_routes[row][col];
                            let dot_color = if is_connected {
                                Color32::from_rgb(34, 197, 94)
                            } else {
                                Color32::from_rgb(51, 65, 85)
                            };

                            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
                            ui.painter().circle_filled(rect.center(), if is_connected { 6.0 } else { 3.5 }, dot_color);
                            if is_connected {
                                ui.painter().circle_stroke(rect.center(), 8.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(34, 197, 94, 120)));
                            }

                            if resp.clicked() {
                                st.matrix_routes[row][col] = !is_connected;
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    }

    /// Tab 3: Render Master Calibration & Dither
    fn render_master_calibration(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Box 1: Dither Settings
            egui::Frame::NONE
                .fill(Color32::from_rgb(24, 30, 44))
                .corner_radius(8.0)
                .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
                .inner_margin(egui::Margin::symmetric(14, 12))
                .show(ui, |ui| {
                    ui.set_width(260.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("NOISE SHAPING & DITHER").strong().size(11.5).color(Color32::WHITE));
                        ui.separator();
                        let dither_options = [
                            "TPDF Triangular (Standard Audiophile)",
                            "High-Pass Psychoacoustic Shaped",
                            "Truncation / None (Bit-Exact Passthrough)",
                        ];
                        for (idx, opt) in dither_options.iter().enumerate() {
                            let is_sel = self.advance_studio_state.dither_mode_idx == idx;
                            if ui.selectable_label(is_sel, *opt).clicked() {
                                self.advance_studio_state.dither_mode_idx = idx;
                            }
                        }
                    });
                });

            ui.add_space(12.0);

            // Box 2: Clock & Engine Specs
            egui::Frame::NONE
                .fill(Color32::from_rgb(24, 30, 44))
                .corner_radius(8.0)
                .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
                .inner_margin(egui::Margin::symmetric(14, 12))
                .show(ui, |ui| {
                    ui.set_width(280.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("INTERNAL ENGINE SPECS").strong().size(11.5).color(Color32::WHITE));
                        ui.separator();
                        ui.label(egui::RichText::new("• Internal Processing: 32-bit Floating Point").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                        ui.label(egui::RichText::new("• Latency: 0.67 ms (64 samples buffer)").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                        ui.label(egui::RichText::new("• Dynamic Range Floor: -144 dBFS").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                        ui.label(egui::RichText::new("• Phase Deviation: 0.00° (Linear Phase Sync)").size(10.5).color(Color32::from_rgb(34, 197, 94)));
                    });
                });
        });
    }
}
