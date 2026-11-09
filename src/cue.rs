use std::path::{Path, PathBuf};
use lofty::prelude::*;
use crate::models::{ReplayGainData, Track};

#[derive(Debug, Clone, PartialEq)]
pub struct RawCueTrack {
    pub number: u32,
    pub title: String,
    pub performer: Option<String>,
    pub index_01_sec: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CueSheet {
    pub album_title: Option<String>,
    pub album_performer: Option<String>,
    pub audio_file: Option<String>,
    pub tracks: Vec<RawCueTrack>,
}

impl CueSheet {
    /// Parse standard CUE sheet text content into structured CueSheet metadata.
    pub fn parse(content: &str) -> Self {
        let mut sheet = CueSheet::default();
        let mut current_track: Option<RawCueTrack> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("REM ") {
                continue;
            }

            let upper = trimmed.to_ascii_uppercase();

            if upper.starts_with("FILE ") {
                // E.g. FILE "Album.flac" WAVE or FILE Album.wav WAVE
                if let Some(first_quote) = trimmed.find('"') {
                    if let Some(second_quote) = trimmed[first_quote + 1..].find('"') {
                        let filename = &trimmed[first_quote + 1..first_quote + 1 + second_quote];
                        sheet.audio_file = Some(filename.to_string());
                    }
                } else {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        sheet.audio_file = Some(parts[1].to_string());
                    }
                }
            } else if upper.starts_with("PERFORMER ") {
                let performer = extract_quoted_or_rest(trimmed, "PERFORMER");
                if current_track.is_none() {
                    sheet.album_performer = Some(performer);
                } else if let Some(ref mut trk) = current_track {
                    trk.performer = Some(performer);
                }
            } else if upper.starts_with("TITLE ") {
                let title = extract_quoted_or_rest(trimmed, "TITLE");
                if current_track.is_none() {
                    sheet.album_title = Some(title);
                } else if let Some(ref mut trk) = current_track {
                    trk.title = title;
                }
            } else if upper.starts_with("TRACK ") {
                // Flush previous track
                if let Some(prev) = current_track.take() {
                    sheet.tracks.push(prev);
                }
                let num_str = trimmed.split_whitespace().nth(1).unwrap_or("0");
                let num = num_str.parse::<u32>().unwrap_or(sheet.tracks.len() as u32 + 1);
                current_track = Some(RawCueTrack {
                    number: num,
                    title: format!("Track {:02}", num),
                    performer: None,
                    index_01_sec: 0.0,
                });
            } else if upper.starts_with("INDEX 01 ") || upper.starts_with("INDEX 1 ") {
                if let Some(ref mut trk) = current_track {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if let Some(time_str) = parts.last() {
                        if let Some(sec) = parse_cue_timestamp(time_str) {
                            trk.index_01_sec = sec;
                        }
                    }
                }
            }
        }

        if let Some(last) = current_track {
            sheet.tracks.push(last);
        }

        sheet
    }

    /// Read a `.cue` file from disk and convert its tracks into virtual `Track` items.
    pub fn load_tracks(cue_path: &Path) -> Result<Vec<Track>, String> {
        let content = std::fs::read_to_string(cue_path)
            .map_err(|e| format!("Gagal membaca berkas CUE: {}", e))?;
        let sheet = Self::parse(&content);

        if sheet.tracks.is_empty() {
            return Err("Tidak ada trek audio valid ditemukan dalam CUE sheet".to_string());
        }

        let cue_parent = cue_path.parent().unwrap_or_else(|| Path::new("."));
        let audio_path = if let Some(ref audio_name) = sheet.audio_file {
            let candidate = cue_parent.join(audio_name);
            if candidate.is_file() {
                candidate
            } else {
                find_audio_file_insensitive(cue_parent, audio_name)
                    .unwrap_or(candidate)
            }
        } else {
            // If no FILE directive, search for an audio file with the same stem as the cue file
            let stem = cue_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            find_audio_with_stem(cue_parent, stem)
                .ok_or_else(|| "Tidak dapat menemukan berkas audio yang dirujuk oleh CUE sheet".to_string())?
        };

        if !audio_path.is_file() {
            return Err(format!("Berkas audio rujukan tidak ditemukan di: {}", audio_path.display()));
        }

        // Extract format and metadata from actual audio file using lofty
        let (file_duration, sample_rate, bit_depth, format_str, file_size_mb, bitrate_kbps, replay_gain) =
            probe_audio_properties(&audio_path);

        let album_name = sheet.album_title.clone().unwrap_or_else(|| {
            audio_path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "CD Album".to_string())
        });
        let album_artist = sheet.album_performer.clone().unwrap_or_else(|| "Various Artists".to_string());

        let mut tracks = Vec::with_capacity(sheet.tracks.len());
        for (i, raw) in sheet.tracks.iter().enumerate() {
            let start = raw.index_01_sec;
            let duration = if i + 1 < sheet.tracks.len() {
                (sheet.tracks[i + 1].index_01_sec - start).max(1.0)
            } else {
                (file_duration - start).max(1.0)
            };

            let artist = raw.performer.clone().unwrap_or_else(|| album_artist.clone());
            tracks.push(Track {
                title: raw.title.clone(),
                artist,
                album: album_name.clone(),
                duration,
                format: format!("{}+CUE", format_str),
                sample_rate,
                bit_depth,
                waveform: Vec::new(),
                waveform_bass: Vec::new(),
                waveform_mid: Vec::new(),
                waveform_treble: Vec::new(),
                file_path: Some(audio_path.to_string_lossy().to_string()),
                file_size_mb,
                bitrate_kbps,
                is_favorite: false,
                replay_gain,
                cue_offset_sec: Some(start),
                cue_duration_sec: Some(duration),
                genre: None,
            });
        }

        Ok(tracks)
    }
}

