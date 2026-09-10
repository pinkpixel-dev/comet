pub mod animation;
pub mod app;
pub mod config;
pub mod event;
pub mod history;
pub mod pet;
pub mod telemetry;
pub mod theme;
pub mod ui;

use anyhow::Result;
use app::App;
use config::Config;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use event::{AppEvent, EventHandler};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, stdout};
use std::panic;
use telemetry::TelemetryManager;

fn main() -> Result<()> {
    let config = Config::load();

    // Setup panic hook to cleanly restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize telemetry manager & event loop at 30 FPS
    let (telemetry_mgr, telemetry_rx) = TelemetryManager::start(
        config.telemetry_refresh_ms,
        config.process_refresh_ms,
    );
    let events = EventHandler::new(telemetry_rx, 30);
    let mut app = App::new(&config);

    // Initial draw
    terminal.draw(|f| app.draw(f))?;

    // Main event loop
    while !app.should_quit {
        match events.receiver.recv() {
            Ok(AppEvent::Tick) => {
                app.on_tick();
                terminal.draw(|f| app.draw(f))?;
            }
            Ok(AppEvent::Telemetry(state)) => {
                app.on_telemetry(state);
                terminal.draw(|f| app.draw(f))?;
            }
            Ok(AppEvent::Key(key)) => {
                app.on_key(key);
                terminal.draw(|f| app.draw(f))?;
            }
            Ok(AppEvent::Resize(_, _)) => {
                terminal.draw(|f| app.draw(f))?;
            }
            Ok(AppEvent::Mouse(_)) => {}
            Err(_) => break,
        }
    }

    // Stop background workers
    telemetry_mgr.stop();
    events.stop();

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
