# Canvas and spatial workspaces for terminals and agents (state as of 2026-09-29)

Date: 2026-09-29. Ticket: #43 (map #41). Method: repository metadata through the GitHub REST API
(`gh api repos/<repo>`, releases, contributors; stars and last push on that date); READMEs, docs,
manifests and source trees read at pinned commits; the Herdr plugin registry index
(`assets.herdr.dev/plugins/index.json`, 1379 entries on that date) scanned for canvas, graph and
overview plugins; Hacker News threads through the Algolia API for user reports; three HCI papers for
evidence on overviews and zooming. No app was installed or run. Links to sources are in the text and
in the reference list at the end. "Unconfirmed" marks what the sources did not settle. User reports
from Hacker News are secondary sources and are labelled as such.

This note does not repeat the command-center landscape (`2026-09-29-command-centers-landscape.md`):
Paperclip, the Herdr plugin catalogue and the web clients are covered there. Here the question is
spatial: canvases, zoom and notes.

## 1. CanvasTTY (howdeploy/CanvasTTY) in detail

### 1.1 Identity and maturity

- An Electron "spatial desktop for real local PTYs and AI-agent CLI sessions", described as "Your
  infinite desktop just for your AI agents. Inspired by DriftWM." [ctty-readme], [ctty-repo]
- MIT; TypeScript; created 2026-08-05; 82 stars, 19 forks, 6 open issues, last push 2026-09-29
  [ctty-repo]. Releases v1.2.8 (2026-08-25) through v1.5.2 (2026-09-21); `package.json` on `main` is
  already 1.7.0 [ctty-releases], [ctty-pkg].
- 12 contributors; the top account has 121 of 306 attributed commits [ctty-contrib].
- Stack: Electron 43 with electron-vite, React 19, xterm.js 6 with node-pty 1.1 [ctty-pkg]. The
  dependency list has no canvas or graph library (no React Flow, tldraw, Konva or Pixi), so the canvas
  is custom code under `src/renderer/src/features/workspace/` [ctty-pkg], [ctty-tree].
- Providers: Codex, Claude Code, Qwen Code, Kimi, OpenCode, Hermes, Grok Build, plus `omp` and `pi`
  launchers [ctty-start]. Builds: Linux AppImage/deb, Windows x64 (unsigned), Apple Silicon macOS
  (ad-hoc signed, not notarized); no Intel Mac build [ctty-readme].
