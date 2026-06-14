use sha1::{Sha1, Digest};
use std::io::{self, Write};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use crate::core::hash::Hash;
use crate::core::storage::Storage;
use std::collections::HashMap;

pub const REGULAR_FILE_MODE: u32 = 0o100644;
pub const DIRECTORY_MODE: u32 = 0o040000;

/// Calculates the SHA1 hash of a tree in Git-canonical format
pub fn hash_tree(entries: &[(u32, String, Hash)]) -> io::Result<Hash> {
    let mut hasher = Sha1::new();
    let content = build_tree_content(entries);

    let header = format!("tree {}\0", content.len());
    hasher.update(header.as_bytes());
    hasher.update(&content);

    let result = hasher.finalize();
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&result);
    Ok(Hash(hash))
}

/// Compresses a tree object using Zlib
pub fn compress_tree<W: Write>(entries: &[(u32, String, Hash)], writer: W) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(writer, Compression::default());
    let content = build_tree_content(entries);

    let header = format!("tree {}\0", content.len());
    encoder.write_all(header.as_bytes())?;
    encoder.write_all(&content)?;

    encoder.finish()?;
    Ok(())
}

fn build_tree_content(entries: &[(u32, String, Hash)]) -> Vec<u8> {
    let mut content = Vec::new();
    for (mode, name, hash) in entries {
        content.extend_from_slice(format!("{:o} {}\0", mode, name).as_bytes());
        content.extend_from_slice(&hash.0);
    }
    content
}

/// Builds a hierarchy of tree objects from staged files and stores them in the database.
/// Returns the hash and content of the root tree.
pub fn build_and_store_tree(
    storage: &Storage,
    staged_files: Vec<(String, Hash)>,
) -> crate::error::Result<(Hash, Vec<u8>)> {
    // 1. Group files by their top-level directory/file
    let mut tree_map: HashMap<String, Vec<(String, Hash)>> = HashMap::new();
    let mut root_entries: Vec<(u32, String, Hash)> = Vec::new();

    for (path, hash) in staged_files {
        // Normalize path separators
        let normalized = path.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').collect();

        if parts.len() == 1 {
            // Direct file in current level
            root_entries.push((REGULAR_FILE_MODE, parts[0].to_string(), hash));
        } else {
            // File in a subdirectory
            let dir_name = parts[0].to_string();
            let remaining_path = parts[1..].join("/");
            tree_map.entry(dir_name).or_default().push((remaining_path, hash));
        }
    }

    // 2. Recursively build sub-trees
    for (dir_name, sub_files) in tree_map {
        let (sub_tree_hash, sub_tree_content) = build_and_store_tree(storage, sub_files)?;
        
        // Store the sub-tree object in the database
        // We need a method in storage for this, or use raw access via facade if available
        // For now, we'll assume we can use commit_transaction-like logic or add a new method.
        // Let's use a simpler approach: return all created trees up to the caller.
        // Actually, the easiest is to just write it to the DB here.
        let write_txn = storage.repo.db.begin_write()?;
        {
            let mut table = write_txn.open_table(crate::core::storage::repository::OBJECTS)?;
            table.insert(&sub_tree_hash.0, sub_tree_content)?;
        }
        write_txn.commit()?;

        root_entries.push((DIRECTORY_MODE, dir_name, sub_tree_hash));
    }

    // 3. Finalize current tree
    root_entries.sort_by(|a, b| a.1.cmp(&b.1));
    let tree_hash = hash_tree(&root_entries)?;
    let mut tree_content = Vec::new();
    compress_tree(&root_entries, &mut tree_content)?;

    Ok((tree_hash, tree_content))
}
