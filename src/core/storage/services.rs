use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::storage::repository::*;
use redb::ReadableTable;
use std::io::Read;
use std::collections::HashMap;

// Internal helper for logging transactions
pub(crate) fn log_transaction(
    write_txn: &redb::WriteTransaction,
    action: crate::core::models::UndoAction,
) -> Result<()> {
    let mut table = write_txn.open_table(TRANSACTION_LOG)?;
    
    let next_id = table
        .iter()?
        .next_back()
        .transpose()?
        .map(|(id, _)| id.value() + 1)
        .unwrap_or(1);

    let record = crate::core::models::TransactionRecord {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        action,
    };

    let bytes = bincode::serialize(&record)?;
    table.insert(next_id, bytes)?;
    Ok(())
}

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
                if let Some(guard) = index.get(path)? {
                    Some(Hash(*guard.value()))
                } else {
                    None
                }
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

    pub fn commit_transaction(
        &self,
        tree_hash: Hash,
        tree_content: Vec<u8>,
        commit_hash: Hash,
        commit_content: Vec<u8>,
        parent_hash: Option<Hash>,
        meta: crate::core::models::CommitMeta,
    ) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS)?;
            objects.insert(&tree_hash.0, tree_content)?;
            objects.insert(&commit_hash.0, commit_content)?;

            let mut metadata = write_txn.open_table(COMMITS_METADATA)?;
            let meta_bytes = bincode::serialize(&meta)?;
            metadata.insert(&commit_hash.0, meta_bytes)?;

            let mut heads = write_txn.open_table(HEADS)?;
            if let Some(parent) = parent_hash {
                heads.remove(&parent.0)?;
            }
            heads.insert(&commit_hash.0, 1)?;

            log_transaction(&write_txn, crate::core::models::UndoAction::RevertCommit {
                old_head: parent_hash,
                new_head: commit_hash,
            })?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn set_head(&self, new_head: &Hash) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            let mut heads = write_txn.open_table(HEADS)?;
            let mut keys = Vec::new();
            for result in heads.iter()? {
                let (hash, _) = result?;
                keys.push(*hash.value());
            }
            for key in keys {
                heads.remove(&key)?;
            }
            heads.insert(&new_head.0, 1)?;
        }
        write_txn.commit()?;
        Ok(())
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

    pub fn apply_undo(&self, action: crate::core::models::UndoAction) -> Result<()> {
        let write_txn = self.repo.db.begin_write()?;
        {
            match action {
                crate::core::models::UndoAction::Unstage { path, old_hash } => {
                    let mut index = write_txn.open_table(STAGE_INDEX)?;
                    if let Some(hash) = old_hash {
                        index.insert(path.as_str(), &hash.0)?;
                    } else {
                        index.remove(path.as_str())?;
                    }
                }
                crate::core::models::UndoAction::Stage { path, hash } => {
                    let mut index = write_txn.open_table(STAGE_INDEX)?;
                    index.insert(path.as_str(), &hash.0)?;
                }
                crate::core::models::UndoAction::RevertCommit { old_head, new_head } => {
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

pub struct ObjectService<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> ObjectService<'a> {
    pub fn contains_object(&self, hash: &Hash) -> Result<bool> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS)?;
        let exists = table.get(&hash.0)?.is_some();
        Ok(exists)
    }

    pub fn list_all_objects(&self) -> Result<Vec<Hash>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS)?;
        let mut hashes = Vec::new();
        for result in table.iter()? {
            let (hash_bytes, _) = result?;
            hashes.push(Hash(*hash_bytes.value()));
        }
        Ok(hashes)
    }

    pub fn get_object(&self, hash: &Hash) -> Result<Option<Vec<u8>>> {
        let read_txn = self.repo.db.begin_read()?;
        let table = read_txn.open_table(OBJECTS)?;
        let guard = table.get(&hash.0)?;
        Ok(guard.map(|g| g.value()))
    }

    pub fn get_blob_text(&self, hash: &Hash) -> Result<String> {
        if let Some(compressed_data) = self.get_object(hash)? {
            let (obj_type, _size, content) = crate::core::objects::decompress_object(&compressed_data[..])?;
            if obj_type != "blob" {
                return Err(crate::error::GikError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Object {} is not a blob (type: {})", hash, obj_type)
                )));
            }
            Ok(String::from_utf8_lossy(&content).to_string())
        } else {
            Err(crate::error::GikError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Blob {} not found in storage", hash)
            )))
        }
    }
}


