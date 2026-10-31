pub mod cpal_backend;
#[cfg(target_os = "windows")]
pub mod wasapi_exclusive;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

pub trait OutputBackend: Send {
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn format_desc(&self) -> String;
    fn device_name(&self) -> &str;
    fn is_exclusive(&self) -> bool;
    fn pause(&self);
    fn resume(&self);
    fn is_paused(&self) -> bool;
    fn set_volume(&self, vol: f32);
    fn flush(&self) -> bool;
}

pub fn enumerate_output_devices() -> Vec<OutputDeviceInfo> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let default_name = host.default_output_device().and_then(|d| d.name().ok());

    let mut list = Vec::new();
    if let Ok(devices) = host.output_devices() {
        for d in devices {
            if let Ok(name) = d.name() {
                let is_default = default_name.as_ref().map_or(false, |def| def == &name);
                list.push(OutputDeviceInfo { name, is_default });
            }
        }
    }
    list.sort_by(|a, b| b.is_default.cmp(&a.is_default));
    list.dedup_by(|a, b| a.name == b.name);
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_devices() {
        let devices = enumerate_output_devices();
        // Should execute cleanly on any host
        for dev in &devices {
            assert!(!dev.name.is_empty());
        }
    }
}

