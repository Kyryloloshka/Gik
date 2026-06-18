use super::*;
use crate::commands::test_utils::*;
use std::fs::File;
use std::io::Write;

#[test]
fn test_stage_adds_file_to_storage() {
    let env = crate::commands::test_utils::TestEnv::new();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    stage(&env.storage, file_path.to_string()).unwrap();

    let hash_option = env.storage.index().get_staged_hash(file_path).unwrap();
    assert!(hash_option.is_some());

    let hash = hash_option.unwrap();
    assert_eq!(hex::encode(hash.0), HELLO_HASH);

    assert!(env.storage.objects().contains_object(&hash).unwrap());
}

#[test]
fn test_stage_deletion() {
    let env = crate::commands::test_utils::TestEnv::new();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    // 1. Stage normally
    stage(&env.storage, file_path.to_string()).unwrap();
    assert!(env
        .storage
        .index()
        .get_staged_hash(file_path)
        .unwrap()
        .is_some());

    // 2. Delete from disk
    std::fs::remove_file(file_path).unwrap();

    // 3. Stage the deletion
    stage(&env.storage, file_path.to_string()).unwrap();
    assert!(env
        .storage
        .index()
        .get_staged_hash(file_path)
        .unwrap()
        .is_none());
}

#[test]
fn test_stage_dot_adds_everything() {
    let env = crate::commands::test_utils::TestEnv::new();

    // Create a bunch of files
    File::create("a.txt").unwrap();
    File::create("b.txt").unwrap();
    std::fs::create_dir("subdir").unwrap();
    File::create("subdir/c.txt").unwrap();

    // Stage all
    stage(&env.storage, ".".to_string()).unwrap();

    let staged = env.storage.index().get_all_staged_files().unwrap();
    assert_eq!(staged.len(), 3);

    let paths: Vec<String> = staged.into_iter().map(|(p, _)| p).collect();
    assert!(paths.contains(&"a.txt".to_string()));
    assert!(paths.contains(&"b.txt".to_string()));
    assert!(paths.contains(&"subdir/c.txt".to_string()));
}
