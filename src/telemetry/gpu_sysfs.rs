use super::gpu::{GpuMetrics, GpuVendor};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn sample_amd_sysfs(
    device_path: &Path,
    hwmon_path: Option<&PathBuf>,
    name: &str,
    driver: &str,
) -> Option<GpuMetrics> {
    let utilization = fs::read_to_string(device_path.join("gpu_busy_percent"))
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .unwrap_or(0.0);

    let memory_total = fs::read_to_string(device_path.join("mem_info_vram_total"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);

    let memory_used = fs::read_to_string(device_path.join("mem_info_vram_used"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);

    let is_shared_memory = memory_total == 0;

    let temperature = hwmon_path
        .and_then(|p| fs::read_to_string(p.join("temp1_input")).ok())
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|m| m / 1000.0);

    let fan_speed = hwmon_path.and_then(|p| {
        if let Ok(s) = fs::read_to_string(p.join("pwm1")) {
            s.trim()
                .parse::<f32>()
                .ok()
                .map(|pwm| (pwm / 255.0 * 100.0).clamp(0.0, 100.0))
        } else if let Ok(s) = fs::read_to_string(p.join("fan1_input")) {
            s.trim().parse::<f32>().ok()
        } else {
            None
        }
    });

    let power_usage_watts = hwmon_path
        .and_then(|p| {
            fs::read_to_string(p.join("power1_average"))
                .or_else(|_| fs::read_to_string(p.join("power1_input")))
                .ok()
        })
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|uw| uw / 1_000_000.0);

    let power_limit_watts = hwmon_path
        .and_then(|p| fs::read_to_string(p.join("power1_cap")).ok())
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|uw| uw / 1_000_000.0);

    let graphics_clock_mhz = parse_amd_clock(&device_path.join("pp_dpm_sclk"))
        .or_else(|| hwmon_path.and_then(|p| read_hwmon_freq(&p.join("freq1_input"))));

    let memory_clock_mhz = parse_amd_clock(&device_path.join("pp_dpm_mclk"))
        .or_else(|| hwmon_path.and_then(|p| read_hwmon_freq(&p.join("freq2_input"))));

    Some(GpuMetrics {
        name: name.to_string(),
        vendor: GpuVendor::Amd,
        driver: format!("{} (sysfs)", driver),
        utilization,
        memory_used,
        memory_total,
        is_shared_memory,
        temperature,
        fan_speed,
        power_usage_watts,
        power_limit_watts,
        graphics_clock_mhz,
        memory_clock_mhz,
        processes: Vec::new(),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn sample_intel_sysfs(
    card_name: &str,
    card_path: &Path,
    device_path: &Path,
    hwmon_path: Option<&PathBuf>,
    name: &str,
    driver: &str,
    last_rc6_ms: &mut Option<u64>,
    last_sample_time: &mut Option<Instant>,
) -> Option<GpuMetrics> {
    // 1. Core Utilization: Calculate differential RC6 idle residency
    let rc6_paths = [
        card_path.join("gt/gt0/rc6_residency_ms"),
        card_path.join("gt_rc6_residency_ms"),
        card_path.join("power/rc6_residency_ms"),
    ];

    let mut current_rc6 = None;
    for p in &rc6_paths {
        if let Ok(s) = fs::read_to_string(p) {
            if let Ok(ms) = s.trim().parse::<u64>() {
                current_rc6 = Some(ms);
                break;
            }
        }
    }

    let now = Instant::now();
    let utilization = match (current_rc6, *last_rc6_ms, *last_sample_time) {
        (Some(curr), Some(prev), Some(prev_time)) => {
            let elapsed_ms = now.duration_since(prev_time).as_millis() as u64;
            if elapsed_ms > 0 {
                let delta_rc6 = curr.saturating_sub(prev);
                let idle_ratio = (delta_rc6 as f64 / elapsed_ms as f64).clamp(0.0, 1.0);
                ((1.0 - idle_ratio) * 100.0) as f32
            } else {
                0.0
            }
        }
        _ => 0.0,
    };

    if let Some(curr) = current_rc6 {
        *last_rc6_ms = Some(curr);
        *last_sample_time = Some(now);
    }

    // 2. Clocks
    let freq_paths = [
        card_path.join("gt_act_freq_mhz"),
        card_path.join("gt/gt0/rps_act_freq_mhz"),
        card_path.join("gt_cur_freq_mhz"),
        card_path.join("gt/gt0/rps_cur_freq_mhz"),
    ];

    let mut graphics_clock_mhz = None;
    for p in &freq_paths {
        if let Ok(s) = fs::read_to_string(p) {
            if let Ok(mhz) = s.trim().parse::<u32>() {
                if mhz > 0 {
                    graphics_clock_mhz = Some(mhz);
                    break;
                }
            }
        }
    }

    // 3. VRAM: Dedicated lmem for Arc or 0 for integrated
    let lmem_paths = [
        device_path.join("drm").join(card_name).join("lmem_total_bytes"),
        device_path.join("mem_info_vram_total"),
    ];
    let mut memory_total = 0;
    for p in &lmem_paths {
        if let Ok(s) = fs::read_to_string(p) {
            if let Ok(tot) = s.trim().parse::<u64>() {
                memory_total = tot;
                break;
            }
        }
    }
    let is_shared_memory = memory_total == 0;

    // 4. Temperature
    let temperature = hwmon_path
        .and_then(|p| fs::read_to_string(p.join("temp1_input")).ok())
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|m| m / 1000.0);

    // 5. Power
    let power_usage_watts = hwmon_path
        .and_then(|p| {
            fs::read_to_string(p.join("power1_average"))
                .or_else(|_| fs::read_to_string(p.join("power1_input")))
                .ok()
        })
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|uw| uw / 1_000_000.0);

    Some(GpuMetrics {
        name: name.to_string(),
        vendor: GpuVendor::Intel,
        driver: format!("{} (sysfs)", driver),
        utilization,
        memory_used: 0,
        memory_total,
        is_shared_memory,
        temperature,
        fan_speed: None,
        power_usage_watts,
        power_limit_watts: None,
        graphics_clock_mhz,
        memory_clock_mhz: None,
        processes: Vec::new(),
    })
}

