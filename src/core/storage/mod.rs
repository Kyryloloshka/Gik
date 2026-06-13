use crate::error::Result;
use redb::{Database, TableDefinition, ReadableTable};
use std::path::Path;
use std::io::Read;

pub const OBJECTS: TableDefinition<&[u8; 20], Vec<u8>> = TableDefinition::new("objects");
pub const COMMITS_METADATA: TableDefinition<&[u8; 20], Vec<u8>> = TableDefinition::new("commits_metadata");
pub const HEADS: TableDefinition<&[u8; 20], u8> = TableDefinition::new("heads");
pub const STAGE_INDEX: TableDefinition<&str, &[u8; 20]> = TableDefinition::new("stage_index");
pub const TRANSACTION_LOG: TableDefinition<u64, Vec<u8>> = TableDefinition::new("transaction_log");

pub struct Storage {
    db: Database,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::create(path)?;
        let storage = Self { db };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let _ = write_txn.open_table(OBJECTS)?;
            let _ = write_txn.open_table(COMMITS_METADATA)?;
            let _ = write_txn.open_table(HEADS)?;
            let _ = write_txn.open_table(STAGE_INDEX)?;
            let _ = write_txn.open_table(TRANSACTION_LOG)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn contains_object(&self, hash: &[u8; 20]) -> Result<bool> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS)?;
        let exists = table.get(hash)?.is_some();
        Ok(exists)
    }

    fn log_transaction(
        &self,
        write_txn: &redb::WriteTransaction,
        action: crate::core::models::UndoAction,
    ) -> Result<()> {
        let mut table = write_txn.open_table(TRANSACTION_LOG)?;
        
        let next_id = table
            .iter()?
            .rev()
            .next()
            .transpose()?
            .map(|(id, _)| id.value() + 1)
            .unwrap_or(1);

        let record = crate::core::models::TransactionRecord {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            action,
        };

        let bytes = bincode::serialize(&record)?;
        table.insert(next_id, bytes)?;
        Ok(())
    }

    pub fn stage_file<R: Read>(&self, path: &str, hash: &[u8; 20], size: u64, reader: R) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let old_hash = {
                let table = write_txn.open_table(STAGE_INDEX)?;
                let guard = table.get(path)?;
                guard.map(|g| *g.value())
            };

            let mut objects = write_txn.open_table(OBJECTS)?;
            let exists = objects.get(hash)?.is_some();
            if !exists {
                let mut compressed = Vec::new();
                crate::core::objects::compress_blob(reader, size, &mut compressed)?;
                objects.insert(hash, compressed)?;
            }

            let mut index = write_txn.open_table(STAGE_INDEX)?;
            index.insert(path, hash)?;

            self.log_transaction(&write_txn, crate::core::models::UndoAction::Unstage {
                path: path.to_string(),
                old_hash,
            })?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_staged_hash(&self, path: &str) -> Result<Option<[u8; 20]>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(STAGE_INDEX)?;
        let hash = table.get(path)?.map(|guard| *guard.value());
        Ok(hash)
    }

    pub fn get_all_staged_files(&self) -> Result<Vec<(String, [u8; 20])>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(STAGE_INDEX)?;
        let mut entries = Vec::new();
        for result in table.iter()? {
            let (path, hash) = result?;
            entries.push((path.value().to_string(), *hash.value()));
        }
        Ok(entries)
    }

    pub fn get_current_head(&self) -> Result<Option<[u8; 20]>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(HEADS)?;
        let mut heads = Vec::new();
        for result in table.iter()? {
            let (hash, _) = result?;
            heads.push(*hash.value());
        }
        Ok(heads.first().copied())
    }

    pub fn get_commit_meta(&self, hash: &[u8; 20]) -> Result<Option<crate::core::models::CommitMeta>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(COMMITS_METADATA)?;
        let guard = table.get(hash)?;
        if let Some(g) = guard {
            let meta = bincode::deserialize(&g.value())?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }

    pub fn commit_transaction(
        &self,
        tree_hash: [u8; 20],
        tree_content: Vec<u8>,
        commit_hash: [u8; 20],
        commit_content: Vec<u8>,
        parent_hash: Option<[u8; 20]>,
        meta: crate::core::models::CommitMeta,
    ) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS)?;
            objects.insert(&tree_hash, tree_content)?;
            objects.insert(&commit_hash, commit_content)?;

            let mut metadata = write_txn.open_table(COMMITS_METADATA)?;
            let meta_bytes = bincode::serialize(&meta)?;
            metadata.insert(&commit_hash, meta_bytes)?;

            let mut heads = write_txn.open_table(HEADS)?;
            if let Some(parent) = parent_hash {
                heads.remove(&parent)?;
            }
            heads.insert(&commit_hash, 1)?;

            let mut index = write_txn.open_table(STAGE_INDEX)?;
            let keys: Vec<String> = index.iter()?.map(|r| r.unwrap().0.value().to_string()).collect();
            for key in keys {
                index.remove(key.as_str())?;
            }

            self.log_transaction(&write_txn, crate::core::models::UndoAction::RevertCommit {
                old_head: parent_hash,
                new_head: commit_hash,
            })?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn pop_last_transaction(&self) -> Result<Option<crate::core::models::TransactionRecord>> {
        let write_txn = self.db.begin_write()?;
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
                    return Err(crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Transaction log entry missing")));
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
        let write_txn = self.db.begin_write()?;
        {
            match action {
                crate::core::models::UndoAction::Unstage { path, old_hash } => {
                    let mut index = write_txn.open_table(STAGE_INDEX)?;
                    if let Some(hash) = old_hash {
                        index.insert(path.as_str(), &hash)?;
                    } else {
                        index.remove(path.as_str())?;
                    }
                }
                crate::core::models::UndoAction::RevertCommit { old_head, new_head } => {
                    let mut heads = write_txn.open_table(HEADS)?;
                    heads.remove(&new_head)?;
                    if let Some(old) = old_head {
                        heads.insert(&old, 1)?;
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
