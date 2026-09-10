use sysinfo::System;

#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
    pub status: String,
    pub cmd: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

pub fn collect_processes(system: &System) -> Vec<ProcessMetrics> {
    let mut processes: Vec<ProcessMetrics> = system
        .processes()
        .iter()
        .map(|(pid, proc)| {
            let disk_usage = proc.disk_usage();
            let cmd = proc
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");

            ProcessMetrics {
                pid: pid.as_u32(),
                name: proc.name().to_string_lossy().to_string(),
                cpu_usage: proc.cpu_usage(),
                memory_bytes: proc.memory(),
                read_bytes: disk_usage.read_bytes,
                written_bytes: disk_usage.written_bytes,
                status: format!("{:?}", proc.status()),
                cmd,
            }
        })
        .collect();

    // Default sort by CPU descending
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
    processes
}
