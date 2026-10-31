use eframe::egui;

#[test]
fn test_album_card_button_clicks_not_intercepted() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 1000.0)));

    let card_w = 215.0;
    let card_h = 277.0;

    let btn_rect = egui::Rect::from_min_size(egui::pos2(40.0, 230.0), egui::vec2(58.0, 26.0));
    let click_pos = btn_rect.center();

    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            events: vec![egui::Event::PointerMoved(click_pos)],
            ..Default::default()
        },
        |ui| {
            let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(card_w, card_h));
            let _card_resp = ui.interact(card_rect, ui.scope_id().with("alb_card"), egui::Sense::click());
            let _ = ui.put(btn_rect, egui::Button::new("▶ Putar"));
        },
    );
    out0.textures_delta.clear();

    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(card_w, card_h));
            let _card_resp = ui.interact(card_rect, ui.scope_id().with("alb_card"), egui::Sense::click());
            let _ = ui.put(btn_rect, egui::Button::new("▶ Putar"));
        },
    );
    out1.textures_delta.clear();

    let mut btn_clicked = false;
    let mut card_clicked = false;
    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.1),
            events: vec![egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(card_w, card_h));
            let card_resp = ui.interact(card_rect, ui.scope_id().with("alb_card"), egui::Sense::click());

            let btn = ui.put(btn_rect, egui::Button::new("▶ Putar"));
            btn_clicked = btn.clicked();
            card_clicked = card_resp.clicked();
        },
    );
    out2.textures_delta.clear();

    assert!(btn_clicked, "Button inside Album card must receive the click!");
    assert!(!card_clicked, "Album card background must not steal click from its button!");
}

#[test]
fn test_album_card_background_click_works_outside_buttons() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 1000.0)));

    let card_w = 215.0;
    let card_h = 277.0;

    let art_pos = egui::pos2(100.0, 50.0);
    let btn_rect = egui::Rect::from_min_size(egui::pos2(40.0, 230.0), egui::vec2(58.0, 26.0));

    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            events: vec![egui::Event::PointerMoved(art_pos)],
            ..Default::default()
        },
        |ui| {
            let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(card_w, card_h));
            let _card_resp = ui.interact(card_rect, ui.scope_id().with("alb_card"), egui::Sense::click());
            let _ = ui.put(btn_rect, egui::Button::new("▶ Putar"));
        },
    );
    out0.textures_delta.clear();

    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerButton {
                pos: art_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(card_w, card_h));
            let _card_resp = ui.interact(card_rect, ui.scope_id().with("alb_card"), egui::Sense::click());
            let _ = ui.put(btn_rect, egui::Button::new("▶ Putar"));
        },
    );
    out1.textures_delta.clear();

    let mut btn_clicked = false;
    let mut card_clicked = false;
    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.1),
            events: vec![egui::Event::PointerButton {
                pos: art_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let card_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(card_w, card_h));
            let card_resp = ui.interact(card_rect, ui.scope_id().with("alb_card"), egui::Sense::click());

            let btn = ui.put(btn_rect, egui::Button::new("▶ Putar"));
            btn_clicked = btn.clicked();
            card_clicked = card_resp.clicked();
        },
    );
    out2.textures_delta.clear();

    assert!(!btn_clicked, "Button must not click when clicking on artwork!");
    assert!(card_clicked, "Album card must receive click when clicking outside buttons!");
}

#[test]
fn test_alltracks_row_heart_toggle_not_intercepted() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 1000.0)));

    let row_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 46.0));
    let heart_rect = egui::Rect::from_min_size(egui::pos2(70.0, 6.0), egui::vec2(28.0, 34.0));
    let click_pos = heart_rect.center();

    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            events: vec![egui::Event::PointerMoved(click_pos)],
            ..Default::default()
        },
        |ui| {
            let _row_click = ui.interact(row_rect, ui.scope_id().with("track_row"), egui::Sense::click());
            let _ = ui.put(heart_rect, egui::Button::new("🤍"));
        },
    );
    out0.textures_delta.clear();

    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let _row_click = ui.interact(row_rect, ui.scope_id().with("track_row"), egui::Sense::click());
            let _ = ui.put(heart_rect, egui::Button::new("🤍"));
        },
    );
    out1.textures_delta.clear();

    let mut heart_clicked = false;
    let mut row_clicked = false;
    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.1),
            events: vec![egui::Event::PointerButton {
                pos: click_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            let row_click = ui.interact(row_rect, ui.scope_id().with("track_row"), egui::Sense::click());
            let heart_btn = ui.put(heart_rect, egui::Button::new("🤍"));
            heart_clicked = heart_btn.clicked();
            row_clicked = row_click.clicked();
        },
    );
    out2.textures_delta.clear();

    assert!(heart_clicked, "Heart button must be clicked!");
    assert!(!row_clicked, "Row play must not be triggered when clicking the heart button!");
}
