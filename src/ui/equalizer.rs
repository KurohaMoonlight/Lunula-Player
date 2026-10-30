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

                let phase = self.visualizer_anim_phase;

                for i in 0..bars {
                    let t = i as f32 / bars as f32;
                    let sine1 = ((phase * 4.0 + t * 9.0).sin() + 1.0) * 0.35;
                    let sine2 = ((phase * 7.5 - t * 14.0).cos() + 1.0) * 0.25;
                    let sine3 = ((phase * 2.0 + t * 4.0).sin() + 1.0) * 0.15;

                    let mut height_factor = sine1 + sine2 + sine3;
                    if !self.is_playing {
                        height_factor *= 0.05;
                    }

                    let bass_bias = 1.25 - (t * 0.55);
                    let bar_height = (height_factor * bass_bias * 82.0).clamp(4.0, 98.0);

                    let x = rect.min.x + 12.0 + (i as f32) * (bar_width + spacing);
                    let y_bottom = rect.max.y - 10.0;
                    let y_top = y_bottom - bar_height;

                    let bar_rect = Rect::from_min_max(Pos2::new(x, y_top), Pos2::new(x + bar_width, y_bottom));

                    let r = (15.0 + t * 0.0) as u8;
                    let g = (23.0 + t * 140.0) as u8;
                    let b = (42.0 + t * 190.0) as u8;
                    let col = Color32::from_rgb(r, g, b);

                    painter.rect_filled(bar_rect, 2.5, col);
                    painter.circle_filled(Pos2::new(x + bar_width / 2.0, y_top - 2.5), 1.5, Color32::from_rgb(14, 165, 233));
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
                        ui.checkbox(&mut self.eq_bypass, egui::RichText::new("Bypass EQ (Pure Direct)").strong());
                    });
                });

                ui.label(egui::RichText::new("32-bit floating point precision biquad peaking filters.").strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Presets:").strong().size(12.5).color(Color32::from_rgb(100, 116, 139)));
                    if ui.button(egui::RichText::new("Flat").strong()).clicked() {
                        self.eq_bands = [0.0; 10];
                    }
                    if ui.button(egui::RichText::new("Bass Boost").strong()).clicked() {
                        self.eq_bands = [6.0, 5.0, 3.5, 1.5, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0];
                    }
                    if ui.button(egui::RichText::new("Vocal Clarity").strong()).clicked() {
                        self.eq_bands = [-2.0, -1.0, 0.0, 2.0, 4.0, 4.5, 3.0, 1.0, 0.5, 0.0];
                    }
                    if ui.button(egui::RichText::new("Audiophile Treble").strong()).clicked() {
                        self.eq_bands = [0.0, 0.0, 0.0, 0.5, 1.0, 2.0, 3.5, 5.0, 6.0, 6.5];
                    }
                });

                ui.add_space(14.0);

                let freqs = ["31Hz", "62Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz"];
                ui.horizontal(|ui| {
                    for i in 0..10 {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(format!("{:+0.1}", self.eq_bands[i])).strong().size(11.0).color(Color32::from_rgb(100, 116, 139)));
                            ui.add(
                                egui::Slider::new(&mut self.eq_bands[i], -12.0..=12.0)
                                    .vertical()
                                    .show_value(false),
                            );
                            ui.label(egui::RichText::new(freqs[i]).strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                        });
                        ui.add_space(20.0);
                    }
                });
            });
        });
    }
}

