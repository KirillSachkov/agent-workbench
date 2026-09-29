# Decisions

Owner decisions, newest first. Each entry has a date, the decision and where it came from.

## 2026-09-29 (harness requirements, wayfinder #5)

Decisions from the harness requirements map, one ticket per decision.

- **The artifact contract is the core, neutral to method** (A1, #6). agent-workbench defines its
  own artifact contract and stage model and ships a default method. A stage counts when its artifact
  passes the contract, whatever produced it. No adapters to spec-kit, OpenSpec or BMAD are built or
  maintained. Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §11.2.6, §11.3;
  ADR `docs/adr/0001-method-neutral-artifact-contract.md`.

## 2026-09-29 (open questions from the brief)

Answers to the five open questions of `docs/brief.md`, given by the owner in a working session.

- **License: MIT.**
- **Language: English everywhere in the project** — harness, pipeline document, skills, templates,
  CLI, TUI and web interfaces, README, brief, decisions, research notes, code and comments. Only the
  conversation with the owner is in Russian.
- **Order: pipeline and artifacts first.** #1 (harness and pipeline) → #2 (artifact contract) → #3
  (command center). #4 (workspace and Herdr) runs in parallel or after. The owner still picks which
  task implementation starts with.
- **Stack, chosen for the task rather than for continuity with earlier code:**
  - core, CLI and TUI: Rust with Ratatui, one binary. Same language as Herdr, most Herdr plugins,
    Codex and vibe-kanban.
  - web dashboard: React + Vite + TanStack (Query, Router) + Tailwind, built to static assets and
    served by the core.
  - harness content and artifacts: Markdown with frontmatter; the artifact contract is described in
    JSON Schema so any language and CI can validate it.
  - Evidence: `docs/research/2026-09-29-stack-landscape.md`.
- **Existing code is not a constraint.** Earlier harness lines (`agent-harness`, the Inside harness
  package, the personal Herdr setup) are sources of ideas, not code to port. The stack and the design
  are chosen fresh.
- **A general harness with no project-specific parts.** agent-workbench is a standalone project, not
  tied to any consumer. It is configurable, as flexible as possible and convenient to use. There is
  no Inside profile here. The Inside harness stays where it is and is not touched for now.

## 2026-09-29

- **A separate new repository `agent-workbench`** in the `KirillSachkov` account, public. The earlier
  harness lines (`agent-harness`, the `inside-engineering` package in Workspace Inside, the personal
  Herdr setup) are not developed further as separate products; the owner intends to delete them
  after the useful parts are carried over. Deletion is a separate action after that.
- **Project scope:** the development harness and pipeline, the stage artifact contract, agent and
  Herdr setup with plugins, the data aggregation core, a TUI panel in Herdr and a web dashboard,
  one-command installation.
- **Both command center interfaces are needed** — TUI and web on top of one core. Discussion stage
  now, implementation later.
- **Universality:** the solution must not depend on one agent or vendor app (Codex app, Claude
  desktop) and must work for a team.
- **Own tracker:** project tasks live in this repository's Issues; tasks from Workspace Inside
  (#245, #246) move here.
- **No large ready-made platform** on the level of Paperclip: a combination of small tools, Herdr
  plugins and a thin layer of our own.

## 2026-09-27

- The pipeline specification is a file in the repository approved through a PR, not an issue body
  (decision on workspace#245).
- Agent hooks come as a separate step after the artifact contract (decision on workspace#245).
- The owner needs the final artifacts of each stage, not watching the agent while it works: live
  sessions are already covered by Herdr.
