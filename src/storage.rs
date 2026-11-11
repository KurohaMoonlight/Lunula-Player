use std::path::PathBuf;
use crate::models::SavedAppState;
#[cfg(not(test))]
use crate::models::TrackData;
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
        #[cfg(test)]
        {
            // In unit tests, avoid polluting the host machine's APPDATA directory
            return;
        }

        #[cfg(not(test))]
        {
            let saved_state = SavedAppState {
                playlist: self.playlist.iter().map(TrackData::from).collect(),
                custom_playlists: self.custom_playlists.clone(),
                prefer_exclusive_mode: self.prefer_exclusive,
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
                parametric_bands: Some(self.active_eq.bands.clone()),
                eq_preamp_mode: Some(self.active_eq.preamp_mode),
                eq_manual_preamp_db: Some(self.active_eq.manual_preamp_db),
                eq_current_preset_name: Some(self.active_eq.name.clone()),
                eq_limiter_enabled: Some(self.eq_limiter_enabled),
                eq_dither_mode: Some(self.eq_dither_mode),
                resampler_quality: Some(self.resampler_settings.quality),
                resampler_profile: Some(self.resampler_settings.profile),
                replay_gain_mode: Some(self.replay_gain_mode),
                replay_gain_preamp_db: Some(self.replay_gain_preamp_db),
                replay_gain_clip_prevention: Some(self.replay_gain_clip_prevention),
                crossfade_secs: Some(self.crossfade_secs),
                clipping_detection_enabled: Some(self.clipping_detection_enabled),
                clipping_threshold_db: Some(self.clipping_threshold_db),
            };
            if let Ok(json) = serde_json::to_string_pretty(&saved_state) {
                let path = get_storage_path();
                let _ = std::fs::write(path, json);
            }
        }
    }

    pub fn load_library_state() -> Option<SavedAppState> {
        #[cfg(test)]
        {
            // Hermetic unit tests: never load host state from disk
            return None;
        }

        #[cfg(not(test))]
        {
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
}

