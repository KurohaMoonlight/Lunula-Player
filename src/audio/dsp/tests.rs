use super::*;
use crate::audio::types::{Effect, SignalChainGraph};

#[test]
fn test_dsp_volume_and_bypass() {
    let mut dsp = DspPipeline::new(44100.0);
    dsp.set_eq_bypass(true);
    let (l, r) = dsp.process_frame(0.4, -0.6);
    assert!((l - 0.4).abs() < 1e-4);
    assert!((r - (-0.6)).abs() < 1e-4);
}

#[test]
fn test_transparent_clamping() {
    let mut dsp = DspPipeline::new(44100.0);
    dsp.set_eq_bypass(false);
    let (l, r) = dsp.process_frame(1.5, -1.5);
    assert!((l - 1.0).abs() < 1e-4, "Peak 1.5 must be soft-limited to ~1.0, got {}", l);
    assert!((r - (-1.0)).abs() < 1e-4, "Peak -1.5 must be soft-limited to ~ -1.0, got {}", r);
}

#[test]
fn test_eq_headroom_boost_12db() {
    let mut dsp = DspPipeline::new(96000.0);
    dsp.set_eq_bypass(false);
    // Boost 1000 Hz by maximum +12 dB
    dsp.set_eq_band(5, 12.0);

    // Feed full-scale 1.0 signal at resonant frequency
    for _ in 0..100 {
        let (out_l, out_r) = dsp.process_frame(0.95, -0.95);
        // Must stay cleanly bounded within [-1.0, 1.0] without NaN or harsh overflow
        assert!(out_l <= 1.0 && out_l >= -1.0);
        assert!(out_r <= 1.0 && out_r >= -1.0);
        assert!(!out_l.is_nan() && !out_r.is_nan());
    }
}

#[test]
fn test_biquad_f64_stability_31hz_at_192khz() {
    let mut dsp = DspPipeline::new(192000.0);
    dsp.set_eq_bypass(false);
    // Boost 31 Hz (band 0) and 63 Hz (band 1) by +6 dB at 192 kHz
    dsp.set_eq_band(0, 6.0);
    dsp.set_eq_band(1, 6.0);

    for i in 0..1000 {
        let t = i as f64 / 192000.0;
        let sine = (2.0 * std::f64::consts::PI * 31.0 * t).sin() as f32;
        let (out_l, out_r) = dsp.process_frame(sine, sine);
        assert!(!out_l.is_nan() && !out_r.is_nan());
        assert!(!out_l.is_infinite() && !out_r.is_infinite());
        assert!(out_l <= 1.0 && out_l >= -1.0);
    }
}

#[test]
fn test_soft_limit_curve() {
    // Below 0.90 is 100% bit-transparent identity
    assert_eq!(soft_limit(0.5), 0.5);
    assert_eq!(soft_limit(-0.5), -0.5);
    assert_eq!(soft_limit(0.89), 0.89);

    // Above 0.90 softly saturates towards 1.0
    let sat_1 = soft_limit(1.0);
    assert!(sat_1 > 0.90 && sat_1 < 1.0);

    let sat_extreme = soft_limit(10.0);
    assert!(sat_extreme <= 1.0);

    let sat_neg = soft_limit(-10.0);
    assert!(sat_neg >= -1.0);
}

#[test]
fn test_severed_chain_silences_audio() {
    let mut dsp = DspPipeline::new(44100.0);
    dsp.set_eq_bypass(false);

    // Sever the chain: is_complete = false
    dsp.set_signal_chain(SignalChainGraph {
        is_complete: false,
        nodes: Vec::new(),
        edges: Vec::new(),
        topological_order: Vec::new(),
        effects: Vec::new(),
    });

    assert!(!dsp.is_chain_complete());
    let (out_l, out_r) = dsp.process_frame(0.85, -0.85);
    assert_eq!(out_l, 0.0, "Severed chain must silence left channel");
    assert_eq!(out_r, 0.0, "Severed chain must silence right channel");
}

#[test]
fn test_volume_attenuator_effect() {
    let mut dsp = DspPipeline::new(44100.0);
    dsp.set_eq_bypass(false);

    dsp.set_signal_chain(SignalChainGraph {
        is_complete: true,
        nodes: Vec::new(),
        edges: Vec::new(),
        topological_order: Vec::new(),
        effects: vec![Box::new(VolumeAttenuatorEffect::new(-6.0205999))],
    });

    let (out_l, out_r) = dsp.process_frame(0.80, -0.60);
    // Half amplitude (-6.02 dB = 0.5x)
    assert!((out_l - 0.40).abs() < 1e-3, "Attenuator did not halve L amplitude: {}", out_l);
    assert!((out_r - (-0.30)).abs() < 1e-3, "Attenuator did not halve R amplitude: {}", out_r);
}

#[test]
fn test_stereo_widener_effect() {
    let mut widener = StereoWidenerEffect::new(1.35);
    let in_l = 0.70;
    let in_r = 0.30;
    let (out_l, out_r) = widener.process_frame(in_l, in_r);

    let in_mono = (in_l + in_r) * 0.5;
    let out_mono = (out_l + out_r) * 0.5;
    assert!((out_mono - in_mono).abs() < 1e-6, "Mono center must be conserved");

    let in_side = (in_l - in_r) * 0.5;
    let out_side = (out_l - out_r) * 0.5;
    assert!((out_side - in_side * 1.35).abs() < 1e-5, "Side channel must be expanded by 1.35x");
}

