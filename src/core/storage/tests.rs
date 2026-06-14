use super::*;
use tempfile::NamedTempFile;

#[test]
fn test_storage_init() {
    let tmp_file = NamedTempFile::new().unwrap();
    let storage = Storage::new(tmp_file.path()).unwrap();

    let read_txn = storage.repo.db.begin_read().unwrap();
    assert!(read_txn.open_table(OBJECTS).is_ok());
    assert!(read_txn.open_table(COMMITS_METADATA).is_ok());
    assert!(read_txn.open_table(HEADS).is_ok());
    assert!(read_txn.open_table(STAGE_INDEX).is_ok());
    assert!(read_txn.open_table(TRANSACTION_LOG).is_ok());
}

#[test]
fn test_storage_contains_object() {
    let tmp_file = NamedTempFile::new().unwrap();
    let storage = Storage::new(tmp_file.path()).unwrap();
    let hash = Hash([0u8; 20]);
    assert!(!storage.contains_object(&hash).unwrap());
}

#[test]
fn test_storage_stage_file() {
    let tmp_file = NamedTempFile::new().unwrap();
    let storage = Storage::new(tmp_file.path()).unwrap();
    let path = "test.txt";
    let content = b"hello world";
    let hash = Hash([1u8; 20]); // Dummy hash
    let size = content.len() as u64;

    storage.stage_file(path, &hash, size, &content[..]).unwrap();

    // Verify STAGE_INDEX
    let read_txn = storage.repo.db.begin_read().unwrap();
    let index = read_txn.open_table(STAGE_INDEX).unwrap();
    let staged_hash_guard = index.get(path).unwrap().unwrap();
    assert_eq!(staged_hash_guard.value(), &hash.0);

    // Verify OBJECTS
    let objects = read_txn.open_table(OBJECTS).unwrap();
    assert!(objects.get(&hash.0).unwrap().is_some());
}

