use crate::heatmap::HeatMapData;
use crate::repository::RepositoryData;
use crate::ui::render_app;
use crate::user_commit_info::CommitData;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, stdout};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    Email,
    Commits,
    FirstCommit,
    LastCommit,
    DaysBetween,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub struct AppState {
    pub repository_data: RepositoryData,
    pub selected_row: usize,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub filter_text: String,
    pub show_search: bool,
    pub error_message: Option<String>,
    pub selected_author: Option<String>,
    pub author_heatmap_data: std::collections::HashMap<String, HeatMapData>,
}

impl AppState {
    pub fn new(repository_data: RepositoryData) -> Self {
        Self {
            repository_data,
            selected_row: 0,
            sort_column: SortColumn::FirstCommit,
            sort_direction: SortDirection::Ascending,
            filter_text: String::new(),
            show_search: false,
            error_message: None,
            selected_author: None,
            author_heatmap_data: std::collections::HashMap::new(),
        }
    }

    pub fn filtered_data(&self) -> Vec<&CommitData> {
        self.repository_data
            .commit_data
            .iter()
            .filter(|data| {
                if self.filter_text.is_empty() {
                    true
                } else {
                    data.email
                        .to_lowercase()
                        .contains(&self.filter_text.to_lowercase())
                }
            })
            .collect()
    }

    pub fn get_filtered_heatmap_data(&self) -> &HeatMapData {
        if let Some(selected_email) = &self.selected_author {
            // Return cached author-specific heatmap if available
            if let Some(author_heatmap) = self.author_heatmap_data.get(selected_email) {
                return author_heatmap;
            }
        }
        &self.repository_data.heatmap_data
    }

    pub fn get_or_create_author_heatmap(&mut self, author_email: &str) -> &HeatMapData {
        if !self.author_heatmap_data.contains_key(author_email) {
            // Create new author-specific heatmap from actual timeline data
            if let Some(author_timeline) =
                self.repository_data.author_timeline_data.get(author_email)
            {
                let author_heatmap = HeatMapData::create_from_timeline_data(author_timeline);
                self.author_heatmap_data
                    .insert(author_email.to_string(), author_heatmap);
            } else {
                // Fallback to empty heatmap if no timeline data exists
                let author_heatmap = HeatMapData::new();
                self.author_heatmap_data
                    .insert(author_email.to_string(), author_heatmap);
            }
        }

        self.author_heatmap_data.get(author_email).unwrap()
    }

