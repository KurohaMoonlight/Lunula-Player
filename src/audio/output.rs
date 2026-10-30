use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, SampleFormat, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AudioOutput {
    _stream: Stream,
    sample_rate: u32,
    channels: u16,
    device_name: String,
    is_paused: Arc<AtomicBool>,
}

impl AudioOutput {
    // Create audio output stream connected to a lock-free RingBuffer.
    // buffer_capacity: total samples in buffer (e.g. 96000 samples = ~1 sec stereo at 48kHz).
    pub fn new(buffer_capacity: usize) -> Result<(Self, Producer<f32>), Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "Tidak ditemukan audio output device pada sistem ini")?;

        let device_name = device.name().unwrap_or_else(|_| "Default Audio Device".to_string());

        let supported_config = device.default_output_config()?;
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();

        let sample_rate = config.sample_rate.0;
        let channels = config.channels;

        let (producer, mut consumer) = RingBuffer::<f32>::new(buffer_capacity);
        let is_paused = Arc::new(AtomicBool::new(false));
        let is_paused_cb = Arc::clone(&is_paused);

        let err_fn = |err| eprintln!("Audio stream error: {}", err);
        let channels_count = channels as usize;

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    Self::render_samples_f32(data, channels_count, &mut consumer, &is_paused_cb);
                },
                err_fn,
                None,
            )?,
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _: &OutputCallbackInfo| {
                    Self::render_samples_i16(data, channels_count, &mut consumer, &is_paused_cb);
                },
                err_fn,
                None,
            )?,
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _: &OutputCallbackInfo| {
                    Self::render_samples_u16(data, channels_count, &mut consumer, &is_paused_cb);
                },
                err_fn,
                None,
            )?,
            _ => return Err("Format sample audio tidak didukung oleh perangkat ini".into()),
        };

        stream.play()?;

        Ok((
            Self {
                _stream: stream,
                sample_rate,
                channels,
                device_name,
                is_paused,
            },
            producer,
        ))
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

    // Direct F32 hardware rendering
    fn render_samples_f32(
        data: &mut [f32],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            data.fill(0.0);
            return;
        }

        if channels == 2 {
            for out in data.iter_mut() {
                *out = consumer.pop().unwrap_or(0.0);
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                *out = (l + r) * 0.5;
            }
        } else {
            for frame in data.chunks_mut(channels) {
                frame[0] = consumer.pop().unwrap_or(0.0);
                if frame.len() > 1 {
                    frame[1] = consumer.pop().unwrap_or(0.0);
                }
                for ch in 2..frame.len() {
                    frame[ch] = 0.0;
                }
            }
        }
    }

    // 16-bit Integer hardware conversion
    fn render_samples_i16(
        data: &mut [i16],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            data.fill(0);
            return;
        }

        let to_i16 = |s: f32| -> i16 {
            (s.clamp(-1.0, 1.0) * 32767.0) as i16
        };

        if channels == 2 {
            for out in data.iter_mut() {
                *out = to_i16(consumer.pop().unwrap_or(0.0));
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                *out = to_i16((l + r) * 0.5);
            }
        } else {
            for frame in data.chunks_mut(channels) {
                frame[0] = to_i16(consumer.pop().unwrap_or(0.0));
                if frame.len() > 1 {
                    frame[1] = to_i16(consumer.pop().unwrap_or(0.0));
                }
                for ch in 2..frame.len() {
                    frame[ch] = 0;
                }
            }
        }
    }

    // 16-bit Unsigned hardware conversion
    fn render_samples_u16(
        data: &mut [u16],
        channels: usize,
        consumer: &mut Consumer<f32>,
        is_paused: &AtomicBool,
    ) {
        if is_paused.load(Ordering::Relaxed) {
            data.fill(32768);
            return;
        }

        let to_u16 = |s: f32| -> u16 {
            let clamped = (s * 0.5 + 0.5).clamp(0.0, 1.0);
            (clamped * 65535.0) as u16
        };

        if channels == 2 {
            for out in data.iter_mut() {
                *out = to_u16(consumer.pop().unwrap_or(0.0));
            }
        } else if channels == 1 {
            for out in data.iter_mut() {
                let l = consumer.pop().unwrap_or(0.0);
                let r = consumer.pop().unwrap_or(0.0);
                *out = to_u16((l + r) * 0.5);
            }
        } else {
            for frame in data.chunks_mut(channels) {
                frame[0] = to_u16(consumer.pop().unwrap_or(0.0));
                if frame.len() > 1 {
                    frame[1] = to_u16(consumer.pop().unwrap_or(0.0));
                }
                for ch in 2..frame.len() {
                    frame[ch] = 32768;
                }
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
