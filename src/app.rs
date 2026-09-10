use crate::animation::AnimationState;
use crate::history::MetricHistory;
use crate::pet::PetState;
use crate::telemetry::{ProcessSortBy, TelemetryState};
use crate::config::Config;
use crate::theme::Theme;
use crate::ui::{draw_ui, Tab};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;

pub struct App {
    pub tab: Tab,
    pub telemetry: TelemetryState,
    pub history: MetricHistory,
    pub animation: AnimationState,
    pub theme: Theme,
    pub pet: PetState,
    pub selected_proc_idx: usize,
    pub proc_sort: ProcessSortBy,
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
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            }
            return;
        }

        // Handle modal dismissal
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
                self.theme = Theme::get(self.theme.id.next());
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
                    let total = self.telemetry.processes.len();
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
                }
            }
            _ => {}
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
            &self.search_query,
            self.is_searching,
            self.show_help,
            self.show_theme_picker,
            self.show_pet_panel,
        );
    }
}
