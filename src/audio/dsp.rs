// 10-Band Graphic EQ Frequencies matching LunulaPlayer
pub const EQ_FREQUENCIES: [f32; 10] = [
    31.0, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

#[derive(Clone, Copy, Debug)]
pub struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadCoeffs {
    pub fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    // Robert Bristow-Johnson Audio EQ Cookbook Peaking Filter
    pub fn peaking_eq(center_freq: f32, gain_db: f32, sample_rate: f32, q: f32) -> Self {
        if gain_db.abs() < 0.05 || sample_rate <= 0.0 {
            return Self::identity();
        }

        // Clamp center frequency below Nyquist
        let nyquist = sample_rate * 0.49;
        let f0 = center_freq.clamp(20.0, nyquist);

        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (f0 / sample_rate);
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    #[inline(always)]
    pub fn process(&mut self, input: f32, coeffs: &BiquadCoeffs) -> f32 {
        let out = coeffs.b0 * input + coeffs.b1 * self.x1 + coeffs.b2 * self.x2
            - coeffs.a1 * self.y1
            - coeffs.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = out;

        out
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

pub struct DspPipeline {
    sample_rate: f32,
    eq_coeffs: [BiquadCoeffs; 10],
    eq_state_l: [BiquadState; 10],
    eq_state_r: [BiquadState; 10],
    eq_gains: [f32; 10],
    eq_bypass: bool,
    volume: f32,
}

impl DspPipeline {
    pub fn new(sample_rate: f32) -> Self {
        let mut dsp = Self {
            sample_rate: sample_rate.max(44100.0),
            eq_coeffs: [BiquadCoeffs::identity(); 10],
            eq_state_l: [BiquadState::default(); 10],
            eq_state_r: [BiquadState::default(); 10],
            eq_gains: [0.0; 10],
            eq_bypass: true,
            volume: 0.80,
        };
        dsp.recompute_all_eq_coeffs();
        dsp
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() > 1.0 {
            self.sample_rate = sample_rate.max(44100.0);
            self.recompute_all_eq_coeffs();
            self.reset_states();
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.5);
    }

    pub fn set_eq_band(&mut self, band_idx: usize, gain_db: f32) {
        if band_idx < 10 {
            self.eq_gains[band_idx] = gain_db.clamp(-12.0, 12.0);
            self.recompute_band_coeffs(band_idx);
        }
    }

    pub fn set_eq_bypass(&mut self, bypass: bool) {
        self.eq_bypass = bypass;
        if bypass {
            self.reset_states();
        }
    }

    fn recompute_band_coeffs(&mut self, band_idx: usize) {
        let f0 = EQ_FREQUENCIES[band_idx];
        let gain = self.eq_gains[band_idx];
        // Standard graphic EQ Q factor ~ 1.414 (one octave bandwidth)
        self.eq_coeffs[band_idx] = BiquadCoeffs::peaking_eq(f0, gain, self.sample_rate, 1.414);
    }

    fn recompute_all_eq_coeffs(&mut self) {
        for i in 0..10 {
            self.recompute_band_coeffs(i);
        }
    }

    pub fn reset_states(&mut self) {
        for i in 0..10 {
            self.eq_state_l[i].reset();
            self.eq_state_r[i].reset();
        }
    }

    #[inline(always)]
    pub fn process_frame(&mut self, mut l: f32, mut r: f32) -> (f32, f32) {
        // 10-Band Peaking Graphic EQ
        if !self.eq_bypass {
            for i in 0..10 {
                l = self.eq_state_l[i].process(l, &self.eq_coeffs[i]);
                r = self.eq_state_r[i].process(r, &self.eq_coeffs[i]);
            }
        }

        // Bit-perfect safety clamp: transparent [-1.0, 1.0], zero dynamic compression or double attenuation
        (l.clamp(-1.0, 1.0), r.clamp(-1.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsp_volume_and_bypass() {
        let mut dsp = DspPipeline::new(44100.0);
        dsp.set_volume(0.5);
        dsp.set_eq_bypass(true);

        let (l, r) = dsp.process_frame(0.4, -0.6);
        assert!((l - 0.4).abs() < 1e-4);
        assert!((r - (-0.6)).abs() < 1e-4);
    }

    #[test]
    fn test_transparent_clamping() {
        let mut dsp = DspPipeline::new(44100.0);
        dsp.set_eq_bypass(true);
        let (l, r) = dsp.process_frame(1.5, -1.5);
        assert_eq!(l, 1.0);
        assert_eq!(r, -1.0);
    }
}