    pub fn sorted_data(&self) -> Vec<&CommitData> {
        let mut filtered = self.filtered_data();

        filtered.sort_by(|a, b| {
            let comparison = match self.sort_column {
                SortColumn::Email => a.email.cmp(&b.email),
                SortColumn::Commits => a.commits.cmp(&b.commits),
                SortColumn::FirstCommit => a.first_commit.cmp(&b.first_commit),
                SortColumn::LastCommit => a.last_commit.cmp(&b.last_commit),
                SortColumn::DaysBetween => a.days_between().cmp(&b.days_between()),
            };

            match self.sort_direction {
                SortDirection::Ascending => comparison,
                SortDirection::Descending => comparison.reverse(),
            }
        });

        filtered
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Up => {
                if self.selected_row > 0 {
                    self.selected_row -= 1;
                }
            }
            KeyCode::Down => {
                let max_row = self.sorted_data().len().saturating_sub(1);
                if self.selected_row < max_row {
                    self.selected_row += 1;
                }
            }
            KeyCode::Char('/') => {
                self.show_search = true;
                self.filter_text.clear();
            }
            KeyCode::Enter => {
                if self.show_search {
                    self.show_search = false;
                } else {
                    // Toggle author selection
                    let sorted_data = self.sorted_data();
                    if let Some(selected_data) = sorted_data.get(self.selected_row) {
                        match &self.selected_author {
                            Some(current_email) if current_email == &selected_data.email => {
                                // Deselect if clicking the same author
                                self.selected_author = None;
                            }
                            _ => {
                                // Select new author and generate heatmap
                                let author_email = selected_data.email.clone();
                                self.get_or_create_author_heatmap(&author_email);
                                self.selected_author = Some(author_email);
                            }
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                if self.show_search {
                    self.filter_text.pop();
                }
            }
            KeyCode::Char(c) => {
                if self.show_search {
                    self.filter_text.push(c);
                } else {
                    match c {
                        '1' => {
                            self.sort_column = SortColumn::Email;
                            self.selected_row = 0;
                        }
                        '2' => {
                            self.sort_column = SortColumn::Commits;
                            self.selected_row = 0;
                        }
                        '3' => {
                            self.sort_column = SortColumn::FirstCommit;
                            self.selected_row = 0;
                        }
                        '4' => {
                            self.sort_column = SortColumn::LastCommit;
                            self.selected_row = 0;
                        }
                        '5' => {
                            self.sort_column = SortColumn::DaysBetween;
                            self.selected_row = 0;
                        }
                        'r' | 'R' => {
                            self.sort_direction = match self.sort_direction {
                                SortDirection::Ascending => SortDirection::Descending,
                                SortDirection::Descending => SortDirection::Ascending,
                            };
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        true
    }
}

pub fn run_tui(repository_data: RepositoryData) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app_state = AppState::new(repository_data);

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| render_app(f, &mut app_state))?;

        // Handle events with better error handling
        match event::read() {
            Ok(Event::Key(key_event)) => {
                if !app_state.handle_key_event(key_event) {
                    break;
                }
            }
            Ok(Event::Resize(_, _)) => {
                // Terminal was resized, will be handled on next draw
            }
            Ok(_) => {}
            Err(e) => {
                // Log the error but continue running
                eprintln!("Event read error: {e}");
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heatmap::HeatMapData;
    use crate::user_commit_info::{CommitData, TimelineData};
    use chrono::NaiveDate;
    use crossterm::event::{KeyCode, KeyEvent};

    fn create_test_repository_data() -> RepositoryData {
        let commit_data = vec![
            CommitData::new(
                "alice@example.com".to_string(),
                10,
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            ),
            CommitData::new(
                "bob@example.com".to_string(),
                5,
                NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 6, 30).unwrap(),
            ),
            CommitData::new(
                "charlie@example.com".to_string(),
                15,
                NaiveDate::from_ymd_opt(2023, 3, 1).unwrap(),
                NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
            ),
        ];

        let mut author_timeline_data = std::collections::HashMap::new();
        let mut alice_timeline = TimelineData::default();
        alice_timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 3);
        alice_timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), 7);
        author_timeline_data.insert("alice@example.com".to_string(), alice_timeline);

        let mut heatmap_data = HeatMapData::new();
        heatmap_data.add_commits(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 3);
        heatmap_data.add_commits(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), 7);

        RepositoryData {
            commit_data,
            heatmap_data,
            repo_path: "/test/repo".to_string(),
            author_timeline_data,
        }
    }

    #[test]
    fn test_app_state_new() {
        let repo_data = create_test_repository_data();
        let app_state = AppState::new(repo_data);

        assert_eq!(app_state.selected_row, 0);
        assert_eq!(app_state.sort_column, SortColumn::FirstCommit);
        assert_eq!(app_state.sort_direction, SortDirection::Ascending);
        assert!(app_state.filter_text.is_empty());
        assert!(!app_state.show_search);
        assert!(app_state.error_message.is_none());
        assert!(app_state.selected_author.is_none());
        assert!(app_state.author_heatmap_data.is_empty());
    }

    #[test]
    fn test_app_state_filtered_data_empty_filter() {
        let repo_data = create_test_repository_data();
        let app_state = AppState::new(repo_data);

        let filtered = app_state.filtered_data();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_app_state_filtered_data_with_filter() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.filter_text = "alice".to_string();

        let filtered = app_state.filtered_data();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].email, "alice@example.com");
    }

