#[cfg(test)]
mod tests {
    use crate::core::hash::Hash;
    use crate::commands::branch::branch;
    use crate::commands::test_utils::TestEnv;
    use crate::error::Result;

    #[test]
    fn test_branch_create_and_list() -> Result<()> {
        let env = TestEnv::new();
        let storage = &env.storage;
        
        // Create a commit first
        storage.index().stage_file("test.txt", &Hash([1; 20]), 4, "test".as_bytes())?;
        
        let meta = crate::core::models::CommitMeta {
            message: "initial".to_string(),
            author: "test".to_string(),
            timestamp: 123456789,
            parent_hashes: vec![],
            tree_hash: Hash([2; 20]),
        };
        let head_hash = Hash([3; 20]);
        storage.commits().commit_transaction(
            Hash([2; 20]),
            vec![],
            head_hash,
            vec![],
            None,
            meta,
        )?;

        // Create branch
        branch(storage, Some("main".to_string()), false)?;

        // List branches
        let refs = storage.refs().list_refs()?;
        // Note: commit_transaction above didn't create 'main' automatically because it's a raw service call, not the 'commit' command.
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0, "main");
        assert_eq!(refs[0].1, head_hash);

        Ok(())
    }

    #[test]
    fn test_branch_delete() -> Result<()> {
        let env = TestEnv::new();
        let storage = &env.storage;

        let hash = Hash([1; 20]);
        storage.refs().set_ref("feature", &hash)?;
        
        // Delete branch
        branch(storage, Some("feature".to_string()), true)?;

        let refs = storage.refs().list_refs()?;
        assert!(refs.is_empty());

        Ok(())
    }
}
