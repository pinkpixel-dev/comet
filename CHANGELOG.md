# Changelog

All notable changes to this project will be documented in this file.

## 0.3.0 - September 10, 2026

### ⚡ Processes
- Added hierarchical process tree view toggled with `t` in the Processes tab, visualizing parent-child relationships with Unicode branch glyphs (`├─ `, `└─ `, `│  `).
- Preserved sibling sorting by CPU, memory, PID, or name within the hierarchical tree structure.
- Added process signal dispatching dialog triggered by `x` or `K` on any selected process.
- Implemented explicit confirmation dialog allowing users to select between `SIGTERM (15)` (graceful termination) and `SIGKILL (9)` (force kill) with safe cancellation (`Esc`/`n`).
- Added non-blocking status notice banners reporting the OS result of dispatched signals.

### 🏷️ Versioning
- Bumped version to 0.3.0 for Phase 2 process signals and process tree completion.

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
