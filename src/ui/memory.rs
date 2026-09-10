use super::widgets::history_chart::draw_history_chart;
use super::widgets::meter::{create_utilization_gauge, format_bytes};
use crate::history::MetricHistory;
use crate::telemetry::TelemetryState;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_memory_tab(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    history: &MetricHistory,
    theme: &Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Summary Gauges
            Constraint::Min(10),  // RAM History Chart
            Constraint::Length(7), // Memory Details breakdown
        ])
        .split(area);

    // 1. Gauges
    let sum_block = Block::default()
        .title(" Memory Overview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.secondary))
        .style(Style::default().bg(theme.panel_bg));

    let sum_inner = sum_block.inner(rows[0]);
    frame.render_widget(sum_block, rows[0]);

    let sum_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sum_inner);

    let ram_label = format!(
        "RAM Utilization ({})",
        format_bytes(telemetry.memory.used_bytes)
    );
    let ram_gauge = create_utilization_gauge(
        ram_label,
        telemetry.memory.usage_percent() as f64 / 100.0,
        theme.secondary,
        theme.background,
        theme.text,
    );
    frame.render_widget(ram_gauge, sum_cols[0]);

    let swap_label = format!(
        "Swap Utilization ({})",
        format_bytes(telemetry.memory.swap_used_bytes)
    );
    let swap_gauge = create_utilization_gauge(
        swap_label,
        telemetry.memory.swap_usage_percent() as f64 / 100.0,
        theme.accent,
        theme.background,
        theme.text,
    );
    frame.render_widget(swap_gauge, sum_cols[1]);

    // 2. RAM History Chart
    let chart_data = history.ram_usage_percent.as_chart_data();
    draw_history_chart(
        frame,
        rows[1],
        "RAM Usage History (%)",
        &chart_data,
        theme.secondary,
        theme.border,
        theme.text,
        100.0,
        "%",
    );

    // 3. Detailed breakdown
    let detail_block = Block::default()
        .title(" Memory Allocation Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let detail_inner = detail_block.inner(rows[2]);
    frame.render_widget(detail_block, rows[2]);

    let d_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(detail_inner);

    let ram_details = vec![
        Line::from(vec![
            Span::styled("Total Physical RAM: ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.memory.total_bytes), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Active / In-Use:   ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.memory.used_bytes), Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({:.1}%)", telemetry.memory.usage_percent()), Style::default().fg(theme.text_muted)),
        ]),
        Line::from(vec![
            Span::styled("Available Memory:  ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.memory.available_bytes), Style::default().fg(theme.success)),
        ]),
    ];
    frame.render_widget(Paragraph::new(ram_details), d_cols[0]);

    let swap_details = vec![
        Line::from(vec![
            Span::styled("Total Swap Space:  ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.memory.swap_total_bytes), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Used Swap Space:   ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.memory.swap_used_bytes), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({:.1}%)", telemetry.memory.swap_usage_percent()), Style::default().fg(theme.text_muted)),
        ]),
        Line::from(vec![
            Span::styled("Free Swap Space:   ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.memory.swap_free_bytes), Style::default().fg(theme.success)),
        ]),
    ];
    frame.render_widget(Paragraph::new(swap_details), d_cols[1]);
}
