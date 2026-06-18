use crate::core::hash::Hash;
use crate::core::network::client::GitClient;
use crate::core::network::packfile_decode::decode_packfile;
use crate::core::storage::Storage;
use crate::error::{GikError, Result};

/// Fetches a specific branch from the remote configured in storage.
/// Decodes the packfile and updates the local ref.
/// Returns the remote head hash if the branch was found.
pub fn fetch_remote_branch(
    storage: &Storage,
    branch: &str,
    local_head: Option<&Hash>,
) -> Result<Option<Hash>> {
    let url = storage.config().get("remote.origin.url")?.ok_or_else(|| {
        GikError::Config("No remote configured. Use 'gik config remote.origin.url <url>'".to_string())
    })?;

    let _ = dotenvy::dotenv();
    let token = std::env::var("GITHUB_TOKEN").ok();
    let client = GitClient::new(url, token);

    if let Some(remote_head) = client.discover_fetch_refs(branch)? {
        if local_head == Some(&remote_head) {
            return Ok(Some(remote_head));
        }

        let mut reader = client.fetch_packfile(&remote_head, local_head)?;
        decode_packfile(&mut reader, storage)?;

        storage.refs().set_ref(branch, &remote_head)?;
        Ok(Some(remote_head))
    } else {
        Ok(None)
    }
}
