pub mod mood;
pub mod sprites;

pub use mood::PetMood;
pub use sprites::get_cat_sprite;

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PetState {
    pub name: String,
    pub mood: PetMood,
    pub favorite: &'static str,
    pub idle_duration: Duration,
    pub last_mood_change: Instant,
    pub last_blink_toggle: Instant,
    pub is_blinking: bool,
    pub visible: bool,
}

impl Default for PetState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            name: "Mochi".to_string(),
            mood: PetMood::Happy,
            favorite: "GPU",
            idle_duration: Duration::ZERO,
            last_mood_change: now,
            last_blink_toggle: now,
            is_blinking: false,
            visible: true,
        }
    }
}

impl PetState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates pet mood based on current system conditions with hysteresis.
    pub fn update(
        &mut self,
        cpu_usage: f32,
        ram_usage: f32,
        gpu_usage: Option<f32>,
        max_temp: Option<f32>,
        net_throughput: f64,
        delta: Duration,
    ) {
        let now = Instant::now();

        // Blink logic: every 3-4 seconds, blink for 200ms
        let blink_elapsed = now.duration_since(self.last_blink_toggle);
        if self.is_blinking && blink_elapsed > Duration::from_millis(250) {
            self.is_blinking = false;
            self.last_blink_toggle = now;
        } else if !self.is_blinking && blink_elapsed > Duration::from_millis(3500) {
            self.is_blinking = true;
            self.last_blink_toggle = now;
        }

        // Idle duration tracking
        if cpu_usage < 12.0 {
            self.idle_duration += delta;
        } else {
            self.idle_duration = Duration::ZERO;
        }

        // Determine target mood
        let target_mood = if max_temp.unwrap_or(0.0) >= 80.0 {
            PetMood::Hot
        } else if cpu_usage >= 80.0 {
            PetMood::HighCpu
        } else if gpu_usage.unwrap_or(0.0) >= 75.0 {
            PetMood::Excited
        } else if ram_usage >= 85.0 {
            PetMood::Concerned
        } else if net_throughput >= 10_000_000.0 { // > 10 MB/s
            PetMood::Network
        } else if cpu_usage >= 35.0 {
            PetMood::Busy
        } else if self.idle_duration >= Duration::from_secs(30) {
            PetMood::Sleeping
        } else {
            PetMood::Happy
        };

        // Hysteresis cooldown: only change mood if at least 1.5s passed since last change
        // unless it's a critical spike (Hot or HighCpu)
        let can_change = target_mood == PetMood::Hot
            || target_mood == PetMood::HighCpu
            || now.duration_since(self.last_mood_change) >= Duration::from_millis(1500);

        if self.mood != target_mood && can_change {
            self.mood = target_mood;
            self.last_mood_change = now;
        }
    }
}
