use super::widgets::history_chart::draw_history_chart;
use super::widgets::meter::{create_utilization_gauge, format_bytes};
use crate::history::MetricHistory;
use crate::telemetry::{GpuVendor, TelemetryState};
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw_gpu_tab(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    history: &MetricHistory,
    selected_gpu_index: usize,
    theme: &Theme,
) {
    let active_gpu = telemetry
        .gpus
        .get(selected_gpu_index)
        .or(telemetry.gpu.as_ref());

    if let Some(gpu) = active_gpu {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Gauges & Summary
                Constraint::Min(10),  // History Chart
                Constraint::Min(8),   // GPU Process Table / Details
            ])
            .split(area);

        // 1. GPU Summary Block
        let title_text = if telemetry.gpus.len() > 1 {
            format!(
                " {} GPU ({}/{}): {} [g: switch] ",
                gpu.vendor,
                selected_gpu_index + 1,
                telemetry.gpus.len(),
                gpu.name
            )
        } else {
            format!(" {} GPU: {} ", gpu.vendor, gpu.name)
        };

        let sum_block = Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.panel_bg));

        let sum_inner = sum_block.inner(rows[0]);
        frame.render_widget(sum_block, rows[0]);

        let sum_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(sum_inner);

        let gpu_gauge = create_utilization_gauge(
            "Core Utilization",
            gpu.utilization as f64 / 100.0,
            theme.accent,
            theme.background,
            theme.text,
        );
        frame.render_widget(gpu_gauge, sum_cols[0]);

        let (vram_label, vram_ratio) = if gpu.is_shared_memory || gpu.memory_total == 0 {
            ("VRAM: Shared System Memory (UMA)".to_string(), 0.0)
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
        frame.render_widget(vram_gauge, sum_cols[1]);

        let temp_str = gpu.temperature.map(|t| format!("{:.0}°C", t)).unwrap_or_else(|| "N/A".to_string());
        let fan_str = gpu.fan_speed.map(|f| format!("{:.0}%", f)).unwrap_or_else(|| "N/A".to_string());
        let power_str = gpu.power_usage_watts.map(|p| format!("{:.1} W", p)).unwrap_or_else(|| "N/A".to_string());
        let power_limit = gpu.power_limit_watts.map(|p| format!("{:.1} W", p)).unwrap_or_else(|| "N/A".to_string());
        let gfx_clock = gpu.graphics_clock_mhz.map(|c| format!("{} MHz", c)).unwrap_or_else(|| "N/A".to_string());
        let mem_clock = gpu.memory_clock_mhz.map(|c| format!("{} MHz", c)).unwrap_or_else(|| "N/A".to_string());

        let stats_lines = vec![
            Line::from(vec![
                Span::styled("Temp:  ", Style::default().fg(theme.text_muted)),
                Span::styled(temp_str, Style::default().fg(theme.danger).add_modifier(Modifier::BOLD)),
                Span::styled("   Fan: ", Style::default().fg(theme.text_muted)),
                Span::styled(fan_str, Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled("Power: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("{} / {}", power_str, power_limit), Style::default().fg(theme.warning)),
                Span::styled("   Driver: ", Style::default().fg(theme.text_muted)),
                Span::styled(&gpu.driver, Style::default().fg(theme.primary)),
            ]),
            Line::from(vec![
                Span::styled("Clock: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("GFX {} / MEM {}", gfx_clock, mem_clock), Style::default().fg(theme.success)),
            ]),
        ];
        frame.render_widget(Paragraph::new(stats_lines), sum_cols[2]);

        // 2. GPU Utilization History Chart
        let chart_data = history.gpu_utilization.as_chart_data();
        let chart_title = format!("{} Utilization History (%)", gpu.name);
        draw_history_chart(
            frame,
            rows[1],
            &chart_title,
            &chart_data,
            theme.accent,
            theme.border,
            theme.text,
            100.0,
            "%",
        );

        // 3. GPU Processes Table or Hardware Info
        let proc_title = if gpu.processes.is_empty() {
            " GPU Activity & Processes ".to_string()
        } else {
            format!(" Active GPU Processes ({}) ", gpu.processes.len())
        };

        let proc_block = Block::default()
            .title(proc_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.panel_bg));

        let proc_inner = proc_block.inner(rows[2]);
        frame.render_widget(proc_block, rows[2]);

        if gpu.processes.is_empty() {
            let info_lines = vec![
                Line::from(vec![
                    Span::styled("Device Name:   ", Style::default().fg(theme.text_muted)),
                    Span::styled(&gpu.name, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("Vendor/Driver: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{} [{}]", gpu.vendor, gpu.driver), Style::default().fg(theme.accent)),
                ]),
                Line::from(vec![
                    Span::styled("Process Info:  ", Style::default().fg(theme.text_muted)),
                    Span::styled(
                        if gpu.vendor == GpuVendor::Nvidia {
                            "No active graphics or compute processes registered with NVML."
                        } else {
                            "Per-process VRAM accounting requires proprietary driver APIs or kernel debugfs; engine activity is actively tracked above."
                        },
                        Style::default().fg(theme.text_muted),
                    ),
                ]),
            ];
            frame.render_widget(Paragraph::new(info_lines), proc_inner);
        } else {
            let header = Row::new(vec!["PID", "Process Name", "Used VRAM"])
                .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

            let rows_items: Vec<Row> = gpu
                .processes
                .iter()
                .map(|gp| {
                    let proc_name = telemetry
                        .processes
                        .iter()
                        .find(|p| p.pid == gp.pid)
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());

                    Row::new(vec![
                        gp.pid.to_string(),
                        proc_name,
                        format_bytes(gp.used_memory),
                    ])
                    .style(Style::default().fg(theme.text))
                })
                .collect();

            let table = Table::new(
                rows_items,
                [
                    Constraint::Length(10),
                    Constraint::Min(25),
                    Constraint::Length(15),
                ],
            )
            .header(header);

            frame.render_widget(table, proc_inner);
        }
    } else {
        let block = Block::default()
            .title(" GPU Telemetry ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.panel_bg));

        let p = Paragraph::new("No supported GPU found via NVML or Linux DRM sysfs (/sys/class/drm).")
            .style(Style::default().fg(theme.text_muted));

        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(p, inner);
    }
}
