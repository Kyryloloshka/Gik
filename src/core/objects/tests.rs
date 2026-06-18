use super::*;
use std::io::Cursor;

#[test]
fn test_hash_blob_hello_world() {
    let content = "hello world\n";
    let size = content.len() as u64;
    let reader = Cursor::new(content);

    let hash = hash_blob(reader, size).unwrap();

    let expected_hex = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";
    assert_eq!(hex::encode(hash), expected_hex);
}

#[test]
fn test_hash_blob_git_reference() {
    let content = b"hello\n";
    let size = content.len() as u64;
    let reader = Cursor::new(content);

    let hash = hash_blob(reader, size).unwrap();
    let expected_hex = "ce013625030ba8dba906f756967f9e9ca394464a";
    assert_eq!(hex::encode(hash), expected_hex);
}

#[test]
fn test_compress_decompress_blob() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let storage = crate::core::storage::Storage::new(tmp_dir.path().join("db")).unwrap();

    let content = b"hello test blob compression";
    let size = content.len() as u64;
    let reader = &content[..];

    // Compute hash
    let hash = hash_blob(&content[..], size).unwrap();

    // Compress using new streaming interface directly to storage
    compress_blob(reader, size, &hash, &storage).unwrap();

    // Decompress
    let (obj_type, de_size, actual_content) =
        decompress_object(&storage.objects().get_object(&hash).unwrap().unwrap()[..]).unwrap();

    assert_eq!(obj_type, "blob");
    assert_eq!(de_size, size);
    assert_eq!(actual_content, content);
}

#[test]
fn test_parse_tree() {
    let mut entries = Vec::new();
    let hash1 = crate::core::hash::Hash([1; 20]);
    let hash2 = crate::core::hash::Hash([2; 20]);
    entries.push((0o100644, "file1.txt".to_string(), hash1));
    entries.push((0o040000, "dir1".to_string(), hash2));
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    let content = super::tree::build_tree_content(&entries);
    let parsed_entries = parse_tree(&content).unwrap();

    assert_eq!(parsed_entries.len(), 2);
    assert_eq!(parsed_entries[0].0, 0o040000);
    assert_eq!(parsed_entries[0].1, "dir1");
    assert_eq!(parsed_entries[0].2, hash2);

    assert_eq!(parsed_entries[1].0, 0o100644);
    assert_eq!(parsed_entries[1].1, "file1.txt");
    assert_eq!(parsed_entries[1].2, hash1);
}
