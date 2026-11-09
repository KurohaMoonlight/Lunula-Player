use eframe::egui;
use egui::{Color32, Pos2, Stroke};
use crate::models::{ChainMode, NodeType, SignalNodeStatus};
use crate::app::LunulaApp;

impl LunulaApp {
    /// Floating popup widget displaying all saved equalizers.
    /// Positioned intelligently relative to cursor / anchor edge so it never overlaps or gets clipped.
    pub fn render_saved_eq_floating_popup(&mut self, ui: &mut egui::Ui) {
        if !self.show_saved_eq_palette_popup {
            return;
        }

        let screen_rect = ui.ctx().input(|i| i.raw.screen_rect).unwrap_or_else(|| ui.clip_rect());
        let popup_w = 400.0;
        let popup_h = 440.0;

        // Position calculation:
        // Positioned cleanly to the LEFT of the sidebar palette with a 12px gap,
        // so it NEVER overlaps or covers the sidebar, buttons, or any filter card!
        let (sidebar_left, anchor_y) = if let Some(a) = self.saved_eq_popup_anchor {
            (a.x, a.y)
        } else {
            (screen_rect.max.x - 300.0, screen_rect.center().y)
        };

        let mut x = sidebar_left - popup_w - 12.0;
        if x < screen_rect.min.x + 12.0 {
            x = screen_rect.min.x + 12.0;
        }

        // Align vertically around anchor_y, clamped to stay safely inside screen bounds
        let min_y = screen_rect.min.y + 44.0;
        let max_y = (screen_rect.max.y - popup_h - 16.0).max(min_y);
        let ideal_y = anchor_y - popup_h * 0.5;
        let y = ideal_y.clamp(min_y, max_y);

        let popup_pos = Pos2::new(x, y);

        let mut close_popup = false;
        let mut open_preset: Option<String> = None;
        let mut add_preset_to_canvas: Option<String> = None;
        let mut delete_preset_name: Option<String> = None;

        let popup_id = ui.scope_id().with("saved_eq_floating_popup_area");
        let area_resp = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.5, Color32::from_rgb(147, 51, 234)))
                    .corner_radius(12.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width(popup_w - 28.0);
                        ui.set_max_height(popup_h - 28.0);

                        // Header Bar
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🎛️").size(15.0));
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("Koleksi Preset Equalizer")
                                        .strong()
                                        .size(13.5)
                                        .color(Color32::from_rgb(15, 23, 42))
                                );
                                ui.label(
                                    egui::RichText::new("Pilih preset untuk dipasang ke canvas patchbay atau diedit di EQ.")
                                        .size(10.5)
                                        .color(Color32::from_rgb(100, 116, 139))
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::new(
                                    egui::RichText::new("✖").strong().size(11.0).color(Color32::from_rgb(100, 116, 139))
                                )
                                .fill(Color32::from_rgb(241, 245, 249))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                .corner_radius(6.0);

                                if ui.add_sized([24.0, 24.0], close_btn).on_hover_text("Tutup jendela preset").clicked() {
                                    close_popup = true;
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        egui::ScrollArea::vertical()
                            .id_salt("saved_eq_popup_scroll")
                            .max_height(popup_h - 110.0)
                            .show(ui, |ui| {
                                for preset in &self.eq_presets {
                                    let is_on_canvas = self.nodes.iter().any(|n| n.node_type == NodeType::SavedEqPreset(preset.name.clone()));
                                    let chain_eval = self.evaluate_signal_chain();
                                    let is_flowing_in_chain = self.chain_mode == ChainMode::DspActive && is_on_canvas && self.nodes.iter().any(|n| {
                                        n.node_type == NodeType::SavedEqPreset(preset.name.clone()) && chain_eval.node_status(n.id) == SignalNodeStatus::Flowing
                                    });

                                    let card_bg = if is_flowing_in_chain {
                                        Color32::from_rgb(240, 253, 244)
                                    } else {
                                        Color32::from_rgb(255, 255, 255)
                                    };

                                    let border_stroke = if is_flowing_in_chain {
                                        Stroke::new(1.2, Color32::from_rgb(34, 197, 94))
                                    } else {
                                        Stroke::new(1.0, Color32::from_rgb(226, 232, 240))
                                    };

                                    egui::Frame::new()
                                        .fill(card_bg)
                                        .stroke(border_stroke)
                                        .corner_radius(10.0)
                                        .inner_margin(egui::Margin::symmetric(12, 10))
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                // ROW 1: Preset Name & Badges & Delete Button
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(&preset.name)
                                                            .strong()
                                                            .size(12.5)
                                                            .color(Color32::from_rgb(15, 23, 42))
                                                    );

                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        // Delete Button: only for custom user presets (factory presets are protected)
                                                        let is_factory = crate::models::ParametricEqPreset::default_presets().iter().any(|dp| dp.name == preset.name);
                                                        if !is_factory && self.eq_presets.len() > 1 {
                                                            let del_btn = egui::Button::new(
                                                                egui::RichText::new("🗑").size(10.5).color(Color32::from_rgb(239, 68, 68))
                                                            )
                                                            .fill(Color32::from_rgb(254, 242, 242))
                                                            .stroke(Stroke::new(0.8, Color32::from_rgb(254, 202, 202)))
                                                            .corner_radius(5.0);

                                                            if ui.add_sized([22.0, 20.0], del_btn).on_hover_text("Hapus preset kustom ini").clicked() {
                                                                delete_preset_name = Some(preset.name.clone());
                                                            }
                                                        }

                                                        // Badges: ONLY displayed if actually on canvas
                                                        if is_flowing_in_chain {
                                                            egui::Frame::new()
                                                                .fill(Color32::from_rgb(220, 252, 231))
                                                                .corner_radius(4.0)
                                                                .inner_margin(egui::Margin::symmetric(6, 2))
                                                                .show(ui, |ui| {
                                                                    ui.label(egui::RichText::new("⚡ Aktif di Chain").size(9.5).strong().color(Color32::from_rgb(22, 163, 74)));
                                                                });
                                                        } else if is_on_canvas {
                                                            egui::Frame::new()
                                                                .fill(Color32::from_rgb(243, 232, 255))
                                                                .corner_radius(4.0)
                                                                .inner_margin(egui::Margin::symmetric(6, 2))
                                                                .show(ui, |ui| {
                                                                    ui.label(egui::RichText::new("✓ Di Canvas").size(9.5).strong().color(Color32::from_rgb(147, 51, 234)));
                                                                });
                                                        }
                                                    });
                                                });

                                                ui.add_space(3.0);

                                                // ROW 2: Subtitle Info (Band Count, Preamp, Range)
                                                let preamp_str = if preset.preamp_mode == crate::models::PreampMode::Auto {
                                                    "Auto Preamp".to_string()
                                                } else {
                                                    format!("{:+.1} dB", preset.manual_preamp_db)
                                                };
                                                ui.label(
                                                    egui::RichText::new(format!("{} Band Parametrik  •  {}  •  20 Hz – 20 kHz", preset.bands.len(), preamp_str))
                                                        .size(10.5)
                                                        .color(Color32::from_rgb(100, 116, 139))
                                                );

                                                ui.add_space(8.0);

                                                // ROW 3: Dedicated Action Buttons (Full breathing room, never overlap!)
                                                ui.horizontal(|ui| {
                                                    // Button 1: Buka & Edit di Tab Equalizer
                                                    let btn_open = egui::Button::new(
                                                        egui::RichText::new("🎛️ Buka di EQ").size(11.0).strong().color(Color32::from_rgb(30, 41, 59))
                                                    )
                                                    .fill(Color32::from_rgb(241, 245, 249))
                                                    .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                                    .corner_radius(6.0);

                                                    if ui.add_sized([135.0, 24.0], btn_open)
                                                        .on_hover_text("Buka preset ini di Tab Equalizer untuk melihat kurva respon frekuensi dan mengedit band")
                                                        .clicked()
                                                    {
                                                        open_preset = Some(preset.name.clone());
                                                    }

                                                    ui.add_space(6.0);

                                                    // Button 2: Pasang ke Canvas / Sudah di Canvas
                                                    if !is_on_canvas {
                                                        let btn_canvas = egui::Button::new(
                                                            egui::RichText::new("➕ Pasang ke Canvas").size(11.0).strong().color(Color32::from_rgb(14, 165, 233))
                                                        )
                                                        .fill(Color32::from_rgb(240, 249, 255))
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(186, 230, 253)))
                                                        .corner_radius(6.0);

                                                        if ui.add_sized([145.0, 24.0], btn_canvas)
                                                            .on_hover_text("Pasang modul filter equalizer ini ke dalam diagram patchbay")
                                                            .clicked()
                                                        {
                                                            add_preset_to_canvas = Some(preset.name.clone());
                                                        }
                                                    } else {
                                                        let btn_canvas_on = egui::Button::new(
                                                            egui::RichText::new("✓ Terpasang di Canvas").size(11.0).color(Color32::from_rgb(147, 51, 234))
                                                        )
                                                        .fill(Color32::from_rgb(250, 245, 255))
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(233, 213, 255)))
                                                        .corner_radius(6.0);

                                                        ui.add_sized([145.0, 24.0], btn_canvas_on)
                                                            .on_hover_text("Modul preset equalizer ini sudah aktif di kanvas Signal Chain Studio");
                                                    }
                                                });
                                            });
                                        });

                                    ui.add_space(6.0);
                                }
                            });
                    });
            });

        // Close on clicking outside the popup frame
        if ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let rect = area_resp.response.rect;
                let is_inside_popup = rect.contains(pos);
                // Also ignore if clicking near the sidebar trigger button so toggle works smoothly
                let is_near_trigger = self.saved_eq_popup_anchor.map_or(false, |a| {
                    (pos.x >= a.x && pos.x <= a.x + 350.0) && (pos.y - a.y).abs() <= 24.0
                });
                if !is_inside_popup && !is_near_trigger {
                    close_popup = true;
                }
            }
        }

        if close_popup {
            self.show_saved_eq_palette_popup = false;
        }

        if let Some(name) = add_preset_to_canvas {
            self.add_modular_filter(NodeType::SavedEqPreset(name));
        }

        if let Some(name) = open_preset {
            if let Some(p) = self.eq_presets.iter().find(|p| p.name == name).cloned() {
                self.active_eq = p;
                self.eq_is_dirty = false;
                self.sync_parametric_to_legacy_eq();
                self.current_tab = crate::models::AppTab::Equalizer;
                self.show_saved_eq_palette_popup = false;
            }
        }

        if let Some(name) = delete_preset_name {
            self.eq_presets.retain(|p| p.name != name);
            let nt = NodeType::SavedEqPreset(name);
            let ids: Vec<usize> = self.nodes.iter().filter(|n| n.node_type == nt && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            self.save_library_state();
        }
    }

    /// Floating popup widget for Headroom Guard & Clipping Detection settings.
    /// Lets users toggle detection on/off, adjust the warning threshold slider (-3.0 to +6.0 dBFS),
    /// or choose 1-click audiophile presets (Strict -1.0 dBFS, Standard 0.0 dBFS, Relaxed +2.0 dBFS).
    pub fn render_clipping_settings_popup(&mut self, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
        if !self.show_clipping_settings_popup {
            return;
        }

        let popup_w = 390.0;

        // Position near top-right of canvas (with margin from top edge)
        let x = (canvas_rect.max.x - popup_w - 16.0).max(canvas_rect.min.x + 8.0);
        let y = canvas_rect.min.y + 16.0;
        let popup_pos = Pos2::new(x, y);

        let mut close_popup = false;
        let mut settings_changed = false;

        let popup_id = ui.scope_id().with("clipping_settings_floating_popup_area");
        let area_resp = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgb(255, 255, 255))
                    .stroke(Stroke::new(1.5, Color32::from_rgb(99, 102, 241)))
                    .corner_radius(12.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width(popup_w - 28.0);

                        // Header Bar
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("🛡️ HEADROOM & CLIPPING GUARD")
                                    .strong()
                                    .size(13.5)
                                    .color(Color32::from_rgb(30, 41, 59)),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("✕").size(14.0).strong().color(Color32::from_rgb(148, 163, 184))).clicked() {
                                    close_popup = true;
                                }
                            });
                        });

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // 1. Master Toggle On/Off Switch
                        ui.horizontal(|ui| {
                            let mut enabled = self.clipping_detection_enabled;
                            let checkbox_label = if enabled {
                                egui::RichText::new("Deteksi Headroom & Clipping: AKTIF").strong().size(12.5).color(Color32::from_rgb(16, 185, 129))
                            } else {
                                egui::RichText::new("Deteksi Headroom & Clipping: NONAKTIF").strong().size(12.5).color(Color32::from_rgb(239, 68, 68))
                            };
                            if ui.checkbox(&mut enabled, checkbox_label).changed() {
                                self.clipping_detection_enabled = enabled;
                                settings_changed = true;
                            }
                        });

                        if !self.clipping_detection_enabled {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("💡 Peringatan kabel merah putus-putus dan badge bahaya dinonaktifkan.\nAudio internal Lunula tetap diproses dalam float 64-bit headroom tak terbatas.")
                                    .size(11.0)
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                        } else {
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("Ambang Batas Peringatan (Margin Headroom):")
                                    .strong()
                                    .size(12.0)
                                    .color(Color32::from_rgb(51, 65, 85)),
                            );

                            // Slider: -3.0 dBFS to +6.0 dBFS
                            ui.horizontal(|ui| {
                                let mut threshold = self.clipping_threshold_db;
                                let slider = egui::Slider::new(&mut threshold, -3.0..=6.0)
                                    .step_by(0.1)
                                    .suffix(" dBFS")
                                    .text_color(Color32::from_rgb(30, 41, 59));
                                if ui.add(slider).changed() {
                                    self.clipping_threshold_db = threshold;
                                    settings_changed = true;
                                }
                            });

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Preset Sensitivitas Audiophile:").size(11.0).color(Color32::from_rgb(100, 116, 139)));

                            // 3 Quick Presets
                            ui.horizontal(|ui| {
                                let is_strict = (self.clipping_threshold_db - (-1.0)).abs() < 0.05;
                                let is_standard = self.clipping_threshold_db.abs() < 0.05;
                                let is_relaxed = (self.clipping_threshold_db - 2.0).abs() < 0.05;

                                let strict_btn = if is_strict {
                                    ui.button(egui::RichText::new("🎯 Strict (-1.0 dB)").strong().color(Color32::from_rgb(59, 130, 246)))
                                } else {
                                    ui.button("Strict (-1.0 dB)")
                                };
                                if strict_btn.on_hover_text("Aman dari Inter-Sample Peak (ISP) pada filter analog DAC. Rekomendasi EBU R128.").clicked() {
                                    self.clipping_threshold_db = -1.0;
                                    settings_changed = true;
                                }

                                let std_btn = if is_standard {
                                    ui.button(egui::RichText::new("⚖️ Standar (0.0 dB)").strong().color(Color32::from_rgb(16, 185, 129)))
                                } else {
                                    ui.button("Standar (0.0 dB)")
                                };
                                if std_btn.on_hover_text("Batas mutlak 0 dBFS digital PCM. Hanya menyala jika sampel digital menabrak full-scale.").clicked() {
                                    self.clipping_threshold_db = 0.0;
                                    settings_changed = true;
                                }

                                let rel_btn = if is_relaxed {
                                    ui.button(egui::RichText::new("📻 Relaxed (+2.0 dB)").strong().color(Color32::from_rgb(168, 85, 247)))
                                } else {
                                    ui.button("Relaxed (+2.0 dB)")
                                };
                                if rel_btn.on_hover_text("Toleran terhadap saturasi harmonik analog hangat (Tube Warmer / Preamp drive).").clicked() {
                                    self.clipping_threshold_db = 2.0;
                                    settings_changed = true;
                                }
                            });

                            ui.add_space(8.0);
                            let explanation = if self.clipping_threshold_db < -0.05 {
                                "ℹ️ Mode Strict: Memberi cadangan headroom ekstra untuk DAC agar terhindar dari distorsi analog inter-sample."
                            } else if self.clipping_threshold_db > 0.05 {
                                "ℹ️ Mode Relaxed: Memberi ruang bagi saturasi harmonik hangat tanpa memicu kabel merah di patchbay."
                            } else {
                                "ℹ️ Mode Standar: Referensi presisi 0 dBFS standar studio digital tanpa margin tambahan."
                            };
                            ui.label(egui::RichText::new(explanation).size(11.0).color(Color32::from_rgb(71, 85, 105)));

                            let static_rep = self.get_static_analysis();
                            if !static_rep.is_chain_safe && !static_rep.culprit_nodes.is_empty() {
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(4.0);
                                if ui.button(egui::RichText::new("🪄 Terapkan Auto-Fix untuk Rantai Sinyal Ini").strong().color(Color32::from_rgb(99, 102, 241)))
                                    .on_hover_text("Otomatis menyesuaikan gain modul-modul biang kerok agar berada di batas aman")
                                    .clicked()
                                {
                                    self.apply_all_headroom_auto_fixes();
                                }
                            }
                        }
                    });
            });

        // Close on clicking outside the popup frame
        if ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let rect = area_resp.response.rect;
                if !rect.contains(pos) {
                    close_popup = true;
                }
            }
        }

        if close_popup {
            self.show_clipping_settings_popup = false;
        }

        if settings_changed {
            self.invalidate_chain_cache();
            self.save_library_state();
        }
    }
}
