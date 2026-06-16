use std::collections::{HashSet, VecDeque};
use crate::error::Result;
use crate::core::hash::Hash;
use crate::core::storage::Storage;

/// Finds the Lowest Common Ancestor (LCA) of two commits using BFS.
pub fn find_lowest_common_ancestor(
    storage: &Storage,
    hash1: &Hash,
    hash2: &Hash,
) -> Result<Option<Hash>> {
    if hash1 == hash2 {
        return Ok(Some(*hash1));
    }

    // Use Bidirectional BFS: search upwards from both commits until their
    // visited sets intersect. This efficiently finds the Lowest Common Ancestor (LCA).
    let mut queue1 = VecDeque::new();
    let mut queue2 = VecDeque::new();
    
    let mut visited1 = HashSet::new();
    let mut visited2 = HashSet::new();

    queue1.push_back(*hash1);
    visited1.insert(*hash1);

    queue2.push_back(*hash2);
    visited2.insert(*hash2);

    let commit_service = storage.commits();

    while !queue1.is_empty() || !queue2.is_empty() {
        if let Some(current1) = queue1.pop_front() {
            if visited2.contains(&current1) {
                return Ok(Some(current1));
            }

            if let Some(meta) = commit_service.get_commit_meta(&current1)? {
                for parent in meta.parent_hashes {
                    if visited1.insert(parent) {
                        queue1.push_back(parent);
                    }
                }
            }
        }

        if let Some(current2) = queue2.pop_front() {
            if visited1.contains(&current2) {
                return Ok(Some(current2));
            }

            if let Some(meta) = commit_service.get_commit_meta(&current2)? {
                for parent in meta.parent_hashes {
                    if visited2.insert(parent) {
                        queue2.push_back(parent);
                    }
                }
            }
        }
    }

    Ok(None)
}


