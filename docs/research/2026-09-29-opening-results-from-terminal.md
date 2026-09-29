# Opening an agent's result from the terminal

Date: 2026-09-29. Question from #44 (part of map #41, the command center): how can a terminal-first
tool make "check an agent's result" one step away, and how do agent workspace managers do it?

Method.

- **Sources.** Official docs, specs and changelogs of editors (VS Code, Cursor, Zed, Neovim,
  JetBrains), terminals (kitty, Ghostty, WezTerm, iTerm2, Alacritty), multiplexers (Herdr, tmux,
  Zellij), GitHub CLI manual pages, Ratatui and its image crates, Markdown renderers, and the docs,
  READMEs and source of Conductor, Superset, Agent Orchestrator, Emdash, the Codex app, Claude Code
  desktop, cmux, roamgate, herdr-reviewr, portless and Worktrunk.
- **Pins.** GitHub source links are pinned to a tag or commit read on 2026-09-29. Herdr source was read
  at tag `v0.9.2` and at `master` commit `f0bb782`; the locally installed binary reports `herdr 0.9.1`
  (`herdr --version`). Local checks were read-only (`--version`, `--help`, `herdr api schema`); no
  config was changed and nothing was sent to a pane.
- **Not redone.** The pattern catalog of agent products
  ([agent-products-human-artifacts §8](2026-09-27-agent-products-human-artifacts.md)), review surfaces
  and md-viewers ([artifact-review-surfaces](2026-09-27-artifact-review-surfaces.md)), GitHub result
  cards and previews ([reporting-tooling-github](2026-09-27-reporting-tooling-github.md)), and the
  product and Herdr plugin landscape ([command-centers-landscape](2026-09-29-command-centers-landscape.md)).
- Anything not verified in a primary source is marked "unconfirmed".

## 1. Opening a changed file at a line

### 1.1 Editor command lines and URL schemes

