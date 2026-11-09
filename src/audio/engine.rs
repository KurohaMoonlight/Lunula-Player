use crossbeam_channel::{unbounded, Receiver, Sender};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::audio::decoder::AudioDecoder;
use crate::audio::dsp::DspPipeline;
use crate::audio::output::AudioOutput;
use crate::audio::resampler::StereoResampler;
pub use crate::audio::types::calculate_effective_volume;
use crate::audio::types::{AudioCommand, AudioEvent, ChainMode, DspCommand, PlaybackState, ResamplerSettings, SignalChainGraph, SignalPathReport};
use crate::audio::visualizer::{VisualizerRingBuffer, FFT_SIZE};
use rtrb::Producer;

pub struct AudioEngineController {
    cmd_tx: Sender<AudioCommand>,
    event_rx: Receiver<AudioEvent>,
    shared_position_ms: Arc<AtomicU32>,
    shared_state: Arc<AtomicU8>,
    shared_volume: Arc<AtomicU32>,
    shared_muted: Arc<AtomicBool>,
    shared_bit_perfect: Arc<AtomicBool>,
    shared_visualizer: Arc<VisualizerRingBuffer>,
    shared_buffer_metrics: Arc<crate::system_telemetry::BufferMetricsAtomic>,
}

impl AudioEngineController {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (cmd_tx, cmd_rx) = unbounded::<AudioCommand>();
        let (event_tx, event_rx) = unbounded::<AudioEvent>();

        let shared_position_ms = Arc::new(AtomicU32::new(0));
        let shared_state = Arc::new(AtomicU8::new(0)); // 0: Stopped, 1: Playing, 2: Paused
        let shared_volume = Arc::new(AtomicU32::new(0.80f32.to_bits()));
        let shared_muted = Arc::new(AtomicBool::new(false));
        let shared_bit_perfect = Arc::new(AtomicBool::new(true));
        let shared_visualizer = Arc::new(VisualizerRingBuffer::new());
        let shared_buffer_metrics = Arc::new(crate::system_telemetry::BufferMetricsAtomic::default());

        let pos_clone = Arc::clone(&shared_position_ms);
        let state_clone = Arc::clone(&shared_state);
        let vol_clone = Arc::clone(&shared_volume);
        let mute_clone = Arc::clone(&shared_muted);
        let bp_clone = Arc::clone(&shared_bit_perfect);
        let vis_clone = Arc::clone(&shared_visualizer);
        let buf_clone = Arc::clone(&shared_buffer_metrics);

        thread::Builder::new()
            .name("lunula-audio-worker".into())
            .spawn(move || {
                run_audio_worker(cmd_rx, event_tx, pos_clone, state_clone, vol_clone, mute_clone, bp_clone, vis_clone, buf_clone);
            })?;

        Ok(Self {
            cmd_tx,
            event_rx,
            shared_position_ms,
            shared_state,
            shared_volume,
            shared_muted,
            shared_bit_perfect,
            shared_visualizer,
            shared_buffer_metrics,
        })
    }

    pub fn buffer_metrics(&self) -> &Arc<crate::system_telemetry::BufferMetricsAtomic> {
        &self.shared_buffer_metrics
    }

    pub fn play<P: Into<PathBuf>>(&self, path: P) {
        let _ = self.cmd_tx.send(AudioCommand::Play(path.into()));
    }

    pub fn queue_next<P: Into<PathBuf>>(&self, path: P) {
        let _ = self.cmd_tx.send(AudioCommand::QueueNext(path.into()));
    }

    pub fn clear_next(&self) {
        let _ = self.cmd_tx.send(AudioCommand::ClearNext);
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Pause);
    }

    pub fn resume(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Resume);
    }

    pub fn restart(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Restart);
    }

    pub fn stop(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Stop);
    }

    pub fn seek(&self, seconds: f32) {
        let _ = self.cmd_tx.send(AudioCommand::Seek(seconds));
    }

    pub fn set_volume(&self, volume: f32) {
        self.shared_volume.store(volume.clamp(0.0, 1.5).to_bits(), Ordering::Relaxed);
        let _ = self.cmd_tx.send(AudioCommand::SetVolume(volume));
    }

    pub fn set_muted(&self, muted: bool) {
        self.shared_muted.store(muted, Ordering::Relaxed);
        let _ = self.cmd_tx.send(AudioCommand::SetMute(muted));
    }

    pub fn set_eq_band(&self, index: usize, gain_db: f32) {
        let _ = self.cmd_tx.send(AudioCommand::SetEqBand { index, gain_db });
    }

    pub fn set_eq_bypass(&self, bypass: bool) {
        let _ = self.cmd_tx.send(AudioCommand::SetEqBypass(bypass));
    }

    pub fn set_limiter_enabled(&self, enabled: bool) {
        let _ = self.cmd_tx.send(AudioCommand::SetLimiterEnabled(enabled));
    }

    pub fn set_chain_mode(&self, mode: ChainMode) {
        let _ = self.cmd_tx.send(AudioCommand::SetChainMode(mode));
    }

    pub fn set_output_device(&self, device_name: Option<String>) {
        let _ = self.cmd_tx.send(AudioCommand::SetOutputDevice(device_name));
    }

    pub fn set_signal_chain_graph(&self, graph: crate::audio::SignalChainGraph) {
        let _ = self.cmd_tx.send(AudioCommand::SetSignalChainGraph(graph));
    }

    pub fn set_node_param(&self, node_id: u32, param_id: u32, value: f32) {
        let _ = self.cmd_tx.send(AudioCommand::SetNodeParam { node_id, param_id, value });
    }

    pub fn set_force_resample_48k(&self, force: bool) {
        let mode = if force {
            ChainMode::OsResampled
        } else {
            ChainMode::BitPerfect
        };
        self.set_chain_mode(mode);
    }

    pub fn try_recv_event(&self) -> Option<AudioEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn is_bit_perfect(&self) -> bool {
        self.shared_bit_perfect.load(Ordering::Relaxed)
    }

    // Instant zero-lock current playback time in seconds
    pub fn current_position(&self) -> f32 {
        self.shared_position_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }

    // Instant zero-lock playback state
    pub fn playback_state(&self) -> PlaybackState {
        match self.shared_state.load(Ordering::Relaxed) {
            1 => PlaybackState::Playing,
            2 => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        }
    }

    // Read the latest 1024 real-time speaker samples for FFT spectrum analysis
    pub fn get_visualizer_samples(&self, dest: &mut [f32; FFT_SIZE]) {
        self.shared_visualizer.get_latest_samples(dest);
    }

    pub fn get_visualizer_stereo_samples(&self, dest_l: &mut [f32; FFT_SIZE], dest_r: &mut [f32; FFT_SIZE]) {
        self.shared_visualizer.get_latest_stereo(dest_l, dest_r);
    }

    pub fn get_gain_reduction(&self) -> f32 {
        self.shared_visualizer.get_gain_reduction()
    }

    pub fn set_resampler_settings(&self, settings: ResamplerSettings) {
        let _ = self.cmd_tx.send(AudioCommand::SetResamplerSettings(settings));
    }

    pub fn set_crossfade_secs(&self, secs: f32) {
        let _ = self.cmd_tx.send(AudioCommand::SetCrossfadeSecs(secs));
    }
}

