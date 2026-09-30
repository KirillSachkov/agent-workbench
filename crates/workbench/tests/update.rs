mod common;

use std::fs;
use std::path::PathBuf;

use common::{Env, TDD_V1, TDD_V2, commit_all, git, lock, read};

/// A project on harness v1.0.0 with the install committed.
fn on_v1(env: &Env, name: &str) -> PathBuf {
    let project = env.project(name, &[]);
    let from = format!("{}@v1.0.0", env.source().display());
    env.ok(&project, &["init", "--from", &from]);
    commit_all(&project, "install harness v1");
    project
}

#[test]
fn update_replaces_untouched_files_on_a_new_branch_and_rewrites_the_lock() {
    let env = Env::new();
    let project = on_v1(&env, "untouched");

    let run = env.ok(&project, &["update", "--to", "v2.0.0"]);
    let out = run.stdout();

    let branch = git(&project, &["branch", "--show-current"]);
    assert_eq!(branch.trim(), "workbench/update-v2.0.0");
    assert!(out.contains("on branch workbench/update-v2.0.0"), "{out}");

    assert_eq!(read(project.join(".agents/skills/tdd/SKILL.md")), TDD_V2);
    assert!(read(project.join(".agents/skills/grilling/SKILL.md")).contains("wait for the answer"));
    assert!(
        out.contains("- replaced: .agents/skills/grilling/SKILL.md"),
        "{out}"
    );
    // A file dropped by the new version goes; a new skill arrives with its derived files.
    assert!(
        !project
            .join(".agents/skills/grilling/REFERENCE.md")
            .exists()
    );
    assert!(
        project
            .join(".agents/skills/handoff/agents/openai.yaml")
            .is_file()
    );

    let lock = lock(&project);
    let commit = git(&env.source(), &["rev-parse", "v2.0.0^{commit}"]);
    assert_eq!(lock["source"]["ref"], "v2.0.0");
    assert_eq!(lock["source"]["commit"], commit.trim());
    assert_eq!(lock["harness"]["version"], "2.0.0");
    assert_eq!(
        lock["skills"]["grilling"]["upstream"]["commit"],
        "2222222222222222222222222222222222222222"
    );
    assert!(
        lock["files"]
            .get(".agents/skills/grilling/REFERENCE.md")
            .is_none()
    );
    assert!(
        lock["files"]
            .get(".agents/skills/handoff/SKILL.md")
            .is_some()
    );
}

#[test]
fn update_without_to_takes_the_newest_release_tag() {
    let env = Env::new();
    let project = on_v1(&env, "newest");
    env.ok(&project, &["update"]);
    assert_eq!(lock(&project)["source"]["ref"], "v2.0.0");

    let run = env.ok(&project, &["update"]);
    assert!(run.stdout().contains("Already up to date"), "{}", run.all());
}

#[test]
fn update_merges_a_non_overlapping_edit_cleanly() {
    let env = Env::new();
    let project = on_v1(&env, "clean-merge");
    let edited = TDD_V1.replace(
        "1. Write one failing test.",
        "1. Write one failing test first.",
    );
    fs::write(project.join(".agents/skills/tdd/SKILL.md"), &edited).unwrap();
    commit_all(&project, "edit tdd");

    let run = env.ok(&project, &["update", "--to", "v2.0.0"]);
    assert!(
        run.stdout()
            .contains("- merged: .agents/skills/tdd/SKILL.md"),
        "{}",
        run.all()
    );
    let merged = read(project.join(".agents/skills/tdd/SKILL.md"));
    assert_eq!(
        merged,
        TDD_V2.replace(
            "1. Write one failing test.",
            "1. Write one failing test first."
        )
    );

    // The lock records the harness's content, so the merged file still counts as edited.
    let run = env.ok(&project, &["sync"]);
    assert!(!run.stdout().contains("tdd"), "{}", run.all());
}

#[test]
fn update_leaves_conflict_markers_for_an_overlapping_edit() {
    let env = Env::new();
    let project = on_v1(&env, "conflict");
    let edited = TDD_V1.replace(
        "Keep tests at the public interface.",
        "Keep tests at the public interface, always.",
    );
    fs::write(project.join(".agents/skills/tdd/SKILL.md"), &edited).unwrap();
    commit_all(&project, "edit tdd");

    let run = env.ok(&project, &["update", "--to", "v2.0.0"]);
    assert!(
        run.stdout()
            .contains("- CONFLICT: .agents/skills/tdd/SKILL.md"),
        "{}",
        run.all()
    );
    let text = read(project.join(".agents/skills/tdd/SKILL.md"));
    assert!(text.contains("<<<<<<< project"), "{text}");
    assert!(text.contains("Keep tests at the public interface, always."));
    assert!(text.contains("======="));
    assert!(text.contains("Keep tests at the public interface; never test internals."));
    assert!(text.contains(">>>>>>> harness 2.0.0"));
    assert!(
        text.starts_with("---\nname: tdd"),
        "non-conflicting lines stay: {text}"
    );
}

