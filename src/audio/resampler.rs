// High-precision stereo linear resampler with sub-sample fractional interpolation
pub struct StereoResampler {
    source_rate: u32,
    target_rate: u32,
    fraction: f64,
    prev_l: f32,
    prev_r: f32,
    curr_l: f32,
    curr_r: f32,
    initialized: bool,
}

impl StereoResampler {
    pub fn new(source_rate: u32, target_rate: u32) -> Self {
        Self {
            source_rate: source_rate.max(1),
            target_rate: target_rate.max(1),
            fraction: 0.0,
            prev_l: 0.0,
            prev_r: 0.0,
            curr_l: 0.0,
            curr_r: 0.0,
            initialized: false,
        }
    }

    #[inline(always)]
    pub fn is_passthrough(&self) -> bool {
        self.source_rate == self.target_rate
    }

    // Fetches the next resampled stereo frame (L, R).
    // Calls `fetch` to obtain incoming native frames as needed.
    #[inline(always)]
    pub fn next_frame<F>(&mut self, mut fetch: F) -> Option<(f32, f32)>
    where
        F: FnMut() -> Option<(f32, f32)>,
    {
        // 100% Bit-perfect direct passthrough when sample rates match
        if self.is_passthrough() {
            return fetch();
        }

        if !self.initialized {
            if let Some((l0, r0)) = fetch() {
                self.prev_l = l0;
                self.prev_r = r0;
                if let Some((l1, r1)) = fetch() {
                    self.curr_l = l1;
                    self.curr_r = r1;
                } else {
                    self.curr_l = l0;
                    self.curr_r = r0;
                }
                self.initialized = true;
            } else {
                return None;
            }
        }

        let ratio = self.source_rate as f64 / self.target_rate as f64;

        while self.fraction >= 1.0 {
            self.prev_l = self.curr_l;
            self.prev_r = self.curr_r;
            if let Some((l, r)) = fetch() {
                self.curr_l = l;
                self.curr_r = r;
                self.fraction -= 1.0;
            } else {
                return None; // End of file
            }
        }

        let frac = self.fraction as f32;
        let out_l = self.prev_l + (self.curr_l - self.prev_l) * frac;
        let out_r = self.prev_r + (self.curr_r - self.prev_r) * frac;

        self.fraction += ratio;
        Some((out_l, out_r))
    }

    pub fn reset(&mut self, source_rate: u32, target_rate: u32) {
        self.source_rate = source_rate.max(1);
        self.target_rate = target_rate.max(1);
        self.fraction = 0.0;
        self.prev_l = 0.0;
        self.prev_r = 0.0;
        self.curr_l = 0.0;
        self.curr_r = 0.0;
        self.initialized = false;
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
    fn test_resampling_ratio() {
        // Resample from 1000 Hz to 2000 Hz (upsample 2x)
        let mut resampler = StereoResampler::new(1000, 2000);
        let mut input = vec![(0.0, 0.0), (1.0, 1.0)];
        let mut outs = Vec::new();

        while let Some(f) = resampler.next_frame(|| if !input.is_empty() { Some(input.remove(0)) } else { None }) {
            outs.push(f);
            if outs.len() > 10 { break; }
        }

        assert!(outs.len() >= 2);
        // Frame 0 should be at t=0 (0.0), Frame 1 should be at t=0.5 (0.5)
        assert!((outs[0].0 - 0.0).abs() < 1e-4);
        assert!((outs[1].0 - 0.5).abs() < 1e-4);
    }
}
