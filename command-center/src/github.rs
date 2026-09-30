//! Facts from GitHub, read with `gh` and reused between refreshes.
//!
//! PR titles and bodies are untrusted text: they are parsed into fixed fields, never executed.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::env::{now, state_dir};
use crate::run::run;

const PR_FIELDS: &str = "number,title,url,state,isDraft,headRefName,baseRefName,labels,closingIssuesReferences,statusCheckRollup,reviewDecision,headRefOid,additions,deletions,changedFiles";
const MERGED_FIELDS: &str = "number,title,url,headRefName,mergedAt";
const CARD_FIELDS: &str = "number,title,url,state,isDraft,body,headRefName,baseRefName,labels,closingIssuesReferences,statusCheckRollup,reviewDecision,headRefOid,additions,deletions,changedFiles,files";
const ISSUES_QUERY: &str = "query($owner:String!,$name:String!){repository(owner:$owner,name:$name){issues(first:100,states:OPEN,orderBy:{field:CREATED_AT,direction:ASC}){nodes{number title url assignees(first:10){nodes{login}} labels(first:20){nodes{name}} subIssuesSummary{total completed} issueDependenciesSummary{blockedBy}}}}}";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pr {
    pub number: u64,
    pub title: String,
    pub url: String,
    /// `OPEN`, `MERGED` or `CLOSED`.
    pub state: String,
    pub draft: bool,
    pub head_ref: String,
    pub base_ref: String,
    pub labels: Vec<String>,
    pub closes: Vec<u64>,
    /// `success`, `failure`, `pending` or `none`.
    pub ci: String,
    pub review_decision: Option<String>,
    pub head_sha: String,
    pub additions: u64,
    pub deletions: u64,
    pub changed_files: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub sub_total: u64,
    pub sub_done: u64,
    pub blocked_by: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoFacts {
    pub open_prs: Vec<Pr>,
    pub merged_prs: Vec<Pr>,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Cached<T> {
    fetched_at: u64,
    last_error: Option<String>,
    facts: Option<T>,
}

/// Facts plus how fresh they are.
pub struct Loaded<T> {
    pub facts: Option<T>,
    /// The last fetch failed; `facts` are the last known ones.
    pub stale: bool,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct Settings {
    pub refresh_seconds: u64,
    pub timeout: Duration,
    /// False: use only what is cached (the first frame of an interactive view).
    pub network: bool,
}

fn gh(args: &[&str], timeout: Duration) -> Result<Value, String> {
    let out = run("gh", args, None, Some(timeout));
    if !out.ok {
        return Err(format!(
            "gh {}: {}",
            args.first().unwrap_or(&""),
            out.error_line()
        ));
    }
    serde_json::from_str(&out.stdout).map_err(|err| format!("gh: unreadable output: {err}"))
}

fn cache_file(key: &str) -> PathBuf {
    state_dir()
        .join("github")
        .join(format!("{}.json", key.replace('/', "__")))
}

fn read_cache<T: for<'de> Deserialize<'de> + Default>(key: &str) -> Cached<T> {
    std::fs::read_to_string(cache_file(key))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write_cache<T: Serialize>(key: &str, cached: &Cached<T>) {
    let path = cache_file(key);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string(cached) {
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(tmp, path);
        }
    }
}

fn load<T, F>(key: &str, settings: &Settings, fetch: F) -> Loaded<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Default + Clone,
    F: FnOnce() -> Result<T, String>,
{
    let mut cached: Cached<T> = read_cache(key);
    let fresh = cached.facts.is_some()
        && now().saturating_sub(cached.fetched_at) < settings.refresh_seconds;
    if settings.network && !fresh {
        match fetch() {
            Ok(facts) => {
                cached = Cached {
                    fetched_at: now(),
                    last_error: None,
                    facts: Some(facts),
                }
            }
            Err(err) => cached.last_error = Some(err),
        }
        write_cache(key, &cached);
    }
    Loaded {
        stale: cached.last_error.is_some(),
        error: cached.last_error,
        facts: cached.facts,
    }
}

pub fn repo_facts(slug: &str, settings: &Settings) -> Loaded<RepoFacts> {
    load(slug, settings, || fetch_repo(slug, settings.timeout))
}

/// The last fetched facts, without going to GitHub.
pub fn cached_repo_facts(slug: &str) -> Option<RepoFacts> {
    read_cache::<RepoFacts>(slug).facts
}

fn fetch_repo(slug: &str, timeout: Duration) -> Result<RepoFacts, String> {
    let (owner, name) = slug.split_once('/').ok_or("bad repository")?;
    let open = gh(
        &[
            "pr", "list", "--repo", slug, "--state", "open", "--limit", "100", "--json", PR_FIELDS,
        ],
        timeout,
    )?;
    let merged = gh(
        &[
            "pr",
            "list",
            "--repo",
            slug,
            "--state",
            "merged",
            "--limit",
            "20",
            "--json",
            MERGED_FIELDS,
        ],
        timeout,
    )?;
    let issues = gh(
        &[
            "api",
            "graphql",
            "-f",
            &format!("query={ISSUES_QUERY}"),
            "-F",
            &format!("owner={owner}"),
            "-F",
            &format!("name={name}"),
        ],
        timeout,
    )?;
    let nodes = issues
        .pointer("/data/repository/issues/nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(RepoFacts {
        open_prs: as_list(&open).iter().map(pr_from).collect(),
        merged_prs: as_list(&merged)
            .iter()
            .map(|v| Pr {
                state: "MERGED".into(),
                ..pr_from(v)
            })
            .collect(),
        issues: nodes.iter().map(issue_from).collect(),
    })
}

/// A PR with its body and files, for the result card.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrDetail {
    pub pr: Option<Pr>,
    pub body: String,
    pub files: Vec<crate::git::FileChange>,
}

pub fn pr_detail(slug: &str, number: u64, settings: &Settings) -> Loaded<PrDetail> {
    load(&format!("{slug}#{number}"), settings, || {
        let v = gh(
            &[
                "pr",
                "view",
                &number.to_string(),
                "--repo",
                slug,
                "--json",
                CARD_FIELDS,
            ],
            settings.timeout,
        )?;
        let files = as_list(&v["files"])
            .iter()
            .map(|f| crate::git::FileChange {
                path: text(&f["path"]),
                additions: f["additions"].as_u64().unwrap_or(0),
                deletions: f["deletions"].as_u64().unwrap_or(0),
            })
            .collect();
        Ok(PrDetail {
            pr: Some(pr_from(&v)),
            body: text(&v["body"]),
            files,
        })
    })
}

pub fn open_in_browser(slug: &str, number: u64) -> Result<(), String> {
    let out = run(
        "gh",
        &["pr", "view", &number.to_string(), "--repo", slug, "--web"],
        None,
        Some(Duration::from_secs(20)),
    );
    if out.ok {
        Ok(())
    } else {
        Err(format!("gh pr view --web: {}", out.error_line()))
    }
}

fn as_list(v: &Value) -> Vec<Value> {
    v.as_array().cloned().unwrap_or_default()
}

fn text(v: &Value) -> String {
    v.as_str().unwrap_or("").to_string()
}

fn names(v: &Value, field: &str) -> Vec<String> {
    let list = v.get("nodes").unwrap_or(v);
    as_list(list)
        .iter()
        .map(|item| text(&item[field]))
        .filter(|s| !s.is_empty())
        .collect()
}

fn pr_from(v: &Value) -> Pr {
    let review = text(&v["reviewDecision"]);
    Pr {
        number: v["number"].as_u64().unwrap_or(0),
        title: text(&v["title"]),
        url: text(&v["url"]),
        state: text(&v["state"]),
        draft: v["isDraft"].as_bool().unwrap_or(false),
        head_ref: text(&v["headRefName"]),
        base_ref: text(&v["baseRefName"]),
        labels: names(&v["labels"], "name"),
        closes: as_list(&v["closingIssuesReferences"])
            .iter()
            .filter_map(|i| i["number"].as_u64())
            .collect(),
        ci: ci_rollup(&v["statusCheckRollup"]),
        review_decision: (!review.is_empty()).then_some(review),
        head_sha: text(&v["headRefOid"]),
        additions: v["additions"].as_u64().unwrap_or(0),
        deletions: v["deletions"].as_u64().unwrap_or(0),
        changed_files: v["changedFiles"].as_u64().unwrap_or(0),
    }
}

fn issue_from(v: &Value) -> Issue {
    Issue {
        number: v["number"].as_u64().unwrap_or(0),
        title: text(&v["title"]),
        url: text(&v["url"]),
        assignees: names(&v["assignees"], "login"),
        labels: names(&v["labels"], "name"),
        sub_total: v
            .pointer("/subIssuesSummary/total")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        sub_done: v
            .pointer("/subIssuesSummary/completed")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        blocked_by: v
            .pointer("/issueDependenciesSummary/blockedBy")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    }
}

/// One word for a PR's checks: any failure wins, then anything unfinished.
fn ci_rollup(checks: &Value) -> String {
    let checks = as_list(checks);
    if checks.is_empty() {
        return "none".into();
    }
    let mut pending = false;
    for check in &checks {
        let conclusion = text(&check["conclusion"]).to_uppercase();
        let status = text(&check["status"]).to_uppercase();
        let state = text(&check["state"]).to_uppercase();
        let failed = [
            "FAILURE",
            "TIMED_OUT",
            "CANCELLED",
            "ACTION_REQUIRED",
            "STARTUP_FAILURE",
            "ERROR",
        ];
        if failed.contains(&conclusion.as_str()) || failed.contains(&state.as_str()) {
            return "failure".into();
        }
        let unfinished = (!status.is_empty() && status != "COMPLETED")
            || state == "PENDING"
            || state == "EXPECTED";
        pending |= unfinished;
    }
    if pending {
        "pending".into()
    } else {
        "success".into()
    }
}
