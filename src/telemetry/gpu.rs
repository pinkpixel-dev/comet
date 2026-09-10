use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;

#[derive(Debug, Clone)]
pub struct GpuProcessInfo {
    pub pid: u32,
    pub used_memory: u64,
}

#[derive(Debug, Clone)]
pub struct GpuMetrics {
    pub name: String,
    pub utilization: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: Option<f32>,
    pub fan_speed: Option<f32>,
    pub power_usage_watts: Option<f32>,
    pub power_limit_watts: Option<f32>,
    pub graphics_clock_mhz: Option<u32>,
    pub memory_clock_mhz: Option<u32>,
    pub processes: Vec<GpuProcessInfo>,
}

pub struct GpuSampler {
    nvml: Option<Nvml>,
}

impl GpuSampler {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        Self { nvml }
    }

    pub fn sample(&mut self) -> Option<GpuMetrics> {
        let nvml = self.nvml.as_ref()?;
        let device = nvml.device_by_index(0).ok()?;

        let name = device.name().unwrap_or_else(|_| "NVIDIA GPU".to_string());
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
            utilization,
            memory_used,
            memory_total,
            temperature,
            fan_speed,
            power_usage_watts,
            power_limit_watts,
            graphics_clock_mhz,
            memory_clock_mhz,
            processes,
        })
    }
}
