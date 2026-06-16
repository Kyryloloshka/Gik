use sha1::{Sha1, Digest};
use std::io::{self, Write};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use crate::core::hash::Hash;

/// Calculates the SHA1 hash of a commit in Git-canonical format
pub fn hash_commit(
    tree_hash: Hash,
    parent_hashes: &[Hash],
    author: &str,
    email: &str,
    timestamp: u64,
    message: &str,
) -> io::Result<Hash> {
    let mut hasher = Sha1::new();
    let content = build_commit_content(tree_hash, parent_hashes, author, email, timestamp, message);

    let header = format!("commit {}\0", content.len());
    hasher.update(header.as_bytes());
    hasher.update(content.as_bytes());

    let result = hasher.finalize();
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&result);
    Ok(Hash(hash))
}

/// Compresses a commit object using Zlib
pub fn compress_commit<W: Write>(
    tree_hash: Hash,
    parent_hashes: &[Hash],
    author: &str,
    email: &str,
    timestamp: u64,
    message: &str,
    writer: W,
) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(writer, Compression::default());
    let content = build_commit_content(tree_hash, parent_hashes, author, email, timestamp, message);

    let header = format!("commit {}\0", content.len());
    encoder.write_all(header.as_bytes())?;
    encoder.write_all(content.as_bytes())?;

    encoder.finish()?;
    Ok(())
}

pub fn build_commit_content(
    tree_hash: Hash,
    parent_hashes: &[Hash],
    author: &str,
    email: &str,
    timestamp: u64,
    message: &str,
) -> String {
    let mut content = format!("tree {}\n", hex::encode(tree_hash.0));
    for parent in parent_hashes {
        content.push_str(&format!("parent {}\n", hex::encode(parent.0)));
    }
    content.push_str(&format!(
        "author {} <{}> {} +0000\n",
        author, email, timestamp
    ));
    content.push_str(&format!(
        "committer {} <{}> {} +0000\n",
        author, email, timestamp
    ));
    content.push('\n');
    content.push_str(message);

    if !message.ends_with('\n') {
        content.push('\n');
    }
    content
}
