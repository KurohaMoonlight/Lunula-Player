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
use crate::audio::types::{AudioCommand, AudioEvent, ChainMode, PlaybackState, SignalPathReport};
use crate::audio::visualizer::{VisualizerRingBuffer, FFT_SIZE};
use rtrb::Producer;

pub struct AudioEngineController {
    cmd_tx: Sender<AudioCommand>,
    event_rx: Receiver<AudioEvent>,
    shared_position_ms: Arc<AtomicU32>,
    shared_state: Arc<AtomicU8>,
    shared_volume: Arc<AtomicU32>,
    shared_bit_perfect: Arc<AtomicBool>,
    shared_visualizer: Arc<VisualizerRingBuffer>,
}

impl AudioEngineController {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (cmd_tx, cmd_rx) = unbounded::<AudioCommand>();
        let (event_tx, event_rx) = unbounded::<AudioEvent>();

        let shared_position_ms = Arc::new(AtomicU32::new(0));
        let shared_state = Arc::new(AtomicU8::new(0)); // 0: Stopped, 1: Playing, 2: Paused
        let shared_volume = Arc::new(AtomicU32::new(0.80f32.to_bits()));
        let shared_bit_perfect = Arc::new(AtomicBool::new(true));
        let shared_visualizer = Arc::new(VisualizerRingBuffer::new());

        let pos_clone = Arc::clone(&shared_position_ms);
        let state_clone = Arc::clone(&shared_state);
        let vol_clone = Arc::clone(&shared_volume);
        let bp_clone = Arc::clone(&shared_bit_perfect);
        let vis_clone = Arc::clone(&shared_visualizer);

        thread::Builder::new()
            .name("lunula-audio-worker".into())
            .spawn(move || {
                run_audio_worker(cmd_rx, event_tx, pos_clone, state_clone, vol_clone, bp_clone, vis_clone);
            })?;

