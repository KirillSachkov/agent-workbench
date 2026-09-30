//! The one test seam: the built binary as a black box, with fake `herdr`, `gh` and browser
//! executables first on PATH that answer from fixtures and log every call, against temporary git
//! repositories.

#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

const FAKE: &str = r#"#!/bin/sh
tool=$(basename "$0")
dir="$FAKE_DIR"
printf '%s\n' "$*" >> "$dir/$tool.log"
tab=$(printf '\t')
if [ -f "$dir/$tool.rules" ]; then
  while IFS="$tab" read -r pat file code; do
    case "$*" in
      $pat)
        [ -n "$file" ] && [ "$file" != "-" ] && cat "$dir/$file"
        exit "${code:-0}";;
    esac
  done < "$dir/$tool.rules"
fi
echo "fake $tool: no rule for: $*" >&2
exit 97
"#;

pub struct Env {
    pub root: tempfile::TempDir,
    fixture: std::cell::Cell<usize>,
}

pub struct Run {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    pub fn json(&self) -> Value {
        assert!(
            self.ok,
            "command failed:\nstdout: {}\nstderr: {}",
            self.stdout, self.stderr
        );
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("not JSON ({e}): {}", self.stdout))
    }
}

impl Env {
    pub fn new() -> Env {
        let root = tempfile::tempdir().expect("tempdir");
        let env = Env {
            root,
            fixture: std::cell::Cell::new(0),
        };
        for dir in ["bin", "fake", "state", "config", "home", "repos"] {
            fs::create_dir_all(env.path(dir)).unwrap();
        }
        for tool in ["herdr", "gh", "open", "xdg-open"] {
            let path = env.path("bin").join(tool);
            fs::write(&path, FAKE).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        // GitHub facts are fetched on every run unless a test says otherwise.
        env.write_config("github_refresh_seconds = 0\n");
        env.rule("herdr", "--version", "herdr 0.9.1\n", 0);
        env.rule("herdr", "pane report-metadata *", "{}", 0);
        env.rule("herdr", "plugin pane open *", "{}", 0);
        env.rule("herdr", "agent focus *", "{}", 0);
        env.rule("herdr", "server reload-config", "{}", 0);
        env.rule("open", "*", "", 0);
        env.rule("xdg-open", "*", "", 0);
        env
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.path().join(rel)
    }

    pub fn canonical(&self, rel: &str) -> String {
        fs::canonicalize(self.path(rel))
            .unwrap()
            .display()
            .to_string()
    }

    pub fn write_config(&self, text: &str) {
        fs::write(self.path("config/config.toml"), text).unwrap();
    }

    /// Adds a rule; newer rules win over older ones. `pattern` is a shell glob over the joined args.
    pub fn rule(&self, tool: &str, pattern: &str, response: &str, code: i32) {
        let n = self.fixture.get() + 1;
        self.fixture.set(n);
        let file = format!("fixture-{n}");
        fs::write(self.path("fake").join(&file), response).unwrap();
        let rules = self.path("fake").join(format!("{tool}.rules"));
        let old = fs::read_to_string(&rules).unwrap_or_default();
        fs::write(&rules, format!("{pattern}\t{file}\t{code}\n{old}")).unwrap();
    }

    pub fn rule_json(&self, tool: &str, pattern: &str, value: &Value) {
        self.rule(tool, pattern, &value.to_string(), 0);
    }

    pub fn agents(&self, agents: Value) {
        self.rule_json(
            "herdr",
            "agent list",
            &json!({"id": "cli:agent:list", "result": {"agents": agents}}),
        );
        for agent in agents.as_array().unwrap() {
            let pane = agent["pane_id"].as_str().unwrap();
            self.rule_json(
                "herdr",
                &format!("pane get {pane}"),
                &json!({"id": "cli:pane:get", "result": {"pane": agent}}),
            );
        }
    }

    /// Answers the three calls that load a repository's GitHub facts.
    pub fn github(&self, slug: &str, open_prs: Value, merged_prs: Value, issues: Value) {
        self.rule_json(
            "gh",
            &format!("pr list --repo {slug} --state open *"),
            &open_prs,
        );
        self.rule_json(
            "gh",
            &format!("pr list --repo {slug} --state merged *"),
            &merged_prs,
        );
        let name = slug.split('/').nth(1).unwrap();
        self.rule_json(
            "gh",
            &format!("api graphql *name={name}"),
            &json!({"data": {"repository": {"issues": {"nodes": issues}}}}),
        );
    }

    pub fn pr_view(&self, slug: &str, number: u64, pr: Value) {
        self.rule_json(
            "gh",
            &format!("pr view {number} --repo {slug} --json *"),
            &pr,
        );
    }

    pub fn calls(&self, tool: &str) -> Vec<String> {
        fs::read_to_string(self.path("fake").join(format!("{tool}.log")))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    pub fn clear_calls(&self) {
        for tool in ["herdr", "gh", "open", "xdg-open"] {
            let _ = fs::remove_file(self.path("fake").join(format!("{tool}.log")));
        }
    }

    pub fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_workbench-cc"));
        cmd.args(args);
        for (key, _) in std::env::vars() {
            if key.starts_with("HERDR_") || key.starts_with("WB_") || key.starts_with("XDG_") {
                cmd.env_remove(key);
            }
        }
        let path = format!("{}:/usr/bin:/bin", self.path("bin").display());
        cmd.env("PATH", path)
            .env("HOME", self.path("home"))
            .env("FAKE_DIR", self.path("fake"))
            .env("HERDR_PLUGIN_STATE_DIR", self.path("state"))
            .env("HERDR_PLUGIN_CONFIG_DIR", self.path("config"))
            .env("HERDR_CONFIG_PATH", self.path("herdr-config.toml"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .current_dir(self.path("home"));
        cmd
    }

    pub fn run(&self, args: &[&str]) -> Run {
        self.run_with(args, |_| {})
    }

    pub fn run_with(&self, args: &[&str], setup: impl FnOnce(&mut Command)) -> Run {
        let mut cmd = self.command(args);
        cmd.stdin(Stdio::null());
        setup(&mut cmd);
        finish(cmd.output().expect("run workbench-cc"))
    }

    pub fn run_input(&self, args: &[&str], input: &str) -> Run {
        use std::io::Write;
        let mut cmd = self.command(args);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        finish(child.wait_with_output().unwrap())
    }

    /// A repository with one commit on `main` and an `origin` remote on GitHub (never fetched).
    pub fn repo(&self, name: &str, remote: Option<&str>) -> PathBuf {
        let dir = self.path("repos").join(name);
        fs::create_dir_all(&dir).unwrap();
        self.git(&dir, &["init", "-q", "-b", "main"]);
        fs::write(dir.join("README.md"), "hello\n").unwrap();
        self.git(&dir, &["add", "."]);
        self.git(&dir, &["commit", "-q", "-m", "first"]);
        if let Some(url) = remote {
            self.git(&dir, &["remote", "add", "origin", url]);
        }
        dir
    }

    /// A linked worktree of `repo` on a new branch.
    pub fn worktree(&self, repo: &Path, name: &str, branch: &str) -> PathBuf {
        let dir = self.path("repos").join(name);
        self.git(
            repo,
            &["worktree", "add", "-q", "-b", branch, dir.to_str().unwrap()],
        );
        dir
    }

    pub fn commit_file(&self, dir: &Path, path: &str, text: &str) {
        let file = dir.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, text).unwrap();
        self.git(dir, &["add", "."]);
        self.git(dir, &["commit", "-q", "-m", &format!("change {path}")]);
    }

    pub fn git(&self, dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("HOME", self.path("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }
}

fn finish(out: Output) -> Run {
    Run {
        ok: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// An agent as `herdr agent list` reports it.
pub fn agent(pane: &str, session: &str, status: &str, cwd: &Path) -> Value {
    json!({
        "agent": "claude",
        "agent_session": {"agent": "claude", "kind": "id", "source": "herdr:claude", "value": session},
        "agent_status": status,
        "cwd": cwd.display().to_string(),
        "focused": false,
        "pane_id": pane,
        "revision": 1,
        "tab_id": "w1:t1",
        "terminal_id": format!("term_{pane}"),
        "terminal_title_stripped": format!("task in {pane}"),
        "tokens": {},
        "workspace_id": "w1"
    })
}

/// A pull request as `gh pr list --json` returns it.
pub fn pr(number: u64, head: &str, ci: &str) -> Value {
    let checks = match ci {
        "success" => {
            json!([{"__typename": "CheckRun", "name": "test", "status": "COMPLETED", "conclusion": "SUCCESS"}])
        }
        "failure" => {
            json!([{"__typename": "CheckRun", "name": "test", "status": "COMPLETED", "conclusion": "FAILURE"}])
        }
        "pending" => {
            json!([{"__typename": "CheckRun", "name": "test", "status": "IN_PROGRESS", "conclusion": ""}])
        }
        _ => json!([]),
    };
    json!({
        "number": number,
        "title": format!("PR for {head}"),
        "url": format!("https://github.com/o/app/pull/{number}"),
        "state": "OPEN",
        "isDraft": false,
        "headRefName": head,
        "baseRefName": "main",
        "labels": [],
        "closingIssuesReferences": [],
        "statusCheckRollup": checks,
        "reviewDecision": "",
        "headRefOid": "0123456789abcdef0123456789abcdef01234567",
        "additions": 10,
        "deletions": 2,
        "changedFiles": 2
    })
}

pub fn issue(
    number: u64,
    title: &str,
    labels: &[&str],
    assignees: &[&str],
    sub: (u64, u64),
    blocked_by: u64,
) -> Value {
    json!({
        "number": number,
        "title": title,
        "url": format!("https://github.com/o/app/issues/{number}"),
        "assignees": {"nodes": assignees.iter().map(|l| json!({"login": l})).collect::<Vec<_>>()},
        "labels": {"nodes": labels.iter().map(|l| json!({"name": l})).collect::<Vec<_>>()},
        "subIssuesSummary": {"total": sub.0, "completed": sub.1},
        "issueDependenciesSummary": {"blockedBy": blocked_by}
    })
}
