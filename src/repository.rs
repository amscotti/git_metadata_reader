use chrono::Utc;
use chrono::{Datelike, TimeZone};
use git2::Repository;
use polars::prelude::*;
use std::collections::HashMap;
use std::path::Path;

use crate::heatmap::HeatMapData;
use crate::user_commit_info::{CommitData, TimelineData};

fn collect_commit_info_polars(
    repo: &Repository,
    config: &RepositoryConfig,
) -> Result<(DataFrame, DataFrame), PolarsError> {
    let mut revwalk = repo
        .revwalk()
        .expect("Could not access the repository's commits");

    revwalk.push_head().expect("Could not find HEAD");

    // Use capacity-based pre-allocation with smart defaults
    let estimated_commits = config.max_commits.unwrap_or({
        // Smart estimation: if no limit, assume medium-sized repo
        10_000
    });
    let mut emails = Vec::with_capacity(estimated_commits as usize);
    let mut dates = Vec::with_capacity(estimated_commits as usize);
    let mut commit_messages = Vec::with_capacity(estimated_commits as usize);
    let mut commits_processed = 0u32;

    for commit_oid in revwalk {
        if let Some(max_commits) = config.max_commits {
            if commits_processed >= max_commits {
                break;
            }
        }

        let commit_oid = commit_oid.expect("Invalid commit");
        let commit = repo.find_commit(commit_oid).expect("Could not find commit");

        if let Some(email) = commit.author().email() {
            let commit_time = Utc.timestamp_opt(commit.time().seconds(), 0);
            if let chrono::LocalResult::Single(commit_time) = commit_time {
                let commit_date = commit_time.date_naive();

                // Apply date filters early to avoid unnecessary processing
                if let Some(since_date) = config.since_date {
                    if commit_date < since_date {
                        continue;
                    }
                }
                if let Some(until_date) = config.until_date {
                    if commit_date > until_date {
                        continue;
                    }
                }

                // Optimize string allocations - only convert when necessary
                emails.push(email.to_string());
                dates.push(commit_date);
                // Skip message storage if not used for processing
                commit_messages.push(String::new()); // Placeholder if needed
                commits_processed += 1;
            }
        }
    }

    // Create DataFrame with only necessary columns
    let df = df!(
        "email" => emails,
        "date" => dates,
    )?;

    // Calculate both author statistics and timeline data in a single lazy operation
    let lazy_df = df.lazy();

    let author_stats = lazy_df
        .clone()
        .group_by([col("email")])
        .agg([
            col("date").min().alias("first_commit"),
            col("date").max().alias("last_commit"),
            col("date").count().alias("commit_count"),
        ])
        .collect()?;

    let timeline_df = lazy_df
        .group_by([col("email"), col("date")])
        .agg([col("date").count().alias("commits_on_date")])
        .collect()?;

    Ok((author_stats, timeline_df))
}

fn collect_commit_info(
    repo: Repository,
    config: &RepositoryConfig,
) -> (
    Vec<CommitData>,
    std::collections::HashMap<String, TimelineData>,
    u32,
) {
    let (author_stats, timeline_df) =
        collect_commit_info_polars(&repo, config).expect("Polars processing failed");

    // Convert Polars results back to original data structures
    let commit_data_vec = convert_author_stats_to_commit_info(author_stats);
    let author_timeline_data = convert_timeline_df_to_timeline_data_map(timeline_df);
    let total_commits = commit_data_vec.iter().map(|data| data.commits).sum();

    (commit_data_vec, author_timeline_data, total_commits)
}

