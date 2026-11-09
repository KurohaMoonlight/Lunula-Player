use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use crate::audio::types::{AmplitudeInterval, EffectType, GraphEdge, SignalChainGraph};

/// Safety status classification for an individual node in the modular signal graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeSafetyStatus {
    /// Safe: Amplitude peak <= 1.0 (<= 0.0 dBFS). Zero risk of clipping.
    Safe,
    /// Origin of Overdrive / Clipping (Biang Kerok):
    /// The input was safe (<= 1.0), but this node's processing or multi-input summing caused the signal to exceed 1.0 (> 0.0 dBFS).
    OriginOfClipping,
    /// Overdriven Downstream (Korban):
    /// This node receives an incoming signal that was already overdriven (> 1.0) by an upstream node.
    OverdrivenDownstream,
}

/// Headroom risk severity tier based on decibel overflow above 0 dBFS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// Peak <= 0.0 dBFS: Linear transparent headroom.
    Safe,
    /// Peak between +0.01 dBFS and +3.0 dBFS: Minor headroom deficit, moderate risk on dense audio.
    MildWarning,
    /// Peak > +3.0 dBFS: Extreme headroom deficit, severe clipping distortion hazard.
    SevereHazard,
}

impl DiagnosticSeverity {
    pub fn from_peak_and_threshold(peak_dbfs: f32, threshold_db: f32) -> Self {
        let deficit = peak_dbfs - threshold_db;
        if deficit <= 0.001 {
            DiagnosticSeverity::Safe
        } else if deficit <= 3.001 {
            DiagnosticSeverity::MildWarning
        } else {
            DiagnosticSeverity::SevereHazard
        }
    }

    pub fn from_peak_dbfs(peak_dbfs: f32) -> Self {
        Self::from_peak_and_threshold(peak_dbfs, 0.0)
    }

    pub fn label(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Safe => "Aman (0 dBFS Headroom)",
            DiagnosticSeverity::MildWarning => "Peringatan Headroom (+0.1 s/d +3.0 dBFS)",
            DiagnosticSeverity::SevereHazard => "Bahaya Hard Clipping (> +3.0 dBFS)",
        }
    }

    pub fn badge_text(&self, peak_dbfs: f32) -> String {
        match self {
            DiagnosticSeverity::Safe => "Safe (0 dBFS)".to_string(),
            DiagnosticSeverity::MildWarning => format!("⚠️ +{:.1} dBFS", peak_dbfs),
            DiagnosticSeverity::SevereHazard => format!("💥 +{:.1} dBFS", peak_dbfs),
        }
    }
}

/// Structured actionable remedy that can be applied manually or via 1-click quick-fix buttons.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RemedyAction {
    /// Suggest adjusting a specific parameter on this node to a safe value.
    AdjustParameter {
        param_id: u32,
        suggested_value: f32,
    },
    /// Suggest inserting a passive attenuator pad with recommended decibel attenuation.
    InsertAttenuator {
        suggested_attenuation_db: f32,
    },
    /// Suggest engaging the output stage master soft limiter.
    EnableMasterLimiter,
    /// Suggest replacing direct cable summing with a crossfading Mixer bus.
    UseMixerBus,
}

/// Actionable prescription containing human-readable explanation and structured executable remedy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionableRemedy {
    pub title: String,
    pub description: String,
    pub action: RemedyAction,
}

/// Detailed static analysis report for an individual DAG node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeAnalysisReport {
    pub node_id: u32,
    pub node_name: String,
    pub input_interval: AmplitudeInterval,
    pub output_interval: AmplitudeInterval,
    pub peak_dbfs: f32,
    pub headroom_deficit_db: f32,
    pub status: NodeSafetyStatus,
    pub severity: DiagnosticSeverity,
    pub is_culprit: bool,
    pub reason: String,
    pub recommendations: Vec<String>,
    pub remedies: Vec<ActionableRemedy>,
}

/// Static analysis report for an individual patch cable (graph edge).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EdgeAnalysisReport {
    pub from_node: u32,
    pub from_port: u8,
    pub to_node: u32,
    pub to_port: u8,
    pub interval: AmplitudeInterval,
    pub peak_dbfs: f32,
    /// True if the signal traversing this cable exceeds 0 dBFS (peak > 1.0).
    pub is_hot: bool,
}

fn default_true() -> bool {
    true
}

/// Comprehensive static analysis report for the entire modular signal chain.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphAnalysisReport {
    /// True if the entire signal chain produces 0% clipping risk at every node and terminal.
    pub is_chain_safe: bool,
    /// Maximum peak decibel value found anywhere across the entire signal graph.
    pub max_peak_dbfs: f32,
    /// Maximum headroom deficit in decibels (> 0.0 dB means signal overflows full-scale).
    pub max_headroom_deficit_db: f32,
    /// IDs of nodes identified as the primary culprits (Origin of Clipping).
    pub culprit_nodes: Vec<u32>,
    /// Individual node reports indexed by node ID.
    pub node_reports: HashMap<u32, NodeAnalysisReport>,
    /// Cable status reports for visual heatmap rendering.
    pub edge_reports: Vec<EdgeAnalysisReport>,
    /// Final amplitude interval delivered to the DAC output terminals.
    pub terminal_interval: AmplitudeInterval,
    /// Contextual source peak interval used for this analysis.
    #[serde(default)]
    pub source_interval: AmplitudeInterval,
    /// Title of the active track if context-aware analysis was applied.
    #[serde(default)]
    pub context_track_title: Option<String>,
    /// Track peak in dBFS used as the reference baseline.
    #[serde(default)]
    pub context_peak_dbfs: Option<f32>,
    /// Whether clipping detection was enabled during this analysis.
    #[serde(default = "default_true")]
    pub detection_enabled: bool,
    /// Headroom margin / threshold in dBFS used for this analysis (default 0.0 dBFS).
    #[serde(default)]
    pub threshold_dbfs: f32,
}

