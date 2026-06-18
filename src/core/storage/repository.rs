use crate::error::Result;
use redb::{Database, TableDefinition};
use std::path::Path;

pub const COMMITS_METADATA: TableDefinition<&[u8; 20], Vec<u8>> =
    TableDefinition::new("commits_metadata");
pub const HEADS: TableDefinition<&[u8; 20], u8> = TableDefinition::new("heads");
pub const STAGE_INDEX: TableDefinition<&str, &[u8]> = TableDefinition::new("stage_index");
pub const REFS: TableDefinition<&str, &[u8; 20]> = TableDefinition::new("refs");
pub const TRANSACTION_LOG: TableDefinition<u64, Vec<u8>> = TableDefinition::new("transaction_log");
pub const REDO_LOG: TableDefinition<u64, Vec<u8>> = TableDefinition::new("redo_log");
pub const SESSION: TableDefinition<&str, &str> = TableDefinition::new("session");
pub const CONFIG: TableDefinition<&str, &str> = TableDefinition::new("config");
pub const PACKFILES: TableDefinition<u32, &str> = TableDefinition::new("packfiles");
pub const PACKFILE_INDEX: TableDefinition<&[u8; 20], (u32, u64)> =
    TableDefinition::new("packfile_index");

pub enum DbConnection {
    ReadWrite(Database),
    ReadOnly(redb::ReadOnlyDatabase),
}

impl DbConnection {
    pub fn begin_read(&self) -> std::result::Result<redb::ReadTransaction, redb::TransactionError> {
        use redb::ReadableDatabase;
        match self {
            Self::ReadWrite(db) => db.begin_read(),
            Self::ReadOnly(db) => db.begin_read(),
        }
    }
    pub fn begin_write(
        &self,
    ) -> std::result::Result<redb::WriteTransaction, redb::TransactionError> {
        match self {
            Self::ReadWrite(db) => db.begin_write(),
            Self::ReadOnly(_) => panic!("Cannot begin write on read-only database"),
        }
    }
}

pub struct Repository {
    pub(crate) db: DbConnection,
}

impl Repository {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut attempts = 0;
        let db = loop {
            match Database::create(path.as_ref()) {
                Ok(db) => break db,
                Err(e) => {
                    attempts += 1;
                    if attempts >= 30 {
                        // Max 3 seconds timeout
                        return Err(crate::error::GikError::DbOpen(e));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        };
        let repo = Self {
            db: DbConnection::ReadWrite(db),
        };
        repo.init_tables()?;
        Ok(repo)
    }

    pub fn open_read_only<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = redb::Builder::new()
            .open_read_only(path.as_ref())
            .map_err(|e| crate::error::GikError::DbOpen(e.into()))?;
        Ok(Self {
            db: DbConnection::ReadOnly(db),
        })
    }

    fn init_tables(&self) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let _ = write_txn.open_table(COMMITS_METADATA)?;
            let _ = write_txn.open_table(HEADS)?;
            let _ = write_txn.open_table(STAGE_INDEX)?;
            let _ = write_txn.open_table(REFS)?;
            let _ = write_txn.open_table(TRANSACTION_LOG)?;
            let _ = write_txn.open_table(REDO_LOG)?;
            let _ = write_txn.open_table(SESSION)?;
            let _ = write_txn.open_table(CONFIG)?;
            let _ = write_txn.open_table(PACKFILES)?;
            let _ = write_txn.open_table(PACKFILE_INDEX)?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
