use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use ratatui::Frame;

pub fn draw_history_chart(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    data: &[(f64, f64)],
    color: Color,
    border_color: Color,
    text_color: Color,
    max_y: f64,
    y_unit: &str,
) {
    let max_x = (data.len() as f64).max(60.0);
    let effective_max_y = if max_y <= 0.0 { 100.0 } else { max_y };

    let dataset = Dataset::default()
        .name(title)
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(data);

    let y_labels = vec![
        "0".to_string(),
        format!("{:.0}{}", effective_max_y * 0.5, y_unit),
        format!("{:.0}{}", effective_max_y, y_unit),
    ];

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().fg(text_color)),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, max_x])
                .style(Style::default().fg(Color::DarkGray)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, effective_max_y])
                .labels(y_labels)
                .style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(chart, area);
}
