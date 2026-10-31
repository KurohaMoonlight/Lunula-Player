use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2};
use std::time::Instant;
use crate::models::{
    AppTab, CableConnection, ChainMode, CustomPlaylist, LibraryCategory,
    ModularNode, NodeType, RepeatMode, ScanMsg, Track, WaveformMode,
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

    // EQ bands (-12.0 to +12.0 dB)
    pub eq_bands: [f32; 10],
    pub eq_bypass: bool,

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
            },
            ModularNode {
                id: 2,
                node_type: NodeType::Buffer,
                title: "RAM BUFFER".to_string(),
                sub1: "RingBuffer".to_string(),
                sub2: "100% Loaded".to_string(),
                pos: Pos2::new(176.0, 48.0),
                is_removable: false,
            },
            ModularNode {
                id: 3,
                node_type: NodeType::Dsp,
                title: "DSP ENGINE".to_string(),
                sub1: "BYPASSED (Direct Wire)".to_string(),
                sub2: "0.0 dB Pure Master".to_string(),
                pos: Pos2::new(332.0, 48.0),
                is_removable: false,
            },
            ModularNode {
                id: 4,
                node_type: NodeType::Driver,
                title: "OUTPUT ENGINE".to_string(),
                sub1: "System Audio Endpoint".to_string(),
                sub2: "Direct Bitstream".to_string(),
                pos: Pos2::new(488.0, 48.0),
                is_removable: false,
            },
            ModularNode {
                id: 5,
                node_type: NodeType::Endpoint,
                title: "OUTPUT DAC".to_string(),
                sub1: "Audio Endpoint".to_string(),
                sub2: "Native Sample Rate".to_string(),
                pos: Pos2::new(644.0, 48.0),
                is_removable: false,
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
                    if report.is_exclusive {
                        node.title = "WASAPI EXCLUSIVE".to_string();
                        node.sub1 = report.device_name.clone();
                        node.sub2 = format!("Direct Bitstream • {:.1} kHz", report.device_rate as f32 / 1000.0);
                    } else if report.is_resampling {
                        node.title = "APP RESAMPLER".to_string();
                        node.sub1 = report.device_name.clone();
                        node.sub2 = format!("Rubato Sinc SRC • {:.1} kHz", report.device_rate as f32 / 1000.0);
                    } else {
                        node.title = "SHARED STREAM".to_string();
                        node.sub1 = report.device_name.clone();
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

        let has_custom_nodes = self.nodes.iter().any(|n| n.is_removable);
        if !has_custom_nodes {
            match mode {
                ChainMode::BitPerfect => {
                    // Option B: Cable jumps directly from 2 (RAM Buffer) to 4 (Output Engine), bypassing 3 (DSP Engine)
                    self.connections = vec![
                        CableConnection { from_node: 1, to_node: 2 },
                        CableConnection { from_node: 2, to_node: 4 },
                        CableConnection { from_node: 4, to_node: 5 },
                    ];
                    self.eq_bypass = true;
                }
                ChainMode::DspActive => {
                    // Cable goes through DSP: 1 -> 2 -> 3 -> 4 -> 5
                    self.connections = vec![
                        CableConnection { from_node: 1, to_node: 2 },
                        CableConnection { from_node: 2, to_node: 3 },
                        CableConnection { from_node: 3, to_node: 4 },
                        CableConnection { from_node: 4, to_node: 5 },
                    ];
                    self.eq_bypass = false;
                }
                ChainMode::OsResampled => {
                    self.connections = vec![
                        CableConnection { from_node: 1, to_node: 2 },
                        CableConnection { from_node: 2, to_node: 4 },
                        CableConnection { from_node: 4, to_node: 5 },
                    ];
                    self.eq_bypass = true;
                }
            }
        } else {
            // When custom modular nodes exist on canvas:
            match mode {
                ChainMode::BitPerfect => {
                    self.eq_bypass = true;
                    // In Bit-Perfect mode, bypass Node 3 if directly wired
                    if self.connections.contains(&CableConnection { from_node: 2, to_node: 3 })
                        && self.connections.contains(&CableConnection { from_node: 3, to_node: 4 })
                    {
                        self.connections.retain(|c| !(c.from_node == 2 && c.to_node == 3) && !(c.from_node == 3 && c.to_node == 4));
                        if !self.connections.contains(&CableConnection { from_node: 2, to_node: 4 }) {
                            self.connections.push(CableConnection { from_node: 2, to_node: 4 });
                        }
                    }
                }
                ChainMode::DspActive => {
                    self.eq_bypass = false;
                    // In DspActive mode, route through Node 3 if currently bypassed
                    if self.connections.contains(&CableConnection { from_node: 2, to_node: 4 }) {
                        self.connections.retain(|c| !(c.from_node == 2 && c.to_node == 4));
                        if !self.connections.contains(&CableConnection { from_node: 2, to_node: 3 }) {
                            self.connections.push(CableConnection { from_node: 2, to_node: 3 });
                        }
                        if !self.connections.contains(&CableConnection { from_node: 3, to_node: 4 }) {
                            self.connections.push(CableConnection { from_node: 3, to_node: 4 });
                        }
                    }
                }
                ChainMode::OsResampled => {
                    self.eq_bypass = true;
                }
            }
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
                            n.sub2 = "10-Band Biquad f64 • Headroom Guard".to_string();
                        } else {
                            n.sub1 = "BYPASSED (Direct Wire)".to_string();
                            n.sub2 = "0.0 dB Pure Master".to_string();
                        }
                    }
                    4 => {
                        match mode {
                            ChainMode::BitPerfect => {
                                n.title = "AUDIO ENGINE".to_string();
                                n.sub1 = dev_name.clone();
                                n.sub2 = "Bit-Perfect Requested".to_string();
                            }
                            ChainMode::DspActive => {
                                n.title = "AUDIO ENGINE".to_string();
                                n.sub1 = dev_name.clone();
                                n.sub2 = "DSP Stream Active".to_string();
                            }
                            ChainMode::OsResampled => {
                                n.title = "APP RESAMPLER".to_string();
                                n.sub1 = dev_name.clone();
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

    pub fn evaluate_signal_chain(&self) -> crate::models::ChainEvaluation {
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
            BassGuardEffect, Effect, GraphicEqEffect, PresetEqEffect, StereoWidenerEffect,
            TubeWarmerEffect, VolumeAttenuatorEffect,
        };

        let chain_eval = self.evaluate_signal_chain();
        if !chain_eval.is_complete {
            return crate::audio::SignalChainGraph {
                is_complete: false,
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

        for &id in &ordered_node_ids {
            if let Some(node) = self.nodes.iter().find(|n| n.id == id) {
                match &node.node_type {
                    NodeType::Dsp => {
                        effects.push(Box::new(GraphicEqEffect::new(self.eq_bands, sample_rate)));
                    }
                    NodeType::TubeWarmer => {
                        effects.push(Box::new(TubeWarmerEffect::new()));
                    }
                    NodeType::StereoWidener => {
                        effects.push(Box::new(StereoWidenerEffect::new(1.35)));
                    }
                    NodeType::BassGuard => {
                        effects.push(Box::new(BassGuardEffect::new(sample_rate)));
                    }
                    NodeType::VolumeAttenuator => {
                        effects.push(Box::new(VolumeAttenuatorEffect::new(-6.02)));
                    }
                    NodeType::SavedEqPreset(name) => {
                        effects.push(Box::new(PresetEqEffect::from_preset(name, sample_rate)));
                    }
                    _ => {}
                }
            }
        }

        crate::audio::SignalChainGraph {
            is_complete: true,
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
                        ui.selectable_value(&mut self.current_tab, AppTab::Equalizer, egui::RichText::new("🎛️ Equalizer").strong());
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
                                                egui::RichText::new("💡 Tarik pin Output (kanan) ke pin Input (kiri) untuk menyambung • Klik kabel untuk memutus • Klik kanan tahan (hold) untuk geser canvas")
                                                    .size(11.0)
                                                    .color(Color32::from_rgb(100, 116, 139)),
                                            );
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

