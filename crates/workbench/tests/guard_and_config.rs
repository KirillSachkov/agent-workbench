mod common;

use std::fs;

use common::{Env, commit_all, files_under, git, read};

const NEWER: &str = "99.0.0";

#[test]
fn an_older_binary_refuses_to_write_and_names_the_required_version() {
    let env = Env::new();
    let project = env.project("guarded", &[]);
    env.ok(
        &project,
        &[
            "init",
            "--from",
            &format!("{}@v1.0.0", env.source().display()),
        ],
    );
    let lock_path = project.join("workbench-lock.json");
    let lock = read(&lock_path).replace(
        "\"min_workbench_version\": \"0.1.0\"",
        &format!("\"min_workbench_version\": \"{NEWER}\""),
    );
    fs::write(&lock_path, &lock).unwrap();
    commit_all(&project, "install");
    fs::remove_file(project.join(".agents/skills/REGISTRY.md")).unwrap();

    for args in [vec!["sync"], vec!["update", "--to", "v2.0.0"]] {
        let run = env.run(&project, &args);
        assert!(!run.success(), "{args:?} should refuse");
        assert!(
            run.stderr()
                .contains(&format!("requires workbench {NEWER}")),
            "{args:?}: {}",
            run.all()
        );
    }
    assert!(!project.join(".agents/skills/REGISTRY.md").exists());
    assert_eq!(read(&lock_path), lock);
    assert_eq!(git(&project, &["branch", "--show-current"]).trim(), "main");

    // Reading the configuration is never blocked.
    env.ok(&project, &["config"]);
}

#[test]
fn init_refuses_a_harness_that_needs_a_newer_binary() {
    let env = Env::new();
    let source = env.source();
    let manifest = read(source.join("harness.toml")).replace(
        "min_workbench = \"0.1.0\"",
        &format!("min_workbench = \"{NEWER}\""),
    );
    fs::write(source.join("harness.toml"), manifest).unwrap();
    commit_all(&source, "needs a newer workbench");
    git(&source, &["tag", "v3.0.0"]);

    let project = env.project("too-new", &[]);
    let run = env.run(
        &project,
        &["init", "--from", &format!("{}@v3.0.0", source.display())],
    );
    assert!(!run.success());
    assert!(
        run.stderr()
            .contains(&format!("requires workbench {NEWER}")),
        "{}",
        run.all()
    );
    let written: Vec<String> = files_under(&project)
        .into_iter()
        .filter(|p| !p.starts_with(".git/"))
        .collect();
    assert_eq!(written, vec!["README.md".to_owned()]);
}

#[test]
fn config_prints_the_built_in_default_overlaid_by_the_project() {
    let env = Env::new();
    let project = env.project(
        "config",
        &[(
            "workbench.toml",
            "[repair]\nmax_rounds = 3\n\n[tracker.labels]\nready-for-agent = \"agent:go\"\n",
        )],
    );
    let run = env.ok(&project, &["config"]);
    let resolved: toml::Table = run.stdout().parse().unwrap();

    assert_eq!(resolved["repair"]["max_rounds"].as_integer(), Some(3));
    assert_eq!(resolved["acceptance"]["default"].as_str(), Some("human"));
    let labels = &resolved["tracker"]["labels"];
    assert_eq!(labels["ready-for-agent"].as_str(), Some("agent:go"));
    assert_eq!(labels["acceptance-auto"].as_str(), Some("acceptance:auto"));

    let tracks = &resolved["tracks"];
    let ids = |track: &str| -> Vec<String> {
        tracks[track]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["id"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(ids("small"), ["intent", "delivery", "accepted"]);
    assert_eq!(ids("medium"), ["spec", "tickets", "delivery", "accepted"]);
    assert_eq!(ids("large"), ["map"]);
    assert_eq!(tracks["large"]["then"].as_str(), Some("medium"));
    assert_eq!(
        tracks["medium"]["nodes"][1]["after"].as_array().unwrap()[0].as_str(),
        Some("spec")
    );
}

#[test]
fn config_reports_an_invalid_project_file() {
    let env = Env::new();
    let project = env.project("bad-config", &[("workbench.toml", "[repair\n")]);
    let run = env.run(&project, &["config"]);
    assert!(!run.success());
    assert!(run.stderr().contains("workbench.toml"), "{}", run.all());
}
