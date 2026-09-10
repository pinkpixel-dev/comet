use crate::telemetry::RecordingFormat;
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
    pub recording_dir: Option<String>,
    pub recording_format: String,
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
            recording_dir: None,
            recording_format: "csv".to_string(),
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

    /// Resolves the configured recording format: "csv" or "json".
    pub fn parsed_recording_format(&self) -> RecordingFormat {
        match self.recording_format.trim().to_lowercase().as_str() {
            "json" | "jsonl" | "ndjson" => RecordingFormat::Json,
            _ => RecordingFormat::Csv,
        }
    }

    /// Resolves the recording output directory. If not configured or empty,
    /// defaults to `$XDG_DATA_HOME/comet/recordings` or `~/.local/share/comet/recordings` (fallback to `./recordings`).
    pub fn resolve_recording_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.recording_dir {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                if trimmed.starts_with("~/") {
                    if let Ok(home) = env::var("HOME") {
                        return PathBuf::from(home).join(&trimmed[2..]);
                    }
                }
                return PathBuf::from(trimmed);
            }
        }

        if let Ok(xdg) = env::var("XDG_DATA_HOME") {
            if !xdg.trim().is_empty() {
                return PathBuf::from(xdg).join("comet").join("recordings");
            }
        }

        if let Ok(home) = env::var("HOME") {
            if !home.trim().is_empty() {
                return PathBuf::from(home).join(".local").join("share").join("comet").join("recordings");
            }
        }

        PathBuf::from("./recordings")
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
        assert_eq!(cfg.parsed_recording_format(), RecordingFormat::Csv);
        assert!(cfg.resolve_recording_dir().to_string_lossy().contains("recordings"));
    }

    #[test]
    fn test_parse_custom_toml() {
        let toml_str = r#"
            theme = "Cyberpunk"
            default_tab = "gpu"
            pet_name = "Kiko"
            show_pet = false
            telemetry_refresh_ms = 250
            recording_format = "json"
            recording_dir = "/tmp/comet_bench"
        "#;

        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.parsed_theme(), ThemeId::Cyberpunk);
        assert_eq!(cfg.parsed_tab(), Tab::Gpu);
        assert_eq!(cfg.pet_name, "Kiko");
        assert!(!cfg.show_pet);
        assert_eq!(cfg.telemetry_refresh_ms, 250);
        assert_eq!(cfg.process_refresh_ms, 1000); // defaulted
        assert_eq!(cfg.parsed_recording_format(), RecordingFormat::Json);
        assert_eq!(cfg.resolve_recording_dir(), PathBuf::from("/tmp/comet_bench"));
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
