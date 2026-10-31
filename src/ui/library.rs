use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{CustomPlaylist, LibraryCategory, Track};
use crate::app::LunulaApp;
use crate::ui::components::{draw_album_art, draw_artist_avatar, format_time, lerp_color, open_file_in_explorer, render_card_frame, render_stat_box};

impl LunulaApp {
    pub fn compute_library_stats(&self) -> (usize, usize, usize, f32, usize) {
        let mut albums = std::collections::HashSet::new();
        let mut artists = std::collections::HashSet::new();
        let mut total_mb = 0.0;
        let mut hires = 0;

        for t in &self.playlist {
            if !t.album.trim().is_empty() {
                albums.insert(t.album.trim().to_lowercase());
            }
            if !t.artist.trim().is_empty() {
                artists.insert(t.artist.trim().to_lowercase());
            }
            total_mb += t.file_size_mb;
            if t.sample_rate >= 88200 || t.bit_depth >= 24 {
                hires += 1;
            }
        }
        (self.playlist.len(), albums.len(), artists.len(), total_mb, hires)
    }


    pub fn render_library_panel(&mut self, ui: &mut egui::Ui) {
        // Deferred action queues to avoid borrow checker conflicts
        let mut track_to_play: Option<Track> = None;
        let mut track_play_next: Option<Track> = None;
        let mut track_add_to_queue: Option<Track> = None;
        let mut tracks_to_queue_bulk: Option<Vec<Track>> = None;
        let mut track_to_toggle_fav: Option<(String, String)> = None;
        let mut track_add_to_playlist: Option<(usize, String)> = None;
        let mut track_remove_from_playlist: Option<(String, String)> = None;
        let mut track_to_delete_from_library: Option<(String, String)> = None;
        let mut album_to_delete_from_library: Option<String> = None;
        let mut artist_to_delete_from_library: Option<String> = None;
        let mut playlist_to_delete: Option<usize> = None;
        let mut set_album_filter: Option<String> = None;
        let mut set_artist_filter: Option<String> = None;
        let mut set_quality_filter: Option<String> = None;
        let mut set_playlist_filter: Option<String> = None;
        let mut switch_to_all_tracks = false;
        let mut clear_entire_library = false;

        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                // 1. Header Bar
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("📁 MUSIC LIBRARY").strong().size(15.5).color(Color32::from_rgb(15, 23, 42)));
                    let count_text = format!("• {} Lagu Terdaftar", self.playlist.len());
                    ui.label(egui::RichText::new(count_text).size(12.0).color(Color32::from_rgb(100, 116, 139)));

