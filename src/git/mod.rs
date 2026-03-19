use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub status: char,  // M, A, D, R
    pub path: String,
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Debug, Clone)]
pub struct StagedDiff {
    pub files: Vec<FileDiff>,
    pub total_additions: i32,
    pub total_deletions: i32,
    pub raw_diff: String,
}

impl StagedDiff {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

pub fn get_staged_diff() -> Result<StagedDiff> {
    let numstat = Command::new("git")
        .args(["diff", "--staged", "--numstat"])
        .output()
        .context("Failed to run git diff --numstat")?;

    let namestat = Command::new("git")
        .args(["diff", "--staged", "--name-status"])
        .output()
        .context("Failed to run git diff --name-status")?;

    let raw_diff = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .context("Failed to run git diff --staged")?;
    
    let numstat_str = String::from_utf8_lossy(&numstat.stdout);
    let namestat_str = String::from_utf8_lossy(&namestat.stdout);
    let raw_diff_str = String::from_utf8_lossy(&raw_diff.stdout).to_string();

    let files = parse_diff(numstat_str.as_ref(), namestat_str.as_ref());

    let total_additions = files.iter().map(|f| f.additions).sum();
    let total_deletions = files.iter().map(|f| f.deletions).sum();
    
    Ok(StagedDiff { 
        files, 
        total_additions, 
        total_deletions, 
        raw_diff: raw_diff_str 
    })
}

fn parse_diff(numstat: &str, namestat: &str) -> Vec<FileDiff> {
    // numstat format: "84\t12\tsrc/auth/token.rs"
    // namestat format: "M\tsrc/auth/token.rs"

    let statuses: Vec<(char, &str)> = namestat
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let status = parts.next()?.chars().next()?;
            let path = parts.next()?.trim();
            Some((status, path))
        })
        .collect();

    numstat
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let additions: i32 = parts.next()?.parse().unwrap_or(0);
            let deletions: i32 = parts.next()?.parse().unwrap_or(0);
            let path = parts.next()?.trim().to_string();

            let status = statuses
                .iter()
                .find(|(_, p)| *p == path)
                .map(|(s, _)| *s)
                .unwrap_or('M');

            Some(FileDiff {
                status,
                path,
                additions,
                deletions,
            })
        })
        .collect()
}

pub fn commit(message: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .context("Failed to run git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git commit failed: {}", stderr);
    }

    Ok(())
}