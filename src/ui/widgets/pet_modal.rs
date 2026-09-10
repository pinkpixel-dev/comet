use super::help_modal::centered_rect;
use crate::pet::{get_cat_sprite, PetState};
use crate::theme::Theme;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn draw_pet_modal(frame: &mut Frame, area: Rect, pet: &PetState, theme: &Theme) {
    let popup_area = centered_rect(48, 55, area);
    frame.render_widget(Clear, popup_area);

    let sprites = get_cat_sprite(pet.mood, pet.is_blinking);

    let lines = vec![
        Line::from(vec![
            Span::styled(sprites[0], Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(sprites[1], Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(sprites[2], Style::default().fg(theme.primary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Name:       ", Style::default().fg(theme.text_muted)),
            Span::styled(&pet.name, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Mood:       ", Style::default().fg(theme.text_muted)),
            Span::styled(pet.mood.label(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Favorite:   ", Style::default().fg(theme.text_muted)),
            Span::styled(pet.favorite, Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Role:       ", Style::default().fg(theme.text_muted)),
            Span::styled("Telemetry Guardian Cat", Style::default().fg(theme.primary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.text_muted)),
            Span::styled("Esc", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(" or ", Style::default().fg(theme.text_muted)),
            Span::styled("P", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(" to close", Style::default().fg(theme.text_muted)),
        ]),
    ];

    let block = Block::default()
        .title(" Pet Companion ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.panel_bg));

    let p = Paragraph::new(lines).block(block).alignment(Alignment::Center);
    frame.render_widget(p, popup_area);
}
