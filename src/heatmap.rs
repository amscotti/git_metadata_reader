use chrono::{Datelike, Duration, NaiveDate, Utc};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct HeatMapData {
    pub commits_by_date: HashMap<NaiveDate, u32>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub max_commits: u32,
}

impl Default for HeatMapData {
    fn default() -> Self {
        Self {
            commits_by_date: HashMap::new(),
            start_date: NaiveDate::from_ymd_opt(2099, 1, 1).unwrap(), // Will be updated with actual data
            end_date: NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(), // Will be updated with actual data
            max_commits: 0,
        }
    }
}

impl HeatMapData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_commits(&mut self, date: NaiveDate, count: u32) {
        *self.commits_by_date.entry(date).or_insert(0) += count;
        self.max_commits = self.max_commits.max(self.commits_by_date[&date]);

        // Update date range dynamically
        if date < self.start_date {
            self.start_date = date;
        }
        if date > self.end_date {
            self.end_date = date;
        }
    }

    pub fn get_commits(&self, date: NaiveDate) -> u32 {
        *self.commits_by_date.get(&date).unwrap_or(&0)
    }

    pub fn get_intensity_level(&self, commits: u32) -> u8 {
        if commits == 0 {
            0
        } else if self.max_commits <= 4 {
            commits.min(4) as u8
        } else {
            // Use quartiles for intensity levels
            let quartile = self.max_commits / 4;
            match commits {
                c if c <= quartile => 1,
                c if c <= quartile * 2 => 2,
                c if c <= quartile * 3 => 3,
                _ => 4,
            }
        }
    }

    pub fn create_from_timeline_data(
        timeline_data: &crate::user_commit_info::TimelineData,
    ) -> Self {
        let mut heatmap = Self::new();
        let current_year = Utc::now().date_naive().year();

        // Map each historical commit date to the current year calendar
        for (historical_date, commits) in &timeline_data.commits_by_period {
            let calendar_date = chrono::NaiveDate::from_ymd_opt(
                current_year,
                historical_date.month(),
                historical_date.day(),
            )
            .unwrap_or(*historical_date); // fallback to original date if invalid (e.g., Feb 29)

            heatmap.add_commits(calendar_date, *commits);
        }

        heatmap
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_commit_info::TimelineData;
    use chrono::NaiveDate;

    #[test]
    fn test_heatmap_data_new() {
        let heatmap = HeatMapData::new();

        assert!(heatmap.commits_by_date.is_empty());
        assert_eq!(
            heatmap.start_date,
            NaiveDate::from_ymd_opt(2099, 1, 1).unwrap()
        );
        assert_eq!(
            heatmap.end_date,
            NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()
        );
        assert_eq!(heatmap.max_commits, 0);
    }

    #[test]
    fn test_heatmap_data_add_commits() {
        let mut heatmap = HeatMapData::new();
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let count = 5;

        heatmap.add_commits(date, count);

        assert_eq!(heatmap.get_commits(date), count);
        assert_eq!(heatmap.max_commits, count);
        assert_eq!(heatmap.start_date, date);
        assert_eq!(heatmap.end_date, date);
    }

    #[test]
    fn test_heatmap_data_add_commits_multiple_dates() {
        let mut heatmap = HeatMapData::new();
        let date1 = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2023, 1, 2).unwrap();

        heatmap.add_commits(date1, 3);
        heatmap.add_commits(date2, 7);

        assert_eq!(heatmap.get_commits(date1), 3);
        assert_eq!(heatmap.get_commits(date2), 7);
        assert_eq!(heatmap.max_commits, 7);
        assert_eq!(heatmap.start_date, date1);
        assert_eq!(heatmap.end_date, date2);
    }

    #[test]
    fn test_heatmap_data_add_commits_same_date() {
        let mut heatmap = HeatMapData::new();
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        heatmap.add_commits(date, 3);
        heatmap.add_commits(date, 2);

        assert_eq!(heatmap.get_commits(date), 5);
        assert_eq!(heatmap.max_commits, 5);
    }

    #[test]
    fn test_heatmap_data_get_commits_nonexistent_date() {
        let heatmap = HeatMapData::new();
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

        assert_eq!(heatmap.get_commits(date), 0);
    }

    #[test]
    fn test_heatmap_data_get_intensity_level_zero() {
        let heatmap = HeatMapData::new();

        assert_eq!(heatmap.get_intensity_level(0), 0);
    }

    #[test]
    fn test_heatmap_data_get_intensity_level_small_max() {
        let mut heatmap = HeatMapData::new();
        heatmap.add_commits(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 3);

        assert_eq!(heatmap.get_intensity_level(0), 0);
        assert_eq!(heatmap.get_intensity_level(1), 1);
        assert_eq!(heatmap.get_intensity_level(2), 2);
        assert_eq!(heatmap.get_intensity_level(3), 3);
        assert_eq!(heatmap.get_intensity_level(4), 4); // When max_commits <= 4, it returns commits.min(4)
        assert_eq!(heatmap.get_intensity_level(5), 4); // When max_commits <= 4, it returns commits.min(4)
    }

    #[test]
    fn test_heatmap_data_get_intensity_level_quartiles() {
        let mut heatmap = HeatMapData::new();
        heatmap.add_commits(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 20); // max_commits = 20

        // Quartiles: 5, 10, 15
        assert_eq!(heatmap.get_intensity_level(0), 0);
        assert_eq!(heatmap.get_intensity_level(3), 1); // <= 5
        assert_eq!(heatmap.get_intensity_level(7), 2); // <= 10
        assert_eq!(heatmap.get_intensity_level(12), 3); // <= 15
        assert_eq!(heatmap.get_intensity_level(18), 4); // > 15
    }

    #[test]
    fn test_heatmap_data_get_intensity_level_edge_cases() {
        let mut heatmap = HeatMapData::new();
        heatmap.add_commits(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 8); // max_commits = 8

        // Quartiles: 2, 4, 6
        assert_eq!(heatmap.get_intensity_level(1), 1); // <= 2
        assert_eq!(heatmap.get_intensity_level(2), 1); // <= 2
        assert_eq!(heatmap.get_intensity_level(3), 2); // <= 4
        assert_eq!(heatmap.get_intensity_level(4), 2); // <= 4
        assert_eq!(heatmap.get_intensity_level(5), 3); // <= 6
        assert_eq!(heatmap.get_intensity_level(6), 3); // <= 6
        assert_eq!(heatmap.get_intensity_level(7), 4); // > 6
        assert_eq!(heatmap.get_intensity_level(8), 4); // > 6
    }

    #[test]
    fn test_heatmap_data_create_from_timeline_data() {
        let mut timeline = TimelineData::default();
        timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 3);
        timeline.add_commit(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), 5);

        let heatmap = HeatMapData::create_from_timeline_data(&timeline);

        // Should map to current year calendar
        let current_year = chrono::Utc::now().date_naive().year();
        let expected_date1 = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        let expected_date2 = NaiveDate::from_ymd_opt(current_year, 1, 2).unwrap();

        assert_eq!(heatmap.get_commits(expected_date1), 3);
        assert_eq!(heatmap.get_commits(expected_date2), 5);
        assert_eq!(heatmap.max_commits, 5);
    }

    #[test]
    fn test_heatmap_data_leap_year_handling() {
        let mut timeline = TimelineData::default();
        // Feb 29 from a leap year should map to a valid date
        timeline.add_commit(NaiveDate::from_ymd_opt(2020, 2, 29).unwrap(), 2);

        let heatmap = HeatMapData::create_from_timeline_data(&timeline);

        // Should fallback to a valid date (likely Feb 28 or Mar 1 in non-leap years)
        // The important thing is that it doesn't panic
        assert!(heatmap.max_commits > 0);
    }

    #[test]
    fn test_heatmap_data_create_from_empty_timeline() {
        let timeline = TimelineData::default();
        let heatmap = HeatMapData::create_from_timeline_data(&timeline);

        assert!(heatmap.commits_by_date.is_empty());
        assert_eq!(heatmap.max_commits, 0);
    }

    #[test]
    fn test_heatmap_data_date_range_updates() {
        let mut heatmap = HeatMapData::new();
        let early_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let late_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();

        heatmap.add_commits(late_date, 5);
        assert_eq!(heatmap.start_date, late_date);
        assert_eq!(heatmap.end_date, late_date);

        heatmap.add_commits(early_date, 3);
        assert_eq!(heatmap.start_date, early_date);
        assert_eq!(heatmap.end_date, late_date);
    }
}

