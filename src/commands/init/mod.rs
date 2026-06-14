use crate::error::Result;
use crate::core::storage::Storage;

pub fn init(db_path: &str) -> Result<()> {
    Storage::new(db_path)?;
    Ok(())
}

#[cfg(test)]
mod tests;
