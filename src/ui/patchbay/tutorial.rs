use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::NodeType;
use crate::app::LunulaApp;

impl LunulaApp {
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
                                    .size(12.5)
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
                                    ui.button(egui::RichText::new(*name).size(12.0).strong().color(Color32::WHITE))
                                } else {
                                    ui.button(egui::RichText::new(*name).size(12.0).color(Color32::from_rgb(71, 85, 105)))
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
                                ui.label(egui::RichText::new("• Audio mengalir seperti kabel fisik panggung dari FILE SOURCE (kiri) menuju OUTPUT DAC (kanan).\n• Setiap modul di tengahnya menyaring atau memproses suara secara real-time.\n• Anda bisa menyisipkan efek sebanyak apa pun sesuai selera.").size(12.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            1 => {
                                ui.label(egui::RichText::new("Menyambung & Memutus Kabel").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Menyambung: Klik & tahan bulatan pin kanan (Output) sebuah modul, lalu tarik kabelnya ke pin kiri (Input) modul tujuan.\n• Memutus: Cukup klik langsung pada garis kabel yang ingin dilepas.\n• Geser Kanvas: Tahan klik kanan atau roda mouse lalu geser.").size(12.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            2 => {
                                ui.label(egui::RichText::new("Uji Coba: Low-Pass Filter").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Ingin mendengar perbedaannya langsung? Klik '➕ Tambah ke Chain' pada kartu Low-Pass Filter di sidebar kanan.\n• Sambungkan kabel dari RAM Buffer ke Low-Pass Filter, lalu dari filter ke App Resampler atau Output DAC.\n• Suara vokal & cymbal langsung teredam tebal (efek underwater)!").size(12.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            3 => {
                                ui.label(egui::RichText::new("Keamanan Audio (Auto-Mute)").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Jika kabel terputus di tengah jalan, LunulaPlayer otomatis membisukan audio (silent mute).\n• Ini sengaja dirancang agar telinga dan headphone Anda aman dari suara ledakan/letupan keras (pop/click).\n• Sambungkan kembali kabel sampai hijau untuk melanjutkan musik.").size(12.5).color(Color32::from_rgb(71, 85, 105)));
                            }
                            _ => {
                                ui.label(egui::RichText::new("Percabangan (Splitter) & Penggabung (Mixer)").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("• Aturan 1 Kabel: Modul biasa hanya mengizinkan 1 kabel masuk dan 1 kabel keluar. Menarik kabel baru otomatis menggantikan kabel lama.\n• Membagi Jalur (Splitter / Y-Branch): Gunakan modul '🔀 Splitter' untuk menduplikasi sinyal menjadi 2 cabang (Pin A & Pin B) secara lossless tanpa kehilangan kualitas.\n• Menggabungkan Jalur (Mixer / Summing): Sambungkan kedua cabang ke Pin A & Pin B milik '🎚️ Mixer' untuk mencampur suara Dry & Wet dengan slider crossfader presisi!").size(12.5).color(Color32::from_rgb(71, 85, 105)));
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
                    NodeType::HighPassFilter => Color32::from_rgb(56, 189, 248),
                    NodeType::BandPassFilter => Color32::from_rgb(245, 158, 11),
                    NodeType::Crossover => Color32::from_rgb(168, 85, 247),
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
                            NodeType::LowPassFilter => ("🌊", "LOW-PASS FILTER (LPF)", self.nodes.iter().any(|n| n.node_type == NodeType::LowPassFilter)),
                            NodeType::HighPassFilter => ("🏔️", "HIGH-PASS FILTER (HPF)", self.nodes.iter().any(|n| n.node_type == NodeType::HighPassFilter)),
                            NodeType::BandPassFilter => ("🎯", "BAND-PASS FILTER (BPF)", self.nodes.iter().any(|n| n.node_type == NodeType::BandPassFilter)),
                            NodeType::Crossover => ("⚡", "2-WAY CROSSOVER (LR4)", self.nodes.iter().any(|n| n.node_type == NodeType::Crossover)),
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
                                    .size(13.5)
                                    .color(Color32::from_rgb(15, 23, 42)),
                            );

                            ui.label(
                                egui::RichText::new(format!("Slide {}/4", step + 1))
                                    .strong()
                                    .size(12.5)
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
                                    ui.add(egui::Button::new(egui::RichText::new(*title).size(12.5).strong().color(Color32::WHITE)).fill(theme_color))
                                } else {
                                    ui.button(egui::RichText::new(*title).size(12.5).color(Color32::from_rgb(71, 85, 105)))
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
                                } else if filter_type == NodeType::HighPassFilter {
                                    let x_start = diag_rect.min.x + 14.0;
                                    let x_cut = diag_rect.min.x + diag_rect.width() * 0.45;
                                    let x_end = diag_rect.max.x - 14.0;
                                    let y_pass = diag_rect.min.y + 24.0;
                                    let y_mute = diag_rect.max.y - 20.0;

                                    // Shaded passband area (above cutoff to 20kHz)
                                    let mut shade_poly = Vec::new();
                                    shade_poly.push(Pos2::new(x_cut - 20.0, y_mute));
                                    shade_poly.push(Pos2::new(x_cut, y_pass));
                                    shade_poly.push(Pos2::new(x_end, y_pass));
                                    shade_poly.push(Pos2::new(x_end, y_mute));
                                    p.add(egui::Shape::convex_polygon(shade_poly, Color32::from_rgba_premultiplied(56, 189, 248, 35), Stroke::NONE));

                                    // Flat line in passband
                                    p.line_segment([Pos2::new(x_cut, y_pass), Pos2::new(x_end, y_pass)], Stroke::new(2.5, Color32::from_rgb(56, 189, 248)));
                                    // Rolloff slope curve (low frequencies attenuated)
                                    let cp1 = Pos2::new(x_cut - 6.0, y_pass);
                                    let cp2 = Pos2::new(x_cut - 14.0, y_mute);
                                    let p_start = Pos2::new(x_start, y_mute);
                                    let curve = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [p_start, cp2, cp1, Pos2::new(x_cut, y_pass)],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.5, Color32::from_rgb(56, 189, 248)),
                                    );
                                    p.add(curve);

                                    // Cutoff Marker Dot & Vertical Guideline
                                    p.line_segment([Pos2::new(x_cut, y_pass), Pos2::new(x_cut, y_mute)], Stroke::new(1.0, Color32::from_rgb(186, 230, 253)));
                                    p.circle_filled(Pos2::new(x_cut, y_pass), 4.0, Color32::from_rgb(2, 132, 199));
                                    p.circle_stroke(Pos2::new(x_cut, y_pass), 4.0, Stroke::new(1.5, Color32::WHITE));

                                    // Dynamic wave ripple if playing
                                    if self.is_playing {
                                        let t = ui.input(|i| i.time);
                                        for step_x in 0..8 {
                                            let rx = x_cut + step_x as f32 * ((x_end - x_cut) / 8.0);
                                            let ry = y_pass + (rx * 0.1 + (t * 5.0) as f32).sin() * 2.5;
                                            p.circle_filled(Pos2::new(rx, ry), 1.2, Color32::from_rgb(14, 165, 233));
                                        }
                                    }

                                    // Axis Frequency Labels
                                    p.text(Pos2::new(x_start, y_mute + 4.0), egui::Align2::LEFT_TOP, "20 Hz (Sub-Bass Muted 🔇)", egui::FontId::proportional(8.5), Color32::from_rgb(239, 68, 68));
                                    p.text(Pos2::new(x_cut, y_pass - 12.0), egui::Align2::CENTER_BOTTOM, "Cutoff 80 Hz", egui::FontId::proportional(9.0), Color32::from_rgb(2, 132, 199));
                                    p.text(Pos2::new(x_end, y_mute + 4.0), egui::Align2::RIGHT_TOP, "20 kHz (Treble Lolos ✅)", egui::FontId::proportional(8.5), Color32::from_rgb(16, 185, 129));
                                } else if filter_type == NodeType::BandPassFilter {
                                    let x_start = diag_rect.min.x + 14.0;
                                    let x_center = diag_rect.center().x;
                                    let x_end = diag_rect.max.x - 14.0;
                                    let y_peak = diag_rect.min.y + 24.0;
                                    let y_mute = diag_rect.max.y - 20.0;
                                    let bw = 35.0;

                                    // Shaded bell area
                                    let mut shade_poly = Vec::new();
                                    shade_poly.push(Pos2::new(x_center - bw * 1.5, y_mute));
                                    shade_poly.push(Pos2::new(x_center - bw * 0.5, y_peak + 8.0));
                                    shade_poly.push(Pos2::new(x_center, y_peak));
                                    shade_poly.push(Pos2::new(x_center + bw * 0.5, y_peak + 8.0));
                                    shade_poly.push(Pos2::new(x_center + bw * 1.5, y_mute));
                                    p.add(egui::Shape::convex_polygon(shade_poly, Color32::from_rgba_premultiplied(245, 158, 11, 40), Stroke::NONE));

                                    // Left curve (rise)
                                    let cp1 = Pos2::new(x_center - bw * 0.8, y_mute);
                                    let cp2 = Pos2::new(x_center - bw * 0.3, y_peak);
                                    let c_left = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [Pos2::new(x_start, y_mute), cp1, cp2, Pos2::new(x_center, y_peak)],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.5, Color32::from_rgb(245, 158, 11)),
                                    );
                                    p.add(c_left);

                                    // Right curve (fall)
                                    let cp3 = Pos2::new(x_center + bw * 0.3, y_peak);
                                    let cp4 = Pos2::new(x_center + bw * 0.8, y_mute);
                                    let c_right = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [Pos2::new(x_center, y_peak), cp3, cp4, Pos2::new(x_end, y_mute)],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.5, Color32::from_rgb(245, 158, 11)),
                                    );
                                    p.add(c_right);

                                    // Center Marker Dot & Guideline
                                    p.line_segment([Pos2::new(x_center, y_peak), Pos2::new(x_center, y_mute)], Stroke::new(1.0, Color32::from_rgb(253, 230, 138)));
                                    p.circle_filled(Pos2::new(x_center, y_peak), 4.5, Color32::from_rgb(217, 119, 6));
                                    p.circle_stroke(Pos2::new(x_center, y_peak), 4.5, Stroke::new(1.5, Color32::WHITE));

                                    // Bandwidth indicator
                                    let y_3db = y_peak + 10.0;
                                    p.line_segment([Pos2::new(x_center - bw * 0.6, y_3db), Pos2::new(x_center + bw * 0.6, y_3db)], Stroke::new(1.0, Color32::from_rgb(217, 119, 6)));

                                    // Dynamic ripple if playing
                                    if self.is_playing {
                                        let t = ui.input(|i| i.time);
                                        for step_x in 0..7 {
                                            let rx = x_center - 24.0 + step_x as f32 * 8.0;
                                            let ry = y_peak + 6.0 + (rx * 0.2 + (t * 6.0) as f32).sin() * 2.0;
                                            p.circle_filled(Pos2::new(rx, ry), 1.2, Color32::from_rgb(251, 191, 36));
                                        }
                                    }

                                    p.text(Pos2::new(x_start, y_mute + 4.0), egui::Align2::LEFT_TOP, "Bass Cut 🔇", egui::FontId::proportional(8.5), Color32::from_rgb(239, 68, 68));
                                    p.text(Pos2::new(x_center, y_peak - 12.0), egui::Align2::CENTER_BOTTOM, "Center 1.0 kHz (Q 1.41)", egui::FontId::proportional(9.0), Color32::from_rgb(217, 119, 6));
                                    p.text(Pos2::new(x_end, y_mute + 4.0), egui::Align2::RIGHT_TOP, "Treble Cut 🔇", egui::FontId::proportional(8.5), Color32::from_rgb(239, 68, 68));
                                } else if filter_type == NodeType::Crossover {
                                    let x_start = diag_rect.min.x + 14.0;
                                    let x_xover = diag_rect.center().x;
                                    let x_end = diag_rect.max.x - 14.0;
                                    let y_0db = diag_rect.min.y + 22.0;
                                    let y_6db = diag_rect.min.y + 36.0;
                                    let y_mute = diag_rect.max.y - 20.0;

                                    // Flat 0 dB sum line across top
                                    p.line_segment([Pos2::new(x_start, y_0db), Pos2::new(x_end, y_0db)], Stroke::new(1.5, Color32::from_rgb(168, 85, 247)));

                                    // Low-pass branch (Emerald)
                                    let cp_lp1 = Pos2::new(x_xover - 25.0, y_0db);
                                    let curve_lp = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [Pos2::new(x_start, y_0db), cp_lp1, Pos2::new(x_xover, y_6db), Pos2::new(x_xover + 45.0, y_mute)],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.4, Color32::from_rgb(16, 185, 129)),
                                    );
                                    p.add(curve_lp);

                                    // High-pass branch (Sky Blue)
                                    let cp_hp2 = Pos2::new(x_xover + 25.0, y_0db);
                                    let curve_hp = egui::epaint::CubicBezierShape::from_points_stroke(
                                        [Pos2::new(x_xover - 45.0, y_mute), Pos2::new(x_xover, y_6db), cp_hp2, Pos2::new(x_end, y_0db)],
                                        false,
                                        Color32::TRANSPARENT,
                                        Stroke::new(2.4, Color32::from_rgb(56, 189, 248)),
                                    );
                                    p.add(curve_hp);

                                    // -6dB intersection dot & guideline
                                    p.line_segment([Pos2::new(x_xover, y_0db), Pos2::new(x_xover, y_mute)], Stroke::new(1.0, Color32::from_rgb(233, 213, 255)));
                                    p.circle_filled(Pos2::new(x_xover, y_6db), 4.5, Color32::from_rgb(147, 51, 234));
                                    p.circle_stroke(Pos2::new(x_xover, y_6db), 4.5, Stroke::new(1.5, Color32::WHITE));

                                    // Dynamic wave ripple if playing
                                    if self.is_playing {
                                        let t = ui.input(|i| i.time);
                                        for s in 0..6 {
                                            let rx_lp = x_start + s as f32 * ((x_xover - x_start) / 6.0);
                                            let ry_lp = y_0db + (rx_lp * 0.1 + (t * 5.0) as f32).sin() * 2.0;
                                            p.circle_filled(Pos2::new(rx_lp, ry_lp), 1.2, Color32::from_rgb(52, 211, 153));

                                            let rx_hp = x_xover + s as f32 * ((x_end - x_xover) / 6.0);
                                            let ry_hp = y_0db + (rx_hp * 0.1 + (t * 5.0) as f32).sin() * 2.0;
                                            p.circle_filled(Pos2::new(rx_hp, ry_hp), 1.2, Color32::from_rgb(14, 165, 233));
                                        }
                                    }

                                    p.text(Pos2::new(x_start, y_mute + 4.0), egui::Align2::LEFT_TOP, "Pin 0: LOW (< fc) 🌊", egui::FontId::proportional(8.5), Color32::from_rgb(16, 185, 129));
                                    p.text(Pos2::new(x_xover, y_0db - 12.0), egui::Align2::CENTER_BOTTOM, "X-Over 2.5 kHz (-6dB LR4)", egui::FontId::proportional(9.0), Color32::from_rgb(147, 51, 234));
                                    p.text(Pos2::new(x_end, y_mute + 4.0), egui::Align2::RIGHT_TOP, "Pin 1: HIGH (> fc) 🏔️", egui::FontId::proportional(8.5), Color32::from_rgb(2, 132, 199));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "⚡ Linkwitz-Riley 4th-Order (LR4): Penjumlahan datar 0 dB sempurna tanpa distorsi fase",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(147, 51, 234),
                                    );
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
                                } else if filter_type == NodeType::TubeWarmer {
                                    // Slide 1 Diagram: Asymmetric Triode Saturation Curve & Harmonics Bar
                                    let x_start = diag_rect.min.x + 24.0;
                                    let x_mid = diag_rect.min.x + (diag_rect.width() * 0.44);
                                    let x_curve_end = diag_rect.min.x + (diag_rect.width() * 0.65);
                                    let y_mid = diag_rect.center().y - 6.0;
                                    let curve_h = 36.0;

                                    // Linear baseline (dashed)
                                    p.line_segment([Pos2::new(x_start, y_mid + curve_h), Pos2::new(x_curve_end, y_mid - curve_h)], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
                                    p.text(Pos2::new(x_start, y_mid + curve_h + 3.0), egui::Align2::LEFT_TOP, "-1.0 Linear", egui::FontId::proportional(7.5), Color32::from_rgb(148, 163, 184));
                                    p.text(Pos2::new(x_curve_end, y_mid - curve_h - 3.0), egui::Align2::RIGHT_BOTTOM, "+1.0 Linear", egui::FontId::proportional(7.5), Color32::from_rgb(148, 163, 184));

                                    // Asymmetric triode saturation curve
                                    let mut sat_pts = Vec::new();
                                    let steps = 24;
                                    for s in 0..=steps {
                                        let t = (s as f32 / steps as f32) * 2.0 - 1.0;
                                        let x = x_mid + t * (x_curve_end - x_mid);
                                        let y_val = t + 0.22 * t * t - 0.08 * t * t * t;
                                        let y = y_mid - (y_val * (curve_h * 0.85));
                                        sat_pts.push(Pos2::new(x, y));
                                    }
                                    for seg in 0..sat_pts.len() - 1 {
                                        p.line_segment([sat_pts[seg], sat_pts[seg + 1]], Stroke::new(2.4, Color32::from_rgb(249, 115, 22)));
                                    }

                                    p.circle_filled(Pos2::new(x_mid, y_mid), 3.0, Color32::from_rgb(234, 88, 12));

                                    // Right side: Harmonics Bar Graph Display
                                    let bar_x = diag_rect.max.x - 110.0;
                                    let bar_bot = diag_rect.max.y - 26.0;
                                    p.text(Pos2::new(bar_x + 40.0, diag_rect.min.y + 16.0), egui::Align2::CENTER_TOP, "Harmonik Spektrum", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));

                                    p.rect_filled(Rect::from_min_size(Pos2::new(bar_x, bar_bot - 45.0), Vec2::new(16.0, 45.0)), 2.0, Color32::from_rgb(251, 146, 60));
                                    p.text(Pos2::new(bar_x + 8.0, bar_bot + 3.0), egui::Align2::CENTER_TOP, "1st", egui::FontId::proportional(7.5), Color32::from_rgb(100, 116, 139));

                                    let pulse = if self.is_playing {
                                        0.85 + 0.15 * (ui.input(|i| i.time) * 4.0).sin().abs() as f32
                                    } else {
                                        1.0
                                    };
                                    let h2_h = 28.0 * pulse;
                                    p.rect_filled(Rect::from_min_size(Pos2::new(bar_x + 28.0, bar_bot - h2_h), Vec2::new(16.0, h2_h)), 2.0, Color32::from_rgb(249, 115, 22));
                                    p.text(Pos2::new(bar_x + 36.0, bar_bot + 3.0), egui::Align2::CENTER_TOP, "2nd ✨", egui::FontId::proportional(7.5), Color32::from_rgb(234, 88, 12));

                                    let h3_h = 12.0 * pulse;
                                    p.rect_filled(Rect::from_min_size(Pos2::new(bar_x + 56.0, bar_bot - h3_h), Vec2::new(16.0, h3_h)), 2.0, Color32::from_rgb(217, 119, 6));
                                    p.text(Pos2::new(bar_x + 64.0, bar_bot + 3.0), egui::Align2::CENTER_TOP, "3rd", egui::FontId::proportional(7.5), Color32::from_rgb(100, 116, 139));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🔥 Triode 12AX7 Saturation: Menyuntikkan harmonik genap untuk suara tebal & analog",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(194, 65, 12),
                                    );
                                } else if filter_type == NodeType::StereoWidener {
                                    // Slide 1 Diagram: Haas 3D Spatial Soundstage Cone
                                    let cx = diag_rect.center().x;
                                    let cy = diag_rect.min.y + 40.0;
                                    let listener_pos = Pos2::new(cx, diag_rect.max.y - 30.0);

                                    let spk_std_l = Pos2::new(cx - 48.0, cy);
                                    let spk_std_r = Pos2::new(cx + 48.0, cy);
                                    p.line_segment([listener_pos, spk_std_l], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
                                    p.line_segment([listener_pos, spk_std_r], Stroke::new(1.0, Color32::from_rgb(203, 213, 225)));
                                    p.text(Pos2::new(cx - 50.0, cy - 14.0), egui::Align2::CENTER_CENTER, "Speaker L", egui::FontId::proportional(7.5), Color32::from_rgb(148, 163, 184));
                                    p.text(Pos2::new(cx + 50.0, cy - 14.0), egui::Align2::CENTER_CENTER, "Speaker R", egui::FontId::proportional(7.5), Color32::from_rgb(148, 163, 184));

                                    let spk_wide_l = Pos2::new(cx - 100.0, cy - 8.0);
                                    let spk_wide_r = Pos2::new(cx + 100.0, cy - 8.0);
                                    p.line_segment([listener_pos, spk_wide_l], Stroke::new(2.0, Color32::from_rgb(129, 140, 248)));
                                    p.line_segment([listener_pos, spk_wide_r], Stroke::new(2.0, Color32::from_rgb(129, 140, 248)));

                                    let pulse = if self.is_playing {
                                        0.85 + 0.15 * (ui.input(|i| i.time) * 4.0).sin().abs() as f32
                                    } else {
                                        1.0
                                    };
                                    for r_idx in 1..=3 {
                                        let rad = r_idx as f32 * 18.0 * pulse;
                                        p.circle_stroke(
                                            listener_pos,
                                            rad,
                                            Stroke::new(1.0, Color32::from_rgba_premultiplied(99, 102, 241, (50 * (4 - r_idx)) as u8)),
                                        );
                                    }

                                    p.circle_filled(listener_pos, 5.0, Color32::from_rgb(99, 102, 241));
                                    p.circle_stroke(listener_pos, 5.0, Stroke::new(1.5, Color32::WHITE));
                                    p.text(Pos2::new(listener_pos.x, listener_pos.y + 12.0), egui::Align2::CENTER_TOP, "Pendengar (Mid Vokal Fokus)", egui::FontId::proportional(8.0), Color32::from_rgb(79, 70, 229));

                                    p.text(Pos2::new(spk_wide_l.x, spk_wide_l.y - 2.0), egui::Align2::LEFT_BOTTOM, "◀ Side +35% 3D", egui::FontId::proportional(8.0), Color32::from_rgb(99, 102, 241));
                                    p.text(Pos2::new(spk_wide_r.x, spk_wide_r.y - 2.0), egui::Align2::RIGHT_BOTTOM, "Side +35% 3D ▶", egui::FontId::proportional(8.0), Color32::from_rgb(99, 102, 241));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 10.0),
                                        egui::Align2::CENTER_CENTER,
                                        "🌐 Mid-Side Haas 3D Expansion: Memperlebar panggung suara tanpa mengorbankan vokal tengah",
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
                                    p.text(r1.center(), egui::Align2::CENTER_CENTER, "RAM Buffer (A)", egui::FontId::proportional(8.0), Color32::from_rgb(67, 56, 202));

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
                                } else if filter_type == NodeType::Crossover {
                                    let cy = diag_rect.center().y - 8.0;

                                    let r_src = Rect::from_min_size(Pos2::new(diag_rect.min.x + 6.0, cy - 12.0), Vec2::new(48.0, 24.0));
                                    let r_xover = Rect::from_min_size(Pos2::new(diag_rect.min.x + 60.0, cy - 16.0), Vec2::new(56.0, 32.0));
                                    let r_fx_a = Rect::from_min_size(Pos2::new(diag_rect.center().x - 6.0, cy - 25.0), Vec2::new(54.0, 22.0));
                                    let r_fx_b = Rect::from_min_size(Pos2::new(diag_rect.center().x - 6.0, cy + 7.0), Vec2::new(54.0, 22.0));
                                    let r_mix = Rect::from_min_size(Pos2::new(diag_rect.max.x - 98.0, cy - 16.0), Vec2::new(46.0, 32.0));
                                    let r_dac = Rect::from_min_size(Pos2::new(diag_rect.max.x - 44.0, cy - 12.0), Vec2::new(40.0, 24.0));

                                    // Box 1: RAM Buffer
                                    p.rect_filled(r_src, 4.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(r_src, 4.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                                    p.text(r_src.center(), egui::Align2::CENTER_CENTER, "RAM Buffer", egui::FontId::proportional(7.5), Color32::from_rgb(67, 56, 202));

                                    // Box 2: Crossover
                                    p.rect_filled(r_xover, 5.0, Color32::from_rgb(250, 245, 255));
                                    p.rect_stroke(r_xover, 5.0, Stroke::new(1.5, theme_color), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r_xover.center().x, r_xover.center().y - 4.0), egui::Align2::CENTER_CENTER, "⚡ CROSSOVER", egui::FontId::proportional(7.0), theme_color);
                                    p.text(Pos2::new(r_xover.center().x, r_xover.center().y + 7.0), egui::Align2::CENTER_CENTER, "Pin 0 • Pin 1", egui::FontId::proportional(6.5), Color32::from_rgb(147, 51, 234));

                                    // Box 3: Efek Bass (LOW)
                                    p.rect_filled(r_fx_a, 4.0, Color32::from_rgb(236, 253, 245));
                                    p.rect_stroke(r_fx_a, 4.0, Stroke::new(1.0, Color32::from_rgb(167, 243, 208)), egui::StrokeKind::Outside);
                                    p.text(r_fx_a.center(), egui::Align2::CENTER_CENTER, "Efek Bass", egui::FontId::proportional(7.5), Color32::from_rgb(6, 95, 70));

                                    // Box 4: Efek Treble (HIGH)
                                    p.rect_filled(r_fx_b, 4.0, Color32::from_rgb(240, 249, 255));
                                    p.rect_stroke(r_fx_b, 4.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);
                                    p.text(r_fx_b.center(), egui::Align2::CENTER_CENTER, "Efek Treble", egui::FontId::proportional(7.5), Color32::from_rgb(2, 132, 199));

                                    // Box 5: Mixer (Menggabungkan kembali Pin A & Pin B ke satu jalur)
                                    p.rect_filled(r_mix, 4.0, Color32::from_rgb(243, 232, 255));
                                    p.rect_stroke(r_mix, 4.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r_mix.center().x, r_mix.center().y - 4.0), egui::Align2::CENTER_CENTER, "MIXER", egui::FontId::proportional(7.5), Color32::from_rgb(126, 34, 206));
                                    p.text(Pos2::new(r_mix.center().x, r_mix.center().y + 6.0), egui::Align2::CENTER_CENTER, "Pin A + B", egui::FontId::proportional(6.5), Color32::from_rgb(147, 51, 234));

                                    // Box 6: Output DAC (Satu-satunya muara terminal audio hardware)
                                    p.rect_filled(r_dac, 4.0, Color32::from_rgb(241, 245, 249));
                                    p.rect_stroke(r_dac, 4.0, Stroke::new(1.0, Color32::from_rgb(203, 213, 225)), egui::StrokeKind::Outside);
                                    p.text(r_dac.center(), egui::Align2::CENTER_CENTER, "Output DAC", egui::FontId::proportional(7.0), Color32::from_rgb(71, 85, 105));

                                    // Cables
                                    // RAM Buffer -> Crossover
                                    p.line_segment([Pos2::new(r_src.max.x, r_src.center().y), Pos2::new(r_xover.min.x, r_xover.center().y)], Stroke::new(2.2, Color32::from_rgb(99, 102, 241)));
                                    // Pin 0 (LOW) Green Cable -> Efek Bass
                                    p.line_segment([Pos2::new(r_xover.max.x, r_xover.center().y - 6.0), Pos2::new(r_fx_a.min.x, r_fx_a.center().y)], Stroke::new(2.0, Color32::from_rgb(16, 185, 129)));
                                    // Pin 1 (HIGH) Sky Blue Cable -> Efek Treble
                                    p.line_segment([Pos2::new(r_xover.max.x, r_xover.center().y + 6.0), Pos2::new(r_fx_b.min.x, r_fx_b.center().y)], Stroke::new(2.0, Color32::from_rgb(56, 189, 248)));
                                    // Efek Bass -> Mixer Pin A
                                    p.line_segment([Pos2::new(r_fx_a.max.x, r_fx_a.center().y), Pos2::new(r_mix.min.x, r_mix.center().y - 5.0)], Stroke::new(2.0, Color32::from_rgb(16, 185, 129)));
                                    // Efek Treble -> Mixer Pin B
                                    p.line_segment([Pos2::new(r_fx_b.max.x, r_fx_b.center().y), Pos2::new(r_mix.min.x, r_mix.center().y + 5.0)], Stroke::new(2.0, Color32::from_rgb(56, 189, 248)));
                                    // Mixer -> Output DAC (Kabel tunggal terpadu)
                                    p.line_segment([Pos2::new(r_mix.max.x, r_mix.center().y), Pos2::new(r_dac.min.x, r_dac.center().y)], Stroke::new(2.5, Color32::from_rgb(16, 185, 129)));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "⚡ Alur Dual-Band Studio: Crossover membelah frekuensi ➔ diproses mandiri ➔ disatukan di Mixer ➔ Output DAC",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(147, 51, 234),
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
                                        "✨ Pasang di antara RAM Buffer dan App Resampler / Output DAC",
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
                                } else if filter_type == NodeType::Crossover {
                                    let cy = diag_rect.center().y - 12.0;
                                    let half_w = (diag_rect.width() - 36.0) * 0.5;
                                    let b1 = Rect::from_min_size(Pos2::new(diag_rect.min.x + 12.0, cy - 22.0), Vec2::new(half_w, 44.0));
                                    let b2 = Rect::from_min_size(Pos2::new(diag_rect.center().x + 6.0, cy - 22.0), Vec2::new(half_w, 44.0));

                                    // Bad Box 1: Output dicolok ke Input sendiri (Feedback Loop)
                                    p.rect_filled(b1, 6.0, Color32::from_rgb(254, 242, 242));
                                    p.rect_stroke(b1, 6.0, Stroke::new(1.2, Color32::from_rgb(252, 165, 165)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(b1.center().x, b1.min.y + 13.0), egui::Align2::CENTER_CENTER, "🚫 Feedback Loop", egui::FontId::proportional(9.5), Color32::from_rgb(185, 28, 28));
                                    p.text(Pos2::new(b1.center().x, b1.min.y + 27.0), egui::Align2::CENTER_CENTER, "Pin LOW/HIGH dicolok balik ke Input!", egui::FontId::proportional(7.5), Color32::from_rgb(153, 27, 27));

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
                                } else if filter_type == NodeType::TubeWarmer {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;
                                    let r_box = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 18.0), Vec2::new(line_w, 48.0));
                                    p.rect_filled(r_box, 4.0, Color32::from_rgb(255, 247, 237));
                                    p.rect_stroke(r_box, 4.0, Stroke::new(1.0, Color32::from_rgb(254, 215, 170)), egui::StrokeKind::Outside);

                                    p.text(Pos2::new(r_box.min.x + 10.0, r_box.min.y + 12.0), egui::Align2::LEFT_CENTER, "0% Bypass", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));
                                    p.text(Pos2::new(r_box.min.x + line_w * 0.35, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "35% Triode ✨", egui::FontId::proportional(8.5), Color32::from_rgb(234, 88, 12));
                                    p.text(Pos2::new(r_box.min.x + line_w * 0.65, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "60% Rich", egui::FontId::proportional(8.5), Color32::from_rgb(194, 65, 12));
                                    p.text(Pos2::new(r_box.max.x - 10.0, r_box.min.y + 12.0), egui::Align2::RIGHT_CENTER, "100% Hot", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));

                                    let s_y = r_box.min.y + 32.0;
                                    let s_w = line_w - 40.0;
                                    let s_x = r_box.min.x + 20.0;
                                    p.line_segment([Pos2::new(s_x, s_y), Pos2::new(s_x + s_w, s_y)], Stroke::new(2.5, Color32::from_rgb(254, 215, 170)));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.35, s_y), 5.5, Color32::from_rgb(249, 115, 22));
                                    p.circle_stroke(Pos2::new(s_x + s_w * 0.35, s_y), 5.5, Stroke::new(1.8, Color32::WHITE));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Tarik slider Warmth di kanvas atau sidebar untuk mengubah saturasi tabung Nixie seketika",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(180, 83, 9),
                                    );
                                } else if filter_type == NodeType::StereoWidener {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;
                                    let r_box = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 18.0), Vec2::new(line_w, 48.0));
                                    p.rect_filled(r_box, 4.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(r_box, 4.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);

                                    p.text(Pos2::new(r_box.min.x + 10.0, r_box.min.y + 12.0), egui::Align2::LEFT_CENTER, "+0% Normal", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));
                                    p.text(Pos2::new(r_box.min.x + line_w * 0.35, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "+35% Haas ✨", egui::FontId::proportional(8.5), Color32::from_rgb(79, 70, 229));
                                    p.text(Pos2::new(r_box.min.x + line_w * 0.65, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "+60% Wide", egui::FontId::proportional(8.5), Color32::from_rgb(67, 56, 202));
                                    p.text(Pos2::new(r_box.max.x - 10.0, r_box.min.y + 12.0), egui::Align2::RIGHT_CENTER, "+100% 3D", egui::FontId::proportional(8.5), Color32::from_rgb(49, 46, 129));

                                    let s_y = r_box.min.y + 32.0;
                                    let s_w = line_w - 40.0;
                                    let s_x = r_box.min.x + 20.0;
                                    p.line_segment([Pos2::new(s_x, s_y), Pos2::new(s_x + s_w, s_y)], Stroke::new(2.5, Color32::from_rgb(199, 210, 254)));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.35, s_y), 5.5, Color32::from_rgb(99, 102, 241));
                                    p.circle_stroke(Pos2::new(s_x + s_w * 0.35, s_y), 5.5, Stroke::new(1.8, Color32::WHITE));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Parameter slewing anti-zipper menjaga transisi pelebaran panggung suara tetap hening tanpa letupan",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(79, 70, 229),
                                    );
                                } else if filter_type == NodeType::HighPassFilter {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;
                                    let r_box = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 18.0), Vec2::new(line_w, 48.0));
                                    p.rect_filled(r_box, 4.0, Color32::from_rgb(240, 249, 255));
                                    p.rect_stroke(r_box, 4.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);

                                    p.text(Pos2::new(r_box.min.x + 10.0, r_box.min.y + 12.0), egui::Align2::LEFT_CENTER, "Slope: 12dB • 24dB • 48dB", egui::FontId::proportional(8.5), Color32::from_rgb(2, 132, 199));
                                    p.text(Pos2::new(r_box.center().x, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "Cutoff: 20 Hz - 2.0 kHz", egui::FontId::proportional(8.5), Color32::from_rgb(14, 165, 233));
                                    p.text(Pos2::new(r_box.max.x - 10.0, r_box.min.y + 12.0), egui::Align2::RIGHT_CENTER, "Fringe Q: 0.5 - 2.5", egui::FontId::proportional(8.5), Color32::from_rgb(2, 132, 199));

                                    let s_y = r_box.min.y + 32.0;
                                    let s_w = line_w - 40.0;
                                    let s_x = r_box.min.x + 20.0;
                                    p.line_segment([Pos2::new(s_x, s_y), Pos2::new(s_x + s_w, s_y)], Stroke::new(2.5, Color32::from_rgb(186, 230, 253)));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.35, s_y), 5.5, Color32::from_rgb(56, 189, 248));
                                    p.circle_stroke(Pos2::new(s_x + s_w * 0.35, s_y), 5.5, Stroke::new(1.8, Color32::WHITE));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Atur Cutoff, Slope (dB/oct), dan Fringe Q di kanvas atau Inspector Desk secara live",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(2, 132, 199),
                                    );
                                } else if filter_type == NodeType::BandPassFilter {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;
                                    let r_box = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 18.0), Vec2::new(line_w, 48.0));
                                    p.rect_filled(r_box, 4.0, Color32::from_rgb(255, 251, 235));
                                    p.rect_stroke(r_box, 4.0, Stroke::new(1.0, Color32::from_rgb(253, 230, 138)), egui::StrokeKind::Outside);

                                    p.text(Pos2::new(r_box.min.x + 10.0, r_box.min.y + 12.0), egui::Align2::LEFT_CENTER, "Center: 50 Hz - 16 kHz", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));
                                    p.text(Pos2::new(r_box.center().x, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "Slope: 12dB • 24dB", egui::FontId::proportional(8.5), Color32::from_rgb(217, 119, 6));
                                    p.text(Pos2::new(r_box.max.x - 10.0, r_box.min.y + 12.0), egui::Align2::RIGHT_CENTER, "Bandwidth Q: 0.3 - 5.0", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));

                                    let s_y = r_box.min.y + 32.0;
                                    let s_w = line_w - 40.0;
                                    let s_x = r_box.min.x + 20.0;
                                    p.line_segment([Pos2::new(s_x, s_y), Pos2::new(s_x + s_w, s_y)], Stroke::new(2.5, Color32::from_rgb(253, 230, 138)));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.5, s_y), 5.5, Color32::from_rgb(245, 158, 11));
                                    p.circle_stroke(Pos2::new(s_x + s_w * 0.5, s_y), 5.5, Stroke::new(1.8, Color32::WHITE));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Bandwidth Q mengatur ketajaman isolasi frekuensi (Q tinggi = fokus instrumen sempit)",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(180, 83, 9),
                                    );
                                } else if filter_type == NodeType::Crossover {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;
                                    let r_box = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 18.0), Vec2::new(line_w, 48.0));
                                    p.rect_filled(r_box, 4.0, Color32::from_rgb(250, 245, 255));
                                    p.rect_stroke(r_box, 4.0, Stroke::new(1.0, Color32::from_rgb(233, 213, 255)), egui::StrokeKind::Outside);

                                    p.text(Pos2::new(r_box.min.x + 10.0, r_box.min.y + 12.0), egui::Align2::LEFT_CENTER, "Pin 0: LOW (Nada Rendah)", egui::FontId::proportional(8.5), Color32::from_rgb(16, 185, 129));
                                    p.text(Pos2::new(r_box.center().x, r_box.min.y + 12.0), egui::Align2::CENTER_CENTER, "X-Over: 80 Hz - 8.0 kHz", egui::FontId::proportional(8.5), Color32::from_rgb(147, 51, 234));
                                    p.text(Pos2::new(r_box.max.x - 10.0, r_box.min.y + 12.0), egui::Align2::RIGHT_CENTER, "Pin 1: HIGH (Nada Tinggi)", egui::FontId::proportional(8.5), Color32::from_rgb(56, 189, 248));

                                    let s_y = r_box.min.y + 32.0;
                                    let s_w = line_w - 40.0;
                                    let s_x = r_box.min.x + 20.0;
                                    p.line_segment([Pos2::new(s_x, s_y), Pos2::new(s_x + s_w, s_y)], Stroke::new(2.5, Color32::from_rgb(233, 213, 255)));
                                    p.circle_filled(Pos2::new(s_x + s_w * 0.5, s_y), 5.5, Color32::from_rgb(168, 85, 247));
                                    p.circle_stroke(Pos2::new(s_x + s_w * 0.5, s_y), 5.5, Stroke::new(1.8, Color32::WHITE));

                                    p.text(
                                        Pos2::new(diag_rect.center().x, diag_rect.max.y - 12.0),
                                        egui::Align2::CENTER_CENTER,
                                        "💡 Filter LR4 24dB/oct menjamin magnitude datar sempurna 0 dB saat kedua cabang disatukan",
                                        egui::FontId::proportional(8.5),
                                        Color32::from_rgb(147, 51, 234),
                                    );
                                } else {
                                    let cy = diag_rect.center().y - 14.0;
                                    let line_w = diag_rect.width() - 32.0;

                                    // Lane 1: DSP Active Mode
                                    let r_dsp = Rect::from_min_size(Pos2::new(diag_rect.min.x + 16.0, cy - 20.0), Vec2::new(line_w, 24.0));
                                    p.rect_filled(r_dsp, 4.0, Color32::from_rgb(238, 242, 255));
                                    p.rect_stroke(r_dsp, 4.0, Stroke::new(1.0, Color32::from_rgb(199, 210, 254)), egui::StrokeKind::Outside);
                                    p.text(Pos2::new(r_dsp.min.x + 8.0, r_dsp.center().y), egui::Align2::LEFT_CENTER, "🟣 Mode Modular FX (DSP Active)", egui::FontId::proportional(9.0), Color32::from_rgb(67, 56, 202));
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
                                        "💡 Tekan tombol toggle mode [Modular FX] / [DSP Active] di bilah atas untuk menyalakan modul filter",
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
                                    if filter_type == NodeType::HighPassFilter {
                                        ui.label(egui::RichText::new("Apa itu High-Pass Filter (HPF)?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("High-Pass Filter adalah filter frekuensi biquad berpresisi 64-bit float (f64) dengan pilihan kemiringan curam (12dB, 24dB, hingga 48dB/oktaf). Modul ini meloloskan frekuensi treble & midrange serta memotong getaran sub-bass di bawah titik Cutoff.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Efek Akustik & Karakter Suara:").strong().size(12.5).color(Color32::from_rgb(2, 132, 199)));
                                        ui.label(egui::RichText::new("• Menghilangkan Gemuruh Sub-Bass (Rumble Filter): Menyaring getaran mekanis panggung, hembusan angin rekaman live, atau desis frekuensi sangat rendah (<40 Hz) yang menguras daya amplifier.\n• Vokal & Gitar Transparan: Membuang resonansi 'dengung/muddy' di frekuensi rendah sehingga artikulasi vokal dan petikan akustik terdengar jernih dan terbuka.\n• Proteksi Driver Speaker/IEM: Mencegah excursion (gerakan konus berlebih) pada earphone IEM atau speaker desktop berukuran kecil.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Cutoff, Slope, & Resonansi Q:").strong().size(12.5).color(Color32::from_rgb(56, 189, 248)));
                                        ui.label(egui::RichText::new("• 30 Hz - 50 Hz (Subsonic Filter): Menjaga dentuman bass punchy sambil membuang infrasonik yang tak terdengar.\n• 80 Hz (Vocal Sweet Spot): Standar studio rekaman untuk kejernihan vokal pria & wanita.\n• 160 Hz - 500 Hz (Radio/Thin Style): Memotong seluruh body nada rendah untuk efek vokal tipis atau fokus simbal.\n• Slope 12dB vs 24dB vs 48dB: 12dB untuk transisi natural lembut; 24dB/48dB untuk pemotongan bass tajam ala crossover profesional.\n• Fringe Q: 0.71 (Butterworth datar maksimal). Naikkan ke 1.2 - 2.0 untuk memberi tonjolan resonansi punchy di titik cutoff.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::BandPassFilter {
                                        ui.label(egui::RichText::new("Apa itu Band-Pass Filter (BPF)?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Band-Pass Filter adalah filter biquad 64-bit float yang mengisolasi rentang pita frekuensi tertentu di sekitar Center Frequency (f0) dan meredam seluruh nada bass maupun nada treble di luar pita tersebut.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Efek Akustik & Karakter Suara:").strong().size(12.5).color(Color32::from_rgb(217, 119, 6)));
                                        ui.label(egui::RichText::new("• Efek Telepon & Radio Vintage: Mengisolasi rentang 500 Hz s/d 2.5 kHz untuk menciptakan suara percakapan telepon klasik atau transmisi walkie-talkie retro.\n• Isolasi Melodi Instrumen: Menyorot instrumen solo (misal gitar melodi, snare drum, vokal latar) di tengah keramaian aransemen musik.\n• Efek Wah-Wah & Formant: Dengan menaikkan faktor Q, filter ini menghasilkan resonansi formant tajam bergaya instrumen analog.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Center & Bandwidth Q:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("• Center 250 Hz (Warm Mid): Fokus pada kehangatan bass petik dan senar cello.\n• Center 1.0 kHz (Vocal Band): Titik tengah artikulasi vokal dan kejelasan lirik.\n• Center 4.0 kHz (Presence): Fokus pada gigitan instrumen petik dan vokal tajam.\n• Bandwidth Q (0.3 - 5.0): Q rendah (0.5 - 0.7) memberikan pita selebar 2 oktaf; Q tinggi (2.0 - 5.0) menyempitkan filter menjadi pita laser super fokus.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::Crossover {
                                        ui.label(egui::RichText::new("Apa itu 2-Way Active Crossover (Linkwitz-Riley LR4)?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Crossover Aktif Linkwitz-Riley 4th-Order (LR4) adalah modul pembagi spektrum audio studio berpresisi 64-bit float. Modul ini menerima 1 input audio dan secara simultan membaginya ke 2 pin output mandiri: Pin 0 (LOW untuk frekuensi di bawah titik potong) dan Pin 1 (HIGH untuk frekuensi di atas titik potong).").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Keunggulan Akustik Linkwitz-Riley LR4:").strong().size(12.5).color(Color32::from_rgb(147, 51, 234)));
                                        ui.label(egui::RichText::new("• Penjumlahan Datar Sempurna 0.00 dB (Flat Amplitude): Pada crossover Butterworth biasa, titik potong menimbulkan lonjakan volume +3 dB. Linkwitz-Riley didesain khusus dengan titik potong tepat di -6 dB pada kedua cabang, sehingga ketika kedua sinyal digabungkan kembali di Mixer, respon suaranya 0.00 dB datar murni tanpa distorsi!\n• Zero Phase Distortion (Fase Selaras 360°): Perbedaan fase antara Pin LOW dan HIGH adalah 360° (netral 0°), mencegah pembatalan fase (comb-filtering) saat disatukan kembali ke Output DAC.\n• Kemiringan Curam 24 dB/Oktaf: Memisahkan rentang bass dan treble dengan pemotongan tajam tanpa kebocoran frekuensi.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Titik Frekuensi Crossover:").strong().size(12.5).color(Color32::from_rgb(168, 85, 247)));
                                        ui.label(egui::RichText::new("• 80 Hz - 120 Hz: Memisahkan getaran sub-bass murni dari instrumen vokal & melodi utama.\n• 300 Hz - 500 Hz: Memisahkan dentuman body bass rendah dari kejelasan vokal midrange.\n• 2.5 kHz - 3.5 kHz: Titik pemisah klasik antara vokal/gitar dengan frekuensi kemilau treble (cymbal, hi-hat, udara analog).").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::LowPassFilter {
                                        ui.label(egui::RichText::new("Apa itu Low-Pass Filter (LPF)?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Low-Pass Filter adalah filter frekuensi biquad berpresisi 64-bit float (f64) dengan pilihan kemiringan rolloff (12dB, 24dB, hingga 48dB/oktaf). Modul ini meloloskan frekuensi nada rendah (bass) dan meredam nada tinggi (treble) di atas titik Cutoff yang ditentukan.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Efek Akustik & Karakter Suara:").strong().size(12.5).color(Color32::from_rgb(14, 116, 144)));
                                        ui.label(egui::RichText::new("• Efek 'Underwater / Muffled': Suara musik terdengar seperti didengarkan dari dalam air atau dari balik pintu tebal.\n• Menghilangkan Desis & Harshness: Menyaring suara cymbal tajam, desis kaset tua (tape hiss), atau distorsi treble yang menusuk telinga.\n• Suasana Lo-Fi Santai: Sangat cocok untuk mendengarkan lagu saat malam hari, relaksasi, atau membaca buku.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Nilai Cutoff:").strong().size(12.5).color(Color32::from_rgb(2, 132, 199)));
                                        ui.label(egui::RichText::new("• 100 Hz - 300 Hz: Hanya getaran sub-bass murni yang terdengar (vokal hilang total).\n• 500 Hz - 1.2 kHz: Suara vokal teredam berat, bass hangat mendominasi.\n• 3 kHz - 6 kHz: Nuansa piringan hitam klasik, vokal tetap jelas tanpa 's' tajam.\n• 10 kHz: Pemotongan frekuensi ultrasonik, menjaga kehangatan analog sutra.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Kemiringan Slope & Fringe Q:").strong().size(12.5).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("• Slope 12dB/oct (2nd-Order Butterworth): Transisi pemotongan treble bertahap yang sangat musikal dan lembut, cocok untuk nuansa piringan hitam santai.\n• Slope 24dB/oct (4th-Order Linkwitz-Riley): Pemotongan tajam standar studio profesional untuk isolasi nada bass tanpa kebocoran vokal.\n• Slope 48dB/oct (8th-Order Brickwall): Pemotongan dinding batu curam untuk isolasi frekuensi bass murni.\n• Fringe Q (0.50 s/d 4.00): Nilai 0.707 (Butterworth) memberikan kurva datar tanpa resonansi. Naikkan ke 1.2 - 2.0 untuk memberikan hentakan resonansi punchy di frekuensi cutoff.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Apa itu Mixer / Summing Bus?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Mixer / Summing Bus adalah gerbang perantara (hub) cerdas yang menggabungkan dua aliran audio terpisah (Pin A dan Pin B) menjadi satu jalur output utuh.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Fungsi Utama & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(126, 34, 206)));
                                        ui.label(egui::RichText::new("• Parallel Processing (Dry/Wet): Mencampur suara murni tanpa efek (Dry) di Jalur A dengan suara yang disaring filter (Wet) di Jalur B. Anda dapat mengatur ketebalan efek tanpa merusak kejelasan vokal aslinya.\n• Constant-Power Crossfading: Menggunakan algoritma kurva daya konstan sehingga tidak ada penurunan volume di titik tengah (50/50).\n• Anti-Clipping Headroom: Mencegah distorsi digital saat dua sinyal dijumlahkan bersamaan.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Apa itu Splitter / Y-Branch Bus?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Splitter / Y-Branch Bus adalah modul pembagi sinyal audio cerdas yang menduplikasi 1 input audio menjadi 2 output mandiri (Pin Out A & Pin Out B) dengan teknologi Lossless 64-bit PCM Duplication.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Fungsi Utama & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(217, 119, 6)));
                                        ui.label(egui::RichText::new("• Pasangan Sejati Modul Mixer: Memungkinkan routing Dry/Wet paralel tanpa merusak kabel sinyal utama.\n• Zero Phase Delay (Fase Sinkron): Kedua pin memancarkan sampel bitstream yang sinkron mikrodetik yang sama, mencegah distorsi fase (comb filtering) saat disatukan kembali di Mixer.\n• Bit-Accurate Clone: Tidak ada penurunan gain maupun penurunan dynamic range sinyal.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::VolumeBooster {
                                        ui.label(egui::RichText::new("Apa itu Pure Volume Booster?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Pure Volume Booster adalah modul penguat gain murni berpresisi 64-bit float (f64) dari 0.0 dB hingga +24.0 dB tanpa pewarnaan atau degradasi akustik.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(139, 92, 246)));
                                        ui.label(egui::RichText::new("• 100% Transparent Amp: Penguatan sinyal murni tanpa saturasi trafo atau perubahan respon frekuensi.\n• Soft-Limit Guard: Dilengkapi algoritma proteksi batas lembut di ambang digital peak agar tidak terjadi distorsi keras (harsh digital clipping).\n• Sangat Ideal untuk Lagu Rekaman Rendah: Mengangkat rekaman vinyl atau master audio klasik yang terlalu pelan.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::AnalogPreamp {
                                        ui.label(egui::RichText::new("Apa itu Analog Preamp & Trim?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Analog Preamp & Trim adalah tahap penguatan gain studio (-24.0 dB s/d +12.0 dB) yang mereplikasi saturasi trafo analog diskrit Class-A.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(234, 88, 12)));
                                        ui.label(egui::RichText::new("• Transformer Core Saturation: Saat gain dinaikkan di atas 0 dB, gelombang transient instrumen dan vokal dibulatkan dengan kehangatan harmonik musikal.\n• Input Trim Attenuation: Menurunkan level gain hingga -24 dB untuk rekaman modern yang terlalu panas guna memulihkan headroom DAC.\n• Indikator Headroom Ladder: 4-LED meter visual memberi tahu status aman sinyal.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::DynamicLoudness {
                                        ui.label(egui::RichText::new("Apa itu Dynamic Loudness (ISO 226)?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Dynamic Loudness menerapkan kontur kurva equal-loudness standar internasional ISO 226 (Fletcher-Munson) untuk mengompensasi kelemahan sensitivitas pendengaran manusia pada volume rendah.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("• Bass & Treble Tetap Terdengar Penuh: Mengangkat bass (<120 Hz hingga +8.5 dB) dan treble (>9 kHz hingga +5.0 dB) saat mendengarkan musik di malam hari.\n• Mode Santai Malam Hari: Nikmati musik favorit dengan detail kaya tanpa harus menyetel volume speaker keras-keras.\n• Kendali Kedalaman (Depth 0-100%): Sesuaikan kekuatan kompensasi kontur sesuai headphone atau speaker Anda.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::SteppedAttenuator {
                                        ui.label(egui::RichText::new("Apa itu Stepped Attenuator?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Stepped Attenuator mereplikasi rotary attenuator pasif 64-langkah berbasis resistor ladder presisi tinggi (rentang -60.0 dB hingga 0.0 dB).").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(6, 182, 212)));
                                        ui.label(egui::RichText::new("• Resolusi Diskrit 1.0 dB: Menghilangkan distorsi resolusi dan ketidakseimbangan channel yang sering terjadi pada potensiometer analog murah.\n• Tombol Cepat [DIM -20dB]: Redam volume seketika sebesar 20 dB saat Anda perlu berbicara atau menerima panggilan telepon.\n• Zero Jitter & Bit-Accurate: Sinyal diturunkan dengan perkalian f64 bit-sempurna tanpa truncation noise.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::OpticalLeveler {
                                        ui.label(egui::RichText::new("Apa itu Studio Optical Leveler?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Studio Optical Leveler adalah dynamic leveler legendaris berbasis sel photocell optocoupler (terinspirasi dari kompresor studio Teletronix LA-2A).").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(180, 83, 9)));
                                        ui.label(egui::RichText::new("• Pelepasan 2-Tahap Alami (Two-Stage Release): Pelepasan cepat 50% awal dalam 60 ms, disusul pelepasan lambat bertahap selama beberapa detik. Sangat transparan tanpa efek napas/pumping.\n• Menjinakkan Dinamika Vokal & Akustik: Meratakan lonjakan vokal mendadak sehingga vokal terdengar konsisten dan intim di telinga.\n• Indikator Jarum VU Meter: Jarum meter bergerak secara visual menunjukkan reduksi puncak sinyal.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::StereoBalance {
                                        ui.label(egui::RichText::new("Apa itu Stereo Balance & Trim?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Stereo Balance & Trim adalah kendali posisi panorama panggung suara stereo menggunakan kurva panning Constant-Power sinus dan kosinus (-100% Kiri s/d +100% Kanan).").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Karakteristik & Keunggulan:").strong().size(12.5).color(Color32::from_rgb(79, 70, 229)));
                                        ui.label(egui::RichText::new("• Daya Akustik Konstan (Constant-Power Panning): Tidak ada penurunan energi suara saat instrumen berada di tengah (Center detent 0 dB).\n• Kompensasi Pendengaran & Speaker Ruangan: Koreksi ketidakseimbangan posisi duduk pendengar atau perbedaan sensitivitas telinga kiri dan kanan.\n• Snapping Center Detent: Slider mudah dikembalikan ke titik imbang tengah 0.0 dB.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::TubeWarmer {
                                        ui.label(egui::RichText::new("Apa itu Tube Warmer (Analog Triode Saturation)?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Tube Warmer adalah modul pemroses saturasi tabung triode analog (berbasis tabung legendaris 12AX7) dengan matematika 64-bit float. Modul ini mereplikasi distorsi harmonik musikal yang biasa dihasilkan oleh tabung vakum pre-amp amplifier audiophile kelas atas.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Efek Akustik & Karakter Suara:").strong().size(12.5).color(Color32::from_rgb(234, 88, 12)));
                                        ui.label(egui::RichText::new("• Kehangatan Harmonik Genap (Even Harmonics): Menghasilkan distorsi harmonik ke-2 (oktaf atas) asimetris yang secara alami dirasakan telinga manusia sebagai suara yang lebih tebal, empuk, dan berisi.\n• Menjinakkan Digital Harshness: Melembutkan ujung suara digital yang dingin, kaku, atau menusuk (fatiguing), sehingga musik nyaman didengarkan berjam-jam.\n• Vokal Lebih Intim & Berbobot: Membuat vokal penyanyi terdengar lebih dekat ke telinga (in the room) dengan resonansi tubuh dan dada yang kaya.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Nilai Warmth Tabung Nixie:").strong().size(12.5).color(Color32::from_rgb(194, 65, 12)));
                                        ui.label(egui::RichText::new("• 0%: Transparent bypass murni (0% distorsi tambahan, tabung mati).\n• 20% - 35% (Audiophile Sweet Spot): Nuansa tabung triode 12AX7 halus, menjaga detail instrumen sambil memberi kehangatan vokal.\n• 50% - 65%: Nuansa piringan hitam (vinyl) vintage tebal dan bass analog punchy.\n• 75% - 100%: Saturasi analog pekat (hot tube drive) untuk genre rock klasik atau gitar elektrik.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else if filter_type == NodeType::StereoWidener {
                                        ui.label(egui::RichText::new("Apa itu Stereo 3D Widener (Haas Effect)?").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Stereo 3D Widener adalah modul perluasan medan spasial berbasis matriks Mid/Side (M/S) decoding dan psikoakustik efek Haas. Modul ini memisahkan komponen tengah (Mid = (L+R)/2) dan sisi luar (Side = (L-R)/2), lalu memperluas dimensi suara stereo secara dinamis.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Efek Akustik & Karakter Suara:").strong().size(12.5).color(Color32::from_rgb(99, 102, 241)));
                                        ui.label(egui::RichText::new("• Soundstage Terbuka Lebar: Musik yang tadinya terasa sempit 'di dalam kepala' melebar keluar seolah-olah Anda mendengarkan sepasang speaker studio berjarak 3 meter di hadapan Anda.\n• Vokal Tetap Fokus di Tengah: Berbeda dengan efek surround murahan yang membuat vokal kabur, metode Mid/Side menjaga kanal Mid 100% utuh sehingga vokal penyanyi tetap kokoh di tengah panggung.\n• Pemisahan Instrumen Jernih: Instrumen pengiring (gitar latar, backing vocal, reverb) menyebar ke sisi kiri dan kanan, memberikan ruang lega bagi vokal utama.").size(12.0).color(Color32::from_rgb(71, 85, 105)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("Panduan Nilai Soundstage 3D:").strong().size(12.5).color(Color32::from_rgb(79, 70, 229)));
                                        ui.label(egui::RichText::new("• +10% - +25%: Perluasan natural transparan, ideal untuk headphone closed-back yang berkarakter sempit.\n• +30% - +45% (Rekomendasi Utama): Titik temu sempurna psikoakustik Haas 3D tanpa efek hampa di tengah.\n• +50% - +75%: Nuansa live concert / hall besar, memanjakan telinga pada rekaman live akustik.\n• +100%: Pelebaran ekstrem untuk pengalaman imersif sinematik.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    } else {
                                        ui.label(egui::RichText::new("Karakter Suara Modul").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("Modul ini dirancang khusus untuk memproses karakter tonal, kedalaman analog, atau batas keamanan sinyal audio Anda secara real-time.").size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                    }
                                }
                                1 => {
                                    // Slide 2 text
                                    if filter_type == NodeType::Crossover {
                                        ui.label(egui::RichText::new("Di Mana Crossover Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Pemrosesan Pita Terpisah & Satukan di Mixer [Rekomendasi Utama]:\n   Karena Lunula Player hanya memiliki satu terminal Output DAC fisik, selalu gunakan modul 'Mixer' untuk menggabungkan kedua cabang sebelum menuju DAC:\n   - Tarik Pin 0 (LOW) ke modul efek bass (misal Tube Warmer untuk saturasi analog tebal) ➔ colokkan ke Mixer Pin A.\n   - Tarik Pin 1 (HIGH) ke modul efek treble (misal Stereo Widener untuk panggung 3D) ➔ colokkan ke Mixer Pin B.\n   - Sambungkan output Mixer ke App Resampler ➔ Output DAC. Kedua frekuensi berpadu utuh 0.00 dB flat!\n\n2. Makro Keseimbangan Nada (Tone Balance Shaper):\n   Colokkan Pin 0 (LOW) langsung ke Mixer Pin A, dan Pin 1 (HIGH) langsung ke Mixer Pin B.\n   - Slider crossfader Mixer kini berfungsi sebagai tuas pengatur keseimbangan instan: geser ke A untuk dominasi bass, geser ke B untuk dominasi treble, atau taruh di tengah (50/50) untuk sinyal datar asli.\n\n3. Solo Auditioning (Memantau Pita Frekuensi Tunggal):\n   Jika Anda hanya ingin mendengar nada rendah saja tanpa vokal/treble, tarik Pin 0 (LOW) langsung ke App Resampler / Output DAC (Pin 1 dibiarkan kosong). Sebaliknya, tarik Pin 1 (HIGH) jika Anda hanya ingin mengisolasi vokal dan treble.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Fitur 2 Pin Output Aktif (LOW & HIGH):").strong().size(12.5).color(Color32::from_rgb(147, 51, 234)));
                                        ui.label(egui::RichText::new("Crossover memiliki 2 pin keluaran independen. Ingat bahwa Output DAC hanya menerima 1 kabel input, sehingga modul Mixer adalah pasangan ideal untuk menyatukan kembali kedua pin tersebut.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::HighPassFilter {
                                        ui.label(egui::RichText::new("Di Mana High-Pass Filter Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Rantai Awal Pembersihan (Antara RAM Buffer & App Resampler / Filter Modular) [Rekomendasi Utama]:\n   Membersihkan getaran sub-bass rumble langsung dari hasil decode mentah sebelum sinyal masuk ke efek saturasi atau resampler.\n\n2. Sebelum Tube Warmer atau Bass Guard:\n   Mencegah tabung saturasi mendistorsi akibat frekuensi sub-bass yang terlalu kuat, menghasilkan saturasi vokal yang jauh lebih bersih.\n\n3. Pada Jalur Paralel Pin B Mixer:\n   Hanya memproses treble pada jalur wet.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Presisi Biquad 64-bit:").strong().size(12.5).color(Color32::from_rgb(56, 189, 248)));
                                        ui.label(egui::RichText::new("Perhitungan koefisien biquad dieksekusi dengan presisi ganda f64, menjamin kestabilan filter bahkan pada frekuensi potong serendah 20 Hz.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::BandPassFilter {
                                        ui.label(egui::RichText::new("Di Mana Band-Pass Filter Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Jalur Efek Paralel (Splitter ➔ BPF ➔ Mixer Pin B) [Rekomendasi Utama]:\n   Campurkan suara vokal/instrumen yang diisolasi BPF dengan suara asli lagu untuk menciptakan efek tebal berlapis tanpa merusak audio utama.\n\n2. Seri Setelah Equalizer:\n   Gunakan setelah EQ untuk menonjolkan solo instrumen tertentu secara spesifik.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Respon Bebas Resonansi Buruk:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Gain puncak pada center frequency dipertahankan tepat pada 0.0 dB untuk mencegah clipping digital.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Di Mana Mixer Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Gerbang Penggabung Akhir (Sebelum App Resampler / DAC) [Rekomendasi Utama]:\n   Tarik cabang 1 langsung dari RAM Buffer / Splitter Pin A (Dry), dan cabang 2 melalui Filter Efek ke Pin B (Wet). Output Mixer disambungkan ke App Resampler atau Output DAC.\n\n2. Di Antara Filter Modular (Sub-Bus Mixing):\n   Menggabungkan keluaran dua filter berbeda (misal LPF dan Tube Warmer) sebelum keduanya diteruskan ke Stereo Widener.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Fleksibilitas Input:").strong().size(12.5).color(Color32::from_rgb(126, 34, 206)));
                                        ui.label(egui::RichText::new("Jika Anda hanya menyambungkan 1 kabel ke Pin A (Pin B kosong), Mixer otomatis mengalirkan sinyal Channel A tanpa error.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Di Mana Splitter Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Percabangan Awal (Diamond Loop) [Rekomendasi Utama]:\n   Colokkan kabel dari App Resampler atau RAM Buffer ke Input Splitter. Tarik Pin Out A langsung ke Mixer (Pin A), dan Pin Out B ke Low-Pass Filter lalu ke Mixer (Pin B). Output Mixer teruskan ke DAC.\n\n2. Multi-Filter Bus Routing:\n   Membagi sinyal untuk diproses dua filter berbeda secara independen sebelum disatukan kembali.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Aturan 2 Output:").strong().size(12.5).color(Color32::from_rgb(217, 119, 6)));
                                        ui.label(egui::RichText::new("Berbeda dari kartu biasa yang hanya mengizinkan 1 kabel keluar, Splitter mengizinkan 2 kabel aktif keluar sekaligus (Pin A dan Pin B).").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::TubeWarmer {
                                        ui.label(egui::RichText::new("Di Mana Tube Warmer Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Setelah RAM Buffer atau Equalizer Modular (Rekomendasi Utama Audiophile):\n   Biarkan sinyal mengalir dari RAM Buffer atau EQ modular terlebih dahulu, lalu teruskan ke Tube Warmer agar saturasi harmonik analog memproses karakter nada yang sudah seimbang.\n\n2. Sebelum Stereo Widener:\n   Pasang sebelum Stereo Widener agar panggung suara 3D dapat menyebarkan kehangatan analog ke seluruh ruang stereo secara merata.\n\n3. Di Percabangan Paralel Mixer (Dry/Wet):\n   Gunakan Splitter dan Mixer untuk memisahkan jalur suara murni (Dry) dan suara tabung (Wet), lalu atur persentase campurannya.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Pemrosesan Presisi 64-bit:").strong().size(12.5).color(Color32::from_rgb(234, 88, 12)));
                                        ui.label(egui::RichText::new("Semua komputasi saturasi triode dihitung dalam domain 64-bit float dengan normalisasi gain otomatis sehingga tidak memicu clipping digital.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::StereoWidener {
                                        ui.label(egui::RichText::new("Di Mana Stereo Widener Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Tahap Akhir Sebelum DAC (Rekomendasi Utama Audiophile):\n   Stereo Widener paling ideal dipasang sebagai tahap pemrosesan paling akhir sebelum Output DAC, sehingga seluruh instrumen, vokal, dan efek filter sebelumnya ikut dimekarkan dalam panggung suara 3D.\n\n2. Sesudah Tube Warmer atau Low-Pass Filter:\n   Menyebarkan nada hangat analog atau suara teredam ke sisi luar panggung suara stereo.\n\n3. Jalur Wet Bus pada Mixer:\n   Dipasang pada Channel B Mixer untuk efek pelebaran selektif.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Prasyarat Stereo:").strong().size(12.5).color(Color32::from_rgb(99, 102, 241)));
                                        ui.label(egui::RichText::new("Efek ini membutuhkan audio stereo (memiliki perbedaan kanal L dan R). Jika lagu adalah mono murni (L = R), modul otomatis meloloskan sinyal tanpa error.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::LowPassFilter {
                                        ui.label(egui::RichText::new("Di Mana Low-Pass Filter Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Antara RAM Buffer dan App Resampler (Rekomendasi Utama):\n   Memotong nada tinggi langsung dari hasil decode mentah sebelum sinyal masuk ke App Resampler atau Output DAC.\n\n2. Berurutan Sebelum Tube Warmer atau Efek Lain:\n   Pasang sebelum Tube Warmer agar saturasi analog memproses frekuensi bass hangat yang empuk dan halus tanpa desis treble.\n\n3. Di Percabangan Paralel Pin B Mixer (Dry/Wet):\n   Gunakan Splitter dan Mixer untuk mencampur suara asli tanpa filter (Pin A) dengan suara teredam LPF (Pin B).").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Jaminan Kemurnian:").strong().size(12.5).color(Color32::from_rgb(2, 132, 199)));
                                        ui.label(egui::RichText::new("Semua sampel audio dalam RAM Buffer diproses dengan matematika 64-bit float biquad, sehingga penyisipan filter tidak menimbulkan noise lantai maupun distorsi bit.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else {
                                        ui.label(egui::RichText::new("Di Mana Sinyal Boleh Disambungkan?").strong().size(14.0).color(Color32::from_rgb(16, 185, 129)));
                                        ui.label(egui::RichText::new("1. Antara RAM Buffer dan App Resampler (Rekomendasi Utama):\n   Memproses frekuensi atau modulasi audio langsung sebelum sinyal masuk ke tahap resample atau DAC hardware.\n\n2. Berurutan (In-Series) dengan Filter Lain:\n   Pasang berurutan dengan modul efek lainnya untuk meracik rantai mastering studio profesional.\n\n3. Di Percabangan Paralel Mixer:\n   Pasang pada salah satu pin Mixer untuk pemrosesan paralel.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("💡 Jaminan Kemurnian:").strong().size(12.5).color(Color32::from_rgb(2, 132, 199)));
                                        ui.label(egui::RichText::new("Semua sampel audio dalam RAM Buffer diproses dengan matematika 64-bit float, sehingga penyisipan filter tidak menimbulkan noise lantai maupun distorsi bit.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    }
                                }
                                2 => {
                                    // Slide 3 text
                                    if filter_type == NodeType::Crossover {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Crossover").strong().size(14.0).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Feedback Loop:\n   Jangan menarik kabel dari Pin 0 (LOW) atau Pin 1 (HIGH) kembali ke Pin Input Crossover itu sendiri.\n\n2. 🚫 Dilarang Memasang Crossover Setelah Output DAC:\n   Crossover LR4 adalah modul pemrosesan digital 64-bit di software, sehingga wajib berada sebelum konversi DAC fisik hardware.\n\n3. 🚫 Hindari Menghubungkan 2 Output Langsung ke DAC Tanpa Mixer:\n   Terminal Output DAC hanya memiliki 1 pin input. Untuk mendengarkan kedua rentang LOW dan HIGH secara bersamaan, selalu satukan keduanya menggunakan modul Mixer (Pin A & Pin B) sebelum menuju DAC.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika loop terdeteksi atau kabel putus, sistem otomatis membisukan suara demi melindungi telinga dan transduser speaker Anda.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::HighPassFilter || filter_type == NodeType::BandPassFilter {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Filter").strong().size(14.0).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Sebelum File Source:\n   Filter membutuhkan aliran PCM terdekode, sehingga tidak bisa dipasang sebelum node File Source.\n\n2. 🚫 Dilarang Setelah Output DAC:\n   Output DAC mengirim voltase listrik analog nyata ke kabel headphone. Tidak ada modul efek software yang bisa ditaruh sesudahnya.\n\n3. 🚫 Hindari Loop Output-ke-Input:\n   Menyambungkan kabel keluaran filter kembali ke inputnya sendiri akan memicu distorsi perulangan feedback tak terkendali.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika kabel terputus di tengah pemutaran, sistem secara otomatis membisukan suara (silent mute) untuk mencegah letupan DC offset.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Mixer").strong().size(14.0).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Feedback Loop:\n   Jangan menarik kabel dari pin Output Mixer kembali ke pin Input A atau Input B milik Mixer itu sendiri. Hal ini memicu loop tak berujung yang berbahaya bagi speaker.\n\n2. 🚫 Dilarang Sebelum File Source / Sesudah DAC:\n   Mixer adalah pemroses sinyal digital 64-bit float di software, sehingga wajib berada di antara File Source (decoder) dan Output DAC fisik.\n\n3. 🚫 Duplikasi Jalur Identik:\n   Menyambungkan kabel yang sama persis ke Pin A dan Pin B tanpa filter hanya akan melipatgandakan volume tanpa memberikan variasi suara.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika loop terdeteksi atau kabel putus di tengah jalan, sistem secara otomatis membisukan suara untuk melindungi telinga Anda.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Splitter").strong().size(14.0).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Feedback Loop:\n   Jangan menarik kabel dari pin Output A atau B kembali ke pin Input Splitter itu sendiri.\n\n2. 🚫 Dilarang Sebelum File Source / Sesudah DAC:\n   Splitter bekerja pada domain PCM 64-bit di software, sehingga wajib berada di antara decoder berkas dan Output DAC fisik.\n\n3. 🚫 Jangan Menggantung Output Tanpa Tujuan:\n   Jika hanya 1 jalur yang dibutuhkan, gunakan kabel sambungan langsung tanpa Splitter agar kanvas tetap rapi.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika loop terdeteksi atau kabel putus di tengah jalan, sistem secara otomatis membisukan suara untuk melindungi telinga dan speaker Anda.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::TubeWarmer {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Tube Warmer").strong().size(14.0).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Sebelum File Source:\n   File Source membaca bit data mentah berkas dari storage (FLAC, ALAC, MP3, DSD). Tube Warmer membutuhkan sampel PCM yang sudah didecode, sehingga tidak dapat dipasang sebelum decoder.\n\n2. 🚫 Dilarang Sesudah Output DAC:\n   Output DAC adalah hardware fisik (chip DAC laptop/DAC eksternal) yang mengubah bit digital ke voltase analog listrik ke kabel headphone. Tidak ada modul software yang bisa diletakkan sesudahnya.\n\n3. 🚫 Hindari Percabangan Balik (Feedback Loop):\n   Jangan menyambungkan pin keluaran Tube Warmer kembali ke pin masukan miliknya sendiri, karena akan memicu perulangan saturasi tak berujung.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika kabel terputus di tengah jalan atau loop terdeteksi, LunulaPlayer otomatis membisukan sinyal demi melindungi telinga dan headphone Anda.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else if filter_type == NodeType::StereoWidener {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Kesalahan Routing Stereo Widener").strong().size(14.0).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Sebelum File Source / Sesudah DAC:\n   Modul ini wajib berada di domain transmisi digital PCM software sebelum tahap konversi DAC hardware.\n\n2. 🚫 Dilarang Feedback Loop:\n   Menghubungkan output Stereo Widener kembali ke inputnya sendiri dapat memicu pembatalan fase (phase cancellation) parah.\n\n3. 🚫 Hindari Pemakaian pada Rekaman Mono Murni:\n   Pada berkas mono sejati (kanal L dan R identik), nilai selisih Side adalah 0. Efek pelebaran tidak akan berdampak pada audio mono.").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika kabel terputus di tengah jalan, sistem secara otomatis membisukan suara untuk melindungi speaker Anda dari suara letupan (pop/click).").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    } else {
                                        ui.label(egui::RichText::new("Posisi Terlarang & Bahaya Routing").strong().size(14.0).color(Color32::from_rgb(239, 68, 68)));
                                        ui.label(egui::RichText::new("1. 🚫 Dilarang Sebelum File Source:\n   File Source membaca bit data mentah berkas dari SSD/HDD (FLAC, MP3, DSD). Filter membutuhkan data PCM yang sudah didecode, sehingga tidak bisa ditaruh sebelum decoder.\n\n2. 🚫 Dilarang Sesudah Output DAC:\n   Output DAC adalah terminal hardware fisik terakhir di laptop/PC Anda (chip ESS Sabre, Realtek, USB DAC) yang langsung mengirim gelombang analog ke kabel jack speaker. Tidak ada modul software yang bisa dipasang sesudahnya.\n\n3. 🚫 Hindari Percabangan Balik (Feedback Loop):\n   Jangan menyambungkan pin keluaran (Output) kembali ke pin masukan (Input) modul yang sama, karena akan menciptakan putaran suara melengking (feedback loop).").size(12.0).color(Color32::from_rgb(51, 65, 85)));

                                        ui.add_space(4.0);
                                        ui.label(egui::RichText::new("🛡️ Proteksi Auto-Mute LunulaPlayer:").strong().size(12.5).color(Color32::from_rgb(245, 158, 11)));
                                        ui.label(egui::RichText::new("Jika kabel terputus di tengah jalan, LunulaPlayer otomatis membisukan sinyal (silent mute) untuk melindungi speaker dan telinga Anda dari suara ledakan keras (pop/click).").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                    }
                                }
                                _ => {
                                    // Slide 4 text
                                    if filter_type == NodeType::HighPassFilter {
                                        ui.label(egui::RichText::new("Mengatur HPF, Slope, & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Cutoff Real-Time (20 Hz s/d 2.0 kHz):\n  Tarik fader langsung di kartu kanvas atau di sidebar kanan. Pembaruan koefisien biquad f64 terjadi seketika tanpa jeda audio.\n\n• Tombol Pilihan Kemiringan Slope (dB/oktaf):\n  - 12dB: Pemotongan lembut natural, menjaga body nada rendah tetap ada.\n  - 24dB: Standar studio rekaman untuk isolasi vokal tajam.\n  - 48dB: Brickwall subsonic cut untuk perlindungan speaker bertenaga tinggi.\n\n• Kendali Resonansi Fringe Q:\n  Sesuaikan nilai Q (0.5 hingga 2.5). Nilai 0.707 memberikan respon Butterworth paling linier, sedangkan nilai di atas 1.0 menambahkan sedikit resonansi punchy di titik potong.\n\n• Perilaku pada Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, High-Pass Filter otomatis DILEWATI (bypassed) demi kemurnian 1:1 stream bit audio mentah.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::BandPassFilter {
                                        ui.label(egui::RichText::new("Mengatur BPF, Bandwidth Q, & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Center Frequency (50 Hz s/d 16 kHz):\n  Arahkan titik tengah filter ke instrumen atau vokal yang ingin Anda isolasi.\n\n• Slider Bandwidth Q (0.3 s/d 5.0):\n  - Q = 0.5 - 0.7: Pita lebar untuk pemanasan tonal halus.\n  - Q = 1.41: Standar isolasi 1 oktaf penuh.\n  - Q = 3.0 - 5.0: Resonansi tajam seperti filter formant instrumen analog atau efek radio.\n\n• Pilihan Slope 12dB / 24dB:\n  Memilih seberapa curam nada di luar pita frekuensi diredam.\n\n• Perilaku pada Mode Bit-Perfect:\n  Saat mode Bit-Perfect aktif, seluruh pemrosesan BPF di-bypass tanpa mengubah 1 bit pun data lagu.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::Crossover {
                                        ui.label(egui::RichText::new("Mengatur Titik Potong Crossover LR4 & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Crossover Real-Time (80 Hz s/d 8.0 kHz):\n  Geser frekuensi titik potong Linkwitz-Riley secara live. Kedua cabang LOW dan HIGH otomatis terkomputasi sinkron secara internal tanpa 'zipper noise'.\n\n• Preset Crossover Populer:\n  - 80 Hz: Memisahkan dentuman sub-bass murni dari vokal/melodi.\n  - 500 Hz: Membagi rentang low body dengan artikulasi midrange vokal.\n  - 2.5 kHz: Memisahkan instrumen midrange dengan kemilau treble udara.\n\n• Menyatukan Sinyal ke Output DAC:\n  Agar kedua cabang LOW dan HIGH dapat didengarkan bersamaan di headphone/speaker melalui satu Output DAC, hubungkan Pin 0 ke Mixer Pin A dan Pin 1 ke Mixer Pin B, lalu teruskan output Mixer ke DAC.\n\n• Perilaku pada Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, modul Crossover otomatis dilewati dan sinyal diteruskan 1:1 ke DAC.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::Mixer {
                                        ui.label(egui::RichText::new("Mengatur Crossfader & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Crossfader Real-Time:\n  Tarik slider langsung di atas kartu kanvas atau di sidebar kanan dari 0% (Channel A murni) hingga 100% (Channel B murni). Nilai diperbarui seketika tanpa latency.\n\n• Posisi Slider Populer:\n  - 0% B: Suara asli tanpa efek (Dry murni).\n  - 30% - 50% B: Suara asli tetap dominan, dengan sentuhan hangat/muffled dari filter B.\n  - 50% / 50%: Campuran seimbang sempurna (Constant-Power Crossfade).\n  - 100% B: Suara efek total (Wet murni).\n\n• Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, seluruh rantai pemrosesan termasuk Mixer dilewati (direct stream 1:1) ke DAC hardware.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::Splitter {
                                        ui.label(egui::RichText::new("Karakteristik Operasional & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Otomatis & Bebas Penyetelan:\n  Splitter tidak membutuhkan slider atau kenop karena bertugas menduplikasi sinyal 100% secara akurat dan transparan.\n\n• Pasangan Sempurna dengan Mixer (Diamond Loop):\n  - Splitter: Gerbang Pembagi (1 In ➔ 2 Out)\n  - Mixer: Gerbang Penggabung (2 In ➔ 1 Out)\n  Kombinasi keduanya menciptakan arsitektur parallel processing profesional standar studio rekaman!\n\n• Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, seluruh modul termasuk Splitter dilewati (direct stream 1:1) ke DAC hardware.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::VolumeBooster {
                                        ui.label(egui::RichText::new("Mengatur Volume Booster & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Real-Time (0 s/d +24 dB):\n  Tarik fader langsung di kartu kanvas atau sidebar kanan. Gain bertambah mulus tanpa bunyi 'zipper click'.\n\n• Tombol Cepat Preset (+3dB, +6dB, +12dB, +18dB):\n  Gunakan preset cepat untuk memilih penguatan standar tanpa perlu menggeser manual.\n\n• Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, booster otomatis dilewati demi integritas 1:1 data stream.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::AnalogPreamp {
                                        ui.label(egui::RichText::new("Mengatur Preamp & Saturasi Class-A").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Dial Putar & Slider (-24 s/d +12 dB):\n  Putar dial aluminium atau gunakan slider trim. Titik tengah 0 dB merupakan detent netral (unity gain).\n\n• Preset Studio Cepat:\n  - -6dB Trim: Mengamankan lagu modern dari inter-sample peak.\n  - 0dB Unity: Penyangga linier murni.\n  - +3dB / +6dB: Mengaktifkan saturasi trafo analog Class-A untuk vokal hangat.\n\n• Mode Bit-Perfect:\n  Preamp otomatis di-bypass saat Bit-Perfect diaktifkan.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::DynamicLoudness {
                                        ui.label(egui::RichText::new("Mengatur Kontur Loudness ISO 226").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Kedalaman (0% s/d 100%):\n  Atur intensitas senyum kurva frekuensi dari 0% (datar) hingga 100% (penuh).\n\n• Rekomendasi Penggunaan:\n  - 25% Mild: Untuk speaker desktop jarak dekat (near-field).\n  - 50% Night: Rekomendasi standar mendengarkan musik santai di malam hari.\n  - 75% - 100%: Untuk headphone berkarakter dingin/tipis atau rekaman lama yang minim bass.\n\n• Mode Bit-Perfect:\n  Kompensasi loudness otomatis dilewati saat beralih ke Bit-Perfect.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::SteppedAttenuator {
                                        ui.label(egui::RichText::new("Mengatur Stepped Attenuator & DIM").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Stepped Slider (-60 dB s/d 0 dB):\n  Melompat presisi 1.0 dB per step, meniru klik mekanis resistor ladder audiophile.\n\n• Tombol Cepat [DIM -20dB]:\n  Klik tombol DIM langsung di kartu kanvas atau sidebar untuk menurunkan level ke -20 dB seketika, dan klik lagi untuk kembali ke volume awal.\n\n• Mode Bit-Perfect:\n  Saat mode Bit-Perfect aktif, seluruh peredaman digital dilewati murni ke DAC.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::OpticalLeveler {
                                        ui.label(egui::RichText::new("Mengatur Peak Reduction LA-2A").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Peak Reduction (0% s/d 100%):\n  Mengatur ambang sensitivitas kompresi optik. Semakin tinggi persentase, semakin intim dan rata dinamika suaranya.\n\n• Rekomendasi Preset:\n  - 20% Light: Leveling transparan untuk musik orkestra dan jazz.\n  - 40% Classic: Karakter LA-2A klasik untuk vokal pop dan akustik.\n  - 65% Heavy: Mengunci vokal di depan panggung suara (in-your-face vocal).\n\n• Mode Bit-Perfect:\n  Leveler otomatis di-bypass saat beralih ke Bit-Perfect.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::StereoBalance {
                                        ui.label(egui::RichText::new("Mengatur Panning Stereo & Center Detent").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Panorama (-100% Kiri s/d +100% Kanan):\n  Menggeser panggung suara dengan kurva daya konstan tanpa perubahan level akustik total.\n\n• Tombol Instan Center:\n  Klik tombol '● Center' di sidebar untuk mengembalikan posisi instrumen ke titik tengah 0.0 dB secara instan.\n\n• Mode Bit-Perfect:\n  Panning stereo dilewati murni saat mode Bit-Perfect diaktifkan.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::TubeWarmer {
                                        ui.label(egui::RichText::new("Mengatur Warmth Tabung Nixie & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Warmth Real-Time (0% s/d 100%):\n  Tarik fader langsung di kartu kanvas atau di sidebar kanan. Nilai angka neon pada tabung Nixie akan berubah secara dinamis seketika tanpa jeda.\n\n• Tombol Cepat Preset Studio:\n  - 20% Mild: Sentuhan analog sangat tipis untuk jazz dan orkestra.\n  - 35% Triode: Karakter triode 12AX7 seimbang, vokal intim hangat.\n  - 50% Rich: Nuansa tape analog & piringan hitam klasik.\n  - 75% Hot: Saturasi tabung tebal untuk rock klasik atau synth vintage.\n\n• Perilaku pada Mode Bit-Perfect:\n  Saat beralih ke Bit-Perfect, seluruh filter modular (termasuk Tube Warmer) otomatis DILEWATI (bypassed) demi integritas 1:1 data stream. Lampu tabung Nixie akan mati (dormant) sebagai tanda visual bypass.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::StereoWidener {
                                        ui.label(egui::RichText::new("Mengatur Soundstage 3D & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Slider Soundstage Real-Time (0% s/d +100%):\n  Sesuaikan dimensi panggung suara dari kartu kanvas maupun sidebar. Sinyal bertransisi mulus berkat parameter slewing anti-zipper.\n\n• Tombol Cepat Preset Psikoakustik:\n  - +15% Natural: Sangat direkomendasikan untuk IEM dan headphone rapat (closed-back).\n  - +35% Haas: Titik seimbang panggung suara audio rumahan (sweet spot).\n  - +60% Wide: Nuansa live concert aula besar untuk rekaman konser.\n  - +100% 3D: Panggung suara 3D maksimal untuk pengalaman sinematik imersif.\n\n• Perilaku pada Mode Bit-Perfect:\n  Stereo Widener otomatis dilewati saat beralih ke Bit-Perfect untuk menjaga struktur file master asli tanpa alterasi bit.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else if filter_type == NodeType::LowPassFilter {
                                        ui.label(egui::RichText::new("Mengatur LPF, Slope, & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Geser Slider Cutoff Real-Time (100 Hz s/d 10 kHz):\n  Anda dapat menggeser slider cutoff langsung di atas kartu kanvas atau sidebar kanan seketika tanpa jeda audio.\n\n• Tombol Pilihan Kemiringan Slope (dB/oktaf):\n  - 12dB: Pemotongan lembut natural (Butterworth 2nd-order).\n  - 24dB: Pemotongan tajam standar studio (Linkwitz-Riley 4th-order).\n  - 48dB: Pemotongan brickwall curam untuk isolasi bass murni.\n\n• Kendali Resonansi Fringe Q:\n  Sesuaikan nilai Q (0.5 hingga 2.5) untuk menambah tonjolan resonansi punchy di titik cutoff.\n\n• Perilaku pada Mode Bit-Perfect:\n  Mode Bit-Perfect mewajibkan data lagu dikirim ke DAC murni 1:1 tanpa modifikasi 1 bit pun. Oleh karena itu, saat mode Bit-Perfect aktif, seluruh filter kustom otomatis DILEWATI (bypassed) melalui kawat direct stream hijau langsung.\n\n• Cara Mengaktifkan Efek:\n  Cukup tekan tombol toggle mode ke [Modular FX] / [DSP Active] di bilah atas kanvas. Seluruh filter modular yang tersambung akan seketika memproses suara.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
                                    } else {
                                        ui.label(egui::RichText::new("Mengatur Slider & Mode Audio").strong().size(14.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new("• Geser Slider & Parameter:\n  Anda dapat menggeser slider langsung di atas kartu kanvas atau sidebar kanan secara real-time tanpa jeda dan tanpa perlu me-rebuild graph sinyal.\n\n• Perilaku pada Mode Bit-Perfect:\n  Mode Bit-Perfect mewajibkan data lagu dikirim ke DAC murni 1:1 tanpa modifikasi 1 bit pun. Oleh karena itu, saat mode Bit-Perfect aktif, seluruh filter kustom otomatis DILEWATI (bypassed) melalui kawat direct stream langsung.\n\n• Cara Mengaktifkan Efek:\n  Cukup tekan tombol toggle mode ke [Modular FX] / [DSP Active] di bilah atas kanvas. Seluruh rantai filter modular yang Anda pasang akan seketika aktif memproses suara.").size(12.0).color(Color32::from_rgb(51, 65, 85)));
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
                                    ui.label(egui::RichText::new("✓ Terpasang").size(12.0).color(Color32::from_rgb(16, 185, 129)));
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
