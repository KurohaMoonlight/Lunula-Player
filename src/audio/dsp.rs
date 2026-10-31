use crate::audio::types::{Effect, SignalChainGraph};

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
        if gain_db.abs() < 0.05 || sample_rate <= 0.0 {
            return Self::identity();
        }

        // Clamp center frequency below Nyquist
        let nyquist = sample_rate * 0.49;
        let f0 = center_freq.clamp(10.0, nyquist);

        let a = 10.0_f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (f0 / sample_rate);
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
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

impl BiquadState {
    #[inline(always)]
    pub fn process(&mut self, input: f64, coeffs: &BiquadCoeffs) -> f64 {
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

/// 10-Band Peaking Graphic EQ Effect with Auto-Preamp Headroom
#[derive(Clone, Debug)]
pub struct GraphicEqEffect {
    sample_rate: f64,
    eq_coeffs: [BiquadCoeffs; 10],
    eq_state_l: [BiquadState; 10],
    eq_state_r: [BiquadState; 10],
    eq_gains: [f64; 10],
    auto_preamp: f64,
    is_flat: bool,
}

impl GraphicEqEffect {
    pub fn new(gains: [f32; 10], sample_rate: f64) -> Self {
        let mut eq = Self {
            sample_rate: sample_rate.max(44100.0),
            eq_coeffs: [BiquadCoeffs::identity(); 10],
            eq_state_l: [BiquadState::default(); 10],
            eq_state_r: [BiquadState::default(); 10],
            eq_gains: [0.0; 10],
            auto_preamp: 1.0,
            is_flat: true,
        };
        for (i, &g) in gains.iter().enumerate() {
            eq.eq_gains[i] = g.clamp(-12.0, 12.0) as f64;
        }
        eq.recompute_all();
        eq
    }

    fn update_auto_preamp(&mut self) {
        let max_boost = self.eq_gains.iter().cloned().fold(0.0f64, f64::max);
        self.auto_preamp = if max_boost > 0.0 {
            10.0_f64.powf(-max_boost / 20.0)
        } else {
            1.0
        };
    }

    fn recompute_band(&mut self, idx: usize) {
        let f0 = EQ_FREQUENCIES[idx];
        let gain = self.eq_gains[idx];
        self.eq_coeffs[idx] = BiquadCoeffs::peaking_eq(f0, gain, self.sample_rate, 1.414);
    }

    fn recompute_all(&mut self) {
        self.update_auto_preamp();
        self.is_flat = self.eq_gains.iter().all(|&g| g.abs() < 0.05);
        for i in 0..10 {
            self.recompute_band(i);
        }
    }
}

impl Effect for GraphicEqEffect {
    #[inline(always)]
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64) {
        // Zero-Cost Fast Short-Circuit: bypass 10-band biquad cascade completely when EQ is flat (0.0 dB)
        if self.is_flat {
            return (l * self.auto_preamp, r * self.auto_preamp);
        }
        let mut fl = l * self.auto_preamp;
        let mut fr = r * self.auto_preamp;
        for i in 0..10 {
            fl = self.eq_state_l[i].process(fl, &self.eq_coeffs[i]);
            fr = self.eq_state_r[i].process(fr, &self.eq_coeffs[i]);
        }
        (fl, fr)
    }

    fn reset(&mut self) {
        for i in 0..10 {
            self.eq_state_l[i].reset();
            self.eq_state_r[i].reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        let sr = sample_rate.max(44100.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute_all();
            self.reset();
        }
    }

    fn name(&self) -> &str {
        "10-Band Graphic EQ"
    }

    fn set_param(&mut self, index: usize, val: f64) {
        if index < 10 {
            self.eq_gains[index] = val.clamp(-12.0, 12.0);
            self.update_auto_preamp();
            self.is_flat = self.eq_gains.iter().all(|&g| g.abs() < 0.05);
            self.recompute_band(index);
        }
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Tube Warmer Effect: Asymmetric triode saturation adding 2nd and 3rd harmonics
#[derive(Clone, Debug, Default)]
pub struct TubeWarmerEffect;

impl TubeWarmerEffect {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for TubeWarmerEffect {
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64) {
        #[inline(always)]
        fn warm(x: f64) -> f64 {
            let d = 1.25;
            let xd = x * d;
            let y = xd + 0.15 * xd * xd - 0.05 * xd * xd * xd;
            y * 0.82
        }
        (warm(l), warm(r))
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f64) {}
    fn name(&self) -> &str {
        "Tube Warmer (Triode Saturation)"
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
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64) {
        let mid = (l + r) * 0.5;
        let side = (l - r) * 0.5;
        let widened_side = side * self.width;
        let out_l = mid + widened_side;
        let out_r = mid - widened_side;
        (out_l, out_r)
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f64) {}
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
            sample_rate: sample_rate.max(44100.0),
            hpf_coeffs: BiquadCoeffs::identity(),
            state_l: BiquadState::default(),
            state_r: BiquadState::default(),
        };
        bg.recompute();
        bg
    }

    fn recompute(&mut self) {
        let f0 = 24.0;
        let q = 0.7071067811865476;
        let w0 = 2.0 * std::f64::consts::PI * (f0 / self.sample_rate);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let b0 = (1.0 + cos_w0) * 0.5;
        let b1 = -(1.0 + cos_w0);
        let b2 = (1.0 + cos_w0) * 0.5;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        self.hpf_coeffs = BiquadCoeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        };
    }
}

impl Effect for BassGuardEffect {
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64) {
        let fl = self.state_l.process(l, &self.hpf_coeffs);
        let fr = self.state_r.process(r, &self.hpf_coeffs);
        (fl, fr)
    }

