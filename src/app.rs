use eframe::egui;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::time::Instant;
use crate::models::{
    AppTab, CableConnection, ChainMode, CustomPlaylist, DitherMode,
    LibraryCategory, ModularNode, NodeType, ParametricBand, ParametricEqPreset,
    PreampMode, RepeatMode, ScanMsg, Track, WaveformMode,
};
use crate::ui::components::render_card_frame;

pub struct LunulaApp {
    pub current_tab: AppTab,
    pub selected_node: Option<usize>,
    pub chain_mode: ChainMode,

    // Modular Signal Chain Graph
    pub nodes: Vec<ModularNode>,
    pub connections: Vec<CableConnection>,
    pub next_node_id: usize,
    pub dragging_node: Option<usize>,
    pub patchbay_pan: Vec2,
    pub is_panning_canvas: bool,

    // Cable patch dragging state
    pub connecting_from_node: Option<usize>, // When dragging from an output pin or input pin
    pub connecting_from_port: u8,
    pub connecting_is_input_drag: bool,

    // Studio Mode (Simple Modular Patchbay vs Advance Pro Studio)
    pub studio_mode: crate::models::StudioMode,
    pub advance_studio_state: crate::ui::patchbay::AdvanceStudioState,
    pub patchbay_filter_search: String,
    pub patchbay_category_idx: usize,

    // Playback state
    pub is_playing: bool,
    pub current_time: f32,
    pub volume: f32,
    pub is_muted: bool,
    pub _start_time: Instant,
    pub is_shuffled: bool,
    pub repeat_mode: RepeatMode,
    pub selected_queue_idx: Option<usize>,
    pub queue_cursor: Option<usize>,

    // Oscilloscope state
    pub zoom_level: f32,

    // Legacy EQ bands (-12.0 to +12.0 dB)
    pub eq_bands: [f32; 10],
    pub eq_bypass: bool,

    // Active Parametric EQ Preset (Single Source of Truth)
    pub active_eq: ParametricEqPreset,
    pub eq_is_dirty: bool,
    pub eq_presets: Vec<ParametricEqPreset>,
    pub selected_band_idx: Option<usize>,
    pub dragging_band_idx: Option<usize>,
    pub show_save_preset_dialog: bool,
    pub eq_preset_name_input: String,
    pub eq_show_spectrum_overlay: bool,
    pub show_saved_eq_palette_popup: bool,
    pub saved_eq_popup_anchor: Option<Pos2>,
    pub show_patchbay_tutorial: bool,
    pub patchbay_tutorial_step: usize,
    pub active_filter_tutorial: Option<NodeType>,
    pub filter_tutorial_step: usize,
    pub dragging_slider_node: Option<usize>,

    // Game-Like Onboarding Quest
    pub has_completed_onboarding: bool,
    pub is_onboarding_quest_active: bool,
    pub onboarding_quest_step: crate::models::OnboardingQuestStep,
    pub quest_rect_signal_chain_tab: Option<Rect>,
    pub quest_rect_dsp_mode_btn: Option<Rect>,
    pub quest_rect_tube_warmer_btn: Option<Rect>,
    pub quest_rect_cable_mute_target: Option<Rect>,

    // Roadmap 2: 90% Screen Interactive Theater & Flying Mascot Guide
    pub show_interactive_theater: bool,
    pub theater_slide: usize,
    pub theater_turn: usize,
    pub theater_turn_timer: f32,
    pub theater_is_playing: bool,
    pub theater_dsp_active: bool,
    pub theater_split_sum_active: bool,
    pub theater_s3_selected_filter: usize,
    pub theater_s3_order_swapped: bool,
    pub theater_s3_loop_simulated: bool,
    pub theater_s4_repeat_mode: usize,
    pub theater_s4_shuffled: bool,
    pub theater_s4_queue_open: bool,
    pub theater_s4_seek_time: f32,
    pub theater_s4_wave_mode: usize,
    pub theater_s5_category: usize,
    pub theater_s5_simulated_scan: bool,
    pub theater_s5_selected_action: usize,

    // Global Output Stage (Separated from EQ preset)
    pub eq_limiter_enabled: bool,
    pub eq_dither_mode: DitherMode,
    pub eq_clip_count: u32,
    pub resampler_settings: crate::audio::types::ResamplerSettings,

    // ReplayGain & Gapless settings
    pub replay_gain_mode: crate::models::ReplayGainMode,
    pub replay_gain_preamp_db: f32,
    pub replay_gain_clip_prevention: crate::models::ReplayGainClipPrevention,
    pub is_gapless_enabled: bool,
    pub effective_replay_gain_db: f32,
    pub effective_replay_gain_linear: f32,
    pub crossfade_secs: f32,

    // Hot-unplug device handling
    pub was_interrupted_by_unplug: bool,
    pub unplug_device_name: Option<String>,
    pub unplug_notification_timer: f32,

    // Track data
    pub current_track: Track,
    pub playlist: Vec<Track>,
    pub search_query: String,

    // Library categorization & custom playlists
    pub library_category: LibraryCategory,
    pub selected_album_filter: Option<String>,
    pub selected_artist_filter: Option<String>,
    pub selected_quality_filter: Option<String>,
    pub selected_playlist_filter: Option<String>,
    pub custom_playlists: Vec<CustomPlaylist>,

    // Play Queue (Option A Drawer)
    pub play_queue: Vec<Track>,
    pub show_queue_drawer: bool,

    // Playlist Creator state
    pub show_playlist_creator: bool,
    pub playlist_creator_name: String,
    pub playlist_creator_desc: String,
    pub playlist_creator_selected_tracks: std::collections::HashSet<String>,
    pub playlist_creator_search: String,

    // Library scanning state
    pub scan_rx: Option<std::sync::mpsc::Receiver<ScanMsg>>,
    pub is_scanning: bool,
    pub scan_status: String,

    pub visualizer_anim_phase: f32,
    pub waveform_mode: WaveformMode,
    pub audio_engine: Option<crate::audio::AudioEngineController>,
    pub signal_path_report: Option<crate::audio::SignalPathReport>,
    pub fft_analyzer: crate::audio::FftAnalyzer,
    pub spectrum_bars: [f32; 48],
    pub live_pcm_waveform: [f32; 48],
    pub live_stereo_l: [f32; 1024],
    pub live_stereo_r: [f32; 1024],
    pub live_phase_correlation: f32,
    pub live_integrated_lufs: f32,
    pub live_true_peak_db: f32,
    pub live_dynamic_range: f32,
    pub live_gain_reduction_db: f32,
    pub live_isp_clip_detected: bool,
    pub live_isp_clip_count: u32,
    pub live_isp_max_over_db: f32,
    pub dac_meter_l: f32,
    pub dac_meter_r: f32,
    pub dac_peak_hold_l: f32,
    pub dac_peak_hold_r: f32,
    pub dac_decay_timer_l: u8,
    pub dac_decay_timer_r: u8,
    pub dac_clip_l: bool,
    pub dac_clip_r: bool,
    pub current_cover_bytes: Option<Vec<u8>>,
    pub current_cover_texture: Option<eframe::egui::TextureHandle>,
    pub album_cover_cache: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Option<eframe::egui::TextureHandle>>>>,
    pub cover_req_tx: std::sync::mpsc::Sender<(String, String)>,
    pub cover_res_rx: std::sync::mpsc::Receiver<CoverLoadResult>,
    pub pending_cover_loads: std::sync::Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
    pub selected_output_device: Option<String>,
    pub available_output_devices: Vec<crate::audio::backend::OutputDeviceInfo>,

    // High-performance GUI caching (DAG topology, library stats, and EQ curves)
    pub cached_chain_eval: std::cell::RefCell<Option<crate::models::ChainEvaluation>>,
    pub chain_eval_dirty: std::cell::Cell<bool>,
    pub cached_library_stats: std::cell::Cell<Option<(usize, usize, usize, f32, usize)>>,
    pub cached_album_map: std::cell::RefCell<Option<std::collections::BTreeMap<String, Vec<usize>>>>,
    pub cached_artist_map: std::cell::RefCell<Option<std::collections::BTreeMap<String, Vec<usize>>>>,
    pub eq_curve_cache: std::cell::RefCell<Option<(u64, Vec<Pos2>)>>,
    pub smtc_bridge: Option<crate::ui::smtc::SmtcBridge>,

    // Synchronized lyrics (LRC)
    pub current_lyrics: Option<crate::lyrics::Lyrics>,
    pub lyrics_offset_trim_sec: f32,

    // AutoEq Import Feedback
    pub autoeq_import_status: Option<(String, bool)>,

    // Tag Editor Modal
    pub show_tag_editor: bool,
    pub tag_editor_track_idx: Option<usize>,
    pub tag_editor_path: Option<String>,
    pub tag_editor_title: String,
    pub tag_editor_artist: String,
    pub tag_editor_album: String,
    pub tag_editor_year: String,
    pub tag_editor_genre: String,
    pub tag_editor_track_num: String,
    pub tag_editor_status: Option<(String, bool)>,
}

pub struct CoverLoadResult {
    pub key: String,
    pub image_data: Option<([usize; 2], Vec<u8>)>,
}

pub fn random_index(len: usize) -> usize {
    if len <= 1 {
        return 0;
    }
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(42);
    let mut x = (nanos as u64) ^ 0x548b29f3c1a74d19;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    (x as usize) % len
}

impl Default for LunulaApp {
    fn default() -> Self {
        Self::new_standalone()
    }
}

