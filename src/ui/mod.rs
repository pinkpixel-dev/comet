pub mod cpu;
pub mod disks;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod overview;
pub mod processes;
pub mod sensors;
pub mod widgets;

use crate::animation::AnimationState;
use crate::history::MetricHistory;
use crate::pet::PetState;
use crate::telemetry::{ProcessSortBy, TelemetryState};
use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;
use widgets::help_modal::draw_help_modal;
use widgets::pet_modal::draw_pet_modal;
use widgets::signal_modal::{draw_signal_modal, SignalModalState};
use widgets::theme_modal::draw_theme_modal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Cpu,
    Gpu,
    Memory,
    Disks,
    Network,
    Processes,
    Sensors,
}

impl Tab {
    pub const ALL: [Tab; 8] = [
        Tab::Overview,
        Tab::Cpu,
        Tab::Gpu,
        Tab::Memory,
        Tab::Disks,
        Tab::Network,
        Tab::Processes,
        Tab::Sensors,
    ];

    pub fn titles() -> Vec<&'static str> {
        vec![
            "1:Overview",
            "2:CPU",
            "3:GPU",
            "4:Memory",
            "5:Disks",
            "6:Network",
            "7:Processes",
            "8:Sensors",
        ]
    }

    pub fn index(self) -> usize {
        match self {
            Tab::Overview => 0,
            Tab::Cpu => 1,
            Tab::Gpu => 2,
            Tab::Memory => 3,
            Tab::Disks => 4,
            Tab::Network => 5,
            Tab::Processes => 6,
            Tab::Sensors => 7,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx % 8 {
            0 => Tab::Overview,
            1 => Tab::Cpu,
            2 => Tab::Gpu,
            3 => Tab::Memory,
            4 => Tab::Disks,
            5 => Tab::Network,
            6 => Tab::Processes,
            _ => Tab::Sensors,
        }
    }

    pub fn next(self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn prev(self) -> Self {
        Self::from_index(self.index() + 7)
    }
}

pub fn draw_ui(
    frame: &mut Frame,
    telemetry: &TelemetryState,
    history: &MetricHistory,
    current_tab: Tab,
    theme: &Theme,
    anim: &AnimationState,
    pet: &PetState,
    selected_gpu_index: usize,
    selected_proc_idx: usize,
    proc_sort: ProcessSortBy,
    tree_mode: bool,
    search_query: &str,
    is_searching: bool,
    show_help: bool,
    show_theme_picker: bool,
    show_pet_panel: bool,
    signal_modal: Option<&SignalModalState>,
    status_message: Option<&str>,
    recording_info: Option<(usize, std::time::Duration)>,
) {
    let size = frame.area();

    // Fill overall background
    let bg_block = Block::default().style(Style::default().bg(theme.background));
    frame.render_widget(bg_block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Tabs
            Constraint::Min(10),   // Active Tab Body
            Constraint::Length(1), // Footer Status Bar
        ])
        .split(size);

    // 1. Header (Comet Title + Tabs + Recording Badge + Host & Uptime)
    draw_header(frame, chunks[0], current_tab, telemetry, theme, recording_info);

    // 2. Main Tab View
    match current_tab {
        Tab::Overview => {
            overview::draw_overview(frame, chunks[1], telemetry, history, pet, theme, anim);
        }
        Tab::Cpu => {
            cpu::draw_cpu_tab(frame, chunks[1], telemetry, history, theme);
        }
        Tab::Gpu => {
            gpu::draw_gpu_tab(frame, chunks[1], telemetry, history, selected_gpu_index, theme);
        }
        Tab::Memory => {
            memory::draw_memory_tab(frame, chunks[1], telemetry, history, theme);
        }
        Tab::Disks => {
            disks::draw_disks_tab(frame, chunks[1], telemetry, history, theme);
        }
        Tab::Network => {
            network::draw_network_tab(frame, chunks[1], telemetry, history, theme);
        }
        Tab::Processes => {
            processes::draw_processes_tab(
                frame,
                chunks[1],
                telemetry,
                selected_proc_idx,
                proc_sort,
                tree_mode,
                search_query,
                is_searching,
                status_message,
                theme,
            );
        }
        Tab::Sensors => {
            sensors::draw_sensors_tab(frame, chunks[1], telemetry, theme);
        }
    }

    // 3. Footer Status Bar
    draw_footer(frame, chunks[2], telemetry, selected_gpu_index, theme);

    // Modals
    if let Some(signal_state) = signal_modal {
        draw_signal_modal(frame, size, signal_state, theme);
    } else if show_help {
        draw_help_modal(frame, size, theme);
    } else if show_theme_picker {
        draw_theme_modal(frame, size, theme);
    } else if show_pet_panel {
        draw_pet_modal(frame, size, pet, theme);
    }
}

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    current_tab: Tab,
    telemetry: &TelemetryState,
    theme: &Theme,
    recording_info: Option<(usize, std::time::Duration)>,
) {
    let header_cols = if recording_info.is_some() {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Title
                Constraint::Min(30),    // Tabs
                Constraint::Length(22), // Recording Indicator
                Constraint::Length(32), // Host & Uptime
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Title
                Constraint::Min(40),    // Tabs
                Constraint::Length(32), // Host & Uptime
            ])
            .split(area)
    };

    // Title
    let title_line = Line::from(vec![
        Span::styled("☄ ", Style::default().fg(theme.primary)),
        Span::styled("COMET", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
    ]);
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.panel_bg));
    let t_inner = title_block.inner(header_cols[0]);
    frame.render_widget(title_block, header_cols[0]);
    frame.render_widget(Paragraph::new(title_line).alignment(Alignment::Center), t_inner);

    // Tabs
    let tab_titles = Tab::titles();
    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.panel_bg)),
        )
        .select(current_tab.index())
        .style(Style::default().fg(theme.text_muted))
        .highlight_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .divider(Span::styled(" | ", Style::default().fg(theme.border)));
    frame.render_widget(tabs, header_cols[1]);

    // Optional Recording Badge
    let host_col_idx = if let Some((count, elapsed)) = recording_info {
        let secs = elapsed.as_secs();
        let rec_text = format!("● REC {:02}:{:02} ({})", secs / 60, secs % 60, count);
        let rec_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.danger))
            .style(Style::default().bg(theme.panel_bg));
        let r_inner = rec_block.inner(header_cols[2]);
        frame.render_widget(rec_block, header_cols[2]);
        frame.render_widget(
            Paragraph::new(rec_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.danger).add_modifier(Modifier::BOLD)),
            r_inner,
        );
        3
    } else {
        2
    };

    // Host & Uptime
    let host_text = format!("Host: {} | Up: {}", telemetry.system.host_name, telemetry.system.formatted_uptime());
    let host_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg));
    let h_inner = host_block.inner(header_cols[host_col_idx]);
    frame.render_widget(host_block, header_cols[host_col_idx]);
    frame.render_widget(
        Paragraph::new(host_text).alignment(Alignment::Right).style(Style::default().fg(theme.text_muted)),
        h_inner,
    );
}

fn draw_footer(
    frame: &mut Frame,
    area: Rect,
    telemetry: &TelemetryState,
    selected_gpu_index: usize,
    theme: &Theme,
) {
    let active_gpu = telemetry
        .gpus
        .get(selected_gpu_index)
        .or(telemetry.gpu.as_ref());
    let gpu_status = match active_gpu {
        Some(g) => {
            let multi = if telemetry.gpus.len() > 1 {
                format!(" ({}/{})", selected_gpu_index + 1, telemetry.gpus.len())
            } else {
                String::new()
            };
            format!("GPU{}: {} [{}]", multi, g.vendor, g.driver)
        }
        None => "GPU: None".to_string(),
    };
    let footer_spans = vec![
        Span::styled(" [1-8] Tabs ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("| [r] Record | [t] Themes | [P] Pet | [?] Help | [q] Quit ", Style::default().fg(theme.text_muted)),
        Span::styled(format!(" | Theme: {} | {} ", theme.name, gpu_status), Style::default().fg(theme.accent)),
    ];
    let footer = Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center);
    frame.render_widget(footer, area);
}
