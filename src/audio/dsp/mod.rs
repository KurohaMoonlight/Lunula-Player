pub mod analyzer;
pub mod biquad;
pub mod effects;
pub mod executor;
pub mod pipeline;

pub use analyzer::{
    analyze_signal_chain, analyze_signal_chain_with_context, analyze_signal_chain_advanced, ActionableRemedy, DiagnosticSeverity, EdgeAnalysisReport, GraphAnalysisReport,
    NodeAnalysisReport, NodeSafetyStatus, RemedyAction,
};
pub use biquad::{soft_limit, BiquadCoeffs, BiquadState};
pub use effects::{
    AnalogPreampEffect, BandPassFilterEffect, BassGuardEffect, CompressorEffect, CrossoverHighEffect, CrossoverLowEffect, DynamicLoudnessEffect, GraphicEqEffect, HighPassFilterEffect, LowPassFilterEffect, MixerEffect, OpticalLevelerEffect,
    ParametricBandConfig, ParametricEqEffect, PassthroughEffect, PresetEqEffect, SplitterEffect, SteppedAttenuatorEffect, StereoBalanceEffect, StereoWidenerEffect, TubeWarmerEffect,
    VolumeAttenuatorEffect, VolumeBoosterEffect, EQ_FREQUENCIES,
};
pub use crate::audio::types::EffectType;
pub use executor::{ExecutionStep, GraphExecutor, StepInput};
pub use pipeline::DspPipeline;

#[cfg(test)]
mod tests;

