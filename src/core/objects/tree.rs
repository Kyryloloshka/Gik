use sha1::{Sha1, Digest};
use std::io::{self, Write};
use flate2::write::ZlibEncoder;
use flate2::Compression;

/// Calculates the SHA1 hash of a tree in Git-canonical format
pub fn hash_tree(entries: &[(u32, String, [u8; 20])]) -> io::Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    let content = build_tree_content(entries);

    let header = format!("tree {}\0", content.len());
    hasher.update(header.as_bytes());
    hasher.update(&content);

    let result = hasher.finalize();
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&result);
    Ok(hash)
}

/// Compresses a tree object using Zlib
pub fn compress_tree<W: Write>(entries: &[(u32, String, [u8; 20])], writer: W) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(writer, Compression::default());
    let content = build_tree_content(entries);

    let header = format!("tree {}\0", content.len());
    encoder.write_all(header.as_bytes())?;
    encoder.write_all(&content)?;

    encoder.finish()?;
    Ok(())
}

fn build_tree_content(entries: &[(u32, String, [u8; 20])]) -> Vec<u8> {
    let mut content = Vec::new();
    for (mode, name, hash) in entries {
        content.extend_from_slice(format!("{:o} {}\0", mode, name).as_bytes());
        content.extend_from_slice(hash);
    }
    content
}