                    if !self.playlist.is_empty() {
                        if ui.button(egui::RichText::new("🗑️ Kosongkan Library").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus semua lagu dari library").clicked() {
                            clear_entire_library = true;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("➕ Scan Folder Musik").strong()).on_hover_text("Buka dialog untuk memilih folder musik di komputer").clicked() {
                            self.start_folder_scan();
                        }
                        if ui.button(egui::RichText::new("📄 Tambah File").strong()).on_hover_text("Pilih satu atau beberapa file audio").clicked() {
                            self.start_files_scan();
                        }

                        // Search box
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("🔍 Cari judul, artis, album...")
                                .desired_width(190.0),
                        );
                    });
                });

                // Status banner if scanning or recently completed
                if self.is_scanning || !self.scan_status.is_empty() {
                    ui.add_space(6.0);
                    let bg_color = if self.is_scanning {
                        Color32::from_rgb(238, 242, 255)
                    } else {
                        Color32::from_rgb(236, 253, 245)
                    };
                    let text_color = if self.is_scanning {
                        Color32::from_rgb(79, 70, 229)
                    } else {
                        Color32::from_rgb(5, 150, 105)
                    };

                    egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(6.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let icon = if self.is_scanning { "⏳" } else { "✓" };
                                ui.label(egui::RichText::new(icon).strong().color(text_color));
                                ui.label(egui::RichText::new(&self.scan_status).size(11.5).strong().color(text_color));
                            });
                        });
                }

                ui.add_space(10.0);

                // 2. Aggregate Stats Ribbon (5 Cards)
                let (total_tracks, total_albums, total_artists, total_mb, hires_count) = self.compute_library_stats();
                let storage_str = if total_mb >= 1024.0 {
                    format!("{:.2} GB", total_mb / 1024.0)
                } else {
                    format!("{:.1} MB", total_mb)
                };

                ui.horizontal(|ui| {
                    render_stat_box(ui, "🎵", "TOTAL LAGU", &format!("{} Lagu", total_tracks), "Koleksi Aktif", Color32::from_rgb(15, 23, 42));
                    ui.add_space(6.0);
                    render_stat_box(ui, "💿", "ALBUM", &format!("{} Album", total_albums), "Diskografi", Color32::from_rgb(15, 23, 42));
                    ui.add_space(6.0);
                    render_stat_box(ui, "🎙️", "ARTIS", &format!("{} Artis", total_artists), "Penyanyi / Komposer", Color32::from_rgb(15, 23, 42));
                    ui.add_space(6.0);
                    render_stat_box(ui, "💾", "STORAGE", &storage_str, "Ukuran File Audio", Color32::from_rgb(14, 165, 233));
                    ui.add_space(6.0);
                    render_stat_box(ui, "✨", "HI-RES MASTER", &format!("{} Track", hires_count), "≥24-bit / 96kHz", Color32::from_rgb(16, 185, 129));
                });

                ui.add_space(10.0);

                // 3. Category Navigation Pills
                ui.horizontal(|ui| {
                    let categories = [
                        (LibraryCategory::AllTracks, "📋 Semua Lagu"),
                        (LibraryCategory::Albums, "💿 Album"),
                        (LibraryCategory::Artists, "🎙️ Artis"),
                        (LibraryCategory::Quality, "🎚️ Kualitas Audio"),
                        (LibraryCategory::Favorites, "❤️ Favorit"),
                        (LibraryCategory::Playlists, "🎶 Playlist & Album Buatan"),
                    ];

                    for (cat, label) in categories {
                        let is_selected = self.library_category == cat;
                        let (bg, text_col, stroke_col) = if is_selected {
                            (Color32::from_rgb(15, 23, 42), Color32::WHITE, Color32::from_rgb(15, 23, 42))
                        } else {
                            (Color32::WHITE, Color32::from_rgb(51, 65, 85), Color32::from_rgb(203, 213, 225))
                        };

                        let btn_resp = egui::Frame::new()
                            .fill(bg)
                            .stroke(Stroke::new(1.0, stroke_col))
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(13, 7))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(label).strong().size(12.5).color(text_col));
                            });

                        let click = ui.interact(btn_resp.response.rect, ui.scope_id().with(("cat_pill", cat as usize)), egui::Sense::click());
                        if click.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if click.clicked() {
                            self.library_category = cat;
                        }
                        ui.add_space(4.0);
                    }
                });

                // 4. Active Filter Indicator
                let mut reset_filter = false;
                let active_filter_label = if let Some(ref alb) = self.selected_album_filter {
                    Some(format!("Album: \"{}\"", alb))
                } else if let Some(ref art) = self.selected_artist_filter {
                    Some(format!("Artis: \"{}\"", art))
                } else if let Some(ref q) = self.selected_quality_filter {
                    Some(format!("Kualitas: \"{}\"", q))
                } else if let Some(ref pl) = self.selected_playlist_filter {
                    Some(format!("Playlist: \"{}\"", pl))
                } else {
                    None
                };

                if let Some(lbl) = active_filter_label {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        egui::Frame::new()
                            .fill(Color32::from_rgb(238, 242, 255))
                            .corner_radius(5.0)
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(format!("🔍 Filter Aktif: {}", lbl)).strong().size(11.5).color(Color32::from_rgb(79, 70, 229)));
                            });
                        if ui.button(egui::RichText::new("✕ Reset Filter").strong().size(11.5)).clicked() {
                            reset_filter = true;
                        }
                    });
                }
                if reset_filter {
                    self.selected_album_filter = None;
                    self.selected_artist_filter = None;
                    self.selected_quality_filter = None;
                    self.selected_playlist_filter = None;
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);

                // 5. Main Content Area
                if self.playlist.is_empty() {
                    // Empty State Card
                    ui.add_space(20.0);
                    egui::Frame::new()
                        .fill(Color32::from_rgb(248, 250, 252))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                        .corner_radius(12.0)
                        .inner_margin(32.0)
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("📁 🎶").size(38.0));
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("Library Musik Anda Masih Kosong").strong().size(18.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("Lunula Player membaca file musik lokal secara bit-perfect tanpa upload ke cloud atau kompresi kualitas.").size(12.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.add_space(18.0);
                                ui.horizontal(|ui| {
                                    if ui.button(egui::RichText::new("📂 Pindai Folder Musik (FLAC, WAV, MP3...)").strong().size(13.0)).clicked() {
                                        self.start_folder_scan();
                                    }
                                    ui.add_space(12.0);
                                    if ui.button(egui::RichText::new("📄 Tambah File Audio Satuan").strong().size(13.0)).clicked() {
                                        self.start_files_scan();
                                    }
                                });
                            });
                        });
                } else {
                    // Category-specific rendering
                    match self.library_category {
                        LibraryCategory::Albums => {
                            // Group tracks by album
                            let mut album_map: std::collections::BTreeMap<String, Vec<&Track>> = std::collections::BTreeMap::new();
                            for t in &self.playlist {
                                let alb = if t.album.trim().is_empty() { "Single / Unknown Album".to_string() } else { t.album.clone() };
                                album_map.entry(alb).or_default().push(t);
                            }

                            let available_w = ui.available_width();
                            let card_w = 215.0;
                            let gap = 14.0;
                            let cols = ((available_w + gap) / (card_w + gap)).floor().max(1.0) as usize;
                            let all_albums: Vec<_> = album_map.into_iter().collect();

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for chunk in all_albums.chunks(cols) {
                                    ui.horizontal(|ui| {
                                        for (alb_name, tracks) in chunk {
                                            let primary_artist = &tracks[0].artist;
                                            let total_size: f32 = tracks.iter().map(|t| t.file_size_mb).sum();
                                            let is_hires = tracks.iter().any(|t| t.sample_rate >= 88200 || t.bit_depth >= 24);

                                            let card_id = ui.scope_id().with(("alb_card", alb_name));
                                            let card_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(card_w - 28.0, 245.0));
                                            let is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| card_rect.contains(p)));
                                            let hover_t = ui.ctx().animate_bool(card_id, is_hovered);

                                            let bg_color = lerp_color(Color32::WHITE, Color32::from_rgb(248, 250, 252), hover_t);
                                            let border_color = lerp_color(Color32::from_rgb(226, 232, 240), Color32::from_rgb(14, 165, 233), hover_t);
                                            let stroke_w = 1.0 + hover_t * 0.7;

                                            let frame_resp = egui::Frame::new()
                                                .fill(bg_color)
                                                .stroke(Stroke::new(stroke_w, border_color))
                                                .corner_radius(12.0)
                                                .inner_margin(egui::Margin::symmetric(14, 16))
                                                .show(ui, |ui| {
                                                    ui.set_width(card_w - 28.0);
                                                    ui.set_min_height(245.0);
                                                    ui.vertical(|ui| {
                                                        // Vinyl Art Box or Embedded Cover Art
                                                        let (art_rect, _) = ui.allocate_exact_size(Vec2::new(card_w - 28.0, 110.0), egui::Sense::hover());
                                                        let painter = ui.painter_at(art_rect);
                                                        let cover_tex = self.get_album_cover_texture(ui.ctx(), alb_name, tracks[0].file_path.as_deref());
                                                        if let Some(ref tex) = cover_tex {
                                                            painter.rect_filled(art_rect, 8.0, Color32::from_rgb(15, 23, 42));
                                                            painter.image(
                                                                tex.id(),
                                                                art_rect,
                                                                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                                                Color32::WHITE,
                                                            );
                                                            painter.rect_stroke(art_rect, 8.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);
                                                            if is_hires {
                                                                let badge_rect = Rect::from_min_size(art_rect.min + Vec2::new(6.0, 6.0), Vec2::new(48.0, 18.0));
                                                                painter.rect_filled(badge_rect, 4.0, Color32::from_rgba_unmultiplied(16, 185, 129, 230));
                                                                painter.text(badge_rect.center(), egui::Align2::CENTER_CENTER, "HI-RES", egui::FontId::proportional(9.0), Color32::WHITE);
                                                            }
                                                        } else {
                                                            draw_album_art(&painter, art_rect, alb_name, is_hires);
                                                        }

                                                        ui.add_space(8.0);
                                                        ui.add(egui::Label::new(egui::RichText::new(alb_name).strong().size(13.5).color(Color32::from_rgb(15, 23, 42))).truncate());
                                                        ui.add(egui::Label::new(egui::RichText::new(primary_artist).size(11.5).color(Color32::from_rgb(71, 85, 105))).truncate());

                                                        ui.add_space(4.0);
                                                        let sub_text = format!("{} Lagu • {:.1} MB", tracks.len(), total_size);
                                                        ui.label(egui::RichText::new(sub_text).size(11.0).color(Color32::from_rgb(100, 116, 139)));

                                                        ui.add_space(10.0);
                                                        ui.horizontal(|ui| {
                                                            if ui.add_sized([58.0, 26.0], egui::Button::new(egui::RichText::new("▶ Putar").strong().size(11.5))).clicked() {
                                                                track_to_play = Some((*tracks[0]).clone());
                                                            }
                                                            if ui.add_sized([84.0, 26.0], egui::Button::new(egui::RichText::new("Lihat Lagu").size(11.5))).clicked() {
                                                                set_album_filter = Some(alb_name.clone());
                                                                switch_to_all_tracks = true;
                                                            }
                                                            if ui.add_sized([30.0, 26.0], egui::Button::new(egui::RichText::new("🗑️").size(11.5))).on_hover_text("Hapus Album dari Library").clicked() {
                                                                album_to_delete_from_library = Some(alb_name.clone());
                                                            }
                                                        });
                                                        ui.add_space(4.0);
                                                    });
                                                });

                                            let card_resp = ui.interact(frame_resp.response.rect, ui.scope_id().with(("alb_card", alb_name)), egui::Sense::click());
                                            if card_resp.hovered() {
                                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                            }
                                            card_resp.context_menu(|ui| {
                                                ui.label(egui::RichText::new(alb_name).strong().size(13.0).color(Color32::from_rgb(15, 23, 42)));
                                                ui.label(egui::RichText::new(primary_artist).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                                ui.separator();
                                                if ui.button("▶ Putar Album Ini").clicked() {
                                                    track_to_play = Some((*tracks[0]).clone());
                                                    if tracks.len() > 1 {
                                                        tracks_to_queue_bulk = Some(tracks[1..].iter().map(|&t| t.clone()).collect());
                                                    }
                                                    ui.close();
                                                }
                                                if ui.button("➕ Tambahkan Album ke Antrean").clicked() {
                                                    tracks_to_queue_bulk = Some(tracks.iter().map(|&t| t.clone()).collect());
                                                    ui.close();
                                                }
                                                if ui.button("🔍 Buka & Lihat Lagu Album").clicked() {
                                                    set_album_filter = Some(alb_name.clone());
                                                    switch_to_all_tracks = true;
                                                    ui.close();
                                                }
                                                if ui.button("🗑️ Hapus Seluruh Album dari Library").clicked() {
                                                    album_to_delete_from_library = Some(alb_name.clone());
                                                    ui.close();
                                                }
                                            });

                                            ui.add_space(gap);
                                        }
                                    });
                                    ui.add_space(gap);
                                }
                            });
                        }
                        LibraryCategory::Artists => {
                            // Group tracks by artist
                            let mut artist_map: std::collections::BTreeMap<String, Vec<&Track>> = std::collections::BTreeMap::new();
                            for t in &self.playlist {
                                let art = if t.artist.trim().is_empty() { "Unknown Artist".to_string() } else { t.artist.clone() };
                                artist_map.entry(art).or_default().push(t);
                            }

                            let available_w = ui.available_width();
                            let card_w = 195.0;
                            let gap = 14.0;
                            let cols = ((available_w + gap) / (card_w + gap)).floor().max(1.0) as usize;
                            let all_artists: Vec<_> = artist_map.into_iter().collect();

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for chunk in all_artists.chunks(cols) {
                                    ui.horizontal(|ui| {
                                        for (art_name, tracks) in chunk {
                                            let unique_albums: std::collections::HashSet<_> = tracks.iter().map(|t| &t.album).collect();

                                            let card_id = ui.scope_id().with(("art_card", art_name));
                                            let card_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(card_w - 28.0, 205.0));
                                            let is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| card_rect.contains(p)));
                                            let hover_t = ui.ctx().animate_bool(card_id, is_hovered);

                                            let bg_color = lerp_color(Color32::WHITE, Color32::from_rgb(248, 250, 252), hover_t);
                                            let border_color = lerp_color(Color32::from_rgb(226, 232, 240), Color32::from_rgb(16, 185, 129), hover_t);
                                            let stroke_w = 1.0 + hover_t * 0.7;

                                            let frame_resp = egui::Frame::new()
                                                .fill(bg_color)
                                                .stroke(Stroke::new(stroke_w, border_color))
                                                .corner_radius(12.0)
                                                .inner_margin(egui::Margin::symmetric(14, 16))
                                                .show(ui, |ui| {
                                                    ui.set_width(card_w - 28.0);
                                                    ui.set_min_height(205.0);
                                                    ui.vertical_centered(|ui| {
                                                        // Artist Avatar Circle
                                                        let (avatar_rect, _) = ui.allocate_exact_size(Vec2::new(56.0, 56.0), egui::Sense::hover());
                                                        let painter = ui.painter_at(avatar_rect);
                                                        draw_artist_avatar(&painter, avatar_rect.center(), 28.0, art_name);

                                                        ui.add_space(8.0);
                                                        ui.add(egui::Label::new(egui::RichText::new(art_name).strong().size(14.0).color(Color32::from_rgb(15, 23, 42))).truncate());

                                                        let meta_text = format!("{} Lagu • {} Album", tracks.len(), unique_albums.len());
                                                        ui.label(egui::RichText::new(meta_text).size(11.5).color(Color32::from_rgb(71, 85, 105)));

                                                        ui.add_space(10.0);
                                                        // Horizontally centered action buttons
                                                        ui.horizontal(|ui| {
                                                            let total_btn_w = 84.0 + 32.0 + 6.0;
                                                            let offset = ((card_w - 28.0 - total_btn_w) / 2.0).max(0.0);
                                                            ui.add_space(offset);

                                                            if ui.add_sized([84.0, 26.0], egui::Button::new(egui::RichText::new("Lihat Lagu").size(11.5))).clicked() {
                                                                set_artist_filter = Some(art_name.clone());
                                                                switch_to_all_tracks = true;
                                                            }
                                                            if ui.add_sized([32.0, 26.0], egui::Button::new(egui::RichText::new("🗑️").size(11.5))).on_hover_text("Hapus Artis dari Library").clicked() {
                                                                artist_to_delete_from_library = Some(art_name.clone());
                                                            }
                                                        });
                                                        ui.add_space(4.0);
                                                    });
                                                });

                                            let card_resp = ui.interact(frame_resp.response.rect, ui.scope_id().with(("art_card", art_name)), egui::Sense::click());
                                            if card_resp.hovered() {
                                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                            }
                                            card_resp.context_menu(|ui| {
                                                ui.label(egui::RichText::new(art_name).strong().size(13.0).color(Color32::from_rgb(15, 23, 42)));
                                                ui.separator();
                                                if ui.button("▶ Putar Semua Lagu Artis").clicked() {
                                                    track_to_play = Some((*tracks[0]).clone());
                                                    if tracks.len() > 1 {
                                                        tracks_to_queue_bulk = Some(tracks[1..].iter().map(|&t| t.clone()).collect());
                                                    }
                                                    ui.close();
                                                }
                                                if ui.button("➕ Tambah Semua ke Antrean").clicked() {
                                                    tracks_to_queue_bulk = Some(tracks.iter().map(|&t| t.clone()).collect());
                                                    ui.close();
                                                }
                                                if ui.button("🔍 Buka Diskografi Artis").clicked() {
                                                    set_artist_filter = Some(art_name.clone());
                                                    switch_to_all_tracks = true;
                                                    ui.close();
                                                }
                                                if ui.button("🗑️ Hapus Semua Lagu Artis dari Library").clicked() {
                                                    artist_to_delete_from_library = Some(art_name.clone());
                                                    ui.close();
                                                }
                                            });

                                            ui.add_space(gap);
                                        }
                                    });
                                    ui.add_space(gap);
                                }
                            });
                        }
                        LibraryCategory::Quality => {
                            // 4 Tiered Quality Cards
                            let mut hires_tracks = Vec::new();
                            let mut cd_lossless_tracks = Vec::new();
                            let mut direct_pcm_tracks = Vec::new();
                            let mut compressed_tracks = Vec::new();

                            for t in &self.playlist {
                                let fmt = t.format.to_uppercase();
                                if t.sample_rate >= 88200 || t.bit_depth >= 24 {
                                    hires_tracks.push(t);
                                } else if (fmt == "FLAC" || fmt == "ALAC") && t.bit_depth == 16 {
                                    cd_lossless_tracks.push(t);
                                } else if fmt == "WAV" || fmt == "AIFF" || fmt == "DSD" || fmt == "DSF" {
                                    direct_pcm_tracks.push(t);
                                } else {
                                    compressed_tracks.push(t);
                                }
                            }

                            let tiers = [
                                ("✨ 24-Bit Studio Master Hi-Res", "≥88.2kHz / 24-bit FLAC/WAV Direct Stream", hires_tracks, Color32::from_rgb(16, 185, 129), "hires"),
                                ("💿 16-Bit Red Book CD Lossless", "44.1kHz / 16-bit 1411kbps Bit-Perfect FLAC", cd_lossless_tracks, Color32::from_rgb(14, 165, 233), "lossless"),
                                ("⚡ Raw Uncompressed PCM / DSD", "WAV, AIFF, DSD Direct Bitstream", direct_pcm_tracks, Color32::from_rgb(139, 92, 246), "pcm"),
                                ("🎧 Compressed Dynamic Audio", "MP3, AAC, OGG, Opus VBR/CBR Encoding", compressed_tracks, Color32::from_rgb(100, 116, 139), "compressed"),
                            ];

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (tier_title, desc, tracks, col, tier_id) in tiers {
                                    let total_size: f32 = tracks.iter().map(|t| t.file_size_mb).sum();
                                    let card_id = ui.scope_id().with(("tier_card", tier_id));
                                    let card_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 70.0));
                                    let is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| card_rect.contains(p)));
                                    let hover_t = ui.ctx().animate_bool(card_id, is_hovered);

                                    let bg_color = lerp_color(Color32::WHITE, Color32::from_rgb(248, 250, 252), hover_t);
                                    let border_color = lerp_color(Color32::from_rgb(226, 232, 240), col, hover_t * 0.7);
                                    let stroke = Stroke::new(1.0 + hover_t * 0.7, border_color);

                                    egui::Frame::new()
                                        .fill(bg_color)
                                        .stroke(stroke)
                                        .corner_radius(10.0)
                                        .inner_margin(14.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.label(egui::RichText::new(tier_title).strong().size(14.0).color(col));
                                                    ui.add_space(2.0);
                                                    ui.label(egui::RichText::new(desc).size(11.0).color(Color32::from_rgb(100, 116, 139)));
                                                    ui.label(egui::RichText::new(format!("{} Lagu • {:.1} MB", tracks.len(), total_size)).size(10.5).color(Color32::from_rgb(148, 163, 184)));
                                                });
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button(egui::RichText::new("Filter Kategori Ini ➔").strong().size(11.5)).clicked() {
                                                        set_quality_filter = Some(tier_id.to_string());
                                                        switch_to_all_tracks = true;
                                                    }
                                                });
                                            });
                                        });
                                    ui.add_space(8.0);
                                }
                            });
                        }
                        LibraryCategory::Favorites => {
                            let fav_tracks: Vec<&Track> = self.playlist.iter().filter(|t| t.is_favorite).collect();

                            if fav_tracks.is_empty() {
                                ui.add_space(24.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("🤍").size(36.0));
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new("Belum Ada Lagu Favorit").strong().size(16.0).color(Color32::from_rgb(15, 23, 42)));
                                    ui.label(egui::RichText::new("Klik tanda ♡ di samping lagu atau klik kanan lagu untuk menambahkannya ke favorit.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("❤️ {} Lagu Favorit", fav_tracks.len())).strong().size(13.5).color(Color32::from_rgb(225, 29, 72)));
                                    if ui.button(egui::RichText::new("▶ Putar Semua Favorit").strong()).clicked() {
                                        track_to_play = Some((*fav_tracks[0]).clone());
                                    }
                                });
                                ui.add_space(6.0);

                                egui::ScrollArea::vertical().show_rows(ui, 48.0, fav_tracks.len(), |ui, row_range| {
                                    for idx in row_range {
                                        let song = fav_tracks[idx];
                                        let fav_row_id = ui.scope_id().with(("fav_row", idx));
                                        let row_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 44.0));
                                        let is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| row_rect.contains(p)));
                                        let hover_t = ui.ctx().animate_bool(fav_row_id, is_hovered);

                                        let base_bg = if idx % 2 == 1 {
                                            Color32::from_rgb(248, 250, 252)
                                        } else {
                                            Color32::WHITE
                                        };
                                        let bg_color = lerp_color(base_bg, Color32::from_rgb(241, 245, 249), hover_t);
                                        let border_color = lerp_color(Color32::TRANSPARENT, Color32::from_rgb(254, 205, 211), hover_t);
                                        let stroke = Stroke::new(1.0, border_color);

                                        let row_resp = egui::Frame::new()
                                            .fill(bg_color)
                                            .stroke(stroke)
                                            .corner_radius(6.0)
                                            .inner_margin(egui::Margin::symmetric(10, 6))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    if ui.button(egui::RichText::new("❤️").size(14.0)).on_hover_text("Hapus dari Favorit").clicked() {
                                                        track_to_toggle_fav = Some((song.title.clone(), song.artist.clone()));
                                                    }
                                                    ui.add_space(4.0);
                                                    ui.label(egui::RichText::new(format!("{:02}", idx + 1)).strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));
                                                    ui.add_space(6.0);

                                                    // Cover Art Thumbnail
                                                    let cover_key = if !song.album.trim().is_empty() {
                                                        &song.album
                                                    } else {
                                                        song.file_path.as_deref().unwrap_or(&song.title)
                                                    };
                                                    let thumb_tex = self.get_album_cover_texture(ui.ctx(), cover_key, song.file_path.as_deref());
                                                    let (thumb_rect, _) = ui.allocate_exact_size(Vec2::new(30.0, 30.0), egui::Sense::hover());
                                                    let painter = ui.painter_at(thumb_rect);
                                                    if let Some(ref tex) = thumb_tex {
                                                        painter.rect_filled(thumb_rect, 4.0, Color32::from_rgb(15, 23, 42));
                                                        painter.image(
                                                            tex.id(),
                                                            thumb_rect,
                                                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                                            Color32::WHITE,
                                                        );
                                                        painter.rect_stroke(thumb_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);
                                                    } else {
                                                        painter.rect_filled(thumb_rect, 4.0, Color32::from_rgb(241, 245, 249));
                                                        painter.rect_stroke(thumb_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);
                                                        painter.circle_filled(thumb_rect.center(), 10.0, Color32::from_rgb(30, 41, 59));
                                                        painter.circle_filled(thumb_rect.center(), 3.5, Color32::from_rgb(225, 29, 72));
                                                    }
                                                    ui.add_space(6.0);

                                                    ui.vertical(|ui| {
                                                        ui.label(egui::RichText::new(&song.title).strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                                                        ui.label(egui::RichText::new(format!("{} • {}", song.artist, song.album)).size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                                    });

                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if ui.button(egui::RichText::new("🗑️").size(11.5)).on_hover_text("Hapus Lagu dari Library").clicked() {
                                                            track_to_delete_from_library = Some((song.title.clone(), song.artist.clone()));
                                                        }
                                                        if ui.button(egui::RichText::new("▶").strong().size(12.0)).clicked() {
                                                            track_to_play = Some((*song).clone());
                                                        }
                                                        ui.add_space(8.0);
                                                        ui.label(egui::RichText::new(format_time(song.duration)).strong().size(12.5).color(Color32::from_rgb(30, 41, 59)));
                                                    });
                                                });
                                            });

                                        ui.add_space(2.0);

                                        let row_click = ui.interact(row_resp.response.rect, ui.scope_id().with(("fav_row", idx)), egui::Sense::click());
                                        if row_click.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if row_click.clicked() {
                                            track_to_play = Some((*song).clone());
                                        }
                                        row_click.context_menu(|ui| {
                                            ui.label(egui::RichText::new(&song.title).strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                            ui.label(egui::RichText::new(format!("{} • {}", song.artist, song.album)).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                            ui.separator();
                                            if ui.button("▶ Putar Sekarang").clicked() {
                                                track_to_play = Some((*song).clone());
                                                ui.close();
                                            }
                                            if ui.button("⏳ Putar Berikutnya (Play Next)").clicked() {
                                                track_play_next = Some((*song).clone());
                                                ui.close();
                                            }
                                            if ui.button("➕ Tambahkan ke Antrean").clicked() {
                                                track_add_to_queue = Some((*song).clone());
                                                ui.close();
                                            }
                                            ui.separator();
                                            if ui.button("🤍 Hapus dari Favorit").clicked() {
                                                track_to_toggle_fav = Some((song.title.clone(), song.artist.clone()));
                                                ui.close();
                                            }
                                            let track_id = song.file_path.clone().unwrap_or_else(|| format!("{} - {}", song.artist, song.title));
                                            ui.menu_button("🎶 Tambahkan ke Playlist", |ui| {
                                                for pl in &self.custom_playlists {
                                                    if ui.button(&pl.name).clicked() {
                                                        track_add_to_playlist = Some((pl.id, track_id.clone()));
                                                        ui.close();
                                                    }
                                                }
                                            });
                                            if ui.button("🗑️ Hapus Lagu dari Library").clicked() {
                                                track_to_delete_from_library = Some((song.title.clone(), song.artist.clone()));
                                                ui.close();
                                            }
                                            if let Some(ref path) = song.file_path {
                                                if ui.button("📂 Buka Lokasi File di Explorer").clicked() {
                                                    open_file_in_explorer(path);
                                                    ui.close();
                                                }
                                            }
                                        });

                                        ui.separator();
                                    }
                                });
                            }
                        }
                        LibraryCategory::Playlists => {
                            // Top Action Bar
                            ui.horizontal(|ui| {
                                if !self.show_playlist_creator {
                                    if ui.button(egui::RichText::new("➕ Buat Playlist Baru").strong().size(12.5)).clicked() {
                                        self.show_playlist_creator = true;
                                        self.playlist_creator_name.clear();
                                        self.playlist_creator_desc.clear();
                                        self.playlist_creator_selected_tracks.clear();
                                        self.playlist_creator_search.clear();
                                    }
                                }
                                ui.label(egui::RichText::new(format!("• {} Playlist Tersedia", self.custom_playlists.len())).size(11.5).color(Color32::from_rgb(148, 163, 184)));
                            });

                            ui.add_space(8.0);

                            // Expanded Interactive Playlist Creator Form
                            if self.show_playlist_creator {
                                egui::Frame::new()
                                    .fill(Color32::WHITE)
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                    .corner_radius(12.0)
                                    .inner_margin(16.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("🎶 Buat Playlist / Album Buatan Baru").strong().size(14.5).color(Color32::from_rgb(15, 23, 42)));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("✕ Tutup").clicked() {
                                                    self.show_playlist_creator = false;
                                                }
                                            });
                                        });
                                        ui.separator();
                                        ui.add_space(6.0);

                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("Nama Playlist:").strong().size(12.0));
                                            ui.add(egui::TextEdit::singleline(&mut self.playlist_creator_name).hint_text("Contoh: Late Night Vibes, Anime Masterpieces...").desired_width(260.0));

                                            ui.add_space(10.0);
                                            ui.label(egui::RichText::new("Deskripsi:").size(12.0));
                                            ui.add(egui::TextEdit::singleline(&mut self.playlist_creator_desc).hint_text("Deskripsi singkat (opsional)...").desired_width(240.0));
                                        });

                                        ui.add_space(10.0);
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("Pilih Lagu untuk Dimasukkan ({} dipilih):", self.playlist_creator_selected_tracks.len())).strong().size(12.0).color(Color32::from_rgb(2, 132, 199)));
                                            ui.add(egui::TextEdit::singleline(&mut self.playlist_creator_search).hint_text("🔍 Filter lagu...").desired_width(180.0));

                                            if ui.button("Pilih Semua").clicked() {
                                                for t in &self.playlist {
                                                    let id = t.file_path.clone().unwrap_or_else(|| format!("{} - {}", t.artist, t.title));
                                                    self.playlist_creator_selected_tracks.insert(id);
                                                }
                                            }
                                            if ui.button("Kosongkan Pilihan").clicked() {
                                                self.playlist_creator_selected_tracks.clear();
                                            }
                                        });

                                        ui.add_space(6.0);

                                        // Song picker checklist
                                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                                            let q = self.playlist_creator_search.trim().to_lowercase();
                                            for song in &self.playlist {
                                                if !q.is_empty() && !song.title.to_lowercase().contains(&q) && !song.artist.to_lowercase().contains(&q) {
                                                    continue;
                                                }
                                                let id = song.file_path.clone().unwrap_or_else(|| format!("{} - {}", song.artist, song.title));
                                                let mut checked = self.playlist_creator_selected_tracks.contains(&id);
                                                ui.horizontal(|ui| {
                                                    if ui.checkbox(&mut checked, "").changed() {
                                                        if checked {
                                                            self.playlist_creator_selected_tracks.insert(id.clone());
                                                        } else {
                                                            self.playlist_creator_selected_tracks.remove(&id);
                                                        }
                                                    }
                                                    ui.label(egui::RichText::new(&song.title).strong().size(12.0));
                                                    ui.label(egui::RichText::new(format!("• {} ({})", song.artist, song.format)).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                                });
                                            }
                                        });

                                        ui.add_space(10.0);
                                        ui.horizontal(|ui| {
                                            let can_save = !self.playlist_creator_name.trim().is_empty();
                                            let save_btn = ui.add_enabled(
                                                can_save,
                                                egui::Button::new(egui::RichText::new("✓ Simpan Playlist").strong()).fill(Color32::from_rgb(16, 185, 129)),
                                            );
                                            if save_btn.clicked() {
                                                let next_id = self.custom_playlists.iter().map(|p| p.id).max().unwrap_or(0) + 1;
                                                let desc = if self.playlist_creator_desc.trim().is_empty() {
                                                    "Album / Playlist Buatan Pengguna".to_string()
                                                } else {
                                                    self.playlist_creator_desc.trim().to_string()
                                                };
                                                self.custom_playlists.push(CustomPlaylist {
                                                    id: next_id,
                                                    name: self.playlist_creator_name.trim().to_string(),
                                                    description: desc,
                                                    track_paths: self.playlist_creator_selected_tracks.iter().cloned().collect(),
                                                });
                                                self.show_playlist_creator = false;
                                                self.save_library_state();
                                            }

                                            if ui.button("Batal").clicked() {
                                                self.show_playlist_creator = false;
                                            }
                                        });
                                    });
                                ui.add_space(10.0);
                            }

                            // Playlist Cards
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for pl in &self.custom_playlists {
                                    let pl_id = pl.id;
                                    let pl_name = pl.name.clone();
                                    let track_count = pl.track_paths.len();

                                    // Resolve tracks in this playlist
                                    let pl_tracks: Vec<Track> = self.playlist.iter().filter(|t| {
                                        let track_id = t.file_path.clone().unwrap_or_else(|| format!("{} - {}", t.artist, t.title));
                                        pl.track_paths.contains(&track_id)
                                    }).cloned().collect();

                                    let card_id = ui.scope_id().with(("pl_card", pl_id));
                                    let card_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 65.0));
                                    let is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| card_rect.contains(p)));
                                    let hover_t = ui.ctx().animate_bool(card_id, is_hovered);

                                    let bg_color = lerp_color(Color32::WHITE, Color32::from_rgb(248, 250, 252), hover_t);
                                    let border_color = lerp_color(Color32::from_rgb(226, 232, 240), Color32::from_rgb(99, 102, 241), hover_t);
                                    let stroke = Stroke::new(1.0 + hover_t * 0.8, border_color);

                                    let frame_resp = egui::Frame::new()
                                        .fill(bg_color)
                                        .stroke(stroke)
                                        .corner_radius(10.0)
                                        .inner_margin(12.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(egui::RichText::new("🎶").size(15.0));
                                                        ui.label(egui::RichText::new(&pl.name).strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                                                    });
                                                    ui.add_space(2.0);
                                                    ui.label(egui::RichText::new(&pl.description).size(11.0).color(Color32::from_rgb(100, 116, 139)));
                                                    ui.label(egui::RichText::new(format!("{} Lagu Terdaftar", track_count)).size(10.0).color(Color32::from_rgb(148, 163, 184)));
                                                });

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if pl_id > 2 {
                                                        if ui.button(egui::RichText::new("🗑️").size(12.0)).on_hover_text("Hapus Playlist Ini").clicked() {
                                                            playlist_to_delete = Some(pl_id);
                                                        }
                                                    }
                                                    if ui.button(egui::RichText::new("Lihat Lagu").size(11.0)).clicked() {
                                                        set_playlist_filter = Some(pl_name.clone());
                                                        switch_to_all_tracks = true;
                                                    }
                                                    if !pl_tracks.is_empty() {
                                                        if ui.button(egui::RichText::new("➕ Antrean").size(11.0)).on_hover_text("Tambahkan semua lagu playlist ke antrean").clicked() {
                                                            tracks_to_queue_bulk = Some(pl_tracks.clone());
                                                        }
                                                        if ui.button(egui::RichText::new("▶ Putar").strong().size(11.0)).clicked() {
                                                            track_to_play = Some(pl_tracks[0].clone());
                                                            if pl_tracks.len() > 1 {
                                                                tracks_to_queue_bulk = Some(pl_tracks[1..].to_vec());
                                                            }
                                                        }
                                                    }
                                                });
                                            });
                                        });

                                    let pl_click = ui.interact(frame_resp.response.rect, ui.scope_id().with(("pl_card", pl_id)), egui::Sense::click());
                                    if pl_click.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    pl_click.context_menu(|ui| {
                                        ui.label(egui::RichText::new(&pl.name).strong().size(13.0).color(Color32::from_rgb(15, 23, 42)));
                                        ui.separator();
                                        if !pl_tracks.is_empty() {
                                            if ui.button("▶ Putar Playlist Ini").clicked() {
                                                track_to_play = Some(pl_tracks[0].clone());
                                                if pl_tracks.len() > 1 {
                                                    tracks_to_queue_bulk = Some(pl_tracks[1..].to_vec());
                                                }
                                                ui.close();
                                            }
                                            if ui.button("➕ Tambahkan Semua ke Antrean").clicked() {
                                                tracks_to_queue_bulk = Some(pl_tracks.clone());
                                                ui.close();
                                            }
                                        }
                                        if ui.button("🔍 Buka & Lihat Lagu di Playlist").clicked() {
                                            set_playlist_filter = Some(pl_name.clone());
                                            switch_to_all_tracks = true;
                                            ui.close();
                                        }
                                        if ui.button("🗑️ Hapus Playlist Ini").clicked() {
                                            playlist_to_delete = Some(pl_id);
                                            ui.close();
                                        }
                                    });

                                    ui.add_space(6.0);
                                }
                            });
                        }
                        LibraryCategory::AllTracks => {
                            // Column Headers with soft Slate-100 container
                            egui::Frame::new()
                                .fill(Color32::from_rgb(241, 245, 249))
                                .corner_radius(6.0)
                                .inner_margin(egui::Margin::symmetric(10, 7))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.allocate_ui(Vec2::new(28.0, 18.0), |ui| {
                                            ui.label(egui::RichText::new("#").strong().size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                        });
                                        ui.allocate_ui(Vec2::new(34.0, 18.0), |ui| {
                                            ui.label(egui::RichText::new("ART").strong().size(10.5).color(Color32::from_rgb(71, 85, 105)));
                                        });
                                        ui.allocate_ui(Vec2::new(28.0, 18.0), |ui| {
                                            ui.label(egui::RichText::new("FAV").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                                        });
                                        ui.allocate_ui(Vec2::new(290.0, 18.0), |ui| {
                                            ui.label(egui::RichText::new("JUDUL & ARTIS").strong().size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                        });
                                        ui.allocate_ui(Vec2::new(220.0, 18.0), |ui| {
                                            ui.label(egui::RichText::new("ALBUM").strong().size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                        });
                                        ui.allocate_ui(Vec2::new(170.0, 18.0), |ui| {
                                            ui.label(egui::RichText::new("FORMAT / RESOLUSI").strong().size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                        });
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(egui::RichText::new("DURASI").strong().size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                            ui.add_space(20.0);
                                            ui.label(egui::RichText::new("UKURAN").strong().size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                        });
                                    });
                                });
                            ui.add_space(3.0);

                            // Filter Tracks
                            let query = self.search_query.trim().to_lowercase();
                            let matching_tracks: Vec<(usize, &Track)> = self.playlist.iter().enumerate().filter(|(_idx, song)| {
                                // Search Filter
                                if !query.is_empty() {
                                    let match_title = song.title.to_lowercase().contains(&query);
                                    let match_artist = song.artist.to_lowercase().contains(&query);
                                    let match_album = song.album.to_lowercase().contains(&query);
                                    let match_format = song.format.to_lowercase().contains(&query);
                                    if !match_title && !match_artist && !match_album && !match_format {
                                        return false;
                                    }
                                }

                                // Category Filter
                                if let Some(ref alb) = self.selected_album_filter {
                                    if &song.album != alb {
                                        return false;
                                    }
                                }
                                if let Some(ref art) = self.selected_artist_filter {
                                    if &song.artist != art {
                                        return false;
                                    }
                                }
                                if let Some(ref q) = self.selected_quality_filter {
                                    match q.as_str() {
                                        "hires" => {
                                            if song.sample_rate < 88200 && song.bit_depth < 24 {
                                                return false;
                                            }
                                        }
                                        "lossless" => {
                                            let fmt = song.format.to_uppercase();
                                            if (fmt != "FLAC" && fmt != "ALAC") || song.bit_depth != 16 {
                                                return false;
                                            }
                                        }
                                        "pcm" => {
                                            let fmt = song.format.to_uppercase();
                                            if fmt != "WAV" && fmt != "AIFF" && fmt != "DSD" {
                                                return false;
                                            }
                                        }
                                        "compressed" => {
                                            let fmt = song.format.to_uppercase();
                                            if fmt == "FLAC" || fmt == "WAV" || fmt == "AIFF" || fmt == "ALAC" || fmt == "DSD" {
                                                return false;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                if let Some(ref pl_name) = self.selected_playlist_filter {
                                    let pl = self.custom_playlists.iter().find(|p| &p.name == pl_name);
                                    let track_id = song.file_path.clone().unwrap_or_else(|| format!("{} - {}", song.artist, song.title));
                                    if let Some(p) = pl {
                                        if !p.track_paths.contains(&track_id) {
                                            return false;
                                        }
                                    }
                                }
                                true
                            }).collect();

                            if matching_tracks.is_empty() {
                                ui.add_space(24.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("Tidak ada lagu yang cocok dengan kriteria filter / pencarian.").size(12.5).color(Color32::from_rgb(148, 163, 184)));
                                });
                            } else {
                                // Track Rows Virtual Scroll: Only renders visible rows for instant 0ms latency
                                egui::ScrollArea::vertical().show_rows(ui, 48.0, matching_tracks.len(), |ui, row_range| {
                                    for row_idx in row_range {
                                        let (idx, song) = matching_tracks[row_idx];
                                        let is_current = self.current_track.title == song.title && self.current_track.artist == song.artist;

                                        let row_id = ui.scope_id().with(("track_row", idx));
                                        let row_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 44.0));
                                        let is_hovered = ui.input(|i| i.pointer.hover_pos().map_or(false, |p| row_rect.contains(p)));
                                        let hover_t = ui.ctx().animate_bool(row_id, is_hovered);

                                        let (base_bg, base_border) = if is_current {
                                            (
                                                Color32::from_rgb(236, 253, 245),
                                                Color32::from_rgb(167, 243, 208),
                                            )
                                        } else if row_idx % 2 == 1 {
                                            (
                                                Color32::from_rgb(248, 250, 252),
                                                Color32::TRANSPARENT,
                                            )
                                        } else {
                                            (
                                                Color32::WHITE,
                                                Color32::TRANSPARENT,
                                            )
                                        };

                                    let bg_color = if is_current {
                                        lerp_color(base_bg, Color32::from_rgb(209, 250, 229), hover_t)
                                    } else {
                                        lerp_color(base_bg, Color32::from_rgb(241, 245, 249), hover_t)
                                    };
                                    let border_color = if is_current {
                                        lerp_color(base_border, Color32::from_rgb(52, 211, 153), hover_t)
                                    } else {
                                        lerp_color(base_border, Color32::from_rgb(203, 213, 225), hover_t)
                                    };
                                    let stroke = Stroke::new(if is_current { 1.0 + hover_t * 0.3 } else { hover_t * 1.0 }, border_color);

                                    let row_resp = egui::Frame::new()
                                        .fill(bg_color)
                                        .stroke(stroke)
                                        .corner_radius(6.0)
                                        .inner_margin(egui::Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Track number / Play indicator
                                                ui.allocate_ui(Vec2::new(28.0, 34.0), |ui| {
                                                    ui.centered_and_justified(|ui| {
                                                        if is_current {
                                                            ui.colored_label(Color32::from_rgb(5, 150, 105), "▶");
                                                        } else {
                                                            ui.label(egui::RichText::new(format!("{:02}", idx + 1)).strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                                        }
                                                    });
                                                });

                                                // Cover Art Thumbnail
                                                ui.allocate_ui(Vec2::new(34.0, 34.0), |ui| {
                                                    let cover_key = if !song.album.trim().is_empty() {
                                                        &song.album
                                                    } else {
                                                        song.file_path.as_deref().unwrap_or(&song.title)
                                                    };
                                                    let thumb_tex = self.get_album_cover_texture(ui.ctx(), cover_key, song.file_path.as_deref());
                                                    let (thumb_rect, _) = ui.allocate_exact_size(Vec2::new(30.0, 30.0), egui::Sense::hover());
                                                    let painter = ui.painter_at(thumb_rect);
                                                    if let Some(ref tex) = thumb_tex {
                                                        painter.rect_filled(thumb_rect, 4.0, Color32::from_rgb(15, 23, 42));
                                                        painter.image(
                                                            tex.id(),
                                                            thumb_rect,
                                                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                                            Color32::WHITE,
                                                        );
                                                        painter.rect_stroke(thumb_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);
                                                    } else {
                                                        painter.rect_filled(thumb_rect, 4.0, Color32::from_rgb(241, 245, 249));
                                                        painter.rect_stroke(thumb_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Outside);
                                                        painter.circle_filled(thumb_rect.center(), 10.0, Color32::from_rgb(30, 41, 59));
                                                        painter.circle_filled(thumb_rect.center(), 3.5, Color32::from_rgb(14, 165, 233));
                                                    }
                                                });

                                                // Favorite Heart Toggle
                                                ui.allocate_ui(Vec2::new(28.0, 34.0), |ui| {
                                                    ui.centered_and_justified(|ui| {
                                                        let icon = if song.is_favorite { "❤️" } else { "🤍" };
                                                        if ui.button(egui::RichText::new(icon).size(13.0)).clicked() {
                                                            track_to_toggle_fav = Some((song.title.clone(), song.artist.clone()));
                                                        }
                                                    });
                                                });

                                                // Title & Artist
                                                ui.allocate_ui(Vec2::new(290.0, 34.0), |ui| {
                                                    ui.vertical(|ui| {
                                                        let title_color = if is_current { Color32::from_rgb(5, 150, 105) } else { Color32::from_rgb(15, 23, 42) };
                                                        ui.add(egui::Label::new(egui::RichText::new(&song.title).strong().size(13.5).color(title_color)).truncate());
                                                        ui.add(egui::Label::new(egui::RichText::new(&song.artist).size(11.5).color(Color32::from_rgb(71, 85, 105))).truncate());
                                                    });
                                                });

                                                // Album
                                                ui.allocate_ui(Vec2::new(220.0, 34.0), |ui| {
                                                    ui.add(egui::Label::new(egui::RichText::new(&song.album).size(12.0).color(Color32::from_rgb(51, 65, 85))).truncate());
                                                });

                                                // Format / Quality Badge
                                                ui.allocate_ui(Vec2::new(170.0, 34.0), |ui| {
                                                    let is_hi_res = song.sample_rate >= 88200 || song.bit_depth >= 24;
                                                    let fmt = song.format.to_uppercase();
                                                    let (badge_bg, badge_border, badge_text) = if is_hi_res {
                                                        (Color32::from_rgb(236, 253, 245), Color32::from_rgb(167, 243, 208), Color32::from_rgb(4, 120, 87))
                                                    } else if fmt == "FLAC" || fmt == "ALAC" {
                                                        (Color32::from_rgb(239, 246, 255), Color32::from_rgb(191, 219, 254), Color32::from_rgb(29, 78, 216))
                                                    } else if fmt == "WAV" || fmt == "AIFF" || fmt == "DSD" || fmt == "DSF" {
                                                        (Color32::from_rgb(245, 243, 255), Color32::from_rgb(221, 214, 254), Color32::from_rgb(109, 40, 217))
                                                    } else {
                                                        (Color32::from_rgb(241, 245, 249), Color32::from_rgb(226, 232, 240), Color32::from_rgb(71, 85, 105))
                                                    };

                                                    let rate_khz = song.sample_rate as f32 / 1000.0;
                                                    let quality_str = if song.bit_depth > 0 {
                                                        format!("{} {:.1}k/{}b", song.format, rate_khz, song.bit_depth)
                                                    } else if let Some(br) = song.bitrate_kbps {
                                                        format!("{} {}k", song.format, br)
                                                    } else {
                                                        format!("{} {:.1}k", song.format, rate_khz)
                                                    };

                                                    egui::Frame::new()
                                                        .fill(badge_bg)
                                                        .stroke(Stroke::new(1.0, badge_border))
                                                        .corner_radius(4.0)
                                                        .inner_margin(egui::Margin::symmetric(7, 3))
                                                        .show(ui, |ui| {
                                                            ui.label(egui::RichText::new(quality_str).size(10.5).strong().color(badge_text));
                                                        });
                                                });

                                                // File size & Duration
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui.label(egui::RichText::new(format_time(song.duration)).strong().size(12.5).color(Color32::from_rgb(30, 41, 59)));
                                                    ui.add_space(20.0);
                                                    if song.file_size_mb > 0.0 {
                                                        ui.label(egui::RichText::new(format!("{:.1} MB", song.file_size_mb)).size(11.5).color(Color32::from_rgb(71, 85, 105)));
                                                    } else {
                                                        ui.label(egui::RichText::new("-").size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                                    }
                                                });
                                            });
                                        });

                                    ui.add_space(2.0);

                                    // Click row to play
                                    let row_click = ui.interact(row_resp.response.rect, ui.scope_id().with(("lib_row", idx)), egui::Sense::click());
                                    if row_click.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if row_click.clicked() {
                                        track_to_play = Some(song.clone());
                                    }

                                    // Right click context menu on track row
                                    row_click.context_menu(|ui| {
                                        ui.label(egui::RichText::new(&song.title).strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                        ui.label(egui::RichText::new(format!("{} • {}", song.artist, song.album)).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                        ui.separator();

                                        if ui.button("▶ Putar Sekarang").clicked() {
                                            track_to_play = Some(song.clone());
                                            ui.close();
                                        }

                                        if ui.button("⏳ Putar Berikutnya (Play Next)").clicked() {
                                            track_play_next = Some(song.clone());
                                            ui.close();
                                        }

                                        if ui.button("➕ Tambahkan ke Antrean").clicked() {
                                            track_add_to_queue = Some(song.clone());
                                            ui.close();
                                        }

                                        ui.separator();

                                        let fav_text = if song.is_favorite { "🤍 Hapus dari Favorit" } else { "❤️ Tambah ke Favorit" };
                                        if ui.button(fav_text).clicked() {
                                            track_to_toggle_fav = Some((song.title.clone(), song.artist.clone()));
                                            ui.close();
                                        }

                                        let track_id = song.file_path.clone().unwrap_or_else(|| format!("{} - {}", song.artist, song.title));
                                        ui.menu_button("🎶 Tambahkan ke Playlist", |ui| {
                                            for pl in &self.custom_playlists {
                                                if ui.button(&pl.name).clicked() {
                                                    track_add_to_playlist = Some((pl.id, track_id.clone()));
                                                    ui.close();
                                                }
                                            }
                                        });

                                        if let Some(ref pl_name) = self.selected_playlist_filter {
                                            if ui.button(format!("🗑️ Hapus dari Playlist \"{}\"", pl_name)).clicked() {
                                                track_remove_from_playlist = Some((pl_name.clone(), track_id));
                                                ui.close();
                                            }
                                        }

                                        ui.separator();

                                        if ui.button("🗑️ Hapus Lagu dari Library").clicked() {
                                            track_to_delete_from_library = Some((song.title.clone(), song.artist.clone()));
                                            ui.close();
                                        }

                                        if let Some(ref path) = song.file_path {
                                            if ui.button("📂 Buka Lokasi File di Explorer").clicked() {
                                                open_file_in_explorer(path);
                                                ui.close();
                                            }
                                        }
                                    });

                                    ui.separator();
                                    }
                                });
                            }
                        }
                    }
                }
            });
        });

        // Execute deferred mutations cleanly
        let mut had_mutations = clear_entire_library;

        if clear_entire_library {
            self.playlist.clear();
            self.current_track.title = "Belum Ada Lagu Diputar".to_string();
            self.current_track.artist = "Lunula Audio Engine Ready".to_string();
            self.current_track.album = "Silakan Buka Folder / File Musik".to_string();
            self.current_track.duration = 0.0;
            self.current_time = 0.0;
            self.is_playing = false;
            if let Ok(mut c) = self.album_cover_cache.lock() {
                c.clear();
            }
        }

        if let Some(track) = track_to_play {
            self.queue_cursor = None;
            self.play_track(track);
        }

        if let Some(track) = track_play_next {
            let insert_pos = match self.queue_cursor {
                Some(cur) => (cur + 1).min(self.play_queue.len()),
                None => 0,
            };
            self.play_queue.insert(insert_pos, track);
            had_mutations = true;
        }

        if let Some(track) = track_add_to_queue {
            self.play_queue.push(track);
            had_mutations = true;
        }

        if let Some(tracks) = tracks_to_queue_bulk {
            self.play_queue.extend(tracks);
            had_mutations = true;
        }

        if let Some((title, artist)) = track_to_toggle_fav {
            had_mutations = true;
            if self.current_track.title == title && self.current_track.artist == artist {
                self.current_track.is_favorite = !self.current_track.is_favorite;
            }
            for t in self.playlist.iter_mut() {
                if t.title == title && t.artist == artist {
                    t.is_favorite = !t.is_favorite;
                }
            }
        }

        if let Some((del_title, del_artist)) = track_to_delete_from_library {
            had_mutations = true;
            self.playlist.retain(|t| !(t.title == del_title && t.artist == del_artist));
            let track_identifier = format!("{} - {}", del_artist, del_title);
            for pl in &mut self.custom_playlists {
                pl.track_paths.retain(|p| !p.contains(&del_title) && p != &track_identifier);
            }
            if self.current_track.title == del_title && self.current_track.artist == del_artist {
                if let Some(first) = self.playlist.first() {
                    self.current_track = first.clone();
                    self.current_time = 0.0;
                } else {
                    self.current_track.title = "Belum Ada Lagu Diputar".to_string();
                    self.current_track.artist = "Lunula Audio Engine Ready".to_string();
                    self.current_track.album = "Silakan Buka Folder / File Musik".to_string();
                    self.current_track.duration = 0.0;
                    self.current_time = 0.0;
                    self.is_playing = false;
                }
            }
        }

        if let Some((pl_name, track_id)) = track_remove_from_playlist {
            had_mutations = true;
            if let Some(pl) = self.custom_playlists.iter_mut().find(|p| p.name == pl_name) {
                pl.track_paths.retain(|p| p != &track_id);
            }
        }

        if let Some(del_album) = album_to_delete_from_library {
            had_mutations = true;
            self.playlist.retain(|t| t.album != del_album);
            if self.current_track.album == del_album {
                if let Some(first) = self.playlist.first() {
                    self.current_track = first.clone();
                    self.current_time = 0.0;
                } else {
                    self.current_track.title = "Belum Ada Lagu Diputar".to_string();
                    self.current_track.artist = "Lunula Audio Engine Ready".to_string();
                    self.current_track.album = "Silakan Buka Folder / File Musik".to_string();
                    self.current_track.duration = 0.0;
                    self.current_time = 0.0;
                    self.is_playing = false;
                }
            }
            if self.selected_album_filter == Some(del_album) {
                self.selected_album_filter = None;
            }
        }

        if let Some(del_artist) = artist_to_delete_from_library {
            had_mutations = true;
            self.playlist.retain(|t| t.artist != del_artist);
            if self.current_track.artist == del_artist {
                if let Some(first) = self.playlist.first() {
                    self.current_track = first.clone();
                    self.current_time = 0.0;
                } else {
                    self.current_track.title = "Belum Ada Lagu Diputar".to_string();
                    self.current_track.artist = "Lunula Audio Engine Ready".to_string();
                    self.current_track.album = "Silakan Buka Folder / File Musik".to_string();
                    self.current_track.duration = 0.0;
                    self.current_time = 0.0;
                    self.is_playing = false;
                }
            }
            if self.selected_artist_filter == Some(del_artist) {
                self.selected_artist_filter = None;
            }
        }

        if let Some((pl_id, track_id)) = track_add_to_playlist {
            had_mutations = true;
            if let Some(pl) = self.custom_playlists.iter_mut().find(|p| p.id == pl_id) {
                if !pl.track_paths.contains(&track_id) {
                    pl.track_paths.push(track_id);
                }
            }
        }

        if let Some(pl_id) = playlist_to_delete {
            had_mutations = true;
            self.custom_playlists.retain(|p| p.id != pl_id);
        }

        if let Some(alb) = set_album_filter {
            self.selected_album_filter = Some(alb);
        }
        if let Some(art) = set_artist_filter {
            self.selected_artist_filter = Some(art);
        }
        if let Some(q) = set_quality_filter {
            self.selected_quality_filter = Some(q);
        }
        if let Some(pl) = set_playlist_filter {
            self.selected_playlist_filter = Some(pl);
        }
        if switch_to_all_tracks {
            self.library_category = LibraryCategory::AllTracks;
        }

        if had_mutations {
            self.save_library_state();
        }
    }
}
