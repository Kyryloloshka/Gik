use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitMeta {
    pub parent_hashes: Vec<[u8; 20]>,
    pub tree_hash: [u8; 20],
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionRecord {
    pub timestamp: u64,
    pub operation: String,
    pub details: String,
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
        };

        let encoded: Vec<u8> = bincode::serialize(&meta).unwrap();
        let decoded: CommitMeta = bincode::deserialize(&encoded).unwrap();

        assert_eq!(meta, decoded);
    }

    #[test]
    fn test_transaction_record_serialization() {
        let record = TransactionRecord {
            timestamp: 1234567890,
            operation: "COMMIT".to_string(),
            details: "initial commit".to_string(),
        };

        let encoded: Vec<u8> = bincode::serialize(&record).unwrap();
        let decoded: TransactionRecord = bincode::deserialize(&encoded).unwrap();

        assert_eq!(record, decoded);
    }
}
