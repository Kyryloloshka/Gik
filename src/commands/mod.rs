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

pub fn commit(message: String) -> Result<()> {
    let storage = Storage::new(".gik.db")?;
    
    // 1. Get staged files
    let staged_files = storage.get_all_staged_files()?;
    if staged_files.is_empty() {
        println!("Nothing to commit");
        return Ok(());
    }

    // 2. Create Tree object
    let mut tree_entries = Vec::new();
    for (path, hash) in staged_files {
        // Git mode 100644 for regular files
        tree_entries.push((0o100644, path, hash));
    }
    // Sort entries by name for canonical tree
    tree_entries.sort_by(|a, b| a.1.cmp(&b.1));
    
    let tree_hash = crate::core::objects::hash_tree(&tree_entries)?;
    let mut tree_content = Vec::new();
    crate::core::objects::compress_tree(&tree_entries, &mut tree_content)?;

    // 3. Get current HEAD (parent)
    let parent_hash = storage.get_current_head()?;
    let parent_hashes = if let Some(p) = parent_hash {
        vec![p]
    } else {
        vec![]
    };

    // 4. Create Commit object
    let author = "Gik User";
    let email = "user@gik.local";
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let commit_hash = crate::core::objects::hash_commit(
        tree_hash,
        &parent_hashes,
        author,
        email,
        timestamp,
        &message,
    )?;
    let mut commit_content = Vec::new();
    crate::core::objects::compress_commit(
        tree_hash,
        &parent_hashes,
        author,
        email,
        timestamp,
        &message,
        &mut commit_content,
    )?;

    // 5. Update Storage
    storage.commit_transaction(
        tree_hash,
        tree_content,
        commit_hash,
        commit_content,
        parent_hash,
    )?;

    println!("[main {}] {}", hex::encode(&commit_hash)[..7].to_string(), message);
    
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

    #[test]
    fn test_commit_creates_objects_and_updates_head() {
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
        commit("initial commit".to_string()).unwrap();
        
        let first_head = {
            let storage = Storage::new(".gik.db").unwrap();
            
            // Verify HEAD is updated
            let head = storage.get_current_head().unwrap();
            assert!(head.is_some());
            
            // Verify staged index is cleared
            let staged = storage.get_all_staged_files().unwrap();
            assert!(staged.is_empty());
            
            // Verify commit object exists
            assert!(storage.contains_object(&head.unwrap()).unwrap());
            head
        };
        
        // Second commit
        let file_path2 = "test2.txt";
        {
            let mut f = File::create(file_path2).unwrap();
            f.write_all(b"second file\n").unwrap();
        }
        stage(file_path2.to_string()).unwrap();
        commit("second commit".to_string()).unwrap();
        
        let head2 = {
            let storage = Storage::new(".gik.db").unwrap();
            storage.get_current_head().unwrap()
        };
        assert!(head2.is_some());
        assert_ne!(first_head, head2);
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
