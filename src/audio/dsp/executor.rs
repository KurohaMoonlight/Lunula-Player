use std::collections::HashMap;
use crate::audio::types::{Effect, SignalChainGraph};
use super::biquad::soft_limit;
use super::effects::GraphicEqEffect;

#[derive(Debug)]
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

pub struct ExecutionStep {
    pub node_id: u32,
    pub name: String,
    pub slot: usize,
    pub effect: Box<dyn Effect>,
    pub input: StepInput,
}

pub struct GraphExecutor {
    sample_rate: f32,
    pub graph: SignalChainGraph,
    steps: Vec<ExecutionStep>,
    terminal_slots: Vec<usize>,
    buffers_l: Vec<Vec<f32>>,
    buffers_r: Vec<Vec<f32>>,
    scratch_l: Vec<f32>,
    scratch_r: Vec<f32>,
    eq_bypass: bool,
    limiter_enabled: bool,
    fallback_eq: GraphicEqEffect,
    has_custom_graph: bool,
    has_active_effects: bool,
}

impl GraphExecutor {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(8000.0);
        let fallback_eq = GraphicEqEffect::new([0.0; 10], sr as f64);
        let buffers_l = (0..32).map(|_| vec![0.0f32; 2048]).collect();
        let buffers_r = (0..32).map(|_| vec![0.0f32; 2048]).collect();
        Self {
            sample_rate: sr,
            graph: SignalChainGraph::default(),
            steps: Vec::new(),
            terminal_slots: Vec::new(),
            buffers_l,
            buffers_r,
            scratch_l: vec![0.0f32; 2048],
            scratch_r: vec![0.0f32; 2048],
            eq_bypass: false,
            limiter_enabled: false,
            fallback_eq,
            has_custom_graph: false,
            has_active_effects: false,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.fallback_eq.set_sample_rate(self.sample_rate);
        for step in &mut self.steps {
            step.effect.set_sample_rate(self.sample_rate);
        }
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

    pub fn set_limiter_enabled(&mut self, enabled: bool) {
        self.limiter_enabled = enabled;
    }

    pub fn set_eq_band(&mut self, index: usize, gain_db: f32) {
        self.fallback_eq.set_param(index as u32, gain_db);
        for step in &mut self.steps {
            if step.effect.name() == "10-Band Graphic EQ" {
                step.effect.set_param(index as u32, gain_db);
            }
        }
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
        for step in &mut self.steps {
            if step.node_id == node_id {
                step.effect.set_param(param_id, value);
                if param_id == 0 {
                    if let StepInput::Mixer { ref mut mix_ratio, .. } = step.input {
                        *mix_ratio = value.clamp(0.0, 1.0);
                    }
                }
            }
        }
        if let Some(node) = self.graph.nodes.iter_mut().find(|n| n.id == node_id) {
            node.effect.set_param(param_id, value);
        }
    }

    pub fn reset_states(&mut self) {
        self.fallback_eq.reset();
        for step in &mut self.steps {
            step.effect.reset();
        }
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

        let mut new_steps: Vec<ExecutionStep> = Vec::with_capacity(graph.topological_order.len());
        let mut node_to_slot: HashMap<u32, usize> = HashMap::new();
        let mut has_active = false;

        if !graph.effects.is_empty() {
            has_active = true;
        }

        for &node_id in &graph.topological_order {
            if let Some(mut node) = graph.nodes.iter_mut().find(|n| n.id == node_id).map(|n| n.clone()) {
                let slot = new_steps.len();
                node_to_slot.insert(node_id, slot);

                // Preserve state from previous execution step if same node_id and name
                if let Some(old_step) = self.steps.iter_mut().find(|s| s.node_id == node_id && s.name == node.name) {
                    std::mem::swap(&mut node.effect, &mut old_step.effect);
                    node.effect.set_sample_rate(self.sample_rate);
                }

                let eff_name = node.effect.name();
                if eff_name != "Passthrough (Wire)" && eff_name != "Splitter / Y-Branch Bus" {
                    has_active = true;
                }

                let incoming: Vec<&crate::audio::GraphEdge> = graph.edges.iter()
                    .filter(|e| e.to_node == node_id && node_to_slot.contains_key(&e.from_node))
                    .collect();

                let input = if incoming.is_empty() {
                    StepInput::Source
                } else if node.effect.mix_ratio().is_some() || eff_name.starts_with("Mixer") {
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
                    slot,
                    effect: node.effect,
                    input,
                });
            }
        }

        let terminal_slots: Vec<usize> = new_steps.iter()
            .filter(|step| {
                !graph.edges.iter().any(|e| e.from_node == step.node_id && graph.topological_order.contains(&e.to_node))
            })
            .map(|step| step.slot)
            .collect();

        while self.buffers_l.len() < new_steps.len().max(32) {
            self.buffers_l.push(vec![0.0f32; 2048]);
            self.buffers_r.push(vec![0.0f32; 2048]);
        }

        self.steps = new_steps;
        self.terminal_slots = terminal_slots;
        self.has_active_effects = has_active;
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
            if !self.fallback_eq.is_flat {
                self.fallback_eq.process_block(left, right);
            }
            if self.limiter_enabled {
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
            } else {
                for s in left.iter_mut() {
                    *s = s.clamp(-1.0, 1.0);
                }
                for s in right.iter_mut() {
                    *s = s.clamp(-1.0, 1.0);
                }
            }
            return;
        }

