#[cfg(target_os = "windows")]
use wasapi::*;
use rtrb::{Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::OutputBackend;
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
    format_desc: String,
    device_name: String,
    is_paused: Arc<AtomicBool>,
    flush_requested: Arc<AtomicBool>,
    stop_signal: Arc<AtomicBool>,
    shared_volume: Arc<AtomicU32>,
    worker_handle: Option<JoinHandle<()>>,
}

impl OutputBackend for WasapiExclusiveBackend {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
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

    fn flush(&self) {
        self.flush_requested.store(true, Ordering::SeqCst);
    }
}

impl Drop for WasapiExclusiveBackend {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
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
        let candidate_formats = if preferred_bit_depth > 16 {
            vec![
                (32, 24, SampleType::Int, "24-bit in 32-bit PCM (WASAPI Exclusive)"),
                (32, 32, SampleType::Int, "32-bit Integer PCM (WASAPI Exclusive)"),
                (32, 32, SampleType::Float, "32-bit Float (WASAPI Exclusive)"),
                (16, 16, SampleType::Int, "16-bit Integer PCM (WASAPI Exclusive)"),
            ]
        } else {
            vec![
                (16, 16, SampleType::Int, "16-bit Integer PCM (WASAPI Exclusive)"),
                (32, 24, SampleType::Int, "24-bit in 32-bit PCM (WASAPI Exclusive)"),
                (32, 32, SampleType::Int, "32-bit Integer PCM (WASAPI Exclusive)"),
                (32, 32, SampleType::Float, "32-bit Float (WASAPI Exclusive)"),
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
        let render_client = client.get_audiorenderclient()?;
        let is_paused = Arc::new(AtomicBool::new(false));
        let flush_requested = Arc::new(AtomicBool::new(false));
        let stop_signal = Arc::new(AtomicBool::new(false));

        let is_paused_worker = Arc::clone(&is_paused);
        let flush_worker = Arc::clone(&flush_requested);
        let stop_worker = Arc::clone(&stop_signal);
        let bp_worker = Arc::clone(&bit_perfect);
        let vol_worker = Arc::clone(&shared_volume);
        let vis_worker = Arc::clone(&visualizer);

        let (producer, mut consumer) = RingBuffer::<f32>::new(buffer_capacity);

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
                let mut temp_l;
                let mut temp_r;

                while !stop_worker.load(Ordering::Relaxed) {
                    if let Ok(()) = h_event.wait_for_event(100) {
                        if stop_worker.load(Ordering::Relaxed) {
                            break;
                        }

                        if flush_worker.swap(false, Ordering::SeqCst) {
                            while consumer.pop().is_ok() {}
                        }

                        if is_paused_worker.load(Ordering::Relaxed) {
                            continue;
                        }

                        let padding = client.get_current_padding().unwrap_or(0);
                        let buffer_frame_count = client.get_buffer_size().unwrap_or(0);
                        let available_frames = buffer_frame_count.saturating_sub(padding);

                    if available_frames > 0 {
                        let frames_to_write = available_frames.min(512);
                        let is_bp = bp_worker.load(Ordering::Relaxed);
                        let vol = if is_bp {
                            1.0
                        } else {
                            f32::from_bits(vol_worker.load(Ordering::Relaxed))
                        };

                        if store_bits == 16 {
                            let mut raw_bytes = Vec::with_capacity(frames_to_write as usize * 4);
                            for _ in 0..frames_to_write {
                                temp_l = consumer.pop().unwrap_or(0.0) * vol;
                                temp_r = consumer.pop().unwrap_or(0.0) * vol;
                                vis_worker.push_sample((temp_l + temp_r) * 0.5);

                                let i_l = (temp_l * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
                                let i_r = (temp_r * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
                                raw_bytes.extend_from_slice(&i_l.to_le_bytes());
                                raw_bytes.extend_from_slice(&i_r.to_le_bytes());
                            }
                            let _ = render_client.write_to_device(frames_to_write as usize, &raw_bytes, None);
                        } else if store_bits == 32 && sample_type == SampleType::Int {
                            let mut raw_bytes = Vec::with_capacity(frames_to_write as usize * 8);
                            for _ in 0..frames_to_write {
                                temp_l = consumer.pop().unwrap_or(0.0) * vol;
                                temp_r = consumer.pop().unwrap_or(0.0) * vol;
                                vis_worker.push_sample((temp_l + temp_r) * 0.5);

                                let (scaled_l, scaled_r) = if valid_bits == 24 {
                                    let l = ((temp_l as f64 * 8388607.0).round().clamp(-8388608.0, 8388607.0) as i32) << 8;
                                    let r = ((temp_r as f64 * 8388607.0).round().clamp(-8388608.0, 8388607.0) as i32) << 8;
                                    (l, r)
                                } else {
                                    let l = (temp_l as f64 * 2147483647.0).round().clamp(-2147483648.0, 2147483647.0) as i32;
                                    let r = (temp_r as f64 * 2147483647.0).round().clamp(-2147483648.0, 2147483647.0) as i32;
                                    (l, r)
                                };
                                raw_bytes.extend_from_slice(&scaled_l.to_le_bytes());
                                raw_bytes.extend_from_slice(&scaled_r.to_le_bytes());
                            }
                            let _ = render_client.write_to_device(frames_to_write as usize, &raw_bytes, None);
                        } else {
                            // 32-bit Float
                            let mut raw_bytes = Vec::with_capacity(frames_to_write as usize * 8);
                            for _ in 0..frames_to_write {
                                temp_l = consumer.pop().unwrap_or(0.0) * vol;
                                temp_r = consumer.pop().unwrap_or(0.0) * vol;
                                vis_worker.push_sample((temp_l + temp_r) * 0.5);

                                raw_bytes.extend_from_slice(&temp_l.to_le_bytes());
                                raw_bytes.extend_from_slice(&temp_r.to_le_bytes());
                            }
                            let _ = render_client.write_to_device(frames_to_write as usize, &raw_bytes, None);
                        }
                        } else {
                            thread::sleep(Duration::from_millis(2));
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
                format_desc: format_desc.to_string(),
                device_name,
                is_paused,
                flush_requested,
                stop_signal,
                shared_volume,
                worker_handle: Some(worker_handle),
            },
            producer,
        ))
    }
}
