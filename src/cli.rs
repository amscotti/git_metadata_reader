use crate::repository::RepositoryConfig;
use chrono::NaiveDate;
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

    /// Maximum number of commits to process (for performance)
    #[clap(long)]
    pub max_commits: Option<u32>,

    /// Only analyze commits since this date (YYYY-MM-DD)
    #[clap(long)]
    pub since: Option<String>,

    /// Only analyze commits until this date (YYYY-MM-DD)
    #[clap(long)]
    pub until: Option<String>,
}

impl Args {
    pub fn get_repository_config(&self) -> RepositoryConfig {
        RepositoryConfig {
            max_commits: self.max_commits,
            since_date: self
                .since
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
            until_date: self
                .until
                .as_ref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        }
    }
}
