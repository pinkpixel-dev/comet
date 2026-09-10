pub mod builtin;

use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    Candy,
    Synthwave,
    Aurora,
    Cyberpunk,
    Ocean,
    AmberCrt,
    GreenCrt,
    Monochrome,
    Rainbow,
}

impl ThemeId {
    pub const ALL: [ThemeId; 9] = [
        ThemeId::Candy,
        ThemeId::Synthwave,
        ThemeId::Aurora,
        ThemeId::Cyberpunk,
        ThemeId::Ocean,
        ThemeId::AmberCrt,
        ThemeId::GreenCrt,
        ThemeId::Monochrome,
        ThemeId::Rainbow,
    ];

    pub fn next(self) -> Self {
        match self {
            ThemeId::Candy => ThemeId::Synthwave,
            ThemeId::Synthwave => ThemeId::Aurora,
            ThemeId::Aurora => ThemeId::Cyberpunk,
            ThemeId::Cyberpunk => ThemeId::Ocean,
            ThemeId::Ocean => ThemeId::AmberCrt,
            ThemeId::AmberCrt => ThemeId::GreenCrt,
            ThemeId::GreenCrt => ThemeId::Monochrome,
            ThemeId::Monochrome => ThemeId::Rainbow,
            ThemeId::Rainbow => ThemeId::Candy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: ThemeId,
    pub name: &'static str,
    pub background: Color,
    pub panel_bg: Color,
    pub border: Color,
    pub border_focused: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub text: Color,
    pub text_muted: Color,
    pub chart_colors: Vec<Color>,
}

impl Default for Theme {
    fn default() -> Self {
        builtin::candy()
    }
}

impl Theme {
    pub fn get(id: ThemeId) -> Self {
        builtin::get_theme(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_themes_instantiate() {
        for &id in &ThemeId::ALL {
            let theme = Theme::get(id);
            assert_eq!(theme.id, id);
            assert!(!theme.name.is_empty());
        }
    }

    #[test]
    fn test_theme_cycle() {
        assert_eq!(ThemeId::Candy.next(), ThemeId::Synthwave);
        assert_eq!(ThemeId::Rainbow.next(), ThemeId::Candy);
    }
}
