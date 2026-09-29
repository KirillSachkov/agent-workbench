# Research 4 — where and how to review stage artifacts (review surfaces)

Date: 2026-09-27. Sources are official documentation, changelogs and repositories; secondary sources are marked.
"unconfirmed" means it could not be verified against a primary source in this run.

Question: the owner does not watch agents live but accepts finished stage artifacts (spec, ticket
breakdown, delivery report with tests/screenshots/video, handoff). We need a surface that works
everywhere the harness is installed, for Claude Code and Codex, locally before a PR and later on GitHub.

---

## 1. Vendor-native surfaces

### 1.1 Google Antigravity — Artifacts
- **What the owner sees:** Task list, Implementation Plan (markdown), Walkthrough (summary of changes,
  how to verify), code diffs, screenshots, browser recordings.
  https://antigravity.google/docs/artifacts/ , https://antigravity.google/docs/walkthrough/ ,
  https://antigravity.google/docs/ide/implementation-plan/
- **Feedback:** Google Docs-style inline comments on the plan / task list / diff / walkthrough;
  "provide inline text feedback to steer the agent… before it modifies any local files". The agent
  receives the comments in the same conversation after an explicit "proceed". Secondary:
  https://atamel.dev/posts/2025/12-10_antigravity_provide_feedback/
