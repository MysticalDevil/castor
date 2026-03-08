use crate::config::Config;
use crate::core::Session;
use crate::core::session::SessionHealth;
use std::path::PathBuf;

pub struct DoctorReport {
    pub gemini_base_exists: bool,
    pub sessions_path_exists: bool,
    pub trash_path_exists: bool,
    pub total_sessions: usize,
    pub orphaned_count: usize,
    pub corrupted_count: usize,
    pub untrusted_count: usize,
    pub untracked_hosts_count: usize,
}

impl DoctorReport {
    pub fn generate(sessions: &[Session], config: &Config) -> Self {
        let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
        let gemini_base = home.join(".gemini");

        let mut orphaned_count = 0;
        let mut corrupted_count = 0;
        let mut untrusted_count = 0;
        let mut untracked_hosts_count = 0;

        for s in sessions {
            match s.calculate_health() {
                SessionHealth::Warn => orphaned_count += 1,
                SessionHealth::Error => corrupted_count += 1,
                SessionHealth::Risk => untrusted_count += 1,
                _ => {}
            }
            if s.host_path.is_none() {
                untracked_hosts_count += 1;
            }
        }

        Self {
            gemini_base_exists: gemini_base.exists(),
            sessions_path_exists: config.gemini_sessions_path.exists(),
            trash_path_exists: config.trash_path.exists(),
            total_sessions: sessions.len(),
            orphaned_count,
            corrupted_count,
            untrusted_count,
            untracked_hosts_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_doctor_report_generation() {
        let s1 = Session {
            id: "s1".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: PathBuf::from("fake"),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            size: 0,
            health: SessionHealth::Error,
            validation_notes: vec!["Structural: corrupted".into()],
        };

        let config = Config {
            gemini_sessions_path: PathBuf::from("/tmp"),
            trash_path: PathBuf::from("/tmp"),
            audit_path: PathBuf::from("/tmp"),
            cache_path: PathBuf::from("/tmp"),
            dry_run_by_default: true,
            icon_set: crate::utils::icons::IconSet::Ascii,
        };

        let report = DoctorReport::generate(&[s1], &config);
        assert_eq!(report.total_sessions, 1);
        assert_eq!(report.corrupted_count, 1);
    }
}
