use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, SampleFormat, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use crate::audio::visualizer::VisualizerRingBuffer;

pub struct AudioOutput {
    _stream: Stream,
    sample_rate: u32,
    channels: u16,
    device_name: String,
    is_paused: Arc<AtomicBool>,
    bit_perfect: Arc<AtomicBool>,
    shared_volume: Arc<AtomicU32>,
    visualizer: Arc<VisualizerRingBuffer>,
}

impl AudioOutput {
    // Create audio output stream connected to a lock-free RingBuffer with real-time zero-latency hardware volume scaling.
    // Tries to match preferred_sample_rate natively for bit-perfect output, with graceful fallback.
    pub fn new(
        preferred_sample_rate: u32,
        buffer_capacity: usize,
        shared_volume: Arc<AtomicU32>,
        visualizer: Arc<VisualizerRingBuffer>,
        bit_perfect: Arc<AtomicBool>,
    ) -> Result<(Self, Producer<f32>), Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "Tidak ditemukan audio output device pada sistem ini")?;

        let device_name = device.name().unwrap_or_else(|_| "Default Audio Device".to_string());
        let default_supported = device.default_output_config()?;

        // Attempt 1: Check if device directly supports preferred_sample_rate in stereo
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

        // If matched_config found, attempt building stream with it
        if let Some((config, format)) = matched_config {
            let (producer, consumer) = RingBuffer::<f32>::new(buffer_capacity);
            if let Ok(stream) = Self::build_stream(
                &device,
                &config,
                format,
                consumer,
                Arc::clone(&is_paused),
                Arc::clone(&bit_perfect),
                Arc::clone(&shared_volume),
                Arc::clone(&visualizer),
            ) {
                let _ = stream.play();
                let sample_rate = config.sample_rate.0;
                let channels = config.channels;
                return Ok((
                    Self {
                        _stream: stream,
                        sample_rate,
                        channels,
                        device_name,
                        is_paused,
                        bit_perfect,
                        shared_volume,
                        visualizer,
                    },
                    producer,
                ));
            }
        }

        // Fallback: Default output config from device/OS
        let (producer, consumer) = RingBuffer::<f32>::new(buffer_capacity);
        let config: StreamConfig = default_supported.config();
        let format = default_supported.sample_format();
        let stream = Self::build_stream(
            &device,
            &config,
            format,
            consumer,
            Arc::clone(&is_paused),
            Arc::clone(&bit_perfect),
            Arc::clone(&shared_volume),
            Arc::clone(&visualizer),
        )?;
        stream.play()?;

        let sample_rate = config.sample_rate.0;
        let channels = config.channels;

        Ok((
            Self {
                _stream: stream,
                sample_rate,
                channels,
                device_name,
                is_paused,
                bit_perfect,
                shared_volume,
                visualizer,
            },
            producer,
        ))
    }

    fn build_stream(
        device: &cpal::Device,
        config: &StreamConfig,
        sample_format: SampleFormat,
        mut consumer: Consumer<f32>,
        is_paused: Arc<AtomicBool>,
        bit_perfect: Arc<AtomicBool>,
        shared_volume: Arc<AtomicU32>,
        visualizer: Arc<VisualizerRingBuffer>,
    ) -> Result<Stream, Box<dyn std::error::Error + Send + Sync>> {
        let is_paused_cb = Arc::clone(&is_paused);
        let bp_cb = Arc::clone(&bit_perfect);
        let volume_cb = Arc::clone(&shared_volume);
        let visualizer_cb = Arc::clone(&visualizer);
        let channels_count = config.channels as usize;
        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let stream = match sample_format {
            SampleFormat::F32 => {
                device.build_output_stream(
                    config,
                    move |data: &mut [f32], _: &OutputCallbackInfo| {
                        Self::render_samples_f32(data, channels_count, &mut consumer, &is_paused_cb, &bp_cb, &volume_cb, &visualizer_cb);
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::I16 => {
                let bp_cb2 = Arc::clone(&bit_perfect);
                device.build_output_stream(
                    config,
                    move |data: &mut [i16], _: &OutputCallbackInfo| {
                        Self::render_samples_i16(data, channels_count, &mut consumer, &is_paused_cb, &bp_cb2, &volume_cb, &visualizer_cb);
                    },
                    err_fn,
                    None,
                )?
            }
            SampleFormat::U16 => {
                let bp_cb3 = Arc::clone(&bit_perfect);
                device.build_output_stream(
                    config,
                    move |data: &mut [u16], _: &OutputCallbackInfo| {
                        Self::render_samples_u16(data, channels_count, &mut consumer, &is_paused_cb, &bp_cb3, &volume_cb, &visualizer_cb);
                    },
                    err_fn,
                    None,
                )?
            }
            _ => return Err("Format sample audio tidak didukung oleh perangkat ini".into()),
        };

        Ok(stream)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.is_paused.store(false, Ordering::Relaxed);
    }

    pub fn set_volume(&self, vol: f32) {
        self.shared_volume.store(vol.clamp(0.0, 1.5).to_bits(), Ordering::Relaxed);
    }

    pub fn set_bit_perfect(&self, bp: bool) {
        self.bit_perfect.store(bp, Ordering::Relaxed);
    }

    pub fn visualizer(&self) -> &Arc<VisualizerRingBuffer> {
        &self.visualizer
    }

    // Direct F32 hardware rendering with zero-latency volume scaling and live FFT visualizer feed
    fn render_samples_f32(
        data: &mut [f32],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
        bit_perfect: &AtomicBool,
        volume: &AtomicU32,
        visualizer: &VisualizerRingBuffer,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            data.fill(0.0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let is_bp = bit_perfect.load(Ordering::Relaxed);
        let vol = if is_bp {
            1.0
        } else {
            f32::from_bits(volume.load(Ordering::Relaxed))
        };
        if vol <= 0.0001 && !is_bp {
            data.fill(0.0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        if channels == 2 {
            for frame in data.chunks_exact_mut(2) {
                let l = consumer.pop().unwrap_or(0.0) * vol;
                let r = consumer.pop().unwrap_or(0.0) * vol;
                frame[0] = l;
                frame[1] = r;
                visualizer.push_sample((l + r) * 0.5);
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                let s = ((l + r) * 0.5) * vol;
                *out = s;
                visualizer.push_sample(s);
            }
        } else {
            for frame in data.chunks_mut(channels) {
                let l = consumer.pop().unwrap_or(0.0) * vol;
                let r = if frame.len() > 1 { consumer.pop().unwrap_or(0.0) * vol } else { l };
                frame[0] = l;
                if frame.len() > 1 {
                    frame[1] = r;
                }
                for ch in 2..frame.len() {
                    frame[ch] = 0.0;
                }
                visualizer.push_sample((l + r) * 0.5);
            }
        }
    }

    // 16-bit Integer hardware conversion with zero-latency volume scaling and live FFT visualizer feed
    fn render_samples_i16(
        data: &mut [i16],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
        bit_perfect: &AtomicBool,
        volume: &AtomicU32,
        visualizer: &VisualizerRingBuffer,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            data.fill(0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let is_bp = bit_perfect.load(Ordering::Relaxed);
        let vol = if is_bp {
            1.0
        } else {
            f32::from_bits(volume.load(Ordering::Relaxed))
        };
        if vol <= 0.0001 && !is_bp {
            data.fill(0);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let to_i16 = |s: f32| -> i16 {
            (s.clamp(-1.0, 1.0) * 32767.0) as i16
        };

        if channels == 2 {
            for frame in data.chunks_exact_mut(2) {
                let l = consumer.pop().unwrap_or(0.0) * vol;
                let r = consumer.pop().unwrap_or(0.0) * vol;
                frame[0] = to_i16(l);
                frame[1] = to_i16(r);
                visualizer.push_sample((l + r) * 0.5);
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                let s = ((l + r) * 0.5) * vol;
                *out = to_i16(s);
                visualizer.push_sample(s);
            }
        } else {
            for frame in data.chunks_mut(channels) {
                let l = consumer.pop().unwrap_or(0.0) * vol;
                let r = if frame.len() > 1 { consumer.pop().unwrap_or(0.0) * vol } else { l };
                frame[0] = to_i16(l);
                if frame.len() > 1 {
                    frame[1] = to_i16(r);
                }
                for ch in 2..frame.len() {
                    frame[ch] = 0;
                }
                visualizer.push_sample((l + r) * 0.5);
            }
        }
    }

    // 16-bit Unsigned hardware conversion with zero-latency volume scaling and live FFT visualizer feed
    fn render_samples_u16(
        data: &mut [u16],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
        bit_perfect: &AtomicBool,
        volume: &AtomicU32,
        visualizer: &VisualizerRingBuffer,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            data.fill(32768);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let is_bp = bit_perfect.load(Ordering::Relaxed);
        let vol = if is_bp {
            1.0
        } else {
            f32::from_bits(volume.load(Ordering::Relaxed))
        };
        if vol <= 0.0001 && !is_bp {
            data.fill(32768);
            for _ in 0..(data.len() / channels.max(1)) {
                visualizer.push_sample(0.0);
            }
            return;
        }

        let to_u16 = |s: f32| -> u16 {
            let clamped = (s * 0.5 + 0.5).clamp(0.0, 1.0);
            (clamped * 65535.0) as u16
        };

        if channels == 2 {
            for frame in data.chunks_exact_mut(2) {
                let l = consumer.pop().unwrap_or(0.0) * vol;
                let r = consumer.pop().unwrap_or(0.0) * vol;
                frame[0] = to_u16(l);
                frame[1] = to_u16(r);
                visualizer.push_sample((l + r) * 0.5);
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                let s = ((l + r) * 0.5) * vol;
                *out = to_u16(s);
                visualizer.push_sample(s);
            }
        } else {
            for frame in data.chunks_mut(channels) {
                let l = consumer.pop().unwrap_or(0.0) * vol;
                let r = if frame.len() > 1 { consumer.pop().unwrap_or(0.0) * vol } else { l };
                frame[0] = to_u16(l);
                if frame.len() > 1 {
                    frame[1] = to_u16(r);
                }
                for ch in 2..frame.len() {
                    frame[ch] = 32768;
                }
                visualizer.push_sample((l + r) * 0.5);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ring_buffer_creation() {
        let (mut prod, mut cons) = rtrb::RingBuffer::<f32>::new(1024);
        assert_eq!(prod.slots(), 1024);
        assert_eq!(cons.slots(), 0);

        prod.push(0.75).unwrap();
        assert_eq!(cons.slots(), 1);
        assert_eq!(cons.pop().unwrap(), 0.75);
    }
}
