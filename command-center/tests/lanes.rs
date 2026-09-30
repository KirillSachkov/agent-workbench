//! Lanes: `lane start`, `lane watch` and `lane list` through the same seam as the rest of the
//! command center — the built binary with fake `herdr` and `gh` that log every call.

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::{Env, agent, issue, pr};

/// A GitHub repository with `origin/main`, as the project a lane starts in.
fn project(env: &Env) -> PathBuf {
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    env.git(&repo, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    repo
}

fn issue_view(env: &Env, number: u64, title: &str, state: &str, assignees: &[&str]) {
    env.rule_json(
        "gh",
        &format!("issue view {number} --repo o/app --json *"),
        &json!({
            "number": number,
            "title": title,
            "url": format!("https://github.com/o/app/issues/{number}"),
            "state": state,
            "assignees": assignees.iter().map(|l| json!({"login": l})).collect::<Vec<_>>(),
            "labels": [{"name": "ready-for-agent"}]
        }),
    );
}

/// Herdr's answer to `worktree create`: the new workspace, its root pane and the checkout.
fn worktree_created(env: &Env, path: &Path, branch: &str) {
    env.rule_json(
        "herdr",
        "worktree create *",
        &json!({"id": "cli:worktree:create", "result": {
            "type": "worktree_created",
            "root_pane": {"pane_id": "w9:p1", "workspace_id": "w9", "agent_status": "unknown"},
            "tab": {"tab_id": "w9:t1", "workspace_id": "w9"},
            "workspace": {"workspace_id": "w9", "label": "#70"},
            "worktree": {"branch": branch, "path": path.display().to_string(), "is_linked_worktree": true}
        }}),
    );
}

fn ready_to_start(env: &Env) -> PathBuf {
    let repo = project(env);
    env.agents(json!([]));
    issue_view(env, 70, "Conductor v1: start lanes", "OPEN", &[]);
    worktree_created(
        env,
        &env.path("repos/app-70"),
        "feat/70-conductor-v1-start-lanes",
    );
    env.rule("herdr", "agent start *", r#"{"result": {}}"#, 0);
    env.rule("gh", "issue edit *", "", 0);
    repo
}

fn start(env: &Env, repo: &Path, extra: &[&str]) -> support::Run {
    let mut args = vec!["lane", "start", "70", "--agent"];
    if !extra
        .iter()
        .any(|a| *a == "codex" || *a == "opencode" || *a == "gemini")
    {
        args.push("claude");
    }
    args.extend(extra);
    args.push("--json");
    env.run_with(&args, |cmd| {
        cmd.current_dir(repo);
    })
}

fn brief_path(env: &Env) -> String {
    env.path("state/lanes/o__app-70.md").display().to_string()
}

#[test]
fn start_creates_a_worktree_workspace_starts_the_named_agent_and_claims_the_issue() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    let run = start(&env, &repo, &[]);
    let out = run.json();

    let root = std::fs::canonicalize(&repo).unwrap();
    let brief = brief_path(&env);
    assert_eq!(
        env.calls("herdr"),
        vec![
            "agent list".to_string(),
            format!(
                "worktree create --cwd {} --branch feat/70-conductor-v1-start-lanes --base origin/main --label #70 conductor v1 start lanes --no-focus",
                root.display()
            ),
            format!(
                "agent start app-70 --kind claude --pane w9:p1 --timeout 30000 -- Read and follow the lane instructions in {brief}"
            ),
        ]
    );
    assert_eq!(
        env.calls("gh"),
        vec![
            "issue view 70 --repo o/app --json number,title,url,state,assignees".to_string(),
            "issue edit 70 --repo o/app --add-assignee @me".to_string(),
        ],
        "the issue is claimed after the agent started"
    );

    let text = std::fs::read_to_string(&brief).expect("the brief is written");
    assert!(text.contains("#70 Conductor v1: start lanes"), "{text}");
    assert!(text.contains("feat/70-conductor-v1-start-lanes"), "{text}");
    assert!(text.contains(&env.path("repos/app-70").display().to_string()));
    assert!(text.contains("Closes #70"), "{text}");
    assert!(text.contains("Do not merge"), "{text}");

    assert_eq!(out["lane"], "app-70");
    assert_eq!(out["issue"], 70);
    assert_eq!(out["agent"], "claude");
    assert_eq!(out["pane_id"], "w9:p1");
    assert_eq!(out["workspace_id"], "w9");
    assert_eq!(out["branch"], "feat/70-conductor-v1-start-lanes");
    assert_eq!(out["claimed"], true);
    assert_eq!(out["status"], "started");
}

#[test]
fn the_prompt_argument_follows_the_runtime() {
    for (kind, args) in [
        ("codex", "-- Read and follow"),
        ("opencode", "-- --prompt Read and follow"),
    ] {
        let env = Env::new();
        let repo = ready_to_start(&env);
        start(&env, &repo, &[kind]).json();
        let call = env
            .calls("herdr")
            .into_iter()
            .find(|c| c.starts_with("agent start"))
            .unwrap();
        assert!(
            call.starts_with(&format!(
                "agent start app-70 --kind {kind} --pane w9:p1 --timeout 30000 {args}"
            )),
            "{call}"
        );
    }
}

#[test]
fn runtime_arguments_and_the_branch_come_from_configuration() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    env.write_config(
        "github_refresh_seconds = 0\n[lanes]\nbranch = \"lane/{issue}-{slug}\"\n[lanes.args]\nclaude = [\"--model\", \"opus\", \"{prompt}\"]\n",
    );
    start(&env, &repo, &[]).json();
    let calls = env.calls("herdr");
    assert!(
        calls[1].contains("--branch lane/70-conductor-v1-start-lanes "),
        "{}",
        calls[1]
    );
    assert!(
        calls[2].contains("-- --model opus Read and follow"),
        "{}",
        calls[2]
    );
}

