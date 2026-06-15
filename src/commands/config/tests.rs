use super::*;
use crate::core::storage::Storage;
use tempfile::tempdir;

#[test]
fn test_config_local_set_get() {
    let env = crate::commands::test_utils::TestEnv::new();

    // Set local config
    config(&env.storage, Some("user.name".to_string()), Some("Test User".to_string()), false, false).unwrap();

    // Verify it was set
    let val = env.storage.config().get_local("user.name").unwrap().unwrap();
    assert_eq!(val, "Test User");
}
