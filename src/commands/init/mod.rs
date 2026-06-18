use crate::core::storage::Storage;
use crate::error::Result;

pub fn init(db_path: &str) -> Result<()> {
    Storage::new(db_path)?;
    Ok(())
}

#[cfg(test)]
mod tests;
