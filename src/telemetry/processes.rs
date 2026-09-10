use std::collections::{HashMap, HashSet};
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub parent_pid: Option<u32>,
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

#[derive(Debug, Clone)]
pub struct ProcessTreeItem {
    pub process: ProcessMetrics,
    pub depth: usize,
    pub prefix: String,
    pub is_last_child: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSignal {
    Term,
    Kill,
}

impl ProcessSignal {
    pub fn name(self) -> &'static str {
        match self {
            ProcessSignal::Term => "SIGTERM (15)",
            ProcessSignal::Kill => "SIGKILL (9)",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            ProcessSignal::Term => "SIGTERM",
            ProcessSignal::Kill => "SIGKILL",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            ProcessSignal::Term => "Graceful termination request (allows process to clean up)",
            ProcessSignal::Kill => "Force kill (immediately terminates process; cannot be caught)",
        }
    }

    pub fn as_c_int(self) -> libc::c_int {
        match self {
            ProcessSignal::Term => libc::SIGTERM,
            ProcessSignal::Kill => libc::SIGKILL,
        }
    }
}

pub fn send_signal(pid: u32, signal: ProcessSignal) -> Result<(), String> {
    let res = unsafe { libc::kill(pid as libc::pid_t, signal.as_c_int()) };
    if res == 0 {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        Err(format!("{}", err))
    }
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
                parent_pid: proc.parent().map(|p| p.as_u32()),
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

pub fn sort_pids(pids: &mut [u32], pid_to_proc: &HashMap<u32, &ProcessMetrics>, sort_by: ProcessSortBy) {
    match sort_by {
        ProcessSortBy::Cpu => pids.sort_by(|&a, &b| {
            let pa = pid_to_proc.get(&a).map(|p| p.cpu_usage).unwrap_or(0.0);
            let pb = pid_to_proc.get(&b).map(|p| p.cpu_usage).unwrap_or(0.0);
            pb.partial_cmp(&pa).unwrap_or(std::cmp::Ordering::Equal)
        }),
        ProcessSortBy::Memory => pids.sort_by(|&a, &b| {
            let pa = pid_to_proc.get(&a).map(|p| p.memory_bytes).unwrap_or(0);
            let pb = pid_to_proc.get(&b).map(|p| p.memory_bytes).unwrap_or(0);
            pb.cmp(&pa)
        }),
        ProcessSortBy::Pid => pids.sort_by(|&a, &b| a.cmp(&b)),
        ProcessSortBy::Name => pids.sort_by(|&a, &b| {
            let na = pid_to_proc.get(&a).map(|p| p.name.as_str()).unwrap_or("");
            let nb = pid_to_proc.get(&b).map(|p| p.name.as_str()).unwrap_or("");
            na.to_lowercase().cmp(&nb.to_lowercase())
        }),
    }
}

pub fn build_process_tree(processes: &[ProcessMetrics], sort_by: ProcessSortBy) -> Vec<ProcessTreeItem> {
    if processes.is_empty() {
        return Vec::new();
    }

    let pid_to_proc: HashMap<u32, &ProcessMetrics> = processes.iter().map(|p| (p.pid, p)).collect();
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut root_pids: Vec<u32> = Vec::new();

    for proc in processes {
        match proc.parent_pid {
            Some(ppid) if ppid != 0 && ppid != proc.pid && pid_to_proc.contains_key(&ppid) => {
                children_map.entry(ppid).or_default().push(proc.pid);
            }
            _ => {
                root_pids.push(proc.pid);
            }
        }
    }

    // Sort roots according to active sort option
    sort_pids(&mut root_pids, &pid_to_proc, sort_by);

    // Sort all child slices
    for children in children_map.values_mut() {
        sort_pids(children, &pid_to_proc, sort_by);
    }

    let mut output: Vec<ProcessTreeItem> = Vec::with_capacity(processes.len());
    let mut visited: HashSet<u32> = HashSet::new();
    let mut ancestors_are_last: Vec<bool> = Vec::new();

    let root_count = root_pids.len();
    for (idx, &root_pid) in root_pids.iter().enumerate() {
        let is_last = idx == root_count - 1;
        dfs_tree(
            root_pid,
            0,
            is_last,
            &mut ancestors_are_last,
            &pid_to_proc,
            &children_map,
            &mut visited,
            &mut output,
        );
    }

    // Capture any remaining processes that were part of cyclic dependencies or unvisited
    let mut remaining: Vec<u32> = processes
        .iter()
        .map(|p| p.pid)
        .filter(|pid| !visited.contains(pid))
        .collect();
    sort_pids(&mut remaining, &pid_to_proc, sort_by);
    let rem_count = remaining.len();
    for (idx, &pid) in remaining.iter().enumerate() {
        let is_last = idx == rem_count - 1;
        dfs_tree(
            pid,
            0,
            is_last,
            &mut ancestors_are_last,
            &pid_to_proc,
            &children_map,
            &mut visited,
            &mut output,
        );
    }

    output
}

fn dfs_tree(
    pid: u32,
    depth: usize,
    is_last_child: bool,
    ancestors_are_last: &mut Vec<bool>,
    pid_to_proc: &HashMap<u32, &ProcessMetrics>,
    children_map: &HashMap<u32, Vec<u32>>,
    visited: &mut HashSet<u32>,
    output: &mut Vec<ProcessTreeItem>,
) {
    if !visited.insert(pid) {
        return;
    }

    let Some(proc) = pid_to_proc.get(&pid) else {
        return;
    };

    let mut prefix = String::new();
    if depth > 0 {
        for &was_last in ancestors_are_last.iter() {
            if was_last {
                prefix.push_str("   ");
            } else {
                prefix.push_str("│  ");
            }
        }
        if is_last_child {
            prefix.push_str("└─ ");
        } else {
            prefix.push_str("├─ ");
        }
    }

    output.push(ProcessTreeItem {
        process: (*proc).clone(),
        depth,
        prefix,
        is_last_child,
    });

    if let Some(children) = children_map.get(&pid) {
        let count = children.len();
        for (idx, &child_pid) in children.iter().enumerate() {
            let child_is_last = idx == count - 1;
            if depth > 0 {
                ancestors_are_last.push(is_last_child);
            }
            dfs_tree(
                child_pid,
                depth + 1,
                child_is_last,
                ancestors_are_last,
                pid_to_proc,
                children_map,
                visited,
                output,
            );
            if depth > 0 {
                ancestors_are_last.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_proc(pid: u32, parent_pid: Option<u32>, name: &str, cpu: f32) -> ProcessMetrics {
        ProcessMetrics {
            pid,
            parent_pid,
            name: name.to_string(),
            cpu_usage: cpu,
            memory_bytes: 1024,
            read_bytes: 0,
            written_bytes: 0,
            status: "Running".to_string(),
            cmd: format!("/bin/{}", name),
        }
    }

    #[test]
    fn test_process_tree_hierarchy() {
        let procs = vec![
            dummy_proc(1, None, "systemd", 1.0),
            dummy_proc(100, Some(1), "NetworkManager", 2.0),
            dummy_proc(101, Some(100), "dhclient", 0.5),
            dummy_proc(200, Some(1), "sshd", 0.1),
        ];

        let tree = build_process_tree(&procs, ProcessSortBy::Pid);
        assert_eq!(tree.len(), 4);

        // Root: systemd
        assert_eq!(tree[0].process.pid, 1);
        assert_eq!(tree[0].depth, 0);
        assert_eq!(tree[0].prefix, "");

        // Children of systemd: 100, 200
        assert_eq!(tree[1].process.pid, 100);
        assert_eq!(tree[1].depth, 1);
        assert_eq!(tree[1].prefix, "├─ ");

        // Child of 100: 101
        assert_eq!(tree[2].process.pid, 101);
        assert_eq!(tree[2].depth, 2);
        assert_eq!(tree[2].prefix, "│  └─ ");

        // Second child of 1: 200
        assert_eq!(tree[3].process.pid, 200);
        assert_eq!(tree[3].depth, 1);
        assert_eq!(tree[3].prefix, "└─ ");
    }

    #[test]
    fn test_process_tree_handles_cycles() {
        let procs = vec![
            dummy_proc(10, Some(20), "procA", 1.0),
            dummy_proc(20, Some(10), "procB", 1.0),
        ];

        let tree = build_process_tree(&procs, ProcessSortBy::Pid);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_process_tree_sibling_sorting_by_cpu() {
        let procs = vec![
            dummy_proc(1, None, "systemd", 1.0),
            dummy_proc(10, Some(1), "low_cpu_proc", 5.0),
            dummy_proc(20, Some(1), "high_cpu_proc", 85.0),
        ];

        let tree = build_process_tree(&procs, ProcessSortBy::Cpu);
        assert_eq!(tree.len(), 3);
        assert_eq!(tree[0].process.pid, 1);
        assert_eq!(tree[1].process.pid, 20); // higher CPU sibling first
        assert_eq!(tree[2].process.pid, 10);
    }

    #[test]
    fn test_process_signals() {
        assert_eq!(ProcessSignal::Term.as_c_int(), libc::SIGTERM);
        assert_eq!(ProcessSignal::Kill.as_c_int(), libc::SIGKILL);
        assert_eq!(ProcessSignal::Term.short_name(), "SIGTERM");
        assert_eq!(ProcessSignal::Kill.short_name(), "SIGKILL");
    }
}

