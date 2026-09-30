//! A harness source: a git repository (agent-workbench or a fork) holding `harness.toml`,
//! stable skills in `skills/` and beta skills in `beta/`. Read through a temporary clone with the
//! `git` command, so any URL or path git understands works.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tempfile::TempDir;

use crate::config::Config;
use crate::files::Blob;
use crate::lock;
use crate::skills;

pub const MANIFEST: &str = "harness.toml";
const STABLE_DIR: &str = "skills";
const BETA_DIR: &str = "beta";

/// `--from <repository>@<ref>`, with the repository normalised: a local path becomes absolute and
/// `owner/repo` becomes a GitHub URL.
pub struct Spec {
    pub repository: String,
    pub reference: Option<String>,
}

impl Spec {
    pub fn parse(from: &str) -> Result<Spec> {
        let (repository, reference) = match from.rsplit_once('@') {
            // `git@host:owner/repo` has an `@` that is not a ref separator.
            Some((repo, reference)) if !repo.is_empty() && !reference.contains(':') => {
                (repo, Some(reference.to_owned()))
            }
            _ => (from, None),
        };
        if repository.is_empty() || reference.as_deref() == Some("") {
            bail!("--from must look like <repository>@<ref>, got `{from}`");
        }
        Ok(Spec {
            repository: normalise(repository)?,
            reference,
        })
    }
}

fn normalise(repository: &str) -> Result<String> {
    let path = Path::new(repository);
    if path.exists() {
        let absolute = path
            .canonicalize()
            .with_context(|| format!("resolve {repository}"))?;
        return Ok(absolute.display().to_string());
    }
    let is_github_shorthand = repository.split('/').count() == 2
        && !repository.contains(':')
        && !repository.starts_with('.')
        && repository
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c));
    if is_github_shorthand {
        return Ok(format!("https://github.com/{repository}"));
    }
    Ok(repository.to_owned())
}

/// A temporary clone of a harness source.
pub struct Checkout {
    dir: TempDir,
}

impl Checkout {
    pub fn clone(repository: &str) -> Result<Checkout> {
        let dir = TempDir::new()?;
        let target = dir.path().join("source");
        git(
            dir.path(),
            &[
                "clone",
                "--quiet",
                "--no-checkout",
                repository,
                &target.display().to_string(),
            ],
        )
        .with_context(|| format!("fetch the harness source {repository}"))?;
        Ok(Checkout { dir })
    }

    fn path(&self) -> std::path::PathBuf {
        self.dir.path().join("source")
    }

    fn git(&self, args: &[&str]) -> Result<Vec<u8>> {
        git(&self.path(), args)
    }

