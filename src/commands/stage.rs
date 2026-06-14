use crate::error::Result;
use crate::core::storage::Storage;
use std::fs::File;

pub fn stage(storage: &Storage, path: String) -> Result<()> {
    let matcher = crate::core::ignore::IgnoreMatcher::new();
    if matcher.is_ignored(&path) {
        println!("Path '{}' is ignored by .gik.ignore", path);
        return Ok(());
    }

    let hash = {
        let file = File::open(&path)?;
        let metadata = file.metadata()?;
        let size = metadata.len();
        crate::core::objects::hash_blob(&file, size)?
    };

    let metadata = std::fs::metadata(&path)?;
    let size = metadata.len();
    let file = File::open(&path)?;
    storage.index().stage_file(&path, &hash, size, file)?;


    Ok(())
}
