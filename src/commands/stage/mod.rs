use crate::error::Result;
use crate::core::storage::Storage;
use std::fs::File;

pub fn stage(storage: &Storage, path: String) -> Result<()> {
    if path == "." {
        return crate::core::workspace::auto_stage(storage);
    }

    let matcher = crate::core::ignore::IgnoreMatcher::new();
    let normalized_path = path.replace('\\', "/");
    if matcher.is_ignored(&normalized_path) {
        println!("Path '{}' is ignored by .gik.ignore", path);
        return Ok(());
    }

    // Check if the path exists
    let metadata_res = std::fs::metadata(&path);
    
    match metadata_res {
        Ok(metadata) => {
            if metadata.is_dir() {
                // If it's a directory, recursively stage everything in it
                // Note: we could implement a sub-path scan_and_stage, 
                // but for now let's just use the existing auto_stage if it's the root,
                // or a simpler recursive walker here for a specific folder.
                println!("Staging directory: {}", path);
                stage_directory(storage, &path, &matcher)?;
            } else {
                // It's a file, stage normally
                let hash = {
                    let file = File::open(&path)?;
                    let size = metadata.len();
                    crate::core::objects::hash_blob(&file, size)?
                };

                let file = File::open(&path)?;
                storage.index().stage_file(&path, &hash, metadata.len(), file)?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Path not found on disk. Check if it's in the index to stage deletion.
            if storage.index().get_staged_hash(&path)?.is_some() {
                println!("Staging deletion: {}", path);
                storage.index().unstage_file(&path)?;
            } else {
                return Err(crate::error::GikError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("pathspec '{}' did not match any files", path)
                )));
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

fn stage_directory(storage: &Storage, dir_path: &str, matcher: &crate::core::ignore::IgnoreMatcher) -> Result<()> {
    let mut stack = vec![dir_path.to_string()];
    let root = std::env::current_dir()?;

    while let Some(current_dir) = stack.pop() {
        for entry in std::fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            let relative_path = path.strip_prefix(&root)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let path_str = relative_path.to_str().unwrap_or("");
            if path_str.is_empty() { continue; }
            let normalized = path_str.replace('\\', "/");

            if matcher.is_ignored(&normalized) {
                continue;
            }

            if entry.file_type()?.is_dir() {
                stack.push(path.to_string_lossy().into_owned());
            } else {
                let hash = {
                    let file = File::open(&path)?;
                    let meta = file.metadata()?;
                    crate::core::objects::hash_blob(&file, meta.len())?
                };
                let file = File::open(&path)?;
                let meta = file.metadata()?;
                storage.index().stage_file(&normalized, &hash, meta.len(), file)?;
            }
        }
    }
    
    // Also handle deletions within this directory
    let index_files = storage.index().get_all_staged_files()?;
    let dir_prefix = if dir_path.ends_with('/') || dir_path.ends_with('\\') {
        dir_path.to_string()
    } else {
        format!("{}/", dir_path.replace('\\', "/"))
    };

    for (path, _) in index_files {
        if path.starts_with(&dir_prefix) {
            if !std::path::Path::new(&path).exists() {
                storage.index().unstage_file(&path)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
