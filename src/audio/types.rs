use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Audio signal chain processing mode.
/// Determines how samples flow through resampler, DSP, and volume stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainMode {
    /// No DSP, no volume scaling, no clamp. Resampler passthrough (device rate = track rate).
    BitPerfect,
    /// Full DSP pipeline active (10-band EQ + FX). Software volume applied.
    DspActive,
    /// Force device output to 48 kHz (emulate Windows Shared Audio). DSP bypassed, software volume applied.
    OsResampled,
}

impl Default for ChainMode {
    fn default() -> Self {
        ChainMode::BitPerfect
    }
}

/// Trait for modular audio DSP effects that can be inserted into the signal graph.
pub trait Effect: Send + std::fmt::Debug {
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64);
    fn reset(&mut self);
    fn set_sample_rate(&mut self, sample_rate: f64);
    fn name(&self) -> &str;
    fn set_param(&mut self, _index: usize, _val: f64) {}
    fn clone_box(&self) -> Box<dyn Effect>;
}

impl Clone for Box<dyn Effect> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Compiled modular signal chain graph.
/// Contains the topological execution order of effects and connection completeness status.
pub struct SignalChainGraph {
    pub is_complete: bool,
    pub effects: Vec<Box<dyn Effect>>,
}

impl Clone for SignalChainGraph {
    fn clone(&self) -> Self {
        Self {
            is_complete: self.is_complete,
            effects: self.effects.clone(),
        }
    }
}

impl std::fmt::Debug for SignalChainGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.effects.iter().map(|e| e.name()).collect();
        f.debug_struct("SignalChainGraph")
            .field("is_complete", &self.is_complete)
            .field("effects", &names)
            .finish()
    }
}

impl Default for SignalChainGraph {
    fn default() -> Self {
        Self {
            is_complete: true,
            effects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Play(PathBuf),
    Pause,
    Resume,
    Stop,
    Seek(f32),
    SetVolume(f32),
    SetEqBand { index: usize, gain_db: f32 },
    SetEqBypass(bool),
    /// Atomically switch the entire signal chain mode.
    SetChainMode(ChainMode),
    /// Select specific audio hardware output device by friendly name (None = default OS endpoint)
    SetOutputDevice(Option<String>),
    /// Dynamic modular signal chain graph compiled from patchbay canvas
    SetSignalChainGraph(SignalChainGraph),
}

/// Report of the actual signal path configuration determined by the engine.
/// Sent after every Play and SetChainMode to let the UI display truthful info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalPathReport {
    pub device_name: String,
    pub source_rate: u32,
    pub device_rate: u32,
    pub source_bit_depth: u8,
    pub format_desc: String,
    pub requested_mode: ChainMode,
    pub actual_mode: ChainMode,
    pub is_exclusive: bool,
    pub is_resampling: bool,
    pub is_dsp_active: bool,
    pub is_volume_applied: bool,
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    TrackStarted {
        duration: f32,
        sample_rate: u32,
        channels: u16,
    },
    Position(f32),
    StateChanged(PlaybackState),
    TrackEnded,
    Error(String),
    /// Engine reports the actual signal path after Play or mode change.
    SignalPath(SignalPathReport),
}

