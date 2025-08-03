use crate::heatmap::render_heatmap;
use crate::tui::{AppState, SortColumn, SortDirection};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};
use std::path::Path;

pub fn render_app(f: &mut Frame, app_state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Percentage(25), // Heatmap (25% of height)
            Constraint::Min(0),         // Author section (remaining space)
            Constraint::Length(3),      // Footer
        ])
        .split(f.area());

    render_header(f, chunks[0], app_state);
    render_heatmap(f, chunks[1], app_state.get_filtered_heatmap_data());
    render_author_section(f, chunks[2], app_state);
    render_footer(f, chunks[3], app_state);
}

fn render_header(f: &mut Frame, area: Rect, app_state: &AppState) {
    let title = Block::default()
        .title(" Git History Explorer ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let mut header_text = Text::default();
    header_text.push_line(Line::from(vec![
        Span::styled("Repository: ", Style::default().fg(Color::Cyan)),
        Span::raw(format_repo_path(&app_state.repository_data.repo_path)),
    ]));

    let sorted_data = app_state.sorted_data();
    let mut author_info = Line::from(vec![
        Span::styled("Total Authors: ", Style::default().fg(Color::Cyan)),
        Span::raw(sorted_data.len().to_string()),
    ]);

    if let Some(selected_email) = &app_state.selected_author {
        author_info.spans.push(Span::raw(" | "));
        author_info.spans.push(Span::styled(
            "Selected: ",
            Style::default().fg(Color::Yellow),
        ));
        author_info.spans.push(Span::raw(selected_email.clone()));
    }

    header_text.push_line(author_info);

    let paragraph = Paragraph::new(header_text)
        .block(title)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_author_section(f: &mut Frame, area: Rect, app_state: &mut AppState) {
    // Authors list now takes the full width (no horizontal split)
    render_author_list(f, area, app_state);
}

fn render_author_list(f: &mut Frame, area: Rect, app_state: &mut AppState) {
    let sorted_data = app_state.sorted_data();

    if sorted_data.is_empty() {
        let empty_block = Block::default()
            .title(" Authors ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let empty_text = Text::from("No commit data found");
        let paragraph = Paragraph::new(empty_text).block(empty_block);
        f.render_widget(paragraph, area);
        return;
    }

    let header_cells = ["Email", "Commits", "First", "Last", "Days"]
        .iter()
        .enumerate()
        .map(|(i, &header)| {
            let is_sorted = matches!(
                (i, app_state.sort_column),
                (0, SortColumn::Email)
                    | (1, SortColumn::Commits)
                    | (2, SortColumn::FirstCommit)
                    | (3, SortColumn::LastCommit)
                    | (4, SortColumn::DaysBetween)
            );

            let style = if is_sorted {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let mut text = header.to_string();
            if is_sorted {
                text.push(match app_state.sort_direction {
                    SortDirection::Ascending => '↑',
                    SortDirection::Descending => '↓',
                });
            }

            Cell::from(text).style(style)
        });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1);

    let rows = sorted_data.iter().enumerate().map(|(index, data)| {
        let is_selected = index == app_state.selected_row;
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if index % 2 == 0 {
            Style::default().bg(Color::Rgb(20, 20, 30))
        } else {
            Style::default().bg(Color::Rgb(10, 10, 20))
        };

        Row::new(vec![
            Cell::from(data.email.clone()),
            Cell::from(data.commits.to_string()),
            Cell::from(data.first_commit.format("%m/%d/%Y").to_string()),
            Cell::from(data.last_commit.format("%m/%d/%Y").to_string()),
            Cell::from(data.days_between().to_string()),
        ])
        .style(style)
        .height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Authors ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    )
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .widths([
        Constraint::Percentage(40),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
    ]);

    let mut table_state = TableState::default();
    table_state.select(Some(app_state.selected_row));
    f.render_stateful_widget(table, area, &mut table_state);
}

fn render_footer(f: &mut Frame, area: Rect, app_state: &AppState) {
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let mut footer_lines = Vec::new();

    if app_state.show_search {
        footer_lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app_state.filter_text),
            Span::raw("_"),
        ]));
    } else {
        let controls = vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" Select "),
            Span::styled("1-5", Style::default().fg(Color::Cyan)),
            Span::raw(" Sort "),
            Span::styled("R", Style::default().fg(Color::Cyan)),
            Span::raw(" Reverse "),
            Span::styled("/", Style::default().fg(Color::Cyan)),
            Span::raw(" Search "),
            Span::styled("Q", Style::default().fg(Color::Cyan)),
            Span::raw(" Quit"),
        ];

        footer_lines.push(Line::from(controls));
    }

    if let Some(error) = &app_state.error_message {
        footer_lines.push(Line::from(vec![
            Span::styled("Error: ", Style::default().fg(Color::Red)),
            Span::raw(error.clone()),
        ]));
    }

    let footer_text = Text::from(footer_lines);
    let paragraph = Paragraph::new(footer_text)
        .block(footer_block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn format_repo_path(path: &str) -> String {
    if path == "." {
        "Current Directory".to_string()
    } else if path.len() > 50 {
        // Show just the directory name if path is very long
        Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string())
    } else {
        path.to_string()
    }
}
