use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use std::fs::File;

pub fn init() -> Result<()> {
    Storage::new(crate::config::DB_PATH)?;
    Ok(())
}

pub fn stage(path: String) -> Result<()> {
    let storage = Storage::new(crate::config::DB_PATH)?;
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

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

type StagedTreeResult = (Vec<(u32, String, Hash)>, Hash, Vec<u8>);

fn build_staged_tree(
    staged_files: Vec<(String, Hash)>,
) -> crate::error::Result<StagedTreeResult> {
    let mut tree_entries = Vec::new();
    for (path, hash) in staged_files {
        tree_entries.push((crate::core::objects::tree::REGULAR_FILE_MODE, path, hash));
    }
    // Sort entries by name for canonical tree
    tree_entries.sort_by(|a, b| a.1.cmp(&b.1));

    let tree_hash = crate::core::objects::hash_tree(&tree_entries)?;
    let mut tree_content = Vec::new();
    crate::core::objects::compress_tree(&tree_entries, &mut tree_content)?;

    Ok((tree_entries, tree_hash, tree_content))
}

pub fn commit(message: String) -> Result<()> {
    let storage = Storage::new(crate::config::DB_PATH)?;

    // 1. Get staged files
    let staged_files = storage.get_all_staged_files()?;
    if staged_files.is_empty() {
        println!("Nothing to commit");
        return Ok(());
    }

    // 2. Create Tree object
    let (_tree_entries, tree_hash, tree_content) = build_staged_tree(staged_files)?;

    // 3. Get current HEAD (parent)
    let parent_hash = storage.get_current_head()?;
    let parent_hashes = if let Some(p) = parent_hash {
        vec![p]
    } else {
        vec![]
    };

    // 4. Create Commit object
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

    // 5. Update Storage
    let meta = crate::core::models::CommitMeta {
        parent_hashes: parent_hashes.clone(),
        tree_hash,
        timestamp,
        author,
        message: message.clone(),
    };

    storage.commit_transaction(
        tree_hash,
        tree_content,
        commit_hash,
        commit_content,
        parent_hash,
        meta,
    )?;

    println!("[main {}] {}", &hex::encode(commit_hash)[..7], message);

    Ok(())
}

pub fn log() -> Result<()> {
    let storage = Storage::new(crate::config::DB_PATH)?;
    let mut current_hash = storage.get_current_head()?;

    if current_hash.is_none() {
        println!("No commits yet");
        return Ok(());
    }

    while let Some(hash) = current_hash {
        if let Some(meta) = storage.get_commit_meta(&hash)? {
            println!("commit {}", hex::encode(hash));
            println!("Author: {}", meta.author);
            
            // Format date
            let datetime = chrono::DateTime::from_timestamp(meta.timestamp as i64, 0)
                .map(|dt| dt.format("%a %b %e %H:%M:%S %Y %z").to_string())
                .unwrap_or_else(|| "Unknown date".to_string());
            println!("Date:   {}\n", datetime);
            
            println!("    {}\n", meta.message);

            // Follow the first parent
            current_hash = meta.parent_hashes.first().copied();
        } else {
            break;
        }
    }

    Ok(())
}

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

#[cfg(test)]
mod tests;
