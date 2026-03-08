use crate::core::cache::{CacheEntry, MetadataCache};
use crate::core::scanner::Scanner;
use crate::core::session::Session;
use crate::error::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Registry {
    pub sessions: Vec<Arc<Session>>,
    pub session_indices: HashMap<String, usize>, // O(1) lookup map
    pub scanner: Scanner,
    pub cache: MetadataCache,
    pub cache_path: PathBuf,
}

impl Registry {
    pub fn new(base_path: &Path, cache_path: &Path) -> Self {
        Self {
            sessions: Vec::new(),
            session_indices: HashMap::new(),
            scanner: Scanner::new(base_path),
            cache: MetadataCache::load(cache_path),
            cache_path: cache_path.to_path_buf(),
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        let mut new_sessions = self.scanner.scan()?;

        // Apply cache and perform lazy validation
        for s in &mut new_sessions {
            if let Some(entry) = self.cache.get(&s.path, s.updated_at) {
                s.health = entry.health;
                s.name = entry.name;
                s.validation_notes = entry.notes;
            } else {
                s.deep_validate();
                self.cache.update(
                    s.path.clone(),
                    CacheEntry {
                        mtime: s.updated_at,
                        health: s.health.clone(),
                        name: s.name.clone(),
                        notes: s.validation_notes.clone(),
                    },
                );
            }
        }

        self.sessions = new_sessions.into_iter().map(Arc::new).collect();
        self.rebuild_index();
        self.cache.save(&self.cache_path)?;
        Ok(())
    }

    pub fn rebuild_index(&mut self) {
        self.session_indices.clear();
        for (i, s) in self.sessions.iter().enumerate() {
            self.session_indices.insert(s.id.clone(), i);
        }
    }

    pub fn find_by_id(&self, id: &str) -> Option<Arc<Session>> {
        self.session_indices
            .get(id)
            .and_then(|&i| self.sessions.get(i).cloned())
    }

    pub fn find(&self, query: &str) -> Option<Arc<Session>> {
        if let Some(s) = self.find_by_id(query) {
            return Some(s);
        }
        self.sessions
            .iter()
            .find(|s| s.name.as_ref().is_some_and(|n| n == query))
            .cloned()
    }

    pub fn list(&self) -> &[Arc<Session>] {
        &self.sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_registry_with_cache() {
        let tmp = tempdir().unwrap();
        let base = tmp.path().join("gemini");
        let cache_file = tmp.path().join("cache.json");
        let chat_dir = base.join("p1/chats");
        fs::create_dir_all(&chat_dir).unwrap();

        let s_path = chat_dir.join("session-2026-03-08T12-00-aaaa1111.json");
        fs::write(&s_path, r#"{"messages": [{"type":"user","content":"hi"}]}"#).unwrap();

        let mut registry = Registry::new(&base, &cache_file);
        registry.reload().unwrap();
        assert_eq!(registry.list()[0].name, Some("hi".into()));

        // Modify cache file manually to test reuse
        let mut cache = MetadataCache::load(&cache_file);
        if let Some(entry) = cache.entries.get_mut(&s_path) {
            entry.name = Some("cached_name".into());
        }
        cache.save(&cache_file).unwrap();

        let mut registry2 = Registry::new(&base, &cache_file);
        registry2.reload().unwrap();
        assert_eq!(registry2.list()[0].name, Some("cached_name".into()));
    }
}
