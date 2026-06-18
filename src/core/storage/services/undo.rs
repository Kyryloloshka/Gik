use crate::error::Result;
use crate::core::storage::repository::*;
use crate::core::models::UndoAction;
use redb::ReadableTable;

pub struct UndoService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> UndoService<'a> {
    pub fn pop_last_transaction(&self) -> Result<Option<crate::core::models::TransactionBatch>> {
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

    pub fn peek_last_transaction(&self) -> Result<Option<crate::core::models::TransactionBatch>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(TRANSACTION_LOG)?;
        let mut iter = table.iter()?;
        let next = iter.next_back();
        if let Some(Ok((_, val))) = next {
            let record = bincode::deserialize(&val.value())?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }


    pub fn pop_last_redo(&self) -> Result<Option<crate::core::models::TransactionBatch>> {
        let write_txn = self.repo.db.begin_write()?;
        let last_id = {
            let table = write_txn.open_table(REDO_LOG)?;
            let mut iter = table.iter()?;
            let last = iter.next_back().transpose()?;
            last.map(|(id, _)| id.value())
        };

        if let Some(id_val) = last_id {
            let record = {
                let table = write_txn.open_table(REDO_LOG)?;
                let bytes = table.get(id_val)?;
                if let Some(b) = bytes {
                    bincode::deserialize(&b.value())?
                } else {
                    return Err(crate::error::GikError::NotFound("Redo log entry missing".to_string()));
                }
            };
            {
                let mut table = write_txn.open_table(REDO_LOG)?;
                table.remove(id_val)?;
            }
            write_txn.commit()?;
            Ok(Some(record))
        } else {
            write_txn.commit()?;
            Ok(None)
        }
    }

    pub fn peek_last_redo(&self) -> Result<Option<crate::core::models::TransactionBatch>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(REDO_LOG)?;
        let mut iter = table.iter()?;
        let next = iter.next_back();
        if let Some(Ok((_, val))) = next {
            let record = bincode::deserialize(&val.value())?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }


    pub fn clear_redo_log(&self) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(REDO_LOG)?;
            let mut keys = Vec::new();
            for result in table.iter()? {
                let (id, _) = result?;
                keys.push(id.value());
            }
            for key in keys {
                table.remove(key)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn push_redo(&self, batch: &crate::core::models::TransactionBatch) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(REDO_LOG)?;
            let encoded = bincode::serialize(batch)
                .map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            table.insert(batch.id, encoded)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn push_transaction(&self, batch: &crate::core::models::TransactionBatch) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TRANSACTION_LOG)?;
            let encoded = bincode::serialize(batch)
                .map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            table.insert(batch.id, encoded)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_all_transactions(&self) -> Result<Vec<crate::core::models::TransactionBatch>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(TRANSACTION_LOG)?;
        let mut list = Vec::new();
        for item in table.iter()? {
            let (_, val) = item?;
            let record: crate::core::models::TransactionBatch = bincode::deserialize(&val.value())?;
            list.push(record);
        }
        list.reverse();
        Ok(list)
    }

    pub fn get_all_redos(&self) -> Result<Vec<crate::core::models::TransactionBatch>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(REDO_LOG)?;
        let mut list = Vec::new();
        for item in table.iter()? {
            let (_, val) = item?;
            let record: crate::core::models::TransactionBatch = bincode::deserialize(&val.value())?;
            list.push(record);
        }
        list.reverse();
        Ok(list)
    }

    pub fn apply_undo_batch(&self, batch: &crate::core::models::TransactionBatch) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            // Reverse the actions inside the batch so we undo them in LIFO order
            for action in batch.actions.iter().rev() {
                match action {
                    UndoAction::UpdateIndex { path, old_entry, new_entry: _ } => {
                        let mut index = write_txn.open_table(STAGE_INDEX)?;
                        if let Some(entry) = old_entry {
                            let encoded = bincode::serialize(&entry).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                            index.insert(path.as_str(), encoded.as_slice())?;
                        } else {
                            index.remove(path.as_str())?;
                        }
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
                    UndoAction::MoveBookmark { name, old_hash, new_hash: _ } => {
                        let mut refs = write_txn.open_table(REFS)?;
                        if let Some(old) = old_hash {
                            refs.insert(name.as_str(), &old.0)?;
                        } else {
                            refs.remove(name.as_str())?;
                        }
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn apply_redo_batch(&self, batch: &crate::core::models::TransactionBatch) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            // Apply the actions inside the batch in forward order
            for action in batch.actions.iter() {
                match action {
                    UndoAction::UpdateIndex { path, old_entry: _, new_entry } => {
                        let mut index = write_txn.open_table(STAGE_INDEX)?;
                        if let Some(entry) = new_entry {
                            let encoded = bincode::serialize(&entry).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                            index.insert(path.as_str(), encoded.as_slice())?;
                        } else {
                            index.remove(path.as_str())?;
                        }
                    }
                    UndoAction::RevertCommit { old_head, new_head } => {
                        let mut heads = write_txn.open_table(HEADS)?;
                        if let Some(old) = old_head {
                            heads.remove(&old.0)?;
                        }
                        heads.insert(&new_head.0, 1)?;
                    }
                    UndoAction::Checkout { old_head, new_head } => {
                        let mut heads = write_txn.open_table(HEADS)?;
                        if let Some(old) = old_head {
                            heads.remove(&old.0)?;
                        }
                        heads.insert(&new_head.0, 1)?;
                    }
                    UndoAction::MoveBookmark { name, old_hash: _, new_hash } => {
                        let mut refs = write_txn.open_table(REFS)?;
                        refs.insert(name.as_str(), &new_hash.0)?;
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