    fn reset(&mut self) {
        self.state_l.reset();
        self.state_r.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate.max(44100.0);
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
    gain: f64,
}

impl VolumeAttenuatorEffect {
    pub fn new(gain_db: f64) -> Self {
        let gain = 10.0_f64.powf(gain_db / 20.0);
        Self { gain }
    }
}

impl Effect for VolumeAttenuatorEffect {
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64) {
        (l * self.gain, r * self.gain)
    }

    fn reset(&mut self) {}
    fn set_sample_rate(&mut self, _sample_rate: f64) {}
    fn name(&self) -> &str {
        "Volume Attenuator (-6dB Pad)"
    }
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

/// Preset EQ Curves: Vocal Clarity, Bass Boost Matrix, Air & Sparkle, Warm Tube Vintage
#[derive(Clone, Debug)]
pub struct PresetEqEffect {
    preset_name: String,
    sample_rate: f64,
    coeffs: Vec<BiquadCoeffs>,
    state_l: Vec<BiquadState>,
    state_r: Vec<BiquadState>,
}

impl PresetEqEffect {
    pub fn from_preset(name: &str, sample_rate: f64) -> Self {
        let mut effect = Self {
            preset_name: name.to_string(),
            sample_rate: sample_rate.max(44100.0),
            coeffs: Vec::new(),
            state_l: Vec::new(),
            state_r: Vec::new(),
        };
        effect.recompute();
        effect
    }

    fn recompute(&mut self) {
        let bands: Vec<(f64, f64, f64)> = match self.preset_name.as_str() {
            "Vocal Clarity" => vec![
                (250.0, -2.5, 1.414),
                (2000.0, 3.0, 1.414),
                (4000.0, 3.5, 1.414),
            ],
            "Bass Boost Matrix" => vec![
                (63.0, 5.0, 1.2),
                (125.0, 3.0, 1.414),
                (250.0, 1.5, 1.414),
            ],
            "Air & Sparkle" => vec![
                (8000.0, 2.5, 1.414),
                (16000.0, 4.0, 1.0),
            ],
            "Warm Tube Vintage" => vec![
                (125.0, 2.0, 1.414),
                (8000.0, -2.0, 1.414),
                (16000.0, -3.5, 1.0),
            ],
            _ => vec![
                (1000.0, 2.0, 1.414),
            ],
        };

        self.coeffs = bands
            .iter()
            .map(|&(f0, gain, q)| BiquadCoeffs::peaking_eq(f0, gain, self.sample_rate, q))
            .collect();
        self.state_l = vec![BiquadState::default(); self.coeffs.len()];
        self.state_r = vec![BiquadState::default(); self.coeffs.len()];
    }
}

impl Effect for PresetEqEffect {
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64) {
        let mut fl = l;
        let mut fr = r;
        for i in 0..self.coeffs.len() {
            fl = self.state_l[i].process(fl, &self.coeffs[i]);
            fr = self.state_r[i].process(fr, &self.coeffs[i]);
        }
        (fl, fr)
    }

