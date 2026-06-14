use super::*;
use tempfile::tempdir;

#[test]
fn test_init_creates_db_file() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test.db";
    let result = init(db_path);

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    assert!(dir.path().join(db_path).exists());
}
