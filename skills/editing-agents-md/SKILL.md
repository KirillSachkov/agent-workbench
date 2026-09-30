---
name: editing-agents-md
description: Decide whether an instruction belongs in AGENTS.md at all, and write it so it holds. Use before adding, editing or removing anything in a root or nested AGENTS.md or CLAUDE.md, when a convention keeps being broken and someone proposes documenting it, or when the file has grown and needs a trim.
---

# Editing AGENTS.md

`AGENTS.md` is the project's instructions for every agent, and it belongs to the project. The root file loads into every session, in every runtime, for every task — so a line added there is paid for by every task, whether or not the task touches its subject. The question is never "is this true?" but "is this worth loading into every session?"

## What the harness owns, and what it doesn't

- **The managed block.** The section between `<!-- BEGIN workbench managed block … -->` and `<!-- END workbench managed block -->` is rewritten by `workbench sync`. Do not edit inside it; change the skills or `workbench.toml` and run `workbench sync`.
- **`CLAUDE.md`** is a bridge: one `@AGENTS.md` line so Claude Code reads the same file. Put instructions in `AGENTS.md`, not in `CLAUDE.md`; a Claude-only note may sit below the bridge line when it truly applies to one runtime.
- **Everything else** is the project's own text. Edit it freely, following the rules below.

## 1. Does it belong in the file?

Work down this ladder and stop at the first rung that fits:

1. **A check** — a linter rule, a type, a test, a CI step. When a matcher can see the violation, the check is the rule and a line of prose adds nothing.
2. **A skill** — the instruction matters for one kind of task (writing a migration, releasing, reviewing a PR). A skill loads on demand and can be ten times longer.
3. **A nested `AGENTS.md`** — the instruction applies to one directory. It loads only when an agent works there.
4. **The root `AGENTS.md`** — none of the above can carry it, and it applies across the repository or getting it wrong is expensive and nothing else catches it.

The root file is the last resort, not the default.

## 2. Check the smells

- **Lint leakage** — restating what a linter or CI already blocks. Keep at most the reason and name the check.
- **Context bloat** — content that does not apply to most sessions: environment setup for one tool, product documentation, history. Move it to `docs/` and point at it. Aim for a root file under about 150 lines.
- **Skill leakage** — task-specific steps in the always-loaded file. Move them into the skill for that task.
- **Conflicting instructions** — two lines that disagree, often because one aged. Search the file and nested files for the subject before adding yours.
- **Fossils** — text nobody has checked since it was generated. When you edit near a stale line, verify it and fix it in the same change.
- **Blind references** — a link with no word on what it holds or when to read it. Every pointer says "read this when …".

## 3. Write it so it holds

- State what must be true, specifically and in the imperative: "Every public function has a doc comment", "Use `uv pip`, never `pip`". A preference ("we like uv") changes nothing.
- Name the trigger, so a reader can tell from a diff when the line applies.
- Commands are exact and were run: copy them from a terminal where they worked.
- Never add secrets, personal paths, or one person's preferences — those live in each person's user-level instructions.
- Say each thing once. A rule stated in the root and in a nested file will drift apart.

## 4. Before you commit

- The line is on the lowest rung that can carry it.
- It reads as an invariant with a trigger.
- No other line in this file or a nested one says something different.
- Every path, link and skill name you added resolves.
- The file got shorter, or you can say what the added lines buy every session.

For the craft of writing anything agents read — skills, pointed-at docs — call the Skill tool with "writing-for-agents".
