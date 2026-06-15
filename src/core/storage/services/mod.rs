pub mod index;
pub mod commit;
pub mod undo;
pub mod object;
pub mod refs;
pub mod config;
pub mod session;

pub use index::IndexService;
pub use commit::CommitService;
pub use undo::UndoService;
pub use object::ObjectService;
pub use refs::RefService;
pub use config::ConfigService;
pub use session::SessionService;

use crate::error::Result;
use crate::core::storage::repository::*;
use redb::ReadableTable;

// Internal helper for logging transactions
pub fn log_transaction(
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
