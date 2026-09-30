//! Herdr, read and driven only through its public CLI.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use crate::run::{Output, run};

/// The metadata source our sidebar token is reported under.
pub const SOURCE: &str = "agent-workbench";
/// Our sidebar token; prefixed so it never collides with another plugin's tokens.
pub const TAG_TOKEN: &str = "wb_tag";

const TIMEOUT: Duration = Duration::from_secs(10);

pub fn bin() -> String {
    std::env::var("HERDR_BIN_PATH")
        .ok()
        .filter(|b| !b.is_empty())
        .unwrap_or_else(|| "herdr".into())
}

fn call(args: &[&str]) -> Output {
    run(&bin(), args, None, Some(TIMEOUT))
}

fn result(args: &[&str]) -> Result<Value, String> {
    result_of(args, call(args))
}

fn result_of(args: &[&str], out: Output) -> Result<Value, String> {
    if !out.ok {
        return Err(format!("herdr {}: {}", args.join(" "), out.error_line()));
    }
    let value: Value = serde_json::from_str(&out.stdout)
        .map_err(|err| format!("herdr {}: unreadable output: {err}", args.join(" ")))?;
    Ok(value.get("result").cloned().unwrap_or(value))
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Agent {
    pub pane_id: String,
    /// The live agent name, when one was given (`agent start <name>`, `agent rename`).
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub terminal_id: String,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub display_agent: Option<String>,
    #[serde(default)]
    pub agent_session: Option<Session>,
    #[serde(default)]
    pub agent_status: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub terminal_title_stripped: Option<String>,
    #[serde(default)]
    pub tokens: BTreeMap<String, String>,
}

impl Agent {
    pub fn name(&self) -> String {
        self.display_agent
            .clone()
            .or_else(|| self.agent.clone())
            .unwrap_or_else(|| "agent".into())
    }

    pub fn session(&self) -> Option<String> {
        self.agent_session.as_ref().and_then(|s| s.value.clone())
    }
}

pub fn agents() -> Result<Vec<Agent>, String> {
    let value = result(&["agent", "list"])?;
    let list = value.get("agents").cloned().unwrap_or(Value::Array(vec![]));
    serde_json::from_value(list)
        .map_err(|err| format!("herdr agent list: unreadable agents: {err}"))
}

/// A pane as `herdr pane get` shows it (the same identity fields as an agent).
pub fn pane(pane_id: &str) -> Result<Agent, String> {
    let value = result(&["pane", "get", pane_id])?;
    let pane = value.get("pane").cloned().unwrap_or(value);
    serde_json::from_value(pane).map_err(|err| format!("herdr pane get: unreadable pane: {err}"))
}

pub fn report_tag(pane_id: &str, tag: Option<&str>) -> Result<(), String> {
    let token;
    let mut args = vec!["pane", "report-metadata", pane_id, "--source", SOURCE];
    match tag {
        Some(tag) => {
            token = format!("{TAG_TOKEN}={tag}");
            args.extend(["--token", token.as_str()]);
        }
        None => args.extend(["--clear-token", TAG_TOKEN]),
    }
    let out = call(&args);
    if out.ok {
        Ok(())
    } else {
        Err(format!("herdr pane report-metadata: {}", out.error_line()))
    }
}

/// Opens one of our manifest panes (`entrypoint`) as a split beside `target_pane`.
pub fn open_split(
    entrypoint: &str,
    target_pane: &str,
    cwd: &str,
    env: &[(&str, String)],
) -> Result<(), String> {
    let plugin = crate::env::plugin_id();
    let mut args: Vec<String> = [
        "plugin",
        "pane",
        "open",
        "--plugin",
        &plugin,
        "--entrypoint",
        entrypoint,
        "--placement",
        "split",
        "--target-pane",
        target_pane,
        "--direction",
        "right",
        "--cwd",
        cwd,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (key, value) in env {
        args.push("--env".into());
        args.push(format!("{key}={value}"));
    }
    args.push("--focus".into());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = call(&refs);
    if out.ok {
        Ok(())
    } else {
        Err(format!("herdr plugin pane open: {}", out.error_line()))
    }
}

/// Opens one of our manifest popups, passing the pane the owner came from.
pub fn open_popup(entrypoint: &str, env: &[(&str, String)]) -> Result<(), String> {
    let plugin = crate::env::plugin_id();
    let mut args: Vec<String> = [
        "plugin",
        "pane",
        "open",
        "--plugin",
        &plugin,
        "--entrypoint",
        entrypoint,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (key, value) in env {
        args.push("--env".into());
        args.push(format!("{key}={value}"));
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = call(&refs);
    if out.ok {
        Ok(())
    } else {
        Err(format!("herdr plugin pane open: {}", out.error_line()))
    }
}

pub fn focus_agent(pane_id: &str) -> Result<(), String> {
    let out = call(&["agent", "focus", pane_id]);
    if out.ok {
        Ok(())
    } else {
        Err(format!("herdr agent focus: {}", out.error_line()))
    }
}

pub fn reload_config() -> Result<(), String> {
    let out = call(&["server", "reload-config"]);
    if out.ok {
        Ok(())
    } else {
        Err(out.error_line())
    }
}

pub fn version() -> Result<String, String> {
    let out = call(&["--version"]);
    if !out.ok {
        return Err(out.error_line());
    }
    Ok(out
        .stdout
        .split_whitespace()
        .last()
        .unwrap_or("")
        .to_string())
}

/// A worktree Herdr created and opened as a workspace.
pub struct Worktree {
    pub workspace_id: String,
    pub pane_id: String,
    pub path: String,
}

/// Creates a git worktree on a new branch and opens it as a workspace, without taking focus.
pub fn create_worktree(
    repo: &str,
    branch: &str,
    base: Option<&str>,
    label: &str,
) -> Result<Worktree, String> {
    let mut args = vec!["worktree", "create", "--cwd", repo, "--branch", branch];
    if let Some(base) = base {
        args.extend(["--base", base]);
    }
    args.extend(["--label", label, "--no-focus"]);
    let value = result_of(&args, run(&bin(), &args, None, Some(START_TIMEOUT)))?;
    let text = |pointer: &str| {
        value
            .pointer(pointer)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("herdr worktree create: no {pointer} in its answer"))
    };
    Ok(Worktree {
        workspace_id: text("/workspace/workspace_id")?,
        pane_id: text("/root_pane/pane_id")?,
        path: text("/worktree/path")?,
    })
}

/// How `agent start` ended.
pub enum Started {
    /// Herdr sees the agent ready for input.
    Ready,
    /// The agent is up but stopped at a question or approval during startup (`agent_not_ready`).
    Waiting,
}

const START_TIMEOUT: Duration = Duration::from_secs(60);

/// Starts a supported agent in an existing shell pane under a unique name.
pub fn start_agent(name: &str, kind: &str, pane: &str, args: &[String]) -> Result<Started, String> {
    let mut argv = vec![
        "agent",
        "start",
        name,
        "--kind",
        kind,
        "--pane",
        pane,
        "--timeout",
        "30000",
    ];
    if !args.is_empty() {
        argv.push("--");
        argv.extend(args.iter().map(String::as_str));
    }
    let out = run(&bin(), &argv, None, Some(START_TIMEOUT));
    if out.ok {
        Ok(Started::Ready)
    } else if out.stderr.contains("agent_not_ready") || out.stdout.contains("agent_not_ready") {
        Ok(Started::Waiting)
    } else {
        Err(format!("herdr agent start: {}", out.error_line()))
    }
}

/// Blocks until the agent in `pane` reaches one of `until`; false on a timeout or a failure.
pub fn wait_agent(pane: &str, until: &[&str], timeout: Option<Duration>) -> bool {
    let ms;
    let mut args = vec!["agent", "wait", pane];
    for status in until {
        args.extend(["--until", status]);
    }
    if let Some(limit) = timeout {
        ms = limit.as_millis().to_string();
        args.extend(["--timeout", ms.as_str()]);
    }
    // Herdr enforces the timeout; ours only guards against a hung CLI.
    let guard = timeout.map(|t| t + Duration::from_secs(30));
    run(&bin(), &args, None, guard).ok
}

/// A toast inside Herdr with a sound: `done` for a result, `request` when the owner is needed.
pub fn notify(title: &str, body: &str, sound: &str) -> Result<(), String> {
    let out = call(&[
        "notification",
        "show",
        title,
        "--body",
        body,
        "--sound",
        sound,
    ]);
    if out.ok {
        Ok(())
    } else {
        Err(format!("herdr notification show: {}", out.error_line()))
    }
}