#[test]
fn start_refuses_a_runtime_without_a_launch_template() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    let run = start(&env, &repo, &["gemini"]);
    assert!(!run.ok);
    assert!(run.stderr.contains("[lanes.args]"), "{}", run.stderr);
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("worktree create"))
    );
}

#[test]
fn start_refuses_beyond_the_parallel_limit_unless_overridden() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    let a = env.worktree(&repo, "app-61", "feat/61-one");
    let b = env.worktree(&repo, "app-62", "feat/62-two");
    env.agents(json!([
        agent("w1:p1", "s1", "working", &a),
        agent("w1:p2", "s2", "working", &b),
        agent("w1:p3", "s3", "idle", &repo),
    ]));

    let refused = start(&env, &repo, &[]);
    assert!(!refused.ok);
    assert!(
        refused.stderr.contains("2 lanes") && refused.stderr.contains("--over-limit"),
        "{}",
        refused.stderr
    );
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("worktree create"))
    );
    assert!(!env.calls("gh").iter().any(|c| c.starts_with("issue edit")));

    env.clear_calls();
    start(&env, &repo, &["--over-limit"]).json();

    env.clear_calls();
    env.write_config("github_refresh_seconds = 0\n[lanes]\nmax = 3\n");
    start(&env, &repo, &[]).json();
}

#[test]
fn start_refuses_closed_claimed_or_already_running_issues() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    issue_view(&env, 70, "Conductor", "CLOSED", &[]);
    let closed = start(&env, &repo, &[]);
    assert!(!closed.ok);
    assert!(closed.stderr.contains("closed"), "{}", closed.stderr);

    issue_view(&env, 70, "Conductor", "OPEN", &["alice"]);
    let claimed = start(&env, &repo, &[]);
    assert!(!claimed.ok);
    assert!(claimed.stderr.contains("alice"), "{}", claimed.stderr);

    issue_view(&env, 70, "Conductor", "OPEN", &[]);
    let wt = env.worktree(&repo, "app-70-old", "feat/70-old");
    env.agents(json!([agent("w1:p1", "s1", "working", &wt)]));
    let running = start(&env, &repo, &[]);
    assert!(!running.ok);
    assert!(
        running.stderr.contains("already has a lane"),
        "{}",
        running.stderr
    );
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("worktree create"))
    );
}

