use super::widgets::meter::create_line_gauge;
use crate::telemetry::TelemetryState;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_sensors_tab(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    theme: &Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Summary Banner
            Constraint::Min(10),  // Sensors Grid
        ])
        .split(area);

    // 1. Thermal summary
    let sum_block = Block::default()
        .title(" Thermal Sensors Overview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.danger))
        .style(Style::default().bg(theme.panel_bg));

    let sum_inner = sum_block.inner(rows[0]);
    frame.render_widget(sum_block, rows[0]);

    let max_temp_str = telemetry
        .sensors
        .max_temperature
        .map(|t| format!("{:.1}°C", t))
        .unwrap_or_else(|| "N/A".to_string());

    let sum_text = vec![
        Line::from(vec![
            Span::styled("Max Hardware Temperature: ", Style::default().fg(theme.text_muted)),
            Span::styled(max_temp_str, Style::default().fg(theme.danger).add_modifier(Modifier::BOLD)),
            Span::styled(format!("    Active Sensors: {}", telemetry.sensors.items.len()), Style::default().fg(theme.accent)),
        ]),
    ];
    frame.render_widget(Paragraph::new(sum_text), sum_inner);

    // 2. Sensors Grid
    let grid_block = Block::default()
        .title(" Sensor Components ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let grid_inner = grid_block.inner(rows[1]);
    frame.render_widget(grid_block, rows[1]);

    let count = telemetry.sensors.items.len();
    if count == 0 {
        frame.render_widget(
            Paragraph::new("No temperature sensors detected on this system.").style(Style::default().fg(theme.text_muted)),
            grid_inner,
        );
        return;
    }

    // Two columns of sensor gauges
    let cols_per_row = 2;
    let num_rows = (count + cols_per_row - 1) / cols_per_row;
    let row_constraints = vec![Constraint::Length(3); num_rows];

    let grid_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_inner);

    for (idx, sensor) in telemetry.sensors.items.iter().enumerate() {
        let r = idx / cols_per_row;
        let c = idx % cols_per_row;
        if r < grid_rows.len() {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(grid_rows[r]);

            if c < cols.len() {
                let color = if sensor.temperature >= 80.0 {
                    theme.danger
                } else if sensor.temperature >= 60.0 {
                    theme.warning
                } else {
                    theme.success
                };

                let label = format!("{} ({:.1}°C)", sensor.label, sensor.temperature);
                let ratio = (sensor.temperature as f64 / 100.0).clamp(0.0, 1.0);
                let gauge = create_line_gauge(label, ratio, color, theme.background);
                frame.render_widget(gauge, cols[c]);
            }
        }
    }
}
