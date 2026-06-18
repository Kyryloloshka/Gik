use crate::core::storage::Storage;
use crate::error::Result;
use std::process::Command;

pub fn config(
    storage: &Storage,
    key: Option<String>,
    value: Option<String>,
    global: bool,
    import_git: bool,
) -> Result<()> {
    if import_git {
        println!("Importing Git configuration...");

        let name_output = Command::new("git")
            .args(["config", "--global", "user.name"])
            .output();
        if let Ok(output) = name_output {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                storage.config().set_global("user.name", &name)?;
                println!("Imported user.name = {}", name);
            }
        }

        let email_output = Command::new("git")
            .args(["config", "--global", "user.email"])
            .output();
        if let Ok(output) = email_output {
            if output.status.success() {
                let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
                storage.config().set_global("user.email", &email)?;
                println!("Imported user.email = {}", email);
            }
        }
        return Ok(());
    }

    let k = key.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Key is required when not using --import-git",
        )
    })?;

    if let Some(v) = value {
        if global {
            storage.config().set_global(&k, &v)?;
        } else {
            storage.config().set_local(&k, &v)?;
        }
    } else {
        let val = if global {
            storage.config().get_global(&k)?
        } else {
            storage.config().get(&k)?
        };

        if let Some(v) = val {
            println!("{}", v);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
