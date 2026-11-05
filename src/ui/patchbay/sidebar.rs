use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::NodeType;
use crate::app::LunulaApp;
use crate::ui::components::render_card_frame;
use crate::ui::patchbay::types::*;

impl LunulaApp {
    // Right Sidebar Filter & FX Toolbox Palette
    pub fn render_fx_sidebar_palette(&mut self, ui: &mut egui::Ui) {
        let mut delete_filter_type: Option<NodeType> = None;
        let mut delete_all_saved_eq_nodes = false;

        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("🎛️ FILTER & FX PALETTE").strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                ui.label(egui::RichText::new("Klik kiri untuk menambah • Klik kanan untuk opsi hapus").size(11.0).color(Color32::from_rgb(148, 163, 184)));

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Filter 0: Low-Pass Filter (Featured Test Filter)
                    let lpf_nodes: Vec<(usize, f32)> = self.nodes.iter()
                        .filter(|n| n.node_type == NodeType::LowPassFilter)
                        .map(|n| (n.id, n.param_val))
                        .collect();
                    let has_lpf = !lpf_nodes.is_empty();

                    let active_lpf_idx = lpf_nodes.iter()
                        .position(|(id, _)| Some(*id) == self.selected_node)
                        .unwrap_or(0);

                    let (target_id, current_lpf_cutoff) = if has_lpf {
                        let (id, cut) = lpf_nodes[active_lpf_idx];
                        (Some(id), cut)
                    } else {
                        (None, 800.0)
                    };

                    let cutoff_str = if current_lpf_cutoff >= 1000.0 {
                        format!("{:.1} kHz", current_lpf_cutoff / 1000.0)
                    } else {
                        format!("{:.0} Hz", current_lpf_cutoff)
                    };

                    let mut lpf_sidebar_drag_change: Option<f32> = None;
                    let mut switch_to_lpf_id: Option<usize> = None;

                    let g_lpf = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🌊 Low-Pass Filter").strong().size(12.5).color(Color32::from_rgb(14, 116, 144)));
                                ui.label(egui::RichText::new(&cutoff_str).strong().size(10.5).color(Color32::from_rgb(2, 132, 199)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Low-Pass Filter",
                                        NodeType::LowPassFilter,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });

                            // Opsi B: Tab Selector Pills if multiple LPF cards exist on canvas
                            if lpf_nodes.len() > 1 {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Card Aktif:").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    for (i, (id, cut)) in lpf_nodes.iter().enumerate() {
                                        let is_sel = i == active_lpf_idx;
                                        let lbl = if *cut >= 1000.0 {
                                            format!("#{}: {:.1}k", i + 1, cut / 1000.0)
                                        } else {
                                            format!("#{}: {:.0}Hz", i + 1, cut)
                                        };
                                        let btn = if is_sel {
                                            ui.add(egui::Button::new(egui::RichText::new(lbl).size(9.0).strong().color(Color32::WHITE)).fill(Color32::from_rgb(14, 165, 233)))
                                        } else {
                                            ui.button(egui::RichText::new(lbl).size(9.0).color(Color32::from_rgb(71, 85, 105)))
                                        };
                                        if btn.clicked() {
                                            switch_to_lpf_id = Some(*id);
                                        }
                                    }
                                });
                            }

                            ui.label(egui::RichText::new(format!(
                                "Memotong frekuensi treble di atas {}. Suara vokal dan instrumen tinggi teredam tebal (efek 'underwater / muffled') untuk uji coba signal chain.",
                                cutoff_str
                            )).size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            // Visual mini frequency response graph (synchronized with canvas slider & interactive)
                            let (graph_rect, graph_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), egui::Sense::click_and_drag());
                            let g_painter = ui.painter().with_clip_rect(graph_rect);
                            g_painter.rect_filled(graph_rect, 4.0, Color32::from_rgb(240, 249, 255));
                            g_painter.rect_stroke(graph_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)), egui::StrokeKind::Outside);

                            let x_start = graph_rect.min.x + 8.0;
                            let x_end = graph_rect.max.x - 8.0;
                            let usable_w = (x_end - x_start).max(10.0);

                            // Interactivity: Dragging / Clicking on mini sidebar graph also adjusts cutoff of active LPF
                            if (graph_resp.clicked() || graph_resp.dragged()) && has_lpf {
                                if let Some(pos) = graph_resp.interact_pointer_pos() {
                                    let norm = ((pos.x - x_start) / usable_w).clamp(0.0, 1.0);
                                    let new_cutoff = (50.0 * (400.0_f32).powf(norm)).clamp(50.0, 20000.0);
                                    lpf_sidebar_drag_change = Some(new_cutoff);
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    ui.ctx().request_repaint();
                                }
                            }

                            let y_top = graph_rect.min.y + 7.0;
                            let y_bottom = graph_rect.max.y - 7.0;

                            // Opsi A: Draw Secondary (Ghost) Curve(s) for other LPF cards in canvas
                            for (i, (_id, other_cut)) in lpf_nodes.iter().enumerate() {
                                if i != active_lpf_idx {
                                    let ghost_norm = ((other_cut / 100.0).log10() / 2.0).clamp(0.05, 0.95);
                                    let g_x_cut = x_start + usable_w * ghost_norm;

                                    let mut g_pts = Vec::new();
                                    g_pts.push(Pos2::new(x_start, y_top));
                                    if g_x_cut - 8.0 > x_start {
                                        g_pts.push(Pos2::new(g_x_cut - 8.0, y_top));
                                    }
                                    g_pts.push(Pos2::new(g_x_cut, y_top + 3.0));
                                    let drop_x = (g_x_cut + 14.0).min(x_end);
                                    g_pts.push(Pos2::new(drop_x, y_bottom));
                                    g_pts.push(Pos2::new(x_end, y_bottom));

                                    for seg in 0..g_pts.len() - 1 {
                                        g_painter.line_segment(
                                            [g_pts[seg], g_pts[seg + 1]],
                                            Stroke::new(1.3, Color32::from_rgba_premultiplied(148, 163, 184, 160)),
                                        );
                                    }

                                    let g_marker = Pos2::new(g_x_cut, y_top + 3.0);
                                    g_painter.circle_filled(g_marker, 2.5, Color32::from_rgb(148, 163, 184));
                                    let ghost_lbl = if *other_cut >= 1000.0 {
                                        format!("#{}: {:.1}k", i + 1, other_cut / 1000.0)
                                    } else {
                                        format!("#{}: {:.0}Hz", i + 1, other_cut)
                                    };
                                    g_painter.text(
                                        Pos2::new(g_x_cut, y_bottom - 2.0),
                                        egui::Align2::CENTER_BOTTOM,
                                        ghost_lbl,
                                        egui::FontId::proportional(7.5),
                                        Color32::from_rgb(148, 163, 184),
                                    );
                                }
                            }

                            // Dynamic cutoff X for Primary Active Curve
                            let norm_cut = ((current_lpf_cutoff / 50.0).log10() / (400.0_f32).log10()).clamp(0.05, 0.95);
                            let x_cut = x_start + usable_w * norm_cut;

                            let mut curve_pts = Vec::new();
                            curve_pts.push(Pos2::new(x_start, y_top));
                            if x_cut - 8.0 > x_start {
                                curve_pts.push(Pos2::new(x_cut - 8.0, y_top));
                            }
                            curve_pts.push(Pos2::new(x_cut, y_top + 3.0));
                            let drop_x = (x_cut + 14.0).min(x_end);
                            curve_pts.push(Pos2::new(drop_x, y_bottom));
                            curve_pts.push(Pos2::new(x_end, y_bottom));

                            for i in 0..curve_pts.len() - 1 {
                                g_painter.line_segment([curve_pts[i], curve_pts[i + 1]], Stroke::new(2.2, Color32::from_rgb(2, 132, 199)));
                            }

                            // Animated pulsing cutoff marker dot when audio is playing
                            let time = ui.input(|i| i.time);
                            let pulse = if self.is_playing {
                                ui.ctx().request_repaint();
                                (time * 5.0).sin().abs() as f32
                            } else {
                                0.0
                            };

                            let marker_pos = Pos2::new(x_cut, y_top + 3.0);
                            let marker_r = 3.5 + pulse * 1.5;
                            if self.is_playing {
                                g_painter.circle_filled(marker_pos, marker_r + 2.0, Color32::from_rgba_premultiplied(14, 165, 233, (100.0 + pulse * 90.0) as u8));
                            }
                            g_painter.circle_filled(marker_pos, marker_r, Color32::from_rgb(2, 132, 199));
                            g_painter.circle_stroke(marker_pos, marker_r, Stroke::new(1.0, Color32::WHITE));

                            // Axis edge labels
                            g_painter.text(Pos2::new(x_start + 2.0, y_bottom - 1.0), egui::Align2::LEFT_BOTTOM, "20 Hz", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                            g_painter.text(Pos2::new(x_end - 2.0, y_bottom - 1.0), egui::Align2::RIGHT_BOTTOM, "20 kHz", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));

                            // Knee label: dynamic placement to avoid collision with edge text
                            let label_x = x_cut.clamp(x_start + 30.0, x_end - 34.0);
                            let (label_y, align) = if norm_cut > 0.65 {
                                (y_top + 1.0, egui::Align2::RIGHT_TOP)
                            } else {
                                (y_top - 1.0, egui::Align2::CENTER_BOTTOM)
                            };
                            let active_prefix = if lpf_nodes.len() > 1 { format!("#{} ", active_lpf_idx + 1) } else { String::new() };
                            g_painter.text(
                                Pos2::new(label_x, label_y),
                                align,
                                format!("{}Cutoff {}", active_prefix, cutoff_str),
                                egui::FontId::proportional(8.5),
                                Color32::from_rgb(2, 132, 199),
                            );

                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::LowPassFilter);
                                }
                                if has_lpf {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::LowPassFilter);
                                    }
                                }
                            });
                        });
                    });
                    g_lpf.response.context_menu(|ui| {
                        ui.label(egui::RichText::new(format!("🌊 Low-Pass Filter ({})", cutoff_str)).strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::LowPassFilter);
                            ui.close();
                        }
                        if has_lpf {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::LowPassFilter);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_sel) = switch_to_lpf_id {
                        self.selected_node = Some(new_sel);
                    }

                    if let Some(new_val) = lpf_sidebar_drag_change {
                        if let Some(tid) = target_id {
                            if let Some(n) = self.nodes.iter_mut().find(|n| n.id == tid) {
                                n.param_val = new_val;
                                n.sub1 = if new_val >= 1000.0 {
                                    format!("Cutoff {:.1} kHz", new_val / 1000.0)
                                } else {
                                    format!("Cutoff {:.0} Hz", new_val)
                                };
                            }
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(tid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter: Mixer / Summing Bus (Gerbang Paralel)
                    let mixer_node = self.nodes.iter().find(|n| n.node_type == NodeType::Mixer);
                    let has_mixer = mixer_node.is_some();
                    let current_mix = mixer_node.map(|n| n.param_val).unwrap_or(0.5);
                    let mut mixer_sidebar_drag_change: Option<f32> = None;

                    let g_mix = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎚️ Mixer / Summing Bus").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                let mix_txt = format!("{:.0}% / {:.0}%", (1.0 - current_mix) * 100.0, current_mix * 100.0);
                                ui.label(egui::RichText::new(mix_txt).strong().size(10.5).color(Color32::from_rgb(147, 51, 234)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Mixer / Summing Bus",
                                        NodeType::Mixer,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Gerbang penggabung dua jalur sinyal paralel (Dry & Wet) dengan slider crossfader. Mengamankan headroom sinyal dari clipping.").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            // Mini crossfader visualization
                            let (m_rect, m_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), egui::Sense::click_and_drag());
                            let m_painter = ui.painter().with_clip_rect(m_rect);
                            m_painter.rect_filled(m_rect, 4.0, Color32::from_rgb(250, 245, 255));
                            m_painter.rect_stroke(m_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(233, 213, 255)), egui::StrokeKind::Outside);

                            let mx_start = m_rect.min.x + 20.0;
                            let mx_end = m_rect.max.x - 20.0;
                            let mx_w = (mx_end - mx_start).max(10.0);
                            let m_cy = m_rect.center().y;

                            if (m_resp.clicked() || m_resp.dragged()) && has_mixer {
                                if let Some(pos) = m_resp.interact_pointer_pos() {
                                    let norm = ((pos.x - mx_start) / mx_w).clamp(0.0, 1.0);
                                    mixer_sidebar_drag_change = Some(norm);
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    ui.ctx().request_repaint();
                                }
                            }

                            // Labels A & B
                            m_painter.text(Pos2::new(mx_start - 4.0, m_cy), egui::Align2::RIGHT_CENTER, "A", egui::FontId::proportional(9.0), Color32::from_rgb(126, 34, 206));
                            m_painter.text(Pos2::new(mx_end + 4.0, m_cy), egui::Align2::LEFT_CENTER, "B", egui::FontId::proportional(9.0), Color32::from_rgb(126, 34, 206));

                            // Slider track with center notch
                            m_painter.rect_filled(Rect::from_min_size(Pos2::new(mx_start, m_cy - 2.0), Vec2::new(mx_w, 4.0)), 2.0, Color32::from_rgb(243, 232, 255));
                            m_painter.line_segment([Pos2::new(mx_start + mx_w * 0.5, m_cy - 4.0), Pos2::new(mx_start + mx_w * 0.5, m_cy + 4.0)], Stroke::new(1.2, Color32::from_rgb(192, 132, 252)));

                            let thumb_x = mx_start + mx_w * current_mix;
                            m_painter.circle_filled(Pos2::new(thumb_x, m_cy), 4.5, Color32::WHITE);
                            m_painter.circle_stroke(Pos2::new(thumb_x, m_cy), 4.5, Stroke::new(1.6, Color32::from_rgb(147, 51, 234)));

                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::Mixer);
                                }
                                if has_mixer {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::Mixer);
                                    }
                                }
                            });
                        });
                    });
                    g_mix.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🎚️ Mixer / Summing Bus").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::Mixer);
                            ui.close();
                        }
                        if has_mixer {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::Mixer);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_mix) = mixer_sidebar_drag_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::Mixer) {
                            n.param_val = new_mix;
                            let a_pct = (1.0 - new_mix) * 100.0;
                            let b_pct = new_mix * 100.0;
                            n.sub1 = format!("Mix: {:.0}% A • {:.0}% B", a_pct, b_pct);
                            let mid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(mid as u32, 0, new_mix);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Module: Splitter / Y-Branch Bus
                    let has_splitter = self.nodes.iter().any(|n| n.node_type == NodeType::Splitter);
                    let g_split = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔀 Splitter / Y-Branch").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Splitter / Y-Branch",
                                        NodeType::Splitter,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Gerbang pembagi 1 input ke 2 output (Pin A & Pin B) tanpa kehilangan kualitas sinyal (lossless duplication).").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            // Mini Y-branch visualization
                            let (s_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), egui::Sense::hover());
                            let s_painter = ui.painter().with_clip_rect(s_rect);
                            s_painter.rect_filled(s_rect, 4.0, Color32::from_rgb(255, 251, 235));
                            s_painter.rect_stroke(s_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(253, 230, 138)), egui::StrokeKind::Outside);

                            let sx_in = Pos2::new(s_rect.min.x + 35.0, s_rect.center().y);
                            let sx_mid = Pos2::new(s_rect.center().x, s_rect.center().y);
                            let sx_out_a = Pos2::new(s_rect.max.x - 35.0, s_rect.center().y - 6.0);
                            let sx_out_b = Pos2::new(s_rect.max.x - 35.0, s_rect.center().y + 6.0);

                            s_painter.line_segment([sx_in, sx_mid], Stroke::new(1.8, Color32::from_rgb(245, 158, 11)));
                            s_painter.line_segment([sx_mid, sx_out_a], Stroke::new(1.8, Color32::from_rgb(245, 158, 11)));
                            s_painter.line_segment([sx_mid, sx_out_b], Stroke::new(1.8, Color32::from_rgb(245, 158, 11)));
                            s_painter.circle_filled(sx_mid, 2.5, Color32::from_rgb(217, 119, 6));

                            s_painter.text(Pos2::new(sx_in.x - 4.0, sx_in.y), egui::Align2::RIGHT_CENTER, "In", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));
                            s_painter.text(Pos2::new(sx_out_a.x + 4.0, sx_out_a.y), egui::Align2::LEFT_CENTER, "A", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));
                            s_painter.text(Pos2::new(sx_out_b.x + 4.0, sx_out_b.y), egui::Align2::LEFT_CENTER, "B", egui::FontId::proportional(8.5), Color32::from_rgb(180, 83, 9));

                            ui.add_space(3.0);
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::Splitter);
                                }
                                if has_splitter {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::Splitter);
                                    }
                                }
                            });
                        });
                    });
                    g_split.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔀 Splitter / Y-Branch Bus").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::Splitter);
                            ui.close();
                        }
                        if has_splitter {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::Splitter);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Filter 1: Tube Warmer (Analog Saturation)
                    let tube_node = self.nodes.iter().find(|n| n.node_type == NodeType::TubeWarmer);
                    let has_tube = tube_node.is_some();
                    let current_tube_warmth = tube_node.map(|n| n.param_val).unwrap_or(35.0);
                    let mut tube_change: Option<f32> = None;

                    let g1 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔥 Tube Warmer (Analog)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("Warmth {:.0}%", current_tube_warmth)).strong().size(10.5).color(Color32::from_rgb(249, 115, 22)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Tube Warmer (Analog)",
                                        NodeType::TubeWarmer,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Harmonik triode 12AX7 vintage untuk nada hangat, vokal tebal, dan saturasi analog musikal.").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            if has_tube {
                                let mut val = current_tube_warmth;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("%").show_value(false)).changed() {
                                    tube_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("20% Mild", 20.0), ("35% Triode", 35.0), ("50% Rich", 50.0), ("75% Hot", 75.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            tube_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::TubeWarmer);
                                }
                                if has_tube {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::TubeWarmer);
                                    }
                                }
                            });
                        });
                    });
                    g1.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔥 Tube Warmer (Analog)").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::TubeWarmer);
                            ui.close();
                        }
                        if has_tube {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::TubeWarmer);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = tube_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::TubeWarmer) {
                            n.param_val = new_val;
                            n.sub1 = format!("Warmth {:.0}%", new_val);
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter 2: Spatial Stereo Widener (Active Haas 3D)
                    let widener_node = self.nodes.iter().find(|n| n.node_type == NodeType::StereoWidener);
                    let has_widener = widener_node.is_some();
                    let current_widener_val = widener_node.map(|n| n.param_val).unwrap_or(35.0);
                    let mut widener_change: Option<f32> = None;

                    let g2 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🌐 Stereo 3D Widener").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("+{:.0}% Width", current_widener_val)).strong().size(10.5).color(Color32::from_rgb(99, 102, 241)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Stereo 3D Widener",
                                        NodeType::StereoWidener,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Memperluas dimensi stereo & soundstage 3D spasial secara alami menggunakan pemisahan Mid/Side Haas.").size(10.5).color(Color32::from_rgb(100, 116, 139)));

                            if has_widener {
                                let mut val = current_widener_val;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("%").show_value(false)).changed() {
                                    widener_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("+15% Natural", 15.0), ("+35% Haas", 35.0), ("+60% Wide", 60.0), ("+100% 3D", 100.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            widener_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::StereoWidener);
                                }
                                if has_widener {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::StereoWidener);
                                    }
                                }
                            });
                        });
                    });
                    g2.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🌐 Stereo 3D Widener").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::StereoWidener);
                            ui.close();
                        }
                        if has_widener {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::StereoWidener);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = widener_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::StereoWidener) {
                            n.param_val = new_val;
                            n.sub1 = format!("+{:.0}% Soundstage", new_val);
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter 3: Bass Guard Limiter (Coming Soon)
                    let has_bass = self.nodes.iter().any(|n| n.node_type == NodeType::BassGuard);
                    let g3 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🛡️ Bass Guard & Limiter").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("🚧 Coming Soon").size(10.0).strong().color(Color32::from_rgb(217, 119, 6)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Bass Guard & Limiter",
                                        NodeType::BassGuard,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Mencegah distorsi sub-bass berlebih dan melindungi driver headphone (Mesin DSP sedang dalam pengembangan).").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("🚧 Coming Soon").strong().color(Color32::from_rgb(180, 83, 9))).on_hover_text("Mesin DSP filter ini sedang dalam antrean pengembangan").clicked() {
                                    self.add_modular_filter(NodeType::BassGuard);
                                }
                                if has_bass {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::BassGuard);
                                    }
                                }
                            });
                        });
                    });
                    g3.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🛡️ Bass Guard & Limiter 🚧 Coming Soon").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::BassGuard);
                            ui.close();
                        }
                        if has_bass {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::BassGuard);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Filter 4: Volume Attenuator (Low Volume Pad) (Coming Soon)
                    let has_vol = self.nodes.iter().any(|n| n.node_type == NodeType::VolumeAttenuator);
                    let g4 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔉 Volume Attenuator (-6dB)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("🚧 Coming Soon").size(10.0).strong().color(Color32::from_rgb(217, 119, 6)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Volume Attenuator (-6dB)",
                                        NodeType::VolumeAttenuator,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Peredam gain pasif untuk malam hari (Gunakan Stepped Attenuator di suite bawah untuk kontrol aktif).").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("🚧 Coming Soon").strong().color(Color32::from_rgb(180, 83, 9))).on_hover_text("Mesin DSP filter ini sedang dalam antrean pengembangan").clicked() {
                                    self.add_modular_filter(NodeType::VolumeAttenuator);
                                }
                                if has_vol {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::VolumeAttenuator);
                                    }
                                }
                            });
                        });
                    });
                    g4.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔉 Volume Attenuator (-6dB) 🚧 Coming Soon").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::VolumeAttenuator);
                            ui.close();
                        }
                        if has_vol {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::VolumeAttenuator);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(8.0);

                    // Volume Suite Section Header
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔊 MANIPULASI VOLUME & GAIN (6 FILTER)").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                    });
                    ui.label(egui::RichText::new("Gain staging presisi, saturasi Class-A, kontur ISO 226, attenuator pasif, opto leveling, dan stereo balance.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                    ui.add_space(6.0);

                    // 1. Pure Volume Booster
                    let vb_node = self.nodes.iter().find(|n| n.node_type == NodeType::VolumeBooster);
                    let has_vb = vb_node.is_some();
                    let current_vb_gain = vb_node.map(|n| n.param_val).unwrap_or(6.0);
                    let mut vb_change: Option<f32> = None;

                    let g_vb = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚡ Pure Volume Booster").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("+{:.1} dB", current_vb_gain)).strong().size(10.5).color(Color32::from_rgb(139, 92, 246)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Volume Booster",
                                        NodeType::VolumeBooster,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Penguat gain bersih 0 s/d +24dB berpresisi f64 tanpa saturasi pewarnaan nada.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_vb {
                                let mut val = current_vb_gain;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=24.0).suffix(" dB").show_value(false)).changed() {
                                    vb_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("+3dB", 3.0), ("+6dB", 6.0), ("+12dB", 12.0), ("+18dB", 18.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            vb_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::VolumeBooster);
                                }
                                if has_vb {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::VolumeBooster);
                                    }
                                }
                            });
                        });
                    });
                    g_vb.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("⚡ Pure Volume Booster").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::VolumeBooster);
                            ui.close();
                        }
                        if has_vb {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::VolumeBooster);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = vb_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::VolumeBooster) {
                            n.param_val = new_val;
                            n.sub1 = format!("+{:.1} dB Gain", new_val);
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 2. Analog Preamp & Trim
                    let preamp_node = self.nodes.iter().find(|n| n.node_type == NodeType::AnalogPreamp);
                    let has_preamp = preamp_node.is_some();
                    let current_preamp_gain = preamp_node.map(|n| n.param_val).unwrap_or(0.0);
                    let mut preamp_change: Option<f32> = None;

                    let g_preamp = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎛️ Analog Preamp & Trim").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:+0.1} dB", current_preamp_gain)).strong().size(10.5).color(Color32::from_rgb(234, 88, 12)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Analog Preamp",
                                        NodeType::AnalogPreamp,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Gain staging studio -24 s/d +12dB dengan saturasi trafo analog Class-A halus.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_preamp {
                                let mut val = current_preamp_gain;
                                if ui.add(egui::Slider::new(&mut val, -24.0..=12.0).suffix(" dB").show_value(false)).changed() {
                                    preamp_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("-6dB Trim", -6.0), ("0dB Unity", 0.0), ("+3dB Warm", 3.0), ("+6dB Sat", 6.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            preamp_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::AnalogPreamp);
                                }
                                if has_preamp {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::AnalogPreamp);
                                    }
                                }
                            });
                        });
                    });
                    g_preamp.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🎛️ Analog Preamp & Trim").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::AnalogPreamp);
                            ui.close();
                        }
                        if has_preamp {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::AnalogPreamp);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = preamp_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::AnalogPreamp) {
                            n.param_val = new_val;
                            n.sub1 = format!("{:+0.1} dB Drive", new_val);
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 3. Dynamic Loudness
                    let loud_node = self.nodes.iter().find(|n| n.node_type == NodeType::DynamicLoudness);
                    let has_loudness = loud_node.is_some();
                    let current_loud_depth = loud_node.map(|n| n.param_val).unwrap_or(50.0);
                    let mut loud_change: Option<f32> = None;

                    let g_loud = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🌙 Dynamic Loudness").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:.0}%", current_loud_depth)).strong().size(10.5).color(Color32::from_rgb(245, 158, 11)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Dynamic Loudness",
                                        NodeType::DynamicLoudness,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Kompensasi kontur equal-loudness ISO 226 untuk bass dan treble penuh di volume rendah.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_loudness {
                                let mut val = current_loud_depth;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("%").show_value(false)).changed() {
                                    loud_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("25% Mild", 25.0), ("50% Night", 50.0), ("75% Deep", 75.0), ("100% Full", 100.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            loud_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::DynamicLoudness);
                                }
                                if has_loudness {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::DynamicLoudness);
                                    }
                                }
                            });
                        });
                    });
                    g_loud.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🌙 Dynamic Loudness (ISO 226)").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::DynamicLoudness);
                            ui.close();
                        }
                        if has_loudness {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::DynamicLoudness);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = loud_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::DynamicLoudness) {
                            n.param_val = new_val;
                            n.sub1 = format!("Loudness {:.0}%", new_val);
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 4. Audiophile Stepped Attenuator
                    let att_node = self.nodes.iter().find(|n| n.node_type == NodeType::SteppedAttenuator);
                    let has_att = att_node.is_some();
                    let current_att_db = att_node.map(|n| n.param_val).unwrap_or(-12.0);
                    let mut att_change: Option<f32> = None;

                    let g_att = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔘 Stepped Attenuator").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:.0} dB", current_att_db)).strong().size(10.5).color(Color32::from_rgb(6, 182, 212)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Stepped Attenuator",
                                        NodeType::SteppedAttenuator,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Resistor ladder 64-step passive attenuator (-60 s/d 0dB) dengan tombol instan DIM -20dB.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_att {
                                let mut val = current_att_db;
                                if ui.add(egui::Slider::new(&mut val, -60.0..=0.0).suffix(" dB").step_by(1.0).show_value(false)).changed() {
                                    att_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("0dB Max", 0.0), ("-12dB", -12.0), ("DIM -20dB", -20.0), ("-36dB", -36.0)] {
                                        let btn = if (current_att_db - p_val).abs() < 0.5 {
                                            ui.add(egui::Button::new(egui::RichText::new(lbl).size(9.0).color(Color32::WHITE)).fill(Color32::from_rgb(6, 182, 212)))
                                        } else {
                                            ui.button(egui::RichText::new(lbl).size(9.0))
                                        };
                                        if btn.clicked() {
                                            att_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::SteppedAttenuator);
                                }
                                if has_att {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::SteppedAttenuator);
                                    }
                                }
                            });
                        });
                    });
                    g_att.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🔘 Stepped Attenuator").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::SteppedAttenuator);
                            ui.close();
                        }
                        if has_att {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::SteppedAttenuator);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = att_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::SteppedAttenuator) {
                            n.param_val = new_val;
                            n.sub1 = format!("{:.0} dB", new_val);
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 5. Studio Optical Leveler (LA-2A Opto)
                    let opt_node = self.nodes.iter().find(|n| n.node_type == NodeType::OpticalLeveler);
                    let has_opt = opt_node.is_some();
                    let current_opt_red = opt_node.map(|n| n.param_val).unwrap_or(30.0);
                    let mut opt_change: Option<f32> = None;

                    let g_opt = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🛡️ Studio Optical Leveler").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new(format!("{:.0}%", current_opt_red)).strong().size(10.5).color(Color32::from_rgb(180, 83, 9)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Optical Leveler",
                                        NodeType::OpticalLeveler,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Leveler optocoupler bergaya LA-2A vintage dengan pelepasan 2-tahap halus tanpa pumping.").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_opt {
                                let mut val = current_opt_red;
                                if ui.add(egui::Slider::new(&mut val, 0.0..=100.0).suffix("% Red").show_value(false)).changed() {
                                    opt_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("20% Light", 20.0), ("40% Classic", 40.0), ("65% Heavy", 65.0), ("85% Max", 85.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            opt_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::OpticalLeveler);
                                }
                                if has_opt {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::OpticalLeveler);
                                    }
                                }
                            });
                        });
                    });
                    g_opt.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🛡️ Studio Optical Leveler").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::OpticalLeveler);
                            ui.close();
                        }
                        if has_opt {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::OpticalLeveler);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = opt_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::OpticalLeveler) {
                            n.param_val = new_val;
                            n.sub1 = format!("Peak Red {:.0}%", new_val);
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(6.0);

                    // 6. Dual-Channel Stereo Balance & Trim
                    let bal_node = self.nodes.iter().find(|n| n.node_type == NodeType::StereoBalance);
                    let has_bal = bal_node.is_some();
                    let current_bal = bal_node.map(|n| n.param_val).unwrap_or(0.0);
                    let mut bal_change: Option<f32> = None;

                    let g_bal = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚖️ Stereo Balance & Trim").strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                let bal_lbl = if current_bal.abs() < 1.0 {
                                    "Center".to_string()
                                } else if current_bal < 0.0 {
                                    format!("L {:.0}%", current_bal.abs())
                                } else {
                                    format!("R {:.0}%", current_bal)
                                };
                                ui.label(egui::RichText::new(bal_lbl).strong().size(10.5).color(Color32::from_rgb(79, 70, 229)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Stereo Balance",
                                        NodeType::StereoBalance,
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Panning kurva daya konstan sinus/kosinus dari -100% (Kiri) ke +100% (Kanan).").size(10.0).color(Color32::from_rgb(100, 116, 139)));

                            if has_bal {
                                let mut val = current_bal;
                                if ui.add(egui::Slider::new(&mut val, -100.0..=100.0).suffix("%").show_value(false)).changed() {
                                    bal_change = Some(val);
                                }
                                ui.horizontal(|ui| {
                                    for (lbl, p_val) in [("◀ L 50%", -50.0), ("● Center", 0.0), ("R 50% ▶", 50.0)] {
                                        if ui.button(egui::RichText::new(lbl).size(9.0)).clicked() {
                                            bal_change = Some(p_val);
                                        }
                                    }
                                });
                            }

                            ui.horizontal(|ui| {
                                if ui.button(egui::RichText::new("➕ Tambah ke Chain").strong()).clicked() {
                                    self.add_modular_filter(NodeType::StereoBalance);
                                }
                                if has_bal {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus modul ini dari canvas").clicked() {
                                        delete_filter_type = Some(NodeType::StereoBalance);
                                    }
                                }
                            });
                        });
                    });
                    g_bal.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("⚖️ Stereo Balance & Trim").strong());
                        ui.separator();
                        if ui.button("➕ Tambah ke Signal Chain").clicked() {
                            self.add_modular_filter(NodeType::StereoBalance);
                            ui.close();
                        }
                        if has_bal {
                            if ui.button("🗑️ Hapus Semua dari Canvas").clicked() {
                                delete_filter_type = Some(NodeType::StereoBalance);
                                ui.close();
                            }
                        }
                    });

                    if let Some(new_val) = bal_change {
                        if let Some(n) = self.nodes.iter_mut().find(|n| n.node_type == NodeType::StereoBalance) {
                            n.param_val = new_val;
                            if new_val.abs() < 1.0 {
                                n.sub1 = "Center 0.0 dB".to_string();
                            } else if new_val < 0.0 {
                                n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 + new_val.abs() * 0.5, 50.0 - new_val.abs() * 0.5);
                            } else {
                                n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 - new_val * 0.5, 50.0 + new_val * 0.5);
                            }
                            let nid = n.id;
                            if let Some(ref engine) = self.audio_engine {
                                engine.set_node_param(nid as u32, 0, new_val);
                            }
                        }
                    }
                    ui.add_space(8.0);

                    // Filter 5: Equalizer Presets Card (Matches Filter 1-4 anatomy perfectly)
                    let has_any_eq_node = self.nodes.iter().any(|n| matches!(n.node_type, NodeType::SavedEqPreset(_)));
                    let is_open = self.show_saved_eq_palette_popup;
                    let sidebar_min_x = ui.max_rect().min.x;

                    let mut btn_clicked = false;
                    let g5 = ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🎛️ Equalizer Presets").strong().size(12.5).color(Color32::from_rgb(15, 23, 42)));
                                ui.label(egui::RichText::new("🚧 Coming Soon").size(10.0).strong().color(Color32::from_rgb(217, 119, 6)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    render_routing_help_button(
                                        ui,
                                        "Equalizer Presets",
                                        NodeType::SavedEqPreset("Default".to_string()),
                                        &mut self.active_filter_tutorial,
                                        &mut self.filter_tutorial_step,
                                    );
                                });
                            });
                            ui.label(egui::RichText::new("Koleksi kurva frekuensi siap pakai untuk vokal, bass, dan akustik (Coming Soon).").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.horizontal(|ui| {
                                let btn_label = if is_open { "🎛️ Tutup Preset" } else { "➕ Tambah ke Chain" };
                                let btn_resp = ui.button(egui::RichText::new(btn_label).strong());
                                if btn_resp.clicked() {
                                    btn_clicked = true;
                                    self.show_saved_eq_palette_popup = !self.show_saved_eq_palette_popup;
                                    let click_y = ui.input(|i| i.pointer.interact_pos()).map(|p| p.y).unwrap_or(btn_resp.rect.center().y);
                                    self.saved_eq_popup_anchor = Some(Pos2::new(sidebar_min_x, click_y));
                                }
                                if has_any_eq_node {
                                    if ui.button(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68))).on_hover_text("Hapus semua modul EQ Preset dari canvas").clicked() {
                                        delete_all_saved_eq_nodes = true;
                                    }
                                }
                            });
                        });
                    });

                    let card_resp = ui.interact(g5.response.rect, ui.scope_id().with("card_eq_presets_click"), egui::Sense::click());
                    if card_resp.clicked() && !btn_clicked {
                        self.show_saved_eq_palette_popup = !self.show_saved_eq_palette_popup;
                        let click_y = ui.input(|i| i.pointer.interact_pos()).map(|p| p.y).unwrap_or(g5.response.rect.center().y);
                        self.saved_eq_popup_anchor = Some(Pos2::new(sidebar_min_x, click_y));
                    }

                    g5.response.context_menu(|ui| {
                        ui.label(egui::RichText::new("🎛️ Equalizer Presets").strong());
                        ui.separator();
                        if ui.button("🎛️ Buka / Tutup Koleksi Preset").clicked() {
                            self.show_saved_eq_palette_popup = !self.show_saved_eq_palette_popup;
                            let click_y = ui.input(|i| i.pointer.interact_pos()).map(|p| p.y).unwrap_or(g5.response.rect.center().y);
                            self.saved_eq_popup_anchor = Some(Pos2::new(sidebar_min_x, click_y));
                            ui.close();
                        }
                        if has_any_eq_node {
                            if ui.button("🗑️ Hapus Semua EQ Preset dari Canvas").clicked() {
                                delete_all_saved_eq_nodes = true;
                                ui.close();
                            }
                        }
                    });
                });
            });
        });

        // Execute batch filter deletion
        if let Some(ref nt) = delete_filter_type {
            let ids: Vec<usize> = self.nodes.iter().filter(|n| &n.node_type == nt && n.is_removable).map(|n| n.id).collect();
            self.nodes.retain(|n| !ids.contains(&n.id));
            self.connections.retain(|c| !ids.contains(&c.from_node) && !ids.contains(&c.to_node));
            if let Some(sel) = self.selected_node {
                if ids.contains(&sel) {
                    self.selected_node = None;
                }
            }
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
        }

        if delete_filter_type.is_some() || delete_all_saved_eq_nodes {
            self.invalidate_chain_cache();
            self.sync_signal_chain_to_engine();
        }

        self.render_saved_eq_floating_popup(ui);
    }
}
