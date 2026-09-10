use super::{Theme, ThemeId};
use ratatui::style::Color;

pub fn get_theme(id: ThemeId) -> Theme {
    match id {
        ThemeId::Candy => candy(),
        ThemeId::Synthwave => synthwave(),
        ThemeId::Aurora => aurora(),
        ThemeId::Cyberpunk => cyberpunk(),
        ThemeId::Ocean => ocean(),
        ThemeId::AmberCrt => amber_crt(),
        ThemeId::GreenCrt => green_crt(),
        ThemeId::Monochrome => monochrome(),
        ThemeId::Rainbow => rainbow(),
    }
}

pub fn candy() -> Theme {
    Theme {
        id: ThemeId::Candy,
        name: "Candy",
        background: Color::Rgb(20, 20, 26),
        panel_bg: Color::Rgb(28, 28, 38),
        border: Color::Rgb(60, 60, 80),
        border_focused: Color::Rgb(244, 114, 182), // Soft bright pink
        primary: Color::Rgb(244, 114, 182),        // Pink
        secondary: Color::Rgb(168, 85, 247),       // Purple
        accent: Color::Rgb(56, 189, 248),          // Cyan
        success: Color::Rgb(52, 211, 153),         // Mint
        warning: Color::Rgb(251, 191, 36),         // Warm Yellow
        danger: Color::Rgb(248, 113, 113),         // Coral Red
        text: Color::Rgb(248, 250, 252),           // Crisp white
        text_muted: Color::Rgb(148, 163, 184),     // Slate muted
        chart_colors: vec![
            Color::Rgb(244, 114, 182),
            Color::Rgb(56, 189, 248),
            Color::Rgb(168, 85, 247),
            Color::Rgb(251, 191, 36),
        ],
    }
}

pub fn synthwave() -> Theme {
    Theme {
        id: ThemeId::Synthwave,
        name: "Synthwave",
        background: Color::Rgb(18, 16, 28),
        panel_bg: Color::Rgb(26, 22, 40),
        border: Color::Rgb(70, 50, 95),
        border_focused: Color::Rgb(236, 72, 153),
        primary: Color::Rgb(236, 72, 153),   // Magenta
        secondary: Color::Rgb(147, 51, 234), // Neon violet
        accent: Color::Rgb(34, 211, 238),    // Electric cyan
        success: Color::Rgb(45, 212, 191),   // Teal
        warning: Color::Rgb(250, 204, 21),   // Bright yellow
        danger: Color::Rgb(239, 68, 68),     // Red
        text: Color::Rgb(253, 244, 255),
        text_muted: Color::Rgb(168, 162, 184),
        chart_colors: vec![
            Color::Rgb(236, 72, 153),
            Color::Rgb(34, 211, 238),
            Color::Rgb(147, 51, 234),
            Color::Rgb(250, 204, 21),
        ],
    }
}

pub fn aurora() -> Theme {
    Theme {
        id: ThemeId::Aurora,
        name: "Aurora",
        background: Color::Rgb(16, 24, 28),
        panel_bg: Color::Rgb(22, 34, 40),
        border: Color::Rgb(45, 75, 80),
        border_focused: Color::Rgb(52, 211, 153),
        primary: Color::Rgb(52, 211, 153),   // Emerald green
        secondary: Color::Rgb(45, 212, 191), // Teal
        accent: Color::Rgb(56, 189, 248),    // Sky
        success: Color::Rgb(134, 239, 172),  // Pale green
        warning: Color::Rgb(253, 224, 71),   // Yellow
        danger: Color::Rgb(248, 113, 113),   // Red
        text: Color::Rgb(240, 253, 250),
        text_muted: Color::Rgb(148, 180, 180),
        chart_colors: vec![
            Color::Rgb(52, 211, 153),
            Color::Rgb(45, 212, 191),
            Color::Rgb(56, 189, 248),
            Color::Rgb(134, 239, 172),
        ],
    }
}

pub fn cyberpunk() -> Theme {
    Theme {
        id: ThemeId::Cyberpunk,
        name: "Cyberpunk",
        background: Color::Rgb(16, 18, 24),
        panel_bg: Color::Rgb(24, 26, 36),
        border: Color::Rgb(65, 70, 90),
        border_focused: Color::Rgb(250, 204, 21),
        primary: Color::Rgb(250, 204, 21),   // Cyber yellow
        secondary: Color::Rgb(6, 182, 212),  // Neon cyan
        accent: Color::Rgb(244, 63, 94),     // Rose magenta
        success: Color::Rgb(74, 222, 128),   // Matrix green
        warning: Color::Rgb(251, 146, 60),   // Orange
        danger: Color::Rgb(239, 68, 68),     // Crimson
        text: Color::Rgb(254, 252, 232),
        text_muted: Color::Rgb(161, 161, 170),
        chart_colors: vec![
            Color::Rgb(250, 204, 21),
            Color::Rgb(6, 182, 212),
            Color::Rgb(244, 63, 94),
            Color::Rgb(74, 222, 128),
        ],
    }
}

