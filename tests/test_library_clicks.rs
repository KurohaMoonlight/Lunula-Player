use eframe::egui;
use lunulaplayer::LunulaApp;
use lunulaplayer::models::{LibraryCategory, Track};

#[test]
fn test_print_card_rects_for_grid() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

    let mut app = LunulaApp::default();
    let names = ["Black Soul", "Compass", "EEYUH!", "EEYUH! x Fluxxwave", "Encore", "Happy End", "INTERNET OVERDOSE", "INTERNET YAMERO", "Literature", "Mon coeur avait raison"];
    for name in names {
        app.playlist.push(Track {
            title: format!("Title {}", name),
            artist: "Artist".to_string(),
            album: name.to_string(),
            duration: 180.0,
            format: "FLAC".to_string(),
            sample_rate: 44100,
            bit_depth: 16,
            waveform: vec![0.5; 10],
            waveform_bass: vec![0.5; 10],
            waveform_mid: vec![0.5; 10],
            waveform_treble: vec![0.5; 10],
            file_path: Some(format!("C:/music/{}.flac", name)),
            file_size_mb: 25.0,
            bitrate_kbps: None,
            is_favorite: false,
        });
    }
    app.library_category = LibraryCategory::Albums;

    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
        },
    );
    out0.textures_delta.clear();

    // Try clicking Putar on the second row (Happy End)
    let encore_btn_pos = egui::pos2(42.8, 704.5);
    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerMoved(encore_btn_pos)],
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
        },
    );
    out1.textures_delta.clear();

    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.1),
            events: vec![egui::Event::PointerButton {
                pos: encore_btn_pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
        },
    );
    out2.textures_delta.clear();

    let mut out3 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.15),
            events: vec![egui::Event::PointerButton {
                pos: encore_btn_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
        },
    );
    out3.textures_delta.clear();

    println!("After clicking second row Putar:");
    println!("app.current_track: {}", app.current_track.title);
    println!("app.is_playing: {}", app.is_playing);
    println!("app.library_category: {:?}", app.library_category);
    println!("app.selected_album_filter: {:?}", app.selected_album_filter);

    assert_eq!(app.current_track.album, "Encore", "Must play second row card!");
}

#[test]
fn test_real_album_card_lihat_lagu_click_triggers_action() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

    let mut app = LunulaApp::default();
    app.playlist = vec![
        Track {
            title: "Song A".to_string(),
            artist: "Artist A".to_string(),
            album: "Album A".to_string(),
            duration: 180.0,
            format: "FLAC".to_string(),
            sample_rate: 44100,
            bit_depth: 16,
            waveform: vec![0.5; 10],
            waveform_bass: vec![0.5; 10],
            waveform_mid: vec![0.5; 10],
            waveform_treble: vec![0.5; 10],
            file_path: Some("C:/music/song_a.flac".to_string()),
            file_size_mb: 25.0,
            bitrate_kbps: None,
            is_favorite: false,
        },
    ];
    app.library_category = LibraryCategory::Albums;

    let mut btn_pos = egui::Pos2::ZERO;
    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
        },
    );
    out0.textures_delta.clear();

    for shape in out0.shapes {
        if let egui::epaint::Shape::Text(text_shape) = shape.shape {
            if text_shape.galley.job.text.contains("Lihat Lagu") {
                btn_pos = text_shape.pos;
                println!("FOUND Lihat Lagu at pos: {:?}", btn_pos);
            }
        }
    }

    assert_ne!(btn_pos, egui::Pos2::ZERO, "Could not locate 'Lihat Lagu' button text!");

    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerMoved(btn_pos)],
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
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
            app.render_library_panel(ui);
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
            app.render_library_panel(ui);
        },
    );
    out3.textures_delta.clear();

    println!("app.library_category: {:?}", app.library_category);
    println!("app.selected_album_filter: {:?}", app.selected_album_filter);

    assert_eq!(app.library_category, LibraryCategory::AllTracks, "Clicking 'Lihat Lagu' MUST switch to AllTracks!");
    assert_eq!(app.selected_album_filter.as_deref(), Some("Album A"), "Album filter must be set!");
}