impl GraphAnalysisReport {
    /// Returns true if a specific patch cable is carrying an overdriven / hot signal (> 0 dBFS).
    pub fn is_edge_hot(&self, from_node: u32, from_port: u8, to_node: u32, to_port: u8) -> bool {
        self.edge_reports.iter().any(|e| {
            e.from_node == from_node && e.from_port == from_port && e.to_node == to_node && e.to_port == to_port && e.is_hot
        })
    }

    /// Returns the diagnostic report for a specific node ID, if present.
    pub fn get_node_report(&self, node_id: u32) -> Option<&NodeAnalysisReport> {
        self.node_reports.get(&node_id)
    }

    /// Returns true if the node is tagged as the primary origin of clipping.
    pub fn is_node_culprit(&self, node_id: u32) -> bool {
        self.culprit_nodes.contains(&node_id)
    }

    /// Recursively traces backwards from `node_id` along incoming edges to find all ancestor nodes
    /// that are culprits (`OriginOfClipping`). Solves the "Why is my DAC/speaker clipping?" root-cause inquiry.
    pub fn get_culprit_path(&self, node_id: u32, graph: &SignalChainGraph) -> Vec<u32> {
        let mut culprits = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(node_id);
        visited.insert(node_id);

        while let Some(curr) = queue.pop_front() {
            if self.is_node_culprit(curr) && curr != node_id && !culprits.contains(&curr) {
                culprits.push(curr);
            }

            for edge in graph.edges.iter().filter(|e| e.to_node == curr) {
                if visited.insert(edge.from_node) {
                    queue.push_back(edge.from_node);
                }
            }
        }

        culprits
    }
}

/// Real-time static dataflow analysis engine for Lunula's modular signal chain (Abstract Interpretation).
/// Evaluates the entire DAG in topological sequence using contextual source amplitude interval with default 0 dBFS margin.
pub fn analyze_signal_chain_with_context(
    graph: &SignalChainGraph,
    source_interval: AmplitudeInterval,
    track_title: Option<String>,
) -> GraphAnalysisReport {
    analyze_signal_chain_advanced(graph, source_interval, track_title, 0.0, true)
}

