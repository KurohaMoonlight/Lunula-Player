use eframe::egui;
use lunulaplayer::LunulaApp;
use lunulaplayer::models::AppTab;

#[test]
fn test_patchbay_zoom_clipping_and_autofit() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1024.0, 710.0)));

    let mut app = LunulaApp::default();
    app.current_tab = AppTab::SignalChain;

    // Test 1: At 45% zoom (zoom-out), no world-layer shape should be truncated by scaled clip_rect
    app.patchbay_zoom = 0.45;
    let canvas_size = egui::vec2(1024.0, 710.0);
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            app.render_modular_patchbay_canvas(ui, canvas_size);
        },
    );

    // Verify canvas world shapes have clip_rect matching the full canvas bounds
    for shape in &out.shapes {
        // Shapes with bounds larger than 100px should not be clipped to a small fraction of the screen
        let b = shape.shape.visual_bounding_rect();
        if b.width() > 100.0 && b.height() > 50.0 {
            assert!(
                shape.clip_rect.max.y >= 700.0,
                "Large world shape should have unconstrained full canvas clip_rect, got {:?}",
                shape.clip_rect
            );
        }
    }
    out.textures_delta.clear();

    // Test 2: Test reset_patchbay_layout auto-fit zoom on narrow canvas (e.g. 760px)
    app.reset_patchbay_layout(760.0, 600.0);
    assert!(app.patchbay_zoom < 1.0, "Narrow canvas should auto-fit zoom to prevent node overlap");
    assert!(app.patchbay_zoom >= 0.65, "Auto-fit zoom should stay within reasonable bounds");

    // All default nodes (1, 2, 4, 5) must be non-overlapping and stay inside effective canvas
    let n1 = app.nodes.iter().find(|n| n.id == 1).unwrap();
    let n2 = app.nodes.iter().find(|n| n.id == 2).unwrap();
    let n4 = app.nodes.iter().find(|n| n.id == 4).unwrap();
    let n5 = app.nodes.iter().find(|n| n.id == 5).unwrap();

    assert!(n1.pos.x < n2.pos.x, "Node 1 must be to the left of Node 2");
    assert!(n2.pos.x < n4.pos.x, "Node 2 must be to the left of Node 4");
    assert!(n4.pos.x < n5.pos.x, "Node 4 must be to the left of Node 5");
    assert!(n5.pos.x - n4.pos.x >= 198.0, "Node 5 must not overlap Node 4 horizontally");
}

