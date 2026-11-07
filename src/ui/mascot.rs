use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

/// Expressive states for the Lunula Beamed Pair ("Lu & Nu") Mascot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotExpression {
    Happy,
    Audiophile,
    ShockedShield,
    CozyWarmth,
    BeatDance,
    WaveHello,
}

impl Default for MascotExpression {
    fn default() -> Self {
        MascotExpression::Happy
    }
}

impl MascotExpression {
    pub fn title(&self) -> &'static str {
        match self {
            MascotExpression::Happy => "Ceria & Santai (Idle)",
            MascotExpression::Audiophile => "Smart Audiophile (Bit-Perfect)",
            MascotExpression::ShockedShield => "Kaget & Shield (Auto-Mute)",
            MascotExpression::CozyWarmth => "Hangat & Empuk (Tube 12AX7)",
            MascotExpression::BeatDance => "Joget Irama (Beat Dance)",
            MascotExpression::WaveHello => "Menyapa Ceria (Wave Hello)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MascotExpression::Happy => {
                "Duo not balok hitam elegan dengan outline putih bersih. Lu melayang santai bersenandung (♪) dengan mata terpejam riang, sementara Nu berkedip ramah menyapa pengguna."
            }
            MascotExpression::Audiophile => {
                "Lu memakai kacamata monocle emas berkilau dan memegang kabel patch audiophile 1:1. Nu memandang takjub dengan mata bintang (★) ke gelombang sinus murni di antara mereka."
            }
            MascotExpression::ShockedShield => {
                "Kabel studio mendadak putus dengan percikan petir! Lu & Nu kaget bergetar dengan mata melotot dan keringat kartun, lalu tameng kubah Auto-Mute 🛡️ seketika aktif menangkal letupan berbahaya."
            }
            MascotExpression::CozyWarmth => {
                "Tabung vakum 12AX7 aktif! Keduanya saling merapat empuk, pipi merona hangat, mata terpejam puas menikmati kehangatan analog diiringi kepulan uap lembut (♨)."
            }
            MascotExpression::BeatDance => {
                "Joget irama 60 FPS! Lu & Nu melompat bergantian dengan efek squash & stretch elastis, tangan bersarung putih memompa irama, dan spektrum equalizer neon berdenyut riang."
            }
            MascotExpression::WaveHello => {
                "Nu mengangkat tangan bersarung kartun putih melambai riang menyambut pengguna baru, sementara Lu mengedipkan mata dengan balon sapaan selamat datang di Lunula Studio!"
            }
        }
    }
}

/// Helper to draw a cartoon white studio glove
fn draw_cartoon_glove(
    painter: &egui::Painter,
    pos: Pos2,
    scale: f32,
    angle_rad: f32,
    is_waving: bool,
    is_pointing: bool,
) {
    let palm_r = 7.0 * scale;
    // Palm background (White with black outline)
    painter.circle_filled(pos, palm_r, Color32::WHITE);
    painter.circle_stroke(pos, palm_r, Stroke::new(1.8 * scale, Color32::from_rgb(20, 20, 25)));

    // Three classic cartoon glove darts/creases on the back of hand
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    for i in -1..=1 {
        let offset_x = (i as f32 * 2.2) * scale;
        let p_start = Pos2::new(
            pos.x + offset_x * cos_a - (2.0 * scale) * sin_a,
            pos.y + offset_x * sin_a + (2.0 * scale) * cos_a,
        );
        let p_end = Pos2::new(
            pos.x + offset_x * cos_a + (3.5 * scale) * sin_a,
            pos.y + offset_x * sin_a - (3.5 * scale) * cos_a,
        );
        painter.line_segment([p_start, p_end], Stroke::new(1.2 * scale, Color32::from_rgb(71, 85, 105)));
    }

    if is_waving {
        // Extended cartoon fingers waving
        for f in 0..3 {
            let f_angle = angle_rad + (f as f32 - 1.0) * 0.35;
            let f_len = 6.0 * scale;
            let f_base = Pos2::new(pos.x + f_angle.cos() * 4.0 * scale, pos.y + f_angle.sin() * 4.0 * scale);
            let f_tip = Pos2::new(f_base.x + f_angle.cos() * f_len, f_base.y + f_angle.sin() * f_len);
            painter.line_segment([f_base, f_tip], Stroke::new(2.4 * scale, Color32::WHITE));
            painter.circle_filled(f_tip, 1.4 * scale, Color32::WHITE);
            painter.circle_stroke(f_tip, 1.4 * scale, Stroke::new(0.8 * scale, Color32::from_rgb(20, 20, 25)));
        }
    } else if is_pointing {
        // Pointing index finger
        let tip = Pos2::new(pos.x + angle_rad.cos() * 9.0 * scale, pos.y + angle_rad.sin() * 9.0 * scale);
        painter.line_segment([pos, tip], Stroke::new(3.0 * scale, Color32::WHITE));
        painter.circle_filled(tip, 1.6 * scale, Color32::WHITE);
        painter.circle_stroke(tip, 1.6 * scale, Stroke::new(1.0 * scale, Color32::from_rgb(20, 20, 25)));
    }
}

