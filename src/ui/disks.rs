use super::widgets::history_chart::draw_history_chart;
use super::widgets::meter::{create_utilization_gauge, format_bytes, format_rate};
use crate::history::MetricHistory;
use crate::telemetry::TelemetryState;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_disks_tab(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    history: &MetricHistory,
    theme: &Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Overall I/O Summary
            Constraint::Min(10),   // Read & Write Charts side-by-side
            Constraint::Min(8),    // Filesystems Cards
        ])
        .split(area);

    // 1. Overall I/O Summary
    let sum_block = Block::default()
        .title(" Storage & Filesystem Telemetry ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(theme.panel_bg));

    let sum_inner = sum_block.inner(rows[0]);
    frame.render_widget(sum_block, rows[0]);

    let sum_text = vec![
        Line::from(vec![
            Span::styled("Total Read Throughput:  ", Style::default().fg(theme.text_muted)),
            Span::styled(format_rate(telemetry.total_disk_read_rate), Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled("    Total Write Throughput: ", Style::default().fg(theme.text_muted)),
            Span::styled(format_rate(telemetry.total_disk_write_rate), Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(format!("Mounted Volumes: {} active filesystems tracked", telemetry.disks.len()), Style::default().fg(theme.text_muted)),
        ]),
    ];
    frame.render_widget(Paragraph::new(sum_text), sum_inner);

    // 2. Read & Write History Charts
    let chart_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let read_data = history.disk_read_rate.as_chart_data();
    let max_read = read_data.iter().map(|(_, y)| *y).fold(1024.0 * 1024.0, f64::max);
    draw_history_chart(
        frame,
        chart_cols[0],
        "Disk Read Speed",
        &read_data,
        theme.success,
        theme.border,
        theme.text,
        max_read,
        " B/s",
    );

    let write_data = history.disk_write_rate.as_chart_data();
    let max_write = write_data.iter().map(|(_, y)| *y).fold(1024.0 * 1024.0, f64::max);
    draw_history_chart(
        frame,
        chart_cols[1],
        "Disk Write Speed",
        &write_data,
        theme.warning,
        theme.border,
        theme.text,
        max_write,
        " B/s",
    );

    // 3. Filesystems Cards
    let fs_block = Block::default()
        .title(" Mounted Filesystems ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let fs_inner = fs_block.inner(rows[2]);
    frame.render_widget(fs_block, rows[2]);

    let count = telemetry.disks.len().max(1);
    let card_constraints = vec![Constraint::Length(4); count];
    let card_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(card_constraints)
        .split(fs_inner);

    for (i, disk) in telemetry.disks.iter().enumerate() {
        if i < card_rows.len() {
            let label = format!(
                "{} ({}) - {} / {} available",
                disk.mount_point.to_string_lossy(),
                disk.file_system,
                format_bytes(disk.used_bytes),
                format_bytes(disk.total_bytes)
            );
            let gauge = create_utilization_gauge(
                label,
                disk.usage_percent as f64 / 100.0,
                if disk.usage_percent > 85.0 { theme.danger } else { theme.warning },
                theme.background,
                theme.text,
            );
            frame.render_widget(gauge, card_rows[i]);
        }
    }
}
