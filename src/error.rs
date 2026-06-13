use thiserror::Error;

#[derive(Error, Debug)]
pub enum GikError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Db(#[from] redb::Error),
    #[error("Database opening error: {0}")]
    DbOpen(#[from] redb::DatabaseError),
    #[error("Transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
    #[error("Table error: {0}")]
    Table(#[from] redb::TableError),
    #[error("Commit error: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Invalid hash length")]
    InvalidHash,
}

pub type Result<T> = std::result::Result<T, GikError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let gik_error: GikError = io_error.into();
        match gik_error {
            GikError::Io(_) => (),
            _ => panic!("Expected GikError::Io"),
        }
    }

    #[test]
    fn test_display() {
        let err = GikError::InvalidHash;
        assert_eq!(format!("{}", err), "Invalid hash length");
    }
}
