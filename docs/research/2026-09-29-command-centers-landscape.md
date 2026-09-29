# Research 5: a "command center" for parallel coding agents (state as of 2026-09-29)

Method: GitHub metadata via `gh api` (stars, last push, license, archived status) as of 2026-09-29;
READMEs and official product pages; for commercial products — official blogs/docs where
available; secondary sources are marked. "unconfirmed" = could not be verified against a primary source.

## 1. Paperclip (paperclipai/paperclip)

Source: https://github.com/paperclipai/paperclip (README), https://docs.paperclip.ing

- **What it is.** "Open-source orchestration for teams of AI agents… If OpenClaw is an employee,
  Paperclip is the company." Node.js server + React UI. Positioning — "Manage business goals,
  not pull requests": company goal → org chart of agents (CEO, CTO, engineers…) → budgets → heartbeat.
- **Data model.** Company (multi-org, full isolation) → Goals → Projects → Issues (links to
  company/project/goal/parent, blocker dependencies, comments, documents, attachments, work products,
  labels, inbox state) ; Agents (role, title, reporting line, permissions, budget); Heartbeat runs
  (DB wakeup queue, coalescing, budget check, workspace resolution, secret injection, skill loading,
  structured logs, cost events, session state); Approvals/execution policies; Routines (cron/webhook/API
  → each trigger creates an issue); Activity log (immutable audit); Plugins (out-of-process
  workers, UI contributions); Secrets; Company export/import.
- **How it launches agents.** Adapters: Claude Code, Codex, Cursor/Gemini/bash CLI, OpenCode,
  HTTP/webhook (OpenClaw), external adapter plugins. Agents do not "live" in a terminal — they wake up
  on a heartbeat/event (task assignment, @-mention), do an atomic checkout of the task and work.
  Workspaces: project workspace, isolated execution workspaces (git worktrees, operator branches),
  runtime services (dev servers, preview URLs). Sandboxes: e2b, Cloudflare, Daytona, Modal, Novita,
  self-hosted Kubernetes (roadmap: ✅).
- **UI.** Dashboard, task manager (issues/inbox), org chart, costs/budgets, approvals, activity,
  skills studio, evals; mobile-ready UI.
- **Artifacts/review.** "Artifacts & Work Products" ✅, "Deep Planning (revisioned plans, plan
  approvals)" ✅, review gates, verify from diffs/screenshots/tests. But explicitly: "Not a code review
  tool. Paperclip orchestrates work, not pull requests. Bring your own review process."
- **Tracker.** Its own ticket system. "Bring-your-own-ticket-system (Asana / Linear / Jira as
  on-ramps)" — ⚪ roadmap, i.e. **there is no sync with GitHub Issues/Projects** (per the README).
- **Budgets.** Token/cost tracking by company/agent/project/goal/issue/provider/model; hard stop,
  agent pause and queue cancellation on overspend.
- **Stack/hosting.** TypeScript (main), some Rust; embedded PostgreSQL locally or your own
  Postgres; Node.js 24.11+, pnpm 9.15+; `npx paperclipai onboard`; modes local_trusted / lan / tailnet.
- **License/activity.** MIT; created 2026-03-02; ~93k stars; push 2026-09-29; release v2026.916.1
  (2026-09-21); ~6000 open issues+PRs — very high noise/rate of change.
