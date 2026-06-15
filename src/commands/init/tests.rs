use super::*;

#[test]
fn test_init_creates_db_file() {
    let env = crate::commands::test_utils::TestEnv::new();

    let db_path = "gik_test.db";
    let result = init(db_path);

    assert!(result.is_ok());
    assert!(env.dir.path().join(db_path).exists());
}
