use super::biquad::{soft_limit, BiquadCoeffs, BiquadState};
use crate::audio::types::{AmplitudeInterval, Effect, EffectType};

// 10-Band Graphic EQ Frequencies matching LunulaPlayer
pub const EQ_FREQUENCIES: [f64; 10] = [
    31.0, 63.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
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
            let pre = self.auto_preamp;
            if (pre - 1.0).abs() > 1e-6 {
                for l in left.iter_mut() { *l *= pre; }
                for r in right.iter_mut() { *r *= pre; }
            }
            return;
        }

        let pre = self.auto_preamp;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut fl = *l * pre;
            let mut fr = *r * pre;
            for i in 0..10 {
                fl = self.eq_state_l[i].process(fl, &self.eq_coeffs[i]);
                fr = self.eq_state_r[i].process(fr, &self.eq_coeffs[i]);
            }
            *l = fl;
            *r = fr;
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

    fn effect_type(&self) -> EffectType {
        EffectType::GraphicEq
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

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        if self.is_flat {
            let pre = self.auto_preamp as f32;
            return input.scale(pre);
        }
        let max_boost_db = self.target_gains.iter().fold(0.0f64, |acc, &g| acc.max(g)) as f32;
        let lin_gain = 10.0f32.powf(max_boost_db / 20.0);
        let eff_gain = lin_gain * (self.auto_preamp as f32);
        input.scale(eff_gain.max(1.0))
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        // Smooth parameter slewing to eliminate zipper noise during live slider dragging
        let diff = self.drive - self.current_drive;
        if diff.abs() > 0.01 {
            self.current_drive += diff * 0.15;
        } else {
            self.current_drive = self.drive;
        }

        let k = ((self.current_drive / 100.0) as f64).clamp(0.0, 1.0);
        if k < 0.001 {
            return; // 0% Warmth is pure transparent bypass
        }

        // Triode saturation formula in 64-bit float (f64):
        // Asymmetric even harmonics (xd^2) add musical warmth and body,
        // odd harmonics (xd^3) add presence.
        let d = 1.0 + k * 1.5;
        let c2 = 0.22 * k;
        let c3 = 0.08 * k;
        let norm = 1.0 / (1.0 + 0.40 * k);

        let warm = |x: f64| -> f64 {
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
    fn effect_type(&self) -> EffectType {
        EffectType::TubeWarmer
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.drive = val.clamp(0.0, 100.0);
        }
    }
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        // Tube saturation compresses/soft-limits waveform at [-1.0, 1.0] ceiling.
        // It warms harmonics without expanding amplitude headroom beyond input scale!
        input
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Stereo Widener Effect: Mid-Side Haas stereo field widening
#[derive(Clone, Debug)]
pub struct StereoWidenerEffect {
    width: f64,
    current_width: f64,
}

impl StereoWidenerEffect {
    pub fn new(width: f64) -> Self {
        let w = width.clamp(1.0, 3.0);
        Self {
            width: w,
            current_width: w,
        }
    }
}

impl Effect for StereoWidenerEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        // Smooth parameter slewing to eliminate zipper clicks during live slider adjustments
        let diff = self.width - self.current_width;
        if diff.abs() > 0.005 {
            self.current_width += diff * 0.15;
        } else {
            self.current_width = self.width;
        }

        let w = self.current_width;
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
        "Stereo Widener (Haas 3D)"
    }
    fn effect_type(&self) -> EffectType {
        EffectType::StereoWidener
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            let w = if (0.0..=100.0).contains(&val) {
                1.0 + (val as f64 / 100.0)
            } else if val >= 1.0 {
                val as f64
            } else {
                1.35
            };
            self.width = w.clamp(1.0, 3.0);
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        let w = self.width.clamp(1.0, 3.0) as f32;
        input.scale(w)
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            *l = self.state_l.process(*l, &self.hpf_coeffs);
            *r = self.state_r.process(*r, &self.hpf_coeffs);
        }
    }

    fn reset(&mut self) {
        self.state_l.reset();
        self.state_r.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "Bass Guard (24Hz Sub-Sonic Filter)"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::BassGuard
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Volume Attenuator: Precision passive -6.02 dB pad
#[derive(Clone, Debug)]
pub struct VolumeAttenuatorEffect {
    gain: f64,
}

impl VolumeAttenuatorEffect {
    pub fn new(gain_db: f64) -> Self {
        let gain = 10.0_f64.powf(gain_db / 20.0);
        Self { gain }
    }
}

impl Effect for VolumeAttenuatorEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
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
    fn effect_type(&self) -> EffectType {
        EffectType::VolumeAttenuator
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        input.scale(self.gain as f32)
    }
}

/// Pure Volume Booster Effect: Uncolored linear gain (0.0 to +24.0 dB) with transparent soft-limit guard
#[derive(Clone, Debug)]
pub struct VolumeBoosterEffect {
    boost_db: f32,       // 0.0 to +24.0 dB
    target_gain: f64,
    current_gain: f64,   // Slew-smoothed linear multiplier in f64
}

