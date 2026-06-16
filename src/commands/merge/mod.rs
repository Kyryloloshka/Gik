use crate::core::storage::Storage;
use crate::error::{Result, GikError};
use crate::core::hash::Hash;
use crate::core::workspace::get_status;
use crate::core::graph::find_lowest_common_ancestor;
use crate::core::merge::analyzer::{analyze_trees, MergeAction};
use crate::core::merge::strategy::{MergeStrategy, MergeResult};
use crate::core::merge::text::TextMergeStrategy;
use dialoguer::{Select, theme::ColorfulTheme};
use std::fs;
use std::path::Path;

pub fn merge(storage: &Storage, target: &str) -> Result<()> {
    let current_head = storage.commits().get_current_head()?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No HEAD found"))?;

    let status = get_status(storage)?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Err(GikError::Io(std::io::Error::other("Working directory is not clean. Please commit or restore changes before merging.")));
    }

    let full_hash = if let Some(h) = storage.refs().get_ref(target)? {
        h
    } else if target.len() == 40 {
        Hash::from_hex(target).map_err(|e| GikError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?
    } else {
        let all_objects = storage.objects().list_all_objects()?;
        let matches: Vec<Hash> = all_objects
            .into_iter()
            .filter(|h| h.to_string().starts_with(target))
            .collect();

        if matches.is_empty() {
            return Err(GikError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Commit not found: {}", target))));
        }
        if matches.len() > 1 {
            return Err(GikError::Io(std::io::Error::other(format!("Ambiguous hash: {}", target))));
        }
        matches[0]
    };

    if full_hash == current_head {
        println!("Already up to date.");
        return Ok(());
    }

    let lca = find_lowest_common_ancestor(storage, &current_head, &full_hash)?;
    if lca == Some(full_hash) {
        println!("Already up to date (Fast-forward is not needed as target is an ancestor).");
        return Ok(());
    }
    
    if lca == Some(current_head) {
        println!("Fast-forwarding to {}", full_hash);
        crate::commands::checkout::checkout(storage, target, false)?;
        return Ok(());
    }

    println!("Merging {} into {}", target, current_head);

    let head_meta = storage.commits().get_commit_meta(&current_head)?.unwrap();
    let target_meta = storage.commits().get_commit_meta(&full_hash)?.unwrap();
    let lca_meta = if let Some(lca_hash) = lca {
        storage.commits().get_commit_meta(&lca_hash)?
    } else {
        None
    };

    let base_tree = lca_meta.map(|m| m.tree_hash);
    
    let actions = analyze_trees(storage, base_tree.as_ref(), &head_meta.tree_hash, &target_meta.tree_hash)?;

    let text_strategy = TextMergeStrategy;
    let mut conflicts_exist = false;

    for (path, action) in actions {
        match action {
            MergeAction::KeepOurs => {}
            MergeAction::TakeTheirs(hash) => {
                println!("Updating {}", path);
                write_blob_to_disk(storage, &hash, &path)?;
                crate::commands::stage::stage(storage, path)?;
            }
            MergeAction::DeleteOurs => {
                println!("Removing {}", path);
                if Path::new(&path).exists() {
                    fs::remove_file(&path)?;
                }
                storage.index().unstage_file(&path)?;
            }
            MergeAction::Merge { base, ours, theirs } => {
                let base_content = if let Some(h) = base { get_blob_content(storage, &h)? } else { None };
                let ours_content = if let Some(h) = ours { get_blob_content(storage, &h)?.unwrap_or_default() } else { Vec::new() };
                let theirs_content = if let Some(h) = theirs { get_blob_content(storage, &h)?.unwrap_or_default() } else { Vec::new() };

                let result = text_strategy.merge(base_content.as_deref(), &ours_content, &theirs_content);
                
                match result {
                    MergeResult::Resolved(content) => {
                        println!("Auto-merged {}", path);
                        write_content_to_disk(&content, &path)?;
                        crate::commands::stage::stage(storage, path)?;
                    }
                    MergeResult::Conflict { ours, theirs, .. } => {
                        conflicts_exist = true;
                        println!("Conflict in {}", path);
                        
                        let selections = &[
                            "Keep Ours", 
                            "Take Theirs", 
                            "Accept Both (Ours + Theirs)", 
                            "Accept Both (Theirs + Ours)", 
                            "Abort Merge"
                        ];
                        
                        let selection = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt(format!("Resolve conflict in {}", path))
                            .default(0)
                            .items(&selections[..])
                            .interact()
                            .map_err(|e| GikError::Io(std::io::Error::other(e)))?;

                        match selection {
                            0 => {
                                write_content_to_disk(&ours, &path)?;
                                crate::commands::stage::stage(storage, path)?;
                            }
                            1 => {
                                write_content_to_disk(&theirs, &path)?;
                                crate::commands::stage::stage(storage, path)?;
                            }
                            2 => {
                                let mut combined = ours.clone();
                                combined.extend_from_slice(b"\n");
                                combined.extend_from_slice(&theirs);
                                write_content_to_disk(&combined, &path)?;
                                crate::commands::stage::stage(storage, path)?;
                            }
                            3 => {
                                let mut combined = theirs.clone();
                                combined.extend_from_slice(b"\n");
                                combined.extend_from_slice(&ours);
                                write_content_to_disk(&combined, &path)?;
                                crate::commands::stage::stage(storage, path)?;
                            }
                            _ => {
                                return Err(GikError::Io(std::io::Error::other("Merge aborted by user")));
                            }
                        }
                    }
                }
            }
        }
    }

    if conflicts_exist {
        println!("All conflicts resolved!");
    }
    
    storage.session().set_merge_head(&full_hash)?;
    println!("Merge files staged. Run `gik commit` to finalize the merge.");

    Ok(())
}

fn get_blob_content(storage: &Storage, hash: &Hash) -> Result<Option<Vec<u8>>> {
    if let Some(compressed_data) = storage.objects().get_object(hash)? {
        let (_, _, content) = crate::core::objects::decompress_object(&compressed_data[..])?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

fn write_blob_to_disk(storage: &Storage, hash: &Hash, path: &str) -> Result<()> {
    if let Some(content) = get_blob_content(storage, hash)? {
        write_content_to_disk(&content, path)?;
    }
    Ok(())
}

fn write_content_to_disk(content: &[u8], path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests;
