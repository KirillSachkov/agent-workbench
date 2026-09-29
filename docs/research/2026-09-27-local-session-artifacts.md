# Local traces of agent work: mechanisms and tools (2026-09-27)

Task: obtain human-readable artifacts locally and automatically for every agent session (Claude
Code, Codex CLI) in git worktrees — what the task was, what was done, what changed, which checks
ran and with what result, where to look, what remains — and link them to a future GitHub PR.

Versions checked locally: Claude Code 2.1.283, codex-cli 0.157.1. File formats were checked by
keys only, without contents.

---

## 1. Claude Code

### 1.1 Hooks

Source: https://code.claude.com/docs/en/hooks

**Events (33)**: SessionStart, Setup, UserPromptSubmit, UserPromptExpansion, PreToolUse,
PermissionRequest, PermissionDenied, PostToolUse, PostToolUseFailure, PostToolBatch, Notification,
MessageDisplay, SubagentStart, SubagentStop, TaskCreated, TaskCompleted, Stop, StopFailure,
TeammateIdle, InstructionsLoaded, ConfigChange, CwdChanged, DirectoryAdded, FileChanged,
WorktreeCreate, WorktreeRemove, PreCompact, PostCompact, PreModelSwitch, PostModelSwitch,
Elicitation, ElicitationResult, SessionEnd.

**Common stdin JSON fields**: `session_id`, `prompt_id`, `transcript_path`, `cwd`, `scratchpad_dir`,
`permission_mode`, `effort`, `hook_event_name`, and for subagents `agent_id`, `agent_type`.

**Key events for a "session journal"**:

| Event | What it provides | Specific fields |
|---|---|---|
| SessionStart | start/resume/clear/compact/fork; can return `additionalContext` (for example, "task #123, branch X") | matcher `startup\|resume\|clear\|compact\|fork` (the name of the source field differs in the docs excerpt: `source`/`how_started` — unconfirmed, check on your own hook) |
| PostToolUse / PostToolUseFailure | every Bash command (tests, lint) with its result | `tool_name`, `tool_input`, `tool_response`, `tool_use_id` |
| Stop / SubagentStop | end of a response; the agent's final message | `last_assistant_message`, `stop_hook_active` |
| TaskCreated / TaskCompleted | tasks in the internal todo list | `task_id`, `task_name` |
| WorktreeCreate / WorktreeRemove | worktree creation/removal | `worktree_path`, `branch` |
| SessionEnd | finalizing the journal | `reason`: `clear\|resume\|logout\|prompt_input_exit\|other` |

**Blocking**: exit 2 blocks PreToolUse, UserPromptSubmit, UserPromptExpansion, Stop,
SubagentStop, TeammateIdle, TaskCreated, PreModelSwitch, WorktreeCreate, WorktreeRemove. For
PostToolUse, Notification, SessionEnd and others, exit 2 does not block. Stop with exit 2 = "don't
finish", stderr goes to the agent — this can force the agent to finish writing the report before
it ends (`stop_hook_active` must be checked to avoid a loop).

**Hook types**: `command`, `http`, `mcp_tool`, `prompt`, `agent`. `async: true` — background run;
`asyncRewake: true` — background, wakes Claude on exit 2.

**Timeouts**: command 600 s by default; **SessionEnd — a shared budget of 1.5 s** (up to 60 s if a
larger timeout is set). So do not do heavy report generation on SessionEnd: write incrementally in
PostToolUse/Stop, and in SessionEnd only finalize or launch a detached process.

**Where to configure**: `~/.claude/settings.json` (all projects), `.claude/settings.json` (in the
repo, for all sessions in this repo and its worktrees), `.claude/settings.local.json` (gitignored),
managed policy, plugin `hooks/hooks.json`, skills/agents frontmatter. Hooks from different levels
are **merged**. Project hooks are subject to workspace trust. `${CLAUDE_PROJECT_DIR}` is the root
where the session started; it does not change in a worktree, so take the current directory from
`cwd`.

**Answer**: yes, a hook from `.claude/settings.json` in the repo fires for every session opened in
this repo (including worktrees, since the file is committed and present in the worktree).

