//! `AGENTS.md` belongs to the project: the harness creates a skeleton only when none exists, owns
//! one marked managed block, and bridges Claude Code to it through `CLAUDE.md`.

use crate::lock::Lock;
use crate::skills::Skill;

pub const FILE: &str = "AGENTS.md";
pub const CLAUDE_FILE: &str = "CLAUDE.md";
pub const BRIDGE: &str = "@AGENTS.md\n";

pub const BEGIN_PREFIX: &str = "<!-- BEGIN workbench managed block";
const BEGIN: &str =
    "<!-- BEGIN workbench managed block: workbench sync rewrites everything up to END -->";
const END: &str = "<!-- END workbench managed block -->";

pub fn skeleton(project_name: &str) -> String {
    format!(
        "# {project_name}\n\
         \n\
         Instructions for agents working in this repository.\n\
         \n\
         ## Project\n\
         \n\
         <!-- What this project is and who it is for, in two or three sentences. -->\n\
         \n\
         ## Commands\n\
         \n\
         <!-- How to set up, run, test, lint and build. -->\n\
         \n\
         ## Boundaries\n\
         \n\
         <!-- What agents may do on their own, what needs the owner, what they must not touch. -->\n"
    )
}

/// The managed block's text, markers included.
pub fn block(skills: &[Skill], lock: Option<&Lock>) -> String {
    let mut out = format!("{BEGIN}\n\n## Harness\n\n");
    if let Some(lock) = lock {
        out.push_str(&format!(
            "This project's harness is {} {}, installed by `workbench` from `{}` at `{}`.\n\n",
            lock.harness.name, lock.harness.version, lock.source.repository, lock.source.reference
        ));
    }
    out.push_str(
        "- Skills live in `.agents/skills`; Claude Code reads them through `.claude/skills`. \
         `.agents/skills/REGISTRY.md` lists them.\n",
    );
    let user: Vec<String> = skills
        .iter()
        .filter(|s| s.user_invoked)
        .map(|s| format!("`{}`", s.name))
        .collect();
    if !user.is_empty() {
        out.push_str(&format!(
            "- User-invoked skills start only when the user calls them by name: {}.\n",
            user.join(", ")
        ));
    }
    out.push_str(
        "- The tracks (small, medium, large) and what workbench's code reads are in \
         `workbench.toml`; `workbench config` prints the resolved configuration.\n\
         - `workbench update` brings a new harness version in on its own branch; \
         `workbench sync` regenerates derived files. Edit harness files freely: updates merge \
         your edits.\n",
    );
    out.push('\n');
    out.push_str(END);
    out
}

/// Inserts or refreshes the managed block; text outside it is kept byte for byte. `None` when a
/// BEGIN marker has no END, since the block's extent is then unknown.
pub fn with_block(agents_md: &str, block: &str) -> Option<String> {
    if let Some(start) = agents_md.find(BEGIN_PREFIX) {
        let end_offset = agents_md[start..].find(END)?;
        let end = start + end_offset + END.len();
        return Some(format!(
            "{}{block}{}",
            &agents_md[..start],
            &agents_md[end..]
        ));
    }
    let mut out = agents_md.to_owned();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(block);
    out.push('\n');
    Some(out)
}

pub fn has_bridge(claude_md: &str) -> bool {
    claude_md.lines().any(|line| line.trim() == "@AGENTS.md")
}
