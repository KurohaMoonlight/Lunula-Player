pub mod autoeq;
pub mod backend;
pub mod decoder;
pub mod dsp;
pub mod engine;
pub mod output;
pub mod resampler;
pub mod types;
pub mod visualizer;

pub use autoeq::parse_autoeq_text;

pub use decoder::{AudioDecoder, scan_audio_peak_fast, scan_audio_peak_and_waveform};
pub use dsp::{
    analyze_signal_chain, analyze_signal_chain_with_context, analyze_signal_chain_advanced, ActionableRemedy, AnalogPreampEffect, BandPassFilterEffect, BassGuardEffect, CompressorEffect, CrossoverHighEffect, CrossoverLowEffect, DiagnosticSeverity, DspPipeline, DynamicLoudnessEffect, EdgeAnalysisReport, GraphicEqEffect, GraphAnalysisReport, HighPassFilterEffect, LowPassFilterEffect, MixerEffect, NodeAnalysisReport, NodeSafetyStatus, OpticalLevelerEffect,
    ParametricBandConfig, ParametricEqEffect, PassthroughEffect, PresetEqEffect, RemedyAction, SplitterEffect, SteppedAttenuatorEffect, StereoBalanceEffect, StereoMatrixEffect, StereoWidenerEffect, TimeAlignmentEffect, TubeWarmerEffect,
    VolumeAttenuatorEffect, VolumeBoosterEffect, EQ_FREQUENCIES,
};
pub use engine::AudioEngineController;
pub use output::AudioOutput;
pub use resampler::StereoResampler;
pub use types::{
    AmplitudeInterval, AudioCommand, AudioEvent, ChainMode, ChannelContract, EdgeIr, Effect, EffectType,
    GraphCompilationError, GraphEdge, GraphIr, GraphNode, NodeIr, PlaybackState, ResamplerSettings, RollOffProfile,
    SignalChainGraph, SignalPathReport, SincQuality,
};
pub use visualizer::{FftAnalyzer, VisualizerRingBuffer, FFT_SIZE, NUM_BANDS};