        Ok(Self {
            cmd_tx,
            event_rx,
            shared_position_ms,
            shared_state,
            shared_volume,
            shared_bit_perfect,
            shared_visualizer,
        })
    }

    pub fn play<P: Into<PathBuf>>(&self, path: P) {
        let _ = self.cmd_tx.send(AudioCommand::Play(path.into()));
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Pause);
    }

    pub fn resume(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Resume);
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

    pub fn set_eq_band(&self, index: usize, gain_db: f32) {
        let _ = self.cmd_tx.send(AudioCommand::SetEqBand { index, gain_db });
    }

    pub fn set_eq_bypass(&self, bypass: bool) {
        let _ = self.cmd_tx.send(AudioCommand::SetEqBypass(bypass));
    }

    pub fn set_chain_mode(&self, mode: ChainMode) {
        let _ = self.cmd_tx.send(AudioCommand::SetChainMode(mode));
    }

    pub fn set_output_device(&self, device_name: Option<String>) {
        let _ = self.cmd_tx.send(AudioCommand::SetOutputDevice(device_name));
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
            if track_rate == device_rate {
                ChainMode::BitPerfect
            } else {
                // Fallback: device hardware does not natively support track rate
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

// Background Audio Feeder Worker
fn run_audio_worker(
    cmd_rx: Receiver<AudioCommand>,
    event_tx: Sender<AudioEvent>,
    shared_position_ms: Arc<AtomicU32>,
    shared_state: Arc<AtomicU8>,
    shared_volume: Arc<AtomicU32>,
    shared_bit_perfect: Arc<AtomicBool>,
    shared_visualizer: Arc<VisualizerRingBuffer>,
) {
    let mut decoder: Option<AudioDecoder> = None;
    let mut output: Option<AudioOutput> = None;
    let mut producer: Option<Producer<f32>> = None;
    let mut dsp = DspPipeline::new(44100.0);
    let mut resampler = StereoResampler::new(44100, 44100);
    let mut chain_mode = ChainMode::BitPerfect;
    let mut state = PlaybackState::Stopped;
    let mut selected_device: Option<String> = None;
    let is_dsp_active = Arc::new(AtomicBool::new(false));

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
                                            || (chain_mode == ChainMode::BitPerfect && !out.is_exclusive())
                                            || (chain_mode != ChainMode::BitPerfect && out.is_exclusive())
                                            || (selected_device.is_some() && Some(out.device_name()) != selected_device.as_deref())
                                    }
                                    None => true,
                                };

                                if need_recreate {
                                    match AudioOutput::new(
                                        selected_device.as_deref(),
                                        desired_rate,
                                        track_bit_depth,
                                        chain_mode,
                                        96000,
                                        Arc::clone(&shared_volume),
                                        Arc::clone(&shared_visualizer),
                                        Arc::clone(&shared_bit_perfect),
                                        Arc::clone(&is_dsp_active),
                                    ) {
                                        Ok((out, prod)) => {
                                            output = Some(out);
                                            producer = Some(prod);
                                        }
                                        Err(err) => {
                                            let _ = event_tx.send(AudioEvent::Error(format!(
                                                "Inisialisasi perangkat audio gagal: {}",
                                                err
                                            )));
                                        }
                                    }
                                } else if let Some(out) = &output {
                                    // Flush ring buffer without stream recreation: zero clicks, keeps exclusive mode lock
                                    out.flush();
                                }

                                if let Some(out) = &output {
                                    out.resume();
                                    let device_rate = out.sample_rate();
                                    // Always sync resampler: from track rate to actual device consumption rate
                                    resampler.reset(track_sample_rate, device_rate);
                                    dsp.set_sample_rate(device_rate as f32);
                                    dsp.set_eq_bypass(chain_mode != ChainMode::DspActive);
                                    dsp.reset_states();

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
                                let _ = event_tx.send(AudioEvent::Error(format!(
                                    "Gagal membuka file audio: {}",
                                    err
                                )));
                            }
                        }
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
                        if state == PlaybackState::Paused {
                            if let Some(out) = &output {
                                out.resume();
                            }
                            state = PlaybackState::Playing;
                            shared_state.store(1, Ordering::Relaxed);
                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Playing));
                        }
                    }
                    AudioCommand::Stop => {
                        if let Some(out) = &output {
                            out.pause();
                        }
                        decoder = None;
                        raw_pos = 0;
                        raw_len = 0;
                        state = PlaybackState::Stopped;
                        shared_state.store(0, Ordering::Relaxed);
                        shared_position_ms.store(0, Ordering::Relaxed);
                        let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Stopped));
                    }
                    AudioCommand::Seek(seconds) => {
                        if let Some(dec) = &mut decoder {
                            if let Ok(()) = dec.seek(seconds) {
                                let pos = dec.current_time();
                                shared_position_ms.store((pos * 1000.0) as u32, Ordering::Relaxed);
                                let _ = event_tx.send(AudioEvent::Position(pos));

                                raw_pos = 0;
                                raw_len = 0;

                                if let Some(out) = &output {
                                    out.flush();
                                    let sr = out.sample_rate();
                                    resampler.reset(dec.sample_rate(), sr);
                                }
                            }
                        }
                    }
                    AudioCommand::SetVolume(vol) => {
                        dsp.set_volume(vol);
                    }
                    AudioCommand::SetEqBand { index, gain_db } => {
                        dsp.set_eq_band(index, gain_db);
                    }
                    AudioCommand::SetEqBypass(bypass) => {
                        dsp.set_eq_bypass(bypass);
                    }
                    AudioCommand::SetOutputDevice(dev) => {
                        selected_device = dev;
                        if let (Some(dec), Some(_)) = (&decoder, &output) {
                            let track_sample_rate = dec.sample_rate();
                            let track_bit_depth = dec.bits_per_sample();
                            let desired_rate = desired_device_rate(chain_mode, track_sample_rate);
                            match AudioOutput::new(
                                selected_device.as_deref(),
                                desired_rate,
                                track_bit_depth,
                                chain_mode,
                                96000,
                                Arc::clone(&shared_volume),
                                Arc::clone(&shared_visualizer),
                                Arc::clone(&shared_bit_perfect),
                                Arc::clone(&is_dsp_active),
                            ) {
                                Ok((new_out, new_prod)) => {
                                    if state == PlaybackState::Playing {
                                        new_out.resume();
                                    }
                                    let device_rate = new_out.sample_rate();
                                    resampler.reset(track_sample_rate, device_rate);
                                    dsp.set_sample_rate(device_rate as f32);
                                    dsp.set_eq_bypass(chain_mode != ChainMode::DspActive);
                                    dsp.reset_states();

                                    let report = create_signal_path_report(&new_out, track_sample_rate, track_bit_depth, chain_mode);
                                    shared_bit_perfect.store(report.actual_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                                    is_dsp_active.store(report.is_dsp_active, Ordering::Relaxed);
                                    let _ = event_tx.send(AudioEvent::SignalPath(report));

                                    output = Some(new_out);
                                    producer = Some(new_prod);
                                }
                                Err(err) => {
                                    let _ = event_tx.send(AudioEvent::Error(format!(
                                        "Gagal beralih ke perangkat output: {}",
                                        err
                                    )));
                                }
                            }
                        }
                    }
                    AudioCommand::SetChainMode(mode) => {
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
                                        || (chain_mode == ChainMode::BitPerfect && !out.is_exclusive())
                                        || (chain_mode != ChainMode::BitPerfect && out.is_exclusive())
                                        || (selected_device.is_some() && Some(out.device_name()) != selected_device.as_deref())
                                }
                                None => true,
                            };

                            if need_recreate {
                                if let Ok((new_out, new_prod)) = AudioOutput::new(
                                    selected_device.as_deref(),
                                    desired_rate,
                                    track_bit_depth,
                                    chain_mode,
                                    96000,
                                    Arc::clone(&shared_volume),
                                    Arc::clone(&shared_visualizer),
                                    Arc::clone(&shared_bit_perfect),
                                    Arc::clone(&is_dsp_active),
                                ) {
                                    if state == PlaybackState::Playing {
                                        new_out.resume();
                                    }
                                    output = Some(new_out);
                                    producer = Some(new_prod);
                                }
                            } else if let Some(out) = &output {
                                out.flush();
                            }

                            if let Some(out) = &output {
                                let device_rate = out.sample_rate();
                                resampler.reset(track_sample_rate, device_rate);
                                dsp.set_sample_rate(device_rate as f32);
                                dsp.set_eq_bypass(chain_mode != ChainMode::DspActive);
                                dsp.reset_states();

                                let report = create_signal_path_report(out, track_sample_rate, track_bit_depth, chain_mode);
                                shared_bit_perfect.store(report.actual_mode == ChainMode::BitPerfect, Ordering::Relaxed);
                                is_dsp_active.store(report.is_dsp_active, Ordering::Relaxed);
                                let _ = event_tx.send(AudioEvent::SignalPath(report));
                            }
                        } else {
                            shared_bit_perfect.store(chain_mode == ChainMode::BitPerfect, Ordering::Relaxed);
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

                    for _ in 0..frames_to_generate {
                        let frame = resampler.next_frame(|| {
                            if raw_pos >= raw_len {
                                raw_pos = 0;
                                raw_len = dec.read_stereo_samples(&mut raw_chunk);
                            }
                            if raw_pos < raw_len {
                                let l = raw_chunk[raw_pos];
                                let r = raw_chunk[raw_pos + 1];
                                raw_pos += 2;
                                Some((l, r))
                            } else {
                                None
                            }
                        });

                        match frame {
                            Some((l, r)) => {
                                // Branch signal path strictly based on chain_mode
                                let (out_l, out_r) = match chain_mode {
                                    ChainMode::BitPerfect => (l, r), // 100% pure sample passthrough: no EQ, no clamp
                                    _ => dsp.process_frame(l, r),
                                };
                                let _ = prod.push(out_l);
                                let _ = prod.push(out_r);
                                generated_frames += 1;
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    if generated_frames > 0 {
                        let cur_pos = dec.current_time();
                        shared_position_ms.store((cur_pos * 1000.0) as u32, Ordering::Relaxed);
                    } else if raw_pos >= raw_len {
                        // EOF reached and ring buffer emptying
                        if prod.slots() >= 95000 {
                            state = PlaybackState::Stopped;
                            shared_state.store(0, Ordering::Relaxed);
                            let _ = event_tx.send(AudioEvent::TrackEnded);
                            let _ = event_tx.send(AudioEvent::StateChanged(PlaybackState::Stopped));
                        } else {
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                } else {
                    // Buffer is well supplied, yield CPU
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
}