        // 4. Zero-cost transparent passthrough when no active effects are connected
        if !self.has_active_effects {
            return;
        }

        // 5. Legacy sequential effects (if provided without DAG nodes)
        if !self.graph.effects.is_empty() {
            for effect in &mut self.graph.effects {
                effect.process_block(left, right);
            }
        }

        // 6. Zero-allocation execution of pre-compiled DAG steps
        if !self.steps.is_empty() {
            for step in &mut self.steps {
                let slot = step.slot;
                match step.input {
                    StepInput::Source => {
                        self.buffers_l[slot][..n].copy_from_slice(&left[..n]);
                        self.buffers_r[slot][..n].copy_from_slice(&right[..n]);
                    }
                    StepInput::Single(src) => {
                        self.scratch_l[..n].copy_from_slice(&self.buffers_l[src][..n]);
                        self.scratch_r[..n].copy_from_slice(&self.buffers_r[src][..n]);
                        self.buffers_l[slot][..n].copy_from_slice(&self.scratch_l[..n]);
                        self.buffers_r[slot][..n].copy_from_slice(&self.scratch_r[..n]);
                    }
                    StepInput::Mixer { slot_a, slot_b, mix_ratio } => {
                        // Linear crossfade law: (1 - m, m)
                        let gain_a = 1.0 - mix_ratio;
                        let gain_b = mix_ratio;
                        match (slot_a, slot_b) {
                            (Some(sa), Some(sb)) => {
                                for i in 0..n {
                                    self.buffers_l[slot][i] = self.buffers_l[sa][i] * gain_a + self.buffers_l[sb][i] * gain_b;
                                    self.buffers_r[slot][i] = self.buffers_r[sa][i] * gain_a + self.buffers_r[sb][i] * gain_b;
                                }
                            }
                            (Some(sa), None) => {
                                for i in 0..n {
                                    self.buffers_l[slot][i] = self.buffers_l[sa][i] * gain_a;
                                    self.buffers_r[slot][i] = self.buffers_r[sa][i] * gain_a;
                                }
                            }
                            (None, Some(sb)) => {
                                for i in 0..n {
                                    self.buffers_l[slot][i] = self.buffers_l[sb][i] * gain_b;
                                    self.buffers_r[slot][i] = self.buffers_r[sb][i] * gain_b;
                                }
                            }
                            (None, None) => {
                                self.buffers_l[slot][..n].fill(0.0);
                                self.buffers_r[slot][..n].fill(0.0);
                            }
                        }
                    }
                    StepInput::Multi(ref parents) => {
                        let scale = 1.0f32 / (parents.len() as f32).sqrt();
                        for i in 0..n {
                            let mut sum_l = 0.0f32;
                            let mut sum_r = 0.0f32;
                            for &p in parents {
                                sum_l += self.buffers_l[p][i];
                                sum_r += self.buffers_r[p][i];
                            }
                            self.buffers_l[slot][i] = sum_l * scale;
                            self.buffers_r[slot][i] = sum_r * scale;
                        }
                    }
                }

                let (buf_l, buf_r) = (&mut self.buffers_l[slot][..n], &mut self.buffers_r[slot][..n]);
                step.effect.process_block(buf_l, buf_r);
            }

            // Copy final output from terminal nodes
            if self.terminal_slots.len() == 1 {
                let term = self.terminal_slots[0];
                left.copy_from_slice(&self.buffers_l[term][..n]);
                right.copy_from_slice(&self.buffers_r[term][..n]);
            } else if !self.terminal_slots.is_empty() {
                let scale = 1.0f32 / (self.terminal_slots.len() as f32).sqrt();
                for i in 0..n {
                    let mut sum_l = 0.0f32;
                    let mut sum_r = 0.0f32;
                    for &t in &self.terminal_slots {
                        sum_l += self.buffers_l[t][i];
                        sum_r += self.buffers_r[t][i];
                    }
                    left[i] = sum_l * scale;
                    right[i] = sum_r * scale;
                }
            }
        }

        // 6. Soft-limiting: ONLY when limiter is explicitly enabled by user
        if self.limiter_enabled {
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

