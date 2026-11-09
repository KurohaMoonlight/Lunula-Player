use rubato::{
    Async, FixedAsync, Indexing, Resampler, SincInterpolationParameters,
    SincInterpolationType, WindowFunction,
};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use crate::audio::types::ResamplerSettings;

// Audiophile-grade stereo sinc resampler using Rubato
// Replaces naive linear interpolation with bandlimited windowed sinc convolution,
// eliminating aliasing upon downsampling and imaging artifacts upon upsampling.
pub struct StereoResampler {
    source_rate: u32,
    target_rate: u32,
    settings: ResamplerSettings,
    sinc_resampler: Option<Async<f32>>,
    in_buffer: Vec<f32>,
    out_buffer: Vec<f32>,
    out_queue: Vec<(f32, f32)>,
    out_queue_idx: usize,
    flushed_eof: bool,
}

impl StereoResampler {
    pub fn new(source_rate: u32, target_rate: u32) -> Self {
        Self::new_with_settings(source_rate, target_rate, ResamplerSettings::default())
    }

    pub fn new_with_settings(source_rate: u32, target_rate: u32, settings: ResamplerSettings) -> Self {
        let mut resampler = Self {
            source_rate: source_rate.max(1),
            target_rate: target_rate.max(1),
            settings,
            sinc_resampler: None,
            in_buffer: Vec::new(),
            out_buffer: Vec::new(),
            out_queue: Vec::new(),
            out_queue_idx: 0,
            flushed_eof: false,
        };
        resampler.setup_sinc();
        resampler
    }

    pub fn settings(&self) -> ResamplerSettings {
        self.settings
    }

    pub fn set_settings(&mut self, settings: ResamplerSettings) {
        if self.settings != settings {
            self.settings = settings;
            self.setup_sinc();
        }
    }

    #[inline(always)]
    pub fn is_passthrough(&self) -> bool {
        self.source_rate == self.target_rate
    }

