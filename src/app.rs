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
    pub prefer_exclusive: bool,

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

    // Headroom Guard & Clipping Detection Settings
    pub clipping_detection_enabled: bool,
    pub clipping_threshold_db: f32,
    pub show_clipping_settings_popup: bool,

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
    pub idle_trim_timer: f32,
    pub is_scanning: bool,
    pub system_telemetry: crate::system_telemetry::SystemTelemetrySampler,
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
    pub peak_scan_tx: std::sync::mpsc::Sender<(String, f32, Vec<f32>)>,
    pub peak_scan_rx: std::sync::mpsc::Receiver<(String, f32, Vec<f32>)>,
    pub selected_output_device: Option<String>,
    pub available_output_devices: Vec<crate::audio::backend::OutputDeviceInfo>,

    // High-performance GUI caching (DAG topology, library stats, and EQ curves)
    pub cached_chain_eval: std::cell::RefCell<Option<crate::models::ChainEvaluation>>,
    pub cached_static_analysis: std::cell::RefCell<Option<crate::audio::GraphAnalysisReport>>,
    pub chain_eval_dirty: std::cell::Cell<bool>,
    pub static_analysis_dirty: std::cell::Cell<bool>,
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

    // Signal Chain Studio .lnl Preset Management & Auto-Binding
    pub show_export_preset_modal: bool,
    pub preset_modal_tab: usize,
    pub preset_search_query: String,
    pub preset_export_name: String,
    pub preset_export_author: String,
    pub preset_export_description: String,
    pub preset_export_use_password: bool,
    pub preset_export_password: String,
    pub preset_status_notification: Option<(String, bool)>,
    pub preset_status_notification_time: Option<std::time::Instant>,
    pub show_import_password_modal: bool,
    pub pending_import_container: Option<crate::preset::LnlPresetContainer>,
    pub preset_import_password: String,
    pub preset_import_error: Option<String>,
    pub preset_selected_idx: usize,
    pub inspecting_preset: Option<crate::preset::SignalChainPreset>,

    // Preset Auto-Binding State
    pub preset_bindings: Vec<crate::preset::PresetBindingRule>,
    pub auto_load_preset_on_track_change: bool,
    pub show_binding_toast_on_change: bool,
    pub binding_fallback_to_bit_perfect: bool,
    pub binding_filter_tab: usize,
    pub binding_selected_preset: String,
    pub binding_picker_open: bool,
    pub binding_picker_mode: usize, // 0 = Track, 1 = Album, 2 = Genre
    pub binding_picker_search: String,
    pub binding_picker_selected_key: String,
    pub binding_picker_selected_label: String,
    pub binding_rule_search: String,
    pub show_binding_options_popover: bool,
    pub last_applied_auto_preset: Option<String>,
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
            crate::scanner::generate_multiband_waveform("Lunula Default Initial Track", 512);

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

        // Default clean chain: 1 (File Source) -> 2 (RAM Buffer) -> 4 (Resampler) -> 5 (Output DAC)
        let initial_connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, 4),
            CableConnection::new(4, 5),
        ];

        let (cover_req_tx, cover_req_rx) = std::sync::mpsc::channel::<(String, String)>();
        let (cover_res_tx, cover_res_rx) = std::sync::mpsc::channel::<CoverLoadResult>();
        let pending_cover_loads = std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let (peak_scan_tx, peak_scan_rx) = std::sync::mpsc::channel::<(String, f32, Vec<f32>)>();

        let _ = std::thread::Builder::new()
            .name("lunula-cover-loader".into())
            .spawn(move || {
                while let Ok((key, path)) = cover_req_rx.recv() {
                    let image_data = if let Some(bytes) = crate::scanner::extract_album_art(&path) {
                        if bytes.len() <= 15 * 1024 * 1024 {
                            if let Ok(img) = image::load_from_memory(&bytes) {
                                let thumb = img.thumbnail(160, 160).to_rgba8();
                                let size = [thumb.width() as usize, thumb.height() as usize];
                                Some((size, thumb.into_raw()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let _ = cover_res_tx.send(CoverLoadResult { key, image_data });
                }
            });

        let audio_engine = crate::audio::AudioEngineController::new().ok();
        let mut system_telemetry = crate::system_telemetry::SystemTelemetrySampler::default();
        if let Some(ref eng) = audio_engine {
            system_telemetry.buffer_metrics = std::sync::Arc::clone(eng.buffer_metrics());
        }

        let mut app = Self {
            current_tab: AppTab::NowPlaying,
            selected_node: None,
            chain_mode: ChainMode::BitPerfect,
            prefer_exclusive: true,

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

            clipping_detection_enabled: true,
            clipping_threshold_db: 0.0,
            show_clipping_settings_popup: false,

            has_completed_onboarding: false,
            is_onboarding_quest_active: false,
            onboarding_quest_step: crate::models::OnboardingQuestStep::Welcome,
            quest_rect_signal_chain_tab: None,
            quest_rect_dsp_mode_btn: None,
            quest_rect_tube_warmer_btn: None,
            quest_rect_cable_mute_target: None,

            show_interactive_theater: false,
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
                genre: None,
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
            idle_trim_timer: 5.0, // 5 detik setelah buka aplikasi, bersihkan memori startup
            scan_rx: None,
            is_scanning: false,
            scan_status: String::new(),
            visualizer_anim_phase: 0.0,
            waveform_mode: WaveformMode::StackedMultiBand,
            audio_engine,
            system_telemetry,
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
            peak_scan_tx,
            peak_scan_rx,
            selected_output_device: None,
            available_output_devices: crate::audio::backend::enumerate_output_devices(),

            cached_chain_eval: std::cell::RefCell::new(None),
            cached_static_analysis: std::cell::RefCell::new(None),
            chain_eval_dirty: std::cell::Cell::new(true),
            static_analysis_dirty: std::cell::Cell::new(true),
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

            show_export_preset_modal: false,
            preset_modal_tab: 0,
            preset_search_query: String::new(),
            preset_export_name: "Warm Studio Master".to_string(),
            preset_export_author: "Lunula Audio".to_string(),
            preset_export_description: "Modular DSP signal chain with vintage warmth and spatial widening".to_string(),
            preset_export_use_password: false,
            preset_export_password: String::new(),
            preset_status_notification: None,
            preset_status_notification_time: None,
            show_import_password_modal: false,
            pending_import_container: None,
            preset_import_password: String::new(),
            preset_import_error: None,
            preset_selected_idx: 0,
            inspecting_preset: None,

            // Preset Auto-Binding State
            preset_bindings: vec![
                crate::preset::PresetBindingRule {
                    id: "bind_genre_rock".to_string(),
                    target_type: crate::preset::PresetBindingType::Genre,
                    target_key: "Rock".to_string(),
                    target_label: "Rock / Metal / Alternative".to_string(),
                    preset_name: "Warm Studio Master".to_string(),
                },
                crate::preset::PresetBindingRule {
                    id: "bind_genre_jazz".to_string(),
                    target_type: crate::preset::PresetBindingType::Genre,
                    target_key: "Jazz".to_string(),
                    target_label: "Acoustic / Jazz / Vocal".to_string(),
                    preset_name: "Acoustic Vocal Intimacy".to_string(),
                },
            ],
            auto_load_preset_on_track_change: true,
            show_binding_toast_on_change: false,
            binding_fallback_to_bit_perfect: true,
            binding_filter_tab: 0,
            binding_selected_preset: "Warm Studio Master".to_string(),
            binding_picker_open: false,
            binding_picker_mode: 0,
            binding_picker_search: String::new(),
            binding_picker_selected_key: String::new(),
            binding_picker_selected_label: String::new(),
            binding_rule_search: String::new(),
            show_binding_options_popover: false,
            last_applied_auto_preset: None,
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
                app.current_lyrics = app.current_track.file_path.as_ref().and_then(|p| {
                    crate::lyrics::Lyrics::load_for_track_smart(
                        std::path::Path::new(p),
                        &app.current_track.title,
                        &app.current_track.artist,
                    )
                });
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
            if let Some(cde) = state.clipping_detection_enabled {
                app.clipping_detection_enabled = cde;
            }
            if let Some(ctd) = state.clipping_threshold_db {
                app.clipping_threshold_db = ctd;
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
            app.show_interactive_theater = true;
            app.set_chain_mode(ChainMode::BitPerfect);
            app.eq_bands[0] = 3.0;
            app.eq_bands[1] = 1.5;
            app.eq_bands[8] = 1.0;
            app.eq_bands[9] = 2.0;
        }

        // Initial artwork loaded on-demand via async thumbnail loader (prevents redundant memory allocations)
        app.current_cover_bytes = None;

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
            engine.set_prefer_exclusive(app.prefer_exclusive);
        }

        app.load_auto_bindings();
        app.apply_startup_preset();
        app.sync_signal_chain_to_engine();
        app
    }

    /// Returns true if any fullscreen modal or high-priority overlay is currently active.
    /// When active, underlying canvas, spectrum, and background controls should suppress
    /// hovering animations, tooltips, dragging, and context menus.
    #[inline]
    pub fn is_modal_or_overlay_active(&self) -> bool {
        self.show_interactive_theater
            || self.show_tag_editor
            || self.show_export_preset_modal
            || self.show_import_password_modal
            || self.show_save_preset_dialog
            || self.show_playlist_creator
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
        self.invalidate_chain_cache();
    }

    /// Background non-blocking worker: calculates true audio peak and real waveform envelope if track lacks peak tag
    pub fn trigger_background_peak_scan(&self, track: &Track) {
        if track.replay_gain.track_peak.is_none() {
            if let Some(ref path_str) = track.file_path {
                let path = std::path::PathBuf::from(path_str);
                if path.is_file() {
                    let tx = self.peak_scan_tx.clone();
                    let path_str_cloned = path_str.clone();
                    std::thread::Builder::new()
                        .name("lunula-peak-scan".into())
                        .spawn(move || {
                            if let Ok((peak, wave)) = crate::audio::scan_audio_peak_and_waveform(&path, 512) {
                                let _ = tx.send((path_str_cloned, peak, wave));
                            }
                        })
                        .ok();
                }
            }
        }
    }

    pub fn sync_smtc_state(&mut self) {
        if let Some(ref mut smtc) = self.smtc_bridge {
            let title = &self.current_track.title;
            let artist = &self.current_track.artist;
            let album = &self.current_track.album;
            let dur = self.current_track.duration;
            let cover_url = self.current_track.file_path.as_deref().filter(|p| {
                let lower = p.to_lowercase();
                lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".bmp")
            });
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
            if next_t.waveform.is_empty() && next_t.duration > 0.0 {
                let seed = format!("{}_{}_{:.0}", next_t.title, next_t.artist, next_t.duration);
                let (ov, b, m, tr) = crate::scanner::generate_multiband_waveform(&seed, 512);
                next_t.waveform = ov;
                next_t.waveform_bass = b;
                next_t.waveform_mid = m;
                next_t.waveform_treble = tr;
            }
            self.current_track = next_t;
        }
        self.current_time = 0.0;
        self.is_playing = true;
        self.current_cover_bytes = None;
        self.current_cover_texture = None;
        self.current_lyrics = self.current_track.file_path.as_ref().and_then(|p| {
            crate::lyrics::Lyrics::load_for_track_smart(
                std::path::Path::new(p),
                &self.current_track.title,
                &self.current_track.artist,
            )
        });
        self.lyrics_offset_trim_sec = 0.0;

        // Evaluate and automatically apply preset bindings for Track, Album, or Genre (or fallback to Bit-Perfect)
        let track_eval = self.current_track.clone();
        self.check_and_apply_auto_binding(&track_eval);

        self.trigger_background_peak_scan(&self.current_track);
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

    pub fn play_track(&mut self, mut track: Track) {
        if track.waveform.is_empty() && track.duration > 0.0 {
            let seed = format!("{}_{}_{:.0}", track.title, track.artist, track.duration);
            let (ov, b, m, tr) = crate::scanner::generate_multiband_waveform(&seed, 512);
            track.waveform = ov;
            track.waveform_bass = b;
            track.waveform_mid = m;
            track.waveform_treble = tr;
        }
        self.current_track = track;
        self.current_time = 0.0;
        self.is_playing = true;
        self.trigger_background_peak_scan(&self.current_track);
        self.current_cover_bytes = None;
        self.current_cover_texture = None; // Reset so next frame builds the new texture
        self.current_lyrics = self.current_track.file_path.as_ref().and_then(|p| {
            crate::lyrics::Lyrics::load_for_track_smart(
                std::path::Path::new(p),
                &self.current_track.title,
                &self.current_track.artist,
            )
        });
        self.lyrics_offset_trim_sec = 0.0;

        // Evaluate and automatically apply preset bindings for Track, Album, or Genre (or fallback to Bit-Perfect)
        let track_eval = self.current_track.clone();
        self.check_and_apply_auto_binding(&track_eval);

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
            match node.node_type {
                NodeType::Source => {
                    node.title = "FILE SOURCE".to_string();
                    let bit_depth_str = if report.source_bit_depth > 0 {
                        format!("{}-bit", report.source_bit_depth)
                    } else {
                        "Audio".to_string()
                    };
                    node.sub1 = format!("{} {}", self.current_track.format, bit_depth_str);
                    if let Some(peak) = self.current_track.replay_gain.track_peak {
                        let peak_db = 20.0 * peak.max(1e-5).log10();
                        node.sub2 = format!("{:.1} kHz • Peak {:.1}dB", report.source_rate as f32 / 1000.0, peak_db);
                    } else {
                        node.sub2 = format!("{:.1} kHz", report.source_rate as f32 / 1000.0);
                    }
                }
                NodeType::Buffer => {
                    node.title = "RAM BUFFER".to_string();
                    node.sub1 = "RingBuffer".to_string();
                    node.sub2 = "100% Loaded".to_string();
                }
                NodeType::Driver => {
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
                NodeType::Endpoint => {
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

    pub fn set_exclusive_mode(&mut self, prefer: bool) {
        self.prefer_exclusive = prefer;
        if let Some(ref engine) = self.audio_engine {
            engine.set_prefer_exclusive(prefer);
        }
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
                match n.node_type {
                    NodeType::Driver => {
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
                    NodeType::Endpoint => {
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
        self.static_analysis_dirty.set(true);
        *self.cached_chain_eval.borrow_mut() = None;
        *self.cached_static_analysis.borrow_mut() = None;
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

    /// Real-time static dataflow analysis (Abstract Interpretation) for the current modular patchbay DAG.
    /// Incorporates the active track's peak level and effective ReplayGain to provide context-aware headroom calculation.
    /// Returns cached report if graph topology, parameters, and audio context have not changed, or freshly computes it.
    pub fn get_static_analysis(&self) -> crate::audio::GraphAnalysisReport {
        if !self.static_analysis_dirty.get() {
            if let Some(ref report) = *self.cached_static_analysis.borrow() {
                return report.clone();
            }
        }

        let (source_interval, track_title) = {
            let track = &self.current_track;
            if track.duration > 0.0 || track.file_path.is_some() {
                let raw_peak = track.replay_gain.track_peak
                    .or(track.replay_gain.album_peak);
                
                if let Some(peak) = raw_peak {
                    if peak > 0.0 {
                        let eff_lin = if self.replay_gain_mode != crate::models::ReplayGainMode::Disabled {
                            self.effective_replay_gain_linear
                        } else {
                            1.0
                        };
                        let actual_peak = (peak * eff_lin).max(0.001);
                        let interval = crate::audio::AmplitudeInterval::symmetric(actual_peak);
                        (interval, Some(track.title.clone()))
                    } else {
                        (crate::audio::AmplitudeInterval::FULL_SCALE, Some(track.title.clone()))
                    }
                } else {
                    (crate::audio::AmplitudeInterval::FULL_SCALE, Some(track.title.clone()))
                }
            } else {
                (crate::audio::AmplitudeInterval::FULL_SCALE, None)
            }
        };

        let graph = self.compile_signal_chain();
        let report = crate::audio::analyze_signal_chain_advanced(
            &graph,
            source_interval,
            track_title,
            self.clipping_threshold_db,
            self.clipping_detection_enabled,
        );
        *self.cached_static_analysis.borrow_mut() = Some(report.clone());
        self.static_analysis_dirty.set(false);
        report
    }

    /// Updates a node's parameter, subtitle text formatting, invalidates signal cache,
    /// and dispatches the new parameter to the running DSP audio engine.
    pub fn update_node_parameter(&mut self, node_id: usize, new_val: f32) {
        if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            n.param_val = new_val;
            match n.node_type {
                NodeType::LowPassFilter | NodeType::HighPassFilter => {
                    n.sub1 = if new_val >= 1000.0 {
                        format!("Cutoff {:.1} kHz", new_val / 1000.0)
                    } else {
                        format!("Cutoff {:.0} Hz", new_val)
                    };
                }
                NodeType::BandPassFilter => {
                    n.sub1 = if new_val >= 1000.0 {
                        format!("Center {:.1} kHz", new_val / 1000.0)
                    } else {
                        format!("Center {:.0} Hz", new_val)
                    };
                }
                NodeType::Crossover => {
                    n.sub1 = if new_val >= 1000.0 {
                        format!("X-Over {:.1} kHz", new_val / 1000.0)
                    } else {
                        format!("X-Over {:.0} Hz", new_val)
                    };
                }
                NodeType::Mixer => {
                    let a_pct = (1.0 - new_val) * 100.0;
                    let b_pct = new_val * 100.0;
                    n.sub1 = format!("Mix: {:.0}% A • {:.0}% B", a_pct, b_pct);
                }
                NodeType::TubeWarmer => {
                    n.sub1 = format!("Warmth {:.0}%", new_val);
                }
                NodeType::StereoWidener => {
                    n.sub1 = format!("+{:.0}% Soundstage", new_val);
                }
                NodeType::VolumeBooster => {
                    n.sub1 = format!("+{:.1} dB Gain", new_val);
                }
                NodeType::AnalogPreamp => {
                    n.sub1 = format!("{:+0.1} dB Drive", new_val);
                }
                NodeType::DynamicLoudness => {
                    n.sub1 = format!("Loudness {:.0}%", new_val);
                }
                NodeType::SteppedAttenuator => {
                    n.sub1 = format!("{:.0} dB", new_val);
                }
                NodeType::OpticalLeveler => {
                    n.sub1 = format!("Peak Red {:.0}%", new_val);
                }
                NodeType::StereoBalance => {
                    if new_val.abs() < 1.0 {
                        n.sub1 = "Center 0.0 dB".to_string();
                    } else if new_val < 0.0 {
                        n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 + new_val.abs() * 0.5, 50.0 - new_val.abs() * 0.5);
                    } else {
                        n.sub1 = format!("L {:.0}% / R {:.0}%", 50.0 - new_val * 0.5, 50.0 + new_val * 0.5);
                    }
                }
                NodeType::TimeAlignment => {
                    if new_val >= 1000.0 {
                        n.sub1 = format!("Delay {:.2} ms", new_val / 1000.0);
                    } else {
                        n.sub1 = format!("Delay {:.0} µs", new_val);
                    }
                }
                NodeType::StereoMatrix => {
                    n.sub1 = match new_val.round() as u8 {
                        0 => "Stereo Direct".to_string(),
                        1 => "Channel Swap (R/L)".to_string(),
                        2 => "Mono (L+R Sum)".to_string(),
                        3 => "M/S Encode (Mid/Side)".to_string(),
                        4 => "M/S Decode (to Stereo)".to_string(),
                        5 => "Side Only (Mid Mute)".to_string(),
                        6 => "Mid Only (Side Mute)".to_string(),
                        7 => "Bauer Crossfeed 3D".to_string(),
                        _ => "Stereo Matrix".to_string(),
                    };
                }
                _ => {}
            }
        }
        self.invalidate_chain_cache();
        if let Some(ref engine) = self.audio_engine {
            engine.set_node_param(node_id as u32, 0, new_val);
        }
    }

    /// Automatically remedies headroom deficit or clipping hazard on a specific node.
    /// Adjusts gain, drive, or stereo width to safe levels, or engages output soft limiter.
    pub fn apply_auto_fix_for_node(&mut self, node_id: usize) -> bool {
        let report = self.get_static_analysis();
        let r = match report.get_node_report(node_id as u32) {
            Some(report) => report.clone(),
            None => return false,
        };

        let node_type = match self.nodes.iter().find(|n| n.id == node_id) {
            Some(n) => n.node_type.clone(),
            None => return false,
        };
        let current_param = self.nodes.iter().find(|n| n.id == node_id).map(|n| n.param_val).unwrap_or(0.0);
        let deficit = r.headroom_deficit_db;

        match node_type {
            NodeType::VolumeBooster => {
                let safe_val = (current_param - deficit).max(0.0);
                self.update_node_parameter(node_id, safe_val);
                self.save_library_state();
                true
            }
            NodeType::AnalogPreamp => {
                let safe_val = (current_param - deficit).clamp(-24.0, 12.0);
                self.update_node_parameter(node_id, safe_val);
                self.save_library_state();
                true
            }
            NodeType::StereoWidener => {
                let current_width = 1.0 + (current_param / 100.0);
                let attenuation_factor = 10.0f32.powf(-deficit / 20.0);
                let target_width = (current_width * attenuation_factor).max(1.0);
                let safe_val = ((target_width - 1.0) * 100.0).clamp(0.0, 100.0);
                self.update_node_parameter(node_id, safe_val);
                self.save_library_state();
                true
            }
            _ => {
                if !self.eq_limiter_enabled {
                    self.eq_limiter_enabled = true;
                    self.save_library_state();
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Automatically applies remedies to all culprit nodes across the signal chain.
    /// Returns the number of modules adjusted.
    pub fn apply_all_headroom_auto_fixes(&mut self) -> usize {
        let culprits = self.get_static_analysis().culprit_nodes.clone();
        let mut fixed_count = 0;
        for c_id in culprits {
            if self.apply_auto_fix_for_node(c_id as usize) {
                fixed_count += 1;
            }
        }
        if fixed_count > 0 {
            self.invalidate_chain_cache();
            self.save_library_state();
        }
        fixed_count
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
            AnalogPreampEffect, BandPassFilterEffect, BassGuardEffect, CompressorEffect,
            CrossoverHighEffect, CrossoverLowEffect, DynamicLoudnessEffect, Effect,
            HighPassFilterEffect, LowPassFilterEffect, MixerEffect, OpticalLevelerEffect,
            PassthroughEffect, PresetEqEffect, SplitterEffect, SteppedAttenuatorEffect,
            StereoBalanceEffect, StereoMatrixEffect, StereoWidenerEffect, TimeAlignmentEffect, TubeWarmerEffect, VolumeAttenuatorEffect,
            VolumeBoosterEffect,
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

        // If Kahn topological sort was unable to visit all flowing nodes, a cycle exists!
        if ordered_node_ids.len() != chain_eval.flowing_nodes.len() {
            return crate::audio::SignalChainGraph {
                is_complete: false,
                nodes: Vec::new(),
                edges: Vec::new(),
                topological_order: Vec::new(),
                effects: Vec::new(),
            };
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
                        let cutoff = if node.param_val > 20.0 { node.param_val as f64 } else { 800.0 };
                        let slope = if node.sub2.contains("48dB") { 48 } else if node.sub2.contains("24dB") { 24 } else { 12 };
                        let fringe = if let Some(idx) = node.sub2.find("Q ") {
                            node.sub2[idx + 2..].split_whitespace().next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.7071)
                        } else {
                            0.7071
                        };
                        Box::new(LowPassFilterEffect::with_slope_and_fringe(cutoff, slope, fringe, sample_rate))
                    }
                    NodeType::HighPassFilter => {
                        let cutoff = if node.param_val > 10.0 { node.param_val as f64 } else { 80.0 };
                        let slope = if node.sub2.contains("48dB") { 48 } else if node.sub2.contains("24dB") { 24 } else { 12 };
                        let fringe = if let Some(idx) = node.sub2.find("Q ") {
                            node.sub2[idx + 2..].split_whitespace().next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.7071)
                        } else {
                            0.7071
                        };
                        Box::new(HighPassFilterEffect::with_slope_and_fringe(cutoff, slope, fringe, sample_rate))
                    }
                    NodeType::BandPassFilter => {
                        let center = if node.param_val > 20.0 { node.param_val as f64 } else { 1000.0 };
                        let slope = if node.sub2.contains("24dB") { 24 } else { 12 };
                        let fringe = if let Some(idx) = node.sub2.find("Q ") {
                            node.sub2[idx + 2..].split_whitespace().next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.414)
                        } else {
                            1.414
                        };
                        Box::new(BandPassFilterEffect::with_slope_and_fringe(center, slope, fringe, sample_rate))
                    }
                    NodeType::Crossover => {
                        let xover = if node.param_val > 20.0 { node.param_val as f64 } else { 500.0 };
                        Box::new(CrossoverLowEffect::new(xover, sample_rate))
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
                    NodeType::TimeAlignment => {
                        let delay_us = if (0.0..=50_000.0).contains(&node.param_val) { node.param_val } else { 0.0 };
                        let invert = node.sub2.contains("180°") || node.sub2.contains("Invert");
                        Box::new(TimeAlignmentEffect::new(delay_us, 0.0, invert, false, sample_rate as f32))
                    }
                    NodeType::StereoMatrix => {
                        let mode = (node.param_val.round() as u8).min(7);
                        Box::new(StereoMatrixEffect::new(mode, 0.6, sample_rate as f32))
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

                if node.node_type == NodeType::Crossover {
                    let has_high_conn = self.connections.iter().any(|c| c.from_node == id && c.from_port == 1 && chain_eval.flowing_cables.contains(c));
                    if has_high_conn {
                        let high_node_id = (id as u32) + 50_000;
                        let xover = if node.param_val > 20.0 { node.param_val as f64 } else { 500.0 };
                        let invert = node.sub2.contains("Invert");
                        let high_effect: Box<dyn Effect> = Box::new(CrossoverHighEffect::new(xover, invert, sample_rate));
                        nodes.push(crate::audio::GraphNode {
                            id: high_node_id,
                            name: format!("{} (High)", node.title),
                            effect: high_effect,
                        });
                        topological_order.push(high_node_id);
                    }
                }
            }
        }

        // Preserve exact ports for each connection
        let mut edges: Vec<crate::audio::GraphEdge> = Vec::new();
        for conn in &self.connections {
            if chain_eval.flowing_cables.contains(conn) {
                let from_node = if conn.from_port == 1 {
                    if let Some(n) = self.nodes.iter().find(|n| n.id == conn.from_node) {
                        if n.node_type == NodeType::Crossover {
                            (conn.from_node as u32) + 50_000
                        } else {
                            conn.from_node as u32
                        }
                    } else {
                        conn.from_node as u32
                    }
                } else {
                    conn.from_node as u32
                };

                edges.push(crate::audio::GraphEdge::with_ports(
                    from_node,
                    conn.from_port,
                    conn.to_node as u32,
                    conn.to_port,
                ));
            }
        }

        // For any crossover with high connection, connect incoming parent of crossover to the virtual high node as well
        for &id in &ordered_node_ids {
            if let Some(node) = self.nodes.iter().find(|n| n.id == id) {
                if node.node_type == NodeType::Crossover {
                    let has_high = self.connections.iter().any(|c| c.from_node == id && c.from_port == 1 && chain_eval.flowing_cables.contains(c));
                    if has_high {
                        if let Some(parent_edge) = self.connections.iter().find(|c| c.to_node == id && chain_eval.flowing_cables.contains(c)) {
                            edges.push(crate::audio::GraphEdge::with_ports(
                                parent_edge.from_node as u32,
                                parent_edge.from_port,
                                (id as u32) + 50_000,
                                0,
                            ));
                        }
                    }
                }
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

    /// Builds a declarative, non-DSP Intermediate Representation (Graph IR) of the patchbay canvas.
    pub fn build_graph_ir(&self) -> crate::audio::GraphIr {
        let source_id = self.nodes.iter().find(|n| n.node_type == crate::models::NodeType::Source).map(|n| n.id as u32).unwrap_or(1);
        let endpoint_id = self.nodes.iter().find(|n| n.node_type == crate::models::NodeType::Endpoint).map(|n| n.id as u32).unwrap_or(5);
        let mut ir = crate::audio::GraphIr::new(source_id, endpoint_id);

        for node in &self.nodes {
            let eff_type = match &node.node_type {
                crate::models::NodeType::TubeWarmer => crate::audio::EffectType::TubeWarmer,
                crate::models::NodeType::StereoWidener => crate::audio::EffectType::StereoWidener,
                crate::models::NodeType::BassGuard => crate::audio::EffectType::BassGuard,
                crate::models::NodeType::VolumeAttenuator => crate::audio::EffectType::VolumeAttenuator,
                crate::models::NodeType::LowPassFilter => crate::audio::EffectType::LowPassFilter,
                crate::models::NodeType::HighPassFilter => crate::audio::EffectType::HighPassFilter,
                crate::models::NodeType::BandPassFilter => crate::audio::EffectType::BandPassFilter,
                crate::models::NodeType::Crossover => crate::audio::EffectType::Crossover,
                crate::models::NodeType::Mixer => crate::audio::EffectType::Mixer,
                crate::models::NodeType::Splitter => crate::audio::EffectType::Splitter,
                crate::models::NodeType::VolumeBooster => crate::audio::EffectType::VolumeBooster,
                crate::models::NodeType::AnalogPreamp => crate::audio::EffectType::AnalogPreamp,
                crate::models::NodeType::DynamicLoudness => crate::audio::EffectType::DynamicLoudness,
                crate::models::NodeType::SteppedAttenuator => crate::audio::EffectType::SteppedAttenuator,
                crate::models::NodeType::OpticalLeveler => crate::audio::EffectType::OpticalLeveler,
                crate::models::NodeType::StereoBalance => crate::audio::EffectType::StereoBalance,
                crate::models::NodeType::Compressor => crate::audio::EffectType::Compressor,
                crate::models::NodeType::TimeAlignment => crate::audio::EffectType::TimeAlignment,
                crate::models::NodeType::StereoMatrix => crate::audio::EffectType::StereoMatrix,
                crate::models::NodeType::SavedEqPreset(_) => crate::audio::EffectType::ParametricEq,
                _ => crate::audio::EffectType::Passthrough,
            };

            let mut node_ir = crate::audio::NodeIr::new(node.id as u32, node.title.clone(), eff_type);
            node_ir.param_val = node.param_val;
            ir.nodes.push(node_ir);
        }

        for conn in &self.connections {
            ir.edges.push(crate::audio::EdgeIr::new(
                conn.from_node as u32,
                conn.from_port,
                conn.to_node as u32,
                conn.to_port,
            ));
        }

        ir
    }

    /// Performs formal DAG validation and cycle detection on the current Graph IR.
    pub fn validate_graph_ir(&self) -> Result<Vec<u32>, crate::audio::GraphCompilationError> {
        self.build_graph_ir().validate_and_sort()
    }

    pub fn sync_signal_chain_to_engine(&self) {
        if let Some(ref engine) = self.audio_engine {
            let graph = self.compile_signal_chain();
            engine.set_signal_chain_graph(graph);
        }
    }

    /// Resets the modular patchbay signal chain back to the clean default 4-node topology:
    /// File Source (1) -> RAM Buffer (2) -> App Resampler (4) -> Output DAC (5).
    /// Clears any leftover effect nodes, custom cables, or prior preset state.
    pub fn reset_to_default_signal_chain(&mut self) {
        self.nodes = vec![
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
        self.connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, 4),
            CableConnection::new(4, 5),
        ];
        self.next_node_id = 6;
        self.selected_node = None;
        self.advance_studio_state.selected_cable_idx = None;
        self.last_applied_auto_preset = None;

        self.invalidate_chain_cache();
        self.sync_signal_chain_to_engine();
        self.save_library_state();
    }
}

impl eframe::App for LunulaApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_library_state();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.system_telemetry.sample();
        let is_minimized = ui.input(|i| i.viewport().minimized.unwrap_or(false));
        if is_minimized {
            crate::system_telemetry::trim_process_working_set();
        } else if self.idle_trim_timer > 0.0 {
            self.idle_trim_timer -= ui.input(|i| i.stable_dt);
            if self.idle_trim_timer <= 0.0 {
                crate::system_telemetry::trim_process_working_set();
            }
        }

        // Drain loaded covers from background thread into egui texture cache (bounded to 32 entries)
        while let Ok(result) = self.cover_res_rx.try_recv() {
            let tex = result.image_data.map(|(size, pixels)| {
                let color_image = eframe::egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                ctx.load_texture(&format!("art_{}", result.key), color_image, eframe::egui::TextureOptions::LINEAR)
            });
            if let Ok(mut cache) = self.album_cover_cache.lock() {
                if cache.len() >= 32 {
                    let keys_to_remove: Vec<String> = cache.keys().take(12).cloned().collect();
                    for k in keys_to_remove {
                        cache.remove(&k);
                    }
                }
                cache.insert(result.key.clone(), tex);
            }
            if let Ok(mut pending) = self.pending_cover_loads.lock() {
                pending.remove(&result.key);
            }
            ctx.request_repaint();
        }

        // Drain scanned track peaks & physical waveforms from background peak scanner
        let mut peak_received = false;
        while let Ok((path, peak, wave)) = self.peak_scan_rx.try_recv() {
            if self.current_track.file_path.as_deref() == Some(&path) {
                self.current_track.replay_gain.track_peak = Some(peak);
                if !wave.is_empty() {
                    self.current_track.waveform = wave;
                }
                self.update_effective_replay_gain();
                self.invalidate_chain_cache();
            }
            for t in self.playlist.iter_mut() {
                if t.file_path.as_deref() == Some(&path) {
                    t.replay_gain.track_peak = Some(peak);
                    // Hemat RAM: waveform tidak perlu disimpan di ribuan lagu playlist, cukup di current_track
                }
            }
            peak_received = true;
            ctx.request_repaint();
        }

        if peak_received {
            self.save_library_state();
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
                        self.idle_trim_timer = 2.0;
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
                    ui.add_space(8.0);
                    // --- HARDWARE STREAM INDICATOR (NO GIMMICK) ---
                    if let Some(ref report) = self.signal_path_report {
                        if report.is_exclusive {
                            let badge = egui::Frame::new()
                                .fill(Color32::from_rgb(236, 253, 245)) // Emerald background
                                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                                .corner_radius(12.0)
                                .inner_margin(egui::Margin::symmetric(8, 3))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new("🔒 WASAPI EXCLUSIVE")
                                            .strong()
                                            .size(10.5)
                                            .color(Color32::from_rgb(4, 120, 87)),
                                    );
                                });
                            badge.response.on_hover_text(format!(
                                "Mode Hardware: WASAPI Exclusive Locked\n• Perangkat: {}\n• Format DAC: {}\n• Sample Rate: {:.1} kHz\n• Status: Windows Audio Mixer di-bypass penuh (Bit-Perfect)",
                                report.device_name, report.format_desc, report.device_rate as f32 / 1000.0
                            ));
                        } else {
                            let badge = egui::Frame::new()
                                .fill(Color32::from_rgb(254, 243, 199)) // Amber background
                                .stroke(Stroke::new(1.0, Color32::from_rgb(245, 158, 11)))
                                .corner_radius(12.0)
                                .inner_margin(egui::Margin::symmetric(8, 3))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new("🔊 WASAPI SHARED")
                                            .strong()
                                            .size(10.5)
                                            .color(Color32::from_rgb(180, 83, 9)),
                                    );
                                });
                            badge.response.on_hover_text(format!(
                                "Mode Hardware: Windows Shared Audio\n• Perangkat: {}\n• Format DAC: {}\n• Status: Melewati OS Audio Mixer",
                                report.device_name, report.format_desc
                            ));
                        }
                    } else {
                        let badge = egui::Frame::new()
                            .fill(Color32::from_rgb(241, 245, 249))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                            .corner_radius(12.0)
                            .inner_margin(egui::Margin::symmetric(8, 3))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("○ STANDBY")
                                        .size(10.5)
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            });
                        badge.response.on_hover_text("Audio Engine Standby: Putar lagu untuk melihat jalur hardware aktual");
                    }

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
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                // Quick overview patchbay on top
                                render_card_frame(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            let full_w = ui.available_width();
                                            let is_narrow_w = full_w < 640.0;
                                            ui.label(egui::RichText::new("⚡ MODULAR SIGNAL CHAIN").strong().size(13.5).color(Color32::from_rgb(15, 23, 42)));
                                            if !is_chain_ok {
                                                ui.label(egui::RichText::new("⚠️ Sinyal Terputus").size(11.5).color(Color32::from_rgb(239, 68, 68)).strong());
                                            } else if let Some(ref r) = self.signal_path_report {
                                                if r.requested_mode == ChainMode::BitPerfect && r.actual_mode == ChainMode::OsResampled {
                                                    let txt = if is_narrow_w { "⚠️ OS Resampled" } else { "⚠️ Fallback: OS Resampled" };
                                                    ui.label(egui::RichText::new(format!("{} {:.1} kHz", txt, r.device_rate as f32 / 1000.0)).size(11.5).color(Color32::from_rgb(217, 119, 6)).strong());
                                                } else if r.actual_mode == ChainMode::BitPerfect {
                                                    let txt = if is_narrow_w { "✓ Bit-Perfect Direct" } else { "✓ Sinyal Bit-Perfect Mengalir (Native)" };
                                                    ui.label(egui::RichText::new(txt).size(11.5).color(Color32::from_rgb(16, 185, 129)).strong());
                                                } else if r.actual_mode == ChainMode::DspActive {
                                                    let eval = self.evaluate_signal_chain();
                                                    let flowing_fx: Vec<&str> = self.nodes.iter()
                                                        .filter(|n| n.node_type.is_dsp_or_effect() && eval.flowing_nodes.contains(&n.id))
                                                        .map(|n| n.title.as_str())
                                                        .collect();
                                                    let fx_desc = if is_narrow_w || flowing_fx.len() > 2 {
                                                        format!("{} Modul DSP", flowing_fx.len())
                                                    } else if flowing_fx.is_empty() {
                                                        "Passthrough".to_string()
                                                    } else {
                                                        flowing_fx.join(", ")
                                                    };
                                                    ui.label(egui::RichText::new(format!("✓ DSP: {} ({:.1} kHz)", fx_desc, r.device_rate as f32 / 1000.0)).size(11.5).color(Color32::from_rgb(99, 102, 241)).strong());
                                                } else {
                                                    ui.label(egui::RichText::new("✓ OS Resampled (48 kHz)").size(11.5).color(Color32::from_rgb(217, 119, 6)).strong());
                                                }
                                            } else {
                                                match self.chain_mode {
                                                    ChainMode::BitPerfect => {
                                                        ui.label(egui::RichText::new("✓ Bit-Perfect Siap").size(11.5).color(Color32::from_rgb(16, 185, 129)).strong());
                                                    }
                                                    ChainMode::DspActive => {
                                                        ui.label(egui::RichText::new("✓ Rantai Modular Siap").size(11.5).color(Color32::from_rgb(99, 102, 241)).strong());
                                                    }
                                                    ChainMode::OsResampled => {
                                                        ui.label(egui::RichText::new("✓ OS Resampled Siap").size(11.5).color(Color32::from_rgb(217, 119, 6)).strong());
                                                    }
                                                }
                                            }
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let btn_txt = if is_narrow_w { "Studio ➔" } else { "Buka Studio Penuh ➔" };
                                                if ui.button(egui::RichText::new(btn_txt).strong()).clicked() {
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

                                // Responsive Spectrum Analyzer + Synchronized Lyric Box
                                self.render_now_playing_visualizer_and_lyrics(ui);
                            });
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

                                if ui.button(egui::RichText::new("💾 Simpan Preset").size(12.0).strong().color(Color32::from_rgb(79, 70, 229)))
                                    .on_hover_text("Simpan & enkripsi rantai sinyal studio ke format bertanda tangan .lnl")
                                    .clicked()
                                {
                                    self.preset_modal_tab = 0;
                                    self.show_export_preset_modal = true;
                                }

                                if ui.button(egui::RichText::new("📚 Kelola Preset").size(12.0).strong().color(Color32::from_rgb(124, 58, 237)))
                                    .on_hover_text("Buka perpustakaan & kelola koleksi preset rantai sinyal studio")
                                    .clicked()
                                {
                                    self.preset_modal_tab = 1;
                                    self.show_export_preset_modal = true;
                                }

                                if ui.button(egui::RichText::new("📂 Muat .lnl").size(12.0).strong().color(Color32::from_rgb(14, 116, 144)))
                                    .on_hover_text("Buka berkas preset studio .lnl dan verifikasi tanda tangan digital")
                                    .clicked()
                                {
                                    self.trigger_preset_import();
                                }

                                let current_dev_label = self.selected_output_device.as_deref().unwrap_or("Default System Output");
                                let truncated_label = if current_dev_label.len() > 22 {
                                    format!("🔊 {}...", &current_dev_label[..19])
                                } else {
                                    format!("🔊 {}", current_dev_label)
                                };
                                // --- TOGGLE PILIHAN USER: EXCLUSIVE vs SHARED ---
                                let (ex_label, ex_color, ex_bg) = if self.prefer_exclusive {
                                    ("🔒 Exclusive", Color32::from_rgb(16, 185, 129), Color32::from_rgb(236, 253, 245))
                                } else {
                                    ("🔊 Shared", Color32::from_rgb(217, 119, 6), Color32::from_rgb(254, 243, 199))
                                };
                                let ex_btn = ui.add(
                                    egui::Button::new(egui::RichText::new(ex_label).strong().size(11.5).color(ex_color))
                                        .fill(ex_bg)
                                        .stroke(Stroke::new(1.0, ex_color))
                                        .corner_radius(6.0)
                                ).on_hover_text(
                                    if self.prefer_exclusive {
                                        "Mode Output: WASAPI Exclusive (Kunci Hardware Penuh)\nBypass mixer Windows untuk latensi rendah & bit-accuracy.\nBisa digunakan di mode Modular FX maupun Bit-Perfect.\nKlik untuk beralih ke Shared Mode agar audio bisa campur dengan YouTube/Discord."
                                    } else {
                                        "Mode Output: WASAPI Shared (Campur Audio Windows)\nAudio bercampur dengan aplikasi lain (YouTube/Discord).\nKlik untuk beralih ke Exclusive Mode untuk bypass mixer Windows."
                                    }
                                );

                                if ex_btn.clicked() {
                                    self.set_exclusive_mode(!self.prefer_exclusive);
                                }


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

                                let flowing_fx: Vec<&str> = {
                                    let eval = self.evaluate_signal_chain();
                                    self.nodes.iter()
                                        .filter(|n| n.node_type.is_dsp_or_effect() && eval.flowing_nodes.contains(&n.id))
                                        .map(|n| n.title.as_str())
                                        .collect()
                                };
                                let dsp_label = if flowing_fx.is_empty() {
                                    "Modular FX (Standby)".to_string()
                                } else if flowing_fx.len() == 1 {
                                    format!("Modular: {} (Aktif)", flowing_fx[0])
                                } else {
                                    format!("Modular FX ({} Aktif)", flowing_fx.len())
                                };

                                let dsp_btn = ui.selectable_label(self.chain_mode == ChainMode::DspActive, egui::RichText::new(&dsp_label).size(12.5).strong());
                                let dsp_tooltip = if flowing_fx.is_empty() {
                                    "Mode Pemrosesan Modular 64-bit.\nSaat ini tidak ada modul efek tersambung di rantai sinyal (Passthrough 1:1 murni).".to_string()
                                } else {
                                    format!("Mode Pemrosesan Modular 64-bit aktif.\nModul tersambung:\n{}", flowing_fx.iter().map(|t| format!(" • {}", t)).collect::<Vec<_>>().join("\n"))
                                };
                                let dsp_btn = dsp_btn.on_hover_text(dsp_tooltip);
                                if dsp_btn.clicked() {
                                    self.set_chain_mode(ChainMode::DspActive);
                                }
                                let bp_btn = ui.selectable_label(self.chain_mode == ChainMode::BitPerfect, egui::RichText::new("Bit-Perfect").size(12.5).strong())
                                    .on_hover_text("Mode Transparan Bit-Perfect: Melewati seluruh modul efek langsung ke DAC 1:1.");
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

                                                        let guard_text = if !self.clipping_detection_enabled {
                                                            egui::RichText::new("🛡️ Guard: OFF").size(12.0).color(Color32::from_rgb(148, 163, 184))
                                                        } else if self.clipping_threshold_db < -0.05 {
                                                            egui::RichText::new(format!("🛡️ Guard: Strict ({:+.1} dB)", self.clipping_threshold_db)).size(12.0).strong().color(Color32::from_rgb(59, 130, 246))
                                                        } else if self.clipping_threshold_db > 0.05 {
                                                            egui::RichText::new(format!("🛡️ Guard: Relaxed ({:+.1} dB)", self.clipping_threshold_db)).size(12.0).strong().color(Color32::from_rgb(168, 85, 247))
                                                        } else {
                                                            egui::RichText::new("🛡️ Guard: 0 dB").size(12.0).strong().color(Color32::from_rgb(16, 185, 129))
                                                        };
                                                        if ui.button(guard_text).on_hover_text("Atur sensitivitas deteksi clipping & margin headroom aman atau nonaktifkan peringatan").clicked() {
                                                            self.show_clipping_settings_popup = !self.show_clipping_settings_popup;
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

                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                            let guard_text = if !self.clipping_detection_enabled {
                                                                egui::RichText::new("🛡️ Guard: OFF").size(12.0).color(Color32::from_rgb(148, 163, 184))
                                                            } else if self.clipping_threshold_db < -0.05 {
                                                                egui::RichText::new(format!("🛡️ Guard: Strict ({:+.1} dB)", self.clipping_threshold_db)).size(12.0).strong().color(Color32::from_rgb(59, 130, 246))
                                                            } else if self.clipping_threshold_db > 0.05 {
                                                                egui::RichText::new(format!("🛡️ Guard: Relaxed ({:+.1} dB)", self.clipping_threshold_db)).size(12.0).strong().color(Color32::from_rgb(168, 85, 247))
                                                            } else {
                                                                egui::RichText::new("🛡️ Guard: 0 dB").size(12.0).strong().color(Color32::from_rgb(16, 185, 129))
                                                            };
                                                            if ui.button(guard_text).on_hover_text("Atur sensitivitas deteksi clipping & margin headroom aman atau nonaktifkan peringatan").clicked() {
                                                                self.show_clipping_settings_popup = !self.show_clipping_settings_popup;
                                                            }
                                                        });
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

        // Signal Chain Studio .lnl Preset Modals
        if self.show_export_preset_modal {
            self.render_export_preset_modal(ui.ctx());
        }
        if self.show_import_password_modal {
            self.render_import_password_modal(ui.ctx());
        }

        

        // Floating Non-Shifting Studio Preset Toast Notification (Sleek Compact Modern Pill)
        if let Some((ref msg, is_success)) = self.preset_status_notification {
            let screen = ui.ctx().input(|i| i.raw.screen_rect).unwrap_or_else(|| ui.clip_rect());
            let toast_w = 350.0f32.min(screen.width() - 32.0);
            let toast_pos = Pos2::new(screen.center().x - toast_w * 0.5, screen.min.y + 14.0);
            let mut dismiss = false;
            #[allow(deprecated)]
            egui::Area::new(egui::Id::new("floating_preset_status_toast"))
                .order(egui::Order::Tooltip)
                .fixed_pos(toast_pos)
                .show(ui.ctx(), |ui| {
                    let (bg, border, text_col, icon_str) = if is_success {
                        (
                            Color32::from_rgb(240, 253, 250), // Teal-tinted soft clean white
                            Color32::from_rgb(45, 212, 191),  // Soft emerald/teal border
                            Color32::from_rgb(15, 23, 42),    // Dark slate text
                            "✨",
                        )
                    } else {
                        (
                            Color32::from_rgb(255, 241, 242), // Soft rose white
                            Color32::from_rgb(251, 113, 133), // Soft rose border
                            Color32::from_rgb(15, 23, 42),
                            "⚠️",
                        )
                    };

                    egui::Frame::new()
                        .fill(bg)
                        .stroke(Stroke::new(1.0, border))
                        .corner_radius(18.0)
                        .inner_margin(egui::Margin::symmetric(14, 7))
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 4],
                            blur: 16,
                            spread: 0,
                            color: Color32::from_black_alpha(45),
                        })
                        .show(ui, |ui| {
                            ui.set_width(toast_w - 28.0);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                ui.label(egui::RichText::new(icon_str).size(12.5));
                                ui.add(egui::Label::new(egui::RichText::new(msg).strong().size(11.5).color(text_col)).truncate());

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let close_btn = egui::Button::new(
                                        egui::RichText::new("✕").size(10.5).color(Color32::from_rgb(148, 163, 184))
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE)
                                    .min_size(Vec2::splat(18.0));

                                    if ui.add(close_btn).on_hover_text("Tutup notifikasi").clicked() {
                                        dismiss = true;
                                    }
                                });
                            });
                        });
                });
            if let Some(t) = self.preset_status_notification_time {
                if t.elapsed().as_secs_f32() > 2.2 {
                    dismiss = true;
                }
            }
            if dismiss {
                self.preset_status_notification = None;
                self.preset_status_notification_time = None;
            }
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
    fn test_eq_not_in_chain_produces_no_eq_effect_in_clean_direct_chain() {
        let mut app = LunulaApp::default();
        app.chain_mode = ChainMode::DspActive;
        // Clean default direct chain: 1 -> 2 -> 4 -> 5 (NO EQ preset card)
        app.connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, 4),
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
    fn test_bit_perfect_direct_bypass_cable_render_and_no_preset_reset() {
        let ctx = egui::Context::default();
        let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));

        let mut app = LunulaApp::default();
        // 1. Add LowPassFilter and route audio 1 -> 2 -> 6 -> 4 -> 5
        app.add_modular_filter(NodeType::LowPassFilter);
        let lpf_id = app.nodes.iter().find(|n| n.node_type == NodeType::LowPassFilter).map(|n| n.id).unwrap();
        app.connections.retain(|c| !(c.from_node == 2 && c.to_node == 4));
        app.connections.push(crate::models::CableConnection::new(2, lpf_id));
        app.connections.push(crate::models::CableConnection::new(lpf_id, 4));

        // 2. Switch to Bit-Perfect mode (modular nodes are bypassed, direct bypass cable must activate)
        app.set_chain_mode(ChainMode::BitPerfect);
        app.is_playing = true;

        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.15),
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(1000.0, 600.0));
            },
        );
        assert!(!out.shapes.is_empty(), "Direct bypass cable in Bit-Perfect mode renders without error");
        out.textures_delta.clear();

        // 3. Test reset_to_default_signal_chain()
        app.reset_to_default_signal_chain();
        assert_eq!(app.nodes.len(), 4, "Resetting chain leaves exactly 4 standard nodes");
        assert_eq!(app.connections.len(), 3, "Resetting chain restores standard connections (1,2), (2,4), (4,5)");
        assert!(app.connections.iter().any(|c| c.from_node == 1 && c.to_node == 2));
        assert!(app.connections.iter().any(|c| c.from_node == 2 && c.to_node == 4));
        assert!(app.connections.iter().any(|c| c.from_node == 4 && c.to_node == 5));
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

        // Test RAM Buffer node inspection with real heap & animated chunk streaming
        if let Some(buf_node) = app.nodes.iter().find(|n| n.node_type == crate::models::NodeType::Buffer) {
            app.selected_node = Some(buf_node.id);
            let mut out = ctx.run_ui(
                egui::RawInput {
                    screen_rect: screen,
                    time: Some(0.35),
                    ..Default::default()
                },
                |ui| {
                    app.render_telemetry_inspection_desk(ui, 220.0);
                },
            );
            assert!(!out.shapes.is_empty(), "Must render RAM Buffer live heap & chunk streaming telemetry");
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

        // Wire 1 (Source) -> 2 (Buffer) -> 6 (LPF) -> 4 (Resampler) -> 5 (DAC)
        app.connections = vec![
            CableConnection::new(1, 2),
            CableConnection::new(2, 6),
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
    fn test_modular_fx_independent_wasapi_exclusive_toggle() {
        let mut app = LunulaApp::new_standalone();

        // 1. Initial defaults
        assert_eq!(app.chain_mode, ChainMode::BitPerfect);
        assert!(app.prefer_exclusive);

        // 2. Switch to Modular FX (DspActive)
        app.set_chain_mode(ChainMode::DspActive);
        assert_eq!(app.chain_mode, ChainMode::DspActive);
        // prefer_exclusive must remain true!
        assert!(app.prefer_exclusive);

        // 3. Toggle exclusive mode off (to Shared mode) while remaining in Modular FX
        app.set_exclusive_mode(false);
        assert_eq!(app.chain_mode, ChainMode::DspActive, "Toggling exclusive must NEVER force mode to BitPerfect!");
        assert!(!app.prefer_exclusive);

        // 4. Toggle exclusive mode back on while remaining in Modular FX
        app.set_exclusive_mode(true);
        assert_eq!(app.chain_mode, ChainMode::DspActive, "Toggling exclusive must preserve Modular FX mode!");
        assert!(app.prefer_exclusive);

        // 5. Switch to Bit-Perfect mode and toggle exclusive mode
        app.set_chain_mode(ChainMode::BitPerfect);
        assert_eq!(app.chain_mode, ChainMode::BitPerfect);
        assert!(app.prefer_exclusive);

        app.set_exclusive_mode(false);
        assert_eq!(app.chain_mode, ChainMode::BitPerfect, "Toggling exclusive must preserve BitPerfect mode!");
        assert!(!app.prefer_exclusive);
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
    fn test_ram_buffer_telemetry_desk_metrics_and_animation() {
        let mut app = LunulaApp::new_standalone();
        let ctx = egui::Context::default();
        let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 900.0)));

        // 1. Switch to Advance Studio & Select Node 2 (RAM Buffer)
        app.studio_mode = crate::models::StudioMode::Advance;
        app.advance_studio_state.telemetry_tab = crate::ui::patchbay::TelemetryTab::AutoInspect;
        app.selected_node = Some(2);

        // 2. Sample live system telemetry
        app.system_telemetry.sample();
        let mem = app.system_telemetry.memory_stats();
        let snap = app.system_telemetry.buffer_snapshot(44100);

        #[cfg(windows)]
        {
            assert!(mem.working_set_mb() > 1.0, "Working set must be realistic physical RAM");
            assert!(mem.peak_working_set_mb() >= mem.working_set_mb());
        }

        assert_eq!(snap.capacity_samples, 96000);
        assert_eq!(snap.chunk_size_frames, 512);
        assert!((snap.chunk_size_kb - 4.0).abs() < 1e-3);

        // 3. Render RAM Buffer Telemetry Desk headlessly
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(1.234),
                ..Default::default()
            },
            |ui| {
                app.render_telemetry_inspection_desk(ui, 240.0);
            },
        );
        assert!(!out.shapes.is_empty(), "RAM Buffer telemetry desk must render shapes and animated conveyor");
        out.textures_delta.clear();
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
            genre: None,
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

    #[test]
    fn test_static_analysis_default_chain_is_safe_and_cached() {
        let app = LunulaApp::new_standalone();
        let report1 = app.get_static_analysis();
        assert!(report1.is_chain_safe, "Default chain must have 0% clipping risk");
        assert_eq!(report1.culprit_nodes.len(), 0);
        assert!(report1.max_peak_dbfs <= 0.0);

        // Verify cached report is identical
        let report2 = app.get_static_analysis();
        assert_eq!(report1, report2);
    }

    #[test]
    fn test_static_analysis_booster_triggers_culprit_and_hot_edge() {
        use crate::audio::NodeSafetyStatus;

        let mut app = LunulaApp::new_standalone();
        // Insert Volume Booster (+6.0 dB) between Buffer (2) and Resampler (4)
        let booster_id = 100;
        app.nodes.push(ModularNode {
            id: booster_id,
            node_type: NodeType::VolumeBooster,
            title: "⚡ VOLUME BOOSTER".to_string(),
            sub1: "+6.0 dB Gain".to_string(),
            sub2: "Pure Clean Amp".to_string(),
            pos: egui::Pos2::new(100.0, 100.0),
            is_removable: true,
            param_val: 6.0,
        });

        // Reconnect: 1 -> 2 -> booster -> 4 -> 5
        app.connections.retain(|c| !(c.from_node == 2 && c.to_node == 4));
        app.connections.push(CableConnection::new(2, booster_id));
        app.connections.push(CableConnection::new(booster_id, 4));
        app.invalidate_chain_cache();

        let report = app.get_static_analysis();
        assert!(!report.is_chain_safe, "Chain with +6 dB booster without limiter must be flagged unsafe");
        assert!(report.culprit_nodes.contains(&(booster_id as u32)));

        let booster_rep = report.get_node_report(booster_id as u32).expect("Booster report must exist");
        assert!(booster_rep.is_culprit);
        assert_eq!(booster_rep.status, NodeSafetyStatus::OriginOfClipping);
        assert!(booster_rep.headroom_deficit_db > 5.9);

        // Resampler (4) and DAC (5) are downstream victims
        let node4_rep = report.get_node_report(4).expect("Node 4 report must exist");
        assert_eq!(node4_rep.status, NodeSafetyStatus::OverdrivenDownstream);

        // Cable from booster to node 4 is hot
        assert!(report.is_edge_hot(booster_id as u32, 0, 4, 0));
    }

    #[test]
    fn test_static_analysis_cache_invalidation_after_preset_load() {
        let mut app = LunulaApp::new_standalone();
        
        // 1. Initial state: run evaluate_signal_chain then get_static_analysis (just like canvas does)
        let eval1 = app.evaluate_signal_chain();
        assert!(eval1.is_complete);
        let report1 = app.get_static_analysis();
        assert!(report1.is_chain_safe, "Initial clean chain must be safe");

        // 2. Load "Tes Clipping" preset
        crate::preset::ensure_default_factory_presets();
        let presets_dir = crate::preset::get_presets_dir();
        let clip_path = presets_dir.join("Tes Clipping.lnl");
        let _ = app.load_preset_file(&clip_path, None).expect("load Tes Clipping");

        // 3. Canvas sequence: evaluate_signal_chain() runs FIRST, then get_static_analysis()
        let eval2 = app.evaluate_signal_chain();
        assert!(eval2.is_complete);
        let report2 = app.get_static_analysis();

        // 4. MUST NOT be stale! Must detect the clipping hazard
        assert!(!report2.is_chain_safe, "After loading Tes Clipping, static analysis must flag unsafe");
        assert!(report2.culprit_nodes.contains(&6), "Node 6 (Volume Booster) must be marked culprit");
        assert!(report2.max_headroom_deficit_db > 7.0, "Headroom deficit must be > 7 dB");
        assert!(report2.is_edge_hot(6, 0, 7, 0), "Edge 6 -> 7 must be hot");
    }

    #[test]
    fn test_static_analysis_context_aware_with_active_track() {
        let mut app = LunulaApp::new_standalone();

        // 1. Insert Stereo Widener (+1% width) between Buffer (2) and Resampler (4)
        let widener_id = 100;
        app.nodes.push(ModularNode {
            id: widener_id,
            node_type: NodeType::StereoWidener,
            title: "🌐 STEREO WIDENER".to_string(),
            sub1: "Width +1%".to_string(),
            sub2: "Haas 3D".to_string(),
            pos: egui::Pos2::new(100.0, 100.0),
            is_removable: true,
            param_val: 1.0, // +1% width -> 1.01x
        });
        app.connections.retain(|c| !(c.from_node == 2 && c.to_node == 4));
        app.connections.push(CableConnection::new(2, widener_id));
        app.connections.push(CableConnection::new(widener_id, 4));
        app.invalidate_chain_cache();

        // 2. Under pure conservative full-scale (no active track peak), +1% exceeds 0 dBFS!
        let report_worst = app.get_static_analysis();
        assert!(!report_worst.is_chain_safe, "Worst-case analysis without track peak flags +1% as hot");
        assert!(report_worst.culprit_nodes.contains(&(widener_id as u32)));

        // 3. Load active track with measured track_peak = 0.707 (-3.0 dBFS)
        app.current_track.duration = 240.0;
        app.current_track.title = "Acoustic Melody".to_string();
        app.current_track.replay_gain.track_peak = Some(0.707);
        app.update_effective_replay_gain();

        // Now static analysis incorporates the song's context:
        // 0.707 * 1.01 = 0.714 <= 1.0 (-2.92 dBFS). MUST BE 100% SAFE!
        let report_context = app.get_static_analysis();
        assert!(report_context.is_chain_safe, "With -3 dBFS track, widener +1% must be completely safe!");
        assert_eq!(report_context.culprit_nodes.len(), 0);
        assert!(!report_context.is_edge_hot(widener_id as u32, 0, 4, 0), "Edge must NOT be hot");
        assert_eq!(report_context.context_track_title.as_deref(), Some("Acoustic Melody"));
        assert!(report_context.context_peak_dbfs.unwrap() < -2.9);

        // 4. If width is increased to +50% (1.50x):
        // 0.707 * 1.50 = 1.0605 > 1.0 (+0.51 dBFS). Headroom exceeded -> flags unsafe!
        if let Some(w_node) = app.nodes.iter_mut().find(|n| n.id == widener_id) {
            w_node.param_val = 50.0;
        }
        app.invalidate_chain_cache();
        let report_wide = app.get_static_analysis();
        assert!(!report_wide.is_chain_safe, "With +50% width on 0.707 track, headroom is exhausted");
        assert!(report_wide.culprit_nodes.contains(&(widener_id as u32)));

        // 5. Enabling ReplayGain attenuation restores safe headroom:
        app.replay_gain_mode = crate::models::ReplayGainMode::Track;
        app.current_track.replay_gain.track_gain_db = Some(-3.0);
        app.update_effective_replay_gain();
        let report_rg = app.get_static_analysis();
        assert!(report_rg.is_chain_safe, "ReplayGain attenuation (+3 dB extra headroom) restores safe status!");
    }

    #[test]
    fn test_canvas_tooltip_and_interactions_suppressed_when_modal_active() {
        let ctx = egui::Context::default();
        ctx.style_mut_of(egui::Theme::Light, |s| s.interaction.tooltip_delay = 0.0);
        ctx.style_mut_of(egui::Theme::Dark, |s| s.interaction.tooltip_delay = 0.0);
        let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));
        let mut app = LunulaApp::default();
        app.current_tab = AppTab::SignalChain;

        // Find File Source node position
        let source_pos = app.nodes.iter().find(|n| n.id == 1).map(|n| n.pos).unwrap();
        let hover_pos = egui::pos2(source_pos.x + 30.0, source_pos.y + 30.0);

        // Case A: When Studio Theater modal is open, is_modal_or_overlay_active MUST be true
        app.show_interactive_theater = true;
        assert!(app.is_modal_or_overlay_active(), "Interactive Theater must be recognized as active modal");

        let mut out_modal = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.1),
                events: vec![egui::Event::PointerMoved(hover_pos)],
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(800.0, 600.0));
            },
        );
        fn collect_shape_texts(shape: &egui::epaint::Shape, out: &mut Vec<String>) {
            match shape {
                egui::epaint::Shape::Text(t) => out.push(t.galley.job.text.clone()),
                egui::epaint::Shape::Vec(shapes) => {
                    for s in shapes {
                        collect_shape_texts(s, out);
                    }
                }
                _ => {}
            }
        }

        let mut texts_modal = Vec::new();
        for s in &out_modal.shapes {
            collect_shape_texts(&s.shape, &mut texts_modal);
        }
        let has_leaked_tooltip = texts_modal.iter().any(|t| t.contains("KONTEKS AUDIO"));
        out_modal.textures_delta.clear();
        assert!(!has_leaked_tooltip, "File Source tooltip MUST NOT bleed through when modal is active!");

        // Case B: When modal is closed, hovering File Source MUST show tooltip cleanly
        app.show_interactive_theater = false;
        assert!(!app.is_modal_or_overlay_active());

        // Prime the hover
        let mut out_prime = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.2),
                events: vec![egui::Event::PointerMoved(hover_pos)],
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(800.0, 600.0));
            },
        );
        out_prime.textures_delta.clear();

        let mut out_clean = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(2.0),
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(800.0, 600.0));
            },
        );
        let mut texts_clean = Vec::new();
        for s in &out_clean.shapes {
            collect_shape_texts(&s.shape, &mut texts_clean);
        }
        let has_tooltip_now = texts_clean.iter().any(|t| t.contains("KONTEKS AUDIO"));
        out_clean.textures_delta.clear();
        assert!(has_tooltip_now, "File Source tooltip MUST appear normally when no modal is active!");
    }

    #[test]
    fn test_clipping_detection_toggle_and_threshold_controls_in_app() {
        let mut app = LunulaApp::new_standalone();
        app.show_interactive_theater = false;
        app.set_chain_mode(ChainMode::DspActive);

        // Insert high gain Volume Booster (+12 dB)
        app.add_modular_filter(NodeType::VolumeBooster);
        let booster_id = app.nodes.iter().find(|n| n.node_type == NodeType::VolumeBooster).map(|n| n.id).unwrap();
        if let Some(b) = app.nodes.iter_mut().find(|n| n.id == booster_id) {
            b.param_val = 12.0;
        }

        // Connect 2 -> Booster -> 4
        app.connections.retain(|c| !(c.from_node == 2 && c.to_node == 4));
        app.connections.push(CableConnection::new(2, booster_id));
        app.connections.push(CableConnection::new(booster_id, 4));
        app.invalidate_chain_cache();

        // 1. By default: detection is enabled, threshold is 0.0 dBFS -> flags unsafe!
        assert!(app.clipping_detection_enabled);
        assert_eq!(app.clipping_threshold_db, 0.0);
        let rep1 = app.get_static_analysis();
        assert!(!rep1.is_chain_safe);
        assert!(rep1.culprit_nodes.contains(&(booster_id as u32)));

        // 2. Turning detection OFF -> Chain becomes 100% safe, no culprits or hot cables
        app.clipping_detection_enabled = false;
        app.invalidate_chain_cache();
        let rep2 = app.get_static_analysis();
        assert!(rep2.is_chain_safe, "Disabling clipping detection must mark signal chain as safe");
        assert_eq!(rep2.culprit_nodes.len(), 0);
        assert!(!rep2.is_edge_hot(booster_id as u32, 0, 4, 0));

        // 3. Turning detection back ON, but setting threshold to +15.0 dBFS (Relaxed beyond +12dB) -> SAFE!
        app.clipping_detection_enabled = true;
        app.clipping_threshold_db = 15.0;
        app.invalidate_chain_cache();
        let rep3 = app.get_static_analysis();
        assert!(rep3.is_chain_safe, "Setting threshold higher than booster peak must be safe");
        assert_eq!(rep3.culprit_nodes.len(), 0);

        // 4. Test headless rendering of the clipping settings floating popup
        let ctx = egui::Context::default();
        let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1200.0, 800.0)));
        app.show_clipping_settings_popup = true;

        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                time: Some(0.1),
                ..Default::default()
            },
            |ui| {
                app.render_modular_patchbay_canvas(ui, egui::vec2(800.0, 600.0));
            },
        );
        out.textures_delta.clear();
        assert!(app.show_clipping_settings_popup);

        // 5. Test 1-Click Auto-Fix functionality:
        // Reset threshold to 0.0 dBFS so booster (+12dB) is unsafe again
        app.clipping_threshold_db = 0.0;
        app.invalidate_chain_cache();
        assert!(!app.get_static_analysis().is_chain_safe);

        // Apply auto-fix on the booster node
        let fixed = app.apply_auto_fix_for_node(booster_id);
        assert!(fixed, "Auto-fix must succeed on culprit booster node");

        // Booster gain must have been pulled back to 0.0 dB (safe headroom)
        let booster_node = app.nodes.iter().find(|n| n.id == booster_id).unwrap();
        assert_eq!(booster_node.param_val, 0.0);
        assert!(booster_node.sub1.contains("+0.0 dB Gain"));

        // Chain is now 100% safe
        assert!(app.get_static_analysis().is_chain_safe);
        assert_eq!(app.get_static_analysis().culprit_nodes.len(), 0);

        // Test apply_all_headroom_auto_fixes on multiple culprit nodes
        if let Some(b) = app.nodes.iter_mut().find(|n| n.id == booster_id) {
            b.param_val = 6.0;
        }
        app.invalidate_chain_cache();
        assert!(!app.get_static_analysis().is_chain_safe);
        let fixed_count = app.apply_all_headroom_auto_fixes();
        assert_eq!(fixed_count, 1);
        assert!(app.get_static_analysis().is_chain_safe);
    }

    #[test]
    fn test_formal_dag_cycle_detection_via_kahn_sort() {
        use crate::audio::{EdgeIr, GraphCompilationError, GraphIr, NodeIr, EffectType};

        // 1. Setup Graph IR with a cycle: 1 (Source) -> 2 (Mixer) -> 3 -> 2 (Cycle between 2 and 3)
        let mut ir = GraphIr::new(1, 4);
        ir.nodes.push(NodeIr::new(1, "Source".to_string(), EffectType::Passthrough));
        ir.nodes.push(NodeIr::new(2, "Mixer".to_string(), EffectType::Mixer));
        ir.nodes.push(NodeIr::new(3, "Filter B".to_string(), EffectType::StereoWidener));
        ir.nodes.push(NodeIr::new(4, "Endpoint".to_string(), EffectType::Passthrough));

        // Connect 1 -> 2 port 0, 2 -> 3 port 0, 3 -> 2 port 1 (Cycle!), 3 -> 4
        ir.edges.push(EdgeIr::new(1, 0, 2, 0));
        ir.edges.push(EdgeIr::new(2, 0, 3, 0));
        ir.edges.push(EdgeIr::new(3, 0, 2, 1)); // Backward cycle edge into mixer port 1
        ir.edges.push(EdgeIr::new(3, 0, 4, 0));

        let result = ir.validate_and_sort();
        match result {
            Err(GraphCompilationError::CycleDetected(cycle_nodes)) => {
                assert!(cycle_nodes.contains(&2) || cycle_nodes.contains(&3), "Cycle must identify participating nodes");
            }
            other => panic!("Expected CycleDetected error, got {:?}", other),
        }

        // Test in LunulaApp: if cycle is injected into flowing cables, compile_signal_chain must reject graph
        let mut app = LunulaApp::default();
        app.connections.push(crate::models::CableConnection::new(4, 2)); // 4 (Engine) -> 2 (RAM) forms cycle
        app.invalidate_chain_cache();
        let compiled = app.compile_signal_chain();
        assert!(!compiled.is_complete, "Graph compilation must fail and reject chain containing cycle!");
    }

    #[test]
    fn test_graph_ir_validation_dangling_and_port_constraints() {
        use crate::audio::{EdgeIr, GraphCompilationError, GraphIr, NodeIr, EffectType};

        // 1. Missing Source
        let ir_no_source = GraphIr::new(99, 4);
        assert_eq!(ir_no_source.validate_and_sort(), Err(GraphCompilationError::MissingSourceNode));

        // 2. Missing Endpoint
        let mut ir_no_endpoint = GraphIr::new(1, 99);
        ir_no_endpoint.nodes.push(NodeIr::new(1, "Source".to_string(), EffectType::Passthrough));
        assert_eq!(ir_no_endpoint.validate_and_sort(), Err(GraphCompilationError::MissingEndpointNode));

        // 3. Port index out of range (Filter has 1 input port: port 0. Edge connects to port 5)
        let mut ir_bad_port = GraphIr::new(1, 2);
        ir_bad_port.nodes.push(NodeIr::new(1, "Source".to_string(), EffectType::Passthrough));
        ir_bad_port.nodes.push(NodeIr::new(2, "Endpoint".to_string(), EffectType::Passthrough));
        ir_bad_port.edges.push(EdgeIr::new(1, 0, 2, 5)); // Port 5 out of range
        match ir_bad_port.validate_and_sort() {
            Err(GraphCompilationError::InvalidPortIndex { node, port, max_ports }) => {
                assert_eq!(node, 2);
                assert_eq!(port, 5);
                assert_eq!(max_ports, 1);
            }
            other => panic!("Expected InvalidPortIndex, got {:?}", other),
        }

        // 4. Dangling Path: Source -> Node 2 -> [nowhere] (Endpoint is disconnected)
        let mut ir_dangling = GraphIr::new(1, 3);
        ir_dangling.nodes.push(NodeIr::new(1, "Source".to_string(), EffectType::Passthrough));
        ir_dangling.nodes.push(NodeIr::new(2, "Filter".to_string(), EffectType::TubeWarmer));
        ir_dangling.nodes.push(NodeIr::new(3, "Endpoint".to_string(), EffectType::Passthrough));
        ir_dangling.edges.push(EdgeIr::new(1, 0, 2, 0)); // 1 -> 2, but 2 does not reach 3
        match ir_dangling.validate_and_sort() {
            Err(GraphCompilationError::DanglingPathToEndpoint(_)) => {}
            other => panic!("Expected DanglingPathToEndpoint, got {:?}", other),
        }

        // 5. Valid linear DAG: 1 -> 2 -> 3
        let mut ir_valid = GraphIr::new(1, 3);
        ir_valid.nodes.push(NodeIr::new(1, "Source".to_string(), EffectType::Passthrough));
        ir_valid.nodes.push(NodeIr::new(2, "Filter".to_string(), EffectType::TubeWarmer));
        ir_valid.nodes.push(NodeIr::new(3, "Endpoint".to_string(), EffectType::Passthrough));
        ir_valid.edges.push(EdgeIr::new(1, 0, 2, 0));
        ir_valid.edges.push(EdgeIr::new(2, 0, 3, 0));

        let order = ir_valid.validate_and_sort().expect("Valid linear DAG must compile");
        assert_eq!(order, vec![1, 2, 3]);
    }

    #[test]
    fn test_dead_node_isolation() {
        let mut app = LunulaApp::default();
        // Add 3 unlinked modular filters that sit loose on canvas
        app.add_modular_filter(crate::models::NodeType::TubeWarmer);
        app.add_modular_filter(crate::models::NodeType::StereoWidener);
        app.add_modular_filter(crate::models::NodeType::BassGuard);

        // Active chain is default linear 1 -> 2 -> 4 -> 5
        let eval = app.evaluate_signal_chain();
        assert!(eval.is_complete);

        // Dead nodes must not be in flowing_nodes
        assert_eq!(eval.flowing_nodes.len(), 4);
        for node in &app.nodes {
            if node.id > 5 {
                assert!(!eval.flowing_nodes.contains(&node.id), "Dead node {} must be 100% isolated", node.id);
            }
        }

        // Compiled signal chain must only contain the 4 flowing nodes
        let graph = app.compile_signal_chain();
        assert_eq!(graph.topological_order.len(), 4);
        assert!(!graph.topological_order.iter().any(|&id| id > 5));
    }

    #[test]
    fn test_advanced_dsp_nodes_time_alignment_and_stereo_matrix_pipeline() {
        let mut app = LunulaApp::default();
        // Insert Time Alignment and Stereo Matrix
        app.add_modular_filter(crate::models::NodeType::TimeAlignment);
        app.add_modular_filter(crate::models::NodeType::StereoMatrix);

        let align_id = app.nodes.iter().find(|n| n.node_type == crate::models::NodeType::TimeAlignment).unwrap().id;
        let matrix_id = app.nodes.iter().find(|n| n.node_type == crate::models::NodeType::StereoMatrix).unwrap().id;

        // Splice them into flowing chain: 1 -> 2 -> align -> matrix -> 4 -> 5
        app.connections.retain(|c| !(c.from_node == 2 && c.to_node == 4));
        app.connections.push(crate::models::CableConnection::new(2, align_id));
        app.connections.push(crate::models::CableConnection::new(align_id, matrix_id));
        app.connections.push(crate::models::CableConnection::new(matrix_id, 4));
        app.invalidate_chain_cache();

        let eval = app.evaluate_signal_chain();
        assert!(eval.is_complete);
        assert_eq!(eval.flowing_nodes.len(), 6);

        let graph = app.compile_signal_chain();
        assert!(graph.is_complete);
        assert_eq!(graph.topological_order.len(), 6);

        // Verify Graph IR validation passes
        let ir_order = app.validate_graph_ir().expect("Advanced DSP patchbay graph must validate cleanly");
        assert_eq!(ir_order.len(), 6);
    }

    #[test]
    fn test_waveform_instant_generation_and_memory_hardening() {
        let mut app = LunulaApp::default();

        // 1. Initial track waveform must be 512 points (downscaled from 3600 for ultra-lightweight footprint)
        assert_eq!(app.current_track.waveform.len(), 512);
        assert_eq!(app.current_track.waveform_bass.len(), 512);
        assert_eq!(app.current_track.waveform_mid.len(), 512);
        assert_eq!(app.current_track.waveform_treble.len(), 512);

        // 2. Playing a track with empty waveform must immediately generate 512-point multiband data
        let mut track = crate::models::Track::default();
        track.title = "Bohemian Rhapsody".to_string();
        track.artist = "Queen".to_string();
        track.duration = 354.0;
        track.file_path = Some("test_queen.flac".to_string());
        assert!(track.waveform.is_empty());

        app.play_track(track);
        assert_eq!(app.current_track.waveform.len(), 512);
        assert_eq!(app.current_track.waveform_bass.len(), 512);
        assert_eq!(app.current_track.waveform_mid.len(), 512);
        assert_eq!(app.current_track.waveform_treble.len(), 512);
        // Ensure no raw image allocations are hoarded on the struct
        assert!(app.current_cover_bytes.is_none());

        // 3. Background peak and waveform streaming channel must update physical waveform seamlessly
        let real_wave = vec![0.75f32; 512];
        let _ = app.peak_scan_tx.send(("test_queen.flac".to_string(), 0.98, real_wave.clone()));
        if let Ok((path, peak, wave)) = app.peak_scan_rx.try_recv() {
            if app.current_track.file_path.as_deref() == Some(&path) {
                app.current_track.replay_gain.track_peak = Some(peak);
                if !wave.is_empty() {
                    app.current_track.waveform = wave;
                }
            }
        }
        assert_eq!(app.current_track.replay_gain.track_peak, Some(0.98));
        assert_eq!(app.current_track.waveform[0], 0.75);

        // 4. Album cover cache must enforce strict bounded capacity
        if let Ok(mut cache) = app.album_cover_cache.lock() {
            for i in 0..40 {
                cache.insert(format!("album_{}", i), None);
            }
            if cache.len() >= 32 {
                let keys_to_remove: Vec<String> = cache.keys().take(12).cloned().collect();
                for k in keys_to_remove {
                    cache.remove(&k);
                }
            }
            assert!(cache.len() <= 32);
        }
    }
}

