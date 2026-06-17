use crate::error::Result;
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use crate::core::CommitMeta;
use std::collections::{HashSet, HashMap};
use colored::*;
use renderdag::{GraphRenderer, Node, RenderConfig};

pub fn log(storage: &Storage, all: bool, json: bool) -> Result<()> {
    let head = storage.commits().get_current_head()?;
    let refs = storage.refs().list_refs()?;
    
    let mut labels: HashMap<Hash, Vec<String>> = HashMap::new();
    for (name, hash) in &refs {
        labels.entry(*hash).or_default().push(name.clone());
    }
    if let Some(h) = head {
        labels.entry(h).or_default().push("HEAD".to_string());
    }

    let mut start_hashes = HashSet::new();
    if all {
        if let Some(h) = head { start_hashes.insert(h); }
        for (_, hash) in &refs { start_hashes.insert(*hash); }
    } else {
        if let Some(h) = head { start_hashes.insert(h); }
    }

    if start_hashes.is_empty() {
        println!("No commits yet");
        return Ok(());
    }

    // Topological sort using DFS
    let mut visited = HashSet::new();
    let mut sorted_commits = Vec::new();

    fn dfs(
        hash: Hash, 
        storage: &Storage, 
        visited: &mut HashSet<Hash>, 
        sorted: &mut Vec<(Hash, CommitMeta)>
    ) {
        if visited.contains(&hash) { return; }
        visited.insert(hash);
        if let Ok(Some(meta)) = storage.commits().get_commit_meta(&hash) {
            // Visit parents first
            for parent in &meta.parent_hashes {
                dfs(*parent, storage, visited, sorted);
            }
            // Push self
            sorted.push((hash, meta));
        }
    }

    // To ensure deterministic tie-breaking and prefer newer commits in the sort,
    // we should process start_hashes by timestamp. But DFS naturally groups branches.
    // For simplicity, just run DFS.
    let mut heads: Vec<Hash> = start_hashes.into_iter().collect();
    // Sort heads by timestamp so we traverse the newest branch first
    heads.sort_by_key(|h| {
        storage.commits().get_commit_meta(h).ok().flatten().map(|m| std::cmp::Reverse(m.timestamp)).unwrap_or(std::cmp::Reverse(0))
    });

    for head_hash in heads {
        dfs(head_hash, storage, &mut visited, &mut sorted_commits);
    }
    sorted_commits.reverse(); // Now children come before parents

    if json {
        let mut json_commits = Vec::new();
        for (hash, meta) in &sorted_commits {
            let refs = labels.get(hash).cloned().unwrap_or_default();
            let commit_obj = serde_json::json!({
                "hash": hash.to_string(),
                "parents": meta.parent_hashes.iter().map(|h| h.to_string()).collect::<Vec<_>>(),
                "author": meta.author,
                "timestamp": meta.timestamp,
                "message": meta.message,
                "refs": refs
            });
            json_commits.push(commit_obj);
        }
        println!("{}", serde_json::to_string_pretty(&json_commits).unwrap());
        return Ok(());
    }

    let mut dag_nodes = Vec::new();
    for (hash, meta) in &sorted_commits {
        dag_nodes.push(Node::new(
            hash.to_string(),
            meta.parent_hashes.iter().map(|h| h.to_string()).collect()
        ));
    }

    let mut renderer = GraphRenderer::new(RenderConfig::default());
    let actual_glyphs = renderer.render_to_string(&dag_nodes);
    let mut rendered_lines = actual_glyphs.lines();

    for (hash, meta) in sorted_commits {
        let graph_line = rendered_lines.next().unwrap_or("");
        print_commit_graph(hash, meta, &labels, graph_line);
    }

    Ok(())
}

fn graph_padding(rendered_line: &str) -> String {
    rendered_line.chars().map(|c| match c {
        // Nodes that continue downwards
        '⊗' | '⍟' | '●' | '⦿' => '│',
        // Connections that continue downwards
        '│' | '╭' | '╮' | '┬' | '┤' | '├' | '╷' | '┊' => '│',
        // Everything else (Nodes that end, connections that go horizontally or up)
        _ => ' ',
    }).collect()
}

fn print_commit_graph(
    hash: Hash, 
    meta: CommitMeta, 
    labels: &HashMap<Hash, Vec<String>>,
    graph_line: &str
) {
    let padding = graph_padding(graph_line);
    
    let mut header = format!("commit {}", hash).yellow().to_string();
    if let Some(names) = labels.get(&hash) {
        header.push_str(&format!(" ({})", names.join(", ")).cyan().bold().to_string());
    }

    println!("{} {}", graph_line.yellow(), header);
    println!("{} Author: {}", padding.yellow(), meta.author);
    
    let datetime = chrono::DateTime::from_timestamp(meta.timestamp as i64, 0)
        .map(|dt| dt.format("%a %b %e %H:%M:%S %Y %z").to_string())
        .unwrap_or_else(|| "Unknown date".to_string());
    println!("{} Date:   {}", padding.yellow(), datetime);
    println!("{}", padding.yellow());
    
    for msg_line in meta.message.lines() {
        println!("{}     {}", padding.yellow(), msg_line);
    }
}

