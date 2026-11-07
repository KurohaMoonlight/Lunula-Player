use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::path::Path;
use crate::app::LunulaApp;
use crate::models::Track;

fn render_clean_text_input(ui: &mut egui::Ui, text: &mut String, hint: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(248, 250, 252))
        .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(text)
                    .hint_text(hint)
                    .frame(egui::Frame::NONE)
                    .desired_width(ui.available_width()),
            );
        });
}

impl LunulaApp {
    pub fn open_tag_editor(&mut self, track: &Track) {
        self.show_tag_editor = true;
        self.tag_editor_path = track.file_path.clone();
        self.tag_editor_title = track.title.clone();
        self.tag_editor_artist = track.artist.clone();
        self.tag_editor_album = track.album.clone();
        self.tag_editor_year = String::new();
        self.tag_editor_genre = String::new();
        self.tag_editor_track_num = String::new();
        self.tag_editor_status = None;

        if let Some(ref path_str) = track.file_path {
            let p = Path::new(path_str);
            if let Ok(tagged_file) = lofty::read_from_path(p) {
                use lofty::prelude::*;
                if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                    if let Some(y) = tag.get_string(lofty::tag::ItemKey::Year) {
                        self.tag_editor_year = y.to_string();
                    }
                    if let Some(g) = tag.genre() {
                        self.tag_editor_genre = g.to_string();
                    }
                    if let Some(t) = tag.track() {
                        self.tag_editor_track_num = t.to_string();
                    }
                }
            }
        }
    }

    pub fn smart_extract_tags_from_filename(&mut self) {
        if let Some(ref path_str) = self.tag_editor_path {
            let p = Path::new(path_str);
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                if let Some((artist, title)) = stem.split_once(" - ") {
                    self.tag_editor_artist = artist.trim().to_string();
                    self.tag_editor_title = title.trim().to_string();
                } else {
                    self.tag_editor_title = stem.trim().to_string();
                }
            }
        }
    }

    pub fn save_tag_editor(&mut self) {
        let Some(ref path_str) = self.tag_editor_path.clone() else {
            self.tag_editor_status = Some(("Tidak ada jalur berkas audio yang valid".to_string(), false));
            return;
        };

        let path = Path::new(&path_str);
        let year_opt = self.tag_editor_year.trim().parse::<u32>().ok();
        let track_num_opt = self.tag_editor_track_num.trim().parse::<u32>().ok();
        let genre_opt = if self.tag_editor_genre.trim().is_empty() {
            None
        } else {
            Some(self.tag_editor_genre.trim())
        };

        match crate::scanner::save_track_tags(
            path,
            self.tag_editor_title.trim(),
            self.tag_editor_artist.trim(),
            self.tag_editor_album.trim(),
            year_opt,
            genre_opt,
            track_num_opt,
        ) {
            Ok(()) => {
                let new_title = self.tag_editor_title.trim().to_string();
                let new_artist = self.tag_editor_artist.trim().to_string();
                let new_album = self.tag_editor_album.trim().to_string();

                for t in &mut self.playlist {
                    if t.file_path.as_deref() == Some(&path_str) {
                        t.title = new_title.clone();
                        t.artist = new_artist.clone();
                        t.album = new_album.clone();
                    }
                }

                if self.current_track.file_path.as_deref() == Some(&path_str) {
                    self.current_track.title = new_title.clone();
                    self.current_track.artist = new_artist.clone();
                    self.current_track.album = new_album.clone();
                    self.sync_smtc_state();
                }

                for t in &mut self.play_queue {
                    if t.file_path.as_deref() == Some(&path_str) {
                        t.title = new_title.clone();
                        t.artist = new_artist.clone();
                        t.album = new_album.clone();
                    }
                }

                self.cached_album_map.take();
                self.cached_artist_map.take();
                self.cached_library_stats.take();
                self.save_library_state();

                self.tag_editor_status = Some(("✅ Metadata berhasil disimpan permanen ke berkas fisik!".to_string(), true));
            }
            Err(e) => {
                self.tag_editor_status = Some((format!("❌ Gagal menyimpan metadata: {}", e), false));
            }
        }
    }

    pub fn render_tag_editor_modal(&mut self, ctx: &egui::Context) {
        if !self.show_tag_editor {
            return;
        }

        let mut close_requested = false;
        let mut do_save = false;
        let mut do_smart_extract = false;

        let screen_rect = ctx.input(|i| i.raw.screen_rect).unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0)));
        #[allow(deprecated)]
        let modal_area_id = egui::Id::new("tag_editor_modal_area");

        egui::Area::new(modal_area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let painter = ui.painter();
                // 1. Fullscreen Dimmed Frosted Backdrop
                painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(170));

                // Click backdrop to dismiss
                let backdrop_click = ui.interact(screen_rect, modal_area_id.with("backdrop_interact"), egui::Sense::click());
                if backdrop_click.clicked() {
                    close_requested = true;
                }

                // 2. Centered Studio Dialog Card
                let modal_w = 560.0f32.min(screen_rect.width() - 32.0);
                let modal_h = 560.0f32.min(screen_rect.height() - 32.0);
                let modal_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(modal_w, modal_h));

                let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(modal_rect));

                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.6, Color32::from_rgb(79, 70, 229))) // Royal Indigo Border
                    .corner_radius(16.0)
                    .inner_margin(22.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 20],
                        blur: 48,
                        spread: 0,
                        color: Color32::from_black_alpha(160),
                    })
                    .show(&mut child_ui, |ui| {
                        // --- HEADER BAR ---
                        ui.horizontal(|ui| {
                            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
                            ui.painter().rect_filled(icon_rect, 8.0, Color32::from_rgb(238, 242, 255));
                            ui.painter().text(
                                icon_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🏷️",
                                egui::FontId::proportional(17.0),
                                Color32::WHITE,
                            );

                            ui.add_space(8.0);

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("TAG METADATA EDITOR")
                                        .strong()
                                        .size(14.5)
                                        .color(Color32::from_rgb(15, 23, 42)),
                                );
                                ui.label(
                                    egui::RichText::new("Lofty Lossless Engine • In-Place Metadata Injection")
                                        .size(11.0)
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("✕").size(14.0).strong().color(Color32::from_rgb(100, 116, 139)))
                                        .fill(Color32::from_rgb(241, 245, 249))
                                        .corner_radius(14.0)
                                        .min_size(Vec2::splat(28.0)),
                                )
                                .on_hover_text("Tutup editor")
                                .clicked()
                                {
                                    close_requested = true;
                                }
                            });
                        });

                        ui.add_space(8.0);

                        // --- BIT-PERFECT GUARANTEE BANNER ---
                        egui::Frame::new()
                            .fill(Color32::from_rgb(236, 253, 245))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(167, 243, 208)))
                            .corner_radius(8.0)
                            .inner_margin(egui::Margin::symmetric(12, 7))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("🛡️ BIT-PERFECT:").strong().size(11.0).color(Color32::from_rgb(6, 95, 70)));
                                    ui.label(
                                        egui::RichText::new("Data stream audio tidak didekompresi ulang. Tag langsung diinjeksi ke kontainer fisik.")
                                            .size(11.0)
                                            .color(Color32::from_rgb(4, 120, 87)),
                                    );
                                });
                            });

                        ui.add_space(10.0);

                        // --- FIELD 1: JUDUL LAGU ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🎵 JUDUL LAGU").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("🪄 Ekstrak dari Nama Berkas")
                                            .size(10.5)
                                            .strong()
                                            .color(Color32::from_rgb(79, 70, 229)),
                                    )
                                    .fill(Color32::from_rgb(238, 242, 255))
                                    .corner_radius(4.0),
                                )
                                .on_hover_text("Otomatis ekstrak Judul & Artis dari nama file (format: Artis - Judul)")
                                .clicked()
                                {
                                    do_smart_extract = true;
                                }
                            });
                        });
                        ui.add_space(3.0);
                        render_clean_text_input(ui, &mut self.tag_editor_title, "Contoh: Happy End");

                        ui.add_space(8.0);

                        // --- FIELD 2 & 3: ARTIS & ALBUM (2 Columns) ---
                        let col2_w = (ui.available_width() - 10.0) / 2.0;
                        ui.horizontal(|ui| {
                            ui.allocate_ui(Vec2::new(col2_w, 46.0), |ui| {
                                ui.label(egui::RichText::new("🎙️ ARTIS / MUSISI").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                                ui.add_space(3.0);
                                render_clean_text_input(ui, &mut self.tag_editor_artist, "Contoh: Back Number");
                            });
                            ui.add_space(6.0);
                            ui.allocate_ui(Vec2::new(col2_w, 46.0), |ui| {
                                ui.label(egui::RichText::new("💿 ALBUM").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                                ui.add_space(3.0);
                                render_clean_text_input(ui, &mut self.tag_editor_album, "Contoh: Encore");
                            });
                        });

                        ui.add_space(8.0);

                        // --- FIELD 4, 5, 6: TAHUN, GENRE, NOMOR TREK (3 Columns) ---
                        let col3_w = (ui.available_width() - 16.0) / 3.0;
                        ui.horizontal(|ui| {
                            ui.allocate_ui(Vec2::new(col3_w, 46.0), |ui| {
                                ui.label(egui::RichText::new("📅 TAHUN RILIS").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                                ui.add_space(3.0);
                                render_clean_text_input(ui, &mut self.tag_editor_year, "Contoh: 2024");
                            });
                            ui.add_space(6.0);
                            ui.allocate_ui(Vec2::new(col3_w, 46.0), |ui| {
                                ui.label(egui::RichText::new("🎭 GENRE").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                                ui.add_space(3.0);
                                render_clean_text_input(ui, &mut self.tag_editor_genre, "Contoh: J-Pop / Rock");
                            });
                            ui.add_space(6.0);
                            ui.allocate_ui(Vec2::new(col3_w, 46.0), |ui| {
                                ui.label(egui::RichText::new("🔢 NOMOR TREK").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                                ui.add_space(3.0);
                                render_clean_text_input(ui, &mut self.tag_editor_track_num, "Contoh: 1");
                            });
                        });

                        ui.add_space(10.0);

                        // --- PHYSICAL FILE INSPECTOR CARD ---
                        let path_str = self.tag_editor_path.as_deref().unwrap_or("Tidak diketahui");
                        let track_match = self.playlist.iter().find(|t| t.file_path.as_deref() == self.tag_editor_path.as_deref());
                        let format_info = if let Some(t) = track_match {
                            format!("{} • {} kHz / {}-bit • {:.1} MB", t.format.to_uppercase(), t.sample_rate as f32 / 1000.0, t.bit_depth, t.file_size_mb)
                        } else {
                            "Lossless Audio File".to_string()
                        };

                        egui::Frame::new()
                            .fill(Color32::from_rgb(241, 245, 249))
                            .corner_radius(8.0)
                            .inner_margin(egui::Margin::symmetric(12, 8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("📂 BERKAS FISIK:").strong().size(10.5).color(Color32::from_rgb(71, 85, 105)));
                                    ui.label(egui::RichText::new(format_info).size(10.5).strong().color(Color32::from_rgb(14, 165, 233)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if let Some(ref p) = self.tag_editor_path {
                                            if ui.add(
                                                egui::Button::new(egui::RichText::new("📂 Buka Folder").size(10.5))
                                                    .fill(Color32::WHITE)
                                                    .corner_radius(4.0),
                                            )
                                            .on_hover_text("Buka lokasi berkas ini di Windows Explorer")
                                            .clicked()
                                            {
                                                crate::ui::components::open_file_in_explorer(p);
                                            }
                                        }
                                    });
                                });
                                ui.add_space(2.0);
                                let truncated_path = if path_str.len() > 68 {
                                    format!("...{}", &path_str[path_str.len() - 64..])
                                } else {
                                    path_str.to_string()
                                };
                                ui.label(egui::RichText::new(truncated_path).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            });

                        // --- FEEDBACK NOTIFICATION ---
                        if let Some((ref msg, is_ok)) = self.tag_editor_status {
                            ui.add_space(8.0);
                            let (bg, text_col, stroke_col) = if is_ok {
                                (Color32::from_rgb(236, 253, 245), Color32::from_rgb(6, 95, 70), Color32::from_rgb(167, 243, 208))
                            } else {
                                (Color32::from_rgb(254, 242, 242), Color32::from_rgb(185, 28, 28), Color32::from_rgb(254, 202, 202))
                            };
                            egui::Frame::new()
                                .fill(bg)
                                .stroke(Stroke::new(1.0, stroke_col))
                                .corner_radius(6.0)
                                .inner_margin(egui::Margin::symmetric(10, 6))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(msg).size(11.5).strong().color(text_col));
                                });
                        }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // --- FOOTER ACTIONS ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("⚡ Perubahan disimpan instan ke berkas fisik").size(11.0).color(Color32::from_rgb(148, 163, 184)));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("💾 Simpan Metadata")
                                            .size(12.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(79, 70, 229))
                                    .corner_radius(6.0)
                                    .min_size(Vec2::new(150.0, 32.0)),
                                )
                                .on_hover_text("Tulis metadata langsung ke berkas fisik")
                                .clicked()
                                {
                                    do_save = true;
                                }

                                ui.add_space(8.0);

                                if ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("Batal")
                                            .size(12.5)
                                            .color(Color32::from_rgb(100, 116, 139)),
                                    )
                                    .fill(Color32::from_rgb(241, 245, 249))
                                    .corner_radius(6.0)
                                    .min_size(Vec2::new(80.0, 32.0)),
                                )
                                .clicked()
                                {
                                    close_requested = true;
                                }
                            });
                        });
                    });
            });

        if do_smart_extract {
            self.smart_extract_tags_from_filename();
        }

        if do_save {
            self.save_tag_editor();
        }

        if close_requested {
            self.show_tag_editor = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_editor_open_and_save_without_path() {
        let mut app = LunulaApp::new_standalone();
        let track = Track {
            title: "Test Track".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            file_path: None,
            ..Default::default()
        };

        app.open_tag_editor(&track);
        assert!(app.show_tag_editor);
        assert_eq!(app.tag_editor_title, "Test Track");
        assert_eq!(app.tag_editor_artist, "Test Artist");
        assert_eq!(app.tag_editor_album, "Test Album");

        // Saving without path should set error status and not crash
        app.save_tag_editor();
        assert!(app.tag_editor_status.is_some());
        assert!(!app.tag_editor_status.as_ref().unwrap().1);
    }

    #[test]
    fn test_tag_editor_modal_headless_render() {
        let mut app = LunulaApp::new_standalone();
        let track = Track {
            title: "Test Track".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            file_path: None,
            ..Default::default()
        };
        app.open_tag_editor(&track);

        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            app.render_tag_editor_modal(ui.ctx());
        });
        out.textures_delta.clear();

        assert!(app.show_tag_editor);
    }

    #[test]
    fn test_tag_editor_smart_extract_from_filename() {
        let mut app = LunulaApp::new_standalone();
        app.tag_editor_path = Some("C:/Music/Back Number - Happy End.flac".to_string());
        app.smart_extract_tags_from_filename();
        assert_eq!(app.tag_editor_artist, "Back Number");
        assert_eq!(app.tag_editor_title, "Happy End");

        // Filename without separator
        app.tag_editor_path = Some("C:/Music/SingleTrack.mp3".to_string());
        app.smart_extract_tags_from_filename();
        assert_eq!(app.tag_editor_title, "SingleTrack");
    }
}
