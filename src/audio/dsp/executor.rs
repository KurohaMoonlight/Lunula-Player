use std::collections::HashMap;
use crossbeam_channel::Sender;
use crate::audio::types::{Effect, EffectType, SignalChainGraph};
use super::biquad::soft_limit;
use super::effects::GraphicEqEffect;

/// Maximum audio block size supported for zero-allocation DSP execution.
/// Covers all standard pro-audio and consumer block sizes (64, 128, 256, 512, 1024, 2048, and 4096).
pub const MAX_BLOCK_SIZE: usize = 4096;

/// Default preallocated node slot capacity to prevent any heap reallocations in the audio callback.
pub const INITIAL_SLOT_CAPACITY: usize = 8;

#[derive(Debug, Clone)]
pub enum StepInput {
    Source,
    Single(usize),
    Mixer {
        slot_a: Option<usize>,
        slot_b: Option<usize>,
        mix_ratio: f32,
    },
    Multi(Vec<usize>),
}

#[derive(Debug)]
pub struct ExecutionStep {
    pub node_id: u32,
    pub name: String,
    pub effect_type: EffectType,
    pub slot: usize,
    pub effect: Box<dyn Effect>,
    pub input: StepInput,
}

impl Clone for ExecutionStep {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            name: self.name.clone(),
            effect_type: self.effect_type,
            slot: self.slot,
            effect: self.effect.clone(),
            input: self.input.clone(),
        }
    }
}

/// Fully-compiled, immutable execution plan ready for zero-allocation audio callback execution.
#[derive(Debug)]
pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub terminal_slots: Vec<usize>,
    pub has_custom_graph: bool,
    pub has_active_effects: bool,
    pub is_complete: bool,
    pub graph: SignalChainGraph,
}

impl Clone for ExecutionPlan {
    fn clone(&self) -> Self {
        Self {
            steps: self.steps.clone(),
            terminal_slots: self.terminal_slots.clone(),
            has_custom_graph: self.has_custom_graph,
            has_active_effects: self.has_active_effects,
            is_complete: self.is_complete,
            graph: self.graph.clone(),
        }
    }
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            terminal_slots: Vec::new(),
            has_custom_graph: false,
            has_active_effects: false,
            is_complete: true,
            graph: SignalChainGraph::default(),
        }
    }
}

