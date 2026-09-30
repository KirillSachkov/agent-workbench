# How the leading Herdr plugins are built

Date: 2026-09-30. Question from #59 (CC15, part of map #41, the command center): how are the leading
Herdr plugins built, and what can the command center borrow from them? This note goes one level
below [the CC1 note](https://github.com/KirillSachkov/agent-workbench/blob/6513a00/docs/research/2026-09-29-herdr-hosting.md)
(branch `research/42-herdr-hosting`), which covered what a plugin can host and which jobs each
plugin covers. It does not repeat CC1's host-surface table, its star and bus-factor figures, or its
job coverage. It reads how the plugins are built inside: architecture, model, UX, and how they
coexist.

Method.

- **Plugins.** Twelve repositories were downloaded as tarballs at their default-branch head on
  2026-09-30 with `gh api repos/<repo>/tarball/<sha>`. Their `herdr-plugin.toml` files were parsed,
  and their READMEs, docs and the source files that talk to Herdr were read. The ten plugins the
  ticket names are joined by two small plugins that write a `pr` token to the sidebar
  (herdr-github-metadata and herdr-plugin-gh-pr), because they are the public examples of the
  "fact on a pane token" pattern. plannotator-tui, which herdr-annotate bundles, was read only in
  its README.
- **Herdr.** `docs/versions/0.9.2/website/src/content/docs/socket-api.mdx` at `4b4705d`, the same
  commit CC1 used. The API schema came from `herdr api schema --json` on the local `herdr 0.9.1`
  (protocol 22, schema version 1).
- **Marketplace.** `https://assets.herdr.dev/plugins/index.json`, `generatedAt`
  2026-09-30T06:30Z: 1,430 manifests in 1,389 repositories. It was used only to look for further
  plugins that bind sessions to worktrees or PRs.
- **Local machine.** Only read-only commands were run: `herdr --version`, `herdr plugin list`,
  `herdr api schema --json`. No plugin was installed, no config changed, and no input was sent to
  agents.
- **Owner's pattern.** The owner's agent-context plugin is private. It is described only from this
  repository's public [command-center-build-options §1.1](2026-09-29-command-center-build-options.md)
  (commit `2a9b7de`) and from public Herdr and plugin sources.

Citation shorthand: `<repo>@<sha7>:<path>` means
`https://github.com/<owner>/<repo>/blob/<sha>/<path>`, with the owner and full SHA from the table
below. `D:socket-api` means
`https://github.com/herdrdev/herdr/blob/4b4705d/docs/versions/0.9.2/website/src/content/docs/socket-api.mdx`.
`schema` means the local `herdr api schema --json` from 0.9.1. Stars are from `gh api` on
2026-09-30.

| Short name | Repository | Commit | ★ | License | Language |
|---|---|---|---|---|---|
| herdr-projects | eliasstravik/herdr-projects | `4e4548c3c43888e6be1c96906215998f07dc63f0` | 523 | MIT | Rust |
| radar | hhdebb/herdr-radar | `2425fa080b7b72f30c1299481282b23003a2ee61` | 110 | MIT | JavaScript (Node) |
| reviewr | persiyanov/herdr-reviewr | `cd618b48dc1f4d2cc092f3f26827ac72fb44773e` | 794 | MIT | Rust |
| annotate | plannotator/herdr-annotate | `cbba4732229191347ff5128e3da71f64474a6a49` | 587 | MIT | Rust + shell |
| navigator | thanhdat77/herdr-navigator | `313b753b1f98f802b7d861430a13cb95a86e8dd3` | 175 | MIT | Rust |
| sessionizer | andrewchng/herdr-sessionizer | `c3a7c88a5a79f7515471650d2fcbe6cded6b6a44` | 49 | MIT | TypeScript (Bun) |
| board-np | nelsonPires5/herdr-board | `d7a67470309c378778e388982bda67eee6dc1a58` | 163 | none detected | Rust |
| board-bb | bredebjorhovd/herdr-board | `648e8994ba96730560ec2dec2f07f36d6a9e2cc6` | 1 | none detected | Rust |
| lantern | aigorahub/herdr-lantern | `e356848f0a50159cf3a1386125fa38d94e5fe702` | 67 | MIT | shell + Python |
| tsk | smarzban/tsk | `fbe0847469e1576676af2e8f6b67588267c548ba` | 156 | MIT | Rust |
| gh-metadata | ralphilius/herdr-github-metadata | `32791c7aca30b7a06cbac545351721a6d3966815` | 1 | none detected | Python |
| gh-pr | wyattjoh/herdr-plugin-gh-pr | `6fe22de9a90c569f2186595cfddc3707f55ba1bd` | 22 | none detected | TypeScript (Bun) |

---

## 1. Architecture

### 1.1 Summary table

The manifest columns come from each plugin's `herdr-plugin.toml`. The other columns come from the
sources cited in §1.2.

| Plugin | Manifest sections | Build / install | Talks to Herdr through | Background process | Own state and config |
|---|---|---|---|---|---|
| herdr-projects | build, startup, 4 popup panes, 10 actions | `[[build]]` downloads the release binary, checks SHA256SUMS, falls back to `cargo build` | CLI for everything; raw socket only for `agent.view.set` | a **ticker** started from `[[startup]]` with `setsid`, one per projects root, every 15 s | a project is a folder of Markdown and TOML in `~/.herdr-projects/`; its own config in `~/.config/herdr-projects/` |
| radar | build, 3 startup commands, 1 popup, 11 actions, 1 event | `[[build]]` runs `node bin/setup.js`; needs Node 18 | `herdr agent list` snapshots and a read-only event subscription; raw socket for `agent.view.set` | a resident **daemon** woken by events | plugin state dir; its own config in the plugin config dir; three managed blocks in Herdr's `config.toml` |
| reviewr | build, 1 split pane, 3 actions, 2 events | `[[build]]` downloads the release binary by manifest version and verifies its checksum | CLI (`pane list`, `pane process-info`, `agent list`, `pane send-text`) | none; the pane polls `agent list` every 2 s | plugin config dir `config.toml` |
| annotate | build, 2 popups + 1 overlay, 11 actions, 1 link handler; plus `lite/` and `windows-full/` manifests | `[[build]]` fetches pinned prebuilt binaries and verifies SHA-256 | CLI (`agent get`, `agent prompt`, `pane send-text`) | none | JSONL stores in the plugin state dir |
| navigator | build, overlay + split pane, 3 actions | `cargo build --release`; install pinned with `--ref <tag>` | CLI (`workspace list`, `agent list`) | none | plugin config dir |
| sessionizer | build, 2 overlay panes, 2 actions | `bun install` then `bun run build` into a compiled binary | CLI (`worktree create`, `worktree open`) | none | plugin config dir, plus an optional per-repo `.sessionizer/config.toml` |
| board-np | build, 1 overlay pane, 1 action | `cargo build`, then copies a `board` CLI into `~/.local/bin` | its **own socket client**; refuses any protocol other than 22 | `boardd`, which the CLI auto-starts | SQLite (schema v15) in `~/.local/share/herdr-board/` |
| board-bb | build, startup, split + popup, 3 actions, 3 events, 2 link handlers | `cargo build` | CLI; every argv is logged | `syncd`, a daemon ensured from `[[startup]]` | SQLite `board.db` in the plugin state dir; `.env` with API tokens in the plugin config dir |
| lantern | 1 tab pane, 2 actions, no build | shell scripts and Python 3 | CLI through an "inspect-by-default" wrapper | a helper agent chat, plus a field-status watcher pane | plugin config dir (`helper.conf`, `prompt.md`) |
| tsk | build, 1 split pane, 4 actions | the main install is `curl … \| sh` or Homebrew; `tsk setup herdr` then runs `herdr plugin link` | CLI | none | JSON store in the plugin state dir |
| gh-metadata | startup, 1 action | none (Python stdlib) | CLI `pane report-metadata` | a watcher from `[[startup]]`, every 2 s | none |
| gh-pr | 2 actions, 3 events | none; hooks run with `bun` | CLI `pane report-metadata` | none ("No daemon, no config") | none |

### 1.2 Evidence and details

- **Language.** Seven of the twelve are Rust. radar is Node, sessionizer and gh-pr are Bun
  TypeScript, and lantern and gh-metadata are shell and Python (table above, from each repository's
  language and manifest).
- **Binary or script.**
  - The larger Rust plugins ship prebuilt binaries from GitHub Releases. The `[[build]]` step
    downloads the binary named by the manifest version and verifies a checksum:
    - `herdr-projects@4e4548c:scripts/install.sh` (header comment; `cargo` fallback)
    - `herdr-reviewr@cd618b4:herdr/install.sh`
    - `herdr-annotate@cbba473:scripts/fetch-herdr-annotate.sh`
  - Others build from source at install time with `cargo build --release` (navigator, board-bb,
    tsk, board-np), which needs a Rust toolchain on the machine (manifests).
  - `herdr plugin link` skips `[[build]]`. Plugins that must set things up therefore repeat setup on
    first start: radar's daemon launcher "runs the same checks on every startup, so a plugin linked
    from a checkout … sets itself up on the first Herdr start instead"
    (`herdr-radar@2425fa0:bin/setup.js`, header). Also `herdr-reviewr@cd618b4:herdr/install.sh`,
    header.
- **How they talk to Herdr.**
  - Most shell out to the `herdr` CLI. Its output is JSON by default, and errors are one JSON object
    with `error.code` and exit status 1 (`herdr-projects@4e4548c:docs/herdr-notes.md`, stages 1–2).
  - `agent.view.set` has **no CLI command in 0.9.1**. herdr-projects and radar both write one JSON
    line to the socket for it (`herdr-projects@4e4548c:docs/herdr-notes.md` stage 4;
    `herdr-radar@2425fa0:lib/view.js`, header comment).
  - board-np is the only one with its own typed socket client (`crates/board-herdr`). It gates on
    socket protocol 22 and warns rather than refuses on other Herdr versions
    (`herdr-board@d7a6747:README.md` "Supported harnesses"; `docs/herdr.md` "Compatibility gate").
  - board-bb routes every Herdr call through one file that logs each argv, "the only file to touch
    if a herdr verb ever differs from expectation" (`herdr-board@648e899:README.md` "Notes on the
    herdr integration").
- **Background processes.**
  - herdr-projects starts its ticker from `[[startup]]`. A detached `setsid` child survives
    (`docs/herdr-notes.md` stage 2), and a second session with the plugin linked "found a ticker of
    the same version and started nothing" (stage 5). The ticker checks every 15 s, pull requests
    every two minutes, and remote machines once a minute (`docs/operations.md` "How it works").
  - radar's daemon treats events as "WAKE HINTS only — every frame's truth is a fresh agent-list
    snapshot … which makes replayed, throttled, or missed events all equally harmless". Its
    published tokens are "the durable record", so a restarted daemon reads them back
    (`herdr-radar@2425fa0:lib/daemon.js`, header). A `pane.agent_detected` hook acts as a
    watchdog that restarts the daemon (`herdr-plugin.toml`; `bin/agent-state.js` header).
  - board-bb's `syncd` polls Linear and GitHub, reconciles against Herdr panes, and writes
    `board.db`. SQLite in WAL mode is "the only bus between our processes — no sockets of our own"
    (`herdr-board@648e899:README.md` "How it fits together").
  - board-np's CLI auto-starts `boardd` on first use (`crates/board-cli/src/context.rs`, header).
- **PATH.** A Herdr server not started from a login shell gives plugins a minimal `PATH`.
  herdr-projects found `gh` missing for that reason and now appends Homebrew and `~/.local/bin`
  paths itself (`docs/herdr-notes.md` "After the build"). reviewr's `herdr/pane.sh` prepends
  `/opt/homebrew/bin:/usr/local/bin` for the same reason.
- **Getting a binary onto the user's PATH.** herdr-projects, reviewr and board-np put a command in
  `~/.local/bin` so agents and layouts can call it (`scripts/install.sh` → `link-command.sh`;
  `herdr/pane.sh`; board-np README "Supported harnesses").
- **Updates.** Herdr has no `plugin update` (CC1 §4). The plugins handle it themselves:
  - herdr-projects has `herdr-projects update`: fetch, install, `doctor --fix`, restart the ticker
    (`README.md` "Updating").
  - lantern checks the published manifest at light-up and offers the update, never silently
    (`README.md` "Updates").
  - tsk shows an "optional GitHub release check: a cached tag and a dim footer notice"
    (`tsk@fbe0847:src/update.rs`, header).
  - navigator and board-np tell users to install with `--ref <tag>`, and reviewr's release tag
    equals the manifest version (READMEs; `herdr/install.sh`).
  - annotate: "An install stays on the commit it came from; run the same command again to move to
    the current release" (`README.md` "Install").

## 2. Model: projects, tasks, threads, and how facts are learned

### 2.1 What each plugin means by "project", "task" and "thread"

| Plugin | Project | Task / unit of work | Where the task lives |
|---|---|---|---|
| herdr-projects | a folder `~/.herdr-projects/<slug>/` with `PROJECT.md`, `AGENTS.md`, `MEMORY.md`, `TASKS.md`, `threads/`, `inbox/`; `repos` is a list of repositories | a **thread**: one agent in its own worktree and branch (or a tab folder), started from a brief | thread records `threads/<id>.toml`; tasks in `TASKS.md`, which only the coordinator agent writes |
| board-np | "a folder/Git root holding an independent set of named boards"; non-Git directories use their canonical CWD | a **card** with a prompt, harness, model and target session/workspace; runs move it through columns | SQLite |
| board-bb | a route in `routing.toml` from issue source to Herdr workspace label, repo and runtime | a GitHub issue/PR or Linear issue; each dispatch is an **attempt** | GitHub/Linear are the source; attempts and a writeback queue in SQLite |
| tsk | the Git repository you are in, else a "desk" | a task with notes and steps; statuses such as open, ready, review and done | JSON store in the plugin state dir |
| radar | derived: one group per workspace, linked worktrees nested under their repository | none | none; tokens only |
| lantern | none; the helper agent reads the "field" | none; it reads each agent's `/goal` and recap lines from the screen | none |
| reviewr, annotate, navigator, sessionizer | the current workspace / worktree / repository | none | none |

Sources: `herdr-projects@4e4548c:docs/operations.md` "Where things live";
`herdr-board@d7a6747:README.md` "Why herdr-board?", "How it works";
`herdr-board@648e899:README.md` "Configure routing", "How it fits together";
`tsk@fbe0847:README.md` "Open the board", `src/store.rs` header;
`herdr-radar@2425fa0:README.md` "What you get"; `herdr-lantern@e356848:bin/goals-floor`.

### 2.2 Grouping agents and worktrees by repository

- **Herdr already knows the repository of a worktree.** In 0.9.1, `WorkspaceInfo.worktree` carries
  `checkout_path`, `is_linked_worktree`, `repo_key`, `repo_name` and `repo_root`, and `WorktreeInfo`
  carries `branch`, `path` and `open_workspace_id` (schema).
- radar groups by `worktree.repo_key` and hangs each linked worktree under the workspace of the
  same `repo_key` that is not linked (`herdr-radar@2425fa0:lib/state.js`, the grouping code just
  above `labels()`). Worktrees whose parent is not open stay at top level (`lib/frame.js`, `sortKeys`).
- herdr-projects and board-bb create worktrees with `herdr worktree create`, which also opens a
  workspace and "groups it under the parent repo in the spaces sidebar"
  (`herdr-board@648e899:README.md` "Notes on the herdr integration"). Herdr places them under
  `~/.herdr/worktrees/<repo>/<branch>`, and herdr-projects "records the path herdr returns and
  never builds it" (`herdr-projects@4e4548c:docs/herdr-notes.md` stage 3). sessionizer also uses
  `herdr worktree create|open` (`herdr-sessionizer@c3a7c88:src/ops/worktrees.ts`).
- herdr-projects groups by its **own** project, not by repository. It sorts agents by an
  `$hp_group` token in the agent view and moves each project's Spaces into one block with
  `workspace.move` (`src/grouping.rs`, header).
- board-bb's `init` walks Herdr workspaces, reads each git remote and writes one route per GitHub
  repository. It skips linked worktrees, because "those are attempts the board creates, not
  projects to route to" (`README.md` "Configure routing").

### 2.3 How they learn an agent's task, branch and PR

| Plugin | Task | Branch | PR |
|---|---|---|---|
| herdr-projects | it started the thread from a brief, so it knows | recorded when it created the worktree | the **first line of the agent's report**, `PR: <url>`, strictly validated; followed with `gh pr view` every two minutes; `gh pr list --author @me` for open PRs (`src/pr.rs`) |
| board-bb | the GitHub or Linear issue it dispatched | `branch_slug(task)`, one branch per task (`src/dispatch.rs`) | PR endpoints polled; review comments delivered to the author agent (README "Reviewing a pull request reaches the agent that wrote it") |
| reviewr | none | local git | derived from local git and read with `gh` GraphQL; "Nothing ever writes to a forge" (`src/forge.rs`, header) |
| radar | none | reads `.git/HEAD` directly, because Herdr's `branch` token "follows the pane's launch directory" and goes stale after a `git checkout` (`lib/git.js`, header) | none |
| gh-metadata | none | the pane's checkout branch | (1) the newest worktree-or-PR mention in the tail of the agent's **session transcript**, found through the session path the Herdr integration reports; (2) the branch's open PR; (3) if one agent works in the repository, its latest open PR (README "Resolution order") |
| gh-pr | none | the focused agent pane's branch | `gh` for that branch, re-evaluated on `pane.focused` and `worktree.*` events (README; manifest) |
| lantern | regex over the pane's recent screen for `/goal active`, `recap:`, `Goal:`, `Next:` and phrases like "may I merge" (`bin/goals-floor`, top; `agent read --source recent`) | none | none |

### 2.4 Self-reported progress versus derived facts

- **Self-reported.**
  - herdr-projects: "Agents report their own progress. `herdr-projects report --percent N
    --activity "..."` … writes one small JSON file per pane under `<root>/.progress/`", shown for
    five minutes. To make every agent report, `configure` installs hooks into the **user-level**
    settings of Claude Code, Codex, Droid, Gemini CLI and Copilot CLI and links an `autoproject`
    skill into their user skill directories (`docs/operations.md` "How it works";
    `src/setup.rs`, `HARNESSES`).
  - board-np: agents call the `board` CLI to comment and report their outcome, and the daemon then
    moves the card (`README.md` "How it works").
  - board-bb: dispatched agents learn the board's conventions from text that
    `integration install-conventions` writes **between markers into each runtime's global
    instruction file** (`~/.codex/AGENTS.md`, `~/.config/opencode/AGENTS.md`). The author chose an
    instruction file over a skill on purpose: "a skill is loaded when the agent decides it is
    relevant" (`src/conventions.rs`, header).
- **Derived.**
  - radar "does no detection of its own; it mirrors Herdr's verdict" (`README.md` Troubleshooting).
    For panes older than the plugin, it reads the tail of Claude Code and Codex session
    transcripts under `~/.claude/projects` and `~/.codex/sessions` to get a last-activity time
    (`lib/activity.js`).
  - reviewr derives "last turn" per worktree from the rest→work edge of the agents' Herdr status,
    polled every 2 s, so short turns can be missed (`src/turn.rs` header; README "Limitations").
- **Trust in Herdr's state.** board-bb reports that Herdr's stock Claude Code rule let a thinking
  or blocked Claude agent read as `idle` in its tests. It ships a local agent-detection override
  for that (`README.md` "Claude Code state detection"). This was not re-checked on 0.9.x, so it is
  **unconfirmed** for current Herdr.
- **Delivering text to agents is fragile.**
  - `herdr agent prompt` merged with half-typed user text on 0.9.1. herdr-projects therefore keeps
    nudges conservative: it prompts only an idle coordinator whose input box has looked empty for
    10 s (`docs/herdr-notes.md` stage 2; `docs/operations.md` "Nudges and notifications").
  - annotate first checks `herdr agent get` for readiness and refuses when the agent is blocked or
    launching (`rust/src/agent_delivery.rs`, header and `agent_ready`).
  - board-bb verifies that the pane "still holds the agent that wrote the PR … and the pane's cwd
    must still be that attempt's checkout" before it delivers a review (README).

## 3. UX

- **Sidebar.**
  - Plugins cannot own a sidebar region (CC1 §1.2). They write tokens and edit the user's
    `[ui.sidebar.*].rows` so the tokens render.
  - herdr-projects replaces Herdr's agent and Space rows with a **one-line card** (state icon, name,
    state word) and adds one dim sub-line, `$hp_sub` (for example `review · PR #4` or `~40%`), which
    is drawn only when non-empty.
  - Project heads are bold through a styling rule keyed on an **invisible mark** in the name:
    U+200B for coordinators, U+2800 for home Spaces. "A mark in the name, not a list of names in
    the config, because a client draws another machine's rows with its own config"
    (`herdr-projects@4e4548c:src/sidebar.rs` `agent_card`, `space_card`, `sub_row`;
    `src/grouping.rs` header).
  - radar puts the state mark, logo and title into **one token per state**, "because Herdr joins
    adjacent cells with `·` and one token means one colour" (`lib/managed-config.js`). It holds
    `done` until the pane is seen and `blocked` until the agent works again. Idle has three tiers,
    and rows older than two hours dim and sink (`README.md` "What the colours mean").
- **Agent view.** herdr-projects sorts by `hp_group` then `hp_rank`. `focus <slug>` filters to one
  project (`src/sidebar.rs` `default_view`, `project_view`). radar offers `active` (grouped by
  workspace, then by activity) and `recent` (flat). Herdr disables its own grouped/priority toggle
  while an override is active, "so whoever installs one owns the way back out"
  (`lib/view.js` header).
- **Tab bar.** herdr-projects adds a `tab_bar_right` command that prints
  `projects: 2 need you` (`src/setup.rs` `Spec`; README). radar adds the current directory. Herdr
  "drops the whole status area when it is one column too wide rather than truncating it", so radar
  caps its entry at 48 columns (`README.md` Troubleshooting).
- **Popups and panes.**
  - Popups are not panes: they have no pane id, are outside `pane list`, and do not get
    `HERDR_PANE_ID`, though they do get `HERDR_PLUGIN_CONTEXT_JSON`. herdr-projects therefore has
    the action capture the originating pane and hand it to the popup in a `handoff.json`
    (`docs/herdr-notes.md` stage 7). board-bb's picker popup "performs the dispatch and exits; the
    board picks the result up from the database" (README "Notes").
  - Global boards open as a **split beside** the current work, not as an overlay or tab: "A tab
    pins a global queue inside one workspace's layout; an overlay covers the work you are
    consulting the board about" (board-bb README). tsk and reviewr also default to `split`
    (manifests).
  - herdr-projects puts everything in one popup with sections (threads, tasks, inbox, routines,
    settings, memory). "Every key runs a CLI command; the popup can do nothing the CLI cannot"
    (`docs/operations.md` "The popup").
- **Key bindings.**
  - A manifest cannot declare keys; bindings live only in the user's config (`gh-pr@6fe22de:README.md`
    "Optional keybindings").
  - Some plugins leave binding to the user (navigator, sessionizer, board-np, annotate, radar, per
    their READMEs). Others write the binding themselves after checking for conflicts:
    - herdr-projects `configure` refuses a key that clashes and asks for `--key`
      (`src/setup.rs`).
    - `tsk setup herdr` plans edits, detects conflicts and asks (`src/setup.rs`, `edit_bindings`).
