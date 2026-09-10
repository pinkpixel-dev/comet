use super::help_modal::centered_rect;
use crate::theme::{Theme, ThemeId};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn draw_theme_modal(frame: &mut Frame, area: Rect, current_theme: &Theme) {
    let popup_area = centered_rect(50, 60, area);
    frame.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("SELECT THEME", Style::default().fg(current_theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    for (i, id) in ThemeId::ALL.iter().enumerate() {
        let theme_preview = Theme::get(*id);
        let is_active = theme_preview.id == current_theme.id;

        let prefix = if is_active { " ► " } else { "   " };
        let style = if is_active {
            Style::default().fg(theme_preview.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(current_theme.text)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{}. ", i + 1), Style::default().fg(current_theme.text_muted)),
            Span::styled(prefix, Style::default().fg(theme_preview.accent).add_modifier(Modifier::BOLD)),
            Span::styled(theme_preview.name, style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Press ", Style::default().fg(current_theme.text_muted)),
        Span::styled("t", Style::default().fg(current_theme.warning).add_modifier(Modifier::BOLD)),
        Span::styled(" to cycle or ", Style::default().fg(current_theme.text_muted)),
        Span::styled("Esc", Style::default().fg(current_theme.warning).add_modifier(Modifier::BOLD)),
        Span::styled(" to close", Style::default().fg(current_theme.text_muted)),
    ]));

    let block = Block::default()
        .title(" Themes ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(current_theme.primary))
        .style(Style::default().bg(current_theme.panel_bg));

    let p = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    frame.render_widget(p, popup_area);
}
