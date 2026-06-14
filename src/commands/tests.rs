use super::*;
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

const HELLO_CONTENT: &str = "hello world\n";
const HELLO_HASH: &str = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";

#[test]
fn test_init_creates_db_file() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    let result = init(db_path_str);

    assert!(result.is_ok());
    assert!(db_path.exists());
}

#[test]
fn test_stage_adds_file_to_storage() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    let file_path = dir.path().join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();
    stage(&storage, file_path_str.clone()).unwrap();

    let hash_option = storage.index().get_staged_hash(&file_path_str).unwrap();
    assert!(hash_option.is_some());

    let hash = hash_option.unwrap();
    assert_eq!(hex::encode(hash.0), HELLO_HASH);

    assert!(storage.objects().contains_object(&hash).unwrap());
}

#[test]
fn test_commit_creates_objects_and_updates_head() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    let file_path = dir.path().join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();
    stage(&storage, file_path_str).unwrap();
    commit(&storage, "initial commit".to_string(), true).unwrap();

    let first_head_option = {
        let head = storage.commits().get_current_head().unwrap();
        assert!(head.is_some());

        let staged = storage.index().get_all_staged_files().unwrap();
        assert_eq!(staged.len(), 1); // Index is now persistent

        assert!(storage.objects().contains_object(&head.unwrap()).unwrap());
        head
    };

    let file_path2 = dir.path().join("test2.txt");
    {
        let mut file = File::create(&file_path2).unwrap();
        file.write_all(b"second file\n").unwrap();
    }
    let file_path2_str = file_path2.to_str().unwrap().to_string();
    stage(&storage, file_path2_str).unwrap();
    commit(&storage, "second commit".to_string(), true).unwrap();

    let head2_option = storage.commits().get_current_head().unwrap();
    assert!(head2_option.is_some());
    assert_ne!(first_head_option, head2_option);
}

#[test]
fn test_log_runs_successfully() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    // No commits yet
    assert!(log(&storage).is_ok());

    let file_path = dir.path().join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();
    stage(&storage, file_path_str).unwrap();
    commit(&storage, "initial commit".to_string(), true).unwrap();

    // One commit
    assert!(log(&storage).is_ok());
}

#[test]
fn test_undo_works() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("gik_test.db");
    let db_path_str = db_path.to_str().unwrap();

    init(db_path_str).unwrap();
    let storage = Storage::new(db_path_str).unwrap();

    let file_path = dir.path().join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    let file_path_str = file_path.to_str().unwrap().to_string();

    // Undo staging
    stage(&storage, file_path_str.clone()).unwrap();
    assert!(storage.index().get_staged_hash(&file_path_str).unwrap().is_some());
    undo(&storage).unwrap();
    assert!(storage.index().get_staged_hash(&file_path_str).unwrap().is_none());

    // Undo commit
    stage(&storage, file_path_str).unwrap();
    commit(&storage, "initial commit".to_string(), true).unwrap();
    let first_head = storage.commits().get_current_head().unwrap();
    assert!(first_head.is_some());

    undo(&storage).unwrap();
    assert!(storage.commits().get_current_head().unwrap().is_none());
}

#[test]
fn test_commit_auto_stages_files() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    // Call commit WITHOUT staging manually
    commit(&storage, "auto commit".to_string(), false).unwrap();

    // Verify file is indeed in the index (it was auto-staged and remains there)

    let staged_files = storage.index().get_all_staged_files().unwrap();
    assert_eq!(staged_files.len(), 1);

    let expected_blob_hash = Hash::from_hex(HELLO_HASH).unwrap();
    assert!(storage.objects().contains_object(&expected_blob_hash).unwrap());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_ignore_system_removes_from_index() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "ignored_file.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"to be ignored\n").unwrap();
    }

    // 1. Stage the file manually
    stage(&storage, file_path.to_string()).unwrap();
    assert!(storage.index().get_staged_hash(file_path).unwrap().is_some());

    // 2. Add to .gik.ignore
    {
        let mut ignore_file = File::create(".gik.ignore").unwrap();
        ignore_file.write_all(b"ignored_file.txt\n").unwrap();
    }

    // 3. Run commit. It should auto-remove the file from index.
    commit(&storage, "commit with ignore".to_string(), true).unwrap();

    // 4. Verify index is empty (because the only file was removed)
    let staged = storage.index().get_all_staged_files().unwrap();
    assert!(staged.is_empty());

    // 5. Verify HEAD is still empty (because nothing was committed)
    assert!(storage.commits().get_current_head().unwrap().is_none());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_recursive_tree_generation() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // Create nested structure
    let subdir_name = "subdir";
    let nested_dir = dir.path().join(subdir_name);
    std::fs::create_dir(&nested_dir).unwrap();
    let file_path = nested_dir.join("test.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"nested content\n").unwrap();
    }

    // Commit everything
    commit(&storage, "recursive commit".to_string(), false).unwrap();

    let head = storage.commits().get_current_head().unwrap().unwrap();
    let commit_meta = storage.commits().get_commit_meta(&head).unwrap().unwrap();
    let root_tree_hash = commit_meta.tree_hash;

    // Verify root tree is stored
    assert!(storage.objects().contains_object(&root_tree_hash).unwrap());

    // Total objects should be: 
    // 1 blob (test.txt)
    // 1 tree (subdir)
    // 1 tree (root)
    // 1 commit
    // Total = 4
    let all_objects = storage.objects().list_all_objects().unwrap();
    assert_eq!(all_objects.len(), 4);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_status_basic() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    // 1. Clean state
    assert!(status(&storage).is_ok());

    // 2. Untracked file
    let file_path = "untracked.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"untracked content\n").unwrap();
    }
    assert!(status(&storage).is_ok());

    // 3. Staged file
    stage(&storage, file_path.to_string()).unwrap();
    assert!(status(&storage).is_ok());

    // 4. Unstaged (Modified) file
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"modified untracked content\n").unwrap();
    }
    assert!(status(&storage).is_ok());

    // 5. Committed state
    commit(&storage, "commit file".to_string(), true).unwrap();
    assert!(status(&storage).is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}

