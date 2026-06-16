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
    
    let current_branch = storage.session().get_current_bookmark()?.unwrap_or_else(|| "main".to_string());
    
    println!("Discovering remote refs...");
    let remote_head = client.discover_refs(&current_branch)?;
    
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
    let mut temp_pack = tempfile::tempfile().map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
    let _h = write_packfile_header(&mut temp_pack, missing.len() as u32)?;
    
    for hash in missing {
        let (obj_type, size, content) = if let Some(meta) = storage.commits().get_commit_meta(&hash)? {
            let (author_name, author_email) = if let Some(open) = meta.author.find('<') {
                if let Some(close) = meta.author.find('>') {
                    (meta.author[..open].trim(), &meta.author[open+1..close])
                } else {
                    (meta.author.as_str(), "")
                }
            } else {
                (meta.author.as_str(), "")
            };
            
            let payload = crate::core::objects::commit::build_commit_content(
                meta.tree_hash,
                &meta.parent_hashes,
                author_name,
                author_email,
                meta.timestamp,
                &meta.message,
            ).into_bytes();
            (1u8, payload.len(), payload)
        } else if let Some(compressed) = storage.objects().get_object(&hash)? {
            let (type_str, size, payload) = decompress_object(&compressed[..])?;
            let type_id = match type_str.as_str() {
                "tree" => 2u8,
                "blob" => 3u8,
                _ => return Err(crate::error::GikError::Io(std::io::Error::other("Unknown object type in storage"))),
            };
            (type_id, size as usize, payload)
        } else {
            return Err(crate::error::GikError::Io(std::io::Error::other(format!("Missing object {}", hash))));
        };
        
        write_object_header(&mut temp_pack, obj_type, size, &mut dummy_hasher)?;
        
        use std::io::Write;
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&content).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
        let zlibbed = encoder.finish().map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
        temp_pack.write_all(&zlibbed).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
    }
    
    // We must manually compute checksum for the whole file. 
    // This is not perfectly streaming because we read it back once, but it solves the memory issue.
    use std::io::{Read, Seek, SeekFrom};
    temp_pack.seek(SeekFrom::Start(0)).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
    let mut real_hasher = Sha1::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = temp_pack.read(&mut buffer).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
        if n == 0 { break; }
        real_hasher.update(&buffer[..n]);
    }
    let checksum = real_hasher.finalize();
    temp_pack.seek(SeekFrom::End(0)).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
    use std::io::Write;
    temp_pack.write_all(&checksum).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
    
    // Reset cursor to start for sending
    temp_pack.seek(SeekFrom::Start(0)).map_err(|e| crate::error::GikError::Io(std::io::Error::other(e.to_string())))?;
    
    let current_branch = storage.session().get_current_bookmark()?.unwrap_or_else(|| "main".to_string());
    
    client.push_packfile(&current_head, remote_head.as_ref(), temp_pack, &current_branch)?;
    println!("Push successful!");
    
    Ok(())
}
