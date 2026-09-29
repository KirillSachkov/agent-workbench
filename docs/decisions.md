# Decisions

Owner decisions, newest first. Each entry has a date, the decision and where it came from.

## 2026-09-29 (harness requirements, wayfinder #5)

Decisions from the harness requirements map, one ticket per decision.

- **The artifact contract is the core, neutral to method** (A1, #6). agent-workbench defines its
  own artifact contract and stage model and ships a default method. A stage counts when its artifact
  passes the contract, whatever produced it. No adapters to spec-kit, OpenSpec or BMAD are built or
  maintained. Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §11.2.6, §11.3;
  ADR `docs/adr/0001-method-neutral-artifact-contract.md`.
- **Matt Pocock's skills are a tracked fork** (A2, #7). The `mattpocock/skills` set (MIT) in the
  default method is our fork: we edit it as we need, record the upstream commit each skill was forked
  at, follow upstream changes and adopt useful ones selectively after review. Update mechanics belong
  to "Local edits across updates" and "Release channels for our skills". ADR
  `docs/adr/0002-tracked-fork-of-mattpocock-skills.md`.
- **Our skills follow the suite's conventions** (A2, #7): the user-invoked / model-invoked split,
  cross-skill calls phrased as "Call the Skill tool with ...", and a user-invoked skill never calls
  another user-invoked one. Requirement from the owner: the invocation policy must work on every
  supported agent runtime, not only Claude Code. Runtimes express it differently (Claude Code
  `disable-model-invocation`, Codex `policy.allow_implicit_invocation` in `agents/openai.yaml`,
  none per skill in Gemini CLI), so the harness renders one policy into each runtime's native form and
  states degradation explicitly (to be decided in E17 and E20). Evidence:
  `docs/research/2026-09-29-cross-runtime-portability.md` §3.3;
  `docs/research/2026-09-29-mattpocock-skills-design.md` §2.2, §5.
- **Default shape of agent work** (A3, #8). Two owner checkpoints by default — intent approval
  before code and acceptance (the merge) after — with agents working on their own in between. In
  between, machine checks block and review is done by someone other than the author (another agent or a
  fresh-context subagent); AI review gives findings, not verdicts. Every agent session starts from an
  artifact and ends with one; state lives in files, git and the tracker. Three tracks by task size:
  small (the task states intent, preservation and acceptance criteria; one PR), medium (spec file,
  then tickets), large (a wayfinder map, then medium). Evidence: `docs/research/2026-09-27-reviewing-agent-work-practices.md`
  §2, §6; `docs/research/2026-09-25-artifact-pipelines-academic.md` §7;
  `docs/research/2026-09-25-artifact-pipelines-industry.md` §8.
- **The harness stays minimal; process is chosen per task** (A3, #8). TDD, the implementation skill
  and other process choices are picked for the task, not mandated; the core is checks and facts,
  because practices and models change fast.
- **Owner checkpoints are defaults, not laws** (A3, #8). Some tasks may let an agent merge, and some
  may skip the pipeline; how that is allowed and stays visible is a separate ticket. Two more owner
  needs became tickets: a result package the owner can open in one step instead of hunting for PRs and
  files, and batch execution of a ticket set by an orchestrator (which touches the brief's "launching
  and orchestrating agents" exclusion).
- **The pipeline is an artifact graph** (B4, #9). Each track is declared as data: nodes (artifacts
  and facts such as "spec PR merged" or "CI green"), their dependencies, the checks each must pass and
  where the artifact lives. The harness computes each node's state (done, ready, blocked) from facts;
  it runs no steps and keeps no state of its own. How to produce a node stays in skills and is chosen
  per task. No fixed loop with extension points (GSD) and no workflow engine with control flow
  (spec-kit). Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §1.3, §2.3–2.4,
  §6.3, §11.1.5; ADR `docs/adr/0003-pipeline-as-artifact-graph.md`.
- **No profiles: a built-in default plus project configuration** (B5, #10). There is no profile
  system (no inheritance, no shared profile repositories). The harness ships one built-in default for
  software development; each project has one committed project configuration that adapts or replaces
  any part of it — tracks (artifact graphs), checks, enabled skills, tracker settings — and the
  resolved result is printable by a command. Coding standards and domain rules stay in the project's
  own documents (`AGENTS.md`, `CONTEXT.md`, standards files) that skills read; no keyword-selected
  policy packs. Harnesses for other kinds of work (for example content) live in their own
  repositories. Reversible: profiles can be layered on later if copying configuration between
  repositories becomes painful. Amends `docs/brief.md` outcome 1 ("Profiles for different kinds of
  projects"). Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §3.3, §8.3,
  §11.1.7–8, §11.3.3; `docs/research/2026-09-29-hoh-harness-design.md` §5.
- **Customise by editing files, keep configuration minimal** (B6, #11). Close to Matt Pocock's
  "Config is death": a project changes the pipeline by editing the files it owns — skills, templates,
  pipeline documents — as we did with the fork, not through settings. The project configuration holds
  only what our code (CLI, CI) must read to compute facts: artifact graphs, check commands, enabled
  skills, tracker and label mapping, artifact locations. No switches that change skill behaviour
  (no `tdd_mode`-style toggles); preferences that need judgment are plain text in the project's
  documents. No personal uncommitted layer: personal preferences belong in each runtime's user-level
  instruction files, secrets in environment variables. Consequence for "Local edits across updates":
  edited harness files in a project are normal and must survive updates. Evidence:
  `docs/research/2026-09-29-mattpocock-skills-design.md` §2.7, §4, §5;
  `docs/research/2026-09-29-harness-frameworks-compared.md` §6.3–6.4.
- **Agents repair failed checks themselves, within a limit** (B7, #12). When a mechanical check or
  review finding fails with a clear signal, the agent fixes it and reruns without involving the owner,
  for at most 2 repair rounds by default (changeable in the project configuration). Past the limit it
  stops with a report of what it tried and where it is stuck. Ambiguity in intent or a conflict with
  the spec goes back to the owner at once instead of being guessed. Infrastructure failures (network,
  timeouts, runners) do not count as rounds and are reported separately from product failures.
  Repairs must not weaken checks: changes to tests and CI are listed separately in the report. Rounds
  are counted from facts (CI runs on the PR), since the harness does not launch agents. Evidence:
  `docs/research/2026-09-29-hoh-harness-design.md` §2.2, §7;
  `docs/research/2026-09-27-agent-products-human-artifacts.md` §8 (Stripe);
  `docs/research/2026-09-27-reviewing-agent-work-practices.md` §1, §6.

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
