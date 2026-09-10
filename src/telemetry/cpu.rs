use sysinfo::System;

#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub brand: String,
    pub overall_usage: f32,
    pub core_usages: Vec<f32>,
    pub frequency_mhz: u64,
    pub physical_core_count: usize,
    pub logical_core_count: usize,
    pub load_average: (f64, f64, f64),
}

impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            brand: "CPU".to_string(),
            overall_usage: 0.0,
            core_usages: Vec::new(),
            frequency_mhz: 0,
            physical_core_count: 0,
            logical_core_count: 0,
            load_average: (0.0, 0.0, 0.0),
        }
    }
}

impl CpuMetrics {
    pub fn from_system(system: &System) -> Self {
        let cpus = system.cpus();
        let overall_usage = system.global_cpu_usage();

        let brand = cpus
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let frequency_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);
        let core_usages: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let logical_core_count = cpus.len();
        let physical_core_count = System::physical_core_count().unwrap_or(logical_core_count);

        let load_avg = System::load_average();
        let load_average = (load_avg.one, load_avg.five, load_avg.fifteen);

        Self {
            brand,
            overall_usage,
            core_usages,
            frequency_mhz,
            physical_core_count,
            logical_core_count,
            load_average,
        }
    }
}