/// Draws the procedural, resolution-independent Beamed Pair Cuby mascot ("Lu & Nu")
/// Color is strictly CONSTANT JET BLACK with CRISP WHITE OUTLINE.
pub fn draw_beamed_pair_mascot(
    painter: &egui::Painter,
    center: Pos2,
    scale: f32,
    time: f64,
    expression: MascotExpression,
) {
    draw_beamed_pair_mascot_ex(painter, center, scale, time, expression, false, false);
}

/// Extended drawer with animated talking mouths for Lu & Nu
pub fn draw_beamed_pair_mascot_ex(
    painter: &egui::Painter,
    center: Pos2,
    scale: f32,
    time: f64,
    expression: MascotExpression,
    is_lu_talking: bool,
    is_nu_talking: bool,
) {
    let t = time as f32;

    // Strict color rules requested by user:
    // "buat warna mereka itu konstan hitam dengan outline putih saja"
    let body_black = Color32::from_rgb(18, 18, 22); // Constant sleek jet black
    let outline_white = Color32::WHITE;             // Crisp white outline
    let outline_stroke = Stroke::new(2.8 * scale, outline_white);

    // 1. Organic Animation, Squash & Stretch, and Personality Dynamics
    let (lu_bounce_y, nu_bounce_y, lu_squash, nu_squash, tilt_angle, blink_factor) = match expression {
        MascotExpression::BeatDance => {
            // Polyrhythmic staggered beat jumping with elastic squash & stretch
            let lu_phase = (t * 8.0).sin();
            let nu_phase = (t * 8.0 + 1.6).sin();

            let lu_b = lu_phase.max(0.0) * 16.0 * scale;
            let nu_b = nu_phase.max(0.0) * 16.0 * scale;

            // When landing (phase < 0), squash wider; when rising, stretch taller
            let lu_sq = -lu_phase * 0.12;
            let nu_sq = -nu_phase * 0.12;
            let tilt = (t * 4.0).sin() * 0.10;
            (lu_b, nu_b, lu_sq, nu_sq, tilt, 0.0)
        }
        MascotExpression::ShockedShield => {
            // High frequency comedic panic jitter
            let jitter_x = (t * 45.0).sin() * 2.5 * scale;
            let jitter_y = (t * 35.0).cos() * 2.0 * scale;
            (jitter_y, -jitter_y, 0.05, 0.05, jitter_x * 0.02, 0.0)
        }
        MascotExpression::CozyWarmth => {
            // Slow, cuddly breathing; leaning slightly towards each other
            let b = (t * 2.2).sin() * 3.5 * scale;
            let sq = (t * 2.2).sin() * 0.04;
            (b, b * 0.9, sq, sq, 0.02, 1.0) // 1.0 = relaxed closed happy eyes
        }
        MascotExpression::Audiophile => {
            // Lu stands tall and proud, Nu leans forward in awe
            let b = (t * 2.5).sin() * 3.0 * scale;
            (b + 2.0 * scale, b - 1.0 * scale, -0.04, 0.04, -0.03, 0.0)
        }
        MascotExpression::WaveHello => {
            // Bouncy welcoming float
            let b = (t * 4.0).sin() * 6.0 * scale;
            let sq = (t * 4.0).sin() * 0.05;
            let tilt = (t * 3.0).sin() * 0.06;
            (b, b + 2.0 * scale, sq, sq, tilt, 0.0)
        }
        MascotExpression::Happy => {
            // Gentle idle breathing floating with natural blink
            let lu_b = (t * 3.0).sin() * 4.5 * scale;
            let nu_b = (t * 3.0 + 0.5).sin() * 4.5 * scale;
            let lu_sq = (t * 3.0).sin() * 0.03;
            let nu_sq = (t * 3.0 + 0.5).sin() * 0.03;
            let tilt = (t * 1.5).sin() * 0.03;

            // Natural blink cycle: eyes close briefly every 3.5 seconds
            let blink_phase = (t % 3.6) / 3.6;
            let blink = if blink_phase > 0.94 {
                ((blink_phase - 0.94) / 0.06 * std::f32::consts::PI).sin()
            } else {
                0.0
            };
            (lu_b, nu_b, lu_sq, nu_sq, tilt, blink)
        }
    };

    let base_center = Pos2::new(center.x, center.y);

    // 2. Head Coordinates (Lu on left, Nu on right)
    // Left Note ("Lu"): Grounded, slightly larger note head
    let lu_x = base_center.x - 34.0 * scale;
    let lu_y = base_center.y + (28.0 + tilt_angle * 32.0) * scale - lu_bounce_y;
    let lu_pos = Pos2::new(lu_x, lu_y);

    // Right Note ("Nu"): Musically higher beamed note interval
    let nu_x = base_center.x + 34.0 * scale;
    let nu_y = base_center.y + (18.0 - tilt_angle * 32.0) * scale - nu_bounce_y;
    let nu_pos = Pos2::new(nu_x, nu_y);

    // 3. Stems & Musical Beam (Black filled + White outline)
    // Tangkai nada menjulang ke atas dari sisi kanan masing-masing not
    let lu_stem_base = Pos2::new(lu_pos.x + 18.0 * scale, lu_pos.y);
    let lu_stem_top = Pos2::new(lu_pos.x + 18.0 * scale, base_center.y - 44.0 * scale - lu_bounce_y * 0.7);

    let nu_stem_base = Pos2::new(nu_pos.x + 18.0 * scale, nu_pos.y);
    let nu_stem_top = Pos2::new(nu_pos.x + 18.0 * scale, base_center.y - 54.0 * scale - nu_bounce_y * 0.7);

    let stem_w = 8.5 * scale;
    // Left Stem (Lu)
    let lu_stem_rect = Rect::from_min_max(
        Pos2::new(lu_stem_base.x - stem_w * 0.5, lu_stem_top.y),
        Pos2::new(lu_stem_base.x + stem_w * 0.5, lu_stem_base.y),
    );
    painter.rect_filled(lu_stem_rect, 0.0, body_black);
    painter.rect_stroke(lu_stem_rect, 0.0, Stroke::new(outline_stroke.width * 0.85, outline_white), egui::StrokeKind::Outside);

    // Right Stem (Nu)
    let nu_stem_rect = Rect::from_min_max(
        Pos2::new(nu_stem_base.x - stem_w * 0.5, nu_stem_top.y),
        Pos2::new(nu_stem_base.x + stem_w * 0.5, nu_stem_base.y),
    );
    painter.rect_filled(nu_stem_rect, 0.0, body_black);
    painter.rect_stroke(nu_stem_rect, 0.0, Stroke::new(outline_stroke.width * 0.85, outline_white), egui::StrokeKind::Outside);

    // Connecting Beamed Bar (Headphone headband arch)
    let beam_poly = vec![
        Pos2::new(lu_stem_top.x - 7.0 * scale, lu_stem_top.y - 8.0 * scale),
        Pos2::new(nu_stem_top.x + 7.0 * scale, nu_stem_top.y - 5.0 * scale),
        Pos2::new(nu_stem_top.x + 7.0 * scale, nu_stem_top.y + 6.0 * scale),
        Pos2::new(lu_stem_top.x - 7.0 * scale, lu_stem_top.y + 3.0 * scale),
    ];
    painter.add(egui::Shape::convex_polygon(
        beam_poly,
        body_black,
        Stroke::new(outline_stroke.width * 0.9, outline_white),
    ));

    // Studio Headphone Cushion Pad beneath the beam
    let pad_poly = vec![
        Pos2::new(lu_stem_top.x + 3.0 * scale, lu_stem_top.y + 1.0 * scale),
        Pos2::new(nu_stem_top.x - 3.0 * scale, nu_stem_top.y + 4.0 * scale),
        Pos2::new(nu_stem_top.x - 3.0 * scale, nu_stem_top.y + 9.0 * scale),
        Pos2::new(lu_stem_top.x + 3.0 * scale, lu_stem_top.y + 6.0 * scale),
    ];
    painter.add(egui::Shape::convex_polygon(
        pad_poly,
        Color32::from_rgb(30, 41, 59),
        Stroke::new(1.0 * scale, Color32::from_rgb(148, 163, 184)),
    ));

    // Studio Earcups hugging outer flanks
    let earcup_w = 13.0 * scale;
    let earcup_h = 32.0 * scale;

    // Left earcup (Lu)
    let left_earcup = Rect::from_center_size(
        Pos2::new(lu_pos.x - 27.0 * scale, lu_pos.y - 3.0 * scale),
        Vec2::new(earcup_w, earcup_h),
    );
    painter.rect_filled(left_earcup, 6.0 * scale, body_black);
    painter.rect_stroke(left_earcup, 6.0 * scale, outline_stroke, egui::StrokeKind::Outside);
    painter.circle_filled(left_earcup.center(), 3.5 * scale, Color32::from_rgb(245, 158, 11)); // Gold pivot
    painter.circle_stroke(left_earcup.center(), 3.5 * scale, Stroke::new(1.0 * scale, Color32::WHITE));

    // Right earcup (Nu)
    let right_earcup = Rect::from_center_size(
        Pos2::new(nu_pos.x + 27.0 * scale, nu_pos.y - 3.0 * scale),
        Vec2::new(earcup_w, earcup_h),
    );
    painter.rect_filled(right_earcup, 6.0 * scale, body_black);
    painter.rect_stroke(right_earcup, 6.0 * scale, outline_stroke, egui::StrokeKind::Outside);
    painter.circle_filled(right_earcup.center(), 3.5 * scale, Color32::from_rgb(245, 158, 11));
    painter.circle_stroke(right_earcup.center(), 3.5 * scale, Stroke::new(1.0 * scale, Color32::WHITE));

    // 4. Render The Chubby Note Heads (Constant Black with Crisp White Outline)
    // Left Note ("Lu") with squash & stretch
    let lu_w = 48.0 * scale * (1.0 + lu_squash);
    let lu_h = 38.0 * scale * (1.0 - lu_squash);
    let lu_rect = Rect::from_center_size(lu_pos, Vec2::new(lu_w, lu_h));
    painter.rect_filled(lu_rect, 19.0 * scale, body_black);
    painter.rect_stroke(lu_rect, 19.0 * scale, outline_stroke, egui::StrokeKind::Outside);

    // Glossy specular highlight sheen on Lu (Piano Black finish)
    let lu_sheen = Rect::from_min_size(
        Pos2::new(lu_pos.x - 18.0 * scale, lu_pos.y - 14.0 * scale),
        Vec2::new(14.0 * scale, 5.5 * scale),
    );
    painter.rect_filled(lu_sheen, 2.5 * scale, Color32::from_rgba_premultiplied(255, 255, 255, 65));

    // Right Note ("Nu") with squash & stretch
    let nu_w = 46.0 * scale * (1.0 + nu_squash);
    let nu_h = 37.0 * scale * (1.0 - nu_squash);
    let nu_rect = Rect::from_center_size(nu_pos, Vec2::new(nu_w, nu_h));
    painter.rect_filled(nu_rect, 18.5 * scale, body_black);
    painter.rect_stroke(nu_rect, 18.5 * scale, outline_stroke, egui::StrokeKind::Outside);

    // Glossy specular highlight sheen on Nu
    let nu_sheen = Rect::from_min_size(
        Pos2::new(nu_pos.x - 17.0 * scale, nu_pos.y - 13.5 * scale),
        Vec2::new(13.0 * scale, 5.0 * scale),
    );
    painter.rect_filled(nu_sheen, 2.5 * scale, Color32::from_rgba_premultiplied(255, 255, 255, 65));

    // 5. Highly Expressive Anime Faces & Details (Pops vividly on Black!)
    let is_closed_eyes = blink_factor > 0.5 || expression == MascotExpression::CozyWarmth;
    let is_shocked = expression == MascotExpression::ShockedShield;

    // Helper closure to draw kawaii expressive eyes on black body
    let draw_kawaii_eye = |center: Pos2, r: f32, is_closed: bool, is_shock: bool, is_star: bool| {
        if is_closed {
            // Cute closed smiling arc "^" in bright white
            let p_l = Pos2::new(center.x - r * 1.15, center.y + 1.2 * scale);
            let p_m = Pos2::new(center.x, center.y - r * 1.1);
            let p_r = Pos2::new(center.x + r * 1.15, center.y + 1.2 * scale);
            painter.line_segment([p_l, p_m], Stroke::new(2.8 * scale, Color32::WHITE));
            painter.line_segment([p_m, p_r], Stroke::new(2.8 * scale, Color32::WHITE));
        } else if is_shock {
            // Huge cartoon shocked eye with tiny trembling pupil
            painter.circle_filled(center, r * 1.35, Color32::WHITE);
            painter.circle_stroke(center, r * 1.35, Stroke::new(1.8 * scale, Color32::from_rgb(15, 23, 42)));
            // Trembling pinprick pupil
            let pupil_jitter = (t * 50.0).sin() * 0.8 * scale;
            painter.circle_filled(
                Pos2::new(center.x + pupil_jitter, center.y + pupil_jitter),
                r * 0.38,
                Color32::from_rgb(15, 23, 42),
            );
        } else if is_star {
            // Star eye for Audiophile wonder / Dance excitement (★)
            painter.circle_filled(center, r * 1.1, Color32::WHITE);
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "★",
                egui::FontId::proportional(12.0 * scale),
                Color32::from_rgb(245, 158, 11),
            );
        } else {
            // Big kawaii round eye with large pupil & dual specular glints
            painter.circle_filled(center, r * 1.1, Color32::WHITE);
            // Deep pupil
            let pupil_pos = Pos2::new(center.x + 0.4 * scale, center.y + 0.3 * scale);
            painter.circle_filled(pupil_pos, r * 0.72, Color32::from_rgb(15, 23, 42));
            // Sparkle Glint 1 (Large)
            painter.circle_filled(Pos2::new(center.x - r * 0.35, center.y - r * 0.35), r * 0.45, Color32::WHITE);
            // Sparkle Glint 2 (Small)
            painter.circle_filled(Pos2::new(center.x + r * 0.42, center.y + r * 0.32), r * 0.22, Color32::WHITE);
        }
    };

    // --- Lu's Face (Left Note) ---
    let lu_eye_y = lu_pos.y - 3.0 * scale;
    let lu_eye_l = Pos2::new(lu_pos.x - 9.0 * scale, lu_eye_y);
    let lu_eye_r = Pos2::new(lu_pos.x + 7.0 * scale, lu_eye_y);

    // Lu's Eyes
    let lu_closed = is_closed_eyes || (expression == MascotExpression::Happy && (t * 0.8).sin() > 0.0);
    draw_kawaii_eye(lu_eye_l, 5.0 * scale, lu_closed, is_shocked, false);
    draw_kawaii_eye(lu_eye_r, 5.0 * scale, lu_closed, is_shocked, false);

    // Lu's Rosy Anime Cheeks (Coral Pink with cute white highlight lines)
    let blush_col = if expression == MascotExpression::CozyWarmth {
        Color32::from_rgba_premultiplied(244, 63, 94, 235)
    } else {
        Color32::from_rgba_premultiplied(244, 63, 94, 160)
    };
    painter.circle_filled(Pos2::new(lu_pos.x - 14.0 * scale, lu_pos.y + 4.5 * scale), 4.2 * scale, blush_col);
    painter.circle_filled(Pos2::new(lu_pos.x + 13.0 * scale, lu_pos.y + 4.5 * scale), 4.2 * scale, blush_col);

    // Lu's Mouth
    if is_lu_talking {
        // Animated talking mouth flapping open/close
        let open = (t * 14.0).sin() > 0.0;
        let m_c = Pos2::new(lu_pos.x - 1.0 * scale, lu_pos.y + 6.5 * scale);
        if open {
            let m_rect = Rect::from_center_size(m_c, Vec2::new(7.5 * scale, 6.0 * scale));
            painter.rect_filled(m_rect, 2.5 * scale, Color32::WHITE);
            painter.circle_filled(Pos2::new(m_c.x, m_c.y + 1.8 * scale), 2.0 * scale, Color32::from_rgb(244, 63, 94));
        } else {
            painter.line_segment(
                [Pos2::new(m_c.x - 3.5 * scale, m_c.y), Pos2::new(m_c.x + 3.5 * scale, m_c.y)],
                Stroke::new(2.2 * scale, Color32::WHITE),
            );
        }
    } else if is_shocked {
        // Wide vibrating shocked O-mouth
        let m_c = Pos2::new(lu_pos.x - 1.0 * scale, lu_pos.y + 8.5 * scale);
        painter.circle_filled(m_c, 4.0 * scale, Color32::WHITE);
        painter.circle_filled(m_c, 2.8 * scale, Color32::from_rgb(15, 23, 42));
    } else if expression == MascotExpression::Audiophile {
        // Confident smirk / smug smile (3)
        let m_c = Pos2::new(lu_pos.x - 1.0 * scale, lu_pos.y + 6.0 * scale);
        painter.line_segment([Pos2::new(m_c.x - 4.0 * scale, m_c.y), Pos2::new(m_c.x, m_c.y + 2.5 * scale)], Stroke::new(2.2 * scale, Color32::WHITE));
        painter.line_segment([Pos2::new(m_c.x, m_c.y + 2.5 * scale), Pos2::new(m_c.x + 4.0 * scale, m_c.y - 1.0 * scale)], Stroke::new(2.2 * scale, Color32::WHITE));
    } else {
        // Sweet curved white smile
        let m_c = Pos2::new(lu_pos.x - 1.0 * scale, lu_pos.y + 6.0 * scale);
        painter.line_segment([Pos2::new(m_c.x - 4.0 * scale, m_c.y), Pos2::new(m_c.x, m_c.y + 2.2 * scale)], Stroke::new(2.2 * scale, Color32::WHITE));
        painter.line_segment([Pos2::new(m_c.x, m_c.y + 2.2 * scale), Pos2::new(m_c.x + 4.0 * scale, m_c.y)], Stroke::new(2.2 * scale, Color32::WHITE));
    }

    // --- Nu's Face (Right Note) ---
    let nu_eye_y = nu_pos.y - 3.0 * scale;
    let nu_eye_l = Pos2::new(nu_pos.x - 8.0 * scale, nu_eye_y);
    let nu_eye_r = Pos2::new(nu_pos.x + 8.0 * scale, nu_eye_y);

    // Nu's Eyes (Star eyes during Audiophile, wink during WaveHello)
    let nu_star = expression == MascotExpression::Audiophile;
    let nu_wink = expression == MascotExpression::WaveHello;
    draw_kawaii_eye(nu_eye_l, 4.8 * scale, is_closed_eyes, is_shocked, nu_star);
    draw_kawaii_eye(nu_eye_r, 4.8 * scale, is_closed_eyes || nu_wink, is_shocked, nu_star);

    // Nu's Rosy Cheeks
    painter.circle_filled(Pos2::new(nu_pos.x - 13.0 * scale, nu_pos.y + 4.5 * scale), 4.0 * scale, blush_col);
    painter.circle_filled(Pos2::new(nu_pos.x + 14.0 * scale, nu_pos.y + 4.5 * scale), 4.0 * scale, blush_col);

    // Nu's Mouth
    if is_nu_talking {
        // Animated talking mouth flapping open/close
        let open = (t * 14.0).sin() > 0.0;
        let m_c = Pos2::new(nu_pos.x, nu_pos.y + 6.5 * scale);
        if open {
            let m_rect = Rect::from_center_size(m_c, Vec2::new(8.5 * scale, 6.5 * scale));
            painter.rect_filled(m_rect, 2.5 * scale, Color32::WHITE);
            painter.circle_filled(Pos2::new(m_c.x, m_c.y + 1.8 * scale), 2.2 * scale, Color32::from_rgb(244, 63, 94));
        } else {
            painter.line_segment(
                [Pos2::new(m_c.x - 3.8 * scale, m_c.y), Pos2::new(m_c.x + 3.8 * scale, m_c.y)],
                Stroke::new(2.2 * scale, Color32::WHITE),
            );
        }
    } else if is_shocked {
        let m_c = Pos2::new(nu_pos.x, nu_pos.y + 8.5 * scale);
        painter.circle_filled(m_c, 3.8 * scale, Color32::WHITE);
        painter.circle_filled(m_c, 2.6 * scale, Color32::from_rgb(15, 23, 42));
    } else {
        // Cheerful open D-mouth with cute pink tongue
        let m_c = Pos2::new(nu_pos.x, nu_pos.y + 6.0 * scale);
        let m_rect = Rect::from_center_size(m_c, Vec2::new(8.0 * scale, 6.5 * scale));
        painter.rect_filled(m_rect, 2.5 * scale, Color32::WHITE);
        painter.circle_filled(Pos2::new(m_c.x, m_c.y + 1.8 * scale), 2.2 * scale, Color32::from_rgb(244, 63, 94)); // Pink tongue
    }

    // 6. Cartoon White Gloved Hands (High Expressiveness!)
    match expression {
        MascotExpression::WaveHello => {
            // Nu's waving arm raised high
            let wave_ang = (t * 10.0).sin() * 0.45;
            let arm_base = Pos2::new(nu_pos.x + 22.0 * scale, nu_pos.y - 2.0 * scale);
            let arm_tip = Pos2::new(
                arm_base.x + 18.0 * scale * (0.8 + wave_ang.cos() * 0.2),
                arm_base.y - 24.0 * scale + wave_ang * 10.0 * scale,
            );
            // Arm (Black with white outline)
            painter.line_segment([arm_base, arm_tip], Stroke::new(6.0 * scale, body_black));
            painter.line_segment([arm_base, arm_tip], Stroke::new(2.2 * scale, outline_white));
            // Cartoon glove waving
            draw_cartoon_glove(painter, arm_tip, scale, wave_ang, true, false);

            // Lu waves friendly at chest height
            let lu_hand_p = Pos2::new(lu_pos.x - 22.0 * scale, lu_pos.y + 8.0 * scale);
            draw_cartoon_glove(painter, lu_hand_p, scale, -0.2, false, false);
        }
        MascotExpression::Audiophile => {
            // Lu holding up golden patch cable with white glove
            let lu_hand_p = Pos2::new(lu_pos.x - 20.0 * scale, lu_pos.y + 4.0 * scale);
            draw_cartoon_glove(painter, lu_hand_p, scale, -0.4, false, true);

            // Golden audiophile monocle on Lu's right eye
            let monocle_center = Pos2::new(lu_eye_r.x + 1.0 * scale, lu_eye_r.y);
            let monocle_r = 9.0 * scale;
            painter.circle_stroke(monocle_center, monocle_r, Stroke::new(2.5 * scale, Color32::from_rgb(245, 158, 11)));
            // Monocle gold chain dangling down
            painter.line_segment(
                [
                    Pos2::new(monocle_center.x + monocle_r, monocle_center.y),
                    Pos2::new(monocle_center.x + monocle_r + 9.0 * scale, monocle_center.y + 16.0 * scale),
                ],
                Stroke::new(1.4 * scale, Color32::from_rgb(217, 119, 6)),
            );
            // Rotating glint star on monocle
            let glint_phase = t * 3.0;
            let glint_sz = 3.5 * scale;
            let glint_c = Pos2::new(monocle_center.x - 3.5 * scale, monocle_center.y - 3.5 * scale);
            let cos_g = glint_phase.cos() * glint_sz;
            let sin_g = glint_phase.sin() * glint_sz;
            painter.line_segment(
                [Pos2::new(glint_c.x - cos_g, glint_c.y - sin_g), Pos2::new(glint_c.x + cos_g, glint_c.y + sin_g)],
                Stroke::new(1.8 * scale, Color32::WHITE),
            );
            painter.line_segment(
                [Pos2::new(glint_c.x + sin_g, glint_c.y - cos_g), Pos2::new(glint_c.x - sin_g, glint_c.y + cos_g)],
                Stroke::new(1.8 * scale, Color32::WHITE),
            );

            // Holographic pure sine wave floating between them
            let wave_center_x = base_center.x;
            let wave_y = base_center.y + 4.0 * scale;
            let wave_pts: Vec<Pos2> = (-24..=24)
                .map(|i| {
                    let x = wave_center_x + i as f32 * 1.0 * scale;
                    let y = wave_y + (i as f32 * 0.28 + t * 6.0).sin() * 7.0 * scale;
                    Pos2::new(x, y)
                })
                .collect();
            for w in wave_pts.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(2.2 * scale, Color32::from_rgb(56, 189, 248)));
            }
        }
        MascotExpression::ShockedShield => {
            // Panic hands on cheeks
            let lu_hand_p = Pos2::new(lu_pos.x - 18.0 * scale, lu_pos.y + 10.0 * scale);
            let nu_hand_p = Pos2::new(nu_pos.x + 18.0 * scale, nu_pos.y + 10.0 * scale);
            draw_cartoon_glove(painter, lu_hand_p, scale, 0.5, false, false);
            draw_cartoon_glove(painter, nu_hand_p, scale, -0.5, false, false);

            // Glowing Pulsing Protective Energy Forcefield Dome (Pure shield, strictly NO symbols, NO text)
            let shield_center = Pos2::new(base_center.x, base_center.y + 8.0 * scale);
            let shield_pulse = (t * 6.0).sin().abs();
            let shield_w = (150.0 + shield_pulse * 10.0) * scale;
            let shield_h = (94.0 + shield_pulse * 6.0) * scale;
            let shield_rect = Rect::from_center_size(shield_center, Vec2::new(shield_w, shield_h));

            // Expanding concentric energy ripple wave
            let ripple_phase = (t * 3.5) % 1.0;
            let ripple_rect = Rect::from_center_size(
                shield_center,
                Vec2::new((shield_w + ripple_phase * 24.0) * scale, (shield_h + ripple_phase * 16.0) * scale),
            );
            let ripple_alpha = ((1.0 - ripple_phase) * 130.0) as u8;
            painter.rect_stroke(
                ripple_rect,
                20.0 * scale,
                Stroke::new(1.8 * scale, Color32::from_rgba_premultiplied(56, 189, 248, ripple_alpha)),
                egui::StrokeKind::Outside,
            );

            // Primary glowing translucent forcefield dome
            painter.rect_filled(shield_rect, 18.0 * scale, Color32::from_rgba_premultiplied(14, 165, 233, 35));
            painter.rect_stroke(
                shield_rect,
                18.0 * scale,
                Stroke::new(2.4 * scale, Color32::from_rgb(56, 189, 248)),
                egui::StrokeKind::Outside,
            );

            // Inner electric forcefield contour
            let inner_rect = Rect::from_center_size(shield_center, Vec2::new(shield_w - 14.0 * scale, shield_h - 14.0 * scale));
            painter.rect_stroke(
                inner_rect,
                14.0 * scale,
                Stroke::new(1.0 * scale, Color32::from_rgba_premultiplied(186, 230, 253, 130)),
                egui::StrokeKind::Outside,
            );

            // Electric micro-arcs along forcefield perimeter
            for a in 0..4 {
                let arc_ang = t * 4.0 + a as f32 * std::f32::consts::FRAC_PI_2;
                let arc_p1 = Pos2::new(
                    shield_center.x + arc_ang.cos() * (shield_w * 0.46),
                    shield_center.y + arc_ang.sin() * (shield_h * 0.46),
                );
                let arc_p2 = Pos2::new(
                    shield_center.x + (arc_ang + 0.18).cos() * (shield_w * 0.47),
                    shield_center.y + (arc_ang + 0.18).sin() * (shield_h * 0.47),
                );
                painter.line_segment([arc_p1, arc_p2], Stroke::new(2.0 * scale, Color32::from_rgb(186, 230, 253)));
            }
        }
        MascotExpression::CozyWarmth => {
            // Hands comfortably resting on hips
            let lu_hand_p = Pos2::new(lu_pos.x - 20.0 * scale, lu_pos.y + 8.0 * scale);
            let nu_hand_p = Pos2::new(nu_pos.x + 20.0 * scale, nu_pos.y + 8.0 * scale);
            draw_cartoon_glove(painter, lu_hand_p, scale, 0.4, false, false);
            draw_cartoon_glove(painter, nu_hand_p, scale, -0.4, false, false);

            // Warm rising steam / heat wave curls (♨)
            for i in 0..3 {
                let x_off = (-32.0 + i as f32 * 32.0) * scale;
                let phase = t * 2.5 + i as f32 * 1.5;
                let steam_y = base_center.y - (56.0 + (phase % 3.0) * 16.0) * scale;
                let steam_x = base_center.x + x_off + (phase * 2.0).sin() * 6.0 * scale;
                let steam_alpha = ((1.0 - (phase % 3.0) / 3.0) * 200.0) as u8;
                painter.text(
                    Pos2::new(steam_x, steam_y),
                    egui::Align2::CENTER_CENTER,
                    "♨",
                    egui::FontId::proportional(16.0 * scale),
                    Color32::from_rgba_premultiplied(251, 146, 60, steam_alpha),
                );
            }
        }
        MascotExpression::BeatDance => {
            // Hands pumping in the air!
            let lu_hand_p = Pos2::new(lu_pos.x - 22.0 * scale, lu_pos.y - 12.0 * scale);
            let nu_hand_p = Pos2::new(nu_pos.x + 22.0 * scale, nu_pos.y - 14.0 * scale);
            draw_cartoon_glove(painter, lu_hand_p, scale, -1.2, false, true);
            draw_cartoon_glove(painter, nu_hand_p, scale, -1.8, false, true);

            // Dancing sparkles & stars around them
            let stars = [
                (-58.0, -36.0, "✦", Color32::from_rgb(250, 204, 21)),
                (60.0, -44.0, "★", Color32::from_rgb(244, 63, 94)),
                (-66.0, 32.0, "♪", Color32::from_rgb(34, 197, 94)),
                (68.0, 26.0, "♫", Color32::from_rgb(56, 189, 248)),
            ];
            for (sx, sy, sym, col) in stars {
                let pulse_sz = (t * 7.0 + sx).sin().abs() * 3.5 * scale;
                painter.text(
                    Pos2::new(base_center.x + sx * scale, base_center.y + sy * scale),
                    egui::Align2::CENTER_CENTER,
                    sym,
                    egui::FontId::proportional((14.0 * scale) + pulse_sz),
                    col,
                );
            }

            // Mini Equalizer Dancing Bars beneath their feet
            let eq_center_x = base_center.x;
            let eq_base_y = base_center.y + 70.0 * scale;
            let bar_count = 7;
            let bar_w = 12.0 * scale;
            let bar_gap = 4.0 * scale;
            let total_eq_w = bar_count as f32 * bar_w + (bar_count - 1) as f32 * bar_gap;
            let start_x = eq_center_x - total_eq_w * 0.5;

            for b in 0..bar_count {
                let bx = start_x + b as f32 * (bar_w + bar_gap);
                let bh = ((t * 8.5 + b as f32 * 1.2).sin().abs() * 18.0 + 4.0) * scale;
                let b_rect = Rect::from_min_size(Pos2::new(bx, eq_base_y - bh), Vec2::new(bar_w, bh));
                painter.rect_filled(b_rect, 2.0 * scale, Color32::from_rgb(168, 85, 247));
            }
        }
        MascotExpression::Happy => {
            // Relaxed hands resting on sides
            let lu_hand_p = Pos2::new(lu_pos.x - 20.0 * scale, lu_pos.y + 8.0 * scale);
            let nu_hand_p = Pos2::new(nu_pos.x + 20.0 * scale, nu_pos.y + 8.0 * scale);
            draw_cartoon_glove(painter, lu_hand_p, scale, 0.3, false, false);
            draw_cartoon_glove(painter, nu_hand_p, scale, -0.3, false, false);

            // Floating gentle music note (♪) rising softly from Lu
            let note_phase = (t * 1.5) % 2.5;
            let note_y = lu_pos.y - 28.0 * scale - note_phase * 12.0 * scale;
            let note_x = lu_pos.x - 12.0 * scale + (t * 2.0).sin() * 4.0 * scale;
            let note_alpha = ((1.0 - note_phase / 2.5) * 210.0) as u8;
            painter.text(
                Pos2::new(note_x, note_y),
                egui::Align2::CENTER_CENTER,
                "♪",
                egui::FontId::proportional(15.0 * scale),
                Color32::from_rgba_premultiplied(168, 85, 247, note_alpha),
            );
        }
    }
}

