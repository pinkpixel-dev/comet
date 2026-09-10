use super::gpu_sysfs::*;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Other => write!(f, "GPU"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuProcessInfo {
    pub pid: u32,
    pub used_memory: u64,
}

#[derive(Debug, Clone)]
pub struct GpuMetrics {
    pub name: String,
    pub vendor: GpuVendor,
    pub driver: String,
    pub utilization: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub is_shared_memory: bool,
    pub temperature: Option<f32>,
    pub fan_speed: Option<f32>,
    pub power_usage_watts: Option<f32>,
    pub power_limit_watts: Option<f32>,
    pub graphics_clock_mhz: Option<u32>,
    pub memory_clock_mhz: Option<u32>,
    pub processes: Vec<GpuProcessInfo>,
}

enum GpuBackend {
    Nvml {
        device_index: u32,
        fallback_name: String,
    },
    AmdSysfs {
        device_path: PathBuf,
        hwmon_path: Option<PathBuf>,
        name: String,
        driver: String,
    },
    IntelSysfs {
        card_name: String,
        card_path: PathBuf,
        device_path: PathBuf,
        hwmon_path: Option<PathBuf>,
        name: String,
        driver: String,
        last_rc6_ms: Option<u64>,
        last_sample_time: Option<Instant>,
    },
}

pub struct GpuSampler {
    nvml: Option<Nvml>,
    backends: Vec<GpuBackend>,
}

impl Default for GpuSampler {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuSampler {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        let pci_names = query_pci_names();
        let mut backends = Vec::new();
        let mut nvml_has_devices = false;

        // 1. Discover NVIDIA devices via NVML
        if let Some(ref n) = nvml {
            if let Ok(count) = n.device_count() {
                for i in 0..count {
                    let fallback_name = n
                        .device_by_index(i)
                        .and_then(|d| d.name())
                        .unwrap_or_else(|_| "NVIDIA GPU".to_string());
                    backends.push(GpuBackend::Nvml {
                        device_index: i,
                        fallback_name,
                    });
                    nvml_has_devices = true;
                }
            }
        }

        // 2. Discover DRM cards from /sys/class/drm/
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                // Filter only base card nodes (e.g. "card0", "card1") and ignore connectors (e.g. "card1-DP-1")
                if !file_name.starts_with("card") || file_name.contains('-') {
                    continue;
                }

                let card_path = entry.path();
                let device_path = match card_path.join("device").canonicalize() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let vendor_hex = fs::read_to_string(device_path.join("vendor"))
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase();

                let (driver, pci_slot) = parse_uevent(&device_path.join("uevent"));

                // Resolve friendly device name from PCI slot or fallbacks
                let resolved_name = pci_slot
                    .as_deref()
                    .and_then(|slot| pci_names.get(slot).cloned());

                match vendor_hex.as_str() {
                    "0x1002" => {
                        // AMD GPU
                        let hwmon_path = find_device_hwmon(&device_path);
                        let name = resolved_name.unwrap_or_else(|| "AMD Radeon Graphics".to_string());
                        let driver_str = if driver.is_empty() { "amdgpu".to_string() } else { driver };
                        backends.push(GpuBackend::AmdSysfs {
                            device_path,
                            hwmon_path,
                            name,
                            driver: driver_str,
                        });
                    }
                    "0x8086" => {
                        // Intel GPU
                        let hwmon_path = find_device_hwmon(&device_path).or_else(find_cpu_coretemp_hwmon);
                        let name = resolved_name.unwrap_or_else(|| "Intel UHD Graphics".to_string());
                        let driver_str = if driver.is_empty() { "i915".to_string() } else { driver };
                        backends.push(GpuBackend::IntelSysfs {
                            card_name: file_name,
                            card_path,
                            device_path,
                            hwmon_path,
                            name,
                            driver: driver_str,
                            last_rc6_ms: None,
                            last_sample_time: None,
                        });
                    }
                    "0x10de" if !nvml_has_devices => {
                        // NVIDIA GPU: if NVML failed to init, we could note presence, otherwise NVML handles it.
                        let name = resolved_name.unwrap_or_else(|| "NVIDIA Graphics".to_string());
                        let hwmon_path = find_device_hwmon(&device_path);
                        backends.push(GpuBackend::AmdSysfs {
                            device_path,
                            hwmon_path,
                            name,
                            driver: "nouveau".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        Self { nvml, backends }
    }

    pub fn sample_all(&mut self) -> Vec<GpuMetrics> {
        let mut results = Vec::new();

        for backend in &mut self.backends {
            match backend {
                GpuBackend::Nvml {
                    device_index,
                    fallback_name,
                } => {
                    if let Some(metrics) = sample_nvml(self.nvml.as_ref(), *device_index, fallback_name) {
                        results.push(metrics);
                    }
                }
                GpuBackend::AmdSysfs {
                    device_path,
                    hwmon_path,
                    name,
                    driver,
                } => {
                    if let Some(metrics) = sample_amd_sysfs(device_path, hwmon_path.as_ref(), name, driver) {
                        results.push(metrics);
                    }
                }
                GpuBackend::IntelSysfs {
                    card_name,
                    card_path,
                    device_path,
                    hwmon_path,
                    name,
                    driver,
                    last_rc6_ms,
                    last_sample_time,
                } => {
                    if let Some(metrics) = sample_intel_sysfs(
                        card_name,
                        card_path,
                        device_path,
                        hwmon_path.as_ref(),
                        name,
                        driver,
                        last_rc6_ms,
                        last_sample_time,
                    ) {
                        results.push(metrics);
                    }
                }
            }
        }

        results
    }

    pub fn sample(&mut self) -> Option<GpuMetrics> {
        self.sample_all().into_iter().next()
    }
}

fn sample_nvml(nvml: Option<&Nvml>, device_index: u32, fallback_name: &str) -> Option<GpuMetrics> {
    let nvml = nvml?;
    let device = nvml.device_by_index(device_index).ok()?;

    let name = device.name().unwrap_or_else(|_| fallback_name.to_string());
    let utilization = device
        .utilization_rates()
        .map(|u| u.gpu as f32)
        .unwrap_or(0.0);

    let mem_info = device.memory_info().ok();
    let (memory_used, memory_total) = match mem_info {
        Some(m) => (m.used, m.total),
        None => (0, 0),
    };

    let temperature = device
        .temperature(TemperatureSensor::Gpu)
        .ok()
        .map(|t| t as f32);

    let fan_speed = device.fan_speed(0).ok().map(|f| f as f32);

    let power_usage_watts = device
        .power_usage()
        .ok()
        .map(|p| p as f32 / 1000.0);

    let power_limit_watts = device
        .enforced_power_limit()
        .ok()
        .map(|p| p as f32 / 1000.0);

    let graphics_clock_mhz = device
        .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
        .ok();

    let memory_clock_mhz = device
        .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
        .ok();

    let processes = device
        .running_graphics_processes()
        .unwrap_or_default()
        .into_iter()
        .map(|p| GpuProcessInfo {
            pid: p.pid,
            used_memory: match p.used_gpu_memory {
                nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => bytes,
                nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => 0,
            },
        })
        .collect();

    Some(GpuMetrics {
        name,
        vendor: GpuVendor::Nvidia,
        driver: "nvidia (nvml)".to_string(),
        utilization,
        memory_used,
        memory_total,
        is_shared_memory: false,
        temperature,
        fan_speed,
        power_usage_watts,
        power_limit_watts,
        graphics_clock_mhz,
        memory_clock_mhz,
        processes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_vendor_display() {
        assert_eq!(GpuVendor::Nvidia.to_string(), "NVIDIA");
        assert_eq!(GpuVendor::Amd.to_string(), "AMD");
        assert_eq!(GpuVendor::Intel.to_string(), "Intel");
        assert_eq!(GpuVendor::Other.to_string(), "GPU");
    }

    #[test]
    fn test_gpu_sampler_initialization() {
        let mut sampler = GpuSampler::new();
        let gpus = sampler.sample_all();
        println!("Discovered {} GPUs:", gpus.len());
        for (i, gpu) in gpus.iter().enumerate() {
            println!(
                "  [{}] {} | Vendor: {:?} | Driver: {} | Util: {:.1}% | VRAM: {}/{}",
                i, gpu.name, gpu.vendor, gpu.driver, gpu.utilization, gpu.memory_used, gpu.memory_total
            );
        }
        if !gpus.is_empty() {
            let primary = &gpus[0];
            assert!(!primary.name.is_empty());
            assert!(!primary.driver.is_empty());
        }
    }
}
