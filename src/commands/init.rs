use crate::error::Result;
use crate::core::storage::Storage;

pub fn init() -> Result<()> {
    Storage::new(crate::config::DB_PATH)?;
    Ok(())
}