/// Compiles a declarative `SignalChainGraph` into an `ExecutionPlan`.
/// MUST be called on a background/control thread (outside the hardware audio callback).
/// If compilation fails, returns `Err` allowing the audio thread to safely continue running
/// the previous execution plan without glitches or interruptions (Instant Rollback).
pub fn compile_signal_chain_plan(
    graph: &SignalChainGraph,
    sample_rate: f32,
    mut old_steps: Option<&mut [ExecutionStep]>,
) -> Result<ExecutionPlan, String> {
    let mut graph_clone = graph.clone();
    for node in &mut graph_clone.nodes {
        node.effect.set_sample_rate(sample_rate);
    }
    for effect in &mut graph_clone.effects {
        effect.set_sample_rate(sample_rate);
    }

    let mut new_steps: Vec<ExecutionStep> = Vec::with_capacity(graph_clone.topological_order.len());
    let mut node_to_slot: HashMap<u32, usize> = HashMap::with_capacity(graph_clone.topological_order.len());
    let mut has_active = !graph_clone.effects.is_empty();

    for &node_id in &graph_clone.topological_order {
        if let Some(mut node) = graph_clone.nodes.iter_mut().find(|n| n.id == node_id).map(|n| n.clone()) {
            let slot = new_steps.len();
            node_to_slot.insert(node_id, slot);

            let eff_type = node.effect.effect_type();

            // Preserve state from previous execution step if same node_id and effect type
            if let Some(ref mut prev_steps) = old_steps {
                if let Some(old_step) = prev_steps.iter_mut().find(|s| s.node_id == node_id && s.effect_type == eff_type) {
                    std::mem::swap(&mut node.effect, &mut old_step.effect);
                    node.effect.set_sample_rate(sample_rate);
                }
            }

            if !eff_type.is_routing_only() {
                has_active = true;
            }

            let incoming: Vec<&crate::audio::GraphEdge> = graph_clone.edges.iter()
                .filter(|e| e.to_node == node_id && node_to_slot.contains_key(&e.from_node))
                .collect();

            let input = if incoming.is_empty() {
                StepInput::Source
            } else if node.effect.mix_ratio().is_some() || eff_type.is_mixer() {
                has_active = true;
                let mut slot_a = None;
                let mut slot_b = None;
                for edge in incoming {
                    if edge.to_port == 0 && slot_a.is_none() {
                        slot_a = Some(node_to_slot[&edge.from_node]);
                    } else if edge.to_port == 1 && slot_b.is_none() {
                        slot_b = Some(node_to_slot[&edge.from_node]);
                    } else if slot_a.is_none() {
                        slot_a = Some(node_to_slot[&edge.from_node]);
                    } else if slot_b.is_none() {
                        slot_b = Some(node_to_slot[&edge.from_node]);
                    }
                }
                let mix_ratio = node.effect.mix_ratio().unwrap_or(0.5);
                StepInput::Mixer { slot_a, slot_b, mix_ratio }
            } else if incoming.len() == 1 {
                StepInput::Single(node_to_slot[&incoming[0].from_node])
            } else {
                let parents = incoming.iter().map(|e| node_to_slot[&e.from_node]).collect();
                StepInput::Multi(parents)
            };

            new_steps.push(ExecutionStep {
                node_id,
                name: node.name.clone(),
                effect_type: eff_type,
                slot,
                effect: node.effect,
                input,
            });
        }
    }

    let terminal_slots: Vec<usize> = new_steps.iter()
        .filter(|step| {
            !graph_clone.edges.iter().any(|e| e.from_node == step.node_id && graph_clone.topological_order.contains(&e.to_node))
        })
        .map(|step| step.slot)
        .collect();

    Ok(ExecutionPlan {
        steps: new_steps,
        terminal_slots,
        has_custom_graph: true,
        has_active_effects: has_active,
        is_complete: graph_clone.is_complete,
        graph: graph_clone,
    })
}

pub struct GraphExecutor {
    sample_rate: f32,
    pub plan: ExecutionPlan,
    pub graph: SignalChainGraph,
    buffers_l: Vec<Vec<f64>>,
    buffers_r: Vec<Vec<f64>>,
    scratch_l: Vec<f64>,
    scratch_r: Vec<f64>,
    eq_bypass: bool,
    limiter_enabled: bool,
    fallback_eq: GraphicEqEffect,
    gc_tx: Option<Sender<ExecutionPlan>>,
}