- **Notifications.** herdr-projects sends one notification per event, titled
  `<Project> · <thread>`, with a sound only for "needs you" and done events. `mute` silences a
  project except for errors (`docs/operations.md` "Nudges and notifications").
- **Keeping a narrow sidebar readable.**
  - One-line cards, with a sub-line only when it says something Herdr's state word does not
    (herdr-projects).
  - `trim_group_prefix` drops the workspace name from titles under a header that already shows it
    (radar README "Settings").
  - Split screens rank as one unit, via the `tab_key` sort (`lib/view.js`).
  - Stale rows dim and sink (radar).
  - Global boards collapse finished work into "DONE today, enter to expand" (board-bb README
    mock-up).

## 4. Coexistence

Nobody has published a test that runs these plugins together. The collisions below are read from
the code and Herdr's documented rules, so their practical impact is **unconfirmed**.

| Shared surface | Rule in Herdr | Who uses it | Consequence |
|---|---|---|---|
| Agent view | One active view; "A successful set atomically replaces the previous view"; a plugin-owned view dies when its plugin is disabled (D:socket-api "Agent view queries") | herdr-projects, radar | The last one to set wins. Both reapply from `[[startup]]`, so after a restart the order depends on which startup hook runs last |
| Sidebar rows in `config.toml` | One user-owned config; at most 16 rows | herdr-projects edits `rows` in place with `toml_edit` and records an ownership journal for `unconfigure`; radar owns `[ui.sidebar.*]` as a **marker-fenced block** and skips the block when the user already has such a table | Whichever runs `configure` first shapes the rows. radar rewrites its block on the next configure, which drops edits made inside it (radar README "Changed a setting") |
| Pane tokens | "Token maps are per-resource patches … The latest accepted update wins"; ≤16 keys per report, ≤32 per pane; ≤32 distinct sequenced sources per pane lifetime (D:socket-api "Agent state reporting") | herdr-projects prefixes tokens (`hp_sub`, `hp_group`); radar uses unprefixed `sort_key`, `ws_key`, `tab_key`; gh-metadata and gh-pr **both write `pr`** | Token names are one flat namespace per pane. gh-metadata and gh-pr overwrite each other's `pr` |
| Keys | One `[keys]` table | `prefix+a` is the default of herdr-projects' popup, annotate's capture, tsk's quick capture, and radar's suggested view flip; `prefix+t` of tsk's board and navigator | A direct clash. Only herdr-projects and tsk detect it before writing |
| Workspace order | Global; every client sees it | herdr-projects moves Spaces per project; radar's `reorder_workspaces` (off by default "because it changes the global Spaces order, which every connected client sees") | Two reorderers would fight |
| Tab bar | `tab_bar_right` is an array; too wide means nothing is shown | herdr-projects and radar each append one entry | They can coexist until the total width overflows |
| User-level agent config | not Herdr's | herdr-projects writes hooks into `~/.claude/settings.json`, `~/.codex/hooks.json` and others, plus skills; board-bb writes marker blocks into `~/.codex/AGENTS.md` and `~/.config/opencode/AGENTS.md`; tsk `setup` installs a skill into user skill directories | Several plugins can edit the same user files. Each fences only its own entries |

