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
        Ok(Self { repo, objects_dir })
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

    pub fn log_transaction_manual(&self, action: crate::core::models::UndoAction) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        crate::core::storage::services::log_transaction(&write_txn, action)?;
        write_txn.commit()?;
        Ok(())
    }
}


#[cfg(test)]
mod tests;
