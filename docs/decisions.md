# Decisions

Owner decisions, newest first. Each entry has a date, the decision and where it came from.

## 2026-09-30 (default method, #55)

- **Fork base and adoption** (executor of #55, within ADR 0002). The bootstrap copy of the skills
  equals `mattpocock/skills` at `6a34259` byte for byte, so that commit is the recorded fork point
  of every forked skill. `pr` is adopted from upstream `e484a80`, where it is now promoted. The
  later upstream changes (the `CONTEXT.md` → `GLOSSARY.md` rename, the removal of
  `resolving-merge-conflicts`, the em-dash replacement) are adopted in #55's second PR, not the
  first (the owner, 2026-09-30). Upstream's fix for skills that tried to call a user-invoked
  skill is applied in our own words, together with removing every slash-command invocation from
  skill text.
- **Names** (accepted by the owner, 2026-09-30). The read-only coordinator skill is `coordinate`.
  The setup skill is renamed `setup-matt-pocock-skills` → `setup-harness` (recorded in the rename
  map). The router keeps its name `ask-matt`. The skill for editing `AGENTS.md` is `editing-agents-md`, after
  PostHog's.
- **`handoff` becomes model-invoked**, so that `implement` can end with it when a session stops
  unfinished (C10); a user-invoked skill cannot call another one.
- **The artifact contract is a page**, `docs/artifact-contract.md`: the index of the documented
  shapes, with the result-card headings the command center reads marked as read by tools.

## 2026-09-30 (conductor v1, #70)

- **Lanes start without a turn limit** (confirmed by the owner in the coordination session on
  2026-09-30). Claude Code's `--max-turns` and `--max-budget-usd` work only in print mode (`-p`),
  and lanes start interactive executors so the owner can answer them in their pane; Codex and
  OpenCode have no interactive equivalent either. So v1 starts every runtime without a turn limit, documents it, and makes each
  runtime's launch arguments configurable (`[lanes.args]`) so a limit can be added when a runtime
  offers one. The guard in v1 is `lane watch` with the owner's notification. Amends the "turn
  limit where supported" point of the conductor decision below.

## 2026-09-30 (workbench CLI, #54)

- **Open points of PR #72 settled** (answer from the coordination session). The harness lock
  hashes only the skill files the harness installs; derived and merged files are not hashed. An
  existing `CLAUDE.md` without the `@AGENTS.md` line gets the one-line bridge added on top, the
  rest of the file untouched (E18). Windows link handling stays best effort. The project files
  are named `workbench.toml` and `workbench-lock.json`.

## 2026-09-29 (command center, wayfinder #41)

Decisions taken while charting the command center map (#41) and resolving its tickets, newest first.

- **A light conductor; A4 amended** (#67 → spec #70). A user-invoked `conduct` skill for a
  coordination session in Herdr plus `lane start` / `lane watch` / `lane list` commands in the
  command center's binary: it proposes lanes of ready work, starts each approved lane in its own
  worktree and Herdr workspace with the chosen agent, watches it and calls the owner with a Herdr
  notification. The owner approves every start. It never merges, pushes, deletes branches or
  worktrees, answers an executor's question or approval, or types into a working agent. At most
  2–3 parallel lanes by default, shared surfaces not in parallel, executors with a turn limit where
  supported. Lanes show in the Work section; a canvas view later, CanvasTTY as a reference. A4 now
  reads: no orchestration platform of our own; a light conductor that starts executors with the
  owner's approval, watches them and calls the owner is part of the command center. Research:
  `docs/research/2026-09-30-conductor-practices.md`.
- **This repository becomes a harness source in #54** (2026-09-30, the owner's answer to the #54
  executor): `harness.toml` and `skills/` arrive with the CLI so its owner check runs literally; #55
  fills and refines that layout.

- **Coordination of the work itself** (2026-09-30). The owner loses track across many tickets and
  specs and wants a coordinator. Decided now: #62 and #54 run in parallel (their only overlap is
  the workspace root and CI; #55 still waits for #54); the command center slice 1 gains a Work
  section (progress of each spec or map, work ready now, work in flight — #62); the default method
  gains a read-only coordinator skill that returns lanes, blockers, progress and the next step and
  launches nothing (#55). Opened as a design track: a conductor inside Herdr that starts executor
  sessions in their own tabs and worktrees, watches them and calls the owner (#67, revisits A4).
  Until then the owner's coordination session uses the `orchestrate-project` skill and starts
  executors through Herdr by hand.

- **Roadmap in four stages with an owner check after each** (2026-09-30; roadmap #65). Stage 1
  #62, stage 2 #54, stage 3 #55, stage 4 #64 (rollout to other projects and the slice 2 spec). Each
  stage is one issue delivered in one or two PRs and ends with an owner check written in the issue;
  no intermediate tickets. This repository's own CI (fmt, clippy, tests on every PR) comes with
  stage 1; how `workbench` installs on a machine is decided in stage 3 before the first release; the
  rollout plan is made after stage 3.
- **Implementation order and granularity** (2026-09-30, after the map). Three large steps, each
  one or two PRs, without splitting into many small tickets: the command center slice 1 (#62) first,
  because it helps in every repository at once; then the `workbench` CLI (#54); then the default
  method content, smoke check, first release and migration of this repository (#55), whose `pr`
  skill writes the result-card sections #62 reads. #2 is closed as superseded by #1's spec and #55;
  #3 and #4 close when #62 ships. A spec the size of one PR is implemented straight from the spec,
  without `/to-tickets`.
- **The command center ships its own binary** (2026-09-30, amends F26). One Cargo workspace, two
  binaries: `workbench` installs the harness; the command center's Herdr plugin has its own binary
  and release tags, so it can be installed without the harness installer (ADR 0006).

- **The first slice: "see and check"** (CC11, #52). Our Herdr plugin v0.1: the binding agent →
  project, branch, PR, task; the sidebar tag and the "need you" counter; the Overview popup ("needs
  you", "since you last looked", a per-project list agent → task → PR → CI); the result card popup
  (the agent's PR body plus derived facts, with one key each to a "look first" file at the line in
  nvim, the diff, the PR and the app). Plus one harness change: the forked `pr` skill writes the
  card sections in a stable shape (`Result`, `Needs you`, `Look first` with `path:line — reason`,
  `How to try`) as part of the artifact contract. Later slices: the pipeline tree, tidy-up, the
  action menu with "start", pinned projects, the spec review flow, stale evidence, plain phrases for
  every label, the personal inbox, our own UI. Judged after a week: task, PR and CI visible for
  every live agent without lookup; results checked from the card rather than GitHub; at most three
  key presses from notification to a file at the line. The map's destination is reached; the owner
  starts `/to-spec` for this slice.

- **Adopt or build** (CC7, #48). We build only our Herdr plugin: the Overview and result-card
  popups, the action menu and "start", the sidebar tag, the tab-bar counter, tidy-up, and the
  binding agent ↔ branch ↔ worktree ↔ task derived from Herdr, git and GitHub. Everything else is
  adopted: Herdr for agents and notifications; nvim (the owner's LazyVim with codediff.nvim,
  render-markdown, snacks.image) for files and diffs; herdr-nvim for line remarks on code and
  documents back to the agent; `gh` for PRs and merge; the regular browser for the app and
  screenshots. Not used: herdr-radar (competes for the single agent view), annotate (document
  review moves to nvim and the spec PR; amends CC6), the owner's agent-context plugin and Hunk. The
  workspace setup installs only our plugin, pinned; the nvim configuration stays the owner's own and
  the command center only opens `nvim +N path` in the agent's worktree.

- **Labels stay as agents know them; the command center translates** (CC9, #50). The five triage
  roles, `acceptance:human` / `acceptance:auto` and `wayfinder:*` keep their strings and the
  mapping in `docs/agents/triage-labels.md`; the command center shows plain phrases built from
  labels and facts ("Ready for an agent", "Waits for you", …) and never raw labels; GitHub label
  descriptions are rewritten in plain language. No track labels (the track is derived: map → large,
  spec → medium, otherwise small) and no stage labels (computed node states). Colours by meaning,
  the same on GitHub and in the command center.
- **The command center lives in this repository** (CC10, #51) as a Herdr plugin in its own
  subdirectory (`herdr plugin install KirillSachkov/agent-workbench/<subdir>`), with its own release
  tags, in Rust with prebuilt binaries. It reads only the documented conventions and imports nothing
  from the harness's code (ADR 0006); the subdirectory can move out with its history if it becomes a
  product of its own.

- **The flow and the parts of the command center** (CC6, #47). One task runs: the agent finishes
  and Herdr notifies → one key opens the Overview popup → Enter opens the result card popup → from
  the card a file at the line in nvim, the diff in the agent's worktree, the PR on GitHub, the app
  and screenshots in the regular browser → remarks on diff lines or the report go back to the same
  agent and stay in the PR → accept by merging from the card with confirmation or on GitHub → the
  Overview offers to tidy up the worktree and branch. Before code the same flow runs on a spec (reviewed in
  nvim and in the spec PR, CC7). Parts: Herdr is the workspace; our Herdr plugin provides the Overview and result-card
  popups, the sidebar tag, the tab-bar counter, open-at-line and tidy-up; files and changes are read
  in the terminal in nvim (LazyVim) set up minimally as a viewer (research CC16, #60); remarks go
  back through herdr-nvim (CC7); the regular browser shows the app and
  screenshots; GitHub holds the PR and the merge; our own UI comes later.
- **The result card is the PR body plus derived facts** (CC4, #45). The agent writes into the PR
  through the forked `pr` skill: the result in one line, decisions needed, where to look first
  with reasons, criteria with evidence, what is not done, how to try it, screenshots. The command
  center derives CI, reviews, the head SHA and stale evidence, diff size, test changes, branch and
  worktree, the agent's status and the app's address. It is shown as a scrollable Herdr popup with
  few key hints (prototype A on `prototype/45-result-card`) and on GitHub as the PR.
- **Starting work from a task** (CC14, #58). One key on an item in the Overview opens an action
  menu; "start" opens a new Herdr tab with the agent the owner picks in a picker (Claude Code,
  Codex, another runtime), started in the project's directory with a first prompt such as "Work on
  issue #n" passed as a launch argument. The command center creates no branch or worktree: the
  harness's skills do that.
- **Human tasks of a project live in its tracker** (CC14, #58). Issues with a human role
  label and a view on them; no second tracker per project. The owner's personal tasks across
  projects, the personal agent and the practices across all work are a separate private effort;
  the command center owes it only an optional personal-inbox source and a one-key action that turns
  a task into a project issue and starts work.

- **The command center works without our harness** (CC8, #49; ADR
  `docs/adr/0006-harness-agnostic-command-center.md`). Harness and command center are two
  independent parts. The command center shows any folder where an agent runs and adds detail by
  level from the facts it finds: a folder with an agent (status, waits for you, notifications), a git
  repository (branch, worktrees, diff, orphaned worktrees, clean-up), a GitHub repository (PR, CI,
  reviews, linked issue by `Closes #n` or branch name, labels), and a harness that follows the
  documented conventions (stages, criteria and evidence, the spec → tickets tree). No support or
  adapter is required; another harness reaches the top level by following the conventions.
- **Overview in Herdr** (CC5, #46). A project is a git repository with its worktrees, found from
  where agents run plus optional pinned projects in local configuration; all projects at once with a
  filter. The sidebar stays minimal (status icon and a short tag), the tab bar shows a "need you"
  counter, and details live in a full-screen Overview popup opened with one key, with sections:
  since you last looked, needs you, the pipeline tree per project, tidy up, and the owner's task and
  project lists (CC14). herdr-projects is a source of ideas, not the base (its own orchestrator and
  task store conflict with A4 and G28).

- **Jobs of the command center** (CC12, #56). Fifteen jobs, all in scope, in four groups the first
  slice will order: attention (who waits for me now, what happened while I was away, notifications),
  checking and feedback (check a finished result, send remarks back to the same agent and keep them
  in the PR, review a spec or plan before code), map of work (which agent works where — agent, task,
  branch, worktree, PR — and orphaned worktrees; the pipeline state of a project; clean-up after
  merge), memory and spend (past results and artifacts, notes, tokens and provider limits). Starting
  work, jumps and notifications mostly exist in Herdr already and are adopted, not rebuilt. The graph
  of relations (spec → tickets → branches) belongs to the later own UI; for now a simple tree with
  states. The result card is a terminal view (a scrollable popup with few key hints, prototype A on
  the `prototype/45-result-card` branch) plus the full report in the PR; a local review page and a
  guided step-by-step check were rejected.
- **Layers and their order** (CC12, #56). Herdr is the workspace where agents work and is improved
  first and as far as possible; GitHub organises the harness and the workflows across projects;
  artifacts and files are organised so they are easy to read and to leave feedback on. Our own UI
  comes last: a web dashboard for observation, with quick jumps into Herdr, copyable commands and
  perhaps tasks and notes, canvas-like. This keeps the 2026-09-29 "TUI and web on one core" as a
  direction but moves the web part to the end.
- **Two trackers per project** (raised in CC12, #56; decided in CC14, #58). Besides the agents'
  tracker that follows the harness pipeline (G28 unchanged), the owner wants a tracker for human tasks
  per project, possibly one personal board across projects, and a one-click start of agent work from
  a human task. Research is CC13 (#57).
- **Destination of the map** (#41). The map ends when the owner can start `/to-spec` for the first
  slice of the command center: its jobs, its form and how the parts combine (Herdr plugin, own TUI,
  web, canvas), the harness seam, the home repository and the first useful slice are decided. Later
  layers such as a canvas overview stay a sketched direction, not a design.
- **Two jobs on par** (#41). The command center serves two jobs equally: checking an agent's result
  fast (report and artifacts, PR or commit, changed files at the right lines, branches and worktrees,
  the running app, screenshots) and an overview across projects (agents, tasks, how they relate).
  Launching and switching agents stays with Herdr and its plugins.
- **One effort for the command center and the workspace** (#41). The map covers both former outcome
  issues #3 (command center) and #4 (workspace: Herdr plugins, key bindings, setup); both stay open
  as outcome issues. C12 (result package) and G30 (label vocabulary) return as tickets of the map.
  The setup command itself is implementation; the map decides what it sets up.
- **The command center is single-user** (#41). One user on one machine: no team or multi-user
  features (shared boards, access control, a server for colleagues). It may rely on the owner's stack
  (Herdr, GitHub, macOS, their editor), but personal data such as the list of projects and paths
  lives in local configuration, never in code, since the repository is public. It stays adaptable to
  any harness. For the command center this replaces the 2026-09-29 requirement "must work for a
  team"; the harness itself stays team-ready. Team visibility stays in GitHub (PRs, issues, the
  result card in the PR).
- **The term is "command center"** (#41). One name for the whole layer around the harness — the
  owner's workspace in Herdr and the overview. "Shell", "dashboard" and "workspace" (a Herdr term)
  are avoided as names for it.
- **Out of scope of the map** (#41): team and multi-user features, mobile and remote access to
  sessions, trackers other than GitHub in the command center (G28 left this to #3), launching and
  orchestrating agents of our own, reading agents' internal session files.

## 2026-09-29 (harness requirements, wayfinder #5)

Decisions from the harness requirements map, one ticket per decision.

- **The artifact contract is the core, neutral to method** (A1, #6). agent-workbench defines its
  own artifact contract and stage model and ships a default method. A stage counts when its artifact
  passes the contract, whatever produced it. No adapters to spec-kit, OpenSpec or BMAD are built or
  maintained. Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §11.2.6, §11.3;
  ADR `docs/adr/0001-method-neutral-artifact-contract.md`. "Passes the contract" was amended by the
  conventions-over-checks principle below: the contract is followed and judged, not checked by code.
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
- **Evidence is bound to the PR head commit** (C8, #13). The candidate is the commit SHA at the head
  of the pull request; every evidence record carries the SHA it was produced on, as GitHub check runs
  already do with `head_sha`. When the head moves, older records stay visible but are marked stale;
  before acceptance every acceptance criterion needs evidence on the current head. Treated as cheap
  hygiene: the value of binding is argued, not measured. Evidence:
  `docs/research/2026-09-29-hoh-harness-design.md` §4.3, §9.1.3;
  `docs/research/2026-09-25-artifact-mechanics-classic.md` (check runs);
  `docs/research/2026-09-25-artifact-pipelines-academic.md` §7.2.
- **Evidence is recorded per criterion, from execution** (C9, #14). Every acceptance and preservation
  criterion gets a status: verified, failed or unverified (the default). A criterion with no record is
  unverified, as checked by the CLI, not claimed by the agent. A verified status needs at least one
  evidence record: type, reference, candidate SHA and a short observation. Record types: `ci` (check
  run), `command` (command with its real output), `test` (which test; for new behaviour, failing
  before and passing after), `screenshot`, `recording`, `review` (a non-author review report), `manual`
  (a human's note). Command output is captured by a tool that runs the command, never retyped by the
  agent. The schema is strict (JSON Schema): no tolerant parsing of "pass-like" words; the CLI checks
  that links resolve, files exist and SHAs match the current head. These records are the data behind
  the result card the owner opens; the card itself is ticket C12, now a prototype. Amended by the
  conventions-over-checks principle below: this is the documented shape agents follow; no schema
  validation or capture tool is enforced by the harness. Evidence:
  `docs/research/2026-09-29-hoh-harness-design.md` §4.2, §9.2.2;
  `docs/research/2026-09-27-agent-products-human-artifacts.md` §8;
  `docs/research/2026-09-27-reviewing-agent-work-practices.md` §1 (Showboat), §2, §6.
- **Conventions over checks: the harness's code enforces nothing** (principle raised while resolving
  C10, #15; the owner chose it over a hybrid with a minimal blocking set). Like Matt Pocock's skills,
  the pipeline rests on skills, conventions and worked examples. The harness's own code only collects
  facts for the command center and installs (agents read facts with `gh` and `git` themselves, C11); it validates nothing and
  blocks nothing. What still blocks is outside the harness: the project's own CI and the owner's merge.
  Amends A1 and C9 (the artifact contract and evidence are documented shapes with examples, not a
  schema checked by code), softens A3, B4, B7 and C8 into conventions, and replaces the brief's
  principle "what is mandatory is checked by code". Accepted trade-off: the research finds that agents
  over-report and prose-only rules decay; the owner prefers flexibility across models and runtimes
  and the freedom to deviate. ADR `docs/adr/0004-conventions-over-checks.md`.
- **Carry-over between sessions** (C10, #15). By default nothing is handed over: a new session reads
  the artifacts itself — the ticket (which must be self-contained), the spec, the code, the PR and the
  issue — and may ask the CLI for facts. A handoff is written only when a session stops with its work
  unfinished (context exhausted, repair limit reached, switching agent or person): by the agent as a
  convention of the completion step, or by the developer calling `handoff`. It is written with our
  fork of Matt Pocock's `handoff` skill as an issue comment (goal, state, what is verified, next step,
  done criterion, links instead of copies), not to a temp directory. Homes of facts: decisions in
  repository files, delivered work and evidence in the PR, task state and handoffs in the issue;
  nothing durable in CI artifacts or check output, which expire after 90 days. Evidence:
  `docs/research/2026-09-29-mattpocock-skills-design.md` §2.4;
  `docs/research/2026-09-27-artifact-review-surfaces.md` §6;
  `docs/research/2026-09-27-reporting-tooling-github.md` §0.
- **Completion lives in the forked skills, Matt Pocock's way** (C11, #16). No separate completion
  skill of ours, no CLI command, no hook. `implement` closes with `code-review`, then Matt Pocock's
  `pr` skill (beta upstream, adopted into our fork) writes the PR from the primary source (issue or
  spec) following the result-card shape of the artifact contract; unfinished work ends with
  `handoff` instead (C10). Agents get facts themselves with `gh` and `git`; skills do not depend on
  our CLI, which serves the command center and installation. Evidence:
  `docs/research/2026-09-29-mattpocock-skills-design.md` §2.3, §6, §7.
- **The harness executes no gates** (D12, #17). No CLI validators, no hooks and no CI step of the
  harness block anything (follows ADR 0004). Two things block, and both belong to the project: its own
  CI, made required through branch protection, and the owner's merge. Projects may add their own
  hooks; hooks for observation are ticket D15, and the 2026-09-27 order "hooks after the artifact
  contract" stands. (The example CI workflow first agreed here was withdrawn in D16: CI stays
  entirely with the project.) Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md`
  §11.3.2.
- **Acceptance mode is a label on the task** (D13, #18). How a result is accepted depends on the task
  and is set at intent approval as a tracker label: `acceptance:human` (the owner checks — interface,
  new functionality, anything risky) or `acceptance:auto` (accepted when the project's CI is green and
  a non-author review leaves no serious open findings; the agent may merge). No label means
  `acceptance:human`. An agent reviewer's verdict never accepts work by itself; `acceptance:auto` is
  the owner's decision made in advance. This extends Matt Pocock's model, where AFK work still ends in
  a PR for a human ("AFK defers review to the end") and a "dark factory" is only named. Project- or
  track-level rules and skipping the pipeline stay in D17; the overall label vocabulary is a new
  ticket. Evidence: `docs/research/2026-09-27-reviewing-agent-work-practices.md` §1 (tiered review,
  P×I×D); `docs/research/2026-09-25-artifact-pipelines-academic.md` §6.1;
  `docs/research/2026-09-25-artifact-pipelines-industry.md` §1.2;
  `docs/research/2026-09-29-mattpocock-skills-design.md` §2.5.
- **Human gates stay conventions in skills** (D14, #19). No human gate becomes a check in the harness's
  code: grilling waits for confirmation, `tdd` confirms seams, `to-tickets` waits for approval of the
  breakdown, and whether an agent may merge follows the acceptance-mode label (D13) read by the `pr`
  skill. A project that wants platform-level protection against merging can use GitHub branch
  protection with a required review; the harness documentation may mention it as an option, not in
  the built-in default, since it would block `acceptance:auto`.
- **Product focus: a well-designed pipeline and developer convenience, not constraints on agents**
  (stated by the owner while resolving D14, #19). agent-workbench assembles the best practices of Matt
  Pocock's pipeline, Harness of Harness, and Anthropic's and OpenAI's guidance into a pipeline of
  skills and conventions, and invests in making the work visible and understandable for the
  developer (the command center, result cards, clear labels). It does not build rigid wrappers,
  validators or restrictions around agents. Complements ADR 0004.
- **No hooks for now** (D15, #20). The harness ships no hooks. Enforcement hooks are ruled out by
  ADR 0004; observation is already covered — live sessions by Herdr and the runtimes' public
  interfaces, results by GitHub and git (2026-09-27: the owner needs final artifacts, not live
  watching). Hooks port poorly: formats and tool names differ, matchers are not portable, Kimi has
  user-level hooks only, OpenCode uses JS plugins. A hook is added later only for a concrete command
  center need that nothing else can serve, observation-only and fail-open. Projects may add their own.
  Evidence: `docs/research/2026-09-29-cross-runtime-portability.md` §5.
- **No CI in the harness, no agent in CI** (D16, #21). The harness ships nothing for CI — no
  workflow, no example, no AI-review job — because every repository has its own CI requirements.
  Non-author review is done in the session by default (`code-review` with fresh-context subagents)
  or by another agent; a project may add an AI reviewer to its own CI if it wants. Withdraws the
  example CI workflow from D12. Evidence: `docs/research/2026-09-29-cross-runtime-portability.md` §8;
  `docs/research/2026-09-27-agent-products-human-artifacts.md` §8.
- **Runtime tiers: Claude Code, Codex and OpenCode are tier 1** (E17, #22). For a runtime, harness
  support means: it reads the project instructions (`AGENTS.md`, bridged for Claude Code by
  `CLAUDE.md`), discovers the skills, and honours the invocation policy. Tier 1 is verified before
  every harness release. Every other runtime (Cursor, Copilot, Gemini CLI, Kimi and others) is tier 2:
  it gets the same files in the standard locations (`AGENTS.md`, `.agents/skills`) with no guarantee
  and with its limits listed explicitly (for example Gemini CLI has no per-skill user-only field). A
  runtime moves to tier 1 once it is actually used and passes the same proof. Proof of support is a
  smoke check in a fresh session: the agent sees the project instructions, lists the model-invoked
  skills, does not start user-invoked skills on its own and runs one when called by name; it can be
  automated in this repository's own CI through headless modes (`claude -p`, `codex exec`,
  `opencode run`). Open point for E20: OpenCode expresses user-only skills only through `permission`
  rules on its skill tool. Evidence: `docs/research/2026-09-29-cross-runtime-portability.md` §1–3, §8.
- **No role files for now: one agent, behaviour set by the skill** (E21, #26). Roles are not declared
  as data and no subagent definition files are shipped. One agent works in a session and behaves
  according to the skill it follows; skills may start ordinary fresh-context subagents (as Matt
  Pocock's `code-review` does), which already gives non-author review in Claude Code, Codex and
  OpenCode. "The reviewer does not edit" and "one writer per task" are conventions; the command
  center may show two agents on one branch. Revisit role files — neutral Markdown rendered into
  `.claude/agents`, `.codex/agents`, `.opencode/agents` with a native read-only field — if reviewers
  start editing code in practice or when an orchestrator (A4) needs roles as data, as Harness of
  Harness does. Evidence: `docs/research/2026-09-29-cross-runtime-portability.md` §6;
  `docs/research/2026-09-29-hoh-harness-design.md` §2.1, §3.
- **Any tracker, configured by the agent at setup** (G28, #33). As with Matt Pocock's skills, the
  harness works with any tracker: at setup an agent (the setup skill) asks which tracker the project
  uses and writes `docs/agents/issue-tracker.md` describing its operations; skills go through that
  file and stay tracker-agnostic. The command center is code and cannot follow prose, so its first
  version reads facts from GitHub only, behind one fact-source seam; support for other trackers there
  is decided in #3. Homes of facts: the issue and its labels (task, track, acceptance mode, status),
  the PR (delivered work, evidence, result card, acceptance by merge), issue comments (handoffs),
  repository files (lasting decisions, ADRs). Intent approval is the owner's `ready-for-agent`-style
  label on a small-track issue or the merge of the spec PR on a medium track. Owner decisions are
  recognised by the actor of GitHub events (who labelled, merged or approved), so no store of our
  own is needed. Evidence: `docs/research/2026-09-29-mattpocock-skills-design.md` §4;
  `docs/research/2026-09-29-harness-frameworks-compared.md` §3.5, §11.3.1, §11.3.6.
- **Tracker operations are prose templates, Matt Pocock's way** (G29, #34). Skills perform tracker
  operations through `docs/agents/issue-tracker.md`, written by the setup skill for the project's
  tracker; there is no CLI of ours in between. Our fork fixes the known template bugs of upstream:
  `gh issue view --comments` combined with `jq` (#733), comments output without the issue body
  (#964), `issue_dependencies_summary` as an invalid `--json` field (#1118), and the blocking recipe
  hidden in the wayfinder section (#855) — verified `gh api` / `--json` commands and the blocking
  recipe in a shared section. Evidence: `docs/research/2026-09-29-mattpocock-skills-design.md` §4.
- **Standard files are the source; `AGENTS.md` belongs to the project** (E18, #23; research R1, #40).
  No neutral format of our own. `AGENTS.md` and `SKILL.md` are the source files people edit. For
  `AGENTS.md` the harness only: creates a minimal skeleton when none exists and never overwrites an
  existing file; owns one clearly marked managed block with pointers to its skills and pipeline;
  wires the one-line `@AGENTS.md` bridge in `CLAUDE.md`; and ships a skill for editing `AGENTS.md`
  instead of generating its content. Skills live in one canonical directory, `.agents/skills`, with
  links for Claude Code in `.claude/skills` (link strategy and Windows: E19). A `sync` command
  produces only the derived pieces: skill links, Codex `agents/openai.yaml`, OpenCode permission
  rules and the managed block; a stale derived file is reported, not blocked (ADR 0004). This matches
  how popular repositories and `vercel-labs/skills` already lay things out. Evidence:
  `docs/research/2026-09-29-agent-instructions-in-repos.md` §0, §1.3, §1.6, §2, §4;
  `docs/research/2026-09-29-cross-runtime-portability.md` §10.
- **Skills reach Claude Code through a committed directory symlink; Windows is best effort**
  (E19, #24; the owner asked for the usual practice). A relative symlink `.claude/skills` →
  `../.agents/skills` is committed, as next.js, sentry, PostHog and supabase do; a project that needs
  Claude-only skills may switch to per-skill links, which `sync` supports. No copies per runtime.
  Windows is not a first-version target: following `vercel-labs/skills`, `sync` on Windows creates a
  junction or a local uncommitted copy and warns when a committed link arrived as a text stub (the
  failure n8n documents). The smoke check (E17) must confirm how OpenCode treats a skill it sees in
  both `.claude/skills` and `.agents/skills`. Evidence:
  `docs/research/2026-09-29-agent-instructions-in-repos.md` §2.2, §2.4;
  `docs/research/2026-09-29-cross-runtime-portability.md` §3.2, §10.4.
- **The invocation policy lives in `SKILL.md` frontmatter** (E20, #25). `disable-model-invocation:
  true` marks a user-invoked skill; it is the one field several runtimes read (Claude Code, Cursor,
  Copilot in VS Code, Zed, Kimi). `sync` renders it into Codex `agents/openai.yaml`
  (`policy.allow_implicit_invocation: false`) and an OpenCode `permission` rule on the skill tool;
  the smoke check (E17) confirms the skill stays callable by name in OpenCode. A skill registry is
  generated from frontmatter as an index for people and the router, never a source. Gemini CLI has no
  per-skill field and is listed with the limit "user-invoked skills are visible to the model". This
  removes the hand-maintained parity bugs seen upstream (`writing-for-agents` invisible to Codex until
  1.2.2). Evidence: `docs/research/2026-09-29-cross-runtime-portability.md` §3.3;
  `docs/research/2026-09-29-mattpocock-skills-design.md` §3.
- **One distribution channel: our binary writes the harness into the project** (F22, #27; the owner
  asked for the usual practice). The most common way harnesses ship is their own CLI that writes files
  into the project (spec-kit, OpenSpec, BMAD 6.12, GSD, Ruflo, Agent OS; 10 of 19 harnesses have an
  installer, 5 package plugins). The `workbench` binary — which exists anyway for the command center
  — installs, updates and syncs the harness in a repository; skills and harness files are committed,
  so teammates get the pipeline through `git clone` without installing anything into their agents.
  No plugin channel, to avoid the two-path trap (GSD, Ruflo, Matt Pocock's plugin vs skills.sh
  drift). The layout stays compatible with `vercel-labs/skills` (`.agents/skills`, links, a lock) so
  third-party skills can still be added with `npx skills add`. How the binary itself is installed on
  a machine is #4. Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §9.1, §11.2.3;
  `docs/research/2026-09-29-agent-instructions-in-repos.md` §2.3–2.4, §3.2;
  `docs/research/2026-09-29-cross-runtime-portability.md` §4.2.
- **Everything agents need is committed, derived files and lock included** (F23, #28). So that
  `git clone` is enough: `.agents/skills`, the `.claude/skills` symlink, the Codex and OpenCode
  invocation files produced by `sync`, the `CLAUDE.md` bridge, the project's `AGENTS.md` with the
  harness's managed block, the project configuration, `docs/agents/*`, and a lock recording the
  harness version, the upstream commit of every forked skill (A2) and file hashes (like
  `skills-lock.json`). Machine-specific things stay out: secrets (environment) and local copies that
  replace links on Windows. Rejected: committing only configuration and lock with generation on each
  machine (1 of 46 sampled repositories does that). Evidence:
  `docs/research/2026-09-29-agent-instructions-in-repos.md` §2.2–2.4;
  `docs/research/2026-09-29-harness-frameworks-compared.md` §9.1.
- **The harness is a forkable repository; updates are three-way merges** (F24, #29). Anyone makes
  their own harness by forking agent-workbench or creating a repository from it as a template, and
  edits anything. Upstream changes reach a fork through plain git (`git merge upstream/main`, GitHub
  "Sync fork"). A fork — or agent-workbench itself by default — is installed into projects with
  `workbench init --from <repo>@<ref>`, recorded in the harness lock; `workbench update` brings a new
  version into a project on its own branch and pull request: files the project did not touch are
  replaced, files it edited are merged three-way against the base recorded in the lock, conflicts are
  ordinary git conflicts (the fork's `resolving-merge-conflicts` skill helps), and a rename/removal map
  in each release retires old skills. The same model runs on every level: Matt Pocock's skills → our
  harness → a user's fork → projects. A fork is also how a person or team gets "their own variant"
  without a profile system (B5). ADR `docs/adr/0005-forkable-harness-and-three-way-updates.md`.
  Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §3.1, §6.1, §9.1, §11.1.1–2,
  §11.1.9; `docs/research/2026-09-29-mattpocock-skills-design.md` §3.
- **Version skew: content is shared through git; the lock guards only the tool** (F25, #30). Since
  everything agents need is committed (F23), every teammate who pulls has the same harness content,
  whatever binary they run. The harness lock records the harness source, its version and the minimum
  `workbench` binary version; an older binary refuses to write harness files (`sync`, `update`) and
  asks to be upgraded, to avoid corrupting derived files. Agents and the pipeline are never blocked
  (ADR 0004); the command center shows each project's harness version. Evidence:
  `docs/research/2026-09-29-harness-frameworks-compared.md` §11.3.4.
- **The machine gets only the binary; user-level agent configs are left alone** (F26, #31). The
  machine install is the `workbench` binary (CLI, TUI, the command center's web UI) and its own data,
  such as the list of watched projects. The harness writes nothing into `~/.claude`, `~/.codex` or
  `~/.config/opencode` — no user-level skills, hooks or instructions — so an agent behaves the same
  for every teammate in a project; personal preferences stay the user's own (B6), as in Matt Pocock's
  setup, which has no user-level mode. Workspace setup from #4 (Herdr plugins, key bindings, status
  plugins) is a separate explicit command that shows what it will change and asks for confirmation.
  Evidence: `docs/research/2026-09-29-mattpocock-skills-design.md` §2.7;
  `docs/research/2026-09-29-cross-runtime-portability.md` §5.1, §10.4.
- **Release channels are refs; beta skills are opt-in** (F27, #32). The stable channel is semver git
  tags of the harness source, pinned in the harness lock and offered by `update`. Following `@main` is
  allowed as a deliberate, recorded choice, which avoids upstream's drift between pinned plugin users
  and skills.sh users pulling `main`. Beta skills live in a separate directory of the harness
  repository (like Matt Pocock's `in-progress/`), are not installed by default, and are enabled one by
  one in the project configuration. Promotion is a convention: the skill has been used in a real
  project, its description and the router are updated, and the changelog records it. Every release
  carries a changelog and the rename/removal map (F24). Evidence:
  `docs/research/2026-09-29-mattpocock-skills-design.md` §3.
- **No orchestrator of our own** (A4, #38). The owner does not want a heavy orchestration system; the
  brief's exclusion of launching and orchestrating agents stands. The harness stays compatible with
  external runners the owner starts (Matt Pocock's `implement-spec`, a Ralph loop, others): tickets
  with blocking edges and the ready nodes of the artifact graph (ADR 0003) are the queue they consume.
  Role files and roles as data (E21) return only if such a runner needs them.
- **Deviations from the pipeline are visible facts, not permissions** (D17, #37). Since the harness
  enforces nothing (ADR 0004), a deviation is recorded rather than allowed: per task, the
  `acceptance:auto` label (D13); per project, a default acceptance mode in the project configuration;
  work outside the pipeline is simply a PR with no linked task or spec, which the command center shows
  as "outside the pipeline".
- **The shell around the harness is the next effort** (closing the map, #5). With the harness
  requirements settled, the owner's next focus is a flexible shell for working with agents —
  terminal-first and built on Herdr, possibly with a canvas-like overview — to check agents' results
  quickly: open artifacts and changed files, see branches and worktrees. It must stay light (no
  monolithic agent platforms) and adaptable to any harness. Tickets C12 (result package) and G30
  (label vocabulary) move to that effort; the handoff is on #3.

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