    pub fn default_branch(&self) -> Result<String> {
        let head = self.git(&["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])?;
        let head = String::from_utf8(head)?;
        Ok(head.trim().trim_start_matches("origin/").to_owned())
    }

    /// Resolves a tag, branch or commit to a commit id.
    pub fn resolve(&self, reference: &str) -> Result<String> {
        let candidates = [
            format!("refs/tags/{reference}^{{commit}}"),
            format!("refs/remotes/origin/{reference}^{{commit}}"),
            format!("{reference}^{{commit}}"),
        ];
        for candidate in &candidates {
            if let Ok(out) = self.git(&["rev-parse", "--verify", "--quiet", candidate]) {
                return Ok(String::from_utf8(out)?.trim().to_owned());
            }
        }
        bail!("the harness source has no ref `{reference}`")
    }

    pub fn tags(&self) -> Result<Vec<String>> {
        let out = String::from_utf8(self.git(&["tag", "--list"])?)?;
        Ok(out.lines().map(str::to_owned).collect())
    }

    /// Reads the harness at a commit.
    pub fn release(&self, commit: &str) -> Result<Release> {
        let manifest_bytes = self
            .git(&["show", &format!("{commit}:{MANIFEST}")])
            .with_context(|| format!("not a harness source: no {MANIFEST} at {commit}"))?;
        let manifest: Manifest = toml::from_str(&String::from_utf8(manifest_bytes)?)
            .with_context(|| format!("parse {MANIFEST} at {commit}"))?;

        let listing = self.git(&["ls-tree", "-r", "-z", commit, "--", STABLE_DIR, BETA_DIR])?;
        let mut files = BTreeMap::new();
        for entry in listing.split(|b| *b == 0).filter(|e| !e.is_empty()) {
            let entry = String::from_utf8(entry.to_vec())?;
            let (meta, path) = entry
                .split_once('\t')
                .context("unexpected git ls-tree output")?;
            let mut meta = meta.split(' ');
            let (mode, kind, object) = (meta.next(), meta.next(), meta.next());
            if kind != Some("blob") {
                continue;
            }
            let object = object.context("unexpected git ls-tree output")?;
            let bytes = self.git(&["cat-file", "blob", object])?;
            let blob = Blob {
                bytes,
                executable: mode == Some("100755"),
            };
            files.insert(path.to_owned(), blob);
        }
        Ok(Release { manifest, files })
    }
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub harness: HarnessInfo,
    #[serde(default)]
    pub upstream: BTreeMap<String, lock::Upstream>,
    #[serde(default)]
    pub changes: Changes,
}

#[derive(Debug, Deserialize)]
pub struct HarnessInfo {
    pub name: String,
    pub version: String,
    pub min_workbench: String,
}

/// The release's rename/removal map, cumulative across releases.
#[derive(Debug, Default, Deserialize)]
pub struct Changes {
    #[serde(default)]
    pub renamed: BTreeMap<String, String>,
    #[serde(default)]
    pub removed: Vec<String>,
}

pub struct Release {
    pub manifest: Manifest,
    /// Source path → content, for everything under `skills/` and `beta/`.
    files: BTreeMap<String, Blob>,
}

/// What a release installs into a project.
pub struct Install {
    /// Project path → content.
    pub files: BTreeMap<String, Blob>,
    pub skills: BTreeMap<String, lock::Skill>,
    pub warnings: Vec<String>,
}

impl Release {
    fn skill_names(&self, dir: &str) -> BTreeSet<String> {
        let prefix = format!("{dir}/");
        self.files
            .keys()
            .filter_map(|path| path.strip_prefix(&prefix))
            .filter_map(|rest| rest.split_once('/'))
            .filter(|(_, file)| *file == "SKILL.md")
            .map(|(name, _)| name.to_owned())
            .collect()
    }

    /// The skills the configuration selects: enabled stable skills and opted-in beta skills.
    pub fn install(&self, config: &Config) -> Install {
        let stable = self.skill_names(STABLE_DIR);
        let beta = self.skill_names(BETA_DIR);
        let mut warnings = Vec::new();
        let mut chosen = BTreeMap::new();
        for name in stable.iter().filter(|name| config.is_enabled(name)) {
            chosen.insert(name.clone(), "stable");
        }
        for name in &config.beta {
            if beta.contains(name) {
                chosen.insert(name.clone(), "beta");
            } else {
                warnings.push(format!(
                    "beta skill `{name}` is not in the harness source; skipped"
                ));
            }
        }
        for name in config.enabled.iter().filter(|n| *n != "*") {
            if !stable.contains(name) {
                warnings.push(format!(
                    "enabled skill `{name}` is not in the harness source; skipped"
                ));
            }
        }
        self.install_skills(&chosen, warnings)
    }

    /// The given skills (name → channel) as this release holds them; missing ones are skipped.
    pub fn install_skills(
        &self,
        chosen: &BTreeMap<String, &str>,
        warnings: Vec<String>,
    ) -> Install {
        let mut files = BTreeMap::new();
        let mut skills = BTreeMap::new();
        for (name, channel) in chosen {
            let dir = if *channel == "beta" {
                BETA_DIR
            } else {
                STABLE_DIR
            };
            let prefix = format!("{dir}/{name}/");
            for (path, blob) in &self.files {
                if let Some(rest) = path.strip_prefix(&prefix) {
                    files.insert(format!("{}/{name}/{rest}", skills::DIR), blob.clone());
                }
            }
            skills.insert(
                name.clone(),
                lock::Skill {
                    channel: (*channel).to_owned(),
                    upstream: self.manifest.upstream.get(name).cloned(),
                },
            );
        }
        Install {
            files,
            skills,
            warnings,
        }
    }
}

fn git(dir: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .context("run git")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}
