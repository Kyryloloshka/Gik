use crate::core::hash::Hash;
use crate::core::objects::tree::get_commit_tree_files;
use crate::core::storage::Storage;
use crate::error::Result;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq)]
pub enum MergeAction {
    KeepOurs,
    TakeTheirs(Hash),
    DeleteOurs,
    Merge {
        base: Option<Hash>,
        ours: Option<Hash>,
        theirs: Option<Hash>,
    },
}

pub fn analyze_trees(
    storage: &Storage,
    base_tree: Option<&Hash>,
    ours_tree: &Hash,
    theirs_tree: &Hash,
) -> Result<HashMap<String, MergeAction>> {
    let base_map = if let Some(h) = base_tree {
        get_commit_tree_files(storage, h)?
    } else {
        HashMap::new()
    };

    let ours_map = get_commit_tree_files(storage, ours_tree)?;
    let theirs_map = get_commit_tree_files(storage, theirs_tree)?;

    Ok(analyze_tree_maps(&base_map, &ours_map, &theirs_map))
}

pub fn analyze_tree_maps(
    base_map: &HashMap<String, Hash>,
    ours_map: &HashMap<String, Hash>,
    theirs_map: &HashMap<String, Hash>,
) -> HashMap<String, MergeAction> {
    let mut actions = HashMap::new();

    let mut all_paths = HashSet::new();
    for path in base_map
        .keys()
        .chain(ours_map.keys())
        .chain(theirs_map.keys())
    {
        all_paths.insert(path.clone());
    }

    for path in all_paths {
        let base_h = base_map.get(&path).copied();
        let ours_h = ours_map.get(&path).copied();
        let theirs_h = theirs_map.get(&path).copied();

        if ours_h == theirs_h {
            actions.insert(path, MergeAction::KeepOurs);
            continue;
        }

        if ours_h == base_h {
            if let Some(th) = theirs_h {
                actions.insert(path, MergeAction::TakeTheirs(th));
            } else {
                actions.insert(path, MergeAction::DeleteOurs);
            }
            continue;
        }

        if theirs_h == base_h {
            actions.insert(path, MergeAction::KeepOurs);
            continue;
        }

        actions.insert(
            path,
            MergeAction::Merge {
                base: base_h,
                ours: ours_h,
                theirs: theirs_h,
            },
        );
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_hash(val: u8) -> Hash {
        Hash([val; 20])
    }

    #[test]
    fn test_analyzer_keep_ours_when_theirs_unchanged() {
        let mut base = HashMap::new();
        let mut ours = HashMap::new();
        let mut theirs = HashMap::new();

        base.insert("file.txt".to_string(), dummy_hash(1));
        ours.insert("file.txt".to_string(), dummy_hash(2));
        theirs.insert("file.txt".to_string(), dummy_hash(1));

        let actions = analyze_tree_maps(&base, &ours, &theirs);
        assert_eq!(actions["file.txt"], MergeAction::KeepOurs);
    }

    #[test]
    fn test_analyzer_take_theirs_when_ours_unchanged() {
        let mut base = HashMap::new();
        let mut ours = HashMap::new();
        let mut theirs = HashMap::new();

        base.insert("file.txt".to_string(), dummy_hash(1));
        ours.insert("file.txt".to_string(), dummy_hash(1));
        theirs.insert("file.txt".to_string(), dummy_hash(3));

        let actions = analyze_tree_maps(&base, &ours, &theirs);
        assert_eq!(actions["file.txt"], MergeAction::TakeTheirs(dummy_hash(3)));
    }

    #[test]
    fn test_analyzer_delete_ours_when_theirs_deleted_and_ours_unchanged() {
        let mut base = HashMap::new();
        let mut ours = HashMap::new();
        let theirs = HashMap::new();

        base.insert("file.txt".to_string(), dummy_hash(1));
        ours.insert("file.txt".to_string(), dummy_hash(1));

        let actions = analyze_tree_maps(&base, &ours, &theirs);
        assert_eq!(actions["file.txt"], MergeAction::DeleteOurs);
    }

    #[test]
    fn test_analyzer_merge_when_both_changed_differently() {
        let mut base = HashMap::new();
        let mut ours = HashMap::new();
        let mut theirs = HashMap::new();

        base.insert("file.txt".to_string(), dummy_hash(1));
        ours.insert("file.txt".to_string(), dummy_hash(2));
        theirs.insert("file.txt".to_string(), dummy_hash(3));

        let actions = analyze_tree_maps(&base, &ours, &theirs);
        assert_eq!(
            actions["file.txt"],
            MergeAction::Merge {
                base: Some(dummy_hash(1)),
                ours: Some(dummy_hash(2)),
                theirs: Some(dummy_hash(3)),
            }
        );
    }

    #[test]
    fn test_analyzer_keep_ours_when_both_changed_identically() {
        let mut base = HashMap::new();
        let mut ours = HashMap::new();
        let mut theirs = HashMap::new();

        base.insert("file.txt".to_string(), dummy_hash(1));
        ours.insert("file.txt".to_string(), dummy_hash(4));
        theirs.insert("file.txt".to_string(), dummy_hash(4));

        let actions = analyze_tree_maps(&base, &ours, &theirs);
        assert_eq!(actions["file.txt"], MergeAction::KeepOurs);
    }
}
