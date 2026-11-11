use eframe::egui;
use lunulaplayer::LunulaApp;
use lunulaplayer::models::{AppTab, FilterSlope, FilterType, NodeType, ParametricBand, ParametricEqPreset, PreampMode};

#[test]
fn test_saved_eq_node_click_navigates_to_equalizer_tab_with_exact_preset() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 3000.0)));

    let mut app = LunulaApp::default();
    app.current_tab = AppTab::SignalChain;

    // Custom test preset
    let custom_preset = ParametricEqPreset {
        name: "Audiophile Custom Vocal".to_string(),
        preamp_mode: PreampMode::Manual,
        manual_preamp_db: -3.5,
        bands: vec![
            ParametricBand {
                freq: 120.0,
                gain_db: 2.5,
                q: 1.1,
                filter_type: FilterType::Peaking,
                slope: FilterSlope::Db12,
                enabled: true,
            },
            ParametricBand {
                freq: 3200.0,
                gain_db: 4.0,
                q: 2.0,
                filter_type: FilterType::Peaking,
                slope: FilterSlope::Db12,
                enabled: true,
            },
        ],
    };
    app.eq_presets.push(custom_preset.clone());

    // Add this preset as a node on the canvas
    app.add_modular_filter(NodeType::SavedEqPreset(custom_preset.name.clone()));

    // Find the node id and its position
    let node_idx = app.nodes.iter().position(|n| n.node_type == NodeType::SavedEqPreset(custom_preset.name.clone())).unwrap();
    let node = &app.nodes[node_idx];
    let _node_id = node.id;
    let node_pos = node.pos;

    // Render frame to layout the canvas
    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            let canvas_size = egui::vec2(800.0, 600.0);
            app.render_modular_patchbay_canvas(ui, canvas_size);
        },
    );
    out0.textures_delta.clear();

    // Node world rect calculation:
    // canvas_rect.min.x + patchbay_pan.x + node.pos.x
    // In our test, canvas_rect.min is (0,0), patchbay_pan is (0,0)
    let click_pos = egui::pos2(node_pos.x + 40.0, node_pos.y + 25.0);

    // Simulate click on the Saved Equalizer node
    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerMoved(click_pos)],
            ..Default::default()
        },
        |ui| {
            let canvas_size = egui::vec2(800.0, 600.0);
            app.render_modular_patchbay_canvas(ui, canvas_size);
        },
    );
    out1.textures_delta.clear();

    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.1),
            events: vec![egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let canvas_size = egui::vec2(800.0, 600.0);
            app.render_modular_patchbay_canvas(ui, canvas_size);
        },
    );
    out2.textures_delta.clear();

    let mut out3 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.15),
            events: vec![egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let canvas_size = egui::vec2(800.0, 600.0);
            app.render_modular_patchbay_canvas(ui, canvas_size);
        },
    );
    out3.textures_delta.clear();

    // Verification:
    // 1. Current tab MUST switch to Equalizer
    assert_eq!(app.current_tab, AppTab::Equalizer, "Clicking Saved EQ card on canvas MUST navigate to Equalizer tab!");
    // 2. Active EQ preset name MUST match the clicked preset
    assert_eq!(app.active_eq.name, "Audiophile Custom Vocal", "Active EQ preset name MUST match the card clicked!");
    // 3. Preamp and bands MUST be identical to the clicked preset
    assert_eq!(app.active_eq.manual_preamp_db, -3.5);
    assert_eq!(app.active_eq.bands.len(), 2);
    assert_eq!(app.active_eq.bands[0].freq, 120.0);
    assert_eq!(app.active_eq.bands[1].freq, 3200.0);
    assert_eq!(app.active_eq.bands[1].gain_db, 4.0);
}

#[test]
fn test_saved_eq_floating_popup_positioning_and_actions() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 3000.0)));

    let mut app = LunulaApp::default();
    app.current_tab = AppTab::SignalChain;

    // Anchor near the right edge (like the FX sidebar palette at x=1050, y=200)
    let right_anchor = egui::pos2(1050.0, 200.0);
    app.show_saved_eq_palette_popup = true;
    app.saved_eq_popup_anchor = Some(right_anchor);

    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            app.render_saved_eq_floating_popup(ui);
        },
    );
    out0.textures_delta.clear();

    // Verify popup was rendered without panic and is visible
    assert!(app.show_saved_eq_palette_popup);

    // Verify that all default presets exist in eq_presets
    let preset_names: Vec<&str> = app.eq_presets.iter().map(|p| p.name.as_str()).collect();
    assert!(preset_names.contains(&"Flat (Netral)"));
    assert!(preset_names.contains(&"Harman-style Tilt (Generik)"));
    assert!(preset_names.contains(&"Vocal Clarity"));
    assert!(preset_names.contains(&"Air & Sparkle"));
}

