use crate::core::storage::Storage;
use crate::error::Result;

use std::io::{self, Write};

pub fn undo(storage: &Storage, yes: bool, list: bool) -> Result<()> {
    if list {
        let transactions = storage.undo_service().get_all_transactions()?;
        if transactions.is_empty() {
            println!("No transactions available to undo.");
            return Ok(());
        }
        for t in transactions {
            println!("{} - {}", t.id, t.description);
        }
        return Ok(());
    }

    if let Some(record) = storage.undo_service().peek_last_transaction()? {
        if !yes {
            print!("Undo: {} [y/N]? ", record.description);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                println!("Aborted.");
                return Ok(());
            }
        }

        // Actually pop it now
        let record = storage.undo_service().pop_last_transaction()?.unwrap();

        storage.undo_service().apply_undo_batch(&record)?;

        // Sync disk if HEAD changed
        match record.command {
            crate::core::models::CommandType::Commit
            | crate::core::models::CommandType::Checkout
            | crate::core::models::CommandType::Merge => {
                let current_head = storage.commits().get_current_head()?;
                if let Some(h) = current_head {
                    crate::core::workspace::restore_workspace(storage, &h)?;
                } else {
                    storage
                        .index()
                        .set_index_state(&std::collections::HashMap::new())?;
                }
            }
            _ => {}
        }

        storage.undo_service().push_redo(&record)?;
        println!("Undo successful: {}", record.description);
    } else {
        println!("No transactions to undo");
    }

    Ok(())
}

pub fn redo(storage: &Storage, yes: bool, list: bool) -> Result<()> {
    if list {
        let redos = storage.undo_service().get_all_redos()?;
        if redos.is_empty() {
            println!("No undone transactions available to redo.");
            return Ok(());
        }
        for t in redos {
            println!("{} - {}", t.id, t.description);
        }
        return Ok(());
    }

    if let Some(record) = storage.undo_service().peek_last_redo()? {
        if !yes {
            print!("Redo: {} [y/N]? ", record.description);
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                println!("Aborted.");
                return Ok(());
            }
        }

        // Actually pop it now
        let record = storage.undo_service().pop_last_redo()?.unwrap();

        storage.undo_service().apply_redo_batch(&record)?;

        // Sync disk if HEAD changed
        match record.command {
            crate::core::models::CommandType::Commit
            | crate::core::models::CommandType::Checkout
            | crate::core::models::CommandType::Merge => {
                let current_head = storage.commits().get_current_head()?;
                if let Some(h) = current_head {
                    crate::core::workspace::restore_workspace(storage, &h)?;
                }
            }
            _ => {}
        }

        storage.undo_service().push_transaction(&record)?;
        println!("Redo successful: {}", record.description);
    } else {
        println!("No undone transactions to redo");
    }

    Ok(())
}

#[cfg(test)]
mod tests;
