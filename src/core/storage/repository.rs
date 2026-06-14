use crate::error::Result;
use redb::{Database, TableDefinition};
use std::path::Path;

pub const OBJECTS: TableDefinition<&[u8; 20], Vec<u8>> = TableDefinition::new("objects");
pub const COMMITS_METADATA: TableDefinition<&[u8; 20], Vec<u8>> = TableDefinition::new("commits_metadata");
pub const HEADS: TableDefinition<&[u8; 20], u8> = TableDefinition::new("heads");
pub const STAGE_INDEX: TableDefinition<&str, &[u8; 20]> = TableDefinition::new("stage_index");
pub const TRANSACTION_LOG: TableDefinition<u64, Vec<u8>> = TableDefinition::new("transaction_log");

pub struct Repository {
    pub(crate) db: Database,
}

impl Repository {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::create(path)?;
        let repo = Self { db };
        repo.init_tables()?;
        Ok(repo)
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
