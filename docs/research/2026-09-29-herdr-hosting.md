# What Herdr can host today

Date: 2026-09-29. Question from #42 (part of map #41, the command center): what can a Herdr plugin
host today, and which jobs of the command center do existing Herdr plugins already cover? This note
updates and deepens [command-centers-landscape §2b and §6a](2026-09-29-command-centers-landscape.md)
and [command-center-build-options §1.1](2026-09-29-command-center-build-options.md). It does not
repeat them. §6 lists the corrections to those notes.

Method.

- **Herdr itself.** Release notes of v0.9.0–v0.9.2 via `gh api repos/herdrdev/herdr/releases`.
  Docs as versioned in the repository: `docs/versions/0.9.2/website/src/content/docs/*.mdx` and
  `docs/next/...` on `master` at `4b4705d`. For `plugins`, `socket-api` and `marketplace` the two
  copies are byte-identical. `CHANGELOG.md` and `CONTRIBUTING.md` on `master`. The API schema
  `docs/next/api/herdr-api.schema.json` on `master` was compared with `herdr api schema --json` from
  the locally installed `herdr 0.9.1`. Plugin host source: `src/app/api/plugins/mod.rs` on
  `master`. Product direction: herdr.dev and its blog. Open requests: GitHub Discussions and
  issues, via `gh api graphql` and `gh search issues`.
- **Plugins.** The marketplace index `https://assets.herdr.dev/plugins/index.json`
  (`generatedAt` 2026-09-29T17:00Z). Stars, license, push date, open issues and contributors come
  from `gh api repos/<repo>` and `repos/<repo>/contributors` on 2026-09-29. Bus factor is given as
  the top human contributor's share of commits. Twelve repositories were downloaded at a pinned
  commit, and their manifests, READMEs and sources were read (listed in §3).
- **Local machine.** Only read-only commands: `herdr --version`, `herdr api schema --json`,
  `herdr plugin list`. No input was sent to agents, and no config or plugin was changed.

Citation shorthand: `H:<path>` means `https://github.com/herdrdev/herdr/blob/4b4705d/<path>`, and
`D:<page>` means `H:docs/versions/0.9.2/website/src/content/docs/<page>.mdx`. For a plugin,
`<owner>/<repo>@<sha>:<path>` means `https://github.com/<owner>/<repo>/blob/<sha>/<path>`.

---

## 1. Herdr's host surface in 0.9.2

### 1.1 Versions

