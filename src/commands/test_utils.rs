pub const HELLO_CONTENT: &str = "hello world\n";
pub const HELLO_HASH: &str = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";

pub fn setup_test_storage(db_path: &str) -> crate::core::storage::Storage {
    crate::commands::init(db_path).unwrap();
    let storage = crate::core::storage::Storage::new(db_path).unwrap();
    storage.config().set_local("user.name", "Test User").unwrap();
    storage.config().set_local("user.email", "test@example.com").unwrap();
    storage
}
