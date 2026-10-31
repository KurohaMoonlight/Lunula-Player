use eframe::egui::Pos2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppTab {
    NowPlaying,
    SignalChain,
    Library,
    Equalizer,
}

pub use crate::audio::ChainMode;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NodeType {
    Source,
    Buffer,
    Dsp,
    Driver,
    Endpoint,
    // Modular Filters / FX
    TubeWarmer,
    StereoWidener,
    BassGuard,
    VolumeAttenuator,
    SavedEqPreset(String),
}

#[derive(Clone, Debug)]
pub struct ModularNode {
    pub id: usize,
    pub node_type: NodeType,
    pub title: String,
    pub sub1: String,
    pub sub2: String,
    pub pos: Pos2,
    pub is_removable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CableConnection {
    pub from_node: usize,
    pub to_node: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SignalNodeStatus {
    Flowing,   // Green: audio signal actively passing through
    Broken,    // Red: signal severed or unreachable
}

#[derive(Clone, Debug)]
pub struct ChainEvaluation {
    pub is_complete: bool,
    pub reachable_from_source: std::collections::HashSet<usize>,
    pub flowing_nodes: std::collections::HashSet<usize>,
    pub flowing_cables: std::collections::HashSet<(usize, usize)>,
}

impl ChainEvaluation {
    pub fn node_status(&self, node_id: usize) -> SignalNodeStatus {
        if self.is_complete && self.flowing_nodes.contains(&node_id) {
            SignalNodeStatus::Flowing
        } else if !self.is_complete && self.reachable_from_source.contains(&node_id) {
            SignalNodeStatus::Flowing
        } else {
            SignalNodeStatus::Broken
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum WaveformMode {
    StackedMultiBand, // Opsi A: 3 Layer Bertumpuk (Bass, Mid, Treble)
    SpectralBlend,    // Opsi B: Serato Spectral Color Blending
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum RepeatMode {
    #[default]
    Off,      // Tidak repeat (mati setelah selesai antrean/playlist)
    All,      // Ulangi semua lagu (loop playlist/queue)
    One,      // Ulangi satu lagu ini saja
    Album,    // Ulangi hanya lagu-lagu di album ini
}

#[derive(Clone, Debug)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f32,
    pub format: String,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub waveform: Vec<f32>,
    pub waveform_bass: Vec<f32>,
    pub waveform_mid: Vec<f32>,
    pub waveform_treble: Vec<f32>,
    pub file_path: Option<String>,
    pub file_size_mb: f32,
    pub bitrate_kbps: Option<u32>,
    pub is_favorite: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LibraryCategory {
    AllTracks,
    Albums,
    Artists,
    Quality,
    Favorites,
    Playlists,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomPlaylist {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub track_paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackData {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f32,
    pub format: String,
    pub sample_rate: u32,
    pub bit_depth: u8,
    pub file_path: Option<String>,
    pub file_size_mb: f32,
    pub bitrate_kbps: Option<u32>,
    pub is_favorite: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedAppState {
    pub playlist: Vec<TrackData>,
    pub custom_playlists: Vec<CustomPlaylist>,
    pub current_track: Option<TrackData>,
    #[serde(default)]
    pub play_queue: Vec<TrackData>,
    #[serde(default)]
    pub waveform_mode: Option<WaveformMode>,
    pub volume: f32,
    pub eq_bands: [f32; 10],
    pub eq_bypass: bool,
    #[serde(default)]
    pub chain_mode: ChainMode,
    #[serde(default)]
    pub is_shuffled: bool,
    #[serde(default)]
    pub repeat_mode: RepeatMode,
    #[serde(default)]
    pub queue_cursor: Option<usize>,
}

impl From<TrackData> for Track {
    fn from(data: TrackData) -> Self {
        let (waveform, waveform_bass, waveform_mid, waveform_treble) =
            crate::scanner::generate_multiband_waveform(&data.title, 3600);
        Track {
            title: data.title,
            artist: data.artist,
            album: data.album,
            duration: data.duration,
            format: data.format,
            sample_rate: data.sample_rate,
            bit_depth: data.bit_depth,
            waveform,
            waveform_bass,
            waveform_mid,
            waveform_treble,
            file_path: data.file_path,
            file_size_mb: data.file_size_mb,
            bitrate_kbps: data.bitrate_kbps,
            is_favorite: data.is_favorite,
        }
    }
}

impl From<&Track> for TrackData {
    fn from(track: &Track) -> Self {
        TrackData {
            title: track.title.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            duration: track.duration,
            format: track.format.clone(),
            sample_rate: track.sample_rate,
            bit_depth: track.bit_depth,
            file_path: track.file_path.clone(),
            file_size_mb: track.file_size_mb,
            bitrate_kbps: track.bitrate_kbps,
            is_favorite: track.is_favorite,
        }
    }
}

pub enum ScanMsg {
    Started(String),
    TrackFound(Track),
    Finished(usize),
}
