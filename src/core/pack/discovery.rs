use crate::core::hash::Hash;
use crate::core::objects::{decompress_object, tree::parse_tree};
use crate::core::storage::Storage;
use crate::error::Result;
use std::collections::{HashSet, VecDeque};

pub fn discover_missing_objects(
    storage: &Storage,
    remote_head: Option<&Hash>,
    local_head: &Hash,
) -> Result<Vec<Hash>> {
    let mut known_objects = HashSet::new();

    // Step 1: Collect known objects from RemoteHead
    if let Some(r_head) = remote_head {
        let mut queue = VecDeque::new();
        queue.push_back(*r_head);

        while let Some(h) = queue.pop_front() {
            if !known_objects.insert(h) {
                continue;
            }

            if let Some(meta) = storage.commits().get_commit_meta(&h)? {
                queue.push_back(meta.tree_hash);
                for parent in meta.parent_hashes {
                    queue.push_back(parent);
                }
            } else if let Some(compressed) = storage.objects().get_object(&h)? {
                let (obj_type, _, content) = decompress_object(&compressed[..])?;
                if obj_type == "tree" {
                    let entries = parse_tree(&content)?;
                    for entry in entries {
                        queue.push_back(entry.2);
                    }
                }
            }
        }
    }

    // Step 2: Collect missing objects from LocalHead
    let mut missing_objects = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(*local_head);

    while let Some(h) = queue.pop_front() {
        if known_objects.contains(&h) || !visited.insert(h) {
            continue;
        }

        missing_objects.push(h);

        if let Some(meta) = storage.commits().get_commit_meta(&h)? {
            queue.push_back(meta.tree_hash);
            for parent in meta.parent_hashes {
                queue.push_back(parent);
            }
        } else if let Some(compressed) = storage.objects().get_object(&h)? {
            let (obj_type, _, content) = decompress_object(&compressed[..])?;
            if obj_type == "tree" {
                let entries = parse_tree(&content)?;
                for entry in entries {
                    queue.push_back(entry.2);
                }
            }
        }
    }

    Ok(missing_objects)
}
