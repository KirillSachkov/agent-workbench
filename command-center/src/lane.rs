//! Lanes: one executor per issue, in its own worktree and Herdr workspace. The `conduct` skill
//! starts a lane only after the owner's go-ahead; these commands do the mechanics.
//!
//! There is no lane store: a lane is an agent in a linked worktree whose branch names its issue,
//! found again from Herdr, git and GitHub on every call. Nothing here merges, pushes, deletes a
//! branch or worktree, answers an agent's question or approval, or types into a working agent.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;

use crate::env::{self, Config, canonical, fill};
use crate::github::{self, Settings};
use crate::model::{self, AgentView, Options, PrRef, task_from_branch};
use crate::{git, herdr};

pub struct StartArgs {
    pub issue: u64,
    pub agent: String,
    pub repo: Option<PathBuf>,
    pub branch: Option<String>,
    /// Extra instructions appended to the generated brief.
    pub brief: Option<PathBuf>,
    pub over_limit: bool,
}

#[derive(Debug, Serialize)]
pub struct Started {
    pub lane: String,
    pub issue: u64,
    pub title: String,
    pub agent: String,
    pub workspace_id: String,
    pub pane_id: String,
    pub branch: String,
    pub worktree: String,
    pub brief: String,
    /// `started`, or `waits for you` when the agent stopped at a question during startup.
    pub status: String,
    pub claimed: bool,
    pub claim_error: Option<String>,
}

/// The issue title as a short branch-safe slug: `Conductor v1: start lanes` → `conductor-v1-start-lanes`.
pub fn slug(title: &str) -> String {
    let words: Vec<String> = title
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .take(4)
        .map(str::to_ascii_lowercase)
        .collect();
    let mut slug = words.join("-");
    slug.truncate(40);
    let slug = slug.trim_end_matches('-').to_string();
    if slug.is_empty() { "work".into() } else { slug }
}

