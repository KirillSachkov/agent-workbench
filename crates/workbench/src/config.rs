//! Project configuration: the built-in default overlaid by the project's `workbench.toml`.

use std::path::Path;

use anyhow::{Context, Result, bail};
use toml::{Table, Value};

use crate::files;

pub const FILE: &str = "workbench.toml";
const DEFAULT: &str = include_str!("default.toml");

/// Written by `init` when the project has no configuration yet. Everything is commented out, so
/// the built-in default applies until the project changes something.
pub const TEMPLATE: &str = r#"# Project configuration of the harness. It holds only what workbench's code reads; everything
# else is customised by editing the project's own files. Values here win over the built-in
# default; `workbench config` prints the resolved configuration.

# [skills]
# enabled = ["*"]     # stable skills to install; "*" installs all
# beta = []           # beta skills, opted in one by one

# [acceptance]
# default = "human"   # or "auto"

# [repair]
# max_rounds = 2
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkMode {
    Directory,
    PerSkill,
}

/// The resolved configuration and the parts workbench's own code reads.
pub struct Config {
    pub table: Table,
    pub enabled: Vec<String>,
    pub beta: Vec<String>,
    pub links: LinkMode,
}

impl Config {
    pub fn load(project: &Path) -> Result<Config> {
        let mut table: Table = DEFAULT.parse().expect("the built-in default is valid TOML");
        if let Some(bytes) = files::read_optional(&project.join(FILE))? {
            let text = String::from_utf8(bytes).with_context(|| format!("{FILE} is not UTF-8"))?;
            let project_table: Table = text.parse().with_context(|| format!("parse {FILE}"))?;
            overlay(&mut table, project_table);
        }
        let enabled = strings(&table, "skills", "enabled")?;
        let beta = strings(&table, "skills", "beta")?;
        let links = match table
            .get("links")
            .and_then(|l| l.get("claude"))
            .and_then(Value::as_str)
        {
            Some("directory") => LinkMode::Directory,
            Some("per-skill") => LinkMode::PerSkill,
            other => {
                bail!("{FILE}: links.claude must be \"directory\" or \"per-skill\", got {other:?}")
            }
        };
        Ok(Config {
            table,
            enabled,
            beta,
            links,
        })
    }

    pub fn is_enabled(&self, skill: &str) -> bool {
        self.enabled.iter().any(|s| s == "*" || s == skill)
    }
}

pub fn print(project: &Path) -> Result<()> {
    let config = Config::load(project)?;
    let text = toml::to_string_pretty(&config.table)?;
    println!("# Resolved configuration: the built-in default overlaid by {FILE}.\n");
    print!("{text}");
    Ok(())
}

/// Tables merge key by key; any other value replaces the default.
fn overlay(base: &mut Table, over: Table) {
    for (key, value) in over {
        match (base.get_mut(&key), value) {
            (Some(Value::Table(base_table)), Value::Table(over_table)) => {
                overlay(base_table, over_table)
            }
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

fn strings(table: &Table, section: &str, key: &str) -> Result<Vec<String>> {
    let Some(value) = table.get(section).and_then(|s| s.get(key)) else {
        return Ok(Vec::new());
    };
    let items = value
        .as_array()
        .with_context(|| format!("{FILE}: {section}.{key} must be a list of names"))?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .with_context(|| format!("{FILE}: {section}.{key} must be a list of names"))
        })
        .collect()
}
