pub mod repository;
pub mod services;

use crate::error::Result;
use self::repository::*;
use self::services::index::IndexService;
use self::services::commit::CommitService;
use self::services::undo::UndoService;
use self::services::object::ObjectService;
use self::services::refs::RefService;
use self::services::config::ConfigService;
use std::path::{Path, PathBuf};

pub struct Storage {
    pub(crate) repo: Repository,
    pub(crate) objects_dir: PathBuf,
    pub(crate) pending_actions: std::cell::RefCell<Vec<crate::core::models::UndoAction>>,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let parent = path.as_ref().parent().unwrap_or(Path::new("."));
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| crate::error::GikError::Io(e))?;
        }
        let objects_dir = parent.join(crate::config::OBJECTS_DIR_NAME);
        if !objects_dir.exists() {
            std::fs::create_dir_all(&objects_dir).map_err(|e| crate::error::GikError::Io(e))?;
        }
        let repo = Repository::new(path.as_ref())?;
        Ok(Self { 
            repo, 
            objects_dir,
            pending_actions: std::cell::RefCell::new(Vec::new()),
        })
    }

    // Services accessors
    pub fn index(&self) -> IndexService<'_> {
        IndexService { storage: self }
    }

    pub fn commits(&self) -> CommitService<'_> {
        CommitService { repo: &self.repo }
    }

    pub fn undo_service(&self) -> UndoService<'_> {
        UndoService { repo: &self.repo }
    }

    pub fn objects(&self) -> ObjectService<'_> {
        ObjectService { objects_dir: &self.objects_dir }
    }

    pub fn refs(&self) -> RefService<'_> {
        RefService { repo: &self.repo }
    }

    pub fn session(&self) -> self::services::session::SessionService<'_> {
        self::services::session::SessionService { repo: &self.repo }
    }

    pub fn config(&self) -> ConfigService<'_> {
        ConfigService { repo: &self.repo }
    }

    pub fn log_action(&self, action: crate::core::models::UndoAction) {
        self.pending_actions.borrow_mut().push(action);
    }

    pub fn commit_batch(&self, command: crate::core::models::CommandType, description: &str) -> Result<()> {
        let actions = self.pending_actions.replace(Vec::new());
        if actions.is_empty() {
            return Ok(()); // Nothing to log
        }
        
        let batch = crate::core::models::TransactionBatch {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            command,
            description: description.to_string(),
            actions,
        };
        
        let undo_service = self.undo_service();
        undo_service.clear_redo_log()?;
        undo_service.push_transaction(&batch)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests;
