use sha1::{Sha1, Digest};
use std::io::{self, Read, Write};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use crate::core::hash::Hash;

/// Calculates the SHA1 hash of a blob in Git-canonical format: "blob [size]\0[content]"
pub fn hash_blob<R: Read>(mut reader: R, size: u64) -> io::Result<Hash> {
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
    Ok(Hash(hash))
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
