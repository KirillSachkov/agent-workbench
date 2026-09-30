mod support;

use serde_json::{Value, json};
use support::{Env, agent, issue, pr};

/// One GitHub project with three agents (main worktree, two linked worktrees) and a loose folder.
fn scenario(env: &Env) {
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    let login = env.worktree(&repo, "app-62", "feat/62-login");
    let search = env.worktree(&repo, "app-70", "feat/70-search");
    std::fs::create_dir_all(env.path("loose")).unwrap();
    env.agents(json!([
        agent("w1:p1", "s-main", "working", &repo),
        agent("w1:p2", "s-login", "done", &login),
        agent("w1:p3", "s-search", "blocked", &search),
        agent("w1:p4", "s-loose", "idle", &env.path("loose")),
    ]));
    let mut login_pr = pr(81, "feat/62-login", "success");
    login_pr["closingIssuesReferences"] = json!([{"number": 62}]);
    let mut draft = pr(91, "draft-work", "success");
    draft["isDraft"] = json!(true);
    let mut auto = pr(92, "auto-work", "success");
    auto["labels"] = json!([{"name": "acceptance:auto"}]);
    env.github(
        "o/app",
        json!([login_pr, pr(90, "broken", "failure"), draft, auto]),
        json!([]),
        json!([]),
    );
}

fn agent_by_pane<'a>(snapshot: &'a Value, pane: &str) -> &'a Value {
    snapshot["projects"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["agents"].as_array().unwrap())
        .chain(snapshot["loose"].as_array().unwrap())
        .find(|a| a["pane_id"] == pane)
        .unwrap_or_else(|| panic!("no agent {pane} in {snapshot:#}"))
}

#[test]
fn binds_agents_to_project_branch_pr_and_task() {
    let env = Env::new();
    scenario(&env);
    let s = env.run(&["overview", "--json"]).json();

    let projects = s["projects"].as_array().unwrap();
    assert_eq!(
        projects.len(),
        1,
        "linked worktrees are grouped under their main repository"
    );
    assert_eq!(projects[0]["name"], "app");
    assert_eq!(projects[0]["github"], "o/app");
    assert_eq!(projects[0]["root"], env.canonical("repos/app"));
    assert_eq!(projects[0]["agents"].as_array().unwrap().len(), 3);

    let main = agent_by_pane(&s, "w1:p1");
    assert_eq!(main["branch"], "main");
    assert_eq!(main["linked_worktree"], false);
    assert_eq!(main["task"], Value::Null);
    assert_eq!(main["tag"], Value::Null);

    let login = agent_by_pane(&s, "w1:p2");
    assert_eq!(login["project"], "app");
    assert_eq!(login["worktree"], env.canonical("repos/app-62"));
    assert_eq!(login["linked_worktree"], true);
    assert_eq!(login["branch"], "feat/62-login");
    assert_eq!(
        login["pr"]["number"], 81,
        "the PR is found by its head branch"
    );
    assert_eq!(login["pr"]["ci"], "success");
    assert_eq!(
        login["task"],
        json!({"number": 62, "inferred": false}),
        "the task comes from the closing reference"
    );
    assert_eq!(login["tag"], "#62 PR81 ✓");
    assert_eq!(login["detail"], "github");

    let search = agent_by_pane(&s, "w1:p3");
    assert_eq!(search["pr"], Value::Null);
    assert_eq!(
        search["task"],
        json!({"number": 70, "inferred": true}),
        "without a PR the task is inferred from the branch"
    );
    assert_eq!(search["tag"], "#70?");

    let loose = &s["loose"].as_array().unwrap()[0];
    assert_eq!(loose["pane_id"], "w1:p4");
    assert_eq!(loose["detail"], "folder");
    assert_eq!(loose["status_phrase"], "Idle");
    assert_eq!(loose["branch"], Value::Null);
}

#[test]
fn a_repository_without_github_shows_git_facts_only() {
    let env = Env::new();
    let repo = env.repo("local", None);
    let wt = env.worktree(&repo, "local-5", "fix/5-crash");
    env.agents(json!([agent("w1:p1", "s1", "working", &wt)]));
    let s = env.run(&["overview", "--json"]).json();
    assert_eq!(s["projects"][0]["github"], Value::Null);
    assert_eq!(s["projects"][0]["detail"], "git");
    let a = agent_by_pane(&s, "w1:p1");
    assert_eq!(a["detail"], "git");
    assert_eq!(a["branch"], "fix/5-crash");
    assert_eq!(a["task"], json!({"number": 5, "inferred": true}));
    assert!(
        env.calls("gh").is_empty(),
        "no GitHub calls without a GitHub remote"
    );
}

