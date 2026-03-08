use crate::error::{CastorError, Result};
use crate::tui::theme::ThemeConfig;
use crate::utils::icons::IconSet;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreviewConfig {
    pub head_bytes: u64,
    pub tail_bytes: u64,
    pub small_full_parse_bytes: u64,
    pub deep_preview_max_bytes: u64,
    pub deep_preview_char_budget: usize,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            head_bytes: 512 * 1024,
            tail_bytes: 2 * 1024 * 1024,
            small_full_parse_bytes: 2 * 1024 * 1024,
            deep_preview_max_bytes: 64 * 1024 * 1024,
            deep_preview_char_budget: 120_000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub gemini_sessions_path: PathBuf,
    pub trash_path: PathBuf,
    pub audit_path: PathBuf,
    pub cache_path: PathBuf,
    pub dry_run_by_default: bool,
    pub icon_set: IconSet,
    pub theme: ThemeConfig,
    #[serde(default)]
    pub preview: PreviewConfig,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
        let default_gemini = home.join(".gemini").join("tmp");

        let (trash_path, audit_path, cache_path) =
            if let Some(proj_dirs) = ProjectDirs::from("com", "omega", "castor") {
                // XDG Standard paths
                (
                    proj_dirs.data_dir().join("trash"),
                    proj_dirs.data_dir().join("audit"),
                    proj_dirs.cache_dir().join("metadata.json"),
                )
            } else {
                let fallback_base = home.join(".local").join("share").join("castor");
                (
                    fallback_base.join("trash"),
                    fallback_base.join("audit"),
                    home.join(".cache").join("castor").join("metadata.json"),
                )
            };

        Self {
            gemini_sessions_path: default_gemini,
            trash_path,
            audit_path,
            cache_path,
            dry_run_by_default: true,
            icon_set: IconSet::default(),
            theme: ThemeConfig::Preset("TokyoNight".to_string()),
            preview: PreviewConfig::default(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        if let Some(p) = path {
            if p.exists() {
                let content = std::fs::read_to_string(p)?;
                return serde_json::from_str(&content).map_err(CastorError::Serialization);
            } else {
                return Err(CastorError::Config(format!(
                    "Config file not found: {:?}",
                    p
                )));
            }
        }

        let config_path = if let Some(proj_dirs) = ProjectDirs::from("com", "omega", "castor") {
            proj_dirs.config_dir().join("config.json")
        } else {
            let home = std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/tmp"));
            home.join(".config").join("castor").join("config.json")
        };

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            serde_json::from_str(&content).map_err(CastorError::Serialization)
        } else {
            Ok(Self::default())
        }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.gemini_sessions_path)?;
        std::fs::create_dir_all(&self.trash_path)?;
        std::fs::create_dir_all(&self.audit_path)?;
        if let Some(parent) = self.cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.dry_run_by_default);
        assert_eq!(config.icon_set, IconSet::NerdFont);
        // Default theme should now be TokyoNight
        assert_eq!(config.theme, ThemeConfig::Preset("TokyoNight".to_string()));
        // Ensure cache path uses cache_dir (XDG)
        assert!(config.cache_path.to_string_lossy().contains("cache"));
        assert_eq!(config.preview.head_bytes, 512 * 1024);
    }

    #[test]
    fn test_config_load_file() {
        let tmp = tempdir().expect("create tempdir");
        let config_path = tmp.path().join("config.json");
        let data = r#"{
            "gemini_sessions_path": "/tmp/gemini",
            "trash_path": "/tmp/trash",
            "audit_path": "/tmp/audit",
            "cache_path": "/tmp/cache",
            "dry_run_by_default": false,
            "icon_set": "Unicode",
            "theme": "Gruvbox"
        }"#;
        fs::write(&config_path, data).expect("write config file");

        let config = Config::load(Some(&config_path)).expect("load config from file");
        assert!(!config.dry_run_by_default);
        assert_eq!(config.gemini_sessions_path, PathBuf::from("/tmp/gemini"));
        assert_eq!(config.icon_set, IconSet::Unicode);
        assert_eq!(config.theme, ThemeConfig::Preset("Gruvbox".to_string()));
        assert_eq!(config.preview.deep_preview_max_bytes, 64 * 1024 * 1024);
    }

    #[test]
    fn test_ensure_dirs() {
        let tmp = tempdir().expect("create tempdir");
        let config = Config {
            gemini_sessions_path: tmp.path().join("sessions"),
            trash_path: tmp.path().join("trash"),
            audit_path: tmp.path().join("audit"),
            cache_path: tmp.path().join("cache").join("metadata.json"),
            dry_run_by_default: true,
            icon_set: IconSet::Ascii,
            theme: ThemeConfig::default(),
            preview: PreviewConfig::default(),
        };

        config.ensure_dirs().expect("ensure config dirs");
        assert!(tmp.path().join("sessions").exists());
        assert!(tmp.path().join("trash").exists());
        assert!(tmp.path().join("audit").exists());
        assert!(tmp.path().join("cache").exists());
    }
}
