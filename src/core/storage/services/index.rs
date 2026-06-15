use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::storage::repository::*;
use crate::core::storage::services::log_transaction;
use redb::ReadableTable;
use std::io::Read;
use std::collections::HashMap;

pub struct IndexService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> IndexService<'a> {
    pub fn stage_file<R: Read>(&self, path: &str, hash: &Hash, size: u64, reader: R) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let old_hash = {
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

            log_transaction(&write_txn, crate::core::models::UndoAction::Unstage {
                path: path.to_string(),
                old_hash,
            })?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn unstage_file(&self, path: &str) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut index = write_txn.open_table(STAGE_INDEX)?;
            let hash = {
                index.get(path)?.map(|guard| Hash(*guard.value()))
            };

            if let Some(h) = hash {
                index.remove(path)?;
                
                log_transaction(&write_txn, crate::core::models::UndoAction::Stage {
                    path: path.to_string(),
                    hash: h,
                })?;
            }
        }
        write_txn.commit()?;
        Ok(())
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

    pub fn set_index_state(&self, files: &HashMap<String, Hash>) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
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
                index.insert(path.as_str(), &hash.0)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
