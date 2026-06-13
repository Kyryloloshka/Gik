use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitMeta {
    pub parent_hashes: Vec<crate::core::Hash>,
    pub tree_hash: crate::core::Hash,
    pub timestamp: u64,
    pub author: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum UndoAction {
    Unstage { path: String, old_hash: Option<crate::core::Hash> },
    RevertCommit { old_head: Option<crate::core::Hash>, new_head: crate::core::Hash },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionRecord {
    pub timestamp: u64,
    pub action: UndoAction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode;

    #[test]
    fn test_commit_meta_serialization() {
        let meta = CommitMeta {
            parent_hashes: vec![crate::core::Hash([1; 20])],
            tree_hash: crate::core::Hash([2; 20]),
            timestamp: 1234567890,
            author: "Author".to_string(),
            message: "Initial commit".to_string(),
        };

        let encoded: Vec<u8> = bincode::serialize(&meta).unwrap();
        let decoded: CommitMeta = bincode::deserialize(&encoded).unwrap();

        assert_eq!(meta, decoded);
    }

    #[test]
    fn test_transaction_record_serialization() {
        let record = TransactionRecord {
            timestamp: 1234567890,
            action: UndoAction::Unstage {
                path: "test.txt".to_string(),
                old_hash: None,
            },
        };

        let encoded: Vec<u8> = bincode::serialize(&record).unwrap();
        let decoded: TransactionRecord = bincode::deserialize(&encoded).unwrap();

        assert_eq!(record, decoded);
    }
}
