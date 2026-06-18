use crate::core::hash::Hash;
use crate::core::storage::Storage;
use crate::error::{GikError, Result};
use std::io::Write;

pub fn cat_file(storage: &Storage, hash_str: &str, p: bool, t: bool, s: bool) -> Result<()> {
    let hash = Hash::from_hex(hash_str)
        .map_err(|e| GikError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

    let compressed_data = storage
        .objects()
        .get_object(&hash)?
        .ok_or_else(|| GikError::NotFound(format!("Object {} not found", hash_str)))?;

    let (obj_type, size, content) = crate::core::objects::decompress_object(&compressed_data[..])?;

    if t {
        println!("{}", obj_type);
    } else if s {
        println!("{}", size);
    } else if p {
        // Just write the uncompressed bytes to stdout
        let mut stdout = std::io::stdout();
        stdout.write_all(&content)?;
        stdout.flush()?;
    } else {
        return Err(GikError::Validation(
            "Must provide one of -p, -t, or -s".to_string(),
        ));
    }

    Ok(())
}
