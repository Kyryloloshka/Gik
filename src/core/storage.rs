use crate::error::Result;
use redb::{Database, TableDefinition};
use std::path::Path;

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
}
