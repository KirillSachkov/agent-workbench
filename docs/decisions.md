# Decisions

Owner decisions, newest first. Each entry has a date, the decision and where it came from.

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
