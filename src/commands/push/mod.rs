use crate::core::network::client::GitClient;
use crate::core::storage::Storage;
use crate::error::Result;

pub fn push(storage: &Storage) -> Result<()> {
    let _ = dotenvy::dotenv(); // load .env if exists

    let current_head = storage
        .commits()
        .get_current_head()?
        .ok_or_else(|| crate::error::GikError::NotFound("No HEAD found".to_string()))?;

    let remote_url = storage
        .config()
        .get_local("remote.origin.url")?
        .ok_or_else(|| {
            crate::error::GikError::Config(
                "remote.origin.url not set. Use `gik config --local remote.origin.url <url>`"
                    .to_string(),
            )
        })?;

    let token = std::env::var("GITHUB_TOKEN").ok();

    let client = GitClient::new(remote_url, token);

    let current_branch = storage
        .session()
        .get_current_bookmark()?
        .unwrap_or_else(|| "main".to_string());

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

    let missing = discover_missing_objects(storage, remote_head.as_ref(), &current_head)?;
    if missing.is_empty() {
        println!("Everything up-to-date");
        return Ok(());
    }

    println!("Compressing {} objects into Packfile...", missing.len());

    let temp_pack = crate::core::pack::encoder::build_packfile(storage, missing)?;

    client.push_packfile(
        &current_head,
        remote_head.as_ref(),
        temp_pack,
        &current_branch,
    )?;
    println!("Push successful!");

    Ok(())
}
