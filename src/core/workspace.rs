use crate::core::hash::Hash;
use crate::core::ignore::IgnoreMatcher;
use crate::core::models::{FileState, RepoStatus};
use crate::core::objects::get_commit_tree_files;
use crate::core::storage::Storage;
use crate::error::Result;
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
    let disk_files = get_disk_state(storage)?;
    let index_files = storage.index().get_all_staged_files()?;
    for (path, _) in index_files {
        if !disk_files.contains_key(&path) {
            let old_entry = storage.index().unstage_file(&path)?;
            if let Some(_e) = old_entry.clone() {
                storage.log_action(crate::core::models::UndoAction::UpdateIndex {
                    path: path.clone(),
                    old_entry,
                    new_entry: None,
                });
            }
        }
    }

    // 4. Commit transaction batch
    storage.commit_batch(crate::core::models::CommandType::Stage, "gik stage .")?;

    Ok(())
}

/// Only removes files from the index that are now ignored according to ignore rules.
pub fn clean_ignored_from_index(storage: &Storage) -> Result<()> {
    let matcher = IgnoreMatcher::new();
    remove_ignored_from_index(storage, &matcher)
}

/// Computes the repository status by comparing Disk, Index, and HEAD.
pub fn get_status(storage: &Storage) -> Result<RepoStatus> {
    let head_hash = storage.commits().get_current_head()?;
    let head_files = if let Some(h) = head_hash {
        let meta = storage.commits().get_commit_meta(&h)?.ok_or_else(|| {
            crate::error::GikError::NotFound("Head commit meta not found".to_string())
        })?;
        get_commit_tree_files(storage, &meta.tree_hash)?
    } else {
        HashMap::new()
    };

    let index_files: HashMap<String, Hash> = storage
        .index()
        .get_all_staged_files()?
        .into_iter()
        .collect();

    let disk_files = get_disk_state(storage)?;

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
    let meta = storage
        .commits()
        .get_commit_meta(target_commit)?
        .ok_or_else(|| {
            crate::error::GikError::NotFound(format!("Commit {} not found", target_commit))
        })?;

    // 2. Get flat file map from target tree
    let tree_files = get_commit_tree_files(storage, &meta.tree_hash)?;

    // 3. Clean current disk: remove files that are not in the target tree
    let disk_files = get_disk_state(storage)?;
    for (path, _) in disk_files {
        if !tree_files.contains_key(&path) && std::path::Path::new(&path).exists() {
            std::fs::remove_file(&path)?;
        }
    }

    // 4. Restore target files
    for (path, hash) in tree_files {
        let compressed_data = storage
            .objects()
            .get_object(&hash)?
            .ok_or_else(|| crate::error::GikError::NotFound(format!("Blob {} not found", hash)))?;

        let (_obj_type, _size, content) =
            crate::core::objects::decompress_object(&compressed_data[..])?;

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

use ignore::WalkBuilder;

fn get_disk_state(storage: &Storage) -> Result<HashMap<String, Hash>> {
    let mut disk_files = HashMap::new();
    let root = std::env::current_dir()?;

    let index_entries: HashMap<String, crate::core::models::IndexEntry> = storage
        .index()
        .get_all_staged_entries()?
        .into_iter()
        .collect();

    let builder = build_walker(&root);

    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue, // Skip files we don't have access to
        };
        if entry.file_type().map_or(true, |ft| ft.is_dir()) {
            continue;
        }

        let path = entry.path();
        let relative_path = path.strip_prefix(&root).unwrap_or(path);
        let path_str = relative_path.to_str().unwrap_or("");
        if path_str.is_empty() {
            continue;
        }

        let normalized_path = if path_str.contains('\\') {
            std::borrow::Cow::Owned(path_str.replace('\\', "/"))
        } else {
            std::borrow::Cow::Borrowed(path_str)
        };
        if let Ok(metadata) = entry.metadata() {
            let size = metadata.len();
            let mtime = metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs();

            if let Some(cached) = index_entries.get(normalized_path.as_ref()) {
                if cached.size == size && cached.mtime == mtime {
                    disk_files.insert(normalized_path.into_owned(), cached.hash.clone());
                    continue;
                }
            }

            if let Ok(file) = std::fs::File::open(path) {
                if let Ok(hash) = crate::core::objects::hash_blob(file, size) {
                    disk_files.insert(normalized_path.into_owned(), hash);
                }
            }
        }
    }

    Ok(disk_files)
}

fn scan_and_stage(storage: &Storage, _matcher: &IgnoreMatcher) -> Result<()> {
    let root = std::env::current_dir()?;

    let builder = build_walker(&root);

    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().map_or(true, |ft| ft.is_dir()) {
            continue;
        }

        let path = entry.path();
        let relative_path = path.strip_prefix(&root).unwrap_or(path);
        let path_str = relative_path.to_str().unwrap_or("");
        if path_str.is_empty() {
            continue;
        }

        let normalized_path = if path_str.contains('\\') {
            std::borrow::Cow::Owned(path_str.replace('\\', "/"))
        } else {
            std::borrow::Cow::Borrowed(path_str)
        };
        let file = std::fs::File::open(&path)?;
        let meta = file.metadata()?;
        let hash = crate::core::objects::hash_blob(&file, meta.len())?;

        let mtime = meta
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        let file = std::fs::File::open(&path)?;
        let old_entry =
            storage
                .index()
                .stage_file(&normalized_path, &hash, meta.len(), mtime, file)?;
        let new_entry = storage.index().get_staged_entry(&normalized_path)?;
        storage.log_action(crate::core::models::UndoAction::UpdateIndex {
            path: normalized_path.into_owned(),
            old_entry,
            new_entry,
        });
    }

    Ok(())
}

fn remove_ignored_from_index(storage: &Storage, matcher: &IgnoreMatcher) -> Result<()> {
    let currently_staged = storage.index().get_all_staged_files()?;
    for (path, _) in currently_staged {
        if matcher.is_ignored(&path) {
            println!("Removing ignored file from index: {}", path);
            let old_entry = storage.index().unstage_file(&path)?;
            if let Some(_e) = old_entry.clone() {
                storage.log_action(crate::core::models::UndoAction::UpdateIndex {
                    path: path.clone(),
                    old_entry,
                    new_entry: None,
                });
            }
        }
    }
    Ok(())
}

fn build_walker(root: &std::path::Path) -> WalkBuilder {
    let mut builder = WalkBuilder::new(root);
    builder.add_custom_ignore_filename(crate::config::IGNORE_FILE_NAME);
    builder.hidden(false); // Do not skip hidden files like .env

    builder.filter_entry(move |entry| {
        let name = entry.file_name().to_string_lossy();
        name != crate::config::GIK_DIR_NAME
            && name != crate::config::GIT_DIR_NAME
            && !name.contains("gik_test")
    });

    builder
}
