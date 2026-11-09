use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Audio signal chain processing mode.
/// Determines how samples flow through resampler, DSP, and volume stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainMode {
    /// No DSP, no volume scaling, no clamp. Resampler passthrough (device rate = track rate).
    BitPerfect,
    /// Full DSP pipeline active (10-band EQ + FX). Software volume applied.
    DspActive,
    /// Force device output to 48 kHz (emulate Windows Shared Audio). DSP bypassed, software volume applied.
    OsResampled,
}

impl Default for ChainMode {
    fn default() -> Self {
        ChainMode::BitPerfect
    }
}

/// Sinc interpolation quality / length preset for Rubato SRC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SincQuality {
    /// 64-point Sinc convolution: ultra low CPU and lowest latency.
    Fast,
    /// 128-point Sinc convolution: standard audiophile transparency (Default).
    Studio,
    /// 256-point Sinc convolution: extreme mathematical reconstruction precision.
    Extreme,
}

impl Default for SincQuality {
    fn default() -> Self {
        Self::Studio
    }
}

impl SincQuality {
    pub fn sinc_len(&self) -> usize {
        match self {
            SincQuality::Fast => 64,
            SincQuality::Studio => 128,
            SincQuality::Extreme => 256,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SincQuality::Fast => "Fast (64-pt)",
            SincQuality::Studio => "Studio (128-pt)",
            SincQuality::Extreme => "Extreme (256-pt)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            SincQuality::Fast => "64-pt",
            SincQuality::Studio => "128-pt",
            SincQuality::Extreme => "256-pt",
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            SincQuality::Fast => "64-point Sinc • Efisien & Latensi Rendah",
            SincQuality::Studio => "128-point Sinc • Standar Studio Audiophile",
            SincQuality::Extreme => "256-point Sinc • Presisi Rekonstruksi Ekstrim",
        }
    }
}

/// Anti-aliasing roll-off / cutoff frequency profile for Rubato SRC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollOffProfile {
    /// 0.95 cutoff: transparent, crisp, and analytical (Default).
    Sharp,
    /// 0.90 cutoff: relaxed roll-off, warm tone, silky highs.
    Smooth,
    /// 0.98 cutoff: maximal Nyquist bandwidth preservation.
    Nyquist,
}

impl Default for RollOffProfile {
    fn default() -> Self {
        Self::Sharp
    }
}

impl RollOffProfile {
    pub fn cutoff(&self) -> f32 {
        match self {
            RollOffProfile::Sharp => 0.95,
            RollOffProfile::Smooth => 0.90,
            RollOffProfile::Nyquist => 0.98,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RollOffProfile::Sharp => "Sharp (0.95)",
            RollOffProfile::Smooth => "Smooth (0.90)",
            RollOffProfile::Nyquist => "Nyquist (0.98)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            RollOffProfile::Sharp => "Sharp",
            RollOffProfile::Smooth => "Smooth",
            RollOffProfile::Nyquist => "Nyquist",
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            RollOffProfile::Sharp => "0.95 Nyquist Cutoff • Jernih, Analitis & Zero Aliasing",
            RollOffProfile::Smooth => "0.90 Nyquist Cutoff • Lembut, Hangat & Rileks",
            RollOffProfile::Nyquist => "0.98 Nyquist Cutoff • Preservasi Frekuensi Maksimal",
        }
    }
}

/// Configuration settings for the audiophile Rubato Sinc resampler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResamplerSettings {
    pub quality: SincQuality,
    pub profile: RollOffProfile,
}

/// Strongly-typed identifier for all DSP effects and routing blocks in the signal chain.
/// Replaces fragile string name comparisons in the execution engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    GraphicEq,
    ParametricEq,
    TubeWarmer,
    StereoWidener,
    BassGuard,
    VolumeAttenuator,
    VolumeBooster,
    AnalogPreamp,
    DynamicLoudness,
    SteppedAttenuator,
    OpticalLeveler,
    StereoBalance,
    LowPassFilter,
    HighPassFilter,
    BandPassFilter,
    Crossover,
    Mixer,
    Passthrough,
    Splitter,
    Compressor,
    Convolution,
    TimeAlignment,
    StereoMatrix,
    Custom,
}

