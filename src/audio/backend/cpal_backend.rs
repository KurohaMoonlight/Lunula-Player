use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, SampleFormat, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};

use super::OutputBackend;
use crate::audio::dsp::DspPipeline;
use crate::audio::types::DspCommand;
use crate::audio::visualizer::VisualizerRingBuffer;

pub struct CpalBackend {
    _stream: Stream,
    sample_rate: u32,
    channels: u16,
    bit_depth: u8,
    format_desc: String,
    device_name: String,
    is_paused: Arc<AtomicBool>,
    flush_requested: Arc<AtomicBool>,
    flush_ack: Arc<AtomicBool>,
    shared_volume: Arc<AtomicU32>,
    dsp_tx: Sender<DspCommand>,
}

unsafe impl Send for CpalBackend {}

impl OutputBackend for CpalBackend {
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
        false // CPAL standard stream operates in OS Shared Mode
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

impl CpalBackend {
    pub fn new(
        preferred_device_name: Option<&str>,
        preferred_sample_rate: u32,
        buffer_capacity: usize,
        shared_volume: Arc<AtomicU32>,
        visualizer: Arc<VisualizerRingBuffer>,
        bit_perfect: Arc<AtomicBool>,
        shared_muted: Arc<AtomicBool>,
        source_bit_depth: u8,
        is_dsp_active: Arc<AtomicBool>,
        event_tx: Option<Sender<crate::audio::types::AudioEvent>>,
    ) -> Result<(Self, Producer<f32>), Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let device = if let Some(target_name) = preferred_device_name {
            host.output_devices()
                .ok()
                .and_then(|mut devs| devs.find(|d| d.name().map_or(false, |n| n == target_name)))
                .or_else(|| host.default_output_device())
        } else {
            host.default_output_device()
        }
        .ok_or_else(|| "Tidak ditemukan audio output device pada sistem ini")?;

        let device_name = device.name().unwrap_or_else(|_| "Default Audio Device".to_string());
        let default_supported = device.default_output_config()?;

        // Attempt matching preferred_sample_rate
        let mut matched_config: Option<(StreamConfig, SampleFormat)> = None;
        if default_supported.sample_rate().0 == preferred_sample_rate {
            matched_config = Some((default_supported.config(), default_supported.sample_format()));
        } else if let Ok(supported) = device.supported_output_configs() {
            for conf_range in supported {
                if conf_range.channels() == 2
                    && preferred_sample_rate >= conf_range.min_sample_rate().0
                    && preferred_sample_rate <= conf_range.max_sample_rate().0
                {
                    let conf = conf_range.with_sample_rate(cpal::SampleRate(preferred_sample_rate));
                    matched_config = Some((conf.config(), conf.sample_format()));
                    break;
                }
            }
        }

        let is_paused = Arc::new(AtomicBool::new(false));
        let flush_requested = Arc::new(AtomicBool::new(false));
        let flush_ack = Arc::new(AtomicBool::new(false));

        let (config, format) = matched_config.unwrap_or_else(|| {
            (default_supported.config(), default_supported.sample_format())
        });

        let format_desc = match format {
            SampleFormat::F32 => "32-bit Float PCM (Shared)".to_string(),
            SampleFormat::I32 => "32-bit Integer PCM (Shared)".to_string(),
            SampleFormat::I16 => "16-bit Integer PCM (Shared)".to_string(),
            SampleFormat::U16 => "16-bit Unsigned PCM (Shared)".to_string(),
            _ => "PCM (Shared)".to_string(),
        };

        let bit_depth = match format {
            SampleFormat::F32 | SampleFormat::I32 => 32,
            SampleFormat::I16 | SampleFormat::U16 => 16,
            _ => 16,
        };

        let (producer, consumer) = RingBuffer::<f32>::new(buffer_capacity);
        let (dsp_tx, dsp_rx) = crossbeam_channel::unbounded::<DspCommand>();
        let stream = Self::build_stream(
            &device,
            &config,
            format,
            consumer,
            Arc::clone(&is_paused),
            Arc::clone(&flush_requested),
            Arc::clone(&flush_ack),
            Arc::clone(&bit_perfect),
            Arc::clone(&shared_muted),
            Arc::clone(&shared_volume),
            Arc::clone(&visualizer),
            source_bit_depth,
            Arc::clone(&is_dsp_active),
            dsp_rx,
            event_tx,
        )?;
        stream.play()?;

