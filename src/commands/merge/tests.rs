#[cfg(test)]
mod tests {
    use crate::commands::test_utils::TestEnv;
    use crate::commands::{commit, checkout, merge, branch};
    use std::fs;

    #[test]
    fn test_merge_fast_forward() {
        let env = TestEnv::new();
        let storage = &env.storage;
        
        // Initial commit on main
        fs::write("f1.txt", "base").unwrap();
        crate::commands::stage::stage(storage, "f1.txt".to_string()).unwrap();
        commit::commit(storage, "c1".to_string(), false, None).unwrap();
        
        let c1_hash = storage.commits().get_current_head().unwrap().unwrap();
        
        // Create branch and checkout
        branch::branch(storage, Some("feature".to_string()), false).unwrap();
        checkout::checkout(storage, "feature", false).unwrap();
        
        // Commit on feature
        fs::write("f1.txt", "feature update").unwrap();
        crate::commands::stage::stage(storage, "f1.txt".to_string()).unwrap();
        commit::commit(storage, "c2".to_string(), false, None).unwrap();
        
        let c2_hash = storage.commits().get_current_head().unwrap().unwrap();
        
        // Checkout main
        checkout::checkout(storage, "main", false).unwrap();
        
        // Merge feature into main
        merge::merge(storage, "feature").unwrap();
        
        // Assert fast-forwarded
        let current_head = storage.commits().get_current_head().unwrap().unwrap();
        assert_eq!(current_head, c2_hash);
        assert_ne!(current_head, c1_hash);
    }

    #[test]
    fn test_merge_creates_multi_parent_commit() {
        let env = TestEnv::new();
        let storage = &env.storage;
        
        // Initial commit on main
        fs::write("f1.txt", "base").unwrap();
        crate::commands::stage::stage(storage, "f1.txt".to_string()).unwrap();
        commit::commit(storage, "c1".to_string(), false, None).unwrap();
        
        // Create feature branch
        branch::branch(storage, Some("feature".to_string()), false).unwrap();
        
        // Commit on main
        fs::write("f2.txt", "main update").unwrap();
        crate::commands::stage::stage(storage, "f2.txt".to_string()).unwrap();
        commit::commit(storage, "c2".to_string(), false, None).unwrap();
        let main_hash = storage.commits().get_current_head().unwrap().unwrap();
        
        // Commit on feature
        checkout::checkout(storage, "feature", false).unwrap();
        fs::write("f3.txt", "feature update").unwrap();
        crate::commands::stage::stage(storage, "f3.txt".to_string()).unwrap();
        commit::commit(storage, "c3".to_string(), false, None).unwrap();
        let feature_hash = storage.commits().get_current_head().unwrap().unwrap();
        
        // Checkout main and merge feature
        checkout::checkout(storage, "main", false).unwrap();
        merge::merge(storage, "feature").unwrap();
        
        // Verify MERGE_HEAD is set
        let merge_head = storage.session().get_merge_head().unwrap().unwrap();
        assert_eq!(merge_head, feature_hash);
        
        // Finalize merge
        commit::commit(storage, "Merge feature into main".to_string(), true, None).unwrap();
        
        // Verify MERGE_HEAD is cleared
        assert!(storage.session().get_merge_head().unwrap().is_none());
        
        // Verify multi-parent commit
        let merge_commit_hash = storage.commits().get_current_head().unwrap().unwrap();
        let meta = storage.commits().get_commit_meta(&merge_commit_hash).unwrap().unwrap();
        
        assert_eq!(meta.parent_hashes.len(), 2);
        assert_eq!(meta.parent_hashes[0], main_hash);
        assert_eq!(meta.parent_hashes[1], feature_hash);
    }
}
