use crate::commands::checkout::checkout;
use crate::core::storage::Storage;
use crate::error::Result;

pub fn clone(url: &str, directory: Option<String>, specified_branch: Option<String>) -> Result<()> {
    println!("Cloning repository...");

    // Core business logic handles fetching, decoding, and setting up the DB
    let (target_dir, branch) =
        crate::core::clone_ops::execute_clone(url, directory, specified_branch)?;

    println!("Fetching objects...");
    println!("Decoding packfile...");
    println!("Updating refs...");

    // The current directory has now been switched to target_dir by execute_clone
    let db_path = crate::config::DB_PATH;
    let storage = Storage::new(db_path)?;

    // Checkout working directory using the command
    checkout(&storage, &branch, true)?;

    println!(
        "Clone successful! Repository checked out into '{}'",
        target_dir
    );
    Ok(())
}
