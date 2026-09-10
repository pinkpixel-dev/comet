use crate::animation::AnimationState;
use crate::config::Config;
use crate::history::MetricHistory;
use crate::pet::PetState;
use crate::telemetry::{
    build_process_tree, send_signal, ProcessMetrics, ProcessSortBy, TelemetryState,
};
use crate::theme::Theme;
use crate::ui::widgets::signal_modal::SignalModalState;
use crate::ui::{draw_ui, Tab};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use std::time::{Duration, Instant};

pub struct App {
    pub tab: Tab,
    pub telemetry: TelemetryState,
    pub history: MetricHistory,
    pub animation: AnimationState,
    pub theme: Theme,
    pub pet: PetState,
    pub selected_proc_idx: usize,
    pub proc_sort: ProcessSortBy,
    pub tree_mode: bool,
    pub signal_modal: Option<SignalModalState>,
    pub status_message: Option<(String, Instant)>,
    pub search_query: String,
    pub is_searching: bool,
    pub show_help: bool,
    pub show_theme_picker: bool,
    pub show_pet_panel: bool,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

impl App {
    pub fn new(config: &Config) -> Self {
        let mut pet = PetState::default();
        pet.name = config.pet_name.clone();
        pet.visible = config.show_pet;

        Self {
            tab: config.parsed_tab(),
            telemetry: TelemetryState::default(),
            history: MetricHistory::new(config.history_capacity.max(30)),
            animation: AnimationState::default(),
            theme: Theme::get(config.parsed_theme()),
            pet,
            selected_proc_idx: 0,
            proc_sort: ProcessSortBy::Cpu,
            tree_mode: false,
            signal_modal: None,
            status_message: None,
            search_query: String::new(),
            is_searching: false,
            show_help: false,
            show_theme_picker: false,
            show_pet_panel: false,
            should_quit: false,
        }
    }

    pub fn on_tick(&mut self) {
        self.animation.tick();

        // Expire status message after 4 seconds
        if let Some((_, timestamp)) = self.status_message {
            if timestamp.elapsed() > Duration::from_secs(4) {
                self.status_message = None;
            }
        }

        // Update pet mood with latest telemetry
        let max_temp = self.telemetry.sensors.max_temperature.or_else(|| {
            self.telemetry.gpu.as_ref().and_then(|g| g.temperature)
        });
        let gpu_util = self.telemetry.gpu.as_ref().map(|g| g.utilization);
        let net_throughput = self.telemetry.network.total_rx_rate + self.telemetry.network.total_tx_rate;

        self.pet.update(
            self.telemetry.cpu.overall_usage,
            self.telemetry.memory.usage_percent(),
            gpu_util,
            max_temp,
            net_throughput,
            self.animation.delta,
        );
    }

    pub fn on_telemetry(&mut self, state: TelemetryState) {
        // Record into history ring buffers
        self.history.cpu_overall.push(state.cpu.overall_usage as f64);
        self.history.ram_usage_percent.push(state.memory.usage_percent() as f64);
        self.history.ram_used_bytes.push(state.memory.used_bytes as f64);
        self.history.swap_usage_percent.push(state.memory.swap_usage_percent() as f64);

        if let Some(gpu) = &state.gpu {
            self.history.gpu_utilization.push(gpu.utilization as f64);
            let vram_pct = if gpu.memory_total > 0 {
                (gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0
            } else {
                0.0
            };
            self.history.gpu_vram_percent.push(vram_pct);
        }

        self.history.net_rx_rate.push(state.network.total_rx_rate);
        self.history.net_tx_rate.push(state.network.total_tx_rate);
        self.history.disk_read_rate.push(state.total_disk_read_rate);
        self.history.disk_write_rate.push(state.total_disk_write_rate);

        self.telemetry = state;
        self.clamp_selected_proc();
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        // Handle search typing if active in processes tab
        if self.is_searching {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.is_searching = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.clamp_selected_proc();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.clamp_selected_proc();
                }
                _ => {}
            }
            return;
        }

        // Handle process signal confirmation dialog
        if let Some(modal) = &mut self.signal_modal {
            match key.code {
                KeyCode::Char('1') => {
                    modal.select_term();
                    return;
                }
                KeyCode::Char('2') => {
                    modal.select_kill();
                    return;
                }
                KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                    modal.toggle_signal();
                    return;
                }
                KeyCode::Char('y') | KeyCode::Enter => {
                    let pid = modal.pid;
                    let name = modal.name.clone();
                    let sig = modal.signal;
                    self.signal_modal = None;

                    match send_signal(pid, sig) {
                        Ok(()) => {
                            self.status_message = Some((
                                format!("✔ Sent {} to {} (PID: {})", sig.short_name(), name, pid),
                                Instant::now(),
                            ));
                        }
                        Err(err) => {
                            self.status_message = Some((
                                format!("✖ Failed to send {} to PID {}: {}", sig.short_name(), pid, err),
                                Instant::now(),
                            ));
                        }
                    }
                    return;
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
                    self.signal_modal = None;
                    return;
                }
                _ => return,
            }
        }