Sources: `herdr-projects@4e4548c:src/setup.rs` (module header, `HARNESSES`), `src/sidebar.rs`
(`config_edit`, `edit_rows`); `herdr-radar@2425fa0:lib/managed-config.js` (header), README
"Settings" and Troubleshooting; `herdr-github-metadata@32791c7:README.md`;
`herdr-plugin-gh-pr@6fe22de:src/main.ts` (`--token pr=`); annotate README "Use"; `tsk@fbe0847:src/setup.rs`
`BINDINGS`; `herdr-navigator@313b753:README.md` "Install"; `herdr-board@648e899:src/conventions.rs`.

**Combinations that fit together today.** Plugins that only open panes and actions do not touch the
shared surfaces above except keys: reviewr, annotate, navigator and sessionizer (manifests and the
metadata/view grep in §8). So one "sidebar owner" (radar *or* herdr-projects), plus these pane
plugins with keys chosen by hand, is the combination with the fewest collisions. This is an
inference, not a test.

## 5. The session ↔ worktree binding pattern through pane tokens

**What the owner's pattern does, from public notes.** On the owner's machine, `herdr agent list`
showed `tokens {repo, branch, worktree, base}` on agent panes. A private plugin writes them and
"stores the binding 'agent_session → exact git worktree'", with `review` (Hunk on merge-base) and
`files` actions ([command-center-build-options §1.1](2026-09-29-command-center-build-options.md)).
That note proposes it as a join key between Herdr sessions and git (§3 there).

