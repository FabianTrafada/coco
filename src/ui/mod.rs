use std::io::{self, Write};

use owo_colors::OwoColorize;

use crate::git::{FileDiff, StagedDiff};

pub fn print_staged_diff(diff: &StagedDiff) {
    println!();

    let max_path_len = diff.files.iter().map(|f| f.path.len()).max().unwrap_or(0);
    let max_add_len = diff
        .files
        .iter()
        .map(|f| format!("+{}", f.additions).len())
        .max()
        .unwrap_or(0);
    let max_del_len = diff
        .files
        .iter()
        .map(|f| format!("-{}", f.deletions).len())
        .max()
        .unwrap_or(0);

    // 2 + 1(status) + 2 + max_path + 2 + max_add + 2 + max_del + 2 = +11
    let inner_width = max_path_len + max_add_len + max_del_len + 11;
    let border = "─".repeat(inner_width);

    println!("  ┌{}┐", border);

    for file in &diff.files {
        let (status_colored, path_colored) = colorize_status(file);
        let additions_colored = colorize_additions(file);
        let deletions_colored = colorize_deletions(file);

        // padding dihitung dari raw string, bukan colored
        let path_pad = " ".repeat(max_path_len - file.path.len());
        let add_pad  = " ".repeat(max_add_len - format!("+{}", file.additions).len());
        let del_pad  = " ".repeat(max_del_len - format!("-{}", file.deletions).len());

        println!(
            "  │  {}  {}{}  {}{}  {}{}  │",
            status_colored,
            path_colored,
            path_pad,
            add_pad,
            additions_colored,
            del_pad,
            deletions_colored,
        );
    }

    println!("  └{}┘", border);

    let total_add_pad = " ".repeat(max_add_len - format!("+{}", diff.total_additions).len());
    let total_del_pad = " ".repeat(max_del_len - format!("-{}", diff.total_deletions).len());

    println!(
        "     {} files changed  {}{}  {}{}",
        diff.files.len(),
        total_add_pad,
        format!("+{}", diff.total_additions).green(),
        total_del_pad,
        format!("-{}", diff.total_deletions).red(),
    );

    println!();
}

pub fn print_suggested_message(message: &str) {
    println!("  Suggested commit message:");

    let lines: Vec<&str> = message.lines().collect();
    let width = lines.iter().map(|l| l.len()).max().unwrap_or(0) + 4;
    let border = "─".repeat(width);

    println!("  ┌{}┐", border);
    for line in &lines {
        let pad = width - line.len() - 2;
        println!("  │ {}{:pad$} │", line, "", pad = pad);
    }
    println!("  └{}┘", border);
    println!();
}

pub fn print_analyzing() {
    println!();
    println!("  {} Analyzing staged changes...", "✦".bright_purple());
    println!();
}

pub fn print_committed() {
    println!();
    println!("  {} Committed.", "✦".bright_purple());
    println!();
}

pub fn print_aborted() {
    println!();
    println!("  {} Aborted.", "✦".dimmed());
    println!();
}

pub fn prompt_action() -> Result<Action, io::Error> {
    print!(
        "  {}  {}  {}  {}\n  > ",
        "[C]ommit".bright_blue(),
        "[E]dit".white(),
        "[R]egenerate".white(),
        "[A]bort".red(),
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "c" => Ok(Action::Commit),
        "e" => Ok(Action::Edit),
        "r" => Ok(Action::Regenerate),
        "a" => Ok(Action::Abort),
        _ => Ok(Action::Unknown),
    }
}

pub fn prompt_edit(current: &str) -> Result<String, io::Error> {
    println!("  Edit commit message (press Enter to confirm):");
    println!("  Current: {}", current.dimmed());
    print!("  > ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        Ok(current.to_string())
    } else {
        Ok(trimmed)
    }
}

pub fn print_error(msg: &str) {
    eprintln!();
    eprintln!("  {} {}", "✗".red(), msg.red());
    eprintln!();
}

pub fn print_no_staged_changes() {
    println!();
    println!(
        "  {} No staged changes found. Use {} first.",
        "✦".yellow(),
        "git add".bright_white()
    );
    println!();
}

pub enum Action {
    Commit,
    Edit,
    Regenerate,
    Abort,
    Unknown,
}

// -- helpers --

fn colorize_status(file: &FileDiff) -> (String, String) {
    match file.status {
        'A' => (
            file.status.to_string().green().to_string(),
            file.path.green().to_string(),
        ),
        'D' => (
            file.status.to_string().red().to_string(),
            file.path.red().to_string(),
        ),
        'R' => (
            file.status.to_string().bright_blue().to_string(),
            file.path.bright_blue().to_string(),
        ),
        _ => (
            file.status.to_string().yellow().to_string(),
            file.path.yellow().to_string(),
        ),
    }
}

fn colorize_additions(file: &FileDiff) -> String {
    if file.additions == 0 {
        format!("+{}", file.additions).dimmed().to_string()
    } else {
        format!("+{}", file.additions).green().to_string()
    }
}

fn colorize_deletions(file: &FileDiff) -> String {
    if file.deletions == 0 {
        format!("-{}", file.deletions).dimmed().to_string()
    } else {
        format!("-{}", file.deletions).red().to_string()
    }
}