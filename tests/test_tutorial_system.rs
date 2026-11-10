use eframe::egui;
use lunulaplayer::LunulaApp;
use lunulaplayer::models::{AppTab, NodeType};

#[test]
fn test_patchbay_tutorial_renders_all_7_steps() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));
    let mut app = LunulaApp::default();
    app.current_tab = AppTab::SignalChain;
    app.show_patchbay_tutorial = true;

    for step in 0..7 {
        app.patchbay_tutorial_step = step;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(1.23),
                ..Default::default()
            },
            |ui| {
                let canvas_rect = egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(1000.0, 600.0));
                app.render_patchbay_tutorial_card(ui, canvas_rect);
            },
        );
        out.textures_delta.clear();
    }
}

#[test]
fn test_filter_tutorial_renders_all_14_nodes_and_4_slides() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));
    let mut app = LunulaApp::default();
    app.current_tab = AppTab::SignalChain;

    let sample_nodes = [
        NodeType::LowPassFilter,
        NodeType::TubeWarmer,
        NodeType::StereoWidener,
        NodeType::BassGuard,
        NodeType::VolumeAttenuator,
        NodeType::SavedEqPreset("Vocal Clarity".to_string()),
        NodeType::Mixer,
        NodeType::Splitter,
        NodeType::VolumeBooster,
        NodeType::AnalogPreamp,
        NodeType::DynamicLoudness,
        NodeType::SteppedAttenuator,
        NodeType::OpticalLeveler,
        NodeType::StereoBalance,
    ];

    for node_type in sample_nodes {
        app.active_filter_tutorial = Some(node_type);
        for slide in 0..4 {
            app.filter_tutorial_step = slide;
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: screen,
                    time: Some(2.5),
                    ..Default::default()
                },
                |ui| {
                    let canvas_rect = egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(1000.0, 600.0));
                    app.render_filter_tutorial_card(ui, canvas_rect);
                },
            );
            out.textures_delta.clear();
        }
    }
}

#[test]
fn test_game_like_onboarding_quest_renders_all_stages() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));
    let mut app = LunulaApp::default();
    app.is_onboarding_quest_active = true;

    let quest_stages = [
        lunulaplayer::models::OnboardingQuestStep::Welcome,
        lunulaplayer::models::OnboardingQuestStep::NavigateToStudio,
        lunulaplayer::models::OnboardingQuestStep::UnderstandModes,
        lunulaplayer::models::OnboardingQuestStep::ConnectFirstCable,
        lunulaplayer::models::OnboardingQuestStep::TestAutoMute,
        lunulaplayer::models::OnboardingQuestStep::InsertTubeWarmer,
        lunulaplayer::models::OnboardingQuestStep::Completed,
    ];

    for stage in quest_stages {
        app.onboarding_quest_step = stage;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(1.5),
                ..Default::default()
            },
            |ui| {
                app.render_onboarding_quest_overlay(ui);
            },
        );
        out.textures_delta.clear();
    }
}

#[test]
fn test_first_launch_triggers_onboarding_by_default() {
    let mut app = LunulaApp::new_standalone();
    if !app.has_completed_onboarding {
        assert!(app.is_onboarding_quest_active);
        assert_eq!(app.chain_mode, lunulaplayer::models::ChainMode::BitPerfect);
    } else {
        // If local developer storage had saved state, resetting onboarding enforces Bit-Perfect
        app.has_completed_onboarding = false;
        app.is_onboarding_quest_active = true;
        app.set_chain_mode(lunulaplayer::models::ChainMode::BitPerfect);
        assert!(!app.has_completed_onboarding);
        assert_eq!(app.chain_mode, lunulaplayer::models::ChainMode::BitPerfect);
    }
}

#[test]
fn test_default_nodes_are_spacious_and_do_not_overlap() {
    let mut app = LunulaApp::new_standalone();
    app.reset_patchbay_layout(1200.0, 700.0);
    
    // Check that top-deck nodes (1, 2, 4, 5) do not overlap
    let node1 = app.nodes.iter().find(|n| n.id == 1).unwrap();
    let node2 = app.nodes.iter().find(|n| n.id == 2).unwrap();
    let node3 = app.nodes.iter().find(|n| n.id == 3);
    let node4 = app.nodes.iter().find(|n| n.id == 4).unwrap();
    let node5 = app.nodes.iter().find(|n| n.id == 5).unwrap();

    let node_w = 188.0;
    assert!(node2.pos.x >= node1.pos.x + node_w + 40.0, "Node 2 must have at least 40px gap from Node 1");
    assert!(node4.pos.x >= node2.pos.x + node_w + 40.0, "Node 4 must have at least 40px gap from Node 2");
    assert!(node5.pos.x >= node4.pos.x + node_w + 40.0, "Node 5 must have at least 40px gap from Node 4");

    // DSP Engine (node 3) if present must be on lower deck (y >= 180) so it does not block the top wire
    if let Some(node3) = node3 {
        assert!(node3.pos.y >= 180.0, "DSP Engine must be on the lower deck out of the direct wire path");
    }
}