**What public sources say about building this pattern safely.**

- **Pane ids are not stable identities.** "Pane, tab and workspace ids restart from `w1` after a
  server restart, so ids are reused for different panes across restarts." herdr-projects matches
  on "workspace id, tab id, working directory, agent name". Since 0.2.0 it also uses the
  `terminal_id` from `pane current`, which "tells a reused pane id after a restart from the old
  pane" (`herdr-projects@4e4548c:docs/herdr-notes.md` stages 2, 3 and "The 0.2.0 redesign").
- **The session id is exposed.** `agent list` carries `agent_session` (`{agent, kind, source,
  value}`) (same notes, 0.2.0). Official integrations report it through
  `pane.report_agent_session`, which also accepts a `resume_argv` so Herdr can resume that session
  after a restart (D:socket-api "Agent state reporting"; schema `PaneReportAgentSessionParams`).
- **Tokens are volatile.** "Token metadata is not restored after a server restart"
  (D:socket-api). A binding that must survive has to be kept elsewhere, for example keyed by
  session id in a state file, and re-emitted from `[[startup]]` or on `pane.agent_detected`. radar
  instead treats its tokens as the durable record *during* a server's life and reads them back
  when its daemon restarts (`lib/daemon.js` header).
- **The pane's cwd is sticky for Claude Code.** An agent running `cd` did not change the reported
  `cwd` in herdr-projects' test (other agents untested). `cwd` is the physical path, so symlinked
  folders need canonicalisation (`docs/herdr-notes.md` stages 2–3).
