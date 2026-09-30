//! Snapshot tests of the popups, rendered by the binary with `--print`. Set UPDATE_SNAPSHOTS=1 to
//! rewrite the files under `tests/snapshots/` after an intended change.

mod support;

use serde_json::json;
use support::{Env, agent, issue, pr};

fn check(name: &str, env: &Env, output: &str) {
    let root = std::fs::canonicalize(env.root.path())
        .unwrap()
        .display()
        .to_string();
    let actual = output.replace(&root, "<tmp>");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.txt"));
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &actual).unwrap();
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        actual, expected,
        "snapshot {name} differs; rerun with UPDATE_SNAPSHOTS=1 if intended"
    );
}

fn scenario(env: &Env) {
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    let login = env.worktree(&repo, "app-62", "feat/62-login");
    let search = env.worktree(&repo, "app-70", "feat/70-search");
    env.commit_file(&login, "src/auth.rs", &"line\n".repeat(50));
    std::fs::create_dir_all(env.path("notes")).unwrap();
    env.agents(json!([
        agent("w1:p1", "s-main", "working", &repo),
        agent("w1:p2", "s-login", "done", &login),
        agent("w1:p3", "s-search", "blocked", &search),
        agent("w1:p4", "s-notes", "idle", &env.path("notes")),
    ]));
    let mut login_pr = pr(81, "feat/62-login", "success");
    login_pr["closingIssuesReferences"] = json!([{"number": 62}]);
    env.github(
        "o/app",
        json!([login_pr.clone(), pr(90, "broken", "failure")]),
        json!([]),
        json!([
            issue(60, "Spec: sign-in", &[], &[], (4, 1), 0),
            issue(
                62,
                "Passkey login",
                &["ready-for-agent"],
                &["alice"],
                (0, 0),
                0
            ),
            issue(63, "Session timeout", &["ready-for-agent"], &[], (0, 0), 0),
        ]),
    );
    let mut detail = login_pr;
    detail["body"] = json!(
        "## Result\nLogin works with passkeys.\n\n## Needs you\nPick the session timeout.\n\n## Look first\n- src/auth.rs:42 — the token check\n\n## How to try\nOpen http://localhost:3000/login.\n"
    );
    detail["files"] = json!([{"path": "src/auth.rs", "additions": 50, "deletions": 0}, {"path": "tests/auth_test.rs", "additions": 5, "deletions": 0}]);
    env.pr_view("o/app", 81, detail);
}

#[test]
fn overview_popup() {
    let env = Env::new();
    scenario(&env);
    let run = env.run(&["overview", "--print", "140x30"]);
    assert!(run.ok, "{}", run.stderr);
    check("overview", &env, &run.stdout);
}

#[test]
fn result_card_popup() {
    let env = Env::new();
    scenario(&env);
    let run = env.run(&["card", "--pane", "w1:p2", "--print", "140x24"]);
    assert!(run.ok, "{}", run.stderr);
    check("card", &env, &run.stdout);
}

#[test]
fn stale_overview_says_github_is_unreachable() {
    let env = Env::new();
    scenario(&env);
    env.run(&["overview", "--json"]).json();
    env.rule("gh", "*", "", 1);
    let run = env.run(&["overview", "--print", "140x3"]);
    assert!(run.ok, "{}", run.stderr);
    assert!(
        run.stdout
            .lines()
            .next()
            .unwrap()
            .contains("(GitHub unreachable)"),
        "{}",
        run.stdout
    );
}
