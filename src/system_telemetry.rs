use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// High-capacity audiophile lock-free SPSC Ring Buffer size in stereo `f64` samples:
/// 786,432 samples = 393,216 stereo frames = 768 chunks of 512 frames.
/// In 64-bit float (`f64`, 8 bytes/sample), this pre-allocates exactly 6.00 MB (6,144.0 KB / 6,291,456 bytes),
/// providing >2.04 seconds of zero-jitter DMA headroom at 192 kHz stereo and ~8.9 seconds at 44.1 kHz stereo.
pub const SPSC_RING_BUFFER_CAPACITY: usize = 786_432;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessMemoryStats {
    pub working_set_bytes: usize,
    pub peak_working_set_bytes: usize,
    pub private_usage_bytes: usize,
    pub pagefile_usage_bytes: usize,
}

impl ProcessMemoryStats {
    pub fn working_set_mb(&self) -> f32 {
        self.working_set_bytes as f32 / (1024.0 * 1024.0)
    }

    pub fn peak_working_set_mb(&self) -> f32 {
        self.peak_working_set_bytes as f32 / (1024.0 * 1024.0)
    }

    pub fn private_usage_mb(&self) -> f32 {
        self.private_usage_bytes as f32 / (1024.0 * 1024.0)
    }
}

/// Atomic structure shared across Audio Worker and UI to track real-time lock-free ring buffer telemetry
#[derive(Debug)]
pub struct BufferMetricsAtomic {
    pub capacity_samples: AtomicU32,
    pub used_samples: AtomicU32,
    pub chunk_counter: AtomicU64,
    pub total_samples_streamed: AtomicU64,
    pub last_chunk_frames: AtomicU32,
    pub underrun_counter: AtomicU32,
}

