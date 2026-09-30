mod support;

use std::path::PathBuf;

use serde_json::json;
use support::{Env, agent, pr};

const BODY: &str = "## Result\nDone.\n\n## Look first\n- src/auth.rs:42 — the check\n- src/missing.rs:3 — not in the worktree\n\n## How to try\nOpen https://localhost:8443/app and log in.\n";

fn setup(env: &Env) -> PathBuf {
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    let wt = env.worktree(&repo, "app-62", "feat/62-login");
    env.commit_file(&wt, "src/auth.rs", &"line\n".repeat(50));
    env.agents(json!([agent("w1:p2", "s-login", "done", &wt)]));
    env.github(
        "o/app",
        json!([pr(81, "feat/62-login", "success")]),
        json!([]),
        json!([]),
    );
    let mut detail = pr(81, "feat/62-login", "success");
    detail["body"] = json!(BODY);
    detail["files"] = json!([]);
    env.pr_view("o/app", 81, detail);
    env.clear_calls();
    wt
}

fn plugin_open(env: &Env) -> String {
    let calls = env.calls("herdr");
    calls
        .into_iter()
        .find(|c| c.starts_with("plugin pane open"))
        .expect("a plugin pane was opened")
}

#[test]
fn opens_a_look_first_file_at_its_line_in_a_split_in_the_worktree() {
    let env = Env::new();
    let wt = setup(&env);
    let run = env.run(&["open", "file", "--pane", "w1:p2", "--look-first", "1"]);
    assert!(run.ok, "{}", run.stderr);
    assert!(
        env.calls("herdr").contains(&"pane get w1:p2".to_string()),
        "the pane is checked before acting"
    );
    let wt = std::fs::canonicalize(wt).unwrap();
    assert_eq!(
        plugin_open(&env),
        format!(
            "plugin pane open --plugin agent-workbench.command-center --entrypoint exec --placement split --target-pane w1:p2 --direction right --cwd {} --env WB_EXEC=[\"nvim\",\"+42\",\"src/auth.rs\"] --focus",
            wt.display()
        )
    );
}

#[test]
fn the_editor_command_is_configurable() {
    let env = Env::new();
    setup(&env);
    env.write_config("github_refresh_seconds = 0\neditor = [\"hx\", \"{path}:{line}\"]\n");
    assert!(
        env.run(&[
            "open",
            "file",
            "--pane",
            "w1:p2",
            "--path",
            "src/auth.rs",
            "--line",
            "9"
        ])
        .ok
    );
    assert!(plugin_open(&env).contains("--env WB_EXEC=[\"hx\",\"src/auth.rs:9\"]"));
}

#[test]
fn refuses_files_outside_the_worktree_or_missing() {
    let env = Env::new();
    setup(&env);
    let missing = env.run(&["open", "file", "--pane", "w1:p2", "--look-first", "2"]);
    assert!(!missing.ok);
    assert!(
        missing.stderr.contains("no such file in the worktree"),
        "{}",
        missing.stderr
    );
    let escape = env.run(&[
        "open",
        "file",
        "--pane",
        "w1:p2",
        "--path",
        "../app/README.md",
    ]);
    assert!(!escape.ok);
    assert!(
        escape.stderr.contains("outside the worktree"),
        "{}",
        escape.stderr
    );
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("plugin pane open"))
    );
}

#[test]
fn opens_the_diff_against_the_base_beside_the_agent() {
    let env = Env::new();
    setup(&env);
    assert!(env.run(&["open", "diff", "--pane", "w1:p2"]).ok);
    assert!(
        plugin_open(&env).contains("--env WB_EXEC=[\"nvim\",\"-c\",\"CodeDiff main...\"]"),
        "{}",
        plugin_open(&env)
    );
}

#[test]
fn opens_the_pr_in_the_browser_through_gh() {
    let env = Env::new();
    setup(&env);
    env.rule("gh", "pr view 81 --repo o/app --web", "", 0);
    let run = env.run(&["open", "pr", "--pane", "w1:p2"]);
    assert!(run.ok, "{}", run.stderr);
    assert!(
        env.calls("gh")
            .contains(&"pr view 81 --repo o/app --web".to_string())
    );
}

#[test]
fn opens_the_app_address_from_how_to_try_and_runs_nothing_else() {
    let env = Env::new();
    setup(&env);
    env.write_config("github_refresh_seconds = 0\nbrowser = [\"open\", \"{url}\"]\n");
    let run = env.run(&["open", "app", "--pane", "w1:p2"]);
    assert!(run.ok, "{}", run.stderr);
    // The browser is started detached; give it a moment to log.
    for _ in 0..50 {
        if !env.calls("open").is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        env.calls("open"),
        vec!["https://localhost:8443/app".to_string()]
    );
}

#[test]
fn focuses_the_agent_pane_after_checking_it() {
    let env = Env::new();
    setup(&env);
    assert!(
        env.run(&["focus", "--pane", "w1:p2", "--expect-session", "s-login"])
            .ok
    );
    assert_eq!(
        env.calls("herdr"),
        vec![
            "pane get w1:p2".to_string(),
            "agent focus w1:p2".to_string()
        ]
    );
}

#[test]
fn refuses_to_act_when_the_pane_holds_another_agent_or_cwd() {
    let env = Env::new();
    setup(&env);
    let other = env.run(&["focus", "--pane", "w1:p2", "--expect-session", "s-other"]);
    assert!(!other.ok);
    let moved = env.run(&["open", "diff", "--pane", "w1:p2", "--expect-cwd", "/tmp"]);
    assert!(!moved.ok);
    assert!(moved.stderr.contains("now works in"), "{}", moved.stderr);
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("agent focus") || c.starts_with("plugin pane open"))
    );
}

#[test]
fn popups_open_through_herdr_with_the_pane_the_owner_came_from() {
    let env = Env::new();
    let run = env.run_with(&["popup", "card"], |cmd| {
        cmd.env(
            "HERDR_PLUGIN_CONTEXT_JSON",
            r#"{"focused_pane_id":"w1:p2","focused_pane_cwd":"/work/app"}"#,
        );
    });
    assert!(run.ok, "{}", run.stderr);
    assert_eq!(
        env.calls("herdr"),
        vec!["plugin pane open --plugin agent-workbench.command-center --entrypoint card --env WB_PANE=w1:p2 --env WB_CURRENT=/work/app".to_string()]
    );
}

#[test]
fn the_exec_pane_runs_its_argv_without_a_shell() {
    let env = Env::new();
    let marker = env.path("ran");
    let run = env.run_with(&["exec"], |cmd| {
        cmd.env(
            "WB_EXEC",
            serde_json::to_string(&["/usr/bin/touch", marker.to_str().unwrap()]).unwrap(),
        );
    });
    assert!(run.ok, "{}", run.stderr);
    assert!(marker.exists());
}
