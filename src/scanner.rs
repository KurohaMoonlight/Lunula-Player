use lofty::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use crate::models::{ScanMsg, Track};
use crate::app::LunulaApp;

// Generate realistic unique deterministic multi-band waveform (Overall, Bass, Mid, Treble)
pub fn generate_multiband_waveform(
    seed_str: &str,
    sample_count: usize,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut hash: u64 = 5381;
    for b in seed_str.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u64);
    }
    let base_freq = 12.0 + (hash % 16) as f32;
    let sec_freq = 28.0 + ((hash >> 8) % 24) as f32;
    let high_freq = 56.0 + ((hash >> 16) % 36) as f32;

    let mut overall = Vec::with_capacity(sample_count);
    let mut bass = Vec::with_capacity(sample_count);
    let mut mid = Vec::with_capacity(sample_count);
    let mut treble = Vec::with_capacity(sample_count);

    for i in 0..sample_count {
        let t = i as f32 / sample_count as f32;
        let envelope = if t < 0.05 {
            t / 0.05
        } else if t > 0.92 {
            (1.0 - t) / 0.08
        } else {
            1.0
        };

        // Bass (<250Hz): rhythmic kick pulses + sub bassline
        let kick = ((t * base_freq * 4.0).sin().powi(6) * 0.70).max(0.0);
        let sub = (t * base_freq * 0.5).sin().abs() * 0.30;
        let b = ((kick + sub) * envelope).clamp(0.03, 1.0);

        // Mid (250Hz - 4kHz): melodic vocal curves + chord harmonies
        let vocal = (t * sec_freq).sin().abs() * 0.55;
        let chord = (t * (sec_freq * 0.5)).cos().powi(2) * 0.35;
        let m = ((vocal + chord) * envelope).clamp(0.04, 1.0);

        // Treble (>4kHz): crisp hi-hat ticks, cymbal shimmer, sharp transients
        let hihat = if (i % 16) == 0 || (i % 24) == 8 { 0.75 } else { 0.10 };
        let cymbal = (t * high_freq).sin().abs() * 0.25;
        let tr = ((hihat + cymbal) * envelope).clamp(0.03, 1.0);

        // Combined overall envelope
        let ov = (b * 0.45 + m * 0.38 + tr * 0.25).clamp(0.04, 1.0);

        overall.push(ov);
        bass.push(b);
        mid.push(m);
        treble.push(tr);
    }

    (overall, bass, mid, treble)
}

// Generate realistic unique deterministic waveform envelope for scanned tracks
#[allow(dead_code)]
pub fn generate_preview_waveform(seed_str: &str, sample_count: usize) -> Vec<f32> {
    generate_multiband_waveform(seed_str, sample_count).0
}

// Extract audio metadata tags & properties using lofty
pub fn parse_single_audio_file(path: &Path) -> Option<Track> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    let valid_exts = ["flac", "mp3", "wav", "m4a", "aac", "ogg", "opus", "alac", "aiff", "wma"];
    if !valid_exts.contains(&ext.as_str()) {
        return None;
    }

    let file_size_mb = std::fs::metadata(path).map(|m| m.len() as f32 / (1024.0 * 1024.0)).unwrap_or(0.0);
    let fallback_title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown Title").to_string();

    let (title, artist, album, duration, sample_rate, bit_depth, bitrate_kbps) = match lofty::read_from_path(path) {
        Ok(tagged_file) => {
            let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());
            let title = tag.and_then(|t| t.title()).map(|s| s.to_string()).unwrap_or_else(|| fallback_title.clone());
            let artist = tag.and_then(|t| t.artist()).map(|s| s.to_string()).unwrap_or_else(|| "Unknown Artist".to_string());
            let album = tag.and_then(|t| t.album()).map(|s| s.to_string()).unwrap_or_else(|| {
                path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("Single").to_string()
            });

            let props = tagged_file.properties();
            let duration = props.duration().as_secs_f32().max(1.0);
            let sample_rate = props.sample_rate().unwrap_or(44100);
            let bit_depth = props.bit_depth().unwrap_or(16);
            let bitrate = props.audio_bitrate().or_else(|| props.overall_bitrate());

            (title, artist, album, duration, sample_rate, bit_depth, bitrate)
        }
        Err(_) => {
            (fallback_title.clone(), "Unknown Artist".to_string(), "Single".to_string(), 180.0, 44100, 16, None)
        }
    };

    let (waveform, waveform_bass, waveform_mid, waveform_treble) =
        generate_multiband_waveform(&fallback_title, 3600);

    Some(Track {
        title,
        artist,
        album,
        duration,
        format: ext.to_uppercase(),
        sample_rate,
        bit_depth,
        waveform,
        waveform_bass,
        waveform_mid,
        waveform_treble,
        file_path: Some(path.to_string_lossy().to_string()),
        file_size_mb,
        bitrate_kbps,
        is_favorite: false,
    })
}

