use crate::commands::checkout::checkout;
use crate::core::storage::Storage;
use crate::core::workspace::get_status;
use crate::error::{GikError, Result};

pub fn pull(storage: &Storage) -> Result<()> {
    let status = get_status(storage)?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Err(GikError::DirtyWorkspace("Working directory has uncommitted changes. Please commit or stash them before pulling.".to_string()));
    }

    let current_bookmark = storage.session().get_current_bookmark()?;
    let branch = match &current_bookmark {
        Some(b) => b,
        None => return Err(GikError::Branch("You are not currently on a branch (Detached HEAD state). Please checkout a branch before pulling.".to_string())),
    };

    let local_head = storage.commits().get_current_head()?;

    println!("Discovering remote refs for branch '{}'...", branch);
    let remote_head = crate::core::fetch_ops::fetch_remote_branch(storage, branch, local_head.as_ref())?
        .ok_or_else(|| GikError::Branch(format!("Remote branch '{}' not found", branch)))?;

    if local_head == Some(remote_head) {
        println!("Already up to date.");
        return Ok(());
    }

    if local_head.is_some() {
        println!("Merging remote changes...");
        crate::commands::merge::merge(storage, &remote_head.to_string())?;
    } else {
        println!("Updating refs...");
        storage.refs().set_ref(branch, &remote_head)?;
        checkout(storage, branch, true)?;
    }

    println!("Pull successful!");
    Ok(())
}
