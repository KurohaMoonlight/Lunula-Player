use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
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
    SetForceResample48k(bool),
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
}

