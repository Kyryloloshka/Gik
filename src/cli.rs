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
    /// Commit staged changes
    Commit {
        /// The commit message
        #[arg(short, long)]
        message: String,
    },
    /// Show the commit log
    Log,
    /// Undo the last commit
    Undo,
    /// Update Gik to the latest version
    Update,
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
            Commands::Commit { message } => {
                assert_eq!(message, "hello");
            }
            _ => panic!("Expected Commit command"),
        }
    }
}
