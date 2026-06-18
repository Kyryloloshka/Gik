use crate::core::objects::get_commit_tree_files;
use crate::core::storage::Storage;
use crate::core::workspace::{get_status, restore_workspace};
use crate::error::Result;

#[cfg(test)]
mod tests;

pub fn checkout(storage: &Storage, hash: &str, force: bool) -> Result<()> {
    let current_head = storage.commits().get_current_head()?;

    // 1. Safety Check: Check for uncommitted changes if force is false
    if !force {
        let status = get_status(storage)?;
        if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty()
        {
            return Err(crate::error::GikError::DirtyWorkspace(
                "Working directory is not clean. Use --force to discard changes.".to_string(),
            ));
        }
    }

    let (full_hash, resolved_bookmark) = crate::core::utils::resolve_hash(storage, hash)?;

    // 3. Ensure the found hash is a commit
    let meta = storage
        .commits()
        .get_commit_meta(&full_hash)?
        .ok_or_else(|| {
            crate::error::GikError::NotFound(format!("Object {} is not a commit", full_hash))
        })?;

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
    storage.log_action(crate::core::models::UndoAction::Checkout {
        old_head: current_head,
        new_head: full_hash.clone(),
    });

    println!("Switched to commit {}", full_hash);
    storage.commit_batch(
        crate::core::models::CommandType::Checkout,
        &format!("gik checkout {}", hash),
    )?;
    Ok(())
}
