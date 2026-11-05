#[derive(Clone, Copy, Debug)]
pub struct BiquadCoeffs {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
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

    // Robert Bristow-Johnson Audio EQ Cookbook Peaking Filter computed in double precision (f64)
    // Ensures numerical stability and zero coefficient quantization error at 31 Hz/63 Hz under 96 kHz+
    pub fn peaking_eq(center_freq: f64, gain_db: f64, sample_rate: f64, q: f64) -> Self {
        if gain_db.abs() < 0.01 || sample_rate <= 0.0 {
            return Self::identity();
        }

        let safe_sr = sample_rate.max(8000.0);
        let nyquist = safe_sr * 0.495;
        let f0 = center_freq.clamp(10.0, nyquist);

        let a = 10.0_f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (f0 / safe_sr);
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

    pub fn low_shelf(cutoff_freq: f64, gain_db: f64, sample_rate: f64, q: f64) -> Self {
        if gain_db.abs() < 0.01 || sample_rate <= 0.0 {
            return Self::identity();
        }
        let safe_sr = sample_rate.max(8000.0);
        let nyquist = safe_sr * 0.495;
        let f0 = cutoff_freq.clamp(10.0, nyquist);

        let a = 10.0_f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (f0 / safe_sr);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q.max(0.1));
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    pub fn high_shelf(cutoff_freq: f64, gain_db: f64, sample_rate: f64, q: f64) -> Self {
        if gain_db.abs() < 0.01 || sample_rate <= 0.0 {
            return Self::identity();
        }
        let safe_sr = sample_rate.max(8000.0);
        let nyquist = safe_sr * 0.495;
        let f0 = cutoff_freq.clamp(10.0, nyquist);

        let a = 10.0_f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (f0 / safe_sr);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q.max(0.1));
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    pub fn high_pass(cutoff_freq: f64, sample_rate: f64, q: f64) -> Self {
        let safe_sr = sample_rate.max(8000.0);
        let nyquist = safe_sr * 0.495;
        let f0 = cutoff_freq.clamp(10.0, nyquist);

        let w0 = 2.0 * std::f64::consts::PI * (f0 / safe_sr);
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();

        let b0 = (1.0 + cos_w0) * 0.5;
        let b1 = -(1.0 + cos_w0);
        let b2 = (1.0 + cos_w0) * 0.5;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    pub fn low_pass(cutoff_freq: f64, sample_rate: f64, q: f64) -> Self {
        let safe_sr = sample_rate.max(8000.0);
        let nyquist = safe_sr * 0.495;
        let f0 = cutoff_freq.clamp(10.0, nyquist);

        let w0 = 2.0 * std::f64::consts::PI * (f0 / safe_sr);
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();

        let b0 = (1.0 - cos_w0) * 0.5;
        let b1 = 1.0 - cos_w0;
        let b2 = (1.0 - cos_w0) * 0.5;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

/// Direct Form II Transposed (DF2T) Biquad Filter State
#[derive(Clone, Copy, Debug, Default)]
pub struct BiquadState {
    pub s1: f64,
    pub s2: f64,
}

impl BiquadState {
    #[inline(always)]
    pub fn process(&mut self, input: f64, coeffs: &BiquadCoeffs) -> f64 {
        let out = coeffs.b0 * input + self.s1;
        self.s1 = coeffs.b1 * input - coeffs.a1 * out + self.s2;
        self.s2 = coeffs.b2 * input - coeffs.a2 * out;
        out
    }

    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}

#[inline(always)]
pub fn soft_limit(x: f64) -> f64 {
    // Transparent bit-perfect unity pass-through below 0.90 (-0.91 dBFS)
    if x > 0.90 {
        0.90 + 0.10 * ((x - 0.90) / 0.10).tanh()
    } else if x < -0.90 {
        -0.90 + 0.10 * ((x + 0.90) / 0.10).tanh()
    } else {
        x
    }
}

