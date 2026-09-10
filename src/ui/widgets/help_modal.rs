use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn draw_help_modal(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup_area = centered_rect(60, 70, area);
    frame.render_widget(Clear, popup_area);

    let shortcuts = vec![
        ("1 / o", "Overview Dashboard"),
        ("2 / c", "CPU Details & Core Telemetry"),
        ("3 / g", "GPU Details (NVIDIA NVML)"),
        ("4 / m", "Memory & Swap Telemetry"),
        ("5 / d", "Disks & Filesystem I/O"),
        ("6 / n", "Network Interfaces & Throughput"),
        ("7 / p", "Process Manager & Inspection"),
        ("8 / s", "Sensors & Hardware Temperatures"),
        ("Tab / S-Tab", "Cycle Tabs Forward / Backward"),
        ("t", "Quick-Cycle 9 Themes"),
        ("T", "Open Theme Palette Picker"),
        ("P", "Open Pet Companion Panel"),
        ("j / k", "Navigate Process Table (Down / Up)"),
        ("/", "Filter / Search Processes"),
        ("? / h", "Toggle This Help Screen"),
        ("q / Esc", "Quit Comet / Close Modal"),
    ];

    let mut lines = vec![
        Line::from(vec![
            Span::styled("COMET SHORTCUTS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    for (key, desc) in shortcuts {
        lines.push(Line::from(vec![
            Span::styled(format!("{:>14}  ", key), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(desc, Style::default().fg(theme.text)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Press ", Style::default().fg(theme.text_muted)),
        Span::styled("Esc", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(theme.text_muted)),
        Span::styled("?", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
        Span::styled(" to close", Style::default().fg(theme.text_muted)),
    ]));

    let block = Block::default()
        .title(" Help & Keybindings ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.panel_bg));

    let paragraph = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, popup_area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
