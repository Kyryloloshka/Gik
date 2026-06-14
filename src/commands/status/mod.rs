use crate::core::storage::Storage;
use crate::error::Result;
use crate::core::models::FileState;
use colored::*;

/// Show the working tree status
pub fn status(storage: &Storage) -> Result<()> {
    let repo_status = crate::core::workspace::get_status(storage)?;
    let current_bookmark = storage.session().get_current_bookmark()?;

    // Presentation
    if let Some(name) = current_bookmark {
        println!("On bookmark: {}", name.green().bold());
    } else {
        println!("On anonymous commit");
    }

    if repo_status.staged.is_empty() && repo_status.unstaged.is_empty() && repo_status.untracked.is_empty() {
        println!("nothing to commit, working tree clean");
        return Ok(());
    }

    if !repo_status.staged.is_empty() {
        println!("\nChanges to be committed:");
        println!("  (use \"gik undo\" to unstage)");
        
        let mut staged_v: Vec<_> = repo_status.staged.iter().collect();
        staged_v.sort_by(|a, b| a.0.cmp(b.0));
        
        for (path, state) in staged_v {
            let label = match state {
                FileState::New => "new file:",
                FileState::Modified => "modified:",
                FileState::Deleted => "deleted: ",
            };
            println!("\t{}", format!("{}   {}", label, path).green());
        }
    }

    if !repo_status.unstaged.is_empty() {
        println!("\nChanges not staged for commit:");
        println!("  (use \"gik stage <file>...\" to update what will be committed)");
        
        let mut unstaged_v: Vec<_> = repo_status.unstaged.iter().collect();
        unstaged_v.sort_by(|a, b| a.0.cmp(b.0));

        for (path, state) in unstaged_v {
            let label = match state {
                FileState::New => "new file:", // Should not happen
                FileState::Modified => "modified:",
                FileState::Deleted => "deleted: ",
            };
            println!("\t{}", format!("{}   {}", label, path).red());
        }
    }

    if !repo_status.untracked.is_empty() {
        println!("\nUntracked files:");
        println!("  (use \"gik stage <file>...\" to include in what will be committed)");
        for path in &repo_status.untracked {
            println!("\t{}", path.red());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