- The latest stable release is **v0.9.2**, published 2026-09-29. v0.9.1 came out on 2026-09-16 and
  v0.9.0 on 2026-09-07. Preview builds ship between them
  ([releases](https://github.com/herdrdev/herdr/releases)). The owner runs 0.9.1.
- Plugin v1 arrived in **0.7.0 (2026-06-15)**: link/list/enable, manifest actions, event hooks,
  managed panes, link handlers, command logs, keybindings, and `plugin install` from GitHub
  (H:CHANGELOG.md, section 0.7.0). Plugin-relevant additions since then:
  - 0.7.2: `terminal session observe|control`
  - 0.7.4: popup panes
  - 0.7.5: `agent.view.set`, `[[startup]]` hooks, and plugins made global per user
  - 0.8.2: marketplace indexing of subdirectories
  - 0.9.0: OSC 8 `file://` links reach link handlers

  Source: the same changelog, by version.
- The **API contract is unchanged** between 0.9.1 and `master`: protocol 22, `schema_version` 1.
  The request methods number 103 in 0.9.1 and 102 on `master`. 0.9.2 added `pane.clear` and
  `server.ssh_agent.register` and removed `pane.graphics.info|set|clear` (local schema compared
  with H:docs/next/api/herdr-api.schema.json).

### 1.2 What a plugin is and what it may declare

A plugin is a directory with `herdr-plugin.toml` plus any argv commands. "There is no separate
plugin SDK or restricted command set. The entire Herdr CLI is the plugin API" (D:plugins).

The manifest may declare `[[build]]`, `[[startup]]`, `[[actions]]`, `[[events]]`, `[[panes]]` and
`[[link_handlers]]`, each with optional per-item `platforms`. The required fields are `id`, `name`,
`version` and `min_herdr_version` (D:plugins, "Manifest"). "Runtime action registration and native
non-terminal plugin UI are not part of plugin v1" (D:plugins). The socket API doc calls it "an early
interface for executable workflow tools" (D:socket-api, "Plugin APIs").

| Capability | What 0.9.2 provides | Source |
|---|---|---|
| **Panes** | A manifest `[[panes]]` entry runs an argv command as a terminal. Placement is `overlay` (default; temporary zoomed over the active pane), `popup` (session-modal, sized in cells or `%`, no pane id, invisible to `pane.*`/agent APIs, no lifecycle events), `split`, `tab` or `zoomed`. Split, tab, zoomed and overlay panes are ordinary panes afterwards (move, swap, resize), and plugin ownership follows them. `plugin.pane.open` accepts `placement`, `target_pane_id`, `workspace_id`, `cwd`, `env`, `width`, `height` and `focus`. Plugin panes do not restart after a cold server restart (discussion #1367). | D:plugins "Panes"; D:socket-api "Plugin APIs"; schema `PluginPaneOpenParams`; [discussion #1367](https://github.com/herdrdev/herdr/discussions/1367) |
| **Agents sidebar: view** | `agent.view.set` installs **one** transient declarative filter and sort over the built-in Agents view: sidebar, collapsed and mobile list, mouse targets and next/previous navigation. Filters cover `status`, `workspace_id`, `tab_id`, `pane_id`, `agent`, `seen`, `state_change_seq` and `{"token":name}`. A new set atomically replaces the previous view. The view dies with the server or with its owning plugin, so plugins reapply it from `[[startup]]`. It does not change `agent.list`, notifications or attention counts. | D:socket-api "Agent view queries" |
| **Agents sidebar: content** | `pane.report_metadata` and `workspace.report_metadata` set a title, a display agent name, per-state labels and named **tokens**: at most 16 keys per report and 32 per pane or workspace, values capped at 80 characters, optional `ttl_ms`, and a `seq` for ordering. Tokens are display-only and not restored after a restart. They render only where the user's **local config** places `$name` in `[ui.sidebar.agents\|spaces].rows`, which allows at most 16 rows of 16 tokens with fixed styling rules. "Metadata reporters provide values only; styling stays in the local sidebar configuration." | D:socket-api "Agent state reporting"; D:configuration "Sidebar row layouts" |
| **Events** | `[[events]] on = "<event>"` runs a command with `HERDR_PLUGIN_EVENT_JSON`. Subscribable events: `workspace.*`, `tab.*`, `pane.created\|updated\|closed\|focused\|moved\|exited\|agent_detected\|output_matched\|agent_status_changed\|scroll_changed`, `layout.updated`, `worktree.created\|opened\|removed`. `workspace.metadata_updated` reaches socket subscribers but does not invoke plugin hooks. Since 0.9.2 a slow subscriber gets `events_lost` rather than silent gaps. | D:socket-api "Event subscriptions"; v0.9.2 release notes |
| **Link handlers** | Ctrl-click on a terminal URL that matches a Rust regex runs one of the plugin's own actions with `clicked_url`. Since 0.9.0 OSC 8 `file://` links match too. Bare file paths are not clickable (open request, discussion #4465). | D:plugins "Link handlers"; H:CHANGELOG.md 0.9.0; [discussion #4465](https://github.com/herdrdev/herdr/discussions/4465) |
| **Actions and keys** | Actions are manifest-only. They are reached by a `[[keys.command]] type = "plugin_action"` binding, `herdr plugin action invoke`, or any socket client. Requests to list plugin actions in context menus or a command palette were closed as not planned, as were other feature requests filed as issues (see "Roadmap" below). | D:plugins "Keybindings"; issues [#1671](https://github.com/herdrdev/herdr/issues/1671), [#1830](https://github.com/herdrdev/herdr/issues/1830) |
| **Notifications** | `notification.show`: title up to 80 characters, body up to 240, optional position and sound. Delivery is Herdr toast, terminal, system or off. The result can be `rate_limited`, `no_foreground_client` or `busy`. | D:socket-api, notification section |
| **State directory** | `HERDR_PLUGIN_STATE_DIR` and `HERDR_PLUGIN_CONFIG_DIR` are created per plugin. "There is no Herdr-managed plugin storage API in v1." The plugin owns formats, migrations and cleanup. | D:plugins "Commands and environment", "Storage" |
| **Images inside panes** | Any pane program can draw pixels through the standard **Kitty graphics protocol**. Herdr handles placement and clipping, crops under popups (0.9.2), and uses a file transport on local Ghostty. The Herdr-specific `pane.graphics.*` socket API was removed in 0.9.2 with "no replacement socket image API". Rendering needs a Kitty-graphics-capable outer terminal. | D:configuration "Kitty graphics"; v0.9.2 "Breaking Changes" |
| **Live pane streams** | `herdr terminal session observe <pane>` is read-only and prints NDJSON `terminal.frame` records with base64 ANSI. Many observers are allowed. `terminal session control` is writable: input, resize, scroll, mouse (0.9.2) and release, with one controller at a time and `--takeover`. | D:cli-reference "Direct terminal attach"; D:persistence-remote |

**Is non-terminal UI possible?** Not natively. Plugin UI is a terminal pane, sidebar text tokens, a
view query or a toast (D:plugins). Two routes get around it:

- **Pixels in a pane.** Kitty graphics can draw arbitrary pixels inside a pane. `zenbu-labs/terminal-browser`
  (3.5k★, MIT) uses this to run Chromium in a pane through Electron offscreen rendering
  (`zenbu-labs/terminal-browser@a12cb91:README.md`).
- **A separate client over the socket API.** See §2.

**Roadmap.**

- **Plugin UI.** Herdr publishes no roadmap for plugin UI. Its issue tracker accepts only
  reproducible bugs, and feature requests go to Discussions. Proposals filed as issues were closed
  as not planned for that reason, for example the plugin-owned sidebar sections proposal
  ([#1608](https://github.com/herdrdev/herdr/issues/1608)). Herdr also does not accept unsolicited
  pull requests (H:CONTRIBUTING.md).
- **Open requests in Discussions, no maintainer answer seen:**
  - plugin-owned sidebar areas ([#1675](https://github.com/herdrdev/herdr/discussions/1675),
    [#4445](https://github.com/herdrdev/herdr/discussions/4445),
    [#4466](https://github.com/herdrdev/herdr/discussions/4466))
  - several plugins contributing rows
    ([#4716](https://github.com/herdrdev/herdr/discussions/4716))
  - `plugin update` ([#3219](https://github.com/herdrdev/herdr/discussions/3219))
  - pane restore across restarts ([#1367](https://github.com/herdrdev/herdr/discussions/1367))
  - action parameters ([#3604](https://github.com/herdrdev/herdr/discussions/3604))
- **Direction the author has stated:**
  - "the TUI was never meant to be the only client … Herdr will need more clients. More on that
    later" ([YC post, 2026-08-06](https://herdr.dev/blog/herdr-is-joining-y-combinator/))
  - "Herdr Cloud" as the next piece before 1.0: a relay that links your own machines, with
    end-to-end encrypted terminal traffic
    ([Connecting the machines, 2026-09-07](https://herdr.dev/blog/connecting-the-machines/))
  - a $6M seed round, to make Herdr "more extensible"
    ([2026-09-08](https://herdr.dev/blog/herdr-raised-a-seed/))

  Whether "more clients" means a first-party web or GUI client is unconfirmed.

## 2. Web layers and embedding live panes

Herdr has **no first-party web client**. Neither herdr.dev nor the docs index mentions one
([herdr.dev](https://herdr.dev/), H:README.md). The "mobile" wording in the docs refers to the TUI's
single-column layout for narrow terminals (`ui.mobile_width_threshold`, H:CHANGELOG.md). The third-party layers:

| Layer | ★ / push / license / top author | What it shows | How it talks to Herdr |
|---|---|---|---|
| powerfooI/roamgate | 257 / 09-29 / MIT / 79% (13 contributors) | Browser client for desktop and mobile: live terminals, agent sessions, file explorer, diffs with annotations, image previews, a list of plugin actions | Speaks Herdr's **client endpoint protocol** on `herdr-client.sock` ("stable endpoint generation 1": bincode envelope, frozen surface codecs), plus the JSON socket for `agent.list`, `events.subscribe`, `plugin.action.invoke` (`roamgate@b7b02ce:server/src/bridge/endpoint-client.ts`, `web/src/store.ts`) |
| devswha/herdr-web-ui | 53 / 09-29 / MIT / 86% | Browser and PWA: chat view built from agents' session files, live terminal per pane, remote PCs over SSH, web push | JSON socket API for status, and the public `herdr terminal attach` through a node-pty sidecar with one shared attach per pane (`herdr-web-ui@767b747:docs/guide.md` "How it works") |
| AltanS/collie | 1,132 / 09-29 / MIT / 88% | Mobile PWA: status dashboard led by "needs your input", push, quick actions. Also supports tmux and zellij | JSON socket (`session.snapshot`, `pane.read`, `pane.send_*`). It polls, and events only speed polling up (`collie@d70dc0d:README.md` "Architecture") |
| 0cv/herdr-mobile-relay | 263 / 09-29 / NOASSERTION / 91% | Phone web app: approve and monitor, push | JSON socket (`agent.list`, `pane.read`, `events.subscribe`) (source grep at `1560026`) |
| kcosr/herdr-web | 144 / 09-22 / MIT / 69% | Browser UI for workspaces and agent panes | **Vendors private Herdr code** for terminal attach, snapshots and events. Its README says the "runtime/API shape is expected to change" (`kcosr/herdr-web@f1312e2:README.md`). It is not in the marketplace index |
| ZingerLittleBee/Heeler | 434 / 09-29 / Apache-2.0 / 98% | Native iOS console over SSH | Not read |

**Can a web or canvas surface embed live Herdr panes?** Yes, over three routes of different
stability:

1. **The documented public route.** `terminal session observe` and `control` exist "for
   third-party bridges" (D:persistence-remote). A bridge relays the NDJSON base64 ANSI frames to
   xterm.js or ghostty-web in the browser. A user in [discussion #3643](https://github.com/herdrdev/herdr/discussions/3643)
   confirmed this works for exactly that purpose, following herdr-mirror.
2. **`herdr terminal attach` behind a PTY.** This is what herdr-web-ui does.
3. **The client endpoint protocol.** Roamgate reimplements it. Herdr documents it only as the
   contract between its own TUI client and server: "Client and server builds do not need to match"
   (D:socket-api "Protocol stability"). A third party asked whether endpoint generation 1 is a
   supported contract for custom clients
   ([#3913](https://github.com/herdrdev/herdr/discussions/3913), 2026-09-10). No maintainer answer
   is visible, so this is **unconfirmed**.

Embedding in the other direction also works: a web page, such as our own dashboard, can be shown
**inside** a Herdr pane with terminal-browser over Kitty graphics. How well this works depends on
the outer terminal (see §1.2).

## 3. Coverage of the command center's jobs by existing plugins

The marketplace is an automatic, unreviewed index. It admits public GitHub repositories with the
topic `herdr-plugin` and at least one parseable manifest, and refreshes every 30 minutes
(D:marketplace). On 2026-09-29 it held **1,419 manifests in 1,379 repositories** (index.json).

- **Age.** 1,365 of the 1,379 repositories were created in June–September 2026.
- **Stars.** 1,123 have fewer than 5 stars, 88 have at least 20, and 33 have at least 100.
- **Activity.** 803 were pushed in the last 30 days.

Figures in the tables are from `gh api` on 2026-09-29. "Top" is the top human contributor's share of
commits, and "no license" means GitHub detects no license file.

### 3.1 Check one result

| Job | Plugin | ★ / created / push / license / top / release | What it does | Fit |
|---|---|---|---|---|
| Diff | **persiyanov/herdr-reviewr** | 792 / 06-26 / 09-23 / MIT / 96% of 10 / v0.39.0 | One split pane per worktree with four scopes: uncommitted, branch, **last turn**, commits. Line comments are sent to the agent. File viewer, fuzzy search, **read-only PR tab** (gh, glab, az), markdown preview. Auto-opens on `worktree.created\|opened`. Limits: macOS and Linux only; "last turn" polls every 2 s and can miss short turns (`herdr-reviewr@cd618b4:README.md`, `herdr-plugin.toml`) | The most complete single-result reviewer |
| Diff | jhochenbaum/herdr-hunk-diff | 134 / 08-12 / 09-27 / MIT / 83% of 3 / v0.4.0 | Hunk review panes for uncommitted, staged, branch and commit. Inline comments go back to the "responsible agent". Link handler on GitHub commit URLs. Listens to `pane.agent_status_changed` (`herdr-hunk-diff@47146a0:herdr-plugin.toml`) | Alternative to reviewr, depends on the `hunk` tool |
| Diff and files | alexarthurs/herdr-sidebar; smarzban/herdr-file-viewer | 405 / 07-17 / 09-27 / MIT / 97%; 612 / 06-18 / 09-16 / MIT / 95% | Explorer and git source control; read-only git-aware file viewer | Browsing files, not a review workflow |
| Spec, plan, reply | plannotator/herdr-annotate | 582 / 08-09 / 09-27 / MIT / 86% of 5 / no release (manifest 0.7.0) | Annotate any terminal text, markdown documents and agent replies, then send them to the agent. Link handler on `file://…\.md` (`herdr-annotate@663b45a:herdr-plugin.toml`) | Review before code |
| PR | reviewr PR tab (above); wyattjoh/herdr-plugin-gh-pr; Matovidlo/herdr-pr-tracker; jsmenzies/mergr; osolmaz/ghzinga; tomasvarga/herdr-pickr | 22 / – / 07-16 / no license; 12 / – / 08-12 / MIT; 5 / – / 07-30 / MIT; 87 / – / 09-06 / MIT; 20 / – / 07-13 / MIT | PR status of the focused branch as a sidebar token; session→PR tracking; PR status in Space rows; a PR/issue TUI; Ctrl-click routing of PR links to a reviewer | All small, single-author, and most untouched for 1–2 months |
| Files at lines | – | – | No plugin was found that opens `path:line` from agent output. Link handlers match only URLs and `file://` OSC 8 links, and bare paths are not clickable (#4465) | **Gap** |
| Preview (web, markdown) | zenbu-labs/terminal-browser; StructuPath/herdr-browser | 3,511 / 07-06 / 09-29 / MIT / 98% of 6 / v0.11.1; 22 / 07-18 / 09-16 / MIT | Real Chromium in a split pane over Kitty graphics. Installed by a `curl` script piped into `bash` in `[[build]]` (`terminal-browser@a12cb91:herdr-plugin/herdr-plugin.toml`); a drivable agent-browser pane | Local previews of a running app |
| Screenshots and images | Herdr core (Kitty graphics); thuanlm215/advanced-herdr-file-viewer; roamgate (web) | –; 6★; 257★ | Any pane program can draw images. One small file viewer previews images inline. Roamgate shows image changes in the browser | No mature terminal plugin for "screenshots of this result" |

### 3.2 Overview across projects

| Job | Plugin | ★ / created / push / license / top / release | What it does | Fit |
|---|---|---|---|---|
| Agents, "what waits for me" | Herdr core | – | Built-in Agents sidebar with working, blocked, idle and done ("idle and not yet seen"). Combined across saved SSH machines since 0.9.0 (D:socket-api; v0.9.0 notes) | Baseline |
| Agents, "what waits for me" | **hhdebb/herdr-radar** | 105 / 09-07 / 09-28 / MIT / 89% of 9 / v1.3.20 | Sticky "done until seen" and "question until answered", three idle tiers, grouping by project with worktrees nested under their repository. Uses `agent.view.set` and **writes managed blocks into the user's Herdr `config.toml`** (sidebar rows, tab bar, theme) (`herdr-radar@2425fa0:lib/view.js`, `lib/managed-config.js`) | Ready-made "who waits on me" |
| Agents, attention | aigorahub/herdr-lantern; douglascorrea/herdr-agent-inbox; natori-hrj/herdr-triage | 67 / 08-18 / 09-29 / MIT / 75% of 4 / v0.15.0; 10 / 07-23 / 07-28; 6 / 07-19 / 07-23 | "Who needs you"; inbox with settle and mark-unread; attention ranking | Smaller alternatives |
| Tasks, relations | **eliasstravik/herdr-projects** | 517 / **09-18** / 09-28 / MIT / **100% of 1** / v0.2.34 | A coordinator conversation plus worker threads, each thread in its own worktree and branch. Sidebar lines like `review · PR #4` and `~40%` come from progress the agents report themselves. A PR ticker, "needs you" notifications, grouping by project. Uses `agent.view.set`. Keeps a project as a folder of Markdown and TOML (`herdr-projects@4e4548c:README.md`, `src/overview.rs`) | Closest to "tasks ↔ agents ↔ branches ↔ PRs", but 11 days old and has one author |
| Tasks (board) | nelsonPires5/herdr-board; smarzban/tsk; bredebjorhovd/herdr-board; miiraheart/herdr-beads; deimantasnork/captains-deck | 162 / 07-14 / 09-29 / **no license** / 99% / v0.18.0; 153 / 08-19 / 09-29 / MIT / 97%; 1 / 07-26 / 08-14 / **no license** / 100%; 34 / 07-23 / 08-25; 29 / 09-24 / 09-28 / MIT / 100% | A kanban of prompts dispatched into panes; a terminal task board shared by you and your agents; GitHub and Linear issues dispatched to panes with a review loop; a beads task board; a read-only flow kanban | Each owns its own task store; none reads GitHub issues as the source of truth, except the 1★ bredebjorhovd board |
| Relations (graph) | aemrebarut/herdr-dagr | 88 / 08-14 / 08-23 / Apache-2.0 / 100% / v0.3.1 | A swarm as a live DAG with attempts, review gates and evidence | An idea; inactive for 5 weeks |
| Navigation | thanhdat77/herdr-navigator; andrewchng/herdr-sessionizer | 175 / 06-28 / 09-24 / MIT / 94% / v0.3.6; 49 / 06-16 / 09-26 / MIT / 89% / v0.9.0 | Fuzzy jump to a workspace, agent, project or action; open a project with a TOML layout | Quick-jump and launch |

**What is not covered:**

- No plugin joins GitHub issues and specs, agents and PRs into one cross-project view that also
  holds our own stage artifacts.
- No plugin opens a file at a line.
- Every task-oriented plugin keeps its own private task model.

## 4. Limits that matter

- **API stability.**
  - Herdr is pre-1.0 and calls plugin v1 "an early interface" (D:socket-api).
  - The JSON API promises forward tolerance: "JSON API clients should ignore unknown fields and
    handle unsupported methods as normal errors" (D:socket-api "Protocol stability").
  - A **patch release can still remove public methods**: 0.9.2 removed `pane.graphics.*` with no
    replacement (v0.9.2 "Breaking Changes").
  - 0.7.5 made plugins global per user. Anyone who installed a plugin only in a named session on
    0.7.3 had to reinstall it (H:CHANGELOG.md 0.7.5).
  - Manifests declare `min_herdr_version`, and Herdr refuses newer ones (D:plugins). Of the
    indexed manifests, 423 declare 0.7.0 as the minimum and 57 declare 0.9.1 (index.json).
- **Versioning and updates.**
  - There is no `plugin update` in v1: you reinstall. `--ref` pins a revision (D:plugins).
  - `plugin.list` records the requested ref and the resolved commit (D:socket-api).
  - Plugins ship their own updaters. herdr-projects has `herdr-projects update`
    (`herdr-projects@4e4548c:README.md`).
- **Sandboxing: none.**
  - Plugins "run as your user, inherit your environment, and can call the full Herdr CLI". Herdr
    "does not review or sandbox plugin code" (D:plugins "Trust and security").
  - Build steps may run `curl … | bash`, for example terminal-browser.
  - The marketplace listing is "not a reviewed catalog" (D:marketplace).
- **Cross-plugin composition.**
  - Any socket client, including another plugin, can call `plugin.action.invoke` on any installed,
    enabled action and `plugin.pane.open` on any plugin's pane. The handler checks only that the
    plugin is enabled and the platform matches. It does not check the caller
    (H:src/app/api/plugins/mod.rs, `handle_plugin_action_invoke`).
  - Roamgate uses this to list and fire every plugin's actions from the browser
    (`roamgate@b7b02ce:web/src/store.ts`).
  - The contract is thin:
    - Actions take only a fixed `PluginInvocationContext` (workspace, tab, pane, cwd, worktree,
      `selected_text`, `clicked_url`, `correlation_id`), not free arguments (schema; discussion #3604).
    - Panes do accept `env` and `cwd`.
    - The caller gets back only a command log record (status, exit code, capped stdout and stderr
      through `plugin.log.list`), not a typed result (schema `PluginCommandLogInfo`).
- **Shared surfaces collide.**
  - Only one `agent.view.set` is active at a time, and the latest set wins (D:socket-api). Radar
    and herdr-projects both set it (sources above).
  - Sidebar layout is a single user-owned config section. Plugins can supply only `$token` values,
    or edit `config.toml` themselves as radar does.
  - Running radar and herdr-projects together has not been tested, so their actual interaction is
    unconfirmed.
- **Multi-project scope.**
  - One Herdr server holds any number of workspaces, one per project or worktree.
  - Plugins are global to the user across sessions (D:plugins).
  - Named sessions have separate sockets (D:socket-api "Socket paths").
  - Saved SSH machines appear in one combined agent list, and `agent.view.set` filters apply across
    machines (D:socket-api; D:connecting-machines).
  - Herdr has no notion of a "project" beyond workspaces and worktree groups. Plugins that group by
    project (radar, herdr-projects) derive it themselves.
- **Where it runs.** Plugin panes and link handlers exist only inside the TUI. A web or phone client
  sees plugin panes only as terminals, through observe or control.

## 5. Implications for the command center

These are facts and options. The decisions are the owner's.

- **Terminal-native center.** A Herdr plugin can host the center as terminal panes, popups,
  sidebar tokens and one agent view, with actions, events, link handlers and a state directory. It
  cannot own a persistent sidebar region or draw native widgets. It shares the agent view and the
  sidebar layout with any other plugin that uses them.
- **Pixels in the terminal.** Richer visuals inside Herdr need Kitty graphics, either our own or
  terminal-browser showing a local web page. Both depend on the outer terminal.
- **Web center beside Herdr.** A web center has a documented route to live panes (`terminal
  session observe|control`) and to state (`session.snapshot` plus `events.subscribe` with
  `events_lost` recovery). It avoids the undocumented client endpoint protocol that roamgate uses,
  and the vendored private code that kcosr/herdr-web uses.
- **Reuse and dependencies.** Check-one-result is well served: reviewr for diffs, last turn and
  read-only PRs, annotate for specs and plans, terminal-browser for previews. The overview has a
  ready "who waits on me" in radar and an emerging tasks↔threads↔PRs model in herdr-projects. All
  of them are MIT (dagr is Apache-2.0), young, and effectively single-author (top share 83–100%).
  Two options:
  - depend on them at a pinned `--ref`, or
  - invoke them through `plugin.action.invoke` and `plugin.pane.open`, which works but returns no
    typed results.
- **Gaps no plugin fills:**
  - a view of GitHub issues/specs ↔ agents ↔ PRs across projects over our own artifact contract
  - opening a file at a line
  - screenshots as part of a result
- **Upstream.** Herdr accepts neither unsolicited pull requests nor feature issues. Any host
  capability we need, such as plugin sidebar sections or action parameters, can only be requested
  in Discussions, with no committed timeline.

## 6. Corrections to earlier notes

- landscape §2b says "plugins, web client". Herdr has **no first-party web client**. The web
  clients are third-party (§2).
- landscape §6a says "all below are MIT unless noted". **nelsonPires5/herdr-board,
  bredebjorhovd/herdr-board and wyattjoh/herdr-plugin-gh-pr have no detected license**, and
  herdr-mobile-relay is NOASSERTION (§3).
- landscape §6a counts 1,375 plugins. The index now lists 1,419 manifests in 1,379 repositories
  (§3). kcosr/herdr-web is not in the index.
- build-options §1.1 quotes 129 methods and events. The request methods alone number 103 in 0.9.1
  and 102 on `master`, and `pane.graphics.*` is gone in 0.9.2 (§1.1).
- build-options §1.1 lists the plugin env. `HERDR_PLUGIN_CONFIG_DIR`, `HERDR_PLUGIN_CONTEXT_JSON`,
  `[[startup]]` hooks and popup placement also exist (§1.2).
