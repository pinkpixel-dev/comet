use crate::telemetry::TelemetryState;
use crossbeam_channel::{bounded, Receiver};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Telemetry(TelemetryState),
}

pub struct EventHandler {
    pub receiver: Receiver<AppEvent>,
    running: Arc<AtomicBool>,
}

impl EventHandler {
    pub fn new(telemetry_rx: Receiver<TelemetryState>, tick_rate_fps: u64) -> Self {
        let (sender, receiver) = bounded(100);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let tick_duration = Duration::from_millis(1000 / tick_rate_fps.max(1));

        thread::Builder::new()
            .name("comet-events".to_string())
            .spawn(move || {
                let mut last_tick = Instant::now();
                while running_clone.load(Ordering::Relaxed) {
                    let timeout = tick_duration
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(Duration::from_millis(5));

                    if event::poll(timeout).unwrap_or(false) {
                        match event::read() {
                            Ok(CrosstermEvent::Key(key)) => {
                                let _ = sender.send(AppEvent::Key(key));
                            }
                            Ok(CrosstermEvent::Mouse(mouse)) => {
                                let _ = sender.send(AppEvent::Mouse(mouse));
                            }
                            Ok(CrosstermEvent::Resize(w, h)) => {
                                let _ = sender.send(AppEvent::Resize(w, h));
                            }
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_duration {
                        let _ = sender.send(AppEvent::Tick);
                        last_tick = Instant::now();
                    }

                    // Drain any pending telemetry updates
                    while let Ok(telem) = telemetry_rx.try_recv() {
                        let _ = sender.send(AppEvent::Telemetry(telem));
                    }
                }
            })
            .expect("Failed to spawn event handler thread");

        Self { receiver, running }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