    fn reset(&mut self) {
        for s in &mut self.state_l { s.reset(); }
        for s in &mut self.state_r { s.reset(); }
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate.max(44100.0);
        self.recompute();
        self.reset();
    }

    fn name(&self) -> &str {
        &self.preset_name
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

// ---------------------------------------------------------------------------
// DSP Pipeline
// ---------------------------------------------------------------------------

pub struct DspPipeline {
    sample_rate: f64,
    chain_complete: bool,
    has_graph: bool,
    active_effects: Vec<Box<dyn Effect>>,
    eq_coeffs: [BiquadCoeffs; 10],
    eq_state_l: [BiquadState; 10],
    eq_state_r: [BiquadState; 10],
    eq_gains: [f64; 10],
    auto_preamp: f64,
    eq_bypass: bool,
    is_flat: bool,
    volume: f32,
}

impl DspPipeline {
    pub fn new(sample_rate: f32) -> Self {
        let mut dsp = Self {
            sample_rate: (sample_rate as f64).max(44100.0),
            chain_complete: true,
            has_graph: false,
            active_effects: Vec::new(),
            eq_coeffs: [BiquadCoeffs::identity(); 10],
            eq_state_l: [BiquadState::default(); 10],
            eq_state_r: [BiquadState::default(); 10],
            eq_gains: [0.0; 10],
            auto_preamp: 1.0,
            eq_bypass: true,
            is_flat: true,
            volume: 0.80,
        };
        dsp.recompute_all_eq_coeffs();
        dsp
    }

    pub fn is_chain_complete(&self) -> bool {
        self.chain_complete
    }

    // =========================================================================================
    //  ARSITEKTUR AUDIO DSP: EKSEKUSI DAG & PERCABANGAN SINYAL (ROADMAP MENDATANG)
    // =========================================================================================
    //
    //  [IDE C] URUTAN SERIAL & PENCEGAHAN FEEDBACK LOOP:
    //  ----------------------------------------------------------------------------------------
    //  - Efek dieksekusi secara topologis (Kahn's topological sort) dalam alur DAG tanpa cycle.
    //  - Loop feedback (A -> C -> A) dilarang pada level graf karena dalam domain digital real-time
    //    tanpa unit delay (z^-1 buffer circular) akan terjadi infinite recursion / crash.
    //  - Pemrosesan ulang efek dilakukan via multiple card instance (A1 -> C -> A2).
    //
    //  [IDE B] PERCABANGAN PARALEL & SUMMING MIXER:
    //  ----------------------------------------------------------------------------------------
    //  - GUI saat ini sudah mendukung percabangan kabel (Branching Output & Merging Input).
    //  - Rencana Implementasi DSP untuk Percabangan Paralel:
    //    1. SPLIT BUFFER: Ketika sebuah output pin mencabang ke N modul terpisah, frame buffer audio
    //       di-clone/didistribusikan ke input masing-masing modul.
    //    2. PARALLEL PROCESSING: Tiap modul berjalan secara mandiri (misal: Tube Warmer dan Haas Delay
    //       bekerja bersamaan pada frame audio yang sama).
    //    3. SUMMING BUS (MIXER): Di node tempat kedua cabang bertemu (converge), sinyal dijumlahkan:
    //          y(t) = w1 * x1(t) + w2 * x2(t) + ...
    //       dengan automatic headroom attenuation (misal: 1/N atau -3 dB/-6 dB gain compensation)
    //       dan soft-limiting agar tidak timbul inter-sample clipping (+6 dB peak).
    //    4. PHASE COHERENCY: Kompensasi latensi (delay line) jika salah satu cabang memiliki latency.
    // =========================================================================================

    pub fn set_signal_chain(&mut self, graph: SignalChainGraph) {
        self.chain_complete = graph.is_complete;
        self.active_effects = graph.effects;
        self.has_graph = true;
        for effect in &mut self.active_effects {
            effect.set_sample_rate(self.sample_rate);
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = (sample_rate as f64).max(44100.0);
        if (self.sample_rate - sr).abs() > 1.0 {
            self.sample_rate = sr;
            self.recompute_all_eq_coeffs();
            self.reset_states();
            for effect in &mut self.active_effects {
                effect.set_sample_rate(self.sample_rate);
            }
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.5);
    }

    pub fn set_eq_band(&mut self, band_idx: usize, gain_db: f32) {
        if band_idx < 10 {
            self.eq_gains[band_idx] = gain_db.clamp(-12.0, 12.0) as f64;
            self.update_auto_preamp();
            self.is_flat = self.eq_gains.iter().all(|&g| g.abs() < 0.05);
            self.recompute_band_coeffs(band_idx);
            for effect in &mut self.active_effects {
                effect.set_param(band_idx, self.eq_gains[band_idx]);
            }
        }
    }

    fn update_auto_preamp(&mut self) {
        let max_boost = self.eq_gains.iter().cloned().fold(0.0f64, f64::max);
        self.auto_preamp = if max_boost > 0.0 {
            10.0_f64.powf(-max_boost / 20.0)
        } else {
            1.0
        };
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
        self.eq_coeffs[band_idx] = BiquadCoeffs::peaking_eq(f0, gain, self.sample_rate, 1.414);
    }

    fn recompute_all_eq_coeffs(&mut self) {
        self.update_auto_preamp();
        self.is_flat = self.eq_gains.iter().all(|&g| g.abs() < 0.05);
        for i in 0..10 {
            self.recompute_band_coeffs(i);
        }
    }

    pub fn reset_states(&mut self) {
        for i in 0..10 {
            self.eq_state_l[i].reset();
            self.eq_state_r[i].reset();
        }
        for effect in &mut self.active_effects {
            effect.reset();
        }
    }

    #[inline(always)]
    pub fn process_frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        // 1. If signal chain graph is severed (no connection to Endpoint), output silence
        if !self.chain_complete {
            return (0.0, 0.0);
        }

        // 2. Direct transparent passthrough if bypassed
        if self.eq_bypass {
            return (l.clamp(-1.0, 1.0), r.clamp(-1.0, 1.0));
        }

        let mut fl = l as f64;
        let mut fr = r as f64;

        if !self.active_effects.is_empty() {
            // 3. Process through modular dynamic effects in DAG execution order
            for effect in &mut self.active_effects {
                let (next_l, next_r) = effect.process_frame(fl, fr);
                fl = next_l;
                fr = next_r;
            }
            // 4. Smooth soft-knee saturation limiter: prevents harsh clipping on transient inter-band resonance
            let out_l = soft_limit(fl) as f32;
            let out_r = soft_limit(fr) as f32;
            (out_l.clamp(-1.0, 1.0), out_r.clamp(-1.0, 1.0))
        } else if !self.has_graph {
            // Standalone fallback ONLY when no graph has ever been supplied (e.g. standalone DSP tests or engine startup)
            // Zero-Cost Fast Short-Circuit: skip 10-band biquad cascade when gains are 0.0 dB
            if self.is_flat {
                let out_l = soft_limit(fl * self.auto_preamp) as f32;
                let out_r = soft_limit(fr * self.auto_preamp) as f32;
                return (out_l.clamp(-1.0, 1.0), out_r.clamp(-1.0, 1.0));
            }
            fl *= self.auto_preamp;
            fr *= self.auto_preamp;
            for i in 0..10 {
                fl = self.eq_state_l[i].process(fl, &self.eq_coeffs[i]);
                fr = self.eq_state_r[i].process(fr, &self.eq_coeffs[i]);
            }
            let out_l = soft_limit(fl) as f32;
            let out_r = soft_limit(fr) as f32;
            (out_l.clamp(-1.0, 1.0), out_r.clamp(-1.0, 1.0))
        } else {
            // Graph is active and explicitly bypasses effects (e.g. 2 -> 4 directly):
            // 100% transparent passthrough matching the canvas visualization!
            (l.clamp(-1.0, 1.0), r.clamp(-1.0, 1.0))
        }
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

        // Center mono energy (L + R) / 2 must be mathematically conserved
        let in_mono = (in_l + in_r) * 0.5;
        let out_mono = (out_l + out_r) * 0.5;
        assert!((out_mono - in_mono).abs() < 1e-6, "Mono center must be conserved");

        // Stereo side channel must be expanded by +35%
        let in_side = (in_l - in_r) * 0.5;
        let out_side = (out_l - out_r) * 0.5;
        assert!((out_side - in_side * 1.35).abs() < 1e-6, "Side channel must be expanded by 1.35x");
    }

    #[test]
    fn test_tube_warmer_effect() {
        let mut tube = TubeWarmerEffect::new();
        // Triode saturation has asymmetric 2nd harmonic +0.15 x^2
        let (pos_l, _) = tube.process_frame(0.5, 0.0);
        let (neg_l, _) = tube.process_frame(-0.5, 0.0);

        // Positive swing and negative swing should not be symmetric due to even harmonics
        assert!((pos_l.abs() - neg_l.abs()).abs() > 0.01, "Triode warmth must introduce even harmonic asymmetry");
    }

    #[test]
    fn test_bass_guard_effect() {
        let mut bg = BassGuardEffect::new(48000.0);

        // Feed 1000 Hz tone (should pass virtually unattenuated)
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

        // Reset and feed 10 Hz subsonic rumble (should be heavily rolled off below 24 Hz cutoff)
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
        // At 10 Hz with 24 Hz 2nd order HPF (-12 dB/oct), attenuation is > 14 dB (< 0.20 amplitude)
        assert!(max_10hz < 0.25, "10 Hz subsonic rumble must be strongly attenuated: got {}", max_10hz);
    }

    #[test]
    fn test_graphic_eq_zero_cost_flat_bypass() {
        // Initial flat EQ: all gains 0.0 dB
        let mut eq = GraphicEqEffect::new([0.0; 10], 48000.0);
        assert!(eq.is_flat);

        // Frame process on flat EQ should pass through samples identically
        let (out_l, out_r) = eq.process_frame(0.42, -0.73);
        assert!((out_l - 0.42).abs() < 1e-9);
        assert!((out_r - (-0.73)).abs() < 1e-9);

        // Modify a band: is_flat becomes false
        eq.set_param(5, 3.0); // +3 dB at 1000 Hz
        assert!(!eq.is_flat);

        // Reset all bands to 0: is_flat becomes true again
        eq.set_param(5, 0.0);
        assert!(eq.is_flat);
    }

    #[test]
    fn test_dsp_pipeline_zero_cost_flat_bypass() {
        let mut dsp = DspPipeline::new(48000.0);
        dsp.set_eq_bypass(false);
        assert!(dsp.is_flat);

        let (out_l, out_r) = dsp.process_frame(0.5, -0.5);
        assert!((out_l - 0.5).abs() < 1e-4);
        assert!((out_r - (-0.5)).abs() < 1e-4);

        // Boost a band: is_flat becomes false
        dsp.set_eq_band(3, 4.0);
        assert!(!dsp.is_flat);

        // Revert band: is_flat becomes true
        dsp.set_eq_band(3, 0.0);
        assert!(dsp.is_flat);
    }
}