#[test]
fn an_agent_that_waits_at_startup_is_reported_not_answered() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    env.rule(
        "herdr",
        "agent start *",
        r#"{"error": {"code": "agent_not_ready", "message": "agent is blocked"}}"#,
        1,
    );
    let out = start(&env, &repo, &[]).json();
    assert_eq!(out["status"], "waits for you");
    assert_eq!(out["claimed"], true);
    let herdr = env.calls("herdr");
    assert!(
        !herdr
            .iter()
            .any(|c| c.starts_with("agent prompt") || c.starts_with("agent send-keys")),
        "the conductor never answers a startup question: {herdr:?}"
    );
}

#[test]
fn a_failed_agent_start_leaves_the_workspace_and_does_not_claim() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    env.rule("herdr", "agent start *", "", 1);
    let run = start(&env, &repo, &[]);
    assert!(!run.ok);
    assert!(run.stderr.contains("w9"), "{}", run.stderr);
    assert!(!env.calls("gh").iter().any(|c| c.starts_with("issue edit")));
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("worktree remove"))
    );
}

// --- watch ---

/// A running lane: the agent `app-70` in a linked worktree on `feat/70-search`.
fn lane(env: &Env, status: &str) -> Value {
    let repo = project(env);
    let wt = env.worktree(&repo, "app-70", "feat/70-search");
    let mut a = agent("w1:p2", "s-70", status, &wt);
    a["name"] = json!("app-70");
    env.agents(json!([a.clone()]));
    env.rule("herdr", "agent wait *", r#"{"result": {}}"#, 0);
    env.rule("herdr", "notification show *", "{}", 0);
    env.github(
        "o/app",
        json!([]),
        json!([]),
        json!([issue(
            70,
            "Search",
            &["ready-for-agent"],
            &["me"],
            (0, 0),
            0
        )]),
    );
    a
}

fn pane_now(env: &Env, mut a: Value, status: &str) {
    a["agent_status"] = json!(status);
    env.rule_json(
        "herdr",
        "pane get w1:p2",
        &json!({"id": "cli:pane:get", "result": {"pane": a}}),
    );
}

fn notification(env: &Env) -> String {
    let shown: Vec<String> = env
        .calls("herdr")
        .into_iter()
        .filter(|c| c.starts_with("notification show"))
        .collect();
    assert_eq!(shown.len(), 1, "exactly one notification: {shown:?}");
    shown[0].clone()
}

#[test]
fn watch_waits_for_a_working_agent_and_notifies_its_pr() {
    let env = Env::new();
    let a = lane(&env, "working");
    pane_now(&env, a, "done");
    let mut open = pr(81, "feat/70-search", "success");
    open["closingIssuesReferences"] = json!([{"number": 70}]);
    env.github(
        "o/app",
        json!([open]),
        json!([]),
        json!([issue(
            70,
            "Search",
            &["ready-for-agent"],
            &["me"],
            (0, 0),
            0
        )]),
    );
    let out = env.run(&["lane", "watch", "app-70", "--json"]).json();

    let herdr = env.calls("herdr");
    assert!(
        herdr.contains(&"agent wait w1:p2 --until idle --until done --until blocked".to_string()),
        "{herdr:?}"
    );
    let shown = notification(&env);
    assert!(shown.contains("PR #81"), "{shown}");
    assert!(shown.ends_with("--sound done"), "{shown}");
    assert_eq!(out["stop"], "pr");
    assert_eq!(out["status"], "done");
    assert_eq!(out["pr"]["number"], 81);
    assert_eq!(out["issue"], 70);
}

#[test]
fn watch_calls_the_owner_at_once_for_a_blocked_agent() {
    let env = Env::new();
    let a = lane(&env, "blocked");
    pane_now(&env, a, "blocked");
    let out = env.run(&["lane", "watch", "70", "--json"]).json();
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("agent wait")),
        "a blocked agent has already stopped"
    );
    let shown = notification(&env);
    assert!(shown.contains("waits for you"), "{shown}");
    assert!(shown.ends_with("--sound request"), "{shown}");
    assert_eq!(out["stop"], "blocked");
}

