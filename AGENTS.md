# agent-workbench

Canonical instructions for agents in this repository. More specific project instructions
override this file. This repository runs on its own harness: provenance, the upstream commit of
every forked skill and file hashes are in `workbench-lock.json`.

## Project

A general, configurable workbench for AI-first development with several agents (Claude Code, Codex
and others) across projects: a development harness and pipeline with stage artifacts, agent and
Herdr setup, and a command center (TUI in Herdr and a web dashboard) over one data core. It is not
tied to any particular consumer project. Goal and scope: `docs/brief.md`. Owner decisions:
`docs/decisions.md` — read both before any task.

- Type: software (developer tooling)
- Tools/stack: Rust + Ratatui for the core, CLI and TUI (one binary); React + Vite + TanStack +
  Tailwind for the web dashboard; Markdown with frontmatter and JSON Schema for harness content and
  the artifact contract
- Harness: agent-workbench 0.1.0 from this repository's own release (`workbench-lock.json`)
- Stage: building. Shipped: the `workbench` CLI and the default method 0.1.0, the command center
  slice 1 and the conductor v1
- Tracker: GitHub Issues of `KirillSachkov/agent-workbench`; see `docs/agents/issue-tracker.md`
- Base branch: `main`
- License: MIT
- Language: English for everything in the repository (code, docs, skills, issues, PRs, UI). Talk to
  the owner in Russian.

## Commands

The Rust code is one Cargo workspace at the root; each binary is a member crate. The web app has
no slot yet.

```bash
rustup toolchain install stable --component rustfmt,clippy   # setup
cargo run -p workbench-cc -- overview                         # run the command center (see command-center/README.md)
cargo run -p workbench -- --help                              # run the harness installer (init, sync, update, config)
workbench update                                              # bring a newer release of our own harness in, on its own branch
cargo test                                                    # test (black-box tests drive the built binaries)
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings   # lint
cargo build --release                                         # build
scripts/smoke-tier1.sh --report docs/smoke/<date>-tier1.md    # tier-1 smoke check (before a release; spends tokens)
```

Releasing the harness and the binary: `docs/releasing.md`.

## Repository map

- `command-center/` — the command center: a Herdr plugin and its `workbench-cc` binary.
- `docs/brief.md` — why, outcome, principles, scope.
- `docs/decisions.md` — the owner's dated decisions, newest first.
- `docs/research/` — dated research notes; read the ones relevant to the task
  (`docs/research/README.md` is the index).
- `docs/agents/` — configuration the pipeline skills read: issue tracker, triage labels, domain
  docs.
- `GLOSSARY.md`, `docs/adr/` — domain glossary and ADRs, created lazily by `domain-modeling`.
- `.agents/skills/` — the skills agents here run: this repository's harness installed from a
  release of `skills/`, reached by Claude Code through the `.claude/skills` link. With
  `workbench.toml`, `workbench-lock.json` and `opencode.json`.