- **Branch goes stale.** Herdr's own `branch` token follows the launch directory. radar reads
  `.git/HEAD` on every tick instead (`lib/git.js`).
- **Tokens are per server.** Tokens set through `--machine` live on the remote server's panes. The
  local snapshot and the local agent view do not include them (`docs/herdr-notes.md` stage 6).
- **The public variants of "fact on a pane token":**
  - gh-metadata's `pr` token resolved from the transcript, then the branch, then `gh`, "never
    invented"
  - gh-pr's `pr` token from the focused pane's branch
  - herdr-projects' `hp_sub`

  No public plugin in the marketplace index was found that publishes `repo`/`branch`/`worktree`/
  `base` as a general binding for other tools to read. The search was for "agent context",
  "session … worktree" and "branch token" in names, descriptions and manifests (index.json,
  2026-09-30).

## 6. Patterns worth borrowing and traps to avoid

Patterns (each seen in at least one plugin above):

1. **A thin plugin over one binary.** The popup and actions only call CLI subcommands, and "the
   popup can do nothing the CLI cannot" (herdr-projects). The same binary serves humans, agents
   and hooks (board-np "One binary").
2. **Snapshot as truth, events as wake-ups.** Missed or replayed events do no harm (radar daemon).
   board-np additionally gates on protocol 22.
