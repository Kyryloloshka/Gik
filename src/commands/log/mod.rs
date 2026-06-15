use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use std::collections::{HashSet, VecDeque, HashMap};
use colored::*;

pub mod graph;
use graph::GraphRenderer;


pub fn log(storage: &Storage, all: bool, graph: bool) -> Result<()> {
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

    let mut renderer = if graph { Some(GraphRenderer::new()) } else { None };

    if !all {
        // Standard linear log from HEAD
        if head.is_none() {
            println!("No commits yet");
            return Ok(());
        }

        let mut current_hash = head;
        while let Some(hash) = current_hash {
            if let Some(meta) = storage.commits().get_commit_meta(&hash)? {
                let prefixes = renderer.as_mut().map(|r| r.process_commit(&hash, &meta.parent_hashes));
                print_commit(storage, &hash, &labels, prefixes.as_ref())?;
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

        let mut visited = HashSet::new();
        let mut queue: VecDeque<Hash> = start_hashes.into_iter().collect();
        let mut all_commits = Vec::new();

        while let Some(hash) = queue.pop_front() {
            if visited.contains(&hash) {
                continue;
            }
            visited.insert(hash);
            
            if let Some(meta) = storage.commits().get_commit_meta(&hash)? {
                for parent in &meta.parent_hashes {
                    queue.push_back(*parent);
                }
                all_commits.push((hash, meta));
            }
        }

        // Sort by timestamp descending
        all_commits.sort_by_key(|b| std::cmp::Reverse(b.1.timestamp));

        for (hash, meta) in all_commits {
            let prefixes = renderer.as_mut().map(|r| r.process_commit(&hash, &meta.parent_hashes));
            print_commit(storage, &hash, &labels, prefixes.as_ref())?;
        }
    }

    Ok(())
}

fn print_commit(
    storage: &Storage, 
    hash: &Hash, 
    labels: &HashMap<Hash, Vec<String>>,
    graph_prefixes: Option<&(String, String, Vec<String>)>
) -> Result<()> {
    if let Some(meta) = storage.commits().get_commit_meta(hash)? {
        let (commit_prefix, msg_prefix, transitions) = match graph_prefixes {
            Some((c, m, t)) => (c.as_str(), m.as_str(), t.as_slice()),
            None => ("", "", &[][..]),
        };

        let mut line = format!("{}commit {}", commit_prefix, hash).yellow().to_string();
        
        if let Some(names) = labels.get(hash) {
            let label_str = names.join(", ");
            line.push_str(&format!(" ({})", label_str).cyan().bold().to_string());
        }
        
        println!("{}", line);
        println!("{}Author: {}", msg_prefix, meta.author);
        
        let datetime = chrono::DateTime::from_timestamp(meta.timestamp as i64, 0)
            .map(|dt| dt.format("%a %b %e %H:%M:%S %Y %z").to_string())
            .unwrap_or_else(|| "Unknown date".to_string());
        println!("{}Date:   {}", msg_prefix, datetime);
        println!("{}", msg_prefix);
        
        let message_lines: Vec<&str> = meta.message.lines().collect();
        if message_lines.is_empty() {
            println!("{}", msg_prefix);
        } else {
            for msg_line in message_lines {
                println!("{}    {}", msg_prefix, msg_line);
            }
            println!("{}", msg_prefix);
        }

        for trans in transitions {
            println!("{}", trans);
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests;
