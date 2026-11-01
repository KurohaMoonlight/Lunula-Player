use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

pub const VISUALIZER_BUFFER_SIZE: usize = 2048;
pub const FFT_SIZE: usize = 1024;
pub const NUM_BANDS: usize = 48;

// Lock-free circular sample buffer shared between audio rendering callback and UI thread
pub struct VisualizerRingBuffer {
    buffer: [AtomicU32; VISUALIZER_BUFFER_SIZE],
    write_idx: AtomicUsize,
}

impl VisualizerRingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: std::array::from_fn(|_| AtomicU32::new(0)),
            write_idx: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn push_sample(&self, sample: f32) {
        let idx = self.write_idx.fetch_add(1, Ordering::Relaxed) % VISUALIZER_BUFFER_SIZE;
        self.buffer[idx].store(sample.to_bits(), Ordering::Relaxed);
    }

    pub fn clear(&self) {
        for b in &self.buffer {
            b.store(0, Ordering::Relaxed);
        }
    }

    pub fn get_latest_samples(&self, dest: &mut [f32; FFT_SIZE]) {
        let current_idx = self.write_idx.load(Ordering::Relaxed);
        for (i, d) in dest.iter_mut().enumerate() {
            let idx = (current_idx.wrapping_add(VISUALIZER_BUFFER_SIZE - FFT_SIZE + i)) % VISUALIZER_BUFFER_SIZE;
            *d = f32::from_bits(self.buffer[idx].load(Ordering::Relaxed));
        }
    }
}

// 48-Band High Precision FFT Real-Time Spectrum Analyzer
pub struct FftAnalyzer {
    hann_window: [f32; FFT_SIZE],
    band_bin_ranges: [(usize, usize); NUM_BANDS],
    band_weights: [f32; NUM_BANDS],
}

impl FftAnalyzer {
    pub fn new(sample_rate: f32) -> Self {
        // 1. Precalculate Hann window
        let mut hann_window = [0.0; FFT_SIZE];
        let two_pi = 2.0 * std::f32::consts::PI;
        for (i, w) in hann_window.iter_mut().enumerate() {
            *w = 0.5 * (1.0 - (two_pi * (i as f32) / (FFT_SIZE as f32 - 1.0)).cos());
        }

        // 2. Precalculate 48 Logarithmic frequency bands from 20 Hz to 20,000 Hz
        let mut band_bin_ranges = [(0, 0); NUM_BANDS];
        let mut band_weights = [1.0; NUM_BANDS];
        let sr = sample_rate.max(44100.0);
        let bin_hz = sr / (FFT_SIZE as f32); // ~43.0 - 46.8 Hz per bin

        let f_min = 20.0_f32;
        let f_max = (sr * 0.48).min(20000.0);
        let log_ratio = (f_max / f_min).ln() / (NUM_BANDS as f32);

        for (i, range) in band_bin_ranges.iter_mut().enumerate() {
            let f_start = f_min * (log_ratio * i as f32).exp();
            let f_end = f_min * (log_ratio * (i + 1) as f32).exp();

            let bin_start = ((f_start / bin_hz).floor() as usize).clamp(1, FFT_SIZE / 2 - 1);
            let bin_end = ((f_end / bin_hz).ceil() as usize).clamp(bin_start, FFT_SIZE / 2 - 1);

            *range = (bin_start, bin_end);

            // Equal loudness / visual balance weighting (Pink noise tilt: gentle boost for higher frequencies)
            let center_f = (f_start * f_end).sqrt();
            let tilt = (center_f / 1000.0).log10() * 0.45;
            band_weights[i] = (1.0 + tilt).clamp(0.6, 2.2);
        }

        Self {
            hann_window,
            band_bin_ranges,
            band_weights,
        }
    }

