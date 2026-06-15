use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::models::UndoAction;

pub fn undo(storage: &Storage) -> Result<()> {
    if let Some(record) = storage.undo_service().pop_last_transaction()? {
        storage.undo_service().apply_undo(record.action.clone())?;
        
        // Sync disk if HEAD changed
        match record.action {
            UndoAction::RevertCommit { old_head, .. } => {
                if let Some(h) = old_head {
                    crate::core::workspace::restore_workspace(storage, &h)?;
                    // Sync index too
                    let meta = storage.commits().get_commit_meta(&h)?
                        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Parent commit meta missing during undo"))?;
                    let tree_files = crate::core::objects::get_commit_tree_files(storage, &meta.tree_hash)?;
                    storage.index().set_index_state(&tree_files)?;
                } else {
                    // First commit was reverted. Clear disk and index?
                    // For now, let's just clear index.
                    storage.index().set_index_state(&std::collections::HashMap::new())?;
                }
                println!("Undo successful: reverted commit");
            }
            UndoAction::Checkout { old_head, .. } => {
                if let Some(h) = old_head {
                    crate::core::workspace::restore_workspace(storage, &h)?;
                    let meta = storage.commits().get_commit_meta(&h)?
                        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Commit meta missing during undo checkout"))?;
                    let tree_files = crate::core::objects::get_commit_tree_files(storage, &meta.tree_hash)?;
                    storage.index().set_index_state(&tree_files)?;
                } else {
                     storage.index().set_index_state(&std::collections::HashMap::new())?;
                }
                println!("Undo successful: reverted checkout");
            }
            UndoAction::Unstage { .. } => println!("Undo successful: unstaged file"),
            UndoAction::Stage { .. } => println!("Undo successful: staged file"),
        }
    } else {
        println!("No transactions to undo");
    }

    Ok(())
}

#[cfg(test)]
mod tests;
