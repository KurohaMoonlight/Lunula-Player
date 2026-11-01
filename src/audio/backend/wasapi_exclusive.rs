#[cfg(target_os = "windows")]
use wasapi::*;
use rtrb::{Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use crossbeam_channel::Sender;

use super::OutputBackend;
use crate::audio::dsp::DspPipeline;
use crate::audio::types::DspCommand;
use crate::audio::visualizer::VisualizerRingBuffer;

struct SendWrap<T>(T);
unsafe impl<T> Send for SendWrap<T> {}
impl<T> SendWrap<T> {
    pub fn new(val: T) -> Self { Self(val) }
    pub fn into_inner(self) -> T { self.0 }
}

pub struct WasapiExclusiveBackend {
    sample_rate: u32,
    channels: u16,
    bit_depth: u8,
    format_desc: String,
    device_name: String,
    raw_event_handle: isize,
    is_paused: Arc<AtomicBool>,
    flush_requested: Arc<AtomicBool>,
    flush_ack: Arc<AtomicBool>,
    stop_signal: Arc<AtomicBool>,
    shared_volume: Arc<AtomicU32>,
    worker_handle: Option<JoinHandle<()>>,
    dsp_tx: Sender<DspCommand>,
}

unsafe extern "system" {
    fn SetEvent(hEvent: *mut std::ffi::c_void) -> i32;
}

impl OutputBackend for WasapiExclusiveBackend {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn bit_depth(&self) -> u8 {
        self.bit_depth
    }

    fn format_desc(&self) -> String {
        self.format_desc.clone()
    }

    fn device_name(&self) -> &str {
        &self.device_name
    }

    fn is_exclusive(&self) -> bool {
        true // Native hardware WASAPI Exclusive Mode
    }

    fn pause(&self) {
        self.is_paused.store(true, Ordering::Relaxed);
    }

    fn resume(&self) {
        self.is_paused.store(false, Ordering::Relaxed);
    }

    fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }

    fn set_volume(&self, vol: f32) {
        self.shared_volume.store(vol.clamp(0.0, 1.5).to_bits(), Ordering::Relaxed);
    }

    fn flush(&self) -> bool {
        self.flush_ack.store(false, Ordering::SeqCst);
        self.flush_requested.store(true, Ordering::SeqCst);
        let start = std::time::Instant::now();
        while !self.flush_ack.load(Ordering::SeqCst) {
            if start.elapsed() >= std::time::Duration::from_millis(50) {
                self.flush_requested.store(false, Ordering::SeqCst);
                return false;
            }
            std::thread::yield_now();
        }
        true
    }

    fn send_dsp_command(&self, cmd: DspCommand) {
        let _ = self.dsp_tx.send(cmd);
    }
}