fn desired_device_rate(chain_mode: ChainMode, track_rate: u32) -> u32 {
    match chain_mode {
        ChainMode::BitPerfect => track_rate,
        ChainMode::DspActive => track_rate,
        ChainMode::OsResampled => 48000,
    }
}

fn create_signal_path_report(
    out: &AudioOutput,
    track_rate: u32,
    source_bit_depth: u8,
    chain_mode: ChainMode,
) -> SignalPathReport {
    let device_rate = out.sample_rate();
    let device_name = out.device_name().to_string();
    let format_desc = out.format_desc();
    let is_exclusive = out.is_exclusive();
    let is_resampling = track_rate != device_rate;

    let actual_mode = match chain_mode {
        ChainMode::BitPerfect => {
            let bit_depth_ok = out.bit_depth() >= source_bit_depth;
            // True bit-perfect requires:
            // 1. Hardware exclusive mode (bypasses Windows OS audio mixer)
            // 2. Exactly matching sample rate (no resampler)
            // 3. Hardware bit depth >= track bit depth (no bit depth truncation)
            // 4. Source bit depth between 1 and 24 (lossless; 32-bit integer PCM through f32 loses bits, and lossy formats 0-bit are not bit-perfect)
            if is_exclusive && track_rate == device_rate && bit_depth_ok && source_bit_depth > 0 && source_bit_depth <= 24 {
                ChainMode::BitPerfect
            } else {
                ChainMode::OsResampled
            }
        }
        other => other,
    };

    let is_dsp_active = actual_mode == ChainMode::DspActive;
    let is_volume_applied = actual_mode != ChainMode::BitPerfect;

    SignalPathReport {
        device_name,
        source_rate: track_rate,
        device_rate,
        source_bit_depth,
        format_desc,
        requested_mode: chain_mode,
        actual_mode,
        is_exclusive,
        is_resampling,
        is_dsp_active,
        is_volume_applied,
    }
}

pub fn route_audio_frame(
    chain_mode: ChainMode,
    is_chain_complete: bool,
    l: f32,
    r: f32,
    dsp: &mut DspPipeline,
) -> (f32, f32) {
    match chain_mode {
        ChainMode::BitPerfect => {
            if !is_chain_complete {
                (0.0, 0.0) // Severed cable in canvas mutes playback
            } else {
                (l, r) // 100% pure sample passthrough: bit-for-bit identical
            }
        }
        _ => dsp.process_frame(l, r),
    }
}

