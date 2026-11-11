use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::app::LunulaApp;
use crate::models::{ChainMode, DitherMode, FilterSlope, FilterType, NodeType, ParametricBand, ParametricEqPreset, PreampMode, SignalNodeStatus};
use crate::ui::components::render_card_frame;

// Precise f64 Biquad Filter Magnitude Response Calculation (Robert Bristow-Johnson Audio EQ Cookbook)
pub fn compute_biquad_response_db(band: &ParametricBand, freq: f64, sample_rate: f64) -> f64 {
    if !band.enabled || sample_rate <= 0.0 || freq <= 0.0 {
        return 0.0;
    }

    let nyquist = sample_rate * 0.499;
    let f0 = (band.freq as f64).clamp(10.0, nyquist);
    let q = (band.q as f64).clamp(0.1, 10.0);
    let gain_db = (band.gain_db as f64).clamp(-60.0, 30.0);

    let w0 = 2.0 * std::f64::consts::PI * (f0 / sample_rate);
    let cos_w0 = w0.cos();
    let sin_w0 = w0.sin();
    let alpha = sin_w0 / (2.0 * q);

    let (b0, b1, b2, a0, a1, a2) = match band.filter_type {
        FilterType::Peaking => {
            if gain_db.abs() < 0.01 {
                return 0.0;
            }
            let a = 10.0_f64.powf(gain_db / 40.0);
            (
                1.0 + alpha * a,
                -2.0 * cos_w0,
                1.0 - alpha * a,
                1.0 + alpha / a,
                -2.0 * cos_w0,
                1.0 - alpha / a,
            )
        }
        FilterType::LowShelf => {
            if gain_db.abs() < 0.01 {
                return 0.0;
            }
            let a = 10.0_f64.powf(gain_db / 40.0);
            let beta = 2.0 * a.sqrt() * alpha;
            (
                a * ((a + 1.0) - (a - 1.0) * cos_w0 + beta),
                2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0),
                a * ((a + 1.0) - (a - 1.0) * cos_w0 - beta),
                (a + 1.0) + (a - 1.0) * cos_w0 + beta,
                -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0),
                (a + 1.0) + (a - 1.0) * cos_w0 - beta,
            )
        }
        FilterType::HighShelf => {
            if gain_db.abs() < 0.01 {
                return 0.0;
            }
            let a = 10.0_f64.powf(gain_db / 40.0);
            let beta = 2.0 * a.sqrt() * alpha;
            (
                a * ((a + 1.0) + (a - 1.0) * cos_w0 + beta),
                -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0),
                a * ((a + 1.0) + (a - 1.0) * cos_w0 - beta),
                (a + 1.0) - (a - 1.0) * cos_w0 + beta,
                2.0 * ((a - 1.0) - (a + 1.0) * cos_w0),
                (a + 1.0) - (a - 1.0) * cos_w0 - beta,
            )
        }
        FilterType::HighPass => (
            (1.0 + cos_w0) / 2.0,
            -(1.0 + cos_w0),
            (1.0 + cos_w0) / 2.0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        ),
        FilterType::LowPass => (
            (1.0 - cos_w0) / 2.0,
            1.0 - cos_w0,
            (1.0 - cos_w0) / 2.0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        ),
        FilterType::Notch => (
            1.0,
            -2.0 * cos_w0,
            1.0,
            1.0 + alpha,
            -2.0 * cos_w0,
            1.0 - alpha,
        ),
    };

    let b0_norm = b0 / a0;
    let b1_norm = b1 / a0;
    let b2_norm = b2 / a0;
    let a1_norm = a1 / a0;
    let a2_norm = a2 / a0;

    let w = 2.0 * std::f64::consts::PI * (freq / sample_rate);
    let cos1 = w.cos();
    let sin1 = w.sin();
    let cos2 = (2.0 * w).cos();
    let sin2 = (2.0 * w).sin();

    let num_re = b0_norm + b1_norm * cos1 + b2_norm * cos2;
    let num_im = -(b1_norm * sin1 + b2_norm * sin2);
    let den_re = 1.0 + a1_norm * cos1 + a2_norm * cos2;
    let den_im = -(a1_norm * sin1 + a2_norm * sin2);

    let mag_sq = (num_re * num_re + num_im * num_im) / (den_re * den_re + den_im * den_im).max(1e-12);
    let single_biquad_db = 10.0 * mag_sq.max(1e-12).log10();

    // Multiply response in dB by slope order factor for HPF / LPF (12/24/48 dB/oct)
    let slope_factor = match band.filter_type {
        FilterType::HighPass | FilterType::LowPass => band.slope.biquad_count() as f64,
        _ => 1.0,
    };

    (single_biquad_db * slope_factor).clamp(-96.0, 36.0)
}

pub fn compute_total_curve_db(
    bands: &[ParametricBand],
    freq: f64,
    sample_rate: f64,
    preamp_db: f64,
    is_bypassed: bool,
) -> f64 {
    if is_bypassed {
        return 0.0;
    }
    let mut total_db = preamp_db;
    for band in bands {
        if band.enabled {
            total_db += compute_biquad_response_db(band, freq, sample_rate);
        }
    }
    total_db.clamp(-96.0, 36.0)
}

// Evaluates the combined response curve across the spectrum to find true composite peak boost
pub fn calculate_combined_peak_boost(bands: &[ParametricBand], sample_rate: f64) -> f32 {
    let mut max_boost = 0.0_f64;
    // 96 evaluation points across 20 Hz to 20 kHz
    for i in 0..=96 {
        let t = i as f64 / 96.0;
        let freq = 20.0 * (1000.0_f64).powf(t);
        let mut total_db = 0.0;
        for band in bands {
            if band.enabled {
                total_db += compute_biquad_response_db(band, freq, sample_rate);
            }
        }
        if total_db > max_boost {
            max_boost = total_db;
        }
    }
    max_boost as f32
}

// Honest string formatting helpers eliminating negative zero "-0.0 dB"
pub fn format_db_clean(val: f32) -> String {
    if val.abs() < 0.005 {
        "0.0 dB".to_string()
    } else {
        format!("{:+0.1} dB", val)
    }
}

pub fn format_margin_clean(val: f32) -> String {
    if val.abs() < 0.005 {
        "0.0 dB Margin".to_string()
    } else {
        format!("{:0.1} dB Margin", val.max(0.0))
    }
}

pub fn format_freq(f: f32) -> String {
    if f >= 1000.0 {
        let khz = f / 1000.0;
        if (khz.round() - khz).abs() < 0.02 {
            format!("{:.0} kHz", khz)
        } else {
            format!("{:.1} kHz", khz)
        }
    } else {
        format!("{:.0} Hz", f)
    }
}

// Canvas coordinate mappings with generous inset padding preventing label clipping
pub fn freq_to_x(freq: f32, rect: Rect) -> f32 {
    let min_f = 20.0_f32;
    let max_f = 20000.0_f32;
    let inset = 24.0;
    let inner_w = (rect.width() - inset * 2.0).max(10.0);
    let f = freq.clamp(min_f, max_f);
    let t = (f / min_f).log10() / (max_f / min_f).log10();
    rect.min.x + inset + t * inner_w
}

pub fn x_to_freq(x: f32, rect: Rect) -> f32 {
    let min_f = 20.0_f32;
    let max_f = 20000.0_f32;
    let inset = 24.0;
    let inner_w = (rect.width() - inset * 2.0).max(10.0);
    let t = ((x - (rect.min.x + inset)) / inner_w).clamp(0.0, 1.0);
    min_f * (max_f / min_f).powf(t)
}

pub fn gain_to_y(gain_db: f32, rect: Rect) -> f32 {
    let center_y = rect.center().y;
    let half_h = rect.height() * 0.42;
    center_y - (gain_db / 18.0).clamp(-1.0, 1.0) * half_h
}

pub fn y_to_gain(y: f32, rect: Rect) -> f32 {
    let center_y = rect.center().y;
    let half_h = rect.height() * 0.42;
    ((center_y - y) / half_h * 18.0).clamp(-18.0, 18.0)
}