Example (project `.claude/settings.json`):

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command",
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh start" }] }],
    "PostToolUse": [{ "matcher": "Bash", "hooks": [{ "type": "command", "async": true,
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh tool" }] }],
    "Stop": [{ "hooks": [{ "type": "command",
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh stop" }] }],
    "SessionEnd": [{ "hooks": [{ "type": "command",
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh end" }] }]
  }
}
```

`journal.sh` reads the stdin JSON (`jq`), takes `session_id`, `cwd`, `git -C "$cwd" branch
--show-current`, `git diff --stat`, and appends a JSONL + Markdown line to
`<worktree>/.agent-runs/<session_id>/` (or to a shared directory outside the repo).

Maturity: a stable, documented API; the list of events grows quickly between versions.

### 1.2 Transcript files and the link to a PR

Source: https://code.claude.com/docs/en/sessions

- Path: `~/.claude/projects/<project>/<session-id>.jsonl`, `<project>` = the cwd path with
  non-alphanumeric characters replaced by `-`. Subagents: `<session-id>/subagents/agent-<id>.jsonl` +
  `.meta.json` (observed locally).
- The docs explicitly warn: **the format is internal and changes between versions**; for scripts
  use `/export`, `claude -p --output-format json|stream-json`, `transcript_path` from hooks, the
  Agent SDK.
- Retention is 30 days by default: `cleanupPeriodDays`. For an archive — copy in SessionEnd.
- Line types observed locally (undocumented): `user`, `assistant`, `system`,
  `attachment`, `ai-title`, `pr-link` (with `prUrl`, `prNumber`, `prRepository`), `file-history-*`,
  `cost-state`; fields `gitBranch`, `cwd`, `totalLinesAdded/Removed`, `totalCostUSD`.
- **Built-in session↔PR link**: `claude --from-pr <number>` opens a picker filtered to the
  sessions linked to the PR; a PR URL can be pasted into the picker to find the session that
  created it; `Ctrl+W` — all worktrees of the repository, `Ctrl+B` — filter by the current branch.
- Naming: `claude -n <name>`, `/rename`; otherwise a generated title. `claude agents --json`
  lists running sessions.
- `/export [file]` — a readable plain-text transcript (with tool output).
- Summarizing an existing session by script:
  `claude -p --resume <session-id> --output-format json "summarize what we changed" | jq -r .result`.

### 1.3 Statusline

Source: https://code.claude.com/docs/en/statusline

stdin JSON: `session_id`, `session_name`, `transcript_path`, `cwd`, `model`, `workspace`
(`current_dir`, `project_dir`, `git_worktree`), `worktree`, `version`, `cost`
(`total_cost_usd`, `total_lines_added`, …), `context_window`. Updated on events (300 ms debounce)
+ `refreshInterval`. Useful for seeing the task/branch in every parallel tab; a heartbeat can be
appended to a file, but this is a side channel, not a journal.

```json
{ "statusLine": { "type": "command", "command": "~/.claude/statusline.sh", "refreshInterval": 10 } }
```

### 1.4 OpenTelemetry

Source: https://code.claude.com/docs/en/monitoring-usage

```bash
export CLAUDE_CODE_ENABLE_TELEMETRY=1
export OTEL_METRICS_EXPORTER=otlp        # otlp|prometheus|console|none
export OTEL_LOGS_EXPORTER=otlp           # otlp|console|none
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
# content is hidden by default:
export OTEL_LOG_USER_PROMPTS=1 OTEL_LOG_TOOL_DETAILS=1
# traces (beta):
export CLAUDE_CODE_ENHANCED_TELEMETRY_BETA=1 OTEL_TRACES_EXPORTER=otlp
```

Metrics: `claude_code.session.count`, `lines_of_code.count`, `pull_request.count`,
`commit.count`, `cost.usage`, `token.usage`, `code_edit_tool.decision`, `active_time.total`.
Events: `claude_code.user_prompt`, `api_request`, `assistant_response`, `tool_decision`,
`tool_result`, and others. Attributes: `session.id`, `vcs.*` (with `OTEL_METRICS_INCLUDE_REPOSITORY=1`).
Traces: spans `claude_code.interaction` → `llm_request`, `tool`.

What it gives the owner: counters and timings per session, but **not a readable report** — a local
collector + backend is needed (for example, otel-collector → file/ClickHouse/Grafana). For a solo
owner this is excessive compared with hooks; useful as a secondary metrics channel.

---

## 2. OpenAI Codex CLI

### 2.1 Hooks

Source: https://developers.openai.com/codex/hooks (redirects to https://learn.chatgpt.com/docs/hooks)

- Events: `SessionStart`, `SessionEnd`, `SubagentStart`, `UserPromptSubmit`, `PreToolUse`,
  `PermissionRequest`, `PostToolUse`, `PreCompact`, `PostCompact`, `SubagentStop`, `Stop`,
  `Interrupt`.
- Common fields: `session_id`, `transcript_path` (may be null), `cwd`, `hook_event_name`,
  `model`; turn-scoped — `turn_id`; tool — `tool_name`, `tool_input`.
- Where: `~/.codex/hooks.json` or `~/.codex/config.toml`; **project-level
  `<repo>/.codex/hooks.json`** or `.codex/config.toml`; plugins. Enabled by default
  (disable with `[features] hooks = false`). Entire notes: hooks have been enabled by default
  since codex-cli 0.124.0.
- The format is similar in spirit to Claude Code (`matcher` + `hooks[{type:"command",command}]`) —
  one journal script can be reused, distinguishing by `hook_event_name`/fields.

```json
{ "hooks": { "Stop": [ { "hooks": [ { "type": "command",
  "command": "python3 .codex/hooks/journal.py" } ] } ] } }
```

### 2.2 `notify` (legacy mechanism)

Source: https://learn.chatgpt.com/docs/config-file/config-advanced

```toml
notify = ["python3", "/path/to/notify.py"]
```
Only the `agent-turn-complete` event; the JSON is passed as an **argument**, with fields
`thread-id`, `turn-id`, `cwd`, `input-messages`, `last-assistant-message`. Suitable as a fallback.

### 2.3 Session/rollout files

- `~/.codex/sessions/YYYY/MM/DD/rollout-<timestamp>-<uuid>.jsonl` (observed locally; the docs
  say only "sessions and rollout files persisted", the path in the docs — unconfirmed).
- The first line is `session_meta` with fields `id`, `cwd`, `git`, `cli_version`, `source`,
  `parent_thread_id`, `forked_from_id`…; then `response_item` (message, reasoning,
  function_call/_output, custom_tool_call), `event_msg` (task_started/task_complete, token_count),
  `turn_context`. The format is internal.
- `~/.codex/history.jsonl` — prompt history (`[history] persistence`).
- `codex exec --ephemeral` disables writing.

### 2.4 `codex exec --json` and OTel

Source: https://learn.chatgpt.com/docs/non-interactive-mode (Codex non-interactive section)

- `codex exec --json` → JSONL: `thread.started`, `turn.started`, `turn.completed` (with usage),
  `item.started`, `item.completed` (agent_message, reasoning, command_execution, file_change,
  mcp_tool_call, web_search, plan update), `error`.
- `-o/--output-last-message <path>` — the final message to a file; `--output-schema <schema.json>`
  — the final answer per a JSON Schema (handy for a structured "session report").
- `codex exec resume <SESSION_ID>`.
- OTel: `[otel] exporter = "none"|"otlp-http"|"otlp-grpc"`, `log_user_prompt = false`,
  `environment`. Events in the sources: `codex.tool_result` and others
  (https://github.com/openai/codex/blob/main/codex-rs/otel/src/tool_result.rs); the full list in the
  docs — unconfirmed.

---

## 3. Session capture tools

### 3.1 Entire CLI (Checkpoints)

Source: https://github.com/entireio/cli (README), https://entire.io/blog/hello-entire-world

- What it is: on every `git commit` it saves a checkpoint: transcript, prompts, affected files,
  tokens, tool calls, subagents; optionally an AI summary ("intent, outcome, learnings, friction
  points, open items") — almost exactly what the owner needs.
- Where: **separate refs `refs/entire/checkpoints/<shard>/<id>`** (the ID is a 26-character ULID).
  Older versions wrote to the branch `entire/checkpoints/v1` (per third-party reviews; the current
  README describes refs). While working — a local shadow branch (never pushed). No commits are
  created on the working branch.
- Link to commit/PR: the trailer **`Entire-Checkpoint: <id>`** in the commit message → via a PR
  the link is made automatically through the commits.
- Push: checkpoint refs go along with `git push` to one chosen remote
  (`push_sessions`, `checkpoint_push_remote`), or to a separate repo (`checkpoint_remote`),
  `--skip-push-sessions` — local only.
- Agents: Claude Code, Codex (installs `.codex/hooks.json`), Cursor, Copilot CLI, OpenCode, Gemini,
  Droid, Antigravity, Pi. Installs hooks into the agents' native configs.
- Commands: `entire enable [--agent claude-code]`, `entire status`, `entire checkpoint
  list|explain [--generate]|search|tokens`, `entire recap`, `entire dispatch`, `entire session
  resume <branch>`, `entire disable`.
- Config: `.entire/settings.json` (shared) + `.entire/settings.local.json`;
  `strategy_options.summarize.enabled`, `summary_generation.provider = claude-code|codex|…`.
- Secrets are redacted in transcripts; but code snapshots on the shadow branch are raw.
- MIT license; works locally/offline; the entire.io cloud is optional. Maturity: v0.11.x,
  daily nightlies (latest 2026-09-26), ~5k stars — active, but the API is pre-1.0.
- Limitations: the summary spends agent tokens; a checkpoint appears only on commit (work without
  a commit is visible only on the shadow branch); the GitHub UI does not show refs — read via
  the CLI/entire.io.

### 3.2 git-ai

Source: https://github.com/git-ai-project/git-ai, standard
https://github.com/git-ai-project/git-ai/blob/main/specs/git_ai_standard_v3.0.0.md

- What it is: line-by-line attribution of AI code (agent, model, session, prompt) → `git ai blame`,
  `git ai stats [--json] <a>..<b>` (% of AI code, accepted lines, overrides by model).
- Where: **Git Notes** (`git log --show-notes="ai"`); the prompts/sessions themselves are
  **outside git** (redacted, local). No git hooks and no git wrapper.
- Agents: Claude Code, Codex, Cursor and others; local-first, no login. Apache-2.0, v1.7.5
  (2026-09-09), ~2.8k stars. The paid "Git AI for Teams" is optional.
- Tells "who wrote the lines", but not "what was verified/what remains". Implements/supports Agent Trace.

### 3.3 Agent Trace (specification)

Source: https://agent-trace.dev (repo `cursor/agent-trace` — a link from the site; as of 2026-09-27
GitHub returned 404; unconfirmed whether the repo has moved).

- RFC v0.1.0 (Jan 2026), Cursor + Cognition, supported by Cloudflare, Vercel, Jules, Amp,
  OpenCode, git-ai. CC BY 4.0.
- JSON record: `version`, `id`, `timestamp`, `vcs` (type+revision), `tool`, `files[]` →
  `conversations[]` → `ranges` + `contributor` (human/ai/mixed/unknown), `metadata`.
- **Storage is not defined** (files, git notes, a DB). This is an attribution format, not a session
  journal. Useful as a schema for the "authorship" field in one's own artifact, if compatibility
  is needed.

### 3.4 SpecStory

Source: https://github.com/specstoryai/getspecstory

- Saves every session as Markdown in **`<repo>/.specstory/history/`** (local-first);
  the cloud `cloud.specstory.com` is optional (sync/search/share).
- Launched through a wrapper: `specstory run claude` / `specstory run codex` (brew
  `specstoryai/tap/specstory`). The CLI is open source (Apache-2.0), v2.15.1 (2026-09-24).
- Pro: readable .md files next to the code, can be committed. Con: no binding to commits/PRs, no
  notion of checks; a launch wrapper is needed (or agent support via their files — details
  unconfirmed).

### 3.5 claude-code-log

Source: https://github.com/daaain/claude-code-log

- A converter of `~/.claude/projects/**.jsonl` → HTML/Markdown, an index by project, per-session
  pages, tokens, TUI, `watch`, `serve --watch`. Experimental: `--provider codex
  --session-id <id>`.
- `uvx claude-code-log@latest --open-browser`; `--detail full|high|low|minimal|user-only`,
  `--format md`.
- MIT, v1.6.0 (2026-08-31), active. Depends on the internal transcript format → may
  break on new versions of Claude Code.

### 3.6 ccusage

Source: https://github.com/ccusage/ccusage — `npx ccusage` (daily/session/blocks, JSON),
v20.0.26 (2026-09-27). Only tokens/cost per session; not a work journal. The license on GitHub
is marked "Other" (which one — unconfirmed).

---

## 4. Git-native linking

- **Commit trailers**: `git interpret-trailers` (https://git-scm.com/docs/git-interpret-trailers).
  Proposal: `Agent-Session: claude:<session_id>`, `Task: #123`, `Evidence:
  .agent-runs/<id>/report.md`, `Co-Authored-By: …`. Trailers are visible in a PR (commit
  messages), survive rebase/squash (on squash — if the body is kept). They can be added by a
  `prepare-commit-msg` hook or by an instruction to the agent; Entire already does `Entire-Checkpoint:`.
  Search: `git log --format='%(trailers:key=Agent-Session,valueonly)'`.
- **Git notes** (https://git-scm.com/docs/git-notes): metadata without changing the SHA, can be
  appended after the commit. Cons: `refs/notes/*` are not pushed/fetched by default (a refspec is
  needed), they are lost on rebase without `notes.rewriteRef`/`notes.rewrite.rebase`, the GitHub UI
  does not show notes. git-ai solves part of this itself.
- **Separate refs** (like Entire): do not interfere with history, independent push/fetch, but
  invisible in the UI.
- **Branches**: `agent/<task-id>-<slug>` + a worktree with the same name → `gitBranch` in the
  transcript and `Ctrl+B` in the picker immediately group sessions by task.
- **A folder per task** in the repo (for example, `docs/evidence/<task>/`) — visible in the PR
  diff, but clutters history; the alternative is outside the repo (`~/.agent-runs/<repo>/<branch>/`),
  and pasting a link/digest into the PR.

## 5. Local proof of checks without CI

- **Playwright** (https://playwright.dev/docs/test-reporters,
  https://playwright.dev/docs/test-use-options):
  ```ts
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }],
             ['json', { outputFile: 'test-results/results.json' }]],
  use: { trace: 'retain-on-failure', video: 'retain-on-failure', screenshot: 'only-on-failure' }
  ```
  `npx playwright show-report [folder|zip]` — a local server with traces/video;
  `merge-reports` for blob.
- **Vitest** (https://vitest.dev/guide/reporters): built-in `json`, `junit`, `html` (a static
  Vitest UI, no separate package), `blob`, `minimal` (alias `agent`).
  ```ts
  test: { reporters: ['default', 'json', 'junit'],
          outputFile: { json: '.evidence/vitest.json', junit: '.evidence/junit.xml' } }
  ```
- **Evidence folder**: a `PostToolUse(Bash)` hook catches test commands (`pnpm test`, `playwright`),
  copies/links the reports into `.agent-runs/<session>/evidence/`; view with `python3 -m http.server`
  or `npx serve`. An index page is generated by a script from JSON (session, branch, task,
  checks[{cmd, exit, report}], diffstat, open items).
- Aggregators: Allure / Monocart (custom Playwright reporters), JUnit viewers — mentioned in the
  Playwright docs as third-party; a ready-made "dashboard of local agent runs" was not found in
  primary sources (unconfirmed).

## 6. Conclusion for the harness

A minimal setup without external services: project hooks (Claude `.claude/settings.json` + Codex
`.codex/hooks.json`) write `session.json`/`report.md` into a per-session directory; test runners
put JSON/HTML reports in the same place; commits get an `Agent-Session:` trailer; on `gh pr create`
a script inserts a digest and links into the PR description. Claude Code itself already links a
session to a PR (`--from-pr`). Entire covers "transcript + summary + binding to a commit" out of
the box, git-ai — line-by-line attribution; both are open source and run locally.