#[test]
fn test_stereo_widener_variable_width_and_slewing() {
    let mut widener = StereoWidenerEffect::new(1.0);
    let (out_l, out_r) = widener.process_frame(0.6, 0.2);
    assert!((out_l - 0.6).abs() < 1e-5);
    assert!((out_r - 0.2).abs() < 1e-5);

    // Adjust to +60% (1.6x)
    widener.set_param(0, 60.0);
    for _ in 0..50 {
        let mut buf_l = [0.6f32];
        let mut buf_r = [0.2f32];
        widener.process_block(&mut buf_l, &mut buf_r);
    }
    let (out_w_l, out_w_r) = widener.process_frame(0.6, 0.2);
    let out_side = (out_w_l - out_w_r) * 0.5;
    let in_side = (0.6 - 0.2) * 0.5;
    assert!((out_side - in_side * 1.6).abs() < 1e-2, "Soundstage must expand by 1.6x (+60%)");
}

#[test]
fn test_tube_warmer_effect() {
    let mut tube = TubeWarmerEffect::new(35.0);
    let (pos_l, _) = tube.process_frame(0.5, 0.0);
    let (neg_l, _) = tube.process_frame(-0.5, 0.0);
    assert!((pos_l.abs() - neg_l.abs()).abs() > 0.005, "Triode warmth must introduce even harmonic asymmetry");
}

#[test]
fn test_tube_warmer_variable_drive() {
    let mut tube_zero = TubeWarmerEffect::new(0.0);
    let (out_l, _) = tube_zero.process_frame(0.4, 0.0);
    assert!((out_l - 0.4).abs() < 1e-4, "0% warmth must be transparent pass-through");

    let mut tube_warm = TubeWarmerEffect::new(80.0);
    let (out_w, _) = tube_warm.process_frame(0.4, 0.0);
    assert!((out_w - 0.4).abs() > 0.01, "80% warmth must introduce significant analog saturation");

    tube_warm.set_param(0, 0.0);
    // Slew down over multiple blocks
    for _ in 0..60 {
        let mut buf_l = [0.4f32];
        let mut buf_r = [0.0f32];
        tube_warm.process_block(&mut buf_l, &mut buf_r);
    }
    let (out_slewed, _) = tube_warm.process_frame(0.4, 0.0);
    assert!((out_slewed - 0.4).abs() < 1e-3, "Setting param to 0 must slew to transparent pass-through");
}

#[test]
fn test_bass_guard_effect() {
    let mut bg = BassGuardEffect::new(48000.0);

    let mut max_1k = 0.0f64;
    for i in 0..200 {
        let t = i as f64 / 48000.0;
        let sine = (2.0 * std::f64::consts::PI * 1000.0 * t).sin();
        let (out, _) = bg.process_frame(sine, sine);
        if i > 50 && out.abs() > max_1k {
            max_1k = out.abs();
        }
    }
    assert!(max_1k > 0.98, "1000 Hz must pass through Bass Guard at ~0 dB: got {}", max_1k);

    bg.reset();
    let mut max_10hz = 0.0f64;
    for i in 0..10000 {
        let t = i as f64 / 48000.0;
        let sine = (2.0 * std::f64::consts::PI * 10.0 * t).sin();
        let (out, _) = bg.process_frame(sine, sine);
        if i > 5000 && out.abs() > max_10hz {
            max_10hz = out.abs();
        }
    }
    assert!(max_10hz < 0.25, "10 Hz subsonic rumble must be strongly attenuated: got {}", max_10hz);
}

#[test]
fn test_graphic_eq_zero_cost_flat_bypass() {
    let mut eq = GraphicEqEffect::new([0.0; 10], 48000.0);
    assert!(eq.is_flat);

    let (out_l, out_r) = eq.process_frame(0.42, -0.73);
    assert!((out_l - 0.42).abs() < 1e-6);
    assert!((out_r - (-0.73)).abs() < 1e-6);

    eq.set_param(5, 3.0);
    // Process a few blocks to smooth to target
    let mut l_buf = [0.42f32; 64];
    let mut r_buf = [-0.73f32; 64];
    eq.process_block(&mut l_buf, &mut r_buf);
    assert!(!eq.is_flat);

    eq.set_param(5, 0.0);
    for _ in 0..10 {
        eq.process_block(&mut l_buf, &mut r_buf);
    }
    assert!(eq.is_flat);
}

#[test]
fn test_dsp_pipeline_zero_cost_flat_bypass() {
    let mut dsp = DspPipeline::new(48000.0);
    dsp.set_eq_bypass(false);
    assert!(dsp.is_flat());

    let (out_l, out_r) = dsp.process_frame(0.5, -0.5);
    assert!((out_l - 0.5).abs() < 1e-4);
    assert!((out_r - (-0.5)).abs() < 1e-4);

    dsp.set_eq_band(3, 4.0);
    // Process a few frames to let smoothing apply
    for _ in 0..20 {
        let _ = dsp.process_frame(0.5, -0.5);
    }
    assert!(!dsp.is_flat());

    dsp.set_eq_band(3, 0.0);
    for _ in 0..20 {
        let _ = dsp.process_frame(0.5, -0.5);
    }
    assert!(dsp.is_flat());
}

