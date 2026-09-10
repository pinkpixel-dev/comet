use super::widgets::history_chart::draw_history_chart;
use super::widgets::meter::{create_line_gauge, create_utilization_gauge, format_bytes, format_rate};
use super::widgets::pet_widget::draw_pet_widget;
use crate::animation::AnimationState;
use crate::history::MetricHistory;
use crate::pet::PetState;
use crate::telemetry::TelemetryState;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw_overview(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    history: &MetricHistory,
    pet: &PetState,
    theme: &Theme,
    anim: &AnimationState,
) {
    // 3 main vertical sections: Header (fixed or handled in parent), Main Body, Footer
    let body_columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    draw_left_column(frame, body_columns[0], telemetry, history, theme, anim);
    draw_center_column(frame, body_columns[1], telemetry, history, theme, anim);
    draw_right_column(frame, body_columns[2], telemetry, history, pet, theme, anim);
}

fn draw_left_column(
    frame: &mut Frame,
    area: Rect,
    telem: &TelemetryState,
    hist: &MetricHistory,
    theme: &Theme,
    _anim: &AnimationState,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // CPU Gauge & Cores
            Constraint::Min(8),    // CPU History Chart
            Constraint::Length(7), // Memory & Swap
        ])
        .split(area);

    // 1. CPU Gauge & mini core indicators
    let cpu_block = Block::default()
        .title(format!(" CPU: {} ", telem.cpu.brand))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.panel_bg));

    let inner = cpu_block.inner(rows[0]);
    frame.render_widget(cpu_block, rows[0]);

    let cpu_splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let cpu_gauge = create_utilization_gauge(
        "Overall Utilization",
        telem.cpu.overall_usage as f64 / 100.0,
        theme.primary,
        theme.background,
        theme.text,
    );
    frame.render_widget(cpu_gauge, cpu_splits[0]);

    let load_text = format!(
        "Load: {:.2}, {:.2}, {:.2}  |  Freq: {} MHz",
        telem.cpu.load_average.0,
        telem.cpu.load_average.1,
        telem.cpu.load_average.2,
        telem.cpu.frequency_mhz
    );
    let load_p = Paragraph::new(load_text).style(Style::default().fg(theme.text_muted));
    frame.render_widget(load_p, cpu_splits[1]);

    // Mini per-core sparklines / preview (first 4 cores)
    let core_splits = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, 4); 4])
        .split(cpu_splits[2]);

    for (i, &usage) in telem.cpu.core_usages.iter().take(4).enumerate() {
        let gauge = create_line_gauge(
            format!("C{}", i + 1),
            usage as f64 / 100.0,
            theme.accent,
            theme.background,
        );
        frame.render_widget(gauge, core_splits[i]);
    }

    // 2. CPU History Chart
    let chart_data = hist.cpu_overall.as_chart_data();
    draw_history_chart(
        frame,
        rows[1],
        "CPU Activity (%)",
        &chart_data,
        theme.primary,
        theme.border,
        theme.text,
        100.0,
        "%",
    );

    // 3. Memory & Swap
    let mem_block = Block::default()
        .title(" Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary))
        .style(Style::default().bg(theme.panel_bg));

    let mem_inner = mem_block.inner(rows[2]);
    frame.render_widget(mem_block, rows[2]);

    let mem_splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(mem_inner);

    let ram_label = format!(
        "RAM: {} / {}",
        format_bytes(telem.memory.used_bytes),
        format_bytes(telem.memory.total_bytes)
    );
    let ram_gauge = create_utilization_gauge(
        ram_label,
        telem.memory.usage_percent() as f64 / 100.0,
        theme.secondary,
        theme.background,
        theme.text,
    );
    frame.render_widget(ram_gauge, mem_splits[0]);

    let swap_label = format!(
        "Swap: {} / {}",
        format_bytes(telem.memory.swap_used_bytes),
        format_bytes(telem.memory.swap_total_bytes)
    );
    let swap_gauge = create_utilization_gauge(
        swap_label,
        telem.memory.swap_usage_percent() as f64 / 100.0,
        theme.accent,
        theme.background,
        theme.text,
    );
    frame.render_widget(swap_gauge, mem_splits[1]);
}

