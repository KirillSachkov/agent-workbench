# Minimal Neovim as the viewer of agents' work

Date: 2026-09-30. Question from #60 (CC16, part of map #41, the command center): how can Neovim
(LazyVim, `$EDITOR=nvim`) be set up minimally as the place to view what an agent did (files,
changes, the PR) inside Herdr, and how does it connect to the command center?

This note builds on two notes it does not repeat:
[opening-results-from-terminal](https://github.com/KirillSachkov/agent-workbench/blob/research/44-opening-results/docs/research/2026-09-29-opening-results-from-terminal.md)
(branch `research/44-opening-results`: editor line dialects, `nvim +N`, `--server`/`--remote-send`,
OSC 8 and Herdr link handlers, kitty graphics in Herdr, Markdown renderers outside Neovim) and
[herdr-plugins-built](https://github.com/KirillSachkov/agent-workbench/blob/research/59-herdr-plugins-built/docs/research/2026-09-30-herdr-plugins-built.md)
(branch `research/59-herdr-plugins-built`: how reviewr, annotate and other Herdr plugins are built,
shared-surface collisions, fragile text delivery to agents).

Method.

- **Plugins.** Metadata (stars, last push, license, default-branch head) came from `gh api
  repos/<repo>` on 2026-09-30. READMEs were read at the pinned head below. Source was downloaded
  as tarballs at the same commits and read for ChmaraX/herdr-nvim, jhochenbaum/herdr-hunk-diff,
  TianZuo555/herdr.nvim, folke/snacks.nvim, LazyVim/LazyVim, georgeguimaraes/review.nvim and
  esmuellert/codediff.nvim.
- **Neovim.** `runtime/doc/{remote,starting,pack,plugins}.txt` at tag `v0.12.5` (latest stable,
  2026-08-23, per `gh api repos/neovim/neovim/releases`). Cited as `nvim:<file>`.
- **Herdr.** Docs under `docs/next/website/src/content/docs/` and source at `master` commit
  `331775c` (after the 0.9.3 release of 2026-09-29), cited as `herdr@331775c:<path>`. Issues were
  read with `gh api`. Locally only read-only commands ran: `herdr --version` (0.9.1) and
  `herdr --default-config`.
- **Local.** `nvim --version` reports 0.12.3. One timing run of `nvim --clean --headless
  --startuptime` was made. No plugin was installed and no config was changed.
- Stars are a popularity signal, not a quality measure. Anything not verified in a primary source
  is marked **unconfirmed**.

Citation shorthand: `<repo>@<sha7>:<path>` means
`https://github.com/<owner>/<repo>/blob/<full sha>/<path>`, with the owner and full SHA from the
table below.

| Repository | Commit | ★ | Last push | License |
|---|---|---|---|---|
| sindrets/diffview.nvim | `4516612fe98ff56ae0415a259ff6361a89419b0a` | 5,831 | 2024-08-02 | not detected by API |
| dlyongemallo/diffview-plus.nvim | `5152bada7d81cd602373e77534aabc631419e8e0` | 340 | 2026-09-21 | not detected by API |
| lewis6991/gitsigns.nvim | `070a5d7b985546cc57e1fc61e5bc507fecac6045` | 7,125 | 2026-09-22 | MIT |
| nvim-mini/mini.diff (part of mini.nvim, 9,555 ★) | `626b8a5b93874c4d05ca25aedec56cfff0b378fb` | 232 | 2026-07-30 | MIT |
| tpope/vim-fugitive | `3b753cf8c6a4dcde6edee8827d464ba9b8c4a6f0` | 21,804 | 2026-03-07 | not detected by API |
| esmuellert/codediff.nvim | `09d9ebef2cc5a5c04db7a349cd6c61bdf84ecc8e` | 1,578 | 2026-09-15 | MIT |
| georgeguimaraes/review.nvim | `f72a347538913ac558d2440dd3899426a2dd85ae` | 129 | 2026-09-17 | Apache-2.0 |
| pwntester/octo.nvim | `af2411604b51cb4a0f3e2de50b1b7cacc2581c48` | 3,389 | 2026-08-28 | MIT |
| ldelossa/gh.nvim | `6f367b2ab8f9d4a0a23df2b703a3f91137618387` | 650 | 2025-01-21 | MIT |
| folke/snacks.nvim | `882c996cf28183f4d63640de0b4c02ec886d01f2` | 8,111 | 2026-05-25 | Apache-2.0 |
| NeogitOrg/neogit | `c51a1dc7e0e0f40c4c396a2b1ceaa44b44e96032` | 5,639 | 2026-09-29 | MIT |
| modem-dev/hunk | `0a67560cfd4f4ad2a01251018c4caa538a1f107a` | 9,439 | 2026-09-29 | MIT |
| agavra/tuicr | `9175dc95b97a0a7dd290d43d38385745a7fb7d40` | 3,247 | 2026-09-23 | MIT |
| persiyanov/herdr-reviewr | `cd618b48dc1f4d2cc092f3f26827ac72fb44773e` | 794 | 2026-09-23 | MIT |
| jhochenbaum/herdr-hunk-diff | `47146a058858b4a7a116992aaae643436d80e7ab` | 134 | 2026-09-27 | MIT |
| ChmaraX/herdr-nvim | `5e849b5377fd409d2fd4be159c9c9fa36c251f7a` | 222 | 2026-09-28 | MIT |
| TianZuo555/herdr.nvim | `f0571409a4fdaf8761c71d887e6584ccabf399bd` | 2 | 2026-07-20 | MIT |
| coder/claudecode.nvim | `2390c6e45c4789072c293ac69de051d169668b29` | 3,088 | 2026-09-13 | MIT |
| folke/sidekick.nvim | `3d80a47e6375f6d647c9695d3afea6fa1b3275df` | 2,779 | 2026-09-08 | Apache-2.0 |
| olimorris/codecompanion.nvim | `9af12c15d0604f429e034ea7542f6cd038ff909a` | 6,885 | 2026-09-29 | Apache-2.0 |
| avante-corp/avante.nvim | `421209b946d208b2ab27ca95378c2b0073188ae6` | 18,168 | 2026-09-26 | Apache-2.0 |
| carlos-algms/agentic.nvim | `81628c1dc07edadd1c2c3c27d8dbcb424da1dea0` | 632 | 2026-08-23 | MIT |
| nickjvandyke/opencode.nvim | `06770e2e3618b82703e3e7af1f2d5dc67bde0002` | 3,857 | 2026-09-24 | MIT |
| johnseth97/codex.nvim | `e1149cbc875ff71ee21f8832b9fe815a270442b3` | 265 | 2025-11-20 | not detected by API |
| mhinz/neovim-remote (nvr) | `1004d41696a3de12f0911b1949327c3dbe4a62ab` | 1,905 | 2023-09-29 | MIT |
| willothy/flatten.nvim | `d92ca41e9c330f45c1b854a80c89c8488a9d730c` | 725 | 2026-06-19 | MIT |
| samjwill/nvim-unception | `f14eeb22238d2c2bdafe9a2e5e1f4bd200d0e450` | 244 | 2026-05-14 | MIT |
| MeanderingProgrammer/render-markdown.nvim | `640a3ec6d538bad17c328be373c7cad0293d9589` | 5,118 | 2026-09-15 | MIT |
| OXY2DEV/markview.nvim | `190bb227c9d3eb4451b81fa16ea3d2efde2d67c5` | 3,651 | 2026-09-17 | Apache-2.0 |
| 3rd/image.nvim | `365e2ace0f619164ae6fd2f6f1d2807775dd6849` | 2,087 | 2026-09-04 | MIT |
| LazyVim/LazyVim (16.0.1) | `999700997f72227187d49d8b92667183dc7fc809` | 27,573 | 2026-09-08 | Apache-2.0 |
| folke/lazy.nvim | `306a05526ada86a7b30af95c5cc81ffba93fef97` | 21,612 | 2026-06-29 | Apache-2.0 |
| agentclientprotocol/agent-client-protocol | `9b26a3eaa8d3644c2a899f56171bd25f7a5a884f` | 4,356 | — | — |

---

## 1. Viewing changes

### 1.1 Comparison

| Tool | What it shows | Uncommitted | Branch vs base | One commit | Last agent turn | PR | Comments to an agent |
|---|---|---|---|---|---|---|---|
| **gitsigns.nvim** (in LazyVim core) | signs, inline hunk preview, blame, `diffthis` per buffer | yes (per buffer) | `:Gitsigns change_base <rev>` moves the signs' base | via base | no | no | no |
| **mini.diff** (LazyVim extra `editor.mini-diff`) | per-buffer diff against a configurable reference, "overlay" view | yes (index by default) | only with a custom source | no | no | no | no |
| **vim-fugitive** | `:Git`, `:Gdiffsplit`, `:Git difftool` into quickfix | yes | `:Git difftool <base>...` (**unconfirmed** form) | yes | no | no | no |
| **diffview.nvim** / **diffview-plus** | one tab: file panel plus side-by-side diffs, file history | `:DiffviewOpen` | `:DiffviewOpen origin/main...HEAD` | `:DiffviewOpen <sha>^!` (**unconfirmed**) | no | no | no |
| **codediff.nvim** | VS Code–style explorer and diff, inline or side-by-side, history panel, `+N -M` stats | `:CodeDiff` | `:CodeDiff main...` (merge base) | `:CodeDiff A B` | no | `:CodeDiff pr 512` fetches into private refs, no checkout, no provider CLI | only through **review.nvim** |
| **review.nvim** (on codediff) | typed comments (Note, Suggestion, Issue, Praise) on diff lines and on any file | yes | "branch against its base (merge-base aware)" | commit picker | no | no | Markdown export to clipboard plus `on_export(markdown, comments)` callback; direct send to sidekick.nvim |
| **snacks.nvim** picker (LazyVim) | `git_diff` hunks picker with preview, `git_status`, `git_log`, `git_log_file`, `gitbrowse` | `Snacks.picker.git_diff()` | `git_diff({ base = "origin/main" })` runs `git diff --merge-base <base>` (includes working tree) | `git_log` | no | `Snacks.picker.gh_pr()`, `gh_diff({ pr = N })` | review via `gh` (approve, request changes, comment) |
| **octo.nvim** (LazyVim extra `util.octo`) | GitHub issues and PRs as buffers, review mode with a file panel | no | via PR | `Octo review commit` | no | yes | pending review comments and suggestions, submitted to GitHub |
| **Hunk** (outside nvim) | review stream with sidebar, inline agent notes, watch mode | `hunk diff` | via `hunk-diff` plugin `review:branch` | `hunk show <sha>` | no | `hunk gh pr` (no `gh` needed) | through herdr-hunk-diff `send-review` |
| **herdr-reviewr** (outside nvim) | Changes / All files / PR tabs | yes | yes | yes | yes (heuristic, 2 s poll) | read-only tab | comments sent with `herdr pane send-text` |

Sources, by row:

- gitsigns: `gitsigns.nvim@070a5d7:README.md` line 79 (`change_base`); LazyVim maps hunk
  actions under `<leader>gh` (`LazyVim@9997009:lua/lazyvim/plugins/editor.lua` lines 134–199).
- mini.diff: "difference between buffer text and some reference text", overlay view, and "unstaging
  hunks is not supported" (`mini.diff@626b8a5:README.md` lines 31–44, 59, 76–83). The LazyVim
  extra is `lua/lazyvim/plugins/extras/editor/mini-diff.lua`.
- fugitive: `:Git difftool` loads changesets into quickfix, `:Gdiffsplit` (`vim-fugitive@3b753cf:README.md`
  lines 24, 39). The exact base-range form was not checked.
- diffview: the upstream repository was last pushed 2024-08-02. diffview-plus calls itself "an
  actively maintained fork" and documents `:DiffviewOpen origin/main...HEAD " Symmetric diff
  (PR-style)`. It needs Neovim ≥ 0.10 (`diffview-plus.nvim@5152bad:README.md` lines 3, 106–122).
  Neither README mentions comments or annotations. LazyVim 16.0.1 ships no diffview extra (a
  search of `lua/lazyvim` found no match).
- codediff: usage and PR review (`codediff.nvim@09d9ebe:README.md` lines 408–500). It needs a
  prebuilt `libvscode_diff` library downloaded by `:CodeDiff install` (lines 54–93).
- review.nvim: `review.nvim@f72a347:README.md` "Features", lines 79–82, 205, 259–265.
- snacks: `snacks.nvim@882c996:lua/snacks/picker/source/git.lua` lines 259–282 (`--merge-base`);
  `docs/gh.md` lines 8–78. LazyVim binds `<leader>gd` to `git_diff()` and `<leader>gD` to
  `git_diff({ base = "origin", group = true })`
  (`LazyVim@9997009:lua/lazyvim/plugins/extras/editor/snacks_picker.lua` lines 75–77).
- octo: review workflow (`octo.nvim@af24116:README.md` lines 779–793). The LazyVim extra takes
  `<leader>gi/gI/gp/gP` from snacks and sets `picker = "telescope"`
  (`LazyVim@9997009:lua/lazyvim/plugins/extras/util/octo.lua`).
- Hunk: `hunk@0a67560:README.md` "Working with Git", "Working with GitHub", "Working with agents".
- herdr-reviewr: see herdr-plugins-built §1 and §2.4.

### 1.2 Facts that matter for the four scopes

1. **Uncommitted changes and "branch against base"** are covered by at least three maintained
   Neovim tools each: codediff (`:CodeDiff`, `:CodeDiff main...`), diffview-plus, and the snacks
   picker (1.1).
2. **A single commit** is covered by codediff (two revisions), diffview, octo (`review commit`)
   and snacks `git_log` (1.1).
3. **No Neovim plugin read here has a "last agent turn" scope.** Git does not record turn
   boundaries. herdr-reviewr derives one from Herdr status polling and calls it a heuristic
   (herdr-plugins-built §2.4). herdr-nvim's picker lists "the files touched this session", which it
   mines from the agent's session log. That is a file list, not a diff scope
   (`herdr-nvim@5e849b5:README.md` "The file picker"; `src/sessions.rs` lines 31–82 read
   `~/.claude/projects/…`, `~/.pi/agent/sessions/…`).
4. **The PR** can be read in Neovim three ways:
   - octo.nvim, with a review mode that posts to GitHub;
   - snacks `gh_pr` / `gh_diff`, which need `gh`;
   - codediff `pr N`, which fetches with Git credentials, needs no provider CLI and is read-only.
5. **Built-in fallback.** Neovim 0.12 ships an optional `nvim.difftool` plugin: `:DiffTool {left}
   {right}` compares two files or directories, with rename detection and a quickfix list
   (`nvim:plugins.txt` lines 21, 45–61).

## 2. Sending remarks back to the agent

### 2.1 Three delivery mechanisms

| Mechanism | Plugins | Reaches the agent already running in a Herdr pane? | Agents |
|---|---|---|---|
| **Herdr CLI** (`herdr agent prompt` submits; `herdr pane send-text` types without submitting) | herdr-nvim (both), TianZuo555/herdr.nvim (`send-text`), herdr-hunk-diff (`agent prompt`), herdr-reviewr (`send-text`) | yes | any agent Herdr sees |
| **Agent-specific editor protocol** | claudecode.nvim (Claude Code's WebSocket MCP "IDE" protocol); opencode.nvim (OpenCode server API) | yes, if the external agent connects to the editor | one agent each |
| **ACP client inside Neovim** | codecompanion.nvim, avante.nvim, agentic.nvim | **no**: "the editor boots the agent sub-process on demand, and all communication happens over stdin/stdout" | Claude Code, Codex, Gemini and others through ACP adapters |

Sources:

- herdr-nvim: `lua/herdr-nvim/dispatch.lua` calls `herdr agent prompt <pane> <text>` when submitting
  and `herdr pane send-text` otherwise.
- TianZuo555/herdr.nvim: `lua/herdr/transport.lua` line 99.
- herdr-hunk-diff: `src/herdr.ts` line 63.
- claudecode.nvim: `README.md` lines 7–18, 296–317, 644–663.
- opencode.nvim: "Connect to _any_ OpenCode server" (`README.md` "Features").
- codecompanion.nvim: `README.md` line 34. avante.nvim: `README.md` lines 59, 1161–1166.
- ACP: `agent-client-protocol@9b26a3e:docs/get-started/architecture.mdx` line 18.
- Herdr: `agent prompt` "writes text followed by delayed Enter as one ordered submission, including
  while the agent is working"; `pane run` submits atomically; `send-text` is "low-level and
  non-submitting" (`herdr@331775c:docs/next/website/src/content/docs/cli-reference.mdx` lines
  256–269, 343–367).

### 2.2 Details

- **herdr-nvim is the closest existing fit.** It is a Herdr plugin and a Neovim plugin in one
  repository (MIT, 222 ★, v1.1.0, needs nvim ≥ 0.10 and herdr ≥ 0.7.5). It provides:
  - comments on a line or selection, kept in memory and tracked by extmarks, then sent all at once
    to "any agent in the workspace" with `file:line`, repo and branch;
  - `:Herdr ref`, which drops `path:12-20` into the agent's input without submitting;
  - an automatic target when there is one agent in the workspace or one in the same tab, and a
    picker otherwise.

  Source: `herdr-nvim@5e849b5:README.md` "Annotations", "References".
- **Hunk/herdr-hunk-diff** keeps human comments in a live Hunk session. `send-review` formats all
  unsent comments into one prompt. Delivered IDs are recorded so a comment is not sent twice.
  Delivery goes to the agent pane associated with the worktree: one target per worktree, taken
  from the pane that opened the review or from agent-status events
  (`herdr-hunk-diff@47146a0:README.md` "Review and send", "Round-trip prompts", Troubleshooting).
  It talks to Hunk over loopback `127.0.0.1:47657`, which an agent sandbox can block (same README).
- **review.nvim** does not deliver by itself. It exports Markdown and calls `on_export`, "where
  you hook up a tmux pane, a file an agent watches, Avante, or anything else"
  (`review.nvim@f72a347:README.md` lines 259–265). A one-line callback that runs
  `herdr agent prompt` would give codediff the same round trip. This is inferred and was **not
  tested**.
- **claudecode.nvim** is third-party ("I reverse-engineered their extension"). It works with Claude
  Code only. It writes a lock file to `~/.claude/ide/<port>.lock`, and Claude Code finds the
  editor through that file. With `provider = "none"`, Claude runs outside Neovim (for example in a
  Herdr pane) and connects with `claude --ide` or `/ide`. Sends then go over the WebSocket, and
  `ClaudeCodeSendText` does not work (`README.md` lines 237–245, 644–663). The lock file records
  `workspaceFolders` (`PROTOCOL.md` line 24). Whether Claude Code offers the editor when its cwd is
  a different worktree is **unconfirmed**.
- **sidekick.nvim** sends `{file}`, `{selection}` and `{diagnostics}` prompts to AI CLIs it runs in
  its own terminal or multiplexer. The multiplexer backend is `"tmux"|"zellij"` only
  (`sidekick.nvim@3d80a47:README.md` lines 21–23, 317–329, 591–626). It also requires Neovim ≥
  0.11.2 and copilot-language-server for its main feature (lines 27–33). Herdr is not a backend.
- **Codex.** No Codex IDE protocol for Neovim was found. The `codex.nvim` repositories are terminal
  wrappers. The largest has 265 ★, was last pushed 2025-11-20 and has no license (`gh api`). ACP
  clients start their own Codex session (2.1). Reaching a running Codex in Herdr therefore goes
  through the Herdr CLI.
- **Known delivery risk.** `herdr agent prompt` merged with half-typed user text on 0.9.1, and
  plugins guard for this: they prompt only idle agents, or check readiness first
  (herdr-plugins-built §2.4, "Delivering text to agents is fragile").

## 3. Opening from the command center

- **New Neovim in a Herdr pane at the agent's worktree.**
  1. `herdr pane split <agent pane> --direction right --cwd <worktree> [--env KEY=VALUE]`. The new
     id is `.result.pane.pane_id`. `pane split` takes no command argument.
  2. `herdr pane run <new pane> "nvim +42 path"`, which "submits text plus Enter atomically".

  Sources: `cli-reference.mdx` lines 205–233, 269, 545. Herdr sets `HERDR_ENV=1`,
  `HERDR_PANE_ID`, `HERDR_TAB_ID` and `HERDR_WORKSPACE_ID` in pane processes (lines 560–569), so a
  Neovim config can enable Herdr-only plugins with `cond = vim.env.HERDR_ENV == "1"` (lazy.nvim
  `cond`, `lazy.nvim@306a055:doc/lazy.nvim.txt` lines 279–284).
- **Reusing a running Neovim.**
  - Start it with `nvim --listen <addr>`. Then `nvim --server <addr> --remote <file>` builds a
    `:drop`, and `--remote-send` / `--remote-expr` run keys or expressions (`nvim:remote.txt`;
    `nvim:starting.txt` line 423).
  - `--remote +{cmd}` and every `--remote-wait` variant are "not yet supported by Nvim"
    (`nvim:remote.txt` §2), so the line has to go through `--remote-send` or `--remote-expr`.
  - nvr adds `--remote-wait` (useful as `git config core.editor 'nvr --remote-wait-silent'`). Its
    last push was 2023-09-29 (`neovim-remote@1004d41:README.md` lines 100–122).
- **herdr-nvim already does per-tab reuse.**
  - Each Herdr tab gets its own headless Neovim daemon. A sidebar pane attaches with
    `nvim --server <sock> --remote-ui`, and files open through `--server … --remote`
    (`src/bridge.rs` lines 295–330; `src/doctor.rs` lines 245–287).
  - Daemons survive a toggle and a Herdr restart, stop with their tab, and are listed with
    `herdr-nvim daemons` (RAM, uptime, unsaved buffers).
  - `sidebar.nvim_env = ["NVIM_APPNAME=…"]` runs the sidebar under a named config.

  Sources: `README.md` "The sidebar", "Config"; `herdr-plugin.toml` `[[events]]`, `[[startup]]`.
- **Links.**
  - herdr-nvim registers two link handlers: `^file://` (it says Claude Code emits OSC 8 `file://`
    links) and a regex for plain `dir/file.ext:line[:col]` text. Both open the file in the sidebar
    (`herdr-plugin.toml`, `src/openlink.rs` header).
  - Herdr's documentation says link handlers match "the clicked URL". In `master`, plain text under
    the pointer resolves to a URL only when it starts with `http://` or `https://`
    (`herdr@331775c:src/app/actions.rs` lines 1044–1055, 1100–1113).
  - The plain-path handler therefore firing on text that is not OSC 8 is **unconfirmed**. The
    OSC 8 `file://` path is documented (opening-results note §1.3).
- **Herdr's own "open in nvim".** `prefix+e` (`keys.edit_scrollback`) opens the focused pane's
  scrollback in `$EDITOR` in a temporary zoomed pane (`herdr@331775c:CHANGELOG.md` lines 231, 841;
  `herdr --default-config`). With `$EDITOR=nvim` this is already a way to read an agent's output
  in Neovim.
- **flatten.nvim and nvim-unception** solve a different problem: they stop a nested Neovim when a
  file is opened from Neovim's own `:terminal`. flatten names WezTerm and Kitty as the outer
  terminals it can flatten from (`flatten.nvim@d92ca41:README.md` lines 10, 25–33;
  `nvim-unception@f14eeb2:README.md`). Herdr support is **unconfirmed** (not mentioned).

## 4. A minimal profile

- **Isolation.** `$NVIM_APPNAME=foo` makes Neovim read `$XDG_CONFIG_HOME/foo` and use matching data,
  state and cache directories. The docs name "isolate Nvim applications" as a use
  (`nvim:starting.txt` lines 1428–1446). `--clean` loads builtin plugins but no user config or
  shada (lines 90–95).
- **Plugin manager without lazy.nvim.** `vim.pack` is built in, is "still considered experimental,
  yet should be stable enough for daily use", and keeps a lockfile `nvim-pack-lock.json` in the
  config directory, meant to be committed (`nvim:pack.txt` lines 208–233).
- **Stripping LazyVim.**
  - lazy.nvim `enabled = false` drops a plugin from the spec. `cond = false` skips it without
    uninstalling it (`lazy.nvim@306a055:doc/lazy.nvim.txt` lines 279–284).
  - LazyVim 16.0.1 core loads about 25 plugins: bufferline, flash, noice, lualine, which-key,
    trouble, gitsigns, snacks, conform, nvim-lint, treesitter, mason and others (the plugin
    specs in `lua/lazyvim/plugins/*.lua`).
  - It requires Neovim ≥ 0.11.2 and checks for `git`, `rg`, `fd`, `lazygit`, `fzf` and `curl`
    (`lua/lazyvim/health.lua` lines 12–17).
- **Startup.** On this machine, `nvim --clean --headless --startuptime` reached `NVIM STARTED` at
  about 6 ms (one run, nvim 0.12.3). lazy.nvim byte-compiles and caches Lua used before `VimEnter`
  (`doc/lazy.nvim.txt` lines 1187–1193). A LazyVim startup time was **not measured** here.
- **What a "reader" keeps.** No public "reader profile" convention was found. The pieces that the
  sources above tie to viewing, as opposed to editing, are:
  - picker/explorer (snacks);
  - git signs (gitsigns or mini.diff);
  - a multi-file diff (codediff or diffview-plus);
  - a Markdown renderer;
  - the Herdr bridge (herdr-nvim).

  LSP, formatters, linters, completion and AI suggestion plugins serve editing. Treating them as
  optional for a viewer is an inference.
- **Key bindings next to Herdr.**
  - Herdr's default prefix is `ctrl+b`. Its defaults include `prefix+e` (edit scrollback),
    `prefix+o` (open notification target), `prefix+h/j/k/l` (focus), `prefix+v` and `prefix+minus`
    (split) (`herdr --default-config`).
  - With the default prefix, `ctrl+b` "enters prefix mode instead of paging up" (stated for copy
    mode in `herdr@331775c:docs/next/website/src/content/docs/keyboard.mdx` line 87). The same key is Neovim's page-up.
    Herdr accepts several prefixes, `prefix = ["ctrl+space", "ctrl+s"]` (0.9.2 CHANGELOG).
  - herdr-nvim's README suggests binding its sidebar to `prefix+e` and its picker to `prefix+o`.
    Both are Herdr defaults (`herdr-nvim@5e849b5:README.md` lines 42–57).
  - Inside Neovim, herdr-nvim's default prefix `<leader>a` (`ac`, `al`, `as`, `aS`, `ai`)
    overlaps LazyVim's sidekick and avante extras (`<leader>aa`, `as`, `ac`, …;
    `extras/ai/sidekick.lua` lines 82–118, `extras/ai/avante.lua` lines 34–41). herdr-nvim
    "never override[s] a map you already set".