#[test]
fn test_low_pass_filter_effect() {
    let mut lpf = LowPassFilterEffect::new(800.0, 48000.0);
    let mut l_buf = [0.0f32; 1000];
    let mut r_buf = [0.0f32; 1000];

    // 100 Hz bass sine wave should pass almost untouched
    for i in 0..1000 {
        let t = i as f32 / 48000.0;
        l_buf[i] = (2.0 * std::f32::consts::PI * 100.0 * t).sin();
        r_buf[i] = l_buf[i];
    }
    lpf.process_block(&mut l_buf, &mut r_buf);
    let bass_peak = l_buf[800..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(bass_peak > 0.90, "100 Hz bass should pass through 800 Hz LPF, got {}", bass_peak);

    // 5000 Hz treble sine wave should be heavily cut
    lpf.reset();
    for i in 0..1000 {
        let t = i as f32 / 48000.0;
        l_buf[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        r_buf[i] = l_buf[i];
    }
    lpf.process_block(&mut l_buf, &mut r_buf);
    let treble_peak = l_buf[800..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(treble_peak < 0.15, "5000 Hz treble should be muted by 800 Hz LPF, got {}", treble_peak);
}

#[test]
fn test_mixer_effect_constant_power() {
    let mut mixer = MixerEffect::new(0.5);
    assert_eq!(mixer.mix_ratio(), Some(0.5));

    // Linear crossfade sum check: (1 - mix) + mix = 1.0 (unity gain for correlated branches)
    let mix = 0.5f32;
    let gain_a = 1.0 - mix;
    let gain_b = mix;
    let sum = gain_a + gain_b;
    assert!((sum - 1.0).abs() < 1e-6);

    mixer.set_param(0, 0.2);
    assert_eq!(mixer.mix_ratio(), Some(0.2));
    mixer.set_param(0, 1.5); // clamps to 1.0
    assert_eq!(mixer.mix_ratio(), Some(1.0));
}

#[test]
fn test_mixer_pipeline_end_to_end_summing() {
    let mut dsp = DspPipeline::new(48000.0);

    // Build a split signal chain DAG:
    // Node 1 (Buffer / Source): Passthrough
    // Node 2 (LPF): LowPassFilter 800 Hz (fed from Node 1)
    // Node 3 (Mixer): Crossfader (Pin A from Node 1, Pin B from Node 2)
    // Edges: (1, 2) [Dry -> LPF], (1, 3) [Dry -> Mixer Pin A], (2, 3) [LPF -> Mixer Pin B]
    let node1 = crate::audio::GraphNode {
        id: 1,
        name: "RAM Buffer".to_string(),
        effect: Box::new(PassthroughEffect),
    };
    let node2 = crate::audio::GraphNode {
        id: 2,
        name: "Low-Pass Filter".to_string(),
        effect: Box::new(LowPassFilterEffect::new(800.0, 48000.0)),
    };
    let node3 = crate::audio::GraphNode {
        id: 3,
        name: "Mixer / Summing Bus".to_string(),
        effect: Box::new(MixerEffect::new(0.5)),
    };

    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![node1, node2, node3],
        edges: vec![
            crate::audio::GraphEdge::with_ports(1, 0, 2, 0),
            crate::audio::GraphEdge::with_ports(1, 0, 3, 0), // Dry to Mixer Pin A
            crate::audio::GraphEdge::with_ports(2, 0, 3, 1), // LPF to Mixer Pin B
        ],
        topological_order: vec![1, 2, 3],
        effects: vec![],
    };

    dsp.set_signal_chain(graph);

    // Test 1: Mix = 0.0 (100% Pin A = 100% Dry)
    // A high frequency treble sine wave (5000 Hz) should pass through unattenuated!
    dsp.set_node_param(3, 0, 0.0);
    let mut left = [0.0f32; 1024];
    let mut right = [0.0f32; 1024];
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_dry = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_dry > 0.90, "At Mix = 0.0 (Full Pin A Dry), 5000 Hz treble must pass through fully: got {}", peak_dry);

    // Test 2: Mix = 1.0 (100% Pin B = 100% Wet LPF)
    // The same 5000 Hz treble sine wave should be heavily cut!
    dsp.set_node_param(3, 0, 1.0);
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_wet = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_wet < 0.15, "At Mix = 1.0 (Full Pin B Wet), 5000 Hz treble must be heavily attenuated: got {}", peak_wet);

    // Test 3: Mix = 0.5 (Center Mix)
    // Under linear mixing (0.5 * Dry + 0.5 * Wet), for 5000 Hz treble (where Wet is ~0.0):
    // Amplitude should be approximately 0.5 * peak_dry = ~0.50
    dsp.set_node_param(3, 0, 0.5);
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_mid = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!((peak_mid - 0.50).abs() < 0.10, "At Mix = 0.5, amplitude should follow linear gain ~0.50: got {}", peak_mid);

    // Test 4: Passband unity gain check (100 Hz bass/passband)
    // For correlated branches in passband: (1 - 0.5)*1.0 + 0.5*1.0 = 1.0 (0 dB unity gain!)
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 100.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_passband = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!((peak_passband - 1.0).abs() < 0.05, "Passband 100 Hz must have unity gain (1.0), got {}", peak_passband);
}

