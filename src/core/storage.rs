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

    pub fn stage_file<R: Read>(&self, path: &str, hash: &[u8; 20], size: u64, reader: R) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS)?;
            let exists = objects.get(hash)?.is_some();
            if !exists {
                let mut compressed = Vec::new();
                crate::core::objects::compress_blob(reader, size, &mut compressed)?;
                objects.insert(hash, compressed)?;
            }
            
            let mut index = write_txn.open_table(STAGE_INDEX)?;
            index.insert(path, hash)?;
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
        // For MVP, we assume a single HEAD
        let mut heads = Vec::new();
        for result in table.iter()? {
            let (hash, _) = result?;
            heads.push(*hash.value());
        }
        Ok(heads.first().copied())
    }

    pub fn commit_transaction(
        &self,
        tree_hash: [u8; 20],
        tree_content: Vec<u8>,
        commit_hash: [u8; 20],
        commit_content: Vec<u8>,
        parent_hash: Option<[u8; 20]>,
    ) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS)?;
            objects.insert(&tree_hash, tree_content)?;
            objects.insert(&commit_hash, commit_content)?;

            let mut heads = write_txn.open_table(HEADS)?;
            if let Some(parent) = parent_hash {
                heads.remove(&parent)?;
            }
            heads.insert(&commit_hash, 1)?; // 1 is just a dummy value

            let mut index = write_txn.open_table(STAGE_INDEX)?;
            // Clear index
            let keys: Vec<String> = index.iter()?.map(|r| r.unwrap().0.value().to_string()).collect();
            for key in keys {
                index.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_storage_init() {
        let tmp_file = NamedTempFile::new().unwrap();
        let storage = Storage::new(tmp_file.path()).unwrap();
        
        let read_txn = storage.db.begin_read().unwrap();
        assert!(read_txn.open_table(OBJECTS).is_ok());
        assert!(read_txn.open_table(COMMITS_METADATA).is_ok());
        assert!(read_txn.open_table(HEADS).is_ok());
        assert!(read_txn.open_table(STAGE_INDEX).is_ok());
        assert!(read_txn.open_table(TRANSACTION_LOG).is_ok());
    }

    #[test]
    fn test_storage_contains_object() {
        let tmp_file = NamedTempFile::new().unwrap();
        let storage = Storage::new(tmp_file.path()).unwrap();
        let hash = [0u8; 20];
        assert!(!storage.contains_object(&hash).unwrap());
    }

    #[test]
    fn test_storage_stage_file() {
        let tmp_file = NamedTempFile::new().unwrap();
        let storage = Storage::new(tmp_file.path()).unwrap();
        let path = "test.txt";
        let content = b"hello world";
        let hash = [1u8; 20]; // Dummy hash
        let size = content.len() as u64;
        
        storage.stage_file(path, &hash, size, &content[..]).unwrap();
        
        // Verify STAGE_INDEX
        let read_txn = storage.db.begin_read().unwrap();
        let index = read_txn.open_table(STAGE_INDEX).unwrap();
        let staged_hash_guard = index.get(path).unwrap().unwrap();
        assert_eq!(staged_hash_guard.value(), &hash);
        
        // Verify OBJECTS
        let objects = read_txn.open_table(OBJECTS).unwrap();
        assert!(objects.get(&hash).unwrap().is_some());
    }
}