fn filter_color(filter_type: FilterType) -> Color32 {
    match filter_type {
        FilterType::Peaking => Color32::from_rgb(45, 212, 191),   // Teal
        FilterType::LowShelf => Color32::from_rgb(251, 146, 60),  // Warm Orange
        FilterType::HighShelf => Color32::from_rgb(56, 189, 248), // Sky Blue
        FilterType::HighPass => Color32::from_rgb(129, 140, 248), // Indigo
        FilterType::LowPass => Color32::from_rgb(244, 114, 182),  // Pink
        FilterType::Notch => Color32::from_rgb(239, 68, 68),      // Crimson Red
    }
}

impl LunulaApp {
    pub fn is_equalizer_in_active_chain(&self) -> bool {
        if self.chain_mode == ChainMode::BitPerfect {
            return false;
        }
        let chain_eval = self.evaluate_signal_chain();
        self.nodes.iter().any(|n| {
            matches!(n.node_type, NodeType::SavedEqPreset(ref name) if name == &self.active_eq.name)
                && chain_eval.node_status(n.id) == SignalNodeStatus::Flowing
        })
    }

    pub fn sync_parametric_to_legacy_eq(&mut self) {
        let legacy_freqs = [31.0, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
        let max_boost = calculate_combined_peak_boost(&self.active_eq.bands, 44100.0);
        let preamp = if self.active_eq.preamp_mode == PreampMode::Auto {
            if max_boost > 0.0 { -max_boost } else { 0.0 }
        } else {
            self.active_eq.manual_preamp_db
        };

        for (i, &f) in legacy_freqs.iter().enumerate() {
            let total_db = compute_total_curve_db(
                &self.active_eq.bands,
                f,
                44100.0,
                preamp as f64,
                self.eq_bypass,
            );
            self.eq_bands[i] = (total_db as f32).clamp(-12.0, 12.0);
        }

        // Only stream live band parameters if this EQ preset is currently connected in the signal chain!
        if self.is_equalizer_in_active_chain() {
            if let Some(ref engine) = self.audio_engine {
                for (i, &gain) in self.eq_bands.iter().enumerate() {
                    engine.set_eq_band(i, gain);
                }
                engine.set_eq_bypass(self.eq_bypass);
            }
        }

        // Always sync the signal chain so that if this EQ preset card is in the chain,
        // it recompiles with the new bands; if not in the chain, no EQ filter is inserted.
        self.sync_signal_chain_to_engine();
    }

    pub fn render_spectrum_visualizer(&mut self, ui: &mut egui::Ui) {
        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("REAL-TIME SPECTRUM ANALYZER").strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("60 FPS • 48-Band High Precision FFT").strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));
                    });
                });

                ui.add_space(6.0);

                let canvas_h = (ui.available_height() - 14.0).clamp(115.0, 300.0);
                let (rect, _response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), canvas_h), egui::Sense::hover());
                let painter = ui.painter_at(rect);

                painter.rect_filled(rect, 8.0, Color32::from_rgb(248, 250, 252));
                painter.rect_stroke(rect, 8.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

                let bars = 48;
                let spacing = 3.5;
                let total_spacing = (bars - 1) as f32 * spacing;
                let bar_width = (rect.width() - 24.0 - total_spacing) / (bars as f32);

                let vol_scale = if self.is_muted {
                    0.0
                } else {
                    self.volume.clamp(0.0, 1.2)
                };

                let max_bar_h = rect.height() - 20.0;

                for i in 0..bars {
                    let t = i as f32 / bars as f32;
                    let fft_energy = self.spectrum_bars[i];

                    let eq_factor = if !self.eq_bypass {
                        let eq_idx = ((t * 9.99).floor() as usize).min(9);
                        let gain_db = self.eq_bands[eq_idx];
                        10.0_f32.powf(gain_db / 28.0)
                    } else {
                        1.0
                    };

                    let has_energy = fft_energy > 0.005;

                    let bar_height = if vol_scale > 0.001 && has_energy {
                        let combined = fft_energy * eq_factor;
                        (combined * (max_bar_h * 0.92)).clamp(3.0, max_bar_h)
                    } else {
                        3.0
                    };

                    let x = rect.min.x + 12.0 + (i as f32) * (bar_width + spacing);
                    let y_bottom = rect.max.y - 10.0;
                    let y_top = y_bottom - bar_height;

                    let bar_rect = Rect::from_min_max(Pos2::new(x, y_top), Pos2::new(x + bar_width, y_bottom));

                    let r = (15.0 + t * 0.0) as u8;
                    let g = (23.0 + t * 140.0) as u8;
                    let b = (42.0 + t * 190.0) as u8;
                    let col = if vol_scale > 0.001 && has_energy {
                        Color32::from_rgb(r, g, b)
                    } else {
                        Color32::from_rgb(226, 232, 240)
                    };

                    painter.rect_filled(bar_rect, 2.5, col);
                    if bar_height > 6.0 && vol_scale > 0.001 && has_energy {
                        painter.circle_filled(Pos2::new(x + bar_width / 2.0, y_top - 2.5), 1.5, Color32::from_rgb(14, 165, 233));
                    }
                }
            });
        });
    }

    // Main Equalizer Tab renders the reusable Parametric EQ editor on active_eq
    pub fn render_equalizer_panel(&mut self, ui: &mut egui::Ui) {
        let mut active_eq = self.active_eq.clone();
        let mut is_dirty = self.eq_is_dirty;

        self.render_parametric_eq_editor(ui, &mut active_eq, &mut is_dirty);

        self.active_eq = active_eq;
        self.eq_is_dirty = is_dirty;
    }

    // Reusable Parametric EQ Editor Widget operating on any EqPreset reference
    pub fn render_parametric_eq_editor(
        &mut self,
        ui: &mut egui::Ui,
        preset: &mut ParametricEqPreset,
        is_dirty: &mut bool,
    ) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_card_frame(ui, |ui| {
                    ui.vertical(|ui| {


                        // 1. HEADER & PRESET MANAGEMENT BAR
                        self.render_eq_header_bar(ui, preset, is_dirty);

                        ui.add_space(8.0);

                        // Mode status notices with genuine signal chain verification
                        self.render_eq_status_notice(ui);

                        // 2. INTERACTIVE FREQUENCY RESPONSE DISPLAY
                        self.render_eq_interactive_display(ui, preset, is_dirty);

                        ui.add_space(10.0);

                        // 3. PREAMP & HEADROOM COMPENSATOR BAR
                        self.render_eq_preamp_and_headroom_bar(ui, preset, is_dirty);

                        ui.add_space(10.0);

                        // 4. SELECTED BAND PARAMETRIC INSPECTOR (Adaptive controls)
                        self.render_eq_band_inspector(ui, preset, is_dirty);

                        ui.add_space(8.0);

                        // 5. BANDS QUICK STRIP / MINI-CHIPS
                        self.render_eq_bands_strip(ui, preset);

                        ui.add_space(14.0);

                        // 6. SIGNAL TRANSPARENCY & OUTPUT METERING SECTION
                        self.render_eq_transparency_section(ui, preset, is_dirty);
                    });
                });
            });
    }

    fn render_eq_header_bar(
        &mut self,
        ui: &mut egui::Ui,
        preset: &mut ParametricEqPreset,
        is_dirty: &mut bool,
    ) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🎛️ PARAMETRIC EQUALIZER STUDIO").strong().size(15.0).color(Color32::from_rgb(15, 23, 42)));

            // Honest dirty indicator with star
            let dirty_star = if *is_dirty { "*" } else { "" };
            let preset_title = format!("Preset: \"{}{}\"", preset.name, dirty_star);
            let pill_color = if *is_dirty {
                Color32::from_rgb(249, 115, 22) // Orange dirty badge
            } else {
                Color32::from_rgb(100, 116, 139)
            };
            ui.label(egui::RichText::new(preset_title).strong().size(12.5).color(pill_color));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Modular connection status in Signal Chain Studio
                let chain_eval = self.evaluate_signal_chain();
                let is_preset_flowing = self.nodes.iter().any(|n| {
                    n.node_type == NodeType::SavedEqPreset(preset.name.clone()) && chain_eval.node_status(n.id) == SignalNodeStatus::Flowing
                });
                let is_eq_flowing = self.chain_mode != ChainMode::BitPerfect && is_preset_flowing;
                let is_preset_on_canvas = self.nodes.iter().any(|n| n.node_type == NodeType::SavedEqPreset(preset.name.clone()));

                if is_eq_flowing {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(236, 253, 245))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(167, 243, 208)))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(10, 4))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("⚡ Terhubung di Signal Chain (Live Real-Time Tuning)").strong().size(11.5).color(Color32::from_rgb(16, 185, 129)));
                        });
                } else {
                    if !is_preset_on_canvas {
                        if ui.button(egui::RichText::new("➕ Pasang ke Canvas").size(11.0).strong().color(Color32::from_rgb(14, 165, 233)))
                            .on_hover_text("Pasang preset ini langsung ke diagram Signal Chain Studio")
                            .clicked()
                        {
                            self.add_modular_filter(NodeType::SavedEqPreset(preset.name.clone()));
                        }
                    }

                    egui::Frame::new()
                        .fill(Color32::from_rgb(248, 250, 252))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(10, 4))
                        .show(ui, |ui| {
                            let status_text = if self.chain_mode == ChainMode::BitPerfect {
                                "⚪ Dilewati (Bit-Perfect)"
                            } else if is_preset_on_canvas {
                                "⚠️ Terputus di Patchbay (Audio Tidak Terpengaruh)"
                            } else {
                                "⚪ Belum Terhubung di Patchbay (Audio Tidak Terpengaruh)"
                            };
                            ui.label(egui::RichText::new(status_text).size(11.5).color(Color32::from_rgb(100, 116, 139)));
                        });
                }

                // Spectrum Overlay Toggle Button
                let spec_text = if self.eq_show_spectrum_overlay {
                    egui::RichText::new("📊 Spektrum FFT: ON").size(11.5).strong().color(Color32::from_rgb(14, 165, 233))
                } else {
                    egui::RichText::new("📊 Spektrum FFT: OFF").size(11.5).color(Color32::from_rgb(148, 163, 184))
                };
                if ui.button(spec_text).clicked() {
                    self.eq_show_spectrum_overlay = !self.eq_show_spectrum_overlay;
                }
            });
        });

        ui.add_space(6.0);

        // Preset selector in a horizontal scrollable row preventing chip overflow
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Preset:").strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));

            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let preset_names: Vec<String> = self.eq_presets.iter().map(|p| p.name.clone()).collect();
                        for name in preset_names {
                            let is_selected = preset.name == name;
                            let btn = egui::Button::new(egui::RichText::new(&name).size(11.0).strong())
                                .fill(if is_selected { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(241, 245, 249) })
                                .stroke(Stroke::new(1.0, if is_selected { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
                            if ui.add(btn).clicked() {
                                if let Some(p) = self.eq_presets.iter().find(|item| item.name == name).cloned() {
                                    *preset = p;
                                    *is_dirty = false;
                                    self.selected_band_idx = Some(0);
                                    self.active_eq = preset.clone();
                                    self.sync_parametric_to_legacy_eq();
                                }
                            }
                        }

                        ui.separator();

                        // Preset Action Buttons
                        if ui.button(egui::RichText::new("💾 Simpan").size(11.0).strong()).clicked() {
                            let cur_name = preset.name.clone();
                            let new_preset = preset.clone();
                            if let Some(pos) = self.eq_presets.iter().position(|p| p.name == cur_name) {
                                self.eq_presets[pos] = new_preset;
                            } else {
                                self.eq_presets.push(new_preset);
                            }
                            *is_dirty = false;
                            self.active_eq = preset.clone();
                            self.save_library_state();
                        }

                        if ui.button(egui::RichText::new("➕ Simpan Sebagai...").size(11.0)).clicked() {
                            self.show_save_preset_dialog = !self.show_save_preset_dialog;
                        }

                        if ui.button(egui::RichText::new("🔄 Reset").size(11.0)).clicked() {
                            let cur_name = preset.name.clone();
                            if let Some(p) = self.eq_presets.iter().find(|item| item.name == cur_name).cloned() {
                                *preset = p;
                                *is_dirty = false;
                                self.active_eq = preset.clone();
                                self.sync_parametric_to_legacy_eq();
                            } else {
                                *preset = ParametricEqPreset {
                                    name: "Flat (Netral)".to_string(),
                                    bands: ParametricBand::default_10_bands(),
                                    preamp_mode: PreampMode::Auto,
                                    manual_preamp_db: 0.0,
                                };
                                *is_dirty = false;
                                self.active_eq = preset.clone();
                                self.sync_parametric_to_legacy_eq();
                            }
                        }

                        let import_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("📥 Import AutoEq")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(14, 116, 144))
                            )
                            .fill(Color32::from_rgb(240, 249, 255))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(186, 230, 253)))
                        );
                        if import_btn.clicked() {
                            if let Some(file_path) = rfd::FileDialog::new()
                                .set_title("Pilih Berkas Profil AutoEq (ParametricEQ.txt)")
                                .add_filter("AutoEq Parametric Profile", &["txt", "csv"])
                                .pick_file()
                            {
                                if let Ok(content) = std::fs::read_to_string(&file_path) {
                                    let default_name = file_path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("AutoEq Profile")
                                        .to_string();
                                    match crate::audio::parse_autoeq_text(&default_name, &content) {
                                        Ok(new_preset) => {
                                            let name = new_preset.name.clone();
                                            let band_count = new_preset.bands.len();
                                            let preamp_db = new_preset.manual_preamp_db;

                                            *preset = new_preset.clone();
                                            *is_dirty = false;
                                            self.active_eq = new_preset.clone();

                                            self.eq_presets.retain(|p| p.name != name);
                                            self.eq_presets.push(new_preset);
                                            self.selected_band_idx = Some(0);
                                            self.sync_parametric_to_legacy_eq();
                                            self.sync_signal_chain_to_engine();
                                            self.save_library_state();

                                            self.autoeq_import_status = Some((
                                                format!("Berhasil mengimpor '{}' ({} band, Preamp {:.1} dB)", name, band_count, preamp_db),
                                                true,
                                            ));
                                        }
                                        Err(e) => {
                                            self.autoeq_import_status = Some((format!("Gagal mengimpor AutoEq: {}", e), false));
                                        }
                                    }
                                }
                            }
                        }
                        import_btn.on_hover_text("Impor profil kalibrasi headphone AutoEq (ParametricEQ.txt)");
                    });
                });
        });

        // AutoEq Status Notification Banner
        let mut dismiss_autoeq_status = false;
        if let Some((ref msg, is_ok)) = self.autoeq_import_status {
            ui.add_space(4.0);
            let (bg, border, text_color) = if is_ok {
                (Color32::from_rgb(236, 253, 245), Color32::from_rgb(167, 243, 208), Color32::from_rgb(6, 95, 70))
            } else {
                (Color32::from_rgb(254, 242, 242), Color32::from_rgb(254, 202, 202), Color32::from_rgb(153, 27, 27))
            };

            egui::Frame::new()
                .fill(bg)
                .stroke(Stroke::new(1.0, border))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let icon = if is_ok { "✨" } else { "⚠️" };
                        ui.label(egui::RichText::new(format!("{} {}", icon, msg)).size(11.5).strong().color(text_color));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                dismiss_autoeq_status = true;
                            }
                        });
                    });
                });
        }
        if dismiss_autoeq_status {
            self.autoeq_import_status = None;
        }

        // Save preset modal dialog
        if self.show_save_preset_dialog {
            ui.add_space(4.0);
            egui::Frame::new()
                .fill(Color32::from_rgb(241, 245, 249))
                .stroke(Stroke::new(1.0, Color32::from_rgb(14, 165, 233)))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Nama Preset Baru:").strong().size(12.0));
                        ui.text_edit_singleline(&mut self.eq_preset_name_input);
                        if ui.button(egui::RichText::new("Simpan Preset").strong()).clicked() {
                            let name = self.eq_preset_name_input.trim().to_string();
                            if !name.is_empty() {
                                preset.name = name.clone();
                                self.eq_presets.retain(|p| p.name != name);
                                self.eq_presets.push(preset.clone());
                                *is_dirty = false;
                                self.show_save_preset_dialog = false;
                                self.active_eq = preset.clone();
                                self.save_library_state();
                            }
                        }
                        if ui.button("Batal").clicked() {
                            self.show_save_preset_dialog = false;
                        }
                    });
                });
        }
    }

    fn render_eq_status_notice(&self, ui: &mut egui::Ui) {
        if self.chain_mode == ChainMode::BitPerfect {
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(236, 253, 245, 230))
                .stroke(Stroke::new(1.0, Color32::from_rgb(167, 243, 208)))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("⚡ Mode Bit-Perfect Aktif: Rantai equalizer dilewati murni (Bit-Perfect Passthrough) untuk menjaga keutuhan bit audio asli.").size(11.5).color(Color32::from_rgb(6, 95, 70)));
                });
            ui.add_space(6.0);
        } else if self.is_equalizer_in_active_chain() {
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(236, 253, 245, 230))
                .stroke(Stroke::new(1.0, Color32::from_rgb(167, 243, 208)))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(format!("🟢 Status: Aktif di Signal Chain (Live Real-Time Tuning) • Preset \"{}\" sedang memproses audio secara modular.", self.active_eq.name)).size(11.5).color(Color32::from_rgb(6, 95, 70)));
                });
            ui.add_space(6.0);
        } else {
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(248, 250, 252, 230))
                .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(format!("⚪ Status: Kartu EQ Tidak Terhubung di Patchbay • Perubahan slider/kurva tersimpan di preset, tetapi audio yang sedang diputar TIDAK terpengaruh sampai kartu \"{}\" disambungkan di Signal Chain Studio.", self.active_eq.name)).size(11.5).color(Color32::from_rgb(71, 85, 105)));
                });
            ui.add_space(6.0);
        }
    }

    fn render_eq_interactive_display(
        &mut self,
        ui: &mut egui::Ui,
        preset: &mut ParametricEqPreset,
        is_dirty: &mut bool,
    ) {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 250.0), egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // Dark modern pro-audio canvas background
        painter.rect_filled(rect, 8.0, Color32::from_rgb(15, 23, 42)); // Slate 900
        painter.rect_stroke(rect, 8.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Outside);

        // 1. Logarithmic Frequency Grid Lines & Labels (with inset padding so 20 and 20k never clip)
        let grid_freqs = [
            (20.0, "20"), (50.0, "50"), (100.0, "100"), (200.0, "200"),
            (500.0, "500"), (1000.0, "1k"), (2000.0, "2k"), (5000.0, "5k"),
            (10000.0, "10k"), (20000.0, "20k"),
        ];

        for &(f, label) in &grid_freqs {
            let x = freq_to_x(f, rect);
            painter.line_segment(
                [Pos2::new(x, rect.min.y + 6.0), Pos2::new(x, rect.max.y - 18.0)],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 18)),
            );
            painter.text(
                Pos2::new(x, rect.max.y - 10.0),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(9.5),
                Color32::from_rgb(100, 116, 139),
            );
        }

        // 2. Gain Grid Lines & Labels
        let grid_dbs = [18.0, 12.0, 6.0, 0.0, -6.0, -12.0, -18.0];
        for &db in &grid_dbs {
            let y = gain_to_y(db, rect);
            let is_zero = db.abs() < 0.01;
            let stroke = if is_zero {
                Stroke::new(1.2, Color32::from_rgba_unmultiplied(14, 165, 233, 100)) // Cyan 0dB baseline
            } else {
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 14))
            };
            painter.line_segment([Pos2::new(rect.min.x + 8.0, y), Pos2::new(rect.max.x - 8.0, y)], stroke);

            let label = if is_zero { "0 dB".to_string() } else { format!("{:+0.0} dB", db) };
            painter.text(
                Pos2::new(rect.min.x + 10.0, y - 2.0),
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::proportional(9.0),
                if is_zero { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(71, 85, 105) },
            );
        }

        // 3. Real-Time Hardware FFT Spectrum Analyzer Overlay (Ghost Bars in background)
        if self.eq_show_spectrum_overlay {
            let bars = 48;
            let spacing = 2.0;
            let bar_w = (rect.width() - 32.0 - (bars - 1) as f32 * spacing) / bars as f32;
            let vol_scale = if self.is_muted { 0.0 } else { self.volume.clamp(0.0, 1.2) };

            for i in 0..bars {
                let fft_val = self.spectrum_bars[i];
                if fft_val > 0.005 && vol_scale > 0.001 {
                    let h = (fft_val * 110.0 * vol_scale).clamp(2.0, 120.0);
                    let x = rect.min.x + 16.0 + i as f32 * (bar_w + spacing);
                    let y_bottom = rect.max.y - 20.0;
                    let bar_rect = Rect::from_min_max(Pos2::new(x, y_bottom - h), Pos2::new(x + bar_w, y_bottom));
                    painter.rect_filled(bar_rect, 1.5, Color32::from_rgba_unmultiplied(14, 165, 233, 28));
                }
            }
        }

        // 4. Calculate Combined Peak Boost & Response Curve Points (Optimized with Hash Caching & Single-Mesh Batching)
        let max_boost = calculate_combined_peak_boost(&preset.bands, 44100.0);
        let effective_preamp = if preset.preamp_mode == PreampMode::Auto {
            if max_boost > 0.0 { -max_boost } else { 0.0 }
        } else {
            preset.manual_preamp_db
        };

        // Cache curve points to eliminate 2,800 trigonometric calculations on static frames
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        preset.bands.len().hash(&mut hasher);
        for b in &preset.bands {
            (b.freq as u32).hash(&mut hasher);
            ((b.gain_db * 100.0) as i32).hash(&mut hasher);
            ((b.q * 100.0) as u32).hash(&mut hasher);
            (b.filter_type as u8).hash(&mut hasher);
            b.enabled.hash(&mut hasher);
        }
        ((effective_preamp * 100.0) as i32).hash(&mut hasher);
        self.eq_bypass.hash(&mut hasher);
        (rect.min.x as i32).hash(&mut hasher);
        (rect.min.y as i32).hash(&mut hasher);
        (rect.width() as i32).hash(&mut hasher);
        (rect.height() as i32).hash(&mut hasher);
        let curve_hash = hasher.finish();

        let cached_points = {
            let borrow = self.eq_curve_cache.borrow();
            if let Some((h, ref pts)) = *borrow {
                if h == curve_hash {
                    Some(pts.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        let curve_points: Vec<Pos2> = if let Some(pts) = cached_points {
            pts
        } else {
            let sample_rate = 44100.0_f64;
            let points_count = 280;
            let mut pts = Vec::with_capacity(points_count);
            for i in 0..points_count {
                let t = i as f32 / (points_count - 1) as f32;
                let f = 20.0 * (1000.0_f32).powf(t);
                let total_db = compute_total_curve_db(
                    &preset.bands,
                    f as f64,
                    sample_rate,
                    effective_preamp as f64,
                    self.eq_bypass,
                );
                let x = freq_to_x(f, rect);
                let y = gain_to_y(total_db as f32, rect);
                pts.push(Pos2::new(x, y));
            }
            *self.eq_curve_cache.borrow_mut() = Some((curve_hash, pts.clone()));
            pts
        };

        // Draw Filled Area between response curve and 0 dB baseline (1 Single Mesh instead of 279 separate shapes)
        let zero_y = gain_to_y(0.0, rect);
        if curve_points.len() >= 2 && !self.eq_bypass {
            let mut fill_mesh = egui::Mesh::default();
            for i in 0..(curve_points.len() - 1) {
                let p1 = curve_points[i];
                let p2 = curve_points[i + 1];
                let is_boost = p1.y < zero_y || p2.y < zero_y;
                let fill_col = if is_boost {
                    Color32::from_rgba_unmultiplied(14, 165, 233, 30) // Cyan translucent fill
                } else {
                    Color32::from_rgba_unmultiplied(244, 63, 94, 20)  // Subtle rose for cut
                };
                let base_idx = fill_mesh.vertices.len() as u32;
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(p1.x, zero_y), uv: egui::epaint::WHITE_UV, color: fill_col });
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: p1, uv: egui::epaint::WHITE_UV, color: fill_col });
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: p2, uv: egui::epaint::WHITE_UV, color: fill_col });
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(p2.x, zero_y), uv: egui::epaint::WHITE_UV, color: fill_col });
                fill_mesh.indices.extend_from_slice(&[
                    base_idx, base_idx + 1, base_idx + 2,
                    base_idx, base_idx + 2, base_idx + 3,
                ]);
            }
            painter.add(egui::Shape::Mesh(std::sync::Arc::new(fill_mesh)));
        }

        // Draw Response Curve Stroke (1 Single Vectorized Polyline instead of 279 line segments)
        let curve_stroke = if self.eq_bypass {
            Stroke::new(1.8, Color32::from_rgba_unmultiplied(148, 163, 184, 160)) // Muted dashed
        } else {
            Stroke::new(2.4, Color32::from_rgb(56, 189, 248)) // Glowing sky cyan
        };

        if curve_points.len() >= 2 {
            painter.add(egui::Shape::line(curve_points.clone(), curve_stroke));
        }

        // 5. Interactive Draggable Control Nodes
        let mut hovered_band: Option<usize> = None;
        let pointer_pos = response.hover_pos();
        let is_dragging = response.dragged();

        let num_bands = preset.bands.len();
        for i in 0..num_bands {
            let band = &preset.bands[i];
            let nx = freq_to_x(band.freq, rect);
            let ny = if band.filter_type.has_gain() {
                gain_to_y(band.gain_db, rect)
            } else {
                gain_to_y(0.0, rect)
            };
            let node_pos = Pos2::new(nx, ny);

            let is_selected = self.selected_band_idx == Some(i);
            let mut is_hovered = false;

            if let Some(pos) = pointer_pos {
                if pos.distance(node_pos) <= 12.0 {
                    is_hovered = true;
                    hovered_band = Some(i);
                }
            }

            // Outer ring
            let col = filter_color(band.filter_type);
            if is_selected || is_hovered {
                painter.circle_stroke(node_pos, 11.5, Stroke::new(2.2, Color32::WHITE));
                painter.circle_filled(node_pos, 8.5, col);
            } else {
                painter.circle_stroke(node_pos, 8.5, Stroke::new(1.2, Color32::from_rgba_unmultiplied(255, 255, 255, 180)));
                painter.circle_filled(node_pos, 7.0, if band.enabled { col } else { Color32::from_rgb(100, 116, 139) });
            }

            // Band index number inside node
            painter.text(
                node_pos,
                egui::Align2::CENTER_CENTER,
                format!("{}", i + 1),
                egui::FontId::proportional(9.0),
                Color32::from_rgb(15, 23, 42),
            );
        }

        // Handle Mouse Clicks & Drags on Nodes
        if response.clicked() {
            if let Some(idx) = hovered_band {
                self.selected_band_idx = Some(idx);
            } else if response.double_clicked() {
                // Double click on empty space creates a new band!
                if let Some(pos) = pointer_pos {
                    if preset.bands.len() < 32 {
                        let new_freq = x_to_freq(pos.x, rect);
                        let new_gain = y_to_gain(pos.y, rect);
                        let new_band = ParametricBand::new(FilterType::Peaking, new_freq, new_gain, 1.414);
                        preset.bands.push(new_band);
                        self.selected_band_idx = Some(preset.bands.len() - 1);
                        *is_dirty = true;
                        self.active_eq = preset.clone();
                        self.sync_parametric_to_legacy_eq();
                    }
                }
            }
        }

        if is_dragging {
            if let Some(pos) = pointer_pos {
                if self.dragging_band_idx.is_none() {
                    self.dragging_band_idx = hovered_band;
                }
                if let Some(d_idx) = self.dragging_band_idx {
                    if d_idx < preset.bands.len() {
                        self.selected_band_idx = Some(d_idx);
                        let new_f = x_to_freq(pos.x, rect);
                        preset.bands[d_idx].freq = new_f;
                        if preset.bands[d_idx].filter_type.has_gain() {
                            let new_g = y_to_gain(pos.y, rect);
                            preset.bands[d_idx].gain_db = new_g;
                        }
                        *is_dirty = true;
                        self.active_eq = preset.clone();
                        self.sync_parametric_to_legacy_eq();
                    }
                }
            }
        } else {
            self.dragging_band_idx = None;
        }

        // Mouse Wheel over node adjusts Q factor (logarithmic feel)
        if let Some(h_idx) = hovered_band {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.5 {
                let factor = if scroll_delta > 0.0 { 1.05 } else { 0.95 };
                let new_q = (preset.bands[h_idx].q * factor).clamp(0.1, 10.0);
                preset.bands[h_idx].q = new_q;
                *is_dirty = true;
                self.active_eq = preset.clone();
                self.sync_parametric_to_legacy_eq();
            }

            // Hover tooltip
            let band = &preset.bands[h_idx];
            let gain_str = if band.filter_type.has_gain() {
                format!("Gain: {}\n", format_db_clean(band.gain_db))
            } else {
                String::new()
            };
            let slope_str = match band.filter_type {
                FilterType::HighPass | FilterType::LowPass => format!("Slope: {}\n", band.slope.name()),
                _ => String::new(),
            };
            if !self.is_modal_or_overlay_active() {
                response.show_tooltip_text(format!(
                    "Band {} • {}\nFrekuensi: {}\n{}{}Q-Factor: {:0.2}\n(Gulir mouse untuk atur Q)",
                    h_idx + 1,
                    band.filter_type.name(),
                    format_freq(band.freq),
                    gain_str,
                    slope_str,
                    band.q
                ));
            }
        }
    }

    fn render_eq_preamp_and_headroom_bar(
        &mut self,
        ui: &mut egui::Ui,
        preset: &mut ParametricEqPreset,
        is_dirty: &mut bool,
    ) {
        let max_boost = calculate_combined_peak_boost(&preset.bands, 44100.0);
        let required_attenuation = if max_boost > 0.005 { -max_boost } else { 0.0 };

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Headroom & Preamp:").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));

            // Preamp Mode: Auto vs Manual
            let auto_active = preset.preamp_mode == PreampMode::Auto;
            let auto_btn = egui::Button::new(egui::RichText::new("Auto Preamp").size(11.5).strong())
                .fill(if auto_active { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(241, 245, 249) })
                .stroke(Stroke::new(1.0, if auto_active { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
            if ui.add(auto_btn).clicked() {
                preset.preamp_mode = PreampMode::Auto;
                *is_dirty = true;
                self.active_eq = preset.clone();
                self.sync_parametric_to_legacy_eq();
            }

            let manual_active = preset.preamp_mode == PreampMode::Manual;
            let man_btn = egui::Button::new(egui::RichText::new("Manual").size(11.5).strong())
                .fill(if manual_active { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(241, 245, 249) })
                .stroke(Stroke::new(1.0, if manual_active { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
            if ui.add(man_btn).clicked() {
                preset.preamp_mode = PreampMode::Manual;
                *is_dirty = true;
                self.active_eq = preset.clone();
                self.sync_parametric_to_legacy_eq();
            }

            ui.add_space(8.0);

            // Preamp Slider & Display (Without -0.0 dB artifact!)
            if preset.preamp_mode == PreampMode::Auto {
                ui.label(
                    egui::RichText::new(format!("Kompensasi Otomatis: {}", format_db_clean(required_attenuation)))
                        .strong()
                        .size(12.0)
                        .color(Color32::from_rgb(14, 165, 233)),
                );
            } else {
                ui.label(egui::RichText::new(format!("Gain: {}", format_db_clean(preset.manual_preamp_db))).strong().size(12.0));
                let s_res = ui.add(egui::Slider::new(&mut preset.manual_preamp_db, -24.0..=12.0).suffix(" dB").show_value(false));
                if s_res.changed() {
                    *is_dirty = true;
                    self.active_eq = preset.clone();
                    self.sync_parametric_to_legacy_eq();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Headroom Badge & Warning
                let effective_preamp = if preset.preamp_mode == PreampMode::Auto {
                    required_attenuation
                } else {
                    preset.manual_preamp_db
                };

                let peak_level = effective_preamp + max_boost;
                if peak_level > 0.05 {
                    let pill = egui::Frame::new()
                        .fill(Color32::from_rgb(254, 242, 242))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::symmetric(8, 3));
                    pill.show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("⚠️ Risiko Clipping: Defisit {:+0.1} dBFS", peak_level)).strong().size(11.5).color(Color32::from_rgb(220, 38, 38)));
                    });
                } else {
                    let pill = egui::Frame::new()
                        .fill(Color32::from_rgb(240, 253, 244))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(34, 197, 94)))
                        .corner_radius(4.0)
                        .inner_margin(egui::Margin::symmetric(8, 3));
                    pill.show(ui, |ui| {
                        ui.label(egui::RichText::new(format!("✓ Headroom Aman: {}", format_margin_clean(-peak_level))).strong().size(11.5).color(Color32::from_rgb(22, 163, 74)));
                    });
                }
            });
        });
    }

    fn render_eq_band_inspector(
        &mut self,
        ui: &mut egui::Ui,
        preset: &mut ParametricEqPreset,
        is_dirty: &mut bool,
    ) {
        let selected_idx = self.selected_band_idx.unwrap_or(0);
        if selected_idx >= preset.bands.len() {
            return;
        }

        let total_bands = preset.bands.len();
        let mut band = preset.bands[selected_idx].clone();
        let mut changed = false;
        let mut delete_band = false;
        let mut add_band = false;

        egui::Frame::new()
            .fill(Color32::from_rgb(248, 250, 252))
            .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Band Index & Status
                    let mut is_enabled = band.enabled;
                    if ui.checkbox(&mut is_enabled, egui::RichText::new(format!("BAND {} / {}", selected_idx + 1, total_bands)).strong().size(13.0)).changed() {
                        band.enabled = is_enabled;
                        changed = true;
                    }

                    ui.add_space(8.0);

                    // Filter Type Buttons
                    let types = [
                        FilterType::Peaking,
                        FilterType::LowShelf,
                        FilterType::HighShelf,
                        FilterType::HighPass,
                        FilterType::LowPass,
                        FilterType::Notch,
                    ];

                    for f_type in types {
                        let is_cur = band.filter_type == f_type;
                        let btn = egui::Button::new(egui::RichText::new(f_type.short_name()).size(11.0).strong())
                            .fill(if is_cur { filter_color(f_type) } else { Color32::from_rgb(241, 245, 249) })
                            .stroke(Stroke::new(1.0, if is_cur { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) }));
                        if ui.add(btn).clicked() {
                            band.filter_type = f_type;
                            changed = true;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete Band Button
                        let del_btn = egui::Button::new(egui::RichText::new("🗑 Hapus Band").size(11.0).color(Color32::from_rgb(239, 68, 68)));
                        if ui.add_enabled(total_bands > 1, del_btn).clicked() {
                            delete_band = true;
                        }

                        // Add Band Button
                        let add_btn = egui::Button::new(egui::RichText::new("➕ Tambah Band").size(11.0).strong().color(Color32::from_rgb(14, 165, 233)));
                        if ui.add_enabled(total_bands < 32, add_btn).clicked() {
                            add_band = true;
                        }
                    });
                });

                ui.add_space(8.0);

                // Adaptive Controls: Frequency, Gain/Slope, Q Factor
                ui.columns(3, |columns| {
                    // Column 1: Frequency
                    columns[0].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Frekuensi:").strong().size(12.0));
                            let dv = ui.add(egui::DragValue::new(&mut band.freq).range(20.0..=20000.0).speed(5.0).custom_formatter(|f, _| format_freq(f as f32)));
                            if dv.changed() {
                                changed = true;
                            }
                        });
                        let s = ui.add(egui::Slider::new(&mut band.freq, 20.0..=20000.0).logarithmic(true).show_value(false));
                        if s.changed() {
                            changed = true;
                        }
                    });

                    // Column 2: Gain OR Filter Slope (Adaptive!)
                    columns[1].vertical(|ui| {
                        match band.filter_type {
                            FilterType::HighPass | FilterType::LowPass => {
                                // HPF / LPF has no Gain: Show Slope Selector (12/24/48 dB/oct)
                                ui.label(egui::RichText::new("Slope Filter:").strong().size(12.0));
                                ui.horizontal(|ui| {
                                    let slopes = [
                                        (FilterSlope::Db12, "12 dB/oct"),
                                        (FilterSlope::Db24, "24 dB/oct"),
                                        (FilterSlope::Db48, "48 dB/oct"),
                                    ];
                                    for (sl, label) in slopes {
                                        let is_cur = band.slope == sl;
                                        let btn = egui::Button::new(egui::RichText::new(label).size(10.5).strong())
                                            .fill(if is_cur { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(241, 245, 249) })
                                            .stroke(Stroke::new(1.0, if is_cur { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
                                        if ui.add(btn).clicked() {
                                            band.slope = sl;
                                            changed = true;
                                        }
                                    }
                                });
                            }
                            FilterType::Notch => {
                                // Notch has no Gain: Label indicating band-stop
                                ui.label(egui::RichText::new("Karakteristik:").strong().size(12.0));
                                ui.label(egui::RichText::new("Zero-Gain Band Stop Notch").size(11.0).color(Color32::from_rgb(100, 116, 139)));
                            }
                            FilterType::Peaking | FilterType::LowShelf | FilterType::HighShelf => {
                                // Peaking and Shelves have Gain slider
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Gain:").strong().size(12.0));
                                    let dv = ui.add(egui::DragValue::new(&mut band.gain_db).range(-18.0..=18.0).speed(0.1).custom_formatter(|g, _| format_db_clean(g as f32)));
                                    if dv.changed() {
                                        changed = true;
                                    }
                                    if ui.button(egui::RichText::new("0 dB").size(10.0)).clicked() {
                                        band.gain_db = 0.0;
                                        changed = true;
                                    }
                                });
                                let s = ui.add(egui::Slider::new(&mut band.gain_db, -18.0..=18.0).show_value(false));
                                if s.changed() {
                                    changed = true;
                                }
                            }
                        }
                    });

                    // Column 3: Q Factor (Logarithmic slider centered at 1.0..1.414)
                    columns[2].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Q Factor:").strong().size(12.0));
                            let dv = ui.add(egui::DragValue::new(&mut band.q).range(0.1..=10.0).speed(0.02));
                            if dv.changed() {
                                changed = true;
                            }
                        });
                        let s = ui.add(egui::Slider::new(&mut band.q, 0.1..=10.0).logarithmic(true).show_value(false));
                        if s.changed() {
                            changed = true;
                        }
                    });
                });
            });

        if delete_band {
            if preset.bands.len() > 1 {
                preset.bands.remove(selected_idx);
                if selected_idx >= preset.bands.len() && !preset.bands.is_empty() {
                    self.selected_band_idx = Some(preset.bands.len() - 1);
                }
                *is_dirty = true;
                self.active_eq = preset.clone();
                self.sync_parametric_to_legacy_eq();
            }
        } else if add_band {
            if preset.bands.len() < 32 {
                let new_band = ParametricBand::new(FilterType::Peaking, 1000.0, 0.0, 1.414);
                preset.bands.push(new_band);
                self.selected_band_idx = Some(preset.bands.len() - 1);
                *is_dirty = true;
                self.active_eq = preset.clone();
                self.sync_parametric_to_legacy_eq();
            }
        } else if changed {
            preset.bands[selected_idx] = band;
            *is_dirty = true;
            self.active_eq = preset.clone();
            self.sync_parametric_to_legacy_eq();
        }
    }

    fn render_eq_bands_strip(&mut self, ui: &mut egui::Ui, preset: &ParametricEqPreset) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Bands:").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
            let num_bands = preset.bands.len();
            for i in 0..num_bands {
                let band = &preset.bands[i];
                let is_selected = self.selected_band_idx == Some(i);

                let label = if band.filter_type.has_gain() {
                    format!("#{}: {} {} ({})", i + 1, band.filter_type.short_name(), format_freq(band.freq), format_db_clean(band.gain_db))
                } else {
                    format!("#{}: {} {}", i + 1, band.filter_type.short_name(), format_freq(band.freq))
                };

                let col = filter_color(band.filter_type);
                let btn = egui::Button::new(egui::RichText::new(label).size(11.0).strong().color(if is_selected { Color32::WHITE } else { Color32::from_rgb(30, 41, 59) }))
                    .fill(if is_selected { col } else { Color32::from_rgb(241, 245, 249) })
                    .stroke(Stroke::new(1.0, if is_selected { col } else { Color32::from_rgb(203, 213, 225) }));

                if ui.add(btn).clicked() {
                    self.selected_band_idx = Some(i);
                }
            }
        });
    }

    fn render_eq_transparency_section(
        &mut self,
        ui: &mut egui::Ui,
        preset: &ParametricEqPreset,
        is_dirty: &mut bool,
    ) {
        egui::Frame::new()
            .fill(Color32::from_rgb(241, 245, 249))
            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔊 I/O LEVEL & OUTPUT CONFIGURATION").strong().size(13.0).color(Color32::from_rgb(15, 23, 42)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Pemisahan Pengaturan Output Global dari Preset EQ").size(11.0).color(Color32::from_rgb(100, 116, 139)));
                    });
                });

                ui.add_space(8.0);

                ui.columns(3, |columns| {
                    // Column 1: I/O Peak Metering & Clip Counter (Consistent on Flat EQ!)
                    columns[0].vertical(|ui| {
                        ui.label(egui::RichText::new("Sample Peak (dBFS)").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
                        ui.add_space(4.0);

                        let vol = if self.is_muted { 0.0 } else { self.volume.clamp(0.0, 1.2) };
                        let in_peak_db = if vol > 0.001 { -18.0 + vol * 8.0 } else { -60.0 };

                        let max_boost = calculate_combined_peak_boost(&preset.bands, 44100.0);
                        let is_flat_or_bypassed = self.eq_bypass || (max_boost < 0.01 && preset.manual_preamp_db.abs() < 0.01);

                        // On flat EQ, In Peak == Out Peak consistently!
                        let out_peak_db = if is_flat_or_bypassed {
                            in_peak_db
                        } else {
                            let effective_preamp = if preset.preamp_mode == PreampMode::Auto {
                                -max_boost
                            } else {
                                preset.manual_preamp_db
                            };
                            in_peak_db + effective_preamp + max_boost
                        };

                        ui.label(egui::RichText::new(format!("In: {:0.1} dBFS  •  Out: {:0.1} dBFS", in_peak_db, out_peak_db)).size(11.0).color(Color32::from_rgb(71, 85, 105)));

                        ui.horizontal(|ui| {
                            let clip_badge = if self.eq_clip_count == 0 {
                                egui::RichText::new("0 Clips").size(11.0).strong().color(Color32::from_rgb(16, 185, 129))
                            } else {
                                egui::RichText::new(format!("{} Clips!", self.eq_clip_count)).size(11.0).strong().color(Color32::from_rgb(239, 68, 68))
                            };
                            ui.label(clip_badge);
                            if ui.button(egui::RichText::new("Reset").size(10.5)).clicked() {
                                self.eq_clip_count = 0;
                            }
                        });
                    });

                    // Column 2: Limiter Option (Output Stage: Default OFF for bit-accuracy)
                    columns[1].vertical(|ui| {
                        ui.label(egui::RichText::new("Soft Limiter").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
                        ui.add_space(4.0);

                        let mut lim = self.eq_limiter_enabled;
                        let text = if lim {
                            egui::RichText::new("Aktif (-0.9 dBFS)").strong().size(11.5).color(Color32::from_rgb(234, 88, 12))
                        } else {
                            egui::RichText::new("Mati (Transparan)").strong().size(11.5).color(Color32::from_rgb(16, 185, 129))
                        };

                        if ui.checkbox(&mut lim, text).changed() {
                            self.eq_limiter_enabled = lim;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_limiter_enabled(lim);
                            }
                            self.save_library_state();
                            *is_dirty = true;
                        }

                        ui.label(egui::RichText::new("Default MATI agar audio murni linear tanpa pewarnaan.").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                    });

                    // Column 3: Dither Selector (Output Stage)
                    columns[2].vertical(|ui| {
                        ui.label(egui::RichText::new("Dither Output DAC").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            let modes = [
                                (DitherMode::None, "None"),
                                (DitherMode::Tpdf, "TPDF"),
                                (DitherMode::NoiseShaped, "Shaped"),
                            ];

                            for (m, label) in modes {
                                let is_cur = self.eq_dither_mode == m;
                                let btn = egui::Button::new(egui::RichText::new(label).size(10.5).strong())
                                    .fill(if is_cur { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(248, 250, 252) })
                                    .stroke(Stroke::new(1.0, if is_cur { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
                                if ui.add(btn).clicked() {
                                    self.eq_dither_mode = m;
                                    *is_dirty = true;
                                }
                            }
                        });

                        ui.label(egui::RichText::new("Diterapkan pada stage output saat bit depth DAC < sumber.").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                // Section: ReplayGain & Clipping Prevention
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔊 REPLAYGAIN & ANTI-CLIPPING").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let eff_db = self.effective_replay_gain_db;
                        let eff_text = if self.chain_mode == ChainMode::BitPerfect {
                            "Bypass (Bit-Perfect Mode)".to_string()
                        } else if self.replay_gain_mode == crate::models::ReplayGainMode::Disabled {
                            "Nonaktif (0.0 dB)".to_string()
                        } else {
                            format!("{:+.1} dB ({:.2}x)", eff_db, self.effective_replay_gain_linear)
                        };
                        ui.label(egui::RichText::new(format!("Effective Gain: {}", eff_text)).size(11.0).strong().color(Color32::from_rgb(2, 132, 199)));
                    });
                });

                ui.add_space(4.0);
                ui.columns(3, |rg_cols| {
                    // Col 1: Mode
                    rg_cols[0].vertical(|ui| {
                        ui.label(egui::RichText::new("Mode ReplayGain").strong().size(11.0).color(Color32::from_rgb(51, 65, 85)));
                        ui.add_space(2.0);
                        let modes = [
                            (crate::models::ReplayGainMode::Auto, "Auto (Smart)"),
                            (crate::models::ReplayGainMode::Track, "Track Gain"),
                            (crate::models::ReplayGainMode::Album, "Album Gain"),
                            (crate::models::ReplayGainMode::Disabled, "Disabled"),
                        ];
                        for (m, lbl) in modes {
                            let is_cur = self.replay_gain_mode == m;
                            if ui.selectable_label(is_cur, egui::RichText::new(lbl).size(10.5)).clicked() {
                                self.replay_gain_mode = m;
                                self.update_effective_replay_gain();
                                self.save_library_state();
                            }
                        }
                    });

                    // Col 2: Clip Prevention
                    rg_cols[1].vertical(|ui| {
                        ui.label(egui::RichText::new("Pencegah Clipping").strong().size(11.0).color(Color32::from_rgb(51, 65, 85)));
                        ui.add_space(2.0);
                        let clips = [
                            (crate::models::ReplayGainClipPrevention::PreventByPeak, "Prevent by Peak (Scale Down)"),
                            (crate::models::ReplayGainClipPrevention::Limiter, "Apply Soft Limiter"),
                            (crate::models::ReplayGainClipPrevention::None, "None (Unprotected)"),
                        ];
                        for (cp, lbl) in clips {
                            let is_cur = self.replay_gain_clip_prevention == cp;
                            if ui.selectable_label(is_cur, egui::RichText::new(lbl).size(10.5)).clicked() {
                                self.replay_gain_clip_prevention = cp;
                                self.update_effective_replay_gain();
                                self.save_library_state();
                            }
                        }
                    });

                    // Col 3: Preamp Slider
                    rg_cols[2].vertical(|ui| {
                        ui.label(egui::RichText::new("ReplayGain Preamp").strong().size(11.0).color(Color32::from_rgb(51, 65, 85)));
                        ui.add_space(2.0);
                        let mut preamp = self.replay_gain_preamp_db;
                        let slider = egui::Slider::new(&mut preamp, -12.0..=12.0)
                            .suffix(" dB")
                            .step_by(0.5);
                        if ui.add(slider).changed() {
                            self.replay_gain_preamp_db = preamp;
                            self.update_effective_replay_gain();
                            self.save_library_state();
                        }
                        if ui.button(egui::RichText::new("Reset Preamp 0 dB").size(10.0)).clicked() {
                            self.replay_gain_preamp_db = 0.0;
                            self.update_effective_replay_gain();
                            self.save_library_state();
                        }
                    });
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔀 TRANSISI CROSSFADE:").strong().size(11.0).color(Color32::from_rgb(15, 23, 42)));
                    let mut cf = self.crossfade_secs;
                    let cf_slider = egui::Slider::new(&mut cf, 0.0..=12.0)
                        .suffix(" s (0 = Gapless)")
                        .step_by(0.5);
                    if ui.add(cf_slider).changed() {
                        self.set_crossfade_secs(cf);
                    }
                    if cf > 0.0 {
                        ui.label(egui::RichText::new(format!("Overlap: {:.1} detik", cf)).size(10.5).color(Color32::from_rgb(14, 165, 233)));
                    } else {
                        ui.label(egui::RichText::new("⚡ Gapless Murni 0 ms").size(10.5).color(Color32::from_rgb(16, 185, 129)));
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;

    #[test]
    fn test_peaking_filter_response_matches_gain() {
        let band = ParametricBand::new(FilterType::Peaking, 1000.0, 6.0, 1.414);
        let sample_rate = 44100.0;

        let resp_center = compute_biquad_response_db(&band, 1000.0, sample_rate);
        assert!((resp_center - 6.0).abs() < 0.05, "Center freq gain was {}", resp_center);

        let resp_far = compute_biquad_response_db(&band, 100.0, sample_rate);
        assert!(resp_far.abs() < 0.15, "Far freq gain was {}", resp_far);
    }

    #[test]
    fn test_low_shelf_filter_response() {
        let band = ParametricBand::new(FilterType::LowShelf, 100.0, 5.0, 0.707);
        let sample_rate = 44100.0;

        let resp_bass = compute_biquad_response_db(&band, 25.0, sample_rate);
        assert!((resp_bass - 5.0).abs() < 0.15, "Shelf bass response was {}", resp_bass);

        let resp_treble = compute_biquad_response_db(&band, 10000.0, sample_rate);
        assert!(resp_treble.abs() < 0.10, "Shelf treble response was {}", resp_treble);
    }

    #[test]
    fn test_high_shelf_filter_response() {
        let band = ParametricBand::new(FilterType::HighShelf, 10000.0, 4.0, 0.707);
        let sample_rate = 44100.0;

        let resp_air = compute_biquad_response_db(&band, 18000.0, sample_rate);
        assert!((resp_air - 4.0).abs() < 0.20, "Shelf air response was {}", resp_air);

        let resp_bass = compute_biquad_response_db(&band, 100.0, sample_rate);
        assert!(resp_bass.abs() < 0.10, "Shelf bass response was {}", resp_bass);
    }

    #[test]
    fn test_high_pass_filter_slopes() {
        let band_12 = ParametricBand::new_cut(FilterType::HighPass, 100.0, 0.707, FilterSlope::Db12);
        let band_24 = ParametricBand::new_cut(FilterType::HighPass, 100.0, 0.707, FilterSlope::Db24);
        let band_48 = ParametricBand::new_cut(FilterType::HighPass, 100.0, 0.707, FilterSlope::Db48);
        let sample_rate = 44100.0;

        // 1 octave below cutoff (50 Hz)
        let resp_12 = compute_biquad_response_db(&band_12, 50.0, sample_rate);
        let resp_24 = compute_biquad_response_db(&band_24, 50.0, sample_rate);
        let resp_48 = compute_biquad_response_db(&band_48, 50.0, sample_rate);

        assert!((resp_12 - (-12.0)).abs() < 2.5, "12dB slope at 50Hz was {}", resp_12);
        assert!((resp_24 - (-24.0)).abs() < 5.0, "24dB slope at 50Hz was {}", resp_24);
        assert!((resp_48 - (-48.0)).abs() < 10.0, "48dB slope at 50Hz was {}", resp_48);

        // Passband (1 kHz) should pass transparently (~ 0 dB)
        assert!(compute_biquad_response_db(&band_12, 1000.0, sample_rate).abs() < 0.05);
        assert!(compute_biquad_response_db(&band_24, 1000.0, sample_rate).abs() < 0.05);
        assert!(compute_biquad_response_db(&band_48, 1000.0, sample_rate).abs() < 0.05);
    }

    #[test]
    fn test_notch_filter_response() {
        let band = ParametricBand::new(FilterType::Notch, 1000.0, 0.0, 2.0);
        let sample_rate = 44100.0;

        let resp_notch = compute_biquad_response_db(&band, 1000.0, sample_rate);
        assert!(resp_notch < -25.0, "Notch attenuation was {}", resp_notch);

        let resp_pass = compute_biquad_response_db(&band, 100.0, sample_rate);
        assert!(resp_pass.abs() < 0.10, "Notch pass response was {}", resp_pass);
    }

    #[test]
    fn test_coordinate_mapping_roundtrip() {
        let rect = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(800.0, 250.0));

        let test_freqs = [20.0, 60.0, 100.0, 1000.0, 5000.0, 10000.0, 20000.0];
        for &f in &test_freqs {
            let x = freq_to_x(f, rect);
            let roundtrip_f = x_to_freq(x, rect);
            let rel_err = ((roundtrip_f - f) / f).abs();
            assert!(rel_err < 0.001, "Freq roundtrip failed for {}: got {}, err {}", f, roundtrip_f, rel_err);
        }

        let test_gains = [-18.0, -12.0, -6.0, 0.0, 3.5, 9.0, 18.0];
        for &g in &test_gains {
            let y = gain_to_y(g, rect);
            let roundtrip_g = y_to_gain(y, rect);
            let abs_err = (roundtrip_g - g).abs();
            assert!(abs_err < 0.02, "Gain roundtrip failed for {}: got {}, err {}", g, roundtrip_g, abs_err);
        }
    }

    #[test]
    fn test_combined_curve_peak_boost_adjacent_bands() {
        // Two adjacent +6 dB bands at 800 Hz and 1200 Hz
        let bands = vec![
            ParametricBand::new(FilterType::Peaking, 800.0, 6.0, 1.414),
            ParametricBand::new(FilterType::Peaking, 1200.0, 6.0, 1.414),
        ];

        let combined_peak = calculate_combined_peak_boost(&bands, 44100.0);
        // Combined boost must exceed individual band max (+6.0 dB) due to overlap
        assert!(combined_peak > 6.5, "Combined peak should exceed 6.5 dB, got {}", combined_peak);
        assert!(combined_peak < 9.5, "Combined peak should be reasonable (< 9.5 dB), got {}", combined_peak);
    }

    #[test]
    fn test_clean_format_helpers_no_negative_zero() {
        assert_eq!(format_db_clean(0.0), "0.0 dB");
        assert_eq!(format_db_clean(-0.0), "0.0 dB");
        assert_eq!(format_db_clean(-0.0001), "0.0 dB");
        assert_eq!(format_db_clean(5.5), "+5.5 dB");
        assert_eq!(format_db_clean(-3.2), "-3.2 dB");

        assert_eq!(format_margin_clean(0.0), "0.0 dB Margin");
        assert_eq!(format_margin_clean(-0.0), "0.0 dB Margin");
        assert_eq!(format_margin_clean(6.0), "6.0 dB Margin");

        assert_eq!(format_freq(31.0), "31 Hz");
        assert_eq!(format_freq(1000.0), "1 kHz");
        assert_eq!(format_freq(2500.0), "2.5 kHz");
        assert_eq!(format_freq(16000.0), "16 kHz");
    }

    #[test]
    fn test_equalizer_panel_headless_render() {
        let ctx = egui::Context::default();
        let screen1 = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0)));
        let screen2 = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1024.0, 768.0)));

        let mut app = LunulaApp::default();
        app.active_eq = ParametricEqPreset::default_presets()[0].clone();
        assert_eq!(app.active_eq.bands.len(), 10);
        assert!(!app.eq_is_dirty);

        // Frame 1: Initial render
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen1,
                time: Some(0.1),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_equalizer_panel(ui);
                });
            },
        );
        out.textures_delta.clear();

        // Frame 2: Resize screen and modify band gain (guarantees hash change and verifies no RefCell borrow panics)
        app.active_eq.bands[0].gain_db = 4.5;
        let mut out2 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen2,
                time: Some(0.2),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_equalizer_panel(ui);
                });
            },
        );
        out2.textures_delta.clear();

        // Frame 3: Toggle bypass
        app.eq_bypass = true;
        let mut out3 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen2,
                time: Some(0.3),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_equalizer_panel(ui);
                });
            },
        );
        out3.textures_delta.clear();
    }
}
