use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::ignore::IgnoreMatcher;

/// Recursively scans the workspace and stages all non-ignored files.
/// Also removes files from the index that are now ignored.
pub fn auto_stage(storage: &Storage) -> Result<()> {
    let matcher = IgnoreMatcher::new();

    // 1. Recursive auto-staging (file system traversal)
    scan_and_stage(storage, &matcher)?;

    // 2. Auto-remove files from index if they are now ignored
    remove_ignored_from_index(storage, &matcher)?;

    Ok(())
}

/// Only removes files from the index that are now ignored according to .gik.ignore.
pub fn clean_ignored_from_index(storage: &Storage) -> Result<()> {
    let matcher = IgnoreMatcher::new();
    remove_ignored_from_index(storage, &matcher)
}

fn scan_and_stage(storage: &Storage, matcher: &IgnoreMatcher) -> Result<()> {
    let mut stack = vec![".".to_string()];
    while let Some(current_dir) = stack.pop() {
        for entry in std::fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Get path relative to current directory, but strip "./" for matching
            let mut path_str = path.to_str().unwrap_or("").to_string();
            if path_str.starts_with("./") || path_str.starts_with(".\\") {
                path_str = path_str[2..].to_string();
            }
            
            // Skip based on ignore matcher
            if matcher.is_ignored(&path_str) {
                continue;
            }

            if entry.file_type()?.is_dir() {
                stack.push(path_str);
            } else {
                // It's a file, stage it
                crate::commands::stage::stage(storage, path_str)?;
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