/// Advanced real-time static dataflow analysis engine supporting configurable sensitivity threshold (margin)
/// and complete on/off suppression.
pub fn analyze_signal_chain_advanced(
    graph: &SignalChainGraph,
    source_interval: AmplitudeInterval,
    track_title: Option<String>,
    threshold_db: f32,
    detection_enabled: bool,
) -> GraphAnalysisReport {
    if !graph.is_complete || graph.nodes.is_empty() {
        return GraphAnalysisReport {
            is_chain_safe: true,
            max_peak_dbfs: -120.0,
            max_headroom_deficit_db: 0.0,
            culprit_nodes: Vec::new(),
            node_reports: HashMap::new(),
            edge_reports: Vec::new(),
            terminal_interval: AmplitudeInterval::SILENCE,
            source_interval,
            context_track_title: track_title,
            context_peak_dbfs: Some(source_interval.peak_dbfs()),
            detection_enabled,
            threshold_dbfs: threshold_db,
        };
    }

    let mut node_outputs: HashMap<u32, AmplitudeInterval> = HashMap::with_capacity(graph.nodes.len());
    let mut node_reports: HashMap<u32, NodeAnalysisReport> = HashMap::with_capacity(graph.nodes.len());
    let mut edge_reports: Vec<EdgeAnalysisReport> = Vec::with_capacity(graph.edges.len());
    let mut culprit_nodes: Vec<u32> = Vec::new();

    let mut max_peak_dbfs: f32 = -120.0;
    let mut max_headroom_deficit_db: f32 = 0.0;

    for &node_id in &graph.topological_order {
        let node = match graph.nodes.iter().find(|n| n.id == node_id) {
            Some(n) => n,
            None => continue,
        };

        let eff_type = node.effect.effect_type();

        let incoming: Vec<&GraphEdge> = graph.edges
            .iter()
            .filter(|e| e.to_node == node_id && node_outputs.contains_key(&e.from_node))
            .collect();

        let mut multi_summing_culprit = false;

        let input_interval = if incoming.is_empty() {
            // Source root node (or unconnected starting block): contextual source interval
            source_interval
        } else if eff_type.is_mixer() || node.effect.mix_ratio().is_some() {
            // Mixer 2-port crossfade convex combination: (1 - m)*A + m*B
            let mut slot_a: Option<AmplitudeInterval> = None;
            let mut slot_b: Option<AmplitudeInterval> = None;

            for edge in &incoming {
                if edge.to_port == 0 && slot_a.is_none() {
                    slot_a = Some(node_outputs[&edge.from_node]);
                } else if edge.to_port == 1 && slot_b.is_none() {
                    slot_b = Some(node_outputs[&edge.from_node]);
                } else if slot_a.is_none() {
                    slot_a = Some(node_outputs[&edge.from_node]);
                } else if slot_b.is_none() {
                    slot_b = Some(node_outputs[&edge.from_node]);
                }
            }

            let m = node.effect.mix_ratio().unwrap_or(0.5).clamp(0.0, 1.0);
            let ia = slot_a.unwrap_or(AmplitudeInterval::SILENCE);
            let ib = slot_b.unwrap_or(AmplitudeInterval::SILENCE);
            ia.scale(1.0 - m).add(&ib.scale(m))
        } else if incoming.len() == 1 {
            node_outputs[&incoming[0].from_node]
        } else {
            // Multi-branch summing bus: conservative in-phase worst-case scaling by 1/sqrt(N)
            let n_count = incoming.len() as f32;
            let scale = 1.0f32 / n_count.sqrt();

            let mut sum_int = AmplitudeInterval::SILENCE;
            let mut all_parents_safe = true;

            for edge in &incoming {
                let parent_int = node_outputs[&edge.from_node];
                if detection_enabled && parent_int.is_clipping_threshold(threshold_db) {
                    all_parents_safe = false;
                }
                sum_int = sum_int.add(&parent_int);
            }

            let scaled_sum = sum_int.scale(scale);
            if detection_enabled && scaled_sum.is_clipping_threshold(threshold_db) && all_parents_safe {
                multi_summing_culprit = true;
            }
            scaled_sum
        };

        // Propagate through the node's transfer function
        let output_interval = node.effect.static_transfer(input_interval);
        node_outputs.insert(node_id, output_interval);

        let out_peak_db = output_interval.peak_dbfs();
        let out_deficit = if detection_enabled {
            output_interval.headroom_deficit_threshold_db(threshold_db)
        } else {
            0.0
        };

        if out_peak_db > max_peak_dbfs {
            max_peak_dbfs = out_peak_db;
        }
        if out_deficit > max_headroom_deficit_db {
            max_headroom_deficit_db = out_deficit;
        }

        // Generate edge reports for all outgoing cables originating from this node
        for edge in graph.edges.iter().filter(|e| e.from_node == node_id) {
            edge_reports.push(EdgeAnalysisReport {
                from_node: edge.from_node,
                from_port: edge.from_port,
                to_node: edge.to_node,
                to_port: edge.to_port,
                interval: output_interval,
                peak_dbfs: out_peak_db,
                is_hot: detection_enabled && output_interval.is_clipping_threshold(threshold_db),
            });
        }

        // Diagnosis Reasoning & Actionable Remedies Generation
        let (status, is_culprit, reason, recommendations, remedies) = if !detection_enabled {
            (
                NodeSafetyStatus::Safe,
                false,
                "Deteksi headroom dan clipping dinonaktifkan oleh pengguna.".to_string(),
                Vec::new(),
                Vec::new(),
            )
        } else if multi_summing_culprit {
            let n = incoming.len();
            let deficit = input_interval.headroom_deficit_threshold_db(threshold_db);
            culprit_nodes.push(node_id);
            (
                NodeSafetyStatus::OriginOfClipping,
                true,
                format!(
                    "Port input menerima {} koneksi kabel paralel. Penjumlahan kasus terburuk (sinyal in-phase) melipatgandakan energi hingga +{:.2} dBFS (melebihi ambang batas {:+.1} dBFS).",
                    n, deficit, threshold_db
                ),
                vec![
                    "Gunakan modul Mixer / Crossfader agar pembagian sinyal terjamin seimbang (total gain 100%).".to_string(),
                    format!("Sisipkan modul Volume Attenuator (-{:.1} dB) tepat sebelum atau setelah titik percabangan ini.", deficit),
                    "Aktifkan opsi Soft Limiter di stage master output untuk meredam luapan puncak.".to_string(),
                ],
                vec![
                    ActionableRemedy {
                        title: "Gunakan Modul Mixer".to_string(),
                        description: "Ganti penggabungan kabel langsung dengan modul Mixer 2-channel crossfader.".to_string(),
                        action: RemedyAction::UseMixerBus,
                    },
                    ActionableRemedy {
                        title: format!("Sisipkan Attenuator -{:.1} dB", deficit),
                        description: format!("Rendam luapan energi in-phase sebesar -{:.1} dB agar kembali ke batas aman.", deficit),
                        action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                    },
                    ActionableRemedy {
                        title: "Aktifkan Master Soft Limiter".to_string(),
                        description: "Kunci output DAC dengan soft-limiter transparan.".to_string(),
                        action: RemedyAction::EnableMasterLimiter,
                    },
                ],
            )
        } else if incoming.is_empty() && output_interval.is_clipping_threshold(threshold_db) {
            // Source root node clipping (the audio track itself is mastered above threshold or inter-sample peak)
            culprit_nodes.push(node_id);
            let deficit = output_interval.headroom_deficit_threshold_db(threshold_db);
            (
                NodeSafetyStatus::OriginOfClipping,
                true,
                format!(
                    "File audio sumber terdeteksi memiliki level puncak melebihi ambang batas {:+.1} dBFS (+{:.2} dBFS) sebelum efek diaplikasikan (inter-sample peak atau mastering panas).",
                    threshold_db, out_peak_db
                ),
                vec![
                    format!("Aktifkan ReplayGain dengan Preamp negatif (-{:.1} dB) untuk memberikan headroom.", deficit),
                    "Aktifkan opsi Master Soft Limiter pada output.".to_string(),
                ],
                vec![
                    ActionableRemedy {
                        title: format!("Beri Headroom Preamp (-{:.1} dB)", deficit),
                        description: "Turunkan level sumber agar tidak melampaui batas aman.".to_string(),
                        action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                    },
                    ActionableRemedy {
                        title: "Aktifkan Master Soft Limiter".to_string(),
                        description: "Kunci output DAC dengan soft-limiter transparan.".to_string(),
                        action: RemedyAction::EnableMasterLimiter,
                    },
                ],
            )
        } else if !input_interval.is_clipping_threshold(threshold_db) && output_interval.is_clipping_threshold(threshold_db) {
            culprit_nodes.push(node_id);
            let deficit = output_interval.headroom_deficit_threshold_db(threshold_db);

            let (r_text, r_recs, r_rems) = match eff_type {
                EffectType::VolumeBooster => (
                    format!("Clean Headroom Booster meningkatkan amplitudo sinyal hingga +{:.2} dBFS, melampaui batas linear 0 dBFS.", out_peak_db),
                    vec![
                        format!("Turunkan gain booster sebesar -{:.1} dB agar sinyal tetap berada di bawah 0 dBFS.", deficit),
                        "Pasang modul Volume Attenuator setelah booster ini.".to_string(),
                        "Aktifkan Soft Limiter di output master.".to_string(),
                    ],
                    vec![
                        ActionableRemedy {
                            title: format!("Turunkan Booster (-{:.1} dB)", deficit),
                            description: "Sesuaikan gain booster agar output puncak tepat pada 0 dBFS.".to_string(),
                            action: RemedyAction::AdjustParameter { param_id: 0, suggested_value: (out_peak_db - deficit).max(0.0) },
                        },
                        ActionableRemedy {
                            title: format!("Pasang Attenuator -{:.1} dB", deficit),
                            description: "Sisipkan peredam pasif setelah booster.".to_string(),
                            action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                        },
                        ActionableRemedy {
                            title: "Aktifkan Master Limiter".to_string(),
                            description: "Engage soft-limiter pada output master.".to_string(),
                            action: RemedyAction::EnableMasterLimiter,
                        },
                    ],
                ),
                EffectType::StereoWidener => (
                    format!("Pelebaran soundstage stereo melipatgandakan komponen audio out-of-phase secara agresif hingga +{:.2} dBFS tanpa limiter internal.", out_peak_db),
                    vec![
                        "Turunkan slider Width mendekati 1.0x (normal stereo).".to_string(),
                        format!("Sisipkan Volume Attenuator (-{:.1} dB) setelah Stereo Widener.", deficit),
                        "Aktifkan Soft Limiter pada output.".to_string(),
                    ],
                    vec![
                        ActionableRemedy {
                            title: "Turunkan Width ke 1.0x".to_string(),
                            description: "Kembalikan pelebaran spasial stereo ke skala normal (100%).".to_string(),
                            action: RemedyAction::AdjustParameter { param_id: 0, suggested_value: 0.0 }, // 0% parameter = 1.0x width
                        },
                        ActionableRemedy {
                            title: format!("Pasang Attenuator -{:.1} dB", deficit),
                            description: "Redam komponen side yang melebar sebelum menuju DAC.".to_string(),
                            action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                        },
                        ActionableRemedy {
                            title: "Aktifkan Master Limiter".to_string(),
                            description: "Engage soft-limiter pada output master.".to_string(),
                            action: RemedyAction::EnableMasterLimiter,
                        },
                    ],
                ),
                EffectType::AnalogPreamp => (
                    format!("Analog Preamp & Trim memberikan dorongan penguatan melebihi ambang batas 0 dBFS (+{:.2} dBFS).", out_peak_db),
                    vec![
                        format!("Turunkan trim gain sebesar -{:.1} dB.", deficit),
                        "Gunakan attenuator atau compressor downstream.".to_string(),
                    ],
                    vec![
                        ActionableRemedy {
                            title: format!("Turunkan Trim Preamp (-{:.1} dB)", deficit),
                            description: "Reduksi penguatan konsol analog ke batas aman 0 dBFS.".to_string(),
                            action: RemedyAction::AdjustParameter { param_id: 0, suggested_value: 0.0 },
                        },
                        ActionableRemedy {
                            title: format!("Pasang Attenuator -{:.1} dB", deficit),
                            description: "Pasang peredam attenuator setelah preamp.".to_string(),
                            action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                        },
                    ],
                ),
                EffectType::Compressor => (
                    format!("Makeup gain pada compressor mengangkat transien suara awal hingga melampaui 0 dBFS (+{:.2} dBFS).", out_peak_db),
                    vec![
                        format!("Kurangi makeup gain compressor sebesar -{:.1} dB.", deficit),
                        "Percepat attack time untuk menangkap transien lebih awal.".to_string(),
                    ],
                    vec![
                        ActionableRemedy {
                            title: format!("Kurangi Makeup Gain (-{:.1} dB)", deficit),
                            description: "Turunkan kompensasi makeup gain compressor.".to_string(),
                            action: RemedyAction::AdjustParameter { param_id: 4, suggested_value: 0.0 },
                        },
                        ActionableRemedy {
                            title: "Aktifkan Master Limiter".to_string(),
                            description: "Cegah transien escape dengan soft-limiter.".to_string(),
                            action: RemedyAction::EnableMasterLimiter,
                        },
                    ],
                ),
                EffectType::DynamicLoudness => (
                    format!("Kompensasi Dynamic Loudness mengangkat bass dan treble melebihi kapasitas headroom full-scale (+{:.2} dBFS).", out_peak_db),
                    vec![
                        "Kurangi depth kontur Dynamic Loudness.".to_string(),
                        format!("Pasang redaman Volume Attenuator (-{:.1} dB).", deficit),
                    ],
                    vec![
                        ActionableRemedy {
                            title: "Kurangi Depth Loudness".to_string(),
                            description: "Turunkan intensitas kurva kontur akustik.".to_string(),
                            action: RemedyAction::AdjustParameter { param_id: 0, suggested_value: 0.0 },
                        },
                        ActionableRemedy {
                            title: format!("Pasang Attenuator -{:.1} dB", deficit),
                            description: "Redam luapan frekuensi ekstrem.".to_string(),
                            action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                        },
                    ],
                ),
                EffectType::LowPassFilter | EffectType::HighPassFilter | EffectType::BandPassFilter => (
                    format!("Resonansi Fringe Q pada filter menciptakan lonjakan puncak frekuensi cutoff hingga +{:.2} dBFS.", out_peak_db),
                    vec![
                        "Turunkan nilai Fringe Q di bawah 1.0 agar respon filter flat tanpa resonansi tajam.".to_string(),
                        format!("Pasang peredam gain (-{:.1} dB).", deficit),
                    ],
                    vec![
                        ActionableRemedy {
                            title: "Setel Fringe Q ke 0.707 (Butterworth Flat)".to_string(),
                            description: "Hilangkan resonansi puncak pada frekuensi sudut filter.".to_string(),
                            action: RemedyAction::AdjustParameter { param_id: 2, suggested_value: 0.7071 },
                        },
                        ActionableRemedy {
                            title: format!("Pasang Attenuator -{:.1} dB", deficit),
                            description: "Redam lonjakan resonansi filter.".to_string(),
                            action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                        },
                    ],
                ),
                _ => (
                    format!("Modul ini melipatgandakan sinyal input melampaui batas aman full-scale (+{:.2} dBFS).", out_peak_db),
                    vec![
                        format!("Reduksi gain modul ini sebesar -{:.1} dB.", deficit),
                        "Pasang limiter atau attenuator downstream.".to_string(),
                    ],
                    vec![
                        ActionableRemedy {
                            title: format!("Pasang Attenuator -{:.1} dB", deficit),
                            description: "Redam luapan energi sebelum ke DAC.".to_string(),
                            action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                        },
                    ],
                ),
            };

            (NodeSafetyStatus::OriginOfClipping, true, r_text, r_recs, r_rems)
        } else if input_interval.is_clipping_threshold(threshold_db) {
            let deficit = input_interval.headroom_deficit_threshold_db(threshold_db);
            (
                NodeSafetyStatus::OverdrivenDownstream,
                false,
                format!(
                    "Node ini menerima sinyal audio yang sudah panas (+{:.2} dBFS) dari node hulu. Amplitudo keluaran mencapai +{:.2} dBFS.",
                    input_interval.peak_dbfs(), out_peak_db
                ),
                vec![
                    "Periksa node hulu (upstream) bertanda 💥 yang menjadi biang kerok peluapan sinyal.".to_string(),
                    format!("Sisipkan modul Volume Attenuator (-{:.1} dB) sebelum node ini.", deficit),
                ],
                vec![
                    ActionableRemedy {
                        title: format!("Sisipkan Attenuator -{:.1} dB di Input", deficit),
                        description: "Cegah sinyal overdriven memasuki node ini.".to_string(),
                        action: RemedyAction::InsertAttenuator { suggested_attenuation_db: -deficit },
                    },
                    ActionableRemedy {
                        title: "Aktifkan Master Limiter".to_string(),
                        description: "Cegah clipping pada output terminal master.".to_string(),
                        action: RemedyAction::EnableMasterLimiter,
                    },
                ],
            )
        } else {
            let margin_info = if threshold_db.abs() < 0.001 {
                "0 dBFS".to_string()
            } else {
                format!("{:+.1} dBFS", threshold_db)
            };
            (
                NodeSafetyStatus::Safe,
                false,
                format!("Headroom aman (amplitudo berada dalam batas ambang linear {}).", margin_info),
                Vec::new(),
                Vec::new(),
            )
        };

        let severity = if !detection_enabled {
            DiagnosticSeverity::Safe
        } else {
            DiagnosticSeverity::from_peak_and_threshold(out_peak_db, threshold_db)
        };

        node_reports.insert(node_id, NodeAnalysisReport {
            node_id,
            node_name: node.name.clone(),
            input_interval,
            output_interval,
            peak_dbfs: out_peak_db,
            headroom_deficit_db: out_deficit,
            status,
            severity,
            is_culprit,
            reason,
            recommendations,
            remedies,
        });
    }

    // Terminal summation (nodes without outgoing edges to topological nodes)
    let terminal_slots: Vec<u32> = graph.nodes.iter()
        .map(|n| n.id)
        .filter(|&id| {
            !graph.edges.iter().any(|e| e.from_node == id && graph.topological_order.contains(&e.to_node))
        })
        .collect();

    let terminal_interval = if terminal_slots.len() == 1 {
        node_outputs.get(&terminal_slots[0]).copied().unwrap_or(AmplitudeInterval::SILENCE)
    } else if !terminal_slots.is_empty() {
        let m = terminal_slots.len() as f32;
        let scale = 1.0f32 / m.sqrt();
        let mut sum = AmplitudeInterval::SILENCE;
        for &t_id in &terminal_slots {
            if let Some(&int) = node_outputs.get(&t_id) {
                sum = sum.add(&int);
            }
        }
        sum.scale(scale)
    } else {
        AmplitudeInterval::SILENCE
    };

    let is_chain_safe = if !detection_enabled {
        true
    } else {
        culprit_nodes.is_empty() && !terminal_interval.is_clipping_threshold(threshold_db)
    };

    GraphAnalysisReport {
        is_chain_safe,
        max_peak_dbfs,
        max_headroom_deficit_db,
        culprit_nodes,
        node_reports,
        edge_reports,
        terminal_interval,
        source_interval,
        context_track_title: track_title,
        context_peak_dbfs: Some(source_interval.peak_dbfs()),
        detection_enabled,
        threshold_dbfs: threshold_db,
    }
}

