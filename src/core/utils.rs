use crate::error::Result;
use std::path::{Path, PathBuf};

pub fn find_repo_root(current_dir: &Path) -> Result<PathBuf> {
    for ancestor in current_dir.ancestors() {
        if ancestor.join(crate::config::DB_PATH).exists() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Err(crate::error::GikError::NotFound("Not a gik repository (or any of the parent directories): .gik".to_string()))
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

pub fn resolve_hash(storage: &crate::core::storage::Storage, target: &str) -> crate::error::Result<(crate::core::hash::Hash, Option<String>)> {
    if let Some(h) = storage.refs().get_ref(target)? {
        Ok((h, Some(target.to_string())))
    } else if target.len() == 40 {
        let h = crate::core::hash::Hash::from_hex(target)
            .map_err(|e| crate::error::GikError::Validation(format!("Invalid hash format: {}", e)))?;
        Ok((h, None))
    } else {
        let all_objects = storage.objects().list_all_objects()?;
        let matches: Vec<crate::core::hash::Hash> = all_objects
            .into_iter()
            .filter(|h| h.to_string().starts_with(target))
            .collect();

        if matches.is_empty() {
            return Err(crate::error::GikError::NotFound(format!("Object not found: {}", target)));
        }
        if matches.len() > 1 {
            return Err(crate::error::GikError::AmbiguousHash(target.to_string()));
        }
        Ok((matches[0], None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_resolve_path() {
        let root = Path::new("/repo");
        let cwd1 = Path::new("/repo/src");
        
        assert_eq!(resolve_path(cwd1, root, "main.rs"), "src/main.rs");
        assert_eq!(resolve_path(cwd1, root, "."), "src");
        assert_eq!(resolve_path(root, root, "."), ".");
        assert_eq!(resolve_path(root, root, "src/main.rs"), "src/main.rs");
    }
}

