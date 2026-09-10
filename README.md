# Comet ☄

A colorful, animated system monitor for your terminal built with Rust and Ratatui.

Comet tracks real system telemetry including CPU, multi-vendor GPU (NVIDIA NVML, AMD sysfs, Intel sysfs), memory, mounted disks, network activity, processes, and thermal sensors, wrapped up with charts, gauges, animations, 9 themes, and a reactive terminal cat named Mochi.

## Features

- **Real Telemetry**: Direct system metrics using `sysinfo`, native `nvml-wrapper` for NVIDIA, and native Linux sysfs for AMD (`amdgpu`) and Intel (`i915`/`xe`).
- **Dedicated Tabs**:
  - `Overview`: High-density Las Vegas dashboard with CPU, GPU, RAM, Disks, Network, top processes, and Mochi.
  - `CPU`: Overall usage, per-core activity, clock speeds, and load averages.
  - `GPU`: Multi-vendor GPU monitoring for NVIDIA, AMD, and Intel with live core utilization, VRAM usage, temperature, fan speed, power draw, and multi-GPU switching (`g`).
  - `Memory`: RAM and swap breakdowns with rolling usage history.
  - `Disks`: Storage capacity gauges and differential read/write throughput per volume.
  - `Network`: Upload and download throughput charts and interface counters.
  - `Processes`: Interactive process manager with instant search (`/`), sort cycling (`s`), hierarchical process tree view (`t`), signal dispatching (`x` / `K`), and detail inspection.
  - `Sensors`: Hardware thermal sensors with color-coded warning thresholds.
- **Mochi the Cat**: A reactive terminal companion that sleeps when your machine is idle, perks up during work, and gets excited when your GPU spins up.
- **Telemetry Export**: Toggle session recording anytime with `r` to dump live telemetry to CSV or JSON for post-run analysis and benchmarking.
- **9 Built-in Themes**: Candy (default), Synthwave, Aurora, Cyberpunk, Ocean, Amber CRT, Green CRT, Monochrome, and Rainbow.
- **Non-blocking Telemetry**: Background worker threads handle telemetry collection so UI rendering stays fluid at 30 FPS.

## Installation & Running

Ensure you have Rust and Cargo installed:

```bash
cargo build --release
./target/release/comet
```

Or run directly with cargo:

```bash
cargo run --release
```

## Configuration

Comet works out of the box with zero configuration needed. To customize your defaults, create `~/.config/comet/config.toml` (or `$XDG_CONFIG_HOME/comet/config.toml`):

```toml
# Default theme: "candy", "synthwave", "aurora", "cyberpunk",
# "ocean", "amber-crt", "green-crt", "monochrome", "rainbow"
theme = "candy"

# Default starting tab: "overview", "cpu", "gpu", "memory", "disks", "network", "processes", "sensors"
default_tab = "overview"

# Companion pet settings
show_pet = true
pet_name = "Mochi"

# Telemetry polling intervals (milliseconds)
telemetry_refresh_ms = 500
process_refresh_ms = 1000

# Rolling chart history sample count
history_capacity = 120

# UI animations toggle
animations = true

# Telemetry recording export settings
recording_format = "csv" # "csv" or "json"
# recording_dir = "~/.local/share/comet/recordings"
```

## Keyboard Shortcuts

| Key | Action |
| --- | --- |
| `1` / `o` | Switch to Overview tab |
| `2` / `c` | Switch to CPU tab |
| `3` / `g` | Switch to GPU tab / cycle active GPU (`g` on GPU tab) |
| `4` / `m` | Switch to Memory tab |
| `5` / `d` | Switch to Disks tab |
| `6` / `n` | Switch to Network tab |
| `7` / `p` | Switch to Processes tab |
| `8` / `s` | Switch to Sensors tab |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `r` | Toggle telemetry recording (CSV / JSON) |
| `t` | Toggle process tree (in Processes tab) / cycle themes |
| `x` / `K` | Send signal to process (SIGTERM / SIGKILL) |
| `T` | Open theme picker modal |
| `P` | Open pet companion modal |
| `j` / `k` (or `Down`/`Up`) | Navigate process list |
| `/` | Search/filter processes |
| `s` | Cycle process sorting (in Processes tab) |
| `?` / `h` | Toggle help overlay |
| `q` / `Esc` | Quit Comet / close active modal |

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.