- Three languages of docs (English, Russian, Simplified Chinese) and nine ADRs in `docs/adr/`
  [ctty-tree]. Open issues include "Native browser view obscures canvas overlays and widgets" (#14) and
  a user wish list asking for remote control from a phone, own files on the board and a status for
  the `omp` agent (#53)
  [ctty-issues].

### 1.2 The canvas: Home zone, cards, zoom, regions and notes

All from the UI contract [ctty-ui] unless noted.

- **Home zone.** A fixed grid of `82 × 72` px cells (default `16 × 12`, up to `48 × 36`) at a fixed
  place on the canvas. It holds a provider-limits tile, the session list, a clock, a media tile, a
  launcher dock and Settings. An Edit HOME mode moves and resizes tiles; plugins add their own Home
  widgets. A "Go to HOME" shortcut returns the camera there.
- **Cards.** Live terminals, the built-in browser, plugin canvas apps and notes are movable,
  resizable cards with a `54px` header. The minimum terminal card is `420 × 260`; resizing updates the
  xterm viewport. With snapping on, drag and resize use a hidden `10px` grid, magnetic edges and a
  `20px` gap. Cards have no maximize button; a separate shortcut toggles terminal fullscreen.
- **Semantic zoom.** Below `0.5×` a terminal card switches to a summary; its typography counter-scales
  as the camera moves out so cards stay readable. Clicking a summary selects the card. The getting
  started guide describes this as "zoom out to use semantic summaries as navigation targets; zoom
  back in to interact" [ctty-start].
- **Navigation.** Pan by drag or scroll, zoom by pinch or `Cmd/Ctrl + scroll`; an RTS-style minimap
  with a fixed-scale camera rectangle; optional edge panning; a `Cmd/Ctrl+K` palette over live
  sessions and launch actions. An ADR records the wheel-routing rules between canvas and focused card
  [ctty-adr-nav].
- **Regions.** Named colour fields drawn behind cards. Dragging a region moves every card whose
  bounds were fully inside it at drag start; deleting a region never deletes its contents.
- **Sticky notes.** Persisted text and geometry, snap like other cards, resize from every edge.
  Regions and notes each have their own "save after exit" switch (both on by default).
- **Launching.** Right-click on empty canvas offers a region, a sticky note, the launcher submenu,
  Browser and Settings; an opt-in radial menu launches on right-button hold.
- **Restore.** An opt-in restart policy saves window identity, provider, folder, position and size
  and asks each agent CLI to resume its latest session; scrollback is not persisted [ctty-ui],
  [ctty-arch].

### 1.3 Session and provider-limit views

- **Sessions.** The right Home tile "is the only session list": provider mark, localized state and
  identity per row, colour by status (working, waiting for input, idle), and the final output of a
  failed session on hover [ctty-ui]. An on-canvas attention panel and system notifications cover
  sessions that need attention [ctty-ui].
- **Status comes only from provider hooks.** Agents report `idle`, `working` and `needs_approval`
  through lifecycle hooks; an agent stays `unavailable` until the first structured signal. "Terminal
  text and PTY existence are not activity telemetry" [ctty-start], [ctty-ui].
- **Provider limits.** `LimitsService` reads subscription windows through structured adapters: Codex
  through the CLI app-server protocol, Claude and Kimi through their usage endpoints. It keeps a
  60-second cache with a stale fallback, shows the longest real window and its reset countdown, and
  shows "unavailable" instead of `0%` [ctty-metrics], [ctty-ui]. Per-session token accounting is "not
  implemented as a public CanvasTTY API yet" [ctty-metrics].
- The metrics doc states a design rule that applies to any overview: "CanvasTTY treats telemetry as a
  truth problem before it treats it as a visualization problem" [ctty-metrics].

### 1.4 Plugin SDK: widgets, canvas apps, services

From the plugin guide [ctty-plugins] unless noted.

- **Install.** A plugin is a GitHub repository with `canvastty.plugin.json` at its root. CanvasTTY
  downloads the default-branch tarball and never runs `npm install` or repository scripts; limits are
  500 files and 25 MB.
- **Web contributions** (manifest `apiVersion: 1`): `home-widget` (tile on the Home grid),
  `canvas-app` (a movable card with the same header, snapping and semantic summary as terminals) and
  `window` (a separate sandboxed window). They run in sandboxed iframes with an opaque origin, no
  Node.js and no access to the host DOM. "Arbitrary native window embedding is not part of the
  contract" [ctty-ui].
- **Native code** (`apiVersion: 2`): agent hooks and long-lived services (JSON-RPC over stdio), each
  off until the user trusts it per plugin; update, disable or a changed entry file revokes trust.
- **Permissions** gate every SDK call: `sessions:read` (id, provider, title, status only),
  `sessions:events` (metadata including role and parent session), `sessions:read-screen` (last 4000
  characters of output, masked), `sessions:launch` and `sessions:control` (only cards the plugin
  started), `cards:decorate` (badges and card actions), `limits:read`, `network` (HTTPS and loopback
  fetch), and others.
- Examples in the repo: a notes canvas app, a status Home widget, a git-worktree session environment,
  a launch policy and a command-deny decision service [ctty-tree].
- Session roles are `agent`, `orchestrator` and `subagent` with `parentSessionId` [ctty-plugins]; an
  opt-in local control endpoint and CLI let an orchestrator create, prompt and read Codex workers
  without taking the user's focus [ctty-orch].

### 1.5 The parts the owner named, mapped to code

The ticket names notes, arranging windows and the overview. In the source tree these are separable
renderer features [ctty-tree], [ctty-arch]:

| Idea | Where it lives |
|---|---|
| Notes | `features/notes/StickyNoteCard.tsx` |
| Grouping by area | `features/workspace/CanvasRegionCard.tsx`, `canvasRegions.ts` |
| Arranging and resizing cards | `WorkspaceCanvas.tsx`, `snap.ts`, `canvasStacking.ts`, `sessionPlacement.ts` |
| Overview and navigation | `CanvasMinimap.tsx`, semantic summary in `TerminalCard.tsx`, `CanvasCommandPalette.tsx` |
| Fixed "home" with sessions and limits | `features/home/HomeZone.tsx`, `attentionQueue.ts`, `LimitsService.ts` |

The code is TypeScript in one Electron renderer; reusing it outside CanvasTTY would mean extracting
it (MIT allows this). How coupled these files are to the rest of the app is unconfirmed.

## 2. DriftWM, the inspiration

- A "trackpad-first infinite canvas Wayland compositor" in Rust on Smithay; GPL-3.0-or-later;
  1744 stars; created 2026-02-22; v0.19.0 on 2026-09-14; 28 contributors [driftwm-readme],
  [driftwm-repo], [driftwm-license]. Linux only, since it is a Wayland compositor.
- Model: "windows live at their native size on an infinite 2D canvas, and your display is a camera
  viewing it" [driftwm-readme].
- Ideas CanvasTTY shares: zoom-to-fit overview, a Home toggle (origin and back), named bookmarks for
  camera positions, directional jumps to the nearest window, and snapping [driftwm-readme].
- One idea CanvasTTY does not copy: snapped windows form implicit clusters that move, resize and fit
  together, "No explicit grouping to manage" [driftwm-readme]. CanvasTTY uses explicit regions instead
  [ctty-ui].
- The README warns: "This is experimental software, primarily built with AI" [driftwm-readme].

## 3. Other canvas terminals and agent canvases

### 3.1 The larger projects

- **Cate** (0-AI-UG/cate): "An infinite canvas IDE for parallel coding agents." Terminals report
  running, waiting or finished from agent hooks; one click creates a git worktree "with its own
  colored territory on the canvas"; panels (terminal, Monaco editor, browser, document viewers,
  nested canvases) float on the canvas or dock into tabs and splits; a `cate` CLI lets agents drive
  panels [cate-readme]. MIT, Electron with xterm.js; `package.json` lists none of React Flow, tldraw,
  Konva or Pixi [cate-pkg].
  2163 stars, v2.0.4 on 2026-09-26, 20 contributors [cate-repo].
- **OpenCove** (DeadWaveWave/opencove): "agents, terminals, tasks, and notes on the same infinite 2D
  canvas", persistent workspaces, "space archives" (snapshots of past workspace states), global search
  across canvas and terminal output, a "control center", worktree isolation, and an experimental
  worker-hosted Web UI for browsers on the LAN [opencove-readme]. Canvas engine: `@xyflow/react`
  (React Flow) [opencove-readme], [opencove-pkg]. MIT, 1601 stars, alpha, nightly builds
  [opencove-repo].
- **Horizon** (peters/horizon): a "GPU-accelerated terminal board" in Rust (egui, wgpu) with five
  nouns: canvas, colour-coded workspaces with a shared cwd, panels (shell, SSH, agent, browser, git,
  markdown, usage), presets and saved sessions; a minimap; fit-to-workspace; an attention feed that can
  be disabled [horizon-readme], [horizon-repo]. MIT, 713 stars, v0.2.7 [horizon-repo]. Built "in 3
  days with Claude/Codex" per its Show HN post [hn-horizon].
- **TermCanvas** (blueberrycongee/termcanvas): terminals on a canvas in a Project → Worktree →
  Terminal hierarchy; status dots per agent tile; inline diff cards; a Sessions panel listing past
  Claude and Codex conversations; starred-terminal cycling; a `termcanvas state` command that dumps
  the canvas as JSON; the Hydra orchestration CLI [termcanvas-readme]. Canvas engine: `@xyflow/react`
  [termcanvas-pkg]. MIT, 406 stars, but last push 2026-05-31 [termcanvas-repo].
- **Maestri** (closed source, macOS, with Windows and Linux versions per its site): terminals, sticky
  notes, sketches and "portals" on a Metal-rendered canvas; drawing a line between two terminals lets
  one agent type into the other's PTY; Swift and SwiftUI; free tier with one workspace, Pro $18
  one-time; "zero telemetry" [maestri-site].
- **VoiceTree** (voicetreelab/voicetree): "The spatial IDE for recursive multi-agent orchestration.
  It's like an Obsidian graph-view that you work directly inside of." 921 stars, last push 2026-06-07
  [voicetree-repo]. Its README could not be fetched at the default path (unconfirmed details).

### 3.2 Small or young projects

All MIT unless noted; stars and last push from the GitHub API on 2026-09-29.

| Repo | Stars | Last push | What it is |
|---|---|---|---|
| ceIia/finite | 45 | 2026-04-02 | Swift, "spatial terminal multiplexer for macOS" [small-repos] |
| ZipLyne-Agency/CodeGrid | 26 | 2026-09-22 | Tauri, "dozens of agent sessions on a 2D canvas" [small-repos] |
| kylesnowschwartz/hgnucomb | 18 | 2026-02-19 | "2D navigable canvas for terminals and Claude agents" [small-repos] |
| DevoidSloth/ccanvas | 15 | 2026-09-26 | Apache-2.0, "Excalidraw, but for multiple coding agents" [small-repos] |
| lossless1/panescale | 9 | 2026-06-19 | Tauri, terminals, files, SSH and browser tiles [small-repos] |
| aleda145/terminaldraw | 8 | 2026-04-24 | "a tldraw canvas but with terminals" [small-repos] |
| Quzr27/Korum | 6 | 2026-07-14 | Tauri canvas terminal workspace [small-repos] |
| kdonev/termscape | 1 | 2026-09-27 | agents on a canvas that talk through an MCP hub [small-repos] |
| twaldin/canvas | 1 | 2026-09-29 | Swift; point at a line, DOM element or note to send it to the agent [small-repos] |
| AgentOrchestrator/AgentBase | 247 | 2026-01-25 | "Figma-like Canvas for running Claude Code agents" per its Show HN title [small-repos], [hn-canvas] |
| haystackeditor/haystack-editor | 1286 | 2025-02-17 | a VS Code-based canvas editor, called "sadly abandoned" by a commenter [small-repos], [hn-cate] |

### 3.3 What the canvases show

- **Sessions as live terminals** are the core node type everywhere in 3.1 and 3.2.
- **Status** (working, waiting, done) is drawn on the tile in Cate, TermCanvas, Horizon and
  CanvasTTY, always from agent hooks rather than screen scraping in Cate and CanvasTTY [cate-readme],
  [termcanvas-readme], [ctty-ui].
- **Grouping** by worktree or project: Cate ("colored territory"), TermCanvas (hierarchy), Horizon
  (workspaces), CanvasTTY (manual regions) [cate-readme], [termcanvas-readme], [horizon-readme],
  [ctty-ui].
- **Notes and sketches**: CanvasTTY, OpenCove, Maestri, TermCanvas ("Free Canvas" drawing)
  [ctty-ui], [opencove-readme], [maestri-site], [termcanvas-readme].
- **Relations between sessions** as edges: Maestri (lines that route PTY input) and termscape; the
  others show no edges between sessions [maestri-site], [small-repos].
- **Tasks**: OpenCove has task nodes; hivemind (krakujs/hivemind, 0 stars) describes a markdown
  Kanban board on its canvas [opencove-readme], [small-repos]. None of the canvases above reads tasks
  from GitHub Issues (unconfirmed for Maestri).
- **Artifacts**: diffs (TermCanvas diff cards, Horizon git panel), browsers and previews (Cate,
  CanvasTTY, Horizon, Maestri) [termcanvas-readme], [horizon-readme], [cate-readme], [ctty-readme],
  [maestri-site].

## 4. Formats and engines

- **JSON Canvas** (obsidianmd/jsoncanvas, MIT, 3707 stars): an open file format from Obsidian for
  infinite-canvas data. Spec 1.0 (2024-03-11) defines `nodes` (types `text`, `file`, `link`, `group`,
  each with `x`, `y`, `width`, `height`, optional `color`) and `edges` (`fromNode`, `toNode`, sides,
  arrow ends, label) [jsoncanvas-spec], [jsoncanvas-repo]. There is no node type for a live session or
  a status; a custom type would be an extension the spec does not define. The launch thread had 841
  points on Hacker News [hn-jsoncanvas].
- Agents already write it: `kepano/obsidian-skills` ships a `json-canvas` skill to "create and edit
  JSON Canvas files (`.canvas`) with nodes, edges, groups, and connections" [obsidian-skills].
- **React Flow** (xyflow/xyflow, MIT, 38542 stars) is the canvas engine of OpenCove and TermCanvas
  [xyflow-repo], [opencove-pkg], [termcanvas-pkg].
- **tldraw** (50647 stars) is under its own licence that permits use in development environments but
  not in production environments without a licence key [tldraw-license]. Agent boards built on it are
  small (for example Mizora-net/ita, a planning whiteboard where the agent writes questions and specs
  as cards into `board.yaml`, 0 stars) [small-repos].
- **rataflow** (furkankly/rataflow, MIT, 177 stars): "Interactive node-based UIs for the terminal",
  built on Ratatui and inspired by React Flow, with pan, zoom, fit-view, minimap and auto-pan while
  dragging [rataflow-readme], [herdr-plugins-meta]. It is the engine under zoetrope [zoetrope-readme].

## 5. Neighbours that chose other shapes

- **Nimbalyst** (MIT, 1801 stars) runs agents in worktrees and tracks sessions on a **kanban board**,
  not a canvas; it also has a session dashboard of "which agents need you" [nimbalyst-readme],
  [nimbalyst-repo].
- **zoetrope** (MIT, 959 stars): draws one Claude Code or Codex session as a live flow graph (main
  agent, subagents, tools) in a terminal or a browser (WASM). It reads the agents' transcript files
  under `~/.claude/projects/` and `~/.codex/sessions/`, and a Herdr plugin opens the focused pane's
  session over the pane [zoetrope-readme], [zoetrope-repo].
- **herdr-dagr** (Apache-2.0, 88 stars, last push 2026-08-23): a multi-agent run as a live DAG in a
  Herdr split pane, drawn "like `git log --graph`", with attempts, gates and evidence tiers
  (verified, reported, heuristic, asserted). Agents write the run file through a producer skill
  [dagr-readme], [dagr-repo].

## 6. Canvas and Herdr together

### 6.1 What Herdr exposes (0.9.x)

- **State and events.** `session.snapshot` plus `events.subscribe` (workspace, tab, pane, layout,
  worktree events, including `pane.agent_status_changed`); the docs describe the bootstrap: subscribe,
  buffer, snapshot, then apply events [herdr-socket].
- **Read-only live terminal streams.** `herdr terminal session observe <pane> --cols N --rows N`
  prints newline-delimited JSON `terminal.frame` records with base64 ANSI bytes. "Multiple observers
  can watch the same terminal without taking input, resize, scroll, or takeover ownership"
  [herdr-remote]. Added in 0.7.2 [herdr-changelog].
- **Interactive bridge.** `herdr terminal session control <pane> --takeover` adds input, resize,
  scroll and release commands on stdin; "Only one controller owns input and resize at a time"
  [herdr-remote]. Mouse events were added in 0.9.2 (2026-09-29) [herdr-changelog].
- **Direct attach.** `herdr agent attach <name>` or `herdr terminal attach <id>` opens one server-owned
  terminal in the current terminal; one writable client owns input and resize (Linux and macOS only)
  [herdr-remote].
- **Graphics.** `pane.graphics.set` and `pane.graphics.stream` place PNG or raw image layers over a
  pane (up to 16 layers) when kitty graphics are enabled [herdr-socket].
- **Plugin limit.** "Runtime action registration and native non-terminal plugin UI are not part of
  plugin v1." Plugin panes are terminal panes [herdr-plugins-doc].
- **Protocol stability.** JSON API clients should ignore unknown fields; the numbered binary protocol
  "remains for same-install and internal operations" [herdr-socket].

### 6.2 Existing browser bridges

- **kcosr/herdr-web** (MIT, 144 stars) vendors Herdr compatibility code "because the app needs
  private Herdr APIs for terminal attach, terminal resize/scroll/input, workspace snapshots, and event
  subscriptions", and fans one attach out to many browsers [herdrweb-readme].
- **powerfooI/roamgate** (MIT, 257 stars) implements Herdr's client endpoint protocol in TypeScript
  (`server/src/bridge/endpoint-terminal-session.ts`, "Terminal stream over the stable endpoint
  protocol (Herdr >= 0.9.0)") and pins protocol versions 14–20 and 22 [roamgate-src], [roamgate-repo].
- Neither is a canvas: both are browser clients with lists, terminals and diffs [roamgate-readme],
  [herdrweb-readme].

### 6.3 No canvas that embeds Herdr panes was found

Searches of GitHub ("herdr canvas", "herdr spatial", "herdr graph", "herdr map", "herdr web") and of
the 1379-entry Herdr registry for canvas, spatial, zoom, graph and overview terms found no infinite
canvas that hosts Herdr panes [herdr-plugins-meta], [small-repos]. What exists inside Herdr is
terminal-native:

| Plugin | Stars | What it shows |
|---|---|---|
| aorumbayev/herdr-canvas | 8 | a mouse-driven ASCII diagram canvas (box, line, text, draw) stored as one JSON file per diagram that agents can read and edit [herdr-canvas-aor] |
| sebi75/herdr-canvas | 6 | an HTML page the agent rewrites each turn, rendered by headless Chrome into a Herdr pane through the graphics API; "nothing on it is clickable" [herdr-canvas-sebi] |
| vjeantet/herdr-mission-control | 3 | an exposé popup: every pane of the workspace as a live tile with status, grouped by tab; pick one to switch [herdr-mc] |
| iamgp/herdr-overview | 1 | "a live tiled overview of every space" [herdr-plugins-meta] |
| neyham/herdr-paddock | 9 | a card wall of agents sorted blocked, working, done; zoom into one and reply, over SSH [herdr-paddock] |
| tyz-works/herdr-task-graph | 0 | "Live terminal DAG for Herdr task dependencies, agent state, and parallel-ready work" [herdr-plugins-meta] |
| furkankly/zoetrope, aemrebarut/herdr-dagr | 959, 88 | session graph and run DAG (section 5) |

### 6.4 Other terminal-native spatial attempts

- **jzombie/term-wm** (Apache-2.0, 5 stars, 69 open issues): floating, z-ordered windows with snapping
  and multi-viewer sessions "rendered entirely in the character grid" over SSH [term-wm-readme],
  [small-repos]. Its README describes no infinite canvas or zoom.
- **rataflow** (section 4) is the only terminal library found with pan, zoom and a minimap
  [rataflow-readme].

## 7. Evidence of value or failure

### 7.1 Research on overviews and zooming

- A survey of overview+detail, zooming and focus+context interfaces concludes "None of these
  approaches is ideal". Overview+detail is preferred in several studies, while zooming "can easily
  create substantial cognitive load" and "is easy to do badly, as indicated by many studies in which
  it has performed poorly"; animation and concurrent pan-and-zoom controls help [cockburn-2008].
- Code Bubbles (CHI 2010): a canvas of concurrently visible code fragments. In a controlled study
  users completed both tasks 33.2% faster than with Eclipse (significant for the total and for task 1,
  not for task 2) and spent less time navigating (mean 3.5 vs 11.6 minutes) [code-bubbles].
- Debugger Canvas (Microsoft, ICSE 2012), the industrial version: the paradigm scaled to real code
  bases and is "best implemented as a mode in the existing user experience rather than a
  replacement" [debugger-canvas].
- No study was found that measures supervision of many coding agents on a canvas against a list.

### 7.2 What the products do

- Every canvas product above also ships a non-spatial index: CanvasTTY's Home session list ("the
  only session list") and `Cmd+K` palette [ctty-ui]; Horizon's attention feed and sidebar
  [horizon-readme]; TermCanvas's Sessions panel and starred cycling [termcanvas-readme]; Cate's docks,
  tabs and `Cmd+K` [cate-readme]; OpenCove's global search and control center [opencove-readme].
- Nimbalyst, which replaced Crystal (`2026-09-29-command-centers-landscape.md`), tracks sessions on a
  kanban, not a canvas [nimbalyst-readme].
- Several canvas projects stopped pushing for four months or more: Haystack (2025-02), AgentBase
  (2026-01), hgnucomb (2026-02), finite (2026-04), TermCanvas (2026-05) [small-repos],
  [termcanvas-repo]. Why they stopped is unconfirmed.

### 7.3 User reports (secondary: Hacker News)

- Cate v1.0 thread (65 points, 66 comments): one user fears "every canvas will become a mess" and
  compares it to an unmanageable Miro board; another prefers a finite canvas; several argue this
  belongs in a window manager; two found the app too janky for daily use; others value "a spatial map
  of open files and other resources" and a "single-pane-of-glass" project view [hn-cate].
- Horizon thread (84 points, 32 comments): "Spatial memory is really underutilised in computing";
  requests for strategic zoom and "more structure rather than less"; also "The last thing I want is
  the figma experience for my terminal" and "Just use tmux" [hn-horizon].
- Spine Swarm launch (109 points): "Why do I need a canvas to visualize the work that the agents are
  doing?"; the makers answer that the canvas serves tracing why an output went wrong, and add chat,
  task and deliverable panels [hn-spine].
- Most canvas-terminal Show HN posts drew little response (TermCanvas 3 points, AgentBase 3, Maestri
  2) [hn-canvas]. CanvasTTY had no Hacker News post [hn-canvas].

## 8. Comparison table

| Tool | Kind | Canvas objects | Status on tiles | Overview aids | Engine / stack | Licence | Stars, last push |
|---|---|---|---|---|---|---|---|
| CanvasTTY | desktop app | terminals, browser, notes, regions, plugin apps | hooks | semantic zoom, minimap, Home list, `Cmd+K` | custom, Electron, React, xterm | MIT | 82, 09-29 |
| DriftWM | Wayland compositor | any OS window | none | zoom-to-fit, bookmarks, Home | Rust, Smithay | GPL-3.0+ | 1744, 09-21 |
| Cate | desktop app | terminals, editors, browsers, docs, nested canvases | hooks | worktree territories, docks, `Cmd+K` | custom, Electron, xterm | MIT | 2163, 09-29 |
| OpenCove | desktop app + web UI | agents, terminals, tasks, notes, images | yes | search, control center, archives | React Flow, Electron | MIT | 1601, 09-25 |
| Horizon | desktop app | terminals, agents, browser, git, markdown | yes | workspaces, minimap, attention feed | Rust, egui, wgpu | MIT | 713, 09-29 |
| TermCanvas | desktop app | terminals, diffs, drawings | yes | hierarchy, Sessions panel | React Flow, Electron | MIT | 406, 05-31 |
| Maestri | desktop app | terminals, notes, sketches, portals | unconfirmed | bird's-eye zoom | Swift, Metal | proprietary | n/a |
| Nimbalyst | desktop app | none (kanban) | yes | kanban, dashboard | Electron | MIT | 1801, 09-28 |
| zoetrope | TUI + web | agent, subagents, tools of one session | n/a | flow graph | Rust, Ratatui, rataflow | MIT | 959, 09-15 |
| herdr-dagr | Herdr pane | tasks, attempts, gates | evidence tiers | DAG | Rust | Apache-2.0 | 88, 08-23 |
| herdr-canvas (aorumbayev) | Herdr pane | ASCII shapes | none | none | Go, Bubble Tea | MIT | 8, 08-31 |
| herdr-mission-control | Herdr popup | pane tiles | yes | exposé grid | Rust | not detected by GitHub | 3, 09-20 |
| JSON Canvas | file format | text, file, link, group, edges | none | n/a | JSON | MIT | 3707, 07-24 |

Sources: sections 1 to 6; stars and dates from the GitHub API on 2026-09-29 (dates are 2026 unless a
year is given).

## 9. Implications for the command center

Facts and options only; choosing among them is the owner's decision.

**Facts that constrain any option**

1. Herdr documents a public way to watch any pane live without taking ownership
   (`terminal session observe`, many observers) and a way to drive one (`terminal session control`,
   one controller) [herdr-remote]. Existing web bridges use private or endpoint protocols instead
   [herdrweb-readme], [roamgate-src].
2. Herdr plugins cannot draw non-terminal UI; a canvas inside Herdr must be a TUI or images placed
   through `pane.graphics` [herdr-plugins-doc], [herdr-socket].
3. No existing canvas embeds Herdr panes; the canvas apps all own their PTYs through node-pty or a
   native terminal and would be a second terminal runtime next to Herdr [ctty-pkg], [cate-pkg],
   [opencove-readme], [horizon-readme], section 6.3.
4. CanvasTTY shows status only for sessions it launched with its hooks; a Herdr pane attached inside
   a CanvasTTY terminal card would be a plain terminal to CanvasTTY. This follows from its docs
   [ctty-ui] and was not tested.
5. Research and products agree that a spatial view works as a mode next to a list, not instead of it
   [cockburn-2008], [debugger-canvas], section 7.2.
6. The repository's own rule is to read agents through public interfaces, not internal session files
   (`AGENTS.md`, Known pitfalls). zoetrope reads transcript files directly [zoetrope-readme].

**Options for combining a canvas overview with Herdr**

- **A. Read-only map in the web dashboard.** Nodes for projects, worktrees, sessions and issues from
  `session.snapshot` plus `events.subscribe` and GitHub; clicking a node calls `herdr agent focus`.
  Optional live previews per node through `terminal session observe`. Herdr stays the only terminal
  workspace. Engine choices: React Flow (MIT, used by OpenCove and TermCanvas) or custom code as in
  CanvasTTY and Cate.
- **B. Interactive panes on the canvas.** Same as A, but a node can be opened for input through
  `terminal session control --takeover`. Only one controller owns input and resize, so the canvas
  node and a Herdr client would contend for the pane; how Herdr resizes a pane owned by a controller
  is unconfirmed.
- **C. A terminal-native overview inside Herdr.** A plugin pane in Rust and Ratatui using rataflow
  for pan, zoom and minimap, or a tile grid like herdr-mission-control or herdr-paddock. It fits the
  planned Rust and Ratatui core; it has no free-form notes or pixel graphics beyond `pane.graphics`.
- **D. Notes and layout as a file.** Store the owner's notes, groups and node positions as a JSON
  Canvas `.canvas` file per project, readable in Obsidian and writable by agents through the
  `json-canvas` skill. Live session nodes would need a convention on top of `link` or `file` nodes or
  a custom node type, which the spec does not define.
- **E. Run CanvasTTY (or Cate, OpenCove) beside Herdr.** No build work, but two terminal runtimes,
  two status models and two places where sessions live; CanvasTTY's plugin SDK could add a Home
  widget that reads Herdr over loopback `network`, but plugin frames get only HTTPS and loopback
  fetch [ctty-plugins].
- **F. Borrow from CanvasTTY without adopting it.** The ideas the owner liked (sticky notes,
  regions that carry their contents, snapping, semantic summary below a zoom threshold, a fixed
  Home with the list and limits, "truth before visualization" for metrics) are small, separable
  features under MIT [ctty-tree], [ctty-metrics].

**Open questions for the owner**

- Is the canvas the primary cross-project view or a mode next to a list (section 7)?
- Does the canvas need live terminal content or only status, notes and links (A versus B)?
- Browser (A, B) or terminal (C) as the home of the overview?
- Should notes and layout be a portable file (D) or state inside the command center?

## References

- [ctty-repo]: https://github.com/howdeploy/CanvasTTY (API metadata, 2026-09-29)
- [ctty-readme]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/README.md
- [ctty-releases]: https://github.com/howdeploy/CanvasTTY/releases
- [ctty-pkg]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/package.json
- [ctty-contrib]: https://api.github.com/repos/howdeploy/CanvasTTY/contributors
- [ctty-tree]: https://github.com/howdeploy/CanvasTTY/tree/3fe6fed9fe46/src/renderer/src/features
- [ctty-start]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/docs/getting-started.md
- [ctty-ui]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/docs/UI_CONTRACT.md
- [ctty-arch]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/docs/ARCHITECTURE.md
- [ctty-adr-nav]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/docs/adr/ADR-20260808-intent-aware-canvas-navigation.md
- [ctty-metrics]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/docs/metrics-and-telemetry.md
- [ctty-plugins]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/docs/plugins.md
- [ctty-orch]: https://github.com/howdeploy/CanvasTTY/blob/3fe6fed9fe46/docs/agent-orchestration.md
- [ctty-issues]: https://github.com/howdeploy/CanvasTTY/issues/14, https://github.com/howdeploy/CanvasTTY/issues/53
- [driftwm-repo]: https://github.com/malbiruk/driftwm
- [driftwm-readme]: https://github.com/malbiruk/driftwm/blob/352333a8fa1b/README.md
- [driftwm-license]: https://github.com/malbiruk/driftwm/blob/352333a8fa1b/LICENSE
- [cate-repo]: https://github.com/0-AI-UG/cate
- [cate-readme]: https://github.com/0-AI-UG/cate/blob/f6c3d5511313/README.md
- [cate-pkg]: https://github.com/0-AI-UG/cate/blob/f6c3d5511313/package.json
- [opencove-repo]: https://github.com/DeadWaveWave/opencove
- [opencove-readme]: https://github.com/DeadWaveWave/opencove/blob/9126a04abf33/README.md
- [opencove-pkg]: https://github.com/DeadWaveWave/opencove/blob/9126a04abf33/package.json
- [horizon-repo]: https://github.com/peters/horizon
- [horizon-readme]: https://github.com/peters/horizon/blob/fd50d59afb5a/README.md
- [termcanvas-repo]: https://github.com/blueberrycongee/termcanvas
- [termcanvas-readme]: https://github.com/blueberrycongee/termcanvas/blob/fa2598155e44/README.md
- [termcanvas-pkg]: https://github.com/blueberrycongee/termcanvas/blob/fa2598155e44/package.json
- [maestri-site]: https://www.themaestri.app/en
- [voicetree-repo]: https://github.com/voicetreelab/voicetree
- [small-repos]: GitHub API metadata on 2026-09-29 for https://github.com/ceIia/finite,
  https://github.com/ZipLyne-Agency/CodeGrid, https://github.com/kylesnowschwartz/hgnucomb,
  https://github.com/DevoidSloth/ccanvas, https://github.com/lossless1/panescale,
  https://github.com/aleda145/terminaldraw, https://github.com/Quzr27/Korum,
  https://github.com/kdonev/termscape, https://github.com/twaldin/canvas,
  https://github.com/AgentOrchestrator/AgentBase, https://github.com/haystackeditor/haystack-editor,
  https://github.com/krakujs/hivemind, https://github.com/Mizora-net/ita,
  https://github.com/jzombie/term-wm, and `gh search repos` queries listed in section 6.3
- [jsoncanvas-repo]: https://github.com/obsidianmd/jsoncanvas
- [jsoncanvas-spec]: https://github.com/obsidianmd/jsoncanvas/blob/456f843cb293/spec/1.0.md
- [obsidian-skills]: https://github.com/kepano/obsidian-skills/blob/main/README.md
- [xyflow-repo]: https://github.com/xyflow/xyflow
- [tldraw-license]: https://github.com/tldraw/tldraw/blob/e5c0a9bc570e/LICENSE.md
- [rataflow-readme]: https://github.com/furkankly/rataflow/blob/aa3fff38d57e/README.md
- [nimbalyst-repo]: https://github.com/nimbalyst/nimbalyst
- [nimbalyst-readme]: https://github.com/nimbalyst/nimbalyst/blob/ac40c06e2195/README.md
- [zoetrope-repo]: https://github.com/furkankly/zoetrope
- [zoetrope-readme]: https://github.com/furkankly/zoetrope/blob/b1f31dd26bd4/README.md
- [dagr-repo]: https://github.com/aemrebarut/herdr-dagr
- [dagr-readme]: https://github.com/aemrebarut/herdr-dagr/blob/52991f9a95a2/README.md
- [herdr-socket]: https://github.com/herdrdev/herdr/blob/f0bb7827e4a1/docs/versions/0.9.1/website/src/content/docs/socket-api.mdx
- [herdr-remote]: https://github.com/herdrdev/herdr/blob/f0bb7827e4a1/docs/versions/0.9.1/website/src/content/docs/persistence-remote.mdx
- [herdr-plugins-doc]: https://github.com/herdrdev/herdr/blob/f0bb7827e4a1/docs/versions/0.9.1/website/src/content/docs/plugins.mdx
- [herdr-changelog]: https://github.com/herdrdev/herdr/blob/f0bb7827e4a1/CHANGELOG.md (0.7.2, 0.9.2)
- [herdr-plugins-meta]: https://assets.herdr.dev/plugins/index.json (1379 entries, 2026-09-29) and
  GitHub API metadata for the plugins named
- [herdrweb-readme]: https://github.com/kcosr/herdr-web/blob/f1312e2ffbe8/README.md
- [roamgate-repo]: https://github.com/powerfooI/roamgate
- [roamgate-readme]: https://github.com/powerfooI/roamgate/blob/b7b02ce128db/README.md
- [roamgate-src]: https://github.com/powerfooI/roamgate/blob/b7b02ce128db/server/src/bridge/endpoint-terminal-session.ts,
  https://github.com/powerfooI/roamgate/blob/b7b02ce128db/server/src/bridge/protocol-compat.ts
- [herdr-canvas-aor]: https://github.com/aorumbayev/herdr-canvas/blob/da902e1d4e77/README.md
- [herdr-canvas-sebi]: https://github.com/sebi75/herdr-canvas/blob/3910e5d25022/README.md
- [herdr-mc]: https://github.com/vjeantet/herdr-mission-control/blob/7bc8c09a8643/README.md
- [herdr-paddock]: https://github.com/neyham/herdr-paddock/blob/0310458e28e2/README.md
- [term-wm-readme]: https://github.com/jzombie/term-wm/blob/fa5f04706fc8/README.md
- [cockburn-2008]: Cockburn, Karlson, Bederson, "A Review of Overview+Detail, Zooming, and
  Focus+Context Interfaces", ACM Computing Surveys 41(1), 2008, section 8;
  https://faculty.cc.gatech.edu/~stasko/7450/Papers/cockburn-surveys08.pdf
- [code-bubbles]: Bragdon et al., "Code Bubbles: A Working Set-based Interface for Code
  Understanding and Maintenance", CHI 2010; https://cs.brown.edu/people/spr/codebubbles/CHI-final.pdf
- [debugger-canvas]: DeLine et al., "Debugger Canvas: Industrial Experience with the Code Bubbles
  Paradigm", ICSE 2012; https://www.microsoft.com/en-us/research/publication/debugger-canvas-industrial-experience-with-the-code-bubbles-paradigm/
- [hn-cate]: https://news.ycombinator.com/item?id=48265470 (secondary)
- [hn-horizon]: https://news.ycombinator.com/item?id=47416227 (secondary)
- [hn-spine]: https://news.ycombinator.com/item?id=47364116 (secondary)
- [hn-jsoncanvas]: https://news.ycombinator.com/item?id=39670922 (secondary)
- [hn-canvas]: Hacker News Algolia search on 2026-09-29 for "infinite canvas agents", "infinite
  canvas terminal", "CanvasTTY", "TermCanvas"; items 47571499, 46670183, 47729574 (secondary)
