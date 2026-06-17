use std::io::{self, Write};
use sha1::{Sha1, Digest};

pub fn write_packfile_header<W: Write>(mut writer: W, object_count: u32) -> io::Result<Sha1> {
    writer.write_all(b"PACK")?;
    writer.write_all(&2u32.to_be_bytes())?;
    writer.write_all(&object_count.to_be_bytes())?;
    
    let mut hasher = Sha1::new();
    hasher.update(b"PACK");
    hasher.update(&2u32.to_be_bytes());
    hasher.update(&object_count.to_be_bytes());
    
    Ok(hasher)
}

pub fn write_object_header<W: Write>(mut writer: W, obj_type: u8, mut size: usize, hasher: &mut Sha1) -> io::Result<()> {
    // obj_type: Commit=1, Tree=2, Blob=3
    let mut byte = (obj_type << 4) | ((size & 0x0F) as u8);
    size >>= 4;
    if size > 0 {
        byte |= 0x80;
    }
    writer.write_all(&[byte])?;
    hasher.update(&[byte]);

    while size > 0 {
        let mut byte = (size & 0x7F) as u8;
        size >>= 7;
        if size > 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        hasher.update(&[byte]);
    }
    Ok(())
}

pub fn build_packfile(storage: &crate::core::storage::Storage, missing: Vec<crate::core::hash::Hash>) -> crate::error::Result<std::fs::File> {
    let mut dummy_hasher = Sha1::new();
    let mut temp_pack = tempfile::tempfile().map_err(|e| crate::error::GikError::Io(e))?;
    let _h = write_packfile_header(&mut temp_pack, missing.len() as u32)?;
    
    for hash in missing {
        let (obj_type, size, content) = if let Some(meta) = storage.commits().get_commit_meta(&hash)? {
            let (author_name, author_email) = if let Some(open) = meta.author.find('<') {
                if let Some(close) = meta.author.find('>') {
                    (meta.author[..open].trim(), &meta.author[open+1..close])
                } else {
                    (meta.author.as_str(), "")
                }
            } else {
                (meta.author.as_str(), "")
            };
            
            let payload = crate::core::objects::commit::build_commit_content(
                meta.tree_hash,
                &meta.parent_hashes,
                author_name,
                author_email,
                meta.timestamp,
                &meta.message,
            ).into_bytes();
            (1u8, payload.len(), payload)
        } else if let Some(compressed) = storage.objects().get_object(&hash)? {
            let (type_str, size, payload) = crate::core::objects::decompress_object(&compressed[..])?;
            let type_id = match type_str.as_str() {
                "tree" => 2u8,
                "blob" => 3u8,
                _ => return Err(crate::error::GikError::Validation("Unknown object type in storage".to_string())),
            };
            (type_id, size as usize, payload)
        } else {
            return Err(crate::error::GikError::NotFound(format!("Missing object {}", hash)));
        };
        
        write_object_header(&mut temp_pack, obj_type, size, &mut dummy_hasher)?;
        
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&content).map_err(|e| crate::error::GikError::Io(e))?;
        let zlibbed = encoder.finish().map_err(|e| crate::error::GikError::Io(e))?;
        temp_pack.write_all(&zlibbed).map_err(|e| crate::error::GikError::Io(e))?;
    }
    
    // Compute checksum
    use std::io::{Read, Seek, SeekFrom};
    temp_pack.seek(SeekFrom::Start(0)).map_err(|e| crate::error::GikError::Io(e))?;
    let mut real_hasher = Sha1::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = temp_pack.read(&mut buffer).map_err(|e| crate::error::GikError::Io(e))?;
        if n == 0 { break; }
        real_hasher.update(&buffer[..n]);
    }
    let checksum = real_hasher.finalize();
    temp_pack.seek(SeekFrom::End(0)).map_err(|e| crate::error::GikError::Io(e))?;
    temp_pack.write_all(&checksum).map_err(|e| crate::error::GikError::Io(e))?;
    
    // Reset cursor to start for sending
    temp_pack.seek(SeekFrom::Start(0)).map_err(|e| crate::error::GikError::Io(e))?;
    Ok(temp_pack)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_object_header() {
        let mut buf = Vec::new();
        let mut hasher = Sha1::new();
        // type 3 (blob), size 14
        write_object_header(&mut buf, 3, 14, &mut hasher).unwrap();
        assert_eq!(buf, vec![0x3e]); // 0011 1110 -> type 3 (0011), size 14 (1110)
    }
}
