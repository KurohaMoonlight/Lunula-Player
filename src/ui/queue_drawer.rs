use eframe::egui;
use egui::{Color32, Stroke, Vec2};
use crate::models::Track;
use crate::app::LunulaApp;
use crate::ui::components::{draw_artist_avatar, open_file_in_explorer};

impl LunulaApp {
    pub fn render_queue_drawer(&mut self, ui: &mut egui::Ui) {
        let mut track_to_play: Option<Track> = None;
        let mut remove_queue_idx: Option<usize> = None;
        let mut move_up_idx: Option<usize> = None;
        let mut move_down_idx: Option<usize> = None;
        let mut clear_queue = false;

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📑 Antrean Putar").strong().size(15.0).color(Color32::from_rgb(15, 23, 42)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("✕").size(13.0).strong()).on_hover_text("Tutup Panel Antrean").clicked() {
                        self.show_queue_drawer = false;
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            // Now Playing
            ui.label(egui::RichText::new("SEDANG DIPUTAR").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
            ui.add_space(3.0);

            egui::Frame::new()
                .fill(Color32::from_rgb(248, 250, 252))
                .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (thumb_rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
                        let painter = ui.painter_at(thumb_rect);
                        draw_artist_avatar(&painter, thumb_rect.center(), 18.0, &self.current_track.artist);

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

            ui.add_space(10.0);

            // Up Next
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("BERIKUTNYA ({})", self.play_queue.len())).strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, song) in self.play_queue.iter().enumerate() {
                        let row_resp = egui::Frame::new()
                            .fill(Color32::WHITE)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("{:02}", idx + 1)).size(11.0).color(Color32::from_rgb(100, 116, 139)));

                                    ui.vertical(|ui| {
                                        ui.set_max_width(160.0);
                                        ui.add(egui::Label::new(egui::RichText::new(&song.title).strong().size(12.5).color(Color32::from_rgb(15, 23, 42))).truncate());
                                        ui.add(egui::Label::new(egui::RichText::new(&song.artist).size(11.0).color(Color32::from_rgb(71, 85, 105))).truncate());
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(egui::RichText::new("✕").size(10.0)).on_hover_text("Hapus dari Antrean").clicked() {
                                            remove_queue_idx = Some(idx);
                                        }
                                        if idx > 0 {
                                            if ui.button(egui::RichText::new("▲").size(9.0)).on_hover_text("Geser ke Atas").clicked() {
                                                move_up_idx = Some(idx);
                                            }
                                        }
                                        if idx + 1 < self.play_queue.len() {
                                            if ui.button(egui::RichText::new("▼").size(9.0)).on_hover_text("Geser ke Bawah").clicked() {
                                                move_down_idx = Some(idx);
                                            }
                                        }
                                        if ui.button(egui::RichText::new("▶").size(10.0)).on_hover_text("Putar Sekarang").clicked() {
                                            track_to_play = Some(song.clone());
                                            remove_queue_idx = Some(idx);
                                        }
                                    });
                                });
                            });

                        let interact = ui.interact(row_resp.response.rect, ui.scope_id().with(("q_row", idx)), egui::Sense::click());
                        if interact.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if interact.double_clicked() {
                            track_to_play = Some(song.clone());
                            remove_queue_idx = Some(idx);
                        }
                        interact.context_menu(|ui| {
                            ui.label(egui::RichText::new(&song.title).strong().size(12.0));
                            ui.separator();
                            if ui.button("▶ Putar Sekarang").clicked() {
                                track_to_play = Some(song.clone());
                                remove_queue_idx = Some(idx);
                                ui.close();
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

                        ui.add_space(2.0);
                    }
                });
            }
        });

        if clear_queue {
            self.play_queue.clear();
            self.save_library_state();
        }
        if let Some(idx) = move_up_idx {
            if idx > 0 && idx < self.play_queue.len() {
                self.play_queue.swap(idx, idx - 1);
                self.save_library_state();
            }
        }
        if let Some(idx) = move_down_idx {
            if idx + 1 < self.play_queue.len() {
                self.play_queue.swap(idx, idx + 1);
                self.save_library_state();
            }
        }
        if let Some(idx) = remove_queue_idx {
            if idx < self.play_queue.len() {
                self.play_queue.remove(idx);
                self.save_library_state();
            }
        }
        if let Some(track) = track_to_play {
            self.play_track(track);
        }
    }
}

