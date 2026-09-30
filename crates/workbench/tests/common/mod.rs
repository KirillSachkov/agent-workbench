//! Fixtures for black-box tests: a harness source repository with two tagged versions, target
//! projects as temporary git repositories, and a runner for the built `workbench` binary with an
//! isolated home directory.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

pub const TDD_V1: &str = "---
name: tdd
description: Test-driven development in red-green-refactor steps.
---

# TDD

1. Write one failing test.
2. Make it pass.
3. Refactor.

Seams are agreed with the owner.
Keep tests at the public interface.
";

/// v2 changes only the last line, so an edit near the top merges cleanly and an edit of the last
/// line conflicts.
pub const TDD_V2: &str = "---
name: tdd
description: Test-driven development in red-green-refactor steps.
---

# TDD

1. Write one failing test.
2. Make it pass.
3. Refactor.

Seams are agreed with the owner.
Keep tests at the public interface; never test internals.
";

pub struct Env {
    root: TempDir,
}

impl Env {
    pub fn new() -> Self {
        let env = Env {
            root: TempDir::new().unwrap(),
        };
        fs::create_dir_all(env.home()).unwrap();
        env
    }

    pub fn home(&self) -> PathBuf {
        self.root.path().join("home")
    }

    /// A harness source with tags `v1.0.0` and `v2.0.0`. The second version changes `grilling`
    /// and `tdd`, drops `grilling/REFERENCE.md`, renames `old-name` to `new-name`, retires `retired`
    /// and adds `handoff`.
    pub fn source(&self) -> PathBuf {
        let dir = self.root.path().join("harness-source");
        if dir.exists() {
            return dir;
        }
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        write_tree(&dir, &source_v1());
        commit_all(&dir, "v1");
        git(&dir, &["tag", "v1.0.0"]);

        fs::remove_dir_all(dir.join("skills/old-name")).unwrap();
        fs::remove_dir_all(dir.join("skills/retired")).unwrap();
        fs::remove_file(dir.join("skills/grilling/REFERENCE.md")).unwrap();
        write_tree(&dir, &source_v2());
        commit_all(&dir, "v2");
        git(&dir, &["tag", "v2.0.0"]);
        dir
    }

    /// A git repository with one commit, holding the given files.
    pub fn project(&self, name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = self.root.path().join(name);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        write_tree(&dir, files);
        fs::write(dir.join("README.md"), "# project\n").unwrap();
        commit_all(&dir, "initial");
        dir
    }

    pub fn run(&self, dir: &Path, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_workbench"))
            .args(args)
            .current_dir(dir)
            .envs(git_env(&self.home()))
            .output()
            .unwrap();
        Run(output)
    }

    /// Runs the binary and fails the test unless it succeeds.
    pub fn ok(&self, dir: &Path, args: &[&str]) -> Run {
        let run = self.run(dir, args);
        assert!(
            run.0.status.success(),
            "workbench {args:?} failed\nstdout:\n{}\nstderr:\n{}",
            run.stdout(),
            run.stderr()
        );
        run
    }
}

pub struct Run(pub Output);

impl Run {
    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.0.stdout).into_owned()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.0.stderr).into_owned()
    }

    pub fn all(&self) -> String {
        format!("{}{}", self.stdout(), self.stderr())
    }

    pub fn success(&self) -> bool {
        self.0.status.success()
    }
}

fn source_v1() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "harness.toml",
            r#"[harness]
name = "fixture-harness"
version = "1.0.0"
min_workbench = "0.1.0"

[upstream.grilling]
repository = "https://github.com/mattpocock/skills"
commit = "1111111111111111111111111111111111111111"
"#,
        ),
        (
            "skills/grilling/SKILL.md",
            "---\nname: grilling\ndescription: Grill the user about a plan.\n---\n\nAsk one question at a time.\n",
        ),
        ("skills/grilling/REFERENCE.md", "Reference for grilling.\n"),
        (
            "skills/implement/SKILL.md",
            "---\nname: implement\ndescription: \"Implement a ticket | end to end.\"\ndisable-model-invocation: true\n---\n\nImplement the ticket.\n",
        ),
        ("skills/tdd/SKILL.md", TDD_V1),
        ("skills/tdd/scripts/run.sh", "#!/bin/sh\necho run\n"),
        (
            "skills/old-name/SKILL.md",
            "---\nname: old-name\ndescription: A skill that gets renamed.\n---\n\nLine one.\nLine two.\nLine three.\n",
        ),
        (
            "skills/retired/SKILL.md",
            "---\nname: retired\ndescription: A skill that gets retired.\n---\n\nRetired body.\n",
        ),
        (
            "beta/experiment/SKILL.md",
            "---\nname: experiment\ndescription: >\n  A beta skill\n  on two lines.\ndisable-model-invocation: true\n---\n\nExperimental.\n",
        ),
        (
            "CHANGELOG.md",
            "# Changelog\n\n## 1.0.0\n\nFirst release.\n",
        ),
    ]
}

fn source_v2() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "harness.toml",
            r#"[harness]
name = "fixture-harness"
version = "2.0.0"
min_workbench = "0.1.0"

[upstream.grilling]
repository = "https://github.com/mattpocock/skills"
commit = "2222222222222222222222222222222222222222"

[changes]
renamed = { old-name = "new-name" }
removed = ["retired"]
"#,
        ),
        (
            "skills/grilling/SKILL.md",
            "---\nname: grilling\ndescription: Grill the user about a plan.\n---\n\nAsk one question at a time and wait for the answer.\n",
        ),
        ("skills/tdd/SKILL.md", TDD_V2),
        (
            "skills/new-name/SKILL.md",
            "---\nname: new-name\ndescription: A skill that got renamed.\n---\n\nLine one.\nLine two.\nLine three.\n",
        ),
        (
            "skills/handoff/SKILL.md",
            "---\nname: handoff\ndescription: Write a handoff comment.\ndisable-model-invocation: true\n---\n\nHand off.\n",
        ),
        (
            "CHANGELOG.md",
            "# Changelog\n\n## 2.0.0\n\nRenamed old-name, retired retired.\n\n## 1.0.0\n\nFirst release.\n",
        ),
    ]
}

pub fn write_tree(dir: &Path, files: &[(&str, &str)]) {
    for (path, content) in files {
        let path = dir.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        #[cfg(unix)]
        if path.extension().is_some_and(|e| e == "sh") {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}

pub fn commit_all(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

pub fn git(dir: &Path, args: &[&str]) -> String {
    let home = dir.join(".no-home");
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .envs(git_env(&home))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn git_env(home: &Path) -> Vec<(&'static str, String)> {
    vec![
        ("HOME", home.display().to_string()),
        (
            "XDG_CONFIG_HOME",
            home.join(".config").display().to_string(),
        ),
        ("GIT_CONFIG_NOSYSTEM", "1".into()),
        ("GIT_CONFIG_GLOBAL", "/dev/null".into()),
        ("GIT_AUTHOR_NAME", "Test".into()),
        ("GIT_AUTHOR_EMAIL", "test@example.com".into()),
        ("GIT_COMMITTER_NAME", "Test".into()),
        ("GIT_COMMITTER_EMAIL", "test@example.com".into()),
    ]
}

pub fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

pub fn lock(project: &Path) -> serde_json::Value {
    serde_json::from_str(&read(project.join("workbench-lock.json"))).unwrap()
}

/// Every file under `dir`, relative, sorted; used to prove nothing was written there.
pub fn files_under(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(root, &path, out);
        } else {
            out.push(path.strip_prefix(root).unwrap().display().to_string());
        }
    }
}
