use super::*;
use std::fs::File;
use std::io::Write;

#[test]
fn test_diff_unstaged() {
    let env = crate::commands::test_utils::TestEnv::new();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\nline 2\n").unwrap();
    }

    crate::commands::stage(&env.storage, file_path.to_string()).unwrap();

    // Modify disk
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\nline 2 modified\n").unwrap();
    }

    // Unstaged diff should succeed
    assert!(diff(&env.storage, false).is_ok());
}

#[test]
fn test_diff_staged() {
    let env = crate::commands::test_utils::TestEnv::new();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\n").unwrap();
    }

    crate::commands::commit(&env.storage, "initial".to_string(), false, None).unwrap();

    // Stage a modification
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"line 1\nline 2\n").unwrap();
    }
    crate::commands::stage(&env.storage, file_path.to_string()).unwrap();

    // Staged diff should succeed
    assert!(diff(&env.storage, true).is_ok());
}
