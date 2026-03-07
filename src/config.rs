use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use crate::error::{Result, CastorError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub gemini_sessions_path: PathBuf,
    pub trash_path: PathBuf,
    pub audit_path: PathBuf,
    pub dry_run_by_default: bool,
}

impl Default for Config {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("com", "omega", "castor")
            .expect("Could not determine home directory for configuration.");

        let data_dir = proj_dirs.data_dir();
        let _config_dir = proj_dirs.config_dir();

        let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"));
        let default_gemini = home.join(".gemini").join("tmp");

        Self {
            gemini_sessions_path: default_gemini,
            trash_path: data_dir.join("trash"),
            audit_path: data_dir.join("audit"),
            dry_run_by_default: true,
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
                return Err(CastorError::Config(format!("Config file not found: {:?}", p)));
            }
        }

        let proj_dirs = ProjectDirs::from("com", "omega", "castor")
            .expect("Could not determine config directory.");
        let config_path = proj_dirs.config_dir().join("config.json");

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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.dry_run_by_default);
    }

    #[test]
    fn test_config_load_file() {
        let tmp = tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        let data = r#"{
            "gemini_sessions_path": "/tmp/gemini",
            "trash_path": "/tmp/trash",
            "audit_path": "/tmp/audit",
            "dry_run_by_default": false
        }"#;
        fs::write(&config_path, data).unwrap();

        let config = Config::load(Some(&config_path)).unwrap();
        assert!(!config.dry_run_by_default);
        assert_eq!(config.gemini_sessions_path, PathBuf::from("/tmp/gemini"));
    }

    #[test]
    fn test_ensure_dirs() {
        let tmp = tempdir().unwrap();
        let config = Config {
            gemini_sessions_path: tmp.path().join("sessions"),
            trash_path: tmp.path().join("trash"),
            audit_path: tmp.path().join("audit"),
            dry_run_by_default: true,
        };

        config.ensure_dirs().unwrap();
        assert!(tmp.path().join("sessions").exists());
        assert!(tmp.path().join("trash").exists());
        assert!(tmp.path().join("audit").exists());
    }
}
