use crate::error::Result;
use crate::core::storage::Storage;

pub fn undo(storage: &Storage) -> Result<()> {
    if let Some(record) = storage.pop_last_transaction()? {
        storage.apply_undo(record.action)?;
        println!("Undo successful");
    } else {
        println!("No transactions to undo");
    }

    Ok(())
}
