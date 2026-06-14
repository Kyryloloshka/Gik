use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::storage::repository::*;
use redb::ReadableTable;
use std::io::Read;

pub struct IndexService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> IndexService<'a> {
    pub fn stage_file<R: Read>(&self, path: &str, hash: &Hash, size: u64, reader: R) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let _old_hash = {
                let table = write_txn.open_table(STAGE_INDEX)?;
                let guard = table.get(path)?;
                guard.map(|g| Hash(*g.value()))
            };

            let mut objects = write_txn.open_table(OBJECTS)?;
            let exists = objects.get(&hash.0)?.is_some();
            if !exists {
                let mut compressed = Vec::new();
                crate::core::objects::compress_blob(reader, size, &mut compressed)?;
                objects.insert(&hash.0, compressed)?;
            }

            let mut index = write_txn.open_table(STAGE_INDEX)?;
            index.insert(path, &hash.0)?;

            // Note: log_transaction is internal to Storage facade or we can move it to a shared helper
            // For now, let's assume we'll use a unified way to log in the facade
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn unstage_file(&self, path: &str) -> Result<Option<Hash>> {
        let write_txn = self.repo.db.begin_write()?;
        let removed_hash = {
            let mut index = write_txn.open_table(STAGE_INDEX)?;
            let hash = {
                if let Some(guard) = index.get(path)? {
                    Some(Hash(*guard.value()))
                } else {
                    None
                }
            };

            if let Some(h) = hash {
                index.remove(path)?;
                Some(h)
            } else {
                None
            }
        };
        write_txn.commit()?;
        Ok(removed_hash)
    }

    pub fn get_staged_hash(&self, path: &str) -> Result<Option<Hash>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(STAGE_INDEX)?;
        let hash = table.get(path)?.map(|guard| Hash(*guard.value()));
        Ok(hash)
    }

    pub fn get_all_staged_files(&self) -> Result<Vec<(String, Hash)>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(STAGE_INDEX)?;
        let mut entries = Vec::new();
        for result in table.iter()? {
            let (path, hash) = result?;
            entries.push((path.value().to_string(), Hash(*hash.value())));
        }
        Ok(entries)
    }
}

pub struct CommitService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> CommitService<'a> {
    pub fn get_current_head(&self) -> Result<Option<Hash>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(HEADS)?;
        let mut heads = Vec::new();
        for result in table.iter()? {
            let (hash, _) = result?;
            heads.push(Hash(*hash.value()));
        }
        Ok(heads.first().copied())
    }

    pub fn get_commit_meta(&self, hash: &Hash) -> Result<Option<crate::core::models::CommitMeta>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(COMMITS_METADATA)?;
        let guard = table.get(&hash.0)?;
        if let Some(g) = guard {
            let meta = bincode::deserialize(&g.value())?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }
}

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
}
