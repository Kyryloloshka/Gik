use std::io::{Read, Write};
use crate::error::{Result, GikError};
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Sha1, Digest};

pub fn decode_packfile<R: Read>(reader: &mut R, storage: &Storage) -> Result<()> {
    let mut header = [0u8; 12];
    reader.read_exact(&mut header)?;
    if &header[0..4] != b"PACK" {
        return Err(GikError::Io(std::io::Error::other("Invalid packfile signature")));
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
        
        let mut zlib = ZlibDecoder::new(reader.by_ref());
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
    }
    
    Ok(())
}