impl Drop for WasapiExclusiveBackend {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        if self.raw_event_handle != 0 {
            unsafe {
                SetEvent(self.raw_event_handle as *mut std::ffi::c_void);
            }
        }
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl WasapiExclusiveBackend {
    pub fn try_new(
        preferred_device_name: Option<&str>,
        preferred_sample_rate: u32,
        preferred_bit_depth: u8,
        buffer_capacity: usize,
        shared_volume: Arc<AtomicU32>,
        visualizer: Arc<VisualizerRingBuffer>,
        bit_perfect: Arc<AtomicBool>,
        shared_muted: Arc<AtomicBool>,
        event_tx: Sender<crate::audio::types::AudioEvent>,
    ) -> Result<(Self, Producer<f32>), Box<dyn std::error::Error + Send + Sync>> {
        let _ = initialize_mta();

        let enumerator = DeviceEnumerator::new()?;
        let device = if let Some(target_name) = preferred_device_name {
            let found = enumerator
                .get_device_collection(&Direction::Render)
                .ok()
                .and_then(|coll| coll.get_device_with_name(target_name).ok());
            if let Some(d) = found {
                d
            } else {
                enumerator.get_default_device(&Direction::Render)?
            }
        } else {
            enumerator.get_default_device(&Direction::Render)?
        };

        let device_name = device.get_friendlyname().unwrap_or_else(|_| "Default Audio Device".to_string());
        let mut client = device.get_iaudioclient()?;

        // Try supported bit depths in Exclusive mode: 32-bit (for 24-bit audiophile files), 24-bit, 16-bit, or 32-bit Float
        let candidate_formats = if preferred_bit_depth == 0 || preferred_bit_depth > 16 {
            vec![
                (32, 24, SampleType::Int, "24-bit in 32-bit PCM (WASAPI Exclusive)"),
                (32, 32, SampleType::Float, "32-bit Float (WASAPI Exclusive)"),
                (32, 32, SampleType::Int, "32-bit Integer PCM (WASAPI Exclusive)"),
                (16, 16, SampleType::Int, "16-bit Integer PCM (WASAPI Exclusive)"),
            ]
        } else {
            vec![
                (16, 16, SampleType::Int, "16-bit Integer PCM (WASAPI Exclusive)"),
                (32, 24, SampleType::Int, "24-bit in 32-bit PCM (WASAPI Exclusive)"),
                (32, 32, SampleType::Float, "32-bit Float (WASAPI Exclusive)"),
                (32, 32, SampleType::Int, "32-bit Integer PCM (WASAPI Exclusive)"),
            ]
        };

        let mut matched_fmt = None;
        for (store_bits, valid_bits, sample_type, desc) in candidate_formats {
            let wave_fmt = WaveFormat::new(store_bits, valid_bits, &sample_type, preferred_sample_rate as usize, 2, None);
            if client.is_supported(&wave_fmt, &ShareMode::Exclusive).is_ok() {
                matched_fmt = Some((wave_fmt, store_bits, valid_bits, sample_type, desc));
                break;
            }
        }

        let (wave_fmt, store_bits, valid_bits, sample_type, format_desc) = matched_fmt
            .ok_or_else(|| "Hardware audio tidak mendukung sample rate track dalam WASAPI Exclusive Mode")?;

        let (_, min_time) = client.get_device_period()?;
        client.initialize_client(
            &wave_fmt,
            &Direction::Render,
            &StreamMode::EventsExclusive { period_hns: min_time },
        )?;

        let h_event = client.set_get_eventhandle()?;
        let raw_event_handle = h_event.as_handle().0 as isize;
        let render_client = client.get_audiorenderclient()?;
        let is_paused = Arc::new(AtomicBool::new(false));
        let flush_requested = Arc::new(AtomicBool::new(false));
        let flush_ack = Arc::new(AtomicBool::new(false));
        let stop_signal = Arc::new(AtomicBool::new(false));

        let is_paused_worker = Arc::clone(&is_paused);
        let flush_worker = Arc::clone(&flush_requested);
        let flush_ack_worker = Arc::clone(&flush_ack);
        let stop_worker = Arc::clone(&stop_signal);
        let bp_worker = Arc::clone(&bit_perfect);
        let muted_worker = Arc::clone(&shared_muted);
        let vol_worker = Arc::clone(&shared_volume);
        let vis_worker = Arc::clone(&visualizer);
        let event_tx_worker = event_tx.clone();

        let (producer, mut consumer) = RingBuffer::<f32>::new(buffer_capacity);
        let (dsp_tx, dsp_rx) = crossbeam_channel::unbounded::<DspCommand>();

        client.start_stream()?;

        let client_wrap = SendWrap::new(client);
        let h_event_wrap = SendWrap::new(h_event);
        let render_client_wrap = SendWrap::new(render_client);

        let worker_handle = thread::Builder::new()
            .name("lunula-wasapi-exclusive".into())
            .spawn(move || {
                let _ = initialize_mta();
                let client = client_wrap.into_inner();
                let h_event = h_event_wrap.into_inner();
                let render_client = render_client_wrap.into_inner();
                let bytes_per_frame = if store_bits == 16 { 4 } else { 8 };
                let max_frames = client.get_buffer_size().unwrap_or(2048) as usize;
                let mut raw_bytes: Vec<u8> = Vec::with_capacity(max_frames * bytes_per_frame);
                let mut dither_state: u32 = 0x12345678;
                let mut batch_f32 = [0.0f32; 1024];
                let mut dsp = DspPipeline::new(preferred_sample_rate as f32);

                while !stop_worker.load(Ordering::Relaxed) {
                    if let Ok(()) = h_event.wait_for_event(20) {
                        if stop_worker.load(Ordering::Relaxed) {
                            break;
                        }

                        while let Ok(cmd) = dsp_rx.try_recv() {
                            match cmd {
                                DspCommand::SetNodeParam { node_id, param_id, value } => {
                                    dsp.set_node_param(node_id, param_id, value);
                                }
                                DspCommand::SetEqBand { index, gain_db } => {
                                    dsp.set_eq_band(index, gain_db);
                                }
                                DspCommand::SetEqBypass(bp) => {
                                    dsp.set_eq_bypass(bp);
                                }
                                DspCommand::SetSignalChainGraph(g) => {
                                    dsp.set_signal_chain(g);
                                }
                                DspCommand::ResetStates => {
                                    dsp.reset_states();
                                }
                                DspCommand::SetSampleRate(sr) => {
                                    dsp.set_sample_rate(sr);
                                }
                                _ => {}
                            }
                        }

                        let padding = match client.get_current_padding() {
                            Ok(p) => p,
                            Err(e) => {
                                let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                    "WASAPI Exclusive get_padding gagal: {}",
                                    e
                                )));
                                break;
                            }
                        };
                        let buffer_frame_count = match client.get_buffer_size() {
                            Ok(b) => b,
                            Err(e) => {
                                let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                    "WASAPI Exclusive get_buffer_size gagal: {}",
                                    e
                                )));
                                break;
                            }
                        };
                        let available_frames = buffer_frame_count.saturating_sub(padding);

