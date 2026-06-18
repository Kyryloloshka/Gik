use super::*;
use std::io::Write;
use std::fs::File;

#[test]
fn test_undo_works() {
    let env = crate::commands::test_utils::TestEnv::new();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"hello world\n").unwrap();
    }

    // Undo staging
    crate::commands::stage(&env.storage, file_path.to_string()).unwrap();
    assert!(env.storage.index().get_staged_hash(file_path).unwrap().is_some());
    undo(&env.storage, true, false).unwrap();
    assert!(env.storage.index().get_staged_hash(file_path).unwrap().is_none());

    // Undo commit
    crate::commands::stage(&env.storage, file_path.to_string()).unwrap();
    crate::commands::commit(&env.storage, "initial commit".to_string(), true, None).unwrap();
    let first_head = env.storage.commits().get_current_head().unwrap();
    assert!(first_head.is_some());

    undo(&env.storage, true, false).unwrap();
    assert!(env.storage.commits().get_current_head().unwrap().is_none());
}
