use crate::core::Session;
use chrono::{Duration, Utc};
use std::sync::Arc;

pub fn find_sessions_to_prune(sessions: &[Arc<Session>], days: u64) -> Vec<Arc<Session>> {
    let threshold = Utc::now() - Duration::days(days as i64);
    sessions
        .iter()
        .filter(|s| s.updated_at < threshold)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_prune_selection() {
        let now = Utc::now();
        let s1 = Arc::new(Session {
            id: "old".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: PathBuf::from("old.json"),
            created_at: now - Duration::days(40),
            updated_at: now - Duration::days(40),
            size: 0,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        });
        let s2 = Arc::new(Session {
            id: "new".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: PathBuf::from("new.json"),
            created_at: now,
            updated_at: now,
            size: 0,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        });

        let to_prune = find_sessions_to_prune(&[s1, s2], 30);
        assert_eq!(to_prune.len(), 1);
        assert_eq!(to_prune[0].id, "old");
    }
}
