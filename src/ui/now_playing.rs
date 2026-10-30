use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::WaveformMode;
use crate::app::LunulaApp;
use crate::ui::components::{format_time, lerp_color, render_card_frame};

impl LunulaApp {
    pub fn render_ambient_waveform_station(&mut self, ui: &mut egui::Ui) {
        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                // Top Header: Track Title & Live Info
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let fav_icon = if self.current_track.is_favorite { "❤️" } else { "🤍" };
                            if ui.button(egui::RichText::new(fav_icon).size(15.0)).on_hover_text("Tandai sebagai Favorit").clicked() {
                                self.current_track.is_favorite = !self.current_track.is_favorite;
                                let is_fav = self.current_track.is_favorite;
                                for t in self.playlist.iter_mut() {
                                    if t.title == self.current_track.title && t.artist == self.current_track.artist {
                                        t.is_favorite = is_fav;
                                    }
                                }
                            }
                            ui.label(
                                egui::RichText::new(&self.current_track.title)
                                    .strong()
                                    .size(17.5)
                                    .color(Color32::from_rgb(15, 23, 42)),
                            );
                            ui.label(
                                egui::RichText::new(format!("— {}", self.current_track.artist))
                                    .strong()
                                    .size(15.0)
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                        });
                        if self.current_track.duration > 0.0 {
                            ui.label(
                                egui::RichText::new(format!("{} • {} kHz / {}-bit Hi-Res (DR14 Mastering)", self.current_track.format, self.current_track.sample_rate / 1000, self.current_track.bit_depth))
                                    .strong()
                                    .size(11.5)
                                    .color(Color32::from_rgb(14, 165, 233)),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("Lunula Engine Standby • Buka Tab Library untuk Memilih Musik")
                                    .strong()
                                    .size(11.5)
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let zoom_factor = self.zoom_level / 16.0;
                        ui.label(
                            egui::RichText::new(format!("Ctrl + Scroll to Zoom ({:.1}x)", zoom_factor))
                                .strong()
                                .size(12.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                    });
                });

                ui.add_space(8.0);

                // Waveform Canvas (Full width, filled down to bottom as background wave)
                let wave_width = ui.available_width();
                let wave_height = 135.0;
                let (wave_rect, wave_res) = ui.allocate_exact_size(
                    Vec2::new(wave_width, wave_height),
                    egui::Sense::click_and_drag(),
                );

                // Ctrl + Mouse Wheel Zoom
                if wave_res.hovered() {
                    let is_ctrl = ui.input(|i| i.modifiers.ctrl);
                    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll_delta != 0.0 && is_ctrl {
                        let factor = if scroll_delta > 0.0 { 1.20 } else { 0.83 };
                        self.zoom_level = (self.zoom_level * factor).clamp(6.0, 160.0);
                    }
                }

                // Drag Scrubbing
                if wave_res.dragged() {
                    let delta_x = ui.input(|i| i.pointer.delta().x);
                    let time_delta = -delta_x / self.zoom_level;
                    self.current_time = (self.current_time + time_delta).clamp(0.0, self.current_track.duration);
                } else if wave_res.clicked() {
                    if let Some(pos) = wave_res.interact_pointer_pos() {
                        let center_x = wave_rect.center().x;
                        let offset_x = pos.x - center_x;
                        let target_time = self.current_time + (offset_x / self.zoom_level);
                        self.current_time = target_time.clamp(0.0, self.current_track.duration);
                    }
                }

                let painter = ui.painter_at(wave_rect);

                // Container
                painter.rect_filled(wave_rect, 10.0, Color32::from_rgb(248, 250, 252));
                painter.rect_stroke(wave_rect, 10.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);

                let base_y = wave_rect.max.y - 10.0;
                painter.line_segment(
                    [Pos2::new(wave_rect.min.x, base_y), Pos2::new(wave_rect.max.x, base_y)],
                    Stroke::new(1.0, Color32::from_rgb(226, 232, 240)),
                );

