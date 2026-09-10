#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetMood {
    Sleeping,
    Happy,
    Busy,
    HighCpu,
    Hot,
    Concerned,
    Excited,
    Network,
}

impl PetMood {
    pub fn label(self) -> &'static str {
        match self {
            PetMood::Sleeping => "Sleeping",
            PetMood::Happy => "Happy",
            PetMood::Busy => "Busy",
            PetMood::HighCpu => "Overclocked!",
            PetMood::Hot => "Toasty!",
            PetMood::Concerned => "Memory Pressure",
            PetMood::Excited => "GPU Roaring",
            PetMood::Network => "Surfing Web",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_mood_labels() {
        assert_eq!(PetMood::Sleeping.label(), "Sleeping");
        assert_eq!(PetMood::HighCpu.label(), "Overclocked!");
        assert_eq!(PetMood::Hot.label(), "Toasty!");
    }
}
