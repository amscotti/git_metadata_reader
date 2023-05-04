# Git History Explorer

Git History Explorer is a command-line tool that reads commit metadata from a Git repository and displays the number of commits, first commit date, last commit date, and the number of days between the first and last commit for each author.

![Screenshot](screenshot.png)

## Features

- Analyze the commit history of a Git repository
- Display commit statistics for each author, including:
  - Number of commits
  - First commit date
  - Last commit date
  - Days between the first and last commit
- Sort the output by the first commit date and, in case of a tie, by the last commit date in reverse order
- Handle different time zones and daylight saving time changes
- Provide a user-friendly command-line interface

## Usage

Run the command in your terminal:

```bash
git_history_explorer [OPTIONS]
```

Options:

- `-p, --path <PATH>`: Specify the path to the Git repository (default is the current directory)

## Installation

1. Install Rust and Rustup: https://www.rust-lang.org/tools/install
2. Clone this repository: `git clone https://github.com/amscotti/git_history_explorer.git`
3. Change to the project directory: `cd git_history_explorer`
4. Build and install the tool: `cargo install --path .`

You can now run `git_history_explorer` from your command line.