fn convert_author_stats_to_commit_info(author_stats: DataFrame) -> Vec<CommitData> {
    let emails = author_stats.column("email").unwrap().str().unwrap();
    let first_commits = author_stats.column("first_commit").unwrap().date().unwrap();
    let last_commits = author_stats.column("last_commit").unwrap().date().unwrap();
    let commit_counts = author_stats.column("commit_count").unwrap().u32().unwrap();

    let mut result = Vec::with_capacity(author_stats.height());

    // Pre-allocate the epoch date
    static UNIX_EPOCH: once_cell::sync::Lazy<chrono::NaiveDate> =
        once_cell::sync::Lazy::new(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

    for i in 0..author_stats.height() {
        let email = emails.get(i).unwrap();
        let first_commit_days = first_commits.get(i).unwrap();
        let last_commit_days = last_commits.get(i).unwrap();
        let commit_count = commit_counts.get(i).unwrap();

        // Convert Polars date using cached epoch
        let first_commit = *UNIX_EPOCH + chrono::Duration::days(first_commit_days as i64);
        let last_commit = *UNIX_EPOCH + chrono::Duration::days(last_commit_days as i64);

        result.push(CommitData::new(
            email.to_string(),
            commit_count,
            first_commit,
            last_commit,
        ));
    }

    result
}

fn convert_timeline_df_to_timeline_data_map(
    timeline_df: DataFrame,
) -> std::collections::HashMap<String, TimelineData> {
    let mut timeline_map: HashMap<String, TimelineData> = HashMap::new();

    let emails = timeline_df.column("email").unwrap().str().unwrap();
    let dates = timeline_df.column("date").unwrap().date().unwrap();
    let commits = timeline_df
        .column("commits_on_date")
        .unwrap()
        .u32()
        .unwrap();

    // Use cached epoch for date conversion
    static UNIX_EPOCH: once_cell::sync::Lazy<chrono::NaiveDate> =
        once_cell::sync::Lazy::new(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

    for i in 0..timeline_df.height() {
        let email = emails.get(i).unwrap();
        let date_days = dates.get(i).unwrap();
        let commit_count = commits.get(i).unwrap();

        // Convert Polars date using cached epoch
        let date = *UNIX_EPOCH + chrono::Duration::days(date_days as i64);

        // Optimize: use entry API for fewer lookups
        let email_str = email.to_string();
        let timeline_data = timeline_map.entry(email_str.clone()).or_default();

        // Add commits for this date
        timeline_data.add_commit(date, commit_count);
    }

    timeline_map
}

fn prepare_commit_data(mut commits: Vec<CommitData>) -> Vec<CommitData> {
    commits.sort_by(|a, b| {
        a.first_commit
            .cmp(&b.first_commit)
            .then(a.last_commit.cmp(&b.last_commit).reverse())
    });

    commits
}

fn prepare_heatmap_data_from_map(
    timeline_data: &std::collections::HashMap<String, TimelineData>,
) -> HeatMapData {
    let mut heatmap_data = HeatMapData::new();
    let current_year = chrono::Utc::now().date_naive().year();

    // Aggregate commits from all authors by mapping to current year calendar
    for author_timeline in timeline_data.values() {
        for (historical_date, commits) in &author_timeline.commits_by_period {
            // Map historical date to equivalent date in current year
            let calendar_date = chrono::NaiveDate::from_ymd_opt(
                current_year,
                historical_date.month(),
                historical_date.day(),
            )
            .unwrap_or(*historical_date); // fallback to original date if invalid (e.g., Feb 29)

            heatmap_data.add_commits(calendar_date, *commits);
        }
    }

    heatmap_data
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryConfig {
    pub max_commits: Option<u32>,
    pub since_date: Option<chrono::NaiveDate>,
    pub until_date: Option<chrono::NaiveDate>,
}

#[derive(Debug)]
pub struct RepositoryData {
    pub commit_data: Vec<CommitData>,
    pub heatmap_data: HeatMapData,
    pub repo_path: String,
    pub author_timeline_data: std::collections::HashMap<String, TimelineData>,
}

pub fn get_repository_data_with_config(
    repo_path: &str,
    config: &RepositoryConfig,
) -> Result<RepositoryData, String> {
    let start_time = std::time::Instant::now();

    let repo: Repository = match Repository::open(Path::new(repo_path)) {
        Ok(repo) => repo,
        Err(e) => {
            return Err(format!(
                "Could not open the Git repository at '{repo_path}'. Details: {e}"
            ));
        }
    };

    let collection_start = std::time::Instant::now();
    let (commit_data_vec, author_timeline_data, _total_commits_processed) =
        collect_commit_info(repo, config);
    let _collection_duration = collection_start.elapsed();

    let processing_start = std::time::Instant::now();
    let commit_data = prepare_commit_data(commit_data_vec);
    let heatmap_data = prepare_heatmap_data_from_map(&author_timeline_data);
    let _processing_duration = processing_start.elapsed();

    let _total_duration = start_time.elapsed();

    Ok(RepositoryData {
        commit_data,
        heatmap_data,
        repo_path: repo_path.to_string(),
        author_timeline_data,
    })
}