                // Smooth Translucent Multi-Band Waveform
                let center_x = wave_rect.center().x;
                let samples = &self.current_track.waveform;
                let bass_samples = &self.current_track.waveform_bass;
                let mid_samples = &self.current_track.waveform_mid;
                let treble_samples = &self.current_track.waveform_treble;
                let total_samples = samples.len() as f32;
                let duration = self.current_track.duration;
                let _half_visible_sec = (wave_width / 2.0) / self.zoom_level;
                let max_wave_height = wave_height - 24.0;

                if duration > 0.0 && !samples.is_empty() {
                    let px_step = 2.0;
                    let count = (wave_width / px_step) as usize;

                    for i in 0..count {
                        let screen_x = wave_rect.min.x + (i as f32 * px_step);
                        let time_at_x = self.current_time + ((screen_x - center_x) / self.zoom_level);

                        if time_at_x >= 0.0 && time_at_x <= duration {
                            let sample_idx = (((time_at_x / duration) * total_samples) as usize).min(samples.len() - 1);
                            let amp = samples[sample_idx];
                            let b_amp = bass_samples.get(sample_idx).copied().unwrap_or(amp);
                            let m_amp = mid_samples.get(sample_idx).copied().unwrap_or(amp);
                            let tr_amp = treble_samples.get(sample_idx).copied().unwrap_or(amp);

                            let is_played = time_at_x <= self.current_time;

                            match self.waveform_mode {
                                WaveformMode::StackedMultiBand => {
                                    // 3 Layer Bertumpuk (Bass -> Mid -> Treble)
                                    // Bass (Foundation / Lows <250Hz in Magenta #D946EF)
                                    let h_bass = (b_amp * max_wave_height * 0.96).max(1.0);
                                    let bass_top = base_y - h_bass;
                                    let (bass_fill, bass_stroke) = if is_played {
                                        (
                                            Color32::from_rgba_unmultiplied(217, 70, 239, 45),
                                            Color32::from_rgb(217, 70, 239),
                                        )
                                    } else {
                                        (
                                            Color32::from_rgba_unmultiplied(217, 70, 239, 14),
                                            Color32::from_rgba_unmultiplied(217, 70, 239, 90),
                                        )
                                    };
                                    painter.line_segment([Pos2::new(screen_x, bass_top), Pos2::new(screen_x, base_y)], Stroke::new(px_step, bass_fill));
                                    painter.line_segment([Pos2::new(screen_x, bass_top), Pos2::new(screen_x, bass_top + 1.8)], Stroke::new(px_step, bass_stroke));

                                    // Mid (Vocals / Harmonics 250Hz-4kHz in Cyan #0EA5E9)
                                    let h_mid = (m_amp * max_wave_height * 0.72).max(1.0);
                                    let mid_top = base_y - h_mid;
                                    let (mid_fill, mid_stroke) = if is_played {
                                        (
                                            Color32::from_rgba_unmultiplied(14, 165, 233, 55),
                                            Color32::from_rgb(14, 165, 233),
                                        )
                                    } else {
                                        (
                                            Color32::from_rgba_unmultiplied(14, 165, 233, 18),
                                            Color32::from_rgba_unmultiplied(14, 165, 233, 100),
                                        )
                                    };
                                    painter.line_segment([Pos2::new(screen_x, mid_top), Pos2::new(screen_x, base_y)], Stroke::new(px_step, mid_fill));
                                    painter.line_segment([Pos2::new(screen_x, mid_top), Pos2::new(screen_x, mid_top + 1.8)], Stroke::new(px_step, mid_stroke));

                                    // Treble (Hi-hats / Air >4kHz in Amber #F59E0B)
                                    let h_treble = (tr_amp * max_wave_height * 0.48).max(1.0);
                                    let treble_top = base_y - h_treble;
                                    let (treble_fill, treble_stroke) = if is_played {
                                        (
                                            Color32::from_rgba_unmultiplied(245, 158, 11, 70),
                                            Color32::from_rgb(245, 158, 11),
                                        )
                                    } else {
                                        (
                                            Color32::from_rgba_unmultiplied(245, 158, 11, 22),
                                            Color32::from_rgba_unmultiplied(245, 158, 11, 110),
                                        )
                                    };
                                    painter.line_segment([Pos2::new(screen_x, treble_top), Pos2::new(screen_x, base_y)], Stroke::new(px_step, treble_fill));
                                    painter.line_segment([Pos2::new(screen_x, treble_top), Pos2::new(screen_x, treble_top + 1.8)], Stroke::new(px_step, treble_stroke));
                                }
                                WaveformMode::SpectralBlend => {
                                    // Spectral Color Blending (Serato/Rekordbox Style)
                                    let sum = (b_amp + m_amp + tr_amp).max(0.001);
                                    let w_b = b_amp / sum;
                                    let w_m = m_amp / sum;
                                    let w_tr = tr_amp / sum;

                                    let r = ((w_b * 217.0 + w_m * 14.0 + w_tr * 245.0).clamp(0.0, 255.0)) as u8;
                                    let g = ((w_b * 70.0 + w_m * 165.0 + w_tr * 158.0).clamp(0.0, 255.0)) as u8;
                                    let b = ((w_b * 239.0 + w_m * 233.0 + w_tr * 11.0).clamp(0.0, 255.0)) as u8;

                                    let h = (amp * max_wave_height).max(1.0);
                                    let top_y = base_y - h;

                                    let (fill_col, top_stroke_col) = if is_played {
                                        (
                                            Color32::from_rgba_unmultiplied(r, g, b, 60),
                                            Color32::from_rgb(r, g, b),
                                        )
                                    } else {
                                        let gray = ((r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) * 0.6) as u8;
                                        (
                                            Color32::from_rgba_unmultiplied(r, g, b, 20),
                                            Color32::from_rgba_unmultiplied(r / 2 + gray / 2, g / 2 + gray / 2, b / 2 + gray / 2, 120),
                                        )
                                    };

                                    painter.line_segment([Pos2::new(screen_x, top_y), Pos2::new(screen_x, base_y)], Stroke::new(px_step, fill_col));
                                    painter.line_segment([Pos2::new(screen_x, top_y), Pos2::new(screen_x, top_y + 2.0)], Stroke::new(px_step, top_stroke_col));
                                }
                            }
                        }
                    }

                    // Center Playhead
                    painter.line_segment(
                        [Pos2::new(center_x, wave_rect.min.y + 6.0), Pos2::new(center_x, base_y)],
                        Stroke::new(2.0, Color32::from_rgb(15, 23, 42)),
                    );
                    painter.circle_filled(Pos2::new(center_x, wave_rect.min.y + 8.0), 4.0, Color32::from_rgb(14, 165, 233));
                } else {
                    painter.text(
                        wave_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Lunula Engine Siap • Buka Tab Library untuk Memindai Folder / File Musik",
                        egui::FontId::proportional(14.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }

                // Minimap
                ui.add_space(4.0);
                let minimap_h = 6.0;
                let (mini_rect, mini_res) = ui.allocate_exact_size(Vec2::new(wave_width, minimap_h), egui::Sense::click_and_drag());

                if duration > 0.0 {
                    if mini_res.clicked() || mini_res.dragged() {
                        if let Some(pos) = mini_res.interact_pointer_pos() {
                            let ratio = ((pos.x - mini_rect.min.x) / mini_rect.width()).clamp(0.0, 1.0);
                            self.current_time = ratio * duration;
                        }
                    }

                    let mini_painter = ui.painter_at(mini_rect);
                    mini_painter.rect_filled(mini_rect, 3.0, Color32::from_rgb(241, 245, 249));

                    let progress_ratio = (self.current_time / duration).clamp(0.0, 1.0);
                    let played_width = (mini_rect.width() * progress_ratio).max(0.0);
                    if played_width > 0.0 {
                        let played_rect = Rect::from_min_size(
                            mini_rect.min,
                            Vec2::new(played_width, mini_rect.height()),
                        );
                        mini_painter.rect_filled(played_rect, 3.0, Color32::from_rgb(14, 165, 233));
                    }

                    // Sleek thumb indicator on hover/drag
                    let thumb_x = mini_rect.min.x + played_width;
                    let thumb_center = Pos2::new(thumb_x, mini_rect.center().y);
                    if mini_res.hovered() || mini_res.dragged() {
                        mini_painter.circle_filled(thumb_center, 5.0, Color32::from_rgb(14, 165, 233));
                        mini_painter.circle_stroke(thumb_center, 5.0, Stroke::new(1.8, Color32::WHITE));
                    } else if played_width > 0.0 && played_width < mini_rect.width() {
                        mini_painter.circle_filled(thumb_center, 3.0, Color32::from_rgb(14, 165, 233));
                    }
                } else {
                    let mini_painter = ui.painter_at(mini_rect);
                    mini_painter.rect_filled(mini_rect, 3.0, Color32::from_rgb(241, 245, 249));
                }

                // Mode Switcher & Frequency Legend Toolbar
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("TAMPILAN WAVE:")
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(148, 163, 184)),
                    );

