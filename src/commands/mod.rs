use crate::error::Result;
use crate::core::storage::Storage;
use std::fs::File;

pub fn init() -> Result<()> {
    Storage::new(".gik.db")?;
    Ok(())
}

pub fn stage(path: String) -> Result<()> {
    let storage = Storage::new(".gik.db")?;
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
    use std::io::Write;

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

    #[test]
    fn test_stage_adds_file_to_storage() {
        let dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        
        init().unwrap();
        
        let file_path = "test.txt";
        let content = "hello world\n";
        {
            let mut f = File::create(file_path).unwrap();
            f.write_all(content.as_bytes()).unwrap();
        }
        
        stage(file_path.to_string()).unwrap();
        
        // Verify it's in the DB
        let storage = Storage::new(".gik.db").unwrap();
        let hash = storage.get_staged_hash(file_path).unwrap();
        assert!(hash.is_some());
        
        // Expected hash for "hello world\n" (see objects.rs tests)
        let expected_hex = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";
        assert_eq!(hex::encode(hash.unwrap()), expected_hex);
        
        // Verify object is stored
        assert!(storage.contains_object(&hash.unwrap()).unwrap());
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
