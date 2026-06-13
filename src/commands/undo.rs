use crate::error::Result;
use crate::core::storage::Storage;

pub fn undo() -> Result<()> {
    let storage = Storage::new(crate::config::DB_PATH)?;
    
    if let Some(record) = storage.pop_last_transaction()? {
        storage.apply_undo(record.action)?;
        println!("Undo successful");
    } else {
        println!("No transactions to undo");
    }

    Ok(())
}