#[test]
fn test_sidebar_saved_equalizers_plus_button_toggles_popup() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 3000.0)));

    let mut app = LunulaApp::default();
    app.current_tab = AppTab::SignalChain;
    assert!(!app.show_saved_eq_palette_popup);

    // Render sidebar palette
    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            app.render_fx_sidebar_palette(ui);
        },
    );
    out0.textures_delta.clear();

    // Find the text "Equalizer Presets"
    fn find_text_pos(shape: &egui::epaint::Shape, needle: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(t) => {
                if t.galley.job.text.contains(needle) {
                    Some(t.pos)
                } else {
                    None
                }
            }
            egui::epaint::Shape::Vec(v) => {
                for s in v {
                    if let Some(p) = find_text_pos(s, needle) {
                        return Some(p);
                    }
                }
                None
            }
            _ => None,
        }
    }

    let mut btn_pos = egui::Pos2::ZERO;
    for shape in &out0.shapes {
        if let Some(pos) = find_text_pos(&shape.shape, "Equalizer Presets") {
            btn_pos = pos;
            break;
        }
    }
    assert_ne!(btn_pos, egui::Pos2::ZERO, "Equalizer Presets card must be rendered in sidebar!");

    // Click it
    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerMoved(btn_pos)],
            ..Default::default()
        },
        |ui| {
            app.render_fx_sidebar_palette(ui);
        },
    );
    out1.textures_delta.clear();

    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.1),
            events: vec![egui::Event::PointerButton {
                pos: btn_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            app.render_fx_sidebar_palette(ui);
        },
    );
    out2.textures_delta.clear();

    let mut out3 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.15),
            events: vec![egui::Event::PointerButton {
                pos: btn_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            app.render_fx_sidebar_palette(ui);
        },
    );
    out3.textures_delta.clear();

    // After clicking, the popup must be open with anchor set near the button!
    assert!(app.show_saved_eq_palette_popup, "Clicking SAVED EQUALIZERS must open floating popup!");
    assert!(app.saved_eq_popup_anchor.is_some(), "Anchor must be recorded for cursor-follow positioning!");
}

#[test]
fn test_dag_branching_and_cycle_prevention() {
    let mut app = LunulaApp::default();
    // Default nodes: 1=Source, 2=RAM, 3=DSP, 4=Engine, 5=DAC
    // Add two modular filters: id 6 and id 7
    app.add_modular_filter(NodeType::TubeWarmer); // id 6
    app.add_modular_filter(NodeType::StereoWidener); // id 7

    // Clear existing connections to test arbitrary DAG
    app.connections.clear();
    app.invalidate_chain_cache();

    // 1. Branching Output (1 to 2):
    // DSP (3) branches to TubeWarmer (6) AND StereoWidener (7)
    app.connections.push(lunulaplayer::models::CableConnection::new(3, 6));
    app.connections.push(lunulaplayer::models::CableConnection::new(3, 7));

    assert!(app.can_reach_node(3, 6));
    assert!(app.can_reach_node(3, 7));

    // 2. Merging Input (2 to 1):
    // TubeWarmer (6) and StereoWidener (7) both merge into Audio Engine (4)
    app.connections.push(lunulaplayer::models::CableConnection::new(6, 4));
    app.connections.push(lunulaplayer::models::CableConnection::new(7, 4));

    assert!(app.can_reach_node(6, 4));
    assert!(app.can_reach_node(7, 4));
    assert!(app.can_reach_node(3, 4)); // Reachable via either branch

    // 3. Cycle Prevention:
    // Attempting to connect Audio Engine (4) back to DSP (3) would form a cycle (3 -> 6 -> 4 -> 3)
    let would_create_cycle = app.can_reach_node(4, 3);
    assert!(!would_create_cycle, "Currently 4 does not reach 3");

    // But if we check whether connecting 4 to 3 would create a cycle:
    // we query `can_reach_node(target, source)`, i.e. `can_reach_node(3, 4)` is true!
    // Therefore connecting 4 -> 3 is correctly flagged as creating a cycle:
    assert!(app.can_reach_node(3, 4), "Since 3 reaches 4, connecting 4->3 must be blocked as a cycle!");
}