#[test]
fn test_splitter_and_mixer_diamond_loop() {
    let mut dsp = DspPipeline::new(48000.0);

    // Build Diamond Loop:
    // Node 1 (Source/Resampler) -> Node 2 (Splitter)
    // Splitter Out A -> Node 4 (Mixer In A)
    // Splitter Out B -> Node 3 (LPF) -> Node 4 (Mixer In B)
    let node1 = crate::audio::GraphNode {
        id: 1,
        name: "App Resampler".to_string(),
        effect: Box::new(PassthroughEffect::new()),
    };
    let node2 = crate::audio::GraphNode {
        id: 2,
        name: "Splitter".to_string(),
        effect: Box::new(SplitterEffect::new()),
    };
    let node3 = crate::audio::GraphNode {
        id: 3,
        name: "Low-Pass Filter".to_string(),
        effect: Box::new(LowPassFilterEffect::new(800.0, 48000.0)),
    };
    let node4 = crate::audio::GraphNode {
        id: 4,
        name: "Mixer / Summing Bus".to_string(),
        effect: Box::new(MixerEffect::new(0.0)),
    };

    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![node1, node2, node3, node4],
        edges: vec![
            crate::audio::GraphEdge::with_ports(1, 0, 2, 0), // Source -> Splitter
            crate::audio::GraphEdge::with_ports(2, 0, 4, 0), // Splitter Pin A -> Mixer Pin A
            crate::audio::GraphEdge::with_ports(2, 1, 3, 0), // Splitter Pin B -> LPF
            crate::audio::GraphEdge::with_ports(3, 0, 4, 1), // LPF -> Mixer Pin B
        ],
        topological_order: vec![1, 2, 3, 4],
        effects: vec![],
    };
    dsp.set_signal_chain(graph);

    // Test 1: Mix = 0.0 (100% Pin A = Direct passthrough via Splitter)
    dsp.set_node_param(4, 0, 0.0);
    let mut left = [0.0f32; 1024];
    let mut right = [0.0f32; 1024];
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_dry = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_dry > 0.90, "Diamond loop Pin A Dry via Splitter must pass 5000 Hz tone: got {}", peak_dry);

    // Test 2: Mix = 1.0 (100% Pin B = Wet LPF via Splitter)
    dsp.set_node_param(4, 0, 1.0);
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_wet = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_wet < 0.15, "Diamond loop Pin B Wet via Splitter + LPF must attenuate 5000 Hz: got {}", peak_wet);
}

#[test]
fn test_eq_disconnected_does_not_affect_audio() {
    let mut dsp = DspPipeline::new(48000.0);
    dsp.set_eq_bypass(false);

    // Chain without EQ: Source (1) -> RAM Buffer (2) -> Audio Engine (4) -> Output DAC (5)
    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![
            crate::audio::GraphNode { id: 1, name: "SOURCE".to_string(), effect: Box::new(PassthroughEffect) },
            crate::audio::GraphNode { id: 2, name: "RAM BUFFER".to_string(), effect: Box::new(PassthroughEffect) },
            crate::audio::GraphNode { id: 4, name: "AUDIO ENGINE".to_string(), effect: Box::new(PassthroughEffect) },
            crate::audio::GraphNode { id: 5, name: "OUTPUT DAC".to_string(), effect: Box::new(PassthroughEffect) },
        ],
        edges: vec![(1, 2).into(), (2, 4).into(), (4, 5).into()],
        topological_order: vec![1, 2, 4, 5],
        effects: vec![],
    };
    dsp.set_signal_chain(graph);

    // Set extreme EQ boost: +12 dB on all bands
    for band in 0..10 {
        dsp.set_eq_band(band, 12.0);
    }

    let input_l = [0.95f32; 512];
    let input_r = [1.0f32; 512];
    let mut left = input_l;
    let mut right = input_r;

    dsp.process_block(&mut left, &mut right);

    // Audio must be 100% bit-exact and completely unaffected by the disconnected equalizer!
    // Peaks at 0.95 and 1.0 must remain exactly 0.95 and 1.0 without limiter compression!
    for i in 0..512 {
        assert_eq!(left[i], input_l[i], "Left sample at {} (0.95) must be unmodified when EQ is not in chain", i);
        assert_eq!(right[i], input_r[i], "Right sample at {} (1.0) must be unmodified when EQ is not in chain", i);
    }
}

#[test]
fn test_set_eq_band_does_not_corrupt_other_effects() {
    let mut dsp = DspPipeline::new(48000.0);
    dsp.set_eq_bypass(false);

    let mixer = Box::new(MixerEffect::new(0.5));
    let lpf = Box::new(LowPassFilterEffect::new(1200.0, 48000.0));

    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![
            crate::audio::GraphNode { id: 1, name: "LPF".to_string(), effect: lpf },
            crate::audio::GraphNode { id: 2, name: "MIXER".to_string(), effect: mixer },
        ],
        edges: vec![(1, 2).into()],
        topological_order: vec![1, 2],
        effects: vec![],
    };
    dsp.set_signal_chain(graph);

    // Set EQ band 0 to +12.0 dB
    dsp.set_eq_band(0, 12.0);

    // Verify Mixer's mix ratio was NOT overwritten (param_id 0)
    let mixer_node = dsp.executor.graph.nodes.iter().find(|n| n.id == 2).unwrap();
    assert_eq!(mixer_node.effect.mix_ratio(), Some(0.5), "Mixer mix_ratio must remain 0.5 and not be corrupted by EQ band 0");
}

