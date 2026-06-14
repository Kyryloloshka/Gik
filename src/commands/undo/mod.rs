use crate::error::Result;
use crate::core::storage::Storage;

pub fn undo(storage: &Storage) -> Result<()> {
    if let Some(record) = storage.undo_service().pop_last_transaction()? {
        storage.undo_service().apply_undo(record.action)?;
        println!("Undo successful");
    } else {
        println!("No transactions to undo");
    }

    Ok(())
}

#[cfg(test)]
mod tests;
