use crate::error::Result;
use crate::core::storage::Storage;

pub fn init() -> Result<()> {
    Storage::new(".gik.db")?;
    Ok(())
}

pub fn stage(_path: String) -> Result<()> {
    // Logic for gik stage will go here
    Ok(())
}

pub fn commit(_message: String) -> Result<()> {
    // Logic for gik commit will go here
    Ok(())
}

pub fn log() -> Result<()> {
    // Logic for gik log will go here
    Ok(())
}

pub fn undo() -> Result<()> {
    // Logic for gik undo will go here
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_creates_db_file() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join(".gik.db");
        
        // We need to change the current directory for the test 
        // since init uses a hardcoded path ".gik.db"
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        
        let result = init();
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
        
        assert!(result.is_ok());
        assert!(db_path.exists());
    }
}
