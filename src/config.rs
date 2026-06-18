pub const GIK_DIR_NAME: &str = ".gik";
pub const DB_PATH: &str = ".gik/db";
pub const IGNORE_FILE_NAME: &str = ".gik.ignore";
pub const OBJECTS_DIR_NAME: &str = "objects";
pub const GIT_DIR_NAME: &str = ".git";
pub const TMP_OBJECT_PREFIX: &str = "tmp_";
pub const IO_BUFFER_SIZE: usize = 8192;

pub const DEFAULT_AUTHOR_NAME: &str = "Gik User";
pub const DEFAULT_AUTHOR_EMAIL: &str = "user@gik.local";

/// The current version of Gik, sourced from Cargo.toml at compile time.
pub const GIK_VERSION: &str = env!("CARGO_PKG_VERSION");