impl EffectType {
    /// Returns true if this node is purely a routing block (wire passthrough or splitter bus)
    /// without active spectral/dynamic alteration.
    #[inline]
    pub fn is_routing_only(&self) -> bool {
        matches!(self, EffectType::Passthrough | EffectType::Splitter)
    }

    /// Returns true if this node performs summing or crossfading of multiple inputs.
    #[inline]
    pub fn is_mixer(&self) -> bool {
        matches!(self, EffectType::Mixer)
    }

    /// Returns true if this node is an equalizer (Graphic or Parametric).
    #[inline]
    pub fn is_eq(&self) -> bool {
        matches!(self, EffectType::GraphicEq | EffectType::ParametricEq)
    }

    /// Human-readable default display name for this effect type.
    pub fn display_name(&self) -> &'static str {
        match self {
            EffectType::GraphicEq => "10-Band Graphic EQ",
            EffectType::ParametricEq => "Parametric Equalizer",
            EffectType::TubeWarmer => "Vacuum Tube Warmer",
            EffectType::StereoWidener => "Stereo Soundstage Widener",
            EffectType::BassGuard => "Subsonic Bass Guard",
            EffectType::VolumeAttenuator => "Volume Attenuator",
            EffectType::VolumeBooster => "Clean Headroom Booster",
            EffectType::AnalogPreamp => "Class-A Analog Preamp",
            EffectType::DynamicLoudness => "Dynamic Loudness Contour",
            EffectType::SteppedAttenuator => "Stepped Attenuator",
            EffectType::OpticalLeveler => "Optical Peak Leveler",
            EffectType::StereoBalance => "Stereo Balance Control",
            EffectType::LowPassFilter => "Low-Pass Filter",
            EffectType::HighPassFilter => "High-Pass Filter",
            EffectType::BandPassFilter => "Band-Pass Filter",
            EffectType::Crossover => "Linkwitz-Riley 2-Way Crossover",
            EffectType::Mixer => "Mixer (2-In Summer / Crossfader)",
            EffectType::Passthrough => "Passthrough (Wire)",
            EffectType::Splitter => "Splitter / Y-Branch Bus",
            EffectType::Compressor => "Studio Dynamic Compressor",
            EffectType::Convolution => "Convolution / FIR Engine",
            EffectType::TimeAlignment => "Time & Phase Alignment (Delay / Invert)",
            EffectType::StereoMatrix => "Stereo Matrix Router (M/S, Crossfeed, Swap)",
            EffectType::Custom => "Custom DSP Effect",
        }
    }
}

/// Amplitude interval domain [min, max] for Abstract Interpretation and static headroom analysis.
/// Represents the conservative minimum and maximum possible sample values at a given point in the signal chain.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AmplitudeInterval {
    pub min: f32,
    pub max: f32,
}

impl AmplitudeInterval {
    pub const FULL_SCALE: Self = Self { min: -1.0, max: 1.0 };
    pub const SILENCE: Self = Self { min: 0.0, max: 0.0 };

    #[inline(always)]
    pub fn new(min: f32, max: f32) -> Self {
        Self { min: min.min(max), max: min.max(max) }
    }

    #[inline(always)]
    pub fn symmetric(peak: f32) -> Self {
        let p = peak.abs();
        Self { min: -p, max: p }
    }

    #[inline(always)]
    pub fn peak_magnitude(&self) -> f32 {
        self.min.abs().max(self.max.abs())
    }

    #[inline(always)]
    pub fn peak_dbfs(&self) -> f32 {
        let mag = self.peak_magnitude();
        if mag <= 1e-9 {
            -120.0
        } else {
            20.0 * mag.log10()
        }
    }

