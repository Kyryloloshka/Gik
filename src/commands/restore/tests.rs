use super::*;
use std::fs;

#[test]
fn test_restore_file() {
    let env = crate::commands::test_utils::TestEnv::new();

    // 1. Create and commit a file
    fs::write("a.txt", "v1").unwrap();
    crate::commands::stage(&env.storage, "a.txt".to_string()).unwrap();
    crate::commands::commit(&env.storage, "initial".to_string(), true, None).unwrap();

    // 2. Modify on disk
    fs::write("a.txt", "v2").unwrap();
    assert_eq!(fs::read_to_string("a.txt").unwrap(), "v2");

    // 3. Restore
    restore(&env.storage, "a.txt").unwrap();
    assert_eq!(fs::read_to_string("a.txt").unwrap(), "v1");
}

#[test]
fn test_restore_dot() {
    let env = crate::commands::test_utils::TestEnv::new();

    // 1. Create and commit
    fs::write("a.txt", "v1").unwrap();
    fs::write("b.txt", "v1").unwrap();
    crate::commands::stage(&env.storage, ".".to_string()).unwrap();
    crate::commands::commit(&env.storage, "initial".to_string(), true, None).unwrap();

    // 2. Modify and delete
    fs::write("a.txt", "v2").unwrap();
    fs::remove_file("b.txt").unwrap();
    fs::write("c.txt", "untracked").unwrap();

    // 3. Restore dot
    restore(&env.storage, ".").unwrap();
    assert_eq!(fs::read_to_string("a.txt").unwrap(), "v1");
    assert!(fs::metadata("b.txt").is_ok());
    // Note: Standard Git restore . doesn't delete untracked files by default,
    // but our restore_workspace does if it's not in the tree.
    // Actually, restore_workspace deletes files on disk that are NOT in target tree.
    assert!(fs::metadata("c.txt").is_err());
}
