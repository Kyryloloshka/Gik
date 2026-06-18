use crate::core::models::FileState;
use crate::core::storage::Storage;
use crate::error::Result;
use colored::*;

/// Show the working tree status
pub fn status(storage: &Storage, porcelain: bool) -> Result<()> {
    let repo_status = crate::core::workspace::get_status(storage)?;

    if porcelain {
        let mut all_files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for k in repo_status.staged.keys() {
            all_files.insert(k.clone());
        }
        for k in repo_status.unstaged.keys() {
            all_files.insert(k.clone());
        }
        for k in &repo_status.untracked {
            all_files.insert(k.clone());
        }

        for file in all_files {
            let mut staged_char = ' ';
            if let Some(s) = repo_status.staged.get(&file) {
                staged_char = match s {
                    FileState::Modified => 'M',
                    FileState::New => 'A',
                    FileState::Deleted => 'D',
                };
            }

            let mut unstaged_char = ' ';
            if let Some(s) = repo_status.unstaged.get(&file) {
                unstaged_char = match s {
                    FileState::Modified => 'M',
                    FileState::New => '?',
                    FileState::Deleted => 'D',
                };
            } else if repo_status.untracked.contains(&file) {
                staged_char = '?';
                unstaged_char = '?';
            }

            println!("{}{}\t{}", staged_char, unstaged_char, file);
        }
        return Ok(());
    }

    let current_bookmark = storage.session().get_current_bookmark()?;

    // Presentation
    if let Some(name) = current_bookmark {
        println!("On bookmark: {}", name.green().bold());
    } else {
        println!("On anonymous commit");
    }

    if repo_status.staged.is_empty()
        && repo_status.unstaged.is_empty()
        && repo_status.untracked.is_empty()
    {
        println!("nothing to commit, working tree clean");
        return Ok(());
    }

    if !repo_status.staged.is_empty() {
        println!("\nChanges to be committed:");
        println!("  (use \"gik unstage <file>...\" to unstage)");

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
