mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{Env, files_under, read};

fn installed(env: &Env, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project = env.project(name, files);
    let from = format!("{}@v1.0.0", env.source().display());
    env.ok(&project, &["init", "--from", &from]);
    project
}

fn snapshot(project: &Path) -> Vec<(String, String)> {
    files_under(project)
        .into_iter()
        .filter(|path| !path.starts_with(".git/"))
        .map(|path| {
            let content = fs::read(project.join(&path)).unwrap();
            (path, String::from_utf8_lossy(&content).into_owned())
        })
        .collect()
}

#[test]
fn sync_is_idempotent() {
    let env = Env::new();
    let project = installed(&env, "idempotent", &[]);
    let before = snapshot(&project);

    let run = env.ok(&project, &["sync"]);
    assert!(run.stdout().contains("up to date"), "{}", run.all());
    assert!(!run.stdout().contains("stale"), "{}", run.all());
    env.ok(&project, &["sync"]);
    assert_eq!(snapshot(&project), before);
}

#[test]
fn sync_reports_and_regenerates_hand_deleted_derived_files() {
    let env = Env::new();
    let project = installed(&env, "deleted", &[]);
    let codex = ".agents/skills/implement/agents/openai.yaml";
    let codex_content = read(project.join(codex));
    fs::remove_file(project.join(codex)).unwrap();
    fs::remove_file(project.join(".agents/skills/REGISTRY.md")).unwrap();
    fs::remove_file(project.join(".claude/skills")).unwrap();

    let run = env.ok(&project, &["sync"]);
    let out = run.stdout();
    assert!(out.contains(&format!("stale: {codex} (missing)")), "{out}");
    assert!(
        out.contains("stale: .agents/skills/REGISTRY.md (missing)"),
        "{out}"
    );
    assert!(out.contains("stale: .claude/skills (missing)"), "{out}");
    assert_eq!(read(project.join(codex)), codex_content);
    assert!(project.join(".claude/skills/tdd/SKILL.md").is_file());
}

#[test]
fn sync_follows_frontmatter_changes() {
    let env = Env::new();
    let project = installed(&env, "frontmatter", &[]);
    let skill = project.join(".agents/skills/implement/SKILL.md");
    let text = read(&skill).replace("disable-model-invocation: true\n", "");
    fs::write(&skill, text).unwrap();
    let tdd = project.join(".agents/skills/tdd/SKILL.md");
    let text = read(&tdd).replace(
        "---\n\n# TDD",
        "disable-model-invocation: true\n---\n\n# TDD",
    );
    fs::write(&tdd, text).unwrap();

    let run = env.ok(&project, &["sync"]);
    assert!(
        run.stdout()
            .contains("stale: .agents/skills/implement/agents/openai.yaml (no longer needed)"),
        "{}",
        run.all()
    );
    assert!(!project.join(".agents/skills/implement/agents").exists());
    assert!(
        project
            .join(".agents/skills/tdd/agents/openai.yaml")
            .is_file()
    );
    let opencode: serde_json::Value =
        serde_json::from_str(&read(project.join("opencode.json"))).unwrap();
    assert!(opencode["permission"]["skill"].get("implement").is_none());
    assert_eq!(opencode["permission"]["skill"]["tdd"], "deny");
    let agents = read(project.join("AGENTS.md"));
    assert!(
        agents.contains("`tdd`") && !agents.contains("`implement`"),
        "{agents}"
    );
}

#[test]
fn sync_keeps_the_projects_own_opencode_settings() {
    let env = Env::new();
    let own = r#"{
  "model": "anthropic/claude-sonnet",
  "permission": {
    "bash": "ask",
    "skill": {
      "*": "allow",
      "private-*": "deny"
    }
  },
  "instructions": ["docs/style.md"]
}
"#;
    let project = installed(&env, "opencode", &[("opencode.json", own)]);
    let merged: serde_json::Value =
        serde_json::from_str(&read(project.join("opencode.json"))).unwrap();
    assert_eq!(merged["model"], "anthropic/claude-sonnet");
    assert_eq!(merged["instructions"][0], "docs/style.md");
    assert_eq!(merged["permission"]["bash"], "ask");
    assert_eq!(merged["permission"]["skill"]["*"], "allow");
    assert_eq!(merged["permission"]["skill"]["private-*"], "deny");
    assert_eq!(merged["permission"]["skill"]["implement"], "deny");

    // A project edit survives the next sync, and an unchanged file is not rewritten.
    let edited = read(project.join("opencode.json")).replace("\"ask\"", "\"allow\"");
    fs::write(project.join("opencode.json"), &edited).unwrap();
    let run = env.ok(&project, &["sync"]);
    assert!(!run.stdout().contains("opencode.json"), "{}", run.all());
    assert_eq!(read(project.join("opencode.json")), edited);
}

#[test]
fn sync_supports_per_skill_links() {
    let env = Env::new();
    let project = installed(&env, "per-skill", &[]);
    fs::write(
        project.join("workbench.toml"),
        "[links]\nclaude = \"per-skill\"\n",
    )
    .unwrap();
    let run = env.ok(&project, &["sync"]);
    assert!(
        run.stdout().contains("stale: .claude/skills/tdd (missing)"),
        "{}",
        run.all()
    );
    let link = project.join(".claude/skills");
    assert!(fs::symlink_metadata(&link).unwrap().is_dir());
    assert_eq!(
        fs::read_link(link.join("tdd")).unwrap().to_str().unwrap(),
        "../../.agents/skills/tdd"
    );
    assert!(link.join("tdd/SKILL.md").is_file());

    // A Claude-only skill in the directory is left alone; a link to a removed skill goes.
    fs::create_dir_all(link.join("claude-only")).unwrap();
    fs::write(link.join("claude-only/SKILL.md"), "mine\n").unwrap();
    fs::remove_dir_all(project.join(".agents/skills/retired")).unwrap();
    let run = env.ok(&project, &["sync"]);
    assert!(
        run.stdout()
            .contains("stale: .claude/skills/retired (no longer needed)"),
        "{}",
        run.all()
    );
    assert!(fs::symlink_metadata(link.join("retired")).is_err());
    assert_eq!(read(link.join("claude-only/SKILL.md")), "mine\n");
}

#[cfg(unix)]
#[test]
fn sync_replaces_a_text_stub_with_a_link_and_warns() {
    let env = Env::new();
    let project = installed(&env, "stub", &[]);
    let link = project.join(".claude/skills");
    fs::remove_file(&link).unwrap();
    fs::write(&link, "../.agents/skills").unwrap();

    let run = env.ok(&project, &["sync"]);
    assert!(
        run.stderr().contains("text file instead of a symlink"),
        "{}",
        run.all()
    );
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