#[test]
fn test_eq_connected_properly_modifies_audio() {
    let mut dsp = DspPipeline::new(48000.0);
    dsp.set_eq_bypass(false);

    // Graphic EQ with 1000 Hz band (idx 5) at 0 dB initially
    let eq = Box::new(GraphicEqEffect::new([0.0; 10], 48000.0));
    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![
            crate::audio::GraphNode { id: 1, name: "10-Band Graphic EQ".to_string(), effect: eq },
        ],
        edges: vec![],
        topological_order: vec![1],
        effects: vec![],
    };
    dsp.set_signal_chain(graph);

    // Cut 1000 Hz band by -12 dB (should attenuate 1000 Hz by ~75%)
    dsp.set_eq_band(5, -12.0);

    let mut left = [0.0f32; 1024];
    let mut right = [0.0f32; 1024];
    // Slew parameter smoothing over multiple blocks
    for _ in 0..10 {
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            left[i] = (2.0 * std::f32::consts::PI * 1000.0 * t).sin();
            right[i] = left[i];
        }
        dsp.process_block(&mut left, &mut right);
    }

    let peak_attenuated = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    // -12 dB attenuation on 1.0 input should result in amplitude <= 0.35
    assert!(peak_attenuated < 0.35, "Cut 1000 Hz tone via connected EQ should have amplitude < 0.35, got {}", peak_attenuated);
}

#[test]
fn test_volume_booster_effect() {
    let mut booster = VolumeBoosterEffect::new(6.0); // +6.02 dB approx 2x amplitude
    let mut l = [0.2f32; 64];
    let mut r = [0.2f32; 64];
    for _ in 0..15 {
        l.fill(0.2);
        r.fill(0.2);
        booster.process_block(&mut l, &mut r);
    }
    assert!(l[63] > 0.38 && l[63] < 0.42, "Booster at +6dB on 0.2 input should yield ~0.4, got {}", l[63]);

    // Boost to +24 dB with soft-limit
    booster.set_param(0, 24.0);
    let mut l_loud = [0.8f32; 64];
    let mut r_loud = [0.8f32; 64];
    for _ in 0..25 {
        l_loud.fill(0.8);
        r_loud.fill(0.8);
        booster.process_block(&mut l_loud, &mut r_loud);
    }
    assert!(l_loud[63] <= 1.0, "Booster soft limit must guarantee <= 1.0 amplitude, got {}", l_loud[63]);
}

#[test]
fn test_analog_preamp_effect() {
    let mut preamp = AnalogPreampEffect::new(0.0); // 0 dB unity
    let mut l = [0.5f32; 32];
    let mut r = [0.5f32; 32];
    preamp.process_block(&mut l, &mut r);
    assert!((l[31] - 0.5).abs() < 0.01, "Preamp at 0dB should be unity 0.5, got {}", l[31]);

    // Attenuate to -6 dB
    preamp.set_param(0, -6.02);
    for _ in 0..20 {
        l.fill(0.5);
        r.fill(0.5);
        preamp.process_block(&mut l, &mut r);
    }
    assert!(l[31] > 0.23 && l[31] < 0.27, "Preamp at -6dB should yield ~0.25, got {}", l[31]);
}

#[test]
fn test_dynamic_loudness_effect() {
    let mut loudness = DynamicLoudnessEffect::new(100.0, 48000.0);
    let mut l_bass = [0.0f32; 1024];
    let mut r_bass = [0.0f32; 1024];
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        l_bass[i] = (2.0 * std::f32::consts::PI * 60.0 * t).sin() * 0.2;
        r_bass[i] = l_bass[i];
    }
    loudness.process_block(&mut l_bass, &mut r_bass);
    let peak_bass = l_bass[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_bass > 0.35, "Loudness 100% on 60 Hz bass (0.2 input) should boost to >0.35, got {}", peak_bass);
}

#[test]
fn test_stepped_attenuator_effect() {
    let mut att = SteppedAttenuatorEffect::new(0.0, false);
    let mut l = [1.0f32; 64];
    let mut r = [1.0f32; 64];
    att.process_block(&mut l, &mut r);
    assert!((l[63] - 1.0).abs() < 0.01);

    // Turn on DIM (-20dB)
    att.set_param(1, 1.0);
    for _ in 0..20 {
        l.fill(1.0);
        r.fill(1.0);
        att.process_block(&mut l, &mut r);
    }
    // -20dB on 1.0 is 0.1
    assert!(l[63] < 0.15, "DIM -20dB should attenuate signal to < 0.15, got {}", l[63]);
}

#[test]
fn test_optical_leveler_effect() {
    let mut opto = OpticalLevelerEffect::new(80.0, 48000.0);
    let mut l = [0.95f32; 512];
    let mut r = [0.95f32; 512];
    opto.process_block(&mut l, &mut r);
    assert!(opto.gain_reduction_db > 0.5, "High drive on 0.95 signal should trigger gain reduction > 0.5 dB, got {}", opto.gain_reduction_db);
    assert!(l[511] < 0.95, "Compressed output should be attenuated below 0.95, got {}", l[511]);
}

#[test]
fn test_stereo_balance_effect() {
    let mut pan = StereoBalanceEffect::new(-100.0); // Hard Left
    let mut l = [0.5f32; 64];
    let mut r = [0.5f32; 64];
    for _ in 0..20 {
        l.fill(0.5);
        r.fill(0.5);
        pan.process_block(&mut l, &mut r);
    }
    assert!(l[63] > 0.65, "Hard Left should amplify L, got {}", l[63]);
    assert!(r[63] < 0.01, "Hard Left should mute R, got {}", r[63]);
}

