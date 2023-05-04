use chrono::TimeZone;
use chrono::Utc;
use git2::Repository;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;

use crate::user_commit_info::UserCommitInfo;

fn collect_commit_info(repo: Repository) -> Vec<(String, UserCommitInfo)> {
    let mut revwalk = repo
        .revwalk()
        .expect("Could not access the repository's commits");

    revwalk.push_head().expect("Could not find HEAD");

    let mut commit_info_map: HashMap<String, UserCommitInfo> = HashMap::new();

    for commit_oid in revwalk {
        let commit_oid = commit_oid.expect("Invalid commit");
        let commit = repo.find_commit(commit_oid).expect("Could not find commit");

        let email = commit.author().email().map(|s| s.to_owned());
        if let Some(email) = email {
            let commit_time = Utc.timestamp_opt(commit.time().seconds(), 0);
            if let chrono::LocalResult::Single(commit_time) = commit_time {
                commit_info_map
                    .entry(email)
                    .and_modify(|c: &mut UserCommitInfo| c.update(commit_time.date_naive()))
                    .or_insert_with(|| UserCommitInfo::new(commit_time.date_naive()));
            }
        }
    }

    commit_info_map.into_iter().collect()
}

fn print_commits(mut commits: Vec<(String, UserCommitInfo)>) {
    commits.sort_by(|(_, a), (_, b)| {
        a.first_commit
            .cmp(&b.first_commit)
            .then(a.last_commit.cmp(&b.last_commit).reverse())
    });

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    if let Err(e) = writeln!(
        stdout,
        "{:<55} {:<10} {:<12} {:<12} {:<5}",
        "Email", "Commits", "First", "Last", "Days"
    ) {
        eprintln!("Error writing to stdout: {}", e);
    }

    for (email, user_commit_info) in commits {
        if let Err(e) = writeln!(
            stdout,
            "{:<55} {:<10} {:<12} {:<12} {:<5}",
            email,
            user_commit_info.commits,
            user_commit_info.first_commit.format("%m/%d/%Y"),
            user_commit_info.last_commit.format("%m/%d/%Y"),
            user_commit_info.days_between()
        ) {
            if e.kind() != io::ErrorKind::BrokenPipe {
                eprintln!("Error writing to stdout: {}", e);
            }
            break;
        }
    }
}

pub fn get_status(repo_path: &str) {
    let repo: Repository = match Repository::open(Path::new(repo_path)) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!(
                "Error: Could not open the Git repository at '{}'.\nDetails: {}",
                repo_path, e
            );
            return;
        }
    };

    let commit_info_vec: Vec<(String, UserCommitInfo)> = collect_commit_info(repo);

    print_commits(commit_info_vec);
}