- **Storage/versions:** markdown files in `~/.gemini/antigravity/brain/<conversation-id>/`
  (`task.md`, `implementation_plan.md`, `walkthrough.md` + `.metadata.json`) — outside the repository,
  tied to the conversation. The source is secondary (https://kerrick.blog/posts/2026/a-power-user-guide-for-google-antigravity/ ,
  https://github.com/michaelw9999/antigravity-cli ) — officially **unconfirmed**.
- **Claude/Codex:** no — Antigravity's own agent only. For this context it is a UX reference, not a surface.
- **Risks:** IDE lock-in, artifacts outside git, feedback lives only in the session.

### 1.2 Claude — Artifacts from Claude Code (claude.ai/code/artifacts)
Official: https://code.claude.com/docs/en/artifacts
- **What it shows:** a live HTML or Markdown page at a private claude.ai URL; `.md` is rendered
  as a styled document with code highlighting. Gallery — https://claude.ai/code/artifacts ;
  `/artifacts` in the CLI (v2.1.208+) — list, open, attach to a session.
- **Versions:** every publish is a version; in Share you choose which version viewers see. Another
  session updates the artifact by URL (or via `/artifacts`), otherwise it creates a new one.
- **Feedback → agent:** comments exist only on an artifact shared **inside the organization**
  (Team/Enterprise), Claude Code v2.1.221+. "Send to Claude" or `@claude` activates a thread;
  the session that published the artifact "watches that artifact for comments for as long as the
  session runs" (v2.1.228+); auto-reply/auto-edit depend on the permission mode, limit of 60 comments
  per hour. After the session ends — only on request, "read the comments at the URL". On Pro/Max, sharing
  is only via a public link, and comments are disabled on a public artifact → **for Pro/Max
  comments are effectively unavailable**. Known bug: a comment is "sent" but never arrives —
  https://github.com/anthropics/claude-code/issues/92618
- **Media:** per the public docs — a single self-contained page ≤16 MiB, external images are
  blocked by CSP, images only as data URIs, relative links do not work. In this session's runtime the
  Artifact tool already supports `files` (multi-file artifacts) and an asset store (image/video/PDF, ≤15 MB
  per file) — not reflected in the public docs, looks like a gated rollout (**unconfirmed** as GA).
- **Local vs cloud:** cloud only (Anthropic infra), requires a claude.ai login; does not work with an API key,
  Bedrock/Vertex, ZDR/HIPAA; off by default in the Agent SDK and the GitHub Action.
- **Codex:** no (Codex does not publish; it also cannot be read via WebFetch — authorization is required).
- **Other:** `/design` — a canvas artifact (v2.1.265+). Retention is configured by an admin
  (Team/Enterprise). Compliance API for export.
- **Claude Docs** (beta since 2026-09-16): rich-text documents in claude.ai, comments, collaborative
  editing, export to Word/PDF/Google Docs/markdown; stored in the Artifacts tab.
  Secondary: https://www.computerworld.com/article/4223177/anthropic-tries-to-make-claude-stickier-with-launch-of-docs-and-slides.html ,
  https://www.engadget.com/2259938/anthropics-claude-can-now-create-editable-documents-for-you-cowork-chat-together/ .
  This Claude Code session has a Claude Docs connector (MCP `claude_ai_Claude_Docs`) → Claude Code
  can create/read docs and reply to comments. I found no official page on code.claude.com
  — **unconfirmed** as a supported path for Claude Code at all.
- **Claude Code desktop:** preview of the running app, visual diff with inline comments,
  "Review code", PR/CI monitoring with auto-fix (2026-02-20):
  https://claude.com/blog/preview-review-and-merge-with-claude-code ; Code Review on a PR:
  https://code.claude.com/docs/en/code-review
- **Assessment:** the best "presentation" layer for Claude (pretty, versions, a link, phone),
  but not a source of truth: cloud-only, Claude only, comments require Team/Enterprise,
  tied to a live session.

### 1.3 OpenAI Codex (app / cloud / CLI)
- **Review pane (app):** shows git state (Unstaged/Staged/Commit/Branch/Last turn),
  stage/revert per hunk, inline comments on lines: "Codex treats inline comments as review
  guidance" — they go to the same thread. PR context and reviewer comments from GitHub in the sidebar.
  https://learn.chatgpt.com/docs/code-review?surface=app
- **Artifacts viewer:** preview of documents, presentations, spreadsheets, PDF, HTML (interactive + source);
  you can "point to a specific part of a file and tell ChatGPT what to change" — the annotation goes to the
  thread. The CLI has no preview. https://learn.chatgpt.com/docs/artifacts-viewer
  Markdown rendering as a separate type — **unconfirmed**.
- **In-app browser + Annotate mode:** click a page element, note + element screenshot → to the
  agent chat. https://developers.openai.com/codex/appshots (Appshots), secondary:
  https://x.com/kr0der/status/2047510880741364205
- **Codex cloud (web):** task summary + diff, "citations of terminal logs and test outputs",
  a screenshot of the result is attached to the task and to the GitHub PR.
  https://developers.openai.com/codex/cloud , https://openai.com/index/introducing-codex/ ,
  https://openai.com/index/work-with-codex-from-anywhere/
- **Task sidebar** (plan, sources, artifacts, summary) — Platform 26.415, per a secondary source
  https://codex.danielvaughan.com/2026/04/17/codex-app-workspace-pr-review-task-sidebar-artifact-viewer/
- **Assessment:** a strong review surface for Codex sessions, but only for them, and comments live
  in the thread. Claude does not get there.

## 2. GitHub as a surface

- **Markdown:** rendered in issues/PRs/files; prose diff (rich diff) for `.md`:
  https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files
  **Limitation:** in rich diff you cannot leave inline comments — only in the source diff
  (https://github.com/orgs/community/discussions/186730). Workaround — third-party browser extensions
  (https://github.com/sabbour/md-review-extension , https://github.com/chienyuanchang/rich-diff-comments ),
  which post an ordinary review comment on the source line.
- **Feedback → agent:** pull request review comments and issue comments are read via `gh`
  (`gh pr view --comments`, `gh api repos/{o}/{r}/pulls/{n}/comments`, `gh issue view --comments`).
  There is no automatic delivery into a local session — the agent pulls on command / at stage start.
  Claude Code desktop can monitor PR/CI; the Codex app shows PR comments in the sidebar.
- **Media:** images and video (mp4/mov) are attached to comments and the PR description via
  drag-and-drop (size limits depend on the plan — **unconfirmed** in this run);
  files in the repository render as images.
- **Agent HQ / Mission Control / Agents panel:** assign tasks to Copilot, Claude, Codex
  (GitHub cloud agents), view session logs, steer, jump to the PR.
  https://github.blog/changelog/2025-10-28-a-mission-control-to-assign-steer-and-track-copilot-coding-agent-tasks/ ,
  https://github.blog/changelog/2026-02-04-claude-and-codex-are-now-available-in-public-preview-on-github/ ,
  https://github.blog/changelog/2026-02-26-claude-and-codex-now-available-for-copilot-business-pro-users/
  These are cloud agents on the GitHub side (a Copilot subscription is required), not local Claude Code / Codex
  with a harness; not suitable for "viewing the result of a local session".
- **GitHub Pages / Projects:** Pages — publishing a static site from docs (for a private
  repository, Pages visibility restriction is available only on Enterprise Cloud — **unconfirmed**,
  check if needed). Projects — a status board, not a place to read an artifact.
- **Assessment:** the best durable/audit layer, identical for both agents, but only after push/PR;
  reading a long spec in a source diff is inconvenient; comments to the agent are pull, not push.

## 3. Local site generators and md-viewers

| Tool | What it provides | Downsides for this case |
|---|---|---|
| Astro Starlight https://starlight.astro.build | a folder of md/mdx + front matter, dev server with live reload, search, components (an index/board can be built from front matter via content collections) | Node dependency, no comments |
| VitePress https://vitepress.dev | fast dev server, data loaders (`createContentLoader`) for an index from front matter | no comments |
| Material for MkDocs https://squidfunk.github.io/mkdocs-material/ | Python, mature | **end of life 2026-11-05**, development moved to Zensical https://zensical.org (secondary: https://fpgmaas.com/blog/collapse-of-mkdocs/ ) |
| Docusaurus https://docusaurus.io | React, documentation versioning | heavy for a solo developer |
| Quarto https://quarto.org | md + computed reports, `quarto preview` | somewhat heavy, science-oriented |

Common: no built-in channel for comments back to the agent (you can use Giscus → GitHub Discussions
https://giscus.app , but that is GitHub again). Video is a plain `<video>`/markdown link to a file in the folder.

Lightweight viewers: Obsidian (a vault over `docs/`, backlinks, images/video, plugins; no comments
to the agent) https://obsidian.md ; Typora; VS Code markdown preview (built in, available in the Codex IDE
extension and Claude Code IDE integrations); terminal — `glow` https://github.com/charmbracelet/glow ,
`frogmouth` https://github.com/Textualize/frogmouth (no images/video).

**Specialized annotate tools (the most relevant finding):**
- **Plannotator** https://github.com/backnotprop/plannotator , https://plannotator.ai — visual
  annotation of a plan / any `.md` / folder / URL / code diff, `/plannotator-annotate spec.md`,
  `/plannotator-review`; feedback returns to the agent's session (for Claude Code — a hook on
  ExitPlanMode and slash commands). Supports: Claude Code, Codex, Copilot CLI, Gemini CLI, OpenCode,
  Amp, Pi and others. Local-first, no telemetry; share links with AES-256-GCM. Apache-2.0/MIT, ~9k stars.
  Images/video inside the document — **unconfirmed**.
- **plannotator-tui** https://github.com/plannotator/plannotator-tui — markdown annotation in the
  terminal (select, comment, looks-good, delete → send to the agent).
- **herdr-annotate** https://github.com/plannotator/herdr-annotate — the same inside Herdr (the
  owner already has Herdr): comments on terminal text, markdown documents and agent replies,
  sent straight to the agent's pane. The README could not be opened (socket closed), description is from search
  results — details **unconfirmed**.

## 4. GUIs/dashboards over agent sessions (checked whether they exist)

| Product | Status (2026-09) | Per-task artifacts | Comment back to the agent | Claude/Codex |
|---|---|---|---|---|
| Vibe Kanban https://github.com/BloopAI/vibe-kanban | **sunsetting** (README banner), Apache-2.0, self-host | kanban issues, workspace (branch+terminal+dev server), diff, built-in browser preview | inline comments on the diff → to the agent | both + 8 more |
| Conductor https://www.conductor.build/docs/ | active, macOS only, Pro $50/month | workspace, diff viewer, checks, PR/merge | yes (details **unconfirmed**) | Claude Code, Codex, Cursor, OpenCode |
| Nimbalyst (ex-Crystal) https://github.com/nimbalyst/nimbalyst ; Crystal deprecated https://github.com/stravu/crystal | active, MIT, mac/win/linux + iOS | WYSIWYG markdown, mockups, Mermaid, session kanban, red/green diff of agent edits, "plain files on disk in your git repo" | accept/reject edits; comments — **unconfirmed** | Claude Code, Codex |
| Sculptor (Imbue) https://imbue.com/product/sculptor , https://docs.imbue.com/changelog | beta, free, Mac, Docker containers | Pairing Mode, merge review UI, suggestions | — | Claude Code (Codex — **unconfirmed**) |
| CloudCLI / claudecodeui https://github.com/siteboon/claudecodeui | active (secondary) | web/mobile session UI, files, git | chat into the session | Claude Code, Codex, Cursor CLI |
| opcode (ex-Claudia) https://github.com/winfunc/opcode | no commits since 10.2025 (secondary) | session GUI | — | Claude Code |
| Terragon https://www.terragonlabs.com | **shut down 2026-02-09** | — | — | — |
| GitHub Agent HQ | see §2 | session logs → PR | steer in the session | GitHub cloud Claude/Codex |

Conclusion: all live GUIs are a "cockpit" for parallel sessions (worktree + diff), not a
surface for accepting stage artifacts. None of them reads your harness format (spec / delivery
report) as a first-class object; lock-in to their workspace model. The closest in spirit is
Nimbalyst (files in the repo, markdown WYSIWYG), but it is an IDE replacement.

## 5. CLI/TUI review

- `gh dash` https://github.com/dlvhdr/gh-dash — a PR/issues dashboard in the terminal, glamour rendering of
  markdown, comment from the preview pane (`c`, Ctrl+d), Checks tab. Good for "what is waiting for me",
  bad for a long spec and media.
- `lazygit` — git operations, not acceptance.
- Custom TUI (Textual) — cheap for a list of tasks/statuses, but there are effectively no images/video in the terminal
  (partly via the kitty/iTerm graphics protocols).
- Bottom line: a TUI is a navigator and an "inbox", not a place to read a spec or watch video.

## 6. Feedback and handoff mechanics

### How the owner's comment reaches the next session
| Channel | Delivery | Survives the session? | Agent-agnostic? |
|---|---|---|---|
| A file in the repo (comments in `review.md`/inline blocks in the spec, commit) | the agent reads it at stage start | yes, in git | yes |
| Plannotator / plannotator-tui / herdr-annotate | push straight into the current session | no (unless saved to a file) | yes (many agents) |
| GitHub PR review / issue comments | pull via `gh` | yes | yes |
| Claude artifact comments | push to the publishing session while it is alive; later pull by URL | a thread in claude.ai | Claude only, Team/Enterprise only |
| Claude Docs comments | via the Docs connector | yes, in claude.ai | Claude only |
| Codex review pane / artifacts viewer / Annotate | into the thread | in the thread history | Codex only |
| Antigravity comments | into the conversation | in the brain folder | Antigravity only |
| An MCP server over the review folder | pull tool-call | as storage | yes, if both have MCP (they do) |

### Handoff formats between sessions
- **OpenAI ExecPlans** (PLANS.md): a self-contained living document; mandatory sections Progress,
  Surprises & Discoveries, Decision Log, Outcomes & Retrospective; "restart from only the ExecPlan".
  https://github.com/openai/openai-cookbook/blob/main/articles/codex_exec_plans.md
- **Anthropic long-running harness:** `feature_list.json` + `claude-progress.txt` + git log,
  each session restores context from files, one feature per session, e2e via the browser.
  https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- **Matt Pocock `handoff` skill:** a compact document in the OS temp dir; what is already recorded (spec, plan,
  ADR, issues, commits, diffs) — by reference, not by copy; a suggested skills section; secret redaction.
  https://github.com/mattpocock/skills/blob/main/skills/productivity/handoff/SKILL.md ;
  known weakness — it loses the subtle decisions of long sessions (issue #186).
- **Amp Handoff:** generates a draft prompt + a list of files for a new thread, the human edits
  before sending. https://ampcode.com/news/handoff . The claim that in Amp "Neo" handoff was removed in
  favor of compaction is from a secondary source only; https://ampcode.com/news/neo did not open —
  **unconfirmed**.
- **What works better:** a file in the repo (git), self-contained, with sections "goal / state /
  decisions / what was verified (with evidence) / next step / completion criterion", links
  instead of copies, plus machine-readable front matter (status, issue, pr, head SHA, evidence paths).
  This is the intersection of ExecPlan + progress file + Pocock; it matches the Work AGENTS.md rule
  ("don't pass only the chat history").

---

## Comparison table

Scale: ++ excellent, + yes, ± partial, − no.

| Surface | Spec readability | Media (PNG/MP4) | Feedback → agent | Versions | Local before PR | Claude | Codex | Setup | Maturity | Lock-in |
|---|---|---|---|---|---|---|---|---|---|---|
| Files in the repo + VS Code/Obsidian preview | + | + | ± (editing the file) | ++ git | ++ | + | + | ~0 | ++ | none |
| Plannotator / herdr-annotate | ++ | ± | ++ push into the session | ± | ++ | + | + | low | + (young) | low, OSS |
| Local Starlight/VitePress | ++ (index/board) | ++ | − | ++ git | ++ | + | + | medium | ++ | none |
| Claude Artifacts / Docs | ++ | + (data URI; assets gated) | + Team/Ent. only, live session | + | − cloud | + | − | low | + (new, bugs) | high |
| Codex app panes / cloud | + | + (screenshots) | ++ in the thread | ± | + (app) | − | + | low | + | high |
| Antigravity Artifacts | ++ | ++ | ++ | ± | + | − | − | medium | + | high |
| GitHub PR/Issues | ± (source diff) / + (render) | + | ± pull via gh | ++ | − | + | + | ~0 | ++ | medium |
| Agent GUI (Conductor, Nimbalyst, Vibe Kanban) | ± | ± | + on the diff | ± | + | + | + | medium | ± (Vibe Kanban is shutting down) | medium |
| gh dash / TUI | − | − | ± | — | ± | + | + | low | ++ | none |

## Recommendation for this context (in layers)

1. **Source of truth — files in the repository, an agent-agnostic format.** For each stage —
   markdown with front matter: `spec.md`, `breakdown.md`, `delivery.md` (what was done, which checks
   passed with commands and outcome, head SHA, where to try it — staging URL/command, list of evidence),
   `handoff.md` following the ExecPlan/progress scheme. Media — `docs/evidence/<issue>/*.png|mp4` (the project
   already has a docs/evidence convention). The owner's feedback is also a file (`review.md` alongside
   or `> [!owner]` blocks in the spec) — that way any next session, Claude or Codex, will read it.
2. **Local rendering for reading — one harness command.** `inside-harness review <issue>` (or
   equivalent): builds a static HTML page from md + evidence (images, `<video>`) and opens it in the
   browser; an index page from front matter = an "acceptance board" without Projects. The cheapest option is
   no framework (python-markdown + a template); if search/navigation is needed — Starlight or VitePress
   in dev mode over `docs/`. Do not take Material for MkDocs (EOL 2026-11-05).
3. **Channel for comments into a live session — Plannotator / herdr-annotate** (both agents, local-first,
   the owner is already on Herdr). Harness rule: after sending, the agent saves annotations to `review.md`
   so they survive the session.
4. **GitHub — the durable layer after push.** The spec goes into the PR as `.md` (comments on lines in the source
   diff; optionally an extension for rich diff); the delivery report — the PR description / a comment with
   screenshots and video; agents read feedback via `gh`. Projects — statuses only.
5. **Vendor layers — optional, as a convenience, not as the canon.** Claude Artifact/Docs — a pretty
   presentation and a phone link when the session is Claude; remember: comments only Team/Enterprise,
   cloud, Claude only, retention and bug #92618. Codex app review pane / Annotate — when the session is
   Codex. The agent must carry the results of these channels over into the layer 1 files.

Not recommended: building the process around an Agent GUI (Vibe Kanban is shutting down, Terragon is closed,
Conductor is macOS + paid + its own workspace format), around Antigravity (not your agents), or
around a single vendor comment channel.
