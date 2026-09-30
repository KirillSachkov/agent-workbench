mod common;

use std::fs;

use common::{Env, TDD_V1, files_under, git, lock, read};

const BEGIN: &str = "<!-- BEGIN workbench managed block";
const END: &str = "<!-- END workbench managed block -->";

fn from_v1(env: &Env) -> String {
    format!("{}@v1.0.0", env.source().display())
}

#[test]
fn init_writes_skills_links_runtime_files_block_bridge_config_and_lock() {
    let env = Env::new();
    let project = env.project("fresh", &[]);

    let run = env.ok(&project, &["init", "--from", &from_v1(&env)]);
    assert!(
        run.stdout().contains("fixture-harness 1.0.0"),
        "{}",
        run.all()
    );

    // Stable skills are copied into the canonical directory; beta skills are not.
    assert_eq!(read(project.join(".agents/skills/tdd/SKILL.md")), TDD_V1);
    assert!(
        project
            .join(".agents/skills/grilling/REFERENCE.md")
            .is_file()
    );
    assert!(!project.join(".agents/skills/experiment").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(project.join(".agents/skills/tdd/scripts/run.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert!(mode & 0o111 != 0, "script lost its executable bit");
    }

    // Claude Code reads the same directory through a committed relative symlink.
    let link = project.join(".claude/skills");
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_link(&link).unwrap().to_str().unwrap(),
        "../.agents/skills"
    );
    assert!(link.join("tdd/SKILL.md").is_file());

    // Codex: only the user-invoked skill gets a policy file.
    let codex = read(project.join(".agents/skills/implement/agents/openai.yaml"));
    assert!(
        codex.contains("allow_implicit_invocation: false"),
        "{codex}"
    );
    assert!(
        !project
            .join(".agents/skills/tdd/agents/openai.yaml")
            .exists()
    );

    // OpenCode: a deny rule on the skill tool for the user-invoked skill.
    let opencode: serde_json::Value =
        serde_json::from_str(&read(project.join("opencode.json"))).unwrap();
    assert_eq!(opencode["permission"]["skill"]["implement"], "deny");
    assert!(opencode["permission"]["skill"].get("tdd").is_none());

    // Registry index generated from frontmatter.
    let registry = read(project.join(".agents/skills/REGISTRY.md"));
    assert!(registry.contains("| implement | user | Implement a ticket \\| end to end. |"));
    assert!(registry.contains("| tdd | model |"));

    // AGENTS.md skeleton with the managed block, and the Claude Code bridge.
    let agents = read(project.join("AGENTS.md"));
    assert!(agents.contains(BEGIN) && agents.contains(END), "{agents}");
    assert!(agents.contains("`implement`"), "{agents}");
    assert_eq!(read(project.join("CLAUDE.md")), "@AGENTS.md\n");

    // Project configuration written when absent.
    assert!(project.join("workbench.toml").is_file());

    // Lock: source, ref, commit, versions, upstream and a hash of every harness file.
    let lock = lock(&project);
    let commit = git(&env.source(), &["rev-parse", "v1.0.0^{commit}"]);
    assert_eq!(lock["source"]["ref"], "v1.0.0");
    assert_eq!(lock["source"]["commit"], commit.trim());
    assert_eq!(lock["harness"]["name"], "fixture-harness");
    assert_eq!(lock["harness"]["version"], "1.0.0");
    assert_eq!(lock["min_workbench_version"], "0.1.0");
    assert_eq!(
        lock["skills"]["grilling"]["upstream"]["commit"],
        "1111111111111111111111111111111111111111"
    );
    assert!(lock["skills"]["tdd"].get("upstream").is_none());
    let files = lock["files"].as_object().unwrap();
    assert!(files.contains_key(".agents/skills/tdd/SKILL.md"));
    assert!(files.contains_key(".agents/skills/tdd/scripts/run.sh"));
    assert_eq!(
        files[".agents/skills/tdd/SKILL.md"].as_str().unwrap().len(),
        64
    );
    assert!(!files.contains_key(".agents/skills/REGISTRY.md"));
}

#[test]
fn init_keeps_existing_agents_md_and_claude_md_apart_from_the_block() {
    let env = Env::new();
    let own_agents = "# My project\n\nHand-written rules.\n";
    let own_claude = "# Claude notes\n\nSomething only for Claude.\n";
    let project = env.project(
        "existing",
        &[("AGENTS.md", own_agents), ("CLAUDE.md", own_claude)],
    );

    env.ok(&project, &["init", "--from", &from_v1(&env)]);

    let agents = read(project.join("AGENTS.md"));
    assert!(agents.starts_with(own_agents), "{agents}");
    assert!(agents.contains(BEGIN));
    assert_eq!(read(project.join("CLAUDE.md")), own_claude);

    // Refreshing replaces only the block: text after it stays too.
    let edited = agents.replace(END, &format!("{END}\n\nMore of my own text.")) + "Tail.\n";
    let edited = edited.replace("`implement`", "`stale-block-text`");
    fs::write(project.join("AGENTS.md"), &edited).unwrap();
    env.ok(&project, &["sync"]);
    let agents = read(project.join("AGENTS.md"));
    assert!(agents.starts_with(own_agents));
    assert!(agents.contains("`implement`") && !agents.contains("stale-block-text"));
    assert!(
        agents.ends_with("More of my own text.\nTail.\n"),
        "{agents}"
    );
    assert_eq!(agents.matches(BEGIN).count(), 1);
}

#[test]
fn rerunning_init_is_safe_and_keeps_project_edits() {
    let env = Env::new();
    let project = env.project("rerun", &[]);
    env.ok(&project, &["init", "--from", &from_v1(&env)]);
    let lock_before = read(project.join("workbench-lock.json"));

    let edited = format!("{TDD_V1}Project note.\n");
    fs::write(project.join(".agents/skills/tdd/SKILL.md"), &edited).unwrap();
    fs::write(project.join("workbench.toml"), "[repair]\nmax_rounds = 3\n").unwrap();
    fs::remove_file(project.join(".agents/skills/grilling/REFERENCE.md")).unwrap();

    let run = env.ok(&project, &["init", "--from", &from_v1(&env)]);
    assert!(
        run.all().contains(".agents/skills/tdd/SKILL.md"),
        "{}",
        run.all()
    );
    assert_eq!(read(project.join(".agents/skills/tdd/SKILL.md")), edited);
    assert!(
        project
            .join(".agents/skills/grilling/REFERENCE.md")
            .is_file()
    );
    assert_eq!(
        read(project.join("workbench.toml")),
        "[repair]\nmax_rounds = 3\n"
    );
    assert_eq!(read(project.join("workbench-lock.json")), lock_before);
    assert_eq!(read(project.join("AGENTS.md")).matches(BEGIN).count(), 1);
}

#[test]
fn init_installs_opted_in_beta_skills() {
    let env = Env::new();
    let project = env.project(
        "beta",
        &[("workbench.toml", "[skills]\nbeta = [\"experiment\"]\n")],
    );
    env.ok(&project, &["init", "--from", &from_v1(&env)]);
    assert!(project.join(".agents/skills/experiment/SKILL.md").is_file());
    let registry = read(project.join(".agents/skills/REGISTRY.md"));
    assert!(
        registry.contains("| experiment | user | A beta skill on two lines. |"),
        "{registry}"
    );
    assert_eq!(lock(&project)["skills"]["experiment"]["channel"], "beta");
}

#[test]
fn init_refuses_a_different_source_ref_and_points_to_update() {
    let env = Env::new();
    let project = env.project("other-ref", &[]);
    env.ok(&project, &["init", "--from", &from_v1(&env)]);
    let run = env.run(
        &project,
        &[
            "init",
            "--from",
            &format!("{}@v2.0.0", env.source().display()),
        ],
    );
    assert!(!run.success());
    assert!(run.stderr().contains("workbench update"), "{}", run.all());
}

#[test]
fn nothing_is_written_into_user_level_agent_configuration() {
    let env = Env::new();
    let project = env.project("user-level", &[]);
    env.ok(&project, &["init", "--from", &from_v1(&env)]);
    env.ok(&project, &["sync"]);
    env.ok(&project, &["update", "--to", "v2.0.0"]);
    env.ok(&project, &["config"]);
    assert_eq!(files_under(&env.home()), Vec::<String>::new());
}
