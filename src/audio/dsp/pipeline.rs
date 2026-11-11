use crate::audio::types::SignalChainGraph;
use super::executor::GraphExecutor;

pub struct DspPipeline {
    pub executor: GraphExecutor,
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

    pub fn apply_compiled_plan(&mut self, plan: super::executor::ExecutionPlan) {
        self.executor.apply_compiled_plan(plan);
    }

    pub fn set_gc_sender(&mut self, tx: crossbeam_channel::Sender<super::executor::ExecutionPlan>) {
        self.executor.set_gc_sender(tx);
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

    pub fn set_limiter_enabled(&mut self, enabled: bool) {
        self.executor.set_limiter_enabled(enabled);
    }

    #[inline(always)]
    pub fn is_limiter_enabled(&self) -> bool {
        self.executor.is_limiter_enabled()
    }

    pub fn reset_states(&mut self) {
        self.executor.reset_states();
    }

    pub fn get_max_gain_reduction(&self) -> f32 {
        self.executor.get_max_gain_reduction()
    }

    #[inline(always)]
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        self.executor.process_block(left, right);
    }

    #[inline(always)]
    pub fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        self.executor.process_block_f64(left, right);
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
    pub fn process_interleaved_f64(&mut self, stereo: &mut [f64]) {
        let num_frames = stereo.len() / 2;
        if num_frames == 0 {
            return;
        }
        let mut scratch_l = [0.0f64; 1024];
        let mut scratch_r = [0.0f64; 1024];
        for chunk in stereo.chunks_mut(2048) {
            let frames = chunk.len() / 2;
            for i in 0..frames {
                scratch_l[i] = chunk[i * 2];
                scratch_r[i] = chunk[i * 2 + 1];
            }
            self.executor.process_block_f64(&mut scratch_l[..frames], &mut scratch_r[..frames]);
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

    #[inline(always)]
    pub fn process_frame_f64(&mut self, l: f64, r: f64) -> (f64, f64) {
        let mut left = [l];
        let mut right = [r];
        self.executor.process_block_f64(&mut left, &mut right);
        (left[0], right[0])
    }
}