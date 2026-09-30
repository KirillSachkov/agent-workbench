//! `workbench init --from <repo>@<ref>`: writes a harness source into a project. Safe to rerun
//! with the same source: missing files come back, edited files are kept.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, bail};

use crate::agents_md;
use crate::config::{self, Config};
use crate::files;
use crate::guard;
use crate::lock::{self, Lock};
use crate::source::{Checkout, Spec};
use crate::sync;

pub fn run(project: &Path, from: &str) -> Result<()> {
    let spec = Spec::parse(from)?;
    let existing = Lock::load(project)?;
    if let Some(lock) = &existing {
        lock.guard()?;
    }
    let config = Config::load(project)?;

    let checkout = Checkout::clone(&spec.repository)?;
    let reference = match spec.reference {
        Some(reference) => reference,
        None => checkout.default_branch()?,
    };
    let commit = checkout.resolve(&reference)?;
    let release = checkout.release(&commit)?;
    let harness = &release.manifest.harness;
    guard::require(
        &harness.min_workbench,
        &format!("harness {} {}", harness.name, harness.version),
    )?;

    if let Some(lock) = &existing {
        let same = lock.source.repository == spec.repository
            && lock.source.reference == reference
            && lock.source.commit == commit;
        if !same {
            bail!(
                "this project already has a harness from {}@{} ({}); run `workbench update --to \
                 <ref>` to move to another version (nothing was written)",
                lock.source.repository,
                lock.source.reference,
                short(&lock.source.commit)
            );
        }
    }

    let mut install = release.install(&config);
    if install.skills.is_empty() {
        install
            .warnings
            .push("the harness source installs no skills (its skills/ directory is empty)".into());
    }
    let mut kept = Vec::new();
    let mut hashes = BTreeMap::new();
    for (path, blob) in &install.files {
        let full = project.join(path);
        match files::read_optional(&full)? {
            Some(current) if current != blob.bytes => kept.push(path.clone()),
            Some(_) => files::set_executable(&full, blob.executable)?,
            None => files::write_blob(&full, blob)?,
        }
        hashes.insert(path.clone(), files::sha256(&blob.bytes));
    }

    let mut notes = Vec::new();
    let agents_path = project.join(agents_md::FILE);
    if !agents_path.exists() {
        let name = project
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Project".into());
        files::write(&agents_path, agents_md::skeleton(&name).as_bytes())?;
        notes.push(format!("created a minimal {}", agents_md::FILE));
    }
    let claude_path = project.join(agents_md::CLAUDE_FILE);
    match files::read_optional(&claude_path)? {
        None => {
            files::write(&claude_path, agents_md::BRIDGE.as_bytes())?;
            notes.push(format!("created the {} bridge", agents_md::CLAUDE_FILE));
        }
        Some(bytes) if !agents_md::has_bridge(&String::from_utf8_lossy(&bytes)) => {
            notes.push(format!(
                "{} exists without an `@AGENTS.md` line; left as is — add that line so Claude \
                 Code reads AGENTS.md",
                agents_md::CLAUDE_FILE
            ));
        }
        Some(_) => {}
    }
    let config_path = project.join(config::FILE);
    if !config_path.exists() {
        files::write(&config_path, config::TEMPLATE.as_bytes())?;
        notes.push(format!("created {}", config::FILE));
    }

    let lock = Lock::new(
        lock::Source {
            repository: spec.repository,
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

    println!(
        "Installed {} {} from {}@{} ({}): {} skills.",
        lock.harness.name,
        lock.harness.version,
        lock.source.repository,
        lock.source.reference,
        short(&lock.source.commit),
        lock.skills.len()
    );
    for note in &notes {
        println!("- {note}");
    }
    for path in &kept {
        println!("- kept the project's edited {path}");
    }
    for (path, _) in &report.stale {
        println!("- wrote {path}");
    }
    for warning in install.warnings.iter().chain(&report.warnings) {
        eprintln!("warning: {warning}");
    }
    Ok(())
}

pub fn short(commit: &str) -> &str {
    &commit[..commit.len().min(12)]
}