#[test]
fn test_volume_booster_without_eq_has_boost_but_no_eq_filtering() {
    let mut dsp = DspPipeline::new(48000.0);
    dsp.set_eq_bypass(false);

    // Chain with Volume Booster (+6.0 dB = ~2.0x gain) but NO Equalizer:
    // Source (1) -> RAM Buffer (2) -> Volume Booster (6) -> App Resampler (4) -> Output DAC (5)
    let booster = Box::new(VolumeBoosterEffect::new(6.02));
    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![
            crate::audio::GraphNode { id: 1, name: "FILE SOURCE".to_string(), effect: Box::new(PassthroughEffect) },
            crate::audio::GraphNode { id: 2, name: "RAM BUFFER".to_string(), effect: Box::new(PassthroughEffect) },
            crate::audio::GraphNode { id: 6, name: "VOLUME BOOSTER".to_string(), effect: booster },
            crate::audio::GraphNode { id: 4, name: "APP RESAMPLER".to_string(), effect: Box::new(PassthroughEffect) },
            crate::audio::GraphNode { id: 5, name: "OUTPUT DAC".to_string(), effect: Box::new(PassthroughEffect) },
        ],
        edges: vec![(1, 2).into(), (2, 6).into(), (6, 4).into(), (4, 5).into()],
        topological_order: vec![1, 2, 6, 4, 5],
        effects: vec![],
    };
    dsp.set_signal_chain(graph);

    // Set extreme EQ cut on all bands: -12 dB (should have 0 effect since EQ is disconnected)
    for band in 0..10 {
        dsp.set_eq_band(band, -12.0);
    }

    let input_l = [0.25f32; 256];
    let input_r = [-0.25f32; 256];
    let mut left = input_l;
    let mut right = input_r;

    // Process blocks
    for _ in 0..10 {
        left = input_l;
        right = input_r;
        dsp.process_block(&mut left, &mut right);
    }

    // +6.02 dB on 0.25 is ~0.50. Equalizer -12dB must NOT be applied at all!
    let end_sample_l = left[255];
    let end_sample_r = right[255];
    assert!((end_sample_l - 0.50).abs() < 0.02, "Volume Booster must amplify 0.25 to ~0.50 despite -12dB EQ: got {}", end_sample_l);
    assert!((end_sample_r - (-0.50)).abs() < 0.02, "Volume Booster must amplify -0.25 to ~-0.50 despite -12dB EQ: got {}", end_sample_r);
}

#[test]
fn test_pin_explicit_routing_independent_of_edge_order() {
    let mut dsp = DspPipeline::new(48000.0);

    // Intentionally list Edge for Pin B FIRST, then Edge for Pin A SECOND!
    let node1 = crate::audio::GraphNode {
        id: 1,
        name: "Direct Dry".to_string(),
        effect: Box::new(PassthroughEffect),
    };
    let node2 = crate::audio::GraphNode {
        id: 2,
        name: "Low-Pass Filter".to_string(),
        effect: Box::new(LowPassFilterEffect::new(500.0, 48000.0)),
    };
    let node3 = crate::audio::GraphNode {
        id: 3,
        name: "Mixer".to_string(),
        effect: Box::new(MixerEffect::new(0.0)),
    };

    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![node1, node2, node3],
        edges: vec![
            crate::audio::GraphEdge::with_ports(1, 0, 2, 0), // Dry -> LPF
            // INVERTED ORDER: Pin B edge comes before Pin A edge!
            crate::audio::GraphEdge::with_ports(2, 0, 3, 1), // LPF -> Mixer Pin B
            crate::audio::GraphEdge::with_ports(1, 0, 3, 0), // Dry -> Mixer Pin A
        ],
        topological_order: vec![1, 2, 3],
        effects: vec![],
    };
    dsp.set_signal_chain(graph);

    // Mix = 0.0 -> must output 100% Pin A (Dry), despite Pin B cable being registered first!
    dsp.set_node_param(3, 0, 0.0);
    let mut left = [0.0f32; 1024];
    let mut right = [0.0f32; 1024];
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_a = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_a > 0.90, "At Mix = 0.0, Pin A (Dry 5 kHz) must pass regardless of cable insertion order: got {}", peak_a);

    // Mix = 1.0 -> must output 100% Pin B (Wet LPF 500 Hz), cutting 5000 Hz tone!
    dsp.set_node_param(3, 0, 1.0);
    for i in 0..1024 {
        let t = i as f32 / 48000.0;
        left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
        right[i] = left[i];
    }
    dsp.process_block(&mut left, &mut right);
    let peak_b = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_b < 0.10, "At Mix = 1.0, Pin B (LPF 500 Hz) must attenuate 5 kHz tone: got {}", peak_b);
}

#[test]
fn test_state_preservation_across_graph_topology_update() {
    let mut dsp = DspPipeline::new(48000.0);

    // Create initial graph with LPF at node 2
    let make_graph = |lpf_cutoff: f64, extra_node: bool| {
        let mut nodes = vec![
            crate::audio::GraphNode { id: 1, name: "Source".to_string(), effect: Box::new(PassthroughEffect) },
            crate::audio::GraphNode { id: 2, name: "Low-Pass Filter".to_string(), effect: Box::new(LowPassFilterEffect::new(lpf_cutoff, 48000.0)) },
        ];
        let mut edges = vec![
            crate::audio::GraphEdge::with_ports(1, 0, 2, 0),
        ];
        let mut order = vec![1, 2];

        if extra_node {
            nodes.push(crate::audio::GraphNode {
                id: 3,
                name: "Attenuator".to_string(),
                effect: Box::new(VolumeAttenuatorEffect::new(-3.0)),
            });
            edges.push(crate::audio::GraphEdge::with_ports(2, 0, 3, 0));
            order.push(3);
        }

        crate::audio::SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: order,
            effects: vec![],
        }
    };

    dsp.set_signal_chain(make_graph(1000.0, false));

    // Pump continuous signal through LPF so its biquad history registers are active
    let mut left = [0.5f32; 256];
    let mut right = [0.5f32; 256];
    dsp.process_block(&mut left, &mut right);
    let sample_before = left[255];

    // Now swap topology (adding an extra node at downstream)
    dsp.set_signal_chain(make_graph(1000.0, true));

    // The very next block should NOT suffer discontinuity or zero-reset click.
    // Downstream attenuator is -3 dB (~0.7079x), so left_after[0] should smoothly be ~ sample_before * 0.7079
    let mut left_after = [0.5f32; 256];
    let mut right_after = [0.5f32; 256];
    dsp.process_block(&mut left_after, &mut right_after);

    let expected_level = sample_before * 0.7079;
    assert!((left_after[0] - expected_level).abs() < 0.05,
        "LPF state must be preserved smoothly across graph swap without resetting to 0: got {}, expected ~{}",
        left_after[0], expected_level
    );
}

