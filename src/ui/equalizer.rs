use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::app::LunulaApp;
use crate::ui::components::render_card_frame;

impl LunulaApp {
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

                // Synchronize spectrum analyzer directly with live Volume & Mute
                let vol_scale = if self.is_muted {
                    0.0
                } else {
                    self.volume.clamp(0.0, 1.2)
                };

                for i in 0..bars {
                    let t = i as f32 / bars as f32;

                    // Authentic Real-Time Hardware FFT Energy from actual playing audio
                    let fft_energy = self.spectrum_bars[i];

                    // Include 10-band EQ influence if not bypassed
                    let eq_factor = if !self.eq_bypass {
                        let eq_idx = ((t * 9.99).floor() as usize).min(9);
                        let gain_db = self.eq_bands[eq_idx];
                        10.0_f32.powf(gain_db / 28.0)
                    } else {
                        1.0
                    };

                    let has_energy = fft_energy > 0.005;

                    // Scaled strictly by live FFT energy (authentic, not sine-wave simulation!)
                    let bar_height = if vol_scale > 0.001 && has_energy {
                        let combined = fft_energy * eq_factor;
                        (combined * 88.0).clamp(3.0, 96.0)
                    } else {
                        3.0 // Rest at bottom baseline when paused, stopped, or muted
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
                        Color32::from_rgb(226, 232, 240) // Flat neutral line when muted/idle
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
        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎛️ 10-BAND PRECISION EQUALIZER").strong().size(14.5).color(Color32::from_rgb(15, 23, 42)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let cb_res = ui.checkbox(&mut self.eq_bypass, egui::RichText::new("Bypass EQ (Pure Direct)").strong());
                        if cb_res.changed() {
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_eq_bypass(self.eq_bypass);
                            }
                        }
                    });
                });

                ui.label(egui::RichText::new("32-bit floating point precision biquad peaking filters.").strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Presets:").strong().size(12.5).color(Color32::from_rgb(100, 116, 139)));
                    if ui.button(egui::RichText::new("Flat").strong()).clicked() {
                        self.eq_bands = [0.0; 10];
                        self.sync_eq_to_engine();
                    }
                    if ui.button(egui::RichText::new("Bass Boost").strong()).clicked() {
                        self.eq_bands = [6.0, 5.0, 3.5, 1.5, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0];
                        self.sync_eq_to_engine();
                    }
                    if ui.button(egui::RichText::new("Vocal Clarity").strong()).clicked() {
                        self.eq_bands = [-2.0, -1.0, 0.0, 2.0, 4.0, 4.5, 3.0, 1.0, 0.5, 0.0];
                        self.sync_eq_to_engine();
                    }
                    if ui.button(egui::RichText::new("Audiophile Treble").strong()).clicked() {
                        self.eq_bands = [0.0, 0.0, 0.0, 0.5, 1.0, 2.0, 3.5, 5.0, 6.0, 6.5];
                        self.sync_eq_to_engine();
                    }
                });

                ui.add_space(14.0);

                let freqs = ["31Hz", "62Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz"];
                ui.horizontal(|ui| {
                    for i in 0..10 {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(format!("{:+0.1}", self.eq_bands[i])).strong().size(11.0).color(Color32::from_rgb(100, 116, 139)));
                            let s_res = ui.add(
                                egui::Slider::new(&mut self.eq_bands[i], -12.0..=12.0)
                                    .vertical()
                                    .show_value(false),
                            );
                            if s_res.changed() {
                                if let Some(ref engine) = self.audio_engine {
                                    engine.set_eq_band(i, self.eq_bands[i]);
                                }
                            }
                            ui.label(egui::RichText::new(freqs[i]).strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                        });
                        ui.add_space(20.0);
                    }
                });
            });
        });
    }
}
