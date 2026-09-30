//! `workbench update [--to <ref>]`: brings a new harness version in on a new branch. Files the
//! project did not touch are replaced; edited files are merged three-way against the version the
//! lock records; conflicts stay as git conflict markers. The release's rename/removal map is
//! applied, derived files are regenerated and the lock is rewritten. The user reviews the diff and
//! opens a pull request.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempDir;

use crate::config::Config;
use crate::files::{self, Blob};
use crate::guard;
use crate::lock::{self, Lock};
use crate::skills;
use crate::source::{self, Checkout, Release, short};
use crate::sync;

pub fn run(project: &Path, to: Option<&str>) -> Result<()> {
    let Some(old_lock) = Lock::load(project)? else {
        bail!(
            "no {} here; install a harness first with `workbench init --from <repo>@<ref>`",
            lock::FILE
        );
    };
    old_lock.guard()?;
    project_git(project, &["rev-parse", "--git-dir"])
        .context("workbench update needs the project to be a git repository")?;
    let config = Config::load(project)?;

    let checkout = Checkout::clone(project, &old_lock.source.repository)?;
    let reference = match to {
        Some(reference) => reference.to_owned(),
        None => next_ref(&checkout, &old_lock)?,
    };
    let commit = checkout.resolve(&reference)?;
    let new = checkout.release(&commit)?;
    let harness = &new.manifest.harness;
    guard::require(
        &harness.min_workbench,
        &format!("harness {} {}", harness.name, harness.version),
    )?;
    let old = checkout.release(&old_lock.source.commit)?;

    let mut install = new.install(&config);
    let renamed = &new.manifest.changes.renamed;
    // A renamed skill stays installed under its new name, even when the configuration names the
    // old one or it came from the beta channel.
    for (from, to) in renamed {
        if let Some(skill) = old_lock.skills.get(from)
            && !install.skills.contains_key(to)
        {
            let chosen = BTreeMap::from([(to.clone(), skill.channel.as_str())]);
            let extra = new.install_skills(&chosen, Vec::new());
            install.files.extend(extra.files);
            install.skills.extend(extra.skills);
        }
    }
    let same_skills = install.skills.keys().eq(old_lock.skills.keys());
    if commit == old_lock.source.commit && same_skills {
        println!(
            "Already up to date: {} {} at {}@{}.",
            old_lock.harness.name, old_lock.harness.version, old_lock.source.repository, reference
        );
        return Ok(());
    }

    let status = source::git(project, &["status", "--porcelain"])?;
    if !status.is_empty() {
        bail!(
            "the working tree has uncommitted changes; commit or stash them first so the update \
             branch holds only the harness update (nothing was written)"
        );
    }

    let branch = branch_name(&reference, &commit, &old_lock);
    project_git(project, &["switch", "--quiet", "-c", &branch])
        .with_context(|| format!("create the update branch {branch}"))?;

    let mut outcome = Outcome::default();
    let base = base_files(project, &old, &old_lock, renamed, &mut outcome)?;
    let labels = Labels {
        base: format!("harness {}", old_lock.harness.version),
        theirs: format!("harness {}", harness.version),
    };

    let paths: BTreeSet<&String> = base.keys().chain(install.files.keys()).collect();
    for path in paths {
        let full = project.join(path);
        let ours = files::read_optional(&full)?;
        let theirs = install.files.get(path);
        match (base.get(path), theirs, ours) {
            (_, None, None) => {}
            (Some(_), Some(_), None) => outcome.kept_deleted.push(path.clone()),
            (None, Some(theirs), None) => {
                files::write_blob(&full, theirs)?;
                outcome.added.push(path.clone());
            }
            (Some(base), None, Some(ours)) => {
                if files::sha256(&ours) == base.hash {
                    files::remove_and_prune(&full, &project.join(skills::DIR))?;
                    outcome.removed.push(path.clone());
                } else {
                    outcome.kept_edited_removed.push(path.clone());
                }
            }
            (None, None, Some(_)) => unreachable!("every path comes from base or theirs"),
            (base, Some(theirs), Some(ours)) => {
                let untouched = base.is_some_and(|b| files::sha256(&ours) == b.hash);
                if ours == theirs.bytes {
                    files::set_executable(&full, theirs.executable)?;
                } else if untouched {
                    files::write_blob(&full, theirs)?;
                    outcome.replaced.push(path.clone());
                } else if base.is_some_and(|b| b.blob.bytes == theirs.bytes) {
                    // The harness did not change this file; the project's edit stands.
                } else {
                    let base_bytes = base.map(|b| b.blob.bytes.as_slice()).unwrap_or_default();
                    match merge(&ours, base_bytes, &theirs.bytes, &labels)? {
                        Merged::Clean(bytes) => {
                            files::write(&full, &bytes)?;
                            outcome.merged.push(path.clone());
                        }
                        Merged::Conflicts(bytes) => {
                            files::write(&full, &bytes)?;
                            outcome.conflicts.push(path.clone());
                        }
                        Merged::Binary => outcome.kept_binary.push(path.clone()),
                    }
                    files::set_executable(&full, theirs.executable)?;
                }
            }
        }
    }
    for name in &new.manifest.changes.removed {
        if old_lock.skills.contains_key(name) && !install.skills.contains_key(name) {
            outcome.retired.push(name.clone());
        }
    }

    let hashes = install
        .files
        .iter()
        .map(|(path, blob)| (path.clone(), files::sha256(&blob.bytes)))
        .collect();
    let lock = Lock::new(
        lock::Source {
            repository: old_lock.source.repository.clone(),
            reference,
            commit,
        },
        lock::Harness {
            name: harness.name.clone(),
            version: harness.version.clone(),
        },
        harness.min_workbench.clone(),
        install.skills,
        hashes,
    );
    lock.save(project)?;
    let report = sync::derive(project, Some(&lock), &config)?;

    outcome.print(&old_lock, &lock, &branch, &report);
    for warning in install.warnings.iter().chain(&report.warnings) {
        eprintln!("warning: {warning}");
    }
    Ok(())
}

