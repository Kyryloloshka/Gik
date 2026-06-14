pub mod blob;
pub mod tree;
pub mod commit;

pub use blob::*;
pub use tree::*;
pub use commit::*;

use flate2::read::ZlibDecoder;
use std::io::{Read, self};

/// Decompresses an object and strips the Git header.
/// Returns (type, size, content)
pub fn decompress_object<R: Read>(reader: R) -> io::Result<(String, u64, Vec<u8>)> {
    let mut decoder = ZlibDecoder::new(reader);
    let mut content = Vec::new();
    decoder.read_to_end(&mut content)?;

    // Parse header: "[type] [size]\0"
    let null_pos = content.iter().position(|&b| b == 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing null terminator in header"))?;
    
    let header = std::str::from_utf8(&content[..null_pos])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in header"))?;
    
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid header format"));
    }

    let obj_type = parts[0].to_string();
    let size = parts[1].parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid size in header"))?;

    let actual_content = content[null_pos + 1..].to_vec();
    Ok((obj_type, size, actual_content))
}

#[cfg(test)]
mod tests;
