use crate::config::Config;
use crate::core::Session;
use std::sync::Arc;

#[derive(Debug)]
pub struct StorageStats {
    pub total_sessions: usize,
    pub total_size_bytes: u64,
    pub trash_size_bytes: u64,
}

impl StorageStats {
    pub fn calculate(sessions: &[Arc<Session>], config: &Config) -> Self {
        let total_size_bytes = sessions.iter().map(|s| s.size).sum();
        let trash_size_bytes = crate::utils::fs::get_dir_size(&config.trash_path).unwrap_or(0);

        Self {
            total_sessions: sessions.len(),
            total_size_bytes,
            trash_size_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_stats_calculation() {
        let s = Arc::new(Session {
            id: "s1".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: PathBuf::from("/tmp/s1.json"),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 1024,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        });

        let config = Config {
            gemini_sessions_path: PathBuf::from("/tmp/gemini"),
            trash_path: PathBuf::from("/tmp/trash_non_existent"),
            audit_path: PathBuf::from("/tmp/audit"),
            cache_path: PathBuf::from("/tmp/cache"),
            dry_run_by_default: true,
            icon_set: crate::utils::icons::IconSet::Ascii,
            theme: crate::tui::theme::ThemeConfig::default(),
        };

        let stats = StorageStats::calculate(&[s], &config);
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_size_bytes, 1024);
    }
}
