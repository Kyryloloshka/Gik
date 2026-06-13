use crate::error::Result;
use crate::core::storage::Storage;
use std::fs::File;

pub fn init() -> Result<()> {
    Storage::new(".gik.db")?;
    Ok(())
}

pub fn stage(path: String) -> Result<()> {
    let storage = Storage::new(".gik.db")?;
    let file = File::open(&path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();

    // Hash
    let hash = crate::core::objects::hash_blob(&file, size)?;

    // Re-open for compression
    let file = File::open(&path)?;
    storage.stage_file(&path, &hash, size, file)?;

    Ok(())
}

pub fn commit(message: String) -> Result<()> {
    let storage = Storage::new(".gik.db")?;

    // 1. Get staged files
    let staged_files = storage.get_all_staged_files()?;
    if staged_files.is_empty() {
        println!("Nothing to commit");
        return Ok(());
    }

    // 2. Create Tree object
    let mut tree_entries = Vec::new();
    for (path, hash) in staged_files {
        // Git mode 100644 for regular files
        tree_entries.push((0o100644, path, hash));
    }
    // Sort entries by name for canonical tree
    tree_entries.sort_by(|a, b| a.1.cmp(&b.1));

    let tree_hash = crate::core::objects::hash_tree(&tree_entries)?;
    let mut tree_content = Vec::new();
    crate::core::objects::compress_tree(&tree_entries, &mut tree_content)?;

    // 3. Get current HEAD (parent)
    let parent_hash = storage.get_current_head()?;
    let parent_hashes = if let Some(p) = parent_hash {
        vec![p]
    } else {
        vec![]
    };

    // 4. Create Commit object
    let author = "Gik User";
    let email = "user@gik.local";
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let commit_hash = crate::core::objects::hash_commit(
        tree_hash,
        &parent_hashes,
        author,
        email,
        timestamp,
        &message,
    )?;
    let mut commit_content = Vec::new();
    crate::core::objects::compress_commit(
        tree_hash,
        &parent_hashes,
        author,
        email,
        timestamp,
        &message,
        &mut commit_content,
    )?;

    // 5. Update Storage
    storage.commit_transaction(
        tree_hash,
        tree_content,
        commit_hash,
        commit_content,
        parent_hash,
    )?;

    println!("[main {}] {}", &hex::encode(commit_hash)[..7], message);

    Ok(())
}

pub fn log() -> Result<()> {
    // Logic for gik log will go here
    Ok(())
}

pub fn undo() -> Result<()> {
    // Logic for gik undo will go here
    Ok(())
}

#[cfg(test)]
mod tests;
