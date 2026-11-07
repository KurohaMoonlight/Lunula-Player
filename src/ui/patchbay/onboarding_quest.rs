use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{AppTab, ChainMode, NodeType, OnboardingQuestStep};
use crate::app::LunulaApp;

impl LunulaApp {
    /// Renders the action-gated, game-style interactive quest onboarding overlay
    pub fn render_onboarding_quest_overlay(&mut self, ui: &mut egui::Ui) {
        if !self.is_onboarding_quest_active {
            return;
        }

        let ctx = ui.ctx().clone();
        let screen_rect = ctx.input(|i| i.raw.screen_rect).unwrap_or_else(|| ui.clip_rect());
        let time = ctx.input(|i| i.time);

        // 1. Automatic Quest Progression Monitor (Action-Gated detection)
        match self.onboarding_quest_step {
            OnboardingQuestStep::NavigateToStudio => {
                if self.current_tab == AppTab::SignalChain {
                    self.onboarding_quest_step = OnboardingQuestStep::UnderstandModes;
                }
            }
            OnboardingQuestStep::UnderstandModes => {
                if self.chain_mode == ChainMode::DspActive {
                    self.onboarding_quest_step = OnboardingQuestStep::ConnectFirstCable;
                }
            }
            OnboardingQuestStep::ConnectFirstCable => {
                // Check if any cable connects from Buffer (id 2) or to DSP/DAC (id 3 or 4)
                let has_connected = self.connections.iter().any(|c| {
                    (c.from_node == 2 && (c.to_node == 3 || c.to_node == 4))
                        || (c.from_node == 3 && c.to_node == 4)
                });
                if has_connected {
                    self.onboarding_quest_step = OnboardingQuestStep::TestAutoMute;
                }
            }
            OnboardingQuestStep::TestAutoMute => {
                // Detected when the previous connection is severed
                let has_no_direct_connection = !self.connections.iter().any(|c| {
                    c.from_node == 2 && c.to_node == 4
                });
                if has_no_direct_connection && self.connections.len() < 3 {
                    self.onboarding_quest_step = OnboardingQuestStep::InsertTubeWarmer;
                }
            }
            OnboardingQuestStep::InsertTubeWarmer => {
                if self.nodes.iter().any(|n| n.node_type == NodeType::TubeWarmer) {
                    self.onboarding_quest_step = OnboardingQuestStep::Completed;
                }
            }
            _ => {}
        }

        // 2. Render Fullscreen Dimmed Layer with Target Spotlight
        let area_id = ui.scope_id().with("quest_spotlight_overlay");
        egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(&ctx, |ui| {
                let p = ui.painter();

                // Compute target spotlight zone based on active step
                let target_spotlight: Option<Rect> = match self.onboarding_quest_step {
                    OnboardingQuestStep::Welcome => None,
                    OnboardingQuestStep::NavigateToStudio => {
                        self.quest_rect_signal_chain_tab.or_else(|| {
                            Some(Rect::from_min_size(Pos2::new(screen_rect.max.x - 480.0, screen_rect.min.y + 10.0), Vec2::new(160.0, 36.0)))
                        })
                    }
                    OnboardingQuestStep::UnderstandModes => {
                        self.quest_rect_dsp_mode_btn.or_else(|| {
                            Some(Rect::from_min_size(Pos2::new(screen_rect.max.x - 340.0, screen_rect.min.y + 55.0), Vec2::new(140.0, 32.0)))
                        })
                    }
                    OnboardingQuestStep::ConnectFirstCable => {
                        // Central canvas cable area
                        Some(Rect::from_min_size(Pos2::new(screen_rect.min.x + 40.0, screen_rect.min.y + 90.0), Vec2::new(820.0, 320.0)))
                    }
                    OnboardingQuestStep::TestAutoMute => {
                        self.quest_rect_cable_mute_target.or_else(|| {
                            Some(Rect::from_min_size(Pos2::new(screen_rect.min.x + 220.0, screen_rect.min.y + 70.0), Vec2::new(260.0, 100.0)))
                        })
                    }
                    OnboardingQuestStep::InsertTubeWarmer => {
                        self.quest_rect_tube_warmer_btn.or_else(|| {
                            Some(Rect::from_min_size(Pos2::new(screen_rect.max.x - 260.0, screen_rect.min.y + 160.0), Vec2::new(240.0, 60.0)))
                        })
                    }
                    OnboardingQuestStep::Completed => None,
                };

                // Draw spotlight cut-out mask
                if let Some(target) = target_spotlight {
                    // Top mask
                    p.rect_filled(Rect::from_min_max(screen_rect.min, Pos2::new(screen_rect.max.x, target.min.y)), 0.0, Color32::from_black_alpha(150));
                    // Bottom mask
                    p.rect_filled(Rect::from_min_max(Pos2::new(screen_rect.min.x, target.max.y), screen_rect.max), 0.0, Color32::from_black_alpha(150));
                    // Left mask
                    p.rect_filled(Rect::from_min_max(Pos2::new(screen_rect.min.x, target.min.y), Pos2::new(target.min.x, target.max.y)), 0.0, Color32::from_black_alpha(150));
                    // Right mask
                    p.rect_filled(Rect::from_min_max(Pos2::new(target.max.x, target.min.y), Pos2::new(screen_rect.max.x, target.max.y)), 0.0, Color32::from_black_alpha(150));

                    // Glowing animated quest outline around target
                    let pulse = (time * 5.0).sin().abs() as f32;
                    let glow_color = Color32::from_rgba_premultiplied(
                        245,
                        (158.0 + 40.0 * pulse) as u8,
                        11,
                        (180.0 + 75.0 * pulse) as u8,
                    );
                    p.rect_stroke(target.expand(4.0), 8.0, Stroke::new(2.5, glow_color), egui::StrokeKind::Outside);
                } else {
                    p.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(165));
                }

                // 3. Floating Quest Dialog Card (Game HUD Style)
                let card_w = 490.0f32.min(screen_rect.width() - 32.0);
                let card_h = 360.0f32.min(screen_rect.height() - 32.0);

                let card_pos = match self.onboarding_quest_step {
                    OnboardingQuestStep::NavigateToStudio => {
                        Pos2::new(screen_rect.center().x - card_w * 0.5, screen_rect.min.y + 70.0)
                    }
                    OnboardingQuestStep::UnderstandModes => {
                        Pos2::new(screen_rect.min.x + 30.0, screen_rect.min.y + 110.0)
                    }
                    OnboardingQuestStep::ConnectFirstCable | OnboardingQuestStep::TestAutoMute => {
                        Pos2::new(screen_rect.min.x + 30.0, (screen_rect.max.y - card_h - 40.0).max(screen_rect.min.y + 50.0))
                    }
                    OnboardingQuestStep::InsertTubeWarmer => {
                        Pos2::new(screen_rect.min.x + 30.0, screen_rect.min.y + 120.0)
                    }
                    _ => {
                        Pos2::new(screen_rect.center().x - card_w * 0.5, screen_rect.center().y - card_h * 0.5)
                    }
                };

                let card_rect = Rect::from_min_size(card_pos, Vec2::new(card_w, card_h));
                let mut quest_ui = ui.new_child(egui::UiBuilder::new().max_rect(card_rect));

                egui::Frame::new()
                    .fill(Color32::from_rgb(15, 23, 42)) // Sleek Dark Studio Navy
                    .stroke(Stroke::new(2.0, Color32::from_rgb(245, 158, 11))) // Amber Gold Border
                    .corner_radius(12.0)
                    .inner_margin(16.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 8],
                        blur: 24,
                        spread: 0,
                        color: Color32::from_black_alpha(180),
                    })
                    .show(&mut quest_ui, |ui| {
                        ui.set_width(card_w - 32.0);
                        ui.set_max_height(card_h - 32.0);

                        // Header Bar
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🎮").size(18.0));
                            let (step_num, title) = match self.onboarding_quest_step {
                                OnboardingQuestStep::Welcome => ("1/6", "MISI 1: THE AWAKENING"),
                                OnboardingQuestStep::NavigateToStudio => ("2/6", "MISI 2: STUDIO GATEWAY"),
                                OnboardingQuestStep::UnderstandModes => ("3/6", "MISI 3: DUAL ENGINE PHILOSOPHY"),
                                OnboardingQuestStep::ConnectFirstCable => ("4/6", "MISI 4: FIRST PATCH CABLE"),
                                OnboardingQuestStep::TestAutoMute => ("5/6", "MISI 5: POP & CLICK SHIELD"),
                                OnboardingQuestStep::InsertTubeWarmer => ("6/6", "MISI 6: ANALOG COLORATION"),
                                OnboardingQuestStep::Completed => ("★", "TRAINING COMPLETE: STUDIO MASTER"),
                            };

                            ui.label(
                                egui::RichText::new(format!("LUNULA QUEST [{}] • {}", step_num, title))
                                    .strong()
                                    .size(12.0)
                                    .color(Color32::from_rgb(245, 158, 11)),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let skip_resp = crate::ui::theater::render_studio_button(
                                    ui,
                                    "Lewati Tutorial",
                                    crate::ui::theater::ButtonIcon::Close,
                                    crate::ui::theater::IconPosition::Right,
                                    true,
                                    Color32::from_rgb(22, 30, 46),
                                    Stroke::new(1.0, Color32::from_rgb(51, 65, 88)),
                                    Color32::from_rgb(148, 163, 184),
                                    6.0,
                                );
                                if skip_resp.clicked() {
                                    self.has_completed_onboarding = true;
                                    self.is_onboarding_quest_active = false;
                                    self.save_library_state();
                                }
                            });
                        });

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(match self.onboarding_quest_step {
                                OnboardingQuestStep::Welcome => "Selamat Datang di Studio Modular Lunula!",
                                OnboardingQuestStep::NavigateToStudio => "Masuki Ruang Kontrol Signal Chain",
                                OnboardingQuestStep::UnderstandModes => "Pahami Bit-Perfect vs DSP Active",
                                OnboardingQuestStep::ConnectFirstCable => "Sambungkan Jalur Kabel Pertamamu",
                                OnboardingQuestStep::TestAutoMute => "Uji Coba Proteksi Hening Auto-Mute",
                                OnboardingQuestStep::InsertTubeWarmer => "Suntikkan Kehangatan Tabung Triode",
                                OnboardingQuestStep::Completed => "Selamat! Kamu Resmi Menjadi Studio Master 🏆",
                            })
                            .strong()
                            .size(15.5)
                            .color(Color32::WHITE),
                        );

                        ui.add_space(8.0);

                        // Mission Goal Badge (Pill)
                        let goal_text = match self.onboarding_quest_step {
                            OnboardingQuestStep::Welcome => "🎯 TUGAS: Siapkan dirimu & klik 'Mulai Quest'",
                            OnboardingQuestStep::NavigateToStudio => "🎯 TUGAS: Klik tab 'Signal Chain Studio' di atas",
                            OnboardingQuestStep::UnderstandModes => "🎯 TUGAS: Klik tombol 'DSP Active' di bilah atas",
                            OnboardingQuestStep::ConnectFirstCable => "🎯 TUGAS: Tarik kabel dari Pin RAM Buffer ke Output DAC",
                            OnboardingQuestStep::TestAutoMute => "🎯 TUGAS: Klik pada garis kabel tadi untuk memutusnya",
                            OnboardingQuestStep::InsertTubeWarmer => "🎯 TUGAS: Klik '+ Tambah' pada Tube Warmer di sidebar kanan",
                            OnboardingQuestStep::Completed => "🎉 PENCAPAIAN: Seluruh Misi Pelatihan Berhasil Diselesaikan!",
                        };

                        egui::Frame::new()
                            .fill(Color32::from_rgb(30, 41, 59))
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(goal_text).strong().size(12.0).color(Color32::from_rgb(251, 191, 36)));
                            });

                        ui.add_space(8.0);

                        // Quest Detailed Narrative / Instructions
                        egui::ScrollArea::vertical().max_height(145.0).show(ui, |ui| {
                            match self.onboarding_quest_step {
                                OnboardingQuestStep::Welcome => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Lunula Player bukan pemutar musik biasa, melainkan stasiun audio modular dengan mesin komputasi 64-bit float dan arsitektur Bit-Perfect murni.\n\n\
                                            Di sini kamu tidak hanya mendengarkan musik, tapi juga meracik jalur sinyal analog, filter parametrik, dan proteksi akustik layaknya audio engineer di studio rekaman profesional."
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                }
                                OnboardingQuestStep::NavigateToStudio => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Langkah pertama adalah membuka ruang kerja utamamu: Signal Chain Studio.\n\n\
                                            Perhatikan tab di menu navigasi atas. Klik tab bertuliskan '⚡ Signal Chain Studio' untuk membuka kanvas kabel modular dan melihat aliran bitstream lagumu secara langsung."
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                }
                                OnboardingQuestStep::UnderstandModes => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Lunula Player memiliki DUA FILOSOFI AUDIO UTAMA (tombol OS Resample manual telah dihapus karena bekerja otomatis):\n\n\
                                            🟢 1. BIT-PERFECT DIRECT: Mengirim sampel lagu 1:1 ke DAC tanpa modifikasi 1 bit pun. Jika DAC tidak mendukung sample rate asli, sistem otomatis mengalihkan ke OS Resampling dengan aman.\n\n\
                                            🟣 2. DSP ACTIVE: Mengaktifkan seluruh modul filter, tabung 12AX7, dan equalizer studio dalam komputasi 64-bit float.\n\n\
                                            👉 Klik tombol [DSP Active] di kanan atas sekarang untuk menyalakan mesin efek!"
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                }
                                OnboardingQuestStep::ConnectFirstCable => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Di kanvas studio, audio mengalir dari kiri ke kanan melewati modul-modul virtual.\n\n\
                                            • Klik & tahan Pin Output (bulatan di sisi kanan kartu RAM Buffer).\n\
                                            • Tarik kabel kurva elastis ke Pin Input (bulatan di sisi kiri kartu Output DAC atau DSP Engine).\n\
                                            • Lepaskan klik saat garis tersambung hijau!"
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                }
                                OnboardingQuestStep::TestAutoMute => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Kabel audio yang putus mendadak pada software biasa sering memicu lonjakan sinyal DC atau letupan keras (pop/click) yang merusak speaker dan telinga.\n\n\
                                            👉 Sekarang, klik langsung pada garis kabel yang barusan kamu sambungkan untuk memutusnya!\n\n\
                                            Perhatikan: LunulaPlayer seketika membisukan sinyal (Auto-Mute) secara otomatis tanpa letupan berbahaya!"
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                }
                                OnboardingQuestStep::InsertTubeWarmer => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Sekarang saatnya mempercantik suara musikmu dengan kehangatan tabung analog vintage!\n\n\
                                            • Lihat palet filter di sidebar sebelah kanan.\n\
                                            • Temukan kartu '🔥 Tube Warmer' (Saturasi tabung 12AX7).\n\
                                            • Klik tombol '➕ Tambah' untuk memunculkannya ke kanvas!"
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                }
                                OnboardingQuestStep::Completed => {
                                    ui.label(
                                        egui::RichText::new(
                                            "Luar biasa! Kamu telah menguasai seluruh fondasi audio engineer Lunula:\n\n\
                                            ✓ Memahami alur sinyal Bit-Perfect 1:1 vs DSP Active 64-bit.\n\
                                            ✓ Menghubungkan dan memutus kabel patch virtual.\n\
                                            ✓ Memanfaatkan proteksi Auto-Mute anti letupan speaker.\n\
                                            ✓ Menambahkan modul efek saturasi analog ke kanvas.\n\n\
                                            Sekarang kamu bebas berkreasi meracik rantai audio impianmu!"
                                        )
                                        .size(12.0)
                                        .color(Color32::from_rgb(226, 232, 240)),
                                    );
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Action Buttons / Step Advancement
                        ui.horizontal(|ui| {
                            match self.onboarding_quest_step {
                                OnboardingQuestStep::Welcome => {
                                    let start_resp = crate::ui::theater::render_studio_button(
                                        ui,
                                        "Mulai Latihan Studio",
                                        crate::ui::theater::ButtonIcon::Play,
                                        crate::ui::theater::IconPosition::Right,
                                        true,
                                        Color32::from_rgb(20, 48, 40),
                                        Stroke::new(1.2, Color32::from_rgb(52, 211, 153)),
                                        Color32::from_rgb(220, 252, 231),
                                        6.0,
                                    );
                                    if start_resp.clicked() {
                                        self.current_tab = AppTab::SignalChain;
                                        self.onboarding_quest_step = OnboardingQuestStep::UnderstandModes;
                                    }
                                }
                                OnboardingQuestStep::NavigateToStudio => {
                                    let open_resp = crate::ui::theater::render_studio_button(
                                        ui,
                                        "Buka Studio Sekarang",
                                        crate::ui::theater::ButtonIcon::Play,
                                        crate::ui::theater::IconPosition::Right,
                                        true,
                                        Color32::from_rgb(45, 36, 20),
                                        Stroke::new(1.2, Color32::from_rgb(245, 158, 11)),
                                        Color32::from_rgb(254, 243, 199),
                                        6.0,
                                    );
                                    if open_resp.clicked() {
                                        self.current_tab = AppTab::SignalChain;
                                        self.onboarding_quest_step = OnboardingQuestStep::UnderstandModes;
                                    }
                                }
                                OnboardingQuestStep::UnderstandModes => {
                                    if ui.button(egui::RichText::new("⚡ Aktifkan Mode DSP Active").strong().size(12.0).color(Color32::from_rgb(147, 51, 234))).clicked() {
                                        self.set_chain_mode(ChainMode::DspActive);
                                        self.onboarding_quest_step = OnboardingQuestStep::ConnectFirstCable;
                                    }
                                    let next_resp = crate::ui::theater::render_studio_button(
                                        ui,
                                        "Lanjut",
                                        crate::ui::theater::ButtonIcon::Play,
                                        crate::ui::theater::IconPosition::Right,
                                        true,
                                        Color32::from_rgb(24, 32, 48),
                                        Stroke::new(1.0, Color32::from_rgb(51, 65, 88)),
                                        Color32::from_rgb(203, 213, 225),
                                        6.0,
                                    );
                                    if next_resp.clicked() {
                                        self.onboarding_quest_step = OnboardingQuestStep::ConnectFirstCable;
                                    }
                                }
                                OnboardingQuestStep::ConnectFirstCable => {
                                    if ui.button(egui::RichText::new("⚡ Hubungkan Otomatis").size(12.0).color(Color32::from_rgb(16, 185, 129))).clicked() {
                                        // Connect Buffer (2) to DAC (4)
                                        if !self.connections.iter().any(|c| c.from_node == 2 && c.to_node == 4) {
                                            self.connections.push(crate::models::CableConnection::new(2, 4));
                                            self.invalidate_chain_cache();
                                            self.sync_signal_chain_to_engine();
                                        }
                                        self.onboarding_quest_step = OnboardingQuestStep::TestAutoMute;
                                    }
                                    let next_resp = crate::ui::theater::render_studio_button(
                                        ui,
                                        "Lanjut",
                                        crate::ui::theater::ButtonIcon::Play,
                                        crate::ui::theater::IconPosition::Right,
                                        true,
                                        Color32::from_rgb(24, 32, 48),
                                        Stroke::new(1.0, Color32::from_rgb(51, 65, 88)),
                                        Color32::from_rgb(203, 213, 225),
                                        6.0,
                                    );
                                    if next_resp.clicked() {
                                        self.onboarding_quest_step = OnboardingQuestStep::TestAutoMute;
                                    }
                                }
                                OnboardingQuestStep::TestAutoMute => {
                                    if ui.button(egui::RichText::new("✂️ Putuskan Kabel Sekarang").size(12.0).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                        self.connections.retain(|c| !(c.from_node == 2 && c.to_node == 4));
                                        self.invalidate_chain_cache();
                                        self.sync_signal_chain_to_engine();
                                        self.onboarding_quest_step = OnboardingQuestStep::InsertTubeWarmer;
                                    }
                                    let next_resp = crate::ui::theater::render_studio_button(
                                        ui,
                                        "Lanjut",
                                        crate::ui::theater::ButtonIcon::Play,
                                        crate::ui::theater::IconPosition::Right,
                                        true,
                                        Color32::from_rgb(24, 32, 48),
                                        Stroke::new(1.0, Color32::from_rgb(51, 65, 88)),
                                        Color32::from_rgb(203, 213, 225),
                                        6.0,
                                    );
                                    if next_resp.clicked() {
                                        self.onboarding_quest_step = OnboardingQuestStep::InsertTubeWarmer;
                                    }
                                }
                                OnboardingQuestStep::InsertTubeWarmer => {
                                    if ui.button(egui::RichText::new("➕ Tambahkan Tube Warmer").strong().size(12.0).color(Color32::from_rgb(249, 115, 22))).clicked() {
                                        self.add_modular_filter(NodeType::TubeWarmer);
                                        self.onboarding_quest_step = OnboardingQuestStep::Completed;
                                    }
                                    let next_resp = crate::ui::theater::render_studio_button(
                                        ui,
                                        "Lanjut",
                                        crate::ui::theater::ButtonIcon::Play,
                                        crate::ui::theater::IconPosition::Right,
                                        true,
                                        Color32::from_rgb(24, 32, 48),
                                        Stroke::new(1.0, Color32::from_rgb(51, 65, 88)),
                                        Color32::from_rgb(203, 213, 225),
                                        6.0,
                                    );
                                    if next_resp.clicked() {
                                        self.onboarding_quest_step = OnboardingQuestStep::Completed;
                                    }
                                }
                                OnboardingQuestStep::Completed => {
                                    if ui.button(egui::RichText::new("Mulai Meracik Studio Penuh! 🎉").strong().size(13.0).color(Color32::from_rgb(16, 185, 129))).clicked() {
                                        self.has_completed_onboarding = true;
                                        self.is_onboarding_quest_active = false;
                                        self.save_library_state();
                                    }
                                }
                            }
                        });
                    });
            });
    }
}