// Background traversal of music folder
pub fn scan_music_folder(folder: PathBuf, tx: Sender<ScanMsg>) {
    let _ = tx.send(ScanMsg::Started(folder.display().to_string()));
    let mut count = 0;

    for entry in walkdir::WalkDir::new(&folder).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(track) = parse_single_audio_file(entry.path()) {
                count += 1;
                let _ = tx.send(ScanMsg::TrackFound(track));
            }
        }
    }

    let _ = tx.send(ScanMsg::Finished(count));
}

impl LunulaApp {
    // Open native folder picker dialog and start background scan
    pub fn start_folder_scan(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.scan_rx = Some(rx);
        self.is_scanning = true;

        std::thread::spawn(move || {
            if let Some(folder) = rfd::FileDialog::new().set_title("Pilih Folder Musik").pick_folder() {
                scan_music_folder(folder, tx);
            } else {
                let _ = tx.send(ScanMsg::Finished(0));
            }
        });
    }

    // Open native file picker for individual audio files
    pub fn start_files_scan(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.scan_rx = Some(rx);
        self.is_scanning = true;

        std::thread::spawn(move || {
            if let Some(files) = rfd::FileDialog::new()
                .set_title("Pilih File Audio")
                .add_filter("Audio Files", &["flac", "wav", "mp3", "m4a", "aac", "ogg", "opus", "alac", "aiff"])
                .pick_files()
            {
                let _ = tx.send(ScanMsg::Started(format!("{} file terpilih", files.len())));
                let mut count = 0;
                for f in files {
                    if let Some(track) = parse_single_audio_file(&f) {
                        count += 1;
                        let _ = tx.send(ScanMsg::TrackFound(track));
                    }
                }
                let _ = tx.send(ScanMsg::Finished(count));
            } else {
                let _ = tx.send(ScanMsg::Finished(0));
            }
        });
    }
}

// Extract album artwork from ID3v2, FLAC, M4A, or OGG tags
pub fn extract_album_art<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    let tagged_file = lofty::read_from_path(path).ok()?;
    if let Some(tag) = tagged_file.primary_tag() {
        if let Some(pic) = tag.pictures().first() {
            return Some(pic.data().to_vec());
        }
    }
    if let Some(tag) = tagged_file.first_tag() {
        if let Some(pic) = tag.pictures().first() {
            return Some(pic.data().to_vec());
        }
    }
    None
}

// Convert raw image bytes (JPEG / PNG) into an egui TextureHandle
pub fn load_image_to_texture(
    ctx: &eframe::egui::Context,
    name: &str,
    image_bytes: &[u8],
) -> Option<eframe::egui::TextureHandle> {
    let img = image::load_from_memory(image_bytes).ok()?;
    let img = img.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let pixels = img.into_raw();
    let color_image = eframe::egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    Some(ctx.load_texture(name, color_image, eframe::egui::TextureOptions::LINEAR))
}


