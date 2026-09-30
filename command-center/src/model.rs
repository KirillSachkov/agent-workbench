//! The Overview: every agent bound to its project, branch, PR and task, what needs the owner, what
//! changed since they last looked, and each project's work. Rebuilt from the sources on every call;
//! nothing here is task state.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::env::{canonical, now, state_dir};
use crate::github::{self, Pr, RepoFacts, Settings};
use crate::{git, herdr, words};

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub number: u64,
    /// Taken from the branch name, not from a PR's closing reference.
    pub inferred: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrRef {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub state: String,
    pub draft: bool,
    pub ci: String,
    pub ci_phrase: String,
    pub review: String,
}

impl PrRef {
    fn from(pr: &Pr) -> Self {
        PrRef {
            number: pr.number,
            title: pr.title.clone(),
            url: pr.url.clone(),
            state: pr.state.to_lowercase(),
            draft: pr.draft,
            ci: pr.ci.clone(),
            ci_phrase: words::ci(&pr.ci).into(),
            review: words::review(pr.review_decision.as_deref()).into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentView {
    /// Session plus canonical cwd; never the pane id alone.
    pub key: String,
    pub pane_id: String,
    pub session: Option<String>,
    pub agent: String,
    pub status: String,
    pub status_phrase: String,
    pub title: Option<String>,
    pub cwd: String,
    /// `folder`, `git` or `github`: how much the command center can tell about this agent.
    pub detail: String,
    pub project: Option<String>,
    pub github: Option<String>,
    pub worktree: Option<String>,
    pub linked_worktree: bool,
    pub branch: Option<String>,
    pub base: Option<String>,
    pub merge_base: Option<String>,
    pub task: Option<Task>,
    pub pr: Option<PrRef>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Spec {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub done: u64,
    pub total: u64,
    pub progress: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkItem {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub pr: Option<PrRef>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Work {
    pub specs: Vec<Spec>,
    pub ready: Vec<WorkItem>,
    pub in_flight: Vec<WorkItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub name: String,
    pub root: String,
    pub github: Option<String>,
    pub detail: String,
    /// GitHub could not be reached; the facts shown are the last known ones.
    pub stale: bool,
    pub github_error: Option<String>,
    pub agents: Vec<AgentView>,
    pub work: Option<Work>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub kind: String,
    pub phrase: String,
    pub text: String,
    pub project: Option<String>,
    pub pane_id: Option<String>,
    pub pr: Option<u64>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Counter {
    pub needs_you: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub generated_at: u64,
    pub current_project: Option<String>,
    pub needs_you: Vec<Item>,
    pub since_last_looked: Vec<Item>,
    pub projects: Vec<Project>,
    pub loose: Vec<AgentView>,
    pub counter: Counter,
}

pub struct Options {
    pub github: Settings,
    pub current: Option<PathBuf>,
    /// Keep only the current project.
    pub only_current: bool,
}

pub fn task_from_branch(branch: &str) -> Option<u64> {
    let re = Regex::new(r"^(?:[A-Za-z0-9._-]+/)?(\d+)[-_]").expect("valid regex");
    re.captures(branch).and_then(|c| c[1].parse().ok())
}

fn tag(task: Option<&Task>, pr: Option<&PrRef>) -> Option<String> {
    let mut parts = vec![];
    if let Some(task) = task {
        parts.push(format!(
            "#{}{}",
            task.number,
            if task.inferred { "?" } else { "" }
        ));
    }
    if let Some(pr) = pr {
        let mark = if pr.state == "merged" {
            " merged".to_string()
        } else {
            let m = words::ci_mark(&pr.ci);
            if m.is_empty() {
                String::new()
            } else {
                format!(" {m}")
            }
        };
        parts.push(format!("PR{}{}", pr.number, mark));
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

struct Bound {
    agent: herdr::Agent,
    cwd: PathBuf,
    git: Option<git::Facts>,
}

pub fn build(opts: &Options) -> Result<Snapshot, String> {
    let agents = herdr::agents()?;
    let mut git_cache: HashMap<PathBuf, Option<git::Facts>> = HashMap::new();
    let mut bound = vec![];
    for agent in agents {
        let cwd = canonical(Path::new(agent.cwd.as_deref().unwrap_or("/")));
        let facts = git_cache
            .entry(cwd.clone())
            .or_insert_with(|| git::facts(&cwd))
            .clone();
        bound.push(Bound {
            agent,
            cwd,
            git: facts,
        });
    }

    // Projects: every repository an agent runs in, plus the current one.
    let mut roots: BTreeMap<PathBuf, Option<String>> = BTreeMap::new();
    for b in &bound {
        if let Some(g) = &b.git {
            roots.insert(g.project_root.clone(), g.github.clone());
        }
    }
    let current = opts.current.as_deref().and_then(git::facts);
    if let Some(g) = &current {
        roots
            .entry(g.project_root.clone())
            .or_insert(g.github.clone());
    }

    let mut facts: HashMap<String, github::Loaded<RepoFacts>> = HashMap::new();
    for slug in roots.values().flatten() {
        if !facts.contains_key(slug) {
            facts.insert(slug.clone(), github::repo_facts(slug, &opts.github));
        }
    }

    let mut projects = vec![];
    let mut loose = vec![];
    for (root, slug) in &roots {
        let loaded = slug.as_ref().and_then(|s| facts.get(s));
        let repo = loaded.and_then(|l| l.facts.as_ref());
        let mut project = Project {
            name: root
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            root: root.display().to_string(),
            github: slug.clone(),
            detail: if slug.is_some() { "github" } else { "git" }.into(),
            stale: loaded.is_some_and(|l| l.stale),
            github_error: loaded.and_then(|l| l.error.clone()),
            agents: vec![],
            work: repo.map(work),
        };
        for b in bound
            .iter()
            .filter(|b| b.git.as_ref().is_some_and(|g| &g.project_root == root))
        {
            project.agents.push(view(b, Some(&project), repo));
        }
        projects.push(project);
    }
    for b in bound.iter().filter(|b| b.git.is_none()) {
        loose.push(view(b, None, None));
    }

    let needs_you = needs_you(&projects, &loose, &facts);
    let seen = load_seen();
    let since = seen
        .as_ref()
        .map(|s| since(s, &projects, &facts))
        .unwrap_or_default();
    let count = needs_you.len();
    let current_project = current.map(|g| g.project_root.display().to_string());

    let mut snapshot = Snapshot {
        generated_at: now(),
        current_project,
        needs_you,
        since_last_looked: since,
        projects,
        loose,
        counter: Counter {
            needs_you: count,
            text: match count {
                0 => String::new(),
                1 => "1 needs you".into(),
                n => format!("{n} need you"),
            },
        },
    };
    if opts.only_current {
        filter_current(&mut snapshot);
    }
    Ok(snapshot)
}

pub fn filter_current(snapshot: &mut Snapshot) {
    let Some(current) = snapshot.current_project.clone() else {
        return;
    };
    let name = snapshot
        .projects
        .iter()
        .find(|p| p.root == current)
        .map(|p| p.name.clone());
    snapshot.projects.retain(|p| p.root == current);
    snapshot.loose.clear();
    snapshot.needs_you.retain(|i| i.project == name);
    snapshot.since_last_looked.retain(|i| i.project == name);
}

fn view(b: &Bound, project: Option<&Project>, repo: Option<&RepoFacts>) -> AgentView {
    let g = b.git.as_ref();
    let branch = g.and_then(|g| g.branch.clone());
    let pr = branch.as_deref().and_then(|branch| {
        let repo = repo?;
        repo.open_prs.iter().find(|p| p.head_ref == branch)
    });
    let task = match pr.and_then(|p| p.closes.first()) {
        Some(n) => Some(Task {
            number: *n,
            inferred: false,
        }),
        None => branch
            .as_deref()
            .and_then(task_from_branch)
            .map(|number| Task {
                number,
                inferred: true,
            }),
    };
    let pr = pr.map(PrRef::from);
    let session = b.agent.session();
    let cwd = b.cwd.display().to_string();
    let key = match &session {
        Some(s) => format!("{s}@{cwd}"),
        None => format!("{}@{cwd}", b.agent.terminal_id),
    };
    AgentView {
        key,
        pane_id: b.agent.pane_id.clone(),
        session,
        agent: b.agent.name(),
        status: b.agent.agent_status.clone(),
        status_phrase: words::status(&b.agent.agent_status).into(),
        title: b
            .agent
            .terminal_title_stripped
            .clone()
            .filter(|t| !t.is_empty()),
        cwd,
        detail: match (g, project.and_then(|p| p.github.as_ref())) {
            (None, _) => "folder",
            (Some(_), None) => "git",
            (Some(_), Some(_)) => "github",
        }
        .into(),
        project: project.map(|p| p.name.clone()),
        github: project.and_then(|p| p.github.clone()),
        worktree: g.map(|g| g.worktree.display().to_string()),
        linked_worktree: g.is_some_and(|g| g.linked),
        branch,
        base: g.and_then(|g| g.base.clone()),
        merge_base: g.and_then(|g| g.merge_base.clone()),
        tag: tag(task.as_ref(), pr.as_ref()),
        task,
        pr,
    }
}

fn work(repo: &RepoFacts) -> Work {
    let mut w = Work::default();
    for issue in &repo.issues {
        if issue.sub_total > 0 {
            w.specs.push(Spec {
                number: issue.number,
                title: issue.title.clone(),
                url: issue.url.clone(),
                done: issue.sub_done,
                total: issue.sub_total,
                progress: format!("{} of {} done", issue.sub_done, issue.sub_total),
            });
            continue;
        }
        let item = |pr: Option<PrRef>| WorkItem {
            number: issue.number,
            title: issue.title.clone(),
            url: issue.url.clone(),
            labels: issue.labels.iter().map(|l| words::label(l)).collect(),
            assignees: issue.assignees.clone(),
            pr,
        };
        if !issue.assignees.is_empty() {
            let pr = repo
                .open_prs
                .iter()
                .find(|p| p.closes.contains(&issue.number))
                .or_else(|| {
                    repo.open_prs
                        .iter()
                        .find(|p| task_from_branch(&p.head_ref) == Some(issue.number))
                });
            w.in_flight.push(item(pr.map(PrRef::from)));
        } else if issue.labels.iter().any(|l| l == "ready-for-agent") && issue.blocked_by == 0 {
            w.ready.push(item(None));
        }
    }
    w
}

fn needs_you(
    projects: &[Project],
    loose: &[AgentView],
    facts: &HashMap<String, github::Loaded<RepoFacts>>,
) -> Vec<Item> {
    let agents = projects.iter().flat_map(|p| p.agents.iter()).chain(loose);
    let mut items: Vec<Item> = agents
        .filter(|a| a.status == "blocked" || a.status == "done")
        .map(|a| {
            agent_item(
                a,
                if a.status == "blocked" {
                    "agent_waiting"
                } else {
                    "agent_done"
                },
            )
        })
        .collect();
    for p in projects {
        let Some(repo) = p
            .github
            .as_ref()
            .and_then(|s| facts.get(s))
            .and_then(|l| l.facts.as_ref())
        else {
            continue;
        };
        for pr in &repo.open_prs {
            if pr.ci == "failure" {
                items.push(pr_item(p, pr, "ci_failed", "CI failed"));
            } else if !pr.draft
                && pr
                    .labels
                    .iter()
                    .all(|l| !l.starts_with("acceptance:") || l == "acceptance:human")
                && pr.ci == "success"
            {
                items.push(pr_item(p, pr, "pr_ready", "Ready for your acceptance"));
            }
        }
    }
    items
}

fn agent_item(a: &AgentView, kind: &str) -> Item {
    let what = a
        .title
        .clone()
        .or_else(|| a.branch.clone())
        .unwrap_or_default();
    Item {
        kind: kind.into(),
        phrase: a.status_phrase.clone(),
        text: format!(
            "{} · {}{}",
            a.project.clone().unwrap_or_else(|| a.cwd.clone()),
            a.agent,
            if what.is_empty() {
                String::new()
            } else {
                format!(" · {what}")
            }
        ),
        project: a.project.clone(),
        pane_id: Some(a.pane_id.clone()),
        pr: a.pr.as_ref().map(|p| p.number),
        url: None,
    }
}

fn pr_item(p: &Project, pr: &Pr, kind: &str, phrase: &str) -> Item {
    Item {
        kind: kind.into(),
        phrase: phrase.into(),
        text: format!("{} · PR #{} {}", p.name, pr.number, pr.title),
        project: Some(p.name.clone()),
        pane_id: None,
        pr: Some(pr.number),
        url: Some(pr.url.clone()),
    }
}

/// What the Overview showed when the owner last opened it. Display state, not task state.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Seen {
    pub saved_at: u64,
    pub repos: BTreeSet<String>,
    pub agents: BTreeMap<String, String>,
    pub prs: BTreeMap<String, String>,
    pub merged: BTreeSet<String>,
}

fn seen_path() -> PathBuf {
    state_dir().join("last-looked.json")
}

fn load_seen() -> Option<Seen> {
    serde_json::from_str(&std::fs::read_to_string(seen_path()).ok()?).ok()
}

pub fn mark_seen(snapshot: &Snapshot) -> Result<(), String> {
    let mut seen = Seen {
        saved_at: snapshot.generated_at,
        ..Seen::default()
    };
    // What is not visible now (another project while filtered, an unreachable repository) keeps
    // its previous baseline.
    if let Some(prev) = load_seen() {
        seen = Seen {
            saved_at: seen.saved_at,
            ..prev
        };
    }
    for p in &snapshot.projects {
        for a in &p.agents {
            seen.agents.insert(a.key.clone(), a.status.clone());
        }
        let Some(slug) = &p.github else { continue };
        let Some(repo) = github::cached_repo_facts(slug) else {
            continue;
        };
        seen.repos.insert(slug.clone());
        seen.prs.retain(|k, _| !k.starts_with(&format!("{slug}#")));
        for pr in &repo.open_prs {
            seen.prs
                .insert(format!("{slug}#{}", pr.number), pr.ci.clone());
        }
        for pr in &repo.merged_prs {
            seen.merged.insert(format!("{slug}#{}", pr.number));
        }
    }
    for a in &snapshot.loose {
        seen.agents.insert(a.key.clone(), a.status.clone());
    }
    let dir = state_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(
        seen_path(),
        serde_json::to_string_pretty(&seen).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn since(
    seen: &Seen,
    projects: &[Project],
    facts: &HashMap<String, github::Loaded<RepoFacts>>,
) -> Vec<Item> {
    let mut items = vec![];
    for p in projects {
        for a in &p.agents {
            if seen
                .agents
                .get(&a.key)
                .is_some_and(|before| before == "working")
                && (a.status == "done" || a.status == "idle")
            {
                let mut item = agent_item(a, "agent_finished");
                item.phrase = "Finished".into();
                items.push(item);
            }
        }
        let Some(slug) = &p.github else { continue };
        if !seen.repos.contains(slug) {
            continue;
        }
        let Some(repo) = facts.get(slug).and_then(|l| l.facts.as_ref()) else {
            continue;
        };
        for pr in &repo.open_prs {
            let key = format!("{slug}#{}", pr.number);
            match seen.prs.get(&key) {
                None => items.push(pr_item(p, pr, "pr_opened", "PR opened")),
                Some(before) if before == "failure" => continue,
                Some(_) => {}
            }
            if pr.ci == "failure" {
                items.push(pr_item(p, pr, "ci_failed", "CI failed"));
            }
        }
        for pr in &repo.merged_prs {
            if !seen.merged.contains(&format!("{slug}#{}", pr.number)) {
                items.push(pr_item(p, pr, "pr_merged", "PR merged"));
            }
        }
    }
    items
}