## 5. Markdown artifacts in Neovim

| Plugin | What it does | Images |
|---|---|---|
| render-markdown.nvim (LazyVim extra `lang.markdown`, with markdown-preview.nvim for the browser) | renders headings, code blocks, tables, callouts (GitHub and Obsidian), checkboxes and LaTeX in the buffer; modal rendered/raw view; renders only the visible range | an icon for image links, no pixels |
| markview.nvim | Markdown, LaTeX, Typst, HTML and AsciiDoc previewer with a "hybrid" edit-and-preview mode and split view | pixel rendering not found in README (**unconfirmed**) |
| snacks.image | kitty graphics: opens png/jpg/pdf/video frames, renders images and Mermaid/LaTeX inline in Markdown and other file types | yes; needs ImageMagick |
| image.nvim | kitty graphics, ueberzugpp or sixel backends | yes; needs ImageMagick |

Sources: `render-markdown.nvim@640a3ec:README.md` "Features", line 49 (nvim ≥ 0.9, 0.10
recommended); `LazyVim@9997009:lua/lazyvim/plugins/extras/lang/markdown.lua`
(`iamcco/markdown-preview.nvim`, `MeanderingProgrammer/render-markdown.nvim`);
`markview.nvim@190bb22:README.md` lines 54–79; `snacks.nvim@882c996:docs/image.md` lines 5–40;
`image.nvim@365e2ac:README.md` lines 3–46.

