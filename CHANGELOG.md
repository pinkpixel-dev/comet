# Changelog

All notable changes to this project will be documented in this file.

## 0.5.0 - September 10, 2026

### 🚀 Features
- Added session recording engine in `src/telemetry/recorder.rs` for exporting real-time telemetry to CSV or JSON.
- Supported continuous buffered file streaming with low memory footprint, writing samples directly on each telemetry cycle.
- Standardized CSV export columns including timestamps, elapsed seconds, overall and per-core CPU, RAM bytes and percent, Swap, GPU metrics (vendor, name, load, VRAM, temp, power), disk throughput, network rates, and thermal sensors.
- Added valid structured JSON array streaming export compatible with standard JSON parsers and data science toolkits.
- Added configuration options in `~/.config/comet/config.toml` for `recording_format` (`"csv"` or `"json"`) and custom `recording_dir`.

### 🎨 UI & Layout
- Added prominent `● REC` recording badge to the top header bar displaying elapsed recording time (`MM:SS`) and sample count.
- Added interactive toggle keybinding (`r` / `R`) to start and stop recordings with instant confirmation toast messages.
- Updated footer status bar with the `[r] Record` keyboard shortcut.
- Updated help modal shortcuts table with the new recording toggle action.

### 🏷️ Versioning
- Bumped version to 0.5.0 for Phase 4 telemetry export completion.

## 0.4.0 - September 10, 2026

### 🚀 Features
- Added native AMD GPU telemetry via Linux `/sys/class/drm` and `amdgpu` sysfs interfaces, supporting utilization, VRAM usage, temperature, power draw, fan speed, and core/memory clocks.
- Added native Intel GPU telemetry via Linux `/sys/class/drm` and `i915`/`xe` sysfs interfaces with differential RC6 residency utilization tracking and clock frequencies.
- Implemented multi-GPU discovery and interactive cycling via `g` on the GPU tab, allowing users on hybrid laptops and multi-adapter systems to inspect each GPU with live status banners.
- Added modular telemetry architecture separating sysfs inspection into `gpu_sysfs` while preserving sub-millisecond NVML bindings for NVIDIA adapters.

### 🎨 UI & Layout
- Updated GPU tab header with dynamic vendor branding, multi-GPU index counters (`1 of 2`), and `[g: switch]` key hints.
- Added support for integrated graphics shared system memory (UMA), gracefully indicating shared memory instead of 0-byte VRAM.
- Updated the footer status bar to display active GPU vendor and driver backend dynamically.
- Updated the help modal with the GPU cycling shortcut (`g`).

### 🏷️ Versioning
- Bumped version to 0.4.0 for Phase 3 multi-vendor GPU telemetry completion.

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
