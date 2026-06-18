use crate::core::storage::Storage;
use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::workspace::get_status;
use crate::core::graph::find_lowest_common_ancestor;
use crate::core::merge::analyzer::{analyze_trees, MergeAction};
use crate::core::merge::strategy::{MergeStrategy, MergeResult};
use crate::core::merge::text::TextMergeStrategy;
use dialoguer::{Select, theme::ColorfulTheme};
use std::fs;
use std::path::Path;
use similar::TextDiff;
use colored::Colorize;

pub fn merge(storage: &Storage, target: &str) -> Result<()> {
    let current_head = storage.commits().get_current_head()?
        .ok_or_else(|| crate::error::GikError::NotFound("No HEAD found".to_string()))?;

    let status = get_status(storage)?;
    if !status.staged.is_empty() || !status.unstaged.is_empty() || !status.untracked.is_empty() {
        return Err(crate::error::GikError::DirtyWorkspace("Working directory is not clean. Please commit or restore changes before merging.".to_string()));
    }

    let (full_hash, _) = crate::core::utils::resolve_hash(storage, target)?;

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
        
        // Update current branch reference if we are on a branch
        let current_bookmark = storage.session().get_current_bookmark()?;
        if let Some(ref bm) = current_bookmark {
            storage.refs().set_ref(bm, &full_hash)?;
        }
        
        // Restore workspace and index
        let meta = storage.commits().get_commit_meta(&full_hash)?.unwrap();
        crate::core::workspace::restore_workspace(storage, &full_hash)?;
        let tree_files = crate::core::objects::get_commit_tree_files(storage, &meta.tree_hash)?;
        storage.index().set_index_state(&tree_files)?;
        storage.commits().set_head(&full_hash)?;
        
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
    let mut resolve_all_manually = false;
    
    // Set MERGE_HEAD early so if they abort/Ctrl+C, the state is remembered.
    storage.session().set_merge_head(&full_hash)?;

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
                        println!("\nConflict in {}", path);
                        
                        // Print diff
                        let ours_str = String::from_utf8_lossy(&ours);
                        let theirs_str = String::from_utf8_lossy(&theirs);
                        let diff = TextDiff::from_lines(&ours_str, &theirs_str);
                        for change in diff.iter_all_changes() {
                            let text = change.to_string();
                            match change.tag() {
                                similar::ChangeTag::Delete => print!("-{}", text.red()),
                                similar::ChangeTag::Insert => print!("+{}", text.green()),
                                similar::ChangeTag::Equal => print!(" {}", text),
                            }
                        }
                        println!("");

                        if resolve_all_manually {
                            let mut combined = Vec::new();
                            combined.extend_from_slice(b"<<<<<<< HEAD\n");
                            combined.extend_from_slice(&ours);
                            if !ours.ends_with(b"\n") && !ours.is_empty() { combined.extend_from_slice(b"\n"); }
                            combined.extend_from_slice(b"=======\n");
                            combined.extend_from_slice(&theirs);
                            if !theirs.ends_with(b"\n") && !theirs.is_empty() { combined.extend_from_slice(b"\n"); }
                            combined.extend_from_slice(format!(">>>>>>> {}\n", target).as_bytes());
                            
                            write_content_to_disk(&combined, &path)?;
                            println!("Conflict markers written to {}.", path);
                            continue;
                        }

                        let selections = &[
                            "Keep Ours", 
                            "Take Theirs", 
                            "Accept Both (Ours + Theirs)", 
                            "Accept Both (Theirs + Ours)", 
                            "Resolve manually in editor",
                            "Resolve ALL conflicts manually in editor",
                            "Abort Merge"
                        ];
                        
                        let selection = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt(format!("Resolve conflict in {}", path))
                            .default(0)
                            .items(&selections[..])
                            .interact()
                            .map_err(|e| crate::error::GikError::Merge(e.to_string()))?;

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
                            4 => {
                                let mut combined = Vec::new();
                                combined.extend_from_slice(b"<<<<<<< HEAD\n");
                                combined.extend_from_slice(&ours);
                                if !ours.ends_with(b"\n") && !ours.is_empty() { combined.extend_from_slice(b"\n"); }
                                combined.extend_from_slice(b"=======\n");
                                combined.extend_from_slice(&theirs);
                                if !theirs.ends_with(b"\n") && !theirs.is_empty() { combined.extend_from_slice(b"\n"); }
                                combined.extend_from_slice(format!(">>>>>>> {}\n", target).as_bytes());
                                
                                write_content_to_disk(&combined, &path)?;
                                
                                println!("Conflict markers written to {}.", path);
                                println!("Please open the file in your editor, resolve the conflict, save it, and press Enter to continue...");
                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input).unwrap();
                                
                                crate::commands::stage::stage(storage, path)?;
                            }
                            5 => {
                                resolve_all_manually = true;
                                let mut combined = Vec::new();
                                combined.extend_from_slice(b"<<<<<<< HEAD\n");
                                combined.extend_from_slice(&ours);
                                if !ours.ends_with(b"\n") && !ours.is_empty() { combined.extend_from_slice(b"\n"); }
                                combined.extend_from_slice(b"=======\n");
                                combined.extend_from_slice(&theirs);
                                if !theirs.ends_with(b"\n") && !theirs.is_empty() { combined.extend_from_slice(b"\n"); }
                                combined.extend_from_slice(format!(">>>>>>> {}\n", target).as_bytes());
                                
                                write_content_to_disk(&combined, &path)?;
                                println!("Conflict markers written to {}.", path);
                            }
                            _ => {
                                storage.session().clear_merge_head()?;
                                return Err(crate::error::GikError::Aborted("Merge aborted by user".to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    if resolve_all_manually {
        println!("All remaining conflicts have markers written.");
        println!("Please open your editor, resolve them, use `gik stage <file>`, and then run `gik merge --continue`.");
        return Ok(());
    }

    if conflicts_exist {
        println!("All conflicts resolved!");
    }
    
    println!("Merge files staged.");

    crate::commands::commit::commit(
        storage,
        format!("Merge {} into {}", target, current_head),
        false,
        None,
    )?;

    Ok(())
}

pub fn continue_merge(storage: &Storage) -> Result<()> {
    if let Some(merge_head) = storage.session().get_merge_head()? {
        let current_head = storage.commits().get_current_head()?
            .ok_or_else(|| crate::error::GikError::NotFound("No HEAD found".to_string()))?;
        crate::commands::commit::commit(
            storage,
            format!("Merge {} into {}", merge_head, current_head),
            false,
            None,
        )?;
        Ok(())
    } else {
        Err(crate::error::GikError::Validation("No merge in progress".to_string()))
    }
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
