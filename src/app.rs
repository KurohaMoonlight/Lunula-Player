use eframe::egui;
use egui::{Color32, Pos2, Stroke, Vec2};
use std::time::Instant;
use crate::models::{
    AppTab, CableConnection, ChainMode, CustomPlaylist, LibraryCategory,
    ModularNode, NodeType, ScanMsg, Track, WaveformMode,
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
    pub fft_analyzer: crate::audio::FftAnalyzer,
    pub spectrum_bars: [f32; 48],
    pub current_cover_bytes: Option<Vec<u8>>,
    pub current_cover_texture: Option<eframe::egui::TextureHandle>,
    pub album_cover_cache: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Option<eframe::egui::TextureHandle>>>>,
}


impl LunulaApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
                sub1: "Bypassed".to_string(),
                sub2: "32-bit Float".to_string(),
                pos: Pos2::new(332.0, 48.0),
                is_removable: false,
            },
            ModularNode {
                id: 4,
                node_type: NodeType::Driver,
                title: "WASAPI ENGINE".to_string(),
                sub1: "Exclusive Mode".to_string(),
                sub2: "3.2 ms Latency".to_string(),
                pos: Pos2::new(488.0, 48.0),
                is_removable: false,
            },
            ModularNode {
                id: 5,
                node_type: NodeType::Endpoint,
                title: "HARDWARE DAC".to_string(),
                sub1: "USB Audio 2.0".to_string(),
                sub2: "Direct Bitstream".to_string(),
                pos: Pos2::new(644.0, 48.0),
                is_removable: false,
            },
        ];

        // Default chain connections: 1 -> 2 -> 3 -> 4 -> 5
        let initial_connections = vec![
            CableConnection { from_node: 1, to_node: 2 },
            CableConnection { from_node: 2, to_node: 3 },
            CableConnection { from_node: 3, to_node: 4 },
            CableConnection { from_node: 4, to_node: 5 },
        ];

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
            fft_analyzer: crate::audio::FftAnalyzer::new(48000.0),
            spectrum_bars: [0.0; 48],
            current_cover_bytes: None,
            current_cover_texture: None,
            album_cover_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
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
            engine.set_volume(if app.is_muted { 0.0 } else { app.volume });
            engine.set_eq_bypass(app.eq_bypass);
            for (idx, &gain) in app.eq_bands.iter().enumerate() {
                engine.set_eq_band(idx, gain);
            }
        }

        app
    }

    pub fn sync_eq_to_engine(&self) {
        if let Some(ref engine) = self.audio_engine {
            engine.set_eq_bypass(self.eq_bypass);
            for (idx, &gain) in self.eq_bands.iter().enumerate() {
                engine.set_eq_band(idx, gain);
            }
        }
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
                engine.play(std::path::PathBuf::from(path_str));
                engine.set_volume(if self.is_muted { 0.0 } else { self.volume });
                engine.set_eq_bypass(self.eq_bypass);
                for (idx, &gain) in self.eq_bands.iter().enumerate() {
                    engine.set_eq_band(idx, gain);
                }
            }
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

    pub fn play_next_track(&mut self) {
        if !self.play_queue.is_empty() {
            let next = self.play_queue.remove(0);
            self.play_track(next);
        } else if !self.playlist.is_empty() {
            let cur_idx = self.playlist.iter().position(|t| {
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            if let Some(idx) = cur_idx {
                let next_idx = (idx + 1) % self.playlist.len();
                let next = self.playlist[next_idx].clone();
                self.play_track(next);
            } else if let Some(first) = self.playlist.first() {
                let first = first.clone();
                self.play_track(first);
            }
        } else {
            self.current_time = 0.0;
        }
    }

    pub fn play_prev_track(&mut self) {
        if self.current_time > 3.0 {
            self.current_time = 0.0;
            if let Some(ref engine) = self.audio_engine {
                engine.seek(0.0);
            }
            return;
        }
        if !self.playlist.is_empty() {
            let cur_idx = self.playlist.iter().position(|t| {
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            if let Some(idx) = cur_idx {
                let prev_idx = if idx == 0 { self.playlist.len() - 1 } else { idx - 1 };
                let prev = self.playlist[prev_idx].clone();
                self.play_track(prev);
            }
        } else {
            self.current_time = 0.0;
        }
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
            engine.set_volume(self.volume);
        }
        self.save_library_state();
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
        if let Some(ref engine) = self.audio_engine {
            engine.set_volume(if self.is_muted { 0.0 } else { self.volume });
        }
    }

    pub fn toggle_eq_bypass(&mut self) {
        self.eq_bypass = !self.eq_bypass;
        if let Some(ref engine) = self.audio_engine {
            engine.set_eq_bypass(self.eq_bypass);
        }
        self.save_library_state();
    }

    pub fn get_album_cover_texture(
        &self,
        ctx: &eframe::egui::Context,
        key: &str,
        file_path: Option<&str>,
    ) -> Option<eframe::egui::TextureHandle> {
        if let Ok(mut cache) = self.album_cover_cache.lock() {
            if let Some(cached) = cache.get(key) {
                return cached.clone();
            }
            let tex = file_path.and_then(|p| {
                crate::scanner::extract_album_art(p).and_then(|bytes| {
                    crate::scanner::load_image_to_texture(ctx, &format!("art_{}", key), &bytes)
                })
            });
            cache.insert(key.to_string(), tex.clone());
            tex
        } else {
            None
        }
    }
}

impl eframe::App for LunulaApp {
    fn on_exit(&mut self) {
        self.save_library_state();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

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
                    _ => {}
                }
            }
        }

        if let Some(d) = new_duration {
            self.current_track.duration = d;
        }

        if track_ended {
            self.play_next_track();
        }

        // Live Real-Time 48-Band FFT Analysis from speaker audio stream
        let is_audio_active = self.is_playing && !self.is_muted && self.volume > 0.001;
        if is_audio_active {
            if let Some(ref engine) = self.audio_engine {
                let mut raw_samples = [0.0f32; 1024];
                engine.get_visualizer_samples(&mut raw_samples);
                self.fft_analyzer.compute_bands(&raw_samples, &mut self.spectrum_bars);
            }
        } else {
            // Smoothly decay to baseline when paused, stopped, or muted
            for b in self.spectrum_bars.iter_mut() {
                *b = (*b * 0.80).max(0.0);
            }
        }

        if self.is_playing || self.spectrum_bars.iter().any(|&v| v > 0.005) {
            ctx.request_repaint();
            self.visualizer_anim_phase += 0.025;

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
                    self.play_next_track();
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
                    self.set_app_volume(self.volume + 0.05);
                }
                if i.key_pressed(egui::Key::ArrowDown) {
                    self.set_app_volume(self.volume - 0.05);
                }
                if i.key_pressed(egui::Key::M) {
                    self.toggle_mute();
                }
                if i.key_pressed(egui::Key::B) {
                    self.toggle_eq_bypass();
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
                                    ui.label(egui::RichText::new("• Buka tab 'Signal Chain Studio' untuk menghubungkan filter modular").size(11.5).color(Color32::from_rgb(148, 163, 184)));
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

                        ui.add_space(10.0);

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

                                                ui.selectable_value(&mut self.chain_mode, ChainMode::OsResampled, egui::RichText::new("OS Resampled").size(11.0).strong());
                                                ui.selectable_value(&mut self.chain_mode, ChainMode::DspActive, egui::RichText::new("DSP Active").size(11.0).strong());
                                                ui.selectable_value(&mut self.chain_mode, ChainMode::BitPerfect, egui::RichText::new("Bit-Perfect").size(11.0).strong());
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

