mod cli;
mod repository;
mod user_commit_info;

use cli::Args;
use repository::get_status;

use clap::Parser;

fn main() {
    let args = Args::parse();
    let repo_path = &args.path;
    get_status(repo_path);
}
