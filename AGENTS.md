# agent-workbench

Canonical instructions for agents in this repository. More specific project instructions
override this file. Harness provenance and file hashes are recorded in `.harness/harness.lock`.

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
- Capabilities: `mattpocock-suite` (agent-harness 1.3.0)
- Stage: seed — decisions and research are done, no product code yet
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
cargo test                                                    # test (black-box tests drive the built binaries)
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings   # lint
cargo build --release                                         # build
```

Harness health, from a checkout of agent-harness: `python3 harness/bin/harness health <this repo>`.

## Repository map

- `command-center/` — the command center: a Herdr plugin and its `workbench-cc` binary.
- `docs/brief.md` — why, outcome, principles, scope.
- `docs/decisions.md` — the owner's dated decisions, newest first.
- `docs/research/` — dated research notes; read the ones relevant to the task
  (`docs/research/README.md` is the index).
- `docs/agents/` — configuration the pipeline skills read: issue tracker, triage labels, domain
  docs.
- `GLOSSARY.md`, `docs/adr/` — domain glossary and ADRs, created lazily by `domain-modeling`.
- `.harness/` — managed skills, registry and lock. Do not edit by hand.
- `crates/workbench/` — the `workbench` binary: `init`, `sync`, `update`, `config`.
- `harness.toml`, `skills/` — this repository as a harness source for `workbench init --from`:
  the default method (a tracked fork of `mattpocock/skills` with each skill's upstream commit in
  `harness.toml`, plus our own skills, among them the user-invoked `conduct` skill (#70) that
  drives the command center's lane commands). Until this repository migrates onto it (#55),
  agents here still run on the bootstrap copy in `.harness/`.
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
| Implementation | `implement` (drives `tdd`, closes with `code-review`) | branch, PR, issue comment |
| Incoming issues | `triage` | agent-ready issue |
| Hard failure | `diagnosing-bugs` | regression test and fix |
| Research | `research` | `docs/research/YYYY-MM-DD-<slug>.md` with a source for every claim |

`ask-matt` explains the full flow. Once issue #2 ships the artifact contract, this repository
switches its own specs to that contract (specs as files approved through a PR).

## Boundaries

- Allowed: edit files in this repository; create branches and PRs; comment on, label and link
  issues in this repository; run local builds and tests.
- Ask first: new product or scope decisions (record the answer in `docs/decisions.md`); merge to
  `main`; releases and package publishing; changes outside this repository (other repositories,
  local Herdr or agent config); retiring or deleting earlier harness lines; harness capability
  updates.
- Do not touch: files under `.harness/skills/` (managed; update through `harness diff` and
  `harness update`); secrets and credentials. The repository is public: never add personal paths,
  secrets, participant data or content from private repositories.

## Definition of Done

- The issue's acceptance criteria are met.
- Relevant checks pass (tests, lint, build once they exist; `harness health` after harness changes).
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

- Skills live in `.harness/skills` and are discovered through `.agents/skills` and
  `.claude/skills`.
- If a runtime has no native project skill root, search `.harness/skills/REGISTRY.md` by intent and
  open only the exact matching `SKILL.md`, respecting its Invocation column. User-only skills
  require explicit invocation; unknown/conflicting policy needs review before implicit routing.
- Harness-managed skill files and their version lock live in `.harness/`.
- Project MCP, plugin, hook, or runtime config is native; record its path, hash, runtime, and
  secret variable names in `.harness/integrations.json` without storing secret values.
- Capability updates are explicit and require a reviewed harness diff.

## Agent skills

### Issue tracker

GitHub Issues of `KirillSachkov/agent-workbench` via `gh`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default roles: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`,
`wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `GLOSSARY.md` and `docs/adr/` at the root, plus the owner's decision log. See
`docs/agents/domain.md`.

## Known pitfalls

- The harness is bootstrapped from agent-harness. This project is meant to replace that line, so
  when #1 ships its own installer, the harness here moves to it.
- Earlier harness lines (agent-harness, the Inside harness, the personal Herdr setup) are sources of
  ideas, not code to port.
- No project-specific profile (for example Inside) belongs here; consumers configure the harness in
  their own repositories.
- Read agents through public interfaces (Herdr API, `claude agents --json`, Codex app-server), never
  through internal session files.
