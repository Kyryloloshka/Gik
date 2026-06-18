pub mod blob;
pub mod commit;
pub mod tree;

pub use blob::*;
pub use commit::*;
pub use tree::*;

use flate2::read::ZlibDecoder;
use std::io::{self, Read};

/// Decompresses an object and strips the Git header.
/// Returns (type, size, content)
pub fn decompress_object<R: Read>(reader: R) -> io::Result<(String, u64, Vec<u8>)> {
    let mut decoder = ZlibDecoder::new(reader);
    let mut content = Vec::new();
    decoder.read_to_end(&mut content)?;

    // Parse header: "[type] [size]\0"
    let null_pos = content.iter().position(|&b| b == 0).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Missing null terminator in header",
        )
    })?;

    let header = std::str::from_utf8(&content[..null_pos])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in header"))?;

    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid header format",
        ));
    }

    let obj_type = parts[0].to_string();
    let size = parts[1]
        .parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid size in header"))?;

    let actual_content = content[null_pos + 1..].to_vec();
    Ok((obj_type, size, actual_content))
}

pub struct ObjectPayloadReader<R: Read> {
    decoder: ZlibDecoder<R>,
}

impl<R: Read> Read for ObjectPayloadReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.decoder.read(buf)
    }
}

pub fn decompress_object_stream<R: Read>(
    reader: R,
) -> io::Result<(String, u64, ObjectPayloadReader<R>)> {
    let mut decoder = ZlibDecoder::new(reader);
    let mut header_buf = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        let n = decoder.read(&mut byte)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF while reading header",
            ));
        }
        if byte[0] == 0 {
            break;
        }
        header_buf.push(byte[0]);
    }

    let header_str = std::str::from_utf8(&header_buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in header"))?;

    let parts: Vec<&str> = header_str.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid header format",
        ));
    }

    let obj_type = parts[0].to_string();
    let size = parts[1]
        .parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid size in header"))?;

    Ok((obj_type, size, ObjectPayloadReader { decoder }))
}

#[cfg(test)]
mod tests;
