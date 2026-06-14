pub mod repository;
pub mod services;

use crate::error::Result;
use self::repository::*;
use self::services::*;
use std::path::Path;

pub struct Storage {
    pub(crate) repo: Repository,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::new(path)?;
        Ok(Self { repo })
    }

    // Services accessors
    pub fn index(&self) -> IndexService<'_> {
        IndexService { repo: &self.repo }
    }

    pub fn commits(&self) -> CommitService<'_> {
        CommitService { repo: &self.repo }
    }

    pub fn undo_service(&self) -> UndoService<'_> {
        UndoService { repo: &self.repo }
    }

    pub fn objects(&self) -> ObjectService<'_> {
        ObjectService { repo: &self.repo }
    }

    pub fn refs(&self) -> RefService<'_> {
        RefService { repo: &self.repo }
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
