use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

pub struct IgnoreMatcher {
    gitignore: Gitignore,
}

impl Default for IgnoreMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl IgnoreMatcher {
    pub fn new() -> Self {
        let mut builder = GitignoreBuilder::new("");

        // Hardcoded defaults
        let _ = builder.add_line(None, crate::config::GIK_DIR_NAME);
        let _ = builder.add_line(None, &format!("{}/**", crate::config::GIK_DIR_NAME));
        let _ = builder.add_line(None, crate::config::GIT_DIR_NAME);
        let _ = builder.add_line(None, &format!("{}/**", crate::config::GIT_DIR_NAME));
        let _ = builder.add_line(None, "*gik_test*");

        // Load from .gik.ignore
        let ignore_path = Path::new(crate::config::IGNORE_FILE_NAME);
        if ignore_path.exists() {
            if let Some(err) = builder.add(ignore_path) {
                eprintln!("Warning: Error parsing {}: {}", crate::config::IGNORE_FILE_NAME, err);
            }
        }

        let gitignore = builder.build().unwrap();

        Self { gitignore }
    }

    pub fn is_ignored(&self, path: &str) -> bool {
        // We'll pass is_dir=false as a heuristic.
        self.gitignore.matched(path, false).is_ignore()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_default_ignores() {
        let matcher = IgnoreMatcher::new();
        
        assert!(matcher.is_ignored(".gik/db"));
        assert!(matcher.is_ignored(".gik/objects/12"));
        assert!(matcher.is_ignored(".git"));
        assert!(matcher.is_ignored(".git/config"));
        assert!(!matcher.is_ignored("README.md"));
        assert!(!matcher.is_ignored(".github/workflows"));
    }

    #[test]
    fn test_custom_ignore_file() {
        let dir = tempdir().unwrap();
        let ignore_path = dir.path().join(crate::config::IGNORE_FILE_NAME);
        {
            let mut file = fs::File::create(&ignore_path).unwrap();
            file.write_all(b"target\n*.tmp\n# comment\n\n").unwrap();
        }

        // We need to be in the same dir for matcher.new() to see the file
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let matcher = IgnoreMatcher::new();
        assert!(matcher.is_ignored("target"));
        assert!(matcher.is_ignored("test.tmp"));
        assert!(!matcher.is_ignored("src/main.rs"));

        std::env::set_current_dir(original_dir).unwrap();
    }
}