3. **Idempotent, reversible edits of user config.** Edits go through a TOML/JSONC syntax tree with an
   ownership journal (herdr-projects `owned.json`, `unconfigure`) or marker-fenced blocks (radar).
   Both refuse to replace a symlinked dotfile or write through the link instead
   (herdr-projects `src/setup.rs`; radar README "Settings").
4. **Conflict-checked key setup that asks first** (herdr-projects, tsk).
5. **A `doctor`.** It checks the Herdr version, tools on PATH, `gh auth`, the daemon and routes
   (herdr-projects, board-bb).
6. **Treat forge text as hostile.** Keep a fixed set of fields, never place comment bodies into
   prompts, and accept a `PR:` line only as an exact URL (`herdr-projects@4e4548c:src/pr.rs`
   header). board-bb uses per-PR watermarks and a wake latch to stop echo loops when agent and
   human share one GitHub identity (README).
7. **Verify identity before acting on a pane**: pane still exists, holds the expected agent, and its
   cwd is the expected checkout (board-bb, herdr-projects).
8. **Record paths Herdr returns; never build them** (worktree paths, socket paths via
   `herdr session list --json`) (herdr-projects notes).
9. **Show facts, not guesses.** Tokens disappear rather than being invented (gh-metadata).
   Unknown Herdr states map to "mid-turn" so they cannot fabricate an event (reviewr `src/turn.rs`).
