use sysinfo::System;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub host_name: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_arch: String,
    pub uptime_seconds: u64,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            host_name: System::host_name().unwrap_or_else(|| "localhost".to_string()),
            os_name: System::name().unwrap_or_else(|| "Linux".to_string()),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            cpu_arch: System::cpu_arch(),
            uptime_seconds: System::uptime(),
        }
    }
}

impl SystemInfo {
    pub fn refresh_uptime(&mut self) {
        self.uptime_seconds = System::uptime();
    }

    pub fn formatted_uptime(&self) -> String {
        let secs = self.uptime_seconds;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        if days > 0 {
            format!("{}d {}h {}m", days, hours, mins)
        } else {
            format!("{}h {}m", hours, mins)
        }
    }
}
