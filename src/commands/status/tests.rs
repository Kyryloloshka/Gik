use super::*;
use crate::core::storage::Storage;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_status_basic() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // 1. Clean state
    assert!(status(&storage).is_ok());

    // 2. Untracked file
    let file_path = "untracked.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"untracked content\n").unwrap();
    }
    assert!(status(&storage).is_ok());

    // 3. Staged file
    crate::commands::stage(&storage, file_path.to_string()).unwrap();
    assert!(status(&storage).is_ok());

    // 4. Unstaged (Modified) file
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"modified untracked content\n").unwrap();
    }
    assert!(status(&storage).is_ok());

    // 5. Committed state
    crate::commands::commit(&storage, "commit file".to_string(), true).unwrap();
    assert!(status(&storage).is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}
