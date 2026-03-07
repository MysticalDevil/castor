use castor::config::Config;
use castor::core::Registry;
use castor::ops::Executor;
use std::fs;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

pub struct TestContext {
    pub _tmp_dir: TempDir,
    pub config: Config,
}

impl TestContext {
    pub fn new() -> Self {
        let tmp = tempdir().unwrap();
        let base = tmp.path();

        let sessions_path = base.join("gemini_tmp");
        let trash_path = base.join("trash");
        let audit_path = base.join("audit");

        fs::create_dir_all(&sessions_path).unwrap();
        fs::create_dir_all(&trash_path).unwrap();
        fs::create_dir_all(&audit_path).unwrap();

        let config = Config {
            gemini_sessions_path: sessions_path,
            trash_path,
            audit_path,
            cache_path: base.join("cache"),
            dry_run_by_default: false,
            icon_set: castor::utils::icons::IconSet::Ascii,
        };

        Self {
            _tmp_dir: tmp,
            config,
        }
    }

    /// Helper to seed a session
    pub fn seed_session(&self, project: &str, id: &str, content: &str, days_ago: i64) -> PathBuf {
        let chat_dir = self.config.gemini_sessions_path.join(project).join("chats");
        fs::create_dir_all(&chat_dir).unwrap();

        let file_path = chat_dir.join(id);
        let data = format!(
            r#"{{"messages": [{{"type": "user", "content": "{}"}}]}}"#,
            content
        );
        fs::write(&file_path, data).unwrap();

        // Set mtime
        let mtime = filetime::FileTime::from_unix_time(
            chrono::Utc::now().timestamp() - (days_ago * 86400),
            0,
        );
        filetime::set_file_mtime(&file_path, mtime).unwrap();

        file_path
    }

    pub fn get_executor(&self) -> Executor {
        Executor::new(serde_json::from_str(&serde_json::to_string(&self.config).unwrap()).unwrap())
    }

    pub fn get_registry(&self) -> Registry {
        Registry::new(
            &self.config.gemini_sessions_path,
            &self.config.cache_path.join("metadata.json"),
        )
    }
}
