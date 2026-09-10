use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Instant;
use sysinfo::Disks;

#[derive(Debug, Clone)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: PathBuf,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f32,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
}

pub struct DiskSampler {
    disks: Disks,
    last_sample_time: Instant,
    last_device_io: HashMap<String, (u64, u64)>, // (sectors_read, sectors_written)
}

impl DiskSampler {
    pub fn new() -> Self {
        let disks = Disks::new_with_refreshed_list();
        let mut sampler = Self {
            disks,
            last_sample_time: Instant::now(),
            last_device_io: HashMap::new(),
        };
        sampler.sample_io_stats();
        sampler
    }

    fn sample_io_stats(&mut self) -> HashMap<String, (f64, f64)> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sample_time).as_secs_f64();
        self.last_sample_time = now;

        let mut rates = HashMap::new();
        let mut current_io = HashMap::new();

        if let Ok(file) = File::open("/proc/diskstats") {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 10 {
                    let dev = parts[2].to_string();
                    let sectors_read: u64 = parts[5].parse().unwrap_or(0);
                    let sectors_written: u64 = parts[9].parse().unwrap_or(0);

                    if let Some(&(prev_read, prev_written)) = self.last_device_io.get(&dev) {
                        if elapsed > 0.0 {
                            let diff_read = sectors_read.saturating_sub(prev_read) as f64 * 512.0;
                            let diff_written = sectors_written.saturating_sub(prev_written) as f64 * 512.0;
                            rates.insert(dev.clone(), (diff_read / elapsed, diff_written / elapsed));
                        }
                    }

                    current_io.insert(dev, (sectors_read, sectors_written));
                }
            }
        }

        self.last_device_io = current_io;
        rates
    }

    pub fn sample(&mut self) -> (Vec<DiskMetrics>, f64, f64) {
        self.disks.refresh(true);
        let io_rates = self.sample_io_stats();

        let mut total_read_rate = 0.0;
        let mut total_write_rate = 0.0;

        for (r, w) in io_rates.values() {
            total_read_rate += r;
            total_write_rate += w;
        }

        let mut metrics = Vec::new();
        for disk in self.disks.iter() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let usage_percent = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            let dev_name = disk.name().to_string_lossy().to_string();
            // Clean device name if it's a path like /dev/nvme0n1p1 -> nvme0n1p1
            let base_dev = dev_name.rsplit('/').next().unwrap_or(&dev_name);
            let (read_rate, write_rate) = io_rates.get(base_dev).copied().unwrap_or((0.0, 0.0));

            metrics.push(DiskMetrics {
                name: dev_name,
                mount_point: disk.mount_point().to_path_buf(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_bytes: total,
                available_bytes: available,
                used_bytes: used,
                usage_percent,
                read_bytes_per_sec: read_rate,
                write_bytes_per_sec: write_rate,
            });
        }

        metrics.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
        (metrics, total_read_rate, total_write_rate)
    }
}
