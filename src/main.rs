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

use colored::Colorize;

fn print_error_and_help(e: &error::GikError) {
    eprintln!("{}", format!("Error: {}", e).red().bold());
    match e {
        error::GikError::Config(_) => eprintln!("{}", "Help: Use 'gik config <key> <value>' to configure your repository.".yellow()),
        error::GikError::Auth(_) => eprintln!("{}", "Help: Check if your GITHUB_TOKEN is valid and has the necessary permissions.".yellow()),
        error::GikError::DirtyWorkspace(_) => eprintln!("{}", "Help: Run 'gik status' to see your changes. Use 'gik commit' to save them or 'gik restore' to discard.".yellow()),
        error::GikError::Branch(_) => eprintln!("{}", "Help: Use 'gik branch' to see available branches.".yellow()),
        error::GikError::NotFound(_) => eprintln!("{}", "Help: Ensure the object, commit, or path you specified exists.".yellow()),
        error::GikError::Merge(_) => eprintln!("{}", "Help: Ensure you are merging a valid commit and the working directory is clean.".yellow()),
        error::GikError::AmbiguousHash(_) => eprintln!("{}", "Help: Please provide more characters of the hash to uniquely identify it.".yellow()),
        _ => {}
    }
}

fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::init(crate::config::DB_PATH)?;
        }
        Commands::Clone { url, directory } => {
            commands::clone::clone(&url, directory)?;
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
                Commands::Log { all, json } => {
                    commands::log(&storage, all, json)?;
                }
                Commands::Restore { path } => {
                    let resolved_path = crate::core::utils::resolve_path(&cwd, &repo_root, &path);
                    commands::restore(&storage, &resolved_path)?;
                }
                Commands::Undo { yes, list } => {
                    commands::undo(&storage, yes, list)?;
                }
                Commands::Redo { yes, list } => {
                    commands::redo(&storage, yes, list)?;
                }
                Commands::Unstage { path } => {
                    commands::unstage(&storage, path)?;
                }
                Commands::Status { porcelain, is_merging } => {
                    if is_merging {
                        if storage.session().get_merge_head()?.is_some() {
                            std::process::exit(0);
                        } else {
                            std::process::exit(1);
                        }
                    }
                    commands::status::status(&storage, porcelain)?;
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
                Commands::Merge { target, continue_merge } => {
                    if continue_merge {
                        commands::merge::continue_merge(&storage)?;
                    } else if let Some(t) = target {
                        commands::merge::merge(&storage, &t)?;
                    } else {
                        return Err(crate::error::GikError::Validation("Must provide target or --continue".into()));
                    }
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
                Commands::Show { target } => {
                    commands::show(&storage, &target)?;
                }
                Commands::Init | Commands::Update | Commands::Clone { .. } => unreachable!(),
            }
        }
    }


    Ok(())
}

