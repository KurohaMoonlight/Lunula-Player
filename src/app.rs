use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2};
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
    pub connecting_from_node: Option<usize>, // When dragging from an output pin

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

    // Global Output Stage (Separated from EQ preset)
    pub eq_limiter_enabled: bool,
    pub eq_dither_mode: DitherMode,
    pub eq_clip_count: u32,

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
                pos: Pos2::new(20.0, 48.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 2,
                node_type: NodeType::Buffer,
                title: "RAM BUFFER".to_string(),
                sub1: "RingBuffer".to_string(),
                sub2: "100% Loaded".to_string(),
                pos: Pos2::new(176.0, 48.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 3,
                node_type: NodeType::Dsp,
                title: "DSP ENGINE".to_string(),
                sub1: "BYPASSED (Direct Wire)".to_string(),
                sub2: "0.0 dB Pure Master".to_string(),
                pos: Pos2::new(332.0, 48.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 4,
                node_type: NodeType::Driver,
                title: "APP RESAMPLER".to_string(),
                sub1: "System Audio Endpoint".to_string(),
                sub2: "Rubato Sinc SRC • 48.0 kHz".to_string(),
                pos: Pos2::new(488.0, 48.0),
                is_removable: false,
                param_val: 0.0,
            },
            ModularNode {
                id: 5,
                node_type: NodeType::Endpoint,
                title: "OUTPUT DAC".to_string(),
                sub1: "Audio Endpoint".to_string(),
                sub2: "Native Sample Rate".to_string(),
                pos: Pos2::new(644.0, 48.0),
                is_removable: false,
                param_val: 0.0,
            },
        ];

        // Default chain connections: in Bit-Perfect (Option B), cable directly jumps 2 -> 4 bypassing 3
        let initial_connections = vec![
            CableConnection { from_node: 1, to_node: 2 },
            CableConnection { from_node: 2, to_node: 4 },
            CableConnection { from_node: 4, to_node: 5 },
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

            eq_limiter_enabled: false,
            eq_dither_mode: DitherMode::None,
            eq_clip_count: 0,

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
            app.set_chain_mode(state.chain_mode);
        } else {
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
            engine.set_eq_bypass(app.eq_bypass);
            for (idx, &gain) in app.eq_bands.iter().enumerate() {
                engine.set_eq_band(idx, gain);
            }
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

    pub fn play_track(&mut self, track: Track) {
        self.current_track = track;
        self.current_time = 0.0;
        self.is_playing = true;
        self.current_cover_bytes = self.current_track.file_path.as_ref().and_then(|p| {
            crate::scanner::extract_album_art(p)
        });
        self.current_cover_texture = None; // Reset so next frame builds the new texture

        if let Some(ref path_str) = self.current_track.file_path {
            if let Some(ref engine) = self.audio_engine {
                engine.set_chain_mode(self.chain_mode);
                engine.play(std::path::PathBuf::from(path_str));
                engine.set_volume(if self.is_muted { 0.0 } else { self.volume });
                for (idx, &gain) in self.eq_bands.iter().enumerate() {
                    engine.set_eq_band(idx, gain);
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
                    engine.set_volume(if self.is_muted { 0.0 } else { self.volume });
                }
            }
        }
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

    pub fn seek_relative(&mut self, delta_seconds: f32) {
        if self.current_track.duration > 0.0 {
            let target = (self.current_time + delta_seconds).clamp(0.0, self.current_track.duration);
            self.current_time = target;
            if let Some(ref engine) = self.audio_engine {
                engine.seek(target);
            }
        }
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
                        node.sub2 = format!("Rubato Sinc SRC • {:.1} kHz", report.device_rate as f32 / 1000.0);
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
                CableConnection { from_node: 1, to_node: 2 },
                CableConnection { from_node: 2, to_node: 4 },
                CableConnection { from_node: 4, to_node: 5 },
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
                                n.sub2 = "Rubato Sinc SRC • 48.0 kHz".to_string();
                            }
                            ChainMode::OsResampled => {
                                n.sub2 = "Resample to 48.0 kHz".to_string();
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
                    flowing_cables.insert((conn.from_node, conn.to_node));
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
            AnalogPreampEffect, BassGuardEffect, DynamicLoudnessEffect, Effect, LowPassFilterEffect, MixerEffect, OpticalLevelerEffect,
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

        for &(from, to) in &chain_eval.flowing_cables {
            if chain_eval.flowing_nodes.contains(&from) && chain_eval.flowing_nodes.contains(&to) {
                *in_degree.entry(to).or_insert(0) += 1;
                adj.entry(from).or_default().push(to);
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
                        Box::new(StereoWidenerEffect::new(1.35))
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

        // Preserve connection order so Pin A is always parent 0 and Pin B is parent 1
        let mut edges: Vec<(u32, u32)> = Vec::new();
        for conn in &self.connections {
            if chain_eval.flowing_cables.contains(&(conn.from_node, conn.to_node)) {
                edges.push((conn.from_node as u32, conn.to_node as u32));
            }
        }

        crate::audio::SignalChainGraph {
            is_complete: true,
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

        // Synchronize with real Audio Engine
        let mut track_ended = false;
        let mut new_duration = None;
        let mut new_signal_path = None;

        if let Some(ref engine) = self.audio_engine {
            while let Some(event) = engine.try_recv_event() {
                match event {
                    crate::audio::AudioEvent::TrackEnded => {
                        track_ended = true;
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
            if !is_chain_ok {
                engine.set_volume(0.0);
            } else {
                engine.set_volume(if self.is_muted { 0.0 } else { self.volume });
            }
        }

        // Live Real-Time 48-Band FFT Analysis from speaker audio stream
        let is_audio_active = self.is_playing && !self.is_muted && self.volume > 0.001 && is_chain_ok;
        if is_audio_active {
            if let Some(ref engine) = self.audio_engine {
                let mut raw_samples = [0.0f32; 1024];
                engine.get_visualizer_samples(&mut raw_samples);
                self.fft_analyzer.compute_bands(&raw_samples, &mut self.spectrum_bars);
            }
        } else {
            // Smoothly decay to baseline when paused, stopped, muted, or chain severed
            for b in self.spectrum_bars.iter_mut() {
                *b = (*b * 0.80).max(0.0);
            }
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
                    self.current_time = pos;
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
                        ui.selectable_value(&mut self.current_tab, AppTab::Equalizer, egui::RichText::new("🎛️ Equalizer 🚧 Coming Soon").strong());
                        ui.selectable_value(&mut self.current_tab, AppTab::Library, egui::RichText::new("📁 Library").strong());
                        ui.selectable_value(&mut self.current_tab, AppTab::SignalChain, egui::RichText::new("⚡ Signal Chain Studio").strong());
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

                        // Spectrum Analyzer
                        self.render_spectrum_visualizer(ui);
                    }
                    AppTab::SignalChain => {
                        // Full Studio Mode: 3/4 Width Interactive Patchbay Canvas + 1/4 Width FX & Filters Palette
                        let total_width = ui.available_width();
                        let canvas_width = total_width * 0.74;
                        let palette_width = total_width - canvas_width - 12.0;
                        let full_height = ui.available_height() - 8.0;

                        ui.horizontal(|ui| {
                            // Left 3/4 Canvas
                            ui.allocate_ui(Vec2::new(canvas_width, full_height), |ui| {
                                render_card_frame(ui, |ui| {
                                    ui.vertical(|ui| {
                                        // Row 1: Studio Title & Mode Selection + Reset View
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new("⚡ MODULAR PATCHBAY STUDIO")
                                                    .strong()
                                                    .size(14.5)
                                                    .color(Color32::from_rgb(15, 23, 42)),
                                            );

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button(egui::RichText::new("↺ Reset Posisi").size(11.5).strong())
                                                    .on_hover_text("Pusatkan kembali view dan susun rapi 5 blok utama")
                                                    .clicked()
                                                {
                                                    self.reset_patchbay_layout(ui.available_width(), ui.available_height());
                                                }

                                                let current_dev_label = self.selected_output_device.as_deref().unwrap_or("Default System Output");
                                                let truncated_label = if current_dev_label.len() > 22 {
                                                    format!("🔊 {}...", &current_dev_label[..19])
                                                } else {
                                                    format!("🔊 {}", current_dev_label)
                                                };

                                                egui::ComboBox::from_id_salt("patchbay_output_device_combo")
                                                    .selected_text(egui::RichText::new(truncated_label).size(11.0).strong())
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

                                                if ui.selectable_label(self.chain_mode == ChainMode::OsResampled, egui::RichText::new("OS Resampled").size(11.0).strong()).clicked() {
                                                    self.set_chain_mode(ChainMode::OsResampled);
                                                }
                                                if ui.selectable_label(self.chain_mode == ChainMode::DspActive, egui::RichText::new("DSP Active").size(11.0).strong()).clicked() {
                                                    self.set_chain_mode(ChainMode::DspActive);
                                                }
                                                if ui.selectable_label(self.chain_mode == ChainMode::BitPerfect, egui::RichText::new("Bit-Perfect").size(11.0).strong()).clicked() {
                                                    self.set_chain_mode(ChainMode::BitPerfect);
                                                }
                                                ui.label(egui::RichText::new("Mode:").size(11.0).color(Color32::from_rgb(100, 116, 139)));
                                            });
                                        });

                                        ui.add_space(3.0);

                                        // Row 2: Tips & Navigation Guidance (Separated row, never overlaps)
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new("💡 Tarik pin Output (kanan) ke pin Input (kiri) untuk menyambung • Klik kabel untuk memutus • Klik kanan tahan untuk geser canvas")
                                                    .size(11.0)
                                                    .color(Color32::from_rgb(100, 116, 139)),
                                            );
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let tut_text = if self.show_patchbay_tutorial {
                                                    egui::RichText::new("📖 Tutup Tutorial").size(11.0).strong().color(Color32::from_rgb(147, 51, 234))
                                                } else {
                                                    egui::RichText::new("📖 Panduan / Tutorial Pemula").size(11.0).strong().color(Color32::from_rgb(99, 102, 241))
                                                };
                                                if ui.button(tut_text).on_hover_text("Buka modul tutorial bergambar cara kerja Signal Chain Studio").clicked() {
                                                    self.show_patchbay_tutorial = !self.show_patchbay_tutorial;
                                                }
                                            });
                                        });

                                        ui.add_space(6.0);
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
                    AppTab::Library => {
                        self.render_library_panel(ui);
                    }
                    AppTab::Equalizer => {
                        self.render_equalizer_panel(ui);
                    }
                }
            });
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
    fn test_eq_not_in_chain_produces_no_eq_effect_even_with_node3_dsp_engine() {
        let mut app = LunulaApp::default();
        app.chain_mode = ChainMode::DspActive;
        // Connect 1 -> 2 -> 3 -> 4 -> 5 (DSP ENGINE is in path, but NO EQ preset card)
        app.connections = vec![
            CableConnection { from_node: 1, to_node: 2 },
            CableConnection { from_node: 2, to_node: 3 },
            CableConnection { from_node: 3, to_node: 4 },
            CableConnection { from_node: 4, to_node: 5 },
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
            CableConnection { from_node: 1, to_node: 2 },
            CableConnection { from_node: 2, to_node: eq_card_id },
            CableConnection { from_node: eq_card_id, to_node: 4 },
            CableConnection { from_node: 4, to_node: 5 },
        ];
        app.invalidate_chain_cache();

        assert!(app.is_equalizer_in_active_chain());

        let graph = app.compile_signal_chain();
        assert!(graph.is_complete);
        let has_custom_eq = graph.effects.iter().any(|eff| eff.name() == "My Custom EQ");
        assert!(has_custom_eq, "Compiled graph must contain 'My Custom EQ' effect");
    }
}

