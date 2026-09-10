use crate::theme::ThemeId;
use crate::ui::Tab;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub default_tab: String,
    pub show_pet: bool,
    pub pet_name: String,
    pub pet_position: String,
    pub animations: bool,
    pub telemetry_refresh_ms: u64,
    pub process_refresh_ms: u64,
    pub history_capacity: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "candy".to_string(),
            default_tab: "overview".to_string(),
            show_pet: true,
            pet_name: "Mochi".to_string(),
            pet_position: "bottom-right".to_string(),
            animations: true,
            telemetry_refresh_ms: 500,
            process_refresh_ms: 1000,
            history_capacity: 120,
        }
    }
}

impl Config {
    /// Resolves the path to the configuration file following XDG Base Directory specification.
    pub fn config_path() -> Option<PathBuf> {
        if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                return Some(PathBuf::from(xdg).join("comet").join("config.toml"));
            }
        }

        if let Ok(home) = env::var("HOME") {
            if !home.trim().is_empty() {
                return Some(PathBuf::from(home).join(".config").join("comet").join("config.toml"));
            }
        }

        None
    }

    /// Loads the configuration from disk, or returns default if not found or invalid.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<Config>(&content) {
                Ok(config) => config,
                Err(err) => {
                    eprintln!("Warning: Failed to parse {}: {}. Using default configuration.", path.display(), err);
                    Self::default()
                }
            },
            Err(err) => {
                eprintln!("Warning: Failed to read {}: {}. Using default configuration.", path.display(), err);
                Self::default()
            }
        }
    }

    /// Resolves the theme setting into a ThemeId.
    pub fn parsed_theme(&self) -> ThemeId {
        let norm = self.theme.trim().to_lowercase();
        match norm.as_str() {
            "synthwave" => ThemeId::Synthwave,
            "aurora" => ThemeId::Aurora,
            "cyberpunk" => ThemeId::Cyberpunk,
            "ocean" => ThemeId::Ocean,
            "amber" | "amber-crt" | "ambercrt" | "amber_crt" => ThemeId::AmberCrt,
            "green" | "green-crt" | "greencrt" | "green_crt" => ThemeId::GreenCrt,
            "monochrome" | "mono" => ThemeId::Monochrome,
            "rainbow" => ThemeId::Rainbow,
            _ => ThemeId::Candy,
        }
    }

    /// Resolves the default_tab setting into a Tab.
    pub fn parsed_tab(&self) -> Tab {
        let norm = self.default_tab.trim().to_lowercase();
        match norm.as_str() {
            "cpu" => Tab::Cpu,
            "gpu" => Tab::Gpu,
            "memory" | "mem" | "ram" => Tab::Memory,
            "disks" | "disk" | "storage" => Tab::Disks,
            "network" | "net" => Tab::Network,
            "processes" | "process" | "proc" | "procs" => Tab::Processes,
            "sensors" | "sensor" | "temp" => Tab::Sensors,
            _ => Tab::Overview,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.parsed_theme(), ThemeId::Candy);
        assert_eq!(cfg.parsed_tab(), Tab::Overview);
        assert_eq!(cfg.pet_name, "Mochi");
        assert!(cfg.show_pet);
        assert_eq!(cfg.telemetry_refresh_ms, 500);
    }

    #[test]
    fn test_parse_custom_toml() {
        let toml_str = r#"
            theme = "Cyberpunk"
            default_tab = "gpu"
            pet_name = "Kiko"
            show_pet = false
            telemetry_refresh_ms = 250
        "#;

        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.parsed_theme(), ThemeId::Cyberpunk);
        assert_eq!(cfg.parsed_tab(), Tab::Gpu);
        assert_eq!(cfg.pet_name, "Kiko");
        assert!(!cfg.show_pet);
        assert_eq!(cfg.telemetry_refresh_ms, 250);
        assert_eq!(cfg.process_refresh_ms, 1000); // defaulted
    }

    #[test]
    fn test_theme_aliases() {
        let mut cfg = Config::default();

        cfg.theme = "amber-crt".to_string();
        assert_eq!(cfg.parsed_theme(), ThemeId::AmberCrt);

        cfg.theme = "green_crt".to_string();
        assert_eq!(cfg.parsed_theme(), ThemeId::GreenCrt);

        cfg.theme = "MONO".to_string();
        assert_eq!(cfg.parsed_theme(), ThemeId::Monochrome);

        cfg.theme = "invalid-name".to_string();
        assert_eq!(cfg.parsed_theme(), ThemeId::Candy);
    }
}
