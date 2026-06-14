#[cfg(test)]
mod tests {
    use crate::commands::commit::commit;
    use crate::commands::stage::stage;
    use crate::core::storage::Storage;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_checkout_basic() {
        let dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let db_path = "gik_test.db";
        let storage = Storage::new(db_path).unwrap();

        // 1. Create repo, file "a.txt" with "v1", commit (get hash1).
        fs::write("a.txt", "v1").unwrap();
        stage(&storage, "a.txt".to_string()).unwrap();
        commit(&storage, "commit 1".to_string(), true).unwrap();
        let hash1 = storage.commits().get_current_head().unwrap().unwrap().to_string();

        // 2. Modify "a.txt" to "v2", create "b.txt", commit (get hash2).
        fs::write("a.txt", "v2").unwrap();
        fs::write("b.txt", "v1").unwrap();
        stage(&storage, "a.txt".to_string()).unwrap();
        stage(&storage, "b.txt".to_string()).unwrap();
        commit(&storage, "commit 2".to_string(), true).unwrap();
        let hash2 = storage.commits().get_current_head().unwrap().unwrap().to_string();

        // 3. Checkout hash1. Assert "a.txt" is "v1", "b.txt" does not exist.
        crate::commands::checkout::checkout(&storage, &hash1, false).expect("Checkout hash1 failed");
        assert_eq!(fs::read_to_string("a.txt").unwrap(), "v1");
        assert!(!std::path::Path::new("b.txt").exists(), "b.txt should not exist in commit 1");

        // 4. Checkout hash2. Assert "a.txt" is "v2", "b.txt" exists.
        crate::commands::checkout::checkout(&storage, &hash2, false).expect("Checkout hash2 failed");
        assert_eq!(fs::read_to_string("a.txt").unwrap(), "v2");
        assert_eq!(fs::read_to_string("b.txt").unwrap(), "v1");

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_checkout_safety() {
        let dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let db_path = "gik_test.db";
        let storage = Storage::new(db_path).unwrap();

        // 1. Create repo, commit a file.
        fs::write("file.txt", "v1").unwrap();
        stage(&storage, "file.txt".to_string()).unwrap();
        commit(&storage, "initial commit".to_string(), true).unwrap();
        let hash1 = storage.commits().get_current_head().unwrap().unwrap().to_string();

        // Create another commit so we have something to checkout FROM
        fs::write("file.txt", "v2").unwrap();
        stage(&storage, "file.txt".to_string()).unwrap();
        commit(&storage, "second commit".to_string(), true).unwrap();

        // 2. Modify file on disk (unstaged change).
        fs::write("file.txt", "v2 modified").unwrap();

        // 3. Attempt checkout. Assert it returns an error.
        let result = crate::commands::checkout::checkout(&storage, &hash1, false);
        assert!(result.is_err(), "Checkout should fail with unstaged changes");
        // Verify file content is still "v2 modified"
        assert_eq!(fs::read_to_string("file.txt").unwrap(), "v2 modified");

        // 4. Attempt checkout with force: true. Assert it succeeds and disk is restored.
        crate::commands::checkout::checkout(&storage, &hash1, true).expect("Checkout with force:true should succeed");
        assert_eq!(fs::read_to_string("file.txt").unwrap(), "v1");

        std::env::set_current_dir(original_dir).unwrap();
    }
}
