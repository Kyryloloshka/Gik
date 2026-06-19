use crate::core::hash::Hash;
use crate::core::storage::Storage;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::io::{self, Write};

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

pub(crate) fn build_tree_content(entries: &[(u32, String, Hash)]) -> Vec<u8> {
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
    let mut trees_to_store = Vec::new();
    let (root_hash, root_content) = build_tree_recursive(staged_files, &mut trees_to_store)?;

    // Store all trees
    for (hash, content) in trees_to_store {
        storage.objects().write_object(&hash, &content)?;
    }
    // Also insert the root tree
    storage.objects().write_object(&root_hash, &root_content)?;

    Ok((root_hash, root_content))
}

fn build_tree_recursive(
    staged_files: Vec<(String, Hash)>,
    trees_to_store: &mut Vec<(Hash, Vec<u8>)>,
) -> crate::error::Result<(Hash, Vec<u8>)> {
    let mut tree_map: HashMap<String, Vec<(String, Hash)>> = HashMap::new();
    let mut entries: Vec<(u32, String, Hash)> = Vec::new();

    for (path, hash) in staged_files {
        let normalized = path.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').collect();

        if parts.len() == 1 || (parts.len() == 2 && parts[1].is_empty()) {
            // Direct file in current level (or trailing slash case)
            let name = parts[0];
            if !name.is_empty() {
                entries.push((REGULAR_FILE_MODE, name.to_string(), hash));
            }
        } else {
            // File in a subdirectory
            let dir_name = parts[0].to_string();
            if dir_name.is_empty() {
                // Handle leading slash/relative dot: skip it and treat as root
                let remaining = parts[1..].join("/");
                if !remaining.is_empty() {
                    // Re-insert into the flat list for next iteration
                    // Actually, we just need to avoid dir_name being empty.
                }
                continue;
            }
            let remaining_path = parts[1..].join("/");
            tree_map
                .entry(dir_name)
                .or_default()
                .push((remaining_path, hash));
        }
    }

    // 2. Recursively build sub-trees
    for (dir_name, sub_files) in tree_map {
        let (sub_tree_hash, sub_tree_content) = build_tree_recursive(sub_files, trees_to_store)?;
        trees_to_store.push((sub_tree_hash, sub_tree_content));
        entries.push((DIRECTORY_MODE, dir_name, sub_tree_hash));
    }

    entries.sort_by(|a, b| a.1.cmp(&b.1));
    let tree_hash = hash_tree(&entries)?;
    let mut tree_content = Vec::new();
    compress_tree(&entries, &mut tree_content)?;

    Ok((tree_hash, tree_content))
}

/// Parses a Git-canonical tree object content.
pub fn parse_tree(content: &[u8]) -> crate::error::Result<Vec<(u32, String, Hash)>> {
    let mut entries = Vec::new();
    let mut i = 0;
    while i < content.len() {
        // Find space after mode
        let space_pos = content[i..]
            .iter()
            .position(|&b| b == b' ')
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid tree format: missing space",
                )
            })?
            + i;
        let mode_str = std::str::from_utf8(&content[i..space_pos]).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid mode UTF-8")
        })?;
        let mode = u32::from_str_radix(mode_str, 8).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid mode octal")
        })?;

        // Find null after name
        let null_pos = content[space_pos + 1..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid tree format: missing null",
                )
            })?
            + space_pos
            + 1;
        let name = std::str::from_utf8(&content[space_pos + 1..null_pos])
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid name UTF-8")
            })?
            .to_string();

        // Read 20 bytes hash
        if null_pos + 1 + 20 > content.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid tree format: truncated hash",
            )
            .into());
        }
        let mut hash_bytes = [0u8; 20];
        hash_bytes.copy_from_slice(&content[null_pos + 1..null_pos + 1 + 20]);
        let hash = Hash(hash_bytes);

        entries.push((mode, name, hash));
        i = null_pos + 1 + 20;
    }
    Ok(entries)
}

pub fn get_commit_tree_files(
    storage: &Storage,
    tree_hash: &Hash,
) -> crate::error::Result<HashMap<String, Hash>> {
    if let Ok(Some(map)) = storage.commits().get_tree_cache(tree_hash) {
        return Ok(map);
    }

    let mut files = HashMap::new();
    recursive_tree_walk(storage, tree_hash, "", &mut files)?;
    
    let _ = storage.commits().set_tree_cache(tree_hash, &files);
    
    Ok(files)
}

fn recursive_tree_walk(
    storage: &Storage,
    tree_hash: &Hash,
    prefix: &str,
    files: &mut HashMap<String, Hash>,
) -> crate::error::Result<()> {
    let obj_data = storage.objects().get_object(tree_hash)?.ok_or_else(|| {
        crate::error::GikError::NotFound(format!("Tree object {} not found", tree_hash))
    })?;

    let (obj_type, _, content) = crate::core::objects::decompress_object(&obj_data[..])?;
    if obj_type != "tree" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Object {} is not a tree", tree_hash),
        )
        .into());
    }

    let entries = parse_tree(&content)?;
    for (mode, name, hash) in entries {
        let full_path = if prefix.is_empty() {
            name
        } else {
            format!("{}/{}", prefix, name)
        };

        if mode == DIRECTORY_MODE {
            recursive_tree_walk(storage, &hash, &full_path, files)?;
        } else {
            files.insert(full_path, hash);
        }
    }
    Ok(())
}
