use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

pub struct AudioDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    _time_base: Option<TimeBase>,
    sample_rate: u32,
    source_channels: usize,
    duration: f32,
    current_time: f32,
    sample_buf: Option<SampleBuffer<f32>>,
    current_spec: Option<SignalSpec>,
    stereo_cache: Vec<f32>,
    cache_read_idx: usize,
}

impl AudioDecoder {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path_ref.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| "Tidak ditemukan audio track yang valid dalam file")?;

        let track_id = track.id;
        let time_base = track.codec_params.time_base;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let source_channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);

        let duration = if let (Some(n_frames), Some(tb)) = (track.codec_params.n_frames, time_base) {
            let time = tb.calc_time(n_frames);
            time.seconds as f32 + (time.frac as f32)
        } else {
            0.0
        };

        let decoder = symphonia::default::get_codecs().make(
            &track.codec_params,
            &DecoderOptions::default(),
        )?;

        Ok(Self {
            format,
            decoder,
            track_id,
            _time_base: time_base,
            sample_rate,
            source_channels,
            duration,
            current_time: 0.0,
            sample_buf: None,
            current_spec: None,
            stereo_cache: Vec::with_capacity(8192),
            cache_read_idx: 0,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    pub fn source_channels(&self) -> usize {
        self.source_channels
    }

    pub fn seek(&mut self, time_secs: f32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let seek_to = SeekTo::Time {
            time: Time::from(time_secs.max(0.0) as f64),
            track_id: Some(self.track_id),
        };

        self.format.seek(SeekMode::Coarse, seek_to)?;
        self.decoder.reset();
        self.stereo_cache.clear();
        self.cache_read_idx = 0;
        self.current_time = time_secs.max(0.0);
        Ok(())
    }

    // Decode packets until stereo_cache has data or EOF is hit
    fn fill_cache(&mut self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Clear consumed samples
        if self.cache_read_idx > 0 {
            self.stereo_cache.drain(0..self.cache_read_idx);
            self.cache_read_idx = 0;
        }

        while self.stereo_cache.is_empty() {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(false); // EOF
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(err) => return Err(Box::new(err)),
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(audio_buf_ref) => {
                    Self::append_audio_buffer(
                        &mut self.sample_buf,
                        &mut self.current_spec,
                        &mut self.stereo_cache,
                        audio_buf_ref,
                    );
                }
                Err(SymphoniaError::DecodeError(_)) => {
                    // Skip corrupt / unparseable packet and continue
                    continue;
                }
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(err) => return Err(Box::new(err)),
            }
        }

        Ok(!self.stereo_cache.is_empty())
    }

    fn append_audio_buffer(
        sample_buf: &mut Option<SampleBuffer<f32>>,
        current_spec: &mut Option<SignalSpec>,
        stereo_cache: &mut Vec<f32>,
        audio_buf_ref: AudioBufferRef,
    ) {
        let spec = *audio_buf_ref.spec();
        let duration = audio_buf_ref.capacity() as u64;

        let needs_new_buf = match sample_buf {
            Some(b) => current_spec.as_ref() != Some(&spec) || b.capacity() < duration as usize,
            None => true,
        };

        if needs_new_buf {
            *sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
            *current_spec = Some(spec);
        }

        let buf = sample_buf.as_mut().unwrap();
        buf.copy_interleaved_ref(audio_buf_ref);
        let raw_samples = buf.samples();

        let channels = spec.channels.count();
        if channels == 1 {
            // Mono to Stereo: Duplicate L to R
            for &s in raw_samples {
                stereo_cache.push(s);
                stereo_cache.push(s);
            }
        } else if channels == 2 {
            // Direct Stereo
            stereo_cache.extend_from_slice(raw_samples);
        } else if channels > 2 {
            // Downmix to Stereo by taking Left (ch 0) and Right (ch 1)
            for frame in raw_samples.chunks_exact(channels) {
                stereo_cache.push(frame[0]);
                stereo_cache.push(frame[1]);
            }
        }
    }

    // Read up to dest.len() interleaved stereo f32 samples (L, R, L, R...).
    // Returns the number of samples copied (always an even number for complete stereo frames).
    pub fn read_stereo_samples(&mut self, dest: &mut [f32]) -> usize {
        let mut total_written = 0;
        let needed = dest.len();

        while total_written < needed {
            let available = self.stereo_cache.len() - self.cache_read_idx;

            if available == 0 {
                match self.fill_cache() {
                    Ok(true) => continue,
                    _ => break, // EOF or unrecoverable error
                }
            }

            let to_copy = (needed - total_written).min(self.stereo_cache.len() - self.cache_read_idx);
            let end_idx = self.cache_read_idx + to_copy;

            dest[total_written..total_written + to_copy]
                .copy_from_slice(&self.stereo_cache[self.cache_read_idx..end_idx]);

            self.cache_read_idx = end_idx;
            total_written += to_copy;
        }

        // Advance position based on stereo frames read (1 frame = 2 samples)
        if self.sample_rate > 0 {
            let frames_read = (total_written / 2) as f32;
            self.current_time += frames_read / (self.sample_rate as f32);
        }

        total_written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_non_existent_file() {
        let res = AudioDecoder::open("non_existent_song.flac");
        assert!(res.is_err());
    }
}

