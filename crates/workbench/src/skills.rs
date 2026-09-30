//! Skills in a project's canonical `.agents/skills` directory and their `SKILL.md` frontmatter.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub const DIR: &str = ".agents/skills";

#[derive(Clone, Debug)]
pub struct Skill {
    /// The skill's directory name, which is how runtimes and the lock name it.
    pub name: String,
    pub description: String,
    /// `disable-model-invocation: true`: only the user starts it, by name.
    pub user_invoked: bool,
}

/// Every skill directory with a `SKILL.md`, sorted by name.
pub fn scan(project: &Path) -> Result<Vec<Skill>> {
    let dir = project.join(DIR);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut skills = Vec::new();
    for entry in entries {
        let entry = entry?;
        let skill_md = entry.path().join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let text = fs::read_to_string(&skill_md)
            .with_context(|| format!("read {}", skill_md.display()))?;
        let front = Frontmatter::parse(&text);
        skills.push(Skill {
            name,
            description: front.get("description").unwrap_or_default(),
            user_invoked: front.get("disable-model-invocation").as_deref() == Some("true"),
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// The top-level scalar fields of a YAML frontmatter block. Enough for `name`, `description` and
/// `disable-model-invocation`: plain, quoted and folded (`>`, `|`) scalars.
struct Frontmatter {
    lines: Vec<String>,
}

impl Frontmatter {
    fn parse(text: &str) -> Frontmatter {
        let mut lines = text.lines();
        if lines.next().map(str::trim_end) != Some("---") {
            return Frontmatter { lines: Vec::new() };
        }
        let lines = lines
            .take_while(|line| line.trim_end() != "---")
            .map(str::to_owned)
            .collect();
        Frontmatter { lines }
    }

    fn get(&self, key: &str) -> Option<String> {
        let prefix = format!("{key}:");
        let index = self.lines.iter().position(|l| l.starts_with(&prefix))?;
        let value = self.lines[index][prefix.len()..].trim();
        if matches!(value, ">" | ">-" | "|" | "|-") {
            let block: Vec<&str> = self.lines[index + 1..]
                .iter()
                .take_while(|l| l.starts_with(' ') || l.trim().is_empty())
                .map(|l| l.trim())
                .collect();
            let separator = if value.starts_with('>') { " " } else { "\n" };
            return Some(block.join(separator).trim().to_owned());
        }
        Some(unquote(value))
    }
}

fn unquote(value: &str) -> String {
    let quoted = |q: char| value.len() >= 2 && value.starts_with(q) && value.ends_with(q);
    if quoted('"') {
        value[1..value.len() - 1].replace("\\\"", "\"")
    } else if quoted('\'') {
        value[1..value.len() - 1].replace("''", "'")
    } else {
        value.to_owned()
    }
}