#[test]
fn watch_of_an_idle_lane_first_waits_for_it_to_start() {
    let env = Env::new();
    let a = lane(&env, "idle");
    pane_now(&env, a, "idle");
    let out = env
        .run(&["lane", "watch", "w1:p2", "--start-timeout", "5", "--json"])
        .json();
    let waits: Vec<String> = env
        .calls("herdr")
        .into_iter()
        .filter(|c| c.starts_with("agent wait"))
        .collect();
    assert_eq!(
        waits,
        vec![
            "agent wait w1:p2 --until working --until blocked --timeout 5000".to_string(),
            "agent wait w1:p2 --until idle --until done --until blocked".to_string(),
        ]
    );
    let shown = notification(&env);
    assert!(shown.contains("stopped without a PR"), "{shown}");
    assert!(shown.ends_with("--sound request"), "{shown}");
    assert_eq!(out["stop"], "no_pr");
}

#[test]
fn watch_reports_an_agent_that_left_its_pane() {
    let env = Env::new();
    let a = lane(&env, "working");
    let mut other = a.clone();
    other["agent_session"]["value"] = json!("s-other");
    env.rule_json(
        "herdr",
        "pane get w1:p2",
        &json!({"id": "cli:pane:get", "result": {"pane": other}}),
    );
    let out = env.run(&["lane", "watch", "app-70", "--json"]).json();
    assert_eq!(out["stop"], "exited");
    assert!(notification(&env).ends_with("--sound request"));
}

#[test]
fn watch_refuses_an_unknown_lane() {
    let env = Env::new();
    lane(&env, "working");
    let run = env.run(&["lane", "watch", "app-99"]);
    assert!(!run.ok);
    assert!(run.stderr.contains("no lane"), "{}", run.stderr);
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("notification") || c.starts_with("agent wait"))
    );
}

// --- list and the Work section ---

fn two_lanes(env: &Env) {
    let repo = project(env);
    let search = env.worktree(&repo, "app-70", "feat/70-search");
    let login = env.worktree(&repo, "app-62", "feat/62-login");
    let mut a = agent("w1:p2", "s-70", "working", &search);
    a["name"] = json!("app-70");
    let mut b = agent("w1:p3", "s-62", "done", &login);
    b["agent"] = json!("codex");
    env.agents(json!([a, b, agent("w1:p1", "s-main", "idle", &repo),]));
    let mut login_pr = pr(81, "feat/62-login", "success");
    login_pr["closingIssuesReferences"] = json!([{"number": 62}]);
    env.github(
        "o/app",
        json!([login_pr]),
        json!([]),
        json!([
            issue(
                62,
                "Passkey login",
                &["ready-for-agent"],
                &["me"],
                (0, 0),
                0
            ),
            issue(70, "Search", &["ready-for-agent"], &["me"], (0, 0), 0),
            issue(71, "Filters", &["ready-for-agent"], &[], (0, 0), 0),
        ]),
    );
}

