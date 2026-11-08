use eframe::egui::Pos2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FilterType {
    Peaking,
    LowShelf,
    HighShelf,
    HighPass,
    LowPass,
    Notch,
}

impl FilterType {
    pub fn name(&self) -> &'static str {
        match self {
            FilterType::Peaking => "Peaking (Bell)",
            FilterType::LowShelf => "Low Shelf",
            FilterType::HighShelf => "High Shelf",
            FilterType::HighPass => "High-Pass (HPF)",
            FilterType::LowPass => "Low-Pass (LPF)",
            FilterType::Notch => "Notch (Band-Stop)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            FilterType::Peaking => "Peak",
            FilterType::LowShelf => "L-Shelf",
            FilterType::HighShelf => "H-Shelf",
            FilterType::HighPass => "HPF",
            FilterType::LowPass => "LPF",
            FilterType::Notch => "Notch",
        }
    }

    pub fn has_gain(&self) -> bool {
        matches!(self, FilterType::Peaking | FilterType::LowShelf | FilterType::HighShelf)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum FilterSlope {
    #[default]
    Db12,
    Db24,
    Db48,
}

impl FilterSlope {
    pub fn name(&self) -> &'static str {
        match self {
            FilterSlope::Db12 => "12 dB/oct",
            FilterSlope::Db24 => "24 dB/oct",
            FilterSlope::Db48 => "48 dB/oct",
        }
    }

    pub fn biquad_count(&self) -> usize {
        match self {
            FilterSlope::Db12 => 1,
            FilterSlope::Db24 => 2,
            FilterSlope::Db48 => 4,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParametricBand {
    pub enabled: bool,
    pub filter_type: FilterType,
    pub freq: f32,       // 20.0 .. 20000.0 Hz
    pub gain_db: f32,    // -18.0 .. 18.0 dB
    pub q: f32,          // 0.1 .. 10.0
    #[serde(default)]
    pub slope: FilterSlope,
}

impl Default for ParametricBand {
    fn default() -> Self {
        Self {
            enabled: true,
            filter_type: FilterType::Peaking,
            freq: 1000.0,
            gain_db: 0.0,
            q: 1.414,
            slope: FilterSlope::Db12,
        }
    }
}

impl ParametricBand {
    pub fn new(filter_type: FilterType, freq: f32, gain_db: f32, q: f32) -> Self {
        Self {
            enabled: true,
            filter_type,
            freq,
            gain_db,
            q,
            slope: FilterSlope::Db12,
        }
    }

    pub fn new_cut(filter_type: FilterType, freq: f32, q: f32, slope: FilterSlope) -> Self {
        Self {
            enabled: true,
            filter_type,
            freq,
            gain_db: 0.0,
            q,
            slope,
        }
    }

    pub fn default_10_bands() -> Vec<Self> {
        vec![
            Self::new(FilterType::LowShelf, 31.0, 0.0, 0.707),
            Self::new(FilterType::Peaking, 63.0, 0.0, 1.414),
            Self::new(FilterType::Peaking, 125.0, 0.0, 1.414),
            Self::new(FilterType::Peaking, 250.0, 0.0, 1.414),
            Self::new(FilterType::Peaking, 500.0, 0.0, 1.414),
            Self::new(FilterType::Peaking, 1000.0, 0.0, 1.414),
            Self::new(FilterType::Peaking, 2000.0, 0.0, 1.414),
            Self::new(FilterType::Peaking, 4000.0, 0.0, 1.414),
            Self::new(FilterType::Peaking, 8000.0, 0.0, 1.414),
            Self::new(FilterType::HighShelf, 16000.0, 0.0, 0.707),
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum PreampMode {
    #[default]
    Auto,
    Manual,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum DitherMode {
    #[default]
    None,
    Tpdf,
    NoiseShaped,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParametricEqPreset {
    pub name: String,
    pub bands: Vec<ParametricBand>,
    pub preamp_mode: PreampMode,
    pub manual_preamp_db: f32,
}

impl ParametricEqPreset {
    pub fn default_presets() -> Vec<Self> {
        vec![
            Self {
                name: "Flat (Netral)".to_string(),
                bands: ParametricBand::default_10_bands(),
                preamp_mode: PreampMode::Auto,
                manual_preamp_db: 0.0,
            },
            Self {
                name: "Bass Boost & Warmth".to_string(),
                bands: vec![
                    ParametricBand::new(FilterType::LowShelf, 45.0, 5.5, 0.707),
                    ParametricBand::new(FilterType::Peaking, 90.0, 3.5, 1.2),
                    ParametricBand::new(FilterType::Peaking, 200.0, 1.5, 1.414),
                    ParametricBand::new(FilterType::Peaking, 500.0, 0.0, 1.414),
                    ParametricBand::new(FilterType::Peaking, 1000.0, 0.0, 1.414),
                    ParametricBand::new(FilterType::Peaking, 2000.0, 0.0, 1.414),
                    ParametricBand::new(FilterType::Peaking, 4000.0, 0.5, 1.414),
                    ParametricBand::new(FilterType::Peaking, 8000.0, 1.0, 1.414),
                    ParametricBand::new(FilterType::HighShelf, 14000.0, 1.5, 0.707),
                ],
                preamp_mode: PreampMode::Auto,
                manual_preamp_db: -5.5,
            },
            Self {
                name: "Harman-style Tilt (Generik)".to_string(),
                bands: vec![
                    ParametricBand::new(FilterType::LowShelf, 105.0, 4.5, 0.707),
                    ParametricBand::new(FilterType::Peaking, 200.0, 0.0, 1.414),
                    ParametricBand::new(FilterType::Peaking, 500.0, -0.5, 1.414),
                    ParametricBand::new(FilterType::Peaking, 1000.0, -0.5, 1.8),
                    ParametricBand::new(FilterType::Peaking, 3000.0, 2.5, 1.414),
                    ParametricBand::new(FilterType::Peaking, 6000.0, -1.5, 2.0),
                    ParametricBand::new(FilterType::HighShelf, 10000.0, 1.0, 0.707),
                ],
                preamp_mode: PreampMode::Auto,
                manual_preamp_db: -4.5,
            },
            Self {
                name: "Vocal Clarity".to_string(),
                bands: vec![
                    ParametricBand::new_cut(FilterType::HighPass, 75.0, 0.707, FilterSlope::Db12),
                    ParametricBand::new(FilterType::Peaking, 250.0, -2.5, 1.8),
                    ParametricBand::new(FilterType::Peaking, 1000.0, 1.0, 1.414),
                    ParametricBand::new(FilterType::Peaking, 2500.0, 3.5, 1.414),
                    ParametricBand::new(FilterType::Peaking, 5000.0, 2.0, 1.6),
                    ParametricBand::new(FilterType::HighShelf, 10000.0, 1.5, 0.707),
                ],
                preamp_mode: PreampMode::Auto,
                manual_preamp_db: -3.5,
            },
            Self {
                name: "Warm Vinyl / Acoustic".to_string(),
                bands: vec![
                    ParametricBand::new(FilterType::LowShelf, 80.0, 2.0, 0.707),
                    ParametricBand::new(FilterType::Peaking, 160.0, 1.5, 1.2),
                    ParametricBand::new(FilterType::Peaking, 1000.0, 0.5, 1.4),
                    ParametricBand::new(FilterType::Peaking, 8000.0, -1.0, 1.5),
                    ParametricBand::new(FilterType::HighShelf, 14000.0, -1.5, 0.707),
                ],
                preamp_mode: PreampMode::Auto,
                manual_preamp_db: -2.0,
            },
            Self {
                name: "Air & Sparkle".to_string(),
                bands: vec![
                    ParametricBand::new(FilterType::LowShelf, 50.0, 0.5, 0.707),
                    ParametricBand::new(FilterType::Peaking, 8000.0, 2.0, 1.5),
                    ParametricBand::new(FilterType::HighShelf, 12000.0, 4.5, 0.707),
                ],
                preamp_mode: PreampMode::Auto,
                manual_preamp_db: -4.5,
            },
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppTab {
    NowPlaying,
    SignalChain,
    Library,
    Equalizer,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum OnboardingQuestStep {
    #[default]
    Welcome,           // Misi 1: Sambutan & Kenalkan Studio
    NavigateToStudio,  // Misi 2: Buka Tab Signal Chain Studio
    UnderstandModes,   // Misi 3: Filosofi Bit-Perfect vs DSP Active
    ConnectFirstCable, // Misi 4: Sambungkan kabel Buffer -> DAC
    TestAutoMute,      // Misi 5: Putuskan kabel untuk merasakan Auto-Mute
    InsertTubeWarmer,  // Misi 6: Pasang efek Tube Warmer dari sidebar
    Completed,         // Misi 7: Selesai! Quest Complete Certificate
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum StudioMode {
    #[default]
    Simple,
    Advance,
}

pub use crate::audio::{ChainMode, ResamplerSettings, RollOffProfile, SincQuality};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
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
    LowPassFilter,
    HighPassFilter,
    BandPassFilter,
    Crossover,
    Mixer,
    Splitter,
    // Volume Manipulation Suite
    VolumeBooster,
    AnalogPreamp,
    DynamicLoudness,
    SteppedAttenuator,
    OpticalLeveler,
    StereoBalance,
    Compressor,
}

impl NodeType {
    pub fn is_dsp_or_effect(&self) -> bool {
        matches!(
            self,
            NodeType::Dsp
                | NodeType::TubeWarmer
                | NodeType::StereoWidener
                | NodeType::BassGuard
                | NodeType::VolumeAttenuator
                | NodeType::SavedEqPreset(_)
                | NodeType::LowPassFilter
                | NodeType::HighPassFilter
                | NodeType::BandPassFilter
                | NodeType::Crossover
                | NodeType::Mixer
                | NodeType::Splitter
                | NodeType::VolumeBooster
                | NodeType::AnalogPreamp
                | NodeType::DynamicLoudness
                | NodeType::SteppedAttenuator
                | NodeType::OpticalLeveler
                | NodeType::StereoBalance
                | NodeType::Compressor
        )
    }
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
    pub param_val: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CableConnection {
    pub from_node: usize,
    #[serde(default)]
    pub from_port: u8,
    pub to_node: usize,
    #[serde(default)]
    pub to_port: u8,
}

impl CableConnection {
    pub fn new(from_node: usize, to_node: usize) -> Self {
        Self {
            from_node,
            from_port: 0,
            to_node,
            to_port: 0,
        }
    }

    pub fn with_ports(from_node: usize, from_port: u8, to_node: usize, to_port: u8) -> Self {
        Self {
            from_node,
            from_port,
            to_node,
            to_port,
        }
    }
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
    pub flowing_cables: std::collections::HashSet<CableConnection>,
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ReplayGainData {
    pub track_gain_db: Option<f32>,
    pub track_peak: Option<f32>,
    pub album_gain_db: Option<f32>,
    pub album_peak: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReplayGainMode {
    #[default]
    Auto,      // Track gain when shuffling / playlist, Album gain when playing albums
    Track,     // Always Track gain
    Album,     // Always Album gain
    Disabled,  // Off / bypass
}

impl ReplayGainMode {
    pub fn name(&self) -> &'static str {
        match self {
            ReplayGainMode::Auto => "Auto (Smart)",
            ReplayGainMode::Track => "Track Gain",
            ReplayGainMode::Album => "Album Gain",
            ReplayGainMode::Disabled => "Disabled (Off)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReplayGainClipPrevention {
    #[default]
    PreventByPeak, // Automatically scale down boost to avoid exceeding 0 dBFS
    Limiter,        // Use fast transparent soft-limiter
    None,           // No prevention
}

impl ReplayGainClipPrevention {
    pub fn name(&self) -> &'static str {
        match self {
            ReplayGainClipPrevention::PreventByPeak => "Prevent by Peak",
            ReplayGainClipPrevention::Limiter => "Apply Limiter",
            ReplayGainClipPrevention::None => "None (Unprotected)",
        }
    }
}

#[derive(Clone, Debug, Default)]
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
    pub replay_gain: ReplayGainData,
    pub cue_offset_sec: Option<f32>,
    pub cue_duration_sec: Option<f32>,
    pub genre: Option<String>,
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
    #[serde(default)]
    pub replay_gain: Option<ReplayGainData>,
    #[serde(default)]
    pub cue_offset_sec: Option<f32>,
    #[serde(default)]
    pub cue_duration_sec: Option<f32>,
    #[serde(default)]
    pub genre: Option<String>,
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
    #[serde(default)]
    pub selected_output_device: Option<String>,
    #[serde(default)]
    pub parametric_bands: Option<Vec<ParametricBand>>,
    #[serde(default)]
    pub eq_preamp_mode: Option<PreampMode>,
    #[serde(default)]
    pub eq_manual_preamp_db: Option<f32>,
    #[serde(default)]
    pub eq_current_preset_name: Option<String>,
    #[serde(default)]
    pub eq_limiter_enabled: Option<bool>,
    #[serde(default)]
    pub eq_dither_mode: Option<DitherMode>,
    #[serde(default)]
    pub has_completed_onboarding: Option<bool>,
    #[serde(default)]
    pub resampler_quality: Option<SincQuality>,
    #[serde(default)]
    pub resampler_profile: Option<RollOffProfile>,
    #[serde(default)]
    pub replay_gain_mode: Option<ReplayGainMode>,
    #[serde(default)]
    pub replay_gain_preamp_db: Option<f32>,
    #[serde(default)]
    pub replay_gain_clip_prevention: Option<ReplayGainClipPrevention>,
    #[serde(default)]
    pub crossfade_secs: Option<f32>,
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
            replay_gain: data.replay_gain.unwrap_or_default(),
            cue_offset_sec: data.cue_offset_sec,
            cue_duration_sec: data.cue_duration_sec,
            genre: data.genre,
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
            replay_gain: Some(track.replay_gain),
            cue_offset_sec: track.cue_offset_sec,
            cue_duration_sec: track.cue_duration_sec,
            genre: track.genre.clone(),
        }
    }
}

pub enum ScanMsg {
    Started(String),
    TrackFound(Track),
    Finished(usize),
}
