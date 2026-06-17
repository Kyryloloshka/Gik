use crate::error::{Result, GikError};
use crate::core::storage::Storage;
use crate::core::hash::Hash;

pub fn show(storage: &Storage, target: &str) -> Result<()> {
    let parts: Vec<&str> = target.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(GikError::Validation("Expected format <hash-or-ref>:<path>".to_string()));
    }
    
    let ref_name = parts[0];
    let path = parts[1];
    let normalized_path = path.replace("\\", "/");
    
    if ref_name == "STAGE" {
        let files = storage.index().get_all_staged_files()?;
        let mut found = false;
        let mut target_hash = Hash([0; 20]);
        for (fpath, fhash) in files {
            if fpath == normalized_path {
                target_hash = fhash;
                found = true;
                break;
            }
        }
        if !found {
            return Err(GikError::NotFound(format!("Path '{}' not found in stage", path)));
        }
        let text = storage.objects().get_blob_text(&target_hash)?;
        print!("{}", text);
        return Ok(());
    }
    
    let commit_hash = if ref_name == "HEAD" {
        storage.commits().get_current_head()?.ok_or_else(|| GikError::NotFound("HEAD is detached or missing".to_string()))?
    } else if let Ok(Some(h)) = storage.refs().get_ref(ref_name) {
        h
    } else if let Ok(h) = Hash::from_hex(ref_name) {
        h
    } else {
        return Err(GikError::NotFound(format!("Ref or hash not found: {}", ref_name)));
    };
    
    let commit = storage.commits().get_commit_meta(&commit_hash)?
        .ok_or_else(|| GikError::NotFound(format!("Commit not found: {}", commit_hash)))?;
        
    let mut current_hash = commit.tree_hash;
    let normalized_path = path.replace("\\", "/");
    let path_parts: Vec<&str> = normalized_path.split('/').filter(|p| !p.is_empty()).collect();
    
    for part in path_parts.iter() {
        if let Some(compressed) = storage.objects().get_object(&current_hash)? {
            let (obj_type, _, payload) = crate::core::objects::decompress_object(&compressed[..])?;
            if obj_type != "tree" {
                return Err(GikError::Validation(format!("Expected tree, found {}", obj_type)));
            }
            
            let entries = crate::core::objects::tree::parse_tree(&payload)?;
            let mut found = false;
            for (_mode, name, hash) in entries {
                if &name == *part {
                    current_hash = hash;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(GikError::NotFound(format!("Path '{}' not found", part)));
            }
        } else {
            return Err(GikError::NotFound(format!("Object {} not found", current_hash)));
        }
    }
    
    let text = storage.objects().get_blob_text(&current_hash)?;
    print!("{}", text);
    
    Ok(())
}
