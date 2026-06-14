use crate::core::storage::Storage;
use crate::error::{Result, GikError};
use crate::core::hash::Hash;
use crate::core::objects::get_commit_tree_files;
use crate::core::workspace::{get_status, restore_workspace};

#[cfg(test)]
mod tests;

pub fn checkout(storage: &Storage, hash: &str, force: bool) -> Result<()> {
    let current_head = storage.commits().get_current_head()?;

    // 1. Safety Check: Check for uncommitted changes if force is false
    if !force {
        let status = get_status(storage)?;
        if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
            return Err(GikError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "You have uncommitted changes. Use --force to discard them."
            )));
        }
    }

    // 2. Parse Hash: Support bookmark names and prefix matching
    let mut resolved_bookmark = None;
    let full_hash = if let Some(h) = storage.refs().get_ref(hash)? {
        resolved_bookmark = Some(hash.to_string());
        h
    } else if hash.len() == 40 {
        Hash::from_hex(hash).map_err(|e| GikError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?
    } else {
        let all_objects = storage.objects().list_all_objects()?;
        let matches: Vec<Hash> = all_objects
            .into_iter()
            .filter(|h| h.to_string().starts_with(hash))
            .collect();

        if matches.is_empty() {
            return Err(GikError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Commit not found: {}", hash)
            )));
        }
        if matches.len() > 1 {
            return Err(GikError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Ambiguous hash: {}", hash)
            )));
        }
        matches[0]
    };

    // 3. Ensure the found hash is a commit
    let meta = storage.commits().get_commit_meta(&full_hash)?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Object {} is not a commit", full_hash)))?;

    // 4. Restore Workspace
    restore_workspace(storage, &full_hash)?;

    // 5. Update Index and HEAD
    let tree_files = get_commit_tree_files(storage, &meta.tree_hash)?;
    storage.index().set_index_state(&tree_files)?;
    storage.commits().set_head(&full_hash)?;

    // 6. Update Session Hint
    if let Some(name) = resolved_bookmark {
        storage.session().set_current_bookmark(&name)?;
    } else {
        storage.session().clear_current_bookmark()?;
    }

    // 7. Log transaction for undo
    storage.log_transaction_manual(crate::core::models::UndoAction::Checkout {
        old_head: current_head,
        new_head: full_hash,
    })?;

    println!("Switched to commit {}", full_hash);
    Ok(())
}