        let sample_rate = config.sample_rate.0;
        let channels = config.channels;

        Ok((
            Self {
                _stream: stream,
                sample_rate,
                channels,
                bit_depth,
                format_desc,
                device_name,
                is_paused,
                flush_requested,
                flush_ack,
                shared_volume,
                dsp_tx,
            },
            producer,
        ))
    }

    fn apply_dsp_commands(dsp: &mut DspPipeline, dsp_rx: &Receiver<DspCommand>) {
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
                DspCommand::SetLimiterEnabled(enabled) => {
                    dsp.set_limiter_enabled(enabled);
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
    }

    fn build_stream(
        device: &cpal::Device,
        config: &StreamConfig,
        sample_format: SampleFormat,
        mut consumer: Consumer<f32>,
        is_paused: Arc<AtomicBool>,
        flush_requested: Arc<AtomicBool>,
        flush_ack: Arc<AtomicBool>,
        bit_perfect: Arc<AtomicBool>,
        shared_muted: Arc<AtomicBool>,
        shared_volume: Arc<AtomicU32>,
        visualizer: Arc<VisualizerRingBuffer>,
        source_bit_depth: u8,
        _is_dsp_active: Arc<AtomicBool>,
        dsp_rx: Receiver<DspCommand>,
        event_tx: Option<Sender<crate::audio::types::AudioEvent>>,
    ) -> Result<Stream, Box<dyn std::error::Error + Send + Sync>> {
        let is_paused_cb = Arc::clone(&is_paused);
        let flush_cb = Arc::clone(&flush_requested);
        let flush_ack_cb = Arc::clone(&flush_ack);
        let bp_cb = Arc::clone(&bit_perfect);
        let muted_cb = Arc::clone(&shared_muted);
        let volume_cb = Arc::clone(&shared_volume);
        let visualizer_cb = Arc::clone(&visualizer);
        let channels_count = config.channels as usize;
        let dev_name = device.name().unwrap_or_else(|_| "Audio Device".to_string());
        let err_tx = event_tx.clone();
        let err_fn = move |err: cpal::StreamError| {
            eprintln!("Audio stream error: {}", err);
            if let Some(ref tx) = err_tx {
                let err_str = err.to_string();
                let err_lower = err_str.to_lowercase();
                let is_disconnect = err_lower.contains("device")
                    || err_lower.contains("invalidat")
                    || err_lower.contains("disconnect");
                if is_disconnect {
                    let _ = tx.send(crate::audio::types::AudioEvent::DeviceDisconnected {
                        device_name: dev_name.clone(),
                        error: err_str,
                    });
                } else {
                    let _ = tx.send(crate::audio::types::AudioEvent::Error(format!("CPAL error: {}", err_str)));
                }
            }
        };
        let stream_rate = config.sample_rate.0 as f32;

        let stream = match sample_format {
            SampleFormat::F32 => {
                let mut dsp = DspPipeline::new(stream_rate);
                device.build_output_stream(
                    config,
                    move |data: &mut [f32], _: &OutputCallbackInfo| {
                        Self::render_samples_f32(
                            data,
                            channels_count,
                            &mut consumer,
                            &is_paused_cb,
                            &flush_cb,
                            &flush_ack_cb,
                            &bp_cb,
                            &muted_cb,
                            &volume_cb,
                            &visualizer_cb,
                            &mut dsp,
                            &dsp_rx,
                        );
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::I32 => {
                let mut dsp = DspPipeline::new(stream_rate);
                device.build_output_stream(
                    config,
                    move |data: &mut [i32], _: &OutputCallbackInfo| {
                        Self::render_samples_i32(
                            data,
                            channels_count,
                            &mut consumer,
                            &is_paused_cb,
                            &flush_cb,
                            &flush_ack_cb,
                            &bp_cb,
                            &muted_cb,
                            &volume_cb,
                            &visualizer_cb,
                            source_bit_depth,
                            &mut dsp,
                            &dsp_rx,
                        );
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::I16 => {
                let mut dsp = DspPipeline::new(stream_rate);
                let mut dither_state: u32 = 0x12345678;
                device.build_output_stream(
                    config,
                    move |data: &mut [i16], _: &OutputCallbackInfo| {
                        Self::render_samples_i16(
                            data,
                            channels_count,
                            &mut consumer,
                            &is_paused_cb,
                            &flush_cb,
                            &flush_ack_cb,
                            &bp_cb,
                            &muted_cb,
                            &volume_cb,
                            &visualizer_cb,
                            source_bit_depth,
                            &mut dither_state,
                            &mut dsp,
                            &dsp_rx,
                        );
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::U16 => {
                device.build_output_stream(
                    config,
                    move |data: &mut [u16], _: &OutputCallbackInfo| {
                        Self::render_samples_u16(
                            data,
                            channels_count,
                            &mut consumer,
                            &is_paused_cb,
                            &flush_cb,
                            &flush_ack_cb,
                            &bp_cb,
                            &muted_cb,
                            &volume_cb,
                            &visualizer_cb,
                        );
                    },
                    err_fn,
                    None,
                )?
            }
            _ => return Err("Format sample audio tidak didukung oleh perangkat ini".into()),
        };

        Ok(stream)
    }

    // Direct F32 Hardware Rendering
    fn render_samples_f32(
        data: &mut [f32],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
        flush_requested: &AtomicBool,
        flush_ack: &AtomicBool,
        bit_perfect: &AtomicBool,
        muted: &AtomicBool,
        volume: &AtomicU32,
        visualizer: &VisualizerRingBuffer,
        dsp: &mut DspPipeline,
        dsp_rx: &Receiver<DspCommand>,
    ) {
        Self::apply_dsp_commands(dsp, dsp_rx);

        if flush_requested.swap(false, Ordering::SeqCst) {
            while consumer.pop().is_ok() {}
            data.fill(0.0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            flush_ack.store(true, Ordering::SeqCst);
            return;
        }

        if is_paused.load(Ordering::Relaxed) || !dsp.is_chain_complete() {
            data.fill(0.0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let is_bp = bit_perfect.load(Ordering::Relaxed);
        let is_muted = muted.load(Ordering::Relaxed);
        let vol_raw = f32::from_bits(volume.load(Ordering::Relaxed));
        let vol = crate::audio::types::calculate_effective_volume(is_bp, is_muted, vol_raw);

        if channels == 2 {
            // High-Performance Batch Pop: drains hardware buffer in 1 atomic operation
            let (popped, _) = consumer.pop_partial_slice(data);
            let popped_len = popped.len();
            if popped_len < data.len() {
                data[popped_len..].fill(0.0);
            }
            if vol <= 0.0001 || !dsp.is_chain_complete() {
                data.fill(0.0);
                for _ in 0..(data.len() / 2) {
                    visualizer.push_sample(0.0);
                }
                return;
            }
            if !is_bp {
                dsp.process_interleaved(data);
                visualizer.set_gain_reduction(dsp.get_max_gain_reduction());
            } else {
                visualizer.set_gain_reduction(0.0);
            }
            if (vol - 1.0).abs() > 0.0001 {
                for s in data.iter_mut() {
                    *s *= vol;
                }
            }
            for frame in data.chunks_exact(2) {
                visualizer.push_stereo(frame[0], frame[1]);
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    *out = 0.0;
                    visualizer.push_sample(0.0);
                } else {
                    let s = ((l + r) * 0.5) * vol;
                    *out = s;
                    visualizer.push_stereo(s, s);
                }
            }
        } else {
            for frame in data.chunks_mut(channels) {
                let l = consumer.pop().unwrap_or(0.0);
                let r = if frame.len() > 1 { consumer.pop().unwrap_or(0.0) } else { l };
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    frame.fill(0.0);
                    visualizer.push_sample(0.0);
                } else {
                    frame[0] = l * vol;
                    if frame.len() > 1 {
                        frame[1] = r * vol;
                    }
                    for ch in 2..frame.len() {
                        frame[ch] = 0.0;
                    }
                    visualizer.push_stereo(l * vol, r * vol);
                }
            }
        }
    }

    // 32-bit Integer PCM Rendering (24-bit/32-bit Audiophile DACs)
    fn render_samples_i32(
        data: &mut [i32],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
        flush_requested: &AtomicBool,
        flush_ack: &AtomicBool,
        bit_perfect: &AtomicBool,
        muted: &AtomicBool,
        volume: &AtomicU32,
        visualizer: &VisualizerRingBuffer,
        source_bit_depth: u8,
        dsp: &mut DspPipeline,
        dsp_rx: &Receiver<DspCommand>,
    ) {
        Self::apply_dsp_commands(dsp, dsp_rx);

        if flush_requested.swap(false, Ordering::SeqCst) {
            while consumer.pop().is_ok() {}
            data.fill(0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            flush_ack.store(true, Ordering::SeqCst);
            return;
        }

        if is_paused.load(Ordering::Relaxed) || !dsp.is_chain_complete() {
            data.fill(0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let is_bp = bit_perfect.load(Ordering::Relaxed);
        let is_muted = muted.load(Ordering::Relaxed);
        let vol_raw = f32::from_bits(volume.load(Ordering::Relaxed));
        let vol = crate::audio::types::calculate_effective_volume(is_bp, is_muted, vol_raw);

        let to_i32 = |s: f32| -> i32 {
            crate::audio::types::encode_sample_i32(s, source_bit_depth)
        };

        if channels == 2 {
            let mut temp_buf = [0.0f32; 512];
            for frame_chunk in data.chunks_mut(512) {
                let needed = frame_chunk.len();
                let (popped, _) = consumer.pop_partial_slice(&mut temp_buf[..needed]);
                let popped_len = popped.len();
                if popped_len < needed {
                    temp_buf[popped_len..needed].fill(0.0);
                }
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    frame_chunk.fill(0);
                    for _ in 0..(needed / 2) {
                        visualizer.push_sample(0.0);
                    }
                    continue;
                }
                if !is_bp {
                    dsp.process_interleaved(&mut temp_buf[..needed]);
                    visualizer.set_gain_reduction(dsp.get_max_gain_reduction());
                } else {
                    visualizer.set_gain_reduction(0.0);
                }
                for (dst, &src) in frame_chunk.iter_mut().zip(&temp_buf[..needed]) {
                    *dst = to_i32(src * vol);
                }
                for pair in temp_buf[..needed].chunks_exact(2) {
                    visualizer.push_stereo(pair[0] * vol, pair[1] * vol);
                }
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    *out = 0;
                    visualizer.push_sample(0.0);
                } else {
                    let s = ((l + r) * 0.5) * vol;
                    *out = to_i32(s);
                    visualizer.push_stereo(s, s);
                }
            }
        } else {
            for frame in data.chunks_mut(channels) {
                let l = consumer.pop().unwrap_or(0.0);
                let r = if frame.len() > 1 { consumer.pop().unwrap_or(0.0) } else { l };
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    frame.fill(0);
                    visualizer.push_sample(0.0);
                } else {
                    frame[0] = to_i32(l * vol);
                    if frame.len() > 1 {
                        frame[1] = to_i32(r * vol);
                    }
                    for ch in 2..frame.len() {
                        frame[ch] = 0;
                    }
                    visualizer.push_stereo(l * vol, r * vol);
                }
            }
        }
    }

    // 16-bit Integer PCM Rendering with Precision Rounding & Conditional TPDF Dither
    fn render_samples_i16(
        data: &mut [i16],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
        flush_requested: &AtomicBool,
        flush_ack: &AtomicBool,
        bit_perfect: &AtomicBool,
        muted: &AtomicBool,
        volume: &AtomicU32,
        visualizer: &VisualizerRingBuffer,
        source_bit_depth: u8,
        dither_state: &mut u32,
        dsp: &mut DspPipeline,
        dsp_rx: &Receiver<DspCommand>,
    ) {
        Self::apply_dsp_commands(dsp, dsp_rx);
        if flush_requested.swap(false, Ordering::SeqCst) {
            while consumer.pop().is_ok() {}
            data.fill(0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            flush_ack.store(true, Ordering::SeqCst);
            return;
        }

        if is_paused.load(Ordering::Relaxed) || !dsp.is_chain_complete() {
            data.fill(0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let is_bp = bit_perfect.load(Ordering::Relaxed);
        let is_muted = muted.load(Ordering::Relaxed);
        let vol_raw = f32::from_bits(volume.load(Ordering::Relaxed));
        let vol = crate::audio::types::calculate_effective_volume(is_bp, is_muted, vol_raw);

        // Apply TPDF dither ONLY when converting higher bit depth (e.g. 24-bit) down to 16-bit
        // AND NEVER in Bit-Perfect mode (Bit-Perfect keeps pure unmodified bits)
        let should_dither = source_bit_depth > 16 && !is_bp;

        let to_i16 = |s: f32, state: &mut u32| -> i16 {
            crate::audio::types::encode_sample_i16(s, should_dither, state)
        };

        if channels == 2 {
            let mut temp_buf = [0.0f32; 512];
            for frame_chunk in data.chunks_mut(512) {
                let needed = frame_chunk.len();
                let (popped, _) = consumer.pop_partial_slice(&mut temp_buf[..needed]);
                let popped_len = popped.len();
                if popped_len < needed {
                    temp_buf[popped_len..needed].fill(0.0);
                }
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    frame_chunk.fill(0);
                    for _ in 0..(needed / 2) {
                        visualizer.push_sample(0.0);
                    }
                    continue;
                }
                if !is_bp {
                    dsp.process_interleaved(&mut temp_buf[..needed]);
                    visualizer.set_gain_reduction(dsp.get_max_gain_reduction());
                } else {
                    visualizer.set_gain_reduction(0.0);
                }
                for (dst, &src) in frame_chunk.iter_mut().zip(&temp_buf[..needed]) {
                    *dst = to_i16(src * vol, dither_state);
                }
                for pair in temp_buf[..needed].chunks_exact(2) {
                    visualizer.push_stereo(pair[0] * vol, pair[1] * vol);
                }
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    *out = 0;
                    visualizer.push_sample(0.0);
                } else {
                    let s = ((l + r) * 0.5) * vol;
                    *out = to_i16(s, dither_state);
                    visualizer.push_stereo(s, s);
                }
            }
        } else {
            for frame in data.chunks_mut(channels) {
                let l = consumer.pop().unwrap_or(0.0);
                let r = if frame.len() > 1 { consumer.pop().unwrap_or(0.0) } else { l };
                if vol <= 0.0001 || !dsp.is_chain_complete() {
                    frame.fill(0);
                    visualizer.push_sample(0.0);
                } else {
                    frame[0] = to_i16(l * vol, dither_state);
                    if frame.len() > 1 {
                        frame[1] = to_i16(r * vol, dither_state);
                    }
                    for ch in 2..frame.len() {
                        frame[ch] = 0;
                    }
                    visualizer.push_stereo(l * vol, r * vol);
                }
            }
        }
    }

    // 16-bit Unsigned PCM Rendering
    fn render_samples_u16(
        data: &mut [u16],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
        flush_requested: &AtomicBool,
        flush_ack: &AtomicBool,
        bit_perfect: &AtomicBool,
        muted: &AtomicBool,
        volume: &AtomicU32,
        visualizer: &VisualizerRingBuffer,
    ) {
        if flush_requested.swap(false, Ordering::SeqCst) {
            while consumer.pop().is_ok() {}
            data.fill(32768);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            flush_ack.store(true, Ordering::SeqCst);
            return;
        }

        if is_paused.load(Ordering::Relaxed) {
            data.fill(32768);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let is_bp = bit_perfect.load(Ordering::Relaxed);
        let is_muted = muted.load(Ordering::Relaxed);
        let vol_raw = f32::from_bits(volume.load(Ordering::Relaxed));
        let vol = crate::audio::types::calculate_effective_volume(is_bp, is_muted, vol_raw);

        let to_u16 = |s: f32| -> u16 {
            let clamped = (s * 0.5 + 0.5).clamp(0.0, 1.0);
            (clamped * 65535.0).round() as u16
        };

        if channels == 2 {
            for frame in data.chunks_exact_mut(2) {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                if vol <= 0.0001 {
                    frame[0] = 32768;
                    frame[1] = 32768;
                    visualizer.push_sample(0.0);
                } else {
                    frame[0] = to_u16(l * vol);
                    frame[1] = to_u16(r * vol);
                    visualizer.push_sample((l + r) * 0.5 * vol);
                }
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                if vol <= 0.0001 {
                    *out = 32768;
                    visualizer.push_sample(0.0);
                } else {
                    let s = ((l + r) * 0.5) * vol;
                    *out = to_u16(s);
                    visualizer.push_sample(s);
                }
            }
        } else {
            for frame in data.chunks_mut(channels) {
                let l = consumer.pop().unwrap_or(0.0);
                let r = if frame.len() > 1 { consumer.pop().unwrap_or(0.0) } else { l };
                if vol <= 0.0001 {
                    frame.fill(32768);
                    visualizer.push_sample(0.0);
                } else {
                    frame[0] = to_u16(l * vol);
                    if frame.len() > 1 {
                        frame[1] = to_u16(r * vol);
                    }
                    for ch in 2..frame.len() {
                        frame[ch] = 32768;
                    }
                    visualizer.push_sample((l + r) * 0.5 * vol);
                }
            }
        }
    }
}
