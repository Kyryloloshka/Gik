use super::*;
use crate::core::storage::Storage;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_init_creates_db_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    let result = init(db_path_str);

    assert!(result.is_ok());
    assert!(db_path.exists());
}

#[test]
fn test_stage_adds_file_to_storage() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    let file_path = dir.path().join("test.txt");
    let content = "hello world\n";
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();
    stage(&storage, file_path_str.clone()).unwrap();

    let hash_option = storage.get_staged_hash(&file_path_str).unwrap();
    assert!(hash_option.is_some());

    let hash = hash_option.unwrap();
    let expected_hex = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";
    assert_eq!(hex::encode(hash), expected_hex);

    assert!(storage.contains_object(&hash).unwrap());
}

#[test]
fn test_commit_creates_objects_and_updates_head() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    let file_path = dir.path().join("test.txt");
    let content = "hello world\n";
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();
    stage(&storage, file_path_str).unwrap();
    commit(&storage, "initial commit".to_string()).unwrap();

    let first_head_option = {
        let head = storage.get_current_head().unwrap();
        assert!(head.is_some());

        let staged = storage.get_all_staged_files().unwrap();
        assert!(staged.is_empty());

        assert!(storage.contains_object(&head.unwrap()).unwrap());
        head
    };

    let file_path2 = dir.path().join("test2.txt");
    {
        let mut file = File::create(&file_path2).unwrap();
        file.write_all(b"second file\n").unwrap();
    }
    let file_path2_str = file_path2.to_str().unwrap().to_string();
    stage(&storage, file_path2_str).unwrap();
    commit(&storage, "second commit".to_string()).unwrap();

    let head2_option = storage.get_current_head().unwrap();
    assert!(head2_option.is_some());
    assert_ne!(first_head_option, head2_option);
}

#[test]
fn test_log_runs_successfully() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    // No commits yet
    assert!(log(&storage).is_ok());

    let file_path = dir.path().join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world\n").unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();
    stage(&storage, file_path_str).unwrap();
    commit(&storage, "initial commit".to_string()).unwrap();

    // One commit
    assert!(log(&storage).is_ok());
}

#[test]
fn test_undo_works() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    let file_path = dir.path().join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world\n").unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();

    // Undo staging
    stage(&storage, file_path_str.clone()).unwrap();
    assert!(storage.get_staged_hash(&file_path_str).unwrap().is_some());
    undo(&storage).unwrap();
    assert!(storage.get_staged_hash(&file_path_str).unwrap().is_none());

    // Undo commit
    stage(&storage, file_path_str).unwrap();
    commit(&storage, "initial commit".to_string()).unwrap();
    let first_head = storage.get_current_head().unwrap();
    assert!(first_head.is_some());

    undo(&storage).unwrap();
    assert!(storage.get_current_head().unwrap().is_none());
}
