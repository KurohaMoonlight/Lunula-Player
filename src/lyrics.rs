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
        let clean_content = content.trim_start_matches('\u{feff}');
        let mut lyrics = Lyrics::default();
        let mut lines = Vec::new();
        let mut plain_text_fallback = Vec::new();

        for raw_line in clean_content.lines() {
            let line = raw_line.trim().trim_start_matches('\u{feff}');
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
            if !timestamps.is_empty() {
                for ts in timestamps {
                    lines.push(LyricLine {
                        timestamp_sec: ts,
                        text: text.clone(),
                    });
                }
            } else if !text.is_empty() {
                plain_text_fallback.push(text);
            }
        }

        // If no timestamped lines found but plain text lines exist, provide incremental fallback
        if lines.is_empty() && !plain_text_fallback.is_empty() {
            for (i, t) in plain_text_fallback.into_iter().enumerate() {
                lines.push(LyricLine {
                    timestamp_sec: i as f32 * 5.0,
                    text: t,
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
        Self::load_for_track_smart(track_path, "", "")
    }

    /// Smart multi-strategy search and load for `.lrc` files:
    /// Supports exact stem matching, `.flac.lrc`, title/artist matching, and multi-encoding decoding (UTF-8, UTF-16 LE/BE, ANSI).
    pub fn load_for_track_smart(track_path: &Path, title: &str, artist: &str) -> Option<Self> {
        // 1. Direct path with .lrc extension (e.g. Song.flac -> Song.lrc)
        let lrc_path = track_path.with_extension("lrc");
        if lrc_path.is_file() {
            if let Some(content) = read_lrc_file_robust(&lrc_path) {
                let parsed = Self::parse(&content);
                if !parsed.lines.is_empty() {
                    return Some(parsed);
                }
            }
        }

        // 2. Direct path with .flac.lrc / .mp3.lrc (e.g. Song.flac -> Song.flac.lrc)
        let lrc_extra = std::path::PathBuf::from(format!("{}.lrc", track_path.display()));
        if lrc_extra.is_file() {
            if let Some(content) = read_lrc_file_robust(&lrc_extra) {
                let parsed = Self::parse(&content);
                if !parsed.lines.is_empty() {
                    return Some(parsed);
                }
            }
        }

        // 3. Scan directory with case-insensitive and metadata-aware matching
        if let Some(parent) = track_path.parent() {
            let track_stem = track_path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let track_filename = track_path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let clean_title = title.trim().to_lowercase();
            let clean_artist = artist.trim().to_lowercase();
            let artist_title = if !clean_artist.is_empty() && !clean_title.is_empty() {
                Some(format!("{} - {}", clean_artist, clean_title))
            } else {
                None
            };
            let title_artist = if !clean_artist.is_empty() && !clean_title.is_empty() {
                Some(format!("{} - {}", clean_title, clean_artist))
            } else {
                None
            };

            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let is_lrc = path.extension().and_then(|e| e.to_str())
                            .map_or(false, |ext| ext.eq_ignore_ascii_case("lrc") || ext.eq_ignore_ascii_case("txt"));
                        if is_lrc {
                            if let Some(entry_stem) = path.file_stem().and_then(|s| s.to_str()) {
                                let entry_stem_lower = entry_stem.to_lowercase();

                                let matches = entry_stem_lower == track_stem
                                    || entry_stem_lower == track_filename
                                    || (!clean_title.is_empty() && entry_stem_lower == clean_title)
                                    || artist_title.as_ref().map_or(false, |at| entry_stem_lower == *at)
                                    || title_artist.as_ref().map_or(false, |ta| entry_stem_lower == *ta)
                                    || entry_stem_lower.contains(&track_stem)
                                    || (!clean_title.is_empty() && entry_stem_lower.contains(&clean_title));

                                if matches {
                                    if let Some(content) = read_lrc_file_robust(&path) {
                                        let parsed = Self::parse(&content);
                                        if !parsed.lines.is_empty() {
                                            return Some(parsed);
                                        }
                                    }
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

/// Reads file bytes with encoding detection for UTF-8 (with/without BOM), UTF-16 LE/BE, and ANSI fallbacks.
pub fn read_lrc_file_robust(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }

    // 1. UTF-8 with BOM (0xEF, 0xBB, 0xBF)
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return String::from_utf8(bytes[3..].to_vec()).ok()
            .or_else(|| Some(String::from_utf8_lossy(&bytes[3..]).into_owned()));
    }

    // 2. UTF-16 LE BOM (0xFF, 0xFE)
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16(&u16s).ok()
            .or_else(|| Some(String::from_utf16_lossy(&u16s)));
    }

    // 3. UTF-16 BE BOM (0xFE, 0xFF)
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16(&u16s).ok()
            .or_else(|| Some(String::from_utf16_lossy(&u16s)));
    }

    // 4. Standard UTF-8
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        return Some(s);
    }

    // 5. Fallback: UTF-8 lossy / ANSI Latin-1
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Parses "[hh:mm:ss.xx]" or "[mm:ss.xx]" or "[mm:ss]" into total seconds. Supports comma decimals.
fn parse_lrc_timestamp(ts: &str) -> Option<f32> {
    let clean = ts.trim();
    let parts: Vec<&str> = clean.split(':').collect();
    match parts.len() {
        2 => {
            let min: f32 = parts[0].trim().parse().ok()?;
            let sec_str = parts[1].trim().replace(',', ".");
            let sec: f32 = sec_str.parse().ok()?;
            Some(min * 60.0 + sec)
        }
        3 => {
            let hr: f32 = parts[0].trim().parse().ok()?;
            let min: f32 = parts[1].trim().parse().ok()?;
            let sec_str = parts[2].trim().replace(',', ".");
            let sec: f32 = sec_str.parse().ok()?;
            Some(hr * 3600.0 + min * 60.0 + sec)
        }
        _ => None,
    }
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

    #[test]
    fn test_bom_comma_and_hours_timestamp() {
        let lrc_with_bom = "\u{feff}[01:10,50]Line with comma decimal\n[01:02:03.45]Line with hours";
        let parsed = Lyrics::parse(lrc_with_bom);
        assert_eq!(parsed.lines.len(), 2);
        assert!((parsed.lines[0].timestamp_sec - 70.5).abs() < 1e-4);
        assert_eq!(parsed.lines[0].text, "Line with comma decimal");
        assert!((parsed.lines[1].timestamp_sec - 3723.45).abs() < 1e-3);
        assert_eq!(parsed.lines[1].text, "Line with hours");
    }
}

