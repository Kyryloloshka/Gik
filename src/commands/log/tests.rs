use super::*;
use crate::core::storage::Storage;
use crate::commands::test_utils::*;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_log_runs_successfully() {
    let env = crate::commands::test_utils::TestEnv::new();

    // No commits yet
    assert!(log(&env.storage, false).is_ok());

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    crate::commands::stage(&env.storage, file_path.to_string()).unwrap();
    crate::commands::commit(&env.storage, "initial commit".to_string(), true, None).unwrap();

    // One commit
    assert!(log(&env.storage, false).is_ok());
}