impl VolumeBoosterEffect {
    pub fn new(boost_db: f32) -> Self {
        let b = boost_db.clamp(0.0, 24.0);
        let g = 10.0_f64.powf(b as f64 / 20.0);
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
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
            *l = soft_limit(amplified_l);
            *r = soft_limit(amplified_r);
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Pure Volume Booster (0 to +24dB)"
    }
    fn effect_type(&self) -> EffectType {
        EffectType::VolumeBooster
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.boost_db = val.clamp(0.0, 24.0);
            self.target_gain = 10.0_f64.powf(self.boost_db as f64 / 20.0);
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        input.scale(self.target_gain as f32)
    }
}

/// Analog Preamp & Trim Effect: Studio console gain stage (-24.0 to +12.0 dB) with Class-A warm transformer touch
#[derive(Clone, Debug)]
pub struct AnalogPreampEffect {
    trim_db: f32,       // -24.0 to +12.0 dB
    target_gain: f64,
    current_gain: f64,
}

impl AnalogPreampEffect {
    pub fn new(trim_db: f32) -> Self {
        let t = trim_db.clamp(-24.0, 12.0);
        let g = 10.0_f64.powf(t as f64 / 20.0);
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
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
                let drive_factor = (((self.trim_db - 3.0) / 9.0).clamp(0.0, 1.0) * 0.04) as f64;
                sl = sl - drive_factor * sl * sl * sl;
                sr = sr - drive_factor * sr * sr * sr;
            }

            *l = soft_limit(sl);
            *r = soft_limit(sr);
        }
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Analog Preamp & Trim (-24 to +12dB)"
    }
    fn effect_type(&self) -> EffectType {
        EffectType::AnalogPreamp
    }
    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.trim_db = val.clamp(-24.0, 12.0);
            self.target_gain = 10.0_f64.powf(self.trim_db as f64 / 20.0);
        }
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        input.scale(self.target_gain as f32)
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        if self.depth < 0.5 {
            return; // Zero-cost transparent bypass
        }

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sl = self.state_l_low.process(*l, &self.low_shelf_coeffs);
            let mut sr = self.state_r_low.process(*r, &self.low_shelf_coeffs);

            sl = self.state_l_high.process(sl, &self.high_shelf_coeffs);
            sr = self.state_r_high.process(sr, &self.high_shelf_coeffs);

            *l = soft_limit(sl);
            *r = soft_limit(sr);
        }
    }

    fn reset(&mut self) {
        self.state_l_low.reset();
        self.state_r_low.reset();
        self.state_l_high.reset();
        self.state_r_high.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "Dynamic Loudness (Fletcher-Munson ISO 226)"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::DynamicLoudness
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
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        if self.depth < 0.5 {
            return input;
        }
        let k = (self.depth / 100.0) as f32;
        let max_boost_db = k * 8.5; // Low shelf boost up to +8.5 dB
        let gain = 10.0_f32.powf(max_boost_db / 20.0);
        input.scale(gain)
    }
}

/// Audiophile Stepped Attenuator Effect: Resistor ladder 64-step passive attenuator with DIM -20dB and Mute
#[derive(Clone, Debug)]
pub struct SteppedAttenuatorEffect {
    step_db: f32,        // -60.0 to 0.0 dB
    is_dimmed: bool,     // -20dB DIM
    is_muted: bool,      // Mute
    target_gain: f64,
    current_gain: f64,
}

impl SteppedAttenuatorEffect {
    pub fn new(step_db: f32, is_dimmed: bool) -> Self {
        let s = step_db.clamp(-60.0, 0.0).round(); // Discrete 1dB steps
        let effective_db = if is_dimmed { s - 20.0 } else { s };
        let g = 10.0_f64.powf(effective_db as f64 / 20.0);
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
            self.target_gain = 10.0_f64.powf(effective_db as f64 / 20.0);
        }
    }
}

impl Default for SteppedAttenuatorEffect {
    fn default() -> Self {
        Self::new(-12.0, false)
    }
}

