use super::widgets::history_chart::draw_history_chart;
use super::widgets::meter::{create_line_gauge, create_utilization_gauge};
use crate::history::MetricHistory;
use crate::telemetry::TelemetryState;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_cpu_tab(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    history: &MetricHistory,
    theme: &Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Summary & Info
            Constraint::Min(10),   // History Chart
            Constraint::Min(8),    // All Cores Grid
        ])
        .split(area);

    // 1. CPU Summary
    let summary_block = Block::default()
        .title(" CPU Overview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.panel_bg));

    let sum_inner = summary_block.inner(rows[0]);
    frame.render_widget(summary_block, rows[0]);

    let sum_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(sum_inner);

    let gauge = create_utilization_gauge(
        "Total CPU Load",
        telemetry.cpu.overall_usage as f64 / 100.0,
        theme.primary,
        theme.background,
        theme.text,
    );
    frame.render_widget(gauge, sum_cols[0]);

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Processor: ", Style::default().fg(theme.text_muted)),
            Span::styled(&telemetry.cpu.brand, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Cores:     ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{} Physical / {} Logical", telemetry.cpu.physical_core_count, telemetry.cpu.logical_core_count), Style::default().fg(theme.accent)),
            Span::styled("  |  Frequency: ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("{} MHz", telemetry.cpu.frequency_mhz), Style::default().fg(theme.warning)),
        ]),
        Line::from(vec![
            Span::styled("Load Avg:  ", Style::default().fg(theme.text_muted)),
            Span::styled(format!("1m: {:.2}, 5m: {:.2}, 15m: {:.2}", telemetry.cpu.load_average.0, telemetry.cpu.load_average.1, telemetry.cpu.load_average.2), Style::default().fg(theme.success)),
        ]),
    ];
    frame.render_widget(Paragraph::new(info_lines), sum_cols[1]);

    // 2. CPU History Chart
    let chart_data = history.cpu_overall.as_chart_data();
    draw_history_chart(
        frame,
        rows[1],
        "CPU Utilization History (%)",
        &chart_data,
        theme.primary,
        theme.border,
        theme.text,
        100.0,
        "%",
    );

    // 3. Per-Core Grid
    let cores_block = Block::default()
        .title(" Per-Core Telemetry ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let cores_inner = cores_block.inner(rows[2]);
    frame.render_widget(cores_block, rows[2]);

    let core_count = telemetry.cpu.core_usages.len();
    if core_count > 0 {
        // Grid: 4 columns
        let cols_per_row = 4;
        let num_rows = (core_count + cols_per_row - 1) / cols_per_row;
        let row_constraints = vec![Constraint::Length(2); num_rows];
        let grid_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(cores_inner);

        for (core_idx, &usage) in telemetry.cpu.core_usages.iter().enumerate() {
            let r = core_idx / cols_per_row;
            let c = core_idx % cols_per_row;
            if r < grid_rows.len() {
                let row_cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Ratio(1, cols_per_row as u32); cols_per_row])
                    .split(grid_rows[r]);

                if c < row_cols.len() {
                    let label = format!("Core {:02} ({:.0}%)", core_idx + 1, usage);
                    let gauge = create_line_gauge(
                        label,
                        usage as f64 / 100.0,
                        if usage > 80.0 { theme.danger } else { theme.accent },
                        theme.background,
                    );
                    frame.render_widget(gauge, row_cols[c]);
                }
            }
        }
    }
}
