use std::io::Read;
use crate::error::{Result, GikError};

pub fn read_object_header<R: Read>(reader: &mut R) -> Result<(u8, u64)> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte).map_err(|e| GikError::Io(e))?;
    
    let obj_type = (byte[0] >> 4) & 7;
    let mut size = (byte[0] & 15) as u64;
    let mut shift = 4;
    
    let mut current_byte = byte[0];
    while (current_byte & 0x80) != 0 {
        reader.read_exact(&mut byte).map_err(|e| GikError::Io(e))?;
        current_byte = byte[0];
        size |= ((current_byte & 0x7f) as u64) << shift;
        shift += 7;
    }
    
    Ok((obj_type, size))
}

pub fn apply_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
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
