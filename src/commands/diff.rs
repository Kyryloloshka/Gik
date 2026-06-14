use crate::core::storage::Storage;
use crate::core::hash::Hash;
use crate::error::Result;
use crate::core::objects::decompress_object;
use crate::core::workspace::get_status;
use crate::core::models::FileState;
use colored::*;
use similar::{ChangeTag, TextDiff};
use std::io::Cursor;

/// Show changes between commits, commit and working tree, etc
pub fn diff(storage: &Storage, staged: bool) -> Result<()> {
    let repo_status = get_status(storage)?;

    if staged {
        // Staged Diff: Index vs HEAD
        if repo_status.staged.is_empty() {
            return Ok(());
        }

        let head_hash = storage.commits().get_current_head()?;
        let head_files = if let Some(h) = head_hash {
            let meta = storage.commits().get_commit_meta(&h)?
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Head commit meta not found"))?;
            crate::core::objects::get_commit_tree_files(storage, &meta.tree_hash)?
        } else {
            std::collections::HashMap::new()
        };

        let index_files: std::collections::HashMap<String, Hash> = storage.index().get_all_staged_files()?
            .into_iter()
            .collect();

        let mut staged_paths: Vec<_> = repo_status.staged.keys().collect();
        staged_paths.sort();

        for path in staged_paths {
            let state = repo_status.staged.get(path).unwrap();
            let old_content = match state {
                FileState::New => String::new(),
                _ => {
                    let hash = head_files.get(path).unwrap();
                    get_blob_content_as_string(storage, hash)?
                }
            };

            let new_content = match state {
                FileState::Deleted => String::new(),
                _ => {
                    let hash = index_files.get(path).unwrap();
                    get_blob_content_as_string(storage, hash)?
                }
            };

            print_diff(path, &old_content, &new_content);
        }
    } else {
        // Unstaged Diff: Disk vs Index
        if repo_status.unstaged.is_empty() {
            return Ok(());
        }

        let index_files: std::collections::HashMap<String, Hash> = storage.index().get_all_staged_files()?
            .into_iter()
            .collect();

        let mut unstaged_paths: Vec<_> = repo_status.unstaged.keys().collect();
        unstaged_paths.sort();

        for path in unstaged_paths {
            let state = repo_status.unstaged.get(path).unwrap();
            let old_content = {
                let hash = index_files.get(path).unwrap();
                get_blob_content_as_string(storage, hash)?
            };

            let new_content = match state {
                FileState::Deleted => String::new(),
                _ => std::fs::read_to_string(path).unwrap_or_default(),
            };

            print_diff(path, &old_content, &new_content);
        }
    }

    Ok(())
}

fn print_diff(path: &str, old: &str, new: &str) {
    println!("{} {}", "diff --gik".bold(), path.bold());
    
    let diff = TextDiff::from_lines(old, new);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-".red(),
            ChangeTag::Insert => "+".green(),
            ChangeTag::Equal => " ".into(),
        };
        print!("{}{}", sign, change);
    }
    println!();
}

/// Fetch a blob from storage and return its content as a String
pub fn get_blob_content_as_string(storage: &Storage, hash: &Hash) -> Result<String> {
    if let Some(compressed_data) = storage.objects().get_object(hash)? {
        let (obj_type, _size, content) = decompress_object(Cursor::new(compressed_data))?;
        if obj_type != "blob" {
            return Err(crate::error::GikError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Object {} is not a blob (type: {})", hash, obj_type)
            )));
        }
        Ok(String::from_utf8_lossy(&content).to_string())
    } else {
        Err(crate::error::GikError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Blob {} not found in storage", hash)
        )))
    }
}

