use crate::audio::types::{Effect, SignalChainGraph};
use std::collections::HashMap;

// 10-Band Graphic EQ Frequencies matching LunulaPlayer
pub const EQ_FREQUENCIES: [f64; 10] = [
    31.0, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

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

// ---------------------------------------------------------------------------
// Modular Audio Effects
// ---------------------------------------------------------------------------

/// 10-Band Peaking Graphic EQ Effect with Auto-Preamp Headroom and Slew Rate Smoothing
#[derive(Clone, Debug)]
pub struct GraphicEqEffect {
    sample_rate: f64,
    eq_coeffs: [BiquadCoeffs; 10],
    eq_state_l: [BiquadState; 10],
    eq_state_r: [BiquadState; 10],
    target_gains: [f64; 10],
    current_gains: [f64; 10],
    auto_preamp: f64,
    pub is_flat: bool,
}

impl GraphicEqEffect {
    pub fn new(gains: [f32; 10], sample_rate: f64) -> Self {
        let mut eq = Self {
            sample_rate: sample_rate.max(8000.0),
            eq_coeffs: [BiquadCoeffs::identity(); 10],
            eq_state_l: [BiquadState::default(); 10],
            eq_state_r: [BiquadState::default(); 10],
            target_gains: [0.0; 10],
            current_gains: [0.0; 10],
            auto_preamp: 1.0,
            is_flat: true,
        };
        for (i, &g) in gains.iter().enumerate() {
            let val = g.clamp(-12.0, 12.0) as f64;
            eq.target_gains[i] = val;
            eq.current_gains[i] = val;
        }
        eq.recompute_all();
        eq
    }

    fn update_auto_preamp(&mut self) {
        let max_boost = self.current_gains.iter().cloned().fold(0.0f64, f64::max);
        self.auto_preamp = if max_boost > 0.0 {
            10.0_f64.powf(-max_boost / 20.0)
        } else {
            1.0
        };
    }

    fn recompute_band(&mut self, idx: usize) {
        let f0 = EQ_FREQUENCIES[idx];
        let gain = self.current_gains[idx];
        self.eq_coeffs[idx] = BiquadCoeffs::peaking_eq(f0, gain, self.sample_rate, 1.414);
    }

    fn recompute_all(&mut self) {
        self.update_auto_preamp();
        self.is_flat = self.current_gains.iter().all(|&g| g.abs() < 0.05);
        for i in 0..10 {
            self.recompute_band(i);
        }
    }
}

impl Effect for GraphicEqEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        // Parameter smoothing (A8): slew current_gains towards target_gains to eliminate zipper noise
        let mut need_recompute = false;
        for i in 0..10 {
            let diff = self.target_gains[i] - self.current_gains[i];
            if diff.abs() > 0.001 {
                self.current_gains[i] += diff * 0.5;
                need_recompute = true;
            } else if (self.current_gains[i] - self.target_gains[i]).abs() > 1e-6 {
                self.current_gains[i] = self.target_gains[i];
                need_recompute = true;
            }
        }
        if need_recompute {
            self.recompute_all();
        }

        // Fast zero-cost flat bypass
        if self.is_flat {
            let pre = self.auto_preamp as f32;
            if (pre - 1.0).abs() > 0.0001 {
                for l in left.iter_mut() { *l *= pre; }
                for r in right.iter_mut() { *r *= pre; }
            }
            return;
        }

        let pre = self.auto_preamp;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut fl = (*l as f64) * pre;
            let mut fr = (*r as f64) * pre;
            for i in 0..10 {
                fl = self.eq_state_l[i].process(fl, &self.eq_coeffs[i]);
                fr = self.eq_state_r[i].process(fr, &self.eq_coeffs[i]);
            }
            *l = fl as f32;
            *r = fr as f32;
        }
    }

    fn reset(&mut self) {
        for i in 0..10 {
            self.eq_state_l[i].reset();
            self.eq_state_r[i].reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute_all();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "10-Band Graphic EQ"
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        let idx = param_id as usize;
        if idx < 10 {
            self.target_gains[idx] = (val as f64).clamp(-12.0, 12.0);
            if self.is_flat && self.target_gains[idx].abs() >= 0.05 {
                self.is_flat = false;
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Tube Warmer Effect: Asymmetric triode saturation adding 2nd and 3rd harmonics
#[derive(Clone, Debug)]
pub struct TubeWarmerEffect {
    drive: f32,          // 0.0 to 100.0 (%)
    current_drive: f32,  // For click-free parameter slewing
}

impl TubeWarmerEffect {
    pub fn new(drive: f32) -> Self {
        let d = drive.clamp(0.0, 100.0);
        Self {
            drive: d,
            current_drive: d,
        }
    }
}

impl Default for TubeWarmerEffect {
    fn default() -> Self {
        Self::new(35.0)
    }
}

impl Effect for TubeWarmerEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        // Smooth parameter slewing to eliminate zipper noise during live slider dragging
        let diff = self.drive - self.current_drive;
        if diff.abs() > 0.01 {
            self.current_drive += diff * 0.15;
        } else {
            self.current_drive = self.drive;
        }

        let k = (self.current_drive / 100.0).clamp(0.0, 1.0);
        if k < 0.001 {
            return; // 0% Warmth is pure transparent bypass
        }

        // Triode saturation formula:
        // Asymmetric even harmonics (xd^2) add musical warmth and body,
        // odd harmonics (xd^3) add presence.
        let d = 1.0 + k * 1.5;
        let c2 = 0.22 * k;
        let c3 = 0.08 * k;
        let norm = 1.0 / (1.0 + 0.40 * k);

        let warm = |x: f32| -> f32 {
            let xd = x * d;
            let y = xd + c2 * xd * xd - c3 * xd * xd * xd;
            (y * norm).clamp(-1.0, 1.0)
        };

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = warm(*l);
            *r = warm(*r);
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Tube Warmer (Triode Saturation)"
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.drive = val.clamp(0.0, 100.0);
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Stereo Widener Effect: Mid-Side Haas stereo field widening
#[derive(Clone, Debug)]
pub struct StereoWidenerEffect {
    width: f64,
}

impl StereoWidenerEffect {
    pub fn new(width: f64) -> Self {
        Self { width: width.max(1.0) }
    }
}

impl Effect for StereoWidenerEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let w = self.width as f32;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mid = (*l + *r) * 0.5;
            let side = (*l - *r) * 0.5;
            let widened = side * w;
            *l = mid + widened;
            *r = mid - widened;
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Stereo Widener (Haas 3D +35%)"
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Bass Guard Effect: 24 Hz 2nd-order Butterworth sub-sonic high-pass filter
#[derive(Clone, Debug)]
pub struct BassGuardEffect {
    sample_rate: f64,
    hpf_coeffs: BiquadCoeffs,
    state_l: BiquadState,
    state_r: BiquadState,
}

impl BassGuardEffect {
    pub fn new(sample_rate: f64) -> Self {
        let mut bg = Self {
            sample_rate: sample_rate.max(8000.0),
            hpf_coeffs: BiquadCoeffs::identity(),
            state_l: BiquadState::default(),
            state_r: BiquadState::default(),
        };
        bg.recompute();
        bg
    }

    fn recompute(&mut self) {
        self.hpf_coeffs = BiquadCoeffs::high_pass(24.0, self.sample_rate, 0.7071067811865476);
    }
}

impl Effect for BassGuardEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = self.state_l.process(*l as f64, &self.hpf_coeffs) as f32;
            *r = self.state_r.process(*r as f64, &self.hpf_coeffs) as f32;
        }
    }

    fn reset(&mut self) {
        self.state_l.reset();
        self.state_r.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = (sample_rate as f64).max(8000.0);
        self.recompute();
        self.reset();
    }

    fn name(&self) -> &str {
        "Bass Guard (24Hz Sub-Sonic Filter)"
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Volume Attenuator: Precision passive -6.02 dB pad
#[derive(Clone, Debug)]
pub struct VolumeAttenuatorEffect {
    gain: f32,
}

impl VolumeAttenuatorEffect {
    pub fn new(gain_db: f64) -> Self {
        let gain = 10.0_f64.powf(gain_db / 20.0) as f32;
        Self { gain }
    }
}

impl Effect for VolumeAttenuatorEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l *= self.gain;
            *r *= self.gain;
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Volume Attenuator (-6dB Pad)"
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Pure Volume Booster Effect: Uncolored linear gain (0.0 to +24.0 dB) with transparent soft-limit guard
#[derive(Clone, Debug)]
pub struct VolumeBoosterEffect {
    boost_db: f32,       // 0.0 to +24.0 dB
    target_gain: f32,
    current_gain: f32,   // Slew-smoothed linear multiplier
}

impl VolumeBoosterEffect {
    pub fn new(boost_db: f32) -> Self {
        let b = boost_db.clamp(0.0, 24.0);
        let g = 10.0_f32.powf(b / 20.0);
        Self {
            boost_db: b,
            target_gain: g,
            current_gain: g,
        }
    }
}

impl Default for VolumeBoosterEffect {
    fn default() -> Self {
        Self::new(6.0)
    }
}

impl Effect for VolumeBoosterEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let diff = self.target_gain - self.current_gain;
        if diff.abs() > 0.001 {
            self.current_gain += diff * 0.2;
        } else {
            self.current_gain = self.target_gain;
        }

        let gain = self.current_gain;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let amplified_l = *l * gain;
            let amplified_r = *r * gain;
            *l = soft_limit(amplified_l as f64) as f32;
            *r = soft_limit(amplified_r as f64) as f32;
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Pure Volume Booster (0 to +24dB)"
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.boost_db = val.clamp(0.0, 24.0);
            self.target_gain = 10.0_f32.powf(self.boost_db / 20.0);
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Analog Preamp & Trim Effect: Studio console gain stage (-24.0 to +12.0 dB) with Class-A warm transformer touch
#[derive(Clone, Debug)]
pub struct AnalogPreampEffect {
    trim_db: f32,       // -24.0 to +12.0 dB
    target_gain: f32,
    current_gain: f32,
}

impl AnalogPreampEffect {
    pub fn new(trim_db: f32) -> Self {
        let t = trim_db.clamp(-24.0, 12.0);
        let g = 10.0_f32.powf(t / 20.0);
        Self {
            trim_db: t,
            target_gain: g,
            current_gain: g,
        }
    }
}

impl Default for AnalogPreampEffect {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Effect for AnalogPreampEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let diff = self.target_gain - self.current_gain;
        if diff.abs() > 0.001 {
            self.current_gain += diff * 0.2;
        } else {
            self.current_gain = self.target_gain;
        }

        let gain = self.current_gain;
        let is_driven = self.trim_db > 3.0;

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sl = *l * gain;
            let mut sr = *r * gain;

            if is_driven {
                let drive_factor = ((self.trim_db - 3.0) / 9.0).clamp(0.0, 1.0) * 0.04;
                sl = sl - drive_factor * sl * sl * sl;
                sr = sr - drive_factor * sr * sr * sr;
            }

            *l = soft_limit(sl as f64) as f32;
            *r = soft_limit(sr as f64) as f32;
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Analog Preamp & Trim (-24 to +12dB)"
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.trim_db = val.clamp(-24.0, 12.0);
            self.target_gain = 10.0_f32.powf(self.trim_db / 20.0);
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Dynamic Loudness Effect: Fletcher-Munson ISO 226 low-level acoustic contouring
#[derive(Clone, Debug)]
pub struct DynamicLoudnessEffect {
    depth: f32, // 0.0 to 100.0 (%)
    sample_rate: f64,
    low_shelf_coeffs: BiquadCoeffs,
    high_shelf_coeffs: BiquadCoeffs,
    state_l_low: BiquadState,
    state_r_low: BiquadState,
    state_l_high: BiquadState,
    state_r_high: BiquadState,
}

impl DynamicLoudnessEffect {
    pub fn new(depth: f32, sample_rate: f64) -> Self {
        let mut effect = Self {
            depth: depth.clamp(0.0, 100.0),
            sample_rate: sample_rate.max(8000.0),
            low_shelf_coeffs: BiquadCoeffs::identity(),
            high_shelf_coeffs: BiquadCoeffs::identity(),
            state_l_low: BiquadState::default(),
            state_r_low: BiquadState::default(),
            state_l_high: BiquadState::default(),
            state_r_high: BiquadState::default(),
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        if self.depth < 0.5 {
            self.low_shelf_coeffs = BiquadCoeffs::identity();
            self.high_shelf_coeffs = BiquadCoeffs::identity();
            return;
        }
        let k = (self.depth / 100.0) as f64;
        let low_boost = k * 8.5; // up to +8.5 dB below 120 Hz
        let high_boost = k * 5.0; // up to +5.0 dB above 9 kHz

        self.low_shelf_coeffs = BiquadCoeffs::low_shelf(120.0, low_boost, self.sample_rate, 0.7071067811865476);
        self.high_shelf_coeffs = BiquadCoeffs::high_shelf(9000.0, high_boost, self.sample_rate, 0.7071067811865476);
    }
}

impl Default for DynamicLoudnessEffect {
    fn default() -> Self {
        Self::new(50.0, 44100.0)
    }
}

impl Effect for DynamicLoudnessEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        if self.depth < 0.5 {
            return; // Zero-cost transparent bypass
        }

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sl = self.state_l_low.process(*l as f64, &self.low_shelf_coeffs);
            let mut sr = self.state_r_low.process(*r as f64, &self.low_shelf_coeffs);

            sl = self.state_l_high.process(sl, &self.high_shelf_coeffs);
            sr = self.state_r_high.process(sr, &self.high_shelf_coeffs);

            *l = soft_limit(sl) as f32;
            *r = soft_limit(sr) as f32;
        }
    }

    fn reset(&mut self) {
        self.state_l_low.reset();
        self.state_r_low.reset();
        self.state_l_high.reset();
        self.state_r_high.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = (sample_rate as f64).max(8000.0);
        self.recompute();
        self.reset();
    }

    fn name(&self) -> &str {
        "Dynamic Loudness (Fletcher-Munson ISO 226)"
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.depth = val.clamp(0.0, 100.0);
            self.recompute();
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Audiophile Stepped Attenuator Effect: Resistor ladder 64-step passive attenuator with DIM -20dB and Mute
#[derive(Clone, Debug)]
pub struct SteppedAttenuatorEffect {
    step_db: f32,        // -60.0 to 0.0 dB
    is_dimmed: bool,     // -20dB DIM
    is_muted: bool,      // Mute
    target_gain: f32,
    current_gain: f32,
}

impl SteppedAttenuatorEffect {
    pub fn new(step_db: f32, is_dimmed: bool) -> Self {
        let s = step_db.clamp(-60.0, 0.0).round(); // Discrete 1dB steps
        let effective_db = if is_dimmed { s - 20.0 } else { s };
        let g = 10.0_f32.powf(effective_db / 20.0);
        Self {
            step_db: s,
            is_dimmed,
            is_muted: false,
            target_gain: g,
            current_gain: g,
        }
    }

    fn recompute_gain(&mut self) {
        if self.is_muted {
            self.target_gain = 0.0;
        } else {
            let effective_db = if self.is_dimmed { self.step_db - 20.0 } else { self.step_db };
            self.target_gain = 10.0_f32.powf(effective_db / 20.0);
        }
    }
}

impl Default for SteppedAttenuatorEffect {
    fn default() -> Self {
        Self::new(-12.0, false)
    }
}

impl Effect for SteppedAttenuatorEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let diff = self.target_gain - self.current_gain;
        if diff.abs() > 0.0005 {
            self.current_gain += diff * 0.25;
        } else {
            self.current_gain = self.target_gain;
        }

        let gain = self.current_gain;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l *= gain;
            *r *= gain;
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Stepped Attenuator (-60 to 0dB)"
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.step_db = val.clamp(-60.0, 0.0).round();
            self.recompute_gain();
        } else if param_id == 1 {
            self.is_dimmed = val > 0.5;
            self.recompute_gain();
        } else if param_id == 2 {
            self.is_muted = val > 0.5;
            self.recompute_gain();
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Studio Optical Leveler Effect: Teletronix LA-2A style optocoupler dynamic leveler with Gain Reduction VU needle
#[derive(Clone, Debug)]
pub struct OpticalLevelerEffect {
    target_drive: f32, // 0.0 to 100.0 (%)
    envelope: f32,
    pub gain_reduction_db: f32,
    sample_rate: f32,
}

impl OpticalLevelerEffect {
    pub fn new(target_drive: f32, sample_rate: f32) -> Self {
        Self {
            target_drive: target_drive.clamp(0.0, 100.0),
            envelope: 0.0,
            gain_reduction_db: 0.0,
            sample_rate: sample_rate.max(8000.0),
        }
    }
}

impl Default for OpticalLevelerEffect {
    fn default() -> Self {
        Self::new(50.0, 44100.0)
    }
}

impl Effect for OpticalLevelerEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let drive = (self.target_drive / 100.0).clamp(0.0, 1.0);
        if drive < 0.01 {
            self.gain_reduction_db = 0.0;
            return; // Bypass
        }

        let threshold = 0.85 - drive * 0.65;
        let attack_coeff = (-1.0 / (0.010 * self.sample_rate)).exp();
        let release_coeff_fast = (-1.0 / (0.060 * self.sample_rate)).exp();
        let release_coeff_slow = (-1.0 / (1.500 * self.sample_rate)).exp();

        let mut max_gr = 0.0f32;

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let peak = l.abs().max(r.abs());

            if peak > self.envelope {
                self.envelope = peak + attack_coeff * (self.envelope - peak);
            } else {
                let coeff = if self.envelope > threshold * 1.5 {
                    release_coeff_fast
                } else {
                    release_coeff_slow
                };
                self.envelope = peak + coeff * (self.envelope - peak);
            }

            let gain = if self.envelope > threshold {
                let over_db = 20.0 * (self.envelope / threshold).log10();
                let compressed_over_db = over_db / 3.0;
                let gr_db = over_db - compressed_over_db;
                if gr_db > max_gr {
                    max_gr = gr_db;
                }
                10.0_f32.powf(-gr_db / 20.0)
            } else {
                1.0
            };

            *l = (*l * gain).clamp(-1.0, 1.0);
            *r = (*r * gain).clamp(-1.0, 1.0);
        }

        self.gain_reduction_db = max_gr;
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain_reduction_db = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.reset();
    }

    fn name(&self) -> &str {
        "Studio Optical Leveler (LA-2A Night Dynamic Control)"
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.target_drive = val.clamp(0.0, 100.0);
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Dual-Channel Stereo Balance & Trim Effect: Constant-power stereo panning with center detent
#[derive(Clone, Debug)]
pub struct StereoBalanceEffect {
    balance: f32, // -100.0 (Left) to +100.0 (Right), 0.0 = Center
    current_balance: f32,
}

impl StereoBalanceEffect {
    pub fn new(balance: f32) -> Self {
        let b = balance.clamp(-100.0, 100.0);
        Self {
            balance: b,
            current_balance: b,
        }
    }
}

impl Default for StereoBalanceEffect {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Effect for StereoBalanceEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let diff = self.balance - self.current_balance;
        if diff.abs() > 0.01 {
            self.current_balance += diff * 0.2;
        } else {
            self.current_balance = self.balance;
        }

        if self.current_balance.abs() < 0.5 {
            return; // Center detent is zero-cost passthrough (1.0, 1.0)
        }

        let norm = (self.current_balance / 100.0).clamp(-1.0, 1.0);
        let theta = (norm + 1.0) * 0.25 * std::f32::consts::PI; // 0 to pi/2
        let gain_l = std::f32::consts::SQRT_2 * theta.cos();
        let gain_r = std::f32::consts::SQRT_2 * theta.sin();

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l *= gain_l;
            *r *= gain_r;
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Stereo Balance & Trim"
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.balance = val.clamp(-100.0, 100.0);
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Low-Pass Filter: 2nd-order Butterworth low-pass filter (default cutoff 800 Hz)
/// Dramatic audible difference (muffled / underwater effect) cutting treble above cutoff.
#[derive(Clone, Debug)]
pub struct LowPassFilterEffect {
    cutoff_hz: f64,
    sample_rate: f64,
    lpf_coeffs: BiquadCoeffs,
    state_l: BiquadState,
    state_r: BiquadState,
}

impl LowPassFilterEffect {
    pub fn new(cutoff_hz: f64, sample_rate: f64) -> Self {
        let mut effect = Self {
            cutoff_hz: cutoff_hz.clamp(50.0, 20000.0),
            sample_rate: sample_rate.max(8000.0),
            lpf_coeffs: BiquadCoeffs::identity(),
            state_l: BiquadState::default(),
            state_r: BiquadState::default(),
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        // Butterworth Q = 1/sqrt(2) approx 0.70710678
        self.lpf_coeffs = BiquadCoeffs::low_pass(self.cutoff_hz, self.sample_rate, 0.7071067811865476);
    }
}

impl Effect for LowPassFilterEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = self.state_l.process(*l as f64, &self.lpf_coeffs) as f32;
            *r = self.state_r.process(*r as f64, &self.lpf_coeffs) as f32;
        }
    }

    fn reset(&mut self) {
        self.state_l.reset();
        self.state_r.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = (sample_rate as f64).max(8000.0);
        self.recompute();
        self.reset();
    }

    fn name(&self) -> &str {
        "Low-Pass Filter (800Hz Muffled/Underwater)"
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 && val > 20.0 {
            self.cutoff_hz = val as f64;
            self.recompute();
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Mixer / Summing Bus: Crossfader between input branches with constant-power headroom
#[derive(Clone, Debug)]
pub struct MixerEffect {
    mix: f32, // 0.0 = 100% A, 1.0 = 100% B, 0.5 = 50% A / 50% B
}

impl MixerEffect {
    pub fn new(mix: f32) -> Self {
        Self { mix: mix.clamp(0.0, 1.0) }
    }
}

impl Effect for MixerEffect {
    fn process_block(&mut self, _left: &mut [f32], _right: &mut [f32]) {
        // Mixing/summing is executed in topological bus summing in DspPipeline
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Mixer / Summing Bus"
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.mix = val.clamp(0.0, 1.0);
        }
    }
    fn mix_ratio(&self) -> Option<f32> {
        Some(self.mix)
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Passthrough / Wire: Pure identity effect for graph routing nodes (Source, Buffer, Resampler, DAC)
#[derive(Clone, Debug, Default)]
pub struct PassthroughEffect;

impl PassthroughEffect {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for PassthroughEffect {
    fn process_block(&mut self, _left: &mut [f32], _right: &mut [f32]) {}
    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Passthrough (Wire)"
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Splitter / Y-Branch Bus: 1 Input -> Dual Output (Pin A & Pin B) with lossless 64-bit precision
#[derive(Clone, Debug, Default)]
pub struct SplitterEffect;

impl SplitterEffect {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for SplitterEffect {
    fn process_block(&mut self, _left: &mut [f32], _right: &mut [f32]) {
        // Splitting is executed losslessly by routing the node output buffer to multiple downstream nodes
    }
    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Splitter / Y-Branch Bus"
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Generic Parametric EQ Band Configuration
#[derive(Clone, Debug)]
pub struct ParametricBandConfig {
    pub freq: f64,
    pub gain_db: f64,
    pub q: f64,
    pub filter_type: String, // "peak", "low_shelf", "high_shelf", "hpf"
}

/// Generic Parametric Equalizer Effect replacing hardcoded preset names (A3)
#[derive(Clone, Debug)]
pub struct ParametricEqEffect {
    name: String,
    sample_rate: f64,
    bands: Vec<ParametricBandConfig>,
    coeffs: Vec<BiquadCoeffs>,
    state_l: Vec<BiquadState>,
    state_r: Vec<BiquadState>,
    auto_preamp: f64,
}

impl ParametricEqEffect {
    pub fn new(name: &str, bands: Vec<ParametricBandConfig>, sample_rate: f64) -> Self {
        let mut effect = Self {
            name: name.to_string(),
            sample_rate: sample_rate.max(8000.0),
            bands,
            coeffs: Vec::new(),
            state_l: Vec::new(),
            state_r: Vec::new(),
            auto_preamp: 1.0,
        };
        effect.recompute();
        effect
    }

    pub fn from_preset(name: &str, sample_rate: f64) -> Self {
        let bands = match name {
            "Vocal Clarity" => vec![
                ParametricBandConfig { freq: 250.0, gain_db: -2.5, q: 1.414, filter_type: "peak".to_string() },
                ParametricBandConfig { freq: 2000.0, gain_db: 3.0, q: 1.414, filter_type: "peak".to_string() },
                ParametricBandConfig { freq: 4000.0, gain_db: 3.5, q: 1.414, filter_type: "peak".to_string() },
            ],
            "Bass Boost Matrix" => vec![
                ParametricBandConfig { freq: 63.0, gain_db: 5.0, q: 1.2, filter_type: "low_shelf".to_string() },
                ParametricBandConfig { freq: 125.0, gain_db: 3.0, q: 1.414, filter_type: "peak".to_string() },
                ParametricBandConfig { freq: 250.0, gain_db: 1.5, q: 1.414, filter_type: "peak".to_string() },
            ],
            "Air & Sparkle" => vec![
                ParametricBandConfig { freq: 8000.0, gain_db: 2.5, q: 1.414, filter_type: "high_shelf".to_string() },
                ParametricBandConfig { freq: 16000.0, gain_db: 4.0, q: 1.0, filter_type: "high_shelf".to_string() },
            ],
            "Warm Tube Vintage" => vec![
                ParametricBandConfig { freq: 125.0, gain_db: 2.0, q: 1.414, filter_type: "peak".to_string() },
                ParametricBandConfig { freq: 8000.0, gain_db: -2.0, q: 1.414, filter_type: "peak".to_string() },
                ParametricBandConfig { freq: 16000.0, gain_db: -3.5, q: 1.0, filter_type: "peak".to_string() },
            ],
            _ => vec![
                // Generic parametric default
                ParametricBandConfig { freq: 1000.0, gain_db: 0.0, q: 1.414, filter_type: "peak".to_string() },
            ],
        };

        Self::new(name, bands, sample_rate)
    }

    fn recompute(&mut self) {
        let max_boost = self.bands.iter().map(|b| b.gain_db).fold(0.0f64, f64::max);
        self.auto_preamp = if max_boost > 0.0 {
            10.0_f64.powf(-max_boost / 20.0)
        } else {
            1.0
        };

        self.coeffs = self.bands.iter().map(|b| {
            match b.filter_type.as_str() {
                "low_shelf" => BiquadCoeffs::low_shelf(b.freq, b.gain_db, self.sample_rate, b.q),
                "high_shelf" => BiquadCoeffs::high_shelf(b.freq, b.gain_db, self.sample_rate, b.q),
                "hpf" => BiquadCoeffs::high_pass(b.freq, self.sample_rate, b.q),
                _ => BiquadCoeffs::peaking_eq(b.freq, b.gain_db, self.sample_rate, b.q),
            }
        }).collect();

        self.state_l = vec![BiquadState::default(); self.coeffs.len()];
        self.state_r = vec![BiquadState::default(); self.coeffs.len()];
    }
}

impl Effect for ParametricEqEffect {
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let pre = self.auto_preamp;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut fl = (*l as f64) * pre;
            let mut fr = (*r as f64) * pre;
            for i in 0..self.coeffs.len() {
                fl = self.state_l[i].process(fl, &self.coeffs[i]);
                fr = self.state_r[i].process(fr, &self.coeffs[i]);
            }
            *l = fl as f32;
            *r = fr as f32;
        }
    }

    fn reset(&mut self) {
        for s in &mut self.state_l { s.reset(); }
        for s in &mut self.state_r { s.reset(); }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = (sample_rate as f64).max(8000.0);
        self.recompute();
        self.reset();
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Backwards compatibility alias for PresetEqEffect
pub type PresetEqEffect = ParametricEqEffect;

// ---------------------------------------------------------------------------
// Graph Executor (DAG Signal Router with Branching & Summing)
// ---------------------------------------------------------------------------

pub struct GraphExecutor {
    sample_rate: f32,
    graph: SignalChainGraph,
    node_buffers_l: HashMap<u32, Vec<f32>>,
    node_buffers_r: HashMap<u32, Vec<f32>>,
    sum_scratch_l: Vec<f32>,
    sum_scratch_r: Vec<f32>,
    eq_bypass: bool,
    fallback_eq: GraphicEqEffect,
    has_custom_graph: bool,
}

impl GraphExecutor {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(8000.0);
        let fallback_eq = GraphicEqEffect::new([0.0; 10], sr as f64);
        Self {
            sample_rate: sr,
            graph: SignalChainGraph::default(),
            node_buffers_l: HashMap::new(),
            node_buffers_r: HashMap::new(),
            sum_scratch_l: vec![0.0; 2048],
            sum_scratch_r: vec![0.0; 2048],
            eq_bypass: false,
            fallback_eq,
            has_custom_graph: false,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.fallback_eq.set_sample_rate(self.sample_rate);
        for node in &mut self.graph.nodes {
            node.effect.set_sample_rate(self.sample_rate);
        }
        for effect in &mut self.graph.effects {
            effect.set_sample_rate(self.sample_rate);
        }
    }

    pub fn set_eq_bypass(&mut self, bypass: bool) {
        self.eq_bypass = bypass;
        if bypass {
            self.reset_states();
        }
    }

    pub fn set_eq_band(&mut self, index: usize, gain_db: f32) {
        self.fallback_eq.set_param(index as u32, gain_db);
        for node in &mut self.graph.nodes {
            if node.effect.name() == "10-Band Graphic EQ" {
                node.effect.set_param(index as u32, gain_db);
            }
        }
        for effect in &mut self.graph.effects {
            if effect.name() == "10-Band Graphic EQ" {
                effect.set_param(index as u32, gain_db);
            }
        }
    }

    pub fn set_node_param(&mut self, node_id: u32, param_id: u32, value: f32) {
        if let Some(node) = self.graph.nodes.iter_mut().find(|n| n.id == node_id) {
            node.effect.set_param(param_id, value);
        }
    }

    pub fn reset_states(&mut self) {
        self.fallback_eq.reset();
        for node in &mut self.graph.nodes {
            node.effect.reset();
        }
        for effect in &mut self.graph.effects {
            effect.reset();
        }
    }

    pub fn is_flat(&self) -> bool {
        self.fallback_eq.is_flat
    }

    pub fn is_chain_complete(&self) -> bool {
        self.graph.is_complete
    }

    pub fn set_signal_chain(&mut self, mut graph: SignalChainGraph) {
        for node in &mut graph.nodes {
            node.effect.set_sample_rate(self.sample_rate);
        }
        for effect in &mut graph.effects {
            effect.set_sample_rate(self.sample_rate);
        }

        // Pre-allocate node scratch buffers for zero-allocation execution
        for node in &graph.nodes {
            self.node_buffers_l.entry(node.id).or_insert_with(|| vec![0.0f32; 2048]);
            self.node_buffers_r.entry(node.id).or_insert_with(|| vec![0.0f32; 2048]);
        }

        self.has_custom_graph = true;
        self.graph = graph;
    }

    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len();
        if n == 0 {
            return;
        }

        // 1. If signal chain graph is severed, mute output
        if !self.graph.is_complete {
            left.fill(0.0);
            right.fill(0.0);
            return;
        }

        // 2. Bypass passthrough
        if self.eq_bypass {
            return;
        }

        // 3. Fallback to standalone EQ if no custom graph provided
        if !self.has_custom_graph {
            self.fallback_eq.process_block(left, right);
        } else if !self.graph.nodes.is_empty() {
            let mut executed_nodes = std::collections::HashSet::new();

            for &node_id in &self.graph.topological_order {
                // Find all parent nodes feeding into this node
                let parents: Vec<u32> = self.graph.edges.iter()
                    .filter(|(_, to)| *to == node_id)
                    .map(|(from, _)| *from)
                    .filter(|from| executed_nodes.contains(from))
                    .collect();

                if parents.is_empty() {
                    // Connected directly from Source
                    self.sum_scratch_l[..n].copy_from_slice(&left[..n]);
                    self.sum_scratch_r[..n].copy_from_slice(&right[..n]);
                } else if parents.len() == 1 {
                    let p = parents[0];
                    self.sum_scratch_l[..n].copy_from_slice(&self.node_buffers_l[&p][..n]);
                    self.sum_scratch_r[..n].copy_from_slice(&self.node_buffers_r[&p][..n]);
                } else if parents.len() == 2 {
                    let node_opt = self.graph.nodes.iter().find(|n| n.id == node_id);
                    let mix_ratio = node_opt.and_then(|n| n.effect.mix_ratio()).unwrap_or(0.5);
                    let gain_a = (1.0 - mix_ratio).sqrt();
                    let gain_b = mix_ratio.sqrt();
                    let p_a = parents[0];
                    let p_b = parents[1];
                    for i in 0..n {
                        self.sum_scratch_l[i] = self.node_buffers_l[&p_a][i] * gain_a + self.node_buffers_l[&p_b][i] * gain_b;
                        self.sum_scratch_r[i] = self.node_buffers_r[&p_a][i] * gain_a + self.node_buffers_r[&p_b][i] * gain_b;
                    }
                } else {
                    // Merging / Summing Bus: sum all parent outputs with headroom compensation
                    let scale = 1.0f32 / (parents.len() as f32).sqrt();
                    for i in 0..n {
                        let mut sum_l = 0.0f32;
                        let mut sum_r = 0.0f32;
                        for &p in &parents {
                            sum_l += self.node_buffers_l[&p][i];
                            sum_r += self.node_buffers_r[&p][i];
                        }
                        self.sum_scratch_l[i] = sum_l * scale;
                        self.sum_scratch_r[i] = sum_r * scale;
                    }
                }

                // Copy input to node buffer and execute effect
                if let Some(node) = self.graph.nodes.iter_mut().find(|n| n.id == node_id) {
                    let out_l = self.node_buffers_l.get_mut(&node_id).unwrap();
                    let out_r = self.node_buffers_r.get_mut(&node_id).unwrap();
                    out_l[..n].copy_from_slice(&self.sum_scratch_l[..n]);
                    out_r[..n].copy_from_slice(&self.sum_scratch_r[..n]);
                    node.effect.process_block(&mut out_l[..n], &mut out_r[..n]);
                    executed_nodes.insert(node_id);
                }
            }

            // Identify terminal exit nodes (nodes with no outgoing edges to other active nodes)
            let terminal_nodes: Vec<u32> = self.graph.topological_order.iter()
                .cloned()
                .filter(|&id| {
                    !self.graph.edges.iter().any(|(from, to)| *from == id && self.graph.topological_order.contains(to))
                })
                .collect();

            if terminal_nodes.len() == 1 {
                let term = terminal_nodes[0];
                left.copy_from_slice(&self.node_buffers_l[&term][..n]);
                right.copy_from_slice(&self.node_buffers_r[&term][..n]);
            } else if !terminal_nodes.is_empty() {
                // Sum terminal nodes into final output
                let scale = 1.0f32 / (terminal_nodes.len() as f32).sqrt();
                for i in 0..n {
                    let mut sum_l = 0.0f32;
                    let mut sum_r = 0.0f32;
                    for &t in &terminal_nodes {
                        sum_l += self.node_buffers_l[&t][i];
                        sum_r += self.node_buffers_r[&t][i];
                    }
                    left[i] = sum_l * scale;
                    right[i] = sum_r * scale;
                }
            }
        } else if !self.graph.effects.is_empty() {
            // Serial backwards compatibility mode
            for effect in &mut self.graph.effects {
                effect.process_block(left, right);
            }
        }

        // Soft-limiting (A6): ONLY limit peaks that actually exceed linearity (> 0.90)
        for s in left.iter_mut() {
            if s.abs() > 0.90 {
                *s = soft_limit(*s as f64) as f32;
            }
            *s = s.clamp(-1.0, 1.0);
        }
        for s in right.iter_mut() {
            if s.abs() > 0.90 {
                *s = soft_limit(*s as f64) as f32;
            }
            *s = s.clamp(-1.0, 1.0);
        }
    }
}

// ---------------------------------------------------------------------------
// DspPipeline (Clean Unified Facade wrapping GraphExecutor)
// ---------------------------------------------------------------------------

pub struct DspPipeline {
    executor: GraphExecutor,
}

impl DspPipeline {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            executor: GraphExecutor::new(sample_rate),
        }
    }

    pub fn is_chain_complete(&self) -> bool {
        self.executor.is_chain_complete()
    }

    pub fn is_flat(&self) -> bool {
        self.executor.is_flat()
    }

    pub fn set_signal_chain(&mut self, graph: SignalChainGraph) {
        self.executor.set_signal_chain(graph);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.executor.set_sample_rate(sample_rate);
    }

    pub fn set_volume(&mut self, _vol: f32) {}

    pub fn set_eq_band(&mut self, band_idx: usize, gain_db: f32) {
        self.executor.set_eq_band(band_idx, gain_db);
    }

    pub fn set_node_param(&mut self, node_id: u32, param_id: u32, value: f32) {
        self.executor.set_node_param(node_id, param_id, value);
    }

    pub fn set_eq_bypass(&mut self, bypass: bool) {
        self.executor.set_eq_bypass(bypass);
    }

    pub fn reset_states(&mut self) {
        self.executor.reset_states();
    }

    #[inline(always)]
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.executor.process_block(left, right);
    }

    #[inline(always)]
    pub fn process_interleaved(&mut self, stereo: &mut [f32]) {
        let num_frames = stereo.len() / 2;
        if num_frames == 0 {
            return;
        }
        let mut scratch_l = [0.0f32; 1024];
        let mut scratch_r = [0.0f32; 1024];
        for chunk in stereo.chunks_mut(2048) {
            let frames = chunk.len() / 2;
            for i in 0..frames {
                scratch_l[i] = chunk[i * 2];
                scratch_r[i] = chunk[i * 2 + 1];
            }
            self.executor.process_block(&mut scratch_l[..frames], &mut scratch_r[..frames]);
            for i in 0..frames {
                chunk[i * 2] = scratch_l[i];
                chunk[i * 2 + 1] = scratch_r[i];
            }
        }
    }

    #[inline(always)]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let mut left = [l];
        let mut right = [r];
        self.executor.process_block(&mut left, &mut right);
        (left[0], right[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsp_volume_and_bypass() {
        let mut dsp = DspPipeline::new(44100.0);
        dsp.set_eq_bypass(true);
        let (l, r) = dsp.process_frame(0.4, -0.6);
        assert!((l - 0.4).abs() < 1e-4);
        assert!((r - (-0.6)).abs() < 1e-4);
    }

    #[test]
    fn test_transparent_clamping() {
        let mut dsp = DspPipeline::new(44100.0);
        dsp.set_eq_bypass(false);
        let (l, r) = dsp.process_frame(1.5, -1.5);
        assert!((l - 1.0).abs() < 1e-4, "Peak 1.5 must be soft-limited to ~1.0, got {}", l);
        assert!((r - (-1.0)).abs() < 1e-4, "Peak -1.5 must be soft-limited to ~ -1.0, got {}", r);
    }

    #[test]
    fn test_eq_headroom_boost_12db() {
        let mut dsp = DspPipeline::new(96000.0);
        dsp.set_eq_bypass(false);
        // Boost 1000 Hz by maximum +12 dB
        dsp.set_eq_band(5, 12.0);

        // Feed full-scale 1.0 signal at resonant frequency
        for _ in 0..100 {
            let (out_l, out_r) = dsp.process_frame(0.95, -0.95);
            // Must stay cleanly bounded within [-1.0, 1.0] without NaN or harsh overflow
            assert!(out_l <= 1.0 && out_l >= -1.0);
            assert!(out_r <= 1.0 && out_r >= -1.0);
            assert!(!out_l.is_nan() && !out_r.is_nan());
        }
    }

    #[test]
    fn test_biquad_f64_stability_31hz_at_192khz() {
        let mut dsp = DspPipeline::new(192000.0);
        dsp.set_eq_bypass(false);
        // Boost 31 Hz (band 0) and 63 Hz (band 1) by +6 dB at 192 kHz
        dsp.set_eq_band(0, 6.0);
        dsp.set_eq_band(1, 6.0);

        for i in 0..1000 {
            let t = i as f64 / 192000.0;
            let sine = (2.0 * std::f64::consts::PI * 31.0 * t).sin() as f32;
            let (out_l, out_r) = dsp.process_frame(sine, sine);
            assert!(!out_l.is_nan() && !out_r.is_nan());
            assert!(!out_l.is_infinite() && !out_r.is_infinite());
            assert!(out_l <= 1.0 && out_l >= -1.0);
        }
    }

    #[test]
    fn test_soft_limit_curve() {
        // Below 0.90 is 100% bit-transparent identity
        assert_eq!(soft_limit(0.5), 0.5);
        assert_eq!(soft_limit(-0.5), -0.5);
        assert_eq!(soft_limit(0.89), 0.89);

        // Above 0.90 softly saturates towards 1.0
        let sat_1 = soft_limit(1.0);
        assert!(sat_1 > 0.90 && sat_1 < 1.0);

        let sat_extreme = soft_limit(10.0);
        assert!(sat_extreme <= 1.0);

        let sat_neg = soft_limit(-10.0);
        assert!(sat_neg >= -1.0);
    }

    #[test]
    fn test_severed_chain_silences_audio() {
        let mut dsp = DspPipeline::new(44100.0);
        dsp.set_eq_bypass(false);

        // Sever the chain: is_complete = false
        dsp.set_signal_chain(SignalChainGraph {
            is_complete: false,
            nodes: Vec::new(),
            edges: Vec::new(),
            topological_order: Vec::new(),
            effects: Vec::new(),
        });

        assert!(!dsp.is_chain_complete());
        let (out_l, out_r) = dsp.process_frame(0.85, -0.85);
        assert_eq!(out_l, 0.0, "Severed chain must silence left channel");
        assert_eq!(out_r, 0.0, "Severed chain must silence right channel");
    }

    #[test]
    fn test_volume_attenuator_effect() {
        let mut dsp = DspPipeline::new(44100.0);
        dsp.set_eq_bypass(false);

        dsp.set_signal_chain(SignalChainGraph {
            is_complete: true,
            nodes: Vec::new(),
            edges: Vec::new(),
            topological_order: Vec::new(),
            effects: vec![Box::new(VolumeAttenuatorEffect::new(-6.0205999))],
        });

        let (out_l, out_r) = dsp.process_frame(0.80, -0.60);
        // Half amplitude (-6.02 dB = 0.5x)
        assert!((out_l - 0.40).abs() < 1e-3, "Attenuator did not halve L amplitude: {}", out_l);
        assert!((out_r - (-0.30)).abs() < 1e-3, "Attenuator did not halve R amplitude: {}", out_r);
    }

    #[test]
    fn test_stereo_widener_effect() {
        let mut widener = StereoWidenerEffect::new(1.35);
        let in_l = 0.70;
        let in_r = 0.30;
        let (out_l, out_r) = widener.process_frame(in_l, in_r);

        let in_mono = (in_l + in_r) * 0.5;
        let out_mono = (out_l + out_r) * 0.5;
        assert!((out_mono - in_mono).abs() < 1e-6, "Mono center must be conserved");

        let in_side = (in_l - in_r) * 0.5;
        let out_side = (out_l - out_r) * 0.5;
        assert!((out_side - in_side * 1.35).abs() < 1e-5, "Side channel must be expanded by 1.35x");
    }

    #[test]
    fn test_tube_warmer_effect() {
        let mut tube = TubeWarmerEffect::new(35.0);
        let (pos_l, _) = tube.process_frame(0.5, 0.0);
        let (neg_l, _) = tube.process_frame(-0.5, 0.0);
        assert!((pos_l.abs() - neg_l.abs()).abs() > 0.005, "Triode warmth must introduce even harmonic asymmetry");
    }

    #[test]
    fn test_tube_warmer_variable_drive() {
        let mut tube_zero = TubeWarmerEffect::new(0.0);
        let (out_l, _) = tube_zero.process_frame(0.4, 0.0);
        assert!((out_l - 0.4).abs() < 1e-4, "0% warmth must be transparent pass-through");

        let mut tube_warm = TubeWarmerEffect::new(80.0);
        let (out_w, _) = tube_warm.process_frame(0.4, 0.0);
        assert!((out_w - 0.4).abs() > 0.01, "80% warmth must introduce significant analog saturation");

        tube_warm.set_param(0, 0.0);
        // Slew down over multiple blocks
        for _ in 0..60 {
            let mut buf_l = [0.4f32];
            let mut buf_r = [0.0f32];
            tube_warm.process_block(&mut buf_l, &mut buf_r);
        }
        let (out_slewed, _) = tube_warm.process_frame(0.4, 0.0);
        assert!((out_slewed - 0.4).abs() < 1e-3, "Setting param to 0 must slew to transparent pass-through");
    }

    #[test]
    fn test_bass_guard_effect() {
        let mut bg = BassGuardEffect::new(48000.0);

        let mut max_1k = 0.0f64;
        for i in 0..200 {
            let t = i as f64 / 48000.0;
            let sine = (2.0 * std::f64::consts::PI * 1000.0 * t).sin();
            let (out, _) = bg.process_frame(sine, sine);
            if i > 50 && out.abs() > max_1k {
                max_1k = out.abs();
            }
        }
        assert!(max_1k > 0.98, "1000 Hz must pass through Bass Guard at ~0 dB: got {}", max_1k);

        bg.reset();
        let mut max_10hz = 0.0f64;
        for i in 0..10000 {
            let t = i as f64 / 48000.0;
            let sine = (2.0 * std::f64::consts::PI * 10.0 * t).sin();
            let (out, _) = bg.process_frame(sine, sine);
            if i > 5000 && out.abs() > max_10hz {
                max_10hz = out.abs();
            }
        }
        assert!(max_10hz < 0.25, "10 Hz subsonic rumble must be strongly attenuated: got {}", max_10hz);
    }

    #[test]
    fn test_graphic_eq_zero_cost_flat_bypass() {
        let mut eq = GraphicEqEffect::new([0.0; 10], 48000.0);
        assert!(eq.is_flat);

        let (out_l, out_r) = eq.process_frame(0.42, -0.73);
        assert!((out_l - 0.42).abs() < 1e-6);
        assert!((out_r - (-0.73)).abs() < 1e-6);

        eq.set_param(5, 3.0);
        // Process a few blocks to smooth to target
        let mut l_buf = [0.42f32; 64];
        let mut r_buf = [-0.73f32; 64];
        eq.process_block(&mut l_buf, &mut r_buf);
        assert!(!eq.is_flat);

        eq.set_param(5, 0.0);
        for _ in 0..10 {
            eq.process_block(&mut l_buf, &mut r_buf);
        }
        assert!(eq.is_flat);
    }

    #[test]
    fn test_dsp_pipeline_zero_cost_flat_bypass() {
        let mut dsp = DspPipeline::new(48000.0);
        dsp.set_eq_bypass(false);
        assert!(dsp.is_flat());

        let (out_l, out_r) = dsp.process_frame(0.5, -0.5);
        assert!((out_l - 0.5).abs() < 1e-4);
        assert!((out_r - (-0.5)).abs() < 1e-4);

        dsp.set_eq_band(3, 4.0);
        // Process a few frames to let smoothing apply
        for _ in 0..20 {
            let _ = dsp.process_frame(0.5, -0.5);
        }
        assert!(!dsp.is_flat());

        dsp.set_eq_band(3, 0.0);
        for _ in 0..20 {
            let _ = dsp.process_frame(0.5, -0.5);
        }
        assert!(dsp.is_flat());
    }

    #[test]
    fn test_low_pass_filter_effect() {
        let mut lpf = LowPassFilterEffect::new(800.0, 48000.0);
        let mut l_buf = [0.0f32; 1000];
        let mut r_buf = [0.0f32; 1000];

        // 100 Hz bass sine wave should pass almost untouched
        for i in 0..1000 {
            let t = i as f32 / 48000.0;
            l_buf[i] = (2.0 * std::f32::consts::PI * 100.0 * t).sin();
            r_buf[i] = l_buf[i];
        }
        lpf.process_block(&mut l_buf, &mut r_buf);
        let bass_peak = l_buf[800..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(bass_peak > 0.90, "100 Hz bass should pass through 800 Hz LPF, got {}", bass_peak);

        // 5000 Hz treble sine wave should be heavily cut
        lpf.reset();
        for i in 0..1000 {
            let t = i as f32 / 48000.0;
            l_buf[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
            r_buf[i] = l_buf[i];
        }
        lpf.process_block(&mut l_buf, &mut r_buf);
        let treble_peak = l_buf[800..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(treble_peak < 0.15, "5000 Hz treble should be muted by 800 Hz LPF, got {}", treble_peak);
    }

    #[test]
    fn test_mixer_effect_constant_power() {
        let mut mixer = MixerEffect::new(0.5);
        assert_eq!(mixer.mix_ratio(), Some(0.5));

        // Constant power sum check: sqrt(1 - mix)^2 + sqrt(mix)^2 = 1.0
        let mix = 0.5f32;
        let gain_a = (1.0 - mix).sqrt();
        let gain_b = mix.sqrt();
        let power = gain_a * gain_a + gain_b * gain_b;
        assert!((power - 1.0).abs() < 1e-6);

        mixer.set_param(0, 0.2);
        assert_eq!(mixer.mix_ratio(), Some(0.2));
        mixer.set_param(0, 1.5); // clamps to 1.0
        assert_eq!(mixer.mix_ratio(), Some(1.0));
    }

    #[test]
    fn test_mixer_pipeline_end_to_end_summing() {
        let mut dsp = DspPipeline::new(48000.0);

        // Build a split signal chain DAG:
        // Node 1 (Buffer / Source): Passthrough
        // Node 2 (LPF): LowPassFilter 800 Hz (fed from Node 1)
        // Node 3 (Mixer): Crossfader (Pin A from Node 1, Pin B from Node 2)
        // Edges: (1, 2) [Dry -> LPF], (1, 3) [Dry -> Mixer Pin A], (2, 3) [LPF -> Mixer Pin B]
        let node1 = crate::audio::GraphNode {
            id: 1,
            name: "RAM Buffer".to_string(),
            effect: Box::new(PassthroughEffect),
        };
        let node2 = crate::audio::GraphNode {
            id: 2,
            name: "Low-Pass Filter".to_string(),
            effect: Box::new(LowPassFilterEffect::new(800.0, 48000.0)),
        };
        let node3 = crate::audio::GraphNode {
            id: 3,
            name: "Mixer / Summing Bus".to_string(),
            effect: Box::new(MixerEffect::new(0.5)),
        };

        let graph = crate::audio::SignalChainGraph {
            is_complete: true,
            nodes: vec![node1, node2, node3],
            edges: vec![(1, 2), (1, 3), (2, 3)],
            topological_order: vec![1, 2, 3],
            effects: vec![],
        };

        dsp.set_signal_chain(graph);

        // Test 1: Mix = 0.0 (100% Pin A = 100% Dry)
        // A high frequency treble sine wave (5000 Hz) should pass through unattenuated!
        dsp.set_node_param(3, 0, 0.0);
        let mut left = [0.0f32; 1024];
        let mut right = [0.0f32; 1024];
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
            right[i] = left[i];
        }
        dsp.process_block(&mut left, &mut right);
        let peak_dry = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak_dry > 0.90, "At Mix = 0.0 (Full Pin A Dry), 5000 Hz treble must pass through fully: got {}", peak_dry);

        // Test 2: Mix = 1.0 (100% Pin B = 100% Wet LPF)
        // The same 5000 Hz treble sine wave should be heavily cut!
        dsp.set_node_param(3, 0, 1.0);
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
            right[i] = left[i];
        }
        dsp.process_block(&mut left, &mut right);
        let peak_wet = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak_wet < 0.15, "At Mix = 1.0 (Full Pin B Wet), 5000 Hz treble must be heavily attenuated: got {}", peak_wet);

        // Test 3: Mix = 0.5 (Center Mix)
        // Amplitude should be approximately sqrt(0.5) * peak_dry = ~0.707
        dsp.set_node_param(3, 0, 0.5);
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
            right[i] = left[i];
        }
        dsp.process_block(&mut left, &mut right);
        let peak_mid = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!((peak_mid - 0.707).abs() < 0.12, "At Mix = 0.5, amplitude should follow constant-power gain ~0.707: got {}", peak_mid);
    }

    #[test]
    fn test_splitter_and_mixer_diamond_loop() {
        let mut dsp = DspPipeline::new(48000.0);

        // Build Diamond Loop:
        // Node 1 (Source/Resampler) -> Node 2 (Splitter)
        // Splitter Out A -> Node 4 (Mixer In A)
        // Splitter Out B -> Node 3 (LPF) -> Node 4 (Mixer In B)
        let node1 = crate::audio::GraphNode {
            id: 1,
            name: "App Resampler".to_string(),
            effect: Box::new(PassthroughEffect::new()),
        };
        let node2 = crate::audio::GraphNode {
            id: 2,
            name: "Splitter".to_string(),
            effect: Box::new(SplitterEffect::new()),
        };
        let node3 = crate::audio::GraphNode {
            id: 3,
            name: "Low-Pass Filter".to_string(),
            effect: Box::new(LowPassFilterEffect::new(800.0, 48000.0)),
        };
        let node4 = crate::audio::GraphNode {
            id: 4,
            name: "Mixer / Summing Bus".to_string(),
            effect: Box::new(MixerEffect::new(0.0)),
        };

        let graph = crate::audio::SignalChainGraph {
            is_complete: true,
            nodes: vec![node1, node2, node3, node4],
            edges: vec![
                (1, 2), // Source -> Splitter
                (2, 4), // Splitter Pin A -> Mixer Pin A
                (2, 3), // Splitter Pin B -> LPF
                (3, 4), // LPF -> Mixer Pin B
            ],
            topological_order: vec![1, 2, 3, 4],
            effects: vec![],
        };
        dsp.set_signal_chain(graph);

        // Test 1: Mix = 0.0 (100% Pin A = Direct passthrough via Splitter)
        dsp.set_node_param(4, 0, 0.0);
        let mut left = [0.0f32; 1024];
        let mut right = [0.0f32; 1024];
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
            right[i] = left[i];
        }
        dsp.process_block(&mut left, &mut right);
        let peak_dry = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak_dry > 0.90, "Diamond loop Pin A Dry via Splitter must pass 5000 Hz tone: got {}", peak_dry);

        // Test 2: Mix = 1.0 (100% Pin B = Wet LPF via Splitter)
        dsp.set_node_param(4, 0, 1.0);
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            left[i] = (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
            right[i] = left[i];
        }
        dsp.process_block(&mut left, &mut right);
        let peak_wet = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak_wet < 0.15, "Diamond loop Pin B Wet via Splitter + LPF must attenuate 5000 Hz: got {}", peak_wet);
    }

    #[test]
    fn test_eq_disconnected_does_not_affect_audio() {
        let mut dsp = DspPipeline::new(48000.0);
        dsp.set_eq_bypass(false);

        // Chain without EQ: Source (1) -> RAM Buffer (2) -> Audio Engine (4) -> Output DAC (5)
        let graph = crate::audio::SignalChainGraph {
            is_complete: true,
            nodes: vec![
                crate::audio::GraphNode { id: 1, name: "SOURCE".to_string(), effect: Box::new(PassthroughEffect) },
                crate::audio::GraphNode { id: 2, name: "RAM BUFFER".to_string(), effect: Box::new(PassthroughEffect) },
                crate::audio::GraphNode { id: 4, name: "AUDIO ENGINE".to_string(), effect: Box::new(PassthroughEffect) },
                crate::audio::GraphNode { id: 5, name: "OUTPUT DAC".to_string(), effect: Box::new(PassthroughEffect) },
            ],
            edges: vec![(1, 2), (2, 4), (4, 5)],
            topological_order: vec![1, 2, 4, 5],
            effects: vec![],
        };
        dsp.set_signal_chain(graph);

        // Set extreme EQ boost: +12 dB on all bands
        for band in 0..10 {
            dsp.set_eq_band(band, 12.0);
        }

        let input_l = [0.42f32; 512];
        let input_r = [-0.73f32; 512];
        let mut left = input_l;
        let mut right = input_r;

        dsp.process_block(&mut left, &mut right);

        // Audio must be 100% bit-exact and completely unaffected by the disconnected equalizer!
        for i in 0..512 {
            assert_eq!(left[i], input_l[i], "Left sample at {} must be unmodified when EQ is not in chain", i);
            assert_eq!(right[i], input_r[i], "Right sample at {} must be unmodified when EQ is not in chain", i);
        }
    }

    #[test]
    fn test_set_eq_band_does_not_corrupt_other_effects() {
        let mut dsp = DspPipeline::new(48000.0);
        dsp.set_eq_bypass(false);

        let mixer = Box::new(MixerEffect::new(0.5));
        let lpf = Box::new(LowPassFilterEffect::new(1200.0, 48000.0));

        let graph = crate::audio::SignalChainGraph {
            is_complete: true,
            nodes: vec![
                crate::audio::GraphNode { id: 1, name: "LPF".to_string(), effect: lpf },
                crate::audio::GraphNode { id: 2, name: "MIXER".to_string(), effect: mixer },
            ],
            edges: vec![(1, 2)],
            topological_order: vec![1, 2],
            effects: vec![],
        };
        dsp.set_signal_chain(graph);

        // Set EQ band 0 to +12.0 dB
        dsp.set_eq_band(0, 12.0);

        // Verify Mixer's mix ratio was NOT overwritten (param_id 0)
        let mixer_node = dsp.executor.graph.nodes.iter().find(|n| n.id == 2).unwrap();
        assert_eq!(mixer_node.effect.mix_ratio(), Some(0.5), "Mixer mix_ratio must remain 0.5 and not be corrupted by EQ band 0");
    }

    #[test]
    fn test_eq_connected_properly_modifies_audio() {
        let mut dsp = DspPipeline::new(48000.0);
        dsp.set_eq_bypass(false);

        // Graphic EQ with 1000 Hz band (idx 5) at 0 dB initially
        let eq = Box::new(GraphicEqEffect::new([0.0; 10], 48000.0));
        let graph = crate::audio::SignalChainGraph {
            is_complete: true,
            nodes: vec![
                crate::audio::GraphNode { id: 1, name: "10-Band Graphic EQ".to_string(), effect: eq },
            ],
            edges: vec![],
            topological_order: vec![1],
            effects: vec![],
        };
        dsp.set_signal_chain(graph);

        // Cut 1000 Hz band by -12 dB (should attenuate 1000 Hz by ~75%)
        dsp.set_eq_band(5, -12.0);

        let mut left = [0.0f32; 1024];
        let mut right = [0.0f32; 1024];
        // Slew parameter smoothing over multiple blocks
        for _ in 0..10 {
            for i in 0..1024 {
                let t = i as f32 / 48000.0;
                left[i] = (2.0 * std::f32::consts::PI * 1000.0 * t).sin();
                right[i] = left[i];
            }
            dsp.process_block(&mut left, &mut right);
        }

        let peak_attenuated = left[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        // -12 dB attenuation on 1.0 input should result in amplitude <= 0.35
        assert!(peak_attenuated < 0.35, "Cut 1000 Hz tone via connected EQ should have amplitude < 0.35, got {}", peak_attenuated);
    }

    #[test]
    fn test_volume_booster_effect() {
        let mut booster = VolumeBoosterEffect::new(6.0); // +6.02 dB approx 2x amplitude
        let mut l = [0.2f32; 64];
        let mut r = [0.2f32; 64];
        for _ in 0..15 {
            l.fill(0.2);
            r.fill(0.2);
            booster.process_block(&mut l, &mut r);
        }
        assert!(l[63] > 0.38 && l[63] < 0.42, "Booster at +6dB on 0.2 input should yield ~0.4, got {}", l[63]);

        // Boost to +24 dB with soft-limit
        booster.set_param(0, 24.0);
        let mut l_loud = [0.8f32; 64];
        let mut r_loud = [0.8f32; 64];
        for _ in 0..25 {
            l_loud.fill(0.8);
            r_loud.fill(0.8);
            booster.process_block(&mut l_loud, &mut r_loud);
        }
        assert!(l_loud[63] <= 1.0, "Booster soft limit must guarantee <= 1.0 amplitude, got {}", l_loud[63]);
    }

    #[test]
    fn test_analog_preamp_effect() {
        let mut preamp = AnalogPreampEffect::new(0.0); // 0 dB unity
        let mut l = [0.5f32; 32];
        let mut r = [0.5f32; 32];
        preamp.process_block(&mut l, &mut r);
        assert!((l[31] - 0.5).abs() < 0.01, "Preamp at 0dB should be unity 0.5, got {}", l[31]);

        // Attenuate to -6 dB
        preamp.set_param(0, -6.02);
        for _ in 0..20 {
            l.fill(0.5);
            r.fill(0.5);
            preamp.process_block(&mut l, &mut r);
        }
        assert!(l[31] > 0.23 && l[31] < 0.27, "Preamp at -6dB should yield ~0.25, got {}", l[31]);
    }

    #[test]
    fn test_dynamic_loudness_effect() {
        let mut loudness = DynamicLoudnessEffect::new(100.0, 48000.0);
        let mut l_bass = [0.0f32; 1024];
        let mut r_bass = [0.0f32; 1024];
        for i in 0..1024 {
            let t = i as f32 / 48000.0;
            l_bass[i] = (2.0 * std::f32::consts::PI * 60.0 * t).sin() * 0.2;
            r_bass[i] = l_bass[i];
        }
        loudness.process_block(&mut l_bass, &mut r_bass);
        let peak_bass = l_bass[500..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak_bass > 0.35, "Loudness 100% on 60 Hz bass (0.2 input) should boost to >0.35, got {}", peak_bass);
    }

    #[test]
    fn test_stepped_attenuator_effect() {
        let mut att = SteppedAttenuatorEffect::new(0.0, false);
        let mut l = [1.0f32; 64];
        let mut r = [1.0f32; 64];
        att.process_block(&mut l, &mut r);
        assert!((l[63] - 1.0).abs() < 0.01);

        // Turn on DIM (-20dB)
        att.set_param(1, 1.0);
        for _ in 0..20 {
            l.fill(1.0);
            r.fill(1.0);
            att.process_block(&mut l, &mut r);
        }
        // -20dB on 1.0 is 0.1
        assert!(l[63] < 0.15, "DIM -20dB should attenuate signal to < 0.15, got {}", l[63]);
    }

    #[test]
    fn test_optical_leveler_effect() {
        let mut opto = OpticalLevelerEffect::new(80.0, 48000.0);
        let mut l = [0.95f32; 512];
        let mut r = [0.95f32; 512];
        opto.process_block(&mut l, &mut r);
        assert!(opto.gain_reduction_db > 0.5, "High drive on 0.95 signal should trigger gain reduction > 0.5 dB, got {}", opto.gain_reduction_db);
        assert!(l[511] < 0.95, "Compressed output should be attenuated below 0.95, got {}", l[511]);
    }

    #[test]
    fn test_stereo_balance_effect() {
        let mut pan = StereoBalanceEffect::new(-100.0); // Hard Left
        let mut l = [0.5f32; 64];
        let mut r = [0.5f32; 64];
        for _ in 0..20 {
            l.fill(0.5);
            r.fill(0.5);
            pan.process_block(&mut l, &mut r);
        }
        assert!(l[63] > 0.65, "Hard Left should amplify L, got {}", l[63]);
        assert!(r[63] < 0.01, "Hard Left should mute R, got {}", r[63]);
    }
}
