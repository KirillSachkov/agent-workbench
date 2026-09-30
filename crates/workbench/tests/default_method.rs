//! The default method this repository ships: installed from a copy of its own `harness.toml` and
//! `skills/`, the way `workbench init --from` installs it into any project.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{Env, commit_all, git, lock, read};

/// The repository root: this crate lives in `crates/workbench`.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A harness source holding this repository's working-tree `harness.toml`, `skills/` and `beta/`.
fn this_harness(env: &Env) -> PathBuf {
    let dir = env.home().join("this-harness");
    fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    fs::copy(repository().join("harness.toml"), dir.join("harness.toml")).unwrap();
    for sub in ["skills", "beta"] {
        let from = repository().join(sub);
        if from.is_dir() {
            copy_dir(&from, &dir.join(sub));
        }
    }
    commit_all(&dir, "this harness");
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target).unwrap();
        }
    }
}

fn skill_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join("SKILL.md").is_file())
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn user_invoked(skill_md: &str) -> bool {
    skill_md
        .split("\n---")
        .next()
        .unwrap()
        .lines()
        .any(|l| l.trim() == "disable-model-invocation: true")
}

/// Our own skills; every other skill is a fork and must name its upstream commit.
const OWN: &[&str] = &["coordinate", "editing-agents-md"];

#[test]
fn a_project_installed_from_this_repository_locks_every_forked_skill_to_its_upstream_commit() {
    let env = Env::new();
    let source = this_harness(&env);
    let project = env.project("consumer", &[]);
    env.ok(
        &project,
        &["init", "--from", &format!("{}@main", source.display())],
    );

    let lock = lock(&project);
    let shipped = skill_names(&repository().join("skills"));
    for name in &shipped {
        let entry = &lock["skills"][name];
        assert_eq!(entry["channel"], "stable", "{name} is not locked");
        if OWN.contains(&name.as_str()) {
            assert!(entry.get("upstream").is_none(), "{name} is our own skill");
            continue;
        }
        let upstream = &entry["upstream"];
        assert_eq!(
            upstream["repository"], "https://github.com/mattpocock/skills",
            "{name} has no upstream"
        );
        let commit = upstream["commit"].as_str().unwrap();
        assert!(
            commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit()),
            "{name}: `{commit}` is not a full commit id"
        );
    }
    assert_eq!(
        lock["skills"].as_object().unwrap().len(),
        shipped.len(),
        "the lock holds exactly the shipped skills"
    );
}

#[test]
fn every_installed_skill_carries_its_invocation_policy_to_codex_and_opencode() {
    let env = Env::new();
    let source = this_harness(&env);
    let project = env.project("consumer", &[]);
    env.ok(
        &project,
        &["init", "--from", &format!("{}@main", source.display())],
    );

    let opencode = read(project.join("opencode.json"));
    let skills = project.join(".agents/skills");
    for name in skill_names(&skills) {
        let user = user_invoked(&read(skills.join(&name).join("SKILL.md")));
        let codex = skills.join(&name).join("agents/openai.yaml");
        let codex_denies =
            codex.is_file() && read(&codex).contains("allow_implicit_invocation: false");
        assert_eq!(
            codex_denies, user,
            "{name}: Codex policy disagrees with the frontmatter"
        );
        assert_eq!(
            opencode.contains(&format!("\"{name}\": \"deny\"")),
            user,
            "{name}: OpenCode rule disagrees with the frontmatter"
        );
    }
}

#[test]
fn the_rename_map_points_at_shipped_skills() {
    let manifest: toml::Value = toml::from_str(&read(repository().join("harness.toml"))).unwrap();
    let shipped = skill_names(&repository().join("skills"));
    let changes = &manifest["changes"];
    for (old, new) in changes["renamed"].as_table().unwrap() {
        assert!(!shipped.contains(old), "{old} is renamed but still shipped");
        assert!(
            shipped.contains(&new.as_str().unwrap().to_owned()),
            "{old} is renamed to {new}, which is not shipped"
        );
    }
    for removed in changes["removed"].as_array().unwrap() {
        let removed = removed.as_str().unwrap().to_owned();
        assert!(
            !shipped.contains(&removed),
            "{removed} is removed but still shipped"
        );
    }
}

/// Skill text is read by every runtime, so it names skills plainly — never with one runtime's
/// slash syntax.
#[test]
fn skill_text_uses_no_runtime_specific_invocation_syntax() {
    let skills = repository().join("skills");
    let names = skill_names(&skills);
    let mut commands: Vec<String> = names.iter().map(|n| format!("/{n}")).collect();
    commands.extend(["/clear".into(), "/compact".into()]);
    let mut found = Vec::new();
    for file in markdown_files(&skills) {
        let text = read(&file);
        for command in &commands {
            for (i, _) in text.match_indices(command.as_str()) {
                let before = text[..i].chars().last();
                let after = text[i + command.len()..].chars().next();
                let starts_word = before.is_none_or(|c| c.is_whitespace() || "`(*".contains(c));
                let ends_word =
                    after.is_none_or(|c| !(c.is_alphanumeric() || c == '-' || c == '/'));
                if starts_word && ends_word {
                    found.push(format!("{}: {command}", file.display()));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "slash invocations in skill text:\n{}",
        found.join("\n")
    );
}

fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(markdown_files(&path));
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    out
}
