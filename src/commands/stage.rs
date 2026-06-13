use crate::error::Result;
use crate::core::storage::Storage;
use std::fs::File;

pub fn stage(path: String) -> Result<()> {
    let storage = Storage::new(crate::config::DB_PATH)?;
    let file = File::open(&path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();

    // Hash
    let hash = crate::core::objects::hash_blob(&file, size)?;

    // Re-open for compression
    let file = File::open(&path)?;
    storage.stage_file(&path, &hash, size, file)?;

    Ok(())
}
