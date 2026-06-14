use crate::core::storage::Storage;
use crate::error::Result;

/// Show the working tree status
pub fn status(_storage: &Storage) -> Result<()> {
    println!("On branch main");
    println!("nothing to commit, working tree clean (placeholder)");
    Ok(())
}
