use super::widgets::meter::format_bytes;
use crate::telemetry::{ProcessMetrics, ProcessSortBy, TelemetryState};
use crate::theme::Theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

pub fn draw_processes_tab(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    selected_idx: usize,
    sort_by: ProcessSortBy,
    search_query: &str,
    is_searching: bool,
    theme: &Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search & Filter Header
            Constraint::Min(10),  // Process Table
            Constraint::Length(5), // Process Detail Panel
        ])
        .split(area);

    // 1. Search / Filter Header
    let sort_label = match sort_by {
        ProcessSortBy::Cpu => "CPU%",
        ProcessSortBy::Memory => "Memory",
        ProcessSortBy::Pid => "PID",
        ProcessSortBy::Name => "Name",
    };

    let search_border = if is_searching {
        theme.primary
    } else {
        theme.border
    };

    let header_lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(if search_query.is_empty() && !is_searching { "Press '/' to search..." } else { search_query }, Style::default().fg(theme.text)),
            Span::styled(if is_searching { " █" } else { "" }, Style::default().fg(theme.primary)),
            Span::styled(format!("    Sort: [{}] (press 's' to toggle)    Total Processes: {}", sort_label, telemetry.processes.len()), Style::default().fg(theme.text_muted)),
        ]),
    ];
    let search_block = Block::default()
        .title(" Process Filter & Options ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(search_border))
        .style(Style::default().bg(theme.panel_bg));
    let s_inner = search_block.inner(rows[0]);
    frame.render_widget(search_block, rows[0]);
    frame.render_widget(Paragraph::new(header_lines), s_inner);

    // Filter processes by query
    let filtered_procs: Vec<&ProcessMetrics> = telemetry
        .processes
        .iter()
        .filter(|p| {
            if search_query.is_empty() {
                true
            } else {
                p.name.to_lowercase().contains(&search_query.to_lowercase())
                    || p.pid.to_string().contains(search_query)
            }
        })
        .collect();

    // 2. Table
    let table_block = Block::default()
        .title(format!(" Processes ({}) ", filtered_procs.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let t_inner = table_block.inner(rows[1]);
    frame.render_widget(table_block, rows[1]);

    let header_row = Row::new(vec!["PID", "Process Name", "CPU%", "RAM", "Read I/O", "Write I/O", "Status"])
        .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

    let rows_items: Vec<Row> = filtered_procs
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = i == selected_idx;
            let style = if is_selected {
                Style::default().fg(theme.background).bg(theme.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            Row::new(vec![
                p.pid.to_string(),
                p.name.clone(),
                format!("{:.1}%", p.cpu_usage),
                format_bytes(p.memory_bytes),
                format_bytes(p.read_bytes),
                format_bytes(p.written_bytes),
                p.status.clone(),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows_items,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(header_row);

    frame.render_widget(table, t_inner);

    // 3. Process Detail Panel for selected process
    let detail_block = Block::default()
        .title(" Selected Process Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));

    let d_inner = detail_block.inner(rows[2]);
    frame.render_widget(detail_block, rows[2]);

    if let Some(selected_proc) = filtered_procs.get(selected_idx) {
        let detail_text = vec![
            Line::from(vec![
                Span::styled("Process: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("{} (PID: {})", selected_proc.name, selected_proc.pid), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled("   CPU: ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("{:.1}%", selected_proc.cpu_usage), Style::default().fg(theme.accent)),
                Span::styled("   RAM: ", Style::default().fg(theme.text_muted)),
                Span::styled(format_bytes(selected_proc.memory_bytes), Style::default().fg(theme.secondary)),
            ]),
            Line::from(vec![
                Span::styled("Command: ", Style::default().fg(theme.text_muted)),
                Span::styled(if selected_proc.cmd.is_empty() { "[Kernel or system process]" } else { &selected_proc.cmd }, Style::default().fg(theme.text)),
            ]),
        ];
        frame.render_widget(Paragraph::new(detail_text), d_inner);
    } else {
        frame.render_widget(Paragraph::new("No process selected").style(Style::default().fg(theme.text_muted)), d_inner);
    }
}
