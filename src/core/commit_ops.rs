use crate::core::hash::Hash;
use crate::core::storage::Storage;
use crate::error::{GikError, Result};

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn execute_commit(
    storage: &Storage,
    message: String,
    staged_only: bool,
    explicit_branch: Option<String>,
) -> Result<Option<Hash>> {
    if !staged_only {
        crate::core::workspace::auto_stage(storage)?;
    }

    if staged_only {
        crate::core::workspace::clean_ignored_from_index(storage)?;
    }

    let staged_files = storage.index().get_all_staged_files()?;
    if staged_files.is_empty() {
        return Ok(None);
    }

    let (tree_hash, _tree_content) =
        crate::core::objects::tree::build_and_store_tree(storage, staged_files)?;

    let parent_hash = storage.commits().get_current_head()?;
    let mut parent_hashes = if let Some(p) = parent_hash {
        vec![p]
    } else {
        vec![]
    };

    if let Some(merge_head) = storage.session().get_merge_head()? {
        parent_hashes.push(merge_head);
        storage.session().clear_merge_head()?;
    }

    let author_name = storage.config().get("user.name")?;
    let author_email = storage.config().get("user.email")?;

    let (name, email) = match (author_name, author_email) {
        (Some(n), Some(e)) => (n, e),
        _ => {
            return Err(GikError::Validation(
                "Author identity unknown.

Please tell me who you are.

Run:
  gik config --global user.email \"you@example.com\"
  gik config --global user.name \"Your Name\"

Or import from git:
  gik config --import-git
"
                .to_string(),
            ))
        }
    };

    let author = format!("{} <{}>", name, email);
    let timestamp = current_timestamp();

    let commit_hash = crate::core::objects::hash_commit(
        tree_hash,
        &parent_hashes,
        &name,
        &email,
        timestamp,
        &message,
    )?;

    let mut commit_content = Vec::new();
    crate::core::objects::compress_commit(
        tree_hash,
        &parent_hashes,
        &name,
        &email,
        timestamp,
        &message,
        &mut commit_content,
    )?;

    let meta = crate::core::models::CommitMeta {
        parent_hashes: parent_hashes.clone(),
        tree_hash,
        timestamp,
        author,
        message: message.clone(),
    };

    storage
        .objects()
        .write_object(&commit_hash, &commit_content)?;

    storage
        .commits()
        .commit_transaction(commit_hash, parent_hash, meta)?;

    storage.log_action(crate::core::models::UndoAction::RevertCommit {
        old_head: parent_hash,
        new_head: commit_hash,
    });

    resolve_bookmarks(storage, &parent_hash, &commit_hash, explicit_branch)?;

    Ok(Some(commit_hash))
}

pub fn resolve_bookmarks(
    storage: &Storage,
    parent_hash: &Option<Hash>,
    new_hash: &Hash,
    explicit_branch: Option<String>,
) -> Result<()> {
    let refs = storage.refs().list_refs()?;
    let session_hint = storage.session().get_current_bookmark()?;

    if let Some(b) = explicit_branch {
        let old_hash = storage.refs().set_ref(&b, new_hash)?;
        storage.log_action(crate::core::models::UndoAction::MoveBookmark {
            name: b.clone(),
            old_hash,
            new_hash: new_hash.clone(),
        });
        storage.session().set_current_bookmark(&b)?;
        return Ok(());
    }

    if refs.is_empty() {
        let old_hash = storage.refs().set_ref("main", new_hash)?;
        storage.log_action(crate::core::models::UndoAction::MoveBookmark {
            name: "main".to_string(),
            old_hash,
            new_hash: new_hash.clone(),
        });
        storage.session().set_current_bookmark("main")?;
        return Ok(());
    }

    if let Some(parent) = parent_hash {
        let parent_refs: Vec<String> = refs
            .iter()
            .filter(|(_, h)| h == parent)
            .map(|(n, _)| n.clone())
            .collect();

        if parent_refs.is_empty() {
            storage.session().clear_current_bookmark()?;
        } else if parent_refs.len() == 1 {
            let name = &parent_refs[0];
            let old_hash = storage.refs().set_ref(name, new_hash)?;
            storage.log_action(crate::core::models::UndoAction::MoveBookmark {
                name: name.clone(),
                old_hash,
                new_hash: new_hash.clone(),
            });
            storage.session().set_current_bookmark(name)?;
        } else {
            if let Some(hint) = session_hint {
                if parent_refs.contains(&hint) {
                    let old_hash = storage.refs().set_ref(&hint, new_hash)?;
                    storage.log_action(crate::core::models::UndoAction::MoveBookmark {
                        name: hint.clone(),
                        old_hash,
                        new_hash: new_hash.clone(),
                    });
                } else {
                    println!("Warning: Multiple bookmarks found at parent ({}), but none match current session hint ({}). Bookmark left behind.", 
                             parent_refs.join(", "), hint);
                    storage.session().clear_current_bookmark()?;
                }
            } else {
                println!("Warning: Multiple bookmarks found at parent ({}). Please specify which one to move using --branch <name>. Bookmark left behind.", 
                         parent_refs.join(", "));
            }
        }
    }

    Ok(())
}