fn extract_quoted_or_rest(line: &str, keyword: &str) -> String {
    let rest = line[keyword.len()..].trim();
    if let Some(first_quote) = rest.find('"') {
        if let Some(second_quote) = rest[first_quote + 1..].find('"') {
            return rest[first_quote + 1..first_quote + 1 + second_quote].to_string();
        }
    }
    rest.to_string()
}

/// Parses "mm:ss:ff" (75 frames per second CD audio standard) into total seconds
fn parse_cue_timestamp(ts: &str) -> Option<f32> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let min: f32 = parts[0].trim().parse().ok()?;
    let sec: f32 = parts[1].trim().parse().ok()?;
    let frames: f32 = parts[2].trim().parse().ok()?;

    Some(min * 60.0 + sec + (frames / 75.0))
}

fn find_audio_file_insensitive(parent: &Path, filename: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.eq_ignore_ascii_case(filename) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn find_audio_with_stem(parent: &Path, stem: &str) -> Option<PathBuf> {
    const AUDIO_EXTS: &[&str] = &["flac", "wav", "ape", "wv", "m4a", "mp3", "aiff"];
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(s) = p.file_stem().and_then(|s| s.to_str()) {
                if s.eq_ignore_ascii_case(stem) {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                        if AUDIO_EXTS.iter().any(|&ae| ae.eq_ignore_ascii_case(ext)) {
                            return Some(p);
                        }
                    }
                }
            }
        }
    }
    None
}

fn probe_audio_properties(path: &Path) -> (f32, u32, u8, String, f32, Option<u32>, ReplayGainData) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("FLAC").to_uppercase();
    let file_size_mb = std::fs::metadata(path).map(|m| m.len() as f32 / (1024.0 * 1024.0)).unwrap_or(0.0);

    match lofty::read_from_path(path) {
        Ok(tagged_file) => {
            let props = tagged_file.properties();
            let duration = props.duration().as_secs_f32();
            let sample_rate = props.sample_rate().unwrap_or(44100);
            let bit_depth = props.bit_depth().unwrap_or(16);
            let bitrate = props.audio_bitrate();
            let rg = crate::scanner::extract_replay_gain(tagged_file.primary_tag());
            (duration, sample_rate, bit_depth, ext, file_size_mb, bitrate, rg)
        }
        Err(_) => (180.0, 44100, 16, ext, file_size_mb, None, ReplayGainData::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cue_dark_side_of_the_moon() {
        let cue_sample = r#"
REM GENRE Progressive Rock
REM DATE 1973
PERFORMER "Pink Floyd"
TITLE "The Dark Side of the Moon"
FILE "The Dark Side of the Moon.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Speak to Me"
    PERFORMER "Pink Floyd"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Breathe (In the Air)"
    PERFORMER "Pink Floyd"
    INDEX 01 01:07:25
  TRACK 03 AUDIO
    TITLE "On the Run"
    PERFORMER "Pink Floyd"
    INDEX 01 03:56:50
"#;
        let sheet = CueSheet::parse(cue_sample);
        assert_eq!(sheet.album_performer.as_deref(), Some("Pink Floyd"));
        assert_eq!(sheet.album_title.as_deref(), Some("The Dark Side of the Moon"));
        assert_eq!(sheet.audio_file.as_deref(), Some("The Dark Side of the Moon.flac"));
        assert_eq!(sheet.tracks.len(), 3);

        assert_eq!(sheet.tracks[0].title, "Speak to Me");
        assert!((sheet.tracks[0].index_01_sec - 0.0).abs() < 1e-4);

        assert_eq!(sheet.tracks[1].title, "Breathe (In the Air)");
        // 01:07:25 = 1 * 60 + 7 + 25/75 = 67.3333...
        assert!((sheet.tracks[1].index_01_sec - (67.0 + 25.0 / 75.0)).abs() < 1e-3);

        assert_eq!(sheet.tracks[2].title, "On the Run");
        // 03:56:50 = 3 * 60 + 56 + 50/75 = 236.6666...
        assert!((sheet.tracks[2].index_01_sec - (236.0 + 50.0 / 75.0)).abs() < 1e-3);
    }
}