    fn setup_sinc(&mut self) {
        self.flushed_eof = false;
        if self.is_passthrough() {
            self.sinc_resampler = None;
            self.in_buffer.clear();
            self.out_buffer.clear();
            self.out_queue.clear();
            self.out_queue_idx = 0;
            return;
        }

        let ratio = self.target_rate as f64 / self.source_rate as f64;
        let params = SincInterpolationParameters {
            sinc_len: self.settings.quality.sinc_len(),
            f_cutoff: Some(self.settings.profile.cutoff()),
            interpolation: SincInterpolationType::Cubic,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let in_chunk = 256;
        if let Ok(resampler) = Async::<f32>::new_sinc(
            ratio,
            1.01,
            &params,
            in_chunk,
            2,
            FixedAsync::Input,
        ) {
            let in_req = resampler.input_frames_next();
            let out_req = resampler.output_frames_next();
            self.in_buffer = vec![0.0f32; (in_req + 64) * 2];
            self.out_buffer = vec![0.0f32; (out_req + 64) * 2];
            self.out_queue = Vec::with_capacity(out_req + 64);
            self.out_queue_idx = 0;
            self.sinc_resampler = Some(resampler);
        } else {
            self.sinc_resampler = None;
        }
    }

    // Fetches the next resampled stereo frame (L, R).
    // Calls `fetch` to obtain incoming native frames in chunks as needed.
    pub fn next_frame<F>(&mut self, mut fetch: F) -> Option<(f32, f32)>
    where
        F: FnMut() -> Option<(f32, f32)>,
    {
        // 100% Bit-perfect direct passthrough when sample rates match
        if self.is_passthrough() || self.sinc_resampler.is_none() {
            return fetch();
        }

        // Return from pre-computed output queue if available
        if self.out_queue_idx < self.out_queue.len() {
            let frame = self.out_queue[self.out_queue_idx];
            self.out_queue_idx += 1;
            return Some(frame);
        }

        // Output queue empty: fetch input chunk and process via Sinc resampler
        self.out_queue.clear();
        self.out_queue_idx = 0;

        let Some(resampler) = self.sinc_resampler.as_mut() else {
            return fetch();
        };
        let in_frames = resampler.input_frames_next();
        let in_samples_needed = in_frames * 2;
        if self.in_buffer.len() < in_samples_needed {
            self.in_buffer.resize(in_samples_needed + 128, 0.0);
        }

        let mut valid_in_frames = 0;
        while valid_in_frames < in_frames {
            if let Some((l, r)) = fetch() {
                self.in_buffer[valid_in_frames * 2] = l;
                self.in_buffer[valid_in_frames * 2 + 1] = r;
                valid_in_frames += 1;
            } else {
                break;
            }
        }

        if valid_in_frames == 0 {
            if !self.flushed_eof {
                self.flushed_eof = true;
                // Zero-fill input buffer to flush the sinc filter delay line
                self.in_buffer[..in_samples_needed].fill(0.0);
                valid_in_frames = in_frames;
            } else {
                return None; // True EOF: sinc filter has been fully flushed
            }
        } else if valid_in_frames < in_frames {
            // Partial final chunk before EOF: zero-fill remainder
            for i in (valid_in_frames * 2)..in_samples_needed {
                self.in_buffer[i] = 0.0;
            }
        }

        let in_adapter = match InterleavedSlice::new(&self.in_buffer[..in_samples_needed], 2, in_frames) {
            Ok(adapter) => adapter,
            Err(e) => {
                eprintln!("[Resampler Error] in_adapter creation failed: {}", e);
                return None;
            }
        };

        let out_frames_needed = resampler.output_frames_next();
        let out_samples_needed = out_frames_needed * 2;
        if self.out_buffer.len() < out_samples_needed {
            self.out_buffer.resize(out_samples_needed + 128, 0.0);
        }
        let mut out_adapter = match InterleavedSlice::new_mut(&mut self.out_buffer[..out_samples_needed], 2, out_frames_needed) {
            Ok(adapter) => adapter,
            Err(e) => {
                eprintln!("[Resampler Error] out_adapter creation failed: {}", e);
                return None;
            }
        };

        let indexing = if valid_in_frames < in_frames {
            Some(Indexing::new().partial_len(valid_in_frames))
        } else {
            None
        };

        match resampler.process_into_buffer(&in_adapter, &mut out_adapter, indexing.as_ref()) {
            Ok((_, produced_frames)) => {
                for f in 0..produced_frames {
                    let l = self.out_buffer[f * 2];
                    let r = self.out_buffer[f * 2 + 1];
                    self.out_queue.push((l, r));
                }
            }
            Err(e) => {
                eprintln!("[Resampler Error] Rubato process failed: {}", e);
                return None;
            }
        }

        if self.out_queue_idx < self.out_queue.len() {
            let frame = self.out_queue[self.out_queue_idx];
            self.out_queue_idx += 1;
            Some(frame)
        } else {
            None
        }
    }

    pub fn reset(&mut self, source_rate: u32, target_rate: u32) {
        self.source_rate = source_rate.max(1);
        self.target_rate = target_rate.max(1);
        self.setup_sinc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        let mut resampler = StereoResampler::new(44100, 44100);
        assert!(resampler.is_passthrough());

        let mut samples = vec![(0.5, -0.5), (0.2, -0.2)];
        let f1 = resampler.next_frame(|| if !samples.is_empty() { Some(samples.remove(0)) } else { None });
        assert_eq!(f1, Some((0.5, -0.5)));
    }

    #[test]
    fn test_sinc_resampling_ratio_upsample() {
        // Resample from 44100 Hz to 48000 Hz
        let mut resampler = StereoResampler::new(44100, 48000);
        assert!(!resampler.is_passthrough());

        let mut count = 0;
        let total_in = 1000;
        let mut outs = Vec::new();

        while let Some(f) = resampler.next_frame(|| {
            if count < total_in {
                count += 1;
                Some((0.75, -0.75))
            } else {
                None
            }
        }) {
            outs.push(f);
        }

        // Expected output length should be roughly 1000 * (48000 / 44100) = ~1088 frames
        assert!(outs.len() >= 900);
        // Verify samples are bounded
        for (l, r) in outs {
            assert!(l <= 1.0 && l >= -1.0);
            assert!(r <= 1.0 && r >= -1.0);
        }
    }

    #[test]
    fn test_sinc_resampling_ratio_downsample() {
        // Resample from 96000 Hz to 48000 Hz (2:1 downsampling with anti-aliasing)
        let mut resampler = StereoResampler::new(96000, 48000);
        assert!(!resampler.is_passthrough());

        let mut count = 0;
        let total_in = 2000;
        let mut outs = Vec::new();

        while let Some(f) = resampler.next_frame(|| {
            if count < total_in {
                count += 1;
                Some((0.5, -0.5))
            } else {
                None
            }
        }) {
            outs.push(f);
        }

        // Expected output length should be roughly 2000 * (48000 / 96000) = ~1000 frames
        assert!(outs.len() >= 800 && outs.len() <= 1200);
    }

    #[test]
    fn test_resampler_custom_settings() {
        use crate::audio::types::{RollOffProfile, SincQuality};
        let mut resampler = StereoResampler::new(44100, 48000);
        assert_eq!(resampler.settings().quality, SincQuality::Studio);
        assert_eq!(resampler.settings().profile, RollOffProfile::Sharp);

        // Switch to Fast + Smooth
        let custom = ResamplerSettings {
            quality: SincQuality::Fast,
            profile: RollOffProfile::Smooth,
        };
        resampler.set_settings(custom);
        assert_eq!(resampler.settings(), custom);

        // Switch to Extreme + Nyquist
        let extreme = ResamplerSettings {
            quality: SincQuality::Extreme,
            profile: RollOffProfile::Nyquist,
        };
        resampler.set_settings(extreme);
        assert_eq!(resampler.settings(), extreme);

        // Verify it still processes frames properly under extreme settings
        let mut count = 0;
        let mut outs = 0;
        while let Some(_) = resampler.next_frame(|| {
            if count < 500 {
                count += 1;
                Some((0.3, -0.3))
            } else {
                None
            }
        }) {
            outs += 1;
        }
        assert!(outs > 400);
    }

    #[test]
    fn test_resampler_buffer_overrun_resilience() {
        // Test irrational sample rate ratios over 20,000 frames to ensure
        // fluctuating input_frames_next() never exceeds dynamic buffer bounds.
        let rate_pairs = [(44100, 48000), (48000, 44100), (88200, 48000), (96000, 44100), (192000, 44100)];
        for (src, dst) in rate_pairs {
            let mut resampler = StereoResampler::new(src, dst);
            let mut fed = 0;
            let total_in = 20_000;
            let mut total_out = 0;
            while let Some((l, r)) = resampler.next_frame(|| {
                if fed < total_in {
                    fed += 1;
                    Some((0.42, -0.42))
                } else {
                    None
                }
            }) {
                total_out += 1;
                assert!(l.is_finite() && r.is_finite());
            }
            assert!(total_out > 0);
        }
    }
}