#[test]
fn needs_you_lists_waiting_agents_unseen_results_ready_prs_and_failed_ci() {
    let env = Env::new();
    scenario(&env);
    let s = env.run(&["overview", "--json"]).json();
    let items: Vec<(String, String)> = s["needs_you"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["kind"].as_str().unwrap().to_string(),
                i["phrase"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert!(items.contains(&("agent_done".into(), "Done, not seen yet".into())));
    assert!(items.contains(&("agent_waiting".into(), "Waits for you".into())));
    let prs: Vec<(String, u64)> = s["needs_you"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| !i["pr"].is_null() && i["pane_id"].is_null())
        .map(|i| {
            (
                i["kind"].as_str().unwrap().to_string(),
                i["pr"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        prs,
        vec![("pr_ready".to_string(), 81), ("ci_failed".to_string(), 90)],
        "drafts and acceptance:auto PRs do not wait for the owner"
    );
    assert_eq!(s["counter"], json!({"needs_you": 4, "text": "4 need you"}));
}

#[test]
fn since_last_looked_shows_what_changed_after_the_overview_was_opened() {
    let env = Env::new();
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    let wt = env.worktree(&repo, "app-62", "feat/62-login");
    env.agents(json!([agent("w1:p1", "s1", "working", &wt)]));
    env.github(
        "o/app",
        json!([pr(81, "feat/62-login", "pending")]),
        json!([pr(70, "old", "success")]),
        json!([]),
    );

    let first = env.run(&["overview", "--json", "--mark-seen"]).json();
    assert_eq!(
        first["since_last_looked"],
        json!([]),
        "nothing to compare with on the first look"
    );

    env.agents(json!([agent("w1:p1", "s1", "done", &wt)]));
    let mut merged = pr(75, "feat/75-x", "success");
    merged["state"] = json!("MERGED");
    env.github(
        "o/app",
        json!([
            pr(81, "feat/62-login", "failure"),
            pr(82, "feat/64-new", "success")
        ]),
        json!([merged, pr(70, "old", "success")]),
        json!([]),
    );
    let second = env.run(&["overview", "--json"]).json();
    let kinds: Vec<(String, Option<u64>)> = second["since_last_looked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| (i["kind"].as_str().unwrap().to_string(), i["pr"].as_u64()))
        .collect();
    assert!(
        kinds.contains(&("agent_finished".into(), Some(81))),
        "{kinds:?}"
    );
    assert!(kinds.contains(&("ci_failed".into(), Some(81))), "{kinds:?}");
    assert!(kinds.contains(&("pr_opened".into(), Some(82))), "{kinds:?}");
    assert!(kinds.contains(&("pr_merged".into(), Some(75))), "{kinds:?}");
    assert_eq!(kinds.len(), 4, "{kinds:?}");

    // Looking again without marking keeps the same baseline; marking resets it.
    env.run(&["overview", "--json", "--mark-seen"]).json();
    let third = env.run(&["overview", "--json"]).json();
    assert_eq!(third["since_last_looked"], json!([]));
}

#[test]
fn work_section_shows_progress_ready_and_in_flight_work() {
    let env = Env::new();
    let repo = env.repo("app", Some("git@github.com:o/app.git"));
    env.agents(json!([agent("w1:p1", "s1", "idle", &repo)]));
    let mut closing = pr(81, "some-branch", "pending");
    closing["closingIssuesReferences"] = json!([{"number": 12}]);
    env.github(
        "o/app",
        json!([closing, pr(82, "feat/13-by-branch", "success")]),
        json!([]),
        json!([
            issue(10, "Spec: login", &[], &[], (5, 3), 0),
            issue(11, "Ready task", &["ready-for-agent"], &[], (0, 0), 0),
            issue(
                12,
                "Claimed by PR reference",
                &["ready-for-agent"],
                &["alice"],
                (0, 0),
                0
            ),
            issue(13, "Claimed by branch name", &[], &["bob"], (0, 0), 0),
            issue(14, "Blocked task", &["ready-for-agent"], &[], (0, 0), 1),
            issue(15, "Needs triage", &["needs-triage"], &[], (0, 0), 0),
        ]),
    );
    let s = env.run(&["overview", "--json"]).json();
    let work = &s["projects"][0]["work"];
    assert_eq!(work["specs"][0]["number"], 10);
    assert_eq!(work["specs"][0]["progress"], "3 of 5 done");
    let ready: Vec<u64> = work["ready"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["number"].as_u64().unwrap())
        .collect();
    assert_eq!(
        ready,
        vec![11],
        "only unblocked, unclaimed issues labelled for agents"
    );
    assert_eq!(
        work["ready"][0]["labels"],
        json!(["Ready for an agent"]),
        "labels in plain words"
    );
    let flight: Vec<(u64, u64)> = work["in_flight"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["number"].as_u64().unwrap(),
                i["pr"]["number"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(flight, vec![(12, 81), (13, 82)]);
    assert_eq!(work["in_flight"][0]["assignees"], json!(["alice"]));
}

#[test]
fn finished_children_leave_ready_and_count_towards_progress() {
    let env = Env::new();
    let repo = env.repo("app", Some("https://github.com/o/app"));
    env.agents(json!([agent("w1:p1", "s1", "idle", &repo)]));
    env.github(
        "o/app",
        json!([]),
        json!([]),
        json!([
            issue(10, "Spec", &[], &[], (2, 1), 0),
            issue(11, "Next", &["ready-for-agent"], &[], (0, 0), 1)
        ]),
    );
    let before = env.run(&["overview", "--json"]).json();
    assert_eq!(before["projects"][0]["work"]["ready"], json!([]));
    // The blocker closed: GitHub now reports it done and the next task unblocked.
    env.github(
        "o/app",
        json!([]),
        json!([]),
        json!([
            issue(10, "Spec", &[], &[], (2, 2), 0),
            issue(11, "Next", &["ready-for-agent"], &[], (0, 0), 0)
        ]),
    );
    let after = env.run(&["overview", "--json"]).json();
    assert_eq!(
        after["projects"][0]["work"]["specs"][0]["progress"],
        "2 of 2 done"
    );
    assert_eq!(after["projects"][0]["work"]["ready"][0]["number"], 11);
}

#[test]
fn the_current_repository_is_a_project_even_without_agents() {
    let env = Env::new();
    let repo = env.repo("app", Some("https://github.com/o/app.git"));
    let other = env.repo("other", None);
    env.agents(json!([agent("w1:p1", "s1", "idle", &other)]));
    env.github(
        "o/app",
        json!([]),
        json!([]),
        json!([issue(11, "Ready", &["ready-for-agent"], &[], (0, 0), 0)]),
    );
    let s = env
        .run(&["overview", "--json", "--current", repo.to_str().unwrap()])
        .json();
    let names: Vec<&str> = s["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["app", "other"]);
    assert_eq!(s["current_project"], env.canonical("repos/app"));

    let only = env
        .run(&[
            "overview",
            "--json",
            "--this-project",
            "--current",
            repo.to_str().unwrap(),
        ])
        .json();
    assert_eq!(only["projects"].as_array().unwrap().len(), 1);
    assert_eq!(only["projects"][0]["work"]["ready"][0]["number"], 11);
}

#[test]
fn github_failure_shows_the_last_known_facts_marked_stale() {
    let env = Env::new();
    scenario(&env);
    let fresh = env.run(&["overview", "--json"]).json();
    assert_eq!(fresh["projects"][0]["stale"], false);

    env.rule("gh", "*", "", 1);
    let stale = env.run(&["overview", "--json"]).json();
    assert_eq!(stale["projects"][0]["stale"], true);
    assert!(
        stale["projects"][0]["github_error"]
            .as_str()
            .unwrap()
            .starts_with("gh pr")
    );
    assert_eq!(
        agent_by_pane(&stale, "w1:p2")["pr"]["number"],
        81,
        "the last known PR is still shown"
    );
}

#[test]
fn github_facts_are_reused_between_refreshes() {
    let env = Env::new();
    scenario(&env);
    env.write_config("github_refresh_seconds = 600\n");
    env.run(&["overview", "--json"]).json();
    let after_first = env.calls("gh").len();
    assert_eq!(after_first, 3);
    env.run(&["overview", "--json"]).json();
    assert_eq!(
        env.calls("gh").len(),
        after_first,
        "fresh facts are not fetched again"
    );
}
