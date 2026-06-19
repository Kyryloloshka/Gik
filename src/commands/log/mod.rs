use crate::core::hash::Hash;
use crate::core::storage::Storage;
use crate::core::CommitMeta;
use crate::error::Result;
use colored::*;
use renderdag::{GraphRenderer, Node, RenderConfig};
use std::collections::{HashMap, HashSet};

pub fn log(storage: &Storage, all: bool, json: bool, skip: Option<usize>, limit: Option<usize>) -> Result<()> {
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
        if let Some(h) = head {
            start_hashes.insert(h);
        }
        for (_, hash) in &refs {
            start_hashes.insert(*hash);
        }
    } else {
        if let Some(h) = head {
            start_hashes.insert(h);
        }
    }

    if start_hashes.is_empty() {
        println!("No commits yet");
        return Ok(());
    }

    let mut pq = std::collections::BinaryHeap::new();
    let mut visited = HashSet::new();

    for head_hash in start_hashes {
        if visited.insert(head_hash) {
            if let Ok(Some(meta)) = storage.commits().get_commit_meta(&head_hash) {
                // PriorityQueue pops max first, so timestamp is perfect
                pq.push((meta.timestamp, head_hash, meta));
            }
        }
    }

    let skip_val = skip.unwrap_or(0);
    let l = limit.unwrap_or(50);
    let total_needed = if l == 0 { usize::MAX } else { skip_val + l };

    let mut selected_commits = Vec::new();

    while let Some((_ts, hash, meta)) = pq.pop() {
        selected_commits.push((hash, meta.clone()));

        if selected_commits.len() >= total_needed {
            break;
        }

        for parent in &meta.parent_hashes {
            if visited.insert(*parent) {
                if let Ok(Some(p_meta)) = storage.commits().get_commit_meta(parent) {
                    pq.push((p_meta.timestamp, *parent, p_meta));
                }
            }
        }
    }

    let selected_commits: Vec<_> = selected_commits.into_iter().skip(skip_val).collect();

    if json {
        let mut json_commits = Vec::new();
        for (hash, meta) in &selected_commits {
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
    for (hash, meta) in &selected_commits {
        dag_nodes.push(Node::new(
            hash.to_string(),
            meta.parent_hashes.iter().map(|h| h.to_string()).collect(),
        ));
    }

    let mut renderer = GraphRenderer::new(RenderConfig::default());
    let actual_glyphs = renderer.render_to_string(&dag_nodes);
    let mut rendered_lines = actual_glyphs.lines();

    for (hash, meta) in selected_commits {
        let graph_line = rendered_lines.next().unwrap_or("");
        print_commit_graph(hash, meta, &labels, graph_line);
    }

    Ok(())
}

fn graph_padding(rendered_line: &str) -> String {
    rendered_line
        .chars()
        .map(|c| match c {
            // Nodes that continue downwards
            '⊗' | '⍟' | '●' | '⦿' => '│',
            // Connections that continue downwards
            '│' | '╭' | '╮' | '┬' | '┤' | '├' | '╷' | '┊' => '│',
            // Everything else (Nodes that end, connections that go horizontally or up)
            _ => ' ',
        })
        .collect()
}

fn print_commit_graph(
    hash: Hash,
    meta: CommitMeta,
    labels: &HashMap<Hash, Vec<String>>,
    graph_line: &str,
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