    pub fn compute_bands(&self, raw_samples: &[f32; FFT_SIZE], bands_out: &mut [f32; NUM_BANDS]) {
        let mut re = [0.0f32; FFT_SIZE];
        let mut im = [0.0f32; FFT_SIZE];

        // Apply Hann window
        for i in 0..FFT_SIZE {
            re[i] = raw_samples[i] * self.hann_window[i];
            im[i] = 0.0;
        }

        // Run In-Place Cooley-Tukey Radix-2 FFT
        Self::fft_inplace(&mut re, &mut im);

        // Group into 48 bands with ballistic decay smoothing
        for i in 0..NUM_BANDS {
            let (start_bin, end_bin) = self.band_bin_ranges[i];
            let mut max_mag = 0.0f32;

            for bin in start_bin..=end_bin {
                let mag = (re[bin] * re[bin] + im[bin] * im[bin]).sqrt();
                if mag > max_mag {
                    max_mag = mag;
                }
            }

            // Normalize and scale with band weight
            let raw_level = (max_mag / (FFT_SIZE as f32 * 0.25)) * self.band_weights[i];
            // Non-linear perceptual scaling (cubic root for warm dynamic response)
            let target_level = (raw_level.clamp(0.0, 1.0)).powf(0.55);

            // Ballistic decay: instantaneous attack, smooth decay
            if target_level > bands_out[i] {
                bands_out[i] = target_level;
            } else {
                bands_out[i] = (bands_out[i] * 0.82).max(0.0);
            }
        }
    }

    // In-place Cooley-Tukey Radix-2 DIT FFT
    fn fft_inplace(re: &mut [f32; FFT_SIZE], im: &mut [f32; FFT_SIZE]) {
        let n = FFT_SIZE;
        // Bit-reversal permutation
        let mut j = 0;
        for i in 0..n {
            if i < j {
                re.swap(i, j);
                im.swap(i, j);
            }
            let mut bit = n >> 1;
            while bit > 0 && (j & bit) != 0 {
                j &= !bit;
                bit >>= 1;
            }
            j |= bit;
        }

        // Butterfly stages
        let mut len = 2;
        while len <= n {
            let half = len / 2;
            let angle = -2.0 * std::f32::consts::PI / (len as f32);
            let w_step_re = angle.cos();
            let w_step_im = angle.sin();

            let mut i = 0;
            while i < n {
                let mut w_re = 1.0;
                let mut w_im = 0.0;
                for k in 0..half {
                    let u_re = re[i + k];
                    let u_im = im[i + k];
                    let v_re = re[i + k + half] * w_re - im[i + k + half] * w_im;
                    let v_im = re[i + k + half] * w_im + im[i + k + half] * w_re;

                    re[i + k] = u_re + v_re;
                    im[i + k] = u_im + v_im;
                    re[i + k + half] = u_re - v_re;
                    im[i + k + half] = u_im - v_im;

                    let next_w_re = w_re * w_step_re - w_im * w_step_im;
                    let next_w_im = w_re * w_step_im + w_im * w_step_re;
                    w_re = next_w_re;
                    w_im = next_w_im;
                }
                i += len;
            }
            len <<= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_buffer() {
        let buf = VisualizerRingBuffer::new();
        for i in 0..FFT_SIZE {
            buf.push_sample(i as f32);
        }

        let mut dest = [0.0f32; FFT_SIZE];
        buf.get_latest_samples(&mut dest);
        assert_eq!(dest[0], 0.0);
        assert_eq!(dest[FFT_SIZE - 1], (FFT_SIZE - 1) as f32);
    }

    #[test]
    fn test_fft_analyzer_sine_wave() {
        let analyzer = FftAnalyzer::new(48000.0);
        let mut samples = [0.0f32; FFT_SIZE];

        // Generate 440 Hz sine wave
        for (i, s) in samples.iter_mut().enumerate() {
            let t = i as f32 / 48000.0;
            *s = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        }

        let mut bands = [0.0f32; NUM_BANDS];
        analyzer.compute_bands(&samples, &mut bands);

        // Verify that bands around 440 Hz have substantial energy, while far bands are low
        let max_band_idx = bands
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        // 440 Hz should fall in the lower-mid bands (typically index 18-24 out of 48)
        assert!(max_band_idx >= 15 && max_band_idx <= 26);
        assert!(bands[max_band_idx] > 0.2);
    }
}

