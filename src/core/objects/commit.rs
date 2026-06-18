use crate::core::hash::Hash;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::{self, Write};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_commit_content_single_parent() {
        let tree_hash = Hash([1; 20]);
        let parent = Hash([2; 20]);

        let content = build_commit_content(
            tree_hash,
            &[parent],
            "Linus Torvalds",
            "torvalds@linux-foundation.org",
            1718500000,
            "Initial release",
        );

        let expected = format!(
            "tree {}\nparent {}\nauthor Linus Torvalds <torvalds@linux-foundation.org> 1718500000 +0000\ncommitter Linus Torvalds <torvalds@linux-foundation.org> 1718500000 +0000\n\nInitial release\n",
            hex::encode([1; 20]),
            hex::encode([2; 20])
        );

        assert_eq!(content, expected);
    }

    #[test]
    fn test_build_commit_content_merge_commit() {
        let tree_hash = Hash([1; 20]);
        let parent1 = Hash([2; 20]);
        let parent2 = Hash([3; 20]);

        let content = build_commit_content(
            tree_hash,
            &[parent1, parent2],
            "John Doe",
            "john@example.com",
            1718500000,
            "Merge branch 'feature'\n\nCloses #123",
        );

        let expected = format!(
            "tree {}\nparent {}\nparent {}\nauthor John Doe <john@example.com> 1718500000 +0000\ncommitter John Doe <john@example.com> 1718500000 +0000\n\nMerge branch 'feature'\n\nCloses #123\n",
            hex::encode([1; 20]),
            hex::encode([2; 20]),
            hex::encode([3; 20])
        );

        assert_eq!(content, expected);
    }
}
