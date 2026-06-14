use super::*;
use crate::core::storage::Storage;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_restore_file() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // 1. Create and commit a file
    fs::write("a.txt", "v1").unwrap();
    crate::commands::stage(&storage, "a.txt".to_string()).unwrap();
    crate::commands::commit(&storage, "initial".to_string(), true, None).unwrap();

    // 2. Modify on disk
    fs::write("a.txt", "v2").unwrap();
    assert_eq!(fs::read_to_string("a.txt").unwrap(), "v2");

    // 3. Restore
    restore(&storage, "a.txt").unwrap();
    assert_eq!(fs::read_to_string("a.txt").unwrap(), "v1");

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_restore_dot() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // 1. Create and commit
    fs::write("a.txt", "v1").unwrap();
    fs::write("b.txt", "v1").unwrap();
    crate::commands::stage(&storage, ".".to_string()).unwrap();
    crate::commands::commit(&storage, "initial".to_string(), true, None).unwrap();

    // 2. Modify and delete
    fs::write("a.txt", "v2").unwrap();
    fs::remove_file("b.txt").unwrap();
    fs::write("c.txt", "untracked").unwrap();

    // 3. Restore dot
    restore(&storage, ".").unwrap();
    assert_eq!(fs::read_to_string("a.txt").unwrap(), "v1");
    assert!(fs::metadata("b.txt").is_ok());
    // Note: Standard Git restore . doesn't delete untracked files by default, 
    // but our restore_workspace does if it's not in the tree.
    // Actually, restore_workspace deletes files on disk that are NOT in target tree.
    assert!(fs::metadata("c.txt").is_err());

    std::env::set_current_dir(original_dir).unwrap();
}
