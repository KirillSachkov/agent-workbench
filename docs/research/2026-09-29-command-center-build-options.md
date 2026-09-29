# Command center for agent work: data sources, UI options, architecture

Date: 2026-09-29. Owner's machine: herdr 0.9.1, Claude Code 2.1.284, codex-cli 0.158.0.
Method: live read-only commands on the machine + official documentation. Anything not verified
by a command or a primary source is marked "unconfirmed".

## 1. Data sources

### 1.1 Herdr (the richest source of live state)

Verified: `herdr api --help` → subcommands `snapshot`, `schema`; `herdr api schema --json`
(protocol 22, schema_version 1; schemas `request`, `success_response`, `error_response`, `event`,
`subscription_event`). 129 methods/events were extracted from the schema, the key ones:

- Read: `session.snapshot`, `workspace.list|get`, `tab.list|get`, `pane.list|get|read|process_info`,
  `agent.list|get|read|explain|wait`, `worktree.list`, `plugin.list|action.list|log.list`,
  `integration.list`, `server.agent_manifests`.
- Actions: `agent.focus|prompt|send_keys|start|rename`, `pane.split|focus|send_text`,
  `workspace.create|focus`, `worktree.create|open|remove`, `notification.show`,
  `plugin.action.invoke`, `plugin.pane.open|focus|close`, `command.invoke`.
- Metadata from external sources: `pane.report_metadata` (title, display_agent, state_labels,
  up to 16 `tokens`, `ttl_ms`), `workspace.report_metadata`, `pane.report_agent_session`
  (agent_session_id / path), `agent.view.set` (filter/sort in the agents sidebar).
- Events: `events.subscribe` (persistent stream), `events.wait`; types `workspace.*`, `tab.*`,
  `pane.created|closed|exited|updated|agent_detected|agent_status_changed|output_matched`,
  `worktree.created|opened|removed`, `layout.updated`.

`herdr agent list` on the machine returned 7 agents with the fields: `agent` (claude/codex),
`agent_session.value` (runtime session ID), `agent_status` (`idle|working|blocked|done|unknown`),
`cwd`, `foreground_cwd`, `pane_id`, `tab_id`, `workspace_id`, `terminal_title`, `revision`,
`state_change_seq`, and `tokens` {repo, branch, worktree, base}. The tokens are written by the existing
plugin `sachkov.agent-context` (`<personal agent-context plugin>`),
which stores the binding "agent_session → exact git worktree" and already supports `review` (Hunk on
merge-base) and `files`. This is a ready-made core for the "agent ↔ branch ↔ worktree" link.

