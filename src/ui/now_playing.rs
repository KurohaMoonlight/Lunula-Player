use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::WaveformMode;
use crate::app::LunulaApp;
use crate::ui::components::{format_time, lerp_color, render_card_frame};

fn parse_lyric_singer_and_text(raw: &str) -> (Option<&str>, &str) {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') {
        if let Some(end) = trimmed.find('}') {
            let singer = trimmed[1..end].trim();
            let rest = trimmed[end + 1..].trim();
            if !singer.is_empty() && !rest.is_empty() {
                return (Some(singer), rest);
            }
        }
    }
    if trimmed.starts_with('[') && !trimmed.starts_with("[0") && !trimmed.starts_with("[1") && !trimmed.starts_with("[2") {
        if let Some(end) = trimmed.find(']') {
            let singer = trimmed[1..end].trim();
            let rest = trimmed[end + 1..].trim();
            if !singer.is_empty() && !rest.is_empty() {
                return (Some(singer), rest);
            }
        }
    }
    (None, trimmed)
}

impl LunulaApp {
    pub fn render_ambient_waveform_station(&mut self, ui: &mut egui::Ui) {
        render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                // Top Header: Real Embedded Album Cover Art & Live Info
                ui.horizontal(|ui| {
                    let full_header_w = ui.available_width();
                    let is_narrow_header = full_header_w < 560.0;

                    // Album Cover Art Box (56x56 px, or 48x48 on narrow)
                    let art_dim = if is_narrow_header { 48.0 } else { 56.0 };
                    let art_size = Vec2::splat(art_dim);
                    let (art_rect, _) = ui.allocate_exact_size(art_size, egui::Sense::hover());
                    let art_painter = ui.painter_at(art_rect);

                    if let Some(ref texture) = self.current_cover_texture {
                        art_painter.rect_filled(art_rect, 8.0, Color32::from_rgb(15, 23, 42));
                        art_painter.image(
                            texture.id(),
                            art_rect,
                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        art_painter.rect_stroke(art_rect, 8.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(226, 232, 240, 180)), egui::StrokeKind::Outside);
                    } else {
                        crate::ui::components::draw_album_art(
                            &art_painter,
                            art_rect,
                            &self.current_track.title,
                            self.current_track.bit_depth > 16,
                        );
                    }

                    ui.add_space(8.0);

                    let zoom_ctrl_w = if is_narrow_header { 105.0 } else { 170.0 };
                    let text_col_w = (full_header_w - art_dim - 12.0 - zoom_ctrl_w).max(110.0);

                    ui.allocate_ui(Vec2::new(text_col_w, art_dim), |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let fav_icon = if self.current_track.is_favorite { "❤️" } else { "🤍" };
                                if ui.button(egui::RichText::new(fav_icon).size(14.0)).on_hover_text("Tandai sebagai Favorit").clicked() {
                                    self.current_track.is_favorite = !self.current_track.is_favorite;
                                    let is_fav = self.current_track.is_favorite;
                                    for t in self.playlist.iter_mut() {
                                        if t.title == self.current_track.title && t.artist == self.current_track.artist {
                                            t.is_favorite = is_fav;
                                        }
                                    }
                                }
                                let title_txt = egui::RichText::new(&self.current_track.title)
                                    .strong()
                                    .size(if is_narrow_header { 14.5 } else { 16.5 })
                                    .color(Color32::from_rgb(15, 23, 42));
                                ui.add(egui::Label::new(title_txt).truncate());

                                if !is_narrow_header {
                                    let artist_txt = egui::RichText::new(format!("— {}", self.current_track.artist))
                                        .strong()
                                        .size(14.5)
                                        .color(Color32::from_rgb(100, 116, 139));
                                    ui.add(egui::Label::new(artist_txt).truncate());
                                }
                            });
                            if is_narrow_header && !self.current_track.artist.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.current_track.artist)
                                        .size(11.5)
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            } else if self.current_track.duration > 0.0 {
                                ui.label(
                                    egui::RichText::new(format!("{} • {} kHz / {}-bit Hi-Res (DR14)", self.current_track.format, self.current_track.sample_rate / 1000, self.current_track.bit_depth))
                                        .strong()
                                        .size(11.0)
                                        .color(Color32::from_rgb(14, 165, 233)),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("Lunula Engine Standby • Buka Library untuk Memilih Musik")
                                        .strong()
                                        .size(11.0)
                                        .color(Color32::from_rgb(148, 163, 184)),
                                );
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let zoom_factor = self.zoom_level / 16.0;
                        if ui.button(egui::RichText::new("➕").size(10.5)).on_hover_text("Zoom In Waveform").clicked() {
                            self.zoom_level = (self.zoom_level * 1.25).clamp(4.0, 140.0);
                        }
                        let zoom_lbl = if is_narrow_header {
                            format!("🔍{:.1}x", zoom_factor)
                        } else {
                            format!("🔍 {:.1}x (Ctrl+Scroll)", zoom_factor)
                        };
                        ui.label(
                            egui::RichText::new(zoom_lbl)
                                .strong()
                                .size(11.0)
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                        if ui.button(egui::RichText::new("➖").size(10.5)).on_hover_text("Zoom Out Waveform").clicked() {
                            self.zoom_level = (self.zoom_level * 0.80).clamp(4.0, 140.0);
                        }
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
                    let (is_ctrl, scroll_y) = ui.input(|i| {
                        let mut delta = i.smooth_scroll_delta.y;
                        if delta.abs() < 0.01 {
                            for event in &i.raw.events {
                                if let egui::Event::MouseWheel { delta: d, .. } = event {
                                    delta += d.y;
                                }
                            }
                        }
                        (i.modifiers.ctrl, delta)
                    });
                    if is_ctrl && scroll_y.abs() > 0.1 {
                        let factor = if scroll_y > 0.0 { 1.18 } else { 0.85 };
                        self.zoom_level = (self.zoom_level * factor).clamp(4.0, 140.0);
                        ui.ctx().request_repaint();
                    }
                }

                // Drag Scrubbing
                if wave_res.dragged() {
                    let delta_x = ui.input(|i| i.pointer.delta().x);
                    let time_delta = -delta_x / self.zoom_level;
                    self.current_time = (self.current_time + time_delta).clamp(0.0, self.current_track.duration);
                    if let Some(ref engine) = self.audio_engine {
                        engine.seek(self.current_time);
                    }
                } else if wave_res.clicked() {
                    if let Some(pos) = wave_res.interact_pointer_pos() {
                        let center_x = wave_rect.center().x;
                        let offset_x = pos.x - center_x;
                        let target_time = self.current_time + (offset_x / self.zoom_level);
                        self.current_time = target_time.clamp(0.0, self.current_track.duration);
                        if let Some(ref engine) = self.audio_engine {
                            engine.seek(self.current_time);
                        }
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

                // Synchronize wave oscilloscope amplitude directly with live Volume & Mute (steady, clean, no endut-endut)
                let vol_scale = if self.is_muted {
                    0.0
                } else {
                    self.volume.clamp(0.0, 1.2)
                };
                let effective_scale = vol_scale;

                // Extract live physical frequency energy from real-time FFT spectrum
                // Bass (20Hz - 250Hz, bands 0..10)
                let live_bass = self.spectrum_bars[0..10].iter().cloned().fold(0.0f32, f32::max);
                // Mid (250Hz - 4kHz, bands 10..28)
                let live_mid = self.spectrum_bars[10..28].iter().cloned().fold(0.0f32, f32::max);
                // Treble (4kHz - 20kHz, bands 28..48)
                let live_treble = self.spectrum_bars[28..48].iter().cloned().fold(0.0f32, f32::max);

                let bass_boost = if self.is_playing { (1.0 + live_bass * 1.6).min(2.8) } else { 1.0 };
                let mid_boost = if self.is_playing { (1.0 + live_mid * 1.5).min(2.5) } else { 1.0 };
                let treble_boost = if self.is_playing { (1.0 + live_treble * 1.8).min(2.8) } else { 1.0 };

                if duration > 0.0 && !samples.is_empty() {
                    let px_step = 2.0;
                    let half_step = px_step * 0.5;
                    let count = (wave_width / px_step) as usize;
                    let mut wave_mesh = egui::Mesh::default();

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

                            if effective_scale <= 0.001 {
                                // Muted / zero volume: flat oscilloscope trace
                                let flat_col = if is_played {
                                    Color32::from_rgb(14, 165, 233)
                                } else {
                                    Color32::from_rgb(203, 213, 225)
                                };
                                wave_mesh.add_rect_with_uv(
                                    Rect::from_min_max(Pos2::new(screen_x - half_step, base_y), Pos2::new(screen_x + half_step, base_y + 1.2)),
                                    Rect::ZERO,
                                    flat_col,
                                );
                                continue;
                            }

                            match self.waveform_mode {
                                WaveformMode::StackedMultiBand => {
                                    // 3 Layer Bertumpuk (Bass -> Mid -> Treble) scaled to live Volume and real dynamic multi-band energy
                                    // Bass (Foundation / Lows <250Hz in Magenta #D946EF)
                                    let h_bass = (b_amp * max_wave_height * 1.35 * bass_boost * effective_scale).clamp(1.0, max_wave_height * 0.98);
                                    let bass_top = base_y - h_bass;
                                    let (bass_fill, bass_stroke) = if is_played {
                                        (
                                            Color32::from_rgba_unmultiplied(217, 70, 239, 55),
                                            Color32::from_rgb(217, 70, 239),
                                        )
                                    } else {
                                        (
                                            Color32::from_rgba_unmultiplied(217, 70, 239, 18),
                                            Color32::from_rgba_unmultiplied(217, 70, 239, 100),
                                        )
                                    };
                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, bass_top), Pos2::new(screen_x + half_step, base_y)),
                                        Rect::ZERO,
                                        bass_fill,
                                    );
                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, bass_top), Pos2::new(screen_x + half_step, bass_top + 1.8)),
                                        Rect::ZERO,
                                        bass_stroke,
                                    );

