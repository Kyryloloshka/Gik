use super::*;
use std::io::Cursor;
use flate2::read::ZlibDecoder;
use std::io::Read;

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
    let content = "test content for compression";
    let size = content.len() as u64;
    let reader = Cursor::new(content);
    let mut compressed = Vec::new();

    compress_blob(reader, size, &mut compressed).unwrap();

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed).unwrap();

    let expected = format!("blob {}\0{}", size, content);
    assert_eq!(decompressed, expected);
}