#[test]
fn update_applies_the_rename_and_removal_map() {
    let env = Env::new();
    let project = on_v1(&env, "renames");
    // An edit in the renamed skill travels with it and merges with the new version.
    let old = project.join(".agents/skills/old-name/SKILL.md");
    fs::write(
        &old,
        read(&old).replace("Line three.", "Line three, edited."),
    )
    .unwrap();
    commit_all(&project, "edit old-name");

    let run = env.ok(&project, &["update", "--to", "v2.0.0"]);
    let out = run.stdout();
    assert!(out.contains("renamed skill: old-name → new-name"), "{out}");
    assert!(out.contains("retired skill: retired"), "{out}");

    assert!(!project.join(".agents/skills/old-name").exists());
    assert!(!project.join(".agents/skills/retired").exists());
    let renamed = read(project.join(".agents/skills/new-name/SKILL.md"));
    assert!(renamed.contains("name: new-name"), "{renamed}");
    assert!(renamed.contains("Line three, edited."), "{renamed}");

    let lock = lock(&project);
    assert!(lock["skills"].get("old-name").is_none());
    assert!(lock["skills"].get("retired").is_none());
    assert!(lock["skills"].get("new-name").is_some());
    assert!(!read(project.join(".agents/skills/REGISTRY.md")).contains("| retired |"));
}

#[test]
fn update_refuses_an_existing_branch_without_writing() {
    let env = Env::new();
    let project = on_v1(&env, "branch-exists");
    git(&project, &["branch", "workbench/update-v2.0.0"]);
    let run = env.run(&project, &["update", "--to", "v2.0.0"]);
    assert!(!run.success());
    assert!(
        run.stderr().contains("workbench/update-v2.0.0"),
        "{}",
        run.all()
    );
    assert_eq!(lock(&project)["source"]["ref"], "v1.0.0");
    assert_eq!(read(project.join(".agents/skills/tdd/SKILL.md")), TDD_V1);
}

#[test]
fn update_retires_a_skill_the_release_still_ships() {
    let env = Env::new();
    let project = on_v1(&env, "still-shipped");
    let source = env.source();
    git(&source, &["checkout", "-q", "v2.0.0"]);
    fs::write(
        source.join("harness.toml"),
        read(source.join("harness.toml"))
            .replace("version = \"2.0.0\"", "version = \"2.1.0\"")
            .replace(
                "removed = [\"retired\"]",
                "removed = [\"retired\", \"grilling\"]",
            ),
    )
    .unwrap();
    commit_all(&source, "retire grilling but keep shipping it");
    git(&source, &["tag", "v2.1.0"]);

    let run = env.ok(&project, &["update", "--to", "v2.1.0"]);
    assert!(
        run.stdout().contains("retired skill: grilling"),
        "{}",
        run.all()
    );
    assert!(!project.join(".agents/skills/grilling").exists());
    assert!(lock(&project)["skills"].get("grilling").is_none());
}

#[test]
fn update_refuses_a_dirty_working_tree_without_writing() {
    let env = Env::new();
    let project = on_v1(&env, "dirty");
    fs::write(project.join("notes.txt"), "work in progress\n").unwrap();
    let run = env.run(&project, &["update", "--to", "v2.0.0"]);
    assert!(!run.success());
    assert!(
        run.stderr().contains("uncommitted changes"),
        "{}",
        run.all()
    );
    assert_eq!(git(&project, &["branch", "--show-current"]).trim(), "main");
    assert_eq!(lock(&project)["source"]["ref"], "v1.0.0");
}

#[test]
fn a_relative_source_path_is_recorded_as_given_and_used_by_update() {
    let env = Env::new();
    env.source();
    let project = env.project("relative", &[]);
    env.ok(&project, &["init", "--from", "../harness-source@v1.0.0"]);
    assert_eq!(lock(&project)["source"]["repository"], "../harness-source");
    commit_all(&project, "install");
    env.ok(&project, &["update"]);
    assert_eq!(lock(&project)["source"]["ref"], "v2.0.0");
}
