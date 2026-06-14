use super::*;
use crate::core::storage::Storage;
use crate::commands::test_utils::*;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_log_runs_successfully() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // No commits yet
    assert!(log(&storage, false).is_ok());

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    crate::commands::stage(&storage, file_path.to_string()).unwrap();
    crate::commands::commit(&storage, "initial commit".to_string(), true, None).unwrap();

    // One commit
    assert!(log(&storage, false).is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}
