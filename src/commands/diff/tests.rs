use super::*;
use crate::core::storage::Storage;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_diff_unstaged() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\nline 2\n").unwrap();
    }

    crate::commands::stage(&storage, file_path.to_string()).unwrap();

    // Modify disk
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\nline 2 modified\n").unwrap();
    }

    // Unstaged diff should succeed
    assert!(diff(&storage, false).is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_diff_staged() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\n").unwrap();
    }

    crate::commands::commit(&storage, "initial".to_string(), false).unwrap();

    // Stage a modification
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\nline 2\n").unwrap();
    }
    crate::commands::stage(&storage, file_path.to_string()).unwrap();

    // Staged diff should succeed
    assert!(diff(&storage, true).is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}
