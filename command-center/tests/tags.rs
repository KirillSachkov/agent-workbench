mod support;

use serde_json::json;
use support::{Env, agent, pr};

fn setup(env: &Env, shown: Option<&str>) {
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    let wt = env.worktree(&repo, "app-62", "feat/62-login");
    let mut tagged = agent("w1:p2", "s-login", "blocked", &wt);
    if let Some(tag) = shown {
        tagged["tokens"] = json!({"wb_tag": tag});
    }
    let mut stale_tag = agent("w1:p1", "s-main", "working", &repo);
    stale_tag["tokens"] = json!({"wb_tag": "#9 PR1"});
    env.agents(json!([stale_tag, tagged]));
    let mut listed = pr(81, "feat/62-login", "failure");
    listed["closingIssuesReferences"] = json!([{"number": 62}]);
    env.github("o/app", json!([listed]), json!([]), json!([]));
    env.clear_calls();
}

fn reports(env: &Env) -> Vec<String> {
    env.calls("herdr")
        .into_iter()
        .filter(|c| c.starts_with("pane report-metadata"))
        .collect()
}

#[test]
fn refresh_reports_changed_tags_and_clears_tags_that_no_longer_apply() {
    let env = Env::new();
    setup(&env, None);
    assert!(env.run(&["refresh"]).ok);
    assert_eq!(
        reports(&env),
        vec![
            "pane report-metadata w1:p1 --source agent-workbench --clear-token wb_tag".to_string(),
            "pane report-metadata w1:p2 --source agent-workbench --token wb_tag=#62 PR81 ✗"
                .to_string(),
        ]
    );
}

#[test]
fn refresh_skips_tags_herdr_already_shows_unless_forced() {
    let env = Env::new();
    setup(&env, Some("#62 PR81 ✗"));
    assert!(env.run(&["refresh"]).ok);
    assert_eq!(
        reports(&env).len(),
        1,
        "only the stale tag on w1:p1 changes"
    );
    env.clear_calls();
    assert!(env.run(&["refresh", "--force"]).ok);
    assert_eq!(
        reports(&env).len(),
        2,
        "after a Herdr restart every tag is reported again"
    );
}

#[test]
fn tab_status_prints_the_needs_you_counter() {
    let env = Env::new();
    setup(&env, None);
    let run = env.run(&["tab-status"]);
    assert!(run.ok, "{}", run.stderr);
    // The blocked agent and the PR whose CI failed.
    assert_eq!(run.stdout, "2 need you\n");
    assert_eq!(reports(&env).len(), 2);
}

#[test]
fn tab_status_is_empty_when_nothing_needs_the_owner() {
    let env = Env::new();
    let repo = env.repo("app", None);
    env.agents(json!([agent("w1:p1", "s1", "working", &repo)]));
    let run = env.run(&["tab-status"]);
    assert!(run.ok, "{}", run.stderr);
    assert_eq!(run.stdout, "\n");
}
