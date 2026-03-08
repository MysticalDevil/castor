use crate::core::Session;
use crate::error::Result;
use std::fs;

pub fn search_sessions<'a>(
    sessions: &'a [Session],
    pattern: &str,
    ignore_case: bool,
) -> Result<Vec<&'a Session>> {
    let mut matches = Vec::new();
    let pattern_normalized = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    for s in sessions {
        let content = fs::read_to_string(&s.path)?;
        let is_match = if ignore_case {
            content.to_lowercase().contains(&pattern_normalized)
        } else {
            content.contains(pattern)
        };

        if is_match {
            matches.push(s);
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_search_sessions() {
        let tmp = tempdir().unwrap();
        let s1_path = tmp.path().join("s1.json");
        fs::write(&s1_path, "find me").unwrap();

        let s1 = Session {
            id: "s1".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: s1_path,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            size: 0,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        };

        let binding = [s1];
        let results = search_sessions(&binding, "find", false).unwrap();
        assert_eq!(results.len(), 1);

        let binding_none = [results[0].clone()];
        let results_none = search_sessions(&binding_none, "absent", false).unwrap();
        assert_eq!(results_none.len(), 0);
    }
}
