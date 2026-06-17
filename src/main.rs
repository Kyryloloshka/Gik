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
        print_error_and_help(&e);
        std::process::exit(1);
    }
}

fn print_error_and_help(e: &error::GikError) {
    eprintln!("❌ Error: {}", e);
    match e {
        error::GikError::Config(_) => eprintln!("💡 Help: Use 'gik config <key> <value>' to configure your repository."),
        error::GikError::Auth(_) => eprintln!("💡 Help: Check if your GITHUB_TOKEN is valid and has the necessary permissions."),
        error::GikError::DirtyWorkspace(_) => eprintln!("💡 Help: Run 'gik status' to see your changes. Use 'gik commit' to save them or 'gik restore' to discard."),
        error::GikError::Branch(_) => eprintln!("💡 Help: Use 'gik branch' to see available branches."),
        error::GikError::NotFound(_) => eprintln!("💡 Help: Ensure the object, commit, or path you specified exists."),
        error::GikError::Merge(_) => eprintln!("💡 Help: Ensure you are merging a valid commit and the working directory is clean."),
        error::GikError::AmbiguousHash(_) => eprintln!("💡 Help: Please provide more characters of the hash to uniquely identify it."),
        _ => {}
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
                Commands::Push => {
                    commands::push::push(&storage)?;
                }
                Commands::Pull => {
                    commands::pull::pull(&storage)?;
                }
                Commands::CatFile { p, t, s, hash } => {
                    commands::cat_file(&storage, &hash, p, t, s)?;
                }
                Commands::Init | Commands::Update => unreachable!(),
            }
        }
    }


    Ok(())
}

