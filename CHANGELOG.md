# Changelog

Releases of the agent-workbench harness and the `workbench` binary, newest first. A release is a
semver tag `vX.Y.Z` of this repository; see [docs/releasing.md](docs/releasing.md). The command
center has its own tags and notes in [command-center/README.md](command-center/README.md).

Each entry lists the skills added, changed, renamed and removed. Renames and removals are also in
`[changes]` of `harness.toml`, which `workbench update` applies.

## 0.1.0

The first release: the default method and the binary that installs it.

**The `workbench` binary.** `init --from <repository>@<ref>`, `sync`, `update [--to <ref>]` and
`config`; the harness lock `workbench-lock.json` with the upstream commit of every forked skill;
the project configuration `workbench.toml`. Install it with
`cargo install --locked --git https://github.com/KirillSachkov/agent-workbench --tag v0.1.0 workbench`.

**Skills.** A tracked fork of `mattpocock/skills` at `6a34259`, with later upstream changes
adopted: no em-dashes in skill text, the domain glossary is `GLOSSARY.md` (was `CONTEXT.md`), and
`resolving-merge-conflicts` is retired.

- Added: `pr` (from upstream; writes the result card), `coordinate` (read-only control brief),
  `conduct` (starts approved lanes through the command center), `editing-agents-md`.
- Changed:
  - `implement` closes with `code-review` then `pr`, or `handoff` when it stops unfinished; repair
    rounds, ambiguity to the owner, one writer per task.
  - `handoff` writes an issue comment and is model-invoked.
  - `code-review` reviewers are fresh-context and do not edit; findings are serious or minor.
  - `to-spec`, `to-tickets` and the `triage` brief carry preservation criteria.
  - `ask-matt` explains the tracks and acceptance modes.
  - No skill names a runtime's slash syntax.
- Renamed: `setup-matt-pocock-skills` → `setup-harness`, with fixed GitHub tracker templates,
  acceptance labels and a label script.
- Removed: `resolving-merge-conflicts`, as upstream did; agents resolve merge conflicts without a
  dedicated skill. (A removal like this is a major change after 1.0.)

**By hand on update.** Rename the project's glossary: `git mv CONTEXT.md GLOSSARY.md` (and
`CONTEXT-MAP.md` to `GLOSSARY-MAP.md` in a multi-context repository); the skills read only the new
names. Run the `setup-harness` skill again, or edit `docs/agents/*` from its templates, to pick up
the fixed tracker operations and the acceptance labels.

**Runtimes.** Tier 1: Claude Code, Codex and OpenCode, checked by
[the tier-1 smoke check](docs/smoke/2026-09-30-tier1.md).