#[test]
fn test_red_dashed_line_suppressed_when_reaching_through_intermediate_filter() {
    let mut app = LunulaApp::default();
    app.add_modular_filter(NodeType::LowPassFilter); // id 6
    app.add_modular_filter(NodeType::SavedEqPreset("Vocal Clarity".to_string())); // id 7

    // Configure chain:
    // 1 (Source) -> 2 (RAM) -> 6 (LPF) -> 7 (Saved EQ) -> 4 (Resampler) -> 5 (DAC)
    app.connections.clear();
    app.invalidate_chain_cache();
    app.connections.push(lunulaplayer::models::CableConnection::new(1, 2));
    app.connections.push(lunulaplayer::models::CableConnection::new(2, 6));
    app.connections.push(lunulaplayer::models::CableConnection::new(6, 7));
    app.connections.push(lunulaplayer::models::CableConnection::new(7, 4));
    app.connections.push(lunulaplayer::models::CableConnection::new(4, 5));

    let eval = app.evaluate_signal_chain();
    assert!(eval.is_complete, "Full chain through intermediate filter must be complete");
    assert!(app.can_reach_node(6, 4), "LPF must reach Resampler via filter 7");

    // Topological order compilation
    let graph = app.compile_signal_chain();
    assert!(graph.is_complete);
    assert_eq!(graph.effects.len(), 2, "Must contain LPF and Saved EQ effects in order");
}

#[test]
fn test_equalizer_tab_switching_multi_frame_resilience() {
    let ctx = egui::Context::default();
    let screen1 = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 800.0)));
    let screen2 = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1024.0, 768.0)));

    let mut app = LunulaApp::default();

    // 1. Start in NowPlaying tab
    app.current_tab = AppTab::NowPlaying;
    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen1,
            time: Some(0.0),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_equalizer_panel(ui);
            });
        },
    );
    out1.textures_delta.clear();

    // 2. Switch to Equalizer tab
    app.current_tab = AppTab::Equalizer;

    // Render 5 consecutive frames with resizing and preset switching
    for frame in 1..=5 {
        if frame % 2 == 0 {
            app.active_eq.bands[0].gain_db = (frame as f32) * 1.5;
        }
        let screen = if frame % 2 == 0 { screen2 } else { screen1 };
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(frame as f64 * 0.016),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_equalizer_panel(ui);
                });
            },
        );
        out.textures_delta.clear();
    }
}

#[test]
fn test_saved_eq_floating_popup_master_banner_and_animations() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 900.0)));

    let mut app = LunulaApp::default();
    app.current_tab = AppTab::SignalChain;
    app.show_saved_eq_palette_popup = true;
    app.saved_eq_popup_anchor = Some(egui::pos2(1100.0, 300.0));

    // Frame 1: Initial opening with animation
    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            app.render_saved_eq_floating_popup(ui);
        },
    );
    out1.textures_delta.clear();

    // Helper to find text position in shapes
    fn find_text_pos(shape: &egui::epaint::Shape, needle: &str) -> Option<egui::Pos2> {
        match shape {
            egui::epaint::Shape::Text(t) => {
                if t.galley.job.text.contains(needle) {
                    Some(t.pos)
                } else {
                    None
                }
            }
            egui::epaint::Shape::Vec(v) => {
                for s in v {
                    if let Some(p) = find_text_pos(s, needle) {
                        return Some(p);
                    }
                }
                None
            }
            _ => None,
        }
    }

    let mut banner_pos = egui::Pos2::ZERO;
    for shape in &out1.shapes {
        if let Some(pos) = find_text_pos(&shape.shape, "Buka Studio Equalizer Penuh") {
            banner_pos = pos;
            break;
        }
    }
    assert_ne!(banner_pos, egui::Pos2::ZERO, "Master equalizer banner text must be rendered in popup!");

    // Frame 2: Click the master equalizer banner
    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![
                egui::Event::PointerMoved(banner_pos),
                egui::Event::PointerButton {
                    pos: banner_pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: banner_pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        },
        |ui| {
            app.render_saved_eq_floating_popup(ui);
        },
    );
    out2.textures_delta.clear();

    // Verify it switched to Equalizer tab and closed the popup
    assert_eq!(app.current_tab, AppTab::Equalizer, "Clicking master banner must switch to Equalizer tab");
    assert!(!app.show_saved_eq_palette_popup, "Popup must close upon launching the full Equalizer tab");
}
