mod cli;
mod heatmap;
mod repository;
mod tui;
mod ui;
mod user_commit_info;

use cli::Args;
use repository::get_repository_data_with_config;
use std::io;

use clap::Parser;

fn main() -> io::Result<()> {
    let args = Args::parse();
    let repo_path = &args.path;
    let config = args.get_repository_config();

    match get_repository_data_with_config(repo_path, &config) {
        Ok(repository_data) => {
            tui::run_tui(repository_data)?;
        }
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }

    Ok(())
}
