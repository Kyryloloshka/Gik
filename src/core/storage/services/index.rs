use crate::core::hash::Hash;
use crate::core::models::IndexEntry;
use crate::core::storage::repository::*;
use crate::error::Result;
use redb::ReadableTable;
use std::collections::HashMap;
use std::io::Read;

fn deserialize_index_entry(value: &[u8]) -> Result<IndexEntry> {
    match bincode::deserialize(value) {
        Ok(e) => Ok(e),
        Err(_) if value.len() == 20 => {
            let mut h = [0u8; 20];
            h.copy_from_slice(value);
            Ok(IndexEntry {
                hash: Hash(h),
                size: 0,
                mtime: 0,
            })
        }
        Err(e) => Err(crate::error::GikError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e,
        ))),
    }
}

use crate::core::storage::Storage;

pub struct IndexService<'a> {
    pub(crate) storage: &'a Storage,
}

impl<'a> IndexService<'a> {
    pub fn stage_file<R: Read>(
        &self,
        path: &str,
        hash: &Hash,
        size: u64,
        mtime: u64,
        reader: R,
    ) -> Result<Option<IndexEntry>> {
        let exists = self.storage.objects().contains_object(hash)?;
        if !exists {
            crate::core::objects::compress_blob(reader, size, hash, self.storage)?;
        }

        let write_txn = self.storage.repo.db.begin_write()?;
        let result = {
            let old_entry = {
                let table = write_txn.open_table(STAGE_INDEX)?;
                let guard = table.get(path)?;
                if let Some(g) = guard {
                    Some(deserialize_index_entry(g.value())?)
                } else {
                    None
                }
            };

            let mut index = write_txn.open_table(STAGE_INDEX)?;
            let new_entry = IndexEntry {
                hash: hash.clone(),
                size,
                mtime,
            };
            let encoded = bincode::serialize(&new_entry).map_err(|e| {
                crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
            index.insert(path, encoded.as_slice())?;

            Ok(old_entry)
        };
        write_txn.commit()?;
        result
    }

    pub fn unstage_file(&self, path: &str) -> Result<Option<IndexEntry>> {
        let write_txn = self.storage.repo.db.begin_write()?;
        let result = {
            let mut index = write_txn.open_table(STAGE_INDEX)?;
            let entry: Option<IndexEntry> = {
                if let Some(guard) = index.get(path)? {
                    Some(deserialize_index_entry(guard.value())?)
                } else {
                    None
                }
            };

            if let Some(_e) = entry.clone() {
                index.remove(path)?;
            }
            Ok(entry)
        };
        write_txn.commit()?;
        result
    }

    pub fn get_staged_entry(&self, path: &str) -> Result<Option<IndexEntry>> {
        let entry = {
            let read_txn = self.storage.repo.db.begin_read()?;
            let table = read_txn.open_table(STAGE_INDEX)?;
            let guard_opt = table.get(path)?;
            if let Some(guard) = guard_opt {
                Some(deserialize_index_entry(guard.value())?)
            } else {
                None
            }
        };
        Ok(entry)
    }

    pub fn get_staged_hash(&self, path: &str) -> Result<Option<Hash>> {
        Ok(self.get_staged_entry(path)?.map(|e| e.hash))
    }

    pub fn set_staged_entry(&self, path: &str, entry: &IndexEntry) -> Result<Option<IndexEntry>> {
        let write_txn = self.storage.repo.db.begin_write()?;
        let result = {
            let old_entry = {
                let table = write_txn.open_table(STAGE_INDEX)?;
                let guard = table.get(path)?;
                if let Some(g) = guard {
                    Some(deserialize_index_entry(g.value())?)
                } else {
                    None
                }
            };
            let mut index = write_txn.open_table(STAGE_INDEX)?;
            let encoded = bincode::serialize(entry).map_err(|e| {
                crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
            index.insert(path, encoded.as_slice())?;
            Ok(old_entry)
        };
        write_txn.commit()?;
        result
    }

    pub fn set_staged_hash(&self, path: &str, hash: &Hash) -> Result<Option<IndexEntry>> {
        let entry = IndexEntry {
            hash: hash.clone(),
            size: 0,
            mtime: 0,
        };
        self.set_staged_entry(path, &entry)
    }

    pub fn get_all_staged_entries(&self) -> Result<Vec<(String, IndexEntry)>> {
        let read_txn = self.storage.repo.db.begin_read()?;
        let table = read_txn.open_table(STAGE_INDEX)?;
        let mut entries = Vec::new();
        for result in table.iter()? {
            let (path, value) = result?;
            let entry = deserialize_index_entry(value.value())?;
            entries.push((path.value().to_string(), entry));
        }
        Ok(entries)
    }

    pub fn get_all_staged_files(&self) -> Result<Vec<(String, Hash)>> {
        let entries = self.get_all_staged_entries()?;
        Ok(entries.into_iter().map(|(p, e)| (p, e.hash)).collect())
    }

    pub fn set_index_state(&self, files: &HashMap<String, Hash>) -> Result<()> {
        let write_txn = self.storage.repo.db.begin_write()?;
        {
            let mut index = write_txn.open_table(STAGE_INDEX)?;
            let mut keys = Vec::new();
            for result in index.iter()? {
                let (path, _) = result?;
                keys.push(path.value().to_string());
            }
            for key in keys {
                index.remove(key.as_str())?;
            }

            for (path, hash) in files {
                let entry = IndexEntry {
                    hash: hash.clone(),
                    size: 0,
                    mtime: 0,
                };
                let encoded = bincode::serialize(&entry).map_err(|e| {
                    crate::error::GikError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                })?;
                index.insert(path.as_str(), encoded.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