pub struct HeatMap<'a> {
    data: &'a HeatMapData,
    block: Option<Block<'a>>,
}

impl<'a> HeatMap<'a> {
    pub fn new(data: &'a HeatMapData) -> Self {
        Self { data, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn get_color_for_intensity(intensity: u8) -> Color {
        match intensity {
            0 => Color::Rgb(40, 40, 40), // Dark gray - no commits (same as empty cells)
            1 => Color::Rgb(14, 68, 41), // Dark green
            2 => Color::Rgb(0, 109, 50), // Medium green
            3 => Color::Rgb(38, 166, 65), // Bright green
            4 => Color::Rgb(57, 211, 83), // Very bright green
            _ => Color::Rgb(40, 40, 40), // Fallback
        }
    }

    fn create_heatmap_lines(&self, _area_width: u16) -> Vec<Line<'a>> {
        let mut lines = Vec::new();

        if self.data.commits_by_date.is_empty() {
            lines.push(Line::from("No commit data available"));
            return lines;
        }

        // Static calendar year: show current year from Jan 1 to Dec 31
        let current_year = Utc::now().date_naive().year();
        let jan_1 = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();

        // Find Sunday of the week containing January 1st
        let mut grid_start = jan_1;
        while grid_start.weekday().num_days_from_sunday() != 0 {
            grid_start -= Duration::days(1);
        }

        // Always show exactly 52 weeks (full year grid)
        let weeks_to_show = 52;

        // Create month header line
        let mut month_spans = Vec::new();
        month_spans.push(Span::styled("      ", Style::default())); // Space for day labels

        // Static month layout: calculate exact positions for Jan-Dec
        let month_names = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let mut month_positions = Vec::new();

        // Find the week where each month starts
        for month in 1..=12 {
            let month_start = NaiveDate::from_ymd_opt(current_year, month, 1).unwrap();

            // Find which week this month starts in
            let days_from_grid_start = (month_start - grid_start).num_days();
            let week_position = (days_from_grid_start / 7) as usize;

            if week_position < weeks_to_show {
                month_positions.push((week_position, month as usize - 1)); // 0-indexed for array
            }
        }

        // Show only every other month for better alignment (Jan, Mar, May, Jul, Sep, Nov)
        let months_to_display: Vec<usize> = vec![0, 2, 4, 6, 8, 10]; // Jan, Mar, May, Jul, Sep, Nov (0-indexed)

        // Create month header spans with selective months
        for week in 0..weeks_to_show {
            if let Some((_, month_idx)) = month_positions.iter().find(|(w, _)| *w == week) {
                if months_to_display.contains(month_idx) {
                    month_spans.push(Span::styled(
                        format!("{:>2}", month_names[*month_idx]),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    month_spans.push(Span::styled("  ", Style::default()));
                }
            } else {
                month_spans.push(Span::styled("  ", Style::default()));
            }
        }

        lines.push(Line::from(month_spans));

        // Create 7 rows for days of the week
        let day_labels = ["", "Mon", "", "Wed", "", "Fri", ""];

        for (day_of_week, day_label) in day_labels.iter().enumerate() {
            let mut spans = Vec::new();

            // Add day label (only for Mon, Wed, Fri)
            let day_label = *day_label;
            spans.push(Span::styled(
                format!("{day_label:>3}   "),
                Style::default().fg(Color::Gray),
            ));

            // Add squares for each week in the calendar year
            for week in 0..weeks_to_show {
                let current_date =
                    grid_start + Duration::weeks(week as i64) + Duration::days(day_of_week as i64);

                let commits = self.data.get_commits(current_date);
                let intensity = self.data.get_intensity_level(commits);
                let color = Self::get_color_for_intensity(intensity);

                // Use single square blocks with space for distinct cells
                spans.push(Span::styled("■ ", Style::default().fg(color)));
            }

            lines.push(Line::from(spans));
        }

        // Add some spacing before legend
        lines.push(Line::from(""));

        // Create legend line
        let mut legend_spans = Vec::new();
        legend_spans.push(Span::styled("Less ", Style::default().fg(Color::Gray)));

        for i in 0..5 {
            let color = Self::get_color_for_intensity(i);
            legend_spans.push(Span::styled("■", Style::default().fg(color)));
        }

        legend_spans.push(Span::styled(" More", Style::default().fg(Color::Gray)));

        lines.push(Line::from(legend_spans));

        lines
    }
}

impl<'a> Widget for HeatMap<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let lines = self.create_heatmap_lines(area.width);
        let text = ratatui::text::Text::from(lines);

        let paragraph = Paragraph::new(text).block(self.block.unwrap_or_default());
        paragraph.render(area, buf);
    }
}

pub fn render_heatmap(f: &mut Frame, area: Rect, heatmap_data: &HeatMapData) {
    let heatmap_block = Block::default()
        .title(" Commit Activity ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let heatmap = HeatMap::new(heatmap_data).block(heatmap_block);
    f.render_widget(heatmap, area);
}
