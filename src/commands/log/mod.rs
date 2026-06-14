use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use std::collections::{HashSet, VecDeque, HashMap};
use colored::*;

pub fn log(storage: &Storage, all: bool) -> Result<()> {
    let head = storage.commits().get_current_head()?;
    let refs = storage.refs().list_refs()?;
    
    // Map of commit hash -> list of branch/bookmark names
    let mut labels: HashMap<Hash, Vec<String>> = HashMap::new();
    for (name, hash) in &refs {
        labels.entry(*hash).or_default().push(name.clone());
    }
    if let Some(h) = head {
        labels.entry(h).or_default().push("HEAD".to_string());
    }

    if !all {
        // Standard linear log from HEAD
        if head.is_none() {
            println!("No commits yet");
            return Ok(());
        }

        let mut current_hash = head;
        while let Some(hash) = current_hash {
            print_commit(storage, &hash, &labels)?;
            
            if let Some(meta) = storage.commits().get_commit_meta(&hash)? {
                current_hash = meta.parent_hashes.first().copied();
            } else {
                break;
            }
        }
    } else {
        // Log --all: traverse from all refs and HEAD
        let mut start_hashes = HashSet::new();
        if let Some(h) = head { start_hashes.insert(h); }
        for (_, hash) in &refs {
            start_hashes.insert(*hash);
        }

        if start_hashes.is_empty() {
            println!("No commits yet");
            return Ok(());
        }

        // Simple BFS/DFS traversal to show all reachable commits
        let mut visited = HashSet::new();
        let mut queue: VecDeque<Hash> = start_hashes.into_iter().collect();
        
        // To keep it somewhat ordered, let's collect all and sort by timestamp if possible, 
        // but for now, just print as we find them to show everything.
        let mut all_commits = Vec::new();

        while let Some(hash) = queue.pop_front() {
            if visited.contains(&hash) {
                continue;
            }
            visited.insert(hash);
            
            if let Some(meta) = storage.commits().get_commit_meta(&hash)? {
                all_commits.push((hash, meta.timestamp));
                for parent in &meta.parent_hashes {
                    queue.push_back(*parent);
                }
            }
        }

        // Sort by timestamp descending
        all_commits.sort_by(|a, b| b.1.cmp(&a.1));

        for (hash, _) in all_commits {
            print_commit(storage, &hash, &labels)?;
        }
    }

    Ok(())
}

fn print_commit(storage: &Storage, hash: &Hash, labels: &HashMap<Hash, Vec<String>>) -> Result<()> {
    if let Some(meta) = storage.commits().get_commit_meta(hash)? {
        let mut line = format!("commit {}", hash).yellow().to_string();
        
        if let Some(names) = labels.get(hash) {
            let label_str = names.join(", ");
            line.push_str(&format!(" ({})", label_str).cyan().bold().to_string());
        }
        
        println!("{}", line);
        println!("Author: {}", meta.author);
        
        // Format date
        let datetime = chrono::DateTime::from_timestamp(meta.timestamp as i64, 0)
            .map(|dt| dt.format("%a %b %e %H:%M:%S %Y %z").to_string())
            .unwrap_or_else(|| "Unknown date".to_string());
        println!("Date:   {}\n", datetime);
        
        println!("    {}\n", meta.message);
    }
    Ok(())
}


#[cfg(test)]
mod tests;