/// A Herdr agent name (`[a-z][a-z0-9_-]{0,31}`) for the lane: `<repository>-<issue>`.
pub fn lane_name(repo: &str, issue: u64) -> String {
    let suffix = format!("-{issue}");
    let mut base: String = repo
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_start_matches(|c: char| !c.is_ascii_lowercase())
        .to_string();
    base.truncate(32usize.saturating_sub(suffix.len()));
    let base = base.trim_end_matches('-');
    if base.is_empty() {
        format!("lane{suffix}")
    } else {
        format!("{base}{suffix}")
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

/// Agents working on an issue in a linked worktree of the project at `root`: its running lanes.
fn running_lanes(root: &Path) -> Result<Vec<(u64, herdr::Agent, PathBuf)>, String> {
    let mut lanes = vec![];
    for agent in herdr::agents()? {
        let cwd = canonical(Path::new(agent.cwd.as_deref().unwrap_or("/")));
        let Some(facts) = git::facts(&cwd) else {
            continue;
        };
        if facts.project_root != root || !facts.linked {
            continue;
        }
        if let Some(task) = facts.branch.as_deref().and_then(task_from_branch) {
            lanes.push((task, agent, facts.worktree));
        }
    }
    Ok(lanes)
}

fn brief_path(slug: &str, issue: u64) -> PathBuf {
    env::state_dir()
        .join("lanes")
        .join(format!("{}-{issue}.md", slug.replace('/', "__")))
}

fn brief(
    repo: &str,
    issue: &github::IssueView,
    branch: &str,
    worktree: &str,
    extra: &str,
) -> String {
    let mut text = format!(
        "# Lane: {repo} #{n} {title}\n\n\
         You are an executor the owner's coordination session started for one issue. Work only on it.\n\n\
         - Issue: #{n} {title} — {url}. Read it in full, with its comments.\n\
         - Checkout: {worktree}, branch `{branch}`, already created from the base branch. Work only\n  \
           here; do not switch branches or touch other worktrees.\n\
         - Follow the repository's own instructions (`AGENTS.md` or `CLAUDE.md`) and its way of\n  \
           implementing an issue.\n\
         - Deliver one pull request to the base branch with `Closes #{n}` in its body, then stop.\n\
         - When a decision is the owner's, or the issue is ambiguous, ask here in this pane and wait.\n\
         - Do not merge, and do not change the owner's configuration outside this repository.\n",
        n = issue.number,
        title = issue.title,
        url = issue.url,
    );
    if !extra.trim().is_empty() {
        text.push_str("\n## From the coordination session\n\n");
        text.push_str(extra.trim_end());
        text.push('\n');
    }
    text
}

/// A one-line prompt Herdr can pass as an agent argument: printable ASCII only.
fn safe_prompt(prompt: &str) -> Result<(), String> {
    if prompt
        .chars()
        .all(|c| c.is_ascii() && !c.is_ascii_control())
    {
        Ok(())
    } else {
        Err(format!(
            "the first prompt must be one line of plain ASCII (the state directory path is part of it): {prompt}"
        ))
    }
}

pub fn start(args: StartArgs, config: &Config) -> Result<Started, String> {
    let dir = args
        .repo
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .ok_or("no repository given (use --repo)")?;
    let facts =
        git::facts(&dir).ok_or_else(|| format!("{} is not a git repository", dir.display()))?;
    let slug_gh = facts
        .github
        .clone()
        .ok_or("the repository has no GitHub origin; lanes need the tracker")?;
    let template = config.lanes.args_for(&args.agent).ok_or_else(|| {
        format!(
            "no launch arguments for the agent kind {}; add them under [lanes.args] in {}",
            args.agent,
            env::config_path().display()
        )
    })?;
    let timeout = Duration::from_secs(config.github_timeout_seconds.max(1));

    let issue = github::issue(&slug_gh, args.issue, timeout)?;
    if issue.state != "OPEN" {
        return Err(format!("issue #{} is closed", issue.number));
    }
    if !issue.assignees.is_empty() {
        return Err(format!(
            "issue #{} is already claimed by {}",
            issue.number,
            issue.assignees.join(", ")
        ));
    }

    let root = facts.project_root.clone();
    let lanes = running_lanes(&root)?;
    if let Some((_, agent, worktree)) = lanes.iter().find(|(task, ..)| *task == args.issue) {
        return Err(format!(
            "issue #{} already has a lane: {} in {}",
            args.issue,
            agent.name.clone().unwrap_or_else(|| agent.pane_id.clone()),
            worktree.display()
        ));
    }
    let max = config.lanes.max;
    if lanes.len() >= max && !args.over_limit {
        return Err(format!(
            "the project already runs {} (limit {max}); start more only with the owner's explicit override: --over-limit",
            plural(lanes.len(), "lane")
        ));
    }

    let title_slug = slug(&issue.title);
    let branch = match &args.branch {
        Some(b) => b.clone(),
        None => fill(
            std::slice::from_ref(&config.lanes.branch),
            &[("issue", &args.issue.to_string()), ("slug", &title_slug)],
        )
        .remove(0),
    };
    if task_from_branch(&branch) != Some(args.issue) {
        return Err(format!(
            "the branch {branch} must name issue #{} (like feat/{}-name) so the lane can be found again",
            args.issue, args.issue
        ));
    }
    let extra = match &args.brief {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?,
        None => String::new(),
    };
    let brief_file = brief_path(&slug_gh, args.issue);
    let prompt = format!(
        "Read and follow the lane instructions in {}",
        brief_file.display()
    );
    safe_prompt(&prompt)?;
    let name = lane_name(
        &root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        args.issue,
    );
    let label = format!("#{} {}", args.issue, title_slug.replace('-', " "));

    let wt = herdr::create_worktree(
        &root.display().to_string(),
        &branch,
        facts.base.as_deref(),
        &label,
    )?;
    let left = format!(
        "the workspace {} and the worktree {} stay as they are",
        wt.workspace_id, wt.path
    );
    std::fs::create_dir_all(brief_file.parent().expect("lanes directory"))
        .and_then(|_| {
            std::fs::write(
                &brief_file,
                brief(&slug_gh, &issue, &branch, &wt.path, &extra),
            )
        })
        .map_err(|e| format!("cannot write {}: {e}; {left}", brief_file.display()))?;
    let argv = fill(&template, &[("prompt", &prompt)]);
    let started = herdr::start_agent(&name, &args.agent, &wt.pane_id, &argv)
        .map_err(|e| format!("{e}; {left}"))?;
    let claim = github::claim(&slug_gh, args.issue, timeout);
    Ok(Started {
        lane: name,
        issue: args.issue,
        title: issue.title,
        agent: args.agent,
        workspace_id: wt.workspace_id,
        pane_id: wt.pane_id,
        branch,
        worktree: wt.path,
        brief: brief_file.display().to_string(),
        status: match started {
            herdr::Started::Ready => "started",
            herdr::Started::Waiting => "waits for you",
        }
        .into(),
        claimed: claim.is_ok(),
        claim_error: claim.err(),
    })
}

pub struct WatchArgs {
    pub lane: String,
    /// Give up after this long while the agent still works (no notification).
    pub timeout: Option<u64>,
    /// How long an idle lane gets to start working before it counts as stopped.
    pub start_timeout: u64,
}

#[derive(Debug, Serialize)]
pub struct Watched {
    pub lane: String,
    pub pane_id: String,
    pub issue: Option<u64>,
    pub status: String,
    /// `pr`, `blocked`, `no_pr`, `exited`, or `working` when the watch timed out.
    pub stop: String,
    pub pr: Option<PrRef>,
    pub notified: bool,
}

fn find_lane(arg: &str, opts: &Options) -> Result<AgentView, String> {
    let snapshot = model::build(opts)?;
    let issue: Option<u64> = arg.trim_start_matches('#').parse().ok();
    snapshot
        .projects
        .into_iter()
        .flat_map(|p| p.agents)
        .find(|a| {
            a.name.as_deref() == Some(arg)
                || a.pane_id == arg
                || (a.is_lane() && issue.is_some() && a.task.as_ref().map(|t| t.number) == issue)
        })
        .ok_or_else(|| format!("no lane {arg} (see `workbench-cc lane list`)"))
}

const SETTLED: [&str; 3] = ["idle", "done", "blocked"];

pub fn watch(args: WatchArgs, config: &Config, opts: &Options) -> Result<Watched, String> {
    let lane = find_lane(&args.lane, opts)?;
    let pane = lane.pane_id.clone();
    let mut settled = true;
    if lane.status != "blocked" && lane.status != "done" {
        let started = lane.status == "working"
            || herdr::wait_agent(
                &pane,
                &["working", "blocked"],
                Some(Duration::from_secs(args.start_timeout)),
            );
        if started {
            settled = herdr::wait_agent(&pane, &SETTLED, args.timeout.map(Duration::from_secs));
        }
    }

    // Read the pane again and make sure it still holds the lane's agent before calling anyone.
    let now = herdr::pane(&pane)
        .ok()
        .filter(|p| lane.session.is_none() || p.session() == lane.session);
    let status = now
        .as_ref()
        .map(|p| p.agent_status.clone())
        .unwrap_or_else(|| "gone".into());
    let id = lane.lane_id();
    let what = match (&lane.task, lane.branch.as_deref()) {
        (Some(t), _) => format!("#{}", t.number),
        (None, Some(b)) => b.to_string(),
        (None, None) => lane.cwd.clone(),
    };
    let mut watched = Watched {
        lane: id.clone(),
        pane_id: pane,
        issue: lane.task.as_ref().map(|t| t.number),
        status: status.clone(),
        stop: String::new(),
        pr: None,
        notified: false,
    };
    let (title, body, sound) = if now.is_none() {
        watched.stop = "exited".into();
        (
            format!("Lane {id}: its agent is gone"),
            format!("{what}: the pane no longer holds the lane's agent"),
            "request",
        )
    } else if status == "working" && !settled {
        watched.stop = "working".into();
        return Ok(watched);
    } else if status == "blocked" {
        watched.stop = "blocked".into();
        (
            format!("Lane {id} waits for you"),
            format!("{what}: a question or an approval in its pane"),
            "request",
        )
    } else {
        watched.pr = lane_pr(&lane, config);
        match &watched.pr {
            Some(pr) => {
                watched.stop = "pr".into();
                (
                    format!("Lane {id}: PR #{} ready", pr.number),
                    format!("{what} → PR #{} · {}", pr.number, pr.ci_phrase),
                    "done",
                )
            }
            None => {
                watched.stop = "no_pr".into();
                (
                    format!("Lane {id} stopped without a PR"),
                    format!("{what}: it may be asking something in its pane"),
                    "request",
                )
            }
        }
    };
    herdr::notify(&title, &body, sound)?;
    watched.notified = true;
    Ok(watched)
}

/// The lane's open PR, fetched fresh: a stop is when the owner looks.
fn lane_pr(lane: &AgentView, config: &Config) -> Option<PrRef> {
    let slug = lane.github.as_deref()?;
    let branch = lane.branch.as_deref()?;
    let settings = Settings {
        refresh_seconds: 0,
        timeout: Duration::from_secs(config.github_timeout_seconds.max(1)),
        network: true,
    };
    let repo = github::repo_facts(slug, &settings).facts?;
    repo.open_prs
        .iter()
        .find(|p| p.head_ref == branch)
        .map(PrRef::from)
}

#[derive(Debug, Serialize)]
pub struct LaneView {
    pub lane: String,
    pub issue: u64,
    pub title: Option<String>,
    pub agent: String,
    pub status: String,
    pub status_phrase: String,
    pub pane_id: String,
    pub branch: Option<String>,
    pub worktree: Option<String>,
    pub pr: Option<PrRef>,
}

#[derive(Debug, Serialize)]
pub struct ProjectLanes {
    pub name: String,
    pub root: String,
    pub github: Option<String>,
    pub max: usize,
    pub lanes: Vec<LaneView>,
}

#[derive(Debug, Serialize)]
pub struct LaneList {
    pub projects: Vec<ProjectLanes>,
}

/// Every running lane, joined to its issue and PR; projects without lanes are shown only when
/// current, so a coordination session sees its free capacity.
pub fn list(config: &Config, opts: &Options) -> Result<LaneList, String> {
    let snapshot = model::build(opts)?;
    let mut projects = vec![];
    for p in snapshot.projects {
        let titles: Vec<(u64, String)> = p
            .work
            .iter()
            .flat_map(|w| {
                w.in_flight
                    .iter()
                    .chain(&w.ready)
                    .map(|i| (i.number, i.title.clone()))
            })
            .collect();
        let lanes: Vec<LaneView> = p
            .agents
            .iter()
            .filter(|a| a.is_lane())
            .map(|a| {
                let issue = a.task.as_ref().map(|t| t.number).unwrap_or(0);
                LaneView {
                    lane: a.lane_id(),
                    issue,
                    title: titles
                        .iter()
                        .find(|(n, _)| *n == issue)
                        .map(|(_, t)| t.clone())
                        .or_else(|| a.pr.as_ref().map(|pr| pr.title.clone())),
                    agent: a.agent.clone(),
                    status: a.status.clone(),
                    status_phrase: a.status_phrase.clone(),
                    pane_id: a.pane_id.clone(),
                    branch: a.branch.clone(),
                    worktree: a.worktree.clone(),
                    pr: a.pr.clone(),
                }
            })
            .collect();
        if lanes.is_empty() && snapshot.current_project.as_deref() != Some(p.root.as_str()) {
            continue;
        }
        projects.push(ProjectLanes {
            name: p.name,
            root: p.root,
            github: p.github,
            max: config.lanes.max,
            lanes,
        });
    }
    Ok(LaneList { projects })
}

pub fn print_list(list: &LaneList) {
    if list.projects.is_empty() {
        println!("no lanes");
    }
    for p in &list.projects {
        println!(
            "{} ({}) — {} of {} lanes",
            p.name,
            p.github.as_deref().unwrap_or("git only"),
            p.lanes.len(),
            p.max
        );
        for l in &p.lanes {
            let title = l
                .title
                .as_deref()
                .map(|t| format!(" {t}"))
                .unwrap_or_default();
            let pr =
                l.pr.as_ref()
                    .map(|pr| format!("  PR #{} {}", pr.number, pr.ci_phrase))
                    .unwrap_or_default();
            println!(
                "  {}  #{}{title}  {}  {}  {}{pr}",
                l.lane,
                l.issue,
                l.agent,
                l.status_phrase,
                l.branch.as_deref().unwrap_or("")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_and_names_are_safe() {
        assert_eq!(
            slug("Conductor v1: start lanes, watch them"),
            "conductor-v1-start-lanes"
        );
        assert_eq!(slug("¿Qué?"), "qu");
        assert_eq!(slug("!!!"), "work");
        assert_eq!(lane_name("agent-workbench", 70), "agent-workbench-70");
        assert_eq!(lane_name("9Lives.App", 5), "lives-app-5");
        assert_eq!(lane_name("123", 5), "lane-5");
        let long = lane_name(&"x".repeat(60), 12345);
        assert!(long.len() <= 32 && long.ends_with("-12345"), "{long}");
    }
}
