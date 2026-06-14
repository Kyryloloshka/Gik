use super::*;
use crate::core::storage::Storage;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_undo_works() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"hello world\n").unwrap();
    }

    // Undo staging
    crate::commands::stage(&storage, file_path.to_string()).unwrap();
    assert!(storage.index().get_staged_hash(file_path).unwrap().is_some());
    undo(&storage).unwrap();
    assert!(storage.index().get_staged_hash(file_path).unwrap().is_none());

    // Undo commit
    crate::commands::stage(&storage, file_path.to_string()).unwrap();
    crate::commands::commit(&storage, "initial commit".to_string(), true).unwrap();
    let first_head = storage.commits().get_current_head().unwrap();
    assert!(first_head.is_some());

    undo(&storage).unwrap();
    assert!(storage.commits().get_current_head().unwrap().is_none());

    std::env::set_current_dir(original_dir).unwrap();
}
