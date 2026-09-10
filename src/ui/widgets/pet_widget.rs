use crate::pet::{get_cat_sprite, PetState};
use crate::theme::Theme;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_pet_widget(
    frame: &mut Frame,
    area: Rect,
    pet: &PetState,
    theme: &Theme,
    pulse: f32,
) {
    let sprite_lines = get_cat_sprite(pet.mood, pet.is_blinking);
    let mood_label = pet.mood.label();

    let mood_color = match pet.mood {
        crate::pet::PetMood::Sleeping => theme.text_muted,
        crate::pet::PetMood::Happy => theme.success,
        crate::pet::PetMood::Busy => theme.warning,
        crate::pet::PetMood::HighCpu => theme.danger,
        crate::pet::PetMood::Hot => theme.danger,
        crate::pet::PetMood::Concerned => theme.warning,
        crate::pet::PetMood::Excited => theme.primary,
        crate::pet::PetMood::Network => theme.accent,
    };

    let border_color = if pulse > 0.5 && pet.mood != crate::pet::PetMood::Sleeping {
        mood_color
    } else {
        theme.border
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(sprite_lines[0], Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(sprite_lines[1], Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(sprite_lines[2], Style::default().fg(theme.primary)),
        ]),
        Line::from(vec![
            Span::styled(format!(" {} ", mood_label), Style::default().fg(mood_color).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(format!(" {} ", pet.name))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );

    frame.render_widget(widget, area);
}