- `crates/workbench/` — the `workbench` binary: `init`, `sync`, `update`, `config`.
- `harness.toml`, `skills/` — this repository as a harness source for `workbench init --from`:
  the default method (a tracked fork of `mattpocock/skills` with each skill's upstream commit in
  `harness.toml`, plus our own skills, among them the user-invoked `conduct` skill (#70) that
  drives the command center's lane commands); `beta/` for opt-in skills; `CHANGELOG.md` and
  `docs/releasing.md` for releases; `scripts/smoke-tier1.sh` and `docs/smoke/` for the tier-1
  smoke check.
- `docs/artifact-contract.md` — the documented shapes of tasks, specs, tickets, result cards and
  handoffs; read it before changing a skill that writes one or a tool that reads one.

## Pipeline

Stages are idea → spec → tickets → implementation. **The owner starts each stage**; an agent
completes the stage it was given and stops. The skill for each stage:

| Stage | Skill | Artifact |
|---|---|---|
| Idea | `grill-with-docs` (or `wayfinder` for a large, foggy effort) | `GLOSSARY.md`, ADRs, decision tickets |
| Spec | `to-spec` | GitHub issue with the spec |
| Tickets | `to-tickets` | GitHub issues with blocking edges |
| Implementation | `implement` (drives `tdd`, closes with `code-review` then `pr`, or `handoff`) | branch, PR with a result card, issue comment |
| Where things stand | `coordinate` (read-only) or `conduct` (starts approved lanes) | control brief, lanes |
| Incoming issues | `triage` | agent-ready issue |
| Hard failure | `diagnosing-bugs` | regression test and fix |
| Research | `research` | `docs/research/YYYY-MM-DD-<slug>.md` with a source for every claim |

`ask-matt` explains the full flow, the tracks and the acceptance modes. The shapes of the
artifacts are in `docs/artifact-contract.md`.

## Boundaries

- Allowed: edit files in this repository; create branches and PRs; comment on, label and link
  issues in this repository; run local builds and tests.
- Ask first: new product or scope decisions (record the answer in `docs/decisions.md`); merge to
  `main`; releases and package publishing; changes outside this repository (other repositories,
  local Herdr or agent config); retiring or deleting earlier harness lines; harness capability
  updates.
- Do not touch: secrets and credentials. Change skills in `skills/` (the source), not in
  `.agents/skills/` (the installed copy, refreshed by `workbench update` after a release). The repository is public: never add personal paths,
  secrets, participant data or content from private repositories.

## Definition of Done

- The issue's acceptance criteria are met.
- Relevant checks pass (tests, lint, build; after harness changes, `workbench sync` leaves the
  tree unchanged).
- New owner decisions are in `docs/decisions.md`; new research is a dated note with sources.
- The issue has a comment with the result, the verification and links to the artifacts.

Complete the authorized outcome and required checks. Reuse existing decisions; later clarifications
steer the task unless they replace it. State unavailable evidence at its actual level. After checks
pass, broaden verification only for new changes, failures, or unresolved concerns.

## Delivery

Work on a branch per issue and open a PR to `main` with `Closes #<issue>`. The owner reviews and
merges.

Project boundaries define the allowed environment and external actions. Prepare a reviewable
artifact before a required decision; do not make merge, production, or publication a prerequisite
for local work unless the current authorized stage explicitly requires it.

Preserve unrelated dirty changes and live worktrees. The active project skill or an explicit user
instruction chooses the checkout strategy.

## Runtime

- Skills live in `.agents/skills`; Claude Code reads them through the `.claude/skills` link. The
  invocation policy is `disable-model-invocation` in each `SKILL.md`; `workbench sync` renders it
  for Codex (`agents/openai.yaml`) and OpenCode (`opencode.json`, where a user-invoked skill runs
  as the `/<skill>` command).
- If a runtime has no native project skill root, search `.agents/skills/REGISTRY.md` by intent and
  open only the exact matching `SKILL.md`, respecting its Invocation column. User-invoked skills
  start only when the owner calls them by name.
- Harness updates come through `workbench update` on their own branch and PR.

## Agent skills

### Issue tracker

GitHub Issues of `KirillSachkov/agent-workbench` via `gh`. See `docs/agents/issue-tracker.md`.

### Labels

The five triage roles (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`,
`wontfix`) and the acceptance modes (`acceptance:human`, the default, and `acceptance:auto`). See
`docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the root, plus the owner's decision log. See
`docs/agents/domain.md`.

## Known pitfalls

- `skills/` is what this repository ships; `.agents/skills/` is what agents here run, installed
  from a release. A skill change reaches this repository's own agents only after a release and
  `workbench update`.
- The domain glossary is `GLOSSARY.md` (the skills no longer read `CONTEXT.md`).
- Skill text has no em-dashes and names skills plainly, without a runtime's slash syntax;
  `crates/workbench/tests/default_method.rs` checks the syntax and the frontmatter.
- Earlier harness lines (agent-harness, the Inside harness, the personal Herdr setup) are sources of
  ideas, not code to port.
- No project-specific profile (for example Inside) belongs here; consumers configure the harness in
  their own repositories.
- Read agents through public interfaces (Herdr API, `claude agents --json`, Codex app-server), never
  through internal session files.

<!-- BEGIN workbench managed block: workbench sync rewrites everything up to END -->

## Harness

This project's harness is agent-workbench 0.1.0, installed by `workbench` from `https://github.com/KirillSachkov/agent-workbench` at `v0.1.0`.

- Skills live in `.agents/skills`; Claude Code reads them through `.claude/skills`. `.agents/skills/REGISTRY.md` lists them.
- User-invoked skills start only when the user calls them by name: `ask-matt`, `conduct`, `grill-me`, `grill-with-docs`, `implement`, `improve-codebase-architecture`, `setup-harness`, `teach`, `to-questionnaire`, `to-spec`, `to-tickets`, `triage`, `wait-what`, `wayfinder`.
- The tracks (small, medium, large) and what workbench's code reads are in `workbench.toml`; `workbench config` prints the resolved configuration.
- `workbench update` brings a new harness version in on its own branch; `workbench sync` regenerates derived files. Edit harness files freely: updates merge your edits.

<!-- END workbench managed block -->
