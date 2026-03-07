use crate::error::{CastorError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Directory where Gemini sessions are stored.
    pub gemini_sessions_path: PathBuf,

    /// Directory for "soft-deleted" sessions (trash).
    pub trash_path: PathBuf,

    /// Directory for audit logs and batch history.
    pub audit_path: PathBuf,

    /// Enable dry-run by default for all destructive operations.
    pub dry_run_by_default: bool,
}

impl Default for Config {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("com", "omega", "castor")
            .expect("Could not determine home directory for configuration.");

        let data_dir = proj_dirs.data_dir();
        let _config_dir = proj_dirs.config_dir();

        // Gemini typically uses ~/.gemini/sessions (hypothetically, for this tool's purpose)
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
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
    /// Loads configuration from a file or defaults.
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

        // Use standard config path if no path provided
        let proj_dirs = ProjectDirs::from("com", "omega", "castor")
            .expect("Could not determine config directory.");
        let config_path = proj_dirs.config_dir().join("config.json");

        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            serde_json::from_str(&content).map_err(CastorError::Serialization)
        } else {
            // Return defaults if no config file exists
            Ok(Self::default())
        }
    }

    /// Ensures all directories in the config exist.
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.gemini_sessions_path)?;
        std::fs::create_dir_all(&self.trash_path)?;
        std::fs::create_dir_all(&self.audit_path)?;
        Ok(())
    }
}
