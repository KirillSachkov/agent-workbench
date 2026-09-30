//! The harness lock: which harness source and version a project uses, the upstream of each forked
//! skill and the hash of every file the harness wrote (the merge base for updates). Sorted and
//! timestamp-free so that it diffs and merges well.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::files;

pub const FILE: &str = "workbench-lock.json";
const FORMAT: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lock {
    pub version: u32,
    pub source: Source,
    pub harness: Harness,
    pub min_workbench_version: String,
    pub skills: BTreeMap<String, Skill>,
    /// Project path of every file the harness wrote → SHA-256 of the content it wrote.
    pub files: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub repository: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub commit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Harness {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    /// `stable` or `beta`: the source directory the skill came from.
    pub channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<Upstream>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Upstream {
    pub repository: String,
    pub commit: String,
}

impl Lock {
    pub fn new(
        source: Source,
        harness: Harness,
        min_workbench_version: String,
        skills: BTreeMap<String, Skill>,
        files: BTreeMap<String, String>,
    ) -> Self {
        Lock {
            version: FORMAT,
            source,
            harness,
            min_workbench_version,
            skills,
            files,
        }
    }

    pub fn load(project: &Path) -> Result<Option<Lock>> {
        let path = project.join(FILE);
        let Some(bytes) = files::read_optional(&path)? else {
            return Ok(None);
        };
        let lock = serde_json::from_slice(&bytes).with_context(|| format!("parse {FILE}"))?;
        Ok(Some(lock))
    }

    /// Writes the lock; returns whether its content changed.
    pub fn save(&self, project: &Path) -> Result<bool> {
        let mut text = serde_json::to_string_pretty(self)?;
        text.push('\n');
        let path = project.join(FILE);
        if files::read_optional(&path)?.as_deref() == Some(text.as_bytes()) {
            return Ok(false);
        }
        files::write(&path, text.as_bytes())?;
        Ok(true)
    }

    /// Checks this binary against the lock's minimum version.
    pub fn guard(&self) -> Result<()> {
        crate::guard::require(&self.min_workbench_version, FILE)
    }
}
