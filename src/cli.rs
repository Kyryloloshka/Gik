use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gik")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Gik repository
    Init,
    /// Stage a file for commit
    Stage {
        /// The path to the file to stage
        path: String,
    },
    /// Commit changes
    Commit {
        /// The commit message
        #[arg(short, long)]
        message: String,
        /// Only commit currently staged files
        #[arg(long)]
        staged: bool,
    },
    /// Show the commit log
    Log {
        /// Show all commits, not just ancestors of HEAD
        #[arg(short, long)]
        all: bool,
    },
    /// Restore working tree files
    Restore {
        /// Path to restore (use '.' for everything)
        path: String,
    },
    /// Undo the last commit
    Undo,
    /// Show the working tree status
    Status,
    /// Show changes between commits, commit and working tree, etc
    Diff {
        /// Show changes in the stage
        #[arg(long)]
        staged: bool,
    },
    /// Update Gik to the latest version
    Update,
    /// Restore the repository to a previous state
    Checkout {
        /// The commit hash to checkout
        hash: String,
        /// Force checkout even if there are uncommitted changes
        #[arg(short, long)]
        force: bool,
    },
    /// Manage bookmarks (branches)
    Branch {
        /// The name of the bookmark
        name: Option<String>,
        /// Delete the bookmark
        #[arg(short, long)]
        delete: bool,
    },
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_init() {
        let cli = Cli::try_parse_from(&["gik", "init"]).unwrap();
        match cli.command {
            Commands::Init => {}
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_stage() {
        let cli = Cli::try_parse_from(&["gik", "stage", "file.txt"]).unwrap();
        match cli.command {
            Commands::Stage { path } => {
                assert_eq!(path, "file.txt");
            }
            _ => panic!("Expected Stage command"),
        }
    }

    #[test]
    fn test_parse_commit() {
        let cli = Cli::try_parse_from(&["gik", "commit", "-m", "hello"]).unwrap();
        match cli.command {
            Commands::Commit { message, staged } => {
                assert_eq!(message, "hello");
                assert!(!staged);
            }
            _ => panic!("Expected Commit command"),
        }
    }

    #[test]
    fn test_parse_checkout() {
        let cli = Cli::try_parse_from(&["gik", "checkout", "abc1234", "--force"]).unwrap();
        match cli.command {
            Commands::Checkout { hash, force } => {
                assert_eq!(hash, "abc1234");
                assert!(force);
            }
            _ => panic!("Expected Checkout command"),
        }
    }
}
