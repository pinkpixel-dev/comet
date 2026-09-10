pub mod easing;

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub frame: u64,
    pub start_time: Instant,
    pub last_tick: Instant,
    pub elapsed: Duration,
    pub delta: Duration,
    pub pulse: f32,
    pub fast_pulse: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            frame: 0,
            start_time: now,
            last_tick: now,
            elapsed: Duration::ZERO,
            delta: Duration::ZERO,
            pulse: 0.0,
            fast_pulse: 0.0,
        }
    }
}

impl AnimationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.frame = self.frame.wrapping_add(1);
        self.delta = now.duration_since(self.last_tick);
        self.last_tick = now;
        self.elapsed = now.duration_since(self.start_time);

        let t = self.elapsed.as_secs_f32();
        // Regular 1Hz pulse (0.0 to 1.0)
        self.pulse = ((t * 2.0 * std::f32::consts::PI * 0.5).sin() + 1.0) / 2.0;
        // Faster 2Hz pulse for busy/active indicators
        self.fast_pulse = ((t * 2.0 * std::f32::consts::PI * 1.5).sin() + 1.0) / 2.0;
    }
}