    #[inline(always)]
    pub fn is_clipping(&self) -> bool {
        self.is_clipping_threshold(0.0)
    }

    #[inline(always)]
    pub fn is_clipping_threshold(&self, threshold_db: f32) -> bool {
        self.peak_dbfs() > threshold_db + 0.001
    }

    #[inline(always)]
    pub fn headroom_deficit_db(&self) -> f32 {
        self.headroom_deficit_threshold_db(0.0)
    }

    #[inline(always)]
    pub fn headroom_deficit_threshold_db(&self, threshold_db: f32) -> f32 {
        let db = self.peak_dbfs();
        if db > threshold_db {
            db - threshold_db
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn scale(&self, factor: f32) -> Self {
        let p1 = self.min * factor;
        let p2 = self.max * factor;
        Self::new(p1.min(p2), p1.max(p2))
    }

    #[inline(always)]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            min: self.min + other.min,
            max: self.max + other.max,
        }
    }

    #[inline(always)]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

impl Default for AmplitudeInterval {
    fn default() -> Self {
        Self::FULL_SCALE
    }
}

/// Trait for modular audio DSP effects that can be inserted into the signal graph.
pub trait Effect: Send + std::fmt::Debug {
    /// Process a contiguous block of left and right samples (SIMD auto-vectorizable).
    fn process_block(&mut self, left: &mut [f32], right: &mut [f32]);

    /// Single-frame helper for compatibility and fine-grained unit testing.
    fn process_frame(&mut self, l: f64, r: f64) -> (f64, f64) {
        let mut l_buf = [l as f32];
        let mut r_buf = [r as f32];
        self.process_block(&mut l_buf, &mut r_buf);
        (l_buf[0] as f64, r_buf[0] as f64)
    }

    fn reset(&mut self);
    fn set_sample_rate(&mut self, sample_rate: f32);
    fn name(&self) -> &str;
    /// Strongly-typed identifier for this effect.
    fn effect_type(&self) -> EffectType {
        EffectType::Custom
    }
    fn set_param(&mut self, _param_id: u32, _val: f32) {}
    fn mix_ratio(&self) -> Option<f32> {
        None
    }
    /// Returns current gain reduction in decibels (e.g. 0.0 to 30.0 dB for compressors/limiters)
    fn gain_reduction_db(&self) -> f32 {
        0.0
    }
    /// Pure worst-case static transfer function (Abstract Interpretation).
    /// Computes the conservative amplitude bound produced by this node.
    fn static_transfer(&self, input: AmplitudeInterval) -> AmplitudeInterval {
        input
    }
    fn clone_box(&self) -> Box<dyn Effect>;
}

impl Clone for Box<dyn Effect> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Channel contract defining audio stream layout across graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelContract {
    Stereo,
    Mono,
}

impl Default for ChannelContract {
    fn default() -> Self {
        Self::Stereo
    }
}

/// Formal errors produced during Graph IR DAG analysis and topological sorting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphCompilationError {
    /// Graph contains a cycle (directed loop). Includes the node IDs participating in the cycle.
    CycleDetected(Vec<u32>),
    /// A node cannot be reached by signal flowing from the Source node.
    UnreachableFromSource(u32),
    /// A node cannot reach the Endpoint node (severed path).
    DanglingPathToEndpoint(u32),
    /// The specified input port already has a connection and does not support multi-input.
    PortAlreadyConnected(u32, u8),
    /// The number of input connections to this node exceeds its maximum supported inputs.
    InputCountExceeded { node: u32, max: usize },
    /// More than one Endpoint node is present on the canvas.
    MultipleEndpointsNotSupported,
    /// No Source node exists in the graph.
    MissingSourceNode,
    /// No Endpoint node exists in the graph.
    MissingEndpointNode,
    /// The port index is outside the valid range for this node.
    InvalidPortIndex { node: u32, port: u8, max_ports: u8 },
}

