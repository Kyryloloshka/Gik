pub mod cli;
pub mod commands;
pub mod error;
pub mod models;
pub mod objects;
pub mod storage;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::init()?;
        }
        Commands::Stage { path } => {
            commands::stage(path)?;
        }
        Commands::Commit { message } => {
            commands::commit(message)?;
        }
        Commands::Log => {
            commands::log()?;
        }
        Commands::Undo => {
            commands::undo()?;
        }
    }

    Ok(())
}

