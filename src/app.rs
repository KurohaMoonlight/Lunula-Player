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

        app
    }

    pub fn play_next_track(&mut self) {
        if !self.play_queue.is_empty() {
            self.current_track = self.play_queue.remove(0);
            self.current_time = 0.0;
            self.is_playing = true;
            self.save_library_state();
        } else if !self.playlist.is_empty() {
            let cur_idx = self.playlist.iter().position(|t| {
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            if let Some(idx) = cur_idx {
                let next_idx = (idx + 1) % self.playlist.len();
                self.current_track = self.playlist[next_idx].clone();
                self.current_time = 0.0;
            } else if let Some(first) = self.playlist.first() {
                self.current_track = first.clone();
                self.current_time = 0.0;
            }
        } else {
            self.current_time = 0.0;
        }
    }

    pub fn play_prev_track(&mut self) {
        if self.current_time > 3.0 {
            self.current_time = 0.0;
            return;
        }
        if !self.playlist.is_empty() {
            let cur_idx = self.playlist.iter().position(|t| {
                t.title == self.current_track.title && t.artist == self.current_track.artist
            });
            if let Some(idx) = cur_idx {
                let prev_idx = if idx == 0 { self.playlist.len() - 1 } else { idx - 1 };
                self.current_track = self.playlist[prev_idx].clone();
                self.current_time = 0.0;
            }
        } else {
            self.current_time = 0.0;
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

        if self.is_playing {
            ctx.request_repaint();
            self.visualizer_anim_phase += 0.025;
            self.current_time += 1.0 / 60.0;
            if self.current_track.duration > 0.0 && self.current_time >= self.current_track.duration {
                self.play_next_track();
            }
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

