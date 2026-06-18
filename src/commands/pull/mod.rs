use crate::commands::checkout::checkout;
use crate::core::network::client::GitClient;
use crate::core::network::packfile_decode::decode_packfile;
use crate::core::storage::Storage;
use crate::core::workspace::get_status;
use crate::error::{GikError, Result};

pub fn pull(storage: &Storage) -> Result<()> {
    let status = get_status(storage)?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Err(GikError::DirtyWorkspace("Working directory has uncommitted changes. Please commit or stash them before pulling.".to_string()));
    }

    let url = storage.config().get("remote.origin.url")?.ok_or_else(|| {
        GikError::Config(
            "No remote configured. Use 'gik config remote.origin.url <url>'".to_string(),
        )
    })?;

    let _ = dotenvy::dotenv(); // load .env if exists
    let token = std::env::var("GITHUB_TOKEN").ok();
    let client = GitClient::new(url, token);

    let current_bookmark = storage.session().get_current_bookmark()?;
    let branch = match &current_bookmark {
        Some(b) => b,
        None => return Err(GikError::Branch("You are not currently on a branch (Detached HEAD state). Please checkout a branch before pulling.".to_string())),
    };

    println!("Discovering remote refs for branch '{}'...", branch);
    let remote_head = client
        .discover_fetch_refs(branch)?
        .ok_or_else(|| GikError::Branch(format!("Remote branch '{}' not found", branch)))?;

    let local_head = storage.commits().get_current_head()?;

    if local_head == Some(remote_head) {
        println!("Already up to date.");
        return Ok(());
    }

    println!("Fetching from remote...");
    let mut reader = client.fetch_packfile(&remote_head, local_head.as_ref())?;

    println!("Decoding packfile...");
    decode_packfile(&mut reader, storage)?;

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