                    let is_stacked = self.waveform_mode == WaveformMode::StackedMultiBand;
                    let is_spectral = self.waveform_mode == WaveformMode::SpectralBlend;

                    let btn_stacked = egui::Button::new(
                        egui::RichText::new("📊 3-Band Layers")
                            .size(11.5)
                            .strong()
                            .color(if is_stacked { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(100, 116, 139) })
                    )
                    .fill(if is_stacked { Color32::from_rgb(224, 231, 255) } else { Color32::from_rgb(248, 250, 252) })
                    .corner_radius(6.0)
                    .stroke(if is_stacked { Stroke::new(1.2, Color32::from_rgb(99, 102, 241)) } else { Stroke::new(1.0, Color32::from_rgb(226, 232, 240)) });

                    if ui.add(btn_stacked).on_hover_text("Mode A: 3 kurva gelombang bertumpuk (Bass, Mid, Treble)").clicked() {
                        self.waveform_mode = WaveformMode::StackedMultiBand;
                        self.save_library_state();
                    }

                    let btn_spectral = egui::Button::new(
                        egui::RichText::new("🌈 Spectral Blend")
                            .size(11.5)
                            .strong()
                            .color(if is_spectral { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(100, 116, 139) })
                    )
                    .fill(if is_spectral { Color32::from_rgb(254, 243, 199) } else { Color32::from_rgb(248, 250, 252) })
                    .corner_radius(6.0)
                    .stroke(if is_spectral { Stroke::new(1.2, Color32::from_rgb(245, 158, 11)) } else { Stroke::new(1.0, Color32::from_rgb(226, 232, 240)) });