- **Maturity/risks.** Critical vulnerabilities, published 2026-08-05: CVE-2026-41679 (CVSS 10,
  RCE via agent import on networked installations), DNS rebinding in local_trusted (9.6), missing
  auth on part of the API (8.3); fixed in v2026.416.0
  (https://thehackernews.com/2026/08/paperclip-ai-flaws-let-attackers-run.html). Open PRs about
  redaction of secrets in run logs, a "40+ hour outage" due to UUID validation of checkout (issue #4060).
- **Why it is heavy for a solo developer with a GitHub pipeline.**
  1. Its own work model (company/goals/org chart/heartbeat), not your pipeline idea→spec→tickets→
     implementation→review→acceptance; a second tracker alongside GitHub Issues/Projects.
  2. Agents are autonomous "employees" on a schedule; yours are interactive sessions in Herdr under
     the owner's control. A heartbeat = an LLM call, spend is hard to predict (eesel review:
     https://www.eesel.ai/blog/paperclip-ai-review — secondary).
  3. Not a code review tool — PR review still happens in GitHub.
  4. Operations: a server + Postgres + security patches for a fast-changing project.
  5. The README itself: "If you have one agent, you probably don't need Paperclip".

## 2. Tool landscape

### 2a. Local desktop/TUI "workspace managers" (worktree per task, BYO CLI agent)

| Tool | Status (GitHub as of 2026-09-29) | Essence |
|---|---|---|
| **Conductor** (conductor.build) | commercial, closed; YC S24, Series A $22M (secondary) | Mac app; Claude Code, Codex, Cursor, OpenCode in isolated workspaces (branch + worktree + terminals + diff), review diff → PR → merge → archive. Free locally, your own subscriptions. The home page now says "Run a team of coding agents in the cloud" — cloud workspaces (details unconfirmed). No Win/Linux. https://www.conductor.build/docs/ |
| **Agent Orchestrator (AO)** Untrivial-ai/agent-orchestrator (formerly ComposioHQ) | Apache-2.0, 12.5k★, push 09-29, v0.13.1 (09-26); anonymous telemetry | Desktop+daemon+CLI; a worker session per task (branch/worktree), a **project orchestrator agent** plans and hands out tasks; a **Kanban where the card position is derived from session/PR/CI/review facts**; opening a worker: terminal, changed files, PR summary, reviews, preview, send CI/review feedback to the same agent. Tracker adapters in code: GitHub, GitLab. 25+ harnesses. https://github.com/Untrivial-ai/agent-orchestrator |
| **Emdash** generalaction/emdash (YC W26) | Apache-2.0, 5.9k★, push 09-29 | A worktree per agent, local and over SSH; **tasks from Linear, GitHub, Jira, GitLab, Asana…**; diff, PR, CI checks, merge; installs lifecycle hooks into agents for status/notifications/resume. https://github.com/generalaction/emdash |
| **Superset** superset-sh/superset | **Elastic License 2.0** (source-available), 14.7k★, push 09-29 | "agentic IDE": worktree+terminals, diff viewer, browser preview with ports per worktree, PR review → sending lines to the agent, "Pages" for reports with comments. https://github.com/superset-sh/superset |
| **Nimbalyst** (ex Crystal) nimbalyst/nimbalyst | MIT, 1.8k★, push 09-28. Crystal (stravu/crystal, 3.1k★) deprecated 2026-02 | Visual workspace: sessions in worktrees, **session kanban**, task tracking in markdown files in the repo, WYSIWYG red/green diff for documents/mockups/diagrams, iOS companion with push. https://github.com/nimbalyst/nimbalyst |
| **Sculptor** (Imbue) imbue-ai/sculptor | MIT, 233★, push 09-28, "experimental research preview" | Workspaces (worktree), chat, Changes, PR tracking. |
| **Xum** (coder/xum, ex coder/mux) | AGPL-3.0, 2k★, push 09-29 | Its own agent loop; worktree/remote runtimes; code review, agent status sidebar, costs, mermaid plans. |
| **Vibe Kanban** BloopAI/vibe-kanban | Apache-2.0, 28k★; **the Bloop company shut down 2026-04-10**, cloud (issues, projects, orgs) removed; last release 2026-04-24; push 09-19 (community) | Kanban issues → workspaces, inline comments on the diff → agent, preview, 10+ agents, PR. https://www.vibekanban.com/blog/shutdown |
| **Claude Squad** smtg-ai/claude-squad | AGPL-3.0, 8.5k★, push 2026-08-20 | TUI on top of tmux+worktree; Claude Code, Codex, OpenCode, Amp. |
| **parallel-code** johannesjo | MIT, 1k★ | Claude/Codex/Gemini side by side, a worktree for each. |
| **Claude Code Agent Farm** | 919★, push 09-21 | A script framework for 20+ Claude Code instances in tmux, lock coordination. For sweep tasks, not for a pipeline. |

### 2b. Terminal environments (sessions as a primitive) — closest to the owner's current stack

- **Herdr** herdrdev/herdr — Apache-2.0, 41k★, push 09-29; the owner has `herdr 0.9.1` installed.
  Persistent background server, agent status working/blocked/idle, multi-machine over SSH,
  socket API = CLI (`herdr api snapshot|schema`, `herdr agent list|get|read|prompt|wait`,
  `herdr worktree`, `herdr notification`), plugins, web client. https://herdr.dev/ .
  Ecosystem: kcosr/herdr-web (143★), umutciloglu/herdr-session-manager (3★, v0.1 — search across all
  Claude/Codex sessions + agent-to-agent messages).
- **cmux** manaflow-ai/cmux — 27k★, push 09-29; a Ghostty-based macOS terminal: vertical tabs
  with branch, PR status/number, ports, the latest notification; CLI+socket API; built-in browser.
  License: the README says "GPL", the LICENSE on GitHub = NOASSERTION (unconfirmed).
- **Warp** (Oz cloud agents, "Warp Factories" live 2026-08 — secondary, unconfirmed), wraps
  Claude Code/Codex/Gemini/OpenCode. https://docs.warp.dev/changelog/2026/

### 2c. Remote access / mobile clients for local sessions

- **Happy** slopus/happy — MIT, 24k★, push 09-28; a mobile/web client for Codex and Claude Code, E2E encryption.
- **CloudCLI / claudecodeui** siteboon — AGPL-3.0, 13.8k★, push 09-28; a web UI for Claude Code,
  OpenCode, Cursor CLI, Codex.
- **opcode** (ex Claudia) winfunc/opcode — AGPL-3.0, 22k★, push 09-18; a GUI for Claude Code, custom agents.
- **Omnara** — **pivot**: now an "open-source alternative to Claude Managed Agents" (durable agents,
  sandboxes, Postgres state), Apache-2.0, 2.9k★. No longer a "command center for Claude Code" in the
  original sense. https://www.omnara.com/

### 2d. Cloud/vendor panels

- **Codex app** (macOS, since 2026-02-02; OpenAI calls it a "command center for agents"): projects →
  threads, a worktree per thread, review pane (inline diff, stage, revert, request changes),
  automations (local/worktree). Codex only. https://openai.com/index/introducing-the-codex-app/
- **Claude Code desktop** (redesign 2026-04-14): a sidebar with all sessions, filter by status/project/
  environment, a worktree per session (`.claude/worktrees/`), terminal, editor, routines. Claude only.
  https://claude.com/blog/claude-code-desktop-redesign , https://code.claude.com/docs/en/desktop
- **GitHub Agent HQ / Mission Control** — public preview of Claude and Codex since 2026-02-04 for Copilot
  Pro+/Enterprise; tasks from issues (Assignees), Agents tab, VS Code; session logs "View session";
  draft PRs and iterations on review; 1 premium request per session. Cloud (Actions), **native GitHub
  tracker**. https://github.blog/changelog/2026-02-04-claude-and-codex-are-now-available-in-public-preview-on-github/
- **Cursor Cloud Agents** — VMs in the cloud (or self-hosted Enterprise), cursor.com/agents + Agents
  Window, Slack/GitHub/Linear/webhooks triggers, video artifacts. https://cursor.com/blog/cloud-agents
- **Linear Agents** — not a dashboard but a **data model**: AgentSession (states: working / awaiting
  input / error / complete), AgentActivity (thought, action, elicitation, response, error), the plan as a
  full array of steps. A good model of a status protocol. https://linear.app/developers/agent-interaction
- **Devin** (2026 release notes exist: https://docs.devin.ai/release-notes/2026), **Factory**
  (Droids, Missions), **Google Jules** (secondary: no changelog since 2026-03 — unconfirmed), **Amp**
  — all alive, but each is its own agent + its own cloud UI; they do not orchestrate Claude Code/Codex CLI (except Warp).

### 2e. Self-hosted control planes (Paperclip class)

- **builderz-labs/mission-control** — MIT, 6.3k★, alpha; tasks inbox → execution → review →
  receipts, spend, fleet by runtime (OpenClaw, Claude Code, Codex), REST/OpenAPI, MCP, SSE; SQLite.
  The README is honest: "one agent on one machine already stays understandable from its native CLI".
- **Omnara** (after the pivot) — infrastructure for durable agents, not a UI for a developer.

### Dead / wound down, and why

- **Terragon Labs** — shut down 2026-02-09, "weren't able to reach the level of traction"; OSS snapshot
  terragon-labs/terragon-oss without support. https://docs.terragonlabs.com/docs/resources/shutdown
- **Vibe Kanban / Bloop** — 2026-04-10, "vast majority are free users… couldn't find a business
  model"; the cloud part removed, the local one is community-run, per HN "stopped improving… annoying bugs".
- **Crystal** → renamed/replaced by Nimbalyst (2026-02).
- **Omnara** — a pivot from a mobile remote for Claude Code to managed-agents infrastructure.
- The common cause: vendors (Codex app, Claude Code desktop, GitHub Agent HQ, Cursor) built in
  "session list + worktree + diff" for free; wrappers with no value of their own beyond that did not survive.

## 3. User patterns and pains

Pains (primary sources — HN):
- **The bottleneck is review, not generation**: "8 parallel cards means 8x the diffs to read"; "30 minutes of
  planning and 30 minutes of implementation… is too big to review" (HN, Kanbots, ~May 2026 by id —
  the date is an estimate) https://news.ycombinator.com/item?id=48239413
- "What does the kanban interface add here?" — a board without a link to real facts (PR/CI) is not needed.
- You need your own key and a GitHub integration when a service goes away (Terragon) — lock-in of cloud orchestrators
  https://news.ycombinator.com/item?id=46589735
- The market: "handoff" (one prompt → agent) vs "pipeline" (deterministic steps with human gates);
  commercial products are almost all handoff, internal platforms of large companies are pipeline
  (Jack Kora, 2026-06-29, secondary: https://jackkora.com/p/mapping-the-ai-coding-orchestrator).
  The owner's pipeline is of the "pipeline" class, which almost does not exist on the market for solo users.

A catalog of features of a good command center (information architecture):
1. **Portfolio**: projects → open tasks by pipeline stage (from the tracker, not your own DB).
2. **Task card = a join of facts**: issue + stage + branch/worktree + agent sessions (runtime,
   machine, status) + PR + CI + review + stage artifacts. Status is derived from facts (as in AO), not
   moved by hand.
3. **Live agent status**: working / waiting-for-input / blocked / idle / error (Herdr, Linear
   AgentSession) + a "human needed" notification + jump-to-pane.
4. **A feed of "what the agent did"**: transcript/summary, tool actions, changed files, cost.
5. **Stage artifacts**: spec, plan, delivery report, evidence (screenshots/video), handoff — rendered
   right in the card.
6. **Review**: diff with comments → sent back to the same agent (Superset, Vibe Kanban, AO).
7. **Gates**: explicit owner actions "accept the stage / start the next one" with a record in the tracker.
8. **Recent / quick-jump**: recent tasks, command palette, deep links into the terminal/GitHub.
9. **Launch**: "new session for a task" = create a worktree + branch + an agent with the task context.
10. **Open data/API**: all state in git/GitHub/files, the UI is only a projection (Nimbalyst
    "plain files on disk"), so it survives the death of the tool.

## 4. Comparison table

| Tool | Who it is for | Claude Code / Codex | Where | Task source | Worktree | Review/artifacts | API | License/price | Activity | Lock-in |
|---|---|---|---|---|---|---|---|---|---|---|
| Paperclip | "AI companies", autonomous agents | yes/yes (+Cursor, OpenCode, HTTP) | self-host server+Postgres | its own ticket system (BYO tracker — roadmap) | yes | work products, plans, approvals; not PR review | REST, plugins | MIT | 93k★, very active, CVE 10.0 in 2026-08 | high (its own model) |
| Agent Orchestrator | developer/team | yes/yes, 25+ | local desktop+daemon | GitHub, GitLab + orchestrator | yes | PR, CI, reviews, preview in the card | CLI, plugins | Apache-2.0, telemetry | 12.5k★, release 09-26 | low-medium |
| Emdash | developer | yes/yes | local + SSH | Linear, GitHub, Jira, GitLab… | yes | diff, PR, CI, merge | — (unconfirmed) | Apache-2.0 | 5.9k★, active | low |
| Conductor | developer on Mac | yes/yes (+Cursor, OpenCode) | local (+cloud — unconfirmed) | its own input; Linear/GitHub — unconfirmed | yes | diff, PR, merge | — | closed, free local | active, $22M | medium |
| Superset | developer | yes/yes | local | its own | yes | diff, PR feedback, preview, Pages | — | ELv2 | 14.7k★ | medium |
| Nimbalyst | developer, visual | yes/yes (+OpenCode) | local desktop + iOS | markdown trackers in the repo | yes | red/green diff, session kanban | MCP | MIT | 1.8k★ | low |
| Vibe Kanban | developer | yes/yes, 10+ | local | its own kanban | yes | inline diff comments, preview | MCP | Apache-2.0 | company shut down 04-2026 | risk of abandonment |
| Herdr | terminal power users | yes/yes, 25+ | local + SSH | none | helpers | none | socket API, plugins, web | Apache-2.0 | 41k★, active | low |
| cmux | terminal on Mac | yes/yes | local | none | none | PR status in the sidebar | CLI/socket | GPL (unconfirmed) | 27k★ | low |
| Codex app | Codex users | no/yes | local + cloud | its own | yes | review pane | — | OpenAI subscription | active | vendor |
| Claude Code desktop | Claude users | yes/no | local + cloud | its own | yes | diff, terminal | — | Anthropic subscription | active | vendor |
| GitHub Agent HQ | teams on GitHub | yes/yes (+Copilot) | cloud (Actions) | **GitHub Issues** | branches/PR | session log, draft PR | GitHub API | Copilot Pro+/Ent | preview since 02-2026 | medium (GitHub already in use) |
| Cursor Cloud Agents | Cursor teams | its own agent | cloud VM | GitHub, Linear, Slack | branches/PR | video artifacts | API | paid | active | high |
| mission-control | self-host ops | yes/yes | self-host | its own | — | runs, review, receipts | REST/MCP/SSE | MIT, alpha | 6.3k★ | medium |
| Devin/Factory/Jules/Amp | teams | their own agents | cloud | GitHub/Linear/Slack | cloud | their own UIs | API | paid | alive | high |

## 5. Evaluation of options for the owner

Key conclusion: there is no ready-made tool that (a) takes tasks and stages from **GitHub Issues/Projects**,
(b) knows **your versioned pipeline and stage artifacts**, and (c) sees **live Herdr sessions
of Claude Code and Codex** across several repos. The market splits into "workspace managers" (worktree+diff+PR,
no pipeline) and "control planes" (their own DB, autonomous agents, their own tracker).

- **Adopt Paperclip** — not recommended: a second tracker, an "autonomous company" model instead of
  owner-driven stages, not a code review tool, heavy operations and fresh critical CVEs.
- **Adopt the closest ready-made one: Agent Orchestrator (AO).** Closest in idea: a GitHub tracker adapter,
  cards derived from session/PR/CI/review facts, Claude Code+Codex, worktree, Apache-2.0.
  Gaps: it does not know your stages (idea→spec→tickets…), artifacts (spec, delivery report, evidence,
  handoff) and owner gates; it wants to launch workers itself (a conflict with Herdr as the place where sessions live);
  young (v0.13), telemetry. An alternative of the same class is Emdash (broader trackers, SSH, hooks).
- **Combine (recommended as a first step):** keep the Herdr runtime (+ herdr-web for a session overview),
  GitHub Projects as the canon of stages (Stage/Status field), GitHub PRs as the place for review and the delivery report,
  and one of the workspace managers (AO/Emdash/Conductor) optionally for diff/PR iterations. Downside: 2–3
  windows, no single card "task ↔ sessions ↔ artifacts".
- **Build thin (recommended if a presentable single center is needed):** a read-mostly web projection
  without its own DB as a source of truth: sources = GitHub GraphQL (Issues/Projects/PR/checks), `git worktree list` over
  the repos from the registry, `herdr api snapshot` / `herdr agent list|read|wait` (status, jump), artifact
  files in the branch/PR (spec, report, evidence), Claude/Codex transcripts. Actions = thin commands:
  "open a session for an issue" (herdr worktree + agent start), "advance the stage" (Project field +
  comment), "send a review remark to the agent" (`herdr agent prompt`). Take the status model from
  Linear AgentSession. The harness is the source of the stage/artifact definitions (a schema in the repo), the UI only
  renders it. This is the "pipeline" class that does not exist on the market for solo users; the risk is maintaining your own code, but
  with no DB of its own and with GitHub/Herdr as the truth, lock-in and the cost of abandoning it are minimal.

## 6. Owner's clarification: small composable tools (priority)

The owner does not need a Paperclip-level product. Below are the bricks that fit together around the already
installed Herdr (`herdr 0.9.1`), `gh` and a small web interface of their own.

### 6a. The Herdr plugin platform (the basis of the build)

A plugin = `herdr-plugin.toml` (id, version, min_herdr_version) + any executable code (Bash, JS,
Rust, Lua…). It can: actions (keys, `herdr plugin action invoke`), its own terminal panes
(overlay/popup/split/tab), handlers for Herdr events, link handlers (ctrl-click on a link → action),
access to the CLI/socket API and env (`HERDR_PLUGIN_STATE_DIR`, pane/workspace IDs). No sandbox.
Registry: https://herdr.dev/plugins — an auto-index of GitHub repos with the topic `herdr-plugin`
(https://assets.herdr.dev/plugins/index.json: 1375 plugins as of 2026-09-29). Guide:
https://flaviocopes.com/herdr-plugins/ (secondary, the author is known). The API itself: `herdr api schema|snapshot`,
`herdr agent list|get|read|prompt|wait|explain`, `herdr worktree`, `herdr notification`.

Stars/dates are from the Herdr registry as of 2026-09-29; all below are MIT unless noted otherwise (verified for
reviewr, radar, projects, roamgate, herdr-web).

| Plugin / tool | ★ / push | What it provides | Role in the center |
|---|---|---|---|
| **eliasstravik/herdr-projects** | 510 / 09-28 (created 09-18) | a coordinator conversation + worker threads in their own worktrees/branches; a "what needs you" sidebar, a `review · PR #4` line, `~40%`; agents report progress themselves; a ticker watches PRs | closest to an "orchestrator inside Herdr"; take the "thread = task" model, self-reported progress |
| **eliasstravik/herdr-agent-progress** | 31 / 09-15 | the agent itself writes progress/activity to the sidebar | a stage status protocol from the agent |
| **hhdebb/herdr-radar** | 102 / 09-28 | sidebar: working / waiting for you / done-before-viewing / 3 idle levels, grouping by project, worktrees under the repo | a ready "who is where and who is waiting for me" |
| **persiyanov/herdr-reviewr** | 790 / 09-23 | a review panel: diff (uncommitted / branch / last turn / commits), line comments → agent, read-only PR view, markdown preview | review in the terminal, "last-turn diff" = "what the agent did" |
| **jhochenbaum/herdr-hunk-diff** | 133 / 09-27 | review in hunk + inline comments back to the agent | an alternative to reviewr |
| **plannotator/herdr-annotate** | 580 / 09-27 | annotate agent replies and documents (spec/plan) → back to the agent | review of spec/plan at the pre-code stages |
| **tomasvarga/herdr-pickr** | 20 / 07-13 | ctrl-click on a PR link → choose a reviewer tool, optionally an AI first pass | review routing |
| **wyattjoh/herdr-plugin-gh-pr** | 22 / 07-16 | the PR status of the focused pane's branch in the sidebar | a pane ↔ PR link |
| **bredebjorhovd/herdr-board** | 1 / 08-14 | a TUI board: GitHub issues/PRs (+Linear) → dispatch into a pane; BLOCKED/WORKING/READY/REVIEW/FAILED; PR review is delivered to the authoring agent | **conceptually closest to what is needed**, but 1★ — take as a model/fork, not as a dependency |
| nelsonPires5/herdr-board | 162 / 09-29 | kanban, cards = prompts into visible panes | its own board, not GitHub |
| thanhdat77/herdr-navigator | 175 / 09-24 | fuzzy jump to workspace/agent/project/session/action | quick-jump |
| andrewchng/herdr-sessionizer | 49 / 09-26 | open a project/worktree + a TOML layout of tabs/panes/commands | a "session per task" launcher |
| devashish2203/herdr-worktrunk | 164 / 09-21 | Worktrunk integration | a worktree per task |
| tdi/herdr-worktree-setup | 27 / 09-11 | setup on worktree creation (.env, mise, direnv, deps) | worktree readiness |
| cloudmanic/herdr-plus (Go) | 340 / 09-04 | Projects + Quick Actions | projects/quick actions |
| aemrebarut/herdr-dagr | 88 / 08-23 | a swarm as a live DAG: attempts, review gates, evidence | an idea for visualizing stages/gates |
| deimantasnork/captains-deck | 29 / 09-28 | a read-only flow kanban | a model of a read-only board |
| furkankly/zoetrope | 956 / 09-15 | a Claude Code/Codex session as a live flow graph, terminal or browser | "what the agent did" |
| nicosuave/memex | 234 / 09-22 | search across Claude/Codex/… transcripts, resume, tokens | session history |
| Davidcreador/herdr-token-dashboard | 23 / 09-14 | tokens per pane | spend |
| **Herdr web/mobile clients**: powerfooI/roamgate (256, TS; terminals, agents, files and diff annotations, desktop+mobile), kcosr/herdr-web (143, MIT; uses private Herdr APIs — fragile), devswha/herdr-web-ui (35), 0cv/herdr-mobile-relay (261, Go; approve from a phone, push) | | | a ready web layer over sessions; roamgate is a candidate for a base/fork |

The ecosystem is very young (most created 06–09.2026), many have a single author — pick 2–4 plugins,
pin versions, keep the critical logic in-house.

### 6b. Non-Herdr bricks

- **gh-dash** dlvhdr/gh-dash — MIT, 12.6k★, push 09-22: a TUI over PRs/issues with per-repo sections (YAML),
  **custom actions** (for example, "launch a Herdr session for this issue"). https://github.com/dlvhdr/gh-dash
- **Worktrunk** max-sixty/worktrunk — 8.5k★, push 09-29 (Rust; the license in the API is NOASSERTION —
  unconfirmed): `wt switch/create/remove`, template-based paths, hooks. https://worktrunk.dev
- **ghzinga** osolmaz/ghzinga — 87★: a clickable TUI for a single issue/PR.
- **Agent Sessions** jazzyalex/agent-sessions — MIT, 882★: a macOS app — search across local Codex/Claude/…
  sessions, resume, quota/cost per session. **claude-code-log** daaain — 1.2k★: JSONL→HTML/MD
  of a transcript (for evidence/delivery report). **ccusage** — 18.8k★: spend.
- **coder/agentapi** — 1.5k★, push 09-13: an HTTP API on top of Claude Code/Codex/… (an alternative if
  one ever moves away from the terminal).
- **TUI session managers (alternatives to Herdr, not needed with Herdr, but sources of ideas)**:
  agent-of-empires (MIT, 3.3k★; TUI+web, worktree/container), agent-deck (970★), ccmanager (1.25k★),
  amux (162★), seshagy (20★; tmux+herdr, zoxide launcher).
- **Linear AgentSession** as a status schema (not a product): states + activity types + plan.
- **tsk** smarzban/tsk (151★): a terminal tracker, "a TUI for you, a CLI for agents" — an idea, but the
  owner's tracker is GitHub.

### 6c. What to take from the large products (briefly)

| Product | Idea / part to take |
|---|---|
| Paperclip | atomic task checkout (one task — one active session); goal ancestry in the agent brief; routines → issue; immutable activity log; a run = a structured log + a cost event |
| Agent Orchestrator | **the card position is derived from facts** (session, PR, CI, review), not moved by hand; "send CI/review feedback to the same agent"; the tracker adapter interface (backend/internal/ports/tracker.go, Apache-2.0 — can be read/borrowed) |
| Emdash | installing lifecycle hooks into agents for status/notifications/resume; tasks from GitHub/Linear |
| Conductor | the workspace lifecycle: create → review → PR → merge → **archive**; a "shared context folder" per workspace |
| Superset | dev server preview with ports per worktree; "Pages" — a report with comments that the agent reworks at the same link (an analog of the delivery report) |
| Nimbalyst | all state as plain files in the repo; session kanban with a session ↔ files link |
| Vibe Kanban | inline diff comments → agent; the lesson: the cloud part without a business model died, the local one survived |
| Codex app / Claude Code desktop | a session sidebar with a status/project filter; a review pane with revert/stage; automations |
| GitHub Agent HQ | task = issue, launch via Assignees, the session log tied to the PR — a model compatible with your tracker |
| Linear | AgentSession/AgentActivity as a vocabulary of statuses and "elicitation" (the agent waits for a human) |
| mission-control | a "completion receipt" — the task's final receipt (what was executed, what passed review) |
| Cursor Cloud Agents | a video artifact as proof of work |

### 6d. Revised recommendation (taking the clarification into account)

**Combine + a thin layer of your own, no platform.**
1. Runtime and live status: Herdr + `herdr-radar` (who is waiting) + `herdr-navigator` (jumps).
2. A "session per issue" launcher: Worktrunk (+ `herdr-worktrunk`, `herdr-worktree-setup`) +
   the `herdr-sessionizer` layout; invoked from a **gh-dash custom action** or your own Herdr action
   `start-issue <repo>#<n>` → worktree + branch + an agent with a brief from the issue/spec.
3. Review: `herdr-reviewr` (diff/last-turn/PR, comments → agent), `herdr-annotate` for spec/plan;
   the final review and delivery report — in the GitHub PR.
4. "Portfolio and stages": a small web interface of your own (read-mostly, no DB of its own): GitHub GraphQL
   (Project field Stage, issues, PR, checks) × `git worktree list` over the registry × `herdr api snapshot`
   × artifacts in the branch/PR. Actions are `gh` and `herdr` calls (stage transition, `agent prompt`,
   focus). This is the part nobody else has, and it knows your harness.
5. As a starting point for the web part, consider forking **roamgate** (TS, MIT; it already handles terminals, agents,
   diff) or just borrowing its client for the Herdr API; for the board — the ideas of `bredebjorhovd/herdr-board`
   and AO's derived status.
6. A status protocol from the agent: a skill/hook in the harness writes progress (like `herdr-agent-progress`) and
   links to stage artifacts into an issue/PR comment; the center only reads.

Risks: the youth and bus factor of Herdr plugins (pin versions, minimal dependencies); kcosr/herdr-web
uses private Herdr APIs; your own web app is maintenance, but with GitHub/Herdr/git as the only
sources of truth it can be thrown away without losing data.

Research limitations: commercial products (Conductor cloud, Warp Factories, Jules) are partly
from secondary sources; the AO plugin docs did not open (404), trackers were determined from the code tree;
HN thread dates are estimated from ids.
