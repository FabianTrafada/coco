use anyhow::{Context, Result};
use std::path::PathBuf;
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
    pub staged_stat: String,
    pub staged_name_status: String,
    pub repo_status_short: String,
    pub raw_diff_u0: String,
}

impl StagedDiff {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Text sent to the LLM.
    ///
    /// For small changes we send the full unified diff.
    /// For large changes we use an OpenCode-style multi-command context:
    /// `git diff --staged --stat`, `git diff --staged --name-status`, `git status --short`,
    /// plus a capped `git diff --staged -U0` excerpt (so the model sees some real hunks).
    ///
    /// Set `COCO_LLM_FULL_DIFF=1` to always send the entire `raw_diff` to the model.
    pub fn context_for_llm(&self) -> (String, bool) {
        const THRESHOLD_CHARS: usize = 50_000;
        const DIFF_EXCERPT_BUDGET_CHARS: usize = 18_000;

        let raw_len = self.raw_diff.chars().count();
        let force_full = std::env::var("COCO_LLM_FULL_DIFF")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        if force_full || raw_len <= THRESHOLD_CHARS {
            return (self.raw_diff.clone(), false);
        }

        let excerpt: String = self
            .raw_diff_u0
            .chars()
            .take(DIFF_EXCERPT_BUDGET_CHARS)
            .collect();

        let mut s = String::new();
        s.push_str("Staged overview:\n\n");
        s.push_str("--stat--\n");
        s.push_str(self.staged_stat.trim_end());
        s.push_str("\n\n--name-status--\n");
        s.push_str(self.staged_name_status.trim_end());
        s.push_str("\n\n--status --short--\n");
        s.push_str(self.repo_status_short.trim_end());
        s.push_str("\n\n--diff excerpt (-U0, capped)--\n");
        s.push_str(&excerpt);
        s.push_str("\n");

        (s, true)
    }
}

fn run_git(args: &[&str]) -> Result<std::process::Output> {
    let git_bin = resolve_git_binary();

    let output = Command::new(&git_bin).args(args).output();
    match output {
        Ok(out) => Ok(out),
        Err(e) => Err(e).context(format!("Failed to run git {}", args.join(" "))),
    }
}

fn resolve_git_binary() -> PathBuf {
    // Keep the existing behavior first; many systems already have git on PATH.
    let path_candidate = PathBuf::from("git");
    if Command::new(&path_candidate).arg("--version").output().is_ok() {
        return path_candidate;
    }

    // Fallbacks for environments where PATH in GUI apps is incomplete.
    for candidate in ["/usr/bin/git", "/bin/git", "/usr/local/bin/git"] {
        let candidate_path = PathBuf::from(candidate);
        if Command::new(&candidate_path).arg("--version").output().is_ok() {
            return candidate_path;
        }
    }

    PathBuf::from("git")
}

pub fn get_staged_diff() -> Result<StagedDiff> {
    let numstat = run_git(&["diff", "--staged", "--numstat"])?;

    let namestat = run_git(&["diff", "--staged", "--name-status"])?;

    let raw_diff = run_git(&["diff", "--staged"])?;
    let staged_stat = run_git(&["diff", "--staged", "--stat"])?;
    let repo_status_short = run_git(&["status", "--short"])?;
    let raw_diff_u0 = run_git(&["diff", "--staged", "-U0"])?;
    
    let numstat_str = String::from_utf8_lossy(&numstat.stdout);
    let namestat_str = String::from_utf8_lossy(&namestat.stdout);
    let raw_diff_str = String::from_utf8_lossy(&raw_diff.stdout).to_string();
    let staged_stat_str = String::from_utf8_lossy(&staged_stat.stdout).to_string();
    let repo_status_short_str = String::from_utf8_lossy(&repo_status_short.stdout).to_string();
    let raw_diff_u0_str = String::from_utf8_lossy(&raw_diff_u0.stdout).to_string();

    let files = parse_diff(numstat_str.as_ref(), namestat_str.as_ref());

    let total_additions = files.iter().map(|f| f.additions).sum();
    let total_deletions = files.iter().map(|f| f.deletions).sum();
    
    Ok(StagedDiff { 
        files, 
        total_additions, 
        total_deletions, 
        raw_diff: raw_diff_str,
        staged_stat: staged_stat_str,
        staged_name_status: namestat_str.to_string(),
        repo_status_short: repo_status_short_str,
        raw_diff_u0: raw_diff_u0_str,
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
    let git_bin = resolve_git_binary();
    let output = Command::new(git_bin)
        .args(["commit", "-m", message])
        .output()
        .context("Failed to run git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git commit failed: {}", stderr);
    }

    Ok(())
}