#[test]
fn test_real_album_card_button_click_triggers_action() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

    let mut app = LunulaApp::default();
    app.playlist = vec![
        Track {
            title: "Song A".to_string(),
            artist: "Artist A".to_string(),
            album: "Album A".to_string(),
            duration: 180.0,
            format: "FLAC".to_string(),
            sample_rate: 44100,
            bit_depth: 16,
            waveform: vec![0.5; 10],
            waveform_bass: vec![0.5; 10],
            waveform_mid: vec![0.5; 10],
            waveform_treble: vec![0.5; 10],
            file_path: Some("C:/music/song_a.flac".to_string()),
            file_size_mb: 25.0,
            bitrate_kbps: None,
            is_favorite: false,
        },
    ];
    app.library_category = LibraryCategory::Albums;

    // First frame to layout and locate where the button was placed
    let mut btn_pos = egui::Pos2::ZERO;
    let mut out0 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.0),
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
        },
    );
    out0.textures_delta.clear();

    // Find the text "▶ Putar" or button in painter shapes or primitives
    // Let's inspect shapes to find the exact rect of "▶ Putar"
    for shape in out0.shapes {
        if let egui::epaint::Shape::Text(text_shape) = shape.shape {
            if text_shape.galley.job.text.contains("Putar") {
                btn_pos = text_shape.pos;
                println!("FOUND Putar at pos: {:?}", btn_pos);
            }
        }
    }

    assert_ne!(btn_pos, egui::Pos2::ZERO, "Could not locate 'Putar' button text in render output!");

    // Frame 1: Pointer hover over the button
    let mut out1 = ctx.run_ui(
        egui::RawInput {
            screen_rect: screen,
            time: Some(0.05),
            events: vec![egui::Event::PointerMoved(btn_pos)],
            ..Default::default()
        },
        |ui| {
            app.render_library_panel(ui);
        },
    );
    out1.textures_delta.clear();

    // Frame 2: Mouse down on the button
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
            app.render_library_panel(ui);
        },
    );
    out2.textures_delta.clear();

    // Frame 3: Mouse up on the button
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
            app.render_library_panel(ui);
        },
    );
    out3.textures_delta.clear();

    // Now check: did current_track change to Song A or is_playing become true?
    println!("app.current_track: {}", app.current_track.title);
    println!("app.is_playing: {}", app.is_playing);
    println!("app.library_category: {:?}", app.library_category);
    println!("app.selected_album_filter: {:?}", app.selected_album_filter);

    assert_eq!(app.current_track.title, "Song A", "Clicking '▶ Putar' on Album card MUST play the track!");
}

#[test]
fn test_real_library_panel_all_categories_render() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

    let mut app = LunulaApp::default();
    app.playlist = vec![
        Track {
            title: "Test Track 1".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            duration: 180.0,
            format: "FLAC".to_string(),
            sample_rate: 44100,
            bit_depth: 16,
            waveform: vec![0.5; 50],
            waveform_bass: vec![0.5; 50],
            waveform_mid: vec![0.5; 50],
            waveform_treble: vec![0.5; 50],
            file_path: None,
            file_size_mb: 25.0,
            bitrate_kbps: None,
            is_favorite: true,
        },
        Track {
            title: "Test Track 2".to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            duration: 200.0,
            format: "FLAC".to_string(),
            sample_rate: 96000,
            bit_depth: 24,
            waveform: vec![0.5; 50],
            waveform_bass: vec![0.5; 50],
            waveform_mid: vec![0.5; 50],
            waveform_treble: vec![0.5; 50],
            file_path: None,
            file_size_mb: 60.0,
            bitrate_kbps: None,
            is_favorite: false,
        },
    ];

    let categories = [
        LibraryCategory::AllTracks,
        LibraryCategory::Albums,
        LibraryCategory::Artists,
        LibraryCategory::Quality,
        LibraryCategory::Favorites,
        LibraryCategory::Playlists,
    ];

    for cat in categories {
        app.library_category = cat;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.1),
                ..Default::default()
            },
            |ui| {
                app.render_library_panel(ui);
            },
        );
        out.textures_delta.clear();
    }
}

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