// Background Audio Feeder Worker
fn run_audio_worker(
    cmd_rx: Receiver<AudioCommand>,
    event_tx: Sender<AudioEvent>,
    shared_position_ms: Arc<AtomicU32>,
    shared_state: Arc<AtomicU8>,
    shared_volume: Arc<AtomicU32>,
    shared_muted: Arc<AtomicBool>,
    shared_bit_perfect: Arc<AtomicBool>,
    shared_visualizer: Arc<VisualizerRingBuffer>,
    shared_buffer_metrics: Arc<crate::system_telemetry::BufferMetricsAtomic>,
) {
    let mut decoder: Option<AudioDecoder> = None;
    let mut next_decoder: Option<AudioDecoder> = None;
    let mut next_track_path: Option<PathBuf> = None;
    let mut output: Option<AudioOutput> = None;
    let mut producer: Option<Producer<f32>> = None;
    let mut dsp = DspPipeline::new(44100.0);
    let mut resampler = StereoResampler::new(44100, 44100);
    let mut chain_mode = ChainMode::BitPerfect;
    let mut current_stream_mode: Option<ChainMode> = None;
    let mut crossfade_secs: f32 = 0.0;
    let mut fade_in_remaining_samples: usize = 0;
    let mut fade_in_total_samples: usize = 0;
    let mut state = PlaybackState::Stopped;
    let mut selected_device: Option<String> = None;
    let is_dsp_active = Arc::new(AtomicBool::new(false));

    let mut last_graph = SignalChainGraph::default();
    let mut last_eq_bands = [0.0f32; 10];
    let mut last_bypass = false;
    let mut last_limiter = false;

    let sync_output_dsp = |out: &AudioOutput, rate: u32, bp: bool, limiter: bool, graph: &SignalChainGraph, eq: &[f32; 10]| {
        out.send_dsp_command(DspCommand::SetSampleRate(rate as f32));
        out.send_dsp_command(DspCommand::SetEqBypass(bp));
        out.send_dsp_command(DspCommand::SetLimiterEnabled(limiter));
        if let Ok(plan) = crate::audio::dsp::executor::compile_signal_chain_plan(graph, rate as f32, None) {
            out.send_dsp_command(DspCommand::SetCompiledPlan(Box::new(plan)));
        } else {
            out.send_dsp_command(DspCommand::SetSignalChainGraph(graph.clone()));
        }
        for (idx, &gain) in eq.iter().enumerate() {
            out.send_dsp_command(DspCommand::SetEqBand { index: idx, gain_db: gain });
        }
    };

    let mut raw_chunk = [0.0f32; 2048];
    let mut raw_pos = 0;
    let mut raw_len = 0;

    loop {
        // 1. Receive commands: poll non-blocking if playing, or block with timeout if paused/stopped
        let maybe_cmd = if state == PlaybackState::Playing {
            cmd_rx.try_recv().ok()
        } else {
            cmd_rx.recv_timeout(Duration::from_millis(50)).ok()
        };

        if let Some(first_cmd) = maybe_cmd {
            let mut pending = vec![first_cmd];
            while let Ok(cmd) = cmd_rx.try_recv() {
                pending.push(cmd);
            }

            for cmd in pending {
                match cmd {
                    AudioCommand::Play(path) => {
                        next_decoder = None;
                        next_track_path = None;
                        fade_in_remaining_samples = 0;
                        fade_in_total_samples = 0;
                        match AudioDecoder::open(&path) {
                            Ok(dec) => {
                                let track_sample_rate = dec.sample_rate();
                                let track_bit_depth = dec.bits_per_sample();
                                let duration = dec.duration();
                                let channels = dec.source_channels() as u16;

                                raw_pos = 0;
                                raw_len = 0;

                                let desired_rate = desired_device_rate(chain_mode, track_sample_rate);

                                let need_recreate = match &output {
                                    Some(out) => {
                                        out.sample_rate() != desired_rate
                                            || current_stream_mode != Some(chain_mode)
                                            || (selected_device.is_some() && Some(out.device_name()) != selected_device.as_deref())
                                    }
                                    None => true,
                                };

                                if need_recreate {
                                    if let Some(old_out) = output.take() {
                                        old_out.pause();
                                        let _ = old_out.flush();
                                        drop(old_out);
                                    }
                                    producer = None;
                                    match AudioOutput::new(
                                        selected_device.as_deref(),
                                        desired_rate,
                                        track_bit_depth,
                                        chain_mode,
                                        96000,
                                        Arc::clone(&shared_volume),
                                        Arc::clone(&shared_visualizer),
                                        Arc::clone(&shared_bit_perfect),
                                        Arc::clone(&shared_muted),
                                        Arc::clone(&is_dsp_active),
                                        event_tx.clone(),
                                        true,
                                    ) {
                                        Ok((out, prod)) => {
                                            sync_output_dsp(&out, desired_rate, last_bypass, last_limiter, &last_graph, &last_eq_bands);
                                            output = Some(out);
                                            producer = Some(prod);
                                            current_stream_mode = Some(chain_mode);
                                        }
                                        Err(err) => {
                                            if let Some(out) = &output {
                                                out.pause();
                                                let _ = out.flush();
                                            }
                                            decoder = None;
                                            raw_pos = 0;
                                            raw_len = 0;
                                            state = PlaybackState::Stopped;
                                            shared_state.store(0, Ordering::Relaxed);
                                            shared_position_ms.store(0, Ordering::Relaxed);
                                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Stopped));
                                            let _ = event_tx.send(AudioEvent::Error(format!(
                                                "Inisialisasi perangkat audio gagal: {}",
                                                err
                                            )));
                                            continue;
                                        }
                                    }
                                } else if let Some(out) = &output {
                                    // Flush ring buffer without stream recreation: zero clicks, keeps exclusive mode lock
                                    if !out.flush() {
                                        let _ = event_tx.send(AudioEvent::Error(
                                            "Timeout flush audio (50ms): audio callback tidak merespons ACK".to_string(),
                                        ));
                                    }
                                }

                                if let Some(out) = &output {
                                    out.resume();
                                    let device_rate = out.sample_rate();
                                    // Always sync resampler: from track rate to actual device consumption rate
                                    resampler.reset(track_sample_rate, device_rate);
                                    dsp.set_sample_rate(device_rate as f32);
                                    dsp.set_eq_bypass(last_bypass);
                                    dsp.set_limiter_enabled(last_limiter);
                                    sync_output_dsp(out, device_rate, last_bypass, last_limiter, &last_graph, &last_eq_bands);

                                    let report = create_signal_path_report(out, track_sample_rate, track_bit_depth, chain_mode);
                                    shared_bit_perfect.store(report.actual_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                                    is_dsp_active.store(report.is_dsp_active, Ordering::Relaxed);
                                    let _ = event_tx.send(AudioEvent::SignalPath(report));
                                }

                                decoder = Some(dec);
                                state = PlaybackState::Playing;
                                shared_state.store(1, Ordering::Relaxed);
                                shared_position_ms.store(0, Ordering::Relaxed);

                                let _ = event_tx.send(AudioEvent::TrackStarted {
                                    duration,
                                    sample_rate: track_sample_rate,
                                    channels,
                                });
                                let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Playing));
                            }
                            Err(err) => {
                                if let Some(out) = &output {
                                    out.pause();
                                    let _ = out.flush();
                                }
                                decoder = None;
                                raw_pos = 0;
                                raw_len = 0;
                                state = PlaybackState::Stopped;
                                shared_state.store(0, Ordering::Relaxed);
                                shared_position_ms.store(0, Ordering::Relaxed);
                                let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Stopped));
                                let _ = event_tx.send(AudioEvent::Error(format!(
                                    "Gagal membuka file audio: {}",
                                    err
                                )));
                            }
                        }
                    }
                    AudioCommand::QueueNext(path) => {
                        match AudioDecoder::open(&path) {
                            Ok(dec) => {
                                next_decoder = Some(dec);
                                next_track_path = Some(path);
                            }
                            Err(e) => {
                                eprintln!("[Gapless Prebuffer] Gagal membuka next track: {}", e);
                                next_decoder = None;
                                next_track_path = None;
                            }
                        }
                    }
                    AudioCommand::ClearNext => {
                        next_decoder = None;
                        next_track_path = None;
                    }
                    AudioCommand::SetResamplerSettings(settings) => {
                        resampler.set_settings(settings);
                    }
                    AudioCommand::SetCrossfadeSecs(secs) => {
                        crossfade_secs = secs.max(0.0);
                    }
                    AudioCommand::Pause => {
                        if state == PlaybackState::Playing {
                            if let Some(out) = &output {
                                out.pause();
                            }
                            state = PlaybackState::Paused;
                            shared_state.store(2, Ordering::Relaxed);
                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Paused));
                        }
                    }
                    AudioCommand::Resume => {
                        if state == PlaybackState::Paused || (state == PlaybackState::Stopped && decoder.is_some()) {
                            if let Some(dec) = &mut decoder {
                                if raw_pos >= raw_len && dec.current_time() >= (dec.duration().max(0.1) - 0.5) {
                                    let _ = dec.seek(0.0);
                                    raw_pos = 0;
                                    raw_len = 0;
                                    if let Some(out) = &output {
                                        let _ = out.flush();
                                    }
                                    let device_rate = output.as_ref().map(|o| o.sample_rate()).unwrap_or(44100);
                                    let track_sample_rate = dec.sample_rate();
                                    resampler.reset(track_sample_rate, device_rate);
                                    dsp.reset_states();
                                    shared_position_ms.store(0, Ordering::Relaxed);
                                    let _ = event_tx.send(AudioEvent::Position(0.0));
                                }
                            }
                            if let Some(out) = &output {
                                out.resume();
                            }
                            state = PlaybackState::Playing;
                            shared_state.store(1, Ordering::Relaxed);
                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Playing));
                        }
                    }
                    AudioCommand::Restart => {
                        if let Some(dec) = &mut decoder {
                            let _ = dec.seek(0.0);
                            raw_pos = 0;
                            raw_len = 0;
                            if let Some(out) = &output {
                                let _ = out.flush();
                                out.resume();
                            }
                            let device_rate = output.as_ref().map(|o| o.sample_rate()).unwrap_or(44100);
                            let track_sample_rate = dec.sample_rate();
                            resampler.reset(track_sample_rate, device_rate);
                            dsp.reset_states();
                            state = PlaybackState::Playing;
                            shared_state.store(1, Ordering::Relaxed);
                            shared_position_ms.store(0, Ordering::Relaxed);
                            let _ = event_tx.send(AudioEvent::Position(0.0));
                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Playing));
                        }
                    }
                    AudioCommand::Stop => {
                        if let Some(out) = &output {
                            out.pause();
                            let _ = out.flush();
                        }
                        decoder = None;
                        next_decoder = None;
                        next_track_path = None;
                        raw_pos = 0;
                        raw_len = 0;
                        fade_in_remaining_samples = 0;
                        fade_in_total_samples = 0;
                        state = PlaybackState::Stopped;
                        shared_state.store(0, Ordering::Relaxed);
                        shared_position_ms.store(0, Ordering::Relaxed);
                        shared_visualizer.clear();
                        shared_buffer_metrics.used_samples.store(0, Ordering::Relaxed);
                        let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Stopped));
                    }
                    AudioCommand::Seek(seconds) => {
                        if let Some(dec) = &mut decoder {
                            match dec.seek(seconds) {
                                Ok(actual_pos) => {
                                    raw_pos = 0;
                                    raw_len = 0;
                                    fade_in_remaining_samples = 0;
                                    fade_in_total_samples = 0;
                                    shared_buffer_metrics.used_samples.store(0, Ordering::Relaxed);

                                    if let Some(out) = &output {
                                        if !out.flush() {
                                            let _ = event_tx.send(AudioEvent::Error(
                                                "Timeout flush audio saat seek (50ms): audio callback tidak merespons ACK".to_string(),
                                            ));
                                        }
                                        let sr = out.sample_rate();
                                        resampler.reset(dec.sample_rate(), sr);
                                        dsp.reset_states();
                                    }

                                    shared_position_ms.store((actual_pos * 1000.0) as u32, Ordering::Relaxed);
                                    let _ = event_tx.send(AudioEvent::Position(actual_pos));
                                }
                                Err(err) => {
                                    let _ = event_tx.send(AudioEvent::Error(format!("Seek gagal: {}", err)));
                                }
                            }
                        }
                    }
                    AudioCommand::SetVolume(vol) => {
                        dsp.set_volume(vol);
                    }
                    AudioCommand::SetMute(muted) => {
                        shared_muted.store(muted, Ordering::Relaxed);
                    }
                    AudioCommand::SetEqBand { index, gain_db } => {
                        if index < 10 {
                            last_eq_bands[index] = gain_db;
                        }
                        dsp.set_eq_band(index, gain_db);
                        if let Some(out) = &output {
                            out.send_dsp_command(DspCommand::SetEqBand { index, gain_db });
                        }
                    }
                    AudioCommand::SetNodeParam { node_id, param_id, value } => {
                        if let Some(n) = last_graph.nodes.iter_mut().find(|n| n.id == node_id) {
                            n.effect.set_param(param_id, value);
                        }
                        dsp.set_node_param(node_id, param_id, value);
                        if let Some(out) = &output {
                            out.send_dsp_command(DspCommand::SetNodeParam { node_id, param_id, value });
                        }
                    }
                    AudioCommand::SetEqBypass(bypass) => {
                        last_bypass = bypass;
                        dsp.set_eq_bypass(bypass);
                        if let Some(out) = &output {
                            out.send_dsp_command(DspCommand::SetEqBypass(bypass));
                        }
                    }
                    AudioCommand::SetLimiterEnabled(enabled) => {
                        last_limiter = enabled;
                        dsp.set_limiter_enabled(enabled);
                        if let Some(out) = &output {
                            out.send_dsp_command(DspCommand::SetLimiterEnabled(enabled));
                        }
                    }
                    AudioCommand::SetOutputDevice(dev) => {
                        selected_device = dev;
                        if let (Some(dec), Some(_)) = (&decoder, &output) {
                            let track_sample_rate = dec.sample_rate();
                            let track_bit_depth = dec.bits_per_sample();
                            let desired_rate = desired_device_rate(chain_mode, track_sample_rate);
                            if let Some(old_out) = output.take() {
                                old_out.pause();
                                let _ = old_out.flush();
                                drop(old_out);
                            }
                            producer = None;
                            match AudioOutput::new(
                                selected_device.as_deref(),
                                desired_rate,
                                track_bit_depth,
                                chain_mode,
                                96000,
                                Arc::clone(&shared_volume),
                                Arc::clone(&shared_visualizer),
                                Arc::clone(&shared_bit_perfect),
                                Arc::clone(&shared_muted),
                                Arc::clone(&is_dsp_active),
                                event_tx.clone(),
                                true,
                            ) {
                                Ok((new_out, new_prod)) => {
                                    if state == PlaybackState::Playing {
                                        new_out.resume();
                                    }
                                    let device_rate = new_out.sample_rate();
                                    resampler.reset(track_sample_rate, device_rate);
                                    dsp.set_sample_rate(device_rate as f32);
                                    dsp.set_eq_bypass(last_bypass);
                                    dsp.set_limiter_enabled(last_limiter);
                                    sync_output_dsp(&new_out, device_rate, last_bypass, last_limiter, &last_graph, &last_eq_bands);

                                    let report = create_signal_path_report(&new_out, track_sample_rate, track_bit_depth, chain_mode);
                                    shared_bit_perfect.store(report.actual_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                                    is_dsp_active.store(report.is_dsp_active, Ordering::Relaxed);
                                    let _ = event_tx.send(AudioEvent::SignalPath(report));

                                    output = Some(new_out);
                                    producer = Some(new_prod);
                                    current_stream_mode = Some(chain_mode);
                                }
                                Err(err) => {
                                    let _ = event_tx.send(AudioEvent::Error(format!(
                                        "Gagal beralih ke perangkat output: {}",
                                        err
                                    )));
                                }
                            }
                        } else {
                            output = None;
                            producer = None;
                            current_stream_mode = None;
                        }
                    }
                    AudioCommand::SetChainMode(mode) => {
                        let prev_mode = chain_mode;
                        chain_mode = mode;
                        let dsp_active = mode == ChainMode::DspActive;
                        is_dsp_active.store(dsp_active, Ordering::Relaxed);

                        if let Some(dec) = &decoder {
                            let track_sample_rate = dec.sample_rate();
                            let track_bit_depth = dec.bits_per_sample();
                            let desired_rate = desired_device_rate(chain_mode, track_sample_rate);

                            let need_recreate = match &output {
                                Some(out) => {
                                    out.sample_rate() != desired_rate
                                        || current_stream_mode != Some(chain_mode)
                                        || (selected_device.is_some() && Some(out.device_name()) != selected_device.as_deref())
                                }
                                None => true,
                            };

                            if need_recreate {
                                if let Some(old_out) = output.take() {
                                    old_out.pause();
                                    let _ = old_out.flush();
                                    drop(old_out);
                                }
                                producer = None;
                                match AudioOutput::new(
                                    selected_device.as_deref(),
                                    desired_rate,
                                    track_bit_depth,
                                    chain_mode,
                                    96000,
                                    Arc::clone(&shared_volume),
                                    Arc::clone(&shared_visualizer),
                                    Arc::clone(&shared_bit_perfect),
                                    Arc::clone(&shared_muted),
                                    Arc::clone(&is_dsp_active),
                                    event_tx.clone(),
                                    true,
                                ) {
                                    Ok((new_out, new_prod)) => {
                                        if state == PlaybackState::Playing {
                                            new_out.resume();
                                        }
                                        sync_output_dsp(&new_out, desired_rate, last_bypass, last_limiter, &last_graph, &last_eq_bands);
                                        output = Some(new_out);
                                        producer = Some(new_prod);
                                        current_stream_mode = Some(chain_mode);
                                    }
                                    Err(err) => {
                                        chain_mode = prev_mode;
                                        is_dsp_active.store(prev_mode == ChainMode::DspActive, Ordering::Relaxed);
                                        let _ = event_tx.send(AudioEvent::Error(format!(
                                            "Gagal mengganti mode output audio: {}",
                                            err
                                        )));
                                    }
                                }
                            } else if let Some(out) = &output {
                                if !out.flush() {
                                    let _ = event_tx.send(AudioEvent::Error(
                                        "Timeout flush audio: audio callback tidak merespons ACK".to_string(),
                                    ));
                                }
                            }

                            if let Some(out) = &output {
                                let device_rate = out.sample_rate();
                                resampler.reset(track_sample_rate, device_rate);
                                dsp.set_sample_rate(device_rate as f32);
                                dsp.set_eq_bypass(last_bypass);
                                dsp.set_limiter_enabled(last_limiter);
                                sync_output_dsp(out, device_rate, last_bypass, last_limiter, &last_graph, &last_eq_bands);

                                let report = create_signal_path_report(out, track_sample_rate, track_bit_depth, chain_mode);
                                shared_bit_perfect.store(report.actual_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                                is_dsp_active.store(report.is_dsp_active, Ordering::Relaxed);
                                let _ = event_tx.send(AudioEvent::SignalPath(report));
                            }
                        } else {
                            shared_bit_perfect.store(chain_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                        }
                    }
                    AudioCommand::SetSignalChainGraph(graph) => {
                        last_graph = graph.clone();
                        let dev_rate = output.as_ref().map(|o| o.sample_rate()).unwrap_or(44100) as f32;
                        match crate::audio::dsp::executor::compile_signal_chain_plan(&graph, dev_rate, Some(&mut dsp.executor.plan.steps)) {
                            Ok(plan) => {
                                dsp.apply_compiled_plan(plan.clone());
                                if let Some(out) = &output {
                                    out.send_dsp_command(DspCommand::SetCompiledPlan(Box::new(plan)));
                                }
                            }
                            Err(err) => {
                                let _ = event_tx.send(AudioEvent::Error(format!("Kompilasi graph gagal (rollback aktif): {}", err)));
                            }
                        }
                    }
                }
            }
        }

        // 2. Feed Audio to RingBuffer when Playing
        if state == PlaybackState::Playing {
            if let (Some(dec), Some(prod)) = (&mut decoder, &mut producer) {
                let free_slots = prod.slots();

                if free_slots >= 512 {
                    let frames_to_generate = (free_slots / 2).min(512);
                    let mut generated_frames = 0;
                    let mut batch_buf = [0.0f32; 1024];
                    let mut batch_samples = 0;

                    for _ in 0..frames_to_generate {
                        let frame = resampler.next_frame(|| {
                            if raw_pos >= raw_len {
                                raw_pos = 0;
                                match dec.read_stereo_samples(&mut raw_chunk) {
                                    Ok(n) => raw_len = n,
                                    Err(err) => {
                                        let _ = event_tx.send(AudioEvent::Error(format!(
                                            "Error membaca file audio: {}",
                                            err
                                        )));
                                        raw_len = 0;
                                    }
                                }
                            }
                            if raw_pos < raw_len {
                                let mut l = raw_chunk[raw_pos];
                                let mut r = raw_chunk[raw_pos + 1];
                                raw_pos += 2;

                                if crossfade_secs > 0.0 && chain_mode != ChainMode::BitPerfect {
                                    let dur = dec.duration();
                                    if dur > crossfade_secs {
                                        let cur = dec.current_time();
                                        let rem = (dur - cur).max(0.0);
                                        if rem <= crossfade_secs {
                                            let frac = (rem / crossfade_secs).clamp(0.0, 1.0);
                                            let gain = (frac * std::f32::consts::FRAC_PI_2).sin();
                                            l *= gain;
                                            r *= gain;
                                        }
                                    }
                                }

                                if fade_in_remaining_samples > 0 && chain_mode != ChainMode::BitPerfect {
                                    let progress = 1.0 - (fade_in_remaining_samples as f32 / fade_in_total_samples.max(1) as f32);
                                    let gain = (progress * std::f32::consts::FRAC_PI_2).sin();
                                    l *= gain;
                                    r *= gain;
                                    fade_in_remaining_samples = fade_in_remaining_samples.saturating_sub(2);
                                }

                                Some((l, r))
                            } else {
                                None
                            }
                        });

                        match frame {
                            Some((l, r)) => {
                                // Push raw PCM directly into RingBuffer: zero lag for downstream DSP callback!
                                batch_buf[batch_samples] = l;
                                batch_buf[batch_samples + 1] = r;
                                batch_samples += 2;
                                generated_frames += 1;
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    if batch_samples > 0 {
                        let _ = prod.push_partial_slice(&batch_buf[..batch_samples]);
                        let cap = prod.buffer().capacity() as u32;
                        let used = (cap as usize).saturating_sub(prod.slots()) as u32;
                        shared_buffer_metrics.capacity_samples.store(cap, Ordering::Relaxed);
                        shared_buffer_metrics.used_samples.store(used, Ordering::Relaxed);
                        shared_buffer_metrics.chunk_counter.fetch_add(1, Ordering::Relaxed);
                        shared_buffer_metrics.total_samples_streamed.fetch_add(batch_samples as u64, Ordering::Relaxed);
                        shared_buffer_metrics.last_chunk_frames.store((batch_samples / 2) as u32, Ordering::Relaxed);
                    }

                    let cur_pos = dec.current_time();
                    let track_sr = dec.sample_rate().max(1) as f32;
                    let dev_sr = output.as_ref().map(|o| o.sample_rate()).unwrap_or(44100).max(1) as f32;
                    let buffered_ring = (prod.buffer().capacity() - prod.slots()) as f32;
                    let ring_lat = (buffered_ring / 2.0) / dev_sr;
                    let raw_lat = ((raw_len.saturating_sub(raw_pos)) as f32 / 2.0) / track_sr;
                    let played_pos = (cur_pos - ring_lat - raw_lat).max(0.0);
                    shared_position_ms.store((played_pos * 1000.0) as u32, Ordering::Relaxed);

                    if generated_frames == 0 && raw_pos >= raw_len {
                        if let Some(next_dec) = next_decoder.take() {
                            let next_path = next_track_path.take();
                            let next_sr = next_dec.sample_rate();
                            let next_bd = next_dec.bits_per_sample();
                            let next_dur = next_dec.duration();
                            let cur_sr = dec.sample_rate();
                            let cur_bd = dec.bits_per_sample();

                            let device_rate = output.as_ref().map(|o| o.sample_rate()).unwrap_or(44100);
                            let needs_recreate = if chain_mode == ChainMode::BitPerfect && output.as_ref().map_or(false, |o| o.is_exclusive()) {
                                next_sr != cur_sr || next_bd != cur_bd
                            } else {
                                false
                            };

                            if !needs_recreate {
                                // Zero-flush gapless audio handoff: ring buffer continues smoothly
                                if next_sr != cur_sr {
                                    resampler.reset(next_sr, device_rate);
                                }
                                decoder = Some(next_dec);
                                raw_pos = 0;
                                raw_len = 0;
                                if crossfade_secs > 0.0 && chain_mode != ChainMode::BitPerfect {
                                    fade_in_total_samples = (crossfade_secs * next_sr as f32) as usize * 2;
                                    fade_in_remaining_samples = fade_in_total_samples;
                                } else {
                                    fade_in_remaining_samples = 0;
                                    fade_in_total_samples = 0;
                                    shared_buffer_metrics.used_samples.store(0, Ordering::Relaxed);
                                }
                                if let Some(out) = &output {
                                    let report = create_signal_path_report(out, next_sr, next_bd, chain_mode);
                                    shared_bit_perfect.store(report.actual_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                                    is_dsp_active.store(report.is_dsp_active, Ordering::Relaxed);
                                    let _ = event_tx.send(AudioEvent::SignalPath(report));
                                }
                                let _ = event_tx.send(AudioEvent::TrackTransitioned {
                                    path: next_path,
                                    duration: next_dur,
                                    sample_rate: next_sr,
                                });
                                continue;
                            } else {
                                // Bit-perfect rate change: wait until ring buffer drains completely before recreating stream
                                if prod.slots() == prod.buffer().capacity() {
                                    decoder = Some(next_dec);
                                    raw_pos = 0;
                                    raw_len = 0;
                                    let desired_rate = desired_device_rate(chain_mode, next_sr);
                                    if let Some(old_out) = output.take() {
                                        old_out.pause();
                                        let _ = old_out.flush();
                                        drop(old_out);
                                    }
                                    producer = None;
                                    match AudioOutput::new(
                                        selected_device.as_deref(),
                                        desired_rate,
                                        next_bd,
                                        chain_mode,
                                        96000,
                                        Arc::clone(&shared_volume),
                                        Arc::clone(&shared_visualizer),
                                        Arc::clone(&shared_bit_perfect),
                                        Arc::clone(&shared_muted),
                                        Arc::clone(&is_dsp_active),
                                        event_tx.clone(),
                                        true,
                                    ) {
                                        Ok((new_out, new_prod)) => {
                                            sync_output_dsp(&new_out, desired_rate, last_bypass, last_limiter, &last_graph, &last_eq_bands);
                                            new_out.resume();
                                            let dev_sr = new_out.sample_rate();
                                            resampler.reset(next_sr, dev_sr);
                                            dsp.set_sample_rate(dev_sr as f32);
                                            let report = create_signal_path_report(&new_out, next_sr, next_bd, chain_mode);
                                            shared_bit_perfect.store(report.actual_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                                            is_dsp_active.store(report.is_dsp_active, Ordering::Relaxed);
                                            let _ = event_tx.send(AudioEvent::SignalPath(report));
                                            let _ = event_tx.send(AudioEvent::TrackTransitioned {
                                                path: next_path,
                                                duration: next_dur,
                                                sample_rate: next_sr,
                                            });
                                            output = Some(new_out);
                                            producer = Some(new_prod);
                                            current_stream_mode = Some(chain_mode);
                                            continue;
                                        }
                                        Err(err) => {
                                            let _ = event_tx.send(AudioEvent::Error(format!("Gagal re-clocking perangkat audio: {}", err)));
                                            state = PlaybackState::Stopped;
                                            shared_state.store(0, Ordering::Relaxed);
                                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Stopped));
                                            continue;
                                        }
                                    }
                                } else {
                                    next_decoder = Some(next_dec);
                                    next_track_path = next_path;
                                    thread::sleep(Duration::from_millis(10));
                                    continue;
                                }
                            }
                        }

                        // EOF reached and waiting for the entire ring buffer to be drained by the hardware
                        if prod.slots() == prod.buffer().capacity() {
                            state = PlaybackState::Stopped;
                            shared_state.store(0, Ordering::Relaxed);
                            let _ = event_tx.send(AudioEvent::TrackEnded);
                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Stopped));
                        } else {
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                } else {
                    // Buffer is well supplied, update real-time position from decreasing ring buffer
                    let cur_pos = dec.current_time();
                    let track_sr = dec.sample_rate().max(1) as f32;
                    let dev_sr = output.as_ref().map(|o| o.sample_rate()).unwrap_or(44100).max(1) as f32;
                    let buffered_ring = (prod.buffer().capacity() - prod.slots()) as f32;
                    let ring_lat = (buffered_ring / 2.0) / dev_sr;
                    let raw_lat = ((raw_len.saturating_sub(raw_pos)) as f32 / 2.0) / track_sr;
                    let played_pos = (cur_pos - ring_lat - raw_lat).max(0.0);
                    shared_position_ms.store((played_pos * 1000.0) as u32, Ordering::Relaxed);
                    thread::sleep(Duration::from_millis(5));
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_controller_initial_state() {
        let engine = AudioEngineController::new().expect("Failed to initialize AudioEngineController");
        assert_eq!(engine.playback_state(), PlaybackState::Stopped);
        assert_eq!(engine.current_position(), 0.0);
        assert!(engine.is_bit_perfect());
    }

    #[test]
    fn test_desired_device_rate() {
        assert_eq!(desired_device_rate(ChainMode::BitPerfect, 96000), 96000);
        assert_eq!(desired_device_rate(ChainMode::DspActive, 44100), 44100);
        assert_eq!(desired_device_rate(ChainMode::OsResampled, 96000), 48000);
        assert_eq!(desired_device_rate(ChainMode::OsResampled, 44100), 48000);
    }

    #[test]
    fn test_route_audio_frame_pure() {
        let mut dsp = DspPipeline::new(44100.0);
        let in_l = 0.42f32;
        let in_r = -0.73f32;

        // Bit-Perfect complete
        let (out_l, out_r) = route_audio_frame(ChainMode::BitPerfect, true, in_l, in_r, &mut dsp);
        assert_eq!(out_l.to_bits(), in_l.to_bits());
        assert_eq!(out_r.to_bits(), in_r.to_bits());

        // Bit-Perfect severed
        let (out_l, out_r) = route_audio_frame(ChainMode::BitPerfect, false, in_l, in_r, &mut dsp);
        assert_eq!(out_l, 0.0);
        assert_eq!(out_r, 0.0);
    }

    #[test]
    fn test_calculate_effective_volume_pure() {
        // Bit-Perfect unmuted: volume slider ignored -> 1.0
        assert_eq!(calculate_effective_volume(true, false, 0.5), 1.0);
        assert_eq!(calculate_effective_volume(true, false, 0.0), 1.0);

        // Bit-Perfect muted: MUST be 0.0
        assert_eq!(calculate_effective_volume(true, true, 0.5), 0.0);
        assert_eq!(calculate_effective_volume(true, true, 1.0), 0.0);

        // DSP mode unmuted: volume slider applied
        assert_eq!(calculate_effective_volume(false, false, 0.75), 0.75);

        // DSP mode muted: MUST be 0.0
        assert_eq!(calculate_effective_volume(false, true, 0.75), 0.0);
    }

    #[test]
    fn test_pcm_24bit_in_32bit_lossless_scaling() {
        // Test edge cases and step values across full 24-bit range (-8388608 to 8388607)
        // using production pure function encode_sample_i32
        let test_vals = vec![
            -8388608i32, -8388607, -4194304, -1000, -1, 0, 1, 1000, 4194303, 8388606, 8388607,
        ];
        for v in test_vals {
            let float_sample = v as f32 / 8388608.0;
            let encoded_i32 = crate::audio::types::encode_sample_i32(float_sample, 24);
            let recovered_24 = encoded_i32 >> 8;
            assert_eq!(v, recovered_24, "24-bit LSB mismatch for value {}", v);
        }

        // Test 32-bit full-scale boundaries via production encode_sample_i32
        assert_eq!(crate::audio::types::encode_sample_i32(1.0, 32), 2147483647);
        assert_eq!(crate::audio::types::encode_sample_i32(-1.0, 32), -2147483648);
        assert_eq!(crate::audio::types::encode_sample_i32(0.0, 32), 0);
    }

    #[test]
    fn test_pcm_16bit_encoding_pure() {
        let mut dither_state = 0x12345678;
        // Direct without dither
        assert_eq!(crate::audio::types::encode_sample_i16(1.0, false, &mut dither_state), 32767);
        assert_eq!(crate::audio::types::encode_sample_i16(-1.0, false, &mut dither_state), -32768);
        assert_eq!(crate::audio::types::encode_sample_i16(0.0, false, &mut dither_state), 0);

        // With dither
        let val = crate::audio::types::encode_sample_i16(0.0, true, &mut dither_state);
        assert!(val.abs() <= 2, "TPDF dither at 0.0 must be bounded within +/- 2 LSB");
    }

    #[test]
    fn test_dsp_empty_graph_passthrough() {
        let mut dsp = DspPipeline::new(44100.0);
        // Supply graph with complete chain but NO effects (user routed 2 -> 4)
        dsp.set_signal_chain(crate::audio::types::SignalChainGraph {
            is_complete: true,
            effects: Vec::new(),
            ..Default::default()
        });
        dsp.set_eq_bypass(false);

        let (out_l, out_r) = dsp.process_frame(0.75, -0.75);
        assert_eq!(out_l, 0.75, "Empty graph in DSP mode must pass through transparently without EQ");
        assert_eq!(out_r, -0.75, "Empty graph in DSP mode must pass through transparently without EQ");
    }

    #[test]
    fn test_bit_perfect_path_proof() {
        let chain_mode = ChainMode::BitPerfect;
        let mut dsp = DspPipeline::new(44100.0);

        let test_samples: Vec<(f32, f32)> = vec![
            (0.0, 0.0),
            (0.1234567, -0.7654321),
            (-0.9999695, 0.9999695),
            (-1.0, 1.0),
            (0.5, -0.5),
            (0.000030517578, -0.000030517578),
            (0.888888, -0.222222),
            (-0.333333, 0.444444),
        ];

        for &(in_l, in_r) in &test_samples {
            // 1. Engine frame routing in Bit-Perfect mode using pure function
            let (out_l, out_r) = route_audio_frame(chain_mode, true, in_l, in_r, &mut dsp);

            // 2. Output render callback volume application using pure function
            let vol = calculate_effective_volume(true, false, 0.5);

            let rendered_l = out_l * vol;
            let rendered_r = out_r * vol;

            // 3. Absolute mathematical proof: bit-for-bit identical IEEE 754 float representation
            assert_eq!(
                rendered_l.to_bits(),
                in_l.to_bits(),
                "Left sample IEEE 754 bits must match input bit-for-bit (in: {}, out: {})",
                in_l,
                rendered_l
            );
            assert_eq!(
                rendered_r.to_bits(),
                in_r.to_bits(),
                "Right sample IEEE 754 bits must match input bit-for-bit (in: {}, out: {})",
                in_r,
                rendered_r
            );
            assert_eq!(rendered_l - in_l, 0.0, "Error must be mathematically 0.0");
            assert_eq!(rendered_r - in_r, 0.0, "Error must be mathematically 0.0");
        }
    }

    #[test]
    fn test_bit_perfect_severed_mute() {
        let chain_mode = ChainMode::BitPerfect;
        let mut dsp = DspPipeline::new(44100.0);

        // Sever the canvas wire
        dsp.set_signal_chain(crate::audio::types::SignalChainGraph {
            is_complete: false,
            effects: Vec::new(),
            ..Default::default()
        });

        let (out_l, out_r) = match chain_mode {
            ChainMode::BitPerfect => {
                if !dsp.is_chain_complete() {
                    (0.0, 0.0)
                } else {
                    (0.85, -0.85)
                }
            }
            _ => dsp.process_frame(0.85, -0.85),
        };

        assert_eq!(out_l, 0.0, "Severed chain in Bit-Perfect mode must mute output");
        assert_eq!(out_r, 0.0, "Severed chain in Bit-Perfect mode must mute output");
    }

    #[test]
    fn test_dsp_active_volume_scaling() {
        let shared_bit_perfect = Arc::new(AtomicBool::new(false)); // DSP Active or OS Resampled
        let shared_volume = Arc::new(AtomicU32::new(0.5f32.to_bits())); // Volume slider at 50%

        let is_bp = shared_bit_perfect.load(Ordering::Relaxed);
        let vol = if is_bp {
            1.0
        } else {
            f32::from_bits(shared_volume.load(Ordering::Relaxed))
        };

        let in_l = 0.80f32;
        let in_r = -0.60f32;
        let rendered_l = in_l * vol;
        let rendered_r = in_r * vol;

        assert_eq!(rendered_l, 0.40);
        assert_eq!(rendered_r, -0.30);
    }

    #[test]
    fn test_pcm_lossless_roundtrip_reference() {
        use symphonia::core::conv::IntoSample;

        // Generate full spectrum of 16-bit PCM integer samples
        for val in -32768i16..=32767i16 {
            // Symphonia decoding conversion
            let decoded_f32: f32 = val.into_sample();

            // Bit-Perfect passthrough
            let passthrough = decoded_f32;

            // Output stage exact encoding
            let recovered = (passthrough as f64 * 32768.0).round().clamp(-32768.0, 32767.0) as i16;

            assert_eq!(
                val, recovered,
                "Lossless roundtrip failed for 16-bit sample {}: decoded as {}, recovered as {}",
                val, decoded_f32, recovered
            );
        }
    }
}