/// Real-time static dataflow analysis engine evaluating the entire DAG using pure worst-case [-1.0, 1.0] (0 dBFS) arithmetic.
pub fn analyze_signal_chain(graph: &SignalChainGraph) -> GraphAnalysisReport {
    analyze_signal_chain_with_context(graph, AmplitudeInterval::FULL_SCALE, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::{Effect, GraphEdge, GraphNode, SignalChainGraph};
    use crate::audio::dsp::effects::{PassthroughEffect, StereoWidenerEffect, VolumeAttenuatorEffect, VolumeBoosterEffect};

    #[test]
    fn test_interval_arithmetic_properties() {
        let a = AmplitudeInterval::new(-0.5, 0.5);
        assert_eq!(a.peak_magnitude(), 0.5);
        assert!(!a.is_clipping());
        assert_eq!(a.headroom_deficit_db(), 0.0);

        let scaled = a.scale(3.0);
        assert_eq!(scaled.min, -1.5);
        assert_eq!(scaled.max, 1.5);
        assert!(scaled.is_clipping());
        assert!((scaled.peak_dbfs() - 3.5218).abs() < 0.01);
        assert!((scaled.headroom_deficit_db() - 3.5218).abs() < 0.01);
    }

    #[test]
    fn test_diagnostic_severity_levels() {
        assert_eq!(DiagnosticSeverity::from_peak_dbfs(-6.0), DiagnosticSeverity::Safe);
        assert_eq!(DiagnosticSeverity::from_peak_dbfs(0.0), DiagnosticSeverity::Safe);
        assert_eq!(DiagnosticSeverity::from_peak_dbfs(1.5), DiagnosticSeverity::MildWarning);
        assert_eq!(DiagnosticSeverity::from_peak_dbfs(3.0), DiagnosticSeverity::MildWarning);
        assert_eq!(DiagnosticSeverity::from_peak_dbfs(3.1), DiagnosticSeverity::SevereHazard);
        assert_eq!(DiagnosticSeverity::from_peak_dbfs(6.0), DiagnosticSeverity::SevereHazard);
    }

    #[test]
    fn test_stereo_widener_static_transfer() {
        let widener = StereoWidenerEffect::new(2.0);
        let input = AmplitudeInterval::FULL_SCALE;
        let output = widener.static_transfer(input);

        assert_eq!(output.min, -2.0);
        assert_eq!(output.max, 2.0);
        assert!(output.is_clipping());
        assert!((output.peak_dbfs() - 6.0206).abs() < 0.01);
    }

    #[test]
    fn test_volume_booster_static_transfer() {
        let booster = VolumeBoosterEffect::new(6.0); // +6 dB => ~2.0x
        let input = AmplitudeInterval::FULL_SCALE;
        let output = booster.static_transfer(input);

        assert!((output.peak_magnitude() - 1.995).abs() < 0.01);
        assert!(output.is_clipping());
        assert!((output.peak_dbfs() - 6.0).abs() < 0.1);
    }

    #[test]
    fn test_dag_culprit_detection_and_victim_flow() {
        // Build Graph:
        // [1: Source] ---> [2: StereoWidener 2.0x] ---> [3: Wire Passthrough] ---> [4: DAC]
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "Widener".into(), effect: Box::new(StereoWidenerEffect::new(2.0)) },
            GraphNode { id: 3, name: "Buffer".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 4, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![
            GraphEdge::new(1, 2),
            GraphEdge::new(2, 3),
            GraphEdge::new(3, 4),
        ];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2, 3, 4],
            effects: Vec::new(),
        };

        let report = analyze_signal_chain(&graph);

        assert!(!report.is_chain_safe);
        assert_eq!(report.culprit_nodes, vec![2], "Widener must be the only culprit!");

        let r1 = report.get_node_report(1).unwrap();
        assert_eq!(r1.status, NodeSafetyStatus::Safe);
        assert_eq!(r1.severity, DiagnosticSeverity::Safe);

        let r2 = report.get_node_report(2).unwrap();
        assert_eq!(r2.status, NodeSafetyStatus::OriginOfClipping);
        assert!(r2.is_culprit);
        assert_eq!(r2.severity, DiagnosticSeverity::SevereHazard);
        assert!((r2.peak_dbfs - 6.0206).abs() < 0.01);
        assert!(!r2.remedies.is_empty(), "Culprit node must provide actionable remedies!");

        let r3 = report.get_node_report(3).unwrap();
        assert_eq!(r3.status, NodeSafetyStatus::OverdrivenDownstream, "Downstream node is victim!");
        assert!(!r3.is_culprit);
        assert_eq!(r3.severity, DiagnosticSeverity::SevereHazard);

        let r4 = report.get_node_report(4).unwrap();
        assert_eq!(r4.status, NodeSafetyStatus::OverdrivenDownstream);

        // Cable verification:
        // Edge 1 -> 2: Safe
        assert!(!report.is_edge_hot(1, 0, 2, 0));
        // Edge 2 -> 3: Hot!
        assert!(report.is_edge_hot(2, 0, 3, 0));
        // Edge 3 -> 4: Hot!
        assert!(report.is_edge_hot(3, 0, 4, 0));

        // Back-tracing inquiry from DAC (Node 4): Who is the culprit?
        let culprits_of_dac = report.get_culprit_path(4, &graph);
        assert_eq!(culprits_of_dac, vec![2], "Back-tracing from DAC must pinpoint Widener!");
    }

    #[test]
    fn test_dag_attenuator_cures_clipping() {
        // Build Graph:
        // [1: Source] ---> [2: Widener 2.0x (+6dB)] ---> [3: Attenuator -6.02dB] ---> [4: DAC]
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "Widener".into(), effect: Box::new(StereoWidenerEffect::new(2.0)) },
            GraphNode { id: 3, name: "Attenuator".into(), effect: Box::new(VolumeAttenuatorEffect::new(-6.0206)) },
            GraphNode { id: 4, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![
            GraphEdge::new(1, 2),
            GraphEdge::new(2, 3),
            GraphEdge::new(3, 4),
        ];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2, 3, 4],
            effects: Vec::new(),
        };

        let report = analyze_signal_chain(&graph);

        // Edge 2 -> 3 is hot
        assert!(report.is_edge_hot(2, 0, 3, 0));
        // Edge 3 -> 4 is cured and SAFE!
        assert!(!report.is_edge_hot(3, 0, 4, 0));

        let r4 = report.get_node_report(4).unwrap();
        assert_eq!(r4.status, NodeSafetyStatus::Safe);
        assert_eq!(r4.severity, DiagnosticSeverity::Safe);
        assert!(!report.terminal_interval.is_clipping());
    }

    #[test]
    fn test_dag_multi_summing_clipping() {
        // Build Graph:
        // [1: Source] --- Split to 2 wires ---> [2: DAC]
        // 2 wires going into DAC port 0!
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![
            GraphEdge::with_ports(1, 0, 2, 0),
            GraphEdge::with_ports(1, 1, 2, 0),
        ];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2],
            effects: Vec::new(),
        };

        let report = analyze_signal_chain(&graph);

        assert!(!report.is_chain_safe);
        assert_eq!(report.culprit_nodes, vec![2], "Summing port must be culprit!");

        let r2 = report.get_node_report(2).unwrap();
        assert_eq!(r2.status, NodeSafetyStatus::OriginOfClipping);
        // (1 + 1) / sqrt(2) = 1.4142 => +3.01 dBFS
        assert!((r2.peak_dbfs - 3.0103).abs() < 0.01);
        assert_eq!(r2.severity, DiagnosticSeverity::SevereHazard);
        assert!(!r2.remedies.is_empty());
    }

    #[test]
    fn test_contextual_static_analysis_safe_with_headroom() {
        // Track has peak = 0.6 (-4.43 dBFS).
        // Widener has width = 1.20x (+20%).
        // Max theoretical output = 0.6 * 1.20 = 0.72 (-2.85 dBFS <= 0 dBFS).
        // MUST BE SAFE!
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "Widener".into(), effect: Box::new(StereoWidenerEffect::new(1.20)) },
            GraphNode { id: 3, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![
            GraphEdge::new(1, 2),
            GraphEdge::new(2, 3),
        ];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2, 3],
            effects: Vec::new(),
        };

        let source_int = AmplitudeInterval::symmetric(0.6);
        let report = analyze_signal_chain_with_context(&graph, source_int, Some("Acoustic Ballad".into()));

        assert!(report.is_chain_safe, "Chain must be safe when song has sufficient headroom");
        assert_eq!(report.culprit_nodes.len(), 0);
        assert!(!report.is_edge_hot(2, 0, 3, 0), "Edge 2 -> 3 must NOT be hot");
        assert_eq!(report.context_track_title.as_deref(), Some("Acoustic Ballad"));
        assert!((report.max_peak_dbfs - (-2.8533)).abs() < 0.05);
    }

    #[test]
    fn test_contextual_static_analysis_clipping_when_headroom_exceeded() {
        // Track has peak = 0.9 (-0.91 dBFS).
        // Widener has width = 1.20x (+20%).
        // Max theoretical output = 0.9 * 1.20 = 1.08 (+0.67 dBFS > 0 dBFS).
        // MUST BE FLAGGED AS CLIPPING!
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "Widener".into(), effect: Box::new(StereoWidenerEffect::new(1.20)) },
            GraphNode { id: 3, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![
            GraphEdge::new(1, 2),
            GraphEdge::new(2, 3),
        ];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2, 3],
            effects: Vec::new(),
        };

        let source_int = AmplitudeInterval::symmetric(0.9);
        let report = analyze_signal_chain_with_context(&graph, source_int, Some("Loud Master".into()));

        assert!(!report.is_chain_safe, "Chain must be flagged unsafe when boost exceeds headroom");
        assert_eq!(report.culprit_nodes, vec![2], "Widener must be pinpointed as culprit");
        assert!(report.is_edge_hot(2, 0, 3, 0), "Edge 2 -> 3 must be hot");
        assert!(report.max_headroom_deficit_db > 0.6);
    }

    #[test]
    fn test_contextual_static_analysis_track_itself_clipping() {
        // Track is pre-clipped / inter-sample peak at 1.15 (+1.21 dBFS).
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![
            GraphEdge::new(1, 2),
        ];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2],
            effects: Vec::new(),
        };

        let source_int = AmplitudeInterval::symmetric(1.15);
        let report = analyze_signal_chain_with_context(&graph, source_int, Some("Overmastered CD".into()));

        assert!(!report.is_chain_safe);
        assert_eq!(report.culprit_nodes, vec![1], "Source node must be origin of clipping!");
        let r1 = report.get_node_report(1).unwrap();
        assert_eq!(r1.status, NodeSafetyStatus::OriginOfClipping);
        assert!(r1.reason.contains("File audio sumber"));
    }

    #[test]
    fn test_configurable_sensitivity_threshold_strict_standard_relaxed() {
        // Track with peak = 0.95 (-0.445 dBFS).
        // 1. In Standard mode (threshold = 0.0 dBFS): peak is <= 0 dBFS -> SAFE!
        // 2. In Strict mode (threshold = -1.0 dBFS): peak is > -1.0 dBFS -> FLAGGED AS CLIPPING!
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![GraphEdge::new(1, 2)];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2],
            effects: Vec::new(),
        };

        let source_int = AmplitudeInterval::symmetric(0.95);

        // Standard mode (0.0 dBFS)
        let rep_standard = analyze_signal_chain_advanced(&graph, source_int, Some("Hot Master".into()), 0.0, true);
        assert!(rep_standard.is_chain_safe, "Peak -0.45 dBFS must be safe under 0.0 dBFS threshold");
        assert_eq!(rep_standard.culprit_nodes.len(), 0);

        // Strict mode (-1.0 dBFS)
        let rep_strict = analyze_signal_chain_advanced(&graph, source_int, Some("Hot Master".into()), -1.0, true);
        assert!(!rep_strict.is_chain_safe, "Peak -0.45 dBFS must be flagged as hazard under strict -1.0 dBFS threshold");
        assert_eq!(rep_strict.culprit_nodes, vec![1]);
        assert!(rep_strict.max_headroom_deficit_db > 0.5);

        // Relaxed mode with intentional mild saturation (+1.0 dBFS peak, threshold = +2.0 dBFS)
        let nodes_sat = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "Booster".into(), effect: Box::new(VolumeBoosterEffect::new(1.0)) }, // +1.0 dB boost on 1.0 = +1.0 dBFS
            GraphNode { id: 3, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges_sat = vec![GraphEdge::new(1, 2), GraphEdge::new(2, 3)];
        let graph_sat = SignalChainGraph {
            is_complete: true,
            nodes: nodes_sat,
            edges: edges_sat,
            topological_order: vec![1, 2, 3],
            effects: Vec::new(),
        };

        // Standard mode: +1.0 dBFS > 0.0 dBFS -> flags hazard
        let rep_sat_std = analyze_signal_chain_advanced(&graph_sat, AmplitudeInterval::FULL_SCALE, None, 0.0, true);
        assert!(!rep_sat_std.is_chain_safe);
        assert_eq!(rep_sat_std.culprit_nodes, vec![2]);

        // Relaxed mode (+2.0 dBFS): +1.0 dBFS <= +2.0 dBFS -> SAFE!
        let rep_sat_relaxed = analyze_signal_chain_advanced(&graph_sat, AmplitudeInterval::FULL_SCALE, None, 2.0, true);
        assert!(rep_sat_relaxed.is_chain_safe, "Mild +1.0 dB saturation must be tolerated in Relaxed +2.0 dBFS mode");
        assert_eq!(rep_sat_relaxed.culprit_nodes.len(), 0);
        assert!(!rep_sat_relaxed.is_edge_hot(2, 0, 3, 0));
    }

    #[test]
    fn test_clipping_detection_disabled_suppresses_all_warnings() {
        // Even with extreme +12 dB boost, when detection_enabled = false:
        // Chain must report is_chain_safe = true, culprit_nodes = empty, is_hot = false
        let nodes = vec![
            GraphNode { id: 1, name: "Source".into(), effect: Box::new(PassthroughEffect) },
            GraphNode { id: 2, name: "Booster".into(), effect: Box::new(VolumeBoosterEffect::new(12.0)) },
            GraphNode { id: 3, name: "DAC".into(), effect: Box::new(PassthroughEffect) },
        ];
        let edges = vec![GraphEdge::new(1, 2), GraphEdge::new(2, 3)];
        let graph = SignalChainGraph {
            is_complete: true,
            nodes,
            edges,
            topological_order: vec![1, 2, 3],
            effects: Vec::new(),
        };

        let report = analyze_signal_chain_advanced(&graph, AmplitudeInterval::FULL_SCALE, None, 0.0, false);
        assert!(report.is_chain_safe, "Detection disabled must never alarm the user");
        assert_eq!(report.culprit_nodes.len(), 0);
        assert!(!report.is_edge_hot(2, 0, 3, 0));
        assert_eq!(report.max_headroom_deficit_db, 0.0);
        assert_eq!(report.detection_enabled, false);

        let n2 = report.get_node_report(2).unwrap();
        assert_eq!(n2.status, NodeSafetyStatus::Safe);
        assert_eq!(n2.severity, DiagnosticSeverity::Safe);
        assert!(n2.reason.contains("dinonaktifkan"));
    }
}