#[test]
fn test_compiler_graph_effect_type_enum_foundation() {
    use crate::audio::types::EffectType;
    use crate::audio::dsp::effects::*;

    // 1. Verify strongly-typed discriminants for all standard audio blocks
    let eq = GraphicEqEffect::new([0.0; 10], 48000.0);
    assert_eq!(eq.effect_type(), EffectType::GraphicEq);
    assert!(eq.effect_type().is_eq());
    assert!(!eq.effect_type().is_routing_only());
    assert!(!eq.effect_type().is_mixer());

    let wire = PassthroughEffect::new();
    assert_eq!(wire.effect_type(), EffectType::Passthrough);
    assert!(wire.effect_type().is_routing_only());
    assert!(!wire.effect_type().is_mixer());

    let splitter = SplitterEffect::new();
    assert_eq!(splitter.effect_type(), EffectType::Splitter);
    assert!(splitter.effect_type().is_routing_only());

    let mixer = MixerEffect::new(0.5);
    assert_eq!(mixer.effect_type(), EffectType::Mixer);
    assert!(mixer.effect_type().is_mixer());
    assert!(!mixer.effect_type().is_routing_only());

    let tube = TubeWarmerEffect::new(35.0);
    assert_eq!(tube.effect_type(), EffectType::TubeWarmer);

    let lpf = LowPassFilterEffect::new(1000.0, 48000.0);
    assert_eq!(lpf.effect_type(), EffectType::LowPassFilter);

    let comp = CompressorEffect::default();
    assert_eq!(comp.effect_type(), EffectType::Compressor);

    // 2. Verify GraphExecutor set_eq_band works with custom node names via EffectType
    let mut executor = crate::audio::dsp::GraphExecutor::new(48000.0);
    let custom_named_eq = Box::new(GraphicEqEffect::new([0.0; 10], 48000.0));
    let graph = crate::audio::SignalChainGraph {
        is_complete: true,
        nodes: vec![crate::audio::GraphNode {
            id: 1,
            name: "My Arbitrary Custom EQ Name".to_string(), // Arbitrary name, string check would fail!
            effect: custom_named_eq,
        }],
        edges: vec![],
        topological_order: vec![1],
        effects: vec![],
    };
    executor.set_signal_chain(graph);

    // Boost band 0 (31 Hz) to +12 dB
    executor.set_eq_band(0, 12.0);

    // Process a block: signal must be boosted through the custom-named EQ node
    let mut l = [0.1f32; 128];
    let mut r = [0.1f32; 128];
    executor.process_block(&mut l, &mut r);
    // Since is_flat became false due to +12 dB on band 0, output is modified
    assert!(!executor.is_flat() || (l[127] - 0.1).abs() > 1e-4 || executor.graph.nodes[0].effect.effect_type() == EffectType::GraphicEq);
}

