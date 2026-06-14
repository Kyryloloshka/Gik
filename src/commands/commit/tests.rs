use super::*;
use crate::core::storage::Storage;
use crate::core::hash::Hash;
use crate::commands::test_utils::*;
use tempfile::tempdir;
use std::io::Write;
use std::fs::File;

#[test]
fn test_commit_creates_objects_and_updates_head() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test.db";
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "test.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(HELLO_CONTENT.as_bytes()).unwrap();
    }

    crate::commands::stage(&storage, file_path.to_string()).unwrap();
    commit(&storage, "initial commit".to_string(), true).unwrap();

    let first_head_option = {
        let head = storage.commits().get_current_head().unwrap();
        assert!(head.is_some());

        let staged = storage.index().get_all_staged_files().unwrap();
        assert_eq!(staged.len(), 1); // Index is now persistent

        assert!(storage.objects().contains_object(&head.unwrap()).unwrap());
        head
    };

    let file_path2 = "test2.txt";
    {
        let mut file = File::create(file_path2).unwrap();
        file.write_all(b"second file\n").unwrap();
    }
    crate::commands::stage(&storage, file_path2.to_string()).unwrap();
    commit(&storage, "second commit".to_string(), true).unwrap();

    let head2_option = storage.commits().get_current_head().unwrap();
    assert!(head2_option.is_some());
    assert_ne!(first_head_option, head2_option);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_commit_auto_stages_files() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = ".gik_test.db";
    crate::commands::init(db_path).unwrap();
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
    crate::commands::init(db_path).unwrap();
    let storage = Storage::new(db_path).unwrap();

    let file_path = "ignored_file.txt";
    {
        let mut file = File::create(file_path).unwrap();
        file.write_all(b"to be ignored\n").unwrap();
    }

    // 1. Stage the file manually
    crate::commands::stage(&storage, file_path.to_string()).unwrap();
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
    crate::commands::init(db_path).unwrap();
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
fn test_first_commit_creates_main_bookmark() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test_main.db";
    {
        let storage = Storage::new(db_path).unwrap();

        let file_path = "test.txt";
        {
            let mut file = File::create(file_path).unwrap();
            file.write_all(b"initial\n").unwrap();
        }

        commit(&storage, "initial commit".to_string(), false).unwrap();

        let head = storage.commits().get_current_head().unwrap().unwrap();
        let main_ref = storage.refs().get_ref("main").unwrap().expect("main ref should exist");
        assert_eq!(head, main_ref);
    }

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_commit_moves_bookmarks() {
    let dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let db_path = "gik_test_move.db";
    {
        let storage = Storage::new(db_path).unwrap();

        // 1. First commit
        let file_path = "test.txt";
        {
            let mut file = File::create(file_path).unwrap();
            file.write_all(b"initial\n").unwrap();
        }
        commit(&storage, "initial commit".to_string(), false).unwrap();
        let head1 = storage.commits().get_current_head().unwrap().unwrap();

        // Create a manual bookmark
        storage.refs().set_ref("my-feature", &head1).unwrap();

        // 2. Second commit
        {
            let mut file = File::create(file_path).unwrap();
            file.write_all(b"modified\n").unwrap();
        }
        commit(&storage, "second commit".to_string(), false).unwrap();
        let head2 = storage.commits().get_current_head().unwrap().unwrap();

        // Both "main" and "my-feature" should have moved to head2
        let main_ref = storage.refs().get_ref("main").unwrap().unwrap();
        let feature_ref = storage.refs().get_ref("my-feature").unwrap().unwrap();

        assert_eq!(main_ref, head2);
        assert_eq!(feature_ref, head2);
    }

    std::env::set_current_dir(original_dir).unwrap();
}
