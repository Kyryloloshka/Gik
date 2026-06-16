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