                    if ui.add(btn_spectral).on_hover_text("Mode B: Gradien warna spektral dinamis berdasarkan frekuensi (Serato/Rekordbox)").clicked() {
                        self.waveform_mode = WaveformMode::SpectralBlend;
                        self.save_library_state();
                    }

                    // Right-aligned Frequency Legend
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Treble badge
                        ui.label(
                            egui::RichText::new("Treble (>4kHz)")
                                .size(11.0)
                                .strong()
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                        ui.label(egui::RichText::new("●").size(15.0).color(Color32::from_rgb(245, 158, 11)));

                        ui.add_space(10.0);

                        // Mid badge
                        ui.label(
                            egui::RichText::new("Mid (250Hz-4kHz)")
                                .size(11.0)
                                .strong()
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                        ui.label(egui::RichText::new("●").size(15.0).color(Color32::from_rgb(14, 165, 233)));

                        ui.add_space(10.0);

                        // Bass badge
                        ui.label(
                            egui::RichText::new("Bass (<250Hz)")
                                .size(11.0)
                                .strong()
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                        ui.label(egui::RichText::new("●").size(15.0).color(Color32::from_rgb(217, 70, 239)));

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("SPECTRUM:")
                                .size(11.0)
                                .strong()
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                    });
                });

                // Centered Transport Deck
                ui.add_space(12.0);
                self.render_balanced_transport_deck(ui);
            });
        });
    }

    // Perfectly Centered 3-Column Transport Deck with Smooth Animated Hover
    pub fn render_balanced_transport_deck(&mut self, ui: &mut egui::Ui) {
        let full_width = ui.available_width();
        let left_width = 180.0;
        let right_width = 280.0;
        let center_width = (full_width - left_width - right_width).max(200.0);

        ui.horizontal(|ui| {
            // LEFT COLUMN (180px): Confident Time Typography
            ui.allocate_ui_with_layout(
                Vec2::new(left_width, 44.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(
                        egui::RichText::new(format_time(self.current_time))
                            .strong()
                            .size(17.0)
                            .color(Color32::from_rgb(15, 23, 42)),
                    );

                    let remaining = (self.current_track.duration - self.current_time).max(0.0);
                    ui.label(
                        egui::RichText::new(format!(" / -{}", format_time(remaining)))
                            .strong()
                            .size(13.5)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                },
            );

            // CENTER COLUMN: Perfectly Centered Transport Buttons with Silky Smooth Animated Hover
            ui.allocate_ui_with_layout(
                Vec2::new(center_width, 44.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.horizontal(|ui| {
                        self.render_icon_btn(ui, "btn_shuffle", "🔀", 15.0, "Shuffle", |_app| {});
                        ui.add_space(8.0);

                        self.render_icon_btn(ui, "btn_prev", "⏮", 18.0, "Previous", |app| {
                            app.play_prev_track();
                        });
                        ui.add_space(12.0);

                        // Floating Play/Pause Button
                        let play_size = Vec2::splat(44.0);
                        let (play_rect, play_res) = ui.allocate_exact_size(play_size, egui::Sense::click());
                        let play_hover = ui.ctx().animate_bool(ui.scope_id().with("play_pause_hover"), play_res.hovered());

                        if play_res.clicked() {
                            self.is_playing = !self.is_playing;
                        }

                        let painter = ui.painter_at(play_rect);
                        let center = play_rect.center();

                        let shadow_radius = 21.0 + play_hover * 3.0;
                        let shadow_alpha = (22.0 + play_hover * 25.0) as u8;
                        painter.circle_filled(center + Vec2::new(0.0, 2.5), shadow_radius, Color32::from_black_alpha(shadow_alpha));

                        let base_col = Color32::from_rgb(15, 23, 42);
                        let hover_col = Color32::from_rgb(2, 132, 199);
                        let circle_col = lerp_color(base_col, hover_col, play_hover);
                        painter.circle_filled(center, 21.0, circle_col);

                        if self.is_playing {
                            let bar_w = 3.5;
                            let bar_h = 13.0;
                            let gap = 3.0;
                            painter.rect_filled(
                                Rect::from_center_size(center - Vec2::new(gap + bar_w * 0.5, 0.0), Vec2::new(bar_w, bar_h)),
                                1.5,
                                Color32::WHITE,
                            );
                            painter.rect_filled(
                                Rect::from_center_size(center + Vec2::new(gap + bar_w * 0.5, 0.0), Vec2::new(bar_w, bar_h)),
                                1.5,
                                Color32::WHITE,
                            );
                        } else {
                            let r = 8.0;
                            let p1 = center + Vec2::new(-r * 0.6, -r);
                            let p2 = center + Vec2::new(-r * 0.6, r);
                            let p3 = center + Vec2::new(r * 0.9, 0.0);
                            painter.add(egui::epaint::PathShape::convex_polygon(
                                vec![p1, p2, p3],
                                Color32::WHITE,
                                Stroke::NONE,
                            ));
                        }

                        ui.add_space(12.0);

                        self.render_icon_btn(ui, "btn_next", "⏭", 18.0, "Next", |app| {
                            app.play_next_track();
                        });
                        ui.add_space(8.0);

                        self.render_icon_btn(ui, "btn_repeat", "🔁", 15.0, "Repeat", |_app| {});
                    });
                },
            );

            // RIGHT COLUMN (280px): Sleek Modern Volume Slider & Queue Drawer Toggle
            ui.allocate_ui_with_layout(
                Vec2::new(right_width, 44.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    // Queue Drawer Toggle Button
                    let q_count = self.play_queue.len();
                    let q_text = if q_count > 0 {
                        format!("📑 Antrean ({})", q_count)
                    } else {
                        "📑 Antrean".to_string()
                    };
                    let q_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(q_text)
                                .strong()
                                .size(11.5)
                                .color(if self.show_queue_drawer {
                                    Color32::from_rgb(2, 132, 199)
                                } else {
                                    Color32::from_rgb(71, 85, 105)
                                }),
                        )
                        .fill(if self.show_queue_drawer {
                            Color32::from_rgb(224, 242, 254)
                        } else {
                            Color32::from_rgb(241, 245, 249)
                        })
                        .corner_radius(6.0)
                        .min_size(Vec2::new(32.0, 26.0)),
                    ).on_hover_text("Buka / Tutup Panel Antrean Lagu (Queue)");

                    if q_btn.clicked() {
                        self.show_queue_drawer = !self.show_queue_drawer;
                    }

                    ui.add_space(8.0);

                    let vol_pct = (if self.is_muted { 0.0 } else { self.volume } * 100.0) as u32;
                    ui.label(
                        egui::RichText::new(format!("{:3}%", vol_pct))
                            .strong()
                            .size(13.0)
                            .color(Color32::from_rgb(71, 85, 105)),
                    );

                    ui.add_space(8.0);

                    let vol_size = Vec2::new(100.0, 10.0);
                    let (vol_rect, vol_res) = ui.allocate_exact_size(vol_size, egui::Sense::click_and_drag());
                    let vol_hover = ui.ctx().animate_bool(ui.scope_id().with("vol_hover_anim"), vol_res.hovered() || vol_res.dragged());

                    if vol_res.clicked() || vol_res.dragged() {
                        if let Some(pos) = vol_res.interact_pointer_pos() {
                            let ratio = ((pos.x - vol_rect.min.x) / vol_rect.width()).clamp(0.0, 1.0);
                            self.volume = ratio;
                            self.is_muted = false;
                        }
                    }

                    let vol_painter = ui.painter_at(vol_rect);
                    vol_painter.rect_filled(vol_rect, 5.0, Color32::from_rgb(226, 232, 240));

                    let cur_vol = if self.is_muted { 0.0 } else { self.volume };
                    if cur_vol > 0.0 {
                        let fill_w = vol_rect.width() * cur_vol;
                        let fill_rect = Rect::from_min_size(vol_rect.min, Vec2::new(fill_w, vol_rect.height()));
                        
                        let fill_col = lerp_color(Color32::from_rgb(15, 23, 42), Color32::from_rgb(2, 132, 199), vol_hover);
                        vol_painter.rect_filled(fill_rect, 5.0, fill_col);

                        let thumb_radius = 5.0 + vol_hover * 1.5;
                        vol_painter.circle_filled(
                            Pos2::new(vol_rect.min.x + fill_w, vol_rect.center().y),
                            thumb_radius,
                            fill_col,
                        );
                    }

                    ui.add_space(4.0);

                    let vol_icon = if self.is_muted || self.volume == 0.0 {
                        "🔇"
                    } else if self.volume < 0.5 {
                        "🔉"
                    } else {
                        "🔊"
                    };

                    let speaker_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(vol_icon).strong().size(15.0).color(Color32::from_rgb(15, 23, 42)),
                        )
                        .frame(false),
                    );

                    if speaker_btn.clicked() {
                        self.is_muted = !self.is_muted;
                    }
                },
            );
        });
    }

    pub fn render_icon_btn<F>(&mut self, ui: &mut egui::Ui, id_str: &str, icon: &str, font_size: f32, tooltip: &str, mut on_click: F)
    where
        F: FnMut(&mut Self),
    {
        let size = Vec2::splat(36.0);
        let (rect, res) = ui.allocate_exact_size(size, egui::Sense::click());
        let hover_anim = ui.ctx().animate_bool(ui.scope_id().with(id_str), res.hovered());

        if res.clicked() {
            on_click(self);
        }

        let painter = ui.painter_at(rect);
        let center = rect.center();

        if hover_anim > 0.0 {
            let circle_color = Color32::from_rgba_unmultiplied(241, 245, 249, (hover_anim * 255.0) as u8);
            painter.circle_filled(center, 17.0, circle_color);
        }

        let icon_color = lerp_color(Color32::from_rgb(100, 116, 139), Color32::from_rgb(15, 23, 42), hover_anim);

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(font_size),
            icon_color,
        );

        res.on_hover_text(tooltip);
    }
}