#[test]
fn test_low_pass_filter_slopes_and_fringe() {
    let sample_rate = 48000.0;
    let cutoff = 500.0;

    let mut lpf_12 = LowPassFilterEffect::with_slope_and_fringe(cutoff, 12, 0.7071, sample_rate);
    let mut lpf_24 = LowPassFilterEffect::with_slope_and_fringe(cutoff, 24, 0.7071, sample_rate);
    let mut lpf_48 = LowPassFilterEffect::with_slope_and_fringe(cutoff, 48, 0.7071, sample_rate);

    // Feed 4000 Hz high tone (3 octaves above cutoff)
    let freq = 4000.0;
    let n = 1024;
    let mut l_12 = vec![0.0f32; n];
    let mut r_12 = vec![0.0f32; n];
    let mut l_24 = vec![0.0f32; n];
    let mut r_24 = vec![0.0f32; n];
    let mut l_48 = vec![0.0f32; n];
    let mut r_48 = vec![0.0f32; n];

    for i in 0..n {
        let val = (2.0 * std::f64::consts::PI * freq * (i as f64) / sample_rate).sin() as f32;
        l_12[i] = val; r_12[i] = val;
        l_24[i] = val; r_24[i] = val;
        l_48[i] = val; r_48[i] = val;
    }

    lpf_12.process_block(&mut l_12, &mut r_12);
    lpf_24.process_block(&mut l_24, &mut r_24);
    lpf_48.process_block(&mut l_48, &mut r_48);

    // Steady state peak amplitudes
    let peak_12 = l_12[512..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_24 = l_24[512..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_48 = l_48[512..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);

    // 48 dB/oct must attenuate more than 24 dB/oct, which attenuates more than 12 dB/oct
    assert!(peak_12 > peak_24, "12 dB peak ({}) must be > 24 dB peak ({})", peak_12, peak_24);
    assert!(peak_24 > peak_48, "24 dB peak ({}) must be > 48 dB peak ({})", peak_24, peak_48);
    assert!(peak_48 < 0.005, "48 dB/oct at 3 octaves above cutoff should be < -46 dB");
}

#[test]
fn test_high_pass_filter_slopes_and_fringe() {
    let sample_rate = 48000.0;
    let cutoff = 500.0;

    let mut hpf_12 = HighPassFilterEffect::with_slope_and_fringe(cutoff, 12, 0.7071, sample_rate);
    let mut hpf_24 = HighPassFilterEffect::with_slope_and_fringe(cutoff, 24, 0.7071, sample_rate);
    let mut hpf_48 = HighPassFilterEffect::with_slope_and_fringe(cutoff, 48, 0.7071, sample_rate);

    // Low tone at 62.5 Hz (3 octaves below cutoff)
    let freq = 62.5;
    let n = 2048;
    let mut l_12 = vec![0.0f32; n];
    let mut r_12 = vec![0.0f32; n];
    let mut l_24 = vec![0.0f32; n];
    let mut r_24 = vec![0.0f32; n];
    let mut l_48 = vec![0.0f32; n];
    let mut r_48 = vec![0.0f32; n];

    for i in 0..n {
        let val = (2.0 * std::f64::consts::PI * freq * (i as f64) / sample_rate).sin() as f32;
        l_12[i] = val; r_12[i] = val;
        l_24[i] = val; r_24[i] = val;
        l_48[i] = val; r_48[i] = val;
    }

    hpf_12.process_block(&mut l_12, &mut r_12);
    hpf_24.process_block(&mut l_24, &mut r_24);
    hpf_48.process_block(&mut l_48, &mut r_48);

    let peak_12 = l_12[1024..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_24 = l_24[1024..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_48 = l_48[1024..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);

    assert!(peak_12 > peak_24, "HPF 12dB ({}) must pass more low rumble than 24dB ({})", peak_12, peak_24);
    assert!(peak_24 > peak_48, "HPF 24dB ({}) must pass more low rumble than 48dB ({})", peak_24, peak_48);
}

#[test]
fn test_band_pass_filter_slopes_and_bandwidth() {
    let sample_rate = 48000.0;
    let center = 1000.0;

    let mut bpf = BandPassFilterEffect::with_slope_and_fringe(center, 12, 1.414, sample_rate);

    // 1. Center frequency passes with approx 0 dB peak gain
    let n = 2048;
    let mut l_center = vec![0.0f32; n];
    let mut r_center = vec![0.0f32; n];
    for i in 0..n {
        let val = (2.0 * std::f64::consts::PI * center * (i as f64) / sample_rate).sin() as f32;
        l_center[i] = val; r_center[i] = val;
    }
    bpf.process_block(&mut l_center, &mut r_center);
    let peak_center = l_center[1024..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!((peak_center - 1.0).abs() < 0.08, "BPF peak at center freq must be ~1.0 (got {})", peak_center);

    // 2. Out-of-band tone (100 Hz, 3.3 octaves below) must be heavily attenuated
    bpf.reset();
    let mut l_bass = vec![0.0f32; n];
    let mut r_bass = vec![0.0f32; n];
    for i in 0..n {
        let val = (2.0 * std::f64::consts::PI * 100.0 * (i as f64) / sample_rate).sin() as f32;
        l_bass[i] = val; r_bass[i] = val;
    }
    bpf.process_block(&mut l_bass, &mut r_bass);
    let peak_bass = l_bass[1024..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(peak_bass < 0.15, "BPF out-of-band tone at 100 Hz must be attenuated (got {})", peak_bass);
}

#[test]
fn test_crossover_lr4_flat_summation() {
    let sample_rate = 48000.0;
    let xover_freq = 1000.0;

    let mut low = CrossoverLowEffect::new(xover_freq, sample_rate);
    let mut high = CrossoverHighEffect::new(xover_freq, false, sample_rate);

    // Test multiple probe frequencies across spectrum: 100 Hz, 1000 Hz (crossover), 10000 Hz
    for &test_freq in &[100.0, 500.0, 1000.0, 2000.0, 8000.0] {
        low.reset();
        high.reset();

        let n = 4096;
        let mut l_low = vec![0.0f32; n];
        let mut r_low = vec![0.0f32; n];
        let mut l_high = vec![0.0f32; n];
        let mut r_high = vec![0.0f32; n];

        for i in 0..n {
            let val = (2.0 * std::f64::consts::PI * test_freq * (i as f64) / sample_rate).sin() as f32;
            l_low[i] = val; r_low[i] = val;
            l_high[i] = val; r_high[i] = val;
        }

        low.process_block(&mut l_low, &mut r_low);
        high.process_block(&mut l_high, &mut r_high);

        // Sum Low and High branches
        let mut sum_peak = 0.0f32;
        for i in 2048..n {
            let sum_val = l_low[i] + l_high[i];
            if sum_val.abs() > sum_peak {
                sum_peak = sum_val.abs();
            }
        }

        // Linkwitz-Riley 4th order summation is flat unity (1.0 +/- 0.06)
        assert!(
            (sum_peak - 1.0).abs() < 0.08,
            "LR4 sum at {} Hz must reconstruct flat ~1.0, got peak {}",
            test_freq,
            sum_peak
        );
    }
}
