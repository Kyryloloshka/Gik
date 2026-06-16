pub mod cli;
pub mod commands;
pub mod config;
pub mod core;
pub mod error;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::init(crate::config::DB_PATH)?;
        }
        Commands::Update => {
            commands::update()?;
        }
        other => {
            let cwd = std::env::current_dir()?;
            let repo_root = crate::core::utils::find_repo_root(&cwd)?;
            std::env::set_current_dir(&repo_root)?;
            let storage = crate::core::storage::Storage::new(crate::config::DB_PATH)?;

            match other {
                Commands::Stage { path } => {
                    let resolved_path = crate::core::utils::resolve_path(&cwd, &repo_root, &path);
                    commands::stage(&storage, resolved_path)?;
                }
                Commands::Commit { message, staged, branch } => {
                    commands::commit(&storage, message, staged, branch)?;
                }
                Commands::Log { all, graph } => {
                    commands::log(&storage, all, graph)?;
                }
                Commands::Restore { path } => {
                    let resolved_path = crate::core::utils::resolve_path(&cwd, &repo_root, &path);
                    commands::restore(&storage, &resolved_path)?;
                }
                Commands::Undo => {
                    commands::undo(&storage)?;
                }
                Commands::Status => {
                    commands::status(&storage)?;
                }
                Commands::Diff { staged } => {
                    commands::diff(&storage, staged)?;
                }
                Commands::Checkout { hash, force } => {
                    commands::checkout(&storage, &hash, force)?;
                }
                Commands::Branch { name, delete } => {
                    commands::branch::branch(&storage, name, delete)?;
                }
                Commands::Merge { target } => {
                    commands::merge::merge(&storage, &target)?;
                }
                Commands::Config { key, value, global, import_git } => {
                    commands::config(&storage, key, value, global, import_git)?;
                }
                Commands::Init | Commands::Update => unreachable!(),
            }
        }
    }


    Ok(())
}

