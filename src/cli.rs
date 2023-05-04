use clap::Parser;

/// GitHistoryExplorer: Analyze and display commit history information from a Git repository
#[derive(Parser, Debug)]
#[clap(
    name = "GitHistoryExplorer",
    version = "0.1.0",
    about = "Explore commit history in a Git repository"
)]
pub struct Args {
    /// Path to the Git repository (default: current directory)
    #[clap(short, long, default_value = ".")]
    pub path: String,
}