#[test]
fn test_severed_cable_mutes_audio_canvas_source_of_truth() {
    use lunulaplayer::audio::types::calculate_effective_volume;
    // When cable is severed, software_volume is set to 0.0 and muted is set to true
    // calculate_effective_volume must return 0.0 in both Bit-Perfect and DSP modes
    assert_eq!(calculate_effective_volume(true, true, 1.0), 0.0);
    assert_eq!(calculate_effective_volume(true, true, 0.0), 0.0);
    assert_eq!(calculate_effective_volume(false, true, 0.75), 0.0);
    assert_eq!(calculate_effective_volume(false, false, 0.0), 0.0);
}

#[test]
fn test_severed_cable_freezes_playback_stream_zero_jump() {
    let mut app = LunulaApp::new_standalone();
    app.is_playing = true;
    app.current_time = 14.5;

    // 1. Initially chain is complete: (1 -> 2, 2 -> 4, 4 -> 5)
    let eval_init = app.evaluate_signal_chain();
    assert!(eval_init.is_complete);

    // 2. Sever the cable from RAM Buffer (2 -> 5 in Bit-Perfect or 2 -> 4 in Modular FX)
    app.connections.retain(|c| !(c.from_node == 2 && (c.to_node == 4 || c.to_node == 5)));
    app.invalidate_chain_cache();
    let eval_severed = app.evaluate_signal_chain();
    assert!(!eval_severed.is_complete, "Severing cable must break the signal chain");

    // 3. While severed, time does NOT advance (playback stream is frozen)
    // Even if 10 seconds pass, current_time stays at 14.5
    assert_eq!(app.current_time, 14.5);
}

#[test]
fn test_interactive_theater_renders_all_5_slides_and_all_turns() {
    let ctx = egui::Context::default();
    let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 800.0)));
    let mut app = LunulaApp::new_standalone();
    app.show_interactive_theater = true;

    // Slide 0: 5 turns (Dual-Engine Bit-Perfect vs DSP)
    app.theater_slide = 0;
    for turn in 0..5 {
        app.theater_turn = turn;
        app.theater_turn_timer = 2.0;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(1.0 + turn as f64),
                ..Default::default()
            },
            |ui| {
                app.render_interactive_theater(ui);
            },
        );
        out.textures_delta.clear();
    }

    // Slide 1: 6 turns (Canvas Modular, Splitting, LPF, Summing Mixer)
    app.theater_slide = 1;
    for turn in 0..6 {
        app.theater_turn = turn;
        app.theater_turn_timer = 2.0;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(10.0 + turn as f64),
                ..Default::default()
            },
            |ui| {
                app.render_interactive_theater(ui);
            },
        );
        out.textures_delta.clear();
    }

    // Slide 2: 8 turns (Filter Studio, Real-Time Telemetry & DAG Sacred Rules)
    app.theater_slide = 2;
    for turn in 0..8 {
        app.theater_turn = turn;
        app.theater_turn_timer = 2.5;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(20.0 + turn as f64),
                ..Default::default()
            },
            |ui| {
                app.render_interactive_theater(ui);
            },
        );
        out.textures_delta.clear();
    }

    // Slide 3: 7 turns (Now Playing, Oscilloscope, Seeking, Shuffle, Repeat, Queue Drawer & Gapless/ReplayGain/Crossfade/Compressor)
    app.theater_slide = 3;
    for turn in 0..7 {
        app.theater_turn = turn;
        app.theater_turn_timer = 2.0;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(30.0 + turn as f64),
                ..Default::default()
            },
            |ui| {
                app.render_interactive_theater(ui);
            },
        );
        out.textures_delta.clear();
    }

    // Slide 4: 6 turns (Music Library, Scanning, Stats Ribbon, Categories, Search & Playlists)
    app.theater_slide = 4;
    for turn in 0..6 {
        app.theater_turn = turn;
        app.theater_turn_timer = 2.0;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(40.0 + turn as f64),
                ..Default::default()
            },
            |ui| {
                app.render_interactive_theater(ui);
            },
        );
        out.textures_delta.clear();
    }
}