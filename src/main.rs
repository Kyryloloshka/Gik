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
        Commands::Commit { message } => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::commit(&storage, message)?;
        }
        Commands::Log => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::log(&storage)?;
        }
        Commands::Undo => {
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;
            commands::undo(&storage)?;
        }
    }

    Ok(())
}

