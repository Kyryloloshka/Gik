use crate::error::{Result, GikError};
use crate::core::storage::Storage;
use crate::core::hash::Hash;

pub fn show(storage: &Storage, target: &str) -> Result<()> {
    let parts: Vec<&str> = target.splitn(2, ':').collect();
    if parts.len() == 1 {
        return show_commit_files(storage, target);
    }
    if parts.len() != 2 {
        return Err(GikError::Validation("Expected format <hash-or-ref>:<path> or <hash-or-ref>".to_string()));
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
    
    let (real_ref, parent_count) = if ref_name.ends_with('^') {
        let count = ref_name.chars().filter(|c| *c == '^').count();
        (ref_name.trim_end_matches('^'), count)
    } else {
        (ref_name, 0)
    };

    let mut commit_hash = if real_ref == "HEAD" {
        storage.commits().get_current_head()?.ok_or_else(|| GikError::NotFound("HEAD is detached or missing".to_string()))?
    } else if let Ok(Some(h)) = storage.refs().get_ref(real_ref) {
        h
    } else if let Ok(h) = Hash::from_hex(real_ref) {
        h
    } else {
        return Err(GikError::NotFound(format!("Ref or hash not found: {}", real_ref)));
    };
    
    for _ in 0..parent_count {
        let meta = storage.commits().get_commit_meta(&commit_hash)?
            .ok_or_else(|| GikError::NotFound(format!("Commit not found: {}", commit_hash)))?;
        if let Some(parent) = meta.parent_hashes.first() {
            commit_hash = *parent;
        } else {
            return Err(GikError::NotFound(format!("Commit {} has no parent", commit_hash)));
        }
    }
    
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

fn show_commit_files(storage: &Storage, target: &str) -> Result<()> {
    let commit_hash = if target == "HEAD" {
        storage.commits().get_current_head()?.ok_or_else(|| GikError::NotFound("HEAD is detached or missing".to_string()))?
    } else if let Ok(Some(h)) = storage.refs().get_ref(target) {
        h
    } else if let Ok(h) = Hash::from_hex(target) {
        h
    } else {
        return Err(GikError::NotFound(format!("Ref or hash not found: {}", target)));
    };
    
    let commit = storage.commits().get_commit_meta(&commit_hash)?
        .ok_or_else(|| GikError::NotFound(format!("Commit not found: {}", commit_hash)))?;
        
    let current_files = crate::core::objects::get_commit_tree_files(storage, &commit.tree_hash)?;
    
    let parent_files = if let Some(parent_hash) = commit.parent_hashes.first() {
        if let Some(parent_meta) = storage.commits().get_commit_meta(parent_hash)? {
            crate::core::objects::get_commit_tree_files(storage, &parent_meta.tree_hash)?
        } else {
            std::collections::HashMap::new()
        }
    } else {
        std::collections::HashMap::new()
    };
    
    let mut all_paths: Vec<String> = current_files.keys().chain(parent_files.keys()).cloned().collect();
    all_paths.sort();
    all_paths.dedup();
    
    for path in all_paths {
        match (parent_files.get(&path), current_files.get(&path)) {
            (None, Some(_)) => println!("A\t{}", path),
            (Some(_), None) => println!("D\t{}", path),
            (Some(old_hash), Some(new_hash)) if old_hash != new_hash => println!("M\t{}", path),
            _ => {}
        }
    }
    
    Ok(())
}