    #[test]
    fn test_app_state_filtered_data_case_insensitive() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.filter_text = "ALICE".to_string();

        let filtered = app_state.filtered_data();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].email, "alice@example.com");
    }

    #[test]
    fn test_app_state_filtered_data_no_match() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.filter_text = "nonexistent".to_string();

        let filtered = app_state.filtered_data();
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_app_state_sorted_data_email_ascending() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.sort_column = SortColumn::Email;
        app_state.sort_direction = SortDirection::Ascending;

        let sorted = app_state.sorted_data();
        assert_eq!(sorted[0].email, "alice@example.com");
        assert_eq!(sorted[1].email, "bob@example.com");
        assert_eq!(sorted[2].email, "charlie@example.com");
    }

    #[test]
    fn test_app_state_sorted_data_email_descending() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.sort_column = SortColumn::Email;
        app_state.sort_direction = SortDirection::Descending;

        let sorted = app_state.sorted_data();
        assert_eq!(sorted[0].email, "charlie@example.com");
        assert_eq!(sorted[1].email, "bob@example.com");
        assert_eq!(sorted[2].email, "alice@example.com");
    }

    #[test]
    fn test_app_state_sorted_data_commits() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.sort_column = SortColumn::Commits;
        app_state.sort_direction = SortDirection::Ascending;

        let sorted = app_state.sorted_data();
        assert_eq!(sorted[0].commits, 5); // bob
        assert_eq!(sorted[1].commits, 10); // alice
        assert_eq!(sorted[2].commits, 15); // charlie
    }

    #[test]
    fn test_app_state_sorted_data_first_commit() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.sort_column = SortColumn::FirstCommit;
        app_state.sort_direction = SortDirection::Ascending;

        let sorted = app_state.sorted_data();
        assert_eq!(
            sorted[0].first_commit,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        ); // alice
        assert_eq!(
            sorted[1].first_commit,
            NaiveDate::from_ymd_opt(2023, 3, 1).unwrap()
        ); // charlie
        assert_eq!(
            sorted[2].first_commit,
            NaiveDate::from_ymd_opt(2023, 6, 1).unwrap()
        ); // bob
    }

    #[test]
    fn test_app_state_sorted_data_days_between() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);
        app_state.sort_column = SortColumn::DaysBetween;
        app_state.sort_direction = SortDirection::Ascending;

        let sorted = app_state.sorted_data();
        // bob: 29 days, charlie: 213 days, alice: 364 days
        assert_eq!(sorted[0].email, "bob@example.com");
        assert_eq!(sorted[1].email, "charlie@example.com");
        assert_eq!(sorted[2].email, "alice@example.com");
    }

    #[test]
    fn test_app_state_get_filtered_heatmap_data_no_selection() {
        let repo_data = create_test_repository_data();
        let app_state = AppState::new(repo_data);

        let heatmap_data = app_state.get_filtered_heatmap_data();
        // Should return the default heatmap data
        assert!(!heatmap_data.commits_by_date.is_empty());
    }

    #[test]
    fn test_app_state_get_or_create_author_heatmap_new() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        let author_email = "alice@example.com";

        // Initially should not be cached
        assert!(!app_state.author_heatmap_data.contains_key(author_email));

        // Create the heatmap
        let heatmap_data = app_state.get_or_create_author_heatmap(author_email);
        let max_commits = heatmap_data.max_commits;

        // After the reference is dropped, we can check the cache
        assert!(app_state.author_heatmap_data.contains_key(author_email));
        assert!(max_commits > 0);
    }

    #[test]
    fn test_app_state_get_or_create_author_heatmap_cached() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        let author_email = "alice@example.com";

        // Create the heatmap first
        let heatmap1 = app_state.get_or_create_author_heatmap(author_email);
        let ptr1 = heatmap1 as *const HeatMapData;

        // Drop the reference to avoid borrow conflict
        let _ = heatmap1;

        // Get it again - should be cached
        let heatmap2 = app_state.get_or_create_author_heatmap(author_email);
        let ptr2 = heatmap2 as *const HeatMapData;

        // Should return the same cached instance
        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn test_handle_key_event_quit() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Test 'q' key
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('q'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(!result);

        // Test Esc key
        let mut app_state = AppState::new(create_test_repository_data());
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(!result);
    }

    #[test]
    fn test_handle_key_event_navigation() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Test Down key
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert_eq!(app_state.selected_row, 1);

        // Test Down key again
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert_eq!(app_state.selected_row, 2);

        // Test Up key
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Up,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert_eq!(app_state.selected_row, 1);

        // Test Up key at top
        app_state.selected_row = 0;
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Up,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert_eq!(app_state.selected_row, 0); // Should not go below 0
    }

    #[test]
    fn test_handle_key_event_search_toggle() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Test enabling search
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert!(app_state.show_search);
        assert!(app_state.filter_text.is_empty());

        // Test disabling search with Enter
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert!(!app_state.show_search);
    }

    #[test]
    fn test_handle_key_event_author_selection() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Select first author
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert_eq!(
            app_state.selected_author,
            Some("alice@example.com".to_string())
        );

        // Deselect the same author
        let result = app_state.handle_key_event(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(result);
        assert_eq!(app_state.selected_author, None);
    }

    #[test]
    fn test_handle_key_event_author_switching() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Sort by email to ensure predictable order
        app_state.sort_column = SortColumn::Email;
        app_state.sort_direction = SortDirection::Ascending;

        // Select first author (alice)
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(
            app_state.selected_author,
            Some("alice@example.com".to_string())
        );

        // Navigate to second author (bob)
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Down,
            crossterm::event::KeyModifiers::NONE,
        ));

        // Select second author (should switch)
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(
            app_state.selected_author,
            Some("bob@example.com".to_string())
        );
    }

    #[test]
    fn test_handle_key_event_sort_keys() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Test sort by email
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('1'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_column, SortColumn::Email);
        assert_eq!(app_state.selected_row, 0);

        // Test sort by commits
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('2'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_column, SortColumn::Commits);
        assert_eq!(app_state.selected_row, 0);

        // Test sort by first commit
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('3'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_column, SortColumn::FirstCommit);
        assert_eq!(app_state.selected_row, 0);

        // Test sort by last commit
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('4'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_column, SortColumn::LastCommit);
        assert_eq!(app_state.selected_row, 0);

        // Test sort by days between
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('5'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_column, SortColumn::DaysBetween);
        assert_eq!(app_state.selected_row, 0);
    }

    #[test]
    fn test_handle_key_event_reverse_sort() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Test reverse sort with 'r'
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('r'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_direction, SortDirection::Descending);

        // Test reverse sort with 'R'
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('R'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_direction, SortDirection::Ascending);
    }

    #[test]
    fn test_handle_key_event_search_input() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Enable search mode
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert!(app_state.show_search);

        // Test character input
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('a'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.filter_text, "a");

        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('b'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.filter_text, "ab");

        // Test that sort keys don't work in search mode
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('1'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.sort_column, SortColumn::FirstCommit); // Should not change
    }

    #[test]
    fn test_handle_key_event_backspace() {
        let repo_data = create_test_repository_data();
        let mut app_state = AppState::new(repo_data);

        // Enable search mode and add text
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('/'),
            crossterm::event::KeyModifiers::NONE,
        ));
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('a'),
            crossterm::event::KeyModifiers::NONE,
        ));
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Char('b'),
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.filter_text, "ab");

        // Test backspace
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Backspace,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.filter_text, "a");

        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Backspace,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.filter_text, "");

        // Test backspace on empty string
        app_state.handle_key_event(KeyEvent::new(
            KeyCode::Backspace,
            crossterm::event::KeyModifiers::NONE,
        ));
        assert_eq!(app_state.filter_text, "");
    }
}
