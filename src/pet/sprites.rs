use super::mood::PetMood;

/// Returns lines of ASCII art for the specified mood and blink state.
pub fn get_cat_sprite(mood: PetMood, is_blinking: bool) -> [&'static str; 3] {
    if is_blinking && mood != PetMood::Sleeping {
        return [
            r" /\_/\ ",
            r"( -.- )",
            r" > ^ < ",
        ];
    }

    match mood {
        PetMood::Sleeping => [
            r" /\_/\ ",
            r"( -.- ) zZ",
            r" /   \ ",
        ],
        PetMood::Happy => [
            r" /\_/\ ",
            r"( •.• )",
            r" > ^ < ",
        ],
        PetMood::Busy => [
            r" /\_/\ ",
            r"( o.o )",
            r" /| |\ ",
        ],
        PetMood::HighCpu => [
            r" /\_/\ ",
            r"( O.O ) !",
            r" /|!|\ ",
        ],
        PetMood::Hot => [
            r" /\_/\ ",
            r"( x.x ) 🔥",
            r" /| |\ ",
        ],
        PetMood::Concerned => [
            r" /\_/\ ",
            r"( ;.; ) 💦",
            r" > ^ < ",
        ],
        PetMood::Excited => [
            r" /\_/\ ",
            r"( ^.^ ) ✨",
            r" /| |\ ",
        ],
        PetMood::Network => [
            r" /\_/\ ",
            r"( •ᴗ• ) ⚡",
            r" /| |\ ",
        ],
    }
}
