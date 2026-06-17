use crate::error::{Result, GikError};
use crate::core::storage::Storage;

pub fn unstage(storage: &Storage, path: String) -> Result<()> {
    // Determine the HEAD tree
    let head_hash = storage.commits().get_current_head()?;
    let head_files = if let Some(h) = head_hash {
        let meta = storage.commits().get_commit_meta(&h)?.ok_or_else(|| GikError::NotFound("Commit meta missing".to_string()))?;
        crate::core::objects::get_commit_tree_files(storage, &meta.tree_hash)?
    } else {
        std::collections::HashMap::new()
    };

    if path == "." {
        // Unstage all currently staged changes
        let staged_files = storage.index().get_all_staged_files()?;
        for (p, _) in staged_files {
            if let Some(h_hash) = head_files.get(&p) {
                // If it was in HEAD, revert the index to the HEAD version
                storage.index().set_staged_hash(&p, h_hash)?;
            } else {
                // If it wasn't in HEAD (it was newly added), remove it from the index
                storage.index().unstage_file(&p)?;
            }
        }
        println!("Unstaged all files");
        return Ok(());
    }

    let normalized_path = path.replace('\\', "/");
    
    // Check if it's in the index
    if storage.index().get_staged_hash(&normalized_path)?.is_some() {
        if let Some(h_hash) = head_files.get(&normalized_path) {
            storage.index().set_staged_hash(&normalized_path, h_hash)?;
        } else {
            storage.index().unstage_file(&normalized_path)?;
        }
        println!("Unstaged {}", path);
    } else {
        return Err(GikError::NotFound(format!("pathspec '{}' did not match any files in the stage", path)));
    }

    Ok(())
}
