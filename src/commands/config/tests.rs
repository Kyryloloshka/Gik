use super::*;
use crate::core::storage::Storage;
use tempfile::tempdir;

#[test]
fn test_config_local_set_get() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // Set local config
    config(&storage, Some("user.name".to_string()), Some("Test User".to_string()), false, false).unwrap();

    // Verify it was set
    let val = storage.config().get_local("user.name").unwrap().unwrap();
    assert_eq!(val, "Test User");

    std::env::set_current_dir(original_dir).unwrap();
}
