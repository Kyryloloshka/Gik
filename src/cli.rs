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
    /// Clone a repository into a new directory
    Clone {
        /// The URL of the remote repository
        url: String,
        /// The directory to clone into
        directory: Option<String>,
    },
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
        /// Explicitly move or create a bookmark
        #[arg(short, long)]
        branch: Option<String>,
    },
    /// Show the commit log
    Log {
        /// Show all commits, not just ancestors of HEAD
        #[arg(short, long)]
        all: bool,
        /// Output the log as a JSON array
        #[arg(long)]
        json: bool,
    },
    /// Restore working tree files
    Restore {
        /// Path to restore (use '.' for everything)
        path: String,
    },
    /// Undo the last commit
    Undo,
    /// Manage Gik configuration
    Config {
        /// The config key to get or set
        key: Option<String>,
        /// The config value to set
        value: Option<String>,
        /// Use global configuration
        #[arg(long)]
        global: bool,
        /// Import user.name and user.email from git config
        #[arg(long)]
        import_git: bool,
    },
    /// Unstage a file (remove it from the index)
    Unstage {
        /// The path to unstage (or . for all)
        path: String,
    },
    /// Show the working tree status
    Status {
        /// Give the output in an easy-to-parse format for scripts
        #[arg(long)]
        porcelain: bool,
    },
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
    /// Merge another bookmark or commit into the current branch
    Merge {
        /// The bookmark or commit hash to merge
        #[arg(required_unless_present = "continue_merge")]
        target: Option<String>,
        /// Continue a merge after resolving conflicts
        #[arg(long = "continue", id = "continue_merge")]
        continue_merge: bool,
    },
    /// Push commits to remote repository
    Push,
    /// Fetch and fast-forward from remote repository
    Pull,
    /// Provide content or type and size information for repository objects
    CatFile {
        /// Pretty-print the contents of the object
        #[arg(short, long)]
        p: bool,
        /// Show object type
        #[arg(short, long)]
        t: bool,
        /// Show object size
        #[arg(short, long)]
        s: bool,
        /// The object hash
        hash: String,
    },
    /// Show object content
    Show {
        /// Target in format <ref>:<path>
        target: String,
    },
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_init() {
        let cli = Cli::try_parse_from(["gik", "init"]).unwrap();
        match cli.command {
            Commands::Init => {}
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_parse_stage() {
        let cli = Cli::try_parse_from(["gik", "stage", "file.txt"]).unwrap();
        match cli.command {
            Commands::Stage { path } => {
                assert_eq!(path, "file.txt");
            }
            _ => panic!("Expected Stage command"),
        }
    }

    #[test]
    fn test_parse_commit() {
        let cli = Cli::try_parse_from(["gik", "commit", "-m", "hello"]).unwrap();
        match cli.command {
            Commands::Commit { message, staged, branch } => {
                assert_eq!(message, "hello");
                assert!(!staged);
                assert!(branch.is_none());
            }
            _ => panic!("Expected Commit command"),
        }
    }

    #[test]
    fn test_parse_checkout() {
        let cli = Cli::try_parse_from(["gik", "checkout", "abc1234", "--force"]).unwrap();
        match cli.command {
            Commands::Checkout { hash, force } => {
                assert_eq!(hash, "abc1234");
                assert!(force);
            }
            _ => panic!("Expected Checkout command"),
        }
    }
}
