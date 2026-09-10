use crate::telemetry::{ProcessMetrics, ProcessSignal};
use crate::theme::Theme;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone)]
pub struct SignalModalState {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
    pub signal: ProcessSignal,
}

impl SignalModalState {
    pub fn new(process: &ProcessMetrics) -> Self {
        Self {
            pid: process.pid,
            name: process.name.clone(),
            cmd: process.cmd.clone(),
            signal: ProcessSignal::Term,
        }
    }

    pub fn toggle_signal(&mut self) {
        self.signal = match self.signal {
            ProcessSignal::Term => ProcessSignal::Kill,
            ProcessSignal::Kill => ProcessSignal::Term,
        };
    }

    pub fn select_term(&mut self) {
        self.signal = ProcessSignal::Term;
    }

    pub fn select_kill(&mut self) {
        self.signal = ProcessSignal::Kill;
    }
}

pub fn draw_signal_modal(
    frame: &mut Frame,
    area: Rect,
    state: &SignalModalState,
    theme: &Theme,
) {
    let popup_area = centered_box(56, 14, area);
    frame.render_widget(Clear, popup_area);

    let is_term = state.signal == ProcessSignal::Term;
    let is_kill = state.signal == ProcessSignal::Kill;

    let term_style = if is_term {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let kill_style = if is_kill {
        Style::default()
            .fg(theme.warning)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };

    let truncated_cmd = if state.cmd.is_empty() {
        "[kernel or system process]".to_string()
    } else if state.cmd.len() > 46 {
        format!("{}...", &state.cmd[..43])
    } else {
        state.cmd.clone()
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(theme.text_muted)),
            Span::styled(&state.name, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" (PID: {})", state.pid), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("Command: ", Style::default().fg(theme.text_muted)),
            Span::styled(truncated_cmd, Style::default().fg(theme.text_muted)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(if is_term { " [*] " } else { " [ ] " }, term_style),
            Span::styled("1: SIGTERM (15)", term_style),
            Span::styled(" - Request graceful termination", Style::default().fg(theme.text)),
        ]),
        Line::from(vec![
            Span::styled(if is_kill { " [*] " } else { " [ ] " }, kill_style),
            Span::styled("2: SIGKILL (9) ", kill_style),
            Span::styled(" - Force immediate termination (unblockable)", Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions: ", Style::default().fg(theme.text_muted)),
            Span::styled("[1/2/Tab]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" Select Signal  ", Style::default().fg(theme.text_muted)),
            Span::styled("[y / Enter]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(" Confirm  ", Style::default().fg(theme.text_muted)),
            Span::styled("[n / Esc]", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(" Cancel", Style::default().fg(theme.text_muted)),
        ]),
    ];

    let border_color = if is_kill { theme.warning } else { theme.primary };

    let block = Block::default()
        .title(" Send Process Signal ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.panel_bg));

    let paragraph = Paragraph::new(lines).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, popup_area);
}

fn centered_box(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width.saturating_sub(2));
    let h = height.min(area.height.saturating_sub(2));

    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;

    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_modal_state_transitions() {
        let proc = ProcessMetrics {
            pid: 1234,
            parent_pid: Some(1),
            name: "test_proc".to_string(),
            cpu_usage: 12.5,
            memory_bytes: 4096,
            read_bytes: 0,
            written_bytes: 0,
            status: "Running".to_string(),
            cmd: "test_proc --arg".to_string(),
        };

        let mut modal = SignalModalState::new(&proc);
        assert_eq!(modal.pid, 1234);
        assert_eq!(modal.signal, ProcessSignal::Term);

        modal.toggle_signal();
        assert_eq!(modal.signal, ProcessSignal::Kill);

        modal.toggle_signal();
        assert_eq!(modal.signal, ProcessSignal::Term);

        modal.select_kill();
        assert_eq!(modal.signal, ProcessSignal::Kill);

        modal.select_term();
        assert_eq!(modal.signal, ProcessSignal::Term);
    }
}
