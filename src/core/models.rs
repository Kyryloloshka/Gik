use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitMeta {
    pub parent_hashes: Vec<[u8; 20]>,
    pub tree_hash: [u8; 20],
    pub timestamp: u64,
    pub author: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum UndoAction {
    Unstage { path: String, old_hash: Option<[u8; 20]> },
    RevertCommit { old_head: Option<[u8; 20]>, new_head: [u8; 20] },
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
            parent_hashes: vec![[1; 20]],
            tree_hash: [2; 20],
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
