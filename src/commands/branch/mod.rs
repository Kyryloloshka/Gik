use crate::core::storage::Storage;
use crate::error::Result;
use colored::*;

pub fn branch(storage: &Storage, name: Option<String>, delete: bool) -> Result<()> {
    if let Some(n) = name {
        if delete {
            // Delete branch
            if storage.refs().get_ref(&n)?.is_none() {
                println!("Error: bookmark '{}' not found", n);
                return Ok(());
            }
            storage.refs().delete_ref(&n)?;
            println!("Deleted bookmark '{}'", n);
        } else {
            // Create branch
            let head = storage.commits().get_current_head()?;
            match head {
                Some(hash) => {
                    storage.refs().set_ref(&n, &hash)?;
                    println!("Created bookmark '{}' at {}", n, hash);
                }
                None => {
                    println!("Error: cannot create bookmark, no commits yet");
                }
            }
        }
    } else {
        // List branches
        let refs = storage.refs().list_refs()?;
        let head = storage.commits().get_current_head()?;

        if refs.is_empty() {
            println!("No bookmarks found");
            return Ok(());
        }

        let mut ref_list: Vec<_> = refs.into_iter().collect();
        ref_list.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, hash) in ref_list {
            let is_head = head.map(|h| h == hash).unwrap_or(false);
            if is_head {
                println!("* {} ({})", name.green().bold(), hash.to_string().yellow());
            } else {
                println!("  {} ({})", name, hash.to_string().yellow());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