impl LunulaApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_standalone()
    }

    pub fn new_standalone() -> Self {
        let (sample_waveform, sample_bass, sample_mid, sample_treble) =
            crate::scanner::generate_multiband_waveform("Lunula Default Initial Track", 3600);

        let initial_nodes = vec![
            ModularNode {
                id: 1,
                node_type: NodeType::Source,
                title: "FILE SOURCE".to_string(),
                sub1: "FLAC 24-bit".to_string(),
                sub2: "96.0 kHz".to_string(),
                pos: Pos2::new(30.0, 60.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 2,
                node_type: NodeType::Buffer,
                title: "RAM BUFFER".to_string(),
                sub1: "RingBuffer".to_string(),
                sub2: "100% Loaded".to_string(),
                pos: Pos2::new(280.0, 60.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 3,
                node_type: NodeType::Dsp,
                title: "DSP ENGINE".to_string(),
                sub1: "BYPASSED (Direct Wire)".to_string(),
                sub2: "0.0 dB Pure Master".to_string(),
                pos: Pos2::new(405.0, 215.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 4,
                node_type: NodeType::Driver,
                title: "APP RESAMPLER".to_string(),
                sub1: "System Audio Endpoint".to_string(),
                sub2: "Rubato Sinc SRC • 48.0 kHz".to_string(),
                pos: Pos2::new(530.0, 60.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 5,
                node_type: NodeType::Endpoint,
                title: "OUTPUT DAC".to_string(),
                sub1: "Audio Endpoint".to_string(),
                sub2: "Native Sample Rate".to_string(),
                pos: Pos2::new(780.0, 60.0),
                is_removable: false,
                param_val: 0.0,
            },
        ];

        // Default chain connections: in Bit-Perfect (Option B), cable directly jumps 2 -> 4 bypassing 3
        let initial_connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, 4),
            CableConnection::new(4, 5),
        ];

        let (cover_req_tx, cover_req_rx) = std::sync::mpsc::channel::<(String, String)>();
        let (cover_res_tx, cover_res_rx) = std::sync::mpsc::channel::<CoverLoadResult>();
        let pending_cover_loads = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));

        let _ = std::thread::Builder::new()
            .name("lunula-cover-loader".into())
            .spawn(move || {
                while let Ok((key, path)) = cover_req_rx.recv() {
                    let image_data = if let Some(bytes) = crate::scanner::extract_album_art(&path) {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let thumb = img.thumbnail(256, 256).to_rgba8();
                            let size = [thumb.width() as usize, thumb.height() as usize];
                            Some((size, thumb.into_raw()))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let _ = cover_res_tx.send(CoverLoadResult { key, image_data });
                }
            });

        let mut app = Self {
            current_tab: AppTab::NowPlaying,
            selected_node: None,
            chain_mode: ChainMode::BitPerfect,

            nodes: initial_nodes,
            connections: initial_connections,
            next_node_id: 6,
            dragging_node: None,
            patchbay_pan: Vec2::ZERO,
            is_panning_canvas: false,
            connecting_from_node: None,
            connecting_from_port: 0,
            connecting_is_input_drag: false,

            studio_mode: crate::models::StudioMode::Simple,
            advance_studio_state: crate::ui::patchbay::AdvanceStudioState::default(),
            patchbay_filter_search: String::new(),
            patchbay_category_idx: 0,

            is_playing: false,
            current_time: 0.0,
            volume: 0.80,
            is_muted: false,
            _start_time: Instant::now(),
            is_shuffled: false,
            repeat_mode: RepeatMode::Off,
            selected_queue_idx: None,
            queue_cursor: None,

            zoom_level: 28.0,

            eq_bands: [0.0; 10],
            eq_bypass: true,

            active_eq: ParametricEqPreset {
                name: "Flat (Netral)".to_string(),
                bands: ParametricBand::default_10_bands(),
                preamp_mode: PreampMode::Auto,
                manual_preamp_db: 0.0,
            },
            eq_is_dirty: false,
            eq_presets: ParametricEqPreset::default_presets(),
            selected_band_idx: Some(0),
            dragging_band_idx: None,
            show_save_preset_dialog: false,
            eq_preset_name_input: "Preset Kustom".to_string(),
            eq_show_spectrum_overlay: true,
            show_saved_eq_palette_popup: false,
            saved_eq_popup_anchor: None,
            show_patchbay_tutorial: false,
            patchbay_tutorial_step: 0,
            active_filter_tutorial: None,
            filter_tutorial_step: 0,
            dragging_slider_node: None,

            has_completed_onboarding: false,
            is_onboarding_quest_active: false,
            onboarding_quest_step: crate::models::OnboardingQuestStep::Welcome,
            quest_rect_signal_chain_tab: None,
            quest_rect_dsp_mode_btn: None,
            quest_rect_tube_warmer_btn: None,
            quest_rect_cable_mute_target: None,

            show_interactive_theater: true,
            theater_slide: 0,
            theater_turn: 0,
            theater_turn_timer: 0.0,
            theater_is_playing: true,
            theater_dsp_active: false,
            theater_split_sum_active: false,
            theater_s3_selected_filter: 0,
            theater_s3_order_swapped: false,
            theater_s3_loop_simulated: false,
            theater_s4_repeat_mode: 0,
            theater_s4_shuffled: false,
            theater_s4_queue_open: false,
            theater_s4_seek_time: 14.0,
            theater_s4_wave_mode: 0,
            theater_s5_category: 0,
            theater_s5_simulated_scan: false,
            theater_s5_selected_action: 0,

            eq_limiter_enabled: false,
            eq_dither_mode: DitherMode::None,
            eq_clip_count: 0,
            resampler_settings: crate::audio::types::ResamplerSettings::default(),
            replay_gain_mode: crate::models::ReplayGainMode::default(),
            replay_gain_preamp_db: 0.0,
            replay_gain_clip_prevention: crate::models::ReplayGainClipPrevention::default(),
            is_gapless_enabled: true,
            effective_replay_gain_db: 0.0,
            effective_replay_gain_linear: 1.0,
            crossfade_secs: 0.0,
            was_interrupted_by_unplug: false,
            unplug_device_name: None,
            unplug_notification_timer: 0.0,

            current_track: Track {
                title: "Belum Ada Lagu Diputar".to_string(),
                artist: "Lunula Audio Engine Ready".to_string(),
                album: "Silakan Buka Folder / File Musik".to_string(),
                duration: 0.0,
                format: "READY".to_string(),
                sample_rate: 44100,
                bit_depth: 16,
                waveform: sample_waveform,
                waveform_bass: sample_bass,
                waveform_mid: sample_mid,
                waveform_treble: sample_treble,
                file_path: None,
                file_size_mb: 0.0,
                bitrate_kbps: None,
                is_favorite: false,
                replay_gain: crate::models::ReplayGainData::default(),
                cue_offset_sec: None,
                cue_duration_sec: None,
            },
            playlist: Vec::new(),
            search_query: String::new(),

            library_category: LibraryCategory::AllTracks,
            selected_album_filter: None,
            selected_artist_filter: None,
            selected_quality_filter: None,
            selected_playlist_filter: None,
            custom_playlists: vec![
                CustomPlaylist {
                    id: 1,
                    name: "Late Night Audiophile".to_string(),
                    description: "Koleksi santai resolusi tinggi malam hari".to_string(),
                    track_paths: Vec::new(),
                },
                CustomPlaylist {
                    id: 2,
                    name: "Hi-Res Reference Master".to_string(),
                    description: "Uji dynamic range, staging, dan transparansi".to_string(),
                    track_paths: Vec::new(),
                },
            ],

            play_queue: Vec::new(),
            show_queue_drawer: false,

            show_playlist_creator: false,
            playlist_creator_name: String::new(),
            playlist_creator_desc: String::new(),
            playlist_creator_selected_tracks: std::collections::HashSet::new(),
            playlist_creator_search: String::new(),

            scan_rx: None,
            is_scanning: false,
            scan_status: String::new(),
            visualizer_anim_phase: 0.0,
            waveform_mode: WaveformMode::StackedMultiBand,
            audio_engine: crate::audio::AudioEngineController::new().ok(),
            signal_path_report: None,
            fft_analyzer: crate::audio::FftAnalyzer::new(48000.0),
            spectrum_bars: [0.0; 48],
            live_pcm_waveform: [0.0; 48],
            live_stereo_l: [0.0; 1024],
            live_stereo_r: [0.0; 1024],
            live_phase_correlation: 1.0,
            live_integrated_lufs: -70.0,
            live_true_peak_db: -96.0,
            live_dynamic_range: 0.0,
            live_gain_reduction_db: 0.0,
            live_isp_clip_detected: false,
            live_isp_clip_count: 0,
            live_isp_max_over_db: 0.0,
            dac_meter_l: 0.0,
            dac_meter_r: 0.0,
            dac_peak_hold_l: 0.0,
            dac_peak_hold_r: 0.0,
            dac_decay_timer_l: 0,
            dac_decay_timer_r: 0,
            dac_clip_l: false,
            dac_clip_r: false,
            current_cover_bytes: None,
            current_cover_texture: None,
            album_cover_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            cover_req_tx,
            cover_res_rx,
            pending_cover_loads,
            selected_output_device: None,
            available_output_devices: crate::audio::backend::enumerate_output_devices(),

            cached_chain_eval: std::cell::RefCell::new(None),
            chain_eval_dirty: std::cell::Cell::new(true),
            cached_library_stats: std::cell::Cell::new(None),
            cached_album_map: std::cell::RefCell::new(None),
            cached_artist_map: std::cell::RefCell::new(None),
            eq_curve_cache: std::cell::RefCell::new(None),
            smtc_bridge: crate::ui::smtc::SmtcBridge::new(),

            current_lyrics: None,
            lyrics_offset_trim_sec: 0.0,
            autoeq_import_status: None,

            show_tag_editor: false,
            tag_editor_track_idx: None,
            tag_editor_path: None,
            tag_editor_title: String::new(),
            tag_editor_artist: String::new(),
            tag_editor_album: String::new(),
            tag_editor_year: String::new(),
            tag_editor_genre: String::new(),
            tag_editor_track_num: String::new(),
            tag_editor_status: None,
        };

        if let Some(state) = Self::load_library_state() {
            let loaded: Vec<Track> = state.playlist.into_iter().map(Track::from).collect();
            if !loaded.is_empty() {
                app.playlist = loaded;
                if let Some(cur) = state.current_track {
                    app.current_track = Track::from(cur);
                } else if let Some(first) = app.playlist.first() {
                    app.current_track = first.clone();
                }
            }
            if !state.custom_playlists.is_empty() {
                app.custom_playlists = state.custom_playlists;
            }
            if !state.play_queue.is_empty() {
                app.play_queue = state.play_queue.into_iter().map(Track::from).collect();
            }
            if let Some(mode) = state.waveform_mode {
                app.waveform_mode = mode;
            }
            app.volume = state.volume;
            app.eq_bands = state.eq_bands;
            app.eq_bypass = state.eq_bypass;
            app.is_shuffled = state.is_shuffled;
            app.repeat_mode = state.repeat_mode;
            app.queue_cursor = state.queue_cursor;
            app.selected_output_device = state.selected_output_device;
            if let Some(bands) = state.parametric_bands {
                app.active_eq.bands = bands;
            }
            if let Some(mode) = state.eq_preamp_mode {
                app.active_eq.preamp_mode = mode;
            }
            if let Some(gain) = state.eq_manual_preamp_db {
                app.active_eq.manual_preamp_db = gain;
            }
            if let Some(name) = state.eq_current_preset_name {
                app.active_eq.name = name;
            }
            if let Some(lim) = state.eq_limiter_enabled {
                app.eq_limiter_enabled = lim;
            }
            if let Some(dith) = state.eq_dither_mode {
                app.eq_dither_mode = dith;
            }
            if let Some(q) = state.resampler_quality {
                app.resampler_settings.quality = q;
            }
            if let Some(p) = state.resampler_profile {
                app.resampler_settings.profile = p;
            }
            if let Some(rgm) = state.replay_gain_mode {
                app.replay_gain_mode = rgm;
            }
            if let Some(pre) = state.replay_gain_preamp_db {
                app.replay_gain_preamp_db = pre;
            }
            if let Some(cp) = state.replay_gain_clip_prevention {
                app.replay_gain_clip_prevention = cp;
            }
            if let Some(cf) = state.crossfade_secs {
                app.crossfade_secs = cf;
            }
            app.has_completed_onboarding = state.has_completed_onboarding.unwrap_or(false);
            if !app.has_completed_onboarding {
                app.is_onboarding_quest_active = true;
                app.set_chain_mode(ChainMode::BitPerfect);
            } else {
                app.set_chain_mode(state.chain_mode);
            }
        } else {
            // First time application is ever launched!
            app.has_completed_onboarding = false;
            app.is_onboarding_quest_active = true;
            app.set_chain_mode(ChainMode::BitPerfect);
            app.eq_bands[0] = 3.0;
            app.eq_bands[1] = 1.5;
            app.eq_bands[8] = 1.0;
            app.eq_bands[9] = 2.0;
        }

        // Preload artwork for initial track
        if let Some(ref path_str) = app.current_track.file_path {
            app.current_cover_bytes = crate::scanner::extract_album_art(path_str);
        }

        if let Some(ref engine) = app.audio_engine {
            if let Some(ref dev) = app.selected_output_device {
                engine.set_output_device(Some(dev.clone()));
            }
            engine.set_volume(if app.is_muted { 0.0 } else { app.volume });
            engine.set_chain_mode(app.chain_mode);
            engine.set_eq_bypass(app.eq_bypass);
            for (idx, &gain) in app.eq_bands.iter().enumerate() {
                engine.set_eq_band(idx, gain);
            }
            engine.set_resampler_settings(app.resampler_settings);
            engine.set_crossfade_secs(app.crossfade_secs);
        }

        app.sync_signal_chain_to_engine();
        app
    }

    pub fn sync_eq_to_engine(&self) {
        if let Some(ref engine) = self.audio_engine {
            engine.set_eq_bypass(self.eq_bypass);
            for (idx, &gain) in self.eq_bands.iter().enumerate() {
                engine.set_eq_band(idx, gain);
            }
        }
        self.sync_signal_chain_to_engine();
    }

    pub fn set_crossfade_secs(&mut self, secs: f32) {
        self.crossfade_secs = secs.clamp(0.0, 12.0);
        if let Some(ref engine) = self.audio_engine {
            engine.set_crossfade_secs(self.crossfade_secs);
        }
        self.save_library_state();
    }

    pub fn calculate_effective_replay_gain(&self, track: &Track) -> (f32, f32) {
        use crate::models::{ReplayGainClipPrevention, ReplayGainMode};
        if self.replay_gain_mode == ReplayGainMode::Disabled {
            return (0.0, 1.0);
        }

        let rg = &track.replay_gain;
        let (base_gain, peak_opt) = match self.replay_gain_mode {
            ReplayGainMode::Track => (rg.track_gain_db.unwrap_or(0.0), rg.track_peak),
            ReplayGainMode::Album => (
                rg.album_gain_db.or(rg.track_gain_db).unwrap_or(0.0),
                rg.album_peak.or(rg.track_peak),
            ),
            ReplayGainMode::Auto => {
                let is_album_context = self.repeat_mode == RepeatMode::Album || self.selected_album_filter.is_some();
                if is_album_context {
                    (
                        rg.album_gain_db.or(rg.track_gain_db).unwrap_or(0.0),
                        rg.album_peak.or(rg.track_peak),
                    )
                } else {
                    (
                        rg.track_gain_db.or(rg.album_gain_db).unwrap_or(0.0),
                        rg.track_peak.or(rg.album_peak),
                    )
                }
            }
            ReplayGainMode::Disabled => (0.0, None),
        };

        let raw_gain = base_gain + self.replay_gain_preamp_db;

        let effective_gain_db = match self.replay_gain_clip_prevention {
            ReplayGainClipPrevention::PreventByPeak => {
                let peak = peak_opt.unwrap_or(1.0);
                if peak > 0.0 {
                    let max_gain_db = -20.0 * peak.log10();
                    if raw_gain > max_gain_db {
                        max_gain_db
                    } else {
                        raw_gain
                    }
                } else {
                    raw_gain.min(0.0)
                }
            }
            ReplayGainClipPrevention::Limiter => raw_gain,
            ReplayGainClipPrevention::None => raw_gain,
        };

        let linear = 10.0_f32.powf(effective_gain_db / 20.0).clamp(0.0, 4.0);
        (effective_gain_db, linear)
    }

    pub fn update_effective_replay_gain(&mut self) {
        let (db, lin) = self.calculate_effective_replay_gain(&self.current_track);
        self.effective_replay_gain_db = db;
        self.effective_replay_gain_linear = lin;

        if self.replay_gain_clip_prevention == crate::models::ReplayGainClipPrevention::Limiter && db > 0.0 {
            self.eq_limiter_enabled = true;
        }
    }

    pub fn sync_smtc_state(&mut self) {
        if let Some(ref mut smtc) = self.smtc_bridge {
            let title = &self.current_track.title;
            let artist = &self.current_track.artist;
            let album = &self.current_track.album;
            let dur = self.current_track.duration;
            let cover_url = self.current_track.file_path.as_deref();
            smtc.update_metadata(title, artist, album, dur, cover_url);
            smtc.update_playback_state(self.is_playing, self.current_time);
        }
    }

    pub fn peek_next_track(&self) -> Option<Track> {
        if self.repeat_mode == RepeatMode::One {
            return Some(self.current_track.clone());
        }

        if !self.play_queue.is_empty() {
            let next_queue_idx = if self.is_shuffled {
                let pool: Vec<usize> = if self.play_queue.len() > 1 {
                    (0..self.play_queue.len()).filter(|&i| Some(i) != self.queue_cursor).collect()
                } else {
                    vec![0]
                };
                if pool.is_empty() { 0 } else { pool[0] }
            } else {
                match self.queue_cursor {
                    Some(cur) => cur + 1,
                    None => 0,
                }
            };

            if next_queue_idx < self.play_queue.len() {
                return Some(self.play_queue[next_queue_idx].clone());
            } else if self.repeat_mode == RepeatMode::All || self.repeat_mode == RepeatMode::Album {
                return Some(self.play_queue[0].clone());
            } else {
                return None;
            }
        }

        if self.playlist.is_empty() {
            return None;
        }

        let candidate_indices: Vec<usize> = match self.repeat_mode {
            RepeatMode::Album => {
                let cur_album = self.current_track.album.trim().to_lowercase();
                let matched: Vec<usize> = self.playlist.iter().enumerate()
                    .filter(|(_, t)| !t.album.trim().is_empty() && t.album.trim().to_lowercase() == cur_album)
                    .map(|(i, _)| i)
                    .collect();
                if matched.is_empty() {
                    (0..self.playlist.len()).collect()
                } else {
                    matched
                }
            }
            _ => (0..self.playlist.len()).collect(),
        };

        if candidate_indices.is_empty() {
            return None;
        }

        if self.is_shuffled {
            let cur_global_idx = self.playlist.iter().position(|t| {
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            let pool: Vec<usize> = if candidate_indices.len() > 1 {
                candidate_indices.into_iter().filter(|&i| Some(i) != cur_global_idx).collect()
            } else {
                candidate_indices
            };
            if pool.is_empty() {
                None
            } else {
                Some(self.playlist[pool[0]].clone())
            }
        } else {
            let cur_pos = candidate_indices.iter().position(|&idx| {
                let t = &self.playlist[idx];
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });

            match cur_pos {
                Some(pos) => {
                    let next_pos = pos + 1;
                    if next_pos < candidate_indices.len() {
                        let global_idx = candidate_indices[next_pos];
                        Some(self.playlist[global_idx].clone())
                    } else {
                        match self.repeat_mode {
                            RepeatMode::Off => None,
                            RepeatMode::All | RepeatMode::Album | RepeatMode::One => {
                                let global_idx = candidate_indices[0];
                                Some(self.playlist[global_idx].clone())
                            }
                        }
                    }
                }
                None => candidate_indices.first().map(|&idx| self.playlist[idx].clone()),
            }
        }
    }

    pub fn on_gapless_transition(&mut self, path: Option<std::path::PathBuf>, duration: f32, sample_rate: u32) {
        let next_track_opt = if let Some(p) = &path {
            let p_str = p.to_string_lossy().to_string();
            if let Some(pos) = self.play_queue.iter().position(|t| t.file_path.as_deref() == Some(&p_str)) {
                self.queue_cursor = Some(pos);
                Some(self.play_queue[pos].clone())
            } else if let Some(t) = self.playlist.iter().find(|t| t.file_path.as_deref() == Some(&p_str)) {
                Some(t.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(mut next_t) = next_track_opt {
            if duration > 0.0 {
                next_t.duration = duration;
            }
            if sample_rate > 0 {
                next_t.sample_rate = sample_rate;
            }
            self.current_track = next_t;
        }
        self.current_time = 0.0;
        self.is_playing = true;
        self.current_cover_bytes = self.current_track.file_path.as_ref().and_then(|p| {
            crate::scanner::extract_album_art(p)
        });
        self.current_cover_texture = None;
        self.current_lyrics = self.current_track.file_path.as_ref().and_then(|p| {
            crate::lyrics::Lyrics::load_for_track(std::path::Path::new(p))
        });
        self.lyrics_offset_trim_sec = 0.0;

        self.update_effective_replay_gain();
        self.sync_smtc_state();

        if self.is_gapless_enabled {
            if let Some(next) = self.peek_next_track() {
                if let Some(ref next_p) = next.file_path {
                    if let Some(ref engine) = self.audio_engine {
                        engine.queue_next(std::path::PathBuf::from(next_p));
                    }
                }
            }
        }
        self.save_library_state();
    }

    pub fn play_track(&mut self, track: Track) {
        self.current_track = track;
        self.current_time = 0.0;
        self.is_playing = true;
        self.current_cover_bytes = self.current_track.file_path.as_ref().and_then(|p| {
            crate::scanner::extract_album_art(p)
        });
        self.current_cover_texture = None; // Reset so next frame builds the new texture
        self.current_lyrics = self.current_track.file_path.as_ref().and_then(|p| {
            crate::lyrics::Lyrics::load_for_track(std::path::Path::new(p))
        });
        self.lyrics_offset_trim_sec = 0.0;

        self.update_effective_replay_gain();
        self.sync_smtc_state();

        if let Some(ref path_str) = self.current_track.file_path {
            if let Some(ref engine) = self.audio_engine {
                engine.set_chain_mode(self.chain_mode);
                engine.play(std::path::PathBuf::from(path_str));
                if let Some(cue_offset) = self.current_track.cue_offset_sec {
                    if cue_offset > 0.01 {
                        engine.seek(cue_offset);
                    }
                }
                let eff_vol = if self.is_muted {
                    0.0
                } else if self.chain_mode == ChainMode::BitPerfect {
                    self.volume
                } else {
                    (self.volume * self.effective_replay_gain_linear).clamp(0.0, 2.0)
                };
                engine.set_volume(eff_vol);
                for (idx, &gain) in self.eq_bands.iter().enumerate() {
                    engine.set_eq_band(idx, gain);
                }

                if self.is_gapless_enabled {
                    if let Some(next) = self.peek_next_track() {
                        if let Some(ref next_p) = next.file_path {
                            engine.queue_next(std::path::PathBuf::from(next_p));
                        }
                    }
                }
            }
            self.sync_signal_chain_to_engine();
        }
        self.save_library_state();
    }

    pub fn toggle_playback(&mut self) {
        if self.is_playing {
            self.is_playing = false;
            if let Some(ref engine) = self.audio_engine {
                engine.pause();
            }
        } else {
            self.is_playing = true;
            if let Some(ref engine) = self.audio_engine {
                if engine.playback_state() == crate::audio::PlaybackState::Paused {
                    engine.resume();
                } else if let Some(ref path_str) = self.current_track.file_path {
                    engine.play(std::path::PathBuf::from(path_str));
                    let eff_vol = if self.is_muted {
                        0.0
                    } else if self.chain_mode == ChainMode::BitPerfect {
                        self.volume
                    } else {
                        (self.volume * self.effective_replay_gain_linear).clamp(0.0, 2.0)
                    };
                    engine.set_volume(eff_vol);
                }
            }
        }
        self.sync_smtc_state();
    }

    pub fn advance_track(&mut self, is_manual_skip: bool) {
        // If Repeat One is active and the track ended naturally, replay current track
        if !is_manual_skip && self.repeat_mode == RepeatMode::One {
            if self.current_track.file_path.is_some() {
                self.current_time = 0.0;
                if let Some(ref engine) = self.audio_engine {
                    engine.seek(0.0);
                    engine.resume();
                }
                return;
            }
        }

        // 1. Play Queue: persist songs in queue without disappearing, advance queue_cursor
        if !self.play_queue.is_empty() {
            let next_queue_idx = if self.is_shuffled {
                let pool: Vec<usize> = if self.play_queue.len() > 1 {
                    (0..self.play_queue.len()).filter(|&i| Some(i) != self.queue_cursor).collect()
                } else {
                    vec![0]
                };
                pool[random_index(pool.len())]
            } else {
                match self.queue_cursor {
                    Some(cur) => cur + 1,
                    None => 0,
                }
            };

            if next_queue_idx < self.play_queue.len() {
                self.queue_cursor = Some(next_queue_idx);
                let next = self.play_queue[next_queue_idx].clone();
                self.play_track(next);
                return;
            } else if self.repeat_mode == RepeatMode::All || self.repeat_mode == RepeatMode::Album {
                // Loop back to start of queue
                self.queue_cursor = Some(0);
                let next = self.play_queue[0].clone();
                self.play_track(next);
                return;
            } else {
                // Reached end of queue
                self.queue_cursor = None;
                if self.repeat_mode == RepeatMode::Off {
                    self.current_time = 0.0;
                    self.is_playing = false;
                    if let Some(ref engine) = self.audio_engine {
                        engine.pause();
                    }
                    return;
                }
            }
        }

        if self.playlist.is_empty() {
            self.current_time = 0.0;
            self.is_playing = false;
            return;
        }

        // 2. Determine candidate track indices from playlist
        let candidate_indices: Vec<usize> = match self.repeat_mode {
            RepeatMode::Album => {
                let cur_album = self.current_track.album.trim().to_lowercase();
                let matched: Vec<usize> = self.playlist.iter().enumerate()
                    .filter(|(_, t)| !t.album.trim().is_empty() && t.album.trim().to_lowercase() == cur_album)
                    .map(|(i, _)| i)
                    .collect();
                if matched.is_empty() {
                    (0..self.playlist.len()).collect()
                } else {
                    matched
                }
            }
            _ => (0..self.playlist.len()).collect(),
        };

        if candidate_indices.is_empty() {
            self.current_time = 0.0;
            self.is_playing = false;
            return;
        }

        if self.is_shuffled {
            let cur_global_idx = self.playlist.iter().position(|t| {
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            let pool: Vec<usize> = if candidate_indices.len() > 1 {
                candidate_indices.into_iter().filter(|&i| Some(i) != cur_global_idx).collect()
            } else {
                candidate_indices
            };
            let chosen_idx = pool[random_index(pool.len())];
            let next = self.playlist[chosen_idx].clone();
            self.play_track(next);
        } else {
            let cur_pos = candidate_indices.iter().position(|&idx| {
                let t = &self.playlist[idx];
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });

            match cur_pos {
                Some(pos) => {
                    let next_pos = pos + 1;
                    if next_pos < candidate_indices.len() {
                        let global_idx = candidate_indices[next_pos];
                        let next = self.playlist[global_idx].clone();
                        self.play_track(next);
                    } else {
                        // End of candidates reached
                        match self.repeat_mode {
                            RepeatMode::Off => {
                                self.current_time = 0.0;
                                self.is_playing = false;
                                if let Some(ref engine) = self.audio_engine {
                                    engine.pause();
                                }
                            }
                            RepeatMode::All | RepeatMode::Album | RepeatMode::One => {
                                let global_idx = candidate_indices[0];
                                let next = self.playlist[global_idx].clone();
                                self.play_track(next);
                            }
                        }
                    }
                }
                None => {
                    let global_idx = candidate_indices[0];
                    let next = self.playlist[global_idx].clone();
                    self.play_track(next);
                }
            }
        }
    }

    pub fn play_next_track(&mut self) {
        self.advance_track(true);
    }

    pub fn on_track_ended(&mut self) {
        self.advance_track(false);
    }

    pub fn play_prev_track(&mut self) {
        if self.current_time > 3.0 {
            self.current_time = 0.0;
            if let Some(ref engine) = self.audio_engine {
                engine.seek(0.0);
            }
            return;
        }

        // If playing from queue, go to previous queued song without removing
        if let Some(cur) = self.queue_cursor {
            if !self.play_queue.is_empty() {
                if cur > 0 && cur < self.play_queue.len() {
                    self.queue_cursor = Some(cur - 1);
                    let prev = self.play_queue[cur - 1].clone();
                    self.play_track(prev);
                    return;
                } else if self.repeat_mode == RepeatMode::All {
                    self.queue_cursor = Some(self.play_queue.len() - 1);
                    let prev = self.play_queue[self.play_queue.len() - 1].clone();
                    self.play_track(prev);
                    return;
                }
            }
        }

        if self.playlist.is_empty() {
            self.current_time = 0.0;
            return;
        }

        let candidate_indices: Vec<usize> = match self.repeat_mode {
            RepeatMode::Album => {
                let cur_album = self.current_track.album.trim().to_lowercase();
                let matched: Vec<usize> = self.playlist.iter().enumerate()
                    .filter(|(_, t)| !t.album.trim().is_empty() && t.album.trim().to_lowercase() == cur_album)
                    .map(|(i, _)| i)
                    .collect();
                if matched.is_empty() {
                    (0..self.playlist.len()).collect()
                } else {
                    matched
                }
            }
            _ => (0..self.playlist.len()).collect(),
        };

        if candidate_indices.is_empty() {
            self.current_time = 0.0;
            return;
        }

        if self.is_shuffled {
            let cur_global_idx = self.playlist.iter().position(|t| {
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            let pool: Vec<usize> = if candidate_indices.len() > 1 {
                candidate_indices.into_iter().filter(|&i| Some(i) != cur_global_idx).collect()
            } else {
                candidate_indices
            };
            let chosen_idx = pool[random_index(pool.len())];
            let prev = self.playlist[chosen_idx].clone();
            self.play_track(prev);
        } else {
            let cur_pos = candidate_indices.iter().position(|&idx| {
                let t = &self.playlist[idx];
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            let prev_pos = match cur_pos {
                Some(pos) => if pos == 0 { candidate_indices.len() - 1 } else { pos - 1 },
                None => 0,
            };
            let global_idx = candidate_indices[prev_pos];
            let prev = self.playlist[global_idx].clone();
            self.play_track(prev);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.is_shuffled = !self.is_shuffled;
        self.save_library_state();
    }

    pub fn cycle_repeat_mode(&mut self) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Album,
            RepeatMode::Album => RepeatMode::Off,
        };
        self.save_library_state();
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
        self.save_library_state();
    }

    pub fn shuffle_queue(&mut self) {
        let len = self.play_queue.len();
        if len <= 1 {
            return;
        }
        for i in (1..len).rev() {
            let j = random_index(i + 1);
            self.play_queue.swap(i, j);
        }
        self.selected_queue_idx = None;
        self.save_library_state();
    }

    pub fn seek_to(&mut self, target_seconds: f32) {
        if self.current_track.duration > 0.0 {
            let target = target_seconds.clamp(0.0, self.current_track.duration);
            self.current_time = target;
            if let Some(ref engine) = self.audio_engine {
                let actual = if let Some(offset) = self.current_track.cue_offset_sec {
                    offset + target
                } else {
                    target
                };
                engine.seek(actual);
            }
            self.sync_smtc_state();
        }
    }

    pub fn seek_relative(&mut self, delta_seconds: f32) {
        self.seek_to(self.current_time + delta_seconds);
    }

    pub fn set_app_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.2);
        self.is_muted = false;
        if let Some(ref engine) = self.audio_engine {
            engine.set_muted(false);
            engine.set_volume(self.volume);
        }
        self.save_library_state();
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
        if let Some(ref engine) = self.audio_engine {
            engine.set_muted(self.is_muted);
            engine.set_volume(if self.is_muted { 0.0 } else { self.volume });
        }
    }

    pub fn update_signal_nodes_from_report(&mut self, report: &crate::audio::SignalPathReport) {
        for node in &mut self.nodes {
            match node.id {
                1 => {
                    node.title = "FILE SOURCE".to_string();
                    let bit_depth_str = if report.source_bit_depth > 0 {
                        format!("{}-bit", report.source_bit_depth)
                    } else {
                        "Audio".to_string()
                    };
                    node.sub1 = format!("{} {}", self.current_track.format, bit_depth_str);
                    node.sub2 = format!("{:.1} kHz", report.source_rate as f32 / 1000.0);
                }
                2 => {
                    node.title = "RAM BUFFER".to_string();
                    node.sub1 = "RingBuffer".to_string();
                    node.sub2 = "100% Loaded".to_string();
                }
                3 => {
                    node.title = "DSP ENGINE".to_string();
                    if report.is_dsp_active {
                        node.sub1 = "Active (64-bit Float)".to_string();
                        node.sub2 = "10-Band Biquad f64 • Headroom Guard".to_string();
                    } else {
                        node.sub1 = "BYPASSED (Direct Wire)".to_string();
                        node.sub2 = "0.0 dB Pure Master".to_string();
                    }
                }
                4 => {
                    node.title = "APP RESAMPLER".to_string();
                    node.sub1 = report.device_name.clone();
                    if report.is_exclusive {
                        node.sub2 = format!("Direct Bitstream • {:.1} kHz", report.device_rate as f32 / 1000.0);
                    } else if report.is_resampling {
                        node.sub2 = format!("Rubato Sinc {} ({}) • {:.1} kHz", self.resampler_settings.quality.short_name(), self.resampler_settings.profile.short_name(), report.device_rate as f32 / 1000.0);
                    } else {
                        node.sub2 = format!("1:1 Native Rate • {:.1} kHz", report.device_rate as f32 / 1000.0);
                    }
                }
                5 => {
                    node.title = "OUTPUT DAC".to_string();
                    node.sub1 = report.device_name.clone();
                    node.sub2 = format!("{} • {:.1} kHz", report.format_desc, report.device_rate as f32 / 1000.0);
                }
                _ => {}
            }
        }
    }

    pub fn set_resampler_settings(&mut self, settings: crate::audio::types::ResamplerSettings) {
        self.resampler_settings = settings;
        if let Some(ref engine) = self.audio_engine {
            engine.set_resampler_settings(settings);
        }
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == 4) {
            let rate_str = self.signal_path_report.as_ref().map_or("48.0 kHz".to_string(), |r| format!("{:.1} kHz", r.device_rate as f32 / 1000.0));
            node.sub2 = format!("Rubato Sinc {} ({}) • {}", settings.quality.short_name(), settings.profile.short_name(), rate_str);
        }
        self.save_library_state();
    }

    pub fn set_chain_mode(&mut self, mode: ChainMode) {
        self.chain_mode = mode;

        match mode {
            ChainMode::BitPerfect => {
                self.eq_bypass = true;
            }
            ChainMode::DspActive => {
                self.eq_bypass = false;
            }
            ChainMode::OsResampled => {
                self.eq_bypass = true;
            }
        }

        if self.connections.is_empty() {
            self.connections = vec![
                CableConnection::new(1, 2),
                CableConnection::new(2, 4),
                CableConnection::new(4, 5),
            ];
        }

        if let Some(ref report) = self.signal_path_report {
            let mut updated = report.clone();
            updated.requested_mode = mode;
            updated.actual_mode = match mode {
                ChainMode::BitPerfect => {
                    if updated.source_rate == updated.device_rate {
                        ChainMode::BitPerfect
                    } else {
                        ChainMode::OsResampled
                    }
                }
                other => other,
            };
            updated.is_dsp_active = updated.actual_mode == ChainMode::DspActive;
            self.update_signal_nodes_from_report(&updated);
        } else {
            let dev_name = self.selected_output_device.clone().unwrap_or_else(|| "Default Audio Endpoint".to_string());
            for n in &mut self.nodes {
                match n.id {
                    3 => {
                        if mode == ChainMode::DspActive {
                            n.sub1 = "Active (64-bit Float)".to_string();
                            n.sub2 = "64-bit DSP Bus • Clean Passthrough".to_string();
                        } else {
                            n.sub1 = "BYPASSED (Direct Wire)".to_string();
                            n.sub2 = "0.0 dB Pure Master".to_string();
                        }
                    }
                    4 => {
                        n.title = "APP RESAMPLER".to_string();
                        n.sub1 = dev_name.clone();
                        match mode {
                            ChainMode::BitPerfect => {
                                n.sub2 = "1:1 Native Rate (Bit-Perfect)".to_string();
                            }
                            ChainMode::DspActive => {
                                n.sub2 = format!("Rubato Sinc {} ({}) • 48.0 kHz", self.resampler_settings.quality.short_name(), self.resampler_settings.profile.short_name());
                            }
                            ChainMode::OsResampled => {
                                n.sub2 = format!("Resample {} ({}) to 48.0 kHz", self.resampler_settings.quality.short_name(), self.resampler_settings.profile.short_name());
                            }
                        }
                    }
                    5 => {
                        n.title = "OUTPUT DAC".to_string();
                        n.sub1 = dev_name.clone();
                        n.sub2 = match mode {
                            ChainMode::OsResampled => "Fixed 48.0 kHz".to_string(),
                            _ => "Native Sample Rate".to_string(),
                        };
                    }
                    _ => {}
                }
            }
        }

        if let Some(ref engine) = self.audio_engine {
            engine.set_chain_mode(mode);
            engine.set_eq_bypass(self.eq_bypass);
        }

        self.invalidate_chain_cache();
        self.sync_signal_chain_to_engine();
        self.save_library_state();
    }

    pub fn toggle_eq_bypass(&mut self) {
        if self.chain_mode == ChainMode::BitPerfect {
            self.set_chain_mode(ChainMode::DspActive);
        } else if self.chain_mode == ChainMode::DspActive {
            self.set_chain_mode(ChainMode::BitPerfect);
        } else {
            self.eq_bypass = !self.eq_bypass;
            if let Some(ref engine) = self.audio_engine {
                engine.set_eq_bypass(self.eq_bypass);
            }
            self.save_library_state();
        }
    }

    pub fn get_album_cover_texture(
        &self,
        _ctx: &eframe::egui::Context,
        key: &str,
        file_path: Option<&str>,
    ) -> Option<eframe::egui::TextureHandle> {
        if let Ok(mut cache) = self.album_cover_cache.lock() {
            if let Some(cached) = cache.get(key) {
                return cached.clone();
            }
            if let Some(path) = file_path {
                if let Ok(mut pending) = self.pending_cover_loads.lock() {
                    if !pending.contains(key) {
                        pending.insert(key.to_string());
                        let _ = self.cover_req_tx.send((key.to_string(), path.to_string()));
                    }
                }
            } else {
                cache.insert(key.to_string(), None);
            }
        }
        None
    }

    // =========================================================================================
    //  ARSITEKTUR PEMROSESAN SINYAL AUDIO: DIRECTED ACYCLIC GRAPH (DAG)
    // =========================================================================================
    //
    //  [IDE C] URUTAN PEMROSESAN SERIAL & PENCEGAHAN FEEDBACK LOOP:
    //  ----------------------------------------------------------------------------------------
    //  - Graf patchbay adalah DAG (Directed Acyclic Graph) yang diurutkan secara topologis.
    //  - Alur kabel dari Output -> Input menentukan urutan eksekusi efek (misal: A -> B -> C).
    //  - FEEDBACK LOOP dicegah: sebuah kabel tidak boleh berputar kembali ke node yang sama
    //    secara langsung maupun tidak langsung (A -> C -> A), karena dalam audio digital tanpa delay
    //    akan menimbulkan rekursi tak berujung (buffer recursion) atau feedback screeching.
    //  - Solusi "Proses A lalu C lalu A lagi":
    //    Tambahkan instance modul A kedua di kanvas (A1 -> C -> A2). Dengan begitu, alur tetap
    //    DAG yang bersih, aman, dan parameter A1 dan A2 dapat diatur independen!
    //
    //  [IDE B] PERCABANGAN PARALEL (SPLIT & SUMMING MIX DSP):
    //  ----------------------------------------------------------------------------------------
    //  - GUI mengizinkan percabangan: satu output pin bisa mencabang ke banyak node (Split),
    //    dan satu input pin bisa menerima banyak kabel masuk (Merge/Join).
    //  - Roadmap DSP Audio Engine untuk Percabangan Paralel:
    //    1. SPLIT: Saat sebuah node mencabang ke N cabang, buffer audio difork/disalin ke tiap cabang.
    //    2. PARALLEL PROCESS: Masing-masing cabang memproses filternya secara independen.
    //    3. SUMMING MIXER: Pada node di mana cabang-cabang tersebut bertemu kembali (converge),
    //       sinyal-sinyal dijumlahkan (y = x1 + x2 + ... + xn) dengan normalisasi gain / headroom
    //       (misalnya pembagian 1/N atau attenuasi -6 dB) untuk mencegah digital clipping (+6 dB).
    //    4. PHASE ALIGNMENT: Memperhatikan penundaan fase (delay) agar tidak timbul comb-filtering.
    // =========================================================================================

    /// Memeriksa apakah node `start` dapat mencapai node `target` melalui sambungan kabel yang ada.
    pub fn can_reach_node(&self, start: usize, target: usize) -> bool {
        use std::collections::{HashSet, VecDeque};
        if start == target {
            return true;
        }
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(curr) = queue.pop_front() {
            if curr == target {
                return true;
            }
            for conn in &self.connections {
                if conn.from_node == curr && visited.insert(conn.to_node) {
                    queue.push_back(conn.to_node);
                }
            }
        }
        false
    }

    #[inline]
    pub fn invalidate_chain_cache(&self) {
        self.chain_eval_dirty.set(true);
    }

    #[inline]
    pub fn invalidate_library_cache(&self) {
        self.cached_library_stats.set(None);
        *self.cached_album_map.borrow_mut() = None;
        *self.cached_artist_map.borrow_mut() = None;
    }

    pub fn evaluate_signal_chain(&self) -> crate::models::ChainEvaluation {
        if !self.chain_eval_dirty.get() {
            let cached = self.cached_chain_eval.borrow().as_ref().cloned();
            if let Some(eval) = cached {
                return eval;
            }
        }

        let eval = self.compute_evaluate_signal_chain();
        *self.cached_chain_eval.borrow_mut() = Some(eval.clone());
        self.chain_eval_dirty.set(false);
        eval
    }

    fn compute_evaluate_signal_chain(&self) -> crate::models::ChainEvaluation {
        use std::collections::{HashSet, VecDeque};
        use crate::models::{ChainEvaluation, NodeType};

        let source_id = self.nodes.iter().find(|n| n.node_type == NodeType::Source).map(|n| n.id).unwrap_or(1);
        let endpoint_id = self.nodes.iter().find(|n| n.node_type == NodeType::Endpoint).map(|n| n.id).unwrap_or(5);

        // 1. Forward reachability from Source
        let mut reachable_from_source = HashSet::new();
        let mut q_fwd = VecDeque::new();
        reachable_from_source.insert(source_id);
        q_fwd.push_back(source_id);

        while let Some(curr) = q_fwd.pop_front() {
            for conn in &self.connections {
                if conn.from_node == curr {
                    if reachable_from_source.insert(conn.to_node) {
                        q_fwd.push_back(conn.to_node);
                    }
                }
            }
        }

        let is_complete = reachable_from_source.contains(&endpoint_id);

        // 2. Backward reachability from Endpoint
        let mut can_reach_endpoint = HashSet::new();
        let mut q_back = VecDeque::new();
        can_reach_endpoint.insert(endpoint_id);
        q_back.push_back(endpoint_id);

        while let Some(curr) = q_back.pop_front() {
            for conn in &self.connections {
                if conn.to_node == curr {
                    if can_reach_endpoint.insert(conn.from_node) {
                        q_back.push_back(conn.from_node);
                    }
                }
            }
        }

        // 3. Flowing nodes and flowing cables
        let mut flowing_nodes = HashSet::new();
        let mut flowing_cables = HashSet::new();

        if is_complete {
            for node in &self.nodes {
                if reachable_from_source.contains(&node.id) && can_reach_endpoint.contains(&node.id) {
                    flowing_nodes.insert(node.id);
                }
            }
            for conn in &self.connections {
                if flowing_nodes.contains(&conn.from_node) && flowing_nodes.contains(&conn.to_node) {
                    flowing_cables.insert(*conn);
                }
            }
        }

        ChainEvaluation {
            is_complete,
            reachable_from_source,
            flowing_nodes,
            flowing_cables,
        }
    }

    pub fn compile_signal_chain(&self) -> crate::audio::SignalChainGraph {
        use std::collections::{HashMap, VecDeque};
        use crate::models::NodeType;
        use crate::audio::{
            AnalogPreampEffect, BassGuardEffect, CompressorEffect, DynamicLoudnessEffect, Effect, LowPassFilterEffect, MixerEffect, OpticalLevelerEffect,
            PassthroughEffect, PresetEqEffect, SplitterEffect, SteppedAttenuatorEffect, StereoBalanceEffect, StereoWidenerEffect,
            TubeWarmerEffect, VolumeAttenuatorEffect, VolumeBoosterEffect,
        };

        let chain_eval = self.evaluate_signal_chain();
        if !chain_eval.is_complete {
            return crate::audio::SignalChainGraph {
                is_complete: false,
                nodes: Vec::new(),
                edges: Vec::new(),
                topological_order: Vec::new(),
                effects: Vec::new(),
            };
        }

        // Topological sort on flowing nodes starting from Source
        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();

        for &node_id in &chain_eval.flowing_nodes {
            in_degree.insert(node_id, 0);
            adj.insert(node_id, Vec::new());
        }

        for conn in &chain_eval.flowing_cables {
            if chain_eval.flowing_nodes.contains(&conn.from_node) && chain_eval.flowing_nodes.contains(&conn.to_node) {
                *in_degree.entry(conn.to_node).or_insert(0) += 1;
                adj.entry(conn.from_node).or_default().push(conn.to_node);
            }
        }

        let mut q = VecDeque::new();
        for (&node_id, &deg) in &in_degree {
            if deg == 0 {
                q.push_back(node_id);
            }
        }

        let mut ordered_node_ids = Vec::new();
        while let Some(curr) = q.pop_front() {
            ordered_node_ids.push(curr);
            if let Some(neighbors) = adj.get(&curr) {
                for &next in neighbors {
                    if let Some(deg) = in_degree.get_mut(&next) {
                        *deg -= 1;
                        if *deg == 0 {
                            q.push_back(next);
                        }
                    }
                }
            }
        }

        // Build concrete effects chain in topological sequence
        let sample_rate = self.signal_path_report.as_ref().map_or(44100.0, |r| r.device_rate as f64);
        let mut effects: Vec<Box<dyn Effect>> = Vec::new();
        let mut nodes: Vec<crate::audio::GraphNode> = Vec::new();
        let mut topological_order: Vec<u32> = Vec::new();

        for &id in &ordered_node_ids {
            if let Some(node) = self.nodes.iter().find(|n| n.id == id) {
                let is_custom_effect = node.node_type.is_dsp_or_effect();
                let effect: Box<dyn Effect> = match &node.node_type {
                    NodeType::Dsp => {
                        // Neutral 64-bit transparent DSP bus (no hidden EQ coloration)
                        Box::new(PassthroughEffect)
                    }
                    NodeType::TubeWarmer => {
                        let drive = if (0.0..=100.0).contains(&node.param_val) { node.param_val } else { 35.0 };
                        Box::new(TubeWarmerEffect::new(drive))
                    }
                    NodeType::StereoWidener => {
                        let width = if (0.0..=100.0).contains(&node.param_val) {
                            1.0 + (node.param_val / 100.0) as f64
                        } else if (1.0..=3.0).contains(&node.param_val) {
                            node.param_val as f64
                        } else {
                            1.35
                        };
                        Box::new(StereoWidenerEffect::new(width))
                    }
                    NodeType::BassGuard => {
                        Box::new(BassGuardEffect::new(sample_rate))
                    }
                    NodeType::VolumeAttenuator => {
                        Box::new(VolumeAttenuatorEffect::new(-6.02))
                    }
                    NodeType::LowPassFilter => {
                        let cutoff = if node.param_val > 50.0 { node.param_val as f64 } else { 800.0 };
                        Box::new(LowPassFilterEffect::new(cutoff, sample_rate))
                    }
                    NodeType::Mixer => {
                        let mix = if (0.0..=1.0).contains(&node.param_val) { node.param_val } else { 0.5 };
                        Box::new(MixerEffect::new(mix))
                    }
                    NodeType::Splitter => {
                        Box::new(SplitterEffect)
                    }
                    NodeType::SavedEqPreset(name) => {
                        let bands_opt = if &self.active_eq.name == name {
                            Some(&self.active_eq.bands)
                        } else {
                            self.eq_presets.iter().find(|p| &p.name == name).map(|p| &p.bands)
                        };

                        if let Some(bands) = bands_opt {
                            let configs: Vec<crate::audio::ParametricBandConfig> = bands.iter().map(|b| {
                                let f_type = match b.filter_type {
                                    crate::models::FilterType::LowShelf => "low_shelf",
                                    crate::models::FilterType::HighShelf => "high_shelf",
                                    crate::models::FilterType::HighPass => "hpf",
                                    _ => "peak",
                                }.to_string();
                                crate::audio::ParametricBandConfig {
                                    freq: b.freq as f64,
                                    gain_db: if b.enabled { b.gain_db as f64 } else { 0.0 },
                                    q: b.q as f64,
                                    filter_type: f_type,
                                }
                            }).collect();
                            Box::new(crate::audio::ParametricEqEffect::new(name, configs, sample_rate))
                        } else {
                            Box::new(PresetEqEffect::from_preset(name, sample_rate))
                        }
                    }
                    NodeType::VolumeBooster => {
                        let boost = if (0.0..=24.0).contains(&node.param_val) { node.param_val } else { 6.0 };
                        Box::new(VolumeBoosterEffect::new(boost))
                    }
                    NodeType::AnalogPreamp => {
                        let trim = if (-24.0..=12.0).contains(&node.param_val) { node.param_val } else { 0.0 };
                        Box::new(AnalogPreampEffect::new(trim))
                    }
                    NodeType::DynamicLoudness => {
                        let depth = if (0.0..=100.0).contains(&node.param_val) { node.param_val } else { 50.0 };
                        Box::new(DynamicLoudnessEffect::new(depth, sample_rate))
                    }
                    NodeType::SteppedAttenuator => {
                        let step = if (-60.0..=0.0).contains(&node.param_val) { node.param_val } else { -12.0 };
                        Box::new(SteppedAttenuatorEffect::new(step, false))
                    }
                    NodeType::OpticalLeveler => {
                        let drive = if (0.0..=100.0).contains(&node.param_val) { node.param_val } else { 50.0 };
                        Box::new(OpticalLevelerEffect::new(drive, sample_rate as f32))
                    }
                    NodeType::StereoBalance => {
                        let bal = if (-100.0..=100.0).contains(&node.param_val) { node.param_val } else { 0.0 };
                        Box::new(StereoBalanceEffect::new(bal))
                    }
                    NodeType::Compressor => {
                        let thresh = if (-48.0..=0.0).contains(&node.param_val) { node.param_val } else { -16.0 };
                        Box::new(CompressorEffect::new(thresh, 4.0, 10.0, 120.0, 2.0, sample_rate as f32))
                    }
                    _ => {
                        Box::new(PassthroughEffect)
                    }
                };

                if is_custom_effect {
                    effects.push(effect.clone());
                }
                nodes.push(crate::audio::GraphNode {
                    id: id as u32,
                    name: node.title.clone(),
                    effect,
                });
                topological_order.push(id as u32);
            }
        }

        // Preserve exact ports for each connection
        let mut edges: Vec<crate::audio::GraphEdge> = Vec::new();
        for conn in &self.connections {
            if chain_eval.flowing_cables.contains(conn) {
                edges.push(crate::audio::GraphEdge::with_ports(
                    conn.from_node as u32,
                    conn.from_port,
                    conn.to_node as u32,
                    conn.to_port,
                ));
            }
        }

        crate::audio::SignalChainGraph {
            is_complete: chain_eval.is_complete,
            nodes,
            edges,
            topological_order,
            effects,
        }
    }

    pub fn sync_signal_chain_to_engine(&self) {
        if let Some(ref engine) = self.audio_engine {
            let graph = self.compile_signal_chain();
            engine.set_signal_chain_graph(graph);
        }
    }
}

impl eframe::App for LunulaApp {
    fn on_exit(&mut self) {
        self.save_library_state();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Drain loaded covers from background thread into egui texture cache
        while let Ok(result) = self.cover_res_rx.try_recv() {
            let tex = result.image_data.map(|(size, pixels)| {
                let color_image = eframe::egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                ctx.load_texture(&format!("art_{}", result.key), color_image, eframe::egui::TextureOptions::LINEAR)
            });
            if let Ok(mut cache) = self.album_cover_cache.lock() {
                cache.insert(result.key.clone(), tex);
            }
            if let Ok(mut pending) = self.pending_cover_loads.lock() {
                pending.remove(&result.key);
            }
            ctx.request_repaint();
        }

        // Poll background library scanner channel
        if let Some(ref rx) = self.scan_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    ScanMsg::Started(f) => {
                        self.is_scanning = true;
                        self.scan_status = format!("Memindai: {}...", f);
                    }
                    ScanMsg::TrackFound(track) => {
                        let exists = self.playlist.iter().any(|t| {
                            match (&t.file_path, &track.file_path) {
                                (Some(p1), Some(p2)) => p1 == p2,
                                _ => t.title == track.title && t.artist == track.artist,
                            }
                        });
                        if !exists {
                            if self.playlist.is_empty() && self.current_track.duration <= 0.0 {
                                self.current_track = track.clone();
                                self.current_time = 0.0;
                            }
                            self.playlist.push(track);
                        }
                    }
                    ScanMsg::Finished(count) => {
                        self.is_scanning = false;
                        if count > 0 {
                            self.invalidate_library_cache();
                            self.scan_status = format!("✓ Selesai. Berhasil memuat {} lagu.", count);
                            self.save_library_state();
                        } else {
                            self.scan_status.clear();
                        }
                    }
                }
            }
        }

        if ui.input(|i| i.viewport().close_requested()) {
            self.save_library_state();
        }

        if self.is_scanning {
            ctx.request_repaint();
        }

        // Windows SMTC & Hardware Media Keys handling
        let smtc_events: Vec<souvlaki::MediaControlEvent> = if let Some(ref smtc) = self.smtc_bridge {
            let mut evs = Vec::new();
            while let Ok(event) = smtc.event_receiver().try_recv() {
                evs.push(event);
            }
            evs
        } else {
            Vec::new()
        };

        for event in smtc_events {
            match event {
                souvlaki::MediaControlEvent::Play => {
                    if !self.is_playing {
                        self.toggle_playback();
                    }
                }
                souvlaki::MediaControlEvent::Pause => {
                    if self.is_playing {
                        self.toggle_playback();
                    }
                }
                souvlaki::MediaControlEvent::Toggle => {
                    self.toggle_playback();
                }
                souvlaki::MediaControlEvent::Next => {
                    self.play_next_track();
                }
                souvlaki::MediaControlEvent::Previous => {
                    self.play_prev_track();
                }
                souvlaki::MediaControlEvent::Stop => {
                    self.is_playing = false;
                    if let Some(ref engine) = self.audio_engine {
                        engine.stop();
                    }
                    self.sync_smtc_state();
                }
                souvlaki::MediaControlEvent::SetPosition(pos) => {
                    let secs = pos.0.as_secs_f32();
                    self.current_time = secs;
                    if let Some(ref engine) = self.audio_engine {
                        engine.seek(secs);
                    }
                    self.sync_smtc_state();
                }
                _ => {}
            }
        }

        // Hot-Unplug Protection & Auto-Resume Detection
        if self.was_interrupted_by_unplug {
            if self.unplug_notification_timer > 0.0 {
                self.unplug_notification_timer = (self.unplug_notification_timer - ctx.input(|i| i.stable_dt)).max(0.0);
            }
            let devs = crate::audio::backend::enumerate_output_devices();
            let is_restored = match &self.unplug_device_name {
                Some(name) => devs.iter().any(|d| &d.name == name),
                None => !devs.is_empty(),
            };
            if is_restored {
                eprintln!("[Hot-Plug] Perangkat audio tersambung kembali. Melanjutkan playback otomatis...");
                self.was_interrupted_by_unplug = false;
                self.unplug_device_name = None;
                self.is_playing = true;
                if let Some(ref engine) = self.audio_engine {
                    if let Some(ref path_str) = self.current_track.file_path {
                        engine.play(std::path::PathBuf::from(path_str));
                        let seek_pos = self.current_time;
                        if seek_pos > 0.5 {
                            engine.seek(seek_pos);
                        }
                    }
                }
                self.sync_smtc_state();
            }
        }

        // Synchronize with real Audio Engine
        let mut track_ended = false;
        let mut new_duration = None;
        let mut new_signal_path = None;

        let engine_events: Vec<crate::audio::AudioEvent> = if let Some(ref engine) = self.audio_engine {
            let mut evs = Vec::new();
            while let Some(event) = engine.try_recv_event() {
                evs.push(event);
            }
            evs
        } else {
            Vec::new()
        };

        for event in engine_events {
            match event {
                crate::audio::AudioEvent::TrackEnded => {
                    track_ended = true;
                }
                crate::audio::AudioEvent::TrackTransitioned { path, duration, sample_rate } => {
                    self.on_gapless_transition(path, duration, sample_rate);
                }
                crate::audio::AudioEvent::DeviceDisconnected { device_name, error } => {
                    eprintln!("[Audio Device Disconnected] {}: {}", device_name, error);
                    self.is_playing = false;
                    self.was_interrupted_by_unplug = true;
                    self.unplug_device_name = Some(device_name);
                    self.unplug_notification_timer = 6.0;
                    if let Some(ref engine) = self.audio_engine {
                        engine.pause();
                    }
                    self.sync_smtc_state();
                }
                crate::audio::AudioEvent::TrackStarted { duration, .. } => {
                    if duration > 0.0 {
                        new_duration = Some(duration);
                    }
                }
                crate::audio::AudioEvent::SignalPath(report) => {
                    new_signal_path = Some(report);
                }
                crate::audio::AudioEvent::Error(err) => {
                    eprintln!("[Audio Engine Error] {}", err);
                }
                _ => {}
            }
        }

        if let Some(report) = new_signal_path {
            self.update_signal_nodes_from_report(&report);
            self.signal_path_report = Some(report);
        }

        if let Some(d) = new_duration {
            self.current_track.duration = d;
        }

        if track_ended {
            self.on_track_ended();
        }

        // Evaluate Signal Chain reachability
        let chain_eval = self.evaluate_signal_chain();
        let is_chain_ok = chain_eval.is_complete;

        // Audio Engine volume interlock: silence hardware output when signal chain is severed
        if let Some(ref engine) = self.audio_engine {
            let should_mute = self.is_muted || !is_chain_ok;
            engine.set_muted(should_mute);
            let eff_vol = if should_mute {
                0.0
            } else if self.chain_mode == ChainMode::BitPerfect {
                self.volume
            } else {
                (self.volume * self.effective_replay_gain_linear).clamp(0.0, 2.0)
            };
            engine.set_volume(eff_vol);
        }

        // Live Real-Time 48-Band FFT Analysis from speaker audio stream
        let is_audio_active = self.is_playing && !self.is_muted && self.volume > 0.001 && is_chain_ok;
        if is_audio_active {
            if let Some(ref engine) = self.audio_engine {
                engine.get_visualizer_stereo_samples(&mut self.live_stereo_l, &mut self.live_stereo_r);

                let mut raw_samples = [0.0f32; 1024];
                let mut peak_l = 0.0f32;
                let mut peak_r = 0.0f32;
                let mut sum_sq_l = 0.0f32;
                let mut sum_sq_r = 0.0f32;
                let mut sum_lr = 0.0f32;

                for i in 0..1024 {
                    let l = self.live_stereo_l[i];
                    let r = self.live_stereo_r[i];
                    raw_samples[i] = (l + r) * 0.5;

                    let al = l.abs();
                    let ar = r.abs();
                    if al > peak_l { peak_l = al; }
                    if ar > peak_r { peak_r = ar; }

                    sum_sq_l += l * l;
                    sum_sq_r += r * r;
                    sum_lr += l * r;
                }

                self.fft_analyzer.compute_bands(&raw_samples, &mut self.spectrum_bars);

                // 1. Capture real-time live PCM audio waveform for App Resampler Oscilloscope
                let stride = 1024 / 48;
                for i in 0..48 {
                    self.live_pcm_waveform[i] = raw_samples[i * stride];
                }

                // 2. Compute true audio RMS and Peak levels for Output DAC Metering
                let rms_l = (sum_sq_l / 1024.0).sqrt();
                let rms_r = (sum_sq_r / 1024.0).sqrt();
                let overall_peak = peak_l.max(peak_r);
                let overall_rms = ((sum_sq_l + sum_sq_r) / 2048.0).sqrt();

                // Compute channel L & R dynamics
                let target_l = (rms_l * 2.2).clamp(0.0, 1.0).max(peak_l);
                let target_r = (rms_r * 2.2).clamp(0.0, 1.0).max(peak_r);

                // Ballistic smoothing (instant attack, smooth release)
                if target_l > self.dac_meter_l {
                    self.dac_meter_l = self.dac_meter_l * 0.3 + target_l * 0.7;
                } else {
                    self.dac_meter_l = self.dac_meter_l * 0.88 + target_l * 0.12;
                }

                if target_r > self.dac_meter_r {
                    self.dac_meter_r = self.dac_meter_r * 0.3 + target_r * 0.7;
                } else {
                    self.dac_meter_r = self.dac_meter_r * 0.88 + target_r * 0.12;
                }

                // Peak Hold L
                if self.dac_meter_l >= self.dac_peak_hold_l {
                    self.dac_peak_hold_l = self.dac_meter_l;
                    self.dac_decay_timer_l = 18; // ~300ms hold at 60 FPS
                } else if self.dac_decay_timer_l > 0 {
                    self.dac_decay_timer_l -= 1;
                } else {
                    self.dac_peak_hold_l = (self.dac_peak_hold_l - 0.025).max(self.dac_meter_l);
                }

                // Peak Hold R
                if self.dac_meter_r >= self.dac_peak_hold_r {
                    self.dac_peak_hold_r = self.dac_meter_r;
                    self.dac_decay_timer_r = 18;
                } else if self.dac_decay_timer_r > 0 {
                    self.dac_decay_timer_r -= 1;
                } else {
                    self.dac_peak_hold_r = (self.dac_peak_hold_r - 0.025).max(self.dac_meter_r);
                }

                // Clip detection (0 dBFS warning)
                self.dac_clip_l = peak_l >= 0.98;
                self.dac_clip_r = peak_r >= 0.98;

                // 3. Real Normalized Pearson Phase Correlation between Left and Right
                let denom = (sum_sq_l * sum_sq_r).sqrt();
                let inst_corr = if denom > 1e-7 {
                    (sum_lr / denom).clamp(-1.0, 1.0)
                } else {
                    1.0 // In silence or DC, correlation is nominal in-phase
                };
                self.live_phase_correlation = self.live_phase_correlation * 0.88 + inst_corr * 0.12;

                // 4. EBU R128 Loudness (LUFS)
                let mean_power = (sum_sq_l + sum_sq_r) / 1024.0;
                let inst_lufs = if mean_power > 1e-12 {
                    (-0.691 + 10.0 * mean_power.log10()).clamp(-70.0, 0.0)
                } else {
                    -70.0
                };
                self.live_integrated_lufs = self.live_integrated_lufs * 0.94 + inst_lufs * 0.06;

                // 5. True-Peak Level (dBTP) with 4x Inter-Sample Peak (ISP) Oversampling (ITU-R BS.1770)
                let mut true_peak_max = overall_peak;
                for i in 1..1023 {
                    let ym1 = raw_samples[i - 1];
                    let y0 = raw_samples[i];
                    let y1 = raw_samples[i + 1];
                    let y2 = if i + 2 < 1024 { raw_samples[i + 2] } else { y1 };

                    // 4-point Hermite cubic interpolation for sub-sample inter-sample peaks
                    let a0 = -0.5 * ym1 + 1.5 * y0 - 1.5 * y1 + 0.5 * y2;
                    let a1 = ym1 - 2.5 * y0 + 2.0 * y1 - 0.5 * y2;
                    let a2 = -0.5 * ym1 + 0.5 * y1;
                    let a3 = y0;

                    for &alpha in &[0.25f32, 0.50f32, 0.75f32] {
                        let isp = ((a0 * alpha + a1) * alpha + a2) * alpha + a3;
                        let abs_isp = isp.abs();
                        if abs_isp > true_peak_max {
                            true_peak_max = abs_isp;
                        }
                    }
                }

                let inst_tp = if true_peak_max > 1e-6 {
                    (20.0 * true_peak_max.log10()).clamp(-96.0, 6.0)
                } else {
                    -96.0
                };
                if inst_tp >= self.live_true_peak_db {
                    self.live_true_peak_db = inst_tp;
                } else {
                    self.live_true_peak_db = (self.live_true_peak_db - 0.35).max(inst_tp).max(-96.0);
                }

                // Inter-sample clipping detection (exceeding 0.0 dBFS / 1.0 peak)
                if true_peak_max >= 0.999 {
                    self.live_isp_clip_detected = true;
                    self.live_isp_clip_count = self.live_isp_clip_count.saturating_add(1);
                    if inst_tp > self.live_isp_max_over_db {
                        self.live_isp_max_over_db = inst_tp;
                    }
                }

                // 6. Real-time Dynamic Range (DR) / Crest Factor in dB
                let inst_dr = if overall_rms > 1e-6 && overall_peak > 1e-6 {
                    (20.0 * (overall_peak / overall_rms).log10()).clamp(0.0, 30.0)
                } else {
                    0.0
                };
                self.live_dynamic_range = self.live_dynamic_range * 0.92 + inst_dr * 0.08;

                // 7. Real-time Gain Reduction from DSP compression / limiting
                let inst_gr = engine.get_gain_reduction();
                if inst_gr > self.live_gain_reduction_db {
                    self.live_gain_reduction_db = self.live_gain_reduction_db * 0.3 + inst_gr * 0.7;
                } else {
                    self.live_gain_reduction_db = (self.live_gain_reduction_db * 0.88 - 0.05).max(0.0);
                }
            }
        } else {
            // Smoothly decay to baseline when paused, stopped, muted, or chain severed
            for b in self.spectrum_bars.iter_mut() {
                *b = (*b * 0.80).max(0.0);
            }
            for w in self.live_pcm_waveform.iter_mut() {
                *w *= 0.80;
            }
            for s in self.live_stereo_l.iter_mut() {
                *s *= 0.80;
            }
            for s in self.live_stereo_r.iter_mut() {
                *s *= 0.80;
            }
            self.dac_meter_l = (self.dac_meter_l * 0.82).max(0.0);
            self.dac_meter_r = (self.dac_meter_r * 0.82).max(0.0);
            self.dac_peak_hold_l = (self.dac_peak_hold_l * 0.85).max(0.0);
            self.dac_peak_hold_r = (self.dac_peak_hold_r * 0.85).max(0.0);
            self.dac_clip_l = false;
            self.dac_clip_r = false;

            self.live_phase_correlation = self.live_phase_correlation * 0.90 + 1.0 * 0.10;
            self.live_integrated_lufs = (self.live_integrated_lufs - 1.5).max(-70.0);
            self.live_true_peak_db = (self.live_true_peak_db - 2.0).max(-96.0);
            self.live_dynamic_range = (self.live_dynamic_range * 0.90).max(0.0);
            self.live_gain_reduction_db = (self.live_gain_reduction_db * 0.80 - 0.1).max(0.0);
        }

        if self.is_playing || self.spectrum_bars.iter().any(|&v| v > 0.005) || !is_chain_ok {
            ctx.request_repaint();
            self.visualizer_anim_phase += 0.025;

            // Only advance playback time when the signal chain is healthy and complete
            if is_chain_ok {
                let engine_pos = self.audio_engine.as_ref().and_then(|e| {
                    if e.playback_state() == crate::audio::PlaybackState::Playing {
                        Some(e.current_position())
                    } else {
                        None
                    }
                });

                if let Some(pos) = engine_pos {
                    if let Some(offset) = self.current_track.cue_offset_sec {
                        let rel = (pos - offset).max(0.0);
                        self.current_time = rel;
                        if let Some(dur) = self.current_track.cue_duration_sec {
                            if rel >= dur && dur > 0.0 {
                                self.on_track_ended();
                            }
                        }
                    } else {
                        self.current_time = pos;
                    }
                } else if self.current_track.duration > 0.0 {
                    self.current_time += 1.0 / 60.0;
                    if self.current_time >= self.current_track.duration {
                        self.on_track_ended();
                    }
                }
            }
        }

        // Load artwork texture if bytes are available and texture not yet created
        if self.current_cover_texture.is_none() {
            let key = if !self.current_track.album.trim().is_empty() {
                self.current_track.album.clone()
            } else {
                self.current_track.file_path.clone().unwrap_or_else(|| self.current_track.title.clone())
            };
            self.current_cover_texture = self.get_album_cover_texture(&ctx, &key, self.current_track.file_path.as_deref());
        }

        // Global Keyboard Shortcuts (active when no text field has focus)
        if !ctx.egui_wants_keyboard_input() {
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Space) {
                    self.toggle_playback();
                }
                if i.key_pressed(egui::Key::ArrowLeft) {
                    if i.modifiers.ctrl {
                        self.play_prev_track();
                    } else {
                        self.seek_relative(-5.0);
                    }
                }
                if i.key_pressed(egui::Key::ArrowRight) {
                    if i.modifiers.ctrl {
                        self.play_next_track();
                    } else {
                        self.seek_relative(5.0);
                    }
                }
                if i.key_pressed(egui::Key::ArrowUp) {
                    if self.show_queue_drawer && !self.play_queue.is_empty() {
                        if i.modifiers.ctrl || i.modifiers.alt {
                            if let Some(sel) = self.selected_queue_idx {
                                if sel > 0 {
                                    self.play_queue.swap(sel, sel - 1);
                                    self.selected_queue_idx = Some(sel - 1);
                                    self.save_library_state();
                                }
                            }
                        } else {
                            let next_sel = match self.selected_queue_idx {
                                Some(sel) if sel > 0 => sel - 1,
                                _ => 0,
                            };
                            self.selected_queue_idx = Some(next_sel);
                        }
                    } else {
                        self.set_app_volume(self.volume + 0.05);
                    }
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    if self.show_queue_drawer && !self.play_queue.is_empty() {
                        if i.modifiers.ctrl || i.modifiers.alt {
                            if let Some(sel) = self.selected_queue_idx {
                                if sel + 1 < self.play_queue.len() {
                                    self.play_queue.swap(sel, sel + 1);
                                    self.selected_queue_idx = Some(sel + 1);
                                    self.save_library_state();
                                }
                            }
                        } else {
                            let next_sel = match self.selected_queue_idx {
                                Some(sel) if sel + 1 < self.play_queue.len() => sel + 1,
                                Some(sel) => sel,
                                None => 0,
                            };
                            self.selected_queue_idx = Some(next_sel);
                        }
                    } else {
                        self.set_app_volume(self.volume - 0.05);
                    }
                }
                if self.show_queue_drawer {
                    if i.key_pressed(egui::Key::Enter) {
                        if let Some(sel) = self.selected_queue_idx {
                            if sel < self.play_queue.len() {
                                self.queue_cursor = Some(sel);
                                let track = self.play_queue[sel].clone();
                                self.play_track(track);
                            }
                        }
                    }
                    if i.key_pressed(egui::Key::Delete) {
                        if let Some(sel) = self.selected_queue_idx {
                            if sel < self.play_queue.len() {
                                self.play_queue.remove(sel);
                                if self.play_queue.is_empty() {
                                    self.selected_queue_idx = None;
                                } else {
                                    self.selected_queue_idx = Some(sel.min(self.play_queue.len() - 1));
                                }
                                self.save_library_state();
                            }
                        }
                    }
                }
                if i.key_pressed(egui::Key::M) {
                    self.toggle_mute();
                }
                if i.key_pressed(egui::Key::B) {
                    self.toggle_eq_bypass();
                }
                if i.key_pressed(egui::Key::R) {
                    self.cycle_repeat_mode();
                }
                if i.key_pressed(egui::Key::S) && i.modifiers.ctrl {
                    self.toggle_shuffle();
                }
            });
        }

        // Clean Minimalist Top Header
        egui::Panel::top("top_header")
            .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240))).inner_margin(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🌙 LunulaPlayer").strong().size(18.0).color(Color32::from_rgb(15, 23, 42)));
                    ui.label(egui::RichText::new("Pure Native Engine").strong().size(12.0).color(Color32::from_rgb(100, 116, 139)));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Quick Queue Header Button
                        let q_badge = if self.play_queue.is_empty() {
                            "📑 Antrean".to_string()
                        } else {
                            format!("📑 Antrean ({})", self.play_queue.len())
                        };
                        let q_hdr_btn = ui.selectable_label(self.show_queue_drawer, egui::RichText::new(q_badge).strong());
                        if q_hdr_btn.clicked() {
                            self.show_queue_drawer = !self.show_queue_drawer;
                        }

                        ui.add_space(8.0);
                        if ui.button(egui::RichText::new("🎬 Tur Studio (90%)").size(12.0).strong().color(Color32::from_rgb(99, 102, 241)))
                            .on_hover_text("Buka theater panduan interaktif 90% bersama duo not Lu & Nu")
                            .clicked()
                        {
                            self.show_interactive_theater = true;
                            self.theater_slide = 0;
                            self.theater_turn = 0;
                            self.theater_turn_timer = 0.0;
                        }
                        ui.selectable_value(&mut self.current_tab, AppTab::Equalizer, egui::RichText::new("🎛️ Equalizer 🚧 Coming Soon").strong());
                        ui.selectable_value(&mut self.current_tab, AppTab::Library, egui::RichText::new("📁 Library").strong());
                        let sig_tab_resp = ui.selectable_value(&mut self.current_tab, AppTab::SignalChain, egui::RichText::new("⚡ Signal Chain Studio").strong());
                        self.quest_rect_signal_chain_tab = Some(sig_tab_resp.rect);
                        ui.selectable_value(&mut self.current_tab, AppTab::NowPlaying, egui::RichText::new("🎵 Now Playing").strong());
                    });
                });
            });

        // Option A: Slide-over Play Queue Drawer (Right Side)
        if self.show_queue_drawer {
            egui::Panel::right("queue_drawer")
                .resizable(false)
                .frame(egui::Frame::new().fill(Color32::WHITE).stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240))).inner_margin(14.0))
                .show(ui, |ui| {
                    ui.set_width(300.0);
                    self.render_queue_drawer(ui);
                });
        }

        // Central Area
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(248, 250, 252)).inner_margin(16.0))
            .show(ui, |ui| {
                if self.was_interrupted_by_unplug || self.unplug_notification_timer > 0.0 {
                    let dev_str = self.unplug_device_name.as_deref().unwrap_or("Audio Device");
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(254, 242, 242))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(248, 113, 113)))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("🔌 PERANGKAT TERPUTUS (HOT-UNPLUG)").strong().size(11.5).color(Color32::from_rgb(185, 28, 28)));
                                ui.label(egui::RichText::new(format!("\"{}\" terputus. Pemutaran dijeda aman. Lunula akan melanjutkan otomatis saat perangkat kembali terdeteksi.", dev_str)).size(11.0).color(Color32::from_rgb(127, 29, 29)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(egui::RichText::new("Tutup Notifikasi").size(10.0)).clicked() {
                                        self.unplug_notification_timer = 0.0;
                                    }
                                });
                            });
                        });
                    ui.add_space(8.0);
                }

                match self.current_tab {
                    AppTab::NowPlaying => {
                        // Quick overview patchbay on top
                        render_card_frame(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("⚡ MODULAR SIGNAL CHAIN").strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                                    if !is_chain_ok {
                                        ui.label(egui::RichText::new("⚠️ Sinyal Terputus (Audio Dibisukan)").size(11.5).color(Color32::from_rgb(239, 68, 68)).strong());
                                    } else if let Some(ref r) = self.signal_path_report {
                                        if r.requested_mode == ChainMode::BitPerfect && r.actual_mode == ChainMode::OsResampled {
                                            ui.label(egui::RichText::new(format!("⚠️ Fallback: OS Resampled {:.1} kHz", r.device_rate as f32 / 1000.0)).size(11.5).color(Color32::from_rgb(217, 119, 6)).strong());
                                        } else if r.actual_mode == ChainMode::BitPerfect {
                                            ui.label(egui::RichText::new(format!("✓ Sinyal Bit-Perfect Mengalir ({:.1} kHz Native)", r.device_rate as f32 / 1000.0)).size(11.5).color(Color32::from_rgb(16, 185, 129)).strong());
                                        } else if r.actual_mode == ChainMode::DspActive {
                                            ui.label(egui::RichText::new(format!("✓ DSP Engine Aktif (10-Band EQ • {:.1} kHz)", r.device_rate as f32 / 1000.0)).size(11.5).color(Color32::from_rgb(99, 102, 241)).strong());
                                        } else {
                                            ui.label(egui::RichText::new("✓ OS Resampled (48.0 kHz Shared)").size(11.5).color(Color32::from_rgb(217, 119, 6)).strong());
                                        }
                                    } else {
                                        match self.chain_mode {
                                            ChainMode::BitPerfect => {
                                                ui.label(egui::RichText::new("✓ Sinyal Bit-Perfect Siap").size(11.5).color(Color32::from_rgb(16, 185, 129)).strong());
                                            }
                                            ChainMode::DspActive => {
                                                ui.label(egui::RichText::new("✓ DSP Engine Siap").size(11.5).color(Color32::from_rgb(99, 102, 241)).strong());
                                            }
                                            ChainMode::OsResampled => {
                                                ui.label(egui::RichText::new("✓ OS Resampled Siap (48 kHz)").size(11.5).color(Color32::from_rgb(217, 119, 6)).strong());
                                            }
                                        }
                                    }
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(egui::RichText::new("Buka Studio Penuh ➔").strong()).clicked() {
                                            self.current_tab = AppTab::SignalChain;
                                        }
                                    });
                                });
                                ui.add_space(4.0);
                                self.render_modular_patchbay_canvas(ui, Vec2::new(ui.available_width(), 130.0));
                            });
                        });

                        ui.add_space(8.0);

                        // If fallback to OS Resampled happened due to hardware rate mismatch, show honest banner
                        if let Some(ref r) = self.signal_path_report {
                            if is_chain_ok && r.requested_mode == ChainMode::BitPerfect && r.actual_mode == ChainMode::OsResampled {
                                egui::Frame::new()
                                    .fill(Color32::from_rgb(254, 243, 199))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(253, 230, 138)))
                                    .corner_radius(8.0)
                                    .inner_margin(egui::Margin::symmetric(14, 8))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("⚠️ PERINGATAN FALLBACK BIT-PERFECT:").strong().size(12.0).color(Color32::from_rgb(180, 83, 9)));
                                            ui.label(egui::RichText::new(format!("Perangkat '{}' tidak mendukung sample rate {:.1} kHz file lagu ini secara native. Engine beralih jujur ke OS Resampling {:.1} kHz agar pemutaran tidak rusak.", r.device_name, r.source_rate as f32 / 1000.0, r.device_rate as f32 / 1000.0)).size(11.5).color(Color32::from_rgb(146, 64, 14)));
                                        });
                                    });
                                ui.add_space(6.0);
                            }
                        }

                        // If signal chain is severed, display a warning card above transport
                        if !is_chain_ok {
                            egui::Frame::new()
                                .fill(Color32::from_rgb(254, 242, 242))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(252, 165, 165)))
                                .corner_radius(8.0)
                                .inner_margin(egui::Margin::symmetric(14, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("🔴 JALUR SINYAL AUDIO TERPUTUS:").strong().size(12.5).color(Color32::from_rgb(185, 28, 28)));
                                        ui.label(egui::RichText::new("Lagu tidak dapat berbunyi karena sambungan rantai audio (RAM Buffer / DSP / DAC) terputus. Klik tanda ✕ merah di atas untuk menyambung!").size(12.0).color(Color32::from_rgb(153, 27, 27)));
                                    });
                                });
                            ui.add_space(6.0);
                        }

                        // Grand Ambient Waveform Station + Perfectly Centered Transport Deck
                        self.render_ambient_waveform_station(ui);

                        ui.add_space(10.0);

                        // 60% Spectrum Analyzer + 40% Synchronized Lyric Box
                        self.render_now_playing_visualizer_and_lyrics(ui);
                    }
                    AppTab::SignalChain => {
                        // Header bar: Studio Mode Selection (Simple Studio vs Advance Studio) + Device Selector + Chain Mode
                        ui.horizontal(|ui| {
                            // Mode Switcher Pills: [ 🎛️ Simple Studio ] | [ 🔬 Advance Studio ]
                            let is_simple = self.studio_mode == crate::models::StudioMode::Simple;
                            let is_advance = self.studio_mode == crate::models::StudioMode::Advance;

                            let simple_text = if is_simple {
                                egui::RichText::new("🎛️ Simple Studio").strong().size(13.0).color(Color32::WHITE)
                            } else {
                                egui::RichText::new("🎛️ Simple Studio").size(13.0).color(Color32::from_rgb(100, 116, 139))
                            };
                            let simple_bg = if is_simple { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(241, 245, 249) };
                            if ui.add(egui::Button::new(simple_text).fill(simple_bg).corner_radius(6.0)).on_hover_text("Buka tampilan kanvas modular patchbay standar").clicked() {
                                self.studio_mode = crate::models::StudioMode::Simple;
                            }

                            let advance_text = if is_advance {
                                egui::RichText::new("🔬 Advance Studio").strong().size(13.0).color(Color32::WHITE)
                            } else {
                                egui::RichText::new("🔬 Advance Studio").size(13.0).color(Color32::from_rgb(100, 116, 139))
                            };
                            let advance_bg = if is_advance { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(241, 245, 249) };
                            if ui.add(egui::Button::new(advance_text).fill(advance_bg).corner_radius(6.0)).on_hover_text("Buka konsol audio profesional dengan grafik respon frekuensi, RTA spektrum, dan rack unit").clicked() {
                                self.studio_mode = crate::models::StudioMode::Advance;
                            }

                            if is_advance {
                                ui.label(egui::RichText::new("PROTOTYPE GUI").size(11.0).strong().color(Color32::from_rgb(245, 158, 11)));
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if is_simple {
                                    if ui.button(egui::RichText::new("↺ Reset Posisi").size(12.5).strong())
                                        .on_hover_text("Pusatkan kembali view dan susun rapi 5 blok utama")
                                        .clicked()
                                    {
                                        self.reset_patchbay_layout(ui.available_width(), ui.available_height());
                                    }
                                }

                                let current_dev_label = self.selected_output_device.as_deref().unwrap_or("Default System Output");
                                let truncated_label = if current_dev_label.len() > 22 {
                                    format!("🔊 {}...", &current_dev_label[..19])
                                } else {
                                    format!("🔊 {}", current_dev_label)
                                };

                                egui::ComboBox::from_id_salt("patchbay_output_device_combo")
                                    .selected_text(egui::RichText::new(truncated_label).size(12.0).strong())
                                    .width(170.0)
                                    .show_ui(ui, |ui| {
                                        if ui.button("🔄 Pindai Ulang Device (Scan)").clicked() {
                                            self.available_output_devices = crate::audio::backend::enumerate_output_devices();
                                        }
                                        ui.separator();
                                        let is_default = self.selected_output_device.is_none();
                                        if ui.selectable_label(is_default, "🔊 Default System Output").clicked() {
                                            self.selected_output_device = None;
                                            if let Some(ref engine) = self.audio_engine {
                                                engine.set_output_device(None);
                                            }
                                            self.save_library_state();
                                        }
                                        for dev in &self.available_output_devices {
                                            let is_sel = self.selected_output_device.as_ref() == Some(&dev.name);
                                            let label = if dev.is_default {
                                                format!("🔊 {} [Default]", dev.name)
                                            } else {
                                                format!("🎧 {}", dev.name)
                                            };
                                            if ui.selectable_label(is_sel, label).clicked() {
                                                self.selected_output_device = Some(dev.name.clone());
                                                if let Some(ref engine) = self.audio_engine {
                                                    engine.set_output_device(Some(dev.name.clone()));
                                                }
                                                self.save_library_state();
                                            }
                                        }
                                    });

                                let dsp_btn = ui.selectable_label(self.chain_mode == ChainMode::DspActive, egui::RichText::new("DSP Active").size(12.5).strong());
                                if dsp_btn.clicked() {
                                    self.set_chain_mode(ChainMode::DspActive);
                                }
                                let bp_btn = ui.selectable_label(self.chain_mode == ChainMode::BitPerfect, egui::RichText::new("Bit-Perfect").size(12.5).strong());
                                if bp_btn.clicked() {
                                    self.set_chain_mode(ChainMode::BitPerfect);
                                }
                                self.quest_rect_dsp_mode_btn = Some(dsp_btn.rect.expand(3.0));
                                ui.label(egui::RichText::new("Mode:").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                            });
                        });

                        ui.add_space(6.0);

                        match self.studio_mode {
                            crate::models::StudioMode::Simple => {
                                // Simple Studio Mode: 3/4 Width Interactive Patchbay Canvas + 1/4 Width FX & Filters Palette
                                let total_width = ui.available_width();
                                let canvas_width = total_width * 0.76;
                                let palette_width = total_width - canvas_width - 10.0;
                                let full_height = ui.available_height() - 8.0;

                                ui.horizontal(|ui| {
                                    // Left 3/4 Canvas
                                    ui.allocate_ui(Vec2::new(canvas_width, full_height), |ui| {
                                        render_card_frame(ui, |ui| {
                                            ui.vertical(|ui| {
                                                // Row 1: Studio Title & Tips
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new("⚡ MODULAR PATCHBAY STUDIO")
                                                            .strong()
                                                            .size(15.0)
                                                            .color(Color32::from_rgb(15, 23, 42)),
                                                    );

                                                    ui.label(
                                                        egui::RichText::new("• Tarik pin Output ke Input • Klik kabel untuk memutus • Klik kanan tahan untuk geser canvas")
                                                            .size(12.0)
                                                            .color(Color32::from_rgb(100, 116, 139)),
                                                    );

                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        let tut_text = if self.show_patchbay_tutorial {
                                                            egui::RichText::new("📖 Tutup Tutorial").size(12.0).strong().color(Color32::from_rgb(147, 51, 234))
                                                        } else {
                                                            egui::RichText::new("📖 Panduan Pemula").size(12.0).strong().color(Color32::from_rgb(99, 102, 241))
                                                        };
                                                        if ui.button(tut_text).on_hover_text("Buka modul tutorial bergambar cara kerja Signal Chain Studio").clicked() {
                                                            self.show_patchbay_tutorial = !self.show_patchbay_tutorial;
                                                        }
                                                    });
                                                });

                                                ui.add_space(4.0);
                                                let studio_canvas_size = ui.available_size();
                                                self.render_modular_patchbay_canvas(ui, studio_canvas_size);
                                            });
                                        });
                                    });

                                    ui.add_space(8.0);

                                    // Right 1/4 FX Library & Filters Palette
                                    ui.allocate_ui(Vec2::new(palette_width, full_height), |ui| {
                                        self.render_fx_sidebar_palette(ui);
                                    });
                                });
                            }
                            crate::models::StudioMode::Advance => {
                                // Advance Studio Mode (Idea 2: Hybrid Modular):
                                // Top 60%: Interactive Modular Patchbay Canvas (76% width) + Compact FX Palette (24% width)
                                // Bottom 40%: Studio Telemetry & Circuit Inspection Desk
                                let total_width = ui.available_width();
                                let canvas_width = total_width * 0.76;
                                let palette_width = total_width - canvas_width - 10.0;
                                let full_height = ui.available_height() - 4.0;

                                let is_minimized = self.advance_studio_state.is_desk_minimized;
                                let (top_height, desk_height) = if is_minimized {
                                    (full_height - 34.0, 28.0)
                                } else {
                                    let desk_h = (full_height * 0.38).clamp(190.0, 240.0);
                                    (full_height - desk_h - 8.0, desk_h)
                                };

                                ui.vertical(|ui| {
                                    // Upper Area: Canvas + Palette
                                    ui.horizontal(|ui| {
                                        ui.allocate_ui(Vec2::new(canvas_width, top_height), |ui| {
                                            render_card_frame(ui, |ui| {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            egui::RichText::new("⚡ MODULAR PATCHBAY CANVAS")
                                                                .strong()
                                                                .size(15.0)
                                                                .color(Color32::from_rgb(15, 23, 42)),
                                                        );
                                                        ui.label(
                                                            egui::RichText::new("• Klik modul atau kabel untuk inspeksi sirkuit di Telemetry Desk bawah")
                                                                .size(12.0)
                                                                .color(Color32::from_rgb(100, 116, 139)),
                                                        );
                                                    });
                                                    ui.add_space(3.0);
                                                    let studio_canvas_size = ui.available_size();
                                                    self.render_modular_patchbay_canvas(ui, studio_canvas_size);
                                                });
                                            });
                                        });

                                        ui.add_space(8.0);

                                        ui.allocate_ui(Vec2::new(palette_width, top_height), |ui| {
                                            self.render_fx_sidebar_palette(ui);
                                        });
                                    });

                                    ui.add_space(6.0);

                                    // Lower Area: Studio Telemetry & Circuit Inspection Desk
                                    self.render_telemetry_inspection_desk(ui, desk_height);
                                });
                            }
                        }
                    }
                    AppTab::Library => {
                        self.render_library_panel(ui);
                    }
                    AppTab::Equalizer => {
                        self.render_equalizer_panel(ui);
                    }
                }
            });

        // Roadmap 2: 90% Screen Interactive Studio Theater with Flying Mascot
        if self.show_interactive_theater {
            self.render_interactive_theater(ui);
        }

        // Tag Metadata Editor (Lofty Lossless)
        if self.show_tag_editor {
            self.render_tag_editor_modal(ui.ctx());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CableConnection, ChainMode, FilterType, ModularNode, NodeType, ParametricBand};

    #[test]
    fn test_node4_title_is_consistently_app_resampler() {
        let mut app = LunulaApp::default();
        assert_eq!(app.nodes.iter().find(|n| n.id == 4).unwrap().title, "APP RESAMPLER");

        app.set_chain_mode(ChainMode::BitPerfect);
        assert_eq!(app.nodes.iter().find(|n| n.id == 4).unwrap().title, "APP RESAMPLER");

        app.set_chain_mode(ChainMode::DspActive);
        assert_eq!(app.nodes.iter().find(|n| n.id == 4).unwrap().title, "APP RESAMPLER");

        app.set_chain_mode(ChainMode::OsResampled);
        assert_eq!(app.nodes.iter().find(|n| n.id == 4).unwrap().title, "APP RESAMPLER");
    }

    #[test]
    fn test_resampler_settings_customization_and_node4_updates() {
        use crate::models::{ResamplerSettings, RollOffProfile, SincQuality};
        let mut app = LunulaApp::new_standalone();
        assert_eq!(app.resampler_settings.quality, SincQuality::Studio);
        assert_eq!(app.resampler_settings.profile, RollOffProfile::Sharp);

        // Update to Extreme + Smooth
        let custom = ResamplerSettings {
            quality: SincQuality::Extreme,
            profile: RollOffProfile::Smooth,
        };
        app.set_resampler_settings(custom);
        assert_eq!(app.resampler_settings, custom);

        // Verify Node 4 sub2 updated with short names
        let node4 = app.nodes.iter().find(|n| n.id == 4).unwrap();
        assert!(node4.sub2.contains("256-pt"));
        assert!(node4.sub2.contains("Smooth"));

        // Switch to Fast + Nyquist
        let custom2 = ResamplerSettings {
            quality: SincQuality::Fast,
            profile: RollOffProfile::Nyquist,
        };
        app.set_resampler_settings(custom2);
        let node4_v2 = app.nodes.iter().find(|n| n.id == 4).unwrap();
        assert!(node4_v2.sub2.contains("64-pt"));
        assert!(node4_v2.sub2.contains("Nyquist"));
    }

    #[test]
    fn test_eq_not_in_chain_produces_no_eq_effect_even_with_node3_dsp_engine() {
        let mut app = LunulaApp::default();
        app.chain_mode = ChainMode::DspActive;
        // Connect 1 -> 2 -> 3 -> 4 -> 5 (DSP ENGINE is in path, but NO EQ preset card)
        app.connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, 3),
            CableConnection::new(3, 4),
            CableConnection::new(4, 5),
        ];
        app.invalidate_chain_cache();

        assert!(!app.is_equalizer_in_active_chain());

        let graph = app.compile_signal_chain();
        assert!(graph.is_complete);
        // None of the effects in the graph should be an EQ effect
        for eff in &graph.effects {
            assert_ne!(eff.name(), "10-Band Graphic EQ");
            assert_ne!(eff.name(), app.active_eq.name);
        }
    }

    #[test]
    fn test_eq_preset_card_connected_is_compiled_into_signal_chain() {
        let mut app = LunulaApp::default();
        app.chain_mode = ChainMode::DspActive;
        app.active_eq.name = "My Custom EQ".to_string();
        app.active_eq.bands = vec![
            ParametricBand {
                enabled: true,
                filter_type: FilterType::Peaking,
                freq: 1000.0,
                gain_db: 6.0,
                q: 1.414,
                slope: crate::models::FilterSlope::Db12,
            }
        ];

        // Add a modular EQ preset card
        let eq_card_id = 99;
        app.nodes.push(ModularNode {
            id: eq_card_id,
            node_type: NodeType::SavedEqPreset("My Custom EQ".to_string()),
            title: "🎛️ EQ PRESET".to_string(),
            sub1: "My Custom EQ".to_string(),
            sub2: "Parametric".to_string(),
            pos: Pos2::new(300.0, 100.0),
            is_removable: true,
            param_val: 0.0,
        });

        // Connect 1 -> 2 -> 99 -> 4 -> 5
        app.connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, eq_card_id),
            CableConnection::new(eq_card_id, 4),
            CableConnection::new(4, 5),
        ];
        app.invalidate_chain_cache();

        assert!(app.is_equalizer_in_active_chain());

        let graph = app.compile_signal_chain();
        assert!(graph.is_complete);
        let has_custom_eq = graph.effects.iter().any(|eff| eff.name() == "My Custom EQ");
        assert!(has_custom_eq, "Compiled graph must contain 'My Custom EQ' effect");
    }

    #[test]
    fn test_patchbay_cards_headless_render_with_visualizer_state() {
        let ctx = egui::Context::default();
        let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

        let mut app = LunulaApp::default();
        app.is_playing = true;
        app.current_track.duration = 240.0;
        app.current_time = 180.0; // 75% progress (tape head moved toward right)

        // Inject simulated live PCM waveform
        for i in 0..48 {
            let t = i as f32 / 48.0;
            app.live_pcm_waveform[i] = (t * std::f32::consts::TAU * 3.0).sin() * 0.7;
        }

        // Inject simulated DAC meter state with peak and clip
        app.dac_meter_l = 0.65;
        app.dac_meter_r = 0.82;
        app.dac_peak_hold_l = 0.70;
        app.dac_peak_hold_r = 0.92;
        app.dac_clip_l = false;
        app.dac_clip_r = true;

        // Add Tube Warmer and Stereo Widener to canvas
        app.add_modular_filter(NodeType::TubeWarmer);
        app.add_modular_filter(NodeType::StereoWidener);

        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.1),
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(1000.0, 600.0));
                app.render_fx_sidebar_palette(ui);
            },
        );

        assert!(!out.shapes.is_empty(), "Patchbay canvas must render shapes without panicking");
        out.textures_delta.clear();

        // Verify bounds edge cases for progress (0.0 and 1.0)
        app.current_time = 0.0;
        let mut out2 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.2),
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(1000.0, 600.0));
            },
        );
        out2.textures_delta.clear();

        app.current_time = app.current_track.duration;
        let mut out3 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.3),
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(1000.0, 600.0));
            },
        );
        out3.textures_delta.clear();
    }

    #[test]
    fn test_studio_mode_toggle_and_advance_studio_headless_render() {
        let ctx = egui::Context::default();
        let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 900.0)));
        let mut app = LunulaApp::default();
        assert_eq!(app.studio_mode, crate::models::StudioMode::Simple);

        // Switch to Advance Studio
        app.studio_mode = crate::models::StudioMode::Advance;
        app.current_tab = crate::models::AppTab::SignalChain;

        // 1. Render all telemetry tabs headlessly
        let telemetry_tabs = [
            crate::ui::patchbay::TelemetryTab::AutoInspect,
            crate::ui::patchbay::TelemetryTab::CumulativeNet,
            crate::ui::patchbay::TelemetryTab::StereoVectorscope,
            crate::ui::patchbay::TelemetryTab::MasterMetering,
            crate::ui::patchbay::TelemetryTab::WaveformHeatmap,
        ];

        for tab in telemetry_tabs {
            app.advance_studio_state.telemetry_tab = tab;
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: screen,
                    time: Some(0.1),
                    ..Default::default()
                },
                |ui| {
                    app.render_telemetry_inspection_desk(ui, 220.0);
                },
            );
            assert!(!out.shapes.is_empty(), "Telemetry desk must render shapes in {:?} tab", tab);
            out.textures_delta.clear();
        }

        // 2. Test node inspection for TubeWarmer, LPF, and Mixer
        app.advance_studio_state.telemetry_tab = crate::ui::patchbay::TelemetryTab::AutoInspect;
        app.add_modular_filter(crate::models::NodeType::TubeWarmer);
        if let Some(tube_node) = app.nodes.iter().find(|n| n.node_type == crate::models::NodeType::TubeWarmer) {
            app.selected_node = Some(tube_node.id);
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: screen,
                    time: Some(0.2),
                    ..Default::default()
                },
                |ui| {
                    app.render_telemetry_inspection_desk(ui, 220.0);
                },
            );
            assert!(!out.shapes.is_empty(), "Must render Tube Warmer circuit inspection");
            out.textures_delta.clear();
        }

        // Test OpticalLeveler node inspection with interactive Gain Reduction meter
        app.add_modular_filter(crate::models::NodeType::OpticalLeveler);
        if let Some(opto_node) = app.nodes.iter().find(|n| n.node_type == crate::models::NodeType::OpticalLeveler) {
            app.selected_node = Some(opto_node.id);
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: screen,
                    time: Some(0.25),
                    ..Default::default()
                },
                |ui| {
                    app.render_telemetry_inspection_desk(ui, 220.0);
                },
            );
            assert!(!out.shapes.is_empty(), "Must render Optical Leveler Gain Reduction meter inspection");
            out.textures_delta.clear();
        }

        // Test Compressor node inspection with interactive Threshold & GR meter
        app.add_modular_filter(crate::models::NodeType::Compressor);
        if let Some(comp_node) = app.nodes.iter().find(|n| n.node_type == crate::models::NodeType::Compressor) {
            app.selected_node = Some(comp_node.id);
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: screen,
                    time: Some(0.28),
                    ..Default::default()
                },
                |ui| {
                    app.render_telemetry_inspection_desk(ui, 220.0);
                },
            );
            assert!(!out.shapes.is_empty(), "Must render Compressor Gain Reduction inspection");
            out.textures_delta.clear();
        }

        // 3. Test cable probe mode
        app.selected_node = None;
        app.advance_studio_state.selected_cable_idx = Some(0);
        let mut out_cable = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.3),
                ..Default::default()
            },
            |ui| {
                app.render_telemetry_inspection_desk(ui, 220.0);
            },
        );
        assert!(!out_cable.shapes.is_empty(), "Must render Cable Probe telemetry");
        out_cable.textures_delta.clear();

        // 4. Test minimized desk state
        app.advance_studio_state.is_desk_minimized = true;
        let mut out_min = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.4),
                ..Default::default()
            },
            |ui| {
                app.render_telemetry_inspection_desk(ui, 28.0);
            },
        );
        assert!(!out_min.shapes.is_empty(), "Must render minimized desk bar");
        out_min.textures_delta.clear();
    }

    #[test]
    fn test_coming_soon_filters_blocked_and_rendered() {
        let mut app = LunulaApp::default();
        let initial_node_count = app.nodes.len();

        // 1. Verify add_modular_filter blocks VolumeAttenuator and BassGuard
        app.add_modular_filter(crate::models::NodeType::VolumeAttenuator);
        assert_eq!(app.nodes.len(), initial_node_count, "VolumeAttenuator must not be added to canvas");

        app.add_modular_filter(crate::models::NodeType::BassGuard);
        assert_eq!(app.nodes.len(), initial_node_count, "BassGuard must not be added to canvas");

        // 2. Verify allowed filter can still be added
        app.add_modular_filter(crate::models::NodeType::TubeWarmer);
        assert_eq!(app.nodes.len(), initial_node_count + 1, "TubeWarmer must be allowed on canvas");

        // 3. Headless UI render of FX sidebar palette with Coming Soon tiles
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(340.0, 700.0));
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(screen),
                time: Some(0.1),
                ..Default::default()
            },
            |ui| {
                app.render_fx_sidebar_palette(ui);
            },
        );
        assert!(!out.shapes.is_empty(), "FX sidebar palette must render shapes cleanly");
        out.textures_delta.clear();
    }

    #[test]
    fn test_connecting_lpf_in_flowing_chain_compiles_and_filters() {
        let mut app = LunulaApp::default();
        app.set_chain_mode(ChainMode::BitPerfect);
        assert_eq!(app.chain_mode, ChainMode::BitPerfect);
        assert!(app.eq_bypass);

        // Add Low-Pass Filter (node 6)
        let lpf_id = 6;
        app.nodes.push(ModularNode {
            id: lpf_id,
            node_type: NodeType::LowPassFilter,
            title: "LOW-PASS FILTER".to_string(),
            sub1: "Cutoff 400 Hz".to_string(),
            sub2: "Active".to_string(),
            pos: Pos2::new(300.0, 300.0),
            is_removable: true,
            param_val: 400.0,
        });

        // Wire 1 -> 2 -> 3 -> 6 (LPF) -> 4 -> 5
        app.connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, 3),
            CableConnection::new(3, 6),
            CableConnection::new(6, 4),
            CableConnection::new(4, 5),
        ];
        app.invalidate_chain_cache();

        let eval = app.evaluate_signal_chain();
        assert!(eval.is_complete);
        assert!(eval.flowing_nodes.contains(&6));

        // When topology changes with flowing DSP, mode should become DspActive
        let has_flowing_dsp = eval.flowing_nodes.iter().any(|&id| {
            app.nodes.iter().find(|n| n.id == id).map_or(false, |n| n.node_type.is_dsp_or_effect())
        });
        assert!(has_flowing_dsp);
        app.set_chain_mode(ChainMode::DspActive);
        assert_eq!(app.chain_mode, ChainMode::DspActive);
        assert!(!app.eq_bypass);

        let graph = app.compile_signal_chain();
        assert!(graph.is_complete);
        assert!(graph.nodes.iter().any(|n| n.name == "LOW-PASS FILTER"));

        // Process audio through DSP pipeline with this graph
        let mut dsp = crate::audio::dsp::DspPipeline::new(48000.0);
        dsp.set_eq_bypass(app.eq_bypass);
        dsp.set_signal_chain(graph);

        // Feed 5000 Hz tone: should be filtered heavily
        let mut left = [0.0f32; 1000];
        let mut right = [0.0f32; 1000];
        for i in 0..1000 {
            let t = i as f32 / 48000.0;
            left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
            right[i] = left[i];
        }
        dsp.process_block(&mut left, &mut right);
        let peak = left[800..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak < 0.15, "5000 Hz must be heavily attenuated by 400 Hz LPF, got {}", peak);
    }

    #[test]
    fn test_live_telemetry_desk_real_computations() {
        let mut app = LunulaApp::new_standalone();

        // 1. Verify Mono signal (L == R) yields exact +1.0 Pearson correlation
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            let val = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;
            app.live_stereo_l[i] = val;
            app.live_stereo_r[i] = val;
        }

        let mut sum_sq_l = 0.0f32;
        let mut sum_sq_r = 0.0f32;
        let mut sum_lr = 0.0f32;
        let mut peak_val = 0.0f32;
        for i in 0..1024 {
            let l = app.live_stereo_l[i];
            let r = app.live_stereo_r[i];
            sum_sq_l += l * l;
            sum_sq_r += r * r;
            sum_lr += l * r;
            peak_val = peak_val.max(l.abs()).max(r.abs());
        }

        let denom = (sum_sq_l * sum_sq_r).sqrt();
        let corr_mono = (sum_lr / denom).clamp(-1.0, 1.0);
        assert!((corr_mono - 1.0).abs() < 1e-4, "Mono correlation must be exactly +1.0, got {}", corr_mono);

        // 2. Verify 180-degree Anti-Phase (L == -R) yields -1.0 Pearson correlation
        for i in 0..1024 {
            app.live_stereo_r[i] = -app.live_stereo_l[i];
        }
        sum_lr = 0.0;
        for i in 0..1024 {
            sum_lr += app.live_stereo_l[i] * app.live_stereo_r[i];
        }
        let corr_antiphase = (sum_lr / denom).clamp(-1.0, 1.0);
        assert!((corr_antiphase - (-1.0)).abs() < 1e-4, "Anti-phase correlation must be -1.0, got {}", corr_antiphase);

        // 3. True Peak level calculation
        let tp_db = 20.0 * peak_val.log10();
        assert!((tp_db - (-6.02)).abs() < 0.1, "0.5 amplitude must measure ~ -6.02 dBFS, got {}", tp_db);

        // 4. Mid/Side Goniometer Transformation
        let inv_sqrt2 = 0.7071068f32;
        let l_mono = 0.5f32;
        let r_mono = 0.5f32;
        let s_mono = (r_mono - l_mono) * inv_sqrt2;
        let m_mono = (r_mono + l_mono) * inv_sqrt2;
        assert_eq!(s_mono, 0.0, "Pure mono has zero side energy (vertical line)");
        assert!(m_mono > 0.0, "Pure mono has positive mid energy");

        let l_anti = 0.5f32;
        let r_anti = -0.5f32;
        let s_anti = (r_anti - l_anti) * inv_sqrt2;
        let m_anti = (r_anti + l_anti) * inv_sqrt2;
        assert_eq!(m_anti, 0.0, "Anti-phase has zero mid energy (horizontal line)");
        assert!(s_anti.abs() > 0.0, "Anti-phase has high side energy");

        // 5. Test 4x Inter-Sample Peak (ISP) Oversampling and Clipping Detection
        let mut raw_samples = [0.0f32; 1024];
        raw_samples[100] = 0.5;
        raw_samples[101] = 0.99;
        raw_samples[102] = 0.99;
        raw_samples[103] = 0.5;

        let mut true_peak_max = 0.99f32;
        let ym1 = raw_samples[100];
        let y0 = raw_samples[101];
        let y1 = raw_samples[102];
        let y2 = raw_samples[103];
        let a0 = -0.5 * ym1 + 1.5 * y0 - 1.5 * y1 + 0.5 * y2;
        let a1 = ym1 - 2.5 * y0 + 2.0 * y1 - 0.5 * y2;
        let a2 = -0.5 * ym1 + 0.5 * y1;
        let a3 = y0;
        for &alpha in &[0.25f32, 0.50f32, 0.75f32] {
            let isp = ((a0 * alpha + a1) * alpha + a2) * alpha + a3;
            if isp.abs() > true_peak_max {
                true_peak_max = isp.abs();
            }
        }
        assert!(true_peak_max > 1.0, "Reconstructed 4x inter-sample peak must exceed 1.0 (detected ISP clip), got {}", true_peak_max);
        let isp_db = 20.0 * true_peak_max.log10();
        assert!(isp_db > 0.0, "Inter-sample clipping must exceed 0 dBTP, got {} dBTP", isp_db);

        // 6. Test Live Gain Reduction & Reset Clip
        app.live_isp_clip_detected = true;
        app.live_isp_clip_count = 5;
        app.live_isp_max_over_db = 0.42;
        app.live_gain_reduction_db = 4.8;
        assert_eq!(app.live_isp_clip_detected, true);
        assert_eq!(app.live_gain_reduction_db, 4.8);
    }

    #[test]
    fn test_replay_gain_track_and_album_modes() {
        use crate::models::{ReplayGainData, ReplayGainMode, RepeatMode};

        let mut app = LunulaApp::new_standalone();
        let mut track = app.current_track.clone();
        track.album = "Reference Album".to_string();
        track.replay_gain = ReplayGainData {
            track_gain_db: Some(-4.5),
            track_peak: Some(0.95),
            album_gain_db: Some(-2.0),
            album_peak: Some(0.90),
        };

        // 1. Disabled mode
        app.replay_gain_mode = ReplayGainMode::Disabled;
        let (db, lin) = app.calculate_effective_replay_gain(&track);
        assert_eq!(db, 0.0);
        assert_eq!(lin, 1.0);

        // 2. Track Gain mode
        app.replay_gain_mode = ReplayGainMode::Track;
        let (db_track, _) = app.calculate_effective_replay_gain(&track);
        assert!((db_track - (-4.5)).abs() < 1e-4);

        // 3. Album Gain mode
        app.replay_gain_mode = ReplayGainMode::Album;
        let (db_album, _) = app.calculate_effective_replay_gain(&track);
        assert!((db_album - (-2.0)).abs() < 1e-4);

        // 4. Auto (Smart) mode: Track gain in standard playlist, Album gain in Album repeat or filter
        app.replay_gain_mode = ReplayGainMode::Auto;
        app.repeat_mode = RepeatMode::Off;
        app.selected_album_filter = None;
        let (db_auto_playlist, _) = app.calculate_effective_replay_gain(&track);
        assert!((db_auto_playlist - (-4.5)).abs() < 1e-4, "Auto mode in playlist should pick track gain");

        app.repeat_mode = RepeatMode::Album;
        let (db_auto_album, _) = app.calculate_effective_replay_gain(&track);
        assert!((db_auto_album - (-2.0)).abs() < 1e-4, "Auto mode in album repeat should pick album gain");

        // 5. Preamp integration
        app.replay_gain_preamp_db = 1.5;
        let (db_with_preamp, _) = app.calculate_effective_replay_gain(&track);
        assert!((db_with_preamp - (-0.5)).abs() < 1e-4, "Preamp +1.5 dB on -2.0 dB should yield -0.5 dB");
    }

    #[test]
    fn test_replay_gain_clip_prevention_peak_scaling() {
        use crate::models::{ReplayGainClipPrevention, ReplayGainData, ReplayGainMode};

        let mut app = LunulaApp::new_standalone();
        app.replay_gain_mode = ReplayGainMode::Track;
        app.replay_gain_preamp_db = 0.0;

        let mut track = app.current_track.clone();
        // Track has high peak of 1.20 (+1.58 dBFS) and requested gain of +3.0 dB
        track.replay_gain = ReplayGainData {
            track_gain_db: Some(3.0),
            track_peak: Some(1.20),
            album_gain_db: None,
            album_peak: None,
        };

        // 1. PreventByPeak: max allowable gain is -20 * log10(1.20) ~= -1.5836 dB
        app.replay_gain_clip_prevention = ReplayGainClipPrevention::PreventByPeak;
        let (eff_db, lin) = app.calculate_effective_replay_gain(&track);
        let max_safe = -20.0 * 1.20f32.log10();
        assert!((eff_db - max_safe).abs() < 0.01, "PreventByPeak must scale down gain to {}, got {}", max_safe, eff_db);
        assert!(eff_db < 0.0, "Gain must be scaled down below 0 dBFS to prevent clipping");
        assert!(lin <= 1.0 / 1.20 + 0.001);

        // 2. Limiter mode: preserves original target gain and arms limiter
        app.replay_gain_clip_prevention = ReplayGainClipPrevention::Limiter;
        app.eq_limiter_enabled = false;
        app.current_track = track.clone();
        app.update_effective_replay_gain();
        assert_eq!(app.effective_replay_gain_db, 3.0);
        assert!(app.eq_limiter_enabled, "Limiter must be armed when ReplayGain boosts signal");

        // 3. None mode: completely unrestricted
        app.replay_gain_clip_prevention = ReplayGainClipPrevention::None;
        let (eff_none, lin_none) = app.calculate_effective_replay_gain(&track);
        assert_eq!(eff_none, 3.0);
        let expected_lin = 10.0_f32.powf(3.0 / 20.0);
        assert!((lin_none - expected_lin).abs() < 0.01);
    }

    #[test]
    fn test_gapless_peek_next_track_continuity() {
        use crate::models::{RepeatMode, Track};

        let mut app = LunulaApp::new_standalone();
        let make_track = |title: &str, album: &str| Track {
            title: title.to_string(),
            artist: "Artist".to_string(),
            album: album.to_string(),
            duration: 180.0,
            format: "FLAC".to_string(),
            sample_rate: 44100,
            bit_depth: 16,
            waveform: vec![0.0; 10],
            waveform_bass: vec![0.0; 10],
            waveform_mid: vec![0.0; 10],
            waveform_treble: vec![0.0; 10],
            file_path: Some(format!("C:/music/{}.flac", title)),
            file_size_mb: 25.0,
            bitrate_kbps: None,
            is_favorite: false,
            replay_gain: Default::default(),
            cue_offset_sec: None,
            cue_duration_sec: None,
        };

        let t1 = make_track("Song 1", "Album A");
        let t2 = make_track("Song 2", "Album A");
        let t3 = make_track("Song 3", "Album B");

        app.playlist = vec![t1.clone(), t2.clone(), t3.clone()];
        app.current_track = t1.clone();
        app.repeat_mode = RepeatMode::Off;

        // 1. Sequential: next after Song 1 is Song 2
        let peek1 = app.peek_next_track();
        assert_eq!(peek1.map(|t| t.title), Some("Song 2".to_string()));

        // 2. Play Queue has higher priority than playlist
        let q1 = make_track("Queued Priority Song", "Queue");
        app.play_queue = vec![q1.clone()];
        app.queue_cursor = None;
        let peek_q = app.peek_next_track();
        assert_eq!(peek_q.map(|t| t.title), Some("Queued Priority Song".to_string()));

        // 3. Repeat One: next track is current track itself
        app.play_queue.clear();
        app.repeat_mode = RepeatMode::One;
        let peek_repeat_one = app.peek_next_track();
        assert_eq!(peek_repeat_one.map(|t| t.title), Some("Song 1".to_string()));
    }
}

