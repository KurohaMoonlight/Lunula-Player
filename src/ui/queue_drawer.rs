use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{RepeatMode, Track};
use crate::app::LunulaApp;
use crate::ui::components::{draw_artist_avatar, lerp_color, open_file_in_explorer};

impl LunulaApp {
    pub fn render_queue_drawer(&mut self, ui: &mut egui::Ui) {
        let mut track_to_play: Option<Track> = None;
        let mut remove_queue_idx: Option<usize> = None;
        let mut move_up_idx: Option<usize> = None;
        let mut move_down_idx: Option<usize> = None;
        let mut clear_queue = false;
        let mut shuffle_queue_now = false;
        let mut set_selected_idx: Option<usize> = None;
        let mut set_queue_cursor: Option<usize> = None;

        ui.vertical(|ui| {
            // 1. Header Toolbar
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📑 Antrean Putar").strong().size(15.0).color(Color32::from_rgb(15, 23, 42)));
                let q_count_txt = format!("• {} Lagu", self.play_queue.len());
                ui.label(egui::RichText::new(q_count_txt).size(12.0).color(Color32::from_rgb(100, 116, 139)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("✕").size(13.0).strong()).on_hover_text("Tutup Panel Antrean").clicked() {
                        self.show_queue_drawer = false;
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // 2. Playback & Repeat Mode Options Box with animated hover
            let opt_card_id = ui.scope_id().with("queue_options_card");
            let opt_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 62.0));
            let opt_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| opt_rect.contains(p)));
            let opt_hover_t = ui.ctx().animate_bool(opt_card_id, opt_hovered);
            let opt_bg = lerp_color(Color32::from_rgb(248, 250, 252), Color32::from_rgb(255, 255, 255), opt_hover_t);
            let opt_border = lerp_color(Color32::from_rgb(226, 232, 240), Color32::from_rgb(199, 210, 254), opt_hover_t);

            egui::Frame::new()
                .fill(opt_bg)
                .stroke(Stroke::new(1.0 + opt_hover_t * 0.4, opt_border))
                .corner_radius(8.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Repeat options
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🔁 Repeat:").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));

                            let repeat_modes = [
                                (RepeatMode::Off, "✕ Mati", "Tidak mengulang (berhenti setelah selesai)"),
                                (RepeatMode::All, "🔁 Semua", "Mengulang seluruh playlist & antrean"),
                                (RepeatMode::One, "🔂 Lagu", "Mengulang lagu yang sedang diputar"),
                                (RepeatMode::Album, "💿 Album Ini", "Mengulang hanya lagu di album yang sama"),
                            ];

                            for (mode, label, tip) in repeat_modes {
                                let is_active = self.repeat_mode == mode;
                                let btn = if is_active {
                                    egui::Button::new(egui::RichText::new(label).strong().size(10.5).color(Color32::WHITE))
                                        .fill(Color32::from_rgb(2, 132, 199))
                                        .corner_radius(4.0)
                                } else {
                                    egui::Button::new(egui::RichText::new(label).size(10.5).color(Color32::from_rgb(71, 85, 105)))
                                        .fill(Color32::WHITE)
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                        .corner_radius(4.0)
                                };
                                if ui.add(btn).on_hover_text(tip).clicked() {
                                    self.set_repeat_mode(mode);
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // Shuffle & Queue Tools
                        ui.horizontal(|ui| {
                            // Shuffle Mode Toggle
                            let shuf_label = if self.is_shuffled { "🔀 Acak: Aktif" } else { "🔀 Acak: Mati" };
                            let shuf_btn = if self.is_shuffled {
                                egui::Button::new(egui::RichText::new(shuf_label).strong().size(10.5).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(14, 165, 233))
                                    .corner_radius(4.0)
                            } else {
                                egui::Button::new(egui::RichText::new(shuf_label).size(10.5).color(Color32::from_rgb(71, 85, 105)))
                                    .fill(Color32::WHITE)
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                    .corner_radius(4.0)
                            };
                            if ui.add(shuf_btn).on_hover_text("Nyalakan / Matikan pemutaran acak").clicked() {
                                self.toggle_shuffle();
                            }

                            if !self.play_queue.is_empty() {
                                if ui.button(egui::RichText::new("🎲 Acak Antrean").size(10.5)).on_hover_text("Kocok urutan antrean saat ini").clicked() {
                                    shuffle_queue_now = true;
                                }
                            }

                            // Selected item Move Up/Down shortcuts
                            if let Some(sel) = self.selected_queue_idx {
                                if sel < self.play_queue.len() {
                                    ui.add_enabled_ui(sel > 0, |ui| {
                                        if ui.button(egui::RichText::new("⬆ Naik").size(10.0)).on_hover_text("Pindahkan lagu terpilih ke atas").clicked() {
                                            move_up_idx = Some(sel);
                                        }
                                    });
                                    ui.add_enabled_ui(sel + 1 < self.play_queue.len(), |ui| {
                                        if ui.button(egui::RichText::new("⬇ Turun").size(10.0)).on_hover_text("Pindahkan lagu terpilih ke bawah").clicked() {
                                            move_down_idx = Some(sel);
                                        }
                                    });
                                }
                            }
                        });

                        // Album Repeat Info Note
                        if self.repeat_mode == RepeatMode::Album {
                            ui.add_space(2.0);
                            let alb_name = if self.current_track.album.trim().is_empty() {
                                "Single".to_string()
                            } else {
                                self.current_track.album.clone()
                            };
                            ui.label(
                                egui::RichText::new(format!("💿 Mengulang khusus album: \"{}\"", alb_name))
                                    .size(10.5)
                                    .color(Color32::from_rgb(2, 132, 199))
                                    .strong(),
                            );
                        }
                    });
                });