impl std::fmt::Display for GraphCompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CycleDetected(nodes) => write!(f, "Siklus graph terdeteksi pada node: {:?}", nodes),
            Self::UnreachableFromSource(node) => write!(f, "Node {} tidak terjangkau dari Source", node),
            Self::DanglingPathToEndpoint(node) => write!(f, "Node {} tidak memiliki jalur menuju Endpoint", node),
            Self::PortAlreadyConnected(node, port) => write!(f, "Port {} pada node {} sudah terhubung", port, node),
            Self::InputCountExceeded { node, max } => write!(f, "Jumlah input pada node {} melebihi batas maksimum {}", node, max),
            Self::MultipleEndpointsNotSupported => write!(f, "Lebih dari satu Endpoint terdeteksi pada graph"),
            Self::MissingSourceNode => write!(f, "Node Source tidak ditemukan"),
            Self::MissingEndpointNode => write!(f, "Node Endpoint tidak ditemukan"),
            Self::InvalidPortIndex { node, port, max_ports } => write!(f, "Port {} pada node {} di luar rentang (maks: {})", port, node, max_ports),
        }
    }
}

impl std::error::Error for GraphCompilationError {}

/// Pure declarative node representation in Intermediate Representation (IR).
/// Decoupled from concrete trait object Box<dyn Effect>.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeIr {
    pub id: u32,
    pub name: String,
    pub effect_type: EffectType,
    pub input_ports: u8,
    pub output_ports: u8,
    pub channel_contract: ChannelContract,
    pub param_val: f32,
    pub param_min: f32,
    pub param_max: f32,
}

impl NodeIr {
    pub fn new(id: u32, name: String, effect_type: EffectType) -> Self {
        let (input_ports, output_ports) = match effect_type {
            EffectType::Mixer => (2, 1),
            EffectType::Splitter | EffectType::Crossover => (1, 2),
            _ => (1, 1),
        };
        Self {
            id,
            name,
            effect_type,
            input_ports,
            output_ports,
            channel_contract: ChannelContract::Stereo,
            param_val: 0.0,
            param_min: -24.0,
            param_max: 24.0,
        }
    }
}

/// Pure declarative edge representation in Intermediate Representation (IR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeIr {
    pub from_node: u32,
    pub from_port: u8,
    pub to_node: u32,
    pub to_port: u8,
}

impl EdgeIr {
    pub fn new(from_node: u32, from_port: u8, to_node: u32, to_port: u8) -> Self {
        Self { from_node, from_port, to_node, to_port }
    }
}

/// Pure declarative graph Intermediate Representation (IR).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphIr {
    pub nodes: Vec<NodeIr>,
    pub edges: Vec<EdgeIr>,
    pub source_id: u32,
    pub endpoint_id: u32,
}

