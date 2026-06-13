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
