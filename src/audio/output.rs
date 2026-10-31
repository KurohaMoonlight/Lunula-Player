use crate::audio::backend::{cpal_backend::CpalBackend, OutputBackend};
#[cfg(target_os = "windows")]
use crate::audio::backend::wasapi_exclusive::WasapiExclusiveBackend;
use crate::audio::types::ChainMode;
use crate::audio::visualizer::VisualizerRingBuffer;
use rtrb::Producer;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;

pub struct AudioOutput {
    backend: Box<dyn OutputBackend>,
}

impl AudioOutput {
    // Create audio output stream connected to a lock-free RingBuffer.
    // In Bit-Perfect mode on Windows, attempts native WASAPI Exclusive Mode first,
    // with automatic zero-glitch fallback to CPAL Shared Mode.
    pub fn new(
        preferred_device_name: Option<&str>,
        preferred_sample_rate: u32,
        preferred_bit_depth: u8,
        chain_mode: ChainMode,
        buffer_capacity: usize,
        shared_volume: Arc<AtomicU32>,
        visualizer: Arc<VisualizerRingBuffer>,
        bit_perfect: Arc<AtomicBool>,
        shared_muted: Arc<AtomicBool>,
        is_dsp_active: Arc<AtomicBool>,
    ) -> Result<(Self, Producer<f32>), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "windows")]
        if chain_mode == ChainMode::BitPerfect {
            if let Ok((ex_backend, prod)) = WasapiExclusiveBackend::try_new(
                preferred_device_name,
                preferred_sample_rate,
                preferred_bit_depth,
                buffer_capacity,
                Arc::clone(&shared_volume),
                Arc::clone(&visualizer),
                Arc::clone(&bit_perfect),
                Arc::clone(&shared_muted),
            ) {
                return Ok((
                    Self {
                        backend: Box::new(ex_backend),
                    },
                    prod,
                ));
            }
        }

        // Shared Mode fallback (CPAL)
        let (cpal_be, prod) = CpalBackend::new(
            preferred_device_name,
            preferred_sample_rate,
            buffer_capacity,
            shared_volume,
            visualizer,
            bit_perfect,
            shared_muted,
            preferred_bit_depth,
            is_dsp_active,
        )?;

        Ok((
            Self {
                backend: Box::new(cpal_be),
            },
            prod,
        ))
    }

    pub fn sample_rate(&self) -> u32 {
        self.backend.sample_rate()
    }

    pub fn channels(&self) -> u16 {
        self.backend.channels()
    }

    pub fn format_desc(&self) -> String {
        self.backend.format_desc()
    }

    pub fn device_name(&self) -> &str {
        self.backend.device_name()
    }

    pub fn is_exclusive(&self) -> bool {
        self.backend.is_exclusive()
    }

    pub fn is_paused(&self) -> bool {
        self.backend.is_paused()
    }

    pub fn pause(&self) {
        self.backend.pause();
    }

    pub fn resume(&self) {
        self.backend.resume();
    }

    pub fn set_volume(&self, vol: f32) {
        self.backend.set_volume(vol);
    }

    // Zero-stream atomic ring buffer flush on Seek & track transition.
    // Backend flush synchronizes with the audio callback thread via ACK, ensuring
    // zero discarded initial samples of new tracks without arbitrary sleep.
    // Returns true if acknowledged by the audio callback, false if timed out (50ms).
    pub fn flush(&self) -> bool {
        self.backend.flush()
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