Documentation (https://raw.githubusercontent.com/herdrdev/herdr/v0.9.1/docs/next/website/src/content/docs/socket-api.mdx):
NDJSON over the unix socket `~/.config/herdr/herdr.sock`; `session.snapshot` — "one-time bootstrap snapshot
for clients that keep their own local runtime cache"; subscriptions "do not replay events retained
before that point" → pattern: open a subscription, then take a snapshot, then apply events.
Compatibility: "Client and server builds do not need to match", methods are declared by the server,
unsupported ones return an ordinary error. This is a public, versioned contract (schema + protocol).
`done` = "idle and not yet seen".

Agent status is reliable because the official integrations are installed (`herdr integration status`:
claude v10, codex v8, copilot, kimi, opencode — via runtime hooks).

Plugins (https://raw.githubusercontent.com/herdrdev/herdr/v0.9.1/docs/next/website/src/content/docs/plugins.mdx):
a directory with `herdr-plugin.toml`: `actions`, `panes` (any argv-TUI; placement overlay/popup/split/
tab/zoomed), event hooks, link handlers; env `HERDR_SOCKET_PATH`, `HERDR_PLUGIN_STATE_DIR`,
`HERDR_PANE_ID` and others; `plugin install` (GitHub) / `plugin link` (local). "Native non-terminal
plugin UI are not part of plugin v1" — that is, plugin UI is terminal-only.

### 1.2 Claude Code

- `claude agents --json` (verified): an array of active sessions, interactive and background:
  `sessionId`, `pid`, `cwd`, `kind`, `name`, `status` (`busy|idle`) or `state` (`blocked` for bg),
  `startedAt`; `--all` adds finished bg sessions. A documented scripting interface
  (https://code.claude.com/docs/en/sessions — "identifies the session in listings of running sessions,
  such as agent view and `claude agents --json`").
- Hooks (https://code.claude.com/docs/en/hooks): SessionStart/End, UserPromptSubmit, Stop,
  StopFailure, Notification (matcher `permission_prompt`, `idle_prompt`, `agent_needs_input`,
  `agent_completed`…), PermissionRequest, Subagent*, TaskCreated/Completed, CwdChanged,
  WorktreeCreate/Remove and others. Common fields: `session_id`, `transcript_path`, `cwd`. The `http` type —
  POSTs JSON to a local endpoint: a clean way to stream events into your own index without files.
- Transcripts `~/.claude/projects/<project>/<session-id>.jsonl` — **internal format**:
  "The entry format is internal to Claude Code and changes between versions" (same page). Do not parse.
- `claude --from-pr <n>` — a picker of sessions linked to a PR (the session that created the PR is found by URL).
  Suitable as an "open session by PR" action, but not as a data source.
- Headless: `claude -p --output-format json|stream-json` (`system/init`, `assistant`, `result` with
  `session_id`, cost) (https://code.claude.com/docs/en/headless) — for launching review/summary from the UI,
  e.g. `claude -p --resume <id> --output-format json "summarize"`.
- OpenTelemetry (https://code.claude.com/docs/en/monitoring-usage): `CLAUDE_CODE_ENABLE_TELEMETRY=1`,
  OTLP/Prometheus; metrics `session.count`, `cost.usage`, `token.usage`, `pull_request.count`,
  `commit.count`; events `user_prompt`, `tool_result`… with `session.id`. Useful for cost and
  activity, excessive for status (status is already available in Herdr).

### 1.3 Codex

- `codex exec --json`: JSONL `thread.started{thread_id}` … `turn.completed{usage}` (secondary
  source https://takopi.dev/reference/runners/codex/exec-json-cheatsheet/; official — unconfirmed).
- App-server (https://learn.chatgpt.com/docs/app-server): JSON-RPC 2.0 over stdio / unix socket
  (stable) / WebSocket (experimental); methods `thread/list|read|resume|start`, `turn/start|interrupt`,
  `review/start`; notifications `turn/started|completed`, `item/*`. The schema is generated by
  `codex app-server generate-json-schema|generate-ts` for a specific version. The machine already has a
  shared daemon (`~/.codex/app-server-daemon`, `codex agents` "Browse all agent sessions on the shared
  local app-server daemon"). The `codex app-server` command is marked `[experimental]` in the CLI — treat
  the contract as stable only for the thread/turn core.
- Hooks (https://developers.openai.com/codex/hooks): SessionStart, PreToolUse, PermissionRequest,
  PostToolUse, UserPromptSubmit, Subagent*, Stop (the full list for version 0.150+ is from a secondary source,
  unconfirmed).
- Local sqlite files (`~/.codex/state_5.sqlite`, `thread_history_1.sqlite`, rollouts in
  `~/.codex/sessions/`) are **internal**, do not use.

### 1.4 GitHub

- Projects v2 (verified with `gh project field-list`): Developer Pipeline — Status {Inbox, Blocked, Ready,
  In progress, Review, Acceptance, Done}, Area, Priority, Linked pull requests, Parent issue,
  Sub-issues progress, Repository, Reviewers. Human Backlog — Status {Todo, In Progress, Done}, Priority.
  Statuses are already projected by `inside_tracker.py` (see `docs/agents/tracker-automation.md`), and
  session ownership is a trusted issue comment from `tracker_sessions.py start` (a receipt with the session id and
  branch). This is a ready-made canonical join "issue ↔ session ↔ branch".
- Read: GraphQL `organization.projectV2(number).items(first:100){fieldValues, content{Issue|PullRequest}}`;
  PR: `statusCheckRollup`, `reviewDecision`, `isDraft`; search `gh search prs --owner sachkov-inside`.
- Limits (verified with `gh api rate_limit`): core 5000/h, graphql 5000 points/h. Polling Projects every
  60–120 s for a few hundred items fits with a large margin. Conditional REST requests with ETag
  (304) do not consume the primary limit (https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api).
- Webhooks: `projects_v2_item` — **org webhooks only, public preview, "subject to change"**
  (https://docs.github.com/en/webhooks/webhook-events-and-payloads#projects_v2_item). A local
  app needs a public endpoint or relay (`gh webhook forward` is a dev tool). For a solo machine
  polling is simpler and more honest; webhooks only if a server component appears.

### 1.5 Git and harness

- `git worktree list --porcelain` — a stable machine-readable format (verified on platform: worktrees
  in `inside/worktrees/*` and foreign ones, e.g. `<another tool's directory>`).
  `herdr worktree list` gives the same plus `open_workspace_id`.
- `~/Work/projects.json` — a registry of 18 projects (`id`, `path`, `remote`, `stacks`, `status`).
- `harness/bin/inside-harness` — install/update/diff/health/rollback; it has no JSON status output of its own
  (unconfirmed whether `--json` exists). Pipeline-stage artifacts (spec/tickets) live in issues.

### Stability summary

| Source | Contract | Use for |
|---|---|---|
| Herdr socket API + events | public, versioned schema | live agents, status, focus, actions |
| `claude agents --json`, hooks (`http`) | public | Claude sessions outside Herdr, events |
| Codex app-server thread/turn | stable core, the rest experimental | Codex sessions, launching review |
| GitHub GraphQL/REST, `gh` | public | tasks, stage, PR, checks, review |
| `git worktree --porcelain` | stable | branches/worktrees |
| Transcripts `.jsonl`, codex sqlite | **internal** | do not use |
| Projects webhooks | public preview | not now |

## 2. UI options

**(a) GitHub Projects views + saved searches + gh-dash.** Effort: hours. Maintenance: almost zero.
Presentability: high (Projects is web, GitHub Mobile). Actions: gh-dash keybindings with
templates `{{.RepoName}} {{.PrNumber}} {{.HeadRefName}} {{.RepoPath}}`, `repoPaths` maps a repo
to a local path (https://gh-dash.dev/configuration/keybindings/). Downside: it does not see agents, worktrees or
Herdr; Projects fields are not supported in gh-dash (unconfirmed). Example:

```yaml
# ~/.config/gh-dash/config.yml
prSections:
  - title: Awaiting my review
    filters: is:open org:sachkov-inside review-requested:@me
  - title: My agents (draft/open)
    filters: is:open org:sachkov-inside author:@me
issuesSections:
  - title: Ready for agent
    filters: is:open org:sachkov-inside label:ready-for-agent
  - title: Owner gate
    filters: is:open org:sachkov-inside label:ready-for-human
repoPaths:
  sachkov-inside/platform: ~/Work/Products/inside/repositories/platform
  sachkov-inside/*: ~/Work/Products/inside/repositories/*
keybindings:
  prs:
    - key: R
      name: review in Claude
      command: cd {{.RepoPath}} && claude --from-pr {{.PrNumber}}
    - key: v
      name: approve
      command: gh pr review --repo {{.RepoName}} --approve {{.PrNumber}}
```

**(b) TUI as a Herdr plugin pane (Textual / Bubble Tea / Ratatui).** Effort: 2–5 days for v1.
Maintenance: medium, but in the same language as the harness (Python → Textual). Presentability: good for
a demo in the terminal, not for a link to a colleague. Native actions: `agent.focus`, `plugin.action.invoke`
(agent-context review), `pane.split` + `claude --resume`, `gh pr view --web`. It lives where the
agents live; subscribing to Herdr events gives instant status.

**(c) Local web (FastAPI + HTMX/SSE or SvelteKit).** Effort: 1–2 weeks. Maintenance: higher
(frontend, build). Presentability: the best, you can show the screen/a screenshot, and optionally
publish a read-only snapshot. Actions: the backend calls the same CLI/socket; terminal focus is
`agent.focus` via Herdr, opening a PR is a link. Risk: a localhost server with the ability to send input to agents
must be protected (bind 127.0.0.1, token).

**(d) Desktop Tauri.** Effort: (c) + packaging/signing. Benefit over (c) — menubar, native notifications.
Does not pay off for a solo developer right now.

**(e) Backstage / Port.** Backstage is a heavy Node monorepo for org catalogs; Port is SaaS
(data leaves the machine). Neither knows about local agents; overkill for one person.

**(f) Raycast extension.** Effort: 1–2 days (TypeScript). Good as a launcher on top of a ready index:
"find a task/agent → focus/open PR". Does not replace an overview screen. Makes sense as a second client
of the same read-model.

## 3. Architecture without hacks

1. **One read-model**, built by adapters, each reading only a public contract:
   `herdr` (snapshot + events.subscribe), `claude` (`agents --json`, optionally an http-hook),
   `codex` (app-server `thread/list` or only status from Herdr), `github` (GraphQL Projects + PR
   rollup, polling with an `updatedAt` cursor), `git` (`worktree list --porcelain` over `projects.json`),
   `tracker` (session receipts from issue comments, already in the harness format).
2. **Join keys**: `session_id` (Herdr `agent_session.value` = Claude/Codex id), `worktree path`
   (agent-context binding), `branch` → `issue #` (convention `feat/<n>-slug`), issue → Project item →
   Status (= pipeline stage), PR `Closes #n`. Mark fuzzy links as "inferred", not as fact.
3. **Events + polling**: Herdr — events (local, cheap); GitHub — polling 60–120 s + ETag;
   git — on Herdr `worktree.*` events and every N minutes. Webhooks are deferred.
4. **Where state lives**: the truth stays in the sources (GitHub — tasks and stages; Herdr — live;
   git — branches). The index is a disposable cache (SQLite in `~/.local/state/<tool>/`), rebuilt
   from scratch. Do not introduce your own task statuses — that would be a second tracker (forbidden by Work's `AGENTS.md`).
5. **Runtime-agnostic**: the core operates on the concept `AgentSession{runtime, id, status, cwd, worktree}`;
   Herdr already normalizes statuses for 5+ runtimes, so the main status adapter is Herdr, and
   the runtime adapters only add what Herdr lacks (session name, Claude bg sessions).
6. **Actions = calls to existing CLIs**, not input simulation: `herdr agent focus`,
   `herdr plugin action invoke sachkov.agent-context review`, `gh pr view --web`,
   `claude --resume <id>` / `codex resume <id>` in a new pane, `gh pr review --approve` (owner gate —
   only by an explicit owner action).
7. **Installability**: the core is a Python package with a CLI `… snapshot --json`; the UI layer is a Herdr plugin
   (`herdr-plugin.toml` with a pane and actions). Project config comes from `projects.json`, org/Projects from the
   harness (`docs/agents/issue-tracker.md` already defines the Projects numbers). Decision for the owner: where
   the code lives. The tool is personal and cross-project → naturally next to `sachkov-agent-context` in
   the personal plugins directory or a separate repo; in the `inside-engineering` package — only
   if it must be installed in every Inside repo (probably not).

## 4. What to reuse

- **sachkov-agent-context** (already exists): session→worktree binding, repo/branch/base tokens, the review action.
- **gh-dash** (https://github.com/dlvhdr/gh-dash): the GitHub part today, custom keybindings.
- **lazygit**, **Hunk**, **octo.nvim** (https://github.com/pwntester/octo.nvim) — review inside
  the terminal/Neovim; launch as an action, do not embed.
- **GitHub Mobile / Projects web** — a presentable read-only view for other people.
- `gh` extensions: `gh-dash`, `gh webhook forward` (dev relay), `gh project` (built in).
- Open-source dashboards for borrowing ideas/code (not examined in detail): agent-deck
  (https://github.com/asheshgoplani/agent-deck), claudecodeui (https://github.com/siteboon/claudecodeui),
  lists https://github.com/andyrewlee/awesome-agent-orchestrators and
  https://www.augmentcode.com/tools/open-source-agent-orchestrators. Most own agent launching
  (tmux) themselves — this conflicts with Herdr; take UI ideas, not the runtime.
- Linear: gives a nice UI and GitHub sync, but it is a second tracker → contradicts the rule "the project
  tracker is the single source of status". Not recommended.

## 5. Recommendation

**Architecture:** a Python core "read-model + adapters" (Herdr socket/events, `claude agents --json`,
GitHub GraphQL polling, `git worktree --porcelain`, tracker receipts) → SQLite cache → two clients:
(1) a Textual TUI as a Herdr plugin pane (primary, with actions), (2) later — a FastAPI+HTMX read-only
page for showing to others. GitHub Projects stay the source of truth for tasks and stages;
gh-dash covers the GitHub slice without code.

**Minimal first version (≈2–3 days):**
1. Day 0: the gh-dash config above (review-queue, ready-for-agent, owner gate) — zero code.
2. CLI `cc snapshot --json`: a join of `herdr agent list` + `claude agents --json` + `git worktree list`
   over `projects.json` + Developer Pipeline items (Status, Linked PR, checks rollup). Key: worktree/branch
   → issue #. No daemon, polling at launch.
3. A Textual pane in a Herdr plugin: three tables — "Agents" (runtime, status, repo/branch, issue, stage),
   "Tasks by project" (Ready/In progress/Review/Acceptance), "Worktrees without agent/without PR".
   Actions: Enter → `herdr agent focus`; `r` → agent-context review; `o` → `gh pr view --web`;
   `s` → a new pane with `claude --resume`.
4. Done criterion: for every live agent the task, stage and PR are visible without manual lookup;
   every action is a call to a public CLI/API; deleting the cache loses nothing.

Next steps after v1: `events.subscribe` subscription instead of polling Herdr; a Claude `http`-hook for
Notification/Stop; a web read-only view; a Raycast launcher on top of `cc snapshot --json`.