impl GraphIr {
    pub fn new(source_id: u32, endpoint_id: u32) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            source_id,
            endpoint_id,
        }
    }

    /// Validates DAG integrity, detects cycles via Kahn's algorithm, verifies port constraints,
    /// and isolates dead nodes. Returns the verified topological order of flowing nodes.
    pub fn validate_and_sort(&self) -> Result<Vec<u32>, GraphCompilationError> {
        // 1. Verify Source and Endpoint nodes exist
        if !self.nodes.iter().any(|n| n.id == self.source_id) {
            return Err(GraphCompilationError::MissingSourceNode);
        }
        if !self.nodes.iter().any(|n| n.id == self.endpoint_id) {
            return Err(GraphCompilationError::MissingEndpointNode);
        }

        let node_map: HashMap<u32, &NodeIr> = self.nodes.iter().map(|n| (n.id, n)).collect();

        // 2. Validate Port indices and Input constraints
        let mut port_connections: HashMap<(u32, u8), Vec<EdgeIr>> = HashMap::new();
        for &edge in &self.edges {
            if let Some(from) = node_map.get(&edge.from_node) {
                if edge.from_port >= from.output_ports && from.output_ports > 0 {
                    return Err(GraphCompilationError::InvalidPortIndex {
                        node: edge.from_node,
                        port: edge.from_port,
                        max_ports: from.output_ports,
                    });
                }
            }
            if let Some(to) = node_map.get(&edge.to_node) {
                if edge.to_port >= to.input_ports && to.input_ports > 0 {
                    return Err(GraphCompilationError::InvalidPortIndex {
                        node: edge.to_node,
                        port: edge.to_port,
                        max_ports: to.input_ports,
                    });
                }
            }
            port_connections.entry((edge.to_node, edge.to_port)).or_default().push(edge);
        }

        // Validate that single-input ports do not have conflicting duplicate cables
        for (&(node_id, port), edges) in &port_connections {
            if let Some(node) = node_map.get(&node_id) {
                if edges.len() > 1 && !node.effect_type.is_mixer() {
                    return Err(GraphCompilationError::PortAlreadyConnected(node_id, port));
                }
            }
        }

        // 3. Forward reachability from Source
        let mut reachable_from_source = HashSet::new();
        let mut q_fwd = VecDeque::new();
        reachable_from_source.insert(self.source_id);
        q_fwd.push_back(self.source_id);

        while let Some(curr) = q_fwd.pop_front() {
            for edge in &self.edges {
                if edge.from_node == curr {
                    if reachable_from_source.insert(edge.to_node) {
                        q_fwd.push_back(edge.to_node);
                    }
                }
            }
        }

        if !reachable_from_source.contains(&self.endpoint_id) {
            return Err(GraphCompilationError::DanglingPathToEndpoint(self.source_id));
        }

        // 4. Backward reachability from Endpoint
        let mut can_reach_endpoint = HashSet::new();
        let mut q_back = VecDeque::new();
        can_reach_endpoint.insert(self.endpoint_id);
        q_back.push_back(self.endpoint_id);

        while let Some(curr) = q_back.pop_front() {
            for edge in &self.edges {
                if edge.to_node == curr {
                    if can_reach_endpoint.insert(edge.from_node) {
                        q_back.push_back(edge.from_node);
                    }
                }
            }
        }

        // Flowing nodes: strictly reachable from Source AND capable of reaching Endpoint.
        // Dead / disconnected nodes are completely isolated.
        let flowing_nodes: HashSet<u32> = self.nodes.iter()
            .map(|n| n.id)
            .filter(|id| reachable_from_source.contains(id) && can_reach_endpoint.contains(id))
            .collect();

        // 5. Kahn's Topological Sort with Formal Cycle Detection
        let mut in_degree: HashMap<u32, usize> = HashMap::new();
        let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();

        for &node_id in &flowing_nodes {
            in_degree.insert(node_id, 0);
            adj.insert(node_id, Vec::new());
        }

        for edge in &self.edges {
            if flowing_nodes.contains(&edge.from_node) && flowing_nodes.contains(&edge.to_node) {
                *in_degree.entry(edge.to_node).or_insert(0) += 1;
                adj.entry(edge.from_node).or_default().push(edge.to_node);
            }
        }

        let mut q = VecDeque::new();
        for (&node_id, &deg) in &in_degree {
            if deg == 0 {
                q.push_back(node_id);
            }
        }

        let mut ordered_node_ids = Vec::new();
        while let Some(curr) = q.pop_front() {
            ordered_node_ids.push(curr);
            if let Some(neighbors) = adj.get(&curr) {
                for &next in neighbors {
                    if let Some(deg) = in_degree.get_mut(&next) {
                        *deg -= 1;
                        if *deg == 0 {
                            q.push_back(next);
                        }
                    }
                }
            }
        }

        // Formal Cycle Detection: if sorted count != flowing count, cycle exists!
        if ordered_node_ids.len() != flowing_nodes.len() {
            let cycle_nodes: Vec<u32> = in_degree.iter()
                .filter(|(_, deg)| **deg > 0)
                .map(|(id, _)| *id)
                .collect();
            return Err(GraphCompilationError::CycleDetected(cycle_nodes));
        }

        Ok(ordered_node_ids)
    }
}

