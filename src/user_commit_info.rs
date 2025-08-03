use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct CommitData {
    pub email: String,
    pub commits: u32,
    pub first_commit: NaiveDate,
    pub last_commit: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct TimelineData {
    pub commits_by_period: std::collections::HashMap<NaiveDate, u32>,
    pub total_commits: u32,
    pub first_commit: NaiveDate,
    pub last_commit: NaiveDate,
}

impl Default for TimelineData {
    fn default() -> Self {
        Self {
            commits_by_period: std::collections::HashMap::new(),
            total_commits: 0,
            first_commit: NaiveDate::from_ymd_opt(2099, 1, 1).unwrap(),
            last_commit: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
        }
    }
}

impl TimelineData {
    pub fn add_commit(&mut self, date: NaiveDate, commits: u32) {
        *self.commits_by_period.entry(date).or_insert(0) += commits;
        self.total_commits += commits;

        if date < self.first_commit {
            self.first_commit = date;
        }
        if date > self.last_commit {
            self.last_commit = date;
        }
    }
}

impl CommitData {
    pub fn new(
        email: String,
        commits: u32,
        first_commit: NaiveDate,
        last_commit: NaiveDate,
    ) -> Self {
        Self {
            email,
            commits,
            first_commit,
            last_commit,
        }
    }

    pub fn days_between(&self) -> i64 {
        (self.last_commit - self.first_commit).num_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_commit_data_creation() {
        let email = "test@example.com".to_string();
        let commits = 10;
        let first_commit = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let last_commit = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();

        let commit_data = CommitData::new(email.clone(), commits, first_commit, last_commit);

        assert_eq!(commit_data.email, email);
        assert_eq!(commit_data.commits, commits);
        assert_eq!(commit_data.first_commit, first_commit);
        assert_eq!(commit_data.last_commit, last_commit);
    }

    #[test]
    fn test_commit_data_days_between() {
        let first_commit = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let last_commit = NaiveDate::from_ymd_opt(2023, 1, 11).unwrap(); // 10 days later

        let commit_data =
            CommitData::new("test@example.com".to_string(), 5, first_commit, last_commit);

        assert_eq!(commit_data.days_between(), 10);
    }

    #[test]
    fn test_commit_data_days_between_same_day() {
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        let commit_data = CommitData::new("test@example.com".to_string(), 3, date, date);

        assert_eq!(commit_data.days_between(), 0);
    }

    #[test]
    fn test_commit_data_days_between_negative() {
        let first_commit = NaiveDate::from_ymd_opt(2023, 1, 11).unwrap();
        let last_commit = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(); // Earlier date

        let commit_data =
            CommitData::new("test@example.com".to_string(), 2, first_commit, last_commit);

        // Should still work even if dates are "backwards"
        assert_eq!(commit_data.days_between(), -10);
    }

    #[test]
    fn test_timeline_data_default() {
        let timeline = TimelineData::default();

        assert!(timeline.commits_by_period.is_empty());
        assert_eq!(timeline.total_commits, 0);
        assert_eq!(
            timeline.first_commit,
            NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()
        );
        assert_eq!(
            timeline.last_commit,
            NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()
        );
    }

    #[test]
    fn test_timeline_data_add_commit() {
        let mut timeline = TimelineData::default();
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let commits = 3;

        timeline.add_commit(date, commits);

        assert_eq!(timeline.total_commits, commits);
        assert_eq!(timeline.commits_by_period.get(&date), Some(&commits));
        assert_eq!(timeline.first_commit, date);
        assert_eq!(timeline.last_commit, date);
    }

    #[test]
    fn test_timeline_data_add_multiple_commits_same_date() {
        let mut timeline = TimelineData::default();
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        timeline.add_commit(date, 3);
        timeline.add_commit(date, 2);

        assert_eq!(timeline.total_commits, 5);
        assert_eq!(timeline.commits_by_period.get(&date), Some(&5));
        assert_eq!(timeline.first_commit, date);
        assert_eq!(timeline.last_commit, date);
    }

    #[test]
    fn test_timeline_data_date_range_updates() {
        let mut timeline = TimelineData::default();
        let early_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let middle_date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
        let late_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();

        timeline.add_commit(middle_date, 2);
        assert_eq!(timeline.first_commit, middle_date);
        assert_eq!(timeline.last_commit, middle_date);

        timeline.add_commit(early_date, 1);
        assert_eq!(timeline.first_commit, early_date);
        assert_eq!(timeline.last_commit, middle_date);

        timeline.add_commit(late_date, 4);
        assert_eq!(timeline.first_commit, early_date);
        assert_eq!(timeline.last_commit, late_date);
    }

    #[test]
    fn test_timeline_data_total_commits() {
        let mut timeline = TimelineData::default();

        timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 3);
        timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), 2);
        timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 3).unwrap(), 5);

        assert_eq!(timeline.total_commits, 10);
    }

    #[test]
    fn test_timeline_data_multiple_dates() {
        let mut timeline = TimelineData::default();
        let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap();

        timeline.add_commit(date1, 3);
        timeline.add_commit(date2, 2);

        assert_eq!(timeline.total_commits, 5);
        assert_eq!(timeline.commits_by_period.get(&date1), Some(&3));
        assert_eq!(timeline.commits_by_period.get(&date2), Some(&2));
        assert_eq!(timeline.first_commit, date1);
        assert_eq!(timeline.last_commit, date2);
    }
}
