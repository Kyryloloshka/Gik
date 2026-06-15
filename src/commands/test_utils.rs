pub const HELLO_CONTENT: &str = "hello world\n";
pub const HELLO_HASH: &str = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";

pub struct TestEnv {
    pub dir: tempfile::TempDir,
    pub original_dir: std::path::PathBuf,
    pub storage: crate::core::storage::Storage,
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original_dir).unwrap();
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnv {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let storage = setup_test_storage(".gik_test.db");
        Self {
            dir,
            original_dir,
            storage,
        }
    }
}

pub fn setup_test_storage(db_path: &str) -> crate::core::storage::Storage {
    crate::commands::init(db_path).unwrap();
    let storage = crate::core::storage::Storage::new(db_path).unwrap();
    storage.config().set_local("user.name", "Test User").unwrap();
    storage.config().set_local("user.email", "test@example.com").unwrap();
    storage
}
