use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::ignore::IgnoreMatcher;
use crate::core::hash::Hash;
use crate::core::models::{FileState, RepoStatus};
use crate::core::objects::get_commit_tree_files;
use std::collections::{HashMap, HashSet};

/// Recursively scans the workspace and stages all non-ignored files.
/// Also removes files from the index that are now ignored or deleted.
pub fn auto_stage(storage: &Storage) -> Result<()> {
    let matcher = IgnoreMatcher::new();

    // 1. Recursive auto-staging (file system traversal)
    scan_and_stage(storage, &matcher)?;

    // 2. Auto-remove files from index if they are now ignored
    remove_ignored_from_index(storage, &matcher)?;

    // 3. Remove files from index that have been deleted from disk
    let disk_files = get_disk_state()?;
    let index_files = storage.index().get_all_staged_files()?;
    for (path, _) in index_files {
        if !disk_files.contains_key(&path) {
            storage.index().unstage_file(&path)?;
        }
    }

    Ok(())
}

/// Only removes files from the index that are now ignored according to .gik.ignore.
pub fn clean_ignored_from_index(storage: &Storage) -> Result<()> {
    let matcher = IgnoreMatcher::new();
    remove_ignored_from_index(storage, &matcher)
}

/// Computes the repository status by comparing Disk, Index, and HEAD.
pub fn get_status(storage: &Storage) -> Result<RepoStatus> {
    let head_hash = storage.commits().get_current_head()?;
    let head_files = if let Some(h) = head_hash {
        let meta = storage
            .commits()
            .get_commit_meta(&h)?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Head commit meta not found"))?;
        get_commit_tree_files(storage, &meta.tree_hash)?
    } else {
        HashMap::new()
    };

    let index_files: HashMap<String, Hash> = storage
        .index()
        .get_all_staged_files()?
        .into_iter()
        .collect();

    let disk_files = get_disk_state()?;

    let mut status = RepoStatus::default();

    // 1. Staged Changes (Index vs HEAD)
    let mut all_staged_keys: HashSet<_> = index_files.keys().cloned().collect();
    all_staged_keys.extend(head_files.keys().cloned());

    for path in all_staged_keys {
        match (head_files.get(&path), index_files.get(&path)) {
            (None, Some(_)) => {
                status.staged.insert(path, FileState::New);
            }
            (Some(h), Some(i)) if h != i => {
                status.staged.insert(path, FileState::Modified);
            }
            (Some(_), None) => {
                status.staged.insert(path, FileState::Deleted);
            }
            _ => {}
        }
    }

    // 2. Unstaged Changes (Disk vs Index)
    for (path, index_hash) in &index_files {
        match disk_files.get(path) {
            Some(disk_hash) => {
                if disk_hash != index_hash {
                    status.unstaged.insert(path.clone(), FileState::Modified);
                }
            }
            None => {
                status.unstaged.insert(path.clone(), FileState::Deleted);
            }
        }
    }

    // 3. Untracked Files (Disk not in Index and not in HEAD)
    for path in disk_files.keys() {
        if !index_files.contains_key(path) && !head_files.contains_key(path) {
            status.untracked.push(path.clone());
        }
    }
    status.untracked.sort();

    Ok(status)
}

/// Restores the workspace files from a specific commit.
pub fn restore_workspace(storage: &Storage, target_commit: &Hash) -> Result<()> {
    // 1. Get commit meta
    let meta = storage.commits().get_commit_meta(target_commit)?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Commit {} not found", target_commit)))?;

    // 2. Get flat file map from target tree
    let tree_files = get_commit_tree_files(storage, &meta.tree_hash)?;

    // 3. Clean current disk: remove files that are not in the target tree
    let disk_files = get_disk_state()?;
    for (path, _) in disk_files {
        if !tree_files.contains_key(&path) {
            if std::path::Path::new(&path).exists() {
                std::fs::remove_file(&path)?;
            }
        }
    }

    // 4. Restore target files
    for (path, hash) in tree_files {
        let compressed_data = storage.objects().get_object(&hash)?
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Blob {} not found", hash)))?;
        
        let (_obj_type, _size, content) = crate::core::objects::decompress_object(&compressed_data[..])?;
        
        // Ensure parent directories exist
        if let Some(parent) = std::path::Path::new(&path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        std::fs::write(&path, content)?;
    }

    Ok(())
}

fn get_disk_state() -> Result<HashMap<String, Hash>> {
    let mut disk_files = HashMap::new();
    let matcher = IgnoreMatcher::new();
    let root = std::env::current_dir()?;
    let mut stack = vec![root.clone()];

    while let Some(current_dir) = stack.pop() {
        for entry in std::fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Get path relative to the repository root
            let relative_path = path.strip_prefix(&root)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let path_str = relative_path.to_str().unwrap_or("");
            
            if path_str.is_empty() { continue; }

            // Normalize separators to forward slashes for the ignore matcher
            let normalized_path = path_str.replace('\\', "/");

            if matcher.is_ignored(&normalized_path) {
                continue;
            }

            if entry.file_type()?.is_dir() {
                stack.push(path);
            } else {
                let metadata = entry.metadata()?;
                let file = std::fs::File::open(&path)?;
                let hash = crate::core::objects::hash_blob(file, metadata.len())?;
                disk_files.insert(normalized_path, hash);
            }
        }
    }

    Ok(disk_files)
}

fn scan_and_stage(storage: &Storage, matcher: &IgnoreMatcher) -> Result<()> {
    let root = std::env::current_dir()?;
    let mut stack = vec![root.clone()];
    
    while let Some(current_dir) = stack.pop() {
        for entry in std::fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            let relative_path = path.strip_prefix(&root)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let path_str = relative_path.to_str().unwrap_or("");
            
            if path_str.is_empty() { continue; }
            
            // Normalize separators to forward slashes
            let normalized_path = path_str.replace('\\', "/");
            
            if matcher.is_ignored(&normalized_path) {
                continue;
            }

            if entry.file_type()?.is_dir() {
                stack.push(path);
            } else {
                // It's a file, stage it
                crate::commands::stage::stage(storage, normalized_path)?;
            }
        }
    }
    Ok(())
}


fn remove_ignored_from_index(storage: &Storage, matcher: &IgnoreMatcher) -> Result<()> {
    let currently_staged = storage.index().get_all_staged_files()?;
    for (path, _) in currently_staged {
        if matcher.is_ignored(&path) {
            println!("Removing ignored file from index: {}", path);
            storage.index().unstage_file(&path)?;
        }
    }
    Ok(())
}
