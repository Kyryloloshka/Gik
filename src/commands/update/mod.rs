use crate::error::Result;
use self_update::cargo_crate_version;

pub fn update() -> Result<()> {
    let target = self_update::get_target();
    println!("Checking for updates (target: {})...", target);

    let status = self_update::backends::github::Update::configure()
        .repo_owner("Kyryloloshka")
        .repo_name("Gik")
        .bin_name("gik")
        .target(target)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()
        .map_err(|e| std::io::Error::other(e.to_string()))?
        .update()
        .map_err(|e| std::io::Error::other(e.to_string()))?;


    if status.updated() {
        println!("Successfully updated to version {}!", status.version());
    } else {
        println!("Gik is already up to date (version {}).", cargo_crate_version!());
    }

    Ok(())
}