pub fn ocean() -> Theme {
    Theme {
        id: ThemeId::Ocean,
        name: "Ocean",
        background: Color::Rgb(15, 23, 42),
        panel_bg: Color::Rgb(23, 37, 66),
        border: Color::Rgb(47, 68, 108),
        border_focused: Color::Rgb(56, 189, 248),
        primary: Color::Rgb(56, 189, 248),   // Deep cyan
        secondary: Color::Rgb(96, 165, 250), // Sky blue
        accent: Color::Rgb(45, 212, 191),    // Aqua
        success: Color::Rgb(52, 211, 153),   // Seafoam
        warning: Color::Rgb(251, 191, 36),   // Gold
        danger: Color::Rgb(248, 113, 113),   // Coral
        text: Color::Rgb(241, 245, 249),
        text_muted: Color::Rgb(148, 163, 184),
        chart_colors: vec![
            Color::Rgb(56, 189, 248),
            Color::Rgb(96, 165, 250),
            Color::Rgb(45, 212, 191),
            Color::Rgb(129, 140, 248),
        ],
    }
}

pub fn amber_crt() -> Theme {
    Theme {
        id: ThemeId::AmberCrt,
        name: "Amber CRT",
        background: Color::Rgb(18, 14, 8),
        panel_bg: Color::Rgb(28, 22, 12),
        border: Color::Rgb(90, 65, 25),
        border_focused: Color::Rgb(245, 158, 11),
        primary: Color::Rgb(245, 158, 11),   // Vivid Amber
        secondary: Color::Rgb(217, 119, 6),  // Dark Amber
        accent: Color::Rgb(251, 191, 36),    // Bright Amber
        success: Color::Rgb(234, 179, 8),    // Yellow
        warning: Color::Rgb(249, 115, 22),   // Orange
        danger: Color::Rgb(239, 68, 68),     // Red
        text: Color::Rgb(254, 243, 199),
        text_muted: Color::Rgb(180, 140, 90),
        chart_colors: vec![
            Color::Rgb(245, 158, 11),
            Color::Rgb(251, 191, 36),
            Color::Rgb(217, 119, 6),
            Color::Rgb(249, 115, 22),
        ],
    }
}

pub fn green_crt() -> Theme {
    Theme {
        id: ThemeId::GreenCrt,
        name: "Green CRT",
        background: Color::Rgb(10, 20, 12),
        panel_bg: Color::Rgb(16, 32, 18),
        border: Color::Rgb(35, 75, 42),
        border_focused: Color::Rgb(34, 197, 94),
        primary: Color::Rgb(34, 197, 94),   // Phosphor Green
        secondary: Color::Rgb(22, 163, 74), // Medium Green
        accent: Color::Rgb(74, 222, 128),   // Bright Green
        success: Color::Rgb(134, 239, 172), // Light Green
        warning: Color::Rgb(250, 204, 21),  // Soft Yellow
        danger: Color::Rgb(248, 113, 113),  // Pale Red
        text: Color::Rgb(240, 253, 244),
        text_muted: Color::Rgb(110, 160, 120),
        chart_colors: vec![
            Color::Rgb(34, 197, 94),
            Color::Rgb(74, 222, 128),
            Color::Rgb(22, 163, 74),
            Color::Rgb(134, 239, 172),
        ],
    }
}

pub fn monochrome() -> Theme {
    Theme {
        id: ThemeId::Monochrome,
        name: "Monochrome",
        background: Color::Rgb(18, 18, 18),
        panel_bg: Color::Rgb(28, 28, 28),
        border: Color::Rgb(65, 65, 65),
        border_focused: Color::Rgb(240, 240, 240),
        primary: Color::Rgb(240, 240, 240), // Pure crisp white
        secondary: Color::Rgb(180, 180, 180),
        accent: Color::Rgb(215, 215, 215),
        success: Color::Rgb(220, 220, 220),
        warning: Color::Rgb(190, 190, 190),
        danger: Color::Rgb(250, 100, 100),
        text: Color::Rgb(255, 255, 255),
        text_muted: Color::Rgb(130, 130, 130),
        chart_colors: vec![
            Color::Rgb(245, 245, 245),
            Color::Rgb(190, 190, 190),
            Color::Rgb(140, 140, 140),
            Color::Rgb(220, 220, 220),
        ],
    }
}

pub fn rainbow() -> Theme {
    Theme {
        id: ThemeId::Rainbow,
        name: "Rainbow",
        background: Color::Rgb(18, 18, 26),
        panel_bg: Color::Rgb(26, 26, 36),
        border: Color::Rgb(60, 60, 80),
        border_focused: Color::Rgb(244, 63, 94),
        primary: Color::Rgb(244, 63, 94),    // Red/Rose
        secondary: Color::Rgb(249, 115, 22), // Orange
        accent: Color::Rgb(59, 130, 246),    // Blue
        success: Color::Rgb(34, 197, 94),    // Green
        warning: Color::Rgb(234, 179, 8),    // Yellow
        danger: Color::Rgb(168, 85, 247),    // Violet
        text: Color::Rgb(255, 255, 255),
        text_muted: Color::Rgb(156, 163, 175),
        chart_colors: vec![
            Color::Rgb(244, 63, 94),
            Color::Rgb(249, 115, 22),
            Color::Rgb(234, 179, 8),
            Color::Rgb(34, 197, 94),
            Color::Rgb(59, 130, 246),
            Color::Rgb(168, 85, 247),
        ],
    }
}
