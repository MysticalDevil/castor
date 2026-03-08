use crate::config::Config;
use crate::core::session::{Session, SessionHealth};
use std::sync::Arc;

#[derive(Debug)]
pub struct DoctorReport {
    pub total_sessions: usize,
    pub corrupted_count: usize,
    pub orphaned_count: usize,
    pub high_risk_count: usize,
    pub suggestions: Vec<String>,
}

impl DoctorReport {
    pub fn generate(sessions: &[Arc<Session>], _config: &Config) -> Self {
        let total_sessions = sessions.len();
        let mut corrupted_count = 0;
        let mut orphaned_count = 0;
        let mut high_risk_count = 0;
        let mut suggestions = Vec::new();

        for s in sessions {
            match s.health {
                SessionHealth::Warn => orphaned_count += 1,
                SessionHealth::Error => corrupted_count += 1,
                SessionHealth::Risk => high_risk_count += 1,
                _ => {}
            }
        }

        if orphaned_count > 0 {
            suggestions.push(format!(
                "Found {} orphaned sessions. You might want to run `prune`.",
                orphaned_count
            ));
        }
        if corrupted_count > 0 {
            suggestions.push(format!(
                "Found {} corrupted JSON files. These should be manually inspected or deleted.",
                corrupted_count
            ));
        }
        if high_risk_count > 0 {
            suggestions.push(format!(
                "Found {} high-risk anomalies (temporal mismatches or extreme sizes).",
                high_risk_count
            ));
        }

        if suggestions.is_empty() && total_sessions > 0 {
            suggestions.push("Your Gemini environment is healthy!".to_string());
        }

        Self {
            total_sessions,
            corrupted_count,
            orphaned_count,
            high_risk_count,
            suggestions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_doctor_report_generation() {
        let mut s1 = Session {
            id: "s1".into(),
            display_id: "s1".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: PathBuf::from("/tmp/s1.json"),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 0,
            health: SessionHealth::Error,
            validation_notes: Vec::new(),
        };
        s1.health = SessionHealth::Error;

        let config = Config {
            gemini_sessions_path: PathBuf::from("/tmp"),
            trash_path: PathBuf::from("/tmp"),
            audit_path: PathBuf::from("/tmp"),
            cache_path: PathBuf::from("/tmp"),
            dry_run_by_default: true,
            icon_set: crate::utils::icons::IconSet::Ascii,
            theme: crate::tui::theme::ThemeConfig::default(),
        };

        let report = DoctorReport::generate(&[Arc::new(s1)], &config);
        assert_eq!(report.total_sessions, 1);
        assert_eq!(report.corrupted_count, 1);
    }
}
