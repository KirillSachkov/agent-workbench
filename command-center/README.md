# Command center

The command center of agent-workbench, slice 1 "see and check": a Herdr plugin with one binary,
`workbench-cc`. It joins facts from Herdr, git and GitHub and keeps no task state of its own.

- Every agent in Herdr is bound to its **project** (the main git worktree), branch, worktree, PR and
  task. Linked worktrees are grouped under their repository.
- The agents sidebar shows a short tag per agent (`#62 PR81 ✓`: task, PR, CI), and the tab bar
  shows how many things need you (`2 need you`).
- One key opens the **Overview**: what needs you, what changed since you last looked, every
  project's agents with task → PR → CI, and each project's work (specs and maps with progress, work
  ready now, work in flight).
- Enter on an agent, or a key on its pane, opens its **result card**: the report the agent wrote
  into the PR plus derived facts, with one key each for a "look first" file at its line in nvim,
  the diff, the PR and the running app.

It works in any repository, with or without the agent-workbench harness, at the detail level its
facts allow (ADR 0006): a folder with an agent shows its status; a git repository adds branch and
diff; a GitHub repository adds PR, CI, review and the linked task; a PR written in the result-card
shape adds the card sections.

## Install

Slice 1 builds from source, so the machine needs a Rust toolchain (`cargo`).

```bash
herdr plugin install KirillSachkov/agent-workbench/command-center
herdr plugin list                 # shows the directory the plugin was installed into
cd <that directory>
bin/workbench-cc doctor           # checks Herdr, gh and its login, git and the editor
bin/workbench-cc configure        # shows the plan and asks before writing
```

`configure` proposes the key bindings, the tab-bar counter and the sidebar row. It checks the keys
against Herdr's default bindings, your own `[keys]` and every `[[keys.command]]` (other plugins'
bindings included) and refuses a conflict. It writes only between the markers
`# >>> agent-workbench command center … >>>` and `# <<< agent-workbench command center <<<`, keeps a
backup next to the file (`config.toml.workbench-cc.bak`) and leaves your own `tab_bar_right` and
sidebar rows alone, printing what to add by hand instead. Default keys: `prefix+i` for the Overview
and `prefix+u` for the card of the focused agent; change them with `--overview-key` / `--card-key`
or in the config file below. `bin/workbench-cc unconfigure` removes everything `configure` wrote.

## Keys in the popups

| Overview | | Result card | |
|---|---|---|---|
| ↑ ↓ | select an agent | 1–9 | open a "look first" file at its line |
| Enter | the agent's result card | d | the branch diff against its base |
| f | go to the agent's pane | o | the PR in the browser |
| o | the PR in the browser | a | the app address from "How to try" |
| p | only this project | f | go to the agent's pane |
| r | refresh | ↑ ↓ | scroll |
| q | close | q | back |

Files and the diff open in a split beside the agent's pane, in its worktree.

## The result card in a PR

The card reads these sections from the PR body by heading (any level): `Result`, `Needs you`,
`Look first` and `How to try`. A `Look first` line is `path:line — reason` (a list item; the path
may be in backticks; `–` or `-` also separate); other lines are ignored, and paths must be
relative and inside the worktree. The first `http(s)` address
under `How to try` is the app. A body without these sections is shown whole. The forked `pr` skill
writes this shape (#55); any other harness can follow it too.

Derived facts: CI rollup, review decision, head commit, files and lines changed, whether test files
changed (`test/`, `tests/`, `__tests__/`, `spec/`, `*_test.*`, `*.test.*`, `*.spec.*`, `test_*`) and
whether CI configuration changed (`.github/workflows/`, `.gitlab-ci.yml`, `.circleci/`,
`.buildkite/`, `Jenkinsfile`, `azure-pipelines.yml`, `.travis.yml`). An agent without a PR gets a
card with its branch and the files changed since the merge base, committed or not.

## What it reads and what it never does

- Herdr through its CLI (`agent list`, `pane get`, `pane report-metadata`, `plugin pane open`,
  `agent focus`; for lanes also `worktree create`, `agent start`, `agent wait` and
  `notification show`), git in the agent's working directory, GitHub through `gh` (read only,
  except the one claim `lane start` makes). Never agents' internal session files.
- It never types into an agent's prompt and never runs commands found in PR text: PR bodies are
  parsed into fixed fields, the app address is opened in the browser only, and files open through
  an argv array without a shell.
