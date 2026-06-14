use crate::error::Result;
use crate::core::storage::Storage;

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn commit(storage: &Storage, message: String, staged: bool) -> Result<()> {
    if !staged {
        crate::core::workspace::auto_stage(storage)?;
    }

    // 1. Auto-remove files from index if they are now ignored
    // (This is now handled by core::workspace::auto_stage)
    // But if staged=true, we still might want to clean up.
    if staged {
        crate::core::workspace::clean_ignored_from_index(storage)?;
    }

    // 2. Get staged files
    let staged_files = storage.index().get_all_staged_files()?;
    if staged_files.is_empty() {
        println!("Nothing to commit");
        return Ok(());
    }

    // 3. Create Tree object using core domain logic
    let (tree_hash, tree_content) = crate::core::objects::tree::build_and_store_tree(storage, staged_files)?;

    // 4. Get current HEAD (parent)
    let parent_hash = storage.commits().get_current_head()?;
    let parent_hashes = if let Some(p) = parent_hash {
        vec![p]
    } else {
        vec![]
    };

    // 5. Create Commit object
    let author_name = crate::config::DEFAULT_AUTHOR_NAME;
    let author_email = crate::config::DEFAULT_AUTHOR_EMAIL;
    let author = format!("{} <{}>", author_name, author_email);
    let timestamp = current_timestamp();

    let commit_hash = crate::core::objects::hash_commit(
        tree_hash,
        &parent_hashes,
        author_name,
        author_email,
        timestamp,
        &message,
    )?;
    let mut commit_content = Vec::new();
    crate::core::objects::compress_commit(
        tree_hash,
        &parent_hashes,
        author_name,
        author_email,
        timestamp,
        &message,
        &mut commit_content,
    )?;

    // 6. Update Storage
    let meta = crate::core::models::CommitMeta {
        parent_hashes: parent_hashes.clone(),
        tree_hash,
        timestamp,
        author,
        message: message.clone(),
    };

    storage.commits().commit_transaction(
        tree_hash,
        tree_content,
        commit_hash,
        commit_content,
        parent_hash,
        meta,
    )?;

    println!("[main {}] {}", &hex::encode(commit_hash.0)[..7], message);

    Ok(())
}

#[cfg(test)]
mod tests;
