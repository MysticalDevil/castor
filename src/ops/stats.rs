use crate::config::Config;
use crate::core::Session;
use std::path::Path;

pub struct StorageStats {
    pub total_sessions: usize,
    pub total_size_bytes: u64,
    pub trash_size_bytes: u64,
}

impl StorageStats {
    pub fn calculate(sessions: &[Session], config: &Config) -> Self {
        let total_sessions = sessions.len();
        let total_size_bytes = sessions.iter().map(|s| s.size).sum();
        let trash_size_bytes = Self::calculate_dir_size(&config.trash_path);

        Self {
            total_sessions,
            total_size_bytes,
            trash_size_bytes,
        }
    }

    fn calculate_dir_size(path: &Path) -> u64 {
        let mut size = 0;
        if path.exists() {
            for e in walkdir::WalkDir::new(path).into_iter().flatten() {
                if e.file_type().is_file() {
                    size += e.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_stats_calculation() {
        let s = Session {
            id: "s1".into(),
            project_id: "p".into(),
            host_path: None,
            name: None,
            path: PathBuf::from("s1"),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 1024,
            validation_notes: Vec::new(),
        };

        let config = Config {
            gemini_sessions_path: PathBuf::from("/tmp"),
            trash_path: PathBuf::from("/tmp/trash_non_existent"),
            audit_path: PathBuf::from("/tmp/audit"),
            dry_run_by_default: true,
        };

        let stats = StorageStats::calculate(&[s], &config);
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_size_bytes, 1024);
        assert_eq!(stats.trash_size_bytes, 0);
    }
}