                        if flush_worker.swap(false, Ordering::SeqCst) {
                            while consumer.pop().is_ok() {}
                            if available_frames > 0 {
                                raw_bytes.clear();
                                raw_bytes.resize(available_frames as usize * bytes_per_frame, 0);
                                if let Err(e) = render_client.write_to_device(available_frames as usize, &raw_bytes, None) {
                                    let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                        "WASAPI Exclusive flush write gagal: {}",
                                        e
                                    )));
                                    break;
                                }
                            }
                            flush_ack_worker.store(true, Ordering::SeqCst);
                            continue;
                        }

                        if is_paused_worker.load(Ordering::Relaxed) {
                            // On pause in WASAPI exclusive mode, write silence to prevent circular hardware buffer looping
                            if available_frames > 0 {
                                raw_bytes.clear();
                                raw_bytes.resize(available_frames as usize * bytes_per_frame, 0);
                                if let Err(e) = render_client.write_to_device(available_frames as usize, &raw_bytes, None) {
                                    let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                        "WASAPI Exclusive pause write gagal: {}",
                                        e
                                    )));
                                    break;
                                }
                                for _ in 0..available_frames {
                                    vis_worker.push_sample(0.0);
                                }
                            }
                            continue;
                        }

                        if available_frames > 0 {
                            let frames_to_write = available_frames as usize;
                            let is_bp = bp_worker.load(Ordering::Relaxed);
                            let is_muted = muted_worker.load(Ordering::Relaxed);
                            let vol_raw = f32::from_bits(vol_worker.load(Ordering::Relaxed));
                            let vol = crate::audio::types::calculate_effective_volume(is_bp, is_muted, vol_raw);

                            if vol <= 0.0001 {
                                let mut frames_remaining = frames_to_write;
                                while frames_remaining > 0 {
                                    let chunk_frames = frames_remaining.min(512);
                                    let chunk_samples = chunk_frames * 2;
                                    let _ = consumer.pop_partial_slice(&mut batch_f32[..chunk_samples]);
                                    frames_remaining -= chunk_frames;
                                }
                                for _ in 0..frames_to_write {
                                    vis_worker.push_sample(0.0);
                                }
                                raw_bytes.clear();
                                raw_bytes.resize(frames_to_write * bytes_per_frame, 0);
                                if let Err(e) = render_client.write_to_device(frames_to_write, &raw_bytes, None) {
                                    let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                        "WASAPI Exclusive write gagal: {}",
                                        e
                                    )));
                                    break;
                                }
                                continue;
                            }

                            raw_bytes.clear();

                            // High-Performance Batch Pop: drains hardware buffer in 512-frame chunks with 1 atomic operation per chunk
                            let mut frames_remaining = frames_to_write;
                            if store_bits == 16 {
                                let should_dither = preferred_bit_depth > 16 && !is_bp;
                                while frames_remaining > 0 {
                                    let chunk_frames = frames_remaining.min(512);
                                    let chunk_samples = chunk_frames * 2;
                                    let (popped, _) = consumer.pop_partial_slice(&mut batch_f32[..chunk_samples]);
                                    let popped_len = popped.len();
                                    if popped_len < chunk_samples {
                                        batch_f32[popped_len..chunk_samples].fill(0.0);
                                    }
                                    if !is_bp {
                                        dsp.process_interleaved(&mut batch_f32[..chunk_samples]);
                                    }

                                    for pair in batch_f32[..chunk_samples].chunks_exact(2) {
                                        let l = pair[0] * vol;
                                        let r = pair[1] * vol;
                                        vis_worker.push_sample((l + r) * 0.5);

                                        let i_l = crate::audio::types::encode_sample_i16(l, should_dither, &mut dither_state);
                                        let i_r = crate::audio::types::encode_sample_i16(r, should_dither, &mut dither_state);
                                        raw_bytes.extend_from_slice(&i_l.to_le_bytes());
                                        raw_bytes.extend_from_slice(&i_r.to_le_bytes());
                                    }
                                    frames_remaining -= chunk_frames;
                                }
                                if let Err(e) = render_client.write_to_device(frames_to_write, &raw_bytes, None) {
                                    let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                        "WASAPI Exclusive write gagal: {}",
                                        e
                                    )));
                                    break;
                                }
                            } else if store_bits == 32 && sample_type == SampleType::Int {
                                while frames_remaining > 0 {
                                    let chunk_frames = frames_remaining.min(512);
                                    let chunk_samples = chunk_frames * 2;
                                    let (popped, _) = consumer.pop_partial_slice(&mut batch_f32[..chunk_samples]);
                                    let popped_len = popped.len();
                                    if popped_len < chunk_samples {
                                        batch_f32[popped_len..chunk_samples].fill(0.0);
                                    }
                                    if !is_bp {
                                        dsp.process_interleaved(&mut batch_f32[..chunk_samples]);
                                    }

                                    for pair in batch_f32[..chunk_samples].chunks_exact(2) {
                                        let l = pair[0] * vol;
                                        let r = pair[1] * vol;
                                        vis_worker.push_sample((l + r) * 0.5);

                                        let scaled_l = crate::audio::types::encode_sample_i32(l, valid_bits as u8);
                                        let scaled_r = crate::audio::types::encode_sample_i32(r, valid_bits as u8);
                                        raw_bytes.extend_from_slice(&scaled_l.to_le_bytes());
                                        raw_bytes.extend_from_slice(&scaled_r.to_le_bytes());
                                    }
                                    frames_remaining -= chunk_frames;
                                }
                                if let Err(e) = render_client.write_to_device(frames_to_write, &raw_bytes, None) {
                                    let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                        "WASAPI Exclusive write gagal: {}",
                                        e
                                    )));
                                    break;
                                }
                            } else {
                                // 32-bit Float
                                while frames_remaining > 0 {
                                    let chunk_frames = frames_remaining.min(512);
                                    let chunk_samples = chunk_frames * 2;
                                    let (popped, _) = consumer.pop_partial_slice(&mut batch_f32[..chunk_samples]);
                                    let popped_len = popped.len();
                                    if popped_len < chunk_samples {
                                        batch_f32[popped_len..chunk_samples].fill(0.0);
                                    }
                                    if !is_bp {
                                        dsp.process_interleaved(&mut batch_f32[..chunk_samples]);
                                    }

                                    for pair in batch_f32[..chunk_samples].chunks_exact(2) {
                                        let l = pair[0] * vol;
                                        let r = pair[1] * vol;
                                        vis_worker.push_sample((l + r) * 0.5);

                                        raw_bytes.extend_from_slice(&l.to_le_bytes());
                                        raw_bytes.extend_from_slice(&r.to_le_bytes());
                                    }
                                    frames_remaining -= chunk_frames;
                                }
                                if let Err(e) = render_client.write_to_device(frames_to_write, &raw_bytes, None) {
                                    let _ = event_tx_worker.send(crate::audio::types::AudioEvent::Error(format!(
                                        "WASAPI Exclusive write gagal: {}",
                                        e
                                    )));
                                    break;
                                }
                            }
                        } else {
                            thread::sleep(Duration::from_millis(1));
                        }
                    } else {
                        thread::sleep(Duration::from_millis(2));
                    }
                }

                let _ = client.stop_stream();
            })?;

        Ok((
            Self {
                sample_rate: preferred_sample_rate,
                channels: 2,
                bit_depth: valid_bits as u8,
                format_desc: format_desc.to_string(),
                device_name,
                raw_event_handle,
                is_paused,
                flush_requested,
                flush_ack,
                stop_signal,
                shared_volume,
                worker_handle: Some(worker_handle),
                dsp_tx,
            },
            producer,
        ))
    }
}
