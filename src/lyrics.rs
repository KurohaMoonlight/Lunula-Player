use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct LyricLine {
    pub timestamp_sec: f32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Lyrics {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub offset_sec: f32,
    pub lines: Vec<LyricLine>,
}

impl Lyrics {
    /// Parse an LRC format string into structured synchronized lyrics.
    pub fn parse(content: &str) -> Self {
        let mut lyrics = Lyrics::default();
        let mut lines = Vec::new();

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            // Check for metadata tags: [ti:Title], [ar:Artist], [al:Album], [offset:+/-ms]
            if line.starts_with('[') && line.contains(':') && !line[1..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                if let Some(end_bracket) = line.find(']') {
                    let tag_content = &line[1..end_bracket];
                    if let Some(colon) = tag_content.find(':') {
                        let key = tag_content[..colon].trim().to_ascii_lowercase();
                        let val = tag_content[colon + 1..].trim();
                        match key.as_str() {
                            "ti" => lyrics.title = Some(val.to_string()),
                            "ar" => lyrics.artist = Some(val.to_string()),
                            "al" => lyrics.album = Some(val.to_string()),
                            "offset" => {
                                if let Ok(ms) = val.parse::<f32>() {
                                    lyrics.offset_sec = ms / 1000.0;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                continue;
            }

            // Parse timestamp tags: e.g. [01:23.45] or multiple [01:23.45][02:34.56]
            let mut timestamps = Vec::new();
            let mut remaining = line;

            while remaining.starts_with('[') {
                if let Some(end_bracket) = remaining.find(']') {
                    let ts_str = &remaining[1..end_bracket];
                    if let Some(sec) = parse_lrc_timestamp(ts_str) {
                        timestamps.push(sec);
                    }
                    remaining = remaining[end_bracket + 1..].trim_start();
                } else {
                    break;
                }
            }

            let text = remaining.trim().to_string();
            for ts in timestamps {
                lines.push(LyricLine {
                    timestamp_sec: ts,
                    text: text.clone(),
                });
            }
        }

        // Sort lines chronologically
        lines.sort_by(|a, b| a.timestamp_sec.partial_cmp(&b.timestamp_sec).unwrap_or(std::cmp::Ordering::Equal));
        lyrics.lines = lines;
        lyrics
    }

    /// Finds the index of the currently playing lyric line given playback position in seconds.
    pub fn find_active_index(&self, current_time_sec: f32) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }

        let effective_time = current_time_sec - self.offset_sec;
        if effective_time < self.lines[0].timestamp_sec {
            return None;
        }

        // Binary search or linear scan for the latest line where timestamp <= effective_time
        let mut active_idx = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if line.timestamp_sec <= effective_time {
                active_idx = i;
            } else {
                break;
            }
        }

        Some(active_idx)
    }

    /// Automatically search and load a corresponding `.lrc` file for an audio track path.
    pub fn load_for_track(track_path: &Path) -> Option<Self> {
        let lrc_path = track_path.with_extension("lrc");
        if lrc_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&lrc_path) {
                return Some(Self::parse(&content));
            }
        }

        // Try case-insensitive matching in the same directory
        if let Some(parent) = track_path.parent() {
            if let Some(stem) = track_path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()).map_or(false, |ext| ext.eq_ignore_ascii_case("lrc")) {
                            if path.file_stem().and_then(|s| s.to_str()).map_or(false, |s| s.eq_ignore_ascii_case(stem)) {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    return Some(Self::parse(&content));
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

/// Parses "[mm:ss.xx]" or "[mm:ss]" into total seconds.
fn parse_lrc_timestamp(ts: &str) -> Option<f32> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let min: f32 = parts[0].trim().parse().ok()?;
    let sec: f32 = parts[1].trim().parse().ok()?;
    Some(min * 60.0 + sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc_standard_lyrics() {
        let lrc = r#"
[ti:Honest Melody]
[ar:Lunula Artist]
[al:Audiophile Dreams]
[offset:500]
[00:05.00]First line of the song
[00:10.50]Second line with feeling
[00:18.00]Chorus explodes with energy
"#;
        let parsed = Lyrics::parse(lrc);
        assert_eq!(parsed.title.as_deref(), Some("Honest Melody"));
        assert_eq!(parsed.artist.as_deref(), Some("Lunula Artist"));
        assert_eq!(parsed.album.as_deref(), Some("Audiophile Dreams"));
        assert!((parsed.offset_sec - 0.5).abs() < 1e-4);
        assert_eq!(parsed.lines.len(), 3);

        assert!((parsed.lines[0].timestamp_sec - 5.0).abs() < 1e-4);
        assert_eq!(parsed.lines[0].text, "First line of the song");

        assert!((parsed.lines[1].timestamp_sec - 10.5).abs() < 1e-4);
        assert_eq!(parsed.lines[1].text, "Second line with feeling");

        // Test active index lookups with offset 0.5s
        // At 4.0s (effective 3.5s): None
        assert_eq!(parsed.find_active_index(4.0), None);
        // At 6.0s (effective 5.5s): line 0
        assert_eq!(parsed.find_active_index(6.0), Some(0));
        // At 12.0s (effective 11.5s): line 1
        assert_eq!(parsed.find_active_index(12.0), Some(1));
        // At 20.0s (effective 19.5s): line 2
        assert_eq!(parsed.find_active_index(20.0), Some(2));
    }

    #[test]
    fn test_multiple_timestamps_per_line() {
        let lrc = "[00:10.00][00:30.00]Repeated vocal hook";
        let parsed = Lyrics::parse(lrc);
        assert_eq!(parsed.lines.len(), 2);
        assert!((parsed.lines[0].timestamp_sec - 10.0).abs() < 1e-4);
        assert!((parsed.lines[1].timestamp_sec - 30.0).abs() < 1e-4);
        assert_eq!(parsed.lines[0].text, "Repeated vocal hook");
        assert_eq!(parsed.lines[1].text, "Repeated vocal hook");
    }
}