Inside Herdr:

- Herdr re-renders kitty graphics from panes, on by default since 0.9.0. Sixel and iTerm2 images
  are not supported (opening-results note §4.2). So only the kitty paths (snacks.image, image.nvim
  `kitty`) apply. image.nvim's `sixel` backend does not.
- snacks.image detects the terminal from environment variables and an XTVERSION (`CSI > q`) query.
  Its table knows kitty, ghostty, wezterm, tmux, zellij and ssh, not Herdr, and it can be forced
  with `SNACKS_<NAME>=true` (`lua/snacks/image/terminal.lua` lines 7–37, 225–300;
  `docs/image.md`).
- Herdr issue [#494](https://github.com/herdrdev/herdr/issues/494) ("images don't render inside
  Neovim panes") closed on 2026-06-25. Its last comment reports that `SNACKS_KITTY=1` plus
  `SNACKS_SSH=1` made inline Mermaid render in a Neovim pane on Herdr 0.7.1. It names "XTVERSION
  passthrough + `t=f` file-path loading" as the native fixes. Whether 0.9.x still needs the
  variables is **unconfirmed**.
- Related Herdr issues:
  - [#732](https://github.com/herdrdev/herdr/issues/732): blank space for snacks placeholders,
    released in 0.7.1.
  - [#3900](https://github.com/herdrdev/herdr/issues/3900): Neovim "choppy" in Herdr, released in
    0.9.1.
  - Still open: [#3941](https://github.com/herdrdev/herdr/issues/3941) (no pane images from an
    iTerm2 host), [#1178](https://github.com/herdrdev/herdr/issues/1178) (Neovim underline colours
    wrong), [#3923](https://github.com/herdrdev/herdr/issues/3923) (OSC 52 clipboard from Neovim
    stops after emoji) and [#3663](https://github.com/herdrdev/herdr/issues/3663) (stale row when
    Neovim opens a Markdown file from Yazi).

## 6. Implications for the command center

These are facts and options. The decisions are the owner's.

- **Most of the viewer already exists.** Diff scopes (uncommitted, branch vs merge base, commit,
  PR) are covered by codediff, diffview-plus and the snacks picker. Remarks can reach a running
  agent through the Herdr CLI (herdr-nvim, herdr-hunk-diff). The center does not need its own diff
  UI to offer "open in Neovim". It needs a worktree path, a base ref and an agent pane id, and it
  already has those from Herdr (herdr-plugins-built §2.2, §5).
- **Last turn is the one missing scope.** No Neovim plugin has it (§1.2). Options:
  - the center records a snapshot ref at each turn start (for example a `git stash create` commit
    kept under its own ref namespace) and opens `:CodeDiff <ref>`;
  - leave "last turn" to herdr-reviewr's heuristic;
  - drop the scope.

  The first is an inference and was not tested.
- **One delivery channel works for every agent.** `herdr agent prompt` / `pane send-text` reach
  Claude Code, Codex and others in Herdr panes. claudecode.nvim is Claude-only and
  reverse-engineered. ACP plugins start a separate agent session (§2.1). If the center sends
  remarks itself, it inherits the known risk of merging with a half-typed draft (§2.2).
- **Avoid depending on session files.** herdr-nvim's "files touched" list reads Claude Code and pi
  transcripts from disk (§1.2). The repository's AGENTS.md rules that out for the center's own
  code. A center-provided list (git status plus the agent's result card) avoids it.
- **Keys.** herdr-nvim's suggested `prefix+e` / `prefix+o` would shadow Herdr's own defaults.
  `<leader>a` is crowded in LazyVim's AI extras (§4). A workspace setup command that checks for
  conflicts before writing keys matches the pattern in herdr-plugins-built §3.
- **Option A: a separate reader profile via `NVIM_APPNAME`.**
  - A small config (for example `$XDG_CONFIG_HOME/nvim-reader`) with `vim.pack` and its lockfile,
    or lazy.nvim.
  - Plugins: snacks (picker, explorer, `git_diff`, `gh`, `image`), gitsigns, codediff with
    review.nvim, render-markdown, and herdr-nvim with `cond = vim.env.HERDR_ENV == "1"`.
  - The center opens it with `herdr pane split … --cwd <worktree> --env NVIM_APPNAME=nvim-reader`
    and then `pane run "nvim +N path"`. herdr-nvim's `sidebar.nvim_env` points its sidebar at the
    same profile.
  - This separates the viewer from the owner's editing config, at the cost of maintaining two
    configs.
- **Option B: keep LazyVim and add three specs.**
  - codediff (plus review.nvim with an `on_export` that calls `herdr agent prompt`);
  - the `lang.markdown` extra and `snacks.image` (with `SNACKS_KITTY`/`SNACKS_SSH` if detection
    still fails in Herdr);
  - herdr-nvim for comments and the Ctrl-click sidebar.

  The center only emits `nvim +N path`, or `nvim --server <sock> --remote-send` for a running
  instance. There is a single config, but LazyVim's editing stack also loads in the viewer.
- **Where the center stops.** In both options the center's job is limited to four things: pick the
  worktree, pick the scope (a ref), open the pane, and name the target agent. Rendering and
  commenting stay in existing plugins.

## 7. Limits

- Plugins were read, not run. Behaviour claims come from READMEs, docs and source at the pinned
  commits.
- The fugitive and diffview single-commit and range forms, claudecode.nvim with a cwd that differs
  from the editor's, herdr-nvim's plain-path link handler, and snacks.image inside Herdr 0.9.x
  without the environment override were not verified.
- The only startup time measured was vanilla `--clean` (one run).
- GitHub search found few Neovim "review comments to agent" plugins. Plugins with other names may
  have been missed.
