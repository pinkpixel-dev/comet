use super::widgets::history_chart::draw_history_chart;
use super::widgets::meter::{create_utilization_gauge, format_bytes};
use crate::history::MetricHistory;
use crate::telemetry::TelemetryState;
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
    theme: &Theme,
) {
    if let Some(gpu) = &telemetry.gpu {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Gauges & Summary
                Constraint::Min(10),  // History Chart
                Constraint::Min(8),   // GPU Process Table
            ])
            .split(area);

        // 1. GPU Summary Block
        let sum_block = Block::default()
            .title(format!(" NVIDIA GPU: {} ", gpu.name))
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

        let vram_ratio = if gpu.memory_total > 0 {
            gpu.memory_used as f64 / gpu.memory_total as f64
        } else {
            0.0
        };
        let vram_label = format!(
            "VRAM: {} / {}",
            format_bytes(gpu.memory_used),
            format_bytes(gpu.memory_total)
        );
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
            ]),
            Line::from(vec![
                Span::styled("Clock: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("GFX {} / MEM {}", gfx_clock, mem_clock), Style::default().fg(theme.success)),
            ]),
        ];
        frame.render_widget(Paragraph::new(stats_lines), sum_cols[2]);

        // 2. GPU Utilization History Chart
        let chart_data = history.gpu_utilization.as_chart_data();
        draw_history_chart(
            frame,
            rows[1],
            "GPU Utilization History (%)",
            &chart_data,
            theme.accent,
            theme.border,
            theme.text,
            100.0,
            "%",
        );

        // 3. GPU Processes Table
        let proc_block = Block::default()
            .title(format!(" Active GPU Processes ({}) ", gpu.processes.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.panel_bg));

        let proc_inner = proc_block.inner(rows[2]);
        frame.render_widget(proc_block, rows[2]);

        let header = Row::new(vec!["PID", "Process Name", "Used VRAM"])
            .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

        let rows_items: Vec<Row> = gpu
            .processes
            .iter()
            .map(|gp| {
                // Find matching system process name if available
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
    } else {
        let block = Block::default()
            .title(" GPU Telemetry ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.panel_bg));

        let p = Paragraph::new("No dedicated NVIDIA GPU found or NVML is unavailable on this host.")
            .style(Style::default().fg(theme.text_muted));

        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(p, inner);
    }
}
