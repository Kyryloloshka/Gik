use std::io::{Read, Write};
use crate::error::{Result, GikError};
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Sha1, Digest};
use crate::core::models::CommitMeta;

struct ByteReader<'a, R: Read> {
    inner: &'a mut R,
}

impl<'a, R: Read> Read for ByteReader<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() { return Ok(0); }
        self.inner.read(&mut buf[0..1])
    }
}

pub fn decode_packfile<R: Read>(reader: &mut R, storage: &Storage) -> Result<()> {
    let mut header = [0u8; 12];
    reader.read_exact(&mut header)?;
    if &header[0..4] != b"PACK" {
        return Err(GikError::Io(std::io::Error::other(format!("Invalid packfile signature: {:?}", String::from_utf8_lossy(&header)))));
    }
    
    let object_count = u32::from_be_bytes(header[8..12].try_into().unwrap());
    println!("Packfile contains {} objects", object_count);
    
    for _i in 0..object_count {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        
        let obj_type = (byte[0] >> 4) & 7;
        let mut _size = (byte[0] & 15) as u64;
        let mut shift = 4;
        
        let mut current_byte = byte[0];
        while (current_byte & 0x80) != 0 {
            reader.read_exact(&mut byte)?;
            current_byte = byte[0];
            _size |= ((current_byte & 0x7f) as u64) << shift;
            shift += 7;
        }
        
        let type_str = match obj_type {
            1 => "commit",
            2 => "tree",
            3 => "blob",
            _ => return Err(GikError::Io(std::io::Error::other(format!("Unsupported pack object type: {}", obj_type)))),
        };
        let mut zlib = ZlibDecoder::new(ByteReader { inner: reader.by_ref() });
        let mut decompressed = Vec::new();
        zlib.read_to_end(&mut decompressed)?;
        
        // Construct git object header + data
        let header_str = format!("{} {}\0", type_str, decompressed.len());
        let mut full_obj = header_str.into_bytes();
        full_obj.extend_from_slice(&decompressed);
        
        // Hash it
        let mut hasher = Sha1::new();
        hasher.update(&full_obj);
        let hash_bytes: [u8; 20] = hasher.finalize().into();
        let hash = Hash::from(hash_bytes);
        
        // Compress it for our storage
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_obj)?;
        let compressed = encoder.finish()?;
        
        storage.objects().write_object(&hash, &compressed)?;
        
        if obj_type == 1 {
            let meta = parse_commit_meta(&decompressed)?;
            storage.commits().insert_commit_meta(&hash, meta)?;
        }
    }
    
    Ok(())
}

fn parse_commit_meta(content: &[u8]) -> Result<CommitMeta> {
    let text = String::from_utf8_lossy(content);
    let mut tree_hash = Hash([0; 20]);
    let mut parent_hashes = Vec::new();
    let mut author = String::new();
    let mut timestamp = 0;
    
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.is_empty() {
            break;
        }
        if let Some(rest) = line.strip_prefix("tree ") {
            tree_hash = Hash::from_hex(rest).map_err(|e| GikError::Io(std::io::Error::other(e)))?;
        } else if let Some(rest) = line.strip_prefix("parent ") {
            parent_hashes.push(Hash::from_hex(rest).map_err(|e| GikError::Io(std::io::Error::other(e)))?);
        } else if let Some(rest) = line.strip_prefix("author ") {
            // author Name <email> 1234567890 +0000
            if let Some(tz_idx) = rest.rfind(" +") {
                if let Some(ts_idx) = rest[..tz_idx].rfind(' ') {
                    let ts_str = &rest[ts_idx + 1..tz_idx];
                    timestamp = ts_str.parse().unwrap_or(0);
                    author = rest[..ts_idx].to_string();
                }
            }
            if author.is_empty() { author = rest.to_string(); }
        }
    }
    
    let mut message = String::new();
    for line in lines {
        message.push_str(line);
        message.push('\n');
    }
    if message.ends_with('\n') {
        message.pop();
    }
    
    Ok(CommitMeta {
        tree_hash,
        parent_hashes,
        author,
        timestamp,
        message,
    })
}