            ui.add_space(6.0);

            // 3. Now Playing Card with smooth animated hover
            ui.label(egui::RichText::new("SEDANG DIPUTAR").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
            ui.add_space(3.0);

            let np_card_id = ui.scope_id().with("queue_now_playing_card");
            let np_card_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 56.0));
            let np_is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| np_card_rect.contains(p)));
            let np_hover_t = ui.ctx().animate_bool(np_card_id, np_is_hovered);
            let np_bg = lerp_color(Color32::from_rgb(248, 250, 252), Color32::from_rgb(241, 245, 249), np_hover_t);
            let np_border = lerp_color(Color32::from_rgb(226, 232, 240), Color32::from_rgb(16, 185, 129), np_hover_t);

            egui::Frame::new()
                .fill(np_bg)
                .stroke(Stroke::new(1.0 + np_hover_t * 0.5, np_border))
                .corner_radius(8.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (thumb_rect, _) = ui.allocate_exact_size(Vec2::splat(38.0), egui::Sense::hover());
                        let painter = ui.painter_at(thumb_rect);

                        // Use actual cover art if available
                        let cover_key = if !self.current_track.album.trim().is_empty() {
                            &self.current_track.album
                        } else {
                            self.current_track.file_path.as_deref().unwrap_or(&self.current_track.title)
                        };
                        let cover_tex = self.get_album_cover_texture(ui.ctx(), cover_key, self.current_track.file_path.as_deref());
                        if let Some(ref tex) = cover_tex {
                            painter.rect_filled(thumb_rect, 6.0, Color32::from_rgb(15, 23, 42));
                            painter.image(
                                tex.id(),
                                thumb_rect,
                                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            painter.rect_stroke(thumb_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);
                        } else {
                            draw_artist_avatar(&painter, thumb_rect.center(), 19.0, &self.current_track.artist);
                        }

                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.set_max_width(220.0);
                            let title_color = if self.is_playing { Color32::from_rgb(16, 185, 129) } else { Color32::from_rgb(15, 23, 42) };
                            ui.add(egui::Label::new(egui::RichText::new(&self.current_track.title).strong().size(13.0).color(title_color)).truncate());
                            ui.add(egui::Label::new(egui::RichText::new(format!("{} • {}", self.current_track.artist, self.current_track.album)).size(11.0).color(Color32::from_rgb(71, 85, 105))).truncate());

                            let is_hi_res = self.current_track.sample_rate >= 88200 || self.current_track.bit_depth >= 24;
                            let rate_khz = self.current_track.sample_rate as f32 / 1000.0;
                            let badge_txt = if is_hi_res {
                                format!("✨ {} {}b/{:.1}k", self.current_track.format, self.current_track.bit_depth, rate_khz)
                            } else {
                                format!("💿 {} {:.1}k", self.current_track.format, rate_khz)
                            };
                            ui.label(egui::RichText::new(badge_txt).size(10.5).color(Color32::from_rgb(71, 85, 105)));
                        });
                    });
                });

            ui.add_space(8.0);

            // 4. Up Next Queue Section
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("DAFTAR ANTREAN ({})", self.play_queue.len())).strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                if !self.play_queue.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("🗑️ Kosongkan").size(10.5)).on_hover_text("Hapus semua lagu dalam antrean").clicked() {
                            clear_queue = true;
                        }
                    });
                }
            });

            ui.add_space(4.0);

            if self.play_queue.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("⏳").size(32.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Antrean Sedang Kosong").strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Klik kanan pada lagu di Library lalu pilih 'Putar Berikutnya' atau 'Tambah ke Antrean'.").size(11.5).color(Color32::from_rgb(71, 85, 105)));
                });
            } else {
                ui.label(egui::RichText::new("💡 Tips: Klik lagu untuk memilih • Gunakan tombol ▲ ▼ atau tombol panah keyboard ↑ ↓ untuk geser").size(10.0).color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(2.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, song) in self.play_queue.iter().enumerate() {
                        let is_selected = self.selected_queue_idx == Some(idx);
                        let is_active_track = self.queue_cursor == Some(idx);

                        let q_card_id = ui.scope_id().with(("q_item_card", idx));
                        let card_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 38.0));
                        let is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| card_rect.contains(p)));
                        let hover_t = ui.ctx().animate_bool(q_card_id, is_hovered);

                        let base_bg = if is_selected {
                            Color32::from_rgb(238, 242, 255)
                        } else if is_active_track {
                            Color32::from_rgb(236, 253, 245)
                        } else if idx % 2 == 1 {
                            Color32::from_rgb(248, 250, 252)
                        } else {
                            Color32::WHITE
                        };

                        let hover_target_bg = if is_selected {
                            Color32::from_rgb(224, 231, 255)
                        } else if is_active_track {
                            Color32::from_rgb(209, 250, 229)
                        } else {
                            Color32::from_rgb(241, 245, 249)
                        };

                        let bg_color = lerp_color(base_bg, hover_target_bg, hover_t);

                        let base_border = if is_selected {
                            Color32::from_rgb(129, 140, 248)
                        } else if is_active_track {
                            Color32::from_rgb(16, 185, 129)
                        } else {
                            Color32::from_rgb(226, 232, 240)
                        };

                        let hover_target_border = if is_selected {
                            Color32::from_rgb(79, 70, 229)
                        } else if is_active_track {
                            Color32::from_rgb(5, 150, 105)
                        } else {
                            Color32::from_rgb(148, 163, 184)
                        };

                        let border_color = lerp_color(base_border, hover_target_border, hover_t);
                        let stroke = Stroke::new(
                            if is_selected || is_active_track { 1.2 + hover_t * 0.4 } else { 1.0 + hover_t * 0.5 },
                            border_color,
                        );

                        egui::Frame::new()
                            .fill(bg_color)
                            .stroke(stroke)
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(8, 5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Row index or active playing icon
                                    ui.allocate_ui(Vec2::new(20.0, 28.0), |ui| {
                                        ui.centered_and_justified(|ui| {
                                            if is_active_track {
                                                if self.is_playing {
                                                    ui.colored_label(Color32::from_rgb(5, 150, 105), "▶");
                                                } else {
                                                    ui.colored_label(Color32::from_rgb(16, 185, 129), "⏸");
                                                }
                                            } else {
                                                ui.label(egui::RichText::new(format!("{:02}", idx + 1)).size(11.0).color(Color32::from_rgb(100, 116, 139)));
                                            }
                                        });
                                    });

                                    // Cover Thumbnail (28x28)
                                    ui.allocate_ui(Vec2::new(30.0, 28.0), |ui| {
                                        let cover_key = if !song.album.trim().is_empty() {
                                            &song.album
                                        } else {
                                            song.file_path.as_deref().unwrap_or(&song.title)
                                        };
                                        let thumb_tex = self.get_album_cover_texture(ui.ctx(), cover_key, song.file_path.as_deref());
                                        let (thumb_rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::hover());
                                        let painter = ui.painter_at(thumb_rect);
                                        if let Some(ref tex) = thumb_tex {
                                            painter.rect_filled(thumb_rect, 3.0, Color32::from_rgb(15, 23, 42));
                                            painter.image(
                                                tex.id(),
                                                thumb_rect,
                                                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                                Color32::WHITE,
                                            );
                                        } else {
                                            painter.rect_filled(thumb_rect, 3.0, Color32::from_rgb(241, 245, 249));
                                            painter.circle_filled(thumb_rect.center(), 8.0, Color32::from_rgb(30, 41, 59));
                                            painter.circle_filled(thumb_rect.center(), 3.0, Color32::from_rgb(14, 165, 233));
                                        }
                                    });

                                    // Title & Artist (clickable to select or double-click to play without removing)
                                    let text_w = (ui.available_width() - 110.0).max(80.0);
                                    let (text_rect, text_res) = ui.allocate_exact_size(Vec2::new(text_w, 28.0), egui::Sense::click());
                                    if text_res.clicked() {
                                        set_selected_idx = Some(idx);
                                    }
                                    if text_res.double_clicked() {
                                        track_to_play = Some(song.clone());
                                        set_queue_cursor = Some(idx);
                                    }
                                    text_res.context_menu(|ui| {
                                        ui.label(egui::RichText::new(&song.title).strong().size(12.0));
                                        ui.separator();
                                        if ui.button("▶ Putar Sekarang").clicked() {
                                            track_to_play = Some(song.clone());
                                            set_queue_cursor = Some(idx);
                                            ui.close();
                                        }
                                        if idx > 0 {
                                            if ui.button("▲ Geser ke Atas").clicked() {
                                                move_up_idx = Some(idx);
                                                ui.close();
                                            }
                                        }
                                        if idx + 1 < self.play_queue.len() {
                                            if ui.button("▼ Geser ke Bawah").clicked() {
                                                move_down_idx = Some(idx);
                                                ui.close();
                                            }
                                        }
                                        if ui.button("✕ Hapus dari Antrean").clicked() {
                                            remove_queue_idx = Some(idx);
                                            ui.close();
                                        }
                                        if let Some(ref path) = song.file_path {
                                            if ui.button("📂 Buka Lokasi File").clicked() {
                                                open_file_in_explorer(path);
                                                ui.close();
                                            }
                                        }
                                    });

                                    let text_painter = ui.painter_at(text_rect);
                                    let title_color = if is_active_track {
                                        Color32::from_rgb(5, 150, 105)
                                    } else if is_selected {
                                        Color32::from_rgb(67, 56, 202)
                                    } else {
                                        Color32::from_rgb(15, 23, 42)
                                    };
                                    let title_font = egui::FontId::proportional(12.0);
                                    let artist_font = egui::FontId::proportional(10.5);

                                    text_painter.text(
                                        Pos2::new(text_rect.min.x, text_rect.min.y + 1.0),
                                        egui::Align2::LEFT_TOP,
                                        &song.title,
                                        title_font,
                                        title_color,
                                    );

                                    let (artist_txt, artist_col) = if is_active_track {
                                        (format!("{} • ▶ Sedang Diputar", song.artist), Color32::from_rgb(5, 150, 105))
                                    } else {
                                        (song.artist.clone(), Color32::from_rgb(100, 116, 139))
                                    };

                                    text_painter.text(
                                        Pos2::new(text_rect.min.x, text_rect.min.y + 15.0),
                                        egui::Align2::LEFT_TOP,
                                        &artist_txt,
                                        artist_font,
                                        artist_col,
                                    );

                                    // Action Buttons with FIXED POSITIONS (Never jumping!)
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Delete [ ✕ ]
                                        if ui.add_sized([22.0, 24.0], egui::Button::new(egui::RichText::new("✕").size(10.0))).on_hover_text("Hapus dari Antrean").clicked() {
                                            remove_queue_idx = Some(idx);
                                        }

                                        // Move Down [ ▼ ] (Fixed slot, disabled on last item)
                                        ui.add_enabled_ui(idx + 1 < self.play_queue.len(), |ui| {
                                            if ui.add_sized([22.0, 24.0], egui::Button::new(egui::RichText::new("▼").size(9.5))).on_hover_text("Geser Lagu ke Bawah (atau Ctrl+Down)").clicked() {
                                                move_down_idx = Some(idx);
                                            }
                                        });

                                        // Move Up [ ▲ ] (Fixed slot, disabled on first item)
                                        ui.add_enabled_ui(idx > 0, |ui| {
                                            if ui.add_sized([22.0, 24.0], egui::Button::new(egui::RichText::new("▲").size(9.5))).on_hover_text("Geser Lagu ke Atas (atau Ctrl+Up)").clicked() {
                                                move_up_idx = Some(idx);
                                            }
                                        });

                                        // Play Now [ ▶ ] (Plays and sets active queue cursor without deleting)
                                        if ui.add_sized([24.0, 24.0], egui::Button::new(egui::RichText::new("▶").size(10.5))).on_hover_text("Putar Sekarang").clicked() {
                                            track_to_play = Some(song.clone());
                                            set_queue_cursor = Some(idx);
                                        }
                                    });
                                });
                            });

                        ui.add_space(2.0);
                    }
                });
            }
        });

        // Execute deferred queue actions
        if clear_queue {
            self.play_queue.clear();
            self.selected_queue_idx = None;
            self.queue_cursor = None;
            self.save_library_state();
        }
        if shuffle_queue_now {
            self.shuffle_queue();
        }
        if let Some(idx) = move_up_idx {
            if idx > 0 && idx < self.play_queue.len() {
                self.play_queue.swap(idx, idx - 1);
                self.selected_queue_idx = Some(idx - 1);
                if self.queue_cursor == Some(idx) {
                    self.queue_cursor = Some(idx - 1);
                } else if self.queue_cursor == Some(idx - 1) {
                    self.queue_cursor = Some(idx);
                }
                self.save_library_state();
            }
        }
        if let Some(idx) = move_down_idx {
            if idx + 1 < self.play_queue.len() {
                self.play_queue.swap(idx, idx + 1);
                self.selected_queue_idx = Some(idx + 1);
                if self.queue_cursor == Some(idx) {
                    self.queue_cursor = Some(idx + 1);
                } else if self.queue_cursor == Some(idx + 1) {
                    self.queue_cursor = Some(idx);
                }
                self.save_library_state();
            }
        }
        if let Some(idx) = remove_queue_idx {
            if idx < self.play_queue.len() {
                self.play_queue.remove(idx);
                if self.play_queue.is_empty() {
                    self.selected_queue_idx = None;
                    self.queue_cursor = None;
                } else {
                    if let Some(sel) = self.selected_queue_idx {
                        if sel >= self.play_queue.len() {
                            self.selected_queue_idx = Some(self.play_queue.len() - 1);
                        }
                    }
                    if let Some(cur) = self.queue_cursor {
                        if idx < cur {
                            self.queue_cursor = Some(cur - 1);
                        } else if idx == cur {
                            self.queue_cursor = Some(idx.min(self.play_queue.len() - 1));
                        }
                    }
                }
                self.save_library_state();
            }
        }
        if let Some(idx) = set_selected_idx {
            self.selected_queue_idx = Some(idx);
        }
        if let Some(idx) = set_queue_cursor {
            self.queue_cursor = Some(idx);
            self.selected_queue_idx = Some(idx);
            self.save_library_state();
        }
        if let Some(track) = track_to_play {
            self.play_track(track);
        }
    }
}