- Before acting on a pane it checks the pane still holds the expected agent session and directory.
- It writes nothing into agents' user-level configuration. Its own files are display state in the
  plugin state directory: the GitHub cache and the snapshot from when you last opened the
  Overview.
- When GitHub is slow or offline, the last known facts are shown marked as unreachable.

## Configuration

`config.toml` in the plugin's config directory (`herdr plugin config-dir agent-workbench.command-center`).
Every field is optional:

```toml
editor = ["nvim", "+{line}", "--", "{path}"]  # argv; {path} and {line} are replaced
diff = ["nvim", "-c", "CodeDiff {base}..."]   # {base} is the base branch, e.g. origin/main
browser = ["open", "{url}"]                   # xdg-open on Linux
github_refresh_seconds = 60                   # GitHub facts are reused this long
github_timeout_seconds = 20

[keys]
overview = "prefix+i"
card = "prefix+u"

[lanes]
max = 2                              # lanes running at once per project
branch = "feat/{issue}-{slug}"       # {slug} comes from the issue title
[lanes.args]                         # per agent kind; {prompt} is the one-line first prompt
claude = ["{prompt}"]                # built in: claude, codex ["{prompt}"], opencode ["--prompt", "{prompt}"]
```

## Commands

Every popup key and Herdr action calls one of these; `--json` prints the same model the popups show.

```text
workbench-cc overview [--json] [--this-project] [--mark-seen] [--offline]
workbench-cc card [--pane ID] [--json]
workbench-cc open file|diff|pr|app --pane ID [--look-first N | --path P --line N]
workbench-cc focus --pane ID
workbench-cc refresh [--force]      # sidebar tags; run on Herdr start and on agent events
workbench-cc tab-status             # the tab-bar counter
workbench-cc configure | unconfigure | doctor
workbench-cc lane start <issue> --agent <kind> [--brief FILE] [--branch B] [--over-limit] [--json]
workbench-cc lane watch <lane> [--timeout S] [--start-timeout S] [--json]
workbench-cc lane list [--this-project] [--json]
```

## Lanes

A lane is one executor agent working on one issue in its own worktree and Herdr workspace. The
`conduct` skill (in this repository's `skills/`) proposes lanes, and after the owner's go-ahead runs
these commands; they can also be run by hand. There is no lane store: a lane is an agent in a linked
worktree whose branch names its issue (`feat/70-…`), found again from Herdr, git and GitHub.

- `lane start` checks the issue is open and unclaimed, that it has no lane yet and that the project
  runs fewer than `[lanes] max` lanes (`--over-limit` when the owner says so). It then calls
  `herdr worktree create` on the project (branch from `[lanes] branch`, base `origin/HEAD` or
  `origin/main`), writes the executor's brief into the state directory (`lanes/<owner>__<repo>-<issue>.md`),
  starts the agent in the workspace's root pane with `herdr agent start <repo>-<issue>` and the
  one-line prompt "Read and follow the lane instructions in <brief>", and claims the issue with
  `gh issue edit --add-assignee @me`. Herdr rejects agent arguments it cannot encode safely, so the
  instructions live in the file, not in the argument. An agent that stops at a question during
  startup (a folder trust prompt) is reported as `waits for you` and left for the owner. When the
  start fails the workspace and worktree stay as they are.
- `lane watch` waits through `herdr agent wait` until the agent stops (`idle`, `done` or `blocked`;
  an idle lane first gets `--start-timeout` to start working), checks the pane still holds the same
  agent session, then shows `herdr notification show` with a sound: `done` when the lane has a PR,
  `request` when it waits at a question or approval, stopped without a PR, or its agent is gone.
- `lane list` shows the running lanes with issue, agent, status and PR. The Overview's Work section
  shows the same agent and status beside each issue in flight.

**Turn limits.** Executors start interactive, so the owner can answer them in their pane. Claude
Code's `--max-turns` and `--max-budget-usd` work only in print mode (`-p`), and Codex and OpenCode
have no interactive equivalent, so lanes start without a turn limit (decision 2026-09-30). The
watch and the owner's notification are the guard; a runtime flag can be added per kind under
`[lanes.args]` once one exists.

Lanes never merge, push, delete a branch or worktree, answer an agent's question or approval, or
type into a working agent.

## Development

Tests run the built binary as a black box with fake `herdr` and `gh` first on `PATH` (they answer
from fixtures and log every call) against temporary git repositories: `cargo test`. The popups are
checked with rendered snapshots under `tests/snapshots/`; rerun with `UPDATE_SNAPSHOTS=1` after an
intended change.
