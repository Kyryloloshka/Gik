use crate::error::{Result, GikError};
use crate::core::storage::Storage;
use crate::core::network::client::GitClient;
use crate::core::network::packfile_decode::decode_packfile;
use std::path::Path;
use std::fs;

/// Executes the core logic of cloning a repository.
/// Creates the directory, initializes the storage, configures the remote,
/// fetches the packfile, and updates the local refs.
/// Returns the name of the directory cloned into and the target branch name.
pub fn execute_clone(url: &str, directory: Option<String>) -> Result<(String, String)> {
    let (clean_url, token_from_url) = parse_url(url);
    
    let target_dir_name = match directory {
        Some(dir) => dir,
        None => extract_repo_name(&clean_url)?,
    };
    
    let target_path = Path::new(&target_dir_name);
    
    // Validation: directory must not exist or be empty
    if target_path.exists() {
        if target_path.is_file() {
            return Err(GikError::Io(std::io::Error::other("Target path is a file.")));
        }
        if fs::read_dir(target_path)?.next().is_some() {
            return Err(GikError::Io(std::io::Error::other("Target directory is not empty.")));
        }
    } else {
        fs::create_dir_all(target_path)?;
    }
    
    // Switch to the new directory
    std::env::set_current_dir(target_path)?;
    
    let db_path = Path::new(".gik");
    let storage = Storage::new(db_path.to_str().unwrap())?;
    
    storage.config().set_local("remote.origin.url", &clean_url)?;
    
    let _ = dotenvy::dotenv(); // might not exist in empty dir, but safe
    let token = token_from_url.or_else(|| std::env::var("GITHUB_TOKEN").ok());
    
    let client = GitClient::new(clean_url.clone(), token);
    
    let branch = "main";
    let remote_head = match client.discover_fetch_refs(branch) {
        Ok(Some(hash)) => hash,
        Ok(None) => return Err(GikError::Branch(format!("Remote branch '{}' not found. Empty repository?", branch))),
        Err(e) => return Err(e),
    };
    
    let mut reader = client.fetch_packfile(&remote_head, None)?;
    decode_packfile(&mut reader, &storage)?;
    
    storage.refs().set_ref(branch, &remote_head)?;
    storage.session().set_current_bookmark(branch)?;
    
    Ok((target_dir_name, branch.to_string()))
}

/// Parses the URL and extracts credentials if they exist.
/// Returns (clean_url, Option<token>)
pub fn parse_url(url: &str) -> (String, Option<String>) {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return (url.to_string(), None);
    }
    
    let parts: Vec<&str> = url.splitn(3, "://").collect();
    if parts.len() != 2 {
        return (url.to_string(), None);
    }
    
    let protocol = parts[0];
    let rest = parts[1];
    
    if let Some((auth, host_path)) = rest.split_once('@') {
        let clean_url = format!("{}://{}", protocol, host_path);
        (clean_url, Some(auth.to_string()))
    } else {
        (url.to_string(), None)
    }
}

pub fn extract_repo_name(url: &str) -> Result<String> {
    let url_trimmed = url.trim_end_matches('/');
    if let Some(last_slash) = url_trimmed.rfind('/') {
        let mut name = &url_trimmed[last_slash + 1..];
        if name.ends_with(".git") {
            name = &name[..name.len() - 4];
        }
        if !name.is_empty() {
            return Ok(name.to_string());
        }
    }
    Err(GikError::Branch("Could not extract repository name from URL".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let (clean, token) = parse_url("https://ghp_1234@github.com/Kyrylo/gik.git");
        assert_eq!(clean, "https://github.com/Kyrylo/gik.git");
        assert_eq!(token, Some("ghp_1234".to_string()));

        let (clean, token) = parse_url("https://github.com/Kyrylo/gik.git");
        assert_eq!(clean, "https://github.com/Kyrylo/gik.git");
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(extract_repo_name("https://github.com/Kyrylo/gik.git").unwrap(), "gik");
        assert_eq!(extract_repo_name("https://github.com/Kyrylo/gik").unwrap(), "gik");
        assert_eq!(extract_repo_name("https://github.com/Kyrylo/gik/").unwrap(), "gik");
    }
}
