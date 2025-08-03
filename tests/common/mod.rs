//! Common test utilities and helpers for the Git History Explorer project

use chrono::NaiveDate;
use git_history_explorer::heatmap::HeatMapData;
use git_history_explorer::repository::{RepositoryConfig, RepositoryData};
use git_history_explorer::tui::AppState;
use git_history_explorer::user_commit_info::{CommitData, TimelineData};

/// Creates a test CommitData instance with sample data
pub fn create_test_commit_data() -> CommitData {
    CommitData::new(
        "test@example.com".to_string(),
        10,
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
    )
}

/// Creates a test TimelineData instance with sample commits
pub fn create_test_timeline_data() -> TimelineData {
    let mut timeline = TimelineData::default();
    timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 3);
    timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), 2);
    timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 3).unwrap(), 5);
    timeline
}

/// Creates a test HeatMapData instance with sample data
pub fn create_test_heatmap_data() -> HeatMapData {
    let mut heatmap = HeatMapData::new();
    heatmap.add_commits(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 3);
    heatmap.add_commits(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), 2);
    heatmap.add_commits(NaiveDate::from_ymd_opt(2023, 1, 3).unwrap(), 5);
    heatmap
}

/// Creates a test AppState instance with sample data
pub fn create_test_app_state() -> AppState {
    let commit_data = vec![
        create_test_commit_data(),
        CommitData::new(
            "another@example.com".to_string(),
            5,
            NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 6, 30).unwrap(),
        ),
    ];

    let mut author_timeline_data = std::collections::HashMap::new();
    author_timeline_data.insert("test@example.com".to_string(), create_test_timeline_data());

    let repository_data = RepositoryData {
        commit_data,
        heatmap_data: create_test_heatmap_data(),
        repo_path: "/test/repo".to_string(),
        author_timeline_data,
    };

    AppState::new(repository_data)
}

/// Creates a temporary Git repository for testing
pub fn create_temp_git_repo() -> tempfile::TempDir {
    use git2::Repository;
    use std::fs;
    use std::process::Command;

    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path();

    // Initialize git repository
    let _repo = Repository::init(repo_path).expect("Failed to initialize git repository");

    // Configure git user
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to set git user email");

    // Create a test file and commit
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Initial content").expect("Failed to write test file");

    // Add and commit the file
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit file");

    temp_dir
}

/// Creates a test RepositoryConfig with default values
pub fn create_test_repository_config() -> RepositoryConfig {
    RepositoryConfig::default()
}

/// Helper function to create a date for testing
pub fn test_date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}