                                    // Mid (Vocals / Harmonics 250Hz-4kHz in Cyan #0EA5E9) modulated with live Mid energy
                                    let h_mid = (m_amp * max_wave_height * 0.85 * mid_boost * effective_scale).clamp(1.0, max_wave_height * 0.92);
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
                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, mid_top), Pos2::new(screen_x + half_step, base_y)),
                                        Rect::ZERO,
                                        mid_fill,
                                    );
                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, mid_top), Pos2::new(screen_x + half_step, mid_top + 1.8)),
                                        Rect::ZERO,
                                        mid_stroke,
                                    );

                                    // Treble (Hi-hats / Air >4kHz in Amber #F59E0B) modulated with live Treble energy
                                    let h_treble = (tr_amp * max_wave_height * 0.60 * treble_boost * effective_scale).clamp(1.0, max_wave_height * 0.88);
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
                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, treble_top), Pos2::new(screen_x + half_step, base_y)),
                                        Rect::ZERO,
                                        treble_fill,
                                    );
                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, treble_top), Pos2::new(screen_x + half_step, treble_top + 1.8)),
                                        Rect::ZERO,
                                        treble_stroke,
                                    );
                                }
                                WaveformMode::SpectralBlend => {
                                    // Spectral Color Blending dynamically weighted by live Bass, Mid, and Treble
                                    let eff_b = b_amp * bass_boost;
                                    let eff_m = m_amp * mid_boost;
                                    let eff_tr = tr_amp * treble_boost;
                                    let sum = (eff_b + eff_m + eff_tr).max(0.001);
                                    let w_b = eff_b / sum;
                                    let w_m = eff_m / sum;
                                    let w_tr = eff_tr / sum;

                                    let r = ((w_b * 217.0 + w_m * 14.0 + w_tr * 245.0).clamp(0.0, 255.0)) as u8;
                                    let g = ((w_b * 70.0 + w_m * 165.0 + w_tr * 158.0).clamp(0.0, 255.0)) as u8;
                                    let b = ((w_b * 239.0 + w_m * 233.0 + w_tr * 11.0).clamp(0.0, 255.0)) as u8;

                                    let blend_height_scale = (0.85 + live_bass * 0.35 + live_mid * 0.25 + live_treble * 0.20).min(2.6);
                                    let h = (amp * max_wave_height * blend_height_scale * effective_scale).clamp(1.0, max_wave_height * 0.98);
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

                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, top_y), Pos2::new(screen_x + half_step, base_y)),
                                        Rect::ZERO,
                                        fill_col,
                                    );
                                    wave_mesh.add_rect_with_uv(
                                        Rect::from_min_max(Pos2::new(screen_x - half_step, top_y), Pos2::new(screen_x + half_step, top_y + 2.0)),
                                        Rect::ZERO,
                                        top_stroke_col,
                                    );
                                }
                            }
                        }
                    }

                    // Submit entire waveform in 1 single GPU mesh call!
                    painter.add(egui::Shape::Mesh(std::sync::Arc::new(wave_mesh)));

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
                            if let Some(ref engine) = self.audio_engine {
                                engine.seek(self.current_time);
                            }
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
                    let avail_w = ui.available_width();
                    let is_compact_bar = avail_w < 720.0;
                    let is_narrow_bar = avail_w < 540.0;

                    if !is_narrow_bar {
                        ui.label(
                            egui::RichText::new("TAMPILAN WAVE:")
                                .size(11.0)
                                .strong()
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                    }

                    let is_stacked = self.waveform_mode == WaveformMode::StackedMultiBand;
                    let is_spectral = self.waveform_mode == WaveformMode::SpectralBlend;

                    let stacked_lbl = if is_compact_bar { "📊 3-Band" } else { "📊 3-Band Layers" };
                    let btn_stacked = egui::Button::new(
                        egui::RichText::new(stacked_lbl)
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

                    let spectral_lbl = if is_compact_bar { "🌈 Spectral" } else { "🌈 Spectral Blend" };
                    let btn_spectral = egui::Button::new(
                        egui::RichText::new(spectral_lbl)
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

                    // Right-aligned Frequency Legend (adapts cleanly to available width)
                    if !is_narrow_bar {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let treble_lbl = if is_compact_bar { "Treble" } else { "Treble (>4kHz)" };
                            ui.label(
                                egui::RichText::new(treble_lbl)
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                            ui.label(egui::RichText::new("●").size(15.0).color(Color32::from_rgb(245, 158, 11)));

                            ui.add_space(8.0);

                            let mid_lbl = if is_compact_bar { "Mid" } else { "Mid (250Hz-4kHz)" };
                            ui.label(
                                egui::RichText::new(mid_lbl)
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                            ui.label(egui::RichText::new("●").size(15.0).color(Color32::from_rgb(14, 165, 233)));

                            ui.add_space(8.0);

                            let bass_lbl = if is_compact_bar { "Bass" } else { "Bass (<250Hz)" };
                            ui.label(
                                egui::RichText::new(bass_lbl)
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                            ui.label(egui::RichText::new("●").size(15.0).color(Color32::from_rgb(217, 70, 239)));

                            if !is_compact_bar {
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new("SPECTRUM:")
                                        .size(11.0)
                                        .strong()
                                        .color(Color32::from_rgb(148, 163, 184)),
                                );
                            }
                        });
                    }
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

        let (left_width, right_width) = if full_width < 540.0 {
            let lw = (full_width * 0.25).clamp(80.0, 130.0);
            let rw = (full_width * 0.36).clamp(120.0, 180.0);
            (lw, rw)
        } else if full_width < 760.0 {
            let lw = (full_width * 0.28).clamp(130.0, 190.0);
            let rw = (full_width * 0.36).clamp(180.0, 240.0);
            (lw, rw)
        } else {
            (240.0, 280.0)
        };

        let center_width = (full_width - left_width - right_width).max(150.0);

        ui.horizontal(|ui| {
            // LEFT COLUMN: Cover Thumbnail & Confident Time Typography
            ui.allocate_ui_with_layout(
                Vec2::new(left_width, 44.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    if left_width >= 130.0 {
                        let thumb_size = Vec2::splat(36.0);
                        let (thumb_rect, _) = ui.allocate_exact_size(thumb_size, egui::Sense::hover());
                        let thumb_painter = ui.painter_at(thumb_rect);

                        if let Some(ref texture) = self.current_cover_texture {
                            thumb_painter.rect_filled(thumb_rect, 6.0, Color32::from_rgb(15, 23, 42));
                            thumb_painter.image(
                                texture.id(),
                                thumb_rect,
                                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                            thumb_painter.rect_stroke(thumb_rect, 6.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(226, 232, 240, 150)), egui::StrokeKind::Outside);
                        } else {
                            crate::ui::components::draw_album_art(
                                &thumb_painter,
                                thumb_rect,
                                &self.current_track.title,
                                self.current_track.bit_depth > 16,
                            );
                        }

                        ui.add_space(6.0);
                    }

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format_time(self.current_time))
                                .strong()
                                .size(if left_width < 130.0 { 14.5 } else { 16.0 })
                                .color(Color32::from_rgb(15, 23, 42)),
                        );

                        if left_width >= 120.0 {
                            let remaining = (self.current_track.duration - self.current_time).max(0.0);
                            ui.label(
                                egui::RichText::new(format!("-{}", format_time(remaining)))
                                    .strong()
                                    .size(11.0)
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );
                        }
                    });
                },
            );

            // CENTER COLUMN: Perfectly Centered Transport Buttons with Silky Smooth Animated Hover
            ui.allocate_ui_with_layout(
                Vec2::new(center_width, 44.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.horizontal(|ui| {
                        let btn_gap = if center_width < 220.0 { 4.0 } else { 8.0 };
                        let play_gap = if center_width < 220.0 { 6.0 } else { 12.0 };

                        let shuffle_tip = if self.is_shuffled {
                            "Shuffle: Aktif (Lagu Diacak) - Klik untuk Matikan"
                        } else {
                            "Shuffle: Mati (Urutan Normal) - Klik untuk Acak"
                        };
                        self.render_toggle_icon_btn(ui, "btn_shuffle", "🔀", 14.0, self.is_shuffled, shuffle_tip, |app| {
                            app.toggle_shuffle();
                        });
                        ui.add_space(btn_gap);

                        self.render_icon_btn(ui, "btn_prev", "⏮", 17.0, "Previous", |app| {
                            app.play_prev_track();
                        });
                        ui.add_space(play_gap);

                        // Floating Play/Pause Button with Morphing Live Oscilloscope Beam
                        let play_size = Vec2::splat(44.0);
                        let (play_rect, play_res) = ui.allocate_exact_size(play_size, egui::Sense::click());
                        let play_hover = ui.ctx().animate_bool(ui.scope_id().with("play_pause_hover"), play_res.hovered());
                        let play_morph = ui.ctx().animate_bool(ui.scope_id().with("play_morph_anim"), self.is_playing);

                        if play_morph > 0.001 && play_morph < 0.999 {
                            ui.ctx().request_repaint();
                        }

                        if play_res.clicked() {
                            self.toggle_playback();
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

                        let mut raw_samples = [0.0f32; 1024];
                        if let Some(ref engine) = self.audio_engine {
                            engine.get_visualizer_samples(&mut raw_samples);
                        }
                        let vol_scale = if self.is_muted { 0.0 } else { self.volume.clamp(0.0, 1.2) };

                        // Morphing transition:
                        // play_morph = 0.0 -> Stopped/Paused (Play Triangle)
                        // 0.0 < play_morph <= 0.35 -> Triangle collapses vertically into flat horizontal line ("garis datar")
                        // 0.35 < play_morph <= 1.0 -> Flat line begins vibrating and dances as live oscilloscope beam
                        if play_morph < 0.35 {
                            let t = (play_morph / 0.35).clamp(0.0, 1.0);
                            let r = 8.5;
                            // Morphing triangle: p1 and p2 collapse towards y=center.y
                            let p1 = center + Vec2::new(-r * 0.6, -r * (1.0 - t));
                            let p2 = center + Vec2::new(-r * 0.6, r * (1.0 - t));
                            let p3 = center + Vec2::new(r * 0.9, 0.0);
                            let alpha = ((1.0 - t) * 255.0) as u8;
                            if alpha > 5 {
                                painter.add(egui::epaint::PathShape::convex_polygon(
                                    vec![p1, p2, p3],
                                    Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                                    Stroke::NONE,
                                ));
                            }
                            // As t reaches 1.0, draw the emerging flat horizontal line
                            if t > 0.05 {
                                let line_alpha = (t * 255.0) as u8;
                                painter.line_segment(
                                    [Pos2::new(center.x - 12.0, center.y), Pos2::new(center.x + 12.0, center.y)],
                                    Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, line_alpha)),
                                );
                            }
                        } else {
                            // Live Mini Oscilloscope Beam inside the button
                            let wave_intensity = ((play_morph - 0.35) / 0.65).clamp(0.0, 1.0);
                            let beam_w = 26.0;
                            let n_pts = 26;
                            let mut pts = Vec::with_capacity(n_pts);

                            for p in 0..n_pts {
                                let frac = p as f32 / (n_pts - 1) as f32;
                                let px = (center.x - beam_w * 0.5) + frac * beam_w;
                                let window = (std::f32::consts::PI * frac).sin(); // 0 at edges, 1 at center
                                let idx = (p * (1024 / n_pts)).min(1023);
                                let sample = raw_samples[idx] * vol_scale;
                                // If stopping/paused (wave_intensity -> 0), py becomes center.y (flat horizontal line)
                                let py = center.y - (sample * 11.0 * window * wave_intensity);
                                pts.push(Pos2::new(px, py));
                            }

                            if pts.len() >= 2 {
                                // Outer neon glow
                                let glow_alpha = (140.0 * play_morph) as u8;
                                for seg in pts.windows(2) {
                                    painter.line_segment([seg[0], seg[1]], Stroke::new(3.8, Color32::from_rgba_unmultiplied(14, 165, 233, glow_alpha)));
                                }
                                // Crisp electric beam core
                                let core_alpha = (255.0 * play_morph) as u8;
                                for seg in pts.windows(2) {
                                    painter.line_segment([seg[0], seg[1]], Stroke::new(1.8, Color32::from_rgba_unmultiplied(240, 249, 255, core_alpha)));
                                }
                            }

                            // When hovered while playing, show subtle translucent pause icon overlay
                            if play_hover > 0.05 {
                                let pause_alpha = (play_hover * 210.0) as u8;
                                let bar_w = 3.0;
                                let bar_h = 11.0;
                                let gap = 2.5;
                                painter.rect_filled(
                                    Rect::from_center_size(center - Vec2::new(gap + bar_w * 0.5, 0.0), Vec2::new(bar_w, bar_h)),
                                    1.5,
                                    Color32::from_rgba_unmultiplied(255, 255, 255, pause_alpha),
                                );
                                painter.rect_filled(
                                    Rect::from_center_size(center + Vec2::new(gap + bar_w * 0.5, 0.0), Vec2::new(bar_w, bar_h)),
                                    1.5,
                                    Color32::from_rgba_unmultiplied(255, 255, 255, pause_alpha),
                                );
                            }
                        }

                        ui.add_space(play_gap);

                        self.render_icon_btn(ui, "btn_next", "⏭", 17.0, "Next", |app| {
                            app.play_next_track();
                        });
                        ui.add_space(btn_gap);

                        let (repeat_icon, is_rep_active, repeat_tip) = match self.repeat_mode {
                            crate::models::RepeatMode::Off => ("🔁", false, "Repeat: Mati (Klik untuk Ulangi Semua)"),
                            crate::models::RepeatMode::All => ("🔁", true, "Repeat: Semua Lagu Aktif (Klik untuk Ulangi 1 Lagu)"),
                            crate::models::RepeatMode::One => ("🔂", true, "Repeat: 1 Lagu Ini Saja (Klik untuk Ulangi Album Ini)"),
                            crate::models::RepeatMode::Album => ("💿", true, "Repeat: Hanya Lagu Album Ini (Klik untuk Matikan)"),
                        };
                        self.render_toggle_icon_btn(ui, "btn_repeat", repeat_icon, 14.0, is_rep_active, repeat_tip, |app| {
                            app.cycle_repeat_mode();
                        });
                    });
                },
            );

            // RIGHT COLUMN: Sleek Modern Volume Slider & Queue Drawer Toggle
            ui.allocate_ui_with_layout(
                Vec2::new(right_width, 44.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    // Queue Drawer Toggle Button
                    let q_count = self.play_queue.len();
                    let q_text = if right_width < 190.0 {
                        if q_count > 0 { format!("📑 {}", q_count) } else { "📑".to_string() }
                    } else if q_count > 0 {
                        format!("📑 Antrean ({})", q_count)
                    } else {
                        "📑 Antrean".to_string()
                    };
                    let q_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(q_text)
                                .strong()
                                .size(11.0)
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
                        .min_size(Vec2::new(26.0, 26.0)),
                    ).on_hover_text("Buka / Tutup Panel Antrean Lagu (Queue)");

                    if q_btn.clicked() {
                        self.show_queue_drawer = !self.show_queue_drawer;
                    }

                    ui.add_space(6.0);

                    let is_bp = self.chain_mode == crate::models::ChainMode::BitPerfect;

                    if is_bp {
                        let bp_lbl = if right_width < 220.0 { "100%" } else { "100% (Direct)" };
                        ui.label(
                            egui::RichText::new(bp_lbl)
                                .strong()
                                .size(11.0)
                                .color(Color32::from_rgb(16, 185, 129)),
                        ).on_hover_text("Volume dikunci pada 100% (Bit-Perfect Pure Direct) untuk menjaga integritas bit audio asli tanpa attenuasi digital. Gunakan knob hardware DAC / speaker.");
                    } else if right_width >= 160.0 {
                        let vol_pct = (if self.is_muted { 0.0 } else { self.volume } * 100.0) as u32;
                        ui.label(
                            egui::RichText::new(format!("{:3}%", vol_pct))
                                .strong()
                                .size(12.0)
                                .color(Color32::from_rgb(71, 85, 105)),
                        );
                    }

                    ui.add_space(6.0);

                    let slider_w = (right_width - if is_bp { 75.0 } else if right_width < 160.0 { 55.0 } else { 85.0 }).clamp(35.0, 100.0);
                    let vol_size = Vec2::new(slider_w, 10.0);
                    let (vol_rect, vol_res) = ui.allocate_exact_size(vol_size, if is_bp { egui::Sense::hover() } else { egui::Sense::click_and_drag() });
                    let vol_hover = ui.ctx().animate_bool(ui.scope_id().with("vol_hover_anim"), vol_res.hovered() || vol_res.dragged());

                    if is_bp {
                        vol_res.on_hover_text("Mode Bit-Perfect Aktif: Volume digital dikunci di 100% murni. Ubah ke mode 'DSP Active' di Patchbay untuk mengaktifkan pengaturan volume software.");
                    } else if vol_res.clicked() || vol_res.dragged() {
                        if let Some(pos) = vol_res.interact_pointer_pos() {
                            let ratio = ((pos.x - vol_rect.min.x) / vol_rect.width()).clamp(0.0, 1.0);
                            self.set_app_volume(ratio);
                        }
                    }

                    let vol_painter = ui.painter_at(vol_rect);
                    vol_painter.rect_filled(vol_rect, 5.0, Color32::from_rgb(226, 232, 240));

                    if is_bp {
                        // Locked Bit-Perfect direct 100% pure bar
                        vol_painter.rect_filled(vol_rect, 5.0, Color32::from_rgb(16, 185, 129));
                    } else {
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
                            egui::RichText::new(vol_icon).strong().size(14.0).color(Color32::from_rgb(15, 23, 42)),
                        )
                        .frame(false),
                    );

                    if speaker_btn.clicked() {
                        self.toggle_mute();
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

    pub fn render_toggle_icon_btn<F>(
        &mut self,
        ui: &mut egui::Ui,
        id_str: &str,
        icon: &str,
        font_size: f32,
        is_active: bool,
        tooltip: &str,
        mut on_click: F,
    ) where
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

        if is_active {
            let bg_col = if hover_anim > 0.0 {
                Color32::from_rgb(207, 250, 254)
            } else {
                Color32::from_rgb(224, 242, 254)
            };
            painter.circle_filled(center, 16.0, bg_col);
            painter.circle_stroke(center, 16.0, Stroke::new(1.0, Color32::from_rgb(186, 230, 253)));
            painter.circle_filled(center + Vec2::new(0.0, 12.0), 2.0, Color32::from_rgb(2, 132, 199));
        } else if hover_anim > 0.0 {
            let circle_color = Color32::from_rgba_unmultiplied(241, 245, 249, (hover_anim * 255.0) as u8);
            painter.circle_filled(center, 16.0, circle_color);
        }

        let icon_color = if is_active {
            Color32::from_rgb(2, 132, 199)
        } else {
            lerp_color(Color32::from_rgb(100, 116, 139), Color32::from_rgb(15, 23, 42), hover_anim)
        };

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(font_size),
            icon_color,
        );

        res.on_hover_text(tooltip);
    }

    /// Split Layout: 54% Real-Time Spectrum Analyzer + 46% Synchronized Lyric Box (responsive)
    pub fn render_now_playing_visualizer_and_lyrics(&mut self, ui: &mut egui::Ui) {
        let total_w = ui.available_width();
        let (spec_w, lyric_w) = if total_w < 560.0 {
            let half = (total_w - 8.0) * 0.5;
            (half.max(120.0), half.max(120.0))
        } else {
            let sw = (total_w * 0.54).clamp(240.0, total_w - 200.0);
            let lw = total_w - sw - 8.0;
            (sw, lw)
        };

        let stage_h = (ui.available_height() - 8.0).clamp(160.0, 420.0);

        ui.horizontal(|ui| {
            // Left: Spectrum Analyzer
            ui.allocate_ui(egui::vec2(spec_w, stage_h), |ui| {
                self.render_spectrum_visualizer(ui);
            });

            ui.add_space(8.0);

            // Right: Synchronized Lyric Box
            ui.allocate_ui(egui::vec2(lyric_w, stage_h), |ui| {
                self.render_synced_lyric_box(ui);
            });
        });
    }

    pub fn render_synced_lyric_box(&mut self, ui: &mut egui::Ui) {
        if self.current_lyrics.is_none() {
            if let Some(ref p) = self.current_track.file_path {
                self.current_lyrics = crate::lyrics::Lyrics::load_for_track_smart(
                    std::path::Path::new(p),
                    &self.current_track.title,
                    &self.current_track.artist,
                );
            }
        }

        crate::ui::components::render_card_frame(ui, |ui| {
            ui.vertical(|ui| {
                // 1. Sleek Modern Header Bar
                ui.horizontal(|ui| {
                    let box_w = ui.available_width();
                    let is_narrow_box = box_w < 340.0;

                    ui.label(egui::RichText::new("🎙️").size(15.5));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let title_txt = if is_narrow_box { "LIRIK" } else { "LIRIK TERSINKRONISASI" };
                            ui.label(
                                egui::RichText::new(title_txt)
                                    .strong()
                                    .size(12.5)
                                    .color(Color32::from_rgb(15, 23, 42)),
                            );
                            if self.current_lyrics.is_some() {
                                egui::Frame::new()
                                    .fill(Color32::from_rgb(236, 253, 245))
                                    .corner_radius(4.0)
                                    .inner_margin(egui::Margin::symmetric(6, 1))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new("● SYNC")
                                                .strong()
                                                .size(9.5)
                                                .color(Color32::from_rgb(16, 185, 129)),
                                        );
                                    });
                            }
                        });
                        if !is_narrow_box {
                            ui.label(
                                egui::RichText::new("Real-time LRC Engine • Klik lirik lompat audio")
                                    .size(10.0)
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Manual file picker button
                        let lrc_btn_label = if is_narrow_box { "📂 LRC" } else { "📂 Muat .lrc" };
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new(lrc_btn_label)
                                    .size(10.5)
                                    .strong()
                                    .color(Color32::from_rgb(79, 70, 229))
                            )
                            .fill(Color32::from_rgb(238, 242, 255))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(199, 210, 254)))
                            .corner_radius(6.0)
                            .min_size(Vec2::new(if is_narrow_box { 56.0 } else { 82.0 }, 26.0))
                        )
                        .on_hover_text("Pilih berkas lirik .lrc secara manual dari disk")
                        .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new().add_filter("LRC Lyrics", &["lrc", "txt"]).pick_file() {
                                if let Some(content) = crate::lyrics::read_lrc_file_robust(&path) {
                                    self.current_lyrics = Some(crate::lyrics::Lyrics::parse(&content));
                                    self.lyrics_offset_trim_sec = 0.0;
                                }
                            }
                        }

                        // Offset trim segmented controls: [-0.5s] [Offset] [+0.5s]
                        if self.current_lyrics.is_some() {
                            if ui.add(
                                egui::Button::new(egui::RichText::new("+0.5s").size(10.0))
                                    .fill(Color32::from_rgb(241, 245, 249))
                                    .corner_radius(5.0)
                            )
                            .on_hover_text("Mundurkan waktu lirik 0.5 detik")
                            .clicked()
                            {
                                self.lyrics_offset_trim_sec += 0.5;
                            }

                            let offset_label = if self.lyrics_offset_trim_sec.abs() > 0.01 {
                                format!("{:+.1}s", self.lyrics_offset_trim_sec)
                            } else {
                                "±0.0s".to_string()
                            };

                            let offset_col = if self.lyrics_offset_trim_sec.abs() > 0.01 {
                                Color32::from_rgb(79, 70, 229)
                            } else {
                                Color32::from_rgb(148, 163, 184)
                            };

                            if ui.add(
                                egui::Button::new(egui::RichText::new(offset_label).size(10.0).strong().color(offset_col))
                                    .fill(Color32::from_rgb(248, 250, 252))
                                    .corner_radius(5.0)
                            )
                            .on_hover_text("Klik untuk mereset offset waktu lirik ke 0.0 detik")
                            .clicked()
                            {
                                self.lyrics_offset_trim_sec = 0.0;
                            }

                            if ui.add(
                                egui::Button::new(egui::RichText::new("-0.5s").size(10.0))
                                    .fill(Color32::from_rgb(241, 245, 249))
                                    .corner_radius(5.0)
                            )
                            .on_hover_text("Majukan waktu lirik 0.5 detik")
                            .clicked()
                            {
                                self.lyrics_offset_trim_sec -= 0.5;
                            }
                        }
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                let mut seek_target = None;

                if let Some(ref lyrics) = self.current_lyrics {
                    if lyrics.lines.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(30.0);
                            ui.label(egui::RichText::new("📄 Berkas .lrc tidak memiliki baris lirik dengan stempel waktu").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                        });
                    } else {
                        let active_time = (self.current_time + self.lyrics_offset_trim_sec).max(0.0);
                        let active_idx = lyrics.find_active_index(active_time);

                        let scroll_h = (ui.available_height() - 4.0).max(160.0);

                        egui::ScrollArea::vertical()
                            .max_height(scroll_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add_space(24.0); // Top padding so first line can scroll to center

                                for (i, line) in lyrics.lines.iter().enumerate() {
                                    let is_active = Some(i) == active_idx;
                                    let (singer_opt, clean_text) = parse_lyric_singer_and_text(&line.text);
                                    let display_text = if clean_text.trim().is_empty() { "♪ ♪ ♪" } else { clean_text };

                                    let row_id = ui.scope_id().with(("lyric_row", i));
                                    let row_h = if is_active { 42.0 } else { 32.0 };
                                    let row_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), row_h));
                                    let line_click = ui.interact(row_rect, row_id, egui::Sense::click());
                                    let is_hovered = line_click.hovered();

                                    let time_str = format_time(line.timestamp_sec);

                                    if is_active {
                                        let resp = egui::Frame::new()
                                            .fill(Color32::from_rgb(238, 242, 255))
                                            .stroke(Stroke::new(1.2, Color32::from_rgb(199, 210, 254)))
                                            .corner_radius(8.0)
                                            .inner_margin(egui::Margin::symmetric(10, 7))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    // Glowing vertical indicator bar
                                                    let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 18.0), egui::Sense::hover());
                                                    ui.painter().rect_filled(bar_rect, 1.5, Color32::from_rgb(79, 70, 229));
                                                    ui.add_space(6.0);

                                                    // Timestamp on the LEFT: Always visible, bold indigo
                                                    ui.allocate_ui(Vec2::new(42.0, 18.0), |ui| {
                                                        ui.label(
                                                            egui::RichText::new(&time_str)
                                                                .size(11.5)
                                                                .strong()
                                                                .color(Color32::from_rgb(79, 70, 229)),
                                                        );
                                                    });

                                                    ui.add_space(4.0);

                                                    if let Some(singer) = singer_opt {
                                                        egui::Frame::new()
                                                            .fill(Color32::from_rgb(224, 231, 255))
                                                            .corner_radius(4.0)
                                                            .inner_margin(egui::Margin::symmetric(6, 2))
                                                            .show(ui, |ui| {
                                                                ui.label(egui::RichText::new(singer).strong().size(11.0).color(Color32::from_rgb(67, 56, 202)));
                                                            });
                                                        ui.add_space(4.0);
                                                    }

                                                    ui.label(
                                                        egui::RichText::new(display_text)
                                                            .strong()
                                                            .size(15.5)
                                                            .color(Color32::from_rgb(15, 23, 42)),
                                                    );
                                                });
                                            });
                                        resp.response.scroll_to_me(Some(egui::Align::Center));
                                    } else {
                                        let bg_col = if is_hovered {
                                            Color32::from_rgb(241, 245, 249)
                                        } else {
                                            Color32::TRANSPARENT
                                        };

                                        egui::Frame::new()
                                            .fill(bg_col)
                                            .corner_radius(6.0)
                                            .inner_margin(egui::Margin::symmetric(10, 6))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    // Placeholder space matching the 3.0px bar + 6.0px space of active line
                                                    ui.add_space(9.0);

                                                    // Timestamp on the LEFT: Always visible for inactive lines too
                                                    let ts_color = if is_hovered {
                                                        Color32::from_rgb(71, 85, 105)
                                                    } else {
                                                        Color32::from_rgb(148, 163, 184)
                                                    };
                                                    ui.allocate_ui(Vec2::new(42.0, 18.0), |ui| {
                                                        ui.label(
                                                            egui::RichText::new(&time_str)
                                                                .size(11.5)
                                                                .color(ts_color),
                                                        );
                                                    });

                                                    ui.add_space(4.0);

                                                    if let Some(singer) = singer_opt {
                                                        egui::Frame::new()
                                                            .fill(Color32::from_rgb(241, 245, 249))
                                                            .corner_radius(4.0)
                                                            .inner_margin(egui::Margin::symmetric(5, 1))
                                                            .show(ui, |ui| {
                                                                ui.label(egui::RichText::new(singer).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                                            });
                                                        ui.add_space(4.0);
                                                    }

                                                    let text_col = if is_hovered {
                                                        Color32::from_rgb(15, 23, 42)
                                                    } else {
                                                        Color32::from_rgb(100, 116, 139)
                                                    };

                                                    ui.label(
                                                        egui::RichText::new(display_text)
                                                            .size(13.0)
                                                            .color(text_col),
                                                    );
                                                });
                                            });
                                    }

                                    if line_click.clicked() {
                                        seek_target = Some(line.timestamp_sec);
                                    }

                                    ui.add_space(3.0);
                                }

                                ui.add_space(40.0); // Bottom padding so last line can scroll to center
                            });
                    }
                } else {
                    // Empty state
                    ui.vertical_centered(|ui| {
                        ui.add_space(28.0);
                        ui.label(egui::RichText::new("🎙️").size(36.0));
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Belum Ada Lirik Tersinkronisasi")
                                .strong()
                                .size(14.5)
                                .color(Color32::from_rgb(15, 23, 42)),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Simpan file .lrc di folder lagu dengan nama yang sama, atau muat secara manual.")
                                .size(11.5)
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                        ui.add_space(14.0);
                        if ui.add(
                            egui::Button::new(
                                egui::RichText::new("📂 Pilih Berkas Lirik (.lrc)")
                                    .strong()
                                    .size(12.0)
                                    .color(Color32::WHITE)
                            )
                            .fill(Color32::from_rgb(79, 70, 229))
                            .corner_radius(6.0)
                            .min_size(Vec2::new(170.0, 30.0))
                        ).clicked() {
                            if let Some(path) = rfd::FileDialog::new().add_filter("LRC Lyrics", &["lrc", "txt"]).pick_file() {
                                if let Some(content) = crate::lyrics::read_lrc_file_robust(&path) {
                                    self.current_lyrics = Some(crate::lyrics::Lyrics::parse(&content));
                                    self.lyrics_offset_trim_sec = 0.0;
                                }
                            }
                        }
                    });
                }

                if let Some(target) = seek_target {
                    self.seek_to(target);
                }
            });
        });
    }
}

