pub mod cli;
pub mod commands;
pub mod config;
pub mod core;
pub mod error;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;

fn main() -> Result<()> {

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::init(crate::config::DB_PATH)?;
        }
        Commands::Stage { path } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::stage(&storage, path)?;
        }
        Commands::Commit { message, staged, branch } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::commit(&storage, message, staged, branch)?;
        }
        Commands::Log { all } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::log(&storage, all)?;
        }
        Commands::Restore { path } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::restore(&storage, &path)?;
        }
        Commands::Undo => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::undo(&storage)?;
        }
        Commands::Status => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::status(&storage)?;
        }
        Commands::Diff { staged } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::diff(&storage, staged)?;
        }
        Commands::Update => {
            commands::update()?;
        }
        Commands::Checkout { hash, force } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::checkout(&storage, &hash, force)?;
        }
        Commands::Branch { name, delete } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::branch(&storage, name, delete)?;
        }
    }


    Ok(())
}

