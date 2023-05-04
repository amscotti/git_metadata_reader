use chrono::NaiveDate;

#[derive(Debug)]
pub struct UserCommitInfo {
    pub commits: u32,
    pub first_commit: NaiveDate,
    pub last_commit: NaiveDate,
}

impl UserCommitInfo {
    pub fn new(commit_time: NaiveDate) -> Self {
        UserCommitInfo {
            commits: 1,
            first_commit: commit_time,
            last_commit: commit_time,
        }
    }

    pub fn update(&mut self, commit_time: NaiveDate) {
        self.commits += 1;

        if commit_time < self.first_commit {
            self.first_commit = commit_time;
        }

        if commit_time > self.last_commit {
            self.last_commit = commit_time;
        }
    }

    pub fn days_between(&self) -> i64 {
        (self.last_commit - self.first_commit).num_days()
    }
}

#[cfg(test)]
mod tests {
    // this brings everything from parent's scope into this scope
    use super::*;

    #[test]
    fn test_update() {
        let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2023, 1, 20).unwrap();

        let mut user_commit_info = UserCommitInfo::new(date1);

        user_commit_info.update(date2);
        assert_eq!(user_commit_info.commits, 2);
        assert_eq!(user_commit_info.first_commit, date1);
        assert_eq!(user_commit_info.last_commit, date2);

        user_commit_info.update(date3);
        assert_eq!(user_commit_info.commits, 3);
        assert_eq!(user_commit_info.first_commit, date1);
        assert_eq!(user_commit_info.last_commit, date3);
    }

    #[test]
    fn test_days_between() {
        let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();

        let user_commit_info = UserCommitInfo {
            commits: 2,
            first_commit: date1,
            last_commit: date2,
        };

        assert_eq!(user_commit_info.days_between(), (date2 - date1).num_days());
    }
}