#[test]
fn list_joins_lanes_to_issues_and_prs() {
    let env = Env::new();
    two_lanes(&env);
    let out = env.run(&["lane", "list", "--json"]).json();
    let project = &out["projects"][0];
    assert_eq!(project["github"], "o/app");
    assert_eq!(project["max"], 2);
    let lanes = project["lanes"].as_array().unwrap();
    assert_eq!(lanes.len(), 2, "the main checkout is not a lane: {lanes:?}");
    let search = lanes.iter().find(|l| l["issue"] == 70).unwrap();
    assert_eq!(search["lane"], "app-70");
    assert_eq!(search["title"], "Search");
    assert_eq!(search["agent"], "claude");
    assert_eq!(search["status"], "working");
    assert_eq!(search["pr"], Value::Null);
    let login = lanes.iter().find(|l| l["issue"] == 62).unwrap();
    assert_eq!(login["lane"], "w1:p3", "an unnamed agent is its pane");
    assert_eq!(login["agent"], "codex");
    assert_eq!(login["pr"]["number"], 81);
    assert_eq!(login["pr"]["ci"], "success");

    let text = env.run(&["lane", "list"]);
    assert!(text.ok, "{}", text.stderr);
    assert!(text.stdout.contains("2 of 2 lanes"), "{}", text.stdout);
    assert!(text.stdout.contains("#70 Search"), "{}", text.stdout);
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| !c.starts_with("agent list")),
        "list only reads"
    );
    assert!(env.calls("gh").iter().all(|c| !c.contains("edit")));
}

#[test]
fn work_in_flight_shows_the_lane_agent_and_status() {
    let env = Env::new();
    two_lanes(&env);
    let s = env.run(&["overview", "--json"]).json();
    let in_flight = s["projects"][0]["work"]["in_flight"].as_array().unwrap();
    let search = in_flight.iter().find(|i| i["number"] == 70).unwrap();
    assert_eq!(search["lane"]["agent"], "claude");
    assert_eq!(search["lane"]["status"], "working");
    assert_eq!(search["lane"]["status_phrase"], "Working");
    let login = in_flight.iter().find(|i| i["number"] == 62).unwrap();
    assert_eq!(login["lane"]["agent"], "codex");
    assert_eq!(login["lane"]["status"], "done");
}

#[test]
fn a_second_session_in_a_lane_worktree_is_not_a_second_lane() {
    let env = Env::new();
    let repo = ready_to_start(&env);
    let wt = env.worktree(&repo, "app-61", "feat/61-one");
    let mut executor = agent("w1:p1", "s1", "working", &wt);
    executor["name"] = json!("app-61");
    let reviewer = agent("w1:p2", "s2", "idle", &wt);
    env.agents(json!([reviewer, executor]));
    env.github(
        "o/app",
        json!([]),
        json!([]),
        json!([issue(61, "One", &[], &["me"], (0, 0), 0)]),
    );

    let out = env.run(&["lane", "list", "--json"]).json();
    let lanes = out["projects"][0]["lanes"].as_array().unwrap();
    assert_eq!(lanes.len(), 1, "{lanes:?}");
    assert_eq!(lanes[0]["lane"], "app-61", "the named executor is the lane");

    // One lane of two allowed: the start goes ahead.
    start(&env, &repo, &[]).json();
}

#[test]
fn watch_that_times_out_while_working_calls_nobody() {
    let env = Env::new();
    let a = lane(&env, "working");
    pane_now(&env, a, "working");
    env.rule(
        "herdr",
        "agent wait *",
        r#"{"error": {"code": "timeout"}}"#,
        1,
    );
    let out = env
        .run(&["lane", "watch", "app-70", "--timeout", "1", "--json"])
        .json();
    assert_eq!(out["stop"], "working");
    assert_eq!(out["notified"], false);
    assert!(env.calls("herdr").contains(
        &"agent wait w1:p2 --until idle --until done --until blocked --timeout 1000".to_string()
    ));
    assert!(
        !env.calls("herdr")
            .iter()
            .any(|c| c.starts_with("notification"))
    );
}

#[test]
fn watch_treats_a_pane_in_another_directory_as_gone() {
    let env = Env::new();
    let mut a = lane(&env, "working");
    a["cwd"] = json!(env.path("repos/app").display().to_string());
    pane_now(&env, a, "done");
    let out = env.run(&["lane", "watch", "app-70", "--json"]).json();
    assert_eq!(out["stop"], "exited");
}
