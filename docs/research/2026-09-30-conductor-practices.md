# Conductor practices: coordinator/executor setups for coding agents (state as of 2026-09-30)

Date: 2026-09-30. Question 1 of #67 ("Conductor in Herdr: launch agent sessions, watch them, call
the owner"): what are the best practices and failure modes of coordinator/executor setups for
coding agents — how they launch executors, watch them, gate merges, keep parallel lanes from
colliding, limit cost and runaway loops, and hand control back to a human — and what changed
recently in howdeploy/CanvasTTY, and is forking it for a canvas view of lanes viable? Question 2
(the conductor's scope, home, owner channel and the amendment to A4) is the owner's decision and is
not answered here.

This note does not repeat [herdr-plugins-built](2026-09-30-herdr-plugins-built.md) (how herdr-projects,
radar and other plugins are built; its pane-token, identity and forge-text patterns) or
[canvas-workspaces](2026-09-29-canvas-workspaces.md) §1 (CanvasTTY's canvas, Home zone, plugin SDK
as of 2026-09-29). It adds the coordinator side of herdr-projects, the other tools, the vendors'
guidance, and CanvasTTY's changes since then.

Method.

- **Repositories** were read through `gh api` at pinned commits (table below); issues through
  `gh issue view` / `gh api`. Stars and last push are from `gh api repos/<repo>` on 2026-09-30.
- **Docs** were fetched from the vendors' own sites: code.claude.com/docs (Markdown versions),
  learn.chatgpt.com/docs (developers.openai.com/codex/* now 308-redirects there),
  openai.github.io/openai-agents-python, conductor.build/docs, anthropic.com/engineering,
  aihero.dev, ghuntley.com.
- **Herdr**: the local `herdr 0.9.1` (protocol 22, schema version 1), read only through
  `herdr --skill`, the command-group usage (`herdr agent`, `herdr tab`, `herdr worktree`,
  `herdr notification`, `herdr pane`) and `herdr api schema --json`; release notes of 0.9.2/0.9.3
  from GitHub. Nothing in Herdr was changed and no agent was prompted.
- **CanvasTTY**: releases, commits since 2026-09-01, ADRs and source at `15983c4`; the source
  tarball was unpacked to a scratch directory and counted with `tokei`. The app was not installed or
  run.
- Three sub-reads ran in parallel (external tools; vendor guidance; CanvasTTY). Their key quotes were
  spot-checked against the source: the Claude Code Stop-hook cap, the 7× team cost and the
  file-overwrite warning; Pocock's `implement-spec` doc; herdr-projects' `COORDINATOR.md`;
  Agent Orchestrator issue #6032. Other quotes were not re-checked individually.
- "Unconfirmed" marks what the sources did not settle. "Inference" marks a conclusion drawn from two
  sources rather than stated by one.

Citation shorthand (`<P>/<path>` means `<prefix><path>`):

| Short | Repository or site | Pinned at | ★ | License |
|---|---|---|---|---|
| H | https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/ | `4e4548c` (0.2.34, 2026-09-28) | 527 | MIT |
| A | https://github.com/Untrivial-ai/agent-orchestrator/blob/8de638ad9b2d71bd7bf2220deec27dcd92278cea/ | `8de638a` | 12,557 | Apache-2.0 |
| S | https://github.com/superset-sh/superset/blob/7886c5adf0301f79b867cc52b48192b32ec4e1ec/ | `7886c5a` | 14,763 | Elastic License 2.0 (GitHub shows NOASSERTION) |
| V | https://github.com/BloopAI/vibe-kanban/blob/d5cbb5380fa0b32e98ef9b8d987f63decce4be3a/ | `d5cbb53` | 28,220 | Apache-2.0 |
| C | https://www.conductor.build/docs/ | 2026-09-30 | — | proprietary (issues in meltylabs/conductor-releases) |
| M | https://github.com/mattpocock/skills/blob/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/ | `d81f3a1` | — | MIT |
| CC | https://github.com/anthropics/claude-code/blob/732e167ee9d71296b4b63d6f529ac1334513826a/ | `732e167` | — | — |
| T | https://github.com/howdeploy/CanvasTTY/blob/15983c4f4a7832e3f18ae3851a17ea2cf65662ce/ | `15983c4` (2026-09-29) | 91 | MIT |
| D | https://code.claude.com/docs/en/ | 2026-09-30 | — | — |
| HS | `herdr --skill` output of herdr 0.9.1 (the agent skill bundled in the binary) | 0.9.1 | — | Apache-2.0 (herdrdev/herdr) |
| HA | `herdr api schema --json` of herdr 0.9.1 | protocol 22 | — | — |

---

## 1. What Herdr itself offers a conductor

- **Launch.** `herdr tab create [--cwd PATH] [--label TEXT] [--env KEY=VALUE] [--no-focus]` returns
  the new tab and its root pane; `herdr worktree create [--branch NAME] [--base REF] [--label TEXT]
  [--no-focus]` creates a git worktree and opens it as a workspace; `herdr agent start <name> --kind
  KIND --pane ID [--timeout MS] [-- <agent-args...>]` starts a supported agent in an existing shell
  pane and "returns only after Herdr detects the expected agent in the same pane and considers it
  ready for interactive input" (HS "Start and coordinate an agent"; command-group usage). 0.9.1
  lists 24 agent kinds, including `claude`, `codex`, `opencode`, `gemini`, `cursor` and `copilot`
  (`herdr agent` usage). The bundled skill's default is a sibling pane: "Do not create a workspace,
  tab, worktree, or different cwd unless the user explicitly requests that topology or location"
  (HS).
- **Prompt and wait.** `herdr agent prompt <target> <text> [--wait] [--until STATUS] [--timeout MS]`
  sends text plus Enter as one submission. With `--wait` it returns on the first settled `idle`,
  `done` or `blocked`, or `agent_prompt_stalled` if no `working`/`blocked` activity appears within
  five seconds. It refuses an agent already at an approval or question dialog with `agent_blocked`,
  and the skill says "Inspect the blocked UI and ask the user before answering it". "A timeout or
  stalled response does not prove the prompt was never delivered; do not blindly submit it again"
  (HS).
- **States.** `idle`, `working`, `blocked` (Herdr recognised an approval or question UI), `done`
  (finished, not yet seen) and `unknown` ("does not prove completion") (HS; HA `AgentStatus`). They
  come from per-agent hook integrations (`herdr integration install`) or from an agent reporting
  itself with `herdr pane report-agent`; 0.9.2 added self-reported resume commands so Herdr reopens
  an agent's exact session after a server restart ([v0.9.2 notes](https://github.com/herdrdev/herdr/releases/tag/v0.9.2), #4687).
- **Events.** `events.subscribe` streams typed events including `pane.agent_status_changed` (pane,
  workspace, agent, status), `pane.output_matched`, `pane.exited`, `tab.created`/`tab.closed` and
  `worktree.created`/`worktree.removed`; `events.wait` blocks for one matching event (HA). Since 0.9.2
  a reader that falls behind gets `events_lost` "instead of silently skipping events" (v0.9.2 notes,
  #4178). 0.9.2 also fixed "An agent's first task now counts as done, even when it started with a
  prompt … no longer fire false done notifications" (#3338).
- **Calling the human.** `herdr notification show <title> [--body TEXT] [--position …] [--sound
  none|done|request]` is the whole notification surface: a toast inside Herdr with an optional
  sound (usage; HA `NotificationShowParams`). There is no reply channel; the human answers by
  focusing the agent.
- **What is absent.** The 129 method and event names in HA contain no queue, dependency graph,
  budget, iteration cap, merge or PR operation. Those belong to whoever drives Herdr.
- **Versions.** Upstream released 0.9.2 and the hotfix 0.9.3 on 2026-09-29
  ([releases](https://github.com/herdrdev/herdr/releases)); the local 0.9.1 predates both.

## 2. herdr-projects: an agent coordinator inside Herdr

herdr-projects' coordinator is an ordinary agent session that follows `skill/COORDINATOR.md`; the
binary does the deterministic mechanics through the Herdr CLI and the plugin code decides nothing
(H/docs/operations.md "How it works"). No commits after `4e4548c` on 2026-09-30.

- **Launch.** The coordinator runs `hp thread start <slug> --title … --repo … --task-file -`; the task
  is written "for an agent that has not seen the conversation" (H/skill/COORDINATOR.md L44–58). The
  binary calls `herdr worktree create --cwd <repo> --branch hp/<slug>/<id>-<title> --base <base>
  --label … --no-focus` (H/src/herdr.rs L374; H/src/thread.rs L235), base `origin/HEAD` or else the
  current HEAD (H/src/threads.rs L249–258). The agent is chosen only by the name of a profile the
  user created; the coordinator cannot pass launch flags (COORDINATOR.md L55–56). The ticker starts
  the agent and sends the brief when the input box has settled empty; a brief counts as delivered
  only when the agent starts working, and after three unconfirmed attempts it becomes an inbox item
  (COORDINATOR.md L118; operations.md "Commands").
- **Watch.** Herdr's agent state; self-reports (`herdr-projects report --percent N --activity …`,
  kept five minutes, with hooks for Claude Code, Codex, Droid, Gemini and Copilot that remind the
  agent); and a `report.md` whose first line is `PR: <url>` followed by `## Report`, `## Next` and
  optional `## Remember` (H/skill/THREAD.md). The ticker runs every 15 s, PRs every 2 min; groups are
  needs you (blocked > 30 s) → review → landing → working → idle (operations.md "Groups").
- **Merge gate.** The coordinator may never "Merge, force-push, delete branches, remove worktrees,
  resolve threads, delete or archive the project" unless asked in chat (COORDINATOR.md L147). A
  report's `## Next` line such as "merge the PR" is forwarded by the user with `hp thread next`, and
  "the thread does it itself with its own tools. … You never execute a Next line yourself"
  (COORDINATOR.md L72). A `pr-followup` routine fires on `checks-failed` and `review` and prompts the
  thread to fix CI and answer comments; the binary passes only counts, never GitHub text
  (operations.md "Routines").
- **Conflicts.** One worktree and branch per thread; one owner per area — follow-ups go to the thread
  already working there (COORDINATOR.md "Follow-ups"); `TASKS.md` is written only by the coordinator,
  and `threads/`, `inbox/`, `.state/` only by the binary (L76, L132). No merge order, rebase or
  conflict handling exists in the code (unconfirmed absence).
- **Cost and loops.** `max_parallel_threads` (default 10) is a rule for the coordinator; the binary
  only warns (H/src/threads.rs L177–183). Briefs are retried at most three times; routine commands
  time out after 60 s. No budget; the docs say each thread is a full agent session and each nudge
  costs coordinator tokens (operations.md "What the safety settings don't stop").
- **Hand-back.** One Herdr notification per event (needs-you with the `request` sound; report or
  merged with `done`) and a one-line `[hp inbox] …` nudge into the coordinator's pane only when it
  has been idle 60 s and its input empty 10 s (operations.md "Nudges"). Inbox and ticker text "approves
  nothing"; "Only the user decides" on tasks (COORDINATOR.md L23–25, L82). In
  `start_threads=propose` mode a thread needs an explicit go-ahead. Permission prompts: approve once
  only inside the thread's task and worktree; pushing, merging, credentials, installing software go
  to the user; "Never pick 'always allow'" (L113).
- **Failure modes.** A brief was counted as delivered from `herdr agent prompt`'s exit code while
  remote delivery took 150–321 s (issue #70, closed; now `--wait --until working`, H/src/herdr.rs
  L458); an adopted Claude pane never receives its brief (#85, open); only the first of two projects
  on one machine is polled (#80, open); a nudge merged with the user's half-typed text (#60); a new
  worktree shows Claude's trust dialog; pane ids are reused after a Herdr restart
  (H/docs/herdr-notes.md stage 3).

## 3. Agent Orchestrator (Untrivial-ai)

The same GitHub repository (id 1156994049) as ComposioHQ/agent-orchestrator after organisation
renames, not a fork; npm packages moved from `@composio` to `@aoagents`
(A/frontend/src/docs/content/changelog.mdx). The current code is a Go daemon with Electron and
SQLite; the older TypeScript line with YAML `reactions:` and auto-merge is frozen at 0.10.0
(A/frontend/src/docs/content/migration.mdx).

- **Launch.** A worktree per session on branch `ao/<session>/root`
  (A/frontend/src/docs/content/guides/parallel-issues.mdx); a native PTY host, tmux for old handles
  (A/backend/internal/adapters/runtime/runtimeselect/runtimeselect.go); a persistent orchestrator
  agent that only plans and starts workers with `ao spawn` (A/backend/internal/session_manager/prompt.go).
- **Watch.** Agent hooks (SessionStart, Pre/PostToolUse, PermissionRequest, Stop, SubagentStop)
  written into `.claude/settings.local.json` (A/backend/internal/adapters/agent/claudecode/hooks.go);
  screen reading for agents without hooks; `ao report --checkpoint/--needs-input`; a reaper every 5 s
  and SCM polling every 30 s with ETags; status is computed from facts on read (A/docs/architecture.md).
- **Merge gate.** A human merges (button or `ao pr merge`); before merging SCM is re-read and the PR
  must be non-draft, CI green on the current head, no changes requested or unresolved threads,
  mergeable, and the head SHA equal to what the user saw; always squash
  (A/backend/internal/service/pr/action_service.go; A/backend/internal/domain/pr.go).
- **Conflicts.** On a conflict the PR's owner session gets an urgent rebase nudge, suppressed for a
  stacked PR over an open parent (A/backend/internal/lifecycle/reactions.go). No file ownership or
  merge order (unconfirmed absence).
- **Cost and loops.** Nudges are de-duplicated by signature; review nudges at most 3; auto-review at
  most 3 per SHA (A/backend/internal/autoreview/coordinator.go). No session or spend cap (issues
  #2344, #4673 "spend caps" RFC).
- **Hand-back.** `needs_input`, `ready_to_merge`, `pr_merged` notifications as desktop toasts and
  mobile push (A/backend/internal/domain/notification.go); a "Needs you" column; automation does not
  touch a `blocked` session.
- **Failure modes.** "No way to stop a worker stuck in an infinite loop": a Claude Code worker
  "burned ~45M tokens (~$1.5) over 45 minutes while looping", because `ao send` waits for the next
  turn boundary and status checks "piled up in the worker's input queue" (issue #6032, open).
  Lifecycle events do not wake the orchestrator (#4951); a nudge marked delivered while stuck behind
  a hung turn (#5833); an agent took its own review comment as a task (#5574); a reviewer ↔ worker
  loop over broken output (#3171); a merge arrived while the agent was still working (#2879); a race
  on `git worktree add` (#4350); the orchestrator removed its own worktree (#5598); a message merged
  with the operator's draft (#5711).

## 4. Conductor, Superset, Vibe Kanban

**Conductor** (closed source, macOS).

- Launch: a worktree per workspace from a fresh `origin/<base>`; one branch in one workspace only
  (C/concepts/git-worktrees); setup/run/archive scripts in `.conductor/settings.toml`, each workspace
  gets a block of ten ports in `CONDUCTOR_PORT` (C/reference/scripts). Agents: Claude Code through
  the Agent SDK, Codex, Cursor, OpenCode.
- Watch: an API exposes idle/working/errored and transcripts (C/api); how status is obtained is not
  documented (unconfirmed).
- Merge: "Create PR" delegates the PR to the agent; a human merges; the Checks tab "may block or
  discourage merge" (C/reference/checks); `git.archive_on_merge`. "Resolve merge conflicts" hands the
  conflict to the agent (changelog 0.12.1). The docs warn against several chats in one workspace
  (C/reference/checkpoints). A checkpoint is a git ref per turn.
- Limits: no caps or budgets; idle processes are killed after 30 minutes
  (meltylabs/conductor-releases#27); a stray `ANTHROPIC_API_KEY` silently moves billing to the API
  (C/reference/harnesses/claude-code).
- Failure modes: a question arriving during the idle kill becomes a zombie "AWAITING RESPONSE" (#27);
  a silent session restart kills background tasks (#21); a merged workspace returns to "In Progress"
  after restart (#34).

**Superset** (Electron app).

- Launch: a worktree per workspace (S/apps/docs/content/docs/workspaces.mdx); setup/teardown/run in
  `.superset/config.json`; a CLI for orchestration, `superset workspaces create` and
  `superset agents create --prompt` (S/apps/docs/content/docs/orchestration.mdx).
- Watch: hooks and wrappers normalise vendor events into Start, PermissionRequest, Stop
  (S/apps/desktop/src/main/lib/notifications/map-event-type.ts). Its orchestrate skill reads terminal
  snapshots and `SUPERSET_WORKER_DONE` / `SUPERSET_WORKER_BLOCKED` envelopes, which it calls "a prompt
  convention… not durable events" (S/plugins/superset/skills/orchestrate/SKILL.md).
- Merge: a human presses Merge, which runs `gh pr merge`
  (S/apps/desktop/src/lib/trpc/routers/changes/utils/merge-pull-request.ts); the coordinator checks
  workers' claims; worker branches are never merged automatically.
- Conflicts: split by files; a rolling fan-out of two or three slices
  (S/apps/docs/content/docs/recipes/fan-out-refactor.mdx).
- Limits: none — "the limit is your review attention"
  (S/apps/docs/content/docs/recipes/parallel-workstreams.mdx); the skill's only rule is to stop after
  repeated failures.
- Failure modes: a subagent finishing marks the whole terminal finished (#6641); "idle" while the
  agent waits for subagents (#7395); Cursor reports done on every turn (#5259); hooks fire outside
  Superset (#5531).

**Vibe Kanban** (sunsetting: the README says so and Bloop shut down on 2026-04-10,
[blog](https://www.vibekanban.com/blog/shutdown); maintenance commits only since 2026-09-15).

- Launch: a worktree per repository in a workspace
  (V/crates/worktree-manager/src/worktree_manager.rs L81–150); default profiles skip all permissions
  (V/crates/executors/default_profiles.json).
- Watch: process exit polled every 250 ms; Claude has no exit signal
  (V/crates/executors/src/executors/claude.rs L708), hence sessions stuck in "running" forever (#2495).
  A Stop hook refuses to stop with uncommitted changes
  (V/crates/executors/src/executors/claude/client.rs L329–344).
- Merge: a human's "Direct merge" squashes locally and refuses on divergence, staged changes in
  target, conflicts or an open PR, but **runs no tests or CI** (V/crates/git/src/lib.rs L575–670);
  or a PR polled every 60 s (V/crates/services/src/services/pr_monitor.rs).
- Conflicts: `rebase --onto` with abort/continue and a conflict-file list (lib.rs L1129–1333), and
  "Resolve Conflicts" hands instructions to the agent. No overlap detection across lanes
  (unconfirmed).
- Limits: none found (unconfirmed); approvals expire after 10 h as a denial
  (V/crates/utils/src/approvals.rs L6); worktree GC after 72 h idle.
- Failure modes: the Stop hook fired while waiting for subagents and made dummy commits (#2783); its
  hooks override the project's hooks (#3327); ~15 GB memory with parallel work (#890); direct merge
  fails silently when the target is checked out in another worktree (#1897).

## 5. Claude Code: subagents, background sessions, agent teams, loops

- **Subagents** ([D/sub-agents](https://code.claude.com/docs/en/sub-agents)): Markdown definitions
  with `maxTurns`, `model`, `permissionMode`, `background`, `isolation: worktree` and
  `tools: Agent(...)` limiting whom they may spawn; nesting "up to three layers below the main
  conversation"; at 20 concurrent subagents a spawn "fails with `Concurrent subagent limit reached`,
  and the error tells Claude not to retry".
- **Background sessions and agent view** ([D/agent-view](https://code.claude.com/docs/en/agent-view),
  research preview): `claude --bg "task"`, `/bg` or the dispatch box in `claude agents`; a supervisor
  daemon hosts them; states Working, Needs input, Idle, Completed, Failed, Stopped;
  `claude agents --json`, `claude logs`, `claude attach`, `claude stop`; PR badges by CI and merge
  state. "Detaching never stops a background session." Background sessions enter a worktree "before
  editing files … so parallel sessions can read the same checkout but each writes to its own".
  "Running ten agents in parallel uses quota roughly ten times as fast as running one."
- **Cloud** ([D/claude-code-on-the-web](https://code.claude.com/docs/en/claude-code-on-the-web)):
  `claude --cloud "task"` clones the remote at the current branch ("push first"); auto-fix watches a
  PR's CI and review comments, but "GitHub does not emit a webhook when the base branch advances and
  creates a merge conflict, so auto-fix can't react to conflicts on its own".
- **Agent teams** ([D/agent-teams](https://code.claude.com/docs/en/agent-teams)): "experimental and
  disabled by default" (`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`); a lead, teammates, a shared task
  list with file-locked claiming and per-agent mailboxes; interactive only. "Two teammates editing
  the same file leads to overwrites. Break the work so each teammate owns a different set of files."
  "Start with 3-5 teammates … Three focused teammates often outperform five scattered ones." Stated
  limitations: task status can lag and block dependents; the lead "starts implementing tasks itself
  instead of waiting"; teammates stop on errors; no resumption of in-process teammates; "Letting a team
  run unattended for too long increases the risk of wasted effort". A teammate's plan-approval
  request is approved automatically by the lead. Cost: "approximately 7x more tokens than standard
  sessions when teammates run in plan mode" ([D/costs](https://code.claude.com/docs/en/costs)).
- **Hooks as gates** ([D/hooks](https://code.claude.com/docs/en/hooks)): Stop/SubagentStop can block
  and keep the agent working, but "Claude Code applies an 8-consecutive-continuation cap … overrides
  the next block and ends the turn" (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`); `TaskCompleted` exit 2 keeps
  a team task open with the stderr fed back (the example runs `npm test`); `TeammateIdle` exit 2
  keeps a teammate working. Notification matchers include `permission_prompt`, `idle_prompt`,
  `agent_needs_input`, `agent_completed`.
- **`/goal`** ([D/goal](https://code.claude.com/docs/en/goal)): a session-scoped prompt-based Stop
  hook whose evaluator says met / not yet / impossible; "If Claude keeps answering the evaluator
  without making progress … Claude Code stops the loop … and returns control to you".
- **Headless limits** ([D/cli-reference](https://code.claude.com/docs/en/cli-reference),
  [D/headless](https://code.claude.com/docs/en/headless)): `--max-turns` ("No limit by default"),
  `--max-budget-usd` (subagent spend counts; at the cap spawning fails with `Budget limit reached`
  and background subagents stop), `--output-format json` with `total_cost_usd` as "client-side
  estimates"; `-p` gives up waiting on background work after 10 minutes of idle waiting.
- **Best practices** ([D/best-practices](https://code.claude.com/docs/en/best-practices); the former
  engineering post redirects here): a ladder from a check in the prompt → `/goal` → a Stop hook → a
  verification subagent, so "the agent doing the work isn't the one grading it"; but a reviewer told
  to find gaps "will usually report some, even when the work is sound … Chasing every finding leads to
  over-engineering". After two failed corrections, `/clear`. `/batch` splits a change across 5–30
  subagents, each in its own worktree.
- **Worktree cleanup** ([D/worktrees](https://code.claude.com/docs/en/worktrees)): `-p` runs do not
  clean up their worktrees; a periodic sweep removes old subagent/background worktrees unless they
  hold uncommitted or unpushed work.

## 6. Codex

- **`codex exec`** ([non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode)):
  "By default, `codex exec` runs in a read-only sandbox"; `--sandbox workspace-write`; `--json` emits
  JSONL events (`thread.started`, `turn.started`, `turn.completed`, `item.*`, `error`);
  `--output-last-message`, `--output-schema`, `codex exec resume`. No turn or cost cap is documented
  on that page (unconfirmed that none exists).
- **Codex cloud** ([cloud](https://learn.chatgpt.com/docs/cloud);
  [CLI reference](https://learn.chatgpt.com/docs/developer-commands?surface=cli)): per-task
  environments; `codex cloud exec --env ENV_ID --attempts 1-4` for best-of-N; review the diff, then
  "commit or open a pull request when you're ready".
- **Codex app worktrees** ([git worktrees](https://learn.chatgpt.com/docs/environments/git-worktrees)):
  detached HEAD by default; "Git only allows a branch to be checked out in one place at a time", so
  Handoff moves a chat between Local and Worktree; keeps the 15 most recent managed worktrees.
- **app-server** ([app-server](https://learn.chatgpt.com/docs/app-server)): JSON-RPC over stdio with
  thread/turn/item primitives, `turn/started`/`turn/completed` notifications, approval requests to
  the client, interrupt, fork and resume — the public interface for watching and steering Codex.

## 7. Matt Pocock's `implement-spec` and Ralph loops

- **`implement-spec`** (M/skills/engineering/implement-spec/SKILL.md, user-invoked): tickets form a
  task graph with a frontier; one background implementer subagent per ticket, "each in its own
  worktree on its own branch", which resets onto the integration branch, runs `tdd`, and "merges the
  integration branch tip into its own branch before reporting done"; a merger subagent lands each
  finished ticket on one integration branch, then new frontier tickets start; `code-review` runs once
  at the end.
- **Its stated failure modes** (M/docs/engineering/implement-spec.md): "Worktrees don't remove
  collisions; they postpone them to merge time" — two tickets on "different parts of the codebase"
  still share a message catalogue, config registry or type; the fix is a blocking edge or exact names
  fixed in shared notes. Review run mid-run makes unbuilt tickets read as failures; one user's
  "review and fix loop took roughly four hours", and the skill "doesn't yet say when to stop". For
  truly AFK work "a deterministic loop (Sandcastle, a shell script, a CI job) is faster, cheaper, and
  more reliable, because no part of the orchestration can wander off". Also: a GitHub blocked-by count
  drops only when the blocker closes, and a test silently skipped in a worktree reported green
  (same doc; not re-checked individually).
- **Ralph** (Geoffrey Huntley, [ghuntley.com/ralph](https://ghuntley.com/ralph/), 2025-07-14): "Ralph
  is a Bash loop" feeding one prompt to a fresh agent each iteration; one thing per loop; many
  subagents for search but "only 1 subagent for build/tests"; greenfield only.
- **Pocock's Ralph tips** ([aihero.dev, 2026-01-08](https://www.aihero.dev/tips-for-ai-coding-with-ralph-wiggum)):
  start human-in-the-loop, then go AFK; "Always cap your iterations … 5-10 iterations for small tasks,
  or 30-50 for larger ones"; a PRD with `passes` flags and a committed progress file; block commits
  unless everything passes; run AFK in a sandbox. A bash loop gets "a fresh context window" each
  iteration, unlike a Stop-hook loop in one session ([why the plugin sucks](https://www.aihero.dev/why-the-anthropic-ralph-plugin-sucks)).
- **Anthropic's `ralph-wiggum` plugin** (CC/plugins/ralph-wiggum): `/ralph-loop "<prompt>"
  --max-iterations <n> --completion-promise "<text>"`, unlimited by default; its README: "Always rely
  on `--max-iterations` as your primary safety mechanism". Inference: the 8-continuation Stop-hook cap
  (§5) would end such a loop after about eight iterations unless raised.

## 8. Published multi-agent guidance

- **Anthropic, [multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system)**
  (2025-06-13): an orchestrator-worker pattern; "agents typically use about 4× more tokens than chat …
  multi-agent systems use about 15× more tokens than chats"; effort rules in the prompt (one agent for
  simple lookups, 2–4 subagents for comparisons, 10+ only for complex work); each delegation needs
  "an objective, an output format, guidance on the tools and sources to use, and clear task
  boundaries"; failure modes "spawning 50 subagents for simple queries" and duplicated work; "most
  coding tasks involve fewer truly parallelizable tasks than research"; subagent outputs go to storage
  rather than through the lead.
- **Anthropic, [building effective agents](https://www.anthropic.com/engineering/building-effective-agents)**
  (2024-12-19): orchestrator-workers fits "coding products that make complex changes to multiple
  files"; "include stopping conditions (such as a maximum number of iterations)"; "pause for human
  feedback at checkpoints or when encountering blockers".
- **OpenAI, [a practical guide to building agents](https://cdn.openai.com/business-guides-and-resources/a-practical-guide-to-building-agents.pdf)**:
  "maximize a single agent's capabilities first"; the manager pattern is "ideal for workflows where you
  only want one agent to control workflow execution and have access to the user"; rate tools by risk;
  two human-intervention triggers — "Exceeding failure thresholds: Set limits on agent retries or
  actions" and "High-risk actions … sensitive, irreversible, or have high stakes"; "For a coding
  agent, this means handing control back to the user".
- **OpenAI Agents SDK** ([multi-agent](https://openai.github.io/openai-agents-python/multi_agent/),
  [running agents](https://openai.github.io/openai-agents-python/running_agents/),
  [human in the loop](https://openai.github.io/openai-agents-python/human_in_the_loop/)): orchestration
  through code is "more deterministic and predictable" than through an LLM; `max_turns` raises
  `MaxTurnsExceeded` (default 10 in `src/agents/run_config.py` at `c513dd1`); tools with
  `needs_approval` pause the run as interruptions that a human approves or rejects before resuming.

## 9. Failure modes across the tools

1. **Sent is not received.** herdr-projects (#70), Agent Orchestrator (#5833) and Conductor (#27)
   each treated a prompt as delivered when it was sent. herdr-projects now waits for `working`;
   Herdr's own skill says a timeout "does not prove the prompt was never delivered" (§1, §2).
2. **Nudges collide with the human's typing.** herdr-projects (#60) and Agent Orchestrator (#5711);
   herdr-projects now nudges only an idle coordinator with an empty input box (§2).
3. **Status lies around subagents and restarts.** Superset (#6641, #7395), Vibe Kanban (#2783,
   #2495), Conductor (#21, #34), herdr-projects (pane ids reused after restart). Agent Orchestrator
   recomputes status from facts on read (§3).
4. **No stop lever for a looping worker.** Agent Orchestrator #6032 (~45M tokens in 45 min) because
   messages wait for a turn boundary. Claude Code has hard caps (`--max-budget-usd`, `--max-turns`,
   subagent `maxTurns`, 20 concurrent subagents, 8 Stop-hook continuations); none of the five
   external tools has a spend cap (§3–5).
5. **Review loops.** A reviewer told to find gaps always finds some (Claude Code best practices);
   `implement-spec`'s four-hour review-and-fix loop; Agent Orchestrator's reviewer ↔ worker loop
   (#3171) and self-review as a task (#5574) (§3, §5, §7).
6. **Worktrees postpone collisions.** Shared registries and types collide at merge (Pocock); a branch
   can be checked out in one place only (Codex, Conductor, Vibe Kanban #1897); `git worktree add`
   races (Agent Orchestrator #4350); cloud auto-fix cannot see base-branch conflicts (Claude Code).
7. **The coordinator does the work itself.** Claude Code teams' lead "starts implementing tasks
   itself instead of waiting" (§5).
8. **Forge and screen text is data, not orders.** herdr-projects passes counts, not GitHub text, and
   its inbox "approves nothing"; teammates' messages cannot grant consent in Claude Code (§2, §5).

## 10. Comparison

| | Launch | Watch | Merge gate | Parallel lanes | Cost / loop limits | Hand-back |
|---|---|---|---|---|---|---|
| Herdr 0.9.x (substrate) | `tab create`, `worktree create`, `agent start` | hook-fed states, `events.subscribe`, `agent wait` | none | none | none | `notification show` toast + sound; `agent_blocked` refusal |
| herdr-projects | coordinator agent → `hp thread start` → Herdr worktree + profile | Herdr state + self-report + `report.md` with `PR:` line; 15 s ticker | coordinator never merges; thread merges on user's forwarded Next line; CI-fix routine | worktree per thread; one owner per area | soft `max_parallel_threads` 10; 3 brief retries; no budget | Herdr notifications; inbox nudges; propose mode |
| Agent Orchestrator | orchestrator agent → `ao spawn`; own PTY host | hooks + screen + self-report; facts recomputed | human; re-reads CI, threads, head SHA; squash | worktree per session; rebase nudge on conflict | nudge de-dup; ≤3 review nudges; no spend cap (#6032) | desktop + mobile push; "Needs you" |
| Conductor | app → worktree per workspace; setup scripts, port blocks | undocumented; API status | human; checks may block; agent writes PR | one branch per workspace; agent resolves conflicts | idle kill 30 min; none | sounds; approvals; plan mode |
| Superset | app/CLI → worktree per workspace | hooks → Start/PermissionRequest/Stop; envelopes | human `gh pr merge` | split by files; 2–3 slices | none ("your review attention") | notifications; inline approvals |
| Vibe Kanban (sunsetting) | board → worktree per repo | process exit; hooks | human direct squash without CI, or PR | `rebase --onto`; agent resolves | none found; approvals expire 10 h | sound + OS push |
| Claude Code | subagents, `--bg`, `--cloud`, teams (experimental) | agent view, `--json`, hooks, stream-json | Stop/TaskCompleted hooks; verification subagent; human merges | `isolation: worktree`; teams: own files | `--max-budget-usd`, `--max-turns`, `maxTurns`, 20 concurrent, 8 Stop continuations | Notification hooks; permission prompts surface in lead |
| Codex | app threads, `codex exec`, cloud best-of-N | `--json` events; app-server notifications | human; PR from cloud | app worktrees (detached HEAD) | none documented for `exec` (unconfirmed) | approval requests via app-server |
| `implement-spec` | one background subagent per frontier ticket | orchestrator in one session | merger subagent to one integration branch; review at the end | lane merges integration tip before done; blocking edges for shared surfaces | none; review loop unbounded | user-invoked; draft PR after first merge |
| Ralph loop | bash loop, fresh agent per iteration | progress file, tests | "block commits unless everything passes" | one task per iteration, sequential | iteration cap (5–10 / 30–50) | start HITL, then AFK |

Sources: §1–§8.

## 11. CanvasTTY since 2026-09-01 and forking it

### 11.1 What changed

- **Numbers.** 91 stars and 20 forks on 2026-09-30 (82 and 19 the day before), 6 open issues and PRs,
  last push 2026-09-29 (`gh api repos/howdeploy/CanvasTTY`).
- **Releases.** [v1.5.1](https://github.com/howdeploy/CanvasTTY/releases/tag/v1.5.1) (2026-09-07:
  Claude limit tracking fix, radial quick launcher, note closing);
  [v1.5.2](https://github.com/howdeploy/CanvasTTY/releases/tag/v1.5.2) (2026-09-21: OMP and Pi
  providers, ADRs, an Even G2 AR-glasses companion);
  [v1.7.0](https://github.com/howdeploy/CanvasTTY/releases/tag/v1.7.0) (2026-09-29, no 1.6):
  orchestrators with agent-to-agent delegation, plugin services, launch contributors, session
  environments, an attention queue, fail-closed decision hooks, skins, new providers. Its notes list
  open orchestration gaps: no per-subagent model choice and no provider-discovery or wait tools.
- **Commits since 2026-09-01.** 210 with merges, 170 without; by author BIackFIame 86, howdeploy 59,
  teo-nex 27, qSHEMq 19, others 19. Of non-merge commits 91 fix, 25 feat, 20 perf, 15 test
  (`gh api repos/howdeploy/CanvasTTY/commits?since=2026-09-01T00:00:00Z`). All 86 of BIackFIame's
  commits fall on 23–29 September; his PRs were closed unmerged and their content landed through the
  owner's integration PRs #72, #73, #101.
- **Direction.** In [PR #66](https://github.com/howdeploy/CanvasTTY/pull/66) the owner moved SSH
  hosts, containers and server distribution out of the core into "skills, hooks and settings";
  features went into `canvastty-plugin-*` repositories and the core got extension points.
- **SDK** (still `apiVersion` 1 and 2 only, T/docs/canvastty-plugin.schema.json L13): new permissions
  `launch:contribute`, `environment:provide`, `decision:provide`, `tools:agents` among others (L40);
  session environments decide where a card runs, with a git-worktree example
  (T/examples/plugins/env-worktree; T/CHANGELOG.md L19).
- **Attention.** An attention queue on HOME, an attention ring on cards and an OS notification when a
  session enters needs-approval or failed (T/CHANGELOG.md L38–39).
- **New ADRs** (T/docs/adr): five on 2026-09-13/14 (fuses, provider lifecycle, selective intake from
  the v1.2.1 line, marquee selection), two on 2026-09-21 (orchestration MCP rides the agent bridge,
  provider CLI command definitions).
- **No edges or lanes.** No card-to-card links are rendered; `parentSessionId` is not used anywhere
  in `src/renderer` (grep at `15983c4`).
- **No Herdr or tmux.** "herdr" appears in neither code nor issues; "tmux" only in a mouse-reporting
  fix (T/CHANGELOG.md L192). Arbitrary native window embedding is excluded by design
  (T/docs/plugins.md L22).

### 11.2 Its orchestration

- **Control endpoint and CLI** (T/docs/agent-orchestration.md; T/agent/orchestrator/SKILL.md;
  T/src/main/services/agent-control/AgentControlGateway.ts L393–486): off by default; commands
  `create`, `list`, `status`, `screen`, `send`, `result --after N`, `choose`, `dismiss`, `interrupt`;
  no wait, close or delete. Nine providers, but only Codex has `result` (its final answer caught by a
  Stop hook) and menus; for the rest the only evidence is the screen text. Workers start in YOLO mode
  by default (`--profile normal|auto` exists). A 0600 Unix socket, a token that rotates on restart, a
  controller sees only sessions it created; up to 32 controlled sessions; `--request-id` idempotency;
  no sending into an active turn.
- **MCP `canvastty_agents`** for orchestrator-role sessions (T/src/agent-browser/orchestration-catalog.mjs;
  T/docs/adr/ADR-20260921-orchestration-mcp-rides-agent-bridge.md): `spawn_agent`, `send_to_agent`,
  `observe_agent`, `get_agent_result`, `cancel_agent` (kills the process), `list_agents`; only the
  caller's descendants; at most 16 children per parent (T/src/main/services/AgentControlService.ts L14).
  Refusal texts mention `list_routes` and `wait_for_agent`, which are not in the catalogue
  (AgentControlGateway.ts L37).

### 11.3 Fork viability for a canvas view of lanes

- **License**: MIT, "Copyright (c) 2026 howdeploy" (T/LICENSE).
- **Size** (`tokei` at `15983c4`, without `node_modules`): ~106,800 lines of code in 611 files;
  `src` ~60,400 (main 32,300, renderer 22,800, shared 2,200); tests ~28,200 in 182 `node --test` files.
- **Structure.** Electron + electron-vite + React + xterm (WebGL) + node-pty, no graph or canvas
  library (T/package.json). `WorkspaceCanvas.tsx` (1,344 lines) imports the terminal, browser,
  plugin and Home cards directly and takes sessions plus about 40 callbacks as props
  (T/src/renderer/src/features/workspace/WorkspaceCanvas.tsx L1–230): the renderer is separated from
  PTYs by IPC, but the canvas is tied to its card types.
- **CI and build.** Ubuntu, Windows and macOS jobs (T/.github/workflows/ci.yml); macOS builds are
  ad-hoc signed, `notarize: false` (T/electron-builder.yml L118–123).
- **Bus factor.** Last 30 days: BIackFIame 41 %, howdeploy 28 % of commits with merges; only howdeploy
  merges and releases. Since creation (2026-08-05): howdeploy 126 of 322. 22 releases in 55 days.
  No CONTRIBUTING; SECURITY.md exists; no stated stance on forks; one named product fork
  (dmitriy86it/CanvasTTYnextGen) (`gh api repos/howdeploy/CanvasTTY/forks`).
- **Plugin instead of fork.** An `apiVersion: 2` service with `sessions:events` receives each card's
  role, `parentSessionId`, cwd and environment, and can run `git`/`gh` as a Node process; a
  `canvas-app` card can ask its service for data and draw a lane graph inside itself;
  `cards:decorate` adds badges (≤ 24 characters) and menu items to native cards
  (T/docs/plugins.md L336–420). It cannot draw edges between native cards (the canvas app is a
  sandboxed iframe), cannot focus another card (T/docs/plugins.md L465), installs only from the root
  of a public GitHub repository (L497–508), and native services need a trust confirmation that resets
  on every update.
- **Herdr panes on the canvas.** No integration; running `herdr` inside a terminal card would be
  manual (unconfirmed, not tried).

## 12. Implications for the conductor

Facts and options for question 2 of #67; nothing here is decided.

**What Herdr already gives and what it does not.**

- Herdr covers the mechanics of launch (tab, worktree, `agent start` with 24 kinds), watching
  (`agent wait`, `events.subscribe` on `pane.agent_status_changed`) and a toast with a sound (§1).
  It has no queue, graph, budget, merge or reply channel; any conductor supplies those, and the
  command center plan already derives the graph and PR facts (CC8, CC11).
- Herdr's own skill defaults to sibling panes and asks the agent not to create tabs or worktrees
  unless the user asked (§1). A conductor that opens tabs and worktrees is therefore acting on an
  explicit user request in Herdr's terms.
- Herdr's `agent_blocked` refusal plus "ask the user before answering it" is already a hand-back
  rule that a conductor can inherit (§1).

**Options for where the conductor lives** (observed shapes, not recommendations):

- A. *An agent session following a skill*, with deterministic parts in a CLI (herdr-projects,
  Agent Orchestrator, Superset's orchestrate skill, `implement-spec`). Easy to change; the recorded
  failures are the coordinator doing work itself, nudges racing the human, and coordinator token
  cost (§2, §3, §5).
- B. *Deterministic code that launches and watches*, with the LLM only in executors (Ralph loops,
  Sandcastle, Codex cloud, Agents SDK code orchestration). Pocock, OpenAI's SDK docs and Anthropic's
  "include stopping conditions" point this way for AFK work (§7, §8).
- C. *Both*: a skill proposes lanes; a small command applies them. herdr-projects' split (skill
  decides with the user, binary executes and polls) is this shape, and so is the owner's decided
  read-only coordinator skill in #55 plus a separate "start" action (CC14).

**Scope of what may start without the owner — observed boundaries.**

- Nobody merges automatically in the five external tools; the strictest gate re-reads CI, threads
  and the head SHA the human saw (Agent Orchestrator). herdr-projects forbids its coordinator to
  merge, push, delete branches or remove worktrees at all (§2–§4). This matches ADR 0004: "What still
  blocks lives outside the harness: the project's own CI and the owner's merge."
- Starting lanes ranges from "propose, then an explicit go-ahead naming the lane" (herdr-projects
  `start_threads=propose`) to free spawning within caps (Claude Code teams, CanvasTTY's 16 children).
- Permission prompts: approve once only inside the lane's task and worktree, never "always allow";
  anything outward-facing goes to the human (herdr-projects, Claude Code teams) (§2, §5).

**Conflicts between lanes.**

- Worktree per lane is universal; it postpones collisions to merge time (§7). Known mitigations:
  one owner per area (herdr-projects), a blocking edge between tickets that share a surface
  (Pocock), merging the integration tip into the lane before "done" so merges fast-forward
  (`implement-spec`), files split by lane (Claude Code teams, Superset). The harness already models
  blocking edges (ADR 0003), so "what became unblocked" is a read of the graph plus merged PRs; note
  that a GitHub blocked-by count drops only when the blocker closes (§7).

**Cost and runaway loops.**

- Multi-agent costs about 15× chat tokens in Anthropic's system and teams about 7× a session (§5,
  §8); ten parallel sessions use quota ten times as fast (§5).
- Hard caps exist only inside runtimes (Claude Code `--max-budget-usd`, `--max-turns`, `maxTurns`;
  the Agents SDK's `max_turns`); the external coordinators have none, and a looping worker burned
  ~45M tokens where no interrupt could reach it (§3). Options: a parallel-lane cap (soft in
  herdr-projects), passing runtime caps as launch arguments, an interrupt that does not wait for a
  turn boundary (Herdr's `agent send-keys … ctrl+c`/`esc` exists, §1), and a retry cap per signature
  (Agent Orchestrator's 3).
- Review loops need an explicit stop (§7, §9.5).

**Calling the owner.**

- Triggers observed across tools: needs input or approval, blocked longer than a threshold (30 s in
  herdr-projects), result ready for review, ready to merge, failed CI after the lane's own fix, a
  merge conflict, repeated failures (OpenAI's "failure thresholds"), and high-risk actions (§2, §3,
  §8). Herdr gives a toast and two sounds (§1); Agent Orchestrator adds mobile push; Telegram through
  the owner's personal agent would be a new channel not covered by these sources.
- Deliver a brief, not raw output (Pocock's `loop-me` "Brief", Anthropic's delegation contract) —
  the result card (CC4) is already that brief.
- A nudge must not merge with the owner's typing: prompt only when the input is empty and the agent
  idle, and confirm by a state change, not by the send (§9.1–2).

**Watching reliably.**

- Status from hooks is wrong around subagents and restarts in several tools; recomputing from facts
  on read (Agent Orchestrator) and verifying pane identity before acting (herdr-projects) are the
  known counters (§9.3). Herdr 0.9.2 fixed false done notifications and made event loss explicit
  (§1), and the local 0.9.1 predates it.

**Canvas view of lanes.**

- Fork: MIT, ~60k lines of source moving fast (210 commits in September, merges by one person),
  canvas tied to its card types, no edges yet, no Herdr integration, Electron with an un-notarized
  macOS build (§11.3).
- Plugin: lane data, `git`/`gh` and badges are reachable; edges only inside one canvas-app card,
  install only from a public repository root, trust re-confirmed on every update (§11.3).
- Reference only: its scoped control endpoint (own sessions only, idempotent request ids, rotating
  token), per-provider capability table (only Codex returns a result), attention queue and session
  environments are patterns (§11.2). The decided order puts our own UI last (CC12), so a canvas is
  a later-stage question either way.

## References

Pinned repository prefixes are in the table under "Method". Other sources:

- Herdr releases: https://github.com/herdrdev/herdr/releases/tag/v0.9.2,
  https://github.com/herdrdev/herdr/releases/tag/v0.9.3
- Claude Code docs: https://code.claude.com/docs/en/sub-agents, /agent-view, /agent-teams, /hooks,
  /goal, /costs, /cli-reference, /headless, /best-practices, /worktrees, /claude-code-on-the-web
- Anthropic engineering: https://www.anthropic.com/engineering/multi-agent-research-system,
  https://www.anthropic.com/engineering/building-effective-agents
- OpenAI: https://cdn.openai.com/business-guides-and-resources/a-practical-guide-to-building-agents.pdf;
  https://openai.github.io/openai-agents-python/multi_agent/, /running_agents/, /human_in_the_loop/;
  https://learn.chatgpt.com/docs/non-interactive-mode, /cloud, /environments/git-worktrees,
  /app-server, /developer-commands?surface=cli
- Ralph: https://ghuntley.com/ralph/; https://www.aihero.dev/tips-for-ai-coding-with-ralph-wiggum;
  https://www.aihero.dev/getting-started-with-ralph;
  https://www.aihero.dev/why-the-anthropic-ralph-plugin-sucks
- Issues cited by number: eliasstravik/herdr-projects, Untrivial-ai/agent-orchestrator,
  meltylabs/conductor-releases, superset-sh/superset, BloopAI/vibe-kanban (each under
  `https://github.com/<repo>/issues/<n>`).
- Vibe Kanban shutdown: https://www.vibekanban.com/blog/shutdown
- CanvasTTY: https://github.com/howdeploy/CanvasTTY/releases, https://github.com/howdeploy/CanvasTTY/pull/66
