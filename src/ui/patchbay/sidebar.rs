use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::NodeType;
use crate::app::LunulaApp;
use crate::ui::components::render_card_frame;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FilterCategory {
    All,
    Tonal,    // Low-Pass Filter, Saved EQ
    Color,    // Tube Warmer, Analog Preamp
    Dynamics, // Optical Leveler, Bass Guard, Dynamic Loudness, Attenuator, Booster, Stepped
    Spatial,  // Stereo Widener, Stereo Balance
    Routing,  // Splitter, Mixer
}

struct FilterPaletteDef {
    node_type: NodeType,
    category: FilterCategory,
    icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    friendly_desc: &'static str,
    accent_color: Color32,
    bg_accent: Color32,
    keywords: &'static [&'static str],
    is_eq_preset: bool,
    is_coming_soon: bool,
}

impl LunulaApp {
    /// Renders the compact, searchable, and beginner-friendly FX & Filter Palette sidebar.
    pub fn render_fx_sidebar_palette(&mut self, ui: &mut egui::Ui) {
        let mut delete_filter_type: Option<NodeType> = None;
        let mut delete_all_saved_eq_nodes = false;
        let mut add_filter_type: Option<NodeType> = None;
        let mut switch_to_node_id: Option<usize> = None;
        let mut switch_to_tutorial: Option<NodeType> = None;
        let mut open_eq_popup_pos: Option<Pos2> = None;

        let palette_defs: [FilterPaletteDef; 14] = [
            // 1. Tonal & EQ
            FilterPaletteDef {
                node_type: NodeType::LowPassFilter,
                category: FilterCategory::Tonal,
                icon: "🌊",
                title: "Low-Pass Filter",
                subtitle: "Peredam treble / suara tajam",
                friendly_desc: "Memotong frekuensi tinggi (treble). Suara vokal dan instrumen menjadi lebih empuk, hangat, atau teredam halus seperti efek underwater.",
                accent_color: Color32::from_rgb(14, 165, 233),
                bg_accent: Color32::from_rgb(240, 249, 255),
                keywords: &["lpf", "lowpass", "treble", "cut", "potong", "muffled", "underwater", "lembut", "filter"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
            FilterPaletteDef {
                node_type: NodeType::SavedEqPreset("Default".to_string()),
                category: FilterCategory::Tonal,
                icon: "🎛️",
                title: "Equalizer Presets",
                subtitle: "Koleksi kurva EQ kustom",
                friendly_desc: "Mengatur profil frekuensi bass, vokal, dan treble dengan preset favorit Anda (Vocal Boost, Bass Boost, Flat, dll).",
                accent_color: Color32::from_rgb(168, 85, 247),
                bg_accent: Color32::from_rgb(250, 245, 255),
                keywords: &["eq", "equalizer", "preset", "vocal", "bass", "treble", "frekuensi", "kurva"],
                is_eq_preset: true,
                is_coming_soon: false,
            },

            // 2. Karakter & Tabung
            FilterPaletteDef {
                node_type: NodeType::TubeWarmer,
                category: FilterCategory::Color,
                icon: "🔥",
                title: "Tube Warmer",
                subtitle: "Saturasi tabung vakum vintage",
                friendly_desc: "Menghangatkan rekaman audio digital dengan distorsi harmonik genap yang tebal dan lembut khas ampli tabung analog klasik.",
                accent_color: Color32::from_rgb(245, 158, 11),
                bg_accent: Color32::from_rgb(254, 243, 199),
                keywords: &["tube", "warm", "saturasi", "tabung", "hangat", "analog", "vintage", "drive"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
            FilterPaletteDef {
                node_type: NodeType::AnalogPreamp,
                category: FilterCategory::Color,
                icon: "📻",
                title: "Analog Preamp",
                subtitle: "Karakter konsol studio analog",
                friendly_desc: "Memberikan bobot tekstur dan ketebalan suara ala preamp konsol rekaman legendaris tanpa mengubah nada musik.",
                accent_color: Color32::from_rgb(217, 119, 6),
                bg_accent: Color32::from_rgb(254, 243, 199),
                keywords: &["preamp", "analog", "konsol", "tebal", "bobot", "vintage", "karakter"],
                is_eq_preset: false,
                is_coming_soon: false,
            },

            // 3. Dinamika & Level
            FilterPaletteDef {
                node_type: NodeType::OpticalLeveler,
                category: FilterCategory::Dynamics,
                icon: "🎚️",
                title: "Optical Leveler",
                subtitle: "Kompresor optik penstabil volume",
                friendly_desc: "Secara otomatis menyeimbangkan bagian lagu yang terlalu keras dan terlalu pelan agar terdengar rata, mantap, dan nyaman di telinga.",
                accent_color: Color32::from_rgb(234, 179, 8),
                bg_accent: Color32::from_rgb(254, 252, 232),
                keywords: &["compressor", "leveler", "kompresor", "rata", "stabil", "opto", "dinamika"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
            FilterPaletteDef {
                node_type: NodeType::BassGuard,
                category: FilterCategory::Dynamics,
                icon: "🛡️",
                title: "Bass Guard Limiter",
                subtitle: "Pelindung speaker dari distorsi bass",
                friendly_desc: "Meredam getaran sub-bass ekstrem yang tidak terdengar agar bass tetap bertenaga tanpa membuat kerucut speaker bergetar liar atau pecah.",
                accent_color: Color32::from_rgb(148, 163, 184),
                bg_accent: Color32::from_rgb(241, 245, 249),
                keywords: &["bass", "guard", "sub", "lindung", "speaker", "distorsi", "pecah", "low", "limiter"],
                is_eq_preset: false,
                is_coming_soon: true,
            },
            FilterPaletteDef {
                node_type: NodeType::DynamicLoudness,
                category: FilterCategory::Dynamics,
                icon: "🌙",
                title: "Dynamic Loudness",
                subtitle: "Kompensasi dengar volume rendah",
                friendly_desc: "Menaikkan bass dan treble secara cerdas saat mendengarkan musik di malam hari atau volume kecil (standar ISO 226).",
                accent_color: Color32::from_rgb(99, 102, 241),
                bg_accent: Color32::from_rgb(238, 242, 255),
                keywords: &["loudness", "dynamic", "malam", "pelan", "rendah", "fletcher", "munson", "bass", "treble"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
            FilterPaletteDef {
                node_type: NodeType::VolumeAttenuator,
                category: FilterCategory::Dynamics,
                icon: "🔉",
                title: "Volume Attenuator",
                subtitle: "Peredam gain digital presisi",
                friendly_desc: "Menurunkan gain sinyal sebesar beberapa dB untuk memberi ruang headroom sebelum sinyal masuk ke efek yang sensitif.",
                accent_color: Color32::from_rgb(148, 163, 184),
                bg_accent: Color32::from_rgb(241, 245, 249),
                keywords: &["attenuator", "gain", "redam", "turun", "volume", "headroom", "kecil"],
                is_eq_preset: false,
                is_coming_soon: true,
            },
            FilterPaletteDef {
                node_type: NodeType::VolumeBooster,
                category: FilterCategory::Dynamics,
                icon: "🔊",
                title: "Volume Booster",
                subtitle: "Penguat volume lagu pelan",
                friendly_desc: "Menaikkan volume master rekaman lagu yang suaranya terlalu kecil dengan algoritma soft-limiting transparan tanpa distorsi clipping.",
                accent_color: Color32::from_rgb(239, 68, 68),
                bg_accent: Color32::from_rgb(254, 242, 242),
                keywords: &["booster", "boost", "angkat", "keras", "volume", "ampli", "penguat"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
            FilterPaletteDef {
                node_type: NodeType::SteppedAttenuator,
                category: FilterCategory::Dynamics,
                icon: "🪜",
                title: "Stepped Attenuator",
                subtitle: "Volume bertingkat + tombol DIM",
                friendly_desc: "Pengatur volume bertingkat presisi audiophile lengkap dengan tombol instan -20dB DIM untuk mengecilkan suara seketika.",
                accent_color: Color32::from_rgb(71, 85, 105),
                bg_accent: Color32::from_rgb(241, 245, 249),
                keywords: &["stepped", "dim", "tangga", "tingkat", "resistor", "audiophile", "mute"],
                is_eq_preset: false,
                is_coming_soon: false,
            },

            // 4. Spasial & Stereo
            FilterPaletteDef {
                node_type: NodeType::StereoWidener,
                category: FilterCategory::Spatial,
                icon: "↔️",
                title: "Stereo Widener",
                subtitle: "Pelebar panggung suara stereo",
                friendly_desc: "Melebarkan pemisahan instrumen kiri dan kanan sehingga musik terasa lebih luas, megah, dan 3D di telinga Anda.",
                accent_color: Color32::from_rgb(147, 51, 234),
                bg_accent: Color32::from_rgb(250, 245, 255),
                keywords: &["widener", "stereo", "lebar", "spatial", "panggung", "soundstage", "3d", "luas"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
            FilterPaletteDef {
                node_type: NodeType::StereoBalance,
                category: FilterCategory::Spatial,
                icon: "⚖️",
                title: "Stereo Balance",
                subtitle: "Keseimbangan kanal Kiri & Kanan",
                friendly_desc: "Menggeser titik fokus audio lebih ke telinga kiri atau kanan untuk mengoreksi posisi speaker atau kebiasaan dengar.",
                accent_color: Color32::from_rgb(59, 130, 246),
                bg_accent: Color32::from_rgb(239, 246, 255),
                keywords: &["balance", "keseimbangan", "kiri", "kanan", "left", "right", "pan", "geser"],
                is_eq_preset: false,
                is_coming_soon: false,
            },

            // 5. Routing & Cabang
            FilterPaletteDef {
                node_type: NodeType::Splitter,
                category: FilterCategory::Routing,
                icon: "🔀",
                title: "Signal Splitter",
                subtitle: "Pencabang 1 sinyal ke 2 jalur",
                friendly_desc: "Membagi satu kabel input menjadi dua kabel output terpisah untuk pemrosesan paralel (misal: satu ke tabung, satu tetap bersih).",
                accent_color: Color32::from_rgb(16, 185, 129),
                bg_accent: Color32::from_rgb(236, 253, 245),
                keywords: &["splitter", "split", "cabang", "paralel", "bagi", "dua", "mult", "jalur"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
            FilterPaletteDef {
                node_type: NodeType::Mixer,
                category: FilterCategory::Routing,
                icon: "🎛️",
                title: "A/B Signal Mixer",
                subtitle: "Penggabung 2 jalur audio (Pin A & B)",
                friendly_desc: "Menggabungkan dua cabang kabel (Pin A dan Pin B) kembali menjadi satu dengan slider crossfade persentase halus.",
                accent_color: Color32::from_rgb(13, 148, 136),
                bg_accent: Color32::from_rgb(240, 253, 250),
                keywords: &["mixer", "mix", "gabung", "crossfade", "summing", "a", "b", "blend", "kombinasi"],
                is_eq_preset: false,
                is_coming_soon: false,
            },
        ];

        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                // 1. Header Ringkas
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🎛️ FX TOOLBOX").strong().size(15.0).color(Color32::from_rgb(15, 23, 42)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Klik untuk tambah").size(11.5).color(Color32::from_rgb(148, 163, 184)));
                    });
                });

                ui.add_space(4.0);

                // 2. Search Bar Interaktif
                let has_search_text = !self.patchbay_filter_search.is_empty();
                let clear_btn_w = if has_search_text { 24.0 } else { 0.0 };

                ui.horizontal(|ui| {
                    let search_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.patchbay_filter_search)
                            .hint_text("🔍 Cari filter... (mis: tube, eq, bass)")
                            .desired_width(ui.available_width() - clear_btn_w),
                    );

                    // Clear button jika ada teks pencarian
                    if has_search_text {
                        if ui.button(egui::RichText::new("✕").size(11.5).color(Color32::from_rgb(148, 163, 184))).on_hover_text("Bersihkan pencarian").clicked() {
                            self.patchbay_filter_search.clear();
                        }
                    }

                    if search_resp.changed() {
                        ui.ctx().request_repaint();
                    }
                });

                ui.add_space(4.0);

                // 3. Category Filter Chips (Pills Horisontal)
                let categories = [
                    (FilterCategory::All, "Semua"),
                    (FilterCategory::Tonal, "🎛️ Tonal"),
                    (FilterCategory::Color, "🔥 Tabung"),
                    (FilterCategory::Dynamics, "🔊 Dinamika"),
                    (FilterCategory::Spatial, "↔️ Spasial"),
                    (FilterCategory::Routing, "🔀 Routing"),
                ];

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.5, 4.0);
                    for (idx, (_cat, label)) in categories.iter().enumerate() {
                        let is_sel = self.patchbay_category_idx == idx;
                        let chip_color = if is_sel {
                            Color32::from_rgb(79, 70, 229)
                        } else {
                            Color32::from_rgb(241, 245, 249)
                        };
                        let text_color = if is_sel {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(71, 85, 105)
                        };

                        let btn = egui::Button::new(egui::RichText::new(*label).size(11.5).strong().color(text_color))
                            .fill(chip_color)
                            .corner_radius(4.0)
                            .stroke(Stroke::NONE);

                        if ui.add(btn).clicked() {
                            self.patchbay_category_idx = idx;
                        }
                    }
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // 4. Daftar Filter (Compact Tiles List)
                let search_lower = self.patchbay_filter_search.trim().to_lowercase();
                let active_category = categories[self.patchbay_category_idx].0;

                let sidebar_min_x = ui.min_rect().min.x;

                egui::ScrollArea::vertical()
                    .id_salt("patchbay_filter_tiles_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut match_count = 0;

                        for def in &palette_defs {
                            // Filter berdasarkan kategori
                            if active_category != FilterCategory::All && def.category != active_category {
                                continue;
                            }

                            // Filter berdasarkan teks pencarian
                            if !search_lower.is_empty() {
                                let title_match = def.title.to_lowercase().contains(&search_lower);
                                let subtitle_match = def.subtitle.to_lowercase().contains(&search_lower);
                                let desc_match = def.friendly_desc.to_lowercase().contains(&search_lower);
                                let kw_match = def.keywords.iter().any(|k| k.contains(&search_lower));

                                if !title_match && !subtitle_match && !desc_match && !kw_match {
                                    continue;
                                }
                            }

                            match_count += 1;

                            // Cek apakah node ini sudah ada di canvas
                            let canvas_nodes: Vec<usize> = if def.is_eq_preset {
                                self.nodes.iter().filter(|n| matches!(n.node_type, NodeType::SavedEqPreset(_))).map(|n| n.id).collect()
                            } else {
                                self.nodes.iter().filter(|n| n.node_type == def.node_type).map(|n| n.id).collect()
                            };
                            let count_on_canvas = canvas_nodes.len();

                            // Render compact card row (~48px height for effortless readability)
                            let row_height = 48.0;
                            let (row_rect, row_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), row_height), egui::Sense::click());
                            let r_painter = ui.painter_at(row_rect);

                            let is_hovered = row_resp.hovered() && !def.is_coming_soon;
                            let bg_color = if def.is_coming_soon {
                                Color32::from_rgb(248, 250, 252)
                            } else if is_hovered {
                                Color32::from_rgb(248, 250, 252)
                            } else {
                                Color32::WHITE
                            };

                            let border_color = if def.is_coming_soon {
                                Color32::from_rgb(226, 232, 240)
                            } else if is_hovered {
                                def.accent_color
                            } else {
                                Color32::from_rgb(226, 232, 240)
                            };

                            r_painter.rect_filled(row_rect, 6.0, bg_color);
                            r_painter.rect_stroke(row_rect, 6.0, Stroke::new(if is_hovered { 1.2 } else { 0.8 }, border_color), egui::StrokeKind::Inside);

                            // Left Icon Badge (30x30px)
                            let icon_rect = Rect::from_center_size(Pos2::new(row_rect.min.x + 24.0, row_rect.center().y), Vec2::splat(30.0));
                            let icon_bg = if def.is_coming_soon { Color32::from_rgb(241, 245, 249) } else { def.bg_accent };
                            let icon_fg = if def.is_coming_soon { Color32::from_rgb(148, 163, 184) } else { def.accent_color };
                            r_painter.rect_filled(icon_rect, 5.0, icon_bg);
                            r_painter.rect_stroke(icon_rect, 5.0, Stroke::new(1.0, icon_fg.gamma_multiply(0.35)), egui::StrokeKind::Inside);

                            if def.node_type == NodeType::StereoWidener {
                                let c = icon_rect.center();
                                for r in [3.5f32, 7.0, 10.5] {
                                    r_painter.line_segment([Pos2::new(c.x - r, c.y - 5.0), Pos2::new(c.x - r, c.y + 5.0)], Stroke::new(1.3, icon_fg));
                                    r_painter.line_segment([Pos2::new(c.x + r, c.y - 5.0), Pos2::new(c.x + r, c.y + 5.0)], Stroke::new(1.3, icon_fg));
                                }
                                r_painter.circle_filled(c, 2.5, icon_fg);
                            } else {
                                r_painter.text(icon_rect.center(), egui::Align2::CENTER_CENTER, def.icon, egui::FontId::proportional(16.0), icon_fg);
                            }

                            // Center Text Area
                            let text_left = row_rect.min.x + 46.0;
                            let text_right = row_rect.max.x - 76.0;
                            let text_top = row_rect.min.y + 7.0;

                            // Baris 1: Judul + Status Badge
                            let title_color = if def.is_coming_soon { Color32::from_rgb(100, 116, 139) } else { Color32::from_rgb(15, 23, 42) };
                            r_painter.text(Pos2::new(text_left, text_top), egui::Align2::LEFT_TOP, def.title, egui::FontId::proportional(13.0), title_color);

                            if def.is_coming_soon {
                                let badge_pos = Pos2::new(text_right - 4.0, text_top + 1.0);
                                r_painter.text(badge_pos, egui::Align2::RIGHT_TOP, "🔒 Coming Soon", egui::FontId::proportional(10.5), Color32::from_rgb(217, 119, 6));
                            } else if count_on_canvas > 0 {
                                let badge_text = if count_on_canvas == 1 { "✓ Di Canvas".to_string() } else { format!("{}x di Canvas", count_on_canvas) };
                                let badge_pos = Pos2::new(text_right - 4.0, text_top + 1.0);
                                r_painter.text(badge_pos, egui::Align2::RIGHT_TOP, badge_text, egui::FontId::monospace(10.5), Color32::from_rgb(16, 185, 129));
                            }

                            // Baris 2: Subtitle ramah pemula
                            let sub_text = if def.is_coming_soon { "Segera hadir dalam pembaruan berikutnya" } else { def.subtitle };
                            let sub_color = if def.is_coming_soon { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) };
                            r_painter.text(Pos2::new(text_left, text_top + 17.0), egui::Align2::LEFT_TOP, sub_text, egui::FontId::proportional(11.0), sub_color);

                            if def.is_coming_soon {
                                // Locked badge
                                let cs_w = 66.0;
                                let cs_h = 24.0;
                                let cs_rect = Rect::from_center_size(Pos2::new(row_rect.max.x - 38.0, row_rect.center().y), Vec2::new(cs_w, cs_h));
                                r_painter.rect_filled(cs_rect, 4.0, Color32::from_rgb(254, 243, 199));
                                r_painter.rect_stroke(cs_rect, 4.0, Stroke::new(0.8, Color32::from_rgb(251, 191, 36)), egui::StrokeKind::Inside);
                                r_painter.text(cs_rect.center(), egui::Align2::CENTER_CENTER, "🔒 Soon", egui::FontId::proportional(10.5), Color32::from_rgb(180, 83, 9));
                            } else {
                                // Right Action 1: Tombol "? Panduan"
                                let help_w = 20.0;
                                let help_h = 24.0;
                                let help_rect = Rect::from_center_size(Pos2::new(row_rect.max.x - 58.0, row_rect.center().y), Vec2::new(help_w, help_h));
                                let help_resp = ui.interact(help_rect, ui.scope_id().with(("help_fx_btn", def.title)), egui::Sense::click())
                                    .on_hover_text(format!("Buka panduan bergambar & cara kerja {}", def.title));

                                let help_bg = if help_resp.hovered() { Color32::from_rgb(224, 231, 255) } else { Color32::from_rgb(241, 245, 249) };
                                let help_fg = if help_resp.hovered() { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(100, 116, 139) };

                                r_painter.rect_filled(help_rect, 4.0, help_bg);
                                r_painter.text(help_rect.center(), egui::Align2::CENTER_CENTER, "?", egui::FontId::proportional(12.0), help_fg);

                                if help_resp.clicked() {
                                    switch_to_tutorial = Some(def.node_type.clone());
                                }

                                // Right Action 2: Tombol "+ Tambah"
                                let btn_w = 46.0;
                                let btn_h = 24.0;
                                let btn_rect = Rect::from_center_size(Pos2::new(row_rect.max.x - 24.0, row_rect.center().y), Vec2::new(btn_w, btn_h));
                                let btn_resp = ui.interact(btn_rect, ui.scope_id().with(("add_fx_btn", def.title)), egui::Sense::click());

                                let btn_bg = if btn_resp.hovered() { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(241, 245, 249) };
                                let btn_fg = if btn_resp.hovered() { Color32::WHITE } else { Color32::from_rgb(79, 70, 229) };

                                r_painter.rect_filled(btn_rect, 4.0, btn_bg);
                                r_painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "+ Tambah", egui::FontId::proportional(11.0), btn_fg);

                                if def.node_type == NodeType::TubeWarmer {
                                    self.quest_rect_tube_warmer_btn = Some(row_rect);
                                }

                                // Interaksi Tambah
                                if row_resp.clicked() || btn_resp.clicked() {
                                    if def.is_eq_preset {
                                        self.show_saved_eq_palette_popup = !self.show_saved_eq_palette_popup;
                                        open_eq_popup_pos = Some(Pos2::new(sidebar_min_x, row_rect.center().y));
                                    } else {
                                        add_filter_type = Some(def.node_type.clone());
                                    }
                                }
                            }

                            // Context menu pada card
                            row_resp.context_menu(|ui| {
                                if def.is_coming_soon {
                                    ui.label(egui::RichText::new(format!("{} {}", def.icon, def.title)).strong().size(12.0));
                                    ui.label(egui::RichText::new("🔒 Modul Sedang Dikembangkan (Coming Soon)").size(10.5).color(Color32::from_rgb(217, 119, 6)));
                                    ui.separator();
                                    ui.label(egui::RichText::new("Filter ini sedang disiapkan dan belum dapat dimasukkan ke canvas.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                } else {
                                    ui.label(egui::RichText::new(format!("{} {}", def.icon, def.title)).strong().size(12.0));
                                    ui.label(egui::RichText::new(def.subtitle).size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    ui.separator();

                                    if def.is_eq_preset {
                                        if ui.button("🎛️ Buka Koleksi Preset EQ").clicked() {
                                            self.show_saved_eq_palette_popup = true;
                                            self.saved_eq_popup_anchor = Some(Pos2::new(sidebar_min_x, row_rect.center().y));
                                            ui.close();
                                        }
                                    } else {
                                        if ui.button("➕ Tambah ke Canvas").clicked() {
                                            add_filter_type = Some(def.node_type.clone());
                                            ui.close();
                                        }
                                    }

                                    if ui.button("📖 Buka Panduan & Cara Kerja").clicked() {
                                        switch_to_tutorial = Some(def.node_type.clone());
                                        ui.close();
                                    }

                                    if count_on_canvas > 0 {
                                        ui.separator();
                                        let del_label = if count_on_canvas == 1 { "🗑️ Hapus dari Canvas".to_string() } else { format!("🗑️ Hapus Semua ({}x) dari Canvas", count_on_canvas) };
                                        if ui.button(egui::RichText::new(del_label).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                            if def.is_eq_preset {
                                                delete_all_saved_eq_nodes = true;
                                            } else {
                                                delete_filter_type = Some(def.node_type.clone());
                                            }
                                            ui.close();
                                        }
                                    }
                                }
                            });

                            // Tooltip penjelasan ramah pemula
                            let tooltip_text = if def.is_coming_soon {
                                format!(
                                    "{} {}\n{}\n\n🔒 STATUS: COMING SOON\nModul ini sedang disiapkan dan belum dapat dimasukkan ke dalam rangkaian kabel studio.",
                                    def.icon, def.title, def.subtitle
                                )
                            } else {
                                format!(
                                    "{} {}\n{}\n\n💡 Fungsi Audio:\n{}\n\n• Klik kartu atau '+ Tambah' untuk meletakkan modul ke kanvas.\n• Klik kanan untuk opsi manajemen.",
                                    def.icon, def.title, def.subtitle, def.friendly_desc
                                )
                            };
                            row_resp.on_hover_text(tooltip_text);

                            // Jika ada lebih dari 1 kartu di kanvas, tampilkan indikator seleksi instan
                            if count_on_canvas > 1 {
                                ui.horizontal(|ui| {
                                    ui.add_space(46.0);
                                    ui.label(egui::RichText::new("Pilih di Canvas:").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                                    for (i, node_id) in canvas_nodes.iter().enumerate() {
                                        let is_sel = self.selected_node == Some(*node_id);
                                        let chip_btn = egui::Button::new(egui::RichText::new(format!("#{}", i + 1)).size(10.5).color(if is_sel { Color32::WHITE } else { Color32::from_rgb(71, 85, 105) }))
                                            .fill(if is_sel { def.accent_color } else { Color32::from_rgb(241, 245, 249) })
                                            .corner_radius(3.0);
                                        if ui.add(chip_btn).clicked() {
                                            switch_to_node_id = Some(*node_id);
                                        }
                                    }
                                });
                            }

                            ui.add_space(4.0);
                        }

                        // Pesan jika pencarian tidak menemukan hasil
                        if match_count == 0 {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("🔍 Tidak ada filter yang cocok").size(13.0).color(Color32::from_rgb(148, 163, 184)));
                                ui.label(egui::RichText::new(format!("Kata kunci: \"{}\"", self.patchbay_filter_search)).size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                if ui.button(egui::RichText::new("Reset Pencarian").size(12.0).color(Color32::from_rgb(79, 70, 229))).clicked() {
                                    self.patchbay_filter_search.clear();
                                    self.patchbay_category_idx = 0;
                                }
                            });
                        }
                    });
            });
        });

        // Eksekusi aksi
        if let Some(pos) = open_eq_popup_pos {
            self.saved_eq_popup_anchor = Some(pos);
        }

        if let Some(node_id) = switch_to_node_id {
            self.selected_node = Some(node_id);
            self.advance_studio_state.selected_cable_idx = None;
        }

        if let Some(node_type) = add_filter_type {
            self.add_modular_filter(node_type);
        }

        if let Some(tut_nt) = switch_to_tutorial {
            self.active_filter_tutorial = Some(tut_nt);
            self.filter_tutorial_step = 0;
        }

        // Hapus filter jika diminta dari context menu
        if let Some(ref nt) = delete_filter_type {
            let ids: Vec<usize> = self.nodes.iter().filter(|n| &n.node_type == nt && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            if let Some(sel) = self.selected_node {
                if ids.contains(&sel) {
                    self.selected_node = None;
                }
            }
            self.invalidate_chain_cache();
            self.sync_signal_chain_to_engine();
        }

        if delete_all_saved_eq_nodes {
            let ids: Vec<usize> = self.nodes.iter().filter(|n| matches!(n.node_type, NodeType::SavedEqPreset(_)) && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            if let Some(sel) = self.selected_node {
                if ids.contains(&sel) {
                    self.selected_node = None;
                }
            }
            self.invalidate_chain_cache();
            self.sync_signal_chain_to_engine();
        }

        // Floating popup untuk memilih preset EQ
        self.render_saved_eq_floating_popup(ui);
    }
}