| Editor | CLI at a line | URL scheme at a line | Notes |
|---|---|---|---|
| VS Code | `code -g file:line[:col]`; `-r` reuse window, `-n` new window, `--remote <authority>` | `vscode://file/<abs path>:line:col`; Insiders `vscode-insiders://` | [CLI docs](https://code.visualstudio.com/docs/configure/command-line). Opening a `vscode://file/...` URL shows an "An external application wants to open…" dialog unless `security.promptForLocalFileProtocolHandling` is `false` or the user ticked "allow" ([app.ts at 1.139.1, lines 214–217, 923–1030](https://github.com/microsoft/vscode/blob/1.139.1/src/vs/code/electron-main/app.ts)). |
| Cursor | `cursor -g file:line[:col]` (same flags as VS Code, observed in `cursor --help` 3.22.12) | `cursor` scheme is registered in the app's `Info.plist` (observed); `cursor://file/<abs>:line:col` is **unconfirmed** (undocumented, VS Code fork) | A reported bug: `cursor -g` does not escape characters ([forum](https://forum.cursor.com/t/cursor-goto-cli-command-does-not-escape-characters/143723)). |
| Zed | `zed path:line:col`; `-a` add to workspace, `-n` new, `-e` existing window, `-w` wait | `zed://file<path>` and `file://<path>` are accepted; paths go through `PathWithPosition::parse_str`, so `zed://file/abs:12:5` should carry the position (**unconfirmed**, read from source, not run) | [cli main.rs at v1.21.0, lines 68–101, 843–875](https://github.com/zed-industries/zed/blob/v1.21.0/crates/cli/src/main.rs); [open_listener.rs, lines 157–219, 1088–1100](https://github.com/zed-industries/zed/blob/v1.21.0/crates/zed/src/zed/open_listener.rs). First interactive `zed <path>` asks whether to reuse a window or open a new one. |
| Neovim | `nvim +{line} file` | none | [starting.txt v0.12.5, line 113](https://github.com/neovim/neovim/blob/v0.12.5/runtime/doc/starting.txt). A running instance: `nvim --server "$NVIM" --remote-send '<C-\><C-N>:edit +42 /abs/path<CR>'`. `--remote`'s `+{cmd}` is "not yet supported" ([remote.txt](https://github.com/neovim/neovim/blob/v0.12.5/runtime/doc/remote.txt)). `$NVIM` is set to `v:servername` inside `:terminal` and `jobstart()` ([vvars.txt, lines 588–607](https://github.com/neovim/neovim/blob/v0.12.5/runtime/doc/vvars.txt)). |
| JetBrains | `idea --line N [--column M] path` | `jetbrains://…/navigate/reference?…path=file:line:col` is **unconfirmed** (community-documented only; [TBX-3965](https://youtrack.jetbrains.com/projects/TBX/issues/TBX-3965) asks for docs) | [JetBrains docs](https://www.jetbrains.com/help/idea/opening-files-from-command-line.html) |

### 1.2 "Whatever the owner's editor is": how TUIs solve it

- **Editor choice.** Git resolves `$GIT_EDITOR`, then `core.editor`, then `$VISUAL`, then `$EDITOR`,
  then `vi` ([git-var](https://git-scm.com/docs/git-var)). The value does not say how to pass a line.
- **Four line dialects** cover common editors, per herdr-reviewr's table
  ([editor.rs at cd618b4](https://github.com/persiyanov/herdr-reviewr/blob/cd618b48dc1f4d2cc092f3f26827ac72fb44773e/src/editor.rs)):
  `+42 path` (vim, nvim, nano, micro, kak, emacs), `path:42` (helix, Zed, Sublime), `-g path:42`
  (code, code-insiders, codium, cursor, windsurf, positron), `--line 42 path` (JetBrains, Xcode,
  Kate, TextMate). It picks the dialect by the binary name of `$VISUAL`/`$EDITOR`, or takes a
  template such as `editor = "code -g {file}:{line}"`
  ([README](https://github.com/persiyanov/herdr-reviewr/blob/cd618b48dc1f4d2cc092f3f26827ac72fb44773e/README.md)).
- **lazygit** has `os.editPreset` plus templates `os.edit` (`{{filename}}`), `os.editAtLine`
  (`{{filename}}`, `{{line}}`), `os.editAtLineAndWait`, `os.openDirInEditor` (`{{dir}}`). Without a
  preset it guesses from `core.editor`, `$GIT_EDITOR`, `$VISUAL`, `$EDITOR`. Presets include
  `code --reuse-window --goto -- {{filename}}:{{line}}`, `zed -- {{filename}}:{{line}}`, and an
  `nvim-remote` preset that reuses the parent Neovim through `$NVIM`
  ([Config.md at v0.65.1](https://github.com/jesseduffield/lazygit/blob/v0.65.1/docs/Config.md),
  [editor_presets.go](https://github.com/jesseduffield/lazygit/blob/v0.65.1/pkg/config/editor_presets.go)).
- **tig** passes `+<line>` with `editor-line-number` and exposes `%(file)`, `%(lineno)`
  ([tigrc.5 at tig-2.6.1](https://github.com/jonas/tig/blob/tig-2.6.1/doc/tigrc.5.adoc)).
  **gitui** appends only the path, no line
  ([externaleditor.rs at v0.28.1](https://github.com/gitui-org/gitui/blob/v0.28.1/src/popups/externaleditor.rs)).
  **yazi** uses `${EDITOR:-vi} %s` and macOS `open` / `open -R`
  ([yazi-default.toml at v26.9.1](https://github.com/sxyazi/yazi/blob/v26.9.1/yazi-config/preset/yazi-default.toml)).

### 1.3 Clickable links in the terminal (OSC 8) and Herdr

- **OSC 8** marks text as a link: `ESC ] 8 ; params ; URI ESC \` … `ESC ] 8 ; ; ESC \`. For `file://`
  links the tool should put the hostname in the URI; multiplexers should rewrite `id=`
  ([spec](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda)). Supported by Ghostty,
  kitty, WezTerm, iTerm2, Alacritty 0.11+, the VS Code terminal, Zellij, tmux 3.4+
  ([OSC8-Adoption](https://github.com/Alhadis/OSC8-Adoption/blob/2bb57119a930c23501fabdb59ea76947475296d1/README.md)).
  tmux turns it on only for terminals it recognises; otherwise `set -ga terminal-features "*:hyperlinks"`
  ([CHANGES 3.4](https://github.com/tmux/tmux/blob/3.7c/CHANGES), [tty-features.c](https://github.com/tmux/tmux/blob/3.7c/tty-features.c)).
- **File-at-line links in host terminals.** kitty's `open-actions.conf` can match `protocol file` with
  `fragment_matches [0-9]+` and run `$EDITOR +$FRAGMENT $FILE_PATH`, i.e. the `file:///path#42`
  convention; `kitten hyperlinked-grep` emits such links
  ([open actions](https://sw.kovidgoyal.net/kitty/open_actions/), [hyperlinked grep](https://sw.kovidgoyal.net/kitty/kittens/hyperlinked_grep/)).
  iTerm2 3.4+ routes `file:///path#123` through Semantic History, which can run a command with the
  file and line ([escape codes](https://iterm2.com/documentation-escape-codes.html)). WezTerm's
  `open-uri` event can intercept any scheme ([docs](https://wezterm.org/config/lua/window-events/open-uri.html)).
  Ghostty on macOS opens URLs through `NSWorkspace`, so `vscode://file/...:12` reaches its registered
  app ([Ghostty.App.swift at v1.3.1, lines 702–740](https://github.com/ghostty-org/ghostty/blob/v1.3.1/macos/Sources/Ghostty/Ghostty.App.swift));
  whether it honours a `#line` fragment on `file://` is **unconfirmed** (inferred as no).
- **Herdr passes OSC 8 through.** Its renderer re-emits `ESC]8;;{uri}ESC\` for linked cells without
  filtering the scheme ([render_ansi.rs at v0.9.2](https://github.com/herdrdev/herdr/blob/v0.9.2/src/protocol/render_ansi.rs));
  "OSC 8 hyperlinks emitted inside panes now remain clickable" (0.5.6,
  [CHANGELOG](https://github.com/herdrdev/herdr/blob/v0.9.2/CHANGELOG.md)).
- **Clicking inside Herdr.** Ctrl-click opens OSC 8 links and visible plain `http(s)://` URLs; Ctrl
  also on macOS, because captured mouse reports do not carry Command (CHANGELOG 0.6.3, 0.7.0). Herdr's
  own opener launches only http(s); `mouse_capture = false` hands clicks back to the host terminal
  (`herdr --default-config`; [actions.rs](https://github.com/herdrdev/herdr/blob/v0.9.2/src/client/shell/actions.rs)).
  Plain text such as `src/main.rs:42` is not a link in Herdr.
- **Herdr plugin link handlers.** A `[[link_handlers]]` entry in `herdr-plugin.toml` has `id`,
  `title`, a Rust-regex `pattern` and an `action` of the same plugin; the action receives
  `HERDR_PLUGIN_CLICKED_URL` and `invocation_source = "link_click"`
  ([plugins.mdx 0.9.1, lines 94–99, 343–356](https://github.com/herdrdev/herdr/blob/v0.9.2/docs/versions/0.9.1/website/src/content/docs/plugins.mdx)).
  Since 0.9.0, "Plugin link handlers now receive matching OSC 8 `file://` clicks; unmatched file links
  still do not launch the system URL opener" (#2941, CHANGELOG). So a pane program that prints
  `file://host/abs/path#42` as OSC 8, plus a plugin matching `^file://.*#\d+$` that runs `code -g`,
  gives a Ctrl-click to the line.
- **No "open" command in Herdr's CLI.** `herdr --help` has none; the schema's `pane.link.resolve` and
  `pane.link.activate` take a pane id and viewport cell and are client-internal (read locally with
  `herdr api schema --json`, 0.9.1).

### 1.4 From Rust and Ratatui

- Ratatui has no hyperlink style (open [#1028](https://github.com/ratatui/ratatui/issues/1028),
  [#902](https://github.com/ratatui/ratatui/issues/902)); its official example writes OSC 8 into cell
  symbols in 2-character chunks as "a hacky workaround"
  ([hyperlink example at ratatui-v0.30.2](https://github.com/ratatui/ratatui/blob/ratatui-v0.30.2/examples/apps/hyperlink/src/main.rs)).
  Herdr keeps a separate hyperlink table next to its Ratatui buffer and emits OSC 8 itself (render_ansi.rs above).
- Launchers: `open` 5.4.4 (`that`, `with(path, app)`; macOS runs `/usr/bin/open [-a app] -- path`)
  ([docs](https://docs.rs/open/5.4.4/open/), [macos.rs](https://github.com/Byron/open-rs/blob/v5.4.4/src/macos.rs));
  `opener` 0.9.0 with `open_browser` and `reveal` ([docs](https://docs.rs/opener/0.9.0/opener/));
  `webbrowser` 1.2.4 guarantees a browser, not an editor ([docs](https://docs.rs/webbrowser/1.2.4/webbrowser/)).

## 2. Opening a PR, commit or run

GitHub CLI already gives each target one command, and every one can print the URL instead of opening
it (useful for OSC 8 links and for a web page):

- `gh browse main.go:312`, `gh browse main.go:312 --blame`, `gh browse main.go --branch <b>`,
  `gh browse main.go --commit=<sha>`, `gh browse <sha>` (commit page), `gh browse 217` (issue or PR);
  `-n/--no-browser` prints the URL ([manual](https://cli.github.com/manual/gh_browse)).
- `gh pr view --web`; `gh pr view --json url --jq .url` ([manual](https://cli.github.com/manual/gh_pr_view)).
- `gh run view --web` ([manual](https://cli.github.com/manual/gh_run_view)).

## 3. The running app of a worktree

Three port strategies and one naming strategy are in use.

| Strategy | Tool | Mechanism |
|---|---|---|
| **Allocate a range, pass it in env** | Conductor | `CONDUCTOR_PORT` = first of 10 ports per local workspace; setup / run / archive scripts in `.conductor/settings.toml` ([env vars](https://www.conductor.build/docs/reference/environment-variables), [scripts](https://www.conductor.build/docs/reference/scripts)) |
| | Emdash | `EMDASH_PORT = 50000 + (hash(taskPath) % 1000) * 10`, no collision check; `.emdash.json` scripts ([workspace-env.ts at 873a3e2](https://github.com/generalaction/emdash/blob/873a3e2067f4abc136272ed2b61abea3a2c07bcf/apps/emdash-desktop/src/core/features/workspaces/api/node/workspace-env.ts)) |
| | Agent Orchestrator | `.ao/launch.json` with `${PORT}`; binds a preferred port or `127.0.0.1:0`; `ao preview [url|file]` stores `preview_url` per session ([manager.go at 7b212c3](https://github.com/Untrivial-ai/agent-orchestrator/blob/7b212c3f01bea2683de187f99f04cdbe5318b378/backend/internal/previewserver/manager.go), [preview.go](https://github.com/Untrivial-ai/agent-orchestrator/blob/7b212c3f01bea2683de187f99f04cdbe5318b378/backend/internal/cli/preview.go)) |
| | Claude Code desktop | `.claude/launch.json` with `runtimeExecutable`, `port`, `autoPort` (free port in `PORT`), `url` (may be `*.localhost`, or attach to a running server) ([desktop docs](https://code.claude.com/docs/en/desktop.md)); per-worktree servers **unconfirmed** |
| **Hash the branch** | Worktrunk | `{{ branch \| hash_port }}` → 10000–19999, same on any machine; `[list] url = "http://localhost:{{ branch \| hash_port }}"` adds a URL column to `wt list`, dimmed when nothing listens; JSON has `dev_server.{url,listening}`; hooks `pre-/post-` × switch, start, commit, merge, remove ([hook.md at 3949c21](https://github.com/max-sixty/worktrunk/blob/3949c21124a940a4fa79f4d8a7c17b354f9438eb/docs/src/content/docs/hook.md), [list.md](https://github.com/max-sixty/worktrunk/blob/3949c21124a940a4fa79f4d8a7c17b354f9438eb/docs/src/content/docs/list.md), [tips-patterns.md](https://github.com/max-sixty/worktrunk/blob/3949c21124a940a4fa79f4d8a7c17b354f9438eb/docs/src/content/docs/tips-patterns.md)) |
| **Discover what listens** | Superset | no port ranges; `ps` + `lsof`, attributed to a workspace by the inherited `SUPERSET_TERMINAL_ID` ([ports.mdx at 65bc1ab](https://github.com/superset-sh/superset/blob/65bc1abb95b953cec2540c51ee28e37a217552e7/apps/docs/content/docs/ports.mdx), [terminal-env.ts](https://github.com/superset-sh/superset/blob/65bc1abb95b953cec2540c51ee28e37a217552e7/packages/port-scanner/src/terminal-env.ts)) |
| | cmux | batched `ps -t <ttys>` + `lsof -p <pids>` per panel, shown in the sidebar ([PortScanner.swift at 9b0d37a](https://github.com/manaflow-ai/cmux/blob/9b0d37a42c573fe9f6f20fd7eacd6972c15d5e5b/Sources/PortScanner.swift)) |
| | Emdash | also scans terminal output for URLs and probes the port ([runtime.ts](https://github.com/generalaction/emdash/blob/873a3e2067f4abc136272ed2b61abea3a2c07bcf/packages/core/src/runtimes/terminals/node/runtime/runtime.ts)) |
| **Name instead of port** | portless | proxy on 443/80 routes `https://<name>.localhost` to a random `PORT` 4000–4999; in a linked worktree the branch's last segment is prefixed: `fix-ui.myapp.localhost` ([README at 67c4739](https://github.com/vercel-labs/portless/blob/67c4739921eb9f77d4721fa610abcb69f08184bb/README.md), [auto.ts](https://github.com/vercel-labs/portless/blob/67c4739921eb9f77d4721fa610abcb69f08184bb/packages/portless/src/auto.ts)) |

- Conductor's **Spotlight** does the opposite: it syncs one workspace's changes into the root checkout
  so a single running stack hot-reloads ([Spotlight](https://www.conductor.build/docs/reference/scripts/spotlight-testing)).
  The Codex app has no port mechanism and points to Handoff to Local for running the app
  ([git worktrees](https://learn.chatgpt.com/docs/environments/git-worktrees)).
- `*.localhost` names resolve to loopback per RFC 6761 §6.3
  ([RFC](https://www.rfc-editor.org/rfc/rfc6761.html)); portless notes Safari may need `/etc/hosts`
  (README above). Prior art: puma-dev `<name>.test` ([repo](https://github.com/puma/puma-dev)).
- Herdr plugins already exist for Worktrunk and worktree setup (`herdr-worktrunk`,
  `herdr-worktree-setup`; see [command-centers-landscape §6a](2026-09-29-command-centers-landscape.md)).

## 4. Images and media in the terminal

### 4.1 Protocols and terminals

- **kitty graphics protocol.** Its Unicode placeholder mode (`U+10EEEE`, kitty 0.28+) lets images live
  inside "any host application that supports Unicode, foreground colors (tmux, vim…)"
  ([spec](https://sw.kovidgoyal.net/kitty/graphics-protocol/)). Implemented by kitty, Ghostty,
  WezTerm (behind `enable_kitty_graphics`, placeholders not done per
  [#986](https://github.com/wezterm/wezterm/issues/986)), iTerm2, Konsole, Warp and others (spec list).
- **iTerm2 inline images** (`OSC 1337 ; File=`) in iTerm2 and WezTerm; iTerm2 also does sixel (3.3.0)
  and kitty graphics ([iTerm2 images](https://iterm2.com/documentation-images.html),
  [3.3.0 changelog](https://iterm2.com/downloads/stable/iTerm2-3_3_0.changelog)). **Sixel** in WezTerm
  (experimental), iTerm2, foot, Windows Terminal ([WezTerm features](https://wezterm.org/features.html)).
- **Ghostty**: kitty graphics only ([features](https://ghostty.org/docs/features)); its VT library lists
  OSC 1337 `File` as unimplemented and has no sixel parser (libghostty-vt as vendored in Herdr,
  `src/terminal/osc/parsers/iterm2.zig`).
- **Apple Terminal and Alacritty** have no pixel protocol (Alacritty
  [#910](https://github.com/alacritty/alacritty/issues/910) open; Apple Terminal per chafa's terminal
  database, `chafa-term-db.c` lines 594–603 at
  [52acc36](https://github.com/hpjansson/chafa/blob/52acc36e4781b97364f0d8591818613e9128b861/chafa/chafa-term-db.c);
  no vendor statement, **unconfirmed**).

### 4.2 Herdr: kitty graphics re-rendered, no sixel, no iTerm2 images

- Herdr is built on a vendored libghostty-vt (`crates/ghostty-vt`, `vendor/libghostty-vt.vendor.json`
  at [f0bb782](https://github.com/herdrdev/herdr/tree/f0bb7827e4a16b59312cef54cb66349b7e8de8e9)).
- It **parses kitty graphics from pane output and re-emits them** to the host terminal, cropped
  around overlays; on by default since 0.9.0, `[terminal] kitty_graphics = false` turns it off
  ([configuration.mdx lines 524–547](https://github.com/herdrdev/herdr/blob/f0bb7827e4a16b59312cef54cb66349b7e8de8e9/docs/next/website/src/content/docs/configuration.mdx),
  [kitty_graphics.rs](https://github.com/herdrdev/herdr/blob/f0bb7827e4a16b59312cef54cb66349b7e8de8e9/src/kitty_graphics.rs)).
  Unicode placeholders render since 0.6.0 (#136). 0.9.2 removed the Herdr-specific `pane.graphics.*`
  API: "Apps show images by writing standard Kitty graphics to their terminal, which Herdr renders
  natively" (#4561, [CHANGELOG](https://github.com/herdrdev/herdr/blob/v0.9.2/CHANGELOG.md)).
- Fast file transport (`t=f`) is used only for local Ghostty, WezTerm or kitty hosts, not over SSH,
  tmux or screen ([handshake.rs lines 58–69](https://github.com/herdrdev/herdr/blob/f0bb7827e4a16b59312cef54cb66349b7e8de8e9/src/client/handshake.rs)).
- **Sixel and iTerm2 images are not supported and not passed through**: "Herdr's documented image path
  currently supports Kitty graphics only… not passed through because Herdr parses pane output and
  redraws the pane" ([#3030](https://github.com/herdrdev/herdr/issues/3030); also
  [#4330](https://github.com/herdrdev/herdr/issues/4330)).
- **Open bug**: kitty images in panes do not render when the Herdr client runs in iTerm2
  ([#3941](https://github.com/herdrdev/herdr/issues/3941), p2; root cause **unconfirmed**).
- Pane programs see `TERM=xterm-256color` and `TERM_PROGRAM=herdr`
  ([ghostty-vt lib.rs line 34](https://github.com/herdrdev/herdr/blob/f0bb7827e4a16b59312cef54cb66349b7e8de8e9/crates/ghostty-vt/src/lib.rs),
  [pane.rs lines 97–99](https://github.com/herdrdev/herdr/blob/f0bb7827e4a16b59312cef54cb66349b7e8de8e9/src/pane.rs)),
  so an image library must probe with escape queries; Yazi inside Herdr on a Windows Terminal host
  picked sixel and showed nothing ([#1324](https://github.com/herdrdev/herdr/issues/1324)).
- Herdr has no web client of its own (no HTTP server in `Cargo.toml`). A third-party xterm.js client
  states "Kitty in-pane images are not supported by this xterm.js view"
  ([sousavf/herdr-web](https://github.com/sousavf/herdr-web)); another opens file paths in a viewer for
  images, video, PDF instead ([Kinetic27/herdr-web-ui](https://github.com/Kinetic27/herdr-web-ui)).
- For comparison, tmux needs `allow-passthrough on|all` and has sixel since 3.4
  ([CHANGES](https://github.com/tmux/tmux/blob/3.7c/CHANGES)); Zellij has sixel since 0.31 and kitty
  graphics since 0.45, with placeholders rejected ([#5530](https://github.com/zellij-org/zellij/issues/5530)).

### 4.3 Showing screenshots in a Ratatui TUI

- **ratatui-image** supports sixel, kitty and iTerm2, guesses from env then queries the terminal, falls
  back to half-blocks; its kitty backend uses Unicode placeholders (`U=1`), which is the mode Herdr
  renders ([kitty.rs line 388 at 5cec787](https://github.com/benjajaja/ratatui-image/blob/5cec7871e4414fe7e252358432d0353427c38372/src/protocol/kitty.rs),
  [README](https://github.com/benjajaja/ratatui-image)). Ratatui core has no image support
  ([#1227](https://github.com/ratatui/ratatui/issues/1227)).
- Others: `viuer` (half-blocks by default, kitty, iTerm, sixel) ([repo](https://github.com/atanunq/viuer));
  `chafa --format [iterm|kitty|sixels|symbols]` ([man](https://hpjansson.org/chafa/man/));
  `timg` ([repo](https://github.com/hzeller/timg)). Yazi documents the same protocol matrix and tmux
  caveats ([image preview](https://yazi-rs.github.io/docs/image-preview/)).
- That ratatui-image renders correctly inside Herdr in the owner's host terminal was **not tested**
  (no input was sent to live panes).

### 4.4 When a browser page is the better surface

- **Video.** No terminal protocol carries video; mpv's `--vo=kitty` / `sixel` push frames
  ("not synchronized with other terminal output") ([vo.rst](https://github.com/mpv-player/mpv/blob/master/DOCS/man/vo.rst));
  in Herdr #3941 `timg -V` video "renders but is too slow to be usable".
- **Rich reports.** Playwright's HTML report (screenshots, video, traces) is served on
  `localhost:9323` and `show-trace` opens a local viewer
  ([reporters](https://playwright.dev/docs/test-reporters), [trace viewer](https://playwright.dev/docs/trace-viewer)).
- Products that show screenshots or previews all use an embedded browser pane (§5).

## 5. How workspace managers present one finished task

| Product | First view of a task | Changed files | Open file at a line | Preview | Link back / "done" signal |
|---|---|---|---|---|---|
| **Conductor** (docs only) | not described (**unconfirmed**); flow: run → Diff Viewer (⌘⇧D) → Review → Checks → Create PR → merge → archive; a "suggested action" button | file list, unified diff, filter by commit; Checkpoints show the latest turn ([diff viewer](https://www.conductor.build/docs/reference/diff-viewer), [checkpoints](https://www.conductor.build/docs/reference/checkpoints)) | `Open In` ⌘O opens the workspace directory in Cursor/VS Code; at a line not documented ([migrate](https://www.conductor.build/docs/guides/migrate-from-cursor)) | in-app browser after running from the Run tab (0.62.0) ([changelog](https://www.conductor.build/changelog/0.62.0-repo-settings-browser-preview)) | Checks tab: git status, PR, CI, deployments, comments, todos ([checks](https://www.conductor.build/docs/reference/checks)); ⌘⌥L next workspace needing attention |
| **Codex app** (docs only) | review pane, default **Unstaged**; scopes Staged, Commit, Branch, **Last turn** ([code review](https://learn.chatgpt.com/docs/code-review)) | whole repo state; per-file **Mark as viewed** in the PR view | ⌘-click a line opens that line in the chosen editor (editor list undocumented) | built-in browser with Annotate ([browser](https://learn.chatgpt.com/docs/browser)) | PR **Threads** link to chats; turn-completion alerts; Activity view (⌘⌥U) ([notifications](https://learn.chatgpt.com/docs/notifications)) |
| **Claude Code desktop** | `+12 -1` indicator opens file list and per-file diffs; line comments ([desktop](https://code.claude.com/docs/en/desktop.md)) | order undocumented | right-click → **Open in** VS Code, Cursor, Zed; at a line undocumented | Browser pane, auto-verify with screenshots; HTML/PDF/image/video paths open in it | CI status bar after PR; OS notification when a task finishes; CLI `/diff` live panel with +/- counts ([2026-w36](https://code.claude.com/docs/en/whats-new/2026-w36.md)) |
| **Superset** | Changes view; Focus mode walks one file at a time grouped Against base / Commits / Staged / Unstaged ([diff-viewer.mdx at 65bc1ab](https://github.com/superset-sh/superset/blob/65bc1abb95b953cec2540c51ee28e37a217552e7/apps/docs/content/docs/diff-viewer.mdx)) | grouped by git state, not priority | `openFileInEditor` takes `line`/`column` but does not pass them on ([external router](https://github.com/superset-sh/superset/blob/65bc1abb95b953cec2540c51ee28e37a217552e7/apps/desktop/src/lib/trpc/routers/external/index.ts)) | browser pane (⌘⇧B or click a port); Design mode sends DOM + screenshot to an agent ([browser.mdx](https://github.com/superset-sh/superset/blob/65bc1abb95b953cec2540c51ee28e37a217552e7/apps/docs/content/docs/browser.mdx)) | sidebar strip: agents, ports, PR; finished / waiting notifications via hooks ([agent-status.mdx](https://github.com/superset-sh/superset/blob/65bc1abb95b953cec2540c51ee28e37a217552e7/apps/docs/content/docs/agent-status.mdx)) |
| **Agent Orchestrator** | board Working / Needs you / In review / Ready to merge; card opens conversation or terminal, changed files, PR summary, reviews, preview ([README at 7b212c3](https://github.com/Untrivial-ai/agent-orchestrator/blob/7b212c3f01bea2683de187f99f04cdbe5318b378/README.md)) | diff scopes combined/committed/staged/unstaged/untracked (OpenAPI) | 22 editor ids, opens the session workspace, not a line ([editor-handoff.ts](https://github.com/Untrivial-ai/agent-orchestrator/blob/7b212c3f01bea2683de187f99f04cdbe5318b378/frontend/src/main/editor-handoff.ts)) | `ao preview`, isolated browser profile per worker | notifications only `needs_input`, `ready_to_merge`, `pr_merged`, `pr_closed_unmerged` ([notification.go](https://github.com/Untrivial-ai/agent-orchestrator/blob/7b212c3f01bea2683de187f99f04cdbe5318b378/backend/internal/domain/notification.go)) |
| **Emdash** | Changes list: Changed / Staged / Pull Requests ([diff view](https://emdash.com/docs/diff-view)) | status icon per file, no priority | ~30 "Open in" targets as shell templates (`code {{path}}`), directory only ([open-in-apps.ts at 873a3e2](https://github.com/generalaction/emdash/blob/873a3e2067f4abc136272ed2b61abea3a2c07bcf/apps/emdash-desktop/src/core/primitives/open-in-apps/api/open-in-apps.ts)) | dev-server pill → Emdash Browser ([in-app browser](https://emdash.com/docs/in-app-browser)) | hooks installed into agent config |
| **cmux** | vertical tab shows branch, PR status and number, cwd, ports, latest notification ([README at 9b0d37a](https://github.com/manaflow-ai/cmux/blob/9b0d37a42c573fe9f6f20fd7eacd6972c15d5e5b/README.md)) | `cmux diff --source unstaged\|staged\|branch\|last-turn` ([CLI help](https://github.com/manaflow-ai/cmux/blob/9b0d37a42c573fe9f6f20fd7eacd6972c15d5e5b/CLI/CMUXCLI+TaskHelp.swift)) | ⌘-click `path:line` runs the configured editor with `'path:line:col'` as one argument ([PreferredEditorService.swift lines 190–245](https://github.com/manaflow-ai/cmux/blob/9b0d37a42c573fe9f6f20fd7eacd6972c15d5e5b/Packages/macOS/CmuxWorkspaces/Sources/CmuxWorkspaces/FileOpen/PreferredEditorService.swift)); VS Code needs `-g` for that form, so plain `code` would not jump (inferred, **unconfirmed**) | built-in browser | blue ring on a waiting pane, ⌘⇧U jumps to latest unread; notifications from OSC 9/99/777 |
| **roamgate** | Inspector: Files / Changes / Agent History; scopes Working tree / Against main / Last step ("not proof of agent ownership") ([FEATURES.md at b7b02ce](https://github.com/powerfooI/roamgate/blob/b7b02ce128db9bc67d21879f5492501b7494ef27/FEATURES.md)) | badges and +/- counts; large or generated diffs collapsed | no external editor; "Reveal on host" | static HTML preview, scripts blocked | read-only PR card; annotations pre-fill the agent pane; Web Push on completed |
| **herdr-reviewr** | tabs Changes / All files / PR; starts in uncommitted; scopes uncommitted / branch / last turn / commits ([README at cd618b4](https://github.com/persiyanov/herdr-reviewr/blob/cd618b48dc1f4d2cc092f3f26827ac72fb44773e/README.md)) | sorted by path; last turn polled every 2 s ([git.rs](https://github.com/persiyanov/herdr-reviewr/blob/cd618b48dc1f4d2cc092f3f26827ac72fb44773e/src/git.rs)) | `e` opens at the line in four editor dialects (§1.2) | none | read-only PR tab, `o` opens the PR in the browser; auto-opens on a new Herdr worktree workspace |

Cross-cutting facts:

1. **No product documents a "where to look first" order.** Files are grouped by git state or sorted by
   path. The closest aids are Codex's per-file "Mark as viewed" and Conductor's review checklist
   (unrelated edits, missing tests, conflicts) ([review and merge](https://www.conductor.build/docs/guides/review-and-merge)).
   Ordered diffs exist in products outside this list (Devin Review, Amp tour; see
   [agent-products-human-artifacts §8](2026-09-27-agent-products-human-artifacts.md)).
2. **A "last turn" scope is common**: Codex, Claude CLI `/diff`, cmux, roamgate, herdr-reviewr,
   Conductor Checkpoints. roamgate and herdr-reviewr state it is a heuristic.
3. **Opening at a line in an external editor** is implemented fully only in the Codex app (⌘-click a
   line) and herdr-reviewr (dialect table). Superset drops the line; Emdash, Agent Orchestrator and
   Conductor open the directory.
4. **Previews are an embedded browser pane** in every GUI product (Conductor, Codex, Claude desktop,
   Superset, Emdash, cmux); none renders the running app inside a terminal.
5. **"Done" signals** come from hooks installed into agents (Superset, Emdash), terminal notification
   escape codes (cmux), or PR state (Agent Orchestrator, Claude desktop CI bar).

## 6. Markdown artifacts: terminal rendering versus a local page

| Renderer | Images | Links | Tables | Mermaid | Status |
|---|---|---|---|---|---|
| glamour / glow (Go) | alt text + URL as OSC 8, no pixels ([image.go lines 18–50 at 49df656](https://github.com/charmbracelet/glamour/blob/49df6562f7a3740f872c3c96d46d3257aef30e56/ansi/image.go)) | OSC 8 ([link.go](https://github.com/charmbracelet/glamour/blob/49df6562f7a3740f872c3c96d46d3257aef30e56/ansi/link.go)) | fitted to width with wrapping | no | active; `gh pr view` renders with glamour, wrap capped at 120 ([markdown.go](https://github.com/cli/cli/blob/trunk/pkg/markdown/markdown.go)) |
| tui-markdown (Ratatui) | `[img] alt` text | — | box drawing | no | 0.3.10, "experimental Proof of Concept" ([repo at 9143707](https://github.com/joshka/tui-markdown/tree/9143707da309d4924938334697b8fad84d332dfe/tui-markdown)) |
| termimad (Rust) | no | not supported | balanced to width | no | "a terminal isn't really fit for that" ([repo](https://github.com/Canop/termimad)) |
| mdcat | inline via kitty / iTerm2 | OSC 8 | no wrapping in cells | no | **archived** ([repo](https://github.com/swsnr/mdcat)) |
| frogmouth (Textual) | no | browser-like history | yes | no | last push 2024-08 ([repo](https://github.com/Textualize/frogmouth)) |
| gh-markdown-preview (local HTML) | yes, relative files served | yes | GitHub's | Mermaid and MathJax added client-side | renders via `gh api -X POST /markdown`, live reload on :3333, opens the browser ([repo at df155e2](https://github.com/yusukebe/gh-markdown-preview/tree/df155e2440c9be50d32decb35f2a632d7cd200f3)) |
| grip (local HTML) | yes | yes | GitHub's | — | GitHub API, rate-limited without a token; last push 2024-07 ([repo](https://github.com/joeyespo/grip)) |

- GitHub's `POST /markdown` (`mode: gfm`, `context: owner/repo`) returns GitHub-identical HTML with
  `#42` links resolved ([REST docs](https://docs.github.com/en/rest/markdown/markdown)); whether it
  renders Mermaid or math server-side is **unconfirmed** (gh-markdown-preview adds both in the page).
- In the terminal a Markdown artifact keeps structure, links (OSC 8) and tables that fit; it loses
  images, video, Mermaid and wide tables. A local page keeps all of them and matches what GitHub will
  show.

## Implications for the command center

Facts and options for the owner; no decisions are made here.

- **Open at a line is a solved, small piece.** A per-editor dialect table (`+N`, `file:N`,
  `-g file:N`, `--line N`) keyed by `$VISUAL`/`$EDITOR`, plus a template override, covers the common
  editors; herdr-reviewr and lazygit already ship this. Options: reuse herdr-reviewr's `e` for diffs;
  copy the dialect approach into our own code; or emit `vscode://file/…:N` URLs (VS Code shows a
  confirmation dialog unless the owner turns it off).
- **Herdr offers a Ctrl-click path.** Pane output with OSC 8 `file://host/path#N` links, plus a Herdr
  plugin `[[link_handlers]]` entry matching `#\d+$`, runs any editor command. Plain `path:line` text is
  not clickable in Herdr; the program must emit OSC 8. Ratatui needs the documented workaround to emit
  OSC 8.
- **PR, commit, file on GitHub** need no own code: `gh browse <file>:<line>`, `gh browse <sha>`,
  `gh pr view --web`, `gh run view --web`, and `-n` or `--json url` to print links instead.
- **The running app of a worktree** has three proven shapes: a deterministic port per branch
  (Worktrunk `hash_port`, which also exposes `dev_server.{url,listening}` in the JSON output of `wt list`),
  a stable name per branch (portless `branch.app.localhost`), or discovering listeners with
  `ps`/`lsof` (Superset, cmux). The first two are facts the command center could read without starting
  anything; the third reads live state.
- **Images in the terminal work only through kitty graphics in Herdr**, with an open iTerm2-host bug and
  no sixel or iTerm2 images. ratatui-image's kitty placeholder mode matches what Herdr renders, but was
  not tested here. Video has no terminal path. Every GUI product uses a browser pane for previews and
  media.
- **Markdown artifacts:** terminal renderers (glamour, tui-markdown) keep text, links and tables but
  drop images and Mermaid; a local page rendered through GitHub's Markdown API (the gh-markdown-preview
  approach) matches GitHub and keeps media. Options range from "TUI shows the card, a key opens the
  page" to a web-only artifact view.
- **What the products put first for a finished task** is a diff scoped by git state or "last turn",
  PR and CI status, a preview pane and a "needs you" signal. None orders files by importance; a
  "where to look first" list would have to come from the agent's own report (the result card in the
  PR), not from the tools surveyed.
- **Limits of this note.** Conductor and the Codex app are described from docs, not source. The
  `cursor://` and `jetbrains://` URL forms, Zed's URL position handling, Ghostty's `#line` handling,
  cmux with plain `code`, and ratatui-image inside Herdr were not run.
