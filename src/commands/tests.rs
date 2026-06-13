use super::*;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_init_creates_db_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join(".gik.db");

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = init();

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    assert!(db_path.exists());
}

#[test]
fn test_stage_adds_file_to_storage() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    init().unwrap();

    let file_path = "test.txt";
    let content = "hello world\n";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    stage(file_path.to_string()).unwrap();

    let storage = Storage::new(".gik.db").unwrap();
    let hash_option = storage.get_staged_hash(file_path).unwrap();
    assert!(hash_option.is_some());

    let hash = hash_option.unwrap();
    let expected_hex = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";
    assert_eq!(hex::encode(hash), expected_hex);

    assert!(storage.contains_object(&hash).unwrap());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_commit_creates_objects_and_updates_head() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    init().unwrap();

    let file_path = "test.txt";
    let content = "hello world\n";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    stage(file_path.to_string()).unwrap();
    commit("initial commit".to_string()).unwrap();

    let first_head_option = {
        let storage = Storage::new(".gik.db").unwrap();
        let head = storage.get_current_head().unwrap();
        assert!(head.is_some());

        let staged = storage.get_all_staged_files().unwrap();
        assert!(staged.is_empty());

        assert!(storage.contains_object(&head.unwrap()).unwrap());
        head
    };

    let file_path2 = "test2.txt";
    {
        let mut file = File::create(file_path2).unwrap();
        file.write_all(b"second file\n").unwrap();
    }
    stage(file_path2.to_string()).unwrap();
    commit("second commit".to_string()).unwrap();

    let head2_option = {
        let storage = Storage::new(".gik.db").unwrap();
        storage.get_current_head().unwrap()
    };
    assert!(head2_option.is_some());
    assert_ne!(first_head_option, head2_option);

    std::env::set_current_dir(original_dir).unwrap();
}
