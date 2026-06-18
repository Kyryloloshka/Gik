use crate::core::hash::Hash;
use sha1::{Digest, Sha1};
use std::io::{self, Read};

/// Calculates the SHA1 hash of a blob in Git-canonical format: "blob [size]\0[content]"
pub fn hash_blob<R: Read>(mut reader: R, size: u64) -> io::Result<Hash> {
    let mut hasher = Sha1::new();

    // Write header
    let header = format!("blob {}\0", size);
    hasher.update(header.as_bytes());

    // Stream content
    let mut buffer = vec![0; crate::config::IO_BUFFER_SIZE];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&result);
    Ok(Hash(hash))
}

/// Compresses a blob (including its Git header) using Zlib streaming compression directly into Storage
pub fn compress_blob<R: std::io::Read>(
    mut reader: R,
    size: u64,
    hash: &Hash,
    storage: &crate::core::storage::Storage,
) -> crate::error::Result<()> {
    storage.objects().write_object_with_writer(hash, |file| {
        let mut encoder = flate2::write::ZlibEncoder::new(file, flate2::Compression::default());

        let header = format!("blob {}\0", size);
        std::io::Write::write_all(&mut encoder, header.as_bytes())
            .map_err(|e| crate::error::GikError::Io(e))?;

        std::io::copy(&mut reader, &mut encoder).map_err(|e| crate::error::GikError::Io(e))?;

        encoder
            .finish()
            .map_err(|e| crate::error::GikError::Io(e))?;
        Ok(())
    })
}
