use super::*;
use crate::core::storage::Storage;
use crate::commands::test_utils::*;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_stage_adds_file_to_storage() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    stage(&storage, file_path.to_string()).unwrap();

    let hash_option = storage.index().get_staged_hash(file_path).unwrap();
    assert!(hash_option.is_some());

    let hash = hash_option.unwrap();
    assert_eq!(hex::encode(hash.0), HELLO_HASH);

    assert!(storage.objects().contains_object(&hash).unwrap());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_stage_deletion() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    // 1. Stage normally
    stage(&storage, file_path.to_string()).unwrap();
    assert!(storage.index().get_staged_hash(file_path).unwrap().is_some());

    // 2. Delete from disk
    std::fs::remove_file(file_path).unwrap();

    // 3. Stage the deletion
    stage(&storage, file_path.to_string()).unwrap();
    assert!(storage.index().get_staged_hash(file_path).unwrap().is_none());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_stage_dot_adds_everything() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // Create a bunch of files
    File::create("a.txt").unwrap();
    File::create("b.txt").unwrap();
    std::fs::create_dir("subdir").unwrap();
    File::create("subdir/c.txt").unwrap();

    // Stage all
    stage(&storage, ".".to_string()).unwrap();

    let staged = storage.index().get_all_staged_files().unwrap();
    assert_eq!(staged.len(), 3);
    
    let paths: Vec<String> = staged.into_iter().map(|(p, _)| p).collect();
    assert!(paths.contains(&"a.txt".to_string()));
    assert!(paths.contains(&"b.txt".to_string()));
    assert!(paths.contains(&"subdir/c.txt".to_string()));

    std::env::set_current_dir(original_dir).unwrap();
}
