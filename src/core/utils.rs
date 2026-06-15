use crate::error::Result;
use std::path::{Path, PathBuf};

pub fn find_repo_root(current_dir: &Path) -> Result<PathBuf> {
    for ancestor in current_dir.ancestors() {
        if ancestor.join(crate::config::DB_PATH).exists() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Err(crate::error::GikError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Not a gik repository (or any of the parent directories)",
    )))
}

pub fn resolve_path(cwd: &Path, root_dir: &Path, user_path: &str) -> String {
    let joined = cwd.join(user_path);
    if let Ok(stripped) = joined.strip_prefix(root_dir) {
        let normalized = stripped.to_string_lossy().replace("\\", "/");
        if normalized.is_empty() {
            ".".to_string()
        } else {
            normalized
        }
    } else {
        let normalized = user_path.replace("\\", "/");
        if normalized.is_empty() {
            ".".to_string()
        } else {
            normalized
        }
    }
}