/// The newest release tag above the locked one when the lock follows tags; otherwise the locked
/// branch or ref again.
fn next_ref(checkout: &Checkout, lock: &Lock) -> Result<String> {
    let Ok(current) = guard::parse(&lock.source.reference) else {
        return Ok(lock.source.reference.clone());
    };
    let newest = checkout
        .tags()?
        .into_iter()
        .filter_map(|tag| guard::parse(&tag).ok().map(|v| (v, tag)))
        .filter(|(version, _)| version.pre.is_empty() && *version > current)
        .max();
    Ok(newest.map_or_else(|| lock.source.reference.clone(), |(_, tag)| tag))
}

fn branch_name(reference: &str, commit: &str, lock: &Lock) -> String {
    let safe: String = reference
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._-".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect();
    if reference == lock.source.reference {
        format!("workbench/update-{safe}-{}", short(commit))
    } else {
        format!("workbench/update-{safe}")
    }
}

struct BaseFile {
    blob: Blob,
    /// What the harness wrote, per the lock.
    hash: String,
}

/// The merge base of every harness file: the locked version's content, keyed by where the file
/// lives now. Renamed skills are moved to their new directory first.
fn base_files(
    project: &Path,
    old: &Release,
    lock: &Lock,
    renamed: &BTreeMap<String, String>,
    outcome: &mut Outcome,
) -> Result<BTreeMap<String, BaseFile>> {
    let chosen: BTreeMap<String, &str> = lock
        .skills
        .iter()
        .map(|(name, skill)| (name.clone(), skill.channel.as_str()))
        .collect();
    let old_install = old.install_skills(&chosen, Vec::new());

    let mut moves = BTreeMap::new();
    for (from, to) in renamed {
        if !lock.skills.contains_key(from) {
            continue;
        }
        let from_dir = project.join(skills::DIR).join(from);
        let to_dir = project.join(skills::DIR).join(to);
        if to_dir.exists() {
            outcome.notes.push(format!(
                "skill `{from}` is renamed to `{to}`, but {}/{to} already exists; both are kept, \
                 merge `{from}` into `{to}` by hand",
                skills::DIR
            ));
            continue;
        }
        if from_dir.exists() {
            fs::rename(&from_dir, &to_dir).with_context(|| {
                format!("rename {} to {}", from_dir.display(), to_dir.display())
            })?;
        }
        outcome.renamed.push((from.clone(), to.clone()));
        moves.insert(
            format!("{}/{from}/", skills::DIR),
            format!("{}/{to}/", skills::DIR),
        );
    }

    let mut base = BTreeMap::new();
    for (path, hash) in &lock.files {
        let Some(blob) = old_install.files.get(path) else {
            continue;
        };
        let moved = moves
            .iter()
            .find_map(|(from, to)| {
                path.strip_prefix(from.as_str())
                    .map(|rest| format!("{to}{rest}"))
            })
            .unwrap_or_else(|| path.clone());
        base.insert(
            moved,
            BaseFile {
                blob: blob.clone(),
                hash: hash.clone(),
            },
        );
    }
    Ok(base)
}