        // Handle overlay modal dismissal
        if self.show_help || self.show_theme_picker || self.show_pet_panel {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') => {
                    self.show_help = false;
                    self.show_theme_picker = false;
                    self.show_pet_panel = false;
                    return;
                }
                KeyCode::Char('t') => {
                    self.theme = Theme::get(self.theme.id.next());
                    return;
                }
                KeyCode::Char('P') => {
                    self.show_pet_panel = false;
                    return;
                }
                _ => {}
            }
        }

        // Global shortcuts
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('?') | KeyCode::Char('h') => {
                self.show_help = !self.show_help;
            }
            KeyCode::Char('t') => {
                if self.tab == Tab::Processes {
                    self.tree_mode = !self.tree_mode;
                    self.clamp_selected_proc();
                } else {
                    self.theme = Theme::get(self.theme.id.next());
                }
            }
            KeyCode::Char('x') | KeyCode::Char('K') => {
                if self.tab == Tab::Processes {
                    if let Some(proc) = self.get_selected_process() {
                        self.signal_modal = Some(SignalModalState::new(&proc));
                    }
                }
            }
            KeyCode::Char('T') => {
                self.show_theme_picker = !self.show_theme_picker;
            }
            KeyCode::Char('P') => {
                self.show_pet_panel = !self.show_pet_panel;
            }
            // Direct tab jump numbers
            KeyCode::Char('1') | KeyCode::Char('o') => self.tab = Tab::Overview,
            KeyCode::Char('2') | KeyCode::Char('c') => self.tab = Tab::Cpu,
            KeyCode::Char('3') | KeyCode::Char('g') => self.tab = Tab::Gpu,
            KeyCode::Char('4') | KeyCode::Char('m') => self.tab = Tab::Memory,
            KeyCode::Char('5') | KeyCode::Char('d') => self.tab = Tab::Disks,
            KeyCode::Char('6') | KeyCode::Char('n') => self.tab = Tab::Network,
            KeyCode::Char('7') | KeyCode::Char('p') => self.tab = Tab::Processes,
            KeyCode::Char('8') | KeyCode::Char('s') => {
                if self.tab == Tab::Processes {
                    // In processes tab, 's' toggles sort column
                    self.cycle_process_sort();
                } else {
                    self.tab = Tab::Sensors;
                }
            }
            // Tab switching
            KeyCode::Tab => self.tab = self.tab.next(),
            KeyCode::BackTab => self.tab = self.tab.prev(),
            // Navigation
            KeyCode::Down | KeyCode::Char('j') => {
                if self.tab == Tab::Processes {
                    let total = self.current_process_count();
                    if total > 0 && self.selected_proc_idx + 1 < total {
                        self.selected_proc_idx += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.tab == Tab::Processes && self.selected_proc_idx > 0 {
                    self.selected_proc_idx -= 1;
                }
            }
            KeyCode::Char('/') => {
                if self.tab == Tab::Processes {
                    self.is_searching = true;
                }
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.clamp_selected_proc();
                }
            }
            _ => {}
        }
    }

    pub fn get_selected_process(&self) -> Option<ProcessMetrics> {
        let filtered: Vec<ProcessMetrics> = self
            .telemetry
            .processes
            .iter()
            .filter(|p| {
                if self.search_query.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&self.search_query.to_lowercase())
                        || p.pid.to_string().contains(&self.search_query)
                }
            })
            .cloned()
            .collect();

        if self.tree_mode {
            let tree = build_process_tree(&filtered, self.proc_sort);
            tree.get(self.selected_proc_idx).map(|item| item.process.clone())
        } else {
            filtered.get(self.selected_proc_idx).cloned()
        }
    }

    pub fn current_process_count(&self) -> usize {
        if self.search_query.is_empty() {
            self.telemetry.processes.len()
        } else {
            self.telemetry
                .processes
                .iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&self.search_query.to_lowercase())
                        || p.pid.to_string().contains(&self.search_query)
                })
                .count()
        }
    }

    pub fn clamp_selected_proc(&mut self) {
        let total = self.current_process_count();
        if total == 0 {
            self.selected_proc_idx = 0;
        } else if self.selected_proc_idx >= total {
            self.selected_proc_idx = total.saturating_sub(1);
        }
    }

    fn cycle_process_sort(&mut self) {
        self.proc_sort = match self.proc_sort {
            ProcessSortBy::Cpu => ProcessSortBy::Memory,
            ProcessSortBy::Memory => ProcessSortBy::Pid,
            ProcessSortBy::Pid => ProcessSortBy::Name,
            ProcessSortBy::Name => ProcessSortBy::Cpu,
        };

        // Re-sort processes in telemetry state
        match self.proc_sort {
            ProcessSortBy::Cpu => {
                self.telemetry.processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            }
            ProcessSortBy::Memory => {
                self.telemetry.processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
            }
            ProcessSortBy::Pid => {
                self.telemetry.processes.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
            ProcessSortBy::Name => {
                self.telemetry.processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        draw_ui(
            frame,
            &self.telemetry,
            &self.history,
            self.tab,
            &self.theme,
            &self.animation,
            &self.pet,
            self.selected_proc_idx,
            self.proc_sort,
            self.tree_mode,
            &self.search_query,
            self.is_searching,
            self.show_help,
            self.show_theme_picker,
            self.show_pet_panel,
            self.signal_modal.as_ref(),
            self.status_message.as_ref().map(|(msg, _)| msg.as_str()),
        );
    }
}
