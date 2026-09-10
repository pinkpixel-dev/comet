use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge, LineGauge};

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_rate(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

/// Builds a styled gauge for utilization metrics.
pub fn create_utilization_gauge(
    title: impl Into<ratatui::text::Line<'static>>,
    ratio: f64,
    color: Color,
    bg_color: Color,
    text_color: Color,
) -> Gauge<'static> {
    let percent = (ratio * 100.0).clamp(0.0, 100.0);
    let label = format!("{:.0}%", percent);

    Gauge::default()
        .block(
            Block::default()
                .title(title.into())
                .borders(Borders::NONE)
                .style(Style::default().fg(text_color)),
        )
        .gauge_style(Style::default().fg(color).bg(bg_color))
        .ratio(ratio.clamp(0.0, 1.0))
        .label(label)
}

/// Builds a compact line gauge for small spaces.
pub fn create_line_gauge(
    label: impl Into<ratatui::text::Line<'static>>,
    ratio: f64,
    color: Color,
    bg_color: Color,
) -> LineGauge<'static> {
    LineGauge::default()
        .label(label.into())
        .filled_style(Style::default().fg(color))
        .unfilled_style(Style::default().fg(bg_color))
        .ratio(ratio.clamp(0.0, 1.0))
}
