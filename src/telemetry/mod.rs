pub mod cpu;
pub mod disks;
pub mod gpu;
pub mod gpu_sysfs;
pub mod memory;
pub mod network;
pub mod processes;
pub mod recorder;
pub mod sensors;
pub mod system;

pub use cpu::CpuMetrics;
pub use disks::DiskMetrics;
pub use gpu::{GpuMetrics, GpuVendor};
pub use memory::MemoryMetrics;
pub use network::NetworkMetrics;
pub use processes::{
    build_process_tree, collect_processes, send_signal, ProcessMetrics, ProcessSignal,
    ProcessSortBy, ProcessTreeItem,
};
pub use recorder::{RecordingFormat, TelemetryRecorder};
pub use sensors::SensorMetrics;
pub use system::SystemInfo;

use crossbeam_channel::{bounded, Receiver, Sender};
use disks::DiskSampler;
use gpu::GpuSampler;
use network::NetworkSampler;
use sensors::SensorSampler;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

#[derive(Debug, Clone, Default)]
pub struct TelemetryState {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub gpu: Option<GpuMetrics>,
    pub gpus: Vec<GpuMetrics>,
    pub disks: Vec<DiskMetrics>,
    pub total_disk_read_rate: f64,
    pub total_disk_write_rate: f64,
    pub network: NetworkMetrics,
    pub processes: Vec<ProcessMetrics>,
    pub sensors: SensorMetrics,
    pub system: SystemInfo,
}

pub struct TelemetryManager {
    running: Arc<AtomicBool>,
}

impl TelemetryManager {
    pub fn start(telemetry_refresh_ms: u64, process_refresh_ms: u64) -> (Self, Receiver<TelemetryState>) {
        let (sender, receiver) = bounded(2);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        thread::Builder::new()
            .name("comet-telemetry".to_string())
            .spawn(move || {
                run_telemetry_loop(sender, running_clone, telemetry_refresh_ms, process_refresh_ms);
            })
            .expect("Failed to spawn telemetry worker thread");

        (
            Self {
                running,
            },
            receiver,
        )
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn run_telemetry_loop(
    sender: Sender<TelemetryState>,
    running: Arc<AtomicBool>,
    telemetry_refresh_ms: u64,
    process_refresh_ms: u64,
) {
    let mut system = System::new_all();
    let mut gpu_sampler = GpuSampler::new();
    let mut disk_sampler = DiskSampler::new();
    let mut net_sampler = NetworkSampler::new();
    let mut sensor_sampler = SensorSampler::new();
    let mut system_info = SystemInfo::default();

    let mut last_process_refresh = Instant::now();
    let mut cached_processes = Vec::new();
    let mut cached_sensors = SensorMetrics::default();
    let mut cached_disks = Vec::new();
    let mut cached_disk_read_rate = 0.0;
    let mut cached_disk_write_rate = 0.0;

    let proc_interval = Duration::from_millis(process_refresh_ms.max(200));
    let telem_interval = Duration::from_millis(telemetry_refresh_ms.max(100));

    while running.load(Ordering::Relaxed) {
        // Fast refresh: CPU and Memory
        system.refresh_cpu_all();
        system.refresh_memory();

        let cpu = CpuMetrics::from_system(&system);
        let memory = MemoryMetrics::from_system(&system);
        let gpus = gpu_sampler.sample_all();
        let gpu = gpus.first().cloned();
        let network = net_sampler.sample();

        // Cadence for processes, sensors, and disk I/O
        if last_process_refresh.elapsed() >= proc_interval {
            system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            cached_processes = collect_processes(&system);
            cached_sensors = sensor_sampler.sample();

            let (disks, r_rate, w_rate) = disk_sampler.sample();
            cached_disks = disks;
            cached_disk_read_rate = r_rate;
            cached_disk_write_rate = w_rate;

            system_info.refresh_uptime();
            last_process_refresh = Instant::now();
        }

        let state = TelemetryState {
            cpu,
            memory,
            gpu,
            gpus,
            disks: cached_disks.clone(),
            total_disk_read_rate: cached_disk_read_rate,
            total_disk_write_rate: cached_disk_write_rate,
            network,
            processes: cached_processes.clone(),
            sensors: cached_sensors.clone(),
            system: system_info.clone(),
        };

        // Send state to UI channel (overwriting older sample if queue is full)
        let _ = sender.try_send(state);

        // Sleep before next sample
        thread::sleep(telem_interval);
    }
}
