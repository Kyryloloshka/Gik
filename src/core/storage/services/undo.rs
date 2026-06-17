use crate::error::Result;
use crate::core::storage::repository::*;
use crate::core::models::UndoAction;
use redb::ReadableTable;

pub struct UndoService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> UndoService<'a> {
    pub fn pop_last_transaction(&self) -> Result<Option<crate::core::models::TransactionRecord>> {
        let write_txn = self.repo.db.begin_write()?;
        let last_id = {
            let table = write_txn.open_table(TRANSACTION_LOG)?;
            let mut iter = table.iter()?;
            let last = iter.next_back().transpose()?;
            last.map(|(id, _)| id.value())
        };

        if let Some(id_val) = last_id {
            let record = {
                let table = write_txn.open_table(TRANSACTION_LOG)?;
                let bytes = table.get(id_val)?;
                if let Some(b) = bytes {
                    bincode::deserialize(&b.value())?
                } else {
                    return Err(crate::error::GikError::NotFound("Transaction log entry missing".to_string()));
                }
            };
            {
                let mut table = write_txn.open_table(TRANSACTION_LOG)?;
                table.remove(id_val)?;
            }
            write_txn.commit()?;
            Ok(Some(record))
        } else {
            write_txn.commit()?;
            Ok(None)
        }
    }

    pub fn apply_undo(&self, action: crate::core::models::UndoAction) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            match action {
                UndoAction::Unstage { path, old_hash } => {
                    let mut index = write_txn.open_table(STAGE_INDEX)?;
                    if let Some(hash) = old_hash {
                        index.insert(path.as_str(), &hash.0)?;
                    } else {
                        index.remove(path.as_str())?;
                    }
                }
                UndoAction::Stage { path, hash } => {
                    let mut index = write_txn.open_table(STAGE_INDEX)?;
                    index.insert(path.as_str(), &hash.0)?;
                }
                UndoAction::RevertCommit { old_head, new_head } => {
                    let mut heads = write_txn.open_table(HEADS)?;
                    heads.remove(&new_head.0)?;
                    if let Some(old) = old_head {
                        heads.insert(&old.0, 1)?;
                    }
                }
                UndoAction::Checkout { old_head, new_head } => {
                    let mut heads = write_txn.open_table(HEADS)?;
                    heads.remove(&new_head.0)?;
                    if let Some(old) = old_head {
                        heads.insert(&old.0, 1)?;
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
