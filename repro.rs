use sha1::{Sha1, Digest};

fn main() {
    let mut hasher = Sha1::new();
    let header = "blob 6\0";
    let content = "hello\n";
    hasher.update(header.as_bytes());
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    println!("{:x}", result);
}
