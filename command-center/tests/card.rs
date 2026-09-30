mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use support::{Env, agent, pr};

const CARD_BODY: &str = "## Result\nLogin works with passkeys.\n\n## Needs you\nPick the session timeout.\n\n## Look first\n- `src/auth.rs:42` — the token check\n- tests/login_test.rs:7 - the new test\n- ../etc/passwd:1 — must be ignored\n\n## How to try\nRun `rm -rf /` then open http://localhost:3000/login.\n\n## Criteria\n- [x] passkeys\n\nCloses #62\n";

/// An agent in a linked worktree whose PR is #81.
fn with_pr(env: &Env, body: &str, files: Value) -> PathBuf {
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    let wt = env.worktree(&repo, "app-62", "feat/62-login");
    env.commit_file(&wt, "src/auth.rs", &"line\n".repeat(50));
    env.agents(json!([agent("w1:p2", "s-login", "done", &wt)]));
    let mut listed = pr(81, "feat/62-login", "success");
    listed["closingIssuesReferences"] = json!([{"number": 62}]);
    env.github("o/app", json!([listed.clone()]), json!([]), json!([]));
    let mut detail = listed;
    detail["body"] = json!(body);
    detail["files"] = files;
    detail["reviewDecision"] = json!("APPROVED");
    env.pr_view("o/app", 81, detail);
    wt
}

#[test]
fn reads_the_documented_card_shape_from_the_pr_body() {
    let env = Env::new();
    with_pr(&env, CARD_BODY, json!([]));
    let card = env.run(&["card", "--pane", "w1:p2", "--json"]).json();
    assert_eq!(card["shape"], "result-card");
    assert_eq!(card["result"], "Login works with passkeys.");
    assert_eq!(card["needs_you"], "Pick the session timeout.");
    assert_eq!(
        card["look_first"],
        json!([
            {"path": "src/auth.rs", "line": 42, "reason": "the token check"},
            {"path": "tests/login_test.rs", "line": 7, "reason": "the new test"}
        ]),
        "paths leaving the worktree are dropped"
    );
    assert_eq!(card["app_url"], "http://localhost:3000/login");
    assert_eq!(card["other_sections"][0]["title"], "Criteria");
    assert_eq!(card["body"], Value::Null);
    assert_eq!(card["agent"]["task"]["number"], 62);
}

#[test]
fn shows_a_body_in_another_shape_whole() {
    let env = Env::new();
    with_pr(&env, "Fixed the thing.\n\n### Notes\nSome text.", json!([]));
    let card = env.run(&["card", "--pane", "w1:p2", "--json"]).json();
    assert_eq!(card["shape"], "plain");
    assert_eq!(card["body"], "Fixed the thing.\n\n### Notes\nSome text.");
    assert_eq!(card["look_first"], json!([]));
}

#[test]
fn derives_ci_review_head_size_and_whether_tests_or_ci_changed() {
    let env = Env::new();
    with_pr(
        &env,
        CARD_BODY,
        json!([
            {"path": "src/auth.rs", "additions": 8, "deletions": 1},
            {"path": "src/__tests__/auth.spec.ts", "additions": 2, "deletions": 1},
            {"path": ".github/workflows/ci.yml", "additions": 0, "deletions": 0}
        ]),
    );
    let facts = env.run(&["card", "--pane", "w1:p2", "--json"]).json()["facts"].clone();
    assert_eq!(facts["ci"], "CI passed");
    assert_eq!(facts["review"], "approved");
    assert_eq!(
        facts["head_sha"],
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(facts["files_changed"], 3);
    assert_eq!(facts["additions"], 10);
    assert_eq!(facts["deletions"], 2);
    assert_eq!(facts["tests_changed"], true);
    assert_eq!(facts["ci_config_changed"], true);
    assert_eq!(facts["branch"], "feat/62-login");
    assert_eq!(facts["status"], "Done, not seen yet");
}

#[test]
fn no_test_or_ci_changes_are_reported_as_such() {
    let env = Env::new();
    with_pr(
        &env,
        CARD_BODY,
        json!([{"path": "docs/testing-guide.md", "additions": 1, "deletions": 0}]),
    );
    let facts = env.run(&["card", "--pane", "w1:p2", "--json"]).json()["facts"].clone();
    assert_eq!(facts["tests_changed"], false);
    assert_eq!(facts["ci_config_changed"], false);
}

#[test]
fn an_agent_without_a_pr_gets_a_card_from_git() {
    let env = Env::new();
    let repo = env.repo("app", None);
    let wt = env.worktree(&repo, "app-7", "feat/7-draft");
    env.commit_file(&wt, "src/new.rs", "a\nb\n");
    std::fs::write(wt.join("README.md"), "hello\nmore\n").unwrap();
    std::fs::write(wt.join("untracked.txt"), "x\ny\nz\n").unwrap();
    env.agents(json!([agent("w1:p1", "s1", "working", &wt)]));
    let card = env.run(&["card", "--pane", "w1:p1", "--json"]).json();
    assert_eq!(card["shape"], "no-pr");
    assert_eq!(card["facts"]["branch"], "feat/7-draft");
    assert_eq!(
        card["facts"]["files_changed"], 3,
        "committed, uncommitted and new files since the base"
    );
    assert_eq!(card["facts"]["additions"], 6);
    let paths: Vec<&str> = card["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths, vec!["README.md", "src/new.rs", "untracked.txt"]);
}

#[test]
fn the_card_refuses_a_pane_that_holds_another_agent() {
    let env = Env::new();
    with_pr(&env, CARD_BODY, json!([]));
    let run = env.run(&[
        "card",
        "--pane",
        "w1:p2",
        "--json",
        "--expect-session",
        "someone-else",
    ]);
    assert!(!run.ok);
    assert!(
        run.stderr.contains("no longer holds the expected agent"),
        "{}",
        run.stderr
    );
}
