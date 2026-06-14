use glob::Pattern;
use std::fs;

pub struct IgnoreMatcher {
    patterns: Vec<Pattern>,
}

impl IgnoreMatcher {
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        // Hardcoded defaults
        // Using more specific patterns to avoid over-matching (like .github)
        patterns.push(Pattern::new(crate::config::DB_PATH).unwrap());
        let db_glob = format!("{}/**", crate::config::DB_PATH);
        patterns.push(Pattern::new(&db_glob).unwrap());
        patterns.push(Pattern::new(".git").unwrap());
        patterns.push(Pattern::new(".git/**").unwrap());
        patterns.push(Pattern::new("*gik_test*").unwrap()); // For our tests


        // Load from .gik.ignore

        if let Ok(content) = fs::read_to_string(".gik.ignore") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Ok(pattern) = Pattern::new(trimmed) {
                    patterns.push(pattern);
                }
            }
        }

        Self { patterns }
    }

    pub fn is_ignored(&self, path: &str) -> bool {
        // Normalize path for matching (simple string matching for now)
        for pattern in &self.patterns {
            if pattern.matches(path) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_default_ignores() {
        let matcher = IgnoreMatcher::new();
        
        assert!(matcher.is_ignored(".gik.db"));
        assert!(matcher.is_ignored(".git"));
        assert!(matcher.is_ignored(".git/config"));
        assert!(!matcher.is_ignored("README.md"));
        assert!(!matcher.is_ignored(".github/workflows"));
    }

    #[test]
    fn test_custom_ignore_file() {
        let dir = tempdir().unwrap();
        let ignore_path = dir.path().join(".gik.ignore");
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
