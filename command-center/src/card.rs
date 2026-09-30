//! The result card: the report an agent wrote into its PR, in the documented shape (`Result`,
//! `Needs you`, `Look first` with `path:line — reason`, `How to try`), plus facts the command center
//! derives. A body in any other shape is shown whole. The text is data: never executed.

use std::path::Path;

use regex::Regex;
use serde::Serialize;

use crate::git::{self, FileChange};
use crate::github::{self, Settings};
use crate::model::AgentView;
use crate::words;

#[derive(Debug, Clone, Serialize)]
pub struct LookFirst {
    pub path: String,
    pub line: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Facts {
    pub status: String,
    pub branch: Option<String>,
    pub worktree: Option<String>,
    pub ci: Option<String>,
    pub review: Option<String>,
    pub head_sha: Option<String>,
    pub files_changed: usize,
    pub additions: u64,
    pub deletions: u64,
    pub tests_changed: bool,
    pub ci_config_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Card {
    pub agent: AgentView,
    /// `result-card` (the documented shape), `plain` (any other PR body) or `no-pr`.
    pub shape: String,
    pub result: Option<String>,
    pub needs_you: Option<String>,
    pub look_first: Vec<LookFirst>,
    pub how_to_try: Option<String>,
    /// The first http(s) address under `How to try`.
    pub app_url: Option<String>,
    pub other_sections: Vec<Section>,
    /// The whole PR body, when it does not follow the shape.
    pub body: Option<String>,
    pub facts: Facts,
    pub files: Vec<FileChange>,
    pub stale: bool,
}

pub fn build(agent: AgentView, settings: &Settings) -> Card {
    let mut card = Card {
        shape: "no-pr".into(),
        result: None,
        needs_you: None,
        look_first: vec![],
        how_to_try: None,
        app_url: None,
        other_sections: vec![],
        body: None,
        facts: Facts {
            status: agent.status_phrase.clone(),
            branch: agent.branch.clone(),
            worktree: agent.worktree.clone(),
            ..Facts::default()
        },
        files: vec![],
        stale: false,
        agent,
    };
    let detail = match (&card.agent.github, &card.agent.pr) {
        (Some(slug), Some(pr)) => Some(github::pr_detail(slug, pr.number, settings)),
        _ => None,
    };
    match detail
        .as_ref()
        .and_then(|d| d.facts.as_ref().map(|f| (f, d.stale)))
    {
        Some((detail, stale)) => {
            card.stale = stale;
            card.files = detail.files.clone();
            if let Some(pr) = &detail.pr {
                card.facts.ci = Some(words::ci(&pr.ci).into());
                card.facts.review = Some(words::review(pr.review_decision.as_deref()).into());
                card.facts.head_sha = Some(pr.head_sha.clone());
                card.facts.additions = pr.additions;
                card.facts.deletions = pr.deletions;
            }
            read_body(&mut card, &detail.body);
        }
        None => {
            if let (Some(worktree), Some(base)) = (&card.agent.worktree, &card.agent.merge_base) {
                card.files = git::changed_files(Path::new(worktree), base);
                card.facts.head_sha = git::head(Path::new(worktree));
            }
            card.facts.additions = card.files.iter().map(|f| f.additions).sum();
            card.facts.deletions = card.files.iter().map(|f| f.deletions).sum();
        }
    }
    card.facts.files_changed = card.files.len();
    card.facts.tests_changed = card.files.iter().any(|f| is_test(&f.path));
    card.facts.ci_config_changed = card.files.iter().any(|f| is_ci_config(&f.path));
    card
}

fn read_body(card: &mut Card, body: &str) {
    let sections = split_sections(body);
    let known = ["result", "needs you", "look first", "how to try"];
    if !sections
        .iter()
        .any(|s| known.contains(&normalise(&s.title).as_str()))
    {
        card.shape = "plain".into();
        card.body = Some(body.trim().to_string());
        return;
    }
    card.shape = "result-card".into();
    for section in sections {
        match normalise(&section.title).as_str() {
            "result" => card.result = Some(section.text),
            "needs you" => card.needs_you = Some(section.text),
            "look first" => card.look_first = look_first(&section.text),
            "how to try" => {
                card.app_url = first_url(&section.text);
                card.how_to_try = Some(section.text);
            }
            _ if section.title.is_empty() && section.text.is_empty() => {}
            _ => card.other_sections.push(section),
        }
    }
}

fn normalise(title: &str) -> String {
    title.trim().trim_end_matches(':').trim().to_lowercase()
}

/// Splits Markdown into sections by ATX headings; text before the first heading has no title.
fn split_sections(body: &str) -> Vec<Section> {
    let heading = Regex::new(r"^ {0,3}#{1,6}\s+(.*?)\s*#*\s*$").expect("valid regex");
    let mut sections = vec![Section {
        title: String::new(),
        text: String::new(),
    }];
    let mut fence = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            fence = !fence;
        }
        match heading.captures(line).filter(|_| !fence) {
            Some(c) => sections.push(Section {
                title: c[1].to_string(),
                text: String::new(),
            }),
            None => {
                let last = sections.last_mut().expect("one section");
                last.text.push_str(line);
                last.text.push('\n');
            }
        }
    }
    for s in &mut sections {
        s.text = s.text.trim().to_string();
    }
    sections
}

/// `path:line — reason` lines; a path must be relative and stay inside the worktree.
fn look_first(text: &str) -> Vec<LookFirst> {
    let re = Regex::new(r"^\s*(?:[-*+]|\d+[.)])?\s*`?([^`\s:]+):(\d+)`?\s+(?:—|–|-{1,2})\s+(.+)$")
        .expect("valid regex");
    text.lines()
        .filter_map(|line| {
            let c = re.captures(line)?;
            let path = c[1].to_string();
            safe_relative(&path).then(|| LookFirst {
                line: c[2].parse().unwrap_or(1),
                reason: c[3].trim().to_string(),
                path,
            })
        })
        .collect()
}

pub fn safe_relative(path: &str) -> bool {
    let p = Path::new(path);
    !path.is_empty()
        && !path.starts_with('-')
        && p.is_relative()
        && p.components().all(|c| {
            matches!(
                c,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
        && !path.chars().any(char::is_control)
}

fn first_url(text: &str) -> Option<String> {
    let re = Regex::new(r#"https?://[^\s<>()\[\]`"']+"#).expect("valid regex");
    re.find(text).map(|m| {
        m.as_str()
            .trim_end_matches(['.', ',', ';', ':', '!', '?'])
            .to_string()
    })
}

fn is_test(path: &str) -> bool {
    let lower = path.to_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    lower.split('/').any(|part| {
        matches!(
            part,
            "test" | "tests" | "__tests__" | "spec" | "specs" | "e2e"
        )
    }) || name.starts_with("test_")
        || name.contains("_test.")
        || name.contains(".test.")
        || name.contains(".spec.")
        || name.contains("_spec.")
}

fn is_ci_config(path: &str) -> bool {
    path.starts_with(".github/workflows/")
        || path.starts_with(".circleci/")
        || path.starts_with(".buildkite/")
        || matches!(
            path,
            ".gitlab-ci.yml" | "Jenkinsfile" | "azure-pipelines.yml" | ".travis.yml"
        )
}
