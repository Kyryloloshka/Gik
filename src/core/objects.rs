use sha1::{Sha1, Digest};
use std::io::{self, Read, Write};
use flate2::write::ZlibEncoder;
use flate2::Compression;

/// Calculates the SHA1 hash of a blob in Git-canonical format: "blob [size]\0[content]"
pub fn hash_blob<R: Read>(mut reader: R, size: u64) -> io::Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    
    // Write header
    let header = format!("blob {}\0", size);
    hasher.update(header.as_bytes());
    
    // Stream content
    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    
    let result = hasher.finalize();
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&result);
    Ok(hash)
}

/// Compresses a blob (including its Git header) using Zlib streaming compression
pub fn compress_blob<R: Read, W: Write>(mut reader: R, size: u64, writer: W) -> io::Result<()> {
    let mut encoder = ZlibEncoder::new(writer, Compression::default());
    
    // Write header
    let header = format!("blob {}\0", size);
    encoder.write_all(header.as_bytes())?;
    
    // Stream content
    io::copy(&mut reader, &mut encoder)?;
    
    encoder.finish()?;
    Ok(())
}

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

/// Calculates the SHA1 hash of a commit in Git-canonical format
pub fn hash_commit(
    tree_hash: [u8; 20],
    parent_hashes: &[[u8; 20]],
    author: &str,
    email: &str,
    timestamp: u64,
    message: &str,
) -> io::Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    let content = build_commit_content(tree_hash, parent_hashes, author, email, timestamp, message);
    
    let header = format!("commit {}\0", content.len());
    hasher.update(header.as_bytes());
    hasher.update(content.as_bytes());
    
    let result = hasher.finalize();
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&result);
    Ok(hash)
}

/// Compresses a commit object using Zlib
pub fn compress_commit<W: Write>(
    tree_hash: [u8; 20],
    parent_hashes: &[[u8; 20]],
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

fn build_commit_content(
    tree_hash: [u8; 20],
    parent_hashes: &[[u8; 20]],
    author: &str,
    email: &str,
    timestamp: u64,
    message: &str,
) -> String {
    let mut content = format!("tree {}\n", hex::encode(tree_hash));
    for parent in parent_hashes {
        content.push_str(&format!("parent {}\n", hex::encode(parent)));
    }
    content.push_str(&format!(
        "author {} <{}> {} +0000\n",
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
    use std::io::Cursor;
    use flate2::read::ZlibDecoder;

    #[test]
    fn test_hash_blob_hello_world() {
        // Git hash-object -t blob --stdin <<< "hello world"
        // (Note: echo "hello world" adds a newline)
        // Let's test with "hello world\n" which is common in tests
        let content = "hello world\n";
        let size = content.len() as u64;
        let reader = Cursor::new(content);
        
        let hash = hash_blob(reader, size).unwrap();
        
        // Expected hash for "blob 12\0hello world\n"
        // git hash-object for "hello world\n" is 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
        let expected_hex = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";
        assert_eq!(hex::encode(hash), expected_hex);
    }

    #[test]
    fn test_hash_blob_git_reference() {
        // Git hash-object for "hello\n" (6 bytes) is ce013625030ba8dba906f756967f9e9ca394464a
        let content = b"hello\n";
        let size = content.len() as u64;
        let reader = Cursor::new(content);

        let hash = hash_blob(reader, size).unwrap();
        let expected_hex = "ce013625030ba8dba906f756967f9e9ca394464a";
        assert_eq!(hex::encode(hash), expected_hex);
    }

    #[test]
    fn test_compress_decompress_blob() {
        let content = "test content for compression";
        let size = content.len() as u64;
        let reader = Cursor::new(content);
        let mut compressed = Vec::new();
        
        compress_blob(reader, size, &mut compressed).unwrap();
        
        // Decompress and verify
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        
        let expected = format!("blob {}\0{}", size, content);
        assert_eq!(decompressed, expected);
    }
}
