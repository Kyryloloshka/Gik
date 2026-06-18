use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::models::IndexEntry;
use crate::core::storage::repository::*;
use crate::core::storage::services::log_transaction;
use redb::ReadableTable;
use std::io::Read;
use std::collections::HashMap;

pub struct IndexService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> IndexService<'a> {
    pub fn stage_file<R: Read>(&self, path: &str, hash: &Hash, size: u64, mtime: u64, reader: R) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let old_entry = {
                let table = write_txn.open_table(STAGE_INDEX_V2)?;
                let guard = table.get(path)?;
                if let Some(g) = guard {
                    Some(bincode::deserialize(g.value()).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?)
                } else {
                    None
                }
            };

            let mut objects = write_txn.open_table(OBJECTS)?;
            let exists = objects.get(&hash.0)?.is_some();
            if !exists {
                let mut compressed = Vec::new();
                crate::core::objects::compress_blob(reader, size, &mut compressed)?;
                objects.insert(&hash.0, compressed)?;
            }

            let mut index = write_txn.open_table(STAGE_INDEX_V2)?;
            let new_entry = IndexEntry { hash: hash.clone(), size, mtime };
            let encoded = bincode::serialize(&new_entry).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            index.insert(path, encoded.as_slice())?;

            log_transaction(&write_txn, crate::core::models::UndoAction::Unstage {
                path: path.to_string(),
                old_entry,
            })?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn unstage_file(&self, path: &str) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut index = write_txn.open_table(STAGE_INDEX_V2)?;
            let entry: Option<IndexEntry> = {
                if let Some(guard) = index.get(path)? {
                    Some(bincode::deserialize(guard.value()).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?)
                } else {
                    None
                }
            };

            if let Some(e) = entry {
                index.remove(path)?;
                log_transaction(&write_txn, crate::core::models::UndoAction::Stage {
                    path: path.to_string(),
                    entry: e,
                })?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_staged_entry(&self, path: &str) -> Result<Option<IndexEntry>> {
        let entry = {
            let read_txn = self.repo.db.begin_read()?;
            let table = read_txn.open_table(STAGE_INDEX_V2)?;
            let guard_opt = table.get(path)?;
            if let Some(guard) = guard_opt {
                Some(bincode::deserialize(guard.value()).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?)
            } else {
                None
            }
        };
        Ok(entry)
    }

    pub fn get_staged_hash(&self, path: &str) -> Result<Option<Hash>> {
        Ok(self.get_staged_entry(path)?.map(|e| e.hash))
    }

    pub fn set_staged_entry(&self, path: &str, entry: &IndexEntry) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut index = write_txn.open_table(STAGE_INDEX_V2)?;
            let encoded = bincode::serialize(entry).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            index.insert(path, encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn set_staged_hash(&self, path: &str, hash: &Hash) -> Result<()> {
        let entry = IndexEntry { hash: hash.clone(), size: 0, mtime: 0 };
        self.set_staged_entry(path, &entry)
    }

    pub fn get_all_staged_entries(&self) -> Result<Vec<(String, IndexEntry)>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(STAGE_INDEX_V2)?;
        let mut entries = Vec::new();
        for result in table.iter()? {
            let (path, value) = result?;
            let entry: IndexEntry = bincode::deserialize(value.value()).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            entries.push((path.value().to_string(), entry));
        }
        Ok(entries)
    }

    pub fn get_all_staged_files(&self) -> Result<Vec<(String, Hash)>> {
        let entries = self.get_all_staged_entries()?;
        Ok(entries.into_iter().map(|(p, e)| (p, e.hash)).collect())
    }

    pub fn set_index_state(&self, files: &HashMap<String, Hash>) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut index = write_txn.open_table(STAGE_INDEX_V2)?;
            let mut keys = Vec::new();
            for result in index.iter()? {
                let (path, _) = result?;
                keys.push(path.value().to_string());
            }
            for key in keys {
                index.remove(key.as_str())?;
            }

            for (path, hash) in files {
                let entry = IndexEntry { hash: hash.clone(), size: 0, mtime: 0 };
                let encoded = bincode::serialize(&entry).map_err(|e| crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                index.insert(path.as_str(), encoded.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