impl Default for BufferMetricsAtomic {
    fn default() -> Self {
        Self {
            capacity_samples: AtomicU32::new(SPSC_RING_BUFFER_CAPACITY as u32),
            used_samples: AtomicU32::new(0),
            chunk_counter: AtomicU64::new(0),
            total_samples_streamed: AtomicU64::new(0),
            last_chunk_frames: AtomicU32::new(512),
            underrun_counter: AtomicU32::new(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BufferTelemetrySnapshot {
    pub capacity_samples: usize,
    pub capacity_kb: f32,
    pub capacity_mb: f32,
    pub used_samples: usize,
    pub fill_pct: f32,
    pub latency_ms: f32,
    pub chunk_size_frames: usize,
    pub chunk_size_kb: f32,
    pub chunk_counter: u64,
    pub total_streamed_mb: f32,
    pub is_underrun: bool,
}

pub struct SystemTelemetrySampler {
    last_sample_instant: Instant,
    last_cpu_time_100ns: u64,
    cached_cpu_pct: f32,
    cached_mem_stats: ProcessMemoryStats,
    num_cpus: usize,
    pub buffer_metrics: Arc<BufferMetricsAtomic>,
}

impl Default for SystemTelemetrySampler {
    fn default() -> Self {
        let num_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let mut s = Self {
            last_sample_instant: Instant::now(),
            last_cpu_time_100ns: 0,
            cached_cpu_pct: 0.0,
            cached_mem_stats: ProcessMemoryStats::default(),
            num_cpus,
            buffer_metrics: Arc::new(BufferMetricsAtomic::default()),
        };
        s.init_sample();
        s
    }
}

impl SystemTelemetrySampler {
    fn init_sample(&mut self) {
        #[cfg(windows)]
        {
            if let Some(t) = query_windows_process_times() {
                self.last_cpu_time_100ns = t;
            }
            if let Some(m) = query_windows_process_memory() {
                self.cached_mem_stats = m;
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Some(t) = query_linux_process_times_100ns() {
                self.last_cpu_time_100ns = t;
            }
            if let Some(m) = query_linux_process_memory() {
                self.cached_mem_stats = m;
            }
        }
    }

    /// Samples real CPU and RAM metrics with minimal overhead (polled at most once every 150ms).
    pub fn sample(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sample_instant).as_secs_f64();
        if elapsed < 0.15 {
            return;
        }

        #[cfg(windows)]
        {
            if let Some(current_cpu_100ns) = query_windows_process_times() {
                if self.last_cpu_time_100ns > 0 && current_cpu_100ns >= self.last_cpu_time_100ns {
                    let delta_cpu_100ns = current_cpu_100ns - self.last_cpu_time_100ns;
                    let delta_cpu_secs = delta_cpu_100ns as f64 / 10_000_000.0;
                    let pct = (delta_cpu_secs / (elapsed * self.num_cpus as f64)) * 100.0;
                    self.cached_cpu_pct = (pct as f32).clamp(0.0, 100.0);
                }
                self.last_cpu_time_100ns = current_cpu_100ns;
            }

            if let Some(m) = query_windows_process_memory() {
                self.cached_mem_stats = m;
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(current_cpu_100ns) = query_linux_process_times_100ns() {
                if self.last_cpu_time_100ns > 0 && current_cpu_100ns >= self.last_cpu_time_100ns {
                    let delta_cpu_100ns = current_cpu_100ns - self.last_cpu_time_100ns;
                    let delta_cpu_secs = delta_cpu_100ns as f64 / 10_000_000.0;
                    let pct = (delta_cpu_secs / (elapsed * self.num_cpus as f64)) * 100.0;
                    self.cached_cpu_pct = (pct as f32).clamp(0.0, 100.0);
                }
                self.last_cpu_time_100ns = current_cpu_100ns;
            }

            if let Some(m) = query_linux_process_memory() {
                self.cached_mem_stats = m;
            }
        }

        self.last_sample_instant = now;
    }

    pub fn cpu_usage_pct(&self) -> f32 {
        self.cached_cpu_pct
    }

    pub fn memory_stats(&self) -> ProcessMemoryStats {
        self.cached_mem_stats
    }

    pub fn buffer_snapshot(&self, sample_rate: u32) -> BufferTelemetrySnapshot {
        let cap = self.buffer_metrics.capacity_samples.load(Ordering::Relaxed) as usize;
        let used = self.buffer_metrics.used_samples.load(Ordering::Relaxed) as usize;
        let chunk_count = self.buffer_metrics.chunk_counter.load(Ordering::Relaxed);
        let total_samples = self.buffer_metrics.total_samples_streamed.load(Ordering::Relaxed);
        let chunk_frames = self.buffer_metrics.last_chunk_frames.load(Ordering::Relaxed) as usize;
        let underruns = self.buffer_metrics.underrun_counter.load(Ordering::Relaxed);

        let fill_pct = if cap > 0 {
            ((used as f32 / cap as f32) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let sample_bytes = std::mem::size_of::<f64>();
        let capacity_kb = (cap * sample_bytes) as f32 / 1024.0;
        let capacity_mb = (cap * sample_bytes) as f32 / (1024.0 * 1024.0);

        let rate = sample_rate.max(1) as f32;
        let latency_ms = ((used as f32 / 2.0) / rate) * 1000.0;
        let chunk_size_kb = (chunk_frames * 2 * sample_bytes) as f32 / 1024.0;
        let total_streamed_mb = (total_samples * sample_bytes as u64) as f32 / (1024.0 * 1024.0);

        BufferTelemetrySnapshot {
            capacity_samples: cap,
            capacity_kb,
            capacity_mb,
            used_samples: used,
            fill_pct,
            latency_ms,
            chunk_size_frames: chunk_frames,
            chunk_size_kb,
            chunk_counter: chunk_count,
            total_streamed_mb,
            is_underrun: underruns > 0,
        }
    }
}

#[cfg(target_os = "linux")]
fn query_linux_process_times_100ns() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/self/stat").ok()?;
    let end_comm = content.rfind(')')?;
    let fields: Vec<&str> = content[end_comm + 1..].split_whitespace().collect();
    // In /proc/self/stat after ')': index 11 is utime, index 12 is stime (in clock ticks, typically 100 Hz = 10ms)
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    let total_ticks = utime.saturating_add(stime);
    // 1 tick at 100 Hz = 10 ms = 100,000 units of 100ns
    Some(total_ticks.saturating_mul(100_000))
}

#[cfg(target_os = "linux")]
fn query_linux_process_memory() -> Option<ProcessMemoryStats> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let mut vm_rss_kb: usize = 0;
    let mut vm_hwm_kb: usize = 0;
    let mut rss_anon_kb: usize = 0;
    let mut vm_data_kb: usize = 0;

    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            vm_rss_kb = rest.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("VmHWM:") {
            vm_hwm_kb = rest.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("RssAnon:") {
            rss_anon_kb = rest.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("VmData:") {
            vm_data_kb = rest.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0);
        }
    }

    let private_ws_kb = if rss_anon_kb > 0 { rss_anon_kb } else { vm_rss_kb };
    let peak_kb = vm_hwm_kb.max(private_ws_kb);
    let commit_kb = if vm_data_kb > 0 { vm_data_kb } else { vm_rss_kb };

    Some(ProcessMemoryStats {
        working_set_bytes: private_ws_kb * 1024,
        peak_working_set_bytes: peak_kb * 1024,
        private_usage_bytes: commit_kb * 1024,
        pagefile_usage_bytes: commit_kb * 1024,
    })
}

#[cfg(windows)]
fn query_windows_process_times() -> Option<u64> {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

    unsafe {
        let handle = GetCurrentProcess();
        let mut creation: FILETIME = std::mem::zeroed();
        let mut exit: FILETIME = std::mem::zeroed();
        let mut kernel: FILETIME = std::mem::zeroed();
        let mut user: FILETIME = std::mem::zeroed();

        if GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) != 0 {
            let kernel_100ns = ((kernel.dwHighDateTime as u64) << 32) | (kernel.dwLowDateTime as u64);
            let user_100ns = ((user.dwHighDateTime as u64) << 32) | (user.dwLowDateTime as u64);
            Some(kernel_100ns + user_100ns)
        } else {
            None
        }
    }
}

#[cfg(windows)]
fn query_windows_process_memory() -> Option<ProcessMemoryStats> {
    use std::mem::size_of;
    use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    unsafe {
        let handle = GetCurrentProcess();
        let mut pmc: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        let cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;

        if GetProcessMemoryInfo(handle, &mut pmc as *mut _ as *mut _, cb) != 0 {
            let private_ws = query_exact_private_working_set(handle)
                .unwrap_or(pmc.PrivateUsage.min(pmc.WorkingSetSize));

            Some(ProcessMemoryStats {
                working_set_bytes: private_ws,
                peak_working_set_bytes: pmc.PeakWorkingSetSize,
                private_usage_bytes: pmc.PrivateUsage,
                pagefile_usage_bytes: pmc.PagefileUsage,
            })
        } else {
            None
        }
    }
}

#[cfg(windows)]
fn query_exact_private_working_set(handle: windows_sys::Win32::Foundation::HANDLE) -> Option<usize> {
    use windows_sys::Win32::System::ProcessStatus::QueryWorkingSet;
    use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

    unsafe {
        let mut sys_info: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut sys_info);
        let page_size = sys_info.dwPageSize as usize;

        let num_entries: usize = 65536;
        let mut buffer: Vec<usize> = vec![0; num_entries];

        if QueryWorkingSet(
            handle,
            buffer.as_mut_ptr() as *mut _,
            (num_entries * std::mem::size_of::<usize>()) as u32,
        ) != 0
        {
            let count = buffer[0];
            let mut private_pages = 0;
            for i in 1..=count.min(num_entries - 1) {
                let entry = buffer[i];
                // Bit 8 pada entri Working Set Windows: 0 = Private, 1 = Shared
                let is_shared = (entry >> 8) & 1 == 1;
                if !is_shared {
                    private_pages += 1;
                }
            }
            Some(private_pages * page_size)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_telemetry_sampler_creation_and_bounds() {
        let mut sampler = SystemTelemetrySampler::default();
        sampler.sample();

        #[cfg(any(windows, target_os = "linux"))]
        {
            let mem = sampler.memory_stats();
            // A running Rust process should have > 0.5 MB of working set
            assert!(mem.working_set_mb() > 0.5, "Working set should be realistic, got: {}", mem.working_set_mb());
            assert!(mem.peak_working_set_mb() >= mem.working_set_mb());
        }

        let snap = sampler.buffer_snapshot(44100);
        assert_eq!(snap.capacity_samples, SPSC_RING_BUFFER_CAPACITY);
        assert!((snap.capacity_mb - 6.0).abs() < 1e-3);
        assert!((snap.capacity_kb - 6144.0).abs() < 1e-3);
        assert!((snap.chunk_size_kb - 8.0).abs() < 1e-3);
        assert!(snap.fill_pct >= 0.0 && snap.fill_pct <= 100.0);
        assert!(!snap.is_underrun, "Idle buffer with 0 underruns must not report false underrun");
    }
}

/// Menginstruksikan Windows Memory Manager untuk memangkas halaman RAM yang sudah tidak aktif (working set trim)
pub fn trim_process_working_set() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::ProcessStatus::EmptyWorkingSet;
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        let handle = GetCurrentProcess();
        EmptyWorkingSet(handle);
    }
}