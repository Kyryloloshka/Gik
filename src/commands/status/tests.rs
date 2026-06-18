use super::*;
use std::fs::File;
use std::io::Write;

#[test]
fn test_status_basic() {
    let env = crate::commands::test_utils::TestEnv::new();

    // 1. Clean state
    assert!(status(&env.storage, false).is_ok());

    // 2. Untracked file
    let file_path = "untracked.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"untracked content\n").unwrap();
    }
    assert!(status(&env.storage, false).is_ok());

    // 3. Staged file
    crate::commands::stage(&env.storage, file_path.to_string()).unwrap();
    assert!(status(&env.storage, false).is_ok());

    // 4. Unstaged (Modified) file
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"modified untracked content\n").unwrap();
    }
    assert!(status(&env.storage, false).is_ok());

    // 5. Committed state
    crate::commands::commit(&env.storage, "commit file".to_string(), true, None).unwrap();
    assert!(status(&env.storage, false).is_ok());
}