pub fn find_device_hwmon(device_path: &Path) -> Option<PathBuf> {
    let hwmon_dir = device_path.join("hwmon");
    if let Ok(entries) = fs::read_dir(hwmon_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("hwmon") {
                return Some(entry.path());
            }
        }
    }
    None
}

pub fn find_cpu_coretemp_hwmon() -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(name) = fs::read_to_string(p.join("name")) {
                let trimmed = name.trim();
                if trimmed == "coretemp" || trimmed == "k10temp" || trimmed == "acpitz" {
                    return Some(p);
                }
            }
        }
    }
    None
}

pub fn parse_uevent(uevent_path: &Path) -> (String, Option<String>) {
    let mut driver = String::new();
    let mut pci_slot = None;
    if let Ok(content) = fs::read_to_string(uevent_path) {
        for line in content.lines() {
            if let Some(d) = line.strip_prefix("DRIVER=") {
                driver = d.trim().to_string();
            } else if let Some(slot) = line.strip_prefix("PCI_SLOT_NAME=") {
                pci_slot = Some(slot.trim().to_string());
            }
        }
    }
    (driver, pci_slot)
}

pub fn query_pci_names() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(output) = std::process::Command::new("lspci").args(["-mm", "-D"]).output() {
        if output.status.success() {
            if let Ok(text) = String::from_utf8(output.stdout) {
                for line in text.lines() {
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() >= 4 {
                        let slot = parts[0].trim().to_string();
                        let class = parts[1];
                        if class.contains("VGA") || class.contains("3D") || class.contains("Display") {
                            let vendor = parts[3]
                                .replace(" Corporation", "")
                                .replace(", Inc.", "")
                                .replace(" [AMD/ATI]", "");
                            let device_name = if parts.len() >= 6 { parts[5] } else { parts[3] };
                            let clean_name = format!("{} {}", vendor, device_name);
                            map.insert(slot, clean_name.trim().to_string());
                        }
                    }
                }
            }
        }
    }
    map
}

pub fn parse_amd_clock(path: &Path) -> Option<u32> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if line.contains('*') {
            let mhz_part = line.split_whitespace().find(|w| w.to_lowercase().ends_with("mhz"))?;
            let num = mhz_part.trim_end_matches(|c: char| !c.is_numeric()).parse::<u32>().ok()?;
            return Some(num);
        }
    }
    None
}

pub fn read_hwmon_freq(path: &Path) -> Option<u32> {
    let content = fs::read_to_string(path).ok()?;
    let hz = content.trim().parse::<u64>().ok()?;
    Some((hz / 1_000_000) as u32)
}
