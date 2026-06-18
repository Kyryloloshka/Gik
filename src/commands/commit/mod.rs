use crate::error::Result;
use crate::core::storage::Storage;

pub fn commit(storage: &Storage, message: String, staged: bool, explicit_branch: Option<String>) -> Result<()> {
    if let Some(commit_hash) = crate::core::commit_ops::execute_commit(storage, message.clone(), staged, explicit_branch)? {
        println!("[main {}] {}", &hex::encode(commit_hash.0)[..7], message);
        storage.commit_batch(crate::core::models::CommandType::Commit, &format!("gik commit -m '{}'", message))?;
    } else {
        println!("Nothing to commit");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