struct Labels {
    base: String,
    theirs: String,
}

enum Merged {
    Clean(Vec<u8>),
    Conflicts(Vec<u8>),
    Binary,
}

fn merge(ours: &[u8], base: &[u8], theirs: &[u8], labels: &Labels) -> Result<Merged> {
    let dir = TempDir::new()?;
    let write = |name: &str, bytes: &[u8]| -> Result<String> {
        let path = dir.path().join(name);
        fs::write(&path, bytes)?;
        Ok(path.display().to_string())
    };
    let (ours, base, theirs) = (
        write("ours", ours)?,
        write("base", base)?,
        write("theirs", theirs)?,
    );
    let output = Command::new("git")
        .args(["merge-file", "-p", "-L", "project", "-L"])
        .arg(&labels.base)
        .arg("-L")
        .arg(&labels.theirs)
        .args([&ours, &base, &theirs])
        .output()
        .context("run git merge-file")?;
    match output.status.code() {
        Some(0) => Ok(Merged::Clean(output.stdout)),
        Some(code) if (1..=127).contains(&code) => Ok(Merged::Conflicts(output.stdout)),
        _ if String::from_utf8_lossy(&output.stderr).contains("binary") => Ok(Merged::Binary),
        _ => bail!(
            "git merge-file failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    }
}

fn project_git(project: &Path, args: &[&str]) -> Result<()> {
    source::git(project, args).map(drop)
}

#[derive(Default)]
struct Outcome {
    replaced: Vec<String>,
    merged: Vec<String>,
    conflicts: Vec<String>,
    added: Vec<String>,
    removed: Vec<String>,
    renamed: Vec<(String, String)>,
    retired: Vec<String>,
    kept_deleted: Vec<String>,
    kept_edited_removed: Vec<String>,
    kept_binary: Vec<String>,
    notes: Vec<String>,
}

impl Outcome {
    fn print(&self, old: &Lock, new: &Lock, branch: &str, report: &sync::Report) {
        println!(
            "Updated {} {} → {} ({}@{}, {}) on branch {branch}.",
            new.harness.name,
            old.harness.version,
            new.harness.version,
            new.source.repository,
            new.source.reference,
            short(&new.source.commit)
        );
        let list = |title: &str, paths: &[String]| {
            for path in paths {
                println!("- {title}: {path}");
            }
        };
        list("replaced", &self.replaced);
        list("merged", &self.merged);
        list("CONFLICT", &self.conflicts);
        list("added", &self.added);
        list("removed", &self.removed);
        for (from, to) in &self.renamed {
            println!("- renamed skill: {from} → {to}");
        }
        list("retired skill", &self.retired);
        list("kept, deleted in the project", &self.kept_deleted);
        list(
            "kept, edited in the project but dropped by the harness",
            &self.kept_edited_removed,
        );
        list(
            "kept the project's version of an edited binary file",
            &self.kept_binary,
        );
        for note in &self.notes {
            println!("- note: {note}");
        }
        for (path, _) in &report.stale {
            println!("- regenerated {path}");
        }
        if self.conflicts.is_empty() {
            println!("Next: review the diff, commit and open a pull request.");
        } else {
            println!(
                "Next: resolve the conflict markers in {} file(s), then commit and open a pull \
                 request.",
                self.conflicts.len()
            );
        }
    }
}
