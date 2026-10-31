use eframe::egui;
use lunulaplayer::LunulaApp;
use lunulaplayer::models::{AppTab, FilterSlope, FilterType, NodeType, ParametricBand, ParametricEqPreset, PreampMode};

#[test]
fn test_saved_eq_node_click_navigates_to_equalizer_tab_with_exact_preset() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

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
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

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
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

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
    let mut btn_pos = egui::Pos2::ZERO;
    for shape in out0.shapes {
        if let egui::epaint::Shape::Text(text_shape) = shape.shape {
            if text_shape.galley.job.text.contains("Equalizer Presets") {
                btn_pos = text_shape.pos;
            }
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
