use crate::error::Result;
use crate::core::storage::Storage;

pub fn log() -> Result<()> {
    let storage = Storage::new(crate::config::DB_PATH)?;
    let mut current_hash = storage.get_current_head()?;

    if current_hash.is_none() {
        println!("No commits yet");
        return Ok(());
    }

    while let Some(hash) = current_hash {
        if let Some(meta) = storage.get_commit_meta(&hash)? {
            println!("commit {}", hex::encode(hash));
            println!("Author: {}", meta.author);
            
            // Format date
            let datetime = chrono::DateTime::from_timestamp(meta.timestamp as i64, 0)
                .map(|dt| dt.format("%a %b %e %H:%M:%S %Y %z").to_string())
                .unwrap_or_else(|| "Unknown date".to_string());
            println!("Date:   {}\n", datetime);
            
            println!("    {}\n", meta.message);

            // Follow the first parent
            current_hash = meta.parent_hashes.first().copied();
        } else {
            break;
        }
    }

    Ok(())
}
