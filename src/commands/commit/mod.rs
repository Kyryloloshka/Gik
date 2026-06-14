use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::hash::Hash;

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn commit(storage: &Storage, message: String, staged: bool, explicit_branch: Option<String>) -> Result<()> {
    if !staged {
        crate::core::workspace::auto_stage(storage)?;
    }

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

    // 7. Resolve Bookmarks (Smart Deduplication)
    resolve_bookmarks(storage, &parent_hash, &commit_hash, explicit_branch)?;

    println!("[main {}] {}", &hex::encode(commit_hash.0)[..7], message);

    Ok(())
}

fn resolve_bookmarks(
    storage: &Storage, 
    parent_hash: &Option<Hash>, 
    new_hash: &Hash, 
    explicit_branch: Option<String>
) -> Result<()> {
    let refs = storage.refs().list_refs()?;
    let session_hint = storage.session().get_current_bookmark()?;

    // Case 1: Explicit branch provided
    if let Some(b) = explicit_branch {
        storage.refs().set_ref(&b, new_hash)?;
        storage.session().set_current_bookmark(&b)?;
        return Ok(());
    }

    // Case 2: Empty system (first commit)
    if refs.is_empty() {
        storage.refs().set_ref("main", new_hash)?;
        storage.session().set_current_bookmark("main")?;
        return Ok(());
    }

    if let Some(parent) = parent_hash {
        let parent_refs: Vec<String> = refs.iter()
            .filter(|(_, h)| h == parent)
            .map(|(n, _)| n.clone())
            .collect();

        if parent_refs.is_empty() {
            // No bookmarks at parent. Just stay anonymous.
            storage.session().clear_current_bookmark()?;
        } else if parent_refs.len() == 1 {
            // Exactly one bookmark. Move it automatically.
            let name = &parent_refs[0];
            storage.refs().set_ref(name, new_hash)?;
            storage.session().set_current_bookmark(name)?;
        } else {
            // Multiple bookmarks at parent. Use session hint if it's one of them.
            if let Some(hint) = session_hint {
                if parent_refs.contains(&hint) {
                    storage.refs().set_ref(&hint, new_hash)?;
                    // session hint remains the same
                } else {
                    println!("Warning: Multiple bookmarks found at parent ({}), but none match current session hint ({}). Bookmark left behind.", 
                             parent_refs.join(", "), hint);
                    storage.session().clear_current_bookmark()?;
                }
            } else {
                println!("Warning: Multiple bookmarks found at parent ({}). Please specify which one to move using --branch <name>. Bookmark left behind.", 
                         parent_refs.join(", "));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