impl Effect for SteppedAttenuatorEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
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
    fn effect_type(&self) -> EffectType {
        EffectType::SteppedAttenuator
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
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        input.scale(self.target_gain as f32)
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
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
            let peak = (l.abs().max(r.abs())) as f32;

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
                10.0_f64.powf(-(gr_db as f64) / 20.0)
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
        let sr = sample_rate.max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "Studio Optical Leveler (LA-2A Night Dynamic Control)"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::OpticalLeveler
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 {
            self.target_drive = val.clamp(0.0, 100.0);
        }
    }

    fn gain_reduction_db(&self) -> f32 {
        self.gain_reduction_db
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        let drive = (self.target_drive / 100.0).clamp(0.0, 1.0);
        let potential_gain = 1.0 + drive * 1.0; // Up to 2.0x (+6 dB)
        input.scale(potential_gain)
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let diff = self.balance - self.current_balance;
        if diff.abs() > 0.01 {
            self.current_balance += diff * 0.2;
        } else {
            self.current_balance = self.balance;
        }

        if self.current_balance.abs() < 0.5 {
            return; // Center detent is zero-cost passthrough (1.0, 1.0)
        }

        let norm = ((self.current_balance / 100.0) as f64).clamp(-1.0, 1.0);
        let theta = (norm + 1.0) * 0.25 * std::f64::consts::PI; // 0 to pi/2
        let gain_l = std::f64::consts::SQRT_2 * theta.cos();
        let gain_r = std::f64::consts::SQRT_2 * theta.sin();

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
    fn effect_type(&self) -> EffectType {
        EffectType::StereoBalance
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

/// Low-Pass Filter: Variable slope (12/24/48 dB/oct) and resonance fringe Q (0.5 to 4.0)
#[derive(Clone, Debug)]
pub struct LowPassFilterEffect {
    pub cutoff_hz: f64,
    current_cutoff_hz: f64,
    sample_rate: f64,
    pub slope_db: u32,
    pub fringe_q: f64,
    lpf_coeffs: BiquadCoeffs,
    state_l: [BiquadState; 4],
    state_r: [BiquadState; 4],
}

impl LowPassFilterEffect {
    pub fn new(cutoff_hz: f64, sample_rate: f64) -> Self {
        Self::with_slope_and_fringe(cutoff_hz, 12, 0.7071067811865476, sample_rate)
    }

    pub fn with_slope_and_fringe(cutoff_hz: f64, slope_db: u32, fringe_q: f64, sample_rate: f64) -> Self {
        let clamped = cutoff_hz.clamp(20.0, 20000.0);
        let sr = sample_rate.max(8000.0);
        let slope = match slope_db {
            24 => 24,
            48 => 48,
            _ => 12,
        };
        let q = fringe_q.clamp(0.2, 10.0);
        let mut effect = Self {
            cutoff_hz: clamped,
            current_cutoff_hz: clamped,
            sample_rate: sr,
            slope_db: slope,
            fringe_q: q,
            lpf_coeffs: BiquadCoeffs::identity(),
            state_l: [BiquadState::default(); 4],
            state_r: [BiquadState::default(); 4],
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        self.lpf_coeffs = BiquadCoeffs::low_pass(self.current_cutoff_hz, self.sample_rate, self.fringe_q);
    }
}

impl Effect for LowPassFilterEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let diff = self.cutoff_hz - self.current_cutoff_hz;
        if diff.abs() > 0.5 {
            self.current_cutoff_hz += diff * 0.15;
            self.recompute();
        } else if diff.abs() > 0.0 {
            self.current_cutoff_hz = self.cutoff_hz;
            self.recompute();
        }

        let stages = match self.slope_db {
            48 => 4,
            24 => 2,
            _ => 1,
        };

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sample_l = *l;
            let mut sample_r = *r;
            for st in 0..stages {
                sample_l = self.state_l[st].process(sample_l, &self.lpf_coeffs);
                sample_r = self.state_r[st].process(sample_r, &self.lpf_coeffs);
            }
            *l = sample_l;
            *r = sample_r;
        }
    }

    fn reset(&mut self) {
        for s in &mut self.state_l {
            s.reset();
        }
        for s in &mut self.state_r {
            s.reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "Low-Pass Filter"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::LowPassFilter
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        match param_id {
            0 => {
                if val >= 20.0 {
                    self.cutoff_hz = (val as f64).clamp(20.0, 20000.0);
                }
            }
            1 => {
                self.slope_db = match val as u32 {
                    24 => 24,
                    48 => 48,
                    _ => 12,
                };
            }
            2 => {
                self.fringe_q = (val as f64).clamp(0.2, 10.0);
                self.recompute();
            }
            _ => {}
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        let stages = match self.slope_db { 48 => 4, 24 => 2, _ => 1 };
        let q_peak = (self.fringe_q as f32).max(1.0);
        input.scale(q_peak.powi(stages))
    }
}

/// High-Pass Filter: Variable slope (12/24/48 dB/oct) and resonance fringe Q (0.5 to 4.0)
#[derive(Clone, Debug)]
pub struct HighPassFilterEffect {
    pub cutoff_hz: f64,
    current_cutoff_hz: f64,
    sample_rate: f64,
    pub slope_db: u32,
    pub fringe_q: f64,
    hpf_coeffs: BiquadCoeffs,
    state_l: [BiquadState; 4],
    state_r: [BiquadState; 4],
}

impl HighPassFilterEffect {
    pub fn new(cutoff_hz: f64, sample_rate: f64) -> Self {
        Self::with_slope_and_fringe(cutoff_hz, 12, 0.7071067811865476, sample_rate)
    }

    pub fn with_slope_and_fringe(cutoff_hz: f64, slope_db: u32, fringe_q: f64, sample_rate: f64) -> Self {
        let clamped = cutoff_hz.clamp(20.0, 20000.0);
        let sr = sample_rate.max(8000.0);
        let slope = match slope_db {
            24 => 24,
            48 => 48,
            _ => 12,
        };
        let q = fringe_q.clamp(0.2, 10.0);
        let mut effect = Self {
            cutoff_hz: clamped,
            current_cutoff_hz: clamped,
            sample_rate: sr,
            slope_db: slope,
            fringe_q: q,
            hpf_coeffs: BiquadCoeffs::identity(),
            state_l: [BiquadState::default(); 4],
            state_r: [BiquadState::default(); 4],
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        self.hpf_coeffs = BiquadCoeffs::high_pass(self.current_cutoff_hz, self.sample_rate, self.fringe_q);
    }
}

impl Effect for HighPassFilterEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let diff = self.cutoff_hz - self.current_cutoff_hz;
        if diff.abs() > 0.5 {
            self.current_cutoff_hz += diff * 0.15;
            self.recompute();
        } else if diff.abs() > 0.0 {
            self.current_cutoff_hz = self.cutoff_hz;
            self.recompute();
        }

        let stages = match self.slope_db {
            48 => 4,
            24 => 2,
            _ => 1,
        };

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sample_l = *l;
            let mut sample_r = *r;
            for st in 0..stages {
                sample_l = self.state_l[st].process(sample_l, &self.hpf_coeffs);
                sample_r = self.state_r[st].process(sample_r, &self.hpf_coeffs);
            }
            *l = sample_l;
            *r = sample_r;
        }
    }

    fn reset(&mut self) {
        for s in &mut self.state_l {
            s.reset();
        }
        for s in &mut self.state_r {
            s.reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "High-Pass Filter"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::HighPassFilter
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        match param_id {
            0 => {
                if val >= 20.0 {
                    self.cutoff_hz = (val as f64).clamp(20.0, 20000.0);
                }
            }
            1 => {
                self.slope_db = match val as u32 {
                    24 => 24,
                    48 => 48,
                    _ => 12,
                };
            }
            2 => {
                self.fringe_q = (val as f64).clamp(0.2, 10.0);
                self.recompute();
            }
            _ => {}
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        let stages = match self.slope_db { 48 => 4, 24 => 2, _ => 1 };
        let q_peak = (self.fringe_q as f32).max(1.0);
        input.scale(q_peak.powi(stages))
    }
}

/// Band-Pass Filter: Constant 0 dB peak gain with variable center frequency, fringe Q/bandwidth, and slope
#[derive(Clone, Debug)]
pub struct BandPassFilterEffect {
    pub center_freq_hz: f64,
    current_center_freq_hz: f64,
    sample_rate: f64,
    pub fringe_q: f64,
    pub slope_db: u32,
    bpf_coeffs: BiquadCoeffs,
    state_l: [BiquadState; 2],
    state_r: [BiquadState; 2],
}

impl BandPassFilterEffect {
    pub fn new(center_freq_hz: f64, sample_rate: f64) -> Self {
        Self::with_slope_and_fringe(center_freq_hz, 12, 1.414, sample_rate)
    }

    pub fn with_slope_and_fringe(center_freq_hz: f64, slope_db: u32, fringe_q: f64, sample_rate: f64) -> Self {
        let clamped = center_freq_hz.clamp(20.0, 20000.0);
        let sr = sample_rate.max(8000.0);
        let slope = if slope_db == 24 { 24 } else { 12 };
        let q = fringe_q.clamp(0.2, 20.0);
        let mut effect = Self {
            center_freq_hz: clamped,
            current_center_freq_hz: clamped,
            sample_rate: sr,
            fringe_q: q,
            slope_db: slope,
            bpf_coeffs: BiquadCoeffs::identity(),
            state_l: [BiquadState::default(); 2],
            state_r: [BiquadState::default(); 2],
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        self.bpf_coeffs = BiquadCoeffs::band_pass(self.current_center_freq_hz, self.sample_rate, self.fringe_q);
    }
}

impl Effect for BandPassFilterEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let diff = self.center_freq_hz - self.current_center_freq_hz;
        if diff.abs() > 0.5 {
            self.current_center_freq_hz += diff * 0.15;
            self.recompute();
        } else if diff.abs() > 0.0 {
            self.current_center_freq_hz = self.center_freq_hz;
            self.recompute();
        }

        let stages = if self.slope_db == 24 { 2 } else { 1 };

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sample_l = *l;
            let mut sample_r = *r;
            for st in 0..stages {
                sample_l = self.state_l[st].process(sample_l, &self.bpf_coeffs);
                sample_r = self.state_r[st].process(sample_r, &self.bpf_coeffs);
            }
            *l = sample_l;
            *r = sample_r;
        }
    }

    fn reset(&mut self) {
        for s in &mut self.state_l {
            s.reset();
        }
        for s in &mut self.state_r {
            s.reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "Band-Pass Filter"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::BandPassFilter
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        match param_id {
            0 => {
                if val >= 20.0 {
                    self.center_freq_hz = (val as f64).clamp(20.0, 20000.0);
                }
            }
            1 => {
                self.slope_db = if val as u32 == 24 { 24 } else { 12 };
            }
            2 => {
                self.fringe_q = (val as f64).clamp(0.2, 20.0);
                self.recompute();
            }
            _ => {}
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        let stages = if self.slope_db == 24 { 2 } else { 1 };
        let q_peak = (self.fringe_q as f32).max(1.0);
        input.scale(q_peak.powi(stages))
    }
}

/// Linkwitz-Riley 4th-Order (LR4 24 dB/oct) Low-Pass Branch for Crossover
#[derive(Clone, Debug)]
pub struct CrossoverLowEffect {
    pub crossover_freq: f64,
    sample_rate: f64,
    coeffs: BiquadCoeffs,
    state_l: [BiquadState; 2],
    state_r: [BiquadState; 2],
}

impl CrossoverLowEffect {
    pub fn new(crossover_freq: f64, sample_rate: f64) -> Self {
        let safe_f = crossover_freq.clamp(20.0, 10000.0);
        let sr = sample_rate.max(8000.0);
        let mut effect = Self {
            crossover_freq: safe_f,
            sample_rate: sr,
            coeffs: BiquadCoeffs::identity(),
            state_l: [BiquadState::default(); 2],
            state_r: [BiquadState::default(); 2],
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        // Linkwitz-Riley LR4: cascade of two Butterworth filters (Q = 0.70710678)
        self.coeffs = BiquadCoeffs::low_pass(self.crossover_freq, self.sample_rate, 0.7071067811865476);
    }
}

impl Effect for CrossoverLowEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sample_l = *l;
            let mut sample_r = *r;
            for st in 0..2 {
                sample_l = self.state_l[st].process(sample_l, &self.coeffs);
                sample_r = self.state_r[st].process(sample_r, &self.coeffs);
            }
            *l = sample_l;
            *r = sample_r;
        }
    }

    fn reset(&mut self) {
        for s in &mut self.state_l {
            s.reset();
        }
        for s in &mut self.state_r {
            s.reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = (sample_rate as f64).max(8000.0);
        self.recompute();
        self.reset();
    }

    fn name(&self) -> &str {
        "Crossover (Low Output)"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Crossover
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        if param_id == 0 && val >= 20.0 {
            self.crossover_freq = (val as f64).clamp(20.0, 10000.0);
            self.recompute();
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Linkwitz-Riley 4th-Order (LR4 24 dB/oct) High-Pass Branch for Crossover
#[derive(Clone, Debug)]
pub struct CrossoverHighEffect {
    pub crossover_freq: f64,
    sample_rate: f64,
    pub invert_phase: bool,
    coeffs: BiquadCoeffs,
    state_l: [BiquadState; 2],
    state_r: [BiquadState; 2],
}

impl CrossoverHighEffect {
    pub fn new(crossover_freq: f64, invert_phase: bool, sample_rate: f64) -> Self {
        let safe_f = crossover_freq.clamp(20.0, 10000.0);
        let sr = sample_rate.max(8000.0);
        let mut effect = Self {
            crossover_freq: safe_f,
            sample_rate: sr,
            invert_phase,
            coeffs: BiquadCoeffs::identity(),
            state_l: [BiquadState::default(); 2],
            state_r: [BiquadState::default(); 2],
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        // Linkwitz-Riley LR4: cascade of two Butterworth filters (Q = 0.70710678)
        self.coeffs = BiquadCoeffs::high_pass(self.crossover_freq, self.sample_rate, 0.7071067811865476);
    }
}

impl Effect for CrossoverHighEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut sample_l = *l;
            let mut sample_r = *r;
            for st in 0..2 {
                sample_l = self.state_l[st].process(sample_l, &self.coeffs);
                sample_r = self.state_r[st].process(sample_r, &self.coeffs);
            }
            if self.invert_phase {
                *l = -sample_l;
                *r = -sample_r;
            } else {
                *l = sample_l;
                *r = sample_r;
            }
        }
    }

    fn reset(&mut self) {
        for s in &mut self.state_l {
            s.reset();
        }
        for s in &mut self.state_r {
            s.reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = (sample_rate as f64).max(8000.0);
        self.recompute();
        self.reset();
    }

    fn name(&self) -> &str {
        "Crossover (High Output)"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Crossover
    }

    fn set_param(&mut self, param_id: u32, val: f32) {
        match param_id {
            0 => {
                if val >= 20.0 {
                    self.crossover_freq = (val as f64).clamp(20.0, 10000.0);
                    self.recompute();
                }
            }
            1 => {
                self.invert_phase = val > 0.5;
            }
            _ => {}
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Mixer / Summing Bus: Crossfader between input branches with linear crossfade headroom
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
    fn process_block_f64(&mut self, _left: &mut [f64], _right: &mut [f64]) {
        // Mixing/summing is executed in topological bus summing in DspPipeline
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Mixer / Summing Bus"
    }
    fn effect_type(&self) -> EffectType {
        EffectType::Mixer
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
    fn process_block_f64(&mut self, _left: &mut [f64], _right: &mut [f64]) {}
    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Passthrough (Wire)"
    }
    fn effect_type(&self) -> EffectType {
        EffectType::Passthrough
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
    fn process_block_f64(&mut self, _left: &mut [f64], _right: &mut [f64]) {
        // Splitting is executed losslessly by routing the node output buffer to multiple downstream nodes
    }
    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn name(&self) -> &str {
        "Splitter / Y-Branch Bus"
    }
    fn effect_type(&self) -> EffectType {
        EffectType::Splitter
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
        let presets = crate::models::ParametricEqPreset::default_presets();
        if let Some(preset) = presets.into_iter().find(|p| p.name == name) {
            let configs: Vec<ParametricBandConfig> = preset.bands.into_iter().map(|b| {
                let f_type = match b.filter_type {
                    crate::models::FilterType::LowShelf => "low_shelf",
                    crate::models::FilterType::HighShelf => "high_shelf",
                    crate::models::FilterType::HighPass => "hpf",
                    _ => "peak",
                }.to_string();
                ParametricBandConfig {
                    freq: b.freq as f64,
                    gain_db: if b.enabled { b.gain_db as f64 } else { 0.0 },
                    q: b.q as f64,
                    filter_type: f_type,
                }
            }).collect();
            Self::new(name, configs, sample_rate)
        } else {
            let bands = match name {
                "Vocal Clarity" => vec![
                    ParametricBandConfig { freq: 250.0, gain_db: -2.5, q: 1.414, filter_type: "peak".to_string() },
                    ParametricBandConfig { freq: 2000.0, gain_db: 3.0, q: 1.414, filter_type: "peak".to_string() },
                    ParametricBandConfig { freq: 4000.0, gain_db: 3.5, q: 1.414, filter_type: "peak".to_string() },
                ],
                "Bass Boost & Warmth" | "Bass Boost Matrix" => vec![
                    ParametricBandConfig { freq: 63.0, gain_db: 5.0, q: 1.2, filter_type: "low_shelf".to_string() },
                    ParametricBandConfig { freq: 125.0, gain_db: 3.0, q: 1.414, filter_type: "peak".to_string() },
                    ParametricBandConfig { freq: 250.0, gain_db: 1.5, q: 1.414, filter_type: "peak".to_string() },
                ],
                "Air & Sparkle" => vec![
                    ParametricBandConfig { freq: 8000.0, gain_db: 2.5, q: 1.414, filter_type: "high_shelf".to_string() },
                    ParametricBandConfig { freq: 16000.0, gain_db: 4.0, q: 1.0, filter_type: "high_shelf".to_string() },
                ],
                "Warm Vinyl / Acoustic" | "Warm Tube Vintage" => vec![
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
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let pre = self.auto_preamp;
        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let mut fl = *l * pre;
            let mut fr = *r * pre;
            for i in 0..self.coeffs.len() {
                fl = self.state_l[i].process(fl, &self.coeffs[i]);
                fr = self.state_r[i].process(fr, &self.coeffs[i]);
            }
            *l = fl;
            *r = fr;
        }
    }

    fn reset(&mut self) {
        for s in &mut self.state_l { s.reset(); }
        for s in &mut self.state_r { s.reset(); }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(8000.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn effect_type(&self) -> EffectType {
        EffectType::ParametricEq
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        let max_boost_db = self.bands.iter()
            .map(|b| b.gain_db)
            .fold(0.0f64, f64::max) as f32;
        let lin_gain = 10.0f32.powf(max_boost_db / 20.0);
        let eff_gain = lin_gain * (self.auto_preamp as f32);
        input.scale(eff_gain.max(1.0))
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Backwards compatibility alias for PresetEqEffect
pub type PresetEqEffect = ParametricEqEffect;

/// Studio Dynamic Range Compressor Effect
/// Transparently controls signal dynamics with adjustable threshold, ratio, attack, release, and makeup gain.
#[derive(Clone, Debug)]
pub struct CompressorEffect {
    pub threshold_db: f32,   // -48.0 to 0.0 dBFS
    pub ratio: f32,          // 1.0 to 20.0
    pub attack_ms: f32,      // 0.5 to 100.0 ms
    pub release_ms: f32,     // 10.0 to 1000.0 ms
    pub makeup_gain_db: f32, // 0.0 to 24.0 dB
    envelope: f32,
    pub gain_reduction_db: f32,
    sample_rate: f32,
}

impl CompressorEffect {
    pub fn new(threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32, makeup_gain_db: f32, sample_rate: f32) -> Self {
        Self {
            threshold_db: threshold_db.clamp(-60.0, 0.0),
            ratio: ratio.clamp(1.0, 30.0),
            attack_ms: attack_ms.clamp(0.1, 200.0),
            release_ms: release_ms.clamp(5.0, 2000.0),
            makeup_gain_db: makeup_gain_db.clamp(0.0, 24.0),
            envelope: 0.0,
            gain_reduction_db: 0.0,
            sample_rate: sample_rate.max(8000.0),
        }
    }
}

impl Default for CompressorEffect {
    fn default() -> Self {
        Self::new(-16.0, 4.0, 10.0, 120.0, 2.0, 44100.0)
    }
}

impl Effect for CompressorEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        if self.ratio <= 1.01 && self.makeup_gain_db.abs() < 0.01 {
            self.gain_reduction_db = 0.0;
            return; // Bypass
        }

        let thresh_lin = 10.0_f32.powf(self.threshold_db / 20.0);
        let makeup_lin = 10.0_f64.powf(self.makeup_gain_db as f64 / 20.0);
        let att_coeff = (-1.0 / ((self.attack_ms * 0.001) * self.sample_rate)).exp();
        let rel_coeff = (-1.0 / ((self.release_ms * 0.001) * self.sample_rate)).exp();

        let mut max_gr = 0.0f32;

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let peak = (l.abs().max(r.abs())) as f32;

            if peak > self.envelope {
                self.envelope = peak + att_coeff * (self.envelope - peak);
            } else {
                self.envelope = peak + rel_coeff * (self.envelope - peak);
            }

            let env_safe = self.envelope.max(1e-6);
            let gain = if env_safe > thresh_lin {
                let over_db = 20.0 * (env_safe / thresh_lin).log10();
                let compressed_over_db = over_db / self.ratio;
                let gr_db = over_db - compressed_over_db;
                if gr_db > max_gr {
                    max_gr = gr_db;
                }
                10.0_f64.powf(-(gr_db as f64) / 20.0) * makeup_lin
            } else {
                makeup_lin
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
    }

    fn set_param(&mut self, param_id: u32, value: f32) {
        match param_id {
            0 => self.threshold_db = value.clamp(-60.0, 0.0),
            1 => self.ratio = value.clamp(1.0, 30.0),
            2 => self.attack_ms = value.clamp(0.1, 200.0),
            3 => self.release_ms = value.clamp(5.0, 2000.0),
            4 => self.makeup_gain_db = value.clamp(0.0, 24.0),
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "Studio Compressor"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::Compressor
    }

    fn gain_reduction_db(&self) -> f32 {
        self.gain_reduction_db
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        let makeup_lin = 10.0_f32.powf(self.makeup_gain_db / 20.0);
        input.scale(makeup_lin)
    }
}

/// Time & Phase Alignment Effect
/// Allows microsecond-level delay per channel for acoustic distance compensation,
/// and independent 180° phase inversion per channel.
#[derive(Clone, Debug)]
pub struct TimeAlignmentEffect {
    pub delay_left_us: f32,
    pub delay_right_us: f32,
    pub invert_left: bool,
    pub invert_right: bool,
    sample_rate: f32,
    buffer_l: Vec<f64>,
    buffer_r: Vec<f64>,
    write_pos: usize,
    buffer_mask: usize,
}

pub const MAX_ALIGN_DELAY_US: f32 = 50_000.0;
pub const ALIGN_BUFFER_SIZE: usize = 16384;

impl TimeAlignmentEffect {
    pub fn new(delay_left_us: f32, delay_right_us: f32, invert_left: bool, invert_right: bool, sample_rate: f32) -> Self {
        let sr = sample_rate.max(8000.0);
        Self {
            delay_left_us: delay_left_us.clamp(0.0, MAX_ALIGN_DELAY_US),
            delay_right_us: delay_right_us.clamp(0.0, MAX_ALIGN_DELAY_US),
            invert_left,
            invert_right,
            sample_rate: sr,
            buffer_l: vec![0.0f64; ALIGN_BUFFER_SIZE],
            buffer_r: vec![0.0f64; ALIGN_BUFFER_SIZE],
            write_pos: 0,
            buffer_mask: ALIGN_BUFFER_SIZE - 1,
        }
    }
}

impl Effect for TimeAlignmentEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let d_samples_l = (self.delay_left_us as f64 * 1e-6 * self.sample_rate as f64).clamp(0.0, (ALIGN_BUFFER_SIZE - 2) as f64);
        let mut d_int_l = d_samples_l.floor() as usize;
        let mut d_frac_l = d_samples_l - d_int_l as f64;
        if d_frac_l < 1e-6 {
            d_frac_l = 0.0;
        } else if d_frac_l > 0.999999 {
            d_int_l += 1;
            d_frac_l = 0.0;
        }

        let d_samples_r = (self.delay_right_us as f64 * 1e-6 * self.sample_rate as f64).clamp(0.0, (ALIGN_BUFFER_SIZE - 2) as f64);
        let mut d_int_r = d_samples_r.floor() as usize;
        let mut d_frac_r = d_samples_r - d_int_r as f64;
        if d_frac_r < 1e-6 {
            d_frac_r = 0.0;
        } else if d_frac_r > 0.999999 {
            d_int_r += 1;
            d_frac_r = 0.0;
        }

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            self.buffer_l[self.write_pos] = *l;
            self.buffer_r[self.write_pos] = *r;

            let read_idx0_l = (self.write_pos + ALIGN_BUFFER_SIZE - d_int_l) & self.buffer_mask;
            let read_idx1_l = (read_idx0_l + self.buffer_mask) & self.buffer_mask;
            let mut out_l = self.buffer_l[read_idx0_l] * (1.0 - d_frac_l) + self.buffer_l[read_idx1_l] * d_frac_l;

            let read_idx0_r = (self.write_pos + ALIGN_BUFFER_SIZE - d_int_r) & self.buffer_mask;
            let read_idx1_r = (read_idx0_r + self.buffer_mask) & self.buffer_mask;
            let mut out_r = self.buffer_r[read_idx0_r] * (1.0 - d_frac_r) + self.buffer_r[read_idx1_r] * d_frac_r;

            if self.invert_left {
                out_l = -out_l;
            }
            if self.invert_right {
                out_r = -out_r;
            }

            *l = out_l;
            *r = out_r;

            self.write_pos = (self.write_pos + 1) & self.buffer_mask;
        }
    }

    fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_pos = 0;
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.reset();
    }

    fn set_param(&mut self, param_id: u32, value: f32) {
        match param_id {
            0 => self.delay_left_us = value.clamp(0.0, MAX_ALIGN_DELAY_US),
            1 => self.delay_right_us = value.clamp(0.0, MAX_ALIGN_DELAY_US),
            2 => self.invert_left = value > 0.5,
            3 => self.invert_right = value > 0.5,
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "Time & Phase Alignment"
    }

    fn effect_type(&self) -> EffectType {
        EffectType::TimeAlignment
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        AmplitudeInterval::symmetric(input.peak_magnitude())
    }
}

/// Stereo Matrix Router & Headphone Crossfeed Effect
/// Modes:
/// 0: Normal Passthrough (L -> L, R -> R)
/// 1: Channel Swap (L -> R, R -> L)
/// 2: Mono Sum (L = (L+R)/2, R = (L+R)/2)
/// 3: Mid-Side Encode (M = (L+R)/sqrt(2), S = (L-R)/sqrt(2))
/// 4: Mid-Side Decode (L = (M+S)/sqrt(2), R = (M-S)/sqrt(2))
/// 5: Side Only / Mid Mute (L = (L-R)/2, R = (R-L)/2)
/// 6: Mid Only / Side Mute (L = (L+R)/2, R = (L+R)/2)
/// 7: Bauer Headphone Crossfeed
#[derive(Clone, Debug)]
pub struct StereoMatrixEffect {
    pub mode: u8,
    pub crossfeed_amount: f32,
    sample_rate: f32,
    lpf_state_l: f64,
    lpf_state_r: f64,
    delay_l: [f64; 16],
    delay_r: [f64; 16],
    delay_idx: usize,
}

impl StereoMatrixEffect {
    pub fn new(mode: u8, crossfeed_amount: f32, sample_rate: f32) -> Self {
        Self {
            mode: mode.min(7),
            crossfeed_amount: crossfeed_amount.clamp(0.0, 1.0),
            sample_rate: sample_rate.max(8000.0),
            lpf_state_l: 0.0,
            lpf_state_r: 0.0,
            delay_l: [0.0; 16],
            delay_r: [0.0; 16],
            delay_idx: 0,
        }
    }
}

impl Effect for StereoMatrixEffect {
    fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        match self.mode {
            0 => {
                // Passthrough
            }
            1 => {
                // Channel Swap
                for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                    std::mem::swap(l, r);
                }
            }
            2 => {
                // Mono Sum
                for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                    let mono = (*l + *r) * 0.5;
                    *l = mono;
                    *r = mono;
                }
            }
            3 => {
                // Mid-Side Encode in 64-bit precision (lossless round-trip)
                const SQRT_2: f64 = std::f64::consts::SQRT_2;
                for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                    let m = (*l + *r) / SQRT_2;
                    let s = (*l - *r) / SQRT_2;
                    *l = m;
                    *r = s;
                }
            }
            4 => {
                // Mid-Side Decode in 64-bit precision (lossless round-trip)
                const SQRT_2: f64 = std::f64::consts::SQRT_2;
                for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                    let m = *l;
                    let s = *r;
                    let orig_l = (m + s) / SQRT_2;
                    let orig_r = (m - s) / SQRT_2;
                    *l = orig_l;
                    *r = orig_r;
                }
            }
            5 => {
                // Side Only (Mid Muted)
                for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                    let s = (*l - *r) * 0.5;
                    *l = s;
                    *r = -s;
                }
            }
            6 => {
                // Mid Only (Side Muted)
                for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                    let m = (*l + *r) * 0.5;
                    *l = m;
                    *r = m;
                }
            }
            7 => {
                // Bauer Headphone Crossfeed in 64-bit float
                let fc = 700.0f64;
                let sr = self.sample_rate as f64;
                let alpha = 1.0 - (-2.0 * std::f64::consts::PI * fc / sr).exp();
                let delay_samples = ((sr * 0.00015).round() as usize).clamp(1, 15);
                let cross_factor = ((self.crossfeed_amount as f64) * 0.40).clamp(0.0, 0.50);
                let direct_factor = 1.0 - cross_factor * 0.35;

                for (l, r) in left.iter_mut().zip(right.iter_mut()) {
                    let in_l = *l;
                    let in_r = *r;

                    let old_idx = (self.delay_idx + 16 - delay_samples) & 15;
                    let delayed_l = self.delay_l[old_idx];
                    let delayed_r = self.delay_r[old_idx];

                    self.delay_l[self.delay_idx] = in_l;
                    self.delay_r[self.delay_idx] = in_r;
                    self.delay_idx = (self.delay_idx + 1) & 15;

                    self.lpf_state_l += alpha * (delayed_l - self.lpf_state_l);
                    self.lpf_state_r += alpha * (delayed_r - self.lpf_state_r);

                    *l = direct_factor * in_l + cross_factor * self.lpf_state_r;
                    *r = direct_factor * in_r + cross_factor * self.lpf_state_l;
                }
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.lpf_state_l = 0.0;
        self.lpf_state_r = 0.0;
        self.delay_l.fill(0.0);
        self.delay_r.fill(0.0);
        self.delay_idx = 0;
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.reset();
    }

    fn set_param(&mut self, param_id: u32, value: f32) {
        match param_id {
            0 => self.mode = (value.round() as u8).min(7),
            1 => self.crossfeed_amount = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn name(&self) -> &str {
        match self.mode {
            0 => "Stereo Matrix (Passthrough)",
            1 => "Stereo Matrix (Channel Swap)",
            2 => "Stereo Matrix (Mono Sum)",
            3 => "Stereo Matrix (M/S Encode)",
            4 => "Stereo Matrix (M/S Decode)",
            5 => "Stereo Matrix (Side Only)",
            6 => "Stereo Matrix (Mid Only)",
            7 => "Stereo Matrix (Bauer Crossfeed)",
            _ => "Stereo Matrix Router",
        }
    }

    fn effect_type(&self) -> EffectType {
        EffectType::StereoMatrix
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }

    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        if self.mode == 3 || self.mode == 4 {
            input.scale(1.4142)
        } else {
            input
        }
    }
}