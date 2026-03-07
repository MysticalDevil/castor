use crate::audit::{AuditEntry, AuditLogger, OperationType};
use crate::config::Config;
use crate::core::Session;
use crate::error::{CastorError, Result};
use chrono::Utc;
use uuid::Uuid;

pub struct Executor {
    pub config: Config,
    pub logger: AuditLogger,
}

impl Executor {
    pub fn new(config: Config) -> Self {
        let logger = AuditLogger::new(&config.audit_path);
        Self { config, logger }
    }

    /// Performs a soft delete (moves session to trash).
    pub fn delete_soft(&self, session: &Session, dry_run: bool) -> Result<Uuid> {
        let batch_id = Uuid::new_v4();
        // Include project_id in trash path to avoid collisions
        let target_path = self
            .config
            .trash_path
            .join(&session.project_id)
            .join(&session.id);

        let entry = AuditEntry {
            batch_id,
            timestamp: Utc::now(),
            op_type: OperationType::SoftDelete,
            session_id: session.id.clone(),
            original_path: session.path.clone(),
            target_path: Some(target_path.clone()),
        };

        if dry_run {
            println!(
                "[DRY-RUN] Would move session {} (Project: {}) to trash",
                session.id, session.project_id
            );
            return Ok(batch_id);
        }

        self.logger.log(&entry)?;

        if !session.path.exists() {
            return Err(CastorError::PathNotFound(session.path.clone()));
        }

        std::fs::create_dir_all(target_path.parent().unwrap())?;

        if session.path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::new();
            options.copy_inside = true;
            fs_extra::dir::move_dir(&session.path, &target_path, &options)
                .map_err(|e| CastorError::Execution(e.to_string()))?;
        } else {
            std::fs::rename(&session.path, &target_path)?;
        }

        Ok(batch_id)
    }

    /// Performs a hard delete (removes session permanently).
    pub fn delete_hard(&self, session: &Session, dry_run: bool) -> Result<Uuid> {
        let batch_id = Uuid::new_v4();

        let entry = AuditEntry {
            batch_id,
            timestamp: Utc::now(),
            op_type: OperationType::HardDelete,
            session_id: session.id.clone(),
            original_path: session.path.clone(),
            target_path: None,
        };

        if dry_run {
            println!(
                "[DRY-RUN] Would permanently delete session {} (Project: {})",
                session.id, session.project_id
            );
            return Ok(batch_id);
        }

        self.logger.log(&entry)?;

        if !session.path.exists() {
            return Err(CastorError::PathNotFound(session.path.clone()));
        }

        if session.path.is_dir() {
            std::fs::remove_dir_all(&session.path)?;
        } else {
            std::fs::remove_file(&session.path)?;
        }

        Ok(batch_id)
    }

    /// Restores a session from the trash.
    pub fn restore(&self, id: &str, dry_run: bool) -> Result<Uuid> {
        let batch_id = Uuid::new_v4();

        let history = self.logger.load_history()?;
        let latest_entry = history
            .iter()
            .rfind(|e| e.session_id == id && matches!(e.op_type, OperationType::SoftDelete))
            .ok_or_else(|| CastorError::BatchNotFound(id.to_string()))?;

        let trash_path = latest_entry
            .target_path
            .as_ref()
            .ok_or_else(|| CastorError::Audit("Missing target path in audit log".into()))?;

        if !trash_path.exists() {
            return Err(CastorError::PathNotFound(trash_path.clone()));
        }

        let entry = AuditEntry {
            batch_id,
            timestamp: Utc::now(),
            op_type: OperationType::Restore,
            session_id: id.to_string(),
            original_path: trash_path.clone(),
            target_path: Some(latest_entry.original_path.clone()),
        };

        if dry_run {
            println!(
                "[DRY-RUN] Would restore session {} to {:?}",
                id, latest_entry.original_path
            );
            return Ok(batch_id);
        }

        self.logger.log(&entry)?;

        std::fs::create_dir_all(latest_entry.original_path.parent().unwrap())?;

        if trash_path.is_dir() {
            let mut options = fs_extra::dir::CopyOptions::new();
            options.copy_inside = true;
            fs_extra::dir::move_dir(trash_path, &latest_entry.original_path, &options)
                .map_err(|e| CastorError::Execution(e.to_string()))?;
        } else {
            std::fs::rename(trash_path, &latest_entry.original_path)?;
        }

        Ok(batch_id)
    }
}
