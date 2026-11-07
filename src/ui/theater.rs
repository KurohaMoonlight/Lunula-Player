use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use crate::app::LunulaApp;
use crate::ui::mascot::{draw_beamed_pair_mascot_ex, MascotExpression};

/// Speaker identifier for dialogue turns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    Lu,
    Nu,
    Both,
}

impl Speaker {
    pub fn badge(&self) -> &'static str {
        match self {
            Speaker::Lu => "🧐 Lu (Sang Audiophile Purist)",
            Speaker::Nu => "⚡ Nu (Sang Arsitek Studio & DSP)",
            Speaker::Both => "🎭 Lu & Nu (Duo Not Balok Kembar)",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Speaker::Lu => Color32::from_rgb(251, 191, 36),   // Warm Audiophile Gold
            Speaker::Nu => Color32::from_rgb(56, 189, 248),   // Electric Cyan
            Speaker::Both => Color32::from_rgb(168, 85, 247), // Studio Purple
        }
    }
}

/// A single dialogue turn within a slide
pub struct DialogueTurn {
    pub speaker: Speaker,
    pub text: &'static str,
    pub expression: MascotExpression,
}

/// Icons supported by the pixel-perfect studio button renderer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonIcon {
    None,
    Close,   // Procedural crisp ✕
    Play,    // Procedural crisp ▶
    Pause,   // Procedural crisp ⏸
    Prev,    // Procedural crisp ◀
    Rocket,  // Launch rocket
}

/// Placement of the icon relative to the button text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconPosition {
    Left,
    Right,
}

/// Renders a modern studio button with mathematically aligned procedural symbols and text.
/// Solves font metric disparities (such as symbols rendering too high relative to lowercase text).
pub fn render_studio_button(
    ui: &mut egui::Ui,
    text: &str,
    icon: ButtonIcon,
    icon_pos: IconPosition,
    enabled: bool,
    fill: Color32,
    stroke: Stroke,
    text_color: Color32,
    corner_radius: f32,
) -> egui::Response {
    let font_id = egui::FontId::proportional(11.5);
    let painter = ui.painter();

    let (eff_fill, eff_stroke, eff_text_col) = if enabled {
        (fill, stroke, text_color)
    } else {
        (
            Color32::from_rgb(16, 22, 32),
            Stroke::new(1.0, Color32::from_rgb(32, 40, 56)),
            Color32::from_rgb(80, 95, 120),
        )
    };

    let galley = painter.layout_no_wrap(text.to_string(), font_id, eff_text_col);
    let text_w = galley.size().x;
    let text_h = galley.size().y;

    let has_icon = icon != ButtonIcon::None;
    let icon_w = if has_icon { 12.0 } else { 0.0 };
    let icon_gap = if has_icon { 6.5 } else { 0.0 };
    let pad_x = 12.0;
    let total_w = text_w + icon_w + icon_gap + pad_x * 2.0;
    let total_h = 28.0;

    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(total_w, total_h),
        if enabled { egui::Sense::click() } else { egui::Sense::hover() },
    );

    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let cy = rect.center().y;

        // Background & border interaction
        let bg = if !enabled {
            eff_fill
        } else if resp.is_pointer_button_down_on() {
            Color32::from_rgb(
                eff_fill.r().saturating_add(25),
                eff_fill.g().saturating_add(25),
                eff_fill.b().saturating_add(25),
            )
        } else if resp.hovered() {
            Color32::from_rgb(
                eff_fill.r().saturating_add(15),
                eff_fill.g().saturating_add(15),
                eff_fill.b().saturating_add(15),
            )
        } else {
            eff_fill
        };

        let b_stroke = if enabled && resp.hovered() {
            Stroke::new(eff_stroke.width * 1.15, eff_stroke.color)
        } else {
            eff_stroke
        };

        p.rect_filled(rect, corner_radius, bg);
        p.rect_stroke(rect, corner_radius, b_stroke, egui::StrokeKind::Outside);

        // Calculate positions
        let content_w = text_w + if has_icon { icon_w + icon_gap } else { 0.0 };
        let start_x = rect.min.x + (rect.width() - content_w) * 0.5;

        let (text_x, icon_cx) = match icon_pos {
            IconPosition::Left => {
                let ic_x = start_x + icon_w * 0.5;
                let tx = start_x + icon_w + icon_gap;
                (tx, ic_x)
            }
            IconPosition::Right => {
                let tx = start_x;
                let ic_x = start_x + text_w + icon_gap + icon_w * 0.5;
                (tx, ic_x)
            }
        };

        // Render text: centered vertically around cy
        let text_pos = Pos2::new(text_x, cy - text_h * 0.5);
        p.galley(text_pos, galley, eff_text_col);

        // Render procedural icon: Optical center adjusted (+0.5px for perfect harmony with lowercase x-height)
        if has_icon {
            let ic_col = eff_text_col;
            let icon_cy = cy + 0.5;
            match icon {
                ButtonIcon::Close => {
                    let r = 3.6;
                    let strk = Stroke::new(1.6, ic_col);
                    p.line_segment([Pos2::new(icon_cx - r, icon_cy - r), Pos2::new(icon_cx + r, icon_cy + r)], strk);
                    p.line_segment([Pos2::new(icon_cx - r, icon_cy + r), Pos2::new(icon_cx + r, icon_cy - r)], strk);
                }
                ButtonIcon::Play => {
                    let r = 4.0;
                    let pts = vec![
                        Pos2::new(icon_cx - r * 0.65, icon_cy - r),
                        Pos2::new(icon_cx + r * 0.85, icon_cy),
                        Pos2::new(icon_cx - r * 0.65, icon_cy + r),
                    ];
                    p.add(egui::Shape::convex_polygon(pts, ic_col, Stroke::NONE));
                }
                ButtonIcon::Prev => {
                    let r = 4.0;
                    let pts = vec![
                        Pos2::new(icon_cx + r * 0.65, icon_cy - r),
                        Pos2::new(icon_cx - r * 0.85, icon_cy),
                        Pos2::new(icon_cx + r * 0.65, icon_cy + r),
                    ];
                    p.add(egui::Shape::convex_polygon(pts, ic_col, Stroke::NONE));
                }
                ButtonIcon::Pause => {
                    let bar_h = 8.0;
                    let bar_w = 2.4;
                    let b1 = Rect::from_center_size(Pos2::new(icon_cx - 2.5, icon_cy), Vec2::new(bar_w, bar_h));
                    let b2 = Rect::from_center_size(Pos2::new(icon_cx + 2.5, icon_cy), Vec2::new(bar_w, bar_h));
                    p.rect_filled(b1, 0.5, ic_col);
                    p.rect_filled(b2, 0.5, ic_col);
                }
                ButtonIcon::Rocket => {
                    let rk_galley = p.layout_no_wrap("🚀".to_string(), egui::FontId::proportional(11.0), ic_col);
                    let rk_pos = Pos2::new(icon_cx - rk_galley.size().x * 0.5, cy - rk_galley.size().y * 0.5);
                    p.galley(rk_pos, rk_galley, ic_col);
                }
                ButtonIcon::None => {}
            }
        }
    }

    resp
}

