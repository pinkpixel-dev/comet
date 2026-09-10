# Changelog

All notable changes to this project will be documented in this file.

## 0.2.0 - September 10, 2026

### ⚙️ Configuration
- Added TOML configuration file support via `~/.config/comet/config.toml` (and `$XDG_CONFIG_HOME/comet/config.toml`).
- Configurable settings for default theme, initial tab, companion pet name, pet visibility, telemetry refresh cadence, and history buffer capacity.
- Zero-config default fallback ensures Comet continues to work out of the box if no config file is present.

### 🏷️ Versioning
- Bumped version to 0.2.0 for new configuration subsystem.

## 0.1.0 - September 10, 2026

### 🚀 Launch
- Initial release of Comet, an animated system monitor for the terminal built with Rust and Ratatui.
- Real-time telemetry monitoring for CPU, GPU (NVIDIA NVML), RAM, Swap, Disks, Network, and Sensors.
- 8 dedicated tabs: Overview, CPU, GPU, Memory, Disks, Network, Processes, and Sensors.
- Rolling history ring buffers for smooth time-series charting.
- 9 built-in color themes: Candy, Synthwave, Aurora, Cyberpunk, Ocean, Amber CRT, Green CRT, Monochrome, and Rainbow.
- Interactive process manager with instant search, sorting by CPU/RAM/PID/Name, and detail inspection.
- Reactive terminal companion cat (Mochi) with mood animations tied to live system load.
- Non-blocking multithreaded architecture keeping terminal redraws smooth at 30 FPS.