10. **Read-only toward the forge by default** (reviewr "never posts"; board-bb `writeback = false`
    option).

Traps:

1. **Owning shared, single-slot surfaces** (agent view, workspace order, sidebar rows, common key
   defaults, unprefixed token names). They collide as soon as a second plugin does the same (§4).
2. **A private task store** that competes with the tracker: `TASKS.md`, SQLite boards and JSON
   stores in five of the plugins (§2.1). Only board-bb treats GitHub/Linear as the source, and it
   still keeps attempts and a writeback queue locally.
3. **Writing into user-level agent config** (hooks, global `AGENTS.md`, user skills) to teach
   agents a protocol (herdr-projects, board-bb, tsk). This conflicts with this repository's F26
   (`docs/decisions.md`).
4. **Reading internal session files** (radar's activity, gh-metadata's PR resolution,
   annotate/plannotator-tui's "recent replies", where the mechanism is **unconfirmed**).
   `AGENTS.md` "Known pitfalls" rules this out for our tool.
5. **Screen-scraping agent text** for goals and "needs you" (lantern). It is brittle across
   harness versions.
6. **Typing into agent prompts.** It can merge with a user's draft (Herdr 0.9.1), and a thread can
   impersonate the user to a coordinator (herdr-projects README "What if the safety settings
   aren't enough?").
7. **Build-time toolchains** (cargo, Bun, Node) on every machine, versus prebuilt,
   checksum-verified binaries (§1.2).
8. **A minimal PATH** under a Herdr server that was not started from a login shell (§1.2).

## 7. Implications for the command center

These are facts and options. The decisions are the owner's.

- **Where the center's Herdr face sits.** A center that renders in the sidebar through the agent
  view and rows competes with radar and herdr-projects for single slots. Options:
  - set no agent view and write only prefixed tokens (for example `wb_*`) that users may place in
    rows themselves, like gh-pr and gh-metadata;
  - own the view and rows explicitly, like radar and herdr-projects, and document that other
    sidebar plugins are incompatible;
  - keep the rich view in the center's own pane or popup and the web UI, and leave the sidebar
    alone.
- **Grouping by repository needs no own model.** `workspace.worktree.repo_key` and
  `is_linked_worktree` already give repo ↔ worktree grouping (schema; radar's use). Projects
  across repositories can come from the watched-projects list (F26) rather than a plugin-owned
  folder like herdr-projects'.
- **Joining sessions to tasks without a task store.** The public plugins learn PRs from:
  - agent self-reports (herdr-projects' `PR:` line)
  - transcripts (gh-metadata)
  - the branch plus `gh` (reviewr, gh-pr)

  Of these, only the branch plus `gh` stays within "public interfaces, no session files, no
  user-level agent config". With it, a branch → issue convention and `Closes #n` cover the
  remaining join. The build-options note already proposes this (§3 there), marking fuzzy links as
  "inferred".
- **Session ↔ worktree binding.** Options:
  - key the binding on the Herdr `agent_session` value plus canonical cwd, never on pane id alone;
  - hold it in the center's own state rather than in tokens;
  - re-emit any tokens from `[[startup]]`, because tokens die with the server (§5).
  
  Whether the owner's existing plugin already does this is not visible from public sources.
- **Integration style.** The center can shell out to the `herdr` CLI (most plugins) or speak the
  socket with a protocol gate (board-np). `agent.view.set` needs the socket either way in 0.9.1.
- **Distribution.** Prebuilt, checksum-verified release binaries fetched by `[[build]]`, with an
  own `update` and a `doctor`, is the most complete pattern seen (herdr-projects, reviewr,
  annotate). It fits a single `workbench` binary. A `plugin link` install needs first-start setup,
  because `link` skips `[[build]]`.
- **Borrowing without depending.** reviewr, annotate, navigator and sessionizer touch no shared
  single-slot surface except keys. The center can open them through `plugin.pane.open` or
  `plugin.action.invoke` (CC1 §4) and leave key choice to the user's workspace setup command (F26).
- **Where a "needs you" list comes from.** The sticky states (done until seen, blocked until working,
  idle tiers) in radar and the five groups in herdr-projects are both derived from Herdr's status
  plus local rules. radar needs no agent cooperation, while herdr-projects needs the self-report
  hooks for its `needs you` override.

## 8. Limits

- The plugins were read, not run. Behaviour claims come from their own READMEs, docs and code
  comments at the pinned commits.
- "Which plugins write tokens or set the agent view" came from a text search of the sources for
  `report-metadata`, `report_metadata`, `agent.view` and `agent_view`. A plugin that builds those
  calls dynamically could be missed.
- How plannotator-tui finds an agent's recent replies was not read, so that point is
  **unconfirmed**.
- board-bb's Claude state-detection finding and board-np's protocol gate were checked against the
  authors' notes for Herdr 0.7.5–0.9.1, not against 0.9.2.
- No plugin combinations were installed, so every collision in §4 is inferred from code and
  Herdr's documented rules.
