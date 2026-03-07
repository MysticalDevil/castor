use crate::core::session::SessionHealth;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub mtime: DateTime<Utc>,
    pub health: SessionHealth,
    pub name: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MetadataCache {
    pub entries: HashMap<PathBuf, CacheEntry>,
}

impl MetadataCache {
    pub fn load(path: &Path) -> Self {
        if let Ok(content) = std::fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn get(&self, path: &Path, mtime: DateTime<Utc>) -> Option<CacheEntry> {
        self.entries.get(path).filter(|e| e.mtime == mtime).cloned()
    }

    pub fn update(&mut self, path: PathBuf, entry: CacheEntry) {
        self.entries.insert(path, entry);
    }
}
