use crate::core::Session;
use crate::error::Result;
use std::sync::Arc;

pub fn search_sessions(
    sessions: &[Arc<Session>],
    pattern: &str,
    ignore_case: bool,
) -> Result<Vec<Arc<Session>>> {
    let mut matches = Vec::new();
    let pattern_final = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    for s in sessions {
        let content = s.get_content()?;
        let search_target = if ignore_case {
            content.to_lowercase()
        } else {
            content
        };

        if search_target.contains(&pattern_final) {
            matches.push(s.clone());
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_search_sessions() {
        let tmp = tempdir().expect("create tempdir");
        let p1 = tmp.path().join("s1.json");
        fs::write(
            &p1,
            r#"{"messages": [{"type":"user","content":"Rust performance"}]}"#,
        )
        .expect("write grep test session");

        let s1 = Arc::new(Session {
            id: "s1".into(),
            display_id: "s1".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: p1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 0,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        });

        let results =
            search_sessions(&[s1.clone()], "Rust", false).expect("search session for Rust");
        assert_eq!(results.len(), 1);

        let results_none =
            search_sessions(&[s1], "Python", false).expect("search session for Python");
        assert_eq!(results_none.len(), 0);
    }
}