/// An individual processing node in the DAG signal chain.
pub struct GraphNode {
    pub id: u32,
    pub name: String,
    pub effect: Box<dyn Effect>,
}

impl Clone for GraphNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            effect: self.effect.clone(),
        }
    }
}

impl std::fmt::Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphNode")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

/// Directed edge in the signal chain graph with port/pin awareness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct GraphEdge {
    pub from_node: u32,
    pub from_port: u8,
    pub to_node: u32,
    pub to_port: u8,
}

impl GraphEdge {
    pub fn new(from_node: u32, to_node: u32) -> Self {
        Self {
            from_node,
            from_port: 0,
            to_node,
            to_port: 0,
        }
    }

    pub fn with_ports(from_node: u32, from_port: u8, to_node: u32, to_port: u8) -> Self {
        Self {
            from_node,
            from_port,
            to_node,
            to_port,
        }
    }
}

impl From<(u32, u32)> for GraphEdge {
    fn from((from, to): (u32, u32)) -> Self {
        Self::new(from, to)
    }
}

/// Compiled modular signal chain graph.
/// Contains the full DAG topology (nodes, edges, topological order) and completeness status.
pub struct SignalChainGraph {
    pub is_complete: bool,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub topological_order: Vec<u32>,
    pub effects: Vec<Box<dyn Effect>>,
}

impl Clone for SignalChainGraph {
    fn clone(&self) -> Self {
        Self {
            is_complete: self.is_complete,
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            topological_order: self.topological_order.clone(),
            effects: self.effects.clone(),
        }
    }
}

impl std::fmt::Debug for SignalChainGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = if !self.nodes.is_empty() {
            self.nodes.iter().map(|n| n.name.as_str()).collect()
        } else {
            self.effects.iter().map(|e| e.name()).collect()
        };
        f.debug_struct("SignalChainGraph")
            .field("is_complete", &self.is_complete)
            .field("nodes", &self.topological_order)
            .field("effects", &names)
            .field("edges", &self.edges)
            .finish()
    }
}

