use super::widgets::history_chart::draw_history_chart;
use super::widgets::meter::{format_bytes, format_rate};
use crate::history::MetricHistory;
use crate::telemetry::TelemetryState;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw_network_tab(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    history: &MetricHistory,
    theme: &Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Summary Rates
            Constraint::Min(10),  // RX / TX Charts
            Constraint::Min(8),   // Network Interfaces Table
        ])
        .split(area);

    // 1. Throughput summary
    let sum_block = Block::default()
        .title(" Network Traffic Summary ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.panel_bg));

    let sum_inner = sum_block.inner(rows[0]);
    frame.render_widget(sum_block, rows[0]);

    let sum_text = vec![
        Line::from(vec![
            Span::styled(" ↓ Download (RX): ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled(format_rate(telemetry.network.total_rx_rate), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("   Total RX: ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.network.total_rx_bytes), Style::default().fg(theme.accent)),
            Span::styled("      ↑ Upload (TX): ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(format_rate(telemetry.network.total_tx_rate), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("   Total TX: ", Style::default().fg(theme.text_muted)),
            Span::styled(format_bytes(telemetry.network.total_tx_bytes), Style::default().fg(theme.accent)),
        ]),
    ];
    frame.render_widget(Paragraph::new(sum_text), sum_inner);

    // 2. Dual Charts
    let chart_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let rx_data = history.net_rx_rate.as_chart_data();
    let max_rx = rx_data.iter().map(|(_, y)| *y).fold(1024.0 * 1024.0, f64::max);
    draw_history_chart(
        frame,
        chart_cols[0],
        "Download Throughput (RX)",
        &rx_data,
        theme.success,
        theme.border,
        theme.text,
        max_rx,
        " B/s",
    );

    let tx_data = history.net_tx_rate.as_chart_data();
    let max_tx = tx_data.iter().map(|(_, y)| *y).fold(1024.0 * 1024.0, f64::max);
    draw_history_chart(
        frame,
        chart_cols[1],
        "Upload Throughput (TX)",
        &tx_data,
        theme.warning,
        theme.border,
        theme.text,
        max_tx,
        " B/s",
    );

    // 3. Interfaces Table
    let iface_block = Block::default()
        .title(" Network Interfaces ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let iface_inner = iface_block.inner(rows[2]);
    frame.render_widget(iface_block, rows[2]);

    let header = Row::new(vec!["Interface", "Current RX", "Current TX", "Total RX", "Total TX"])
        .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

    let rows_items: Vec<Row> = telemetry
        .network
        .interfaces
        .iter()
        .map(|iface| {
            Row::new(vec![
                iface.name.clone(),
                format_rate(iface.rx_bytes_per_sec),
                format_rate(iface.tx_bytes_per_sec),
                format_bytes(iface.total_rx_bytes),
                format_bytes(iface.total_tx_bytes),
            ])
            .style(Style::default().fg(theme.text))
        })
        .collect();

    let table = Table::new(
        rows_items,
        [
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
        ],
    )
    .header(header);

    frame.render_widget(table, iface_inner);
}
