use crate::core::storage::Storage;
use crate::error::Result;
use crate::core::network::client::GitClient;

pub fn push(storage: &Storage) -> Result<()> {
    let _ = dotenvy::dotenv(); // load .env if exists

    let current_head = storage.commits().get_current_head()?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No HEAD found"))?;

    let remote_url = storage.config().get_local("remote.origin.url")?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "remote.origin.url not set. Use `gik config --local remote.origin.url <url>`"))?;

    let token = std::env::var("GITHUB_TOKEN").ok();
    
    let client = GitClient::new(remote_url, token);
    
    println!("Discovering remote refs...");
    let remote_head = client.discover_refs()?;
    
    if let Some(r_head) = remote_head {
        if r_head == current_head {
            println!("Everything up-to-date");
            return Ok(());
        }
        println!("Remote HEAD is: {}", r_head);
    } else {
        println!("Remote repository is empty or branch missing.");
    }
    
    println!("Pushing objects...");
    use crate::core::pack::discovery::discover_missing_objects;
    use crate::core::pack::encoder::{write_packfile_header, write_object_header};
    use crate::core::objects::decompress_object;
    use sha1::{Sha1, Digest};

    let missing = discover_missing_objects(storage, remote_head.as_ref(), &current_head)?;
    if missing.is_empty() {
        println!("Everything up-to-date");
        return Ok(());
    }
    
    println!("Compressing {} objects into Packfile...", missing.len());
    
    let mut dummy_hasher = Sha1::new();
    let mut temp_pack = Vec::new();
    let _h = write_packfile_header(&mut temp_pack, missing.len() as u32)?;
    
    for hash in missing {
        if let Some(compressed) = storage.objects().get_object(&hash)? {
            let (obj_type, size, content) = decompress_object(&compressed[..])?;
            let type_id = match obj_type.as_str() {
                "commit" => 1,
                "tree" => 2,
                "blob" => 3,
                _ => continue,
            };
            
            write_object_header(&mut temp_pack, type_id, size as usize, &mut dummy_hasher)?;
            
            use std::io::Write;
            let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&content).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
            let zlibbed = encoder.finish().map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
            temp_pack.extend_from_slice(&zlibbed);
        }
    }
    
    let checksum = Sha1::digest(&temp_pack);
    temp_pack.extend_from_slice(&checksum);
    
    client.push_packfile(&current_head, remote_head.as_ref(), &temp_pack)?;
    println!("Push successful!");
    
    Ok(())
}