impl GraphExecutor {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(8000.0);
        let fallback_eq = GraphicEqEffect::new([0.0; 10], sr as f64);
        Self {
            sample_rate: sr,
            plan: ExecutionPlan::default(),
            graph: SignalChainGraph::default(),
            buffers_l: Vec::new(),
            buffers_r: Vec::new(),
            scratch_l: Vec::new(),
            scratch_r: Vec::new(),
            eq_bypass: false,
            limiter_enabled: false,
            fallback_eq,
            gc_tx: None,
        }
    }

    /// Sets the garbage collector return channel.
    /// When a new compiled graph plan is applied, the previous plan is sent here
    /// so that dynamic memory destruction happens on the worker thread, NEVER in the audio callback.
    pub fn set_gc_sender(&mut self, gc_tx: Sender<ExecutionPlan>) {
        self.gc_tx = Some(gc_tx);
    }

    /// Pre-allocates 64-bit (`f64`) channel and scratch buffers up to `required_slots`.
    /// Called before stream processing to ensure zero heap allocation inside callback.
    pub fn ensure_buffer_capacity(&mut self, required_slots: usize) {
        if self.scratch_l.is_empty() {
            self.scratch_l = vec![0.0f64; MAX_BLOCK_SIZE];
            self.scratch_r = vec![0.0f64; MAX_BLOCK_SIZE];
        }
        let target_slots = required_slots.max(INITIAL_SLOT_CAPACITY);
        while self.buffers_l.len() < target_slots {
            self.buffers_l.push(vec![0.0f64; MAX_BLOCK_SIZE]);
            self.buffers_r.push(vec![0.0f64; MAX_BLOCK_SIZE]);
        }
    }

    /// Real-time safe: swaps the active execution plan in $O(1)$ lock-free pointer swap.
    /// The superseded plan is forwarded to the GC channel for destruction outside the audio callback.
    pub fn apply_compiled_plan(&mut self, new_plan: ExecutionPlan) {
        self.ensure_buffer_capacity(new_plan.steps.len());
        self.graph = new_plan.graph.clone();
        let old_plan = std::mem::replace(&mut self.plan, new_plan);
        if let Some(ref tx) = self.gc_tx {
            if !old_plan.steps.is_empty() {
                let _ = tx.try_send(old_plan);
            }
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.fallback_eq.set_sample_rate(self.sample_rate);
        for step in &mut self.plan.steps {
            step.effect.set_sample_rate(self.sample_rate);
        }
        for node in &mut self.plan.graph.nodes {
            node.effect.set_sample_rate(self.sample_rate);
        }
        for effect in &mut self.plan.graph.effects {
            effect.set_sample_rate(self.sample_rate);
        }
        self.graph = self.plan.graph.clone();
    }

    pub fn set_eq_bypass(&mut self, bypass: bool) {
        self.eq_bypass = bypass;
        if bypass {
            self.reset_states();
        }
    }

    pub fn set_limiter_enabled(&mut self, enabled: bool) {
        self.limiter_enabled = enabled;
    }

    pub fn set_eq_band(&mut self, index: usize, gain_db: f32) {
        self.fallback_eq.set_param(index as u32, gain_db);
        for step in &mut self.plan.steps {
            if step.effect_type == EffectType::GraphicEq {
                step.effect.set_param(index as u32, gain_db);
            }
        }
        for node in &mut self.plan.graph.nodes {
            if node.effect.effect_type() == EffectType::GraphicEq {
                node.effect.set_param(index as u32, gain_db);
            }
        }
        for effect in &mut self.plan.graph.effects {
            if effect.effect_type() == EffectType::GraphicEq {
                effect.set_param(index as u32, gain_db);
            }
        }
    }

    pub fn set_node_param(&mut self, node_id: u32, param_id: u32, value: f32) {
        for step in &mut self.plan.steps {
            if step.node_id == node_id || step.node_id == node_id + 50_000 {
                step.effect.set_param(param_id, value);
                if param_id == 0 {
                    if let StepInput::Mixer { ref mut mix_ratio, .. } = step.input {
                        *mix_ratio = value.clamp(0.0, 1.0);
                    }
                }
            }
        }
        for node in &mut self.plan.graph.nodes {
            if node.id == node_id || node.id == node_id + 50_000 {
                node.effect.set_param(param_id, value);
            }
        }
    }

    pub fn get_max_gain_reduction(&self) -> f32 {
        let mut max_gr = 0.0f32;
        for step in &self.plan.steps {
            let gr = step.effect.gain_reduction_db();
            if gr > max_gr {
                max_gr = gr;
            }
        }
        for effect in &self.plan.graph.effects {
            let gr = effect.gain_reduction_db();
            if gr > max_gr {
                max_gr = gr;
            }
        }
        max_gr
    }

    pub fn reset_states(&mut self) {
        self.fallback_eq.reset();
        for step in &mut self.plan.steps {
            step.effect.reset();
        }
        for node in &mut self.plan.graph.nodes {
            node.effect.reset();
        }
        for effect in &mut self.plan.graph.effects {
            effect.reset();
        }
    }

    pub fn is_flat(&self) -> bool {
        self.fallback_eq.is_flat
    }

    pub fn is_chain_complete(&self) -> bool {
        self.plan.is_complete
    }

    /// Convenience helper for direct graph assignment (e.g. unit tests and non-realtime offline processing).
    /// Pre-compiles the graph on caller thread before applying to executor.
    pub fn set_signal_chain(&mut self, graph: SignalChainGraph) {
        if let Ok(plan) = compile_signal_chain_plan(&graph, self.sample_rate, Some(&mut self.plan.steps)) {
            self.apply_compiled_plan(plan);
        }
    }

    /// Top-level block processing in True 64-bit (`f64`) precision with dynamic capacity bounds and block slicing protection.
    /// Accepts `f32` slices from the ring buffer, promotes to `f64` inside the pre-allocated 64-bit Master DSP Bus,
    /// executes the entire DAG and master limiter in `f64`, and converts back to `f32` at the output boundary.
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len().min(right.len());
        if n == 0 {
            return;
        }

        // 1. If signal chain graph is severed, mute output
        if !self.plan.is_complete {
            left[..n].fill(0.0);
            right[..n].fill(0.0);
            return;
        }

        // 2. Bypass passthrough
        if self.eq_bypass {
            return;
        }

        if self.scratch_l.is_empty() {
            self.ensure_buffer_capacity(self.plan.steps.len());
        }

        // 3. Fallback to standalone EQ if no custom graph provided
        if !self.plan.has_custom_graph {
            for chunk_start in (0..n).step_by(MAX_BLOCK_SIZE) {
                let chunk_len = (n - chunk_start).min(MAX_BLOCK_SIZE);
                for i in 0..chunk_len {
                    self.scratch_l[i] = left[chunk_start + i] as f64;
                    self.scratch_r[i] = right[chunk_start + i] as f64;
                }
                if !self.fallback_eq.is_flat {
                    self.fallback_eq.process_block_f64(&mut self.scratch_l[..chunk_len], &mut self.scratch_r[..chunk_len]);
                }
                Self::apply_limiter_or_clamp_f64(self.limiter_enabled, &mut self.scratch_l[..chunk_len], &mut self.scratch_r[..chunk_len]);
                for i in 0..chunk_len {
                    left[chunk_start + i] = self.scratch_l[i] as f32;
                    right[chunk_start + i] = self.scratch_r[i] as f32;
                }
            }
            return;
        }

        // 4. Zero-cost transparent passthrough when no active effects are connected
        if !self.plan.has_active_effects {
            return;
        }

        // 5. Legacy sequential effects (if provided without DAG nodes) — executed in 64-bit f64
        if self.plan.steps.is_empty() && !self.plan.graph.effects.is_empty() {
            for chunk_start in (0..n).step_by(MAX_BLOCK_SIZE) {
                let chunk_len = (n - chunk_start).min(MAX_BLOCK_SIZE);
                for i in 0..chunk_len {
                    self.scratch_l[i] = left[chunk_start + i] as f64;
                    self.scratch_r[i] = right[chunk_start + i] as f64;
                }
                for effect in &mut self.plan.graph.effects {
                    effect.process_block_f64(&mut self.scratch_l[..chunk_len], &mut self.scratch_r[..chunk_len]);
                }
                for i in 0..chunk_len {
                    left[chunk_start + i] = self.scratch_l[i] as f32;
                    right[chunk_start + i] = self.scratch_r[i] as f32;
                }
            }
            return;
        }

        // 6. Zero-allocation execution of pre-compiled DAG steps in bounded MAX_BLOCK_SIZE 64-bit (f64) chunks
        if n <= MAX_BLOCK_SIZE {
            self.process_sub_block_f32_io(&mut left[..n], &mut right[..n]);
        } else {
            for chunk_start in (0..n).step_by(MAX_BLOCK_SIZE) {
                let chunk_len = (n - chunk_start).min(MAX_BLOCK_SIZE);
                let l_chunk = &mut left[chunk_start..chunk_start + chunk_len];
                let r_chunk = &mut right[chunk_start..chunk_start + chunk_len];
                self.process_sub_block_f32_io(l_chunk, r_chunk);
            }
        }
    }

    /// Direct native 64-bit (`f64`) block processing without `f32` I/O conversion.
    /// Useful for ultra-deep 64-bit mastering verification and direct `f64` pipelines.
    pub fn process_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let n = left.len().min(right.len());
        if n == 0 {
            return;
        }
        if !self.plan.is_complete {
            left[..n].fill(0.0);
            right[..n].fill(0.0);
            return;
        }
        if self.eq_bypass {
            return;
        }
        if self.scratch_l.is_empty() {
            self.ensure_buffer_capacity(self.plan.steps.len());
        }
        if !self.plan.has_custom_graph {
            if !self.fallback_eq.is_flat {
                self.fallback_eq.process_block_f64(&mut left[..n], &mut right[..n]);
            }
            Self::apply_limiter_or_clamp_f64(self.limiter_enabled, &mut left[..n], &mut right[..n]);
            return;
        }
        if !self.plan.has_active_effects {
            return;
        }
        if self.plan.steps.is_empty() && !self.plan.graph.effects.is_empty() {
            for effect in &mut self.plan.graph.effects {
                effect.process_block_f64(&mut left[..n], &mut right[..n]);
            }
            return;
        }
        for chunk_start in (0..n).step_by(MAX_BLOCK_SIZE) {
            let chunk_len = (n - chunk_start).min(MAX_BLOCK_SIZE);
            self.process_sub_block_f64(&mut left[chunk_start..chunk_start + chunk_len], &mut right[chunk_start..chunk_start + chunk_len]);
        }
        Self::apply_limiter_or_clamp_f64(self.limiter_enabled, &mut left[..n], &mut right[..n]);
    }

    #[inline(always)]
    fn process_sub_block_f32_io(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len();
        if self.plan.steps.is_empty() {
            return;
        }

        for step in &mut self.plan.steps {
            let slot = step.slot;
            if slot >= self.buffers_l.len() {
                continue;
            }

            match step.input {
                StepInput::Source => {
                    for i in 0..n {
                        self.buffers_l[slot][i] = left[i] as f64;
                        self.buffers_r[slot][i] = right[i] as f64;
                    }
                }
                StepInput::Single(src) => {
                    if src < self.buffers_l.len() {
                        self.scratch_l[..n].copy_from_slice(&self.buffers_l[src][..n]);
                        self.scratch_r[..n].copy_from_slice(&self.buffers_r[src][..n]);
                        self.buffers_l[slot][..n].copy_from_slice(&self.scratch_l[..n]);
                        self.buffers_r[slot][..n].copy_from_slice(&self.scratch_r[..n]);
                    }
                }
                StepInput::Mixer { slot_a, slot_b, mix_ratio } => {
                    let gain_a = (1.0 - mix_ratio) as f64;
                    let gain_b = mix_ratio as f64;
                    match (slot_a, slot_b) {
                        (Some(sa), Some(sb)) if sa < self.buffers_l.len() && sb < self.buffers_l.len() => {
                            for i in 0..n {
                                self.buffers_l[slot][i] = self.buffers_l[sa][i] * gain_a + self.buffers_l[sb][i] * gain_b;
                                self.buffers_r[slot][i] = self.buffers_r[sa][i] * gain_a + self.buffers_r[sb][i] * gain_b;
                            }
                        }
                        (Some(sa), _) if sa < self.buffers_l.len() => {
                            for i in 0..n {
                                self.buffers_l[slot][i] = self.buffers_l[sa][i] * gain_a;
                                self.buffers_r[slot][i] = self.buffers_r[sa][i] * gain_a;
                            }
                        }
                        (_, Some(sb)) if sb < self.buffers_l.len() => {
                            for i in 0..n {
                                self.buffers_l[slot][i] = self.buffers_l[sb][i] * gain_b;
                                self.buffers_r[slot][i] = self.buffers_r[sb][i] * gain_b;
                            }
                        }
                        _ => {
                            self.buffers_l[slot][..n].fill(0.0);
                            self.buffers_r[slot][..n].fill(0.0);
                        }
                    }
                }
                StepInput::Multi(ref parents) => {
                    let scale = 1.0f64 / (parents.len() as f64).sqrt();
                    for i in 0..n {
                        let mut sum_l = 0.0f64;
                        let mut sum_r = 0.0f64;
                        for &p in parents {
                            if p < self.buffers_l.len() {
                                sum_l += self.buffers_l[p][i];
                                sum_r += self.buffers_r[p][i];
                            }
                        }
                        self.buffers_l[slot][i] = sum_l * scale;
                        self.buffers_r[slot][i] = sum_r * scale;
                    }
                }
            }

            let (buf_l, buf_r) = (&mut self.buffers_l[slot][..n], &mut self.buffers_r[slot][..n]);
            step.effect.process_block_f64(buf_l, buf_r);
        }

        // Apply 64-bit output limiter/clamp and write back from terminal nodes
        if self.plan.terminal_slots.len() == 1 {
            let term = self.plan.terminal_slots[0];
            if term < self.buffers_l.len() {
                Self::apply_limiter_or_clamp_f64(
                    self.limiter_enabled,
                    &mut self.buffers_l[term][..n],
                    &mut self.buffers_r[term][..n],
                );
                for i in 0..n {
                    left[i] = self.buffers_l[term][i] as f32;
                    right[i] = self.buffers_r[term][i] as f32;
                }
            }
        } else if !self.plan.terminal_slots.is_empty() {
            let scale = 1.0f64 / (self.plan.terminal_slots.len() as f64).sqrt();
            for i in 0..n {
                let mut sum_l = 0.0f64;
                let mut sum_r = 0.0f64;
                for &t in &self.plan.terminal_slots {
                    if t < self.buffers_l.len() {
                        sum_l += self.buffers_l[t][i];
                        sum_r += self.buffers_r[t][i];
                    }
                }
                self.scratch_l[i] = sum_l * scale;
                self.scratch_r[i] = sum_r * scale;
            }
            Self::apply_limiter_or_clamp_f64(self.limiter_enabled, &mut self.scratch_l[..n], &mut self.scratch_r[..n]);
            for i in 0..n {
                left[i] = self.scratch_l[i] as f32;
                right[i] = self.scratch_r[i] as f32;
            }
        }
    }

    #[inline(always)]
    fn process_sub_block_f64(&mut self, left: &mut [f64], right: &mut [f64]) {
        let n = left.len();
        if self.plan.steps.is_empty() {
            return;
        }

        for step in &mut self.plan.steps {
            let slot = step.slot;
            if slot >= self.buffers_l.len() {
                continue;
            }

            match step.input {
                StepInput::Source => {
                    self.buffers_l[slot][..n].copy_from_slice(&left[..n]);
                    self.buffers_r[slot][..n].copy_from_slice(&right[..n]);
                }
                StepInput::Single(src) => {
                    if src < self.buffers_l.len() {
                        self.scratch_l[..n].copy_from_slice(&self.buffers_l[src][..n]);
                        self.scratch_r[..n].copy_from_slice(&self.buffers_r[src][..n]);
                        self.buffers_l[slot][..n].copy_from_slice(&self.scratch_l[..n]);
                        self.buffers_r[slot][..n].copy_from_slice(&self.scratch_r[..n]);
                    }
                }
                StepInput::Mixer { slot_a, slot_b, mix_ratio } => {
                    let gain_a = (1.0 - mix_ratio) as f64;
                    let gain_b = mix_ratio as f64;
                    match (slot_a, slot_b) {
                        (Some(sa), Some(sb)) if sa < self.buffers_l.len() && sb < self.buffers_l.len() => {
                            for i in 0..n {
                                self.buffers_l[slot][i] = self.buffers_l[sa][i] * gain_a + self.buffers_l[sb][i] * gain_b;
                                self.buffers_r[slot][i] = self.buffers_r[sa][i] * gain_a + self.buffers_r[sb][i] * gain_b;
                            }
                        }
                        (Some(sa), _) if sa < self.buffers_l.len() => {
                            for i in 0..n {
                                self.buffers_l[slot][i] = self.buffers_l[sa][i] * gain_a;
                                self.buffers_r[slot][i] = self.buffers_r[sa][i] * gain_a;
                            }
                        }
                        (_, Some(sb)) if sb < self.buffers_l.len() => {
                            for i in 0..n {
                                self.buffers_l[slot][i] = self.buffers_l[sb][i] * gain_b;
                                self.buffers_r[slot][i] = self.buffers_r[sb][i] * gain_b;
                            }
                        }
                        _ => {
                            self.buffers_l[slot][..n].fill(0.0);
                            self.buffers_r[slot][..n].fill(0.0);
                        }
                    }
                }
                StepInput::Multi(ref parents) => {
                    let scale = 1.0f64 / (parents.len() as f64).sqrt();
                    for i in 0..n {
                        let mut sum_l = 0.0f64;
                        let mut sum_r = 0.0f64;
                        for &p in parents {
                            if p < self.buffers_l.len() {
                                sum_l += self.buffers_l[p][i];
                                sum_r += self.buffers_r[p][i];
                            }
                        }
                        self.buffers_l[slot][i] = sum_l * scale;
                        self.buffers_r[slot][i] = sum_r * scale;
                    }
                }
            }

            let (buf_l, buf_r) = (&mut self.buffers_l[slot][..n], &mut self.buffers_r[slot][..n]);
            step.effect.process_block_f64(buf_l, buf_r);
        }

        if self.plan.terminal_slots.len() == 1 {
            let term = self.plan.terminal_slots[0];
            if term < self.buffers_l.len() {
                left.copy_from_slice(&self.buffers_l[term][..n]);
                right.copy_from_slice(&self.buffers_r[term][..n]);
            }
        } else if !self.plan.terminal_slots.is_empty() {
            let scale = 1.0f64 / (self.plan.terminal_slots.len() as f64).sqrt();
            for i in 0..n {
                let mut sum_l = 0.0f64;
                let mut sum_r = 0.0f64;
                for &t in &self.plan.terminal_slots {
                    if t < self.buffers_l.len() {
                        sum_l += self.buffers_l[t][i];
                        sum_r += self.buffers_r[t][i];
                    }
                }
                left[i] = sum_l * scale;
                right[i] = sum_r * scale;
            }
        }
    }

    #[inline(always)]
    fn apply_limiter_or_clamp_f64(limiter_enabled: bool, left: &mut [f64], right: &mut [f64]) {
        if limiter_enabled {
            for s in left.iter_mut() {
                if s.abs() > 0.90 {
                    *s = soft_limit(*s);
                }
                *s = s.clamp(-1.0, 1.0);
            }
            for s in right.iter_mut() {
                if s.abs() > 0.90 {
                    *s = soft_limit(*s);
                }
                *s = s.clamp(-1.0, 1.0);
            }
        } else {
            for s in left.iter_mut() {
                *s = s.clamp(-1.0, 1.0);
            }
            for s in right.iter_mut() {
                *s = s.clamp(-1.0, 1.0);
            }
        }
    }
}