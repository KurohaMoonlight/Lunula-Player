use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::app::LunulaApp;
use crate::models::{DitherMode, FilterType, ParametricBand, ParametricEqPreset, PreampMode};
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
    (10.0 * mag_sq.max(1e-12).log10()).clamp(-60.0, 30.0)
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
    total_db.clamp(-60.0, 30.0)
}

pub fn calculate_max_boost(bands: &[ParametricBand], sample_rate: f64) -> f32 {
    let mut max_boost = 0.0_f64;
    for i in 0..=48 {
        let t = i as f64 / 48.0;
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

pub fn freq_to_x(freq: f32, rect: Rect) -> f32 {
    let min_f = 20.0_f32;
    let max_f = 20000.0_f32;
    let f = freq.clamp(min_f, max_f);
    let t = (f / min_f).log10() / (max_f / min_f).log10();
    rect.min.x + t * rect.width()
}

pub fn x_to_freq(x: f32, rect: Rect) -> f32 {
    let min_f = 20.0_f32;
    let max_f = 20000.0_f32;
    let t = ((x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
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
    pub fn sync_parametric_to_legacy_eq(&mut self) {
        let legacy_freqs = [31.0, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
        let max_boost = calculate_max_boost(&self.parametric_bands, 44100.0);
        let preamp = if self.eq_preamp_mode == PreampMode::Auto {
            if max_boost > 0.0 { -max_boost } else { 0.0 }
        } else {
            self.eq_manual_preamp_db
        };

        for (i, &f) in legacy_freqs.iter().enumerate() {
            let total_db = compute_total_curve_db(
                &self.parametric_bands,
                f,
                44100.0,
                preamp as f64,
                self.eq_bypass,
            );
            self.eq_bands[i] = (total_db as f32).clamp(-12.0, 12.0);
            if let Some(ref engine) = self.audio_engine {
                engine.set_eq_band(i, self.eq_bands[i]);
            }
        }
        if let Some(ref engine) = self.audio_engine {
            engine.set_eq_bypass(self.eq_bypass);
        }
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

                let (rect, _response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 115.0), egui::Sense::hover());
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
                        (combined * 88.0).clamp(3.0, 96.0)
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

    pub fn render_equalizer_panel(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_card_frame(ui, |ui| {
                    ui.vertical(|ui| {
                        // 1. HEADER & PRESET MANAGEMENT BAR
                        self.render_eq_header_bar(ui);

                        ui.add_space(8.0);

                        // Mode status notices (Bit-Perfect vs DSP vs OS Resampled)
                        if self.chain_mode == crate::models::ChainMode::BitPerfect {
                            egui::Frame::new()
                                .fill(Color32::from_rgba_premultiplied(236, 253, 245, 230))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(167, 243, 208)))
                                .corner_radius(6.0)
                                .inner_margin(egui::Margin::symmetric(12, 6))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("⚡ Mode Bit-Perfect Aktif: Rantai biquad parametric dilewati murni tanpa kalkulasi software untuk menjaga keutuhan bit audio asli. Klik tombol '🎛️ AKTIFKAN EQ' untuk berpindah ke mode DSP.").size(11.5).color(Color32::from_rgb(6, 95, 70)));
                                });
                            ui.add_space(6.0);
                        } else if self.chain_mode == crate::models::ChainMode::OsResampled {
                            egui::Frame::new()
                                .fill(Color32::from_rgba_premultiplied(254, 243, 199, 230))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(253, 230, 138)))
                                .corner_radius(6.0)
                                .inner_margin(egui::Margin::symmetric(12, 6))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("🟡 Mode OS Resampled Aktif: Sinyal dialirkan ke Shared Audio Mixer (48 kHz) dengan bypass filter EQ.").size(11.5).color(Color32::from_rgb(146, 64, 14)));
                                });
                            ui.add_space(6.0);
                        }

                        // 2. INTERACTIVE FREQUENCY RESPONSE DISPLAY (Biquad Spline & Draggable Nodes)
                        self.render_eq_interactive_display(ui);

                        ui.add_space(10.0);

                        // 3. PREAMP & HEADROOM COMPENSATOR BAR
                        self.render_eq_preamp_and_headroom_bar(ui);

                        ui.add_space(10.0);

                        // 4. SELECTED BAND PARAMETRIC INSPECTOR
                        self.render_eq_band_inspector(ui);

                        ui.add_space(8.0);

                        // 5. BANDS QUICK STRIP / MINI-CHIPS
                        self.render_eq_bands_strip(ui);

                        ui.add_space(14.0);

                        // 6. SIGNAL TRANSPARENCY & AUDIOPHILE METERING SECTION (Tahap 2 Preview)
                        self.render_eq_transparency_section(ui);
                    });
                });
            });
    }

    fn render_eq_header_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🎛️ PARAMETRIC EQUALIZER STUDIO").strong().size(15.0).color(Color32::from_rgb(15, 23, 42)));

            // Dirty indicator with star
            let dirty_star = if self.eq_is_dirty { "*" } else { "" };
            let preset_title = format!("Preset: \"{}{}\"", self.eq_current_preset_name, dirty_star);
            let pill_color = if self.eq_is_dirty {
                Color32::from_rgb(249, 115, 22) // Orange dirty badge
            } else {
                Color32::from_rgb(100, 116, 139)
            };
            ui.label(egui::RichText::new(preset_title).strong().size(12.5).color(pill_color));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Per-card EQ Bypass Switch
                let bypass_btn_text = if self.eq_bypass {
                    egui::RichText::new("⚡ BYPASS EQ (Pure Direct)").strong().color(Color32::from_rgb(217, 119, 6))
                } else {
                    egui::RichText::new("🎛️ EQ AKTIF").strong().color(Color32::from_rgb(16, 185, 129))
                };
                if ui.button(bypass_btn_text).clicked() {
                    self.eq_bypass = !self.eq_bypass;
                    if self.eq_bypass {
                        self.set_chain_mode(crate::models::ChainMode::BitPerfect);
                    } else {
                        self.set_chain_mode(crate::models::ChainMode::DspActive);
                    }
                    self.sync_parametric_to_legacy_eq();
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

        // Preset selector bar & actions
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Preset Pilihan:").strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));

            let preset_names: Vec<String> = self.eq_presets.iter().map(|p| p.name.clone()).collect();
            for name in preset_names {
                let is_selected = self.eq_current_preset_name == name;
                let btn = egui::Button::new(egui::RichText::new(&name).size(11.5).strong())
                    .fill(if is_selected { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(241, 245, 249) })
                    .stroke(Stroke::new(1.0, if is_selected { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
                if ui.add(btn).clicked() {
                    if let Some(preset) = self.eq_presets.iter().find(|p| p.name == name).cloned() {
                        self.parametric_bands = preset.bands.clone();
                        self.eq_preamp_mode = preset.preamp_mode;
                        self.eq_manual_preamp_db = preset.manual_preamp_db;
                        self.eq_limiter_enabled = preset.limiter_enabled;
                        self.eq_dither_mode = preset.dither_mode;
                        self.eq_current_preset_name = preset.name.clone();
                        self.eq_is_dirty = false;
                        self.selected_band_idx = Some(0);
                        self.sync_parametric_to_legacy_eq();
                    }
                }
            }

            ui.separator();

            // Preset Action Buttons
            if ui.button(egui::RichText::new("💾 Simpan").size(11.5).strong()).clicked() {
                // Update current preset in list or save as custom
                let cur_name = self.eq_current_preset_name.clone();
                let new_preset = ParametricEqPreset {
                    name: cur_name.clone(),
                    bands: self.parametric_bands.clone(),
                    preamp_mode: self.eq_preamp_mode,
                    manual_preamp_db: self.eq_manual_preamp_db,
                    limiter_enabled: self.eq_limiter_enabled,
                    dither_mode: self.eq_dither_mode,
                };
                if let Some(pos) = self.eq_presets.iter().position(|p| p.name == cur_name) {
                    self.eq_presets[pos] = new_preset;
                } else {
                    self.eq_presets.push(new_preset);
                }
                self.eq_is_dirty = false;
                self.save_library_state();
            }

            if ui.button(egui::RichText::new("➕ Simpan Sebagai...").size(11.5)).clicked() {
                self.show_save_preset_dialog = !self.show_save_preset_dialog;
            }

            if ui.button(egui::RichText::new("🔄 Reset Preset").size(11.5)).clicked() {
                let cur_name = self.eq_current_preset_name.clone();
                if let Some(preset) = self.eq_presets.iter().find(|p| p.name == cur_name).cloned() {
                    self.parametric_bands = preset.bands.clone();
                    self.eq_preamp_mode = preset.preamp_mode;
                    self.eq_manual_preamp_db = preset.manual_preamp_db;
                    self.eq_limiter_enabled = preset.limiter_enabled;
                    self.eq_dither_mode = preset.dither_mode;
                    self.eq_is_dirty = false;
                    self.sync_parametric_to_legacy_eq();
                } else {
                    self.parametric_bands = ParametricBand::default_10_bands();
                    self.eq_is_dirty = false;
                    self.sync_parametric_to_legacy_eq();
                }
            }
        });

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
                                let new_preset = ParametricEqPreset {
                                    name: name.clone(),
                                    bands: self.parametric_bands.clone(),
                                    preamp_mode: self.eq_preamp_mode,
                                    manual_preamp_db: self.eq_manual_preamp_db,
                                    limiter_enabled: self.eq_limiter_enabled,
                                    dither_mode: self.eq_dither_mode,
                                };
                                self.eq_presets.retain(|p| p.name != name);
                                self.eq_presets.push(new_preset);
                                self.eq_current_preset_name = name;
                                self.eq_is_dirty = false;
                                self.show_save_preset_dialog = false;
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

    fn render_eq_interactive_display(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 250.0), egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // Dark modern pro-audio canvas background
        painter.rect_filled(rect, 8.0, Color32::from_rgb(15, 23, 42)); // Slate 900
        painter.rect_stroke(rect, 8.0, Stroke::new(1.0, Color32::from_rgb(30, 41, 59)), egui::StrokeKind::Outside);

        // 1. Logarithmic Frequency Grid Lines & Labels
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
                Pos2::new(rect.min.x + 24.0, y - 2.0),
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

        // 4. Calculate Effective Preamp & Response Curve Points
        let max_boost = calculate_max_boost(&self.parametric_bands, 44100.0);
        let effective_preamp = if self.eq_preamp_mode == PreampMode::Auto {
            if max_boost > 0.0 { -max_boost } else { 0.0 }
        } else {
            self.eq_manual_preamp_db
        };

        let sample_rate = 44100.0_f64;
        let points_count = 280;
        let mut curve_points: Vec<Pos2> = Vec::with_capacity(points_count);

        for i in 0..points_count {
            let t = i as f32 / (points_count - 1) as f32;
            let f = 20.0 * (1000.0_f32).powf(t);
            let total_db = compute_total_curve_db(
                &self.parametric_bands,
                f as f64,
                sample_rate,
                effective_preamp as f64,
                self.eq_bypass,
            );
            let x = freq_to_x(f, rect);
            let y = gain_to_y(total_db as f32, rect);
            curve_points.push(Pos2::new(x, y));
        }

        // Draw Filled Area between response curve and 0 dB baseline
        let zero_y = gain_to_y(0.0, rect);
        if curve_points.len() >= 2 && !self.eq_bypass {
            for i in 0..(curve_points.len() - 1) {
                let p1 = curve_points[i];
                let p2 = curve_points[i + 1];
                let is_boost = p1.y < zero_y || p2.y < zero_y;
                let fill_col = if is_boost {
                    Color32::from_rgba_unmultiplied(14, 165, 233, 30) // Cyan translucent fill
                } else {
                    Color32::from_rgba_unmultiplied(244, 63, 94, 20)  // Subtle rose for cut
                };
                let quad = [
                    Pos2::new(p1.x, zero_y),
                    p1,
                    p2,
                    Pos2::new(p2.x, zero_y),
                ];
                painter.add(egui::epaint::Shape::convex_polygon(quad.to_vec(), fill_col, Stroke::NONE));
            }
        }

        // Draw Response Curve Stroke
        let curve_stroke = if self.eq_bypass {
            Stroke::new(1.8, Color32::from_rgba_unmultiplied(148, 163, 184, 160)) // Muted dashed
        } else {
            Stroke::new(2.4, Color32::from_rgb(56, 189, 248)) // Glowing sky cyan
        };

        for i in 0..(curve_points.len() - 1) {
            painter.line_segment([curve_points[i], curve_points[i + 1]], curve_stroke);
        }

        // 5. Interactive Draggable Control Nodes
        let mut hovered_band: Option<usize> = None;
        let pointer_pos = response.hover_pos();
        let is_dragging = response.dragged();

        let num_bands = self.parametric_bands.len();
        for i in 0..num_bands {
            let band = &self.parametric_bands[i];
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
                    if self.parametric_bands.len() < 32 {
                        let new_freq = x_to_freq(pos.x, rect);
                        let new_gain = y_to_gain(pos.y, rect);
                        let new_band = ParametricBand::new(FilterType::Peaking, new_freq, new_gain, 1.414);
                        self.parametric_bands.push(new_band);
                        self.selected_band_idx = Some(self.parametric_bands.len() - 1);
                        self.eq_is_dirty = true;
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
                    if d_idx < self.parametric_bands.len() {
                        self.selected_band_idx = Some(d_idx);
                        let new_f = x_to_freq(pos.x, rect);
                        self.parametric_bands[d_idx].freq = new_f;
                        if self.parametric_bands[d_idx].filter_type.has_gain() {
                            let new_g = y_to_gain(pos.y, rect);
                            self.parametric_bands[d_idx].gain_db = new_g;
                        }
                        self.eq_is_dirty = true;
                        self.sync_parametric_to_legacy_eq();
                    }
                }
            }
        } else {
            self.dragging_band_idx = None;
        }

        // Mouse Wheel over node adjusts Q factor
        if let Some(h_idx) = hovered_band {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.5 {
                let delta = if scroll_delta > 0.0 { 0.1 } else { -0.1 };
                let new_q = (self.parametric_bands[h_idx].q + delta).clamp(0.1, 10.0);
                self.parametric_bands[h_idx].q = new_q;
                self.eq_is_dirty = true;
                self.sync_parametric_to_legacy_eq();
            }

            // Hover tooltip
            let band = &self.parametric_bands[h_idx];
            response.show_tooltip_text(format!(
                "Band {} • {}\nFrekuensi: {:0.0} Hz\nGain: {:+0.1} dB\nQ-Factor: {:0.2}\n(Gulir mouse untuk atur Q)",
                h_idx + 1,
                band.filter_type.name(),
                band.freq,
                band.gain_db,
                band.q
            ));
        }
    }

    fn render_eq_preamp_and_headroom_bar(&mut self, ui: &mut egui::Ui) {
        let max_boost = calculate_max_boost(&self.parametric_bands, 44100.0);
        let required_attenuation = -max_boost;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Headroom & Preamp:").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));

            // Preamp Mode: Auto vs Manual
            let auto_active = self.eq_preamp_mode == PreampMode::Auto;
            let auto_btn = egui::Button::new(egui::RichText::new("Auto Preamp").size(11.5).strong())
                .fill(if auto_active { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(241, 245, 249) })
                .stroke(Stroke::new(1.0, if auto_active { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
            if ui.add(auto_btn).clicked() {
                self.eq_preamp_mode = PreampMode::Auto;
                self.eq_is_dirty = true;
                self.sync_parametric_to_legacy_eq();
            }

            let manual_active = self.eq_preamp_mode == PreampMode::Manual;
            let man_btn = egui::Button::new(egui::RichText::new("Manual").size(11.5).strong())
                .fill(if manual_active { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(241, 245, 249) })
                .stroke(Stroke::new(1.0, if manual_active { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }));
            if ui.add(man_btn).clicked() {
                self.eq_preamp_mode = PreampMode::Manual;
                self.eq_is_dirty = true;
                self.sync_parametric_to_legacy_eq();
            }

            ui.add_space(8.0);

            // Preamp Slider & Display
            if self.eq_preamp_mode == PreampMode::Auto {
                ui.label(
                    egui::RichText::new(format!("Kompensasi Otomatis: {:+0.1} dB", required_attenuation))
                        .strong()
                        .size(12.0)
                        .color(Color32::from_rgb(14, 165, 233)),
                );
            } else {
                ui.label(egui::RichText::new(format!("Gain: {:+0.1} dB", self.eq_manual_preamp_db)).strong().size(12.0));
                let s_res = ui.add(egui::Slider::new(&mut self.eq_manual_preamp_db, -24.0..=12.0).suffix(" dB").show_value(false));
                if s_res.changed() {
                    self.eq_is_dirty = true;
                    self.sync_parametric_to_legacy_eq();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Headroom Badge & Warning
                let effective_preamp = if self.eq_preamp_mode == PreampMode::Auto {
                    required_attenuation
                } else {
                    self.eq_manual_preamp_db
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
                        ui.label(egui::RichText::new(format!("✓ Headroom Aman: {:0.1} dB Margin", -peak_level)).strong().size(11.5).color(Color32::from_rgb(22, 163, 74)));
                    });
                }
            });
        });
    }

    fn render_eq_band_inspector(&mut self, ui: &mut egui::Ui) {
        let selected_idx = self.selected_band_idx.unwrap_or(0);
        if selected_idx >= self.parametric_bands.len() {
            return;
        }

        let total_bands = self.parametric_bands.len();
        let mut band = self.parametric_bands[selected_idx].clone();
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

                // Sliders & Numeric Inputs: Frequency, Gain, Q Factor
                ui.columns(3, |columns| {
                    // Column 1: Frequency
                    columns[0].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Frekuensi:").strong().size(12.0));
                            let dv = ui.add(egui::DragValue::new(&mut band.freq).range(20.0..=20000.0).speed(5.0).suffix(" Hz"));
                            if dv.changed() {
                                changed = true;
                            }
                        });
                        let s = ui.add(egui::Slider::new(&mut band.freq, 20.0..=20000.0).logarithmic(true).show_value(false));
                        if s.changed() {
                            changed = true;
                        }
                    });

                    // Column 2: Gain (Disabled for HPF / LPF / Notch)
                    columns[1].vertical(|ui| {
                        let has_gain = band.filter_type.has_gain();
                        ui.add_enabled_ui(has_gain, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Gain:").strong().size(12.0));
                                let dv = ui.add(egui::DragValue::new(&mut band.gain_db).range(-18.0..=18.0).speed(0.1).suffix(" dB"));
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
                        });
                    });

                    // Column 3: Q Factor
                    columns[2].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Q Factor:").strong().size(12.0));
                            let dv = ui.add(egui::DragValue::new(&mut band.q).range(0.1..=10.0).speed(0.05));
                            if dv.changed() {
                                changed = true;
                            }
                        });
                        let s = ui.add(egui::Slider::new(&mut band.q, 0.1..=10.0).show_value(false));
                        if s.changed() {
                            changed = true;
                        }
                    });
                });
            });

        if delete_band {
            if self.parametric_bands.len() > 1 {
                self.parametric_bands.remove(selected_idx);
                if selected_idx >= self.parametric_bands.len() && !self.parametric_bands.is_empty() {
                    self.selected_band_idx = Some(self.parametric_bands.len() - 1);
                }
                self.eq_is_dirty = true;
                self.sync_parametric_to_legacy_eq();
            }
        } else if add_band {
            if self.parametric_bands.len() < 32 {
                let new_band = ParametricBand::new(FilterType::Peaking, 1000.0, 0.0, 1.414);
                self.parametric_bands.push(new_band);
                self.selected_band_idx = Some(self.parametric_bands.len() - 1);
                self.eq_is_dirty = true;
                self.sync_parametric_to_legacy_eq();
            }
        } else if changed {
            self.parametric_bands[selected_idx] = band;
            self.eq_is_dirty = true;
            self.sync_parametric_to_legacy_eq();
        }
    }

    fn render_eq_bands_strip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Bands:").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
            let num_bands = self.parametric_bands.len();
            for i in 0..num_bands {
                let band = &self.parametric_bands[i];
                let is_selected = self.selected_band_idx == Some(i);

                let label = if band.filter_type.has_gain() {
                    format!("#{}: {} {:0.0}Hz ({:+0.1}dB)", i + 1, band.filter_type.short_name(), band.freq, band.gain_db)
                } else {
                    format!("#{}: {} {:0.0}Hz", i + 1, band.filter_type.short_name(), band.freq)
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

    fn render_eq_transparency_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_rgb(241, 245, 249)) // Neutral Slate Frame
            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔊 SIGNAL TRANSPARENCY & AUDIOPHILE METERING").strong().size(13.0).color(Color32::from_rgb(15, 23, 42)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Tahap 2: Transparansi Bit-Accurate & True-Peak Precision").size(11.0).color(Color32::from_rgb(100, 116, 139)));
                    });
                });

                ui.add_space(8.0);

                ui.columns(3, |columns| {
                    // Column 1: I/O Peak Metering & Clip Counter
                    columns[0].vertical(|ui| {
                        ui.label(egui::RichText::new("I/O Peak & Clip Counter").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
                        ui.add_space(4.0);

                        // Simulated Peak Level Bar (linked to volume & mute)
                        let vol = if self.is_muted { 0.0 } else { self.volume.clamp(0.0, 1.2) };
                        let in_peak_db = if vol > 0.001 { -18.0 + vol * 8.0 } else { -60.0 };
                        let out_peak_db = if vol > 0.001 { in_peak_db + 2.0 } else { -60.0 };

                        ui.label(egui::RichText::new(format!("In Peak: {:0.1} dBFS  •  Out Peak: {:0.1} dBFS", in_peak_db, out_peak_db)).size(11.0).color(Color32::from_rgb(71, 85, 105)));

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

                    // Column 2: Limiter Option (Default OFF for bit-accuracy)
                    columns[1].vertical(|ui| {
                        ui.label(egui::RichText::new("Soft Limiter").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
                        ui.add_space(4.0);

                        let mut lim = self.eq_limiter_enabled;
                        let text = if lim {
                            egui::RichText::new("Limiter: AKTIF (-0.9 dBFS)").strong().size(11.5).color(Color32::from_rgb(234, 88, 12))
                        } else {
                            egui::RichText::new("Limiter: MATI (Transparan Murni)").strong().size(11.5).color(Color32::from_rgb(16, 185, 129))
                        };

                        if ui.checkbox(&mut lim, text).changed() {
                            self.eq_limiter_enabled = lim;
                            self.eq_is_dirty = true;
                        }

                        ui.label(egui::RichText::new("Default MATI agar output bit-accurate tanpa pewarnaan audio.").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                    });

                    // Column 3: Dither Selector
                    columns[2].vertical(|ui| {
                        ui.label(egui::RichText::new("Dither (Saat Turun Bit-Depth)").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
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
                                    self.eq_is_dirty = true;
                                }
                            }
                        });

                        ui.label(egui::RichText::new("Hanya aktif jika output DAC 16-bit untuk file 24-bit.").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                    });
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

        // At center frequency (1000 Hz), response should be +6.0 dB (+/- 0.05 dB)
        let resp_center = compute_biquad_response_db(&band, 1000.0, sample_rate);
        assert!((resp_center - 6.0).abs() < 0.05, "Center freq gain was {}", resp_center);

        // Far away from center (100 Hz), response should be near 0 dB
        let resp_far = compute_biquad_response_db(&band, 100.0, sample_rate);
        assert!(resp_far.abs() < 0.15, "Far freq gain was {}", resp_far);
    }

    #[test]
    fn test_low_shelf_filter_response() {
        let band = ParametricBand::new(FilterType::LowShelf, 100.0, 5.0, 0.707);
        let sample_rate = 44100.0;

        // Sub-bass (25 Hz) should approach +5.0 dB shelf plateau
        let resp_bass = compute_biquad_response_db(&band, 25.0, sample_rate);
        assert!((resp_bass - 5.0).abs() < 0.15, "Shelf bass response was {}", resp_bass);

        // Treble (10 kHz) should be near 0 dB
        let resp_treble = compute_biquad_response_db(&band, 10000.0, sample_rate);
        assert!(resp_treble.abs() < 0.10, "Shelf treble response was {}", resp_treble);
    }

    #[test]
    fn test_high_shelf_filter_response() {
        let band = ParametricBand::new(FilterType::HighShelf, 10000.0, 4.0, 0.707);
        let sample_rate = 44100.0;

        // High treble (18 kHz) should approach +4.0 dB shelf plateau
        let resp_air = compute_biquad_response_db(&band, 18000.0, sample_rate);
        assert!((resp_air - 4.0).abs() < 0.20, "Shelf air response was {}", resp_air);

        // Bass (100 Hz) should be near 0 dB
        let resp_bass = compute_biquad_response_db(&band, 100.0, sample_rate);
        assert!(resp_bass.abs() < 0.10, "Shelf bass response was {}", resp_bass);
    }

    #[test]
    fn test_high_pass_filter_response() {
        let band = ParametricBand::new(FilterType::HighPass, 80.0, 0.0, 0.707);
        let sample_rate = 44100.0;

        // 20 Hz should be attenuated significantly (< -10 dB)
        let resp_sub = compute_biquad_response_db(&band, 20.0, sample_rate);
        assert!(resp_sub < -10.0, "HPF sub response was {}", resp_sub);

        // 1 kHz should pass transparently (~ 0 dB)
        let resp_mid = compute_biquad_response_db(&band, 1000.0, sample_rate);
        assert!(resp_mid.abs() < 0.05, "HPF pass response was {}", resp_mid);
    }

    #[test]
    fn test_notch_filter_response() {
        let band = ParametricBand::new(FilterType::Notch, 1000.0, 0.0, 2.0);
        let sample_rate = 44100.0;

        // Center frequency (1000 Hz) should be notched out deeply (< -25 dB)
        let resp_notch = compute_biquad_response_db(&band, 1000.0, sample_rate);
        assert!(resp_notch < -25.0, "Notch attenuation was {}", resp_notch);

        // 100 Hz should pass through (~ 0 dB)
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
    fn test_max_boost_and_headroom_calculation() {
        let bands = vec![
            ParametricBand::new(FilterType::LowShelf, 45.0, 6.0, 0.707),
            ParametricBand::new(FilterType::Peaking, 1000.0, 2.0, 1.414),
        ];

        let max_boost = calculate_max_boost(&bands, 44100.0);
        // Maximum boost should be approximately +6.0 dB
        assert!((max_boost - 6.0).abs() < 0.25, "Calculated max boost was {}", max_boost);
    }

    #[test]
    fn test_equalizer_panel_headless_render() {
        let ctx = egui::Context::default();
        let screen = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0)));

        let mut app = LunulaApp::default();
        assert_eq!(app.parametric_bands.len(), 10);
        assert!(!app.eq_is_dirty);

        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.1),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_equalizer_panel(ui);
                });
            },
        );
    }
}

