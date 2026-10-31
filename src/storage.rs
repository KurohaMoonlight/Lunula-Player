use std::path::PathBuf;
use crate::models::{SavedAppState, TrackData};
use crate::app::LunulaApp;

pub fn get_storage_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("LunulaPlayer");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("library_cache.json");
    }
    PathBuf::from("lunula_library_cache.json")
}

impl LunulaApp {
    pub fn save_library_state(&self) {
        let saved_state = SavedAppState {
            playlist: self.playlist.iter().map(TrackData::from).collect(),
            custom_playlists: self.custom_playlists.clone(),
            current_track: if self.current_track.duration > 0.0 {
                Some(TrackData::from(&self.current_track))
            } else {
                None
            },
            play_queue: self.play_queue.iter().map(TrackData::from).collect(),
            waveform_mode: Some(self.waveform_mode),
            volume: self.volume,
            eq_bands: self.eq_bands,
            eq_bypass: self.eq_bypass,
            chain_mode: self.chain_mode,
            is_shuffled: self.is_shuffled,
            repeat_mode: self.repeat_mode,
            queue_cursor: self.queue_cursor,
            selected_output_device: self.selected_output_device.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&saved_state) {
            let path = get_storage_path();
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load_library_state() -> Option<SavedAppState> {
        let path = get_storage_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str::<SavedAppState>(&content) {
                    return Some(state);
                }
            }
        }
        None
    }
}

