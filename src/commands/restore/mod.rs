use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use std::fs;
use std::path::Path;

pub fn restore(storage: &Storage, path: &str) -> Result<()> {
    let head = storage.commits().get_current_head()?;
    let head_hash = match head {
        Some(h) => h,
        None => return Err(crate::error::GikError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No commits to restore from"
        ))),
    };

    let meta = storage.commits().get_commit_meta(&head_hash)?.unwrap();
    let head_files = crate::core::objects::get_commit_tree_files(storage, &meta.tree_hash)?;

    if path == "." {
        println!("Restoring all files from HEAD...");
        crate::core::workspace::restore_workspace(storage, &head_hash)?;
        storage.index().set_index_state(&head_files)?;
    } else {
        // Normalize path
        let normalized = path.replace('\\', "/");
        
        if let Some(blob_hash) = head_files.get(&normalized) {
            println!("Restoring {}...", normalized);
            restore_file(storage, &normalized, blob_hash)?;
            
            // Update index for this file
            // We don't have a partial index update helper yet, let's just use stage logic
            // but we need the raw bytes.
            let bytes = get_blob_bytes(storage, blob_hash)?;
            storage.index().stage_file(&normalized, blob_hash, bytes.len() as u64, &bytes[..])?;
        } else {
            // File not in HEAD. If it's in index, unstage it. If it's on disk, maybe delete?
            // Standard Git: git restore <file> error if not in index/HEAD.
            // Let's check if it exists in current index.
            if storage.index().get_staged_hash(&normalized)?.is_some() {
                println!("File not in HEAD, unstaging {}...", normalized);
                storage.index().unstage_file(&normalized)?;
            } else {
                return Err(crate::error::GikError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("pathspec '{}' did not match any files in HEAD", normalized)
                )));
            }
        }
    }

    Ok(())
}

fn restore_file(storage: &Storage, path: &str, blob_hash: &Hash) -> Result<()> {
    let bytes = get_blob_bytes(storage, blob_hash)?;
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn get_blob_bytes(storage: &Storage, hash: &Hash) -> Result<Vec<u8>> {
    if let Some(compressed) = storage.objects().get_object(hash)? {
        let (obj_type, _size, content) = crate::core::objects::decompress_object(&compressed[..])?;
        if obj_type != "blob" {
            return Err(crate::error::GikError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Object {} is not a blob", hash)
            )));
        }
        Ok(content)
    } else {
        Err(crate::error::GikError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Blob {} not found", hash)
        )))
    }
}

#[cfg(test)]
mod tests;
