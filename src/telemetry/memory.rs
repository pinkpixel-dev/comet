use sysinfo::System;

#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub free_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            free_bytes: 0,
            swap_total_bytes: 0,
            swap_used_bytes: 0,
            swap_free_bytes: 0,
        }
    }
}

impl MemoryMetrics {
    pub fn from_system(system: &System) -> Self {
        Self {
            total_bytes: system.total_memory(),
            used_bytes: system.used_memory(),
            available_bytes: system.available_memory(),
            free_bytes: system.free_memory(),
            swap_total_bytes: system.total_swap(),
            swap_used_bytes: system.used_swap(),
            swap_free_bytes: system.free_swap(),
        }
    }

    pub fn usage_percent(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f32 / self.total_bytes as f32) * 100.0
        }
    }

    pub fn swap_usage_percent(&self) -> f32 {
        if self.swap_total_bytes == 0 {
            0.0
        } else {
            (self.swap_used_bytes as f32 / self.swap_total_bytes as f32) * 100.0
        }
    }
}
