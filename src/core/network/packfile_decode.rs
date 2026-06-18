use std::io::{Read, Write};
use crate::error::{Result, GikError};
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use flate2::bufread::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Sha1, Digest};
use crate::core::models::CommitMeta;
use indicatif::{ProgressBar, ProgressStyle};

struct PendingDelta {
    base_hash: Hash,
    delta_data: Vec<u8>,
}

pub fn decode_packfile<R: Read>(reader_in: &mut R, storage: &Storage) -> Result<()> {
    let mut reader = std::io::BufReader::new(reader_in);
    let mut header = [0u8; 12];
    reader.read_exact(&mut header)?;
    if &header[0..4] != b"PACK" {
        return Err(GikError::Io(std::io::Error::other(format!("Invalid packfile signature: {:?}", String::from_utf8_lossy(&header)))));
    }
    
    let object_count = u32::from_be_bytes(header[8..12].try_into().unwrap());
    let pb = ProgressBar::new(object_count as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_message("Decoding objects...");
    
    let mut pending_deltas = Vec::new();
    
    for _i in 0..object_count {
        pb.inc(1);
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
        
        if obj_type == 7 {
            let mut base_hash_bytes = [0u8; 20];
            reader.read_exact(&mut base_hash_bytes)?;
            let base_hash = Hash(base_hash_bytes);
            
            let mut zlib = ZlibDecoder::new(&mut reader);
            let mut delta_data = Vec::new();
            zlib.read_to_end(&mut delta_data)?;
            
            pending_deltas.push(PendingDelta { base_hash, delta_data });
            continue;
        } else if obj_type == 6 {
            return Err(GikError::Io(std::io::Error::other("OBJ_OFS_DELTA not supported yet")));
        }
        
        let type_str = match obj_type {
            1 => "commit",
            2 => "tree",
            3 => "blob",
            _ => return Err(GikError::Io(std::io::Error::other(format!("Unsupported pack object type: {}", obj_type)))),
        };
        let mut zlib = ZlibDecoder::new(&mut reader);
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
    
    pb.finish_with_message("Decoding completed.");
    
    let pb_deltas = ProgressBar::new(pending_deltas.len() as u64);
    pb_deltas.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.magenta/red}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("#>-"));
    pb_deltas.set_message("Resolving deltas...");
    
    // Resolve deltas
    while !pending_deltas.is_empty() {
        let mut resolved_any = false;
        let mut next_pending = Vec::new();
        
        for delta in pending_deltas {
            if let Ok(Some(compressed_base)) = storage.objects().get_object(&delta.base_hash) {
                let (type_str, _, base_payload) = crate::core::objects::decompress_object(&compressed_base[..])
                    .map_err(|e| GikError::Io(e))?;
                
                let target_payload = apply_delta(&base_payload, &delta.delta_data)?;
                
                let header_str = format!("{} {}\0", type_str, target_payload.len());
                let mut full_obj = header_str.into_bytes();
                full_obj.extend_from_slice(&target_payload);
                
                let mut hasher = Sha1::new();
                hasher.update(&full_obj);
                let hash_bytes: [u8; 20] = hasher.finalize().into();
                let hash = Hash::from(hash_bytes);
                
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&full_obj)?;
                let compressed = encoder.finish()?;
                
                storage.objects().write_object(&hash, &compressed)?;
                
                if type_str == "commit" {
                    let meta = parse_commit_meta(&target_payload)?;
                    storage.commits().insert_commit_meta(&hash, meta)?;
                }
                
                resolved_any = true;
                pb_deltas.inc(1);
            } else {
                next_pending.push(delta);
            }
        }
        
        if !resolved_any && !next_pending.is_empty() {
            return Err(GikError::Io(std::io::Error::other(format!("Missing base object for delta: {}", next_pending[0].base_hash))));
        }
        pb_deltas.set_length((pb_deltas.position() + next_pending.len() as u64) as u64);
        pending_deltas = next_pending;
    }
    
    pb_deltas.finish_with_message("Deltas resolved.");
    Ok(())
}

fn apply_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
    let mut d_idx = 0;
    
    let mut _base_size = 0;
    let mut shift = 0;
    loop {
        if d_idx >= delta.len() { return Err(GikError::Io(std::io::Error::other("Delta truncated"))); }
        let b = delta[d_idx];
        d_idx += 1;
        _base_size |= ((b & 0x7f) as usize) << shift;
        shift += 7;
        if b & 0x80 == 0 { break; }
    }
    
    let mut result_size = 0;
    let mut shift = 0;
    loop {
        if d_idx >= delta.len() { return Err(GikError::Io(std::io::Error::other("Delta truncated"))); }
        let b = delta[d_idx];
        d_idx += 1;
        result_size |= ((b & 0x7f) as usize) << shift;
        shift += 7;
        if b & 0x80 == 0 { break; }
    }
    
    let mut result = Vec::with_capacity(result_size);
    
    while d_idx < delta.len() {
        let cmd = delta[d_idx];
        d_idx += 1;
        
        if cmd & 0x80 != 0 {
            // Copy
            let mut offset = 0;
            let mut size = 0;
            
            if cmd & 0x01 != 0 { offset |= delta[d_idx] as usize; d_idx += 1; }
            if cmd & 0x02 != 0 { offset |= (delta[d_idx] as usize) << 8; d_idx += 1; }
            if cmd & 0x04 != 0 { offset |= (delta[d_idx] as usize) << 16; d_idx += 1; }
            if cmd & 0x08 != 0 { offset |= (delta[d_idx] as usize) << 24; d_idx += 1; }
            
            if cmd & 0x10 != 0 { size |= delta[d_idx] as usize; d_idx += 1; }
            if cmd & 0x20 != 0 { size |= (delta[d_idx] as usize) << 8; d_idx += 1; }
            if cmd & 0x40 != 0 { size |= (delta[d_idx] as usize) << 16; d_idx += 1; }
            
            if size == 0 { size = 0x10000; }
            
            if offset + size > base.len() {
                return Err(GikError::Io(std::io::Error::other(format!("Delta copy out of bounds: offset={}, size={}, base_len={}", offset, size, base.len()))));
            }
            result.extend_from_slice(&base[offset..offset + size]);
        } else if cmd != 0 {
            // Insert
            let size = cmd as usize;
            if d_idx + size > delta.len() {
                return Err(GikError::Io(std::io::Error::other("Delta insert out of bounds")));
            }
            result.extend_from_slice(&delta[d_idx..d_idx + size]);
            d_idx += size;
        } else {
            return Err(GikError::Io(std::io::Error::other("Invalid delta opcode 0")));
        }
    }
    
    if result.len() != result_size {
        return Err(GikError::Io(std::io::Error::other(format!("Delta result size mismatch: expected {}, got {}", result_size, result.len()))));
    }
    
    Ok(result)
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