fn draw_center_column(
    frame: &mut Frame,
    area: Rect,
    telem: &TelemetryState,
    hist: &MetricHistory,
    theme: &Theme,
    _anim: &AnimationState,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // GPU Panel
            Constraint::Min(8),     // Network Chart
            Constraint::Length(5),  // Network Rates
        ])
        .split(area);

    // 1. GPU Panel
    let active_gpu = telem.gpus.first().or(telem.gpu.as_ref());
    let gpu_title = match active_gpu {
        Some(g) => format!(" {} GPU: {} ", g.vendor, g.name),
        None => " GPU: N/A ".to_string(),
    };

    let gpu_block = Block::default()
        .title(gpu_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.panel_bg));

    let gpu_inner = gpu_block.inner(rows[0]);
    frame.render_widget(gpu_block, rows[0]);

    if let Some(gpu) = active_gpu {
        let g_splits = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // GPU Util
                Constraint::Length(2), // VRAM
                Constraint::Length(2), // Temp & Power stats
                Constraint::Length(1), // Fan & Clocks
            ])
            .split(gpu_inner);

        let gpu_gauge = create_utilization_gauge(
            "GPU Core",
            gpu.utilization as f64 / 100.0,
            theme.accent,
            theme.background,
            theme.text,
        );
        frame.render_widget(gpu_gauge, g_splits[0]);

        let (vram_label, vram_ratio) = if gpu.is_shared_memory || gpu.memory_total == 0 {
            ("VRAM: Shared Memory (UMA)".to_string(), 0.0)
        } else {
            let ratio = (gpu.memory_used as f64 / gpu.memory_total as f64).clamp(0.0, 1.0);
            (
                format!(
                    "VRAM: {} / {}",
                    format_bytes(gpu.memory_used),
                    format_bytes(gpu.memory_total)
                ),
                ratio,
            )
        };
        let vram_gauge = create_utilization_gauge(
            vram_label,
            vram_ratio,
            theme.primary,
            theme.background,
            theme.text,
        );
        frame.render_widget(vram_gauge, g_splits[1]);

        let temp_str = gpu
            .temperature
            .map(|t| format!("{:.0}°C", t))
            .unwrap_or_else(|| "N/A".to_string());
        let power_str = gpu
            .power_usage_watts
            .map(|p| format!("{:.1}W", p))
            .unwrap_or_else(|| "N/A".to_string());
        let fan_str = gpu
            .fan_speed
            .map(|f| format!("{:.0}%", f))
            .unwrap_or_else(|| "N/A".to_string());

        let stats_line = format!(
            "Temp: {}  |  Power: {}  |  Fan: {}",
            temp_str, power_str, fan_str
        );
        frame.render_widget(
            Paragraph::new(stats_line).style(Style::default().fg(theme.text_muted)),
            g_splits[2],
        );
    } else {
        let p = Paragraph::new("No supported GPU detected via NVML or Linux DRM sysfs.")
            .style(Style::default().fg(theme.text_muted));
        frame.render_widget(p, gpu_inner);
    }

    // 2. Network History Chart
    let net_data = hist.net_rx_rate.as_chart_data();
    let max_rx = net_data
        .iter()
        .map(|(_, y)| *y)
        .fold(1024.0 * 1024.0, f64::max);

    draw_history_chart(
        frame,
        rows[1],
        "Network Ingress (RX)",
        &net_data,
        theme.success,
        theme.border,
        theme.text,
        max_rx,
        " B/s",
    );

    // 3. Network Throughput Banner
    let net_block = Block::default()
        .title(" Network Throughput ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let net_inner = net_block.inner(rows[2]);
    frame.render_widget(net_block, rows[2]);

    let net_text = vec![
        Line::from(vec![
            Span::styled(" ↓ RX: ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled(format_rate(telem.network.total_rx_rate), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("   ↑ TX: ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(format_rate(telem.network.total_tx_rate), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(format!(" Total: ↓ {} / ↑ {}", format_bytes(telem.network.total_rx_bytes), format_bytes(telem.network.total_tx_bytes)), Style::default().fg(theme.text_muted)),
        ]),
    ];
    frame.render_widget(Paragraph::new(net_text), net_inner);
}

fn draw_right_column(
    frame: &mut Frame,
    area: Rect,
    telem: &TelemetryState,
    _hist: &MetricHistory,
    pet: &PetState,
    theme: &Theme,
    anim: &AnimationState,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Pet Widget
            Constraint::Length(7), // Primary Disk
            Constraint::Min(8),    // Top Processes
        ])
        .split(area);

    // 1. Animated Pet
    draw_pet_widget(frame, rows[0], pet, theme, anim.pulse);

    // 2. Primary Disk / Storage
    let disk_block = Block::default()
        .title(" Storage ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(theme.panel_bg));

    let disk_inner = disk_block.inner(rows[1]);
    frame.render_widget(disk_block, rows[1]);

    if let Some(primary_disk) = telem.disks.first() {
        let d_splits = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(2)])
            .split(disk_inner);

        let d_label = format!(
            "{} ({}): {} / {}",
            primary_disk.mount_point.to_string_lossy(),
            primary_disk.file_system,
            format_bytes(primary_disk.used_bytes),
            format_bytes(primary_disk.total_bytes)
        );
        let d_gauge = create_utilization_gauge(
            d_label,
            primary_disk.usage_percent as f64 / 100.0,
            theme.warning,
            theme.background,
            theme.text,
        );
        frame.render_widget(d_gauge, d_splits[0]);

        let io_text = format!(
            "Read: {}  |  Write: {}",
            format_rate(telem.total_disk_read_rate),
            format_rate(telem.total_disk_write_rate)
        );
        frame.render_widget(
            Paragraph::new(io_text).style(Style::default().fg(theme.text_muted)),
            d_splits[1],
        );
    } else {
        frame.render_widget(
            Paragraph::new("No mounted filesystems found.").style(Style::default().fg(theme.text_muted)),
            disk_inner,
        );
    }

    // 3. Top Processes Table
    let proc_block = Block::default()
        .title(" Top Processes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let proc_inner = proc_block.inner(rows[2]);
    frame.render_widget(proc_block, rows[2]);

    let header = Row::new(vec!["PID", "Name", "CPU%", "RAM"])
        .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

    let rows_items: Vec<Row> = telem
        .processes
        .iter()
        .take(6)
        .map(|p| {
            Row::new(vec![
                p.pid.to_string(),
                p.name.clone(),
                format!("{:.1}%", p.cpu_usage),
                format_bytes(p.memory_bytes),
            ])
            .style(Style::default().fg(theme.text))
        })
        .collect();

    let table = Table::new(
        rows_items,
        [
            Constraint::Length(7),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(9),
        ],
    )
    .header(header);

    frame.render_widget(table, proc_inner);
}
