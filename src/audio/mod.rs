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

pub use decoder::AudioDecoder;
pub use dsp::{
    AnalogPreampEffect, BandPassFilterEffect, BassGuardEffect, CompressorEffect, CrossoverHighEffect, CrossoverLowEffect, DspPipeline, DynamicLoudnessEffect, GraphicEqEffect, HighPassFilterEffect, LowPassFilterEffect, MixerEffect, OpticalLevelerEffect,
    ParametricBandConfig, ParametricEqEffect, PassthroughEffect, PresetEqEffect, SplitterEffect, SteppedAttenuatorEffect, StereoBalanceEffect, StereoWidenerEffect, TubeWarmerEffect,
    VolumeAttenuatorEffect, VolumeBoosterEffect, EQ_FREQUENCIES,
};
pub use engine::AudioEngineController;
pub use output::AudioOutput;
pub use resampler::StereoResampler;
pub use types::{
    AudioCommand, AudioEvent, ChainMode, Effect, EffectType, GraphEdge, GraphNode, PlaybackState, ResamplerSettings, RollOffProfile,
    SignalChainGraph, SignalPathReport, SincQuality,
};
pub use visualizer::{FftAnalyzer, VisualizerRingBuffer, FFT_SIZE, NUM_BANDS};
