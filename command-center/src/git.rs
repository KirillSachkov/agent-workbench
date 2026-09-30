//! Facts from git in an agent's working directory.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::env::canonical;
use crate::run::run;

const TIMEOUT: Duration = Duration::from_secs(10);

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = run("git", args, Some(dir), Some(TIMEOUT));
    out.ok.then(|| out.stdout.trim().to_string())
}

#[derive(Debug, Clone)]
pub struct Facts {
    /// The checkout the agent works in (a main or linked worktree).
    pub worktree: PathBuf,
    /// The main worktree: the project.
    pub project_root: PathBuf,
    pub linked: bool,
    pub branch: Option<String>,
    /// The ref the branch is compared against, for example `origin/main`.
    pub base: Option<String>,
    pub merge_base: Option<String>,
    /// `owner/name` when the `origin` remote is on GitHub.
    pub github: Option<String>,
}

pub fn facts(cwd: &Path) -> Option<Facts> {
    let worktree = canonical(Path::new(&git(cwd, &["rev-parse", "--show-toplevel"])?));
    let common = git(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let common = canonical(Path::new(&common));
    let project_root = if common.file_name().is_some_and(|n| n == ".git") {
        common
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(common.clone())
    } else {
        common.clone()
    };
    let branch =
        git(cwd, &["rev-parse", "--abbrev-ref", "HEAD"]).filter(|b| b != "HEAD" && !b.is_empty());
    let base = default_base(cwd);
    let merge_base = base
        .as_deref()
        .and_then(|b| git(cwd, &["merge-base", b, "HEAD"]));
    let github = git(cwd, &["remote", "get-url", "origin"]).and_then(|url| github_slug(&url));
    Some(Facts {
        linked: worktree != project_root,
        worktree,
        project_root,
        branch,
        base,
        merge_base,
        github,
    })
}

fn default_base(cwd: &Path) -> Option<String> {
    if let Some(head) = git(
        cwd,
        &[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    ) {
        return Some(head);
    }
    ["origin/main", "origin/master", "main", "master"]
        .into_iter()
        .find(|candidate| {
            git(
                cwd,
                &[
                    "rev-parse",
                    "--verify",
                    "--quiet",
                    &format!("{candidate}^{{commit}}"),
                ],
            )
            .is_some()
        })
        .map(str::to_string)
}

/// `owner/name` from a GitHub remote URL in https, ssh or scp form.
pub fn github_slug(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .or_else(|| url.strip_prefix("git@github.com:"))
        .or_else(|| url.strip_prefix("ssh://git@github.com/"))?;
    let rest = rest.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = rest.split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    (parts.next().is_none() && !owner.is_empty() && !name.is_empty())
        .then(|| format!("{owner}/{name}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub additions: u64,
    pub deletions: u64,
}

/// Files changed in the worktree since the merge base: committed, uncommitted and new files.
pub fn changed_files(worktree: &Path, merge_base: &str) -> Vec<FileChange> {
    let mut files: Vec<FileChange> = git(worktree, &["diff", "--numstat", merge_base])
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut cols = line.splitn(3, '\t');
            let additions = cols.next()?.parse().unwrap_or(0);
            let deletions = cols.next()?.parse().unwrap_or(0);
            Some(FileChange {
                path: cols.next()?.to_string(),
                additions,
                deletions,
            })
        })
        .collect();
    let untracked =
        git(worktree, &["ls-files", "--others", "--exclude-standard"]).unwrap_or_default();
    for path in untracked.lines().filter(|p| !p.is_empty()) {
        let lines = std::fs::read(worktree.join(path))
            .map(|b| b.iter().filter(|&&c| c == b'\n').count())
            .unwrap_or(0);
        files.push(FileChange {
            path: path.to_string(),
            additions: lines as u64,
            deletions: 0,
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

pub fn head(worktree: &Path) -> Option<String> {
    git(worktree, &["rev-parse", "HEAD"])
}
