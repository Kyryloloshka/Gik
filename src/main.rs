pub mod cli;
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
            handle_init()?;
        }
        Commands::Stage { path } => {
            handle_stage(path)?;
        }
        Commands::Commit { message } => {
            handle_commit(message)?;
        }
        Commands::Log => {
            handle_log()?;
        }
        Commands::Undo => {
            handle_undo()?;
        }
    }

    Ok(())
}

fn handle_init() -> Result<()> {
    // Placeholder for init
    Ok(())
}

fn handle_stage(_path: String) -> Result<()> {
    // Placeholder for stage
    Ok(())
}

fn handle_commit(_message: String) -> Result<()> {
    // Placeholder for commit
    Ok(())
}

fn handle_log() -> Result<()> {
    // Placeholder for log
    Ok(())
}

fn handle_undo() -> Result<()> {
    // Placeholder for undo
    Ok(())
}