impl Default for SignalChainGraph {
    fn default() -> Self {
        Self {
            is_complete: true,
            nodes: Vec::new(),
            edges: Vec::new(),
            topological_order: Vec::new(),
            effects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Play(PathBuf),
    QueueNext(PathBuf),
    ClearNext,
    Pause,
    Resume,
    Restart,
    Stop,
    Seek(f32),
    SetVolume(f32),
    SetMute(bool),
    SetEqBand { index: usize, gain_db: f32 },
    SetNodeParam { node_id: u32, param_id: u32, value: f32 },
    SetEqBypass(bool),
    SetLimiterEnabled(bool),
    /// Atomically switch the entire signal chain mode.
    SetChainMode(ChainMode),
    /// Select specific audio hardware output device by friendly name (None = default OS endpoint)
    SetOutputDevice(Option<String>),
    /// Dynamic modular signal chain graph compiled from patchbay canvas
    SetSignalChainGraph(SignalChainGraph),
    /// Customize Sinc resampler quality and roll-off profile
    SetResamplerSettings(ResamplerSettings),
    /// Set Crossfade duration in seconds (0.0 = pure gapless, >0.0 = smooth overlap)
    SetCrossfadeSecs(f32),
}

/// Real-time non-blocking command delivered to audio callback DSP executor
#[derive(Debug, Clone)]
pub enum DspCommand {
    SetNodeParam { node_id: u32, param_id: u32, value: f32 },
    SetEqBand { index: usize, gain_db: f32 },
    SetEqBypass(bool),
    SetLimiterEnabled(bool),
    SetChainMode(ChainMode),
    SetSignalChainGraph(SignalChainGraph),
    SetCompiledPlan(Box<crate::audio::dsp::executor::ExecutionPlan>),
    ResetStates,
    SetSampleRate(f32),
}

/// Fast lightweight Xorshift32 PRNG for TPDF Dithering (-1.0 to +1.0 LSB).
#[inline(always)]
pub fn tpdf_dither_val(state: &mut u32) -> f32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    let r1 = (x as f32) / (u32::MAX as f32);

    let mut y = x.wrapping_add(0xdeadbeef);
    y ^= y << 13;
    y ^= y >> 17;
    y ^= y << 5;
    let r2 = (y as f32) / (u32::MAX as f32);

    r1 - r2
}

/// Lossless precision 32-bit integer PCM encoding.
/// For 24-bit within 32-bit container (source_bit_depth <= 24), scales exactly by 8388608.0 and shifts << 8.
/// For 32-bit integer, scales by 2147483648.0.
#[inline(always)]
pub fn encode_sample_i32(s: f32, source_bit_depth: u8) -> i32 {
    if source_bit_depth <= 24 {
        let scaled = (s as f64 * 8388608.0).round().clamp(-8388608.0, 8388607.0) as i32;
        scaled << 8
    } else {
        (s as f64 * 2147483648.0).round().clamp(-2147483648.0, 2147483647.0) as i32
    }
}

/// 16-bit integer PCM encoding with precision rounding and optional TPDF dither.
#[inline(always)]
pub fn encode_sample_i16(s: f32, should_dither: bool, dither_state: &mut u32) -> i16 {
    if should_dither {
        let dither = tpdf_dither_val(dither_state);
        let sample_val = s as f64 * 32768.0 + dither as f64;
        sample_val.round().clamp(-32768.0, 32767.0) as i16
    } else {
        (s as f64 * 32768.0).round().clamp(-32768.0, 32767.0) as i16
    }
}

/// Pure function to calculate effective volume multiplier.
/// - If muted: strictly 0.0 (silence on both Bit-Perfect and DSP modes).
/// - If Bit-Perfect (and not muted): strictly 1.0 (software volume slider bypassed).
/// - If DSP Active / OS Resampled (and not muted): software volume multiplier.
#[inline(always)]
pub fn calculate_effective_volume(
    is_bit_perfect: bool,
    is_muted: bool,
    software_volume: f32,
) -> f32 {
    if is_muted {
        0.0
    } else if is_bit_perfect {
        1.0
    } else if software_volume <= 0.0001 {
        0.0
    } else {
        software_volume
    }
}

/// Report of the actual signal path configuration determined by the engine.
/// Sent after every Play and SetChainMode to let the UI display truthful info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalPathReport {
    pub device_name: String,
    pub source_rate: u32,
    pub device_rate: u32,
    pub source_bit_depth: u8,
    pub format_desc: String,
    pub requested_mode: ChainMode,
    pub actual_mode: ChainMode,
    pub is_exclusive: bool,
    pub is_resampling: bool,
    pub is_dsp_active: bool,
    pub is_volume_applied: bool,
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    TrackStarted {
        duration: f32,
        sample_rate: u32,
        channels: u16,
    },
    TrackTransitioned {
        path: Option<PathBuf>,
        duration: f32,
        sample_rate: u32,
    },
    Position(f32),
    StateChanged(PlaybackState),
    TrackEnded,
    Error(String),
    DeviceDisconnected {
        device_name: String,
        error: String,
    },
    /// Driver menolak format audio (mis. Realtek tidak support 44.1 kHz) — BUKAN hot-unplug fisik.
    /// Engine sudah fallback ke OS Resampled. Playback tetap berjalan.
    StreamFallback {
        device_name: String,
        reason: String,
    },
    /// Engine reports the actual signal path after Play or mode change.
    SignalPath(SignalPathReport),
}