impl LunulaApp {
    /// Renders the 90% Screen Interactive Studio Theater (Slides 1 & 2 Focus)
    pub fn render_interactive_theater(&mut self, ui: &mut egui::Ui) {
        if !self.show_interactive_theater {
            return;
        }

        let ctx = ui.ctx().clone();
        let screen_rect = ctx.input(|i| i.raw.screen_rect).unwrap_or_else(|| ui.clip_rect());
        let dt = ctx.input(|i| i.stable_dt).min(0.05);
        let time = ctx.input(|i| i.time);
        let t = time as f32;

        // Advance dialogue timer for typewriter and auto-play
        self.theater_turn_timer += dt;

        // --- AUTOMATIC SCENE ANIMATION DRIVEN BY DIALOGUE TURNS ---
        // Slide 1:
        // Turns 0, 1 (Lu explains Bit-Perfect 1:1) => Bit-Perfect direct cable to DAC.
        // Turns 2, 3 (Nu interrupts and explains DSP Active) => Automatically routes through DSP Engine to DAC!
        if self.theater_slide == 0 {
            if self.theater_turn < 2 {
                self.theater_dsp_active = false;
            } else if self.theater_turn == 2 || self.theater_turn == 3 {
                self.theater_dsp_active = true;
            }
        } else if self.theater_slide == 1 {
            // Slide 2:
            // Turns 0, 1 (Linear initial DAG chain) => Single series path.
            // Turns 2, 3, 4, 5 (Nu cuts cable, introduces Splitter, LPF, Summing, explains Dry & Wet) =>
            // Automatically spawns the 3 modules and connects Dry & Wet parallel routing!
            if self.theater_turn < 2 {
                self.theater_split_sum_active = false;
            } else {
                self.theater_split_sum_active = true;
            }
        } else if self.theater_slide == 2 {
            // Slide 3: Filter Studio, Telemetric Graphs & DAG Sacred Rules (8 Detailed Turns)
            // Turns 0, 1: 10-Band EQ telemetric curve focus
            // Turn 2: Tube 12AX7 telemetric harmonic spectrum focus
            // Turn 3: Low-Pass Filter telemetric roll-off focus
            // Turn 4: DAG Direction & Linear Flow (Normal Order: Tube -> EQ)
            // Turn 5: Order Matters demonstration (Swapped Order: EQ -> Tube)
            // Turn 6: Acyclic Rule: Anti-Loop Protection Barrier triggered
            // Turn 7: Serial Cascading: Multi-Filter in series to DAC
            match self.theater_turn {
                0 | 1 => {
                    self.theater_s3_selected_filter = 0; // EQ
                    self.theater_s3_order_swapped = false;
                    self.theater_s3_loop_simulated = false;
                }
                2 => {
                    self.theater_s3_selected_filter = 1; // Tube 12AX7
                    self.theater_s3_order_swapped = false;
                    self.theater_s3_loop_simulated = false;
                }
                3 => {
                    self.theater_s3_selected_filter = 2; // LPF
                    self.theater_s3_order_swapped = false;
                    self.theater_s3_loop_simulated = false;
                }
                4 => {
                    self.theater_s3_order_swapped = false; // Normal order: Tube -> EQ
                    self.theater_s3_loop_simulated = false;
                }
                5 => {
                    self.theater_s3_order_swapped = true; // Swapped order: EQ -> Tube
                    self.theater_s3_loop_simulated = false;
                }
                6 => {
                    self.theater_s3_order_swapped = false;
                    self.theater_s3_loop_simulated = true; // Loop attempt blocked!
                }
                _ => {
                    self.theater_s3_order_swapped = false;
                    self.theater_s3_loop_simulated = false;
                }
            }
        } else if self.theater_slide == 3 {
            // Slide 4: Now Playing, Oscilloscope, Seeking, Shuffle, Repeat, Queue Drawer, Gapless & ReplayGain (7 Turns)
            match self.theater_turn {
                0 => {
                    self.theater_s4_queue_open = false;
                }
                1 => {
                    self.theater_s4_queue_open = false;
                }
                2 => {
                    self.theater_s4_queue_open = false;
                }
                3 => {
                    self.theater_s4_queue_open = false;
                    self.theater_s4_shuffled = true;
                }
                4 => {
                    self.theater_s4_queue_open = false;
                }
                5 => {
                    self.theater_s4_queue_open = true;
                }
                _ => {
                    self.theater_s4_queue_open = false;
                }
            }
        } else if self.theater_slide == 4 {
            // Slide 5: Music Library, Scanning, Stats, Categories, Search, Row Actions & Playlists (6 Turns)
            match self.theater_turn {
                0 => {
                    self.theater_s5_category = 0; // AllTracks
                }
                1 => {
                    self.theater_s5_category = 0; // Stats focus
                }
                2 => {
                    self.theater_s5_category = 1; // Albums grid view
                }
                3 => {
                    self.theater_s5_category = 0; // Search & active filter
                }
                4 => {
                    self.theater_s5_category = 0; // Row action controls
                }
                _ => {
                    self.theater_s5_category = 5; // Custom Playlists view
                }
            }
        }

        let area_id = ui.scope_id().with("interactive_theater_overlay");
        egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(&ctx, |ui| {
                let p = ui.painter();

                // 1. Fullscreen Dark Dimmed Frosted Backdrop
                p.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(215));

                // 2. Strict 90% Screen Geometry calculation
                let screen_size = screen_rect.size();
                let theater_w = (screen_size.x * 0.90).clamp(760.0, 1280.0);
                let theater_h = (screen_size.y * 0.90).clamp(520.0, 820.0);
                let theater_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(theater_w, theater_h));

                let mut theater_ui = ui.new_child(egui::UiBuilder::new().max_rect(theater_rect));

                egui::Frame::new()
                    .fill(Color32::from_rgb(11, 15, 26)) // Deep Studio Obsidian Slate
                    .stroke(Stroke::new(1.8, Color32::from_rgb(79, 70, 229))) // Cohesive Royal Indigo Border
                    .corner_radius(16.0)
                    .inner_margin(16.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 16],
                        blur: 48,
                        spread: 0,
                        color: Color32::from_black_alpha(240),
                    })
                    .show(&mut theater_ui, |ui| {
                        let content_w = theater_w - 32.0;
                        ui.set_width(content_w);
                        ui.set_max_height(theater_h - 32.0);

                        // --- 1. HEADER BAR (Clean, Modern Studio Console) ---
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🎬").size(22.0));
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("LUNULA STUDIO THEATER • PANDUAN INTERAKTIF 90%")
                                        .strong()
                                        .size(13.5)
                                        .color(Color32::from_rgb(241, 245, 249)),
                                );
                                ui.label(
                                    egui::RichText::new("Duo Not Balok Kembar \"Lu & Nu\" • Bodi Hitam Outline Putih • Edukasi Audio 60 FPS")
                                        .size(11.0)
                                        .color(Color32::from_rgb(148, 163, 184)),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Modern, eye-friendly dark pill button with mathematically aligned procedural ✕
                                let close_resp = render_studio_button(
                                    ui,
                                    "Lewati & Masuk Studio",
                                    ButtonIcon::Close,
                                    IconPosition::Left,
                                    true,
                                    Color32::from_rgb(20, 26, 40),
                                    Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                    Color32::from_rgb(203, 213, 225),
                                    8.0,
                                );

                                if close_resp.clicked() {
                                    self.show_interactive_theater = false;
                                }
                            });
                        });

                        ui.add_space(8.0);

                        // --- 2. SLIDE NAVIGATION TABS (Slides 1 & 2) ---
                        ui.horizontal(|ui| {
                            let s1_active = self.theater_slide == 0;
                            let s1_btn = egui::Button::new(
                                egui::RichText::new("1. Filosofi Dual-Engine (Bit-Perfect vs DSP)")
                                    .size(11.5)
                                    .strong()
                                    .color(if s1_active { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) }),
                            )
                            .fill(if s1_active { Color32::from_rgb(49, 46, 129) } else { Color32::from_rgb(18, 24, 38) })
                            .stroke(Stroke::new(
                                if s1_active { 1.2 } else { 1.0 },
                                if s1_active { Color32::from_rgb(129, 140, 248) } else { Color32::from_rgb(39, 49, 70) },
                            ))
                            .corner_radius(8.0);

                            if ui.add(s1_btn).clicked() && self.theater_slide != 0 {
                                self.theater_slide = 0;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            }

                            ui.add_space(6.0);

                            let s2_active = self.theater_slide == 1;
                            let s2_btn = egui::Button::new(
                                egui::RichText::new("2. Modular Canvas, Fitur Studio & Arsitektur DAG")
                                    .size(11.5)
                                    .strong()
                                    .color(if s2_active { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) }),
                            )
                            .fill(if s2_active { Color32::from_rgb(49, 46, 129) } else { Color32::from_rgb(18, 24, 38) })
                            .stroke(Stroke::new(
                                if s2_active { 1.2 } else { 1.0 },
                                if s2_active { Color32::from_rgb(129, 140, 248) } else { Color32::from_rgb(39, 49, 70) },
                            ))
                            .corner_radius(8.0);

                            if ui.add(s2_btn).clicked() && self.theater_slide != 1 {
                                self.theater_slide = 1;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            }

                            ui.add_space(6.0);

                            let s3_active = self.theater_slide == 2;
                            let s3_btn = egui::Button::new(
                                egui::RichText::new("3. Filter Studio, Grafik Telemetri & Aturan DAG")
                                    .size(11.5)
                                    .strong()
                                    .color(if s3_active { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) }),
                            )
                            .fill(if s3_active { Color32::from_rgb(49, 46, 129) } else { Color32::from_rgb(18, 24, 38) })
                            .stroke(Stroke::new(
                                if s3_active { 1.2 } else { 1.0 },
                                if s3_active { Color32::from_rgb(129, 140, 248) } else { Color32::from_rgb(39, 49, 70) },
                            ))
                            .corner_radius(8.0);

                            if ui.add(s3_btn).clicked() && self.theater_slide != 2 {
                                self.theater_slide = 2;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            }

                            ui.add_space(6.0);

                            let s4_active = self.theater_slide == 3;
                            let s4_btn = egui::Button::new(
                                egui::RichText::new("4. Now Playing, Osiloskop & Antrean")
                                    .size(11.5)
                                    .strong()
                                    .color(if s4_active { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) }),
                            )
                            .fill(if s4_active { Color32::from_rgb(49, 46, 129) } else { Color32::from_rgb(18, 24, 38) })
                            .stroke(Stroke::new(
                                if s4_active { 1.2 } else { 1.0 },
                                if s4_active { Color32::from_rgb(129, 140, 248) } else { Color32::from_rgb(39, 49, 70) },
                            ))
                            .corner_radius(8.0);

                            if ui.add(s4_btn).clicked() && self.theater_slide != 3 {
                                self.theater_slide = 3;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            }

                            ui.add_space(6.0);

                            let s5_active = self.theater_slide == 4;
                            let s5_btn = egui::Button::new(
                                egui::RichText::new("5. Music Library & Koleksi Lokal")
                                    .size(11.5)
                                    .strong()
                                    .color(if s5_active { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) }),
                            )
                            .fill(if s5_active { Color32::from_rgb(49, 46, 129) } else { Color32::from_rgb(18, 24, 38) })
                            .stroke(Stroke::new(
                                if s5_active { 1.2 } else { 1.0 },
                                if s5_active { Color32::from_rgb(129, 140, 248) } else { Color32::from_rgb(39, 49, 70) },
                            ))
                            .corner_radius(8.0);

                            if ui.add(s5_btn).clicked() && self.theater_slide != 4 {
                                self.theater_slide = 4;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            }
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // --- 3. DYNAMIC STAGE AREA (Guaranteed room for bottom bar!) ---
                        let bottom_bar_reserved = 50.0;
                        let stage_h = (ui.available_height() - bottom_bar_reserved).max(280.0);
                        let (stage_rect, stage_resp) = ui.allocate_exact_size(Vec2::new(content_w, stage_h), egui::Sense::click());
                        let stage_p = ui.painter().with_clip_rect(stage_rect);

                        // Stage floor styling
                        stage_p.rect_filled(stage_rect, 10.0, Color32::from_rgb(8, 12, 22));
                        stage_p.rect_stroke(stage_rect, 10.0, Stroke::new(1.0, Color32::from_rgb(26, 36, 56)), egui::StrokeKind::Outside);

                        let st_c = stage_rect.center();

                        // Get dialogue turns for active slide
                        let turns = Self::get_dialogue_turns(self.theater_slide);
                        let total_turns = turns.len();
                        let turn_idx = self.theater_turn.min(total_turns - 1);
                        let current_turn = &turns[turn_idx];

                        // Typewriter logic: 38 characters per second
                        let speed = 38.0;
                        let full_text = current_turn.text;
                        let total_chars = full_text.chars().count();
                        let visible_count = ((self.theater_turn_timer * speed) as usize).min(total_chars);

                        let visible_text: String = full_text.chars().take(visible_count).collect();
                        let is_typing = visible_count < total_chars;

                        // Auto-advance turn if auto-playing and finished typing
                        if self.theater_is_playing && !is_typing && self.theater_turn_timer > (total_chars as f32 / speed) + 4.5 {
                            if self.theater_turn + 1 < total_turns {
                                self.theater_turn += 1;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 0 {
                                // Auto transition to Slide 2
                                self.theater_slide = 1;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 1 {
                                // Auto transition to Slide 3
                                self.theater_slide = 2;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 2 {
                                // Auto transition to Slide 4
                                self.theater_slide = 3;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 3 {
                                // Auto transition to Slide 5
                                self.theater_slide = 4;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            }
                        }

                        // Clicking stage skips typewriter or advances turn
                        if stage_resp.clicked() {
                            if is_typing {
                                self.theater_turn_timer = 999.0;
                            } else if self.theater_turn + 1 < total_turns {
                                self.theater_turn += 1;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 0 {
                                self.theater_slide = 1;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 1 {
                                self.theater_slide = 2;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 2 {
                                self.theater_slide = 3;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            } else if self.theater_slide == 3 {
                                self.theater_slide = 4;
                                self.theater_turn = 0;
                                self.theater_turn_timer = 0.0;
                            }
                        }

                        // Talking animation state for Lu & Nu
                        let is_lu_talking = is_typing && (current_turn.speaker == Speaker::Lu || current_turn.speaker == Speaker::Both);
                        let is_nu_talking = is_typing && (current_turn.speaker == Speaker::Nu || current_turn.speaker == Speaker::Both);

                        // --- RENDER STAGE DIAGRAM (LOWER HALF OF STAGE) ---
                        let diagram_center = Pos2::new(st_c.x, stage_rect.min.y + 190.0 + (stage_h - 260.0) * 0.45);
                        self.render_slide_diagram(
                            &stage_p,
                            ui,
                            diagram_center,
                            stage_rect,
                            time,
                        );

                        // --- RENDER SPEECH BUBBLE (UPPER HALF OF STAGE) ---
                        let bubble_w = (stage_rect.width() - 220.0).clamp(420.0, 720.0);
                        let bubble_rect = Rect::from_center_size(
                            Pos2::new(st_c.x + 35.0, stage_rect.min.y + 60.0),
                            Vec2::new(bubble_w, 94.0),
                        );

                        // Bubble background with stylish border matching speaker color
                        stage_p.rect_filled(bubble_rect, 10.0, Color32::from_rgba_premultiplied(15, 23, 42, 245));
                        stage_p.rect_stroke(bubble_rect, 10.0, Stroke::new(1.6, current_turn.speaker.color()), egui::StrokeKind::Outside);

                        // Tail pointing to the mascot on the left
                        let tail_p = vec![
                            Pos2::new(bubble_rect.min.x, bubble_rect.min.y + 36.0),
                            Pos2::new(bubble_rect.min.x, bubble_rect.min.y + 54.0),
                            Pos2::new(bubble_rect.min.x - 14.0, bubble_rect.min.y + 45.0),
                        ];
                        stage_p.add(egui::Shape::convex_polygon(
                            tail_p,
                            Color32::from_rgb(15, 23, 42),
                            Stroke::new(1.6, current_turn.speaker.color()),
                        ));

                        // Speaker Badge & Turn Counter
                        stage_p.text(
                            Pos2::new(bubble_rect.min.x + 14.0, bubble_rect.min.y + 16.0),
                            egui::Align2::LEFT_CENTER,
                            current_turn.speaker.badge(),
                            egui::FontId::proportional(12.0),
                            current_turn.speaker.color(),
                        );
                        stage_p.text(
                            Pos2::new(bubble_rect.max.x - 14.0, bubble_rect.min.y + 16.0),
                            egui::Align2::RIGHT_CENTER,
                            format!("Bagian {} / {}", turn_idx + 1, total_turns),
                            egui::FontId::proportional(10.5),
                            Color32::from_rgb(148, 163, 184),
                        );

                        // Wrapped dialogue text with blinking typing cursor
                        let display_str = if is_typing {
                            format!("{}▌", visible_text)
                        } else {
                            visible_text
                        };

                        let text_wrap_w = bubble_w - 28.0;
                        let galley = stage_p.layout(
                            display_str,
                            egui::FontId::proportional(12.0),
                            Color32::from_rgb(241, 245, 249),
                            text_wrap_w,
                        );
                        stage_p.galley(Pos2::new(bubble_rect.min.x + 14.0, bubble_rect.min.y + 30.0), galley, Color32::from_rgb(241, 245, 249));

                        // Helpful footer cue inside bubble
                        let hint_text = if is_typing {
                            "⚡ [Klik untuk tampilkan semua teks]"
                        } else if turn_idx + 1 < total_turns {
                            "💬 [Klik untuk lanjut percakapan ▶]"
                        } else if self.theater_slide == 0 {
                            "✨ [Klik untuk lanjut ke Slide 2 ▶]"
                        } else if self.theater_slide == 1 {
                            "✨ [Klik untuk lanjut ke Slide 3 ▶]"
                        } else if self.theater_slide == 2 {
                            "✨ [Klik untuk lanjut ke Slide 4 ▶]"
                        } else if self.theater_slide == 3 {
                            "✨ [Klik untuk lanjut ke Slide 5 ▶]"
                        } else {
                            "🚀 [Klik untuk masuk ke Studio]"
                        };
                        stage_p.text(
                            Pos2::new(bubble_rect.max.x - 14.0, bubble_rect.max.y - 12.0),
                            egui::Align2::RIGHT_CENTER,
                            hint_text,
                            egui::FontId::proportional(10.0),
                            Color32::from_rgb(100, 116, 139),
                        );

                        // --- RENDER FLYING MASCOT (CLEAR AIRSPACE, NO BLOCKING) ---
                        let mascot_x = bubble_rect.min.x - 70.0 + (t * 2.0).sin() * 8.0;
                        let mascot_y = bubble_rect.min.y + 45.0 + (t * 2.5).cos() * 5.0;
                        let mascot_pos = Pos2::new(mascot_x.max(stage_rect.min.x + 55.0), mascot_y);

                        // Draw procedural beamed pair with animated talking mouth & NO background circle!
                        draw_beamed_pair_mascot_ex(
                            &stage_p,
                            mascot_pos,
                            1.05,
                            time,
                            current_turn.expression,
                            is_lu_talking,
                            is_nu_talking,
                        );

                        ui.ctx().request_repaint();

                        ui.add_space(8.0);

                        // --- 4. BOTTOM CONTROL BAR (Cohesive, Modern Studio Styling) ---
                        ui.horizontal(|ui| {
                            // Auto-play Toggle Button (Procedural icon centered with text)
                            let (play_text, play_icon) = if self.theater_is_playing {
                                ("Jeda Otomatis", ButtonIcon::Pause)
                            } else {
                                ("Putar Otomatis", ButtonIcon::Play)
                            };
                            let play_resp = render_studio_button(
                                ui,
                                play_text,
                                play_icon,
                                IconPosition::Left,
                                true,
                                Color32::from_rgb(20, 26, 40),
                                Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                Color32::from_rgb(203, 213, 225),
                                8.0,
                            );

                            if play_resp.clicked() {
                                self.theater_is_playing = !self.theater_is_playing;
                            }

                            ui.add_space(8.0);

                            // Turn Step Button: Previous Turn (Procedural ◀ centered with text)
                            let prev_resp = render_studio_button(
                                ui,
                                "Balas Sebelumnya",
                                ButtonIcon::Prev,
                                IconPosition::Left,
                                self.theater_turn > 0,
                                Color32::from_rgb(20, 26, 40),
                                Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                Color32::from_rgb(203, 213, 225),
                                8.0,
                            );

                            if prev_resp.clicked() && self.theater_turn > 0 {
                                self.theater_turn -= 1;
                                self.theater_turn_timer = 0.0;
                            }

                            ui.add_space(8.0);

                            // Turn Step Button: Next Turn (Modern Dark Studio Slate + Procedural ▶ centered with text)
                            let next_resp = render_studio_button(
                                ui,
                                "Lanjut Dialog",
                                ButtonIcon::Play,
                                IconPosition::Right,
                                self.theater_turn + 1 < total_turns,
                                Color32::from_rgb(32, 41, 68),
                                Stroke::new(1.2, Color32::from_rgb(99, 102, 241)),
                                Color32::from_rgb(224, 231, 255),
                                8.0,
                            );

                            if next_resp.clicked() && self.theater_turn + 1 < total_turns {
                                self.theater_turn += 1;
                                self.theater_turn_timer = 0.0;
                            }

                            // Right-aligned Actions: Next Slide or Enter Studio
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if self.theater_slide == 0 {
                                    let s2_resp = render_studio_button(
                                        ui,
                                        "Lanjut ke Slide 2 (Canvas DAG)",
                                        ButtonIcon::Play,
                                        IconPosition::Right,
                                        true,
                                        Color32::from_rgb(22, 48, 40),
                                        Stroke::new(1.2, Color32::from_rgb(52, 211, 153)),
                                        Color32::from_rgb(220, 252, 231),
                                        8.0,
                                    );

                                    if s2_resp.clicked() {
                                        self.theater_slide = 1;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }
                                } else if self.theater_slide == 1 {
                                    let s3_resp = render_studio_button(
                                        ui,
                                        "Lanjut ke Slide 3 (Filter & DAG)",
                                        ButtonIcon::Play,
                                        IconPosition::Right,
                                        true,
                                        Color32::from_rgb(22, 48, 40),
                                        Stroke::new(1.2, Color32::from_rgb(52, 211, 153)),
                                        Color32::from_rgb(220, 252, 231),
                                        8.0,
                                    );

                                    if s3_resp.clicked() {
                                        self.theater_slide = 2;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }

                                    ui.add_space(8.0);

                                    let back_s1_resp = render_studio_button(
                                        ui,
                                        "Kembali ke Slide 1",
                                        ButtonIcon::Prev,
                                        IconPosition::Left,
                                        true,
                                        Color32::from_rgb(20, 26, 40),
                                        Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                        Color32::from_rgb(203, 213, 225),
                                        8.0,
                                    );

                                    if back_s1_resp.clicked() {
                                        self.theater_slide = 0;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }
                                } else if self.theater_slide == 2 {
                                    let s4_resp = render_studio_button(
                                        ui,
                                        "Lanjut ke Slide 4 (Now Playing & Antrean)",
                                        ButtonIcon::Play,
                                        IconPosition::Right,
                                        true,
                                        Color32::from_rgb(22, 48, 40),
                                        Stroke::new(1.2, Color32::from_rgb(52, 211, 153)),
                                        Color32::from_rgb(220, 252, 231),
                                        8.0,
                                    );

                                    if s4_resp.clicked() {
                                        self.theater_slide = 3;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }

                                    ui.add_space(8.0);

                                    let back_s2_resp = render_studio_button(
                                        ui,
                                        "Kembali ke Slide 2",
                                        ButtonIcon::Prev,
                                        IconPosition::Left,
                                        true,
                                        Color32::from_rgb(20, 26, 40),
                                        Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                        Color32::from_rgb(203, 213, 225),
                                        8.0,
                                    );

                                    if back_s2_resp.clicked() {
                                        self.theater_slide = 1;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }
                                } else if self.theater_slide == 3 {
                                    let s5_resp = render_studio_button(
                                        ui,
                                        "Lanjut ke Slide 5 (Music Library)",
                                        ButtonIcon::Play,
                                        IconPosition::Right,
                                        true,
                                        Color32::from_rgb(22, 48, 40),
                                        Stroke::new(1.2, Color32::from_rgb(52, 211, 153)),
                                        Color32::from_rgb(220, 252, 231),
                                        8.0,
                                    );

                                    if s5_resp.clicked() {
                                        self.theater_slide = 4;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }

                                    ui.add_space(8.0);

                                    let back_s3_resp = render_studio_button(
                                        ui,
                                        "Kembali ke Slide 3",
                                        ButtonIcon::Prev,
                                        IconPosition::Left,
                                        true,
                                        Color32::from_rgb(20, 26, 40),
                                        Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                        Color32::from_rgb(203, 213, 225),
                                        8.0,
                                    );

                                    if back_s3_resp.clicked() {
                                        self.theater_slide = 2;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }
                                } else {
                                    let studio_resp = render_studio_button(
                                        ui,
                                        "Selesai & Buka Signal Chain Studio",
                                        ButtonIcon::Rocket,
                                        IconPosition::Left,
                                        true,
                                        Color32::from_rgb(22, 48, 40),
                                        Stroke::new(1.2, Color32::from_rgb(52, 211, 153)),
                                        Color32::from_rgb(220, 252, 231),
                                        8.0,
                                    );

                                    if studio_resp.clicked() {
                                        self.show_interactive_theater = false;
                                    }

                                    ui.add_space(8.0);

                                    let back_s4_resp = render_studio_button(
                                        ui,
                                        "Kembali ke Slide 4",
                                        ButtonIcon::Prev,
                                        IconPosition::Left,
                                        true,
                                        Color32::from_rgb(20, 26, 40),
                                        Stroke::new(1.0, Color32::from_rgb(45, 55, 78)),
                                        Color32::from_rgb(203, 213, 225),
                                        8.0,
                                    );

                                    if back_s4_resp.clicked() {
                                        self.theater_slide = 3;
                                        self.theater_turn = 0;
                                        self.theater_turn_timer = 0.0;
                                    }
                                }
                            });
                        });
                    });
            });
    }

    /// Returns the rich, chatty banter dialogue turns for each slide
    fn get_dialogue_turns(slide: usize) -> Vec<DialogueTurn> {
        if slide == 0 {
            // Slide 1: Dual-Engine Philosophy (Bit-Perfect vs DSP Active)
            vec![
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Halo audiophile! Aku Lu. Di Lunula Player, kami punya prinsip mutlak: KEJUJURAN AUDIO. Banyak software mengklaim 'audiophile' tapi memanipulasi sinyal diam-diam. Di Lunula, jalur emas di atas adalah mode BIT-PERFECT DIRECT 1:1!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Di mode Bit-Perfect, sampel lagu dialirkan bit-demi-bit persis seperti master rekaman asli tanpa interpolasi, tanpa re-sampling, dan tanpa sentuhan 1 bit pun ke DAC! Jika DAC tidak mendukung sample rate asli, sistem otomatis mengalihkan ke OS Resampling dengan aman.",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Eits, tunggu dulu Lu! Kenalkan, aku Nu! Bagaimana kalau kamu ingin karakter vokal yang tebal dan empuk seperti mendengarkan piringan hitam vintage? Perhatikan kabelnya: sekarang sinyal dialihkan ke DSP ENGINE!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Lihat kabelnya sekarang: Dari FILE SOURCE, sinyal mengalir masuk ke DSP ENGINE lalu diteruskan ke DAC! Mesin mastering 64-bit float, tabung 12AX7, dan 10-Band EQ aktif memproses audio dengan headroom 300+ dB tanpa clipping!",
                    expression: MascotExpression::BeatDance,
                },
                DialogueTurn {
                    speaker: Speaker::Both,
                    text: "Kamu bebas memilih kapanpun: kemurnian bitstream 1:1 langsung ke DAC, atau kehangatan racikan studio analog DSP. Coba klik tombol toggle di bawah ini untuk menguji perbedaan jalurnya!",
                    expression: MascotExpression::WaveHello,
                },
            ]
        } else if slide == 1 {
            // Slide 2: Modular Canvas, Fitur Studio & Arsitektur DAG (Summing, Splitting, Dry & Wet)
            vec![
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Selamat datang di jantung Lunula: Signal Chain Studio! Ini bukan sekadar equalizer biasa, melainkan arsitektur audio modular berbasis DAG (Directed Acyclic Graph) standar studio rekaman profesional!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Aliran sinyal mengalir terarah dari kiri ke kanan: Dimulai dari FILE SOURCE, lalu dimuat 100% ke RAM BUFFER agar bebas jitter disk. Dari sana sinyal mengalir mulus menuju DAC tanpa latency tersembunyi.",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Nah, sekarang perhatikan keajaiban DAG! Ayo kita putus kabel tunggal ini ✂️, lalu pasang 3 modul studio baru: SPLITTER (pencabang), LOW-PASS FILTER (LPF), dan SUMMING MIXER (penggabung)!",
                    expression: MascotExpression::ShockedShield,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Lihat percabangannya: Jalur atas adalah DRY SOUND (sinyal audio asli murni tanpa filter untuk menjaga kejernihan transien). Sedangkan jalur bawah dialirkan ke LOW-PASS FILTER menjadi WET SOUND (sinyal yang disaring agar hangat dan tebal)!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Lalu apa yang terjadi di SUMMING MIXER? Kedua sinyal (Dry + Wet) dijumlahkan secara presisi dengan kompensasi fasa otomatis! Kamu mendapatkan kehangatan filter sekaligus kejernihan detail vokal asli tanpa phase cancellation!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Both,
                    text: "Inilah teknik Parallel Studio Processing murni! Canvas adalah Source of Truth: apa yang tersambung kabel, itulah yang bersuara. Klik kanan tahan untuk geser (Pan), tarik pin untuk meracik kabelmu sendiri!",
                    expression: MascotExpression::WaveHello,
                },
            ]
        } else if slide == 2 {
            // Slide 3: Filter Studio, Grafik Telemetri & Aturan Sakral DAG (8 Bagian Mendalam)
            vec![
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Selamat datang di laboratorium audio Lunula! Di rak modul kanvas studio, kamu memiliki segudang pemroses suara: 10-Band EQ, Tube Warmer 12AX7, Low-Pass Filter, hingga Stereo Widener. Semuanya adalah modul DSP mandiri berpresisi 64-bit float!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Dan perhatikan layar di bawah: ini adalah GRAFIK TELEMETRI REAL-TIME! Pada modul 10-BAND EQ, telemetri menampilkan Bode Plot kurva respons frekuensi (31 Hz - 16 kHz). Kamu bisa melihat pergeseran desibel dan kurva Q-factor secara langsung tanpa mengira-ngira!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Tengok modul favoritku: TABUNG 12AX7! Di grafik telemetrinya, kamu bisa memantau spektrum FFT distorsi harmonik analog (2nd & 3rd harmonics). Inilah rahasia kenapa vokal terdengar begitu empuk, intim, dan bertekstur piringan hitam klasik!",
                    expression: MascotExpression::CozyWarmth,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Sedangkan pada LOW-PASS FILTER (LPF), telemetri menampilkan kurva pemotongan tajam -12 dB/oktaf. Kamu bisa melihat frekuensi tinggi di atas cutoff disaring agar treble yang kasar dan menusuk telinga diredam dengan fase linear!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Sekarang pahami sifat sakral pertama DAG: DIRECTED (Terarah). Aliran sinyal Lunula mengalir mutlak satu arah dari kiri ke kanan. Tidak ada delay tersembunyi karena setiap sampel dieksekusi secara terurut dalam topologi linier!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Rahasia besar audio: POSISI FILTER SANGAT MEMPENGARUHI SUARA! Coba klik 'Tukar Urutan'. Jika [Tube -> EQ], saturasi tabung disuntikkan duluan lalu diukir EQ. Tapi jika [EQ -> Tube] dengan bass di-boost +6dB duluan, tabung akan terdorong saturasi tebal pada bass!",
                    expression: MascotExpression::BeatDance,
                },
                DialogueTurn {
                    speaker: Speaker::Both,
                    text: "HUKUM SAKRAL DAG: ACYCLIC (DILARANG LOOPING)! Huruf 'A' di DAG berarti bebas siklus. Kamu tidak bisa menyambung output modul kembali ke inputnya sendiri. Karena loop tanpa henti akan melipatgandakan gain ke +∞ dB dan merusak speaker! Lunula seketika memblokir kabel loop!",
                    expression: MascotExpression::ShockedShield,
                },
                DialogueTurn {
                    speaker: Speaker::Both,
                    text: "Meski anti-looping, kamu punya kebebasan penuh menyambung dua, tiga, atau lebih filter berurutan secara kaskade seri (A -> B -> C -> DAC)! Semua berjalan di headroom 300+ dB tanpa distorsi liar. Silakan coba eksplorasi dan racik sendiri di kanvas studio!",
                    expression: MascotExpression::WaveHello,
                },
            ]
        } else if slide == 3 {
            // Slide 4: Now Playing, Osiloskop, Aturan Detik, Shuffle, Repeat & Drawer Antrean (6 Bagian)
            vec![
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Lihat kanvas gelombang suara di tengah ini! Ini bukan sekadar grafik volume biasa, melainkan 3-Band Spectral Waveform! Warna memetakan frekuensi audio secara langsung: Pink Magenta untuk Bass (<250Hz), Cyan untuk Mid (250Hz–4kHz), dan Emas untuk Treble (>4kHz). Kamu bisa perbesar gelombang dengan Ctrl + Scroll!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Perhatikan tombol Play di bawah! Saat musik diputar, ikon segitiga bermutasi secara mulus menjadi Berkas Osiloskop Hidup (Living Oscilloscope Beam) yang bergetar mengikuti sampel sinyal audio 64-bit float! Dan di bawahnya, 48-Band FFT Spectrum Analyzer memetakan spektrum 20Hz-20kHz pada 60 FPS!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Ingin mengulang reff atau melompat intro? Gunakan navigasi waktu presisi kami: Tekan Panah Kanan (→) untuk maju tepat +5 Detik, dan Panah Kiri (←) untuk mundur -5 Detik! Atau tekan Ctrl + Panah untuk ganti trek seketika tanpa menyentuh mouse!",
                    expression: MascotExpression::WaveHello,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Bosan dengan urutan lagu monoton? Lunula punya 2 senjata pengacakan: Tombol toggle Shuffle (🔀) yang memilih trek berikutnya secara random, dan tombol [🎲 Acak Antrean] di panel drawer yang mengocok fisik susunan antrean menggunakan algoritma Fisher-Yates!",
                    expression: MascotExpression::BeatDance,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Sistem Repeat kami punya 4 logo suci: [✕ Mati] berhenti saat antrean habis, [🔁 Semua] memutar ulang seluruh playlist tanpa batas, [🔂 Lagu] mengulang 1 lagu ini saja tanpa henti, dan [💿 Album Ini] filter cerdas yang hanya memutar ulang lagu-lagu di album yang sama!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Klik tombol '📑 Antrean' di bilah kontrol atau atas untuk membuka Drawer Antrean Putar! Kamu bisa melihat lagu Sedang Diputar, menukar urutan lagu dengan tombol ▲ ▼ atau Ctrl+↑/↓, memutar langsung dengan ▶, atau membersihkan antrean dengan [🗑 Kosongkan]!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Both,
                    text: "Inilah 4 pilar kenyamanan audio modern Lunula: ⚡ Gapless Playback (0 ms jeda hening zero-flush sample-accurate), 🔊 ReplayGain (Auto Track/Album loudness matching dengan Peak Scaling & Limiter anti-clipping), 🔀 Crossfade Transition (transisi tumpang-tindih mulus 0-12s), dan 🎛️ Studio Dynamic Compressor yang menjaga kenyaringan tetap stabil dan merata!",
                    expression: MascotExpression::Audiophile,
                },
            ]
        } else {
            // Slide 5: Music Library, Scanning, Stats, Kategori, Search, Row Actions & Playlists (6 Bagian)
            vec![
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Di era di mana layanan streaming mengompresi audio dan melacak privasimu, Lunula Player berpegang teguh pada prinsip Kejujuran & Kepemilikan Fisik! Perpustakaan musik Lunula 100% bekerja secara lokal di komputermu tanpa internet. Cukup klik [Scan Folder Musik] atau [Tambah File], pemindai multi-thread kami akan membaca file FLAC, WAV, ALAC, hingga MP3 dengan integritas bitstream murni!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Tengok pita telemetri 5 kartu di atas! Lunula langsung menghitung data statistik koleksimu secara instan: Total Lagu aktif, jumlah Album diskografi, Artis penyanyi/komposer, dan akumulasi ukuran penyimpanan (MB/GB). Yang paling istimewa adalah kartu Hi-Res Master: sistem mendeteksi otomatis seluruh trek dengan resolusi audiophile ≥24-bit atau sample rate ≥88.2kHz!",
                    expression: MascotExpression::Happy,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Jelajahi musikmu lewat 6 kategori cerdas! Khusus di tabel Semua Lagu: Sekali klik langsung memutar lagu seketika (baris berubah hijau aktif dengan tombol ▶)! Perhatikan juga tanda Love Kecil (💜/❤️) di kolom FAV tepat di samping cover album: klik langsung untuk menyimpan lagu favoritmu!",
                    expression: MascotExpression::Audiophile,
                },
                DialogueTurn {
                    speaker: Speaker::Nu,
                    text: "Mencari lagu di antara ribuan koleksi? Ketikkan sepenggal kata di kolom pencarian kanan atas: pencarian menyaring seketika saat kamu mengetik (real-time filtering)! Selain itu, jika kamu mengklik salah satu kartu Album atau Artis, perpustakaan akan mengunci tampilan pada karya tersebut dan memunculkan badge Filter Aktif. Klik [✕ Reset Filter] untuk kembali!",
                    expression: MascotExpression::BeatDance,
                },
                DialogueTurn {
                    speaker: Speaker::Lu,
                    text: "Inilah rahasia kendali lengkapnya! Untuk mengakses tombol aksi studio, kamu cukup KLIK KANAN pada baris lagu. Jendela Menu Konteks melayang seketika muncul dengan 7 opsi studio: Putar Sekarang, Putar Berikutnya (Play Next), Tambahkan ke Antrean, Tambah ke Favorit, Tambahkan ke Playlist, Hapus dari Library, dan Buka di Explorer!",
                    expression: MascotExpression::WaveHello,
                },
                DialogueTurn {
                    speaker: Speaker::Both,
                    text: "Ingin membuat mixtape bertema malam, vokal akustik, atau mastering bass berat? Di tab Playlist & Album Buatan, kamu bisa membuat playlist kustom sesukamu! Tambahkan lagu dari tabel manapun dan putar seluruh playlist sekaligus (bulk queue). Dan tenang saja, tombol [🗑️ Kosongkan Library] hanya membersihkan indeks aplikasi tanpa pernah menghapus file di hard disk!",
                    expression: MascotExpression::WaveHello,
                },
            ]
        }
    }

    /// Renders the diagrams for Slide 1 or Slide 2 in the lower half of the stage
    fn render_slide_diagram(
        &mut self,
        painter: &egui::Painter,
        ui: &mut egui::Ui,
        center: Pos2,
        _stage_rect: Rect,
        time: f64,
    ) {
        let t = time as f32;

        // --- UNIFORM CABLE ANIMATION HELPER ---
        // Moves pulses at a strictly constant physical velocity (110.0 px/s) along any polyline path.
        let draw_flowing_cable = |pts: &[Pos2], stroke_col: Color32, pulse_col: Color32, pulse_count: usize| {
            if pts.len() < 2 {
                return;
            }
            // 1. Draw solid cable line segments
            let stroke = Stroke::new(2.4, stroke_col);
            for pair in pts.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }

            // 2. Compute exact polyline length and segment offsets
            let mut total_len = 0.0;
            let mut lengths = Vec::with_capacity(pts.len() - 1);
            for pair in pts.windows(2) {
                let l = (pair[1] - pair[0]).length();
                lengths.push(l);
                total_len += l;
            }
            if total_len <= 1.0 {
                return;
            }

            // 3. Strictly uniform physical velocity: 110.0 px/sec across every wire segment
            let speed_px = 110.0;
            let cycle_time = total_len / speed_px;
            let base_phase = (t / cycle_time) % 1.0;

            for i in 0..pulse_count {
                let pulse_phase = (base_phase + (i as f32) / (pulse_count as f32)) % 1.0;
                let mut rem_dist = pulse_phase * total_len;
                let mut dot_pos = pts[0];
                for (idx, pair) in pts.windows(2).enumerate() {
                    let seg_len = lengths[idx];
                    if rem_dist <= seg_len {
                        let frac = if seg_len > 0.0 { rem_dist / seg_len } else { 0.0 };
                        dot_pos = Pos2::new(
                            pair[0].x + (pair[1].x - pair[0].x) * frac,
                            pair[0].y + (pair[1].y - pair[0].y) * frac,
                        );
                        break;
                    } else {
                        rem_dist -= seg_len;
                        dot_pos = pair[1];
                    }
                }
                painter.circle_filled(dot_pos, 3.5, pulse_col);
            }
        };

        if self.theater_slide == 0 {
            // --- SLIDE 1 DIAGRAM: DUAL-ENGINE (BIT-PERFECT vs DSP ACTIVE) ---
            let src_pos = Pos2::new(center.x - 220.0, center.y);
            let dac_pos = Pos2::new(center.x + 220.0, center.y);
            let dsp_node_pos = Pos2::new(center.x, center.y + 60.0);

            let bp_active = !self.theater_dsp_active;

            // Card drawing helper
            let draw_card = |pos: Pos2, title: &str, sub: &str, is_highlighted: bool, accent: Color32| {
                let r = Rect::from_center_size(pos, Vec2::new(125.0, 48.0));
                let bg = if is_highlighted { Color32::from_rgb(18, 25, 42) } else { Color32::from_rgb(14, 18, 28) };
                painter.rect_filled(r, 8.0, bg);
                painter.rect_stroke(
                    r,
                    8.0,
                    Stroke::new(if is_highlighted { 1.5 } else { 1.0 }, if is_highlighted { accent } else { Color32::from_rgb(39, 49, 70) }),
                    egui::StrokeKind::Outside,
                );
                painter.text(Pos2::new(pos.x, pos.y - 7.0), egui::Align2::CENTER_CENTER, title, egui::FontId::proportional(11.5), if is_highlighted { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) });
                painter.text(Pos2::new(pos.x, pos.y + 10.0), egui::Align2::CENTER_CENTER, sub, egui::FontId::proportional(9.5), Color32::from_rgb(100, 116, 139));
            };

            draw_card(src_pos, "📁 FILE SOURCE", "FLAC 96kHz / 24-bit", true, Color32::from_rgb(56, 189, 248));
            draw_card(dac_pos, "🔊 OUTPUT DAC", "ASIO / Exclusive", true, Color32::from_rgb(168, 85, 247));

            // --- 1. Top Cable: Bit-Perfect Direct Wire ---
            let bp_p1 = Pos2::new(src_pos.x + 62.5, src_pos.y - 10.0);
            let bp_mid = Pos2::new(center.x, center.y - 65.0);
            let bp_p2 = Pos2::new(dac_pos.x - 62.5, dac_pos.y - 10.0);

            if bp_active {
                // Active golden wire with uniform flowing pulses
                draw_flowing_cable(&[bp_p1, bp_mid, bp_p2], Color32::from_rgb(245, 158, 11), Color32::WHITE, 3);

                let bp_badge_rect = Rect::from_center_size(bp_mid, Vec2::new(215.0, 20.0));
                painter.rect_filled(bp_badge_rect, 4.0, Color32::from_rgb(254, 243, 199));
                painter.text(
                    bp_badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✨ 1:1 DIRECT BITSTREAM (BIT-PERFECT)",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(180, 83, 9),
                );
            } else {
                // Dimmed bypassed direct wire
                let bp_stroke = Stroke::new(1.2, Color32::from_rgb(39, 49, 70));
                painter.line_segment([bp_p1, bp_mid], bp_stroke);
                painter.line_segment([bp_mid, bp_p2], bp_stroke);

                let bp_badge_rect = Rect::from_center_size(bp_mid, Vec2::new(170.0, 18.0));
                painter.rect_filled(bp_badge_rect, 4.0, Color32::from_rgb(17, 24, 39));
                painter.text(
                    bp_badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "DIRECT WIRE (DILEWATI / BYPASS)",
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(100, 116, 139),
                );
            }

            // --- 2. Bottom Routing: DSP Active Routing through DSP Engine to DAC ---
            let dsp_in_start = Pos2::new(src_pos.x + 62.5, src_pos.y + 10.0);
            let dsp_in_end = Pos2::new(dsp_node_pos.x - 65.0, dsp_node_pos.y);
            let dsp_out_start = Pos2::new(dsp_node_pos.x + 65.0, dsp_node_pos.y);
            let dsp_out_end = Pos2::new(dac_pos.x - 62.5, dac_pos.y + 10.0);

            if self.theater_dsp_active {
                // Active vibrant cyan routing with uniform flowing pulses
                draw_flowing_cable(&[dsp_in_start, dsp_in_end], Color32::from_rgb(56, 189, 248), Color32::from_rgb(52, 211, 153), 2);
                draw_flowing_cable(&[dsp_out_start, dsp_out_end], Color32::from_rgb(56, 189, 248), Color32::from_rgb(52, 211, 153), 2);

                // Highlighted active DSP Engine card
                draw_card(dsp_node_pos, "🎛️ DSP 64-BIT ENGINE", "10-Band EQ + Tabung 12AX7", true, Color32::from_rgb(56, 189, 248));

                // Active badge
                let dsp_badge_rect = Rect::from_center_size(Pos2::new(center.x, dsp_node_pos.y + 36.0), Vec2::new(210.0, 18.0));
                painter.rect_filled(dsp_badge_rect, 4.0, Color32::from_rgb(19, 42, 60));
                painter.text(
                    dsp_badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎛️ JALUR MASTERING DSP AKTIF KE DAC",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(56, 189, 248),
                );
            } else {
                // Inactive DSP Routing (Dimmed)
                let dsp_stroke = Stroke::new(1.0, Color32::from_rgb(30, 38, 55));
                painter.line_segment([dsp_in_start, dsp_in_end], dsp_stroke);
                painter.line_segment([dsp_out_start, dsp_out_end], dsp_stroke);

                draw_card(dsp_node_pos, "🎛️ DSP ENGINE", "Dilewati (0% CPU Load)", false, Color32::from_rgb(51, 65, 88));
            }

            // --- Center Interactive Mode Selector Button ---
            let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 2.0), Vec2::new(220.0, 32.0));
            let btn_hov = ui.rect_contains_pointer(btn_rect);
            let btn_bg = if self.theater_dsp_active {
                Color32::from_rgb(18, 38, 60)
            } else {
                Color32::from_rgb(45, 36, 20)
            };
            let btn_stroke_col = if self.theater_dsp_active {
                Color32::from_rgb(56, 189, 248)
            } else {
                Color32::from_rgb(245, 158, 11)
            };

            painter.rect_filled(btn_rect, 8.0, btn_bg);
            painter.rect_stroke(btn_rect, 8.0, Stroke::new(if btn_hov { 1.6 } else { 1.0 }, btn_stroke_col), egui::StrokeKind::Outside);

            let (btn_lbl, btn_dot_col) = if self.theater_dsp_active {
                ("● Jalur Aktif: [DSP ACTIVE]", Color32::from_rgb(56, 189, 248))
            } else {
                ("● Jalur Aktif: [BIT-PERFECT]", Color32::from_rgb(245, 158, 11))
            };
            painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, btn_lbl, egui::FontId::proportional(11.5), btn_dot_col);

            if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                self.theater_dsp_active = !self.theater_dsp_active;
            }
        } else if self.theater_slide == 1 {
            // --- SLIDE 2 DIAGRAM: MODULAR CANVAS, SPLITTING, LPF & SUMMING (DRY-WET PARALLEL) ---
            let card_src_pos = Pos2::new(center.x - 270.0, center.y);
            let card_buf_pos = Pos2::new(center.x - 160.0, center.y);
            let card_dac_pos = Pos2::new(center.x + 285.0, center.y);

            // Card drawing helper
            let draw_compact_card = |pos: Pos2, title: &str, sub: &str, w: f32, is_accent: bool, accent_col: Color32| {
                let r = Rect::from_center_size(pos, Vec2::new(w, 44.0));
                let bg = if is_accent { Color32::from_rgb(18, 25, 42) } else { Color32::from_rgb(14, 18, 28) };
                painter.rect_filled(r, 6.0, bg);
                painter.rect_stroke(
                    r,
                    6.0,
                    Stroke::new(if is_accent { 1.5 } else { 1.0 }, if is_accent { accent_col } else { Color32::from_rgb(45, 55, 78) }),
                    egui::StrokeKind::Outside,
                );
                painter.text(Pos2::new(pos.x, pos.y - 7.0), egui::Align2::CENTER_CENTER, title, egui::FontId::proportional(10.5), if is_accent { Color32::WHITE } else { Color32::from_rgb(160, 175, 200) });
                painter.text(Pos2::new(pos.x, pos.y + 9.0), egui::Align2::CENTER_CENTER, sub, egui::FontId::proportional(9.0), Color32::from_rgb(100, 116, 139));
            };

            draw_compact_card(card_src_pos, "FILE SOURCE", "FLAC 24-bit", 90.0, true, Color32::from_rgb(56, 189, 248));
            draw_compact_card(card_buf_pos, "RAM BUFFER", "100% Loaded", 85.0, true, Color32::from_rgb(56, 189, 248));
            draw_compact_card(card_dac_pos, "OUTPUT DAC", "32-bit Float", 90.0, true, Color32::from_rgb(168, 85, 247));

            // Cable 1: Source to RAM Buffer (always connected with uniform pulse velocity)
            let p_src_out = Pos2::new(card_src_pos.x + 45.0, card_src_pos.y);
            let p_buf_in = Pos2::new(card_buf_pos.x - 42.5, card_buf_pos.y);
            draw_flowing_cable(&[p_src_out, p_buf_in], Color32::from_rgb(56, 189, 248), Color32::WHITE, 1);

            let p_buf_out = Pos2::new(card_buf_pos.x + 42.5, card_buf_pos.y);

            if !self.theater_split_sum_active {
                // --- Initial Linear Chain (Turns 0 & 1) ---
                let card_dsp_pos = Pos2::new(center.x + 60.0, center.y);
                draw_compact_card(card_dsp_pos, "DSP ENGINE", "64-bit Router", 110.0, true, Color32::from_rgb(99, 102, 241));

                let p_dsp_in = Pos2::new(card_dsp_pos.x - 55.0, card_dsp_pos.y);
                let p_dsp_out = Pos2::new(card_dsp_pos.x + 55.0, card_dsp_pos.y);
                let p_dac_in = Pos2::new(card_dac_pos.x - 45.0, card_dac_pos.y);

                draw_flowing_cable(&[p_buf_out, p_dsp_in], Color32::from_rgb(56, 189, 248), Color32::WHITE, 1);
                draw_flowing_cable(&[p_dsp_out, p_dac_in], Color32::from_rgb(56, 189, 248), Color32::WHITE, 1);
            } else {
                // --- Full Dynamic Parallel DAG Network (Turns 2, 3, 4, 5): Splitter + LPF Filter + Summing Mixer ---
                let card_split_pos = Pos2::new(center.x - 50.0, center.y);
                let card_lpf_pos = Pos2::new(center.x + 60.0, center.y + 36.0);
                let card_mixer_pos = Pos2::new(center.x + 170.0, center.y);

                // 1. SPLITTER Module Card (clean text without broken arrow glyphs)
                draw_compact_card(card_split_pos, "SPLITTER 1:2", "Pencabang Sinyal", 85.0, true, Color32::from_rgb(56, 189, 248));

                // 2. LOW-PASS FILTER (LPF) Module Card (Wet Path)
                draw_compact_card(card_lpf_pos, "LOW-PASS (LPF)", "Penyaring Treble", 95.0, true, Color32::from_rgb(251, 146, 60));

                // 3. SUMMING MIXER Module Card
                draw_compact_card(card_mixer_pos, "SUMMING MIXER", "Phase Aligned", 90.0, true, Color32::from_rgb(168, 85, 247));

                // Cable: Buffer -> Splitter (with uniform flowing pulse!)
                let p_split_in = Pos2::new(card_split_pos.x - 42.5, card_split_pos.y);
                draw_flowing_cable(&[p_buf_out, p_split_in], Color32::from_rgb(56, 189, 248), Color32::WHITE, 1);

                let p_split_out_top = Pos2::new(card_split_pos.x + 42.5, card_split_pos.y - 10.0);
                let p_split_out_bot = Pos2::new(card_split_pos.x + 42.5, card_split_pos.y + 10.0);

                let p_mixer_in_top = Pos2::new(card_mixer_pos.x - 45.0, card_mixer_pos.y - 10.0);
                let p_mixer_in_bot = Pos2::new(card_mixer_pos.x - 45.0, card_mixer_pos.y + 10.0);

                // --- PATH A: DRY SOUND (Direct clean bypass to Summing Mixer) ---
                let dry_wire_y = center.y - 32.0;
                let dry_corner1 = Pos2::new(card_split_pos.x + 55.0, dry_wire_y);
                let dry_corner2 = Pos2::new(card_mixer_pos.x - 55.0, dry_wire_y);
                draw_flowing_cable(
                    &[p_split_out_top, dry_corner1, dry_corner2, p_mixer_in_top],
                    Color32::from_rgb(56, 189, 248),
                    Color32::from_rgb(56, 189, 248),
                    2,
                );

                // Dry Badge: Positioned cleanly ABOVE the wire with 9px vertical clearance (no collision!)
                let dry_badge_pos = Pos2::new((dry_corner1.x + dry_corner2.x) * 0.5, dry_wire_y - 18.0);
                let dry_badge = Rect::from_center_size(dry_badge_pos, Vec2::new(115.0, 18.0));
                painter.rect_filled(dry_badge, 4.0, Color32::from_rgb(18, 38, 60));
                painter.rect_stroke(dry_badge, 4.0, Stroke::new(1.0, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                painter.text(dry_badge.center(), egui::Align2::CENTER_CENTER, "DRY SOUND (MURNI)", egui::FontId::proportional(9.0), Color32::from_rgb(56, 189, 248));

                // --- PATH B: WET SOUND (Passing into LPF, filtered, then into Summing Mixer) ---
                let p_lpf_in = Pos2::new(card_lpf_pos.x - 47.5, card_lpf_pos.y);
                let p_lpf_out = Pos2::new(card_lpf_pos.x + 47.5, card_lpf_pos.y);

                draw_flowing_cable(&[p_split_out_bot, p_lpf_in], Color32::from_rgb(251, 146, 60), Color32::from_rgb(251, 146, 60), 1);
                draw_flowing_cable(&[p_lpf_out, p_mixer_in_bot], Color32::from_rgb(251, 146, 60), Color32::from_rgb(251, 146, 60), 1);

                // Wet Badge: Positioned cleanly below the LPF module card (no collision with cables!)
                let wet_badge_pos = Pos2::new(card_lpf_pos.x, card_lpf_pos.y + 32.0);
                let wet_badge = Rect::from_center_size(wet_badge_pos, Vec2::new(125.0, 18.0));
                painter.rect_filled(wet_badge, 4.0, Color32::from_rgb(45, 36, 20));
                painter.rect_stroke(wet_badge, 4.0, Stroke::new(1.0, Color32::from_rgb(251, 146, 60)), egui::StrokeKind::Outside);
                painter.text(wet_badge.center(), egui::Align2::CENTER_CENTER, "WET SOUND (FILTER LPF)", egui::FontId::proportional(9.0), Color32::from_rgb(251, 146, 60));

                // --- SUMMING MIXER TO OUTPUT DAC (Phase Aligned Combined Signal) ---
                let p_mixer_out = Pos2::new(card_mixer_pos.x + 45.0, card_mixer_pos.y);
                let p_dac_in = Pos2::new(card_dac_pos.x - 45.0, card_dac_pos.y);

                draw_flowing_cable(&[p_mixer_out, p_dac_in], Color32::from_rgb(52, 211, 153), Color32::from_rgb(52, 211, 153), 1);

                // Summing Badge: Positioned cleanly ABOVE the output cable with 9px vertical clearance
                let sum_mid_x = (p_mixer_out.x + p_dac_in.x) * 0.5;
                let sum_badge = Rect::from_center_size(Pos2::new(sum_mid_x, center.y - 18.0), Vec2::new(100.0, 16.0));
                painter.rect_filled(sum_badge, 4.0, Color32::from_rgb(20, 42, 35));
                painter.rect_stroke(sum_badge, 4.0, Stroke::new(1.0, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(sum_badge.center(), egui::Align2::CENTER_CENTER, "DRY + WET MIX", egui::FontId::proportional(8.5), Color32::from_rgb(52, 211, 153));
            }

            // Interactive Splitting & Summing toggle button: Positioned at center.y + 92.0 (plenty of room below wet badge!)
            let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 92.0), Vec2::new(290.0, 32.0));
            let btn_hov = ui.rect_contains_pointer(btn_rect);
            let btn_bg = if self.theater_split_sum_active {
                Color32::from_rgb(32, 24, 48)
            } else {
                Color32::from_rgb(22, 28, 44)
            };
            let btn_stroke_col = if self.theater_split_sum_active {
                Color32::from_rgb(168, 85, 247)
            } else {
                Color32::from_rgb(51, 65, 88)
            };

            painter.rect_filled(btn_rect, 8.0, btn_bg);
            painter.rect_stroke(btn_rect, 8.0, Stroke::new(if btn_hov { 1.6 } else { 1.0 }, btn_stroke_col), egui::StrokeKind::Outside);
            let btn_lbl = if self.theater_split_sum_active {
                "⚡ Mode Alur: [PARALEL DRY-WET (SPLIT & SUM)]"
            } else {
                "⚡ Mode Alur: [JALUR SERI TUNGGAL] (Klik untuk Coba Split)"
            };
            let text_col = if self.theater_split_sum_active {
                Color32::from_rgb(216, 180, 254)
            } else {
                Color32::from_rgb(203, 213, 225)
            };
            painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, btn_lbl, egui::FontId::proportional(10.5), text_col);

            if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                self.theater_split_sum_active = !self.theater_split_sum_active;
            }
        } else if self.theater_slide == 2 {
            // --- SLIDE 3 DIAGRAM: FILTER STUDIO, GRAFIK TELEMETRI & ATURAN SAKRAL DAG (8 BAGIAN) ---
            let draw_compact_card = |pos: Pos2, title: &str, sub: &str, w: f32, is_accent: bool, accent_col: Color32| {
                let r = Rect::from_center_size(pos, Vec2::new(w, 42.0));
                let bg = if is_accent { Color32::from_rgb(18, 25, 42) } else { Color32::from_rgb(14, 18, 28) };
                painter.rect_filled(r, 6.0, bg);
                painter.rect_stroke(
                    r,
                    6.0,
                    Stroke::new(if is_accent { 1.5 } else { 1.0 }, if is_accent { accent_col } else { Color32::from_rgb(45, 55, 78) }),
                    egui::StrokeKind::Outside,
                );
                painter.text(Pos2::new(pos.x, pos.y - 7.0), egui::Align2::CENTER_CENTER, title, egui::FontId::proportional(10.5), if is_accent { Color32::WHITE } else { Color32::from_rgb(160, 175, 200) });
                painter.text(Pos2::new(pos.x, pos.y + 9.0), egui::Align2::CENTER_CENTER, sub, egui::FontId::proportional(9.0), Color32::from_rgb(100, 116, 139));
            };

            if self.theater_turn < 4 {
                // =========================================================================
                // BAGIAN 1 - 4: EKOSISTEM FILTER & GRAFIK TELEMETRI REAL-TIME
                // =========================================================================
                let rack_y = center.y - 68.0;
                let c_eq = Pos2::new(center.x - 170.0, rack_y);
                let c_tube = Pos2::new(center.x, rack_y);
                let c_lpf = Pos2::new(center.x + 170.0, rack_y);

                let sel = self.theater_s3_selected_filter;

                // Card 0: 10-Band EQ
                draw_compact_card(
                    c_eq,
                    "10-BAND EQ",
                    "Bode Plot (31Hz-16kHz)",
                    140.0,
                    sel == 0,
                    Color32::from_rgb(56, 189, 248),
                );
                let eq_click_rect = Rect::from_center_size(c_eq, Vec2::new(140.0, 42.0));
                if ui.rect_contains_pointer(eq_click_rect) && ui.input(|i| i.pointer.any_pressed()) {
                    self.theater_s3_selected_filter = 0;
                }

                // Card 1: Tube Warmer 12AX7
                draw_compact_card(
                    c_tube,
                    "TUBE WARMER 12AX7",
                    "Spektrum FFT Harmonik",
                    145.0,
                    sel == 1,
                    Color32::from_rgb(251, 146, 60),
                );
                let tube_click_rect = Rect::from_center_size(c_tube, Vec2::new(145.0, 42.0));
                if ui.rect_contains_pointer(tube_click_rect) && ui.input(|i| i.pointer.any_pressed()) {
                    self.theater_s3_selected_filter = 1;
                }

                // Card 2: Low-Pass Filter (LPF)
                draw_compact_card(
                    c_lpf,
                    "LOW-PASS (LPF)",
                    "Kurva -12dB/oktaf",
                    140.0,
                    sel == 2,
                    Color32::from_rgb(52, 211, 153),
                );
                let lpf_click_rect = Rect::from_center_size(c_lpf, Vec2::new(140.0, 42.0));
                if ui.rect_contains_pointer(lpf_click_rect) && ui.input(|i| i.pointer.any_pressed()) {
                    self.theater_s3_selected_filter = 2;
                }

                // --- TELEMETRIC OSCILLOSCOPE / ANALYZER SCOPE DISPLAY ---
                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 16.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                // Screen background & studio bezel
                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                let scope_border_col = match sel {
                    0 => Color32::from_rgb(56, 189, 248),
                    1 => Color32::from_rgb(251, 146, 60),
                    _ => Color32::from_rgb(52, 211, 153),
                };
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, scope_border_col), egui::StrokeKind::Outside);

                // Scope Grid lines
                let grid_col = Color32::from_rgba_premultiplied(45, 60, 90, 65);
                let zero_db_y = scope_rect.min.y + scope_h * 0.52;
                // 0 dB center reference line
                painter.line_segment(
                    [Pos2::new(scope_rect.min.x + 8.0, zero_db_y), Pos2::new(scope_rect.max.x - 8.0, zero_db_y)],
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(100, 120, 160, 90)),
                );
                // +6dB and -6dB reference lines
                let p6_y = zero_db_y - 24.0;
                let m6_y = zero_db_y + 24.0;
                painter.line_segment([Pos2::new(scope_rect.min.x + 8.0, p6_y), Pos2::new(scope_rect.max.x - 8.0, p6_y)], Stroke::new(0.8, grid_col));
                painter.line_segment([Pos2::new(scope_rect.min.x + 8.0, m6_y), Pos2::new(scope_rect.max.x - 8.0, m6_y)], Stroke::new(0.8, grid_col));

                // Vertical octave grid lines & frequency labels
                let freq_markers = [
                    (0.08, "31Hz"),
                    (0.24, "125Hz"),
                    (0.42, "500Hz"),
                    (0.60, "2kHz"),
                    (0.78, "8kHz"),
                    (0.92, "16kHz"),
                ];
                for (fx, flbl) in &freq_markers {
                    let gx = scope_rect.min.x + scope_w * fx;
                    painter.line_segment([Pos2::new(gx, scope_rect.min.y + 6.0), Pos2::new(gx, scope_rect.max.y - 18.0)], Stroke::new(0.8, grid_col));
                    painter.text(Pos2::new(gx, scope_rect.max.y - 9.0), egui::Align2::CENTER_CENTER, flbl, egui::FontId::proportional(8.5), Color32::from_rgb(100, 116, 139));
                }

                // Telemetry Header Banner inside Scope
                match sel {
                    0 => {
                        // === CASE 0: 10-BAND EQ BODE PLOT ===
                        let header_txt = "● TELEMETRI BODE PLOT: 10-BAND EQ • Respon 31Hz - 16kHz • Presisi 64-bit Float";
                        painter.text(Pos2::new(scope_rect.min.x + 10.0, scope_rect.min.y + 11.0), egui::Align2::LEFT_CENTER, header_txt, egui::FontId::proportional(9.0), Color32::from_rgb(56, 189, 248));
                        painter.text(Pos2::new(scope_rect.max.x - 10.0, scope_rect.min.y + 11.0), egui::Align2::RIGHT_CENTER, "Gain: Bass +4dB | Mid -2dB | Air +3.5dB | Q=1.414", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                        // Generate smooth Bode Curve points
                        let curve_steps = 70;
                        let mut curve_pts = Vec::with_capacity(curve_steps);
                        for i in 0..=curve_steps {
                            let frac = i as f32 / curve_steps as f32;
                            let px = scope_rect.min.x + 12.0 + (scope_w - 24.0) * frac;
                            // Bode curve equation: bass boost at frac=0.15, dip at 0.45, air lift at 0.85
                            let b_boost = 18.0 * (-(frac - 0.15).powi(2) / 0.018).exp();
                            let m_dip = -11.0 * (-(frac - 0.45).powi(2) / 0.016).exp();
                            let air_lift = 14.0 * (-(frac - 0.85).powi(2) / 0.024).exp();
                            let undulation = (t * 3.0 + frac * 8.0).sin() * 1.5;
                            let py = zero_db_y - (b_boost + m_dip + air_lift + undulation);
                            curve_pts.push(Pos2::new(px, py));
                        }

                        // Draw shaded under-curve fill
                        for i in 0..curve_pts.len() - 1 {
                            let p1 = curve_pts[i];
                            let p2 = curve_pts[i + 1];
                            let poly = vec![
                                p1,
                                p2,
                                Pos2::new(p2.x, zero_db_y),
                                Pos2::new(p1.x, zero_db_y),
                            ];
                            painter.add(egui::Shape::convex_polygon(poly, Color32::from_rgba_premultiplied(56, 189, 248, 25), Stroke::NONE));
                        }

                        // Draw glowing curve line
                        for pair in curve_pts.windows(2) {
                            painter.line_segment([pair[0], pair[1]], Stroke::new(2.0, Color32::from_rgb(56, 189, 248)));
                        }

                        // Highlight EQ band nodes with glowing circles
                        for &(band_x, band_db) in &[(0.15, 18.0), (0.45, -11.0), (0.85, 14.0)] {
                            let b_px = scope_rect.min.x + 12.0 + (scope_w - 24.0) * band_x;
                            let b_py = zero_db_y - band_db;
                            painter.circle_filled(Pos2::new(b_px, b_py), 3.5, Color32::WHITE);
                            painter.circle_stroke(Pos2::new(b_px, b_py), 5.5, Stroke::new(1.0, Color32::from_rgb(56, 189, 248)));
                        }
                    }
                    1 => {
                        // === CASE 1: TUBE 12AX7 HARMONIC SPECTRUM FFT ===
                        let header_txt = "● TELEMETRI FFT HARMONIK: TUBE WARMER 12AX7 • Saturasi Trioda Tri-Pemberat";
                        painter.text(Pos2::new(scope_rect.min.x + 10.0, scope_rect.min.y + 11.0), egui::Align2::LEFT_CENTER, header_txt, egui::FontId::proportional(9.0), Color32::from_rgb(251, 146, 60));
                        painter.text(Pos2::new(scope_rect.max.x - 10.0, scope_rect.min.y + 11.0), egui::Align2::RIGHT_CENTER, "Fundamental 1kHz (0dB) • H2: -18.2dB • H3: -28.4dB • THD: 1.4%", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                        // Draw harmonic spikes
                        let tube_warm_pulse = (t * 5.0).sin() * 2.0;
                        let harmonics = [
                            (0.30, 0.0, "H1 (1kHz)", Color32::from_rgb(251, 191, 36), 46.0),
                            (0.48, -18.2, "H2 (2kHz)", Color32::from_rgb(249, 115, 22), 26.0 + tube_warm_pulse),
                            (0.64, -28.4, "H3 (3kHz)", Color32::from_rgb(245, 158, 11), 16.0),
                            (0.78, -42.0, "H4 (4kHz)", Color32::from_rgb(217, 119, 6), 9.0),
                        ];

                        for &(h_frac, _db, h_lbl, h_col, h_height) in &harmonics {
                            let hx = scope_rect.min.x + scope_w * h_frac;
                            let base_y = zero_db_y + 18.0;
                            let top_y = base_y - h_height;

                            // FFT bar
                            let bar_rect = Rect::from_min_max(Pos2::new(hx - 5.0, top_y), Pos2::new(hx + 5.0, base_y));
                            painter.rect_filled(bar_rect, 2.0, Color32::from_rgba_premultiplied(h_col.r(), h_col.g(), h_col.b(), 75));
                            painter.rect_stroke(bar_rect, 2.0, Stroke::new(1.2, h_col), egui::StrokeKind::Outside);

                            // Peak cap line
                            painter.line_segment([Pos2::new(hx - 7.0, top_y), Pos2::new(hx + 7.0, top_y)], Stroke::new(1.8, Color32::WHITE));
                            painter.text(Pos2::new(hx, top_y - 6.0), egui::Align2::CENTER_CENTER, h_lbl, egui::FontId::proportional(8.0), h_col);
                        }

                        // Analog noise floor shimmer
                        let mut noise_pts = Vec::with_capacity(30);
                        for i in 0..30 {
                            let nx = scope_rect.min.x + 12.0 + (scope_w - 24.0) * (i as f32 / 29.0);
                            let n_jitter = ((i as f32 * 17.3 + t * 20.0).sin() * 2.0).abs();
                            noise_pts.push(Pos2::new(nx, zero_db_y + 18.0 - n_jitter));
                        }
                        for pair in noise_pts.windows(2) {
                            painter.line_segment([pair[0], pair[1]], Stroke::new(0.8, Color32::from_rgba_premultiplied(100, 116, 139, 80)));
                        }
                    }
                    _ => {
                        // === CASE 2: LOW-PASS FILTER ROLL-OFF CURVE ===
                        let header_txt = "● TELEMETRI ROLL-OFF: LOW-PASS FILTER (LPF) • Pemotongan Treble Tajam";
                        painter.text(Pos2::new(scope_rect.min.x + 10.0, scope_rect.min.y + 11.0), egui::Align2::LEFT_CENTER, header_txt, egui::FontId::proportional(9.0), Color32::from_rgb(52, 211, 153));
                        painter.text(Pos2::new(scope_rect.max.x - 10.0, scope_rect.min.y + 11.0), egui::Align2::RIGHT_CENTER, "Cutoff: 2.500 Hz | Slope: -12 dB/oktaf (2-Pole) | Stopband: >3kHz", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                        let cutoff_frac = 0.58;
                        let cutoff_x = scope_rect.min.x + scope_w * cutoff_frac;

                        // Vertical dashed cutoff marker
                        let mut cy_walk = scope_rect.min.y + 20.0;
                        while cy_walk < scope_rect.max.y - 18.0 {
                            painter.line_segment(
                                [Pos2::new(cutoff_x, cy_walk), Pos2::new(cutoff_x, (cy_walk + 4.0).min(scope_rect.max.y - 18.0))],
                                Stroke::new(1.0, Color32::from_rgb(52, 211, 153)),
                            );
                            cy_walk += 7.0;
                        }

                        // Shaded Attenuated Stopband Region
                        let stopband_rect = Rect::from_min_max(
                            Pos2::new(cutoff_x, scope_rect.min.y + 20.0),
                            Pos2::new(scope_rect.max.x - 8.0, scope_rect.max.y - 18.0),
                        );
                        painter.rect_filled(stopband_rect, 0.0, Color32::from_rgba_premultiplied(239, 68, 68, 16));
                        painter.text(
                            Pos2::new((cutoff_x + scope_rect.max.x) * 0.5, scope_rect.min.y + 32.0),
                            egui::Align2::CENTER_CENTER,
                            "WILAYAH TERPOTONG (-12dB/oct)",
                            egui::FontId::proportional(8.0),
                            Color32::from_rgba_premultiplied(248, 113, 113, 180),
                        );

                        // Draw Butterworth curve
                        let curve_steps = 70;
                        let mut lpf_pts = Vec::with_capacity(curve_steps);
                        for i in 0..=curve_steps {
                            let frac = i as f32 / curve_steps as f32;
                            let px = scope_rect.min.x + 12.0 + (scope_w - 24.0) * frac;
                            let py = if frac <= cutoff_frac {
                                zero_db_y - 2.0 // Flat passband
                            } else {
                                let over = (frac - cutoff_frac) / (1.0 - cutoff_frac);
                                (zero_db_y - 2.0) + over.powf(1.6) * 36.0 // Steep roll-off
                            };
                            lpf_pts.push(Pos2::new(px, py));
                        }

                        for pair in lpf_pts.windows(2) {
                            painter.line_segment([pair[0], pair[1]], Stroke::new(2.2, Color32::from_rgb(52, 211, 153)));
                        }

                        // Cutoff point dot
                        painter.circle_filled(Pos2::new(cutoff_x, zero_db_y - 2.0), 4.0, Color32::WHITE);
                        painter.circle_stroke(Pos2::new(cutoff_x, zero_db_y - 2.0), 6.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)));
                        painter.text(Pos2::new(cutoff_x, zero_db_y - 14.0), egui::Align2::CENTER_CENTER, "fc = 2.5kHz (-3dB)", egui::FontId::proportional(8.5), Color32::WHITE);
                    }
                }

                // Interactive Filter Selector Pill at bottom (center.y + 88.0)
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(340.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(18, 25, 42));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, scope_border_col), egui::StrokeKind::Outside);

                let sel_label = match sel {
                    0 => "🔍 Telemetri Aktif: [10-Band EQ] (Klik untuk ganti ke Tabung)",
                    1 => "🔍 Telemetri Aktif: [Tabung 12AX7] (Klik untuk ganti ke LPF)",
                    _ => "🔍 Telemetri Aktif: [Low-Pass Filter] (Klik untuk ganti ke EQ)",
                };
                painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, sel_label, egui::FontId::proportional(10.0), scope_border_col);

                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s3_selected_filter = (self.theater_s3_selected_filter + 1) % 3;
                }
            } else if self.theater_turn == 4 || self.theater_turn == 5 {
                // =========================================================================
                // BAGIAN 5 & 6: KONSEP DAG - POSISI FILTER BERPENGARUH KE SUARA
                // =========================================================================
                let src_pos = Pos2::new(center.x - 225.0, center.y - 12.0);
                let f1_pos = Pos2::new(center.x - 75.0, center.y - 12.0);
                let f2_pos = Pos2::new(center.x + 75.0, center.y - 12.0);
                let dac_pos = Pos2::new(center.x + 225.0, center.y - 12.0);

                let is_swapped = self.theater_s3_order_swapped;

                // Top Order Impact Badge
                let badge_pos = Pos2::new(center.x, center.y - 66.0);
                let badge_rect = Rect::from_center_size(badge_pos, Vec2::new(510.0, 24.0));
                let (badge_bg, badge_strk, badge_txt, badge_col) = if !is_swapped {
                    (
                        Color32::from_rgb(18, 38, 60),
                        Color32::from_rgb(56, 189, 248),
                        "✨ URUTAN A: [TABUNG -> EQ] • Karakter: Harmonik Analog Halus Disuntik Duluan, Lalu Diukir EQ",
                        Color32::from_rgb(224, 242, 254),
                    )
                } else {
                    (
                        Color32::from_rgb(45, 28, 18),
                        Color32::from_rgb(251, 146, 60),
                        "🔥 URUTAN B: [EQ -> TABUNG] • Karakter: Bass Di-boost Dulu -> Tabung Tersaturasi Tebal & Menggigit!",
                        Color32::from_rgb(255, 237, 213),
                    )
                };
                painter.rect_filled(badge_rect, 6.0, badge_bg);
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, badge_strk), egui::StrokeKind::Outside);
                painter.text(badge_rect.center(), egui::Align2::CENTER_CENTER, badge_txt, egui::FontId::proportional(9.5), badge_col);

                // Modules
                draw_compact_card(src_pos, "FILE SOURCE", "FLAC Master", 85.0, true, Color32::from_rgb(56, 189, 248));

                if !is_swapped {
                    // Filter 1: Tube 12AX7, Filter 2: 10-Band EQ
                    draw_compact_card(f1_pos, "TUBE 12AX7", "Drive: 35%", 95.0, true, Color32::from_rgb(251, 146, 60));
                    draw_compact_card(f2_pos, "10-BAND EQ", "Bass +0dB | Air +3dB", 100.0, true, Color32::from_rgb(56, 189, 248));
                } else {
                    // Filter 1: 10-Band EQ, Filter 2: Tube 12AX7
                    draw_compact_card(f1_pos, "10-BAND EQ", "Bass Boost +6dB!", 100.0, true, Color32::from_rgb(56, 189, 248));
                    draw_compact_card(f2_pos, "TUBE 12AX7", "Drive Menjerit (Overdrive)", 105.0, true, Color32::from_rgb(251, 146, 60));
                }

                draw_compact_card(dac_pos, "OUTPUT DAC", "32-bit Float", 85.0, true, Color32::from_rgb(168, 85, 247));

                // Flowing Cables (110 px/s constant physical speed)
                let p_src_out = Pos2::new(src_pos.x + 42.5, src_pos.y);
                let p_f1_in = Pos2::new(f1_pos.x - 47.5, f1_pos.y);
                let p_f1_out = Pos2::new(f1_pos.x + 47.5, f1_pos.y);
                let p_f2_in = Pos2::new(f2_pos.x - 50.0, f2_pos.y);
                let p_f2_out = Pos2::new(f2_pos.x + 50.0, f2_pos.y);
                let p_dac_in = Pos2::new(dac_pos.x - 42.5, dac_pos.y);

                let cable_col = if !is_swapped { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(251, 146, 60) };
                draw_flowing_cable(&[p_src_out, p_f1_in], cable_col, Color32::WHITE, 1);
                draw_flowing_cable(&[p_f1_out, p_f2_in], cable_col, Color32::WHITE, 1);
                draw_flowing_cable(&[p_f2_out, p_dac_in], cable_col, Color32::WHITE, 1);

                // Telemetry summary pill below nodes
                let summary_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 36.0), Vec2::new(450.0, 22.0));
                painter.rect_filled(summary_rect, 4.0, Color32::from_rgb(14, 18, 28));
                painter.rect_stroke(summary_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                let desc_txt = if !is_swapped {
                    "Hasil: Vokal hangat empuk, detail treble tetap jernih dan terkontrol rapi"
                } else {
                    "Hasil: Tabung distorsi tebal karena sinyal bass yang diperkuat mendrive tabung lebih keras"
                };
                painter.text(summary_rect.center(), egui::Align2::CENTER_CENTER, desc_txt, egui::FontId::proportional(9.0), Color32::from_rgb(203, 213, 225));

                // Interactive Order Toggle Button (center.y + 88.0)
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(320.0, 32.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 8.0, if is_swapped { Color32::from_rgb(45, 28, 18) } else { Color32::from_rgb(18, 38, 60) });
                painter.rect_stroke(btn_rect, 8.0, Stroke::new(if btn_hov { 1.6 } else { 1.0 }, badge_strk), egui::StrokeKind::Outside);
                let btn_lbl = if !is_swapped {
                    "🔀 Klik untuk Uji: [Tukar Menjadi EQ -> Tube]"
                } else {
                    "🔀 Klik untuk Uji: [Kembalikan ke Tube -> EQ]"
                };
                painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, btn_lbl, egui::FontId::proportional(10.5), badge_col);

                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s3_order_swapped = !self.theater_s3_order_swapped;
                }
            } else if self.theater_turn == 6 {
                // =========================================================================
                // BAGIAN 7: HUKUM SAKRAL DAG - ACYCLIC (DILARANG FEEDBACK LOOP)
                // =========================================================================
                let f1_pos = Pos2::new(center.x - 120.0, center.y - 25.0);
                let f2_pos = Pos2::new(center.x + 120.0, center.y - 25.0);

                draw_compact_card(f1_pos, "MODUL A: TABUNG", "DSP Input/Output", 120.0, true, Color32::from_rgb(251, 146, 60));
                draw_compact_card(f2_pos, "MODUL B: EQUALIZER", "DSP Input/Output", 125.0, true, Color32::from_rgb(56, 189, 248));

                // Legal Forward Cable: A -> B
                let p_f1_out = Pos2::new(f1_pos.x + 60.0, f1_pos.y);
                let p_f2_in = Pos2::new(f2_pos.x - 62.5, f2_pos.y);
                draw_flowing_cable(&[p_f1_out, p_f2_in], Color32::from_rgb(52, 211, 153), Color32::WHITE, 2);

                let forward_badge = Rect::from_center_size(Pos2::new(center.x, f1_pos.y - 18.0), Vec2::new(140.0, 16.0));
                painter.rect_filled(forward_badge, 4.0, Color32::from_rgb(20, 42, 35));
                painter.text(forward_badge.center(), egui::Align2::CENTER_CENTER, "JALUR SAH (TERARAH)", egui::FontId::proportional(8.5), Color32::from_rgb(52, 211, 153));

                // Illegal Loop Cable (EQ Output -> Tube Input) - Crimson dashed line
                let p_f2_out = Pos2::new(f2_pos.x + 62.5, f2_pos.y);
                let loop_y = center.y + 36.0;
                let c_loop1 = Pos2::new(f2_pos.x + 85.0, f2_pos.y);
                let c_loop2 = Pos2::new(f2_pos.x + 85.0, loop_y);
                let c_barrier = Pos2::new(center.x, loop_y);
                let c_loop3 = Pos2::new(f1_pos.x - 85.0, loop_y);
                let c_loop4 = Pos2::new(f1_pos.x - 85.0, f1_pos.y);
                let p_f1_in = Pos2::new(f1_pos.x - 60.0, f1_pos.y);

                // Draw dashed warning loop wire
                let loop_stroke = Stroke::new(1.8, Color32::from_rgb(239, 68, 68));
                let draw_dashed_seg = |p1: Pos2, p2: Pos2| {
                    let len = (p2 - p1).length();
                    let dir = (p2 - p1) / len.max(1.0);
                    let mut cur = 0.0;
                    while cur < len {
                        let seg_end = (cur + 5.0).min(len);
                        painter.line_segment([p1 + dir * cur, p1 + dir * seg_end], loop_stroke);
                        cur += 9.0;
                    }
                };
                draw_dashed_seg(p_f2_out, c_loop1);
                draw_dashed_seg(c_loop1, c_loop2);
                draw_dashed_seg(c_loop2, c_barrier);
                draw_dashed_seg(c_barrier, c_loop3);
                draw_dashed_seg(c_loop3, c_loop4);
                draw_dashed_seg(c_loop4, p_f1_in);

                // Glowing Anti-Loop Protection Barrier Shield at center of loop
                let shield_pulse = (t * 6.0).sin().abs();
                let shield_r = 16.0 + shield_pulse * 3.0;
                painter.circle_filled(c_barrier, shield_r, Color32::from_rgba_premultiplied(220, 38, 38, 55));
                painter.circle_stroke(c_barrier, shield_r, Stroke::new(1.6, Color32::from_rgb(239, 68, 68)));
                painter.circle_filled(c_barrier, 9.0, Color32::from_rgb(220, 38, 38));

                // Crisp procedural X barrier inside shield
                let bx_r = 4.0;
                painter.line_segment([Pos2::new(c_barrier.x - bx_r, c_barrier.y - bx_r), Pos2::new(c_barrier.x + bx_r, c_barrier.y + bx_r)], Stroke::new(1.8, Color32::WHITE));
                painter.line_segment([Pos2::new(c_barrier.x - bx_r, c_barrier.y + bx_r), Pos2::new(c_barrier.x + bx_r, c_barrier.y - bx_r)], Stroke::new(1.8, Color32::WHITE));

                // Barrier Shield Badge
                let banner_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 78.0), Vec2::new(510.0, 34.0));
                painter.rect_filled(banner_rect, 6.0, Color32::from_rgb(45, 18, 24));
                painter.rect_stroke(banner_rect, 6.0, Stroke::new(1.4, Color32::from_rgb(239, 68, 68)), egui::StrokeKind::Outside);
                painter.text(
                    Pos2::new(banner_rect.center().x, banner_rect.center().y - 6.0),
                    egui::Align2::CENTER_CENTER,
                    "🚫 SIKLUS LOOP DIBLOKIR: Anti-Feedback Interlock Aktif (+∞ dB Proteksi)",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(254, 202, 202),
                );
                painter.text(
                    Pos2::new(banner_rect.center().x, banner_rect.center().y + 8.0),
                    egui::Align2::CENTER_CENTER,
                    "DAG melarang output dihubungkan kembali ke modul sebelumnya demi melindungi speaker & DAC",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(248, 113, 113),
                );
            } else {
                // =========================================================================
                // BAGIAN 8: KASKADE MULTI-FILTER SERI (100% LEGAL & DIDUKUNG PENUH)
                // =========================================================================
                let node_y = center.y - 12.0;
                let c_src = Pos2::new(center.x - 230.0, node_y);
                let c_tube = Pos2::new(center.x - 115.0, node_y);
                let c_eq = Pos2::new(center.x, node_y);
                let c_lpf = Pos2::new(center.x + 115.0, node_y);
                let c_dac = Pos2::new(center.x + 230.0, node_y);

                draw_compact_card(c_src, "FILE SOURCE", "FLAC 24-bit", 80.0, true, Color32::from_rgb(56, 189, 248));
                draw_compact_card(c_tube, "TUBE 12AX7", "Harmonik Empuk", 85.0, true, Color32::from_rgb(251, 146, 60));
                draw_compact_card(c_eq, "10-BAND EQ", "Bode Curve", 85.0, true, Color32::from_rgb(56, 189, 248));
                draw_compact_card(c_lpf, "LOW-PASS", "Treble Shaper", 80.0, true, Color32::from_rgb(52, 211, 153));
                draw_compact_card(c_dac, "OUTPUT DAC", "32-bit Float", 80.0, true, Color32::from_rgb(168, 85, 247));

                // Flowing Emerald Cables connecting all 4 serial stages!
                let p1_out = Pos2::new(c_src.x + 40.0, node_y);
                let p2_in = Pos2::new(c_tube.x - 42.5, node_y);
                let p2_out = Pos2::new(c_tube.x + 42.5, node_y);
                let p3_in = Pos2::new(c_eq.x - 42.5, node_y);
                let p3_out = Pos2::new(c_eq.x + 42.5, node_y);
                let p4_in = Pos2::new(c_lpf.x - 40.0, node_y);
                let p4_out = Pos2::new(c_lpf.x + 40.0, node_y);
                let p5_in = Pos2::new(c_dac.x - 40.0, node_y);

                let em_col = Color32::from_rgb(52, 211, 153);
                draw_flowing_cable(&[p1_out, p2_in], em_col, Color32::WHITE, 1);
                draw_flowing_cable(&[p2_out, p3_in], em_col, Color32::WHITE, 1);
                draw_flowing_cable(&[p3_out, p4_in], em_col, Color32::WHITE, 1);
                draw_flowing_cable(&[p4_out, p5_in], em_col, Color32::WHITE, 1);

                // Top Cascade Badge
                let top_badge = Rect::from_center_size(Pos2::new(center.x, center.y - 66.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(top_badge, 6.0, Color32::from_rgb(18, 42, 32));
                painter.rect_stroke(top_badge, 6.0, Stroke::new(1.2, em_col), egui::StrokeKind::Outside);
                painter.text(
                    top_badge.center(),
                    egui::Align2::CENTER_CENTER,
                    "✅ KASKADE MULTI-FILTER SERI: Boleh & Didukung Penuh 100%!",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(220, 252, 231),
                );

                // Bottom Tech Specifications
                let bot_badge = Rect::from_center_size(Pos2::new(center.x, center.y + 40.0), Vec2::new(480.0, 22.0));
                painter.rect_filled(bot_badge, 4.0, Color32::from_rgb(14, 18, 28));
                painter.rect_stroke(bot_badge, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                painter.text(
                    bot_badge.center(),
                    egui::Align2::CENTER_CENTER,
                    "Presisi 64-bit Float • 300+ dB Headroom Dinamis • Nol Clipping • Zero Latency",
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(148, 163, 184),
                );

                // Lanjut ke Slide 4 Action Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 84.0), Vec2::new(340.0, 32.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 8.0, Color32::from_rgb(22, 48, 40));
                painter.rect_stroke(btn_rect, 8.0, Stroke::new(if btn_hov { 1.6 } else { 1.2 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✨ Lanjut ke Slide 4: Now Playing & Antrean ▶",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(220, 252, 231),
                );

                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_slide = 3;
                    self.theater_turn = 0;
                    self.theater_turn_timer = 0.0;
                }
            }
        } else if self.theater_slide == 3 {
            // --- SLIDE 4 DIAGRAM: NOW PLAYING, OSILOSKOP, NAVIGASI WAKTU & ANTREAN (6 BAGIAN) ---
            let draw_compact_card = |pos: Pos2, title: &str, sub: &str, w: f32, is_accent: bool, accent_col: Color32| {
                let r = Rect::from_center_size(pos, Vec2::new(w, 42.0));
                let bg = if is_accent { Color32::from_rgb(18, 25, 42) } else { Color32::from_rgb(14, 18, 28) };
                painter.rect_filled(r, 6.0, bg);
                painter.rect_stroke(
                    r,
                    6.0,
                    Stroke::new(if is_accent { 1.5 } else { 1.0 }, if is_accent { accent_col } else { Color32::from_rgb(45, 55, 78) }),
                    egui::StrokeKind::Outside,
                );
                painter.text(Pos2::new(pos.x, pos.y - 7.0), egui::Align2::CENTER_CENTER, title, egui::FontId::proportional(10.5), if is_accent { Color32::WHITE } else { Color32::from_rgb(160, 175, 200) });
                painter.text(Pos2::new(pos.x, pos.y + 9.0), egui::Align2::CENTER_CENTER, sub, egui::FontId::proportional(9.0), Color32::from_rgb(100, 116, 139));
            };

            if self.theater_turn == 0 {
                // =========================================================================
                // BAGIAN 1: KANVAS WAVEFORM SPEKTRAL 3-BAND (BASS, MID, TREBLE) & ZOOM
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(18, 32, 54));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✨ 3-BAND SPECTRAL WAVE: Pink (<250Hz) • Cyan (250Hz-4kHz) • Emas (>4kHz) • Zoom [- 1.8x +]",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(224, 242, 254),
                );

                // Waveform Canvas Display Screen
                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 12.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);

                // Track title header inside waveform
                painter.text(
                    Pos2::new(scope_rect.min.x + 12.0, scope_rect.min.y + 12.0),
                    egui::Align2::LEFT_CENTER,
                    "🎵 INTERNET OVERDOSE • NEEDY GIRL OVERDOSE (FLAC 44.1k / 16-bit)",
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(241, 245, 249),
                );
                painter.text(
                    Pos2::new(scope_rect.max.x - 12.0, scope_rect.min.y + 12.0),
                    egui::Align2::RIGHT_CENTER,
                    "🔍 1.8x (Ctrl + Scroll)",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(148, 163, 184),
                );

                // Draw the 3-Band Waveform layers
                let mid_y = scope_rect.center().y + 8.0;
                let steps = 90;

                // 1. Bass Layer (<250Hz) - Smooth swelling sub-bass curve in Neon Pink Magenta
                let mut bass_pts = Vec::with_capacity(steps);
                for i in 0..steps {
                    let frac = i as f32 / (steps - 1) as f32;
                    let px = scope_rect.min.x + 10.0 + (scope_w - 20.0) * frac;
                    let env = ((frac * 3.14159).sin()).powf(1.4);
                    let wave = ((frac * 6.28 * 2.0).sin() * 0.7 + (frac * 6.28 * 5.0).sin() * 0.3) * env;
                    let py = mid_y - wave * 28.0;
                    bass_pts.push(Pos2::new(px, py));
                }
                for pair in bass_pts.windows(2) {
                    painter.line_segment([pair[0], pair[1]], Stroke::new(2.2, Color32::from_rgb(244, 114, 182)));
                }

                // 2. Mid Layer (250Hz-4kHz) - Voice Formants & Guitars in Electric Cyan
                let mut mid_pts = Vec::with_capacity(steps);
                for i in 0..steps {
                    let frac = i as f32 / (steps - 1) as f32;
                    let px = scope_rect.min.x + 10.0 + (scope_w - 20.0) * frac;
                    let env = ((frac * 3.14159 * 1.5).sin()).abs();
                    let wave = ((frac * 6.28 * 7.0 + 1.2).sin() * 0.6 + (frac * 6.28 * 13.0).sin() * 0.4) * env;
                    let py = mid_y - wave * 20.0;
                    mid_pts.push(Pos2::new(px, py));
                }
                for pair in mid_pts.windows(2) {
                    painter.line_segment([pair[0], pair[1]], Stroke::new(1.8, Color32::from_rgb(56, 189, 248)));
                }

                // 3. Treble Layer (>4kHz) - Crisp Shimmer & Air in Warm Gold
                let mut treble_pts = Vec::with_capacity(steps);
                for i in 0..steps {
                    let frac = i as f32 / (steps - 1) as f32;
                    let px = scope_rect.min.x + 10.0 + (scope_w - 20.0) * frac;
                    let wave = ((frac * 6.28 * 21.0).sin() * 0.5 + (frac * 6.28 * 35.0).cos() * 0.5) * 0.6;
                    let py = mid_y - wave * 13.0;
                    treble_pts.push(Pos2::new(px, py));
                }
                for pair in treble_pts.windows(2) {
                    painter.line_segment([pair[0], pair[1]], Stroke::new(1.2, Color32::from_rgb(251, 191, 36)));
                }

                // Dynamic Moving Playhead with Needle & Top Cap
                let playhead_progress = ((t * 0.12) % 1.0).clamp(0.05, 0.95);
                let head_x = scope_rect.min.x + 10.0 + (scope_w - 20.0) * playhead_progress;
                let head_top = scope_rect.min.y + 24.0;
                let head_bot = scope_rect.max.y - 12.0;

                painter.line_segment([Pos2::new(head_x, head_top), Pos2::new(head_x, head_bot)], Stroke::new(2.0, Color32::from_rgb(56, 189, 248)));
                painter.circle_filled(Pos2::new(head_x, head_top), 4.5, Color32::from_rgb(2, 132, 199));
                painter.circle_filled(Pos2::new(head_x, head_top), 2.5, Color32::WHITE);

                // Timecode Pill inside waveform
                let time_txt = format!("{:02}:{:02} / 03:38 (-{:02}:{:02})", (playhead_progress * 218.0) as u32 / 60, (playhead_progress * 218.0) as u32 % 60, ((1.0 - playhead_progress) * 218.0) as u32 / 60, ((1.0 - playhead_progress) * 218.0) as u32 % 60);
                painter.text(Pos2::new(head_x + 8.0, head_top + 4.0), egui::Align2::LEFT_TOP, time_txt, egui::FontId::proportional(8.5), Color32::from_rgb(186, 230, 253));

                // Bottom Interactive Mode Pill
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(340.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(18, 25, 42));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                let wave_lbl = if self.theater_s4_wave_mode == 0 {
                    "📊 Tampilan: [3-Band Layers] (Klik untuk Coba Spectral Blend)"
                } else {
                    "🌈 Tampilan: [Spectral Blend] (Klik untuk Kembali ke 3-Band)"
                };
                painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, wave_lbl, egui::FontId::proportional(10.0), Color32::from_rgb(56, 189, 248));
                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s4_wave_mode = (self.theater_s4_wave_mode + 1) % 2;
                }
            } else if self.theater_turn == 1 {
                // =========================================================================
                // BAGIAN 2: LIVING OSCILLOSCOPE BEAM & 48-BAND SPECTRUM ANALYZER
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(24, 20, 48));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "⚡ LIVING OSCILLOSCOPE BEAM: Tombol Play Bermutasi Menjadi Gelombang Audio 64-bit!",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(243, 232, 255),
                );

                // Scope & Controls Background Area
                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 12.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);

                // Animated Real-Time Spectrum Analyzer Bars (40 Bars)
                let num_bars = 40;
                let bar_area_w = scope_w - 40.0;
                let bar_w = (bar_area_w / num_bars as f32) - 2.0;
                let base_y = scope_rect.max.y - 14.0;

                for b in 0..num_bars {
                    let bx = scope_rect.min.x + 20.0 + b as f32 * (bar_w + 2.0);
                    let frac = b as f32 / num_bars as f32;
                    let speed = 8.0 + frac * 12.0;
                    let bounce = ((t * speed + b as f32 * 0.8).sin().abs()).powf(1.5);
                    let bar_h = (bounce * 42.0).clamp(4.0, 55.0);

                    let bar_col = if frac < 0.25 {
                        Color32::from_rgb(244, 114, 182) // Bass pink
                    } else if frac < 0.70 {
                        Color32::from_rgb(56, 189, 248)  // Mid cyan
                    } else {
                        Color32::from_rgb(251, 191, 36)  // Treble gold
                    };

                    let r = Rect::from_min_max(Pos2::new(bx, base_y - bar_h), Pos2::new(bx + bar_w, base_y));
                    painter.rect_filled(r, 1.5, Color32::from_rgba_premultiplied(bar_col.r(), bar_col.g(), bar_col.b(), 140));
                    // Cap line
                    painter.line_segment([Pos2::new(bx, base_y - bar_h), Pos2::new(bx + bar_w, base_y - bar_h)], Stroke::new(1.5, Color32::WHITE));
                }

                // Center Floating Living Play Button with Morphing Live Oscilloscope Beam
                let play_center = Pos2::new(center.x, scope_rect.min.y + 36.0);
                let btn_r = 24.0;

                // Pulsing glow aura
                let aura_pulse = (t * 5.0).sin().abs() * 4.0;
                painter.circle_filled(play_center, btn_r + aura_pulse + 4.0, Color32::from_rgba_premultiplied(14, 165, 233, 40));
                painter.circle_filled(play_center, btn_r, Color32::from_rgb(15, 23, 42));
                painter.circle_stroke(play_center, btn_r, Stroke::new(2.0, Color32::from_rgb(56, 189, 248)));

                // Live Oscilloscope Beam inside Play Button
                let beam_pts_count = 20;
                let beam_w = 28.0;
                let mut beam_pts = Vec::with_capacity(beam_pts_count);
                for p in 0..beam_pts_count {
                    let frac = p as f32 / (beam_pts_count - 1) as f32;
                    let px = (play_center.x - beam_w * 0.5) + frac * beam_w;
                    let window = (3.14159 * frac).sin();
                    let wave = (t * 16.0 + frac * 12.0).sin() * 7.0 * window;
                    beam_pts.push(Pos2::new(px, play_center.y - wave));
                }
                for pair in beam_pts.windows(2) {
                    painter.line_segment([pair[0], pair[1]], Stroke::new(3.2, Color32::from_rgba_premultiplied(14, 165, 233, 160)));
                    painter.line_segment([pair[0], pair[1]], Stroke::new(1.6, Color32::WHITE));
                }

                // Bottom Spec Pill
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(420.0, 30.0));
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(18, 25, 42));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔥 60 FPS • 48-Band FFT High Precision • True Peak 0 dBFS • Tanpa Delay",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(216, 180, 254),
                );
            } else if self.theater_turn == 2 {
                // =========================================================================
                // BAGIAN 3: ATURAN DETIK MELOMPAT (FORWARD +5S & REWIND -5S)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(38, 28, 14));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(251, 191, 36)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "⏱️ ATURAN DETIK: Panah Kanan (+5.0s) • Panah Kiri (-5.0s) • Ctrl+Panah (Ganti Trek)",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(254, 243, 199),
                );

                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 12.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(251, 191, 36)), egui::StrokeKind::Outside);

                // Timeline Scrubber Track
                let line_y = scope_rect.min.y + 36.0;
                let bar_start_x = scope_rect.min.x + 50.0;
                let bar_end_x = scope_rect.max.x - 50.0;
                let bar_total_w = bar_end_x - bar_start_x;

                let cur_time = self.theater_s4_seek_time.clamp(0.0, 60.0);
                let head_frac = cur_time / 60.0;
                let head_x = bar_start_x + bar_total_w * head_frac;

                // Background track
                painter.line_segment([Pos2::new(bar_start_x, line_y), Pos2::new(bar_end_x, line_y)], Stroke::new(4.0, Color32::from_rgb(30, 41, 59)));
                // Played track
                painter.line_segment([Pos2::new(bar_start_x, line_y), Pos2::new(head_x, line_y)], Stroke::new(4.0, Color32::from_rgb(251, 191, 36)));
                // Needle
                painter.circle_filled(Pos2::new(head_x, line_y), 6.0, Color32::WHITE);
                painter.circle_stroke(Pos2::new(head_x, line_y), 8.0, Stroke::new(1.5, Color32::from_rgb(251, 191, 36)));

                // Time labels
                painter.text(Pos2::new(bar_start_x - 8.0, line_y), egui::Align2::RIGHT_CENTER, "00:00", egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
                painter.text(Pos2::new(bar_end_x + 8.0, line_y), egui::Align2::LEFT_CENTER, "01:00", egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));

                let cur_txt = format!("Posisi Sekarang: 00:{:02.0} s", cur_time);
                painter.text(Pos2::new(head_x, line_y - 16.0), egui::Align2::CENTER_CENTER, cur_txt, egui::FontId::proportional(9.5), Color32::from_rgb(251, 191, 36));

                // Jump Arc Illustrations
                let arc_y = line_y + 32.0;
                let rw_pill = Rect::from_center_size(Pos2::new(center.x - 120.0, arc_y), Vec2::new(160.0, 26.0));
                painter.rect_filled(rw_pill, 4.0, Color32::from_rgb(26, 22, 14));
                painter.rect_stroke(rw_pill, 4.0, Stroke::new(1.0, Color32::from_rgb(251, 146, 60)), egui::StrokeKind::Outside);
                painter.text(rw_pill.center(), egui::Align2::CENTER_CENTER, "◀◀ Rewind: -5 Detik (←)", egui::FontId::proportional(9.0), Color32::from_rgb(253, 186, 116));

                let fw_pill = Rect::from_center_size(Pos2::new(center.x + 120.0, arc_y), Vec2::new(160.0, 26.0));
                painter.rect_filled(fw_pill, 4.0, Color32::from_rgb(18, 32, 26));
                painter.rect_stroke(fw_pill, 4.0, Stroke::new(1.0, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(fw_pill.center(), egui::Align2::CENTER_CENTER, "Forward: +5 Detik (→) ▶▶", egui::FontId::proportional(9.0), Color32::from_rgb(167, 243, 208));

                // Interactive Test Jump Buttons at bottom
                let rw_btn_rect = Rect::from_center_size(Pos2::new(center.x - 110.0, center.y + 88.0), Vec2::new(170.0, 30.0));
                let fw_btn_rect = Rect::from_center_size(Pos2::new(center.x + 110.0, center.y + 88.0), Vec2::new(170.0, 30.0));

                let rw_hov = ui.rect_contains_pointer(rw_btn_rect);
                let fw_hov = ui.rect_contains_pointer(fw_btn_rect);

                painter.rect_filled(rw_btn_rect, 6.0, Color32::from_rgb(28, 22, 16));
                painter.rect_stroke(rw_btn_rect, 6.0, Stroke::new(if rw_hov { 1.5 } else { 1.0 }, Color32::from_rgb(251, 146, 60)), egui::StrokeKind::Outside);
                painter.text(rw_btn_rect.center(), egui::Align2::CENTER_CENTER, "⏪ Klik Uji: Mundur 5s", egui::FontId::proportional(9.5), Color32::from_rgb(253, 186, 116));

                painter.rect_filled(fw_btn_rect, 6.0, Color32::from_rgb(18, 32, 26));
                painter.rect_stroke(fw_btn_rect, 6.0, Stroke::new(if fw_hov { 1.5 } else { 1.0 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(fw_btn_rect.center(), egui::Align2::CENTER_CENTER, "⏩ Klik Uji: Maju 5s", egui::FontId::proportional(9.5), Color32::from_rgb(167, 243, 208));

                if ui.input(|i| i.pointer.any_pressed()) {
                    if rw_hov {
                        self.theater_s4_seek_time = (self.theater_s4_seek_time - 5.0).max(0.0);
                    } else if fw_hov {
                        self.theater_s4_seek_time = (self.theater_s4_seek_time + 5.0).min(60.0);
                    }
                }
            } else if self.theater_turn == 3 {
                // =========================================================================
                // BAGIAN 4: SHUFFLE ENGINE (TOGGLE ACAK VS ACAK ANTREAN FISHER-YATES)
                // =========================================================================
                let is_shuf = self.theater_s4_shuffled;
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(18, 38, 32));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔀 SHUFFLE ENGINE: Pemilihan Trek Acak vs Pengocokan Susunan Fisik Antrean",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(220, 252, 231),
                );

                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 12.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);

                let node_y = scope_rect.center().y - 10.0;
                let t1 = Pos2::new(center.x - 180.0, node_y);
                let t2 = Pos2::new(center.x - 60.0, node_y);
                let t3 = Pos2::new(center.x + 60.0, node_y);
                let t4 = Pos2::new(center.x + 180.0, node_y);

                if !is_shuf {
                    // Sequential Mode: Straight Cyan Line
                    draw_compact_card(t1, "1. INTERNET", "Track 01", 85.0, true, Color32::from_rgb(56, 189, 248));
                    draw_compact_card(t2, "2. FLUXWAVE", "Track 02", 85.0, true, Color32::from_rgb(56, 189, 248));
                    draw_compact_card(t3, "3. TORIKO", "Track 03", 85.0, true, Color32::from_rgb(56, 189, 248));
                    draw_compact_card(t4, "4. OUTRO", "Track 04", 85.0, true, Color32::from_rgb(56, 189, 248));

                    painter.line_segment([Pos2::new(t1.x + 42.5, node_y), Pos2::new(t2.x - 42.5, node_y)], Stroke::new(1.8, Color32::from_rgb(56, 189, 248)));
                    painter.line_segment([Pos2::new(t2.x + 42.5, node_y), Pos2::new(t3.x - 42.5, node_y)], Stroke::new(1.8, Color32::from_rgb(56, 189, 248)));
                    painter.line_segment([Pos2::new(t3.x + 42.5, node_y), Pos2::new(t4.x - 42.5, node_y)], Stroke::new(1.8, Color32::from_rgb(56, 189, 248)));

                    let note_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.max.y - 16.0), Vec2::new(420.0, 18.0));
                    painter.text(note_rect.center(), egui::Align2::CENTER_CENTER, "▶ Mode Runtut (Linear): Lagu diputar urut 1 → 2 → 3 → 4 tanpa loncatan", egui::FontId::proportional(9.0), Color32::from_rgb(148, 163, 184));
                } else {
                    // Shuffled Mode: Random Cross Wires in Emerald
                    draw_compact_card(t1, "1. INTERNET", "Sedang Putar", 85.0, true, Color32::from_rgb(52, 211, 153));
                    draw_compact_card(t2, "3. TORIKO", "Lompat Acak", 85.0, true, Color32::from_rgb(52, 211, 153));
                    draw_compact_card(t3, "4. OUTRO", "Lompat Acak", 85.0, true, Color32::from_rgb(52, 211, 153));
                    draw_compact_card(t4, "2. FLUXWAVE", "Trek Terakhir", 85.0, true, Color32::from_rgb(52, 211, 153));

                    let em_col = Color32::from_rgb(52, 211, 153);
                    draw_flowing_cable(&[Pos2::new(t1.x + 42.5, node_y), Pos2::new(t2.x - 42.5, node_y)], em_col, Color32::WHITE, 1);
                    draw_flowing_cable(&[Pos2::new(t2.x + 42.5, node_y), Pos2::new(t3.x - 42.5, node_y)], em_col, Color32::WHITE, 1);
                    draw_flowing_cable(&[Pos2::new(t3.x + 42.5, node_y), Pos2::new(t4.x - 42.5, node_y)], em_col, Color32::WHITE, 1);

                    let note_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.max.y - 16.0), Vec2::new(420.0, 18.0));
                    painter.text(note_rect.center(), egui::Align2::CENTER_CENTER, "🎲 Mode Acak Aktif: Algoritma Fisher-Yates memilih kandidat acak tanpa duplikasi", egui::FontId::proportional(9.0), Color32::from_rgb(167, 243, 208));
                }

                // Bottom Interactive Toggle Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(340.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, if is_shuf { Color32::from_rgb(18, 38, 28) } else { Color32::from_rgb(18, 25, 42) });
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                let shuf_lbl = if is_shuf {
                    "🔀 Status: [SHUFFLE AKTIF] (Klik untuk Matikan ke Runtut)"
                } else {
                    "🔀 Status: [SHUFFLE MATI] (Klik untuk Mengaktifkan Acak)"
                };
                painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, shuf_lbl, egui::FontId::proportional(10.0), Color32::from_rgb(52, 211, 153));
                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s4_shuffled = !self.theater_s4_shuffled;
                }
            } else if self.theater_turn == 4 {
                // =========================================================================
                // BAGIAN 5: 4 SACRED REPEAT MODES (OFF, ALL, ONE, ALBUM)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(32, 20, 48));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔁 4 MODE REPEAT: Off (Mati) • All (Semua) • One (1 Lagu) • Album (Album Ini)",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(243, 232, 255),
                );

                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 12.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);

                let card_w = 112.0;
                let card_h = 76.0;
                let c_y = scope_rect.center().y;
                let active_mode = self.theater_s4_repeat_mode % 4;

                let repeat_cards = [
                    (0, "✕ MATI", "🔁 (Abu-abu)", "Stop saat habis", Color32::from_rgb(100, 116, 139)),
                    (1, "🔁 SEMUA", "🔁 (Biru)", "Loop antrean penuh", Color32::from_rgb(56, 189, 248)),
                    (2, "🔂 LAGU", "🔂 (Gold 1)", "Ulangi 1 lagu saja", Color32::from_rgb(251, 191, 36)),
                    (3, "💿 ALBUM", "💿 (Vinyl)", "Hanya album ini", Color32::from_rgb(168, 85, 247)),
                ];

                for (idx, title, icon_txt, desc_txt, col) in repeat_cards {
                    let cx = scope_rect.min.x + 22.0 + idx as f32 * (card_w + 14.0) + card_w * 0.5;
                    let r = Rect::from_center_size(Pos2::new(cx, c_y), Vec2::new(card_w, card_h));
                    let is_sel = active_mode == idx;

                    let bg = if is_sel { Color32::from_rgb(24, 20, 42) } else { Color32::from_rgb(14, 18, 28) };
                    painter.rect_filled(r, 6.0, bg);
                    painter.rect_stroke(
                        r,
                        6.0,
                        Stroke::new(if is_sel { 1.8 } else { 1.0 }, if is_sel { col } else { Color32::from_rgb(39, 49, 70) }),
                        egui::StrokeKind::Outside,
                    );

                    painter.text(Pos2::new(cx, r.min.y + 16.0), egui::Align2::CENTER_CENTER, title, egui::FontId::proportional(10.0), if is_sel { Color32::WHITE } else { col });
                    painter.text(Pos2::new(cx, r.min.y + 36.0), egui::Align2::CENTER_CENTER, icon_txt, egui::FontId::proportional(11.0), col);
                    painter.text(Pos2::new(cx, r.max.y - 14.0), egui::Align2::CENTER_CENTER, desc_txt, egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                }

                // Bottom Cycle Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(340.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(24, 20, 42));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                let mode_name = match active_mode {
                    0 => "Off (✕ Mati)",
                    1 => "All (🔁 Ulangi Semua)",
                    2 => "One (🔂 Ulangi 1 Lagu)",
                    _ => "Album (💿 Ulangi Album Ini)",
                };
                let btn_lbl = format!("🔁 Siklus Mode: [{}] (Klik untuk Ganti)", mode_name);
                painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, btn_lbl, egui::FontId::proportional(10.0), Color32::from_rgb(216, 180, 254));
                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s4_repeat_mode = (self.theater_s4_repeat_mode + 1) % 4;
                }
            } else if self.theater_turn == 5 {
                // =========================================================================
                // BAGIAN 6: DRAWER ANTREAN PUTAR (REORDER, ACTIONS & KOSONGKAN)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(18, 42, 32));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📑 DRAWER ANTREAN: Susun Ulang Prioritas, Putar Instan, & Bersihkan Antrean",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(220, 252, 231),
                );

                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 12.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);

                // Header of Drawer
                let header_y = scope_rect.min.y + 12.0;
                painter.text(
                    Pos2::new(scope_rect.min.x + 14.0, header_y),
                    egui::Align2::LEFT_CENTER,
                    "📑 Antrean Putar • 3 Lagu",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(241, 245, 249),
                );
                painter.text(
                    Pos2::new(scope_rect.max.x - 14.0, header_y),
                    egui::Align2::RIGHT_CENTER,
                    "🗑 Kosongkan",
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(248, 113, 113),
                );

                // Row 1: Sedang Diputar (Green active border)
                let r1_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 36.0), Vec2::new(scope_w - 24.0, 24.0));
                painter.rect_filled(r1_rect, 4.0, Color32::from_rgb(18, 38, 28));
                painter.rect_stroke(r1_rect, 4.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(Pos2::new(r1_rect.min.x + 8.0, r1_rect.center().y), egui::Align2::LEFT_CENTER, "▶ 1. INTERNET OVERDOSE • NEEDY GIRL OVERDOSE (Sedang Diputar)", egui::FontId::proportional(9.0), Color32::from_rgb(220, 252, 231));
                painter.text(Pos2::new(r1_rect.max.x - 8.0, r1_rect.center().y), egui::Align2::RIGHT_CENTER, "[ ▲ ] [ ▼ ] [ ✕ ]", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                // Row 2: Next in Queue
                let r2_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 64.0), Vec2::new(scope_w - 24.0, 24.0));
                painter.rect_filled(r2_rect, 4.0, Color32::from_rgb(14, 18, 28));
                painter.rect_stroke(r2_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(39, 49, 70)), egui::StrokeKind::Outside);
                painter.text(Pos2::new(r2_rect.min.x + 8.0, r2_rect.center().y), egui::Align2::LEFT_CENTER, "   2. EEYUH! • Fluxxwave (Antrean Berikutnya)", egui::FontId::proportional(9.0), Color32::from_rgb(203, 213, 225));
                painter.text(Pos2::new(r2_rect.max.x - 8.0, r2_rect.center().y), egui::Align2::RIGHT_CENTER, "[ ▶ ] [ ▲ ] [ ▼ ] [ ✕ ]", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                // Bottom Hotkey Tip inside scope
                painter.text(
                    Pos2::new(center.x, scope_rect.max.y - 12.0),
                    egui::Align2::CENTER_CENTER,
                    "💡 Tips: Klik lagu untuk memilih • Gunakan Ctrl + ↑ / ↓ untuk memindahkan urutan antrean",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(148, 163, 184),
                );

                // Lanjut ke Bagian 7 Action Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 84.0), Vec2::new(340.0, 32.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 8.0, Color32::from_rgb(18, 38, 28));
                painter.rect_stroke(btn_rect, 8.0, Stroke::new(if btn_hov { 1.6 } else { 1.2 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✨ Lanjut ke Bagian 7: Gapless, ReplayGain & FX ▶",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(220, 252, 231),
                );

                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_turn = 6;
                    self.theater_turn_timer = 0.0;
                }
            } else {
                // =========================================================================
                // BAGIAN 7: GAPLESS, REPLAYGAIN, CROSSFADE & STUDIO COMPRESSOR
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(15, 30, 50));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "⚡ ADVANCED PLAYBACK: Gapless 0ms • ReplayGain Loudness • Crossfade • Studio Compressor",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(224, 242, 254),
                );

                let scope_w = 520.0;
                let scope_h = 104.0;
                let scope_center = Pos2::new(center.x, center.y + 12.0);
                let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);

                let col_w = (scope_w - 32.0) * 0.5;
                let left_box = Rect::from_min_size(Pos2::new(scope_rect.min.x + 10.0, scope_rect.min.y + 8.0), Vec2::new(col_w, scope_h - 16.0));
                let right_box = Rect::from_min_size(Pos2::new(scope_rect.center().x + 6.0, scope_rect.min.y + 8.0), Vec2::new(col_w, scope_h - 16.0));

                // Left Box: Gapless & Crossfade Stream Architecture
                painter.rect_filled(left_box, 4.0, Color32::from_rgb(14, 20, 32));
                painter.rect_stroke(left_box, 4.0, Stroke::new(1.0, Color32::from_rgb(38, 52, 78)), egui::StrokeKind::Outside);

                painter.text(
                    Pos2::new(left_box.min.x + 8.0, left_box.min.y + 10.0),
                    egui::Align2::LEFT_CENTER,
                    "⚡ STREAM TRANSISI (GAPLESS / CROSSFADE)",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(56, 189, 248),
                );

                // Diagram of Track A & Track B cross-transition
                let wave_y = left_box.min.y + 36.0;
                let bar_w = left_box.width() - 16.0;
                let start_x = left_box.min.x + 8.0;

                // Track A (fade out curve)
                for i in 0..20 {
                    let frac = i as f32 / 19.0;
                    let x = start_x + frac * (bar_w * 0.6);
                    let amp = (1.0 - frac * 0.8) * 10.0;
                    painter.line_segment(
                        [Pos2::new(x, wave_y - amp), Pos2::new(x, wave_y + amp)],
                        Stroke::new(1.6, Color32::from_rgb(244, 114, 182)),
                    );
                }
                painter.text(
                    Pos2::new(start_x + 4.0, wave_y - 12.0),
                    egui::Align2::LEFT_CENTER,
                    "Trek A (Ending)",
                    egui::FontId::proportional(7.5),
                    Color32::from_rgb(244, 114, 182),
                );

                // Track B (fade in curve)
                for i in 0..20 {
                    let frac = i as f32 / 19.0;
                    let x = start_x + (bar_w * 0.4) + frac * (bar_w * 0.6);
                    let amp = (0.2 + frac * 0.8) * 10.0;
                    painter.line_segment(
                        [Pos2::new(x, wave_y - amp), Pos2::new(x, wave_y + amp)],
                        Stroke::new(1.6, Color32::from_rgb(56, 189, 248)),
                    );
                }
                painter.text(
                    Pos2::new(left_box.max.x - 12.0, wave_y - 12.0),
                    egui::Align2::RIGHT_CENTER,
                    "Trek B (Starting)",
                    egui::FontId::proportional(7.5),
                    Color32::from_rgb(56, 189, 248),
                );

                let cf_text = if self.crossfade_secs > 0.0 {
                    format!("🔀 Crossfade: {:.1}s Overlap Aktif", self.crossfade_secs)
                } else {
                    "⚡ Gapless: 0ms (Sample-Accurate Zero Flush)".to_string()
                };
                painter.text(
                    Pos2::new(left_box.center().x, left_box.max.y - 12.0),
                    egui::Align2::CENTER_CENTER,
                    cf_text,
                    egui::FontId::proportional(8.0),
                    Color32::from_rgb(52, 211, 153),
                );

                // Right Box: ReplayGain & Dynamic Compressor
                painter.rect_filled(right_box, 4.0, Color32::from_rgb(18, 18, 30));
                painter.rect_stroke(right_box, 4.0, Stroke::new(1.0, Color32::from_rgb(55, 45, 80)), egui::StrokeKind::Outside);

                painter.text(
                    Pos2::new(right_box.min.x + 8.0, right_box.min.y + 10.0),
                    egui::Align2::LEFT_CENTER,
                    "🔊 REPLAYGAIN & STUDIO COMPRESSOR",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(168, 85, 247),
                );

                let rg_desc = match self.replay_gain_mode {
                    crate::models::ReplayGainMode::Auto => "Mode: Auto (Smart Track/Album)",
                    crate::models::ReplayGainMode::Track => "Mode: Track Gain",
                    crate::models::ReplayGainMode::Album => "Mode: Album Gain",
                    crate::models::ReplayGainMode::Disabled => "Mode: Disabled (0.0 dB)",
                };
                painter.text(
                    Pos2::new(right_box.min.x + 8.0, right_box.min.y + 26.0),
                    egui::Align2::LEFT_CENTER,
                    rg_desc,
                    egui::FontId::proportional(8.0),
                    Color32::from_rgb(224, 231, 255),
                );

                let eff_text = format!("Target: {:+.1} dB • Headroom: Safe (≤ 0 dBFS)", self.effective_replay_gain_db);
                painter.text(
                    Pos2::new(right_box.min.x + 8.0, right_box.min.y + 40.0),
                    egui::Align2::LEFT_CENTER,
                    eff_text,
                    egui::FontId::proportional(8.0),
                    Color32::from_rgb(251, 191, 36),
                );

                // Compressor VU Meter Simulation
                let vu_bar_rect = Rect::from_min_size(Pos2::new(right_box.min.x + 8.0, right_box.min.y + 54.0), Vec2::new(right_box.width() - 16.0, 10.0));
                painter.rect_filled(vu_bar_rect, 2.0, Color32::from_rgb(10, 10, 18));
                let gr_w = (vu_bar_rect.width() * 0.42).min(vu_bar_rect.width());
                let gr_fill = Rect::from_min_size(vu_bar_rect.min, Vec2::new(gr_w, 10.0));
                painter.rect_filled(gr_fill, 2.0, Color32::from_rgb(168, 85, 247));
                painter.text(
                    Pos2::new(vu_bar_rect.center().x, vu_bar_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    "Compressor GR: -3.2 dB (Ratio 4:1 • Attack 10ms)",
                    egui::FontId::proportional(7.5),
                    Color32::WHITE,
                );

                painter.text(
                    Pos2::new(right_box.center().x, right_box.max.y - 8.0),
                    egui::Align2::CENTER_CENTER,
                    "Anti-Clip Limiter: Aktif • Headroom 300 dB Murni",
                    egui::FontId::proportional(7.5),
                    Color32::from_rgb(148, 163, 184),
                );

                // Lanjut ke Slide 5 Action Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 84.0), Vec2::new(340.0, 32.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 8.0, Color32::from_rgb(22, 48, 40));
                painter.rect_stroke(btn_rect, 8.0, Stroke::new(if btn_hov { 1.6 } else { 1.2 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✨ Lanjut ke Slide 5: Music Library ▶",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(220, 252, 231),
                );

                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_slide = 4;
                    self.theater_turn = 0;
                    self.theater_turn_timer = 0.0;
                }
            }
        } else {
            // --- SLIDE 5 DIAGRAM: MUSIC LIBRARY & KOLEKSI LOKAL (6 BAGIAN) ---
            let scope_w = 520.0;
            let scope_h = 104.0;
            let scope_center = Pos2::new(center.x, center.y + 12.0);
            let scope_rect = Rect::from_center_size(scope_center, Vec2::new(scope_w, scope_h));

            if self.theater_turn == 0 {
                // =========================================================================
                // BAGIAN 1: THE LOCAL FIDELITY PHILOSOPHY (SCAN FOLDER & PRIVASI OFFLINE)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(38, 30, 16));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(251, 191, 36)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📁 THE LOCAL FIDELITY PHILOSOPHY: 100% Offline Lokal • Privasi Terjaga • Tanpa Cloud",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(254, 243, 199),
                );

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(251, 191, 36)), egui::StrokeKind::Outside);

                // Header inside mockup scope
                let header_y = scope_rect.min.y + 14.0;
                painter.text(
                    Pos2::new(scope_rect.min.x + 14.0, header_y),
                    egui::Align2::LEFT_CENTER,
                    "📁 MUSIC LIBRARY • 24 Lagu Terdaftar",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(241, 245, 249),
                );

                // Mock Buttons on right header
                let btn_add_file = Rect::from_min_size(Pos2::new(scope_rect.max.x - 220.0, header_y - 9.0), Vec2::new(90.0, 18.0));
                painter.rect_filled(btn_add_file, 4.0, Color32::from_rgb(26, 32, 46));
                painter.rect_stroke(btn_add_file, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                painter.text(btn_add_file.center(), egui::Align2::CENTER_CENTER, "📄 Tambah File", egui::FontId::proportional(8.5), Color32::from_rgb(203, 213, 225));

                let btn_scan = Rect::from_min_size(Pos2::new(scope_rect.max.x - 124.0, header_y - 9.0), Vec2::new(110.0, 18.0));
                painter.rect_filled(btn_scan, 4.0, Color32::from_rgb(49, 46, 129));
                painter.rect_stroke(btn_scan, 4.0, Stroke::new(1.0, Color32::from_rgb(129, 140, 248)), egui::StrokeKind::Outside);
                painter.text(btn_scan.center(), egui::Align2::CENTER_CENTER, "➕ Scan Folder Musik", egui::FontId::proportional(8.5), Color32::from_rgb(224, 231, 255));

                // Middle Scanning Status Banner
                let status_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 45.0), Vec2::new(scope_w - 28.0, 26.0));
                if self.theater_s5_simulated_scan {
                    painter.rect_filled(status_rect, 4.0, Color32::from_rgb(18, 42, 32));
                    painter.rect_stroke(status_rect, 4.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                    painter.text(
                        Pos2::new(status_rect.min.x + 10.0, status_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "⏳ Memindai D:\\Music\\Audiophile Master... 128 lagu terdeteksi (Selesai ✓)",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(220, 252, 231),
                    );
                    painter.text(
                        Pos2::new(status_rect.max.x - 10.0, status_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        "Bitstream 1:1 Preserved",
                        egui::FontId::proportional(8.5),
                        Color32::from_rgb(52, 211, 153),
                    );
                } else {
                    painter.rect_filled(status_rect, 4.0, Color32::from_rgb(14, 18, 28));
                    painter.rect_stroke(status_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                    painter.text(
                        Pos2::new(status_rect.min.x + 10.0, status_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "📂 Direktori Terhubung: D:\\Music\\Audiophile FLAC Master (24 Track Aktif)",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(203, 213, 225),
                    );
                    painter.text(
                        Pos2::new(status_rect.max.x - 10.0, status_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        "Ready",
                        egui::FontId::proportional(8.5),
                        Color32::from_rgb(100, 116, 139),
                    );
                }

                // Supported Formats Pills
                let format_pills = [
                    ("FLAC Lossless", Color32::from_rgb(56, 189, 248)),
                    ("WAV 32-bit Float", Color32::from_rgb(52, 211, 153)),
                    ("ALAC Apple", Color32::from_rgb(168, 85, 247)),
                    ("MP3 High-VBR", Color32::from_rgb(251, 191, 36)),
                    ("AIFF Uncompressed", Color32::from_rgb(244, 114, 182)),
                ];
                let pill_y = scope_rect.max.y - 14.0;
                let pill_w = 92.0;
                let pill_start_x = scope_rect.min.x + 16.0;
                for (i, (fmt_name, fmt_col)) in format_pills.iter().enumerate() {
                    let pill_rect = Rect::from_center_size(
                        Pos2::new(pill_start_x + i as f32 * (pill_w + 6.0) + pill_w * 0.5, pill_y),
                        Vec2::new(pill_w, 16.0),
                    );
                    painter.rect_filled(pill_rect, 3.0, Color32::from_rgb(18, 24, 38));
                    painter.rect_stroke(pill_rect, 3.0, Stroke::new(0.8, *fmt_col), egui::StrokeKind::Outside);
                    painter.text(pill_rect.center(), egui::Align2::CENTER_CENTER, *fmt_name, egui::FontId::proportional(8.0), *fmt_col);
                }

                // Bottom Interactive Toggle Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(340.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, if self.theater_s5_simulated_scan { Color32::from_rgb(18, 38, 28) } else { Color32::from_rgb(18, 25, 42) });
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(251, 191, 36)), egui::StrokeKind::Outside);
                let scan_lbl = if self.theater_s5_simulated_scan {
                    "⚡ Status: [PEMINDAIAN SELESAI] (Klik untuk Uji Pindai Ulang)"
                } else {
                    "➕ Uji Interaktif: [SIMULASIKAN SCAN FOLDER MUSIK]"
                };
                painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, scan_lbl, egui::FontId::proportional(10.0), Color32::from_rgb(251, 191, 36));
                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s5_simulated_scan = !self.theater_s5_simulated_scan;
                }
            } else if self.theater_turn == 1 {
                // =========================================================================
                // BAGIAN 2: THE TELEMETRY RIBBON (5 KARTU STATISTIK KOLEKSI)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(18, 32, 54));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📊 5 PITA STATISTIK TELEMETRI: Total Lagu • Album • Artis • Storage • Hi-Res Master",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(224, 242, 254),
                );

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);

                // 5 Telemetry Stat Cards
                let card_w = 94.0;
                let card_h = 72.0;
                let card_y = scope_rect.center().y - 4.0;
                let cards_total_w = 5.0 * card_w + 4.0 * 8.0;
                let start_x = scope_rect.min.x + (scope_w - cards_total_w) * 0.5 + card_w * 0.5;

                let stats_data = [
                    ("🎵 TOTAL LAGU", "24 Lagu", "Koleksi Aktif", Color32::from_rgb(203, 213, 225), false),
                    ("💿 ALBUM", "19 Album", "Diskografi", Color32::from_rgb(192, 132, 252), false),
                    ("🎙️ ARTIS", "14 Artis", "Penyanyi / Komposer", Color32::from_rgb(56, 189, 248), false),
                    ("💾 STORAGE", "708.0 MB", "Ukuran File Audio", Color32::from_rgb(56, 189, 248), false),
                    ("✨ HI-RES MASTER", "2 Track", "≥24-bit / 96kHz", Color32::from_rgb(52, 211, 153), true),
                ];

                for (idx, (cat_lbl, val_lbl, sub_lbl, accent_col, is_special)) in stats_data.iter().enumerate() {
                    let cx = start_x + idx as f32 * (card_w + 8.0);
                    let card_r = Rect::from_center_size(Pos2::new(cx, card_y), Vec2::new(card_w, card_h));

                    let bg = if *is_special { Color32::from_rgb(18, 38, 28) } else { Color32::from_rgb(14, 18, 28) };
                    painter.rect_filled(card_r, 6.0, bg);
                    painter.rect_stroke(
                        card_r,
                        6.0,
                        Stroke::new(if *is_special { 1.6 } else { 1.0 }, if *is_special { *accent_col } else { Color32::from_rgb(39, 49, 70) }),
                        egui::StrokeKind::Outside,
                    );

                    painter.text(Pos2::new(cx, card_r.min.y + 14.0), egui::Align2::CENTER_CENTER, *cat_lbl, egui::FontId::proportional(8.5), *accent_col);
                    painter.text(Pos2::new(cx, card_r.min.y + 34.0), egui::Align2::CENTER_CENTER, *val_lbl, egui::FontId::proportional(12.0), Color32::WHITE);
                    painter.text(Pos2::new(cx, card_r.max.y - 12.0), egui::Align2::CENTER_CENTER, *sub_lbl, egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                }

                // Subtitle inside scope
                painter.text(
                    Pos2::new(center.x, scope_rect.max.y - 12.0),
                    egui::Align2::CENTER_CENTER,
                    "💡 Dihitung instan dari metadata berkas lokal tanpa overhead database eksternal",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(148, 163, 184),
                );

                // Bottom Interactive Info Pill
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(340.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(18, 38, 28));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✨ Kartu Spesial: [HI-RES MASTER DETECTED] (2 Track ≥24-bit / 96kHz)",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(52, 211, 153),
                );
            } else if self.theater_turn == 2 {
                // =========================================================================
                // BAGIAN 3: TABEL SEMUA LAGU, KLIK LANGSUNG (PLAY) & LOVE KECIL (FAVORIT)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(32, 20, 48));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🗂️ TABEL SEMUA LAGU: Klik Langsung (Play ▶) • Love Kecil (Favorit 💜) • Klik Kanan (Menu)",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(243, 232, 255),
                );

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);

                // Table Header Row
                let th_y = scope_rect.min.y + 12.0;
                painter.text(Pos2::new(scope_rect.min.x + 14.0, th_y), egui::Align2::LEFT_CENTER, "#   ART   FAV   JUDUL & ARTIS", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                painter.text(Pos2::new(scope_rect.min.x + 230.0, th_y), egui::Align2::LEFT_CENTER, "ALBUM", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                painter.text(Pos2::new(scope_rect.min.x + 355.0, th_y), egui::Align2::LEFT_CENTER, "FORMAT / RESOLUSI", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));
                painter.text(Pos2::new(scope_rect.max.x - 14.0, th_y), egui::Align2::RIGHT_CENTER, "DURASI", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));

                // Row 1: Sedang Diputar (Mint green highlight #E6F9F0 background as in real app)
                let r1_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 34.0), Vec2::new(scope_w - 20.0, 26.0));
                painter.rect_filled(r1_rect, 4.0, Color32::from_rgb(230, 249, 240));
                painter.rect_stroke(r1_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(167, 243, 208)), egui::StrokeKind::Outside);

                // # Play icon
                let play_badge = Rect::from_center_size(Pos2::new(r1_rect.min.x + 14.0, r1_rect.center().y), Vec2::new(16.0, 16.0));
                painter.rect_filled(play_badge, 3.0, Color32::from_rgb(224, 242, 254));
                painter.text(play_badge.center(), egui::Align2::CENTER_CENTER, "▶", egui::FontId::proportional(8.5), Color32::from_rgb(2, 132, 199));

                // ART thumbnail
                let art_rect1 = Rect::from_center_size(Pos2::new(r1_rect.min.x + 36.0, r1_rect.center().y), Vec2::new(16.0, 16.0));
                painter.rect_filled(art_rect1, 2.0, Color32::from_rgb(30, 41, 59));
                painter.rect_stroke(art_rect1, 2.0, Stroke::new(0.8, Color32::WHITE), egui::StrokeKind::Outside);
                painter.text(art_rect1.center(), egui::Align2::CENTER_CENTER, "🎵", egui::FontId::proportional(7.5), Color32::WHITE);

                // FAV: Little Love Icon 💜
                let fav_rect1 = Rect::from_center_size(Pos2::new(r1_rect.min.x + 58.0, r1_rect.center().y), Vec2::new(16.0, 16.0));
                painter.rect_filled(fav_rect1, 3.0, Color32::from_rgb(243, 232, 255));
                painter.rect_stroke(fav_rect1, 3.0, Stroke::new(0.8, Color32::from_rgb(192, 132, 252)), egui::StrokeKind::Outside);
                painter.text(fav_rect1.center(), egui::Align2::CENTER_CENTER, "💜", egui::FontId::proportional(8.5), Color32::from_rgb(147, 51, 234));

                // Title & Artist
                painter.text(Pos2::new(r1_rect.min.x + 76.0, r1_rect.center().y - 4.5), egui::Align2::LEFT_CENTER, "INTERNET OVERDOSE", egui::FontId::proportional(9.0), Color32::from_rgb(5, 150, 105));
                painter.text(Pos2::new(r1_rect.min.x + 76.0, r1_rect.center().y + 6.0), egui::Align2::LEFT_CENTER, "NEEDY GIRL OVERDOSE; Kotoko +81", egui::FontId::proportional(7.5), Color32::from_rgb(71, 85, 105));

                // Album
                painter.text(Pos2::new(scope_rect.min.x + 230.0, r1_rect.center().y), egui::Align2::LEFT_CENTER, "INTERNET OVERDOSE", egui::FontId::proportional(8.0), Color32::from_rgb(51, 65, 85));

                // Format badge
                let fmt_badge1 = Rect::from_center_size(Pos2::new(scope_rect.min.x + 395.0, r1_rect.center().y), Vec2::new(76.0, 16.0));
                painter.rect_filled(fmt_badge1, 3.0, Color32::from_rgb(239, 246, 255));
                painter.rect_stroke(fmt_badge1, 3.0, Stroke::new(0.8, Color32::from_rgb(191, 219, 254)), egui::StrokeKind::Outside);
                painter.text(fmt_badge1.center(), egui::Align2::CENTER_CENTER, "FLAC 44.1k/16b", egui::FontId::proportional(8.0), Color32::from_rgb(29, 78, 216));

                // Duration
                painter.text(Pos2::new(r1_rect.max.x - 8.0, r1_rect.center().y), egui::Align2::RIGHT_CENTER, "03:40", egui::FontId::proportional(8.5), Color32::from_rgb(30, 41, 59));

                // Row 2: Inactive Row (Dark Slate)
                let r2_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 64.0), Vec2::new(scope_w - 20.0, 26.0));
                painter.rect_filled(r2_rect, 4.0, Color32::from_rgb(14, 18, 28));
                painter.rect_stroke(r2_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(39, 49, 70)), egui::StrokeKind::Outside);

                // # 02
                painter.text(Pos2::new(r2_rect.min.x + 14.0, r2_rect.center().y), egui::Align2::CENTER_CENTER, "02", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                // ART thumbnail
                let art_rect2 = Rect::from_center_size(Pos2::new(r2_rect.min.x + 36.0, r2_rect.center().y), Vec2::new(16.0, 16.0));
                painter.rect_filled(art_rect2, 2.0, Color32::from_rgb(26, 32, 46));
                painter.rect_stroke(art_rect2, 2.0, Stroke::new(0.8, Color32::from_rgb(71, 85, 105)), egui::StrokeKind::Outside);
                painter.text(art_rect2.center(), egui::Align2::CENTER_CENTER, "🎵", egui::FontId::proportional(7.5), Color32::from_rgb(148, 163, 184));

                // FAV: Little Love Icon 🤍 (Unselected)
                let fav_rect2 = Rect::from_center_size(Pos2::new(r2_rect.min.x + 58.0, r2_rect.center().y), Vec2::new(16.0, 16.0));
                painter.rect_filled(fav_rect2, 3.0, Color32::from_rgb(26, 32, 46));
                painter.rect_stroke(fav_rect2, 3.0, Stroke::new(0.8, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                painter.text(fav_rect2.center(), egui::Align2::CENTER_CENTER, "🤍", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                // Title & Artist
                painter.text(Pos2::new(r2_rect.min.x + 76.0, r2_rect.center().y - 4.5), egui::Align2::LEFT_CENTER, "INTERNET YAMERO", egui::FontId::proportional(9.0), Color32::from_rgb(241, 245, 249));
                painter.text(Pos2::new(r2_rect.min.x + 76.0, r2_rect.center().y + 6.0), egui::Align2::LEFT_CENTER, "NEEDY GIRL OVERDOSE; Kotoko +81", egui::FontId::proportional(7.5), Color32::from_rgb(148, 163, 184));

                // Album
                painter.text(Pos2::new(scope_rect.min.x + 230.0, r2_rect.center().y), egui::Align2::LEFT_CENTER, "INTERNET YAMERO", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));

                // Format badge
                let fmt_badge2 = Rect::from_center_size(Pos2::new(scope_rect.min.x + 395.0, r2_rect.center().y), Vec2::new(76.0, 16.0));
                painter.rect_filled(fmt_badge2, 3.0, Color32::from_rgb(26, 32, 46));
                painter.rect_stroke(fmt_badge2, 3.0, Stroke::new(0.8, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                painter.text(fmt_badge2.center(), egui::Align2::CENTER_CENTER, "FLAC 44.1k/16b", egui::FontId::proportional(8.0), Color32::from_rgb(148, 163, 184));

                // Duration
                painter.text(Pos2::new(r2_rect.max.x - 8.0, r2_rect.center().y), egui::Align2::RIGHT_CENTER, "04:01", egui::FontId::proportional(8.5), Color32::from_rgb(148, 163, 184));

                // Interactive Guidance Cue inside scope
                painter.text(
                    Pos2::new(center.x, scope_rect.max.y - 10.0),
                    egui::Align2::CENTER_CENTER,
                    "💡 KLIK LANGSUNG = Putar Instan (Hijau ▶)  •  💜 LOVE KECIL = Favorit Instan  •  🖱️ KLIK KANAN = Menu Aksi",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(148, 163, 184),
                );

                // Bottom Interactive Test Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(360.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(24, 20, 42));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "👆 Interaksi Intuitif: [KLIK BARIS = PLAY ▶] • [LOVE KECIL = FAVORIT 💜]",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(216, 180, 254),
                );
                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s5_category = (self.theater_s5_category + 1) % 6;
                }
            } else if self.theater_turn == 3 {
                // =========================================================================
                // BAGIAN 4: INSTANT FILTERING (LIVE SEARCH & ACTIVE FILTER CHIP)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(18, 32, 54));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔍 INSTANT FILTERING: Pencarian Real-Time & Kunci Filter Album/Artis dengan Reset Cepat",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(224, 242, 254),
                );

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);

                // Top Search Row & Filter Chip
                let search_box = Rect::from_min_size(Pos2::new(scope_rect.min.x + 14.0, scope_rect.min.y + 12.0), Vec2::new(260.0, 22.0));
                painter.rect_filled(search_box, 4.0, Color32::from_rgb(14, 18, 28));
                painter.rect_stroke(search_box, 4.0, Stroke::new(1.0, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                let cursor_blink = if ((t * 2.5) as usize) % 2 == 0 { "|" } else { "" };
                let search_txt = format!("🔍 Cari: \"NEEDY\"{}", cursor_blink);
                painter.text(Pos2::new(search_box.min.x + 8.0, search_box.center().y), egui::Align2::LEFT_CENTER, search_txt, egui::FontId::proportional(9.0), Color32::from_rgb(56, 189, 248));

                // Active Filter Chip
                let chip_box = Rect::from_min_size(Pos2::new(scope_rect.min.x + 284.0, scope_rect.min.y + 12.0), Vec2::new(222.0, 22.0));
                painter.rect_filled(chip_box, 4.0, Color32::from_rgb(49, 46, 129));
                painter.rect_stroke(chip_box, 4.0, Stroke::new(1.2, Color32::from_rgb(129, 140, 248)), egui::StrokeKind::Outside);
                painter.text(
                    Pos2::new(chip_box.min.x + 8.0, chip_box.center().y),
                    egui::Align2::LEFT_CENTER,
                    "🔍 Filter: Album \"NEEDY GIRL...\"",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(224, 231, 255),
                );
                painter.text(
                    Pos2::new(chip_box.max.x - 8.0, chip_box.center().y),
                    egui::Align2::RIGHT_CENTER,
                    "✕ Reset",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(248, 113, 113),
                );

                // Filtered Result Row
                let result_row = Rect::from_center_size(Pos2::new(center.x, scope_rect.center().y + 14.0), Vec2::new(scope_w - 28.0, 26.0));
                painter.rect_filled(result_row, 4.0, Color32::from_rgb(18, 38, 28));
                painter.rect_stroke(result_row, 4.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    Pos2::new(result_row.min.x + 8.0, result_row.center().y),
                    egui::Align2::LEFT_CENTER,
                    "▶ 1. INTERNET OVERDOSE • Aiobahn feat. Kotoko • NEEDY GIRL OVERDOSE",
                    egui::FontId::proportional(9.0),
                    Color32::from_rgb(220, 252, 231),
                );
                painter.text(
                    Pos2::new(result_row.max.x - 8.0, result_row.center().y),
                    egui::Align2::RIGHT_CENTER,
                    "FLAC 44.1k • 04:12",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(52, 211, 153),
                );

                // Subtitle cue inside scope
                painter.text(
                    Pos2::new(center.x, scope_rect.max.y - 12.0),
                    egui::Align2::CENTER_CENTER,
                    "💡 Menampilkan 1 dari 24 lagu • Filter instan menyaring simultan pada judul, artis, & album",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(148, 163, 184),
                );

                // Bottom Interactive Test Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(340.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(18, 32, 54));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🔄 Uji Reset Filter: [✕ Reset & Tampilkan Seluruh 24 Lagu]",
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(224, 242, 254),
                );
            } else if self.theater_turn == 4 {
                // =========================================================================
                // BAGIAN 5: RAHASIA MENU KLIK KANAN (CONTEXT MENU 7 AKSI LENGKAP)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(18, 42, 32));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🖱️ KLIK KANAN PADA BARIS: Buka Menu Konteks Studio Melayang (7 Aksi Lengkap)",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(220, 252, 231),
                );

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);

                // Background Ghosted Track Row at Top of Scope
                let bg_row = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 16.0), Vec2::new(scope_w - 24.0, 22.0));
                painter.rect_filled(bg_row, 3.0, Color32::from_rgb(18, 25, 42));
                painter.rect_stroke(bg_row, 3.0, Stroke::new(0.8, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                painter.text(
                    Pos2::new(bg_row.min.x + 8.0, bg_row.center().y),
                    egui::Align2::LEFT_CENTER,
                    "▶ 1. 🎵 💜 INTERNET OVERDOSE • Kotoko; Aiobahn (Klik Kanan Aktif...)",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(148, 163, 184),
                );

                // Floating Context Menu Card (Matching the user screenshot!)
                let menu_w = 480.0;
                let menu_h = 62.0;
                let menu_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.center().y + 14.0), Vec2::new(menu_w, menu_h));
                painter.rect_filled(menu_rect, 6.0, Color32::from_rgb(255, 255, 255));
                painter.rect_stroke(menu_rect, 6.0, Stroke::new(1.4, Color32::from_rgb(148, 163, 184)), egui::StrokeKind::Outside);

                // Header inside floating menu
                painter.text(
                    Pos2::new(menu_rect.min.x + 10.0, menu_rect.min.y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    "INTERNET OVERDOSE • NEEDY GIRL OVERDOSE",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(15, 23, 42),
                );
                painter.line_segment(
                    [Pos2::new(menu_rect.min.x + 8.0, menu_rect.min.y + 18.0), Pos2::new(menu_rect.max.x - 8.0, menu_rect.min.y + 18.0)],
                    Stroke::new(0.8, Color32::from_rgb(226, 232, 240)),
                );

                let act_idx = self.theater_s5_selected_action % 7;
                let context_items = [
                    (0, "▶ Putar Sekarang", "Memutar lagu seketika langsung ke DAC"),
                    (1, "⏳ Putar Berikutnya", "Sisipkan lagu tepat setelah lagu aktif"),
                    (2, "➕ Tambah ke Antrean", "Menambahkan ke urutan terakhir antrean"),
                    (3, "💖 Tambah ke Favorit", "Menyimpan ke daftar koleksi favorit"),
                    (4, "🎶 Tambah ke Playlist", "Menyelipkan ke playlist mixtape custom"),
                    (5, "🗑 Hapus dari Library", "Hapus dari daftar tanpa sentuh file disk"),
                    (6, "📂 Buka di Explorer", "Buka Windows Explorer & sorot file asli"),
                ];

                // Render items in 2 neat rows inside floating menu
                let item_w = 114.0;
                let item_h = 17.0;
                for (idx, (item_id, item_label, _)) in context_items.iter().enumerate() {
                    let col = idx % 4;
                    let row = idx / 4;
                    let ix = menu_rect.min.x + 8.0 + col as f32 * (item_w + 3.0);
                    let iy = menu_rect.min.y + 22.0 + row as f32 * (item_h + 3.0);
                    let item_r = Rect::from_min_size(Pos2::new(ix, iy), Vec2::new(item_w, item_h));

                    let is_sel = *item_id == act_idx;
                    if is_sel {
                        painter.rect_filled(item_r, 3.0, Color32::from_rgb(220, 252, 231));
                        painter.rect_stroke(item_r, 3.0, Stroke::new(1.0, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                    }
                    painter.text(
                        Pos2::new(item_r.min.x + 4.0, item_r.center().y),
                        egui::Align2::LEFT_CENTER,
                        *item_label,
                        egui::FontId::proportional(8.0),
                        if is_sel { Color32::from_rgb(5, 150, 105) } else { Color32::from_rgb(51, 65, 85) },
                    );
                }

                // Bottom Cycle Action Button
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 88.0), Vec2::new(360.0, 30.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 6.0, Color32::from_rgb(18, 38, 28));
                painter.rect_stroke(btn_rect, 6.0, Stroke::new(if btn_hov { 1.5 } else { 1.0 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                let btn_lbl = format!("🎯 Sorot Opsi Klik Kanan: [{}] (Klik untuk Pelajari)", context_items[act_idx].1);
                painter.text(btn_rect.center(), egui::Align2::CENTER_CENTER, btn_lbl, egui::FontId::proportional(10.0), Color32::from_rgb(220, 252, 231));
                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.theater_s5_selected_action = (self.theater_s5_selected_action + 1) % 7;
                }
            } else {
                // =========================================================================
                // BAGIAN 6: CUSTOM PLAYLISTS & KEAMANAN BERKAS LOKAL (SELESAI TUTORIAL)
                // =========================================================================
                let badge_rect = Rect::from_center_size(Pos2::new(center.x, center.y - 68.0), Vec2::new(510.0, 24.0));
                painter.rect_filled(badge_rect, 6.0, Color32::from_rgb(32, 20, 48));
                painter.rect_stroke(badge_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📑 CUSTOM PLAYLISTS: Racik Mixtape Audiophile, Bulk Queue, & Garansi Aman Berkas Fisik",
                    egui::FontId::proportional(9.5),
                    Color32::from_rgb(243, 232, 255),
                );

                painter.rect_filled(scope_rect, 6.0, Color32::from_rgb(9, 13, 23));
                painter.rect_stroke(scope_rect, 6.0, Stroke::new(1.2, Color32::from_rgb(168, 85, 247)), egui::StrokeKind::Outside);

                // Playlists Cards Mockup
                let p1_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 22.0), Vec2::new(scope_w - 24.0, 24.0));
                painter.rect_filled(p1_rect, 4.0, Color32::from_rgb(18, 25, 42));
                painter.rect_stroke(p1_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                painter.text(
                    Pos2::new(p1_rect.min.x + 8.0, p1_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "🎶 Late Night Audiophile — Koleksi santai resolusi tinggi malam hari",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(241, 245, 249),
                );
                painter.text(
                    Pos2::new(p1_rect.max.x - 8.0, p1_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    "[ ▶ Putar Semua ]",
                    egui::FontId::proportional(8.0),
                    Color32::from_rgb(129, 140, 248),
                );

                let p2_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.min.y + 50.0), Vec2::new(scope_w - 24.0, 24.0));
                painter.rect_filled(p2_rect, 4.0, Color32::from_rgb(18, 25, 42));
                painter.rect_stroke(p2_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(45, 55, 78)), egui::StrokeKind::Outside);
                painter.text(
                    Pos2::new(p2_rect.min.x + 8.0, p2_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "🎶 Hi-Res Reference Master — Uji dynamic range, staging, & transparansi",
                    egui::FontId::proportional(8.5),
                    Color32::from_rgb(241, 245, 249),
                );
                painter.text(
                    Pos2::new(p2_rect.max.x - 8.0, p2_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    "[ ▶ Putar Semua ]",
                    egui::FontId::proportional(8.0),
                    Color32::from_rgb(129, 140, 248),
                );

                // Safety Guarantee Cue
                let safe_rect = Rect::from_center_size(Pos2::new(center.x, scope_rect.max.y - 15.0), Vec2::new(scope_w - 24.0, 20.0));
                painter.rect_filled(safe_rect, 3.0, Color32::from_rgb(20, 36, 30));
                painter.rect_stroke(safe_rect, 3.0, Stroke::new(0.8, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    safe_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🛡️ Garansi Keamanan: Tombol [🗑️ Kosongkan Library] TIDAK PERNAH menghapus file asli di hard disk!",
                    egui::FontId::proportional(8.0),
                    Color32::from_rgb(220, 252, 231),
                );

                // Big Launch Button: Finish Tutorial & Open Signal Chain Studio
                let btn_rect = Rect::from_center_size(Pos2::new(center.x, center.y + 84.0), Vec2::new(340.0, 32.0));
                let btn_hov = ui.rect_contains_pointer(btn_rect);
                painter.rect_filled(btn_rect, 8.0, Color32::from_rgb(22, 48, 40));
                painter.rect_stroke(btn_rect, 8.0, Stroke::new(if btn_hov { 1.6 } else { 1.2 }, Color32::from_rgb(52, 211, 153)), egui::StrokeKind::Outside);
                painter.text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🚀 Selesai & Buka Signal Chain Studio Sekarang",
                    egui::FontId::proportional(11.0),
                    Color32::from_rgb(220, 252, 231),
                );

                if ui.input(|i| i.pointer.any_pressed()) && btn_hov {
                    self.show_interactive_theater = false;
                }
            }
        }
    }
}
