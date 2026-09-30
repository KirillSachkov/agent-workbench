//! Opening things from the Overview and the card: a file at a line, the diff, the PR, the app, the
//! agent's pane. Before acting on a pane, check it still holds the expected agent and cwd.

use std::path::Path;

use crate::card::{self, Card};
use crate::env::{Config, canonical, fill};
use crate::github::Settings;
use crate::model::{self, AgentView, Options};
use crate::{github, herdr, run};

/// What the caller believes the pane holds.
#[derive(Default)]
pub struct Expect {
    pub session: Option<String>,
    pub cwd: Option<String>,
}

/// Checks that `pane_id` still holds the agent and cwd the caller expects.
pub fn verify(pane_id: &str, expect: &Expect) -> Result<(), String> {
    let pane = herdr::pane(pane_id)?;
    if let Some(session) = &expect.session
        && pane.session().as_ref() != Some(session)
    {
        return Err(format!("pane {pane_id} no longer holds the expected agent"));
    }
    let cwd = canonical(Path::new(pane.cwd.as_deref().unwrap_or("/")))
        .display()
        .to_string();
    if let Some(expected) = &expect.cwd
        && canonical(Path::new(expected)).display().to_string() != cwd
    {
        return Err(format!("pane {pane_id} now works in {cwd}, not {expected}"));
    }
    Ok(())
}

/// Finds the agent in `pane_id` after checking its identity against `expect`.
pub fn agent_in(pane_id: &str, expect: &Expect, opts: &Options) -> Result<AgentView, String> {
    verify(pane_id, expect)?;
    let snapshot = model::build(opts)?;
    snapshot
        .projects
        .into_iter()
        .flat_map(|p| p.agents)
        .chain(snapshot.loose)
        .find(|a| a.pane_id == pane_id)
        .ok_or_else(|| format!("no agent in pane {pane_id}"))
}

pub fn card_for(pane_id: &str, expect: &Expect, opts: &Options) -> Result<Card, String> {
    let agent = agent_in(pane_id, expect, opts)?;
    Ok(card::build(agent, &opts.github))
}

fn worktree(agent: &AgentView) -> Result<&str, String> {
    agent
        .worktree
        .as_deref()
        .ok_or_else(|| "the agent does not work in a git repository".to_string())
}

/// Runs `argv` in a split beside the agent's pane, with its worktree as cwd, through our `exec`
/// pane: the command is passed as an argv array and never goes through a shell.
fn open_beside(agent: &AgentView, argv: Vec<String>) -> Result<(), String> {
    let cwd = worktree(agent)?;
    let json = serde_json::to_string(&argv).map_err(|e| e.to_string())?;
    herdr::open_split("exec", &agent.pane_id, cwd, &[("WB_EXEC", json)])
}

pub fn open_file(agent: &AgentView, path: &str, line: u64, config: &Config) -> Result<(), String> {
    let root = canonical(Path::new(worktree(agent)?));
    if !card::safe_relative(path) {
        return Err(format!("refusing path outside the worktree: {path}"));
    }
    let full = canonical(&root.join(path));
    if !full.starts_with(&root) || !full.is_file() {
        return Err(format!("no such file in the worktree: {path}"));
    }
    open_beside(
        agent,
        fill(
            &config.editor,
            &[("path", path), ("line", &line.to_string())],
        ),
    )
}

pub fn open_look_first(card: &Card, index: usize, config: &Config) -> Result<(), String> {
    let item = card
        .look_first
        .get(index.wrapping_sub(1))
        .ok_or_else(|| format!("the card has no \"look first\" item {index}"))?;
    open_file(&card.agent, &item.path, item.line, config)
}

pub fn open_diff(agent: &AgentView, config: &Config) -> Result<(), String> {
    let base = agent
        .base
        .as_deref()
        .ok_or("no base branch to compare with")?;
    open_beside(agent, fill(&config.diff, &[("base", base)]))
}

pub fn open_pr(agent: &AgentView) -> Result<(), String> {
    match (&agent.github, &agent.pr) {
        (Some(slug), Some(pr)) => github::open_in_browser(slug, pr.number),
        _ => Err("the agent has no pull request".into()),
    }
}

pub fn open_app(card: &Card, config: &Config) -> Result<(), String> {
    let url = card
        .app_url
        .as_deref()
        .ok_or("the card gives no address under \"How to try\"")?;
    run::spawn_detached(&fill(&config.browser, &[("url", url)]))
}

pub fn focus(agent: &AgentView) -> Result<(), String> {
    herdr::focus_agent(&agent.pane_id)
}

/// Starts the command our `exec` pane was opened with (`WB_EXEC`, a JSON argv array).
pub fn exec_from_env() -> Result<(), String> {
    use std::os::unix::process::CommandExt;
    let raw = std::env::var("WB_EXEC").map_err(|_| "WB_EXEC is not set".to_string())?;
    let argv: Vec<String> = serde_json::from_str(&raw).map_err(|e| format!("WB_EXEC: {e}"))?;
    let (program, args) = argv.split_first().ok_or("WB_EXEC is empty")?;
    let err = std::process::Command::new(program).args(args).exec();
    Err(format!("cannot run {program}: {err}"))
}

pub fn github_settings(config: &Config, network: bool) -> Settings {
    Settings {
        refresh_seconds: config.github_refresh_seconds,
        timeout: std::time::Duration::from_secs(config.github_timeout_seconds.max(1)),
        network,
    }
}
