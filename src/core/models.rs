use crate::core::hash::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitMeta {
    pub parent_hashes: Vec<Hash>,
    pub tree_hash: Hash,
    pub timestamp: u64,
    pub author: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IndexEntry {
    pub hash: Hash,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum UndoAction {
    UpdateIndex { path: String, old_entry: Option<IndexEntry>, new_entry: Option<IndexEntry> },
    RevertCommit { old_head: Option<Hash>, new_head: Hash },
    Checkout { old_head: Option<Hash>, new_head: Hash },
    MoveBookmark { name: String, old_hash: Option<Hash>, new_hash: Hash },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum CommandType {
    Stage,
    Unstage,
    Commit,
    Checkout,
    Merge,
    Branch,
    Restore,
    Pull,
    Revert,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TransactionBatch {
    pub id: u64, // Usually the timestamp
    pub command: CommandType,
    pub description: String,
    pub actions: Vec<UndoAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileState {
    New,
    Modified,
    Deleted,
}

#[derive(Debug, Default)]
pub struct RepoStatus {
    pub staged: std::collections::HashMap<String, FileState>,
    pub unstaged: std::collections::HashMap<String, FileState>,
    pub untracked: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode;

    #[test]
    fn test_commit_meta_serialization() {
        let meta = CommitMeta {
            parent_hashes: vec![Hash([1; 20])],
            tree_hash: Hash([2; 20]),
            timestamp: 1234567890,
            author: "Author".to_string(),
            message: "Initial commit".to_string(),
        };

        let encoded: Vec<u8> = bincode::serialize(&meta).unwrap();
        let decoded: CommitMeta = bincode::deserialize(&encoded).unwrap();

        assert_eq!(meta, decoded);
    }

    #[test]
    fn test_transaction_batch_serialization() {
        let record = TransactionBatch {
            id: 1234567890,
            command: CommandType::Stage,
            description: "gik stage test.txt".to_string(),
            actions: vec![UndoAction::UpdateIndex {
                path: "test.txt".to_string(),
                old_entry: None,
                new_entry: Some(IndexEntry { hash: Hash([0; 20]), size: 0, mtime: 0 }),
            }],
        };

        let encoded: Vec<u8> = bincode::serialize(&record).unwrap();
        let decoded: TransactionBatch = bincode::deserialize(&encoded).unwrap();

        assert_eq!(record.id, decoded.id);
        assert_eq!(record.command, decoded.command);
    }
}
