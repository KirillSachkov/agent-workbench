# How coding-agent products show the agent's work to a human reviewer

State as of 2026-09-27. Sources are primary unless noted otherwise. The mark "unconfirmed" means the claim
was taken from a secondary source or from search results, and the primary page could not be opened.
The "data source" column always distinguishes **execution** (the artifact was produced by a real run:
a browser recording, a command log, a check run) from **model** (text the model wrote, which may
diverge from the facts).

---

## 1. Google Antigravity — Artifacts (the central case)

Timeline: launched November 18–20, 2025 (IDE, a VS Code fork, Editor + Manager view); Antigravity 2.0 —
Google I/O, May 19, 2026: a standalone desktop app, a Go CLI instead of Gemini CLI, an SDK, Enterprise.
- https://antigravity.google/blog/introducing-google-antigravity (18.11.2025)
- https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/ (20.11.2025)
- https://techcrunch.com/2026/05/19/google-launches-antigravity-2-0-with-an-updated-desktop-app-and-cli-tool-at-io-2026/

### Why
Google states the problem directly: "Delegating work to an agent requires trust, but scrolling through raw
tool calls is tedious". Artifacts let you "verify the agent's logic at a glance"; the UI shows
"tool calls grouped within tasks", not every action.
https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/ ,
https://antigravity.google/blog/introducing-google-antigravity

The definition in the docs: an artifact is "a structured deliverable created by the agent to accomplish its task and
communicate its progress and thinking to the human user". The list of types: rich markdown plans, code diffs,
architecture diagrams, images, browser recordings. They are produced mostly in **Planning Mode** (Fast
Mode has no planning). https://antigravity.google/docs/artifacts/ , https://antigravity.google/docs/artifact-review/

### Artifact types

| Artifact | Contents | When | Where | Data source |
|---|---|---|---|---|
| **Task List** | a structured list of steps (checklist), updated as work proceeds | before code and during work | review pane / sidebar | model |
| **Implementation Plan** (`/plan`) | "technical details on what revisions are necessary and are meant to be reviewed by the user"; task breakdown + "verification checkpoints" | after analysis/discovery and clarifying questions, before changes | review pane; a **Proceed** button in the conversation and in the artifact header | model |
| **Code diffs / Review Changes** | all diffs accumulated over the conversation in a separate editor pane | as work proceeds | the `Review Changes` button in the bottom bar of the Agent panel | execution (git diff) |
| **Walkthrough** | a "concise summary of the changes"; for browser tasks — screenshots and screen recordings | after implementation is complete | review pane | text — model; media — execution |
| **Visual Screenshots** | a snapshot of a page or element taken by the browser subagent | autonomously or on request | as an image artifact | execution |
| **Browser Recordings** | a video of the browser subagent's actions; played in a loop | when the subagent "may choose to generate a recording" | at the bottom of the Browser step UI + as a recording artifact | execution |
| Architecture diagrams (Mermaid) | diagrams in markdown artifacts | in the plan | desktop; in the CLI — Kitty graphics / ASCII / raw | model |

Sources: https://antigravity.google/docs/implementation-plan , https://antigravity.google/docs/plan/ ,
https://antigravity.google/docs/walkthrough/ , https://antigravity.google/docs/screenshots ,
https://antigravity.google/docs/ide/browser-recordings/ , https://antigravity.google/docs/ide/review-changes-editor/ ,
https://antigravity.google/docs/cli/artifacts/

Implementation Plan sections. The official docs do not give exact headings. Third-party reproductions
of the `implementation_plan.md` format name the sections Goal Description, User Review Required, Open Questions,
Proposed Changes (files: create/modify/delete) and Verification Plan; for the report they use `walkthrough.md`
(https://github.com/mohmaedeslam00116/cline/issues/42). **Unconfirmed** by the official documentation.
Only one thing is officially confirmed: the plan contains a task breakdown and "verification checkpoints"
(https://antigravity.google/docs/plan/).

### Feedback and approval
- **Comments as in Google Docs** on text artifacts and "select-and-comment feedback on screenshots".
  Feedback enters the agent's work "without interruption".
  https://antigravity.google/blog/introducing-google-antigravity
- In the plan you can leave inline comments on individual steps and ask to change the scope of work. Then
  there are two paths: **Proceed** or the **Review** toggle, which collects all comments and sends
  them to the agent. The agent either redoes the plan and asks for review again, or starts work.
  https://antigravity.google/docs/implementation-plan , https://antigravity.google/docs/plan/
- On diffs in Review Changes you can also leave inline comments, and the agent receives them.
  https://antigravity.google/docs/ide/review-changes-editor/
- A practical example from a Google Developer Advocate: a comment on the plan ("FastAPI instead of Flask"), on the
  task list ("more detailed verification instructions"), on the code, on a screenshot in the walkthrough ("color theme
  from blue to orange"). Comments must be explicitly submitted, and they affect subsequent steps.
  https://atamel.dev/posts/2025/12-10_antigravity_provide_feedback/ (10.12.2025)
- **Artifact Review Policy**: `Request Review` — the recommended and default mode: the agent always
  stops at the plan or diff and waits for explicit approval. `Always Proceed` — the agent does not
  stop. https://antigravity.google/docs/artifact-review/
- Separately there are permission presets (Default / Request Review / Turbo) and Deny > Ask > Allow rules
  for `command(...)`, `read_url`, `execute_url`. For the browser, Ask applies by default.
  https://antigravity.google/docs/permissions/
- **CLI (2.0)**: `ctrl+r` opens the Artifact Picker. Files are split into "Actionable Code Files"
  (code, configs, plans — need approval) and a "Media Drawer" (PNG/JPG/WebP/SVG/MP4/WebM). Keys: `y`
  approve, `n` reject, `Shift+A`/`Shift+R` — bulk actions, `p` — preview, `c` — a line comment
  in the detail viewer (marked 💬), `m` — Mermaid mode. The status bar hints
  "/artifact to review". https://antigravity.google/docs/cli/artifacts/

### Limitations
- The walkthrough is written by the model. Its text is not tied to logs the way it is in Codex, and the official docs do not
  describe citations. Only screenshots and recordings carry evidentiary weight.
  https://antigravity.google/docs/walkthrough/
- A browser recording is created when the subagent "may choose" to make it, that is, it is not guaranteed.
  The docs do not describe the recording format or comments on video (comments on screenshots are described).
  https://antigravity.google/docs/ide/browser-recordings/
- The whole review cycle works only in Planning Mode. The `Always Proceed` mode effectively disables the gate.
  https://antigravity.google/docs/artifact-review/
- I found no public primary complaints (for example, "the walkthrough claims something that did not happen"):
  **unconfirmed**.

---

## 2. OpenAI Codex

### Cloud tasks
- Since launch (May 2025) Codex has been "trained to provide verifiable evidence of its actions through citations of
  terminal logs and files". After a task the user sees a **diff view** and a "comprehensive log of actions".
  Citations lead to changed files and to executed terminal commands, so that one can "verify the
  outcomes of terminal commands, such as tests". System card, 16.05.2025:
  https://cdn.openai.com/pdf/8df7697b-c1b2-4222-be00-1fd3298f351d/codex_system_card.pdf
- The same document has the risk "**falsely claim to have completed a task**": in early tests, on impossible tasks Codex
  often claimed everything was done. Mitigations: an RL penalty for "results inconsistent with its actions" and a reward for
  admitting limitations. The share of correct "couldn't complete" admissions rose from 0.15 to 0.85; also
  helping is "User transparency and diff reviews". (the same PDF, §2.3)
- Per search results: the summary uses file citations for code changes and terminal citations in the
  Testing section. The openai.com page returned 403, so this is **unconfirmed** by direct reading.
  https://openai.com/index/introducing-codex/
- The cloud interface: "watch the task logs", "inspect the summary and diff", "open a pull request",
  follow-up. The task list shows repo, branch, `+31−1` statistics and merge status.
  https://learn.chatgpt.com/docs/cloud
- Upgrades (September 2025): Codex in the cloud can "spin up its own browser, look at what it built, iterate,
  and attach a screenshot of the result to the task and GitHub PR". Taken from search results, openai.com
  returned 403, so **partly unconfirmed**. https://openai.com/index/introducing-upgrades-to-codex/

### Codex app / review pane (2026)
- `/review`: comparison with the base branch, uncommitted changes, a specific commit, or custom instructions. The review
  pane shows Unstaged / Staged / Commit / Branch / **Last turn** (only the agent's latest edits).
  An inline comment is added via `+` on a line. Stage and Revert are available for the whole diff, a file, or
  a hunk. With `gh` present, PR context and reviewer feedback are visible alongside.
  https://learn.chatgpt.com/docs/code-review?surface=app
- The built-in browser: annotation mode with comments on page elements and regions. The agent takes
  screenshots to check rendering, and the human compares the page "alongside the code diff".
  https://learn.chatgpt.com/docs/browser
- September 2026 changelog: an "agent command center" (grouping by model, tokens and cost estimate),
  recaps with a separate "next actions" block, Mermaid directly in replies. https://learn.chatgpt.com/docs/changelog

### Codex code review (GitHub)
- Started via `@codex review` (a 👀 reaction) or Automatic reviews. Only **P0/P1** are shown. Rules
  come from `AGENTS.md`: in one description the section is called `## Code Review Rules`, in secondary
  sources — `## Review guidelines`. After a review you can write `@codex fix the P1 issue`, and Codex
  will start a cloud task that pushes the fix to the branch.
  https://learn.chatgpt.com/docs/third-party/github.md
- The review is claimed to "match the stated intent of a PR to the actual diff" and "execute code and tests to
  validate behavior". A claim from search results: **unconfirmed** by direct reading.
  https://openai.com/index/introducing-upgrades-to-codex/

### Limitations
Citations and summaries are written by the model, but they point to real logs, and that is the main protection.
OpenAI itself acknowledges the risk of falsely claiming a task is complete (system card).

---

## 3. Devin (Cognition)

| Artifact | Contents | Data source |
|---|---|---|
| **Interactive Planning** (Devin 2.0, 03.04.2025) | within seconds: relevant files, findings, a preliminary plan; the plan can be changed before autonomous work | model — https://cognition.com/blog/devin-2 |
| **Progress tab** | a single feed: shell commands, code edits, browser actions; clicking a step opens details; through the command history you can move in time (future steps are grayed out) | execution — https://docs.devin.ai/work-with-devin/devin-session-tools |
| Shell / IDE / Browser | command history with output, VS Code in real time, a browser or desktop; takeover is available (read-only or writable) | execution — same |
| **Test plan** (testing mode) | "single most important end-to-end flow", steps based on real code paths; the plan is sent to the human | model — https://docs.devin.ai/work-with-devin/testing-and-recordings |
| **Test recording** | a screen recording with annotations of key moments, auto-zoom and idle-time compression; delivered as an attachment to a message (webapp/Slack) | execution + agent annotations — same |
| **Test report** | "labeled screenshots from key moments"; a video player with **chapters** and a chronological list of **assertions: passed / failed / untested** | execution + agent markup — https://cognition.com/blog/testing-development (29.05.2026) |
| **PR description** | observed template: Summary / Problem / Solution / **Review & Testing Checklist for Human** / Notes / **Link to Devin run** | model; the template is visible in a real PR https://github.com/BerriAI/litellm/pull/43437 , not officially described (**unconfirmed** as a standard) |
| **Devin Review** | a PR regrouped logically rather than alphabetically, with an explanation of each hunk; recognizes move/copy; Bug Catcher: red (probable bug) / yellow (warning) / gray (FYI); Flags (Investigate / Informational); security with CWE; tabs Changes / Bugs / Flags / Description / Discussion / Commits; chat with codebase context; Auto-Fix | model — https://docs.devin.ai/work-with-devin/devin-review , https://cognition.com/blog/devin-review |

Feedback: testing mode is launched by the "Test the app" button or the "Pre-approve testing" setting.
Edits from the chat in Devin Review are applied as a commit. Devin Review triggers: `/devin review` or
auto-review on open, push and ready-for-review. For stacked PRs there are readiness indicators for each layer.
https://docs.devin.ai/work-with-devin/devin-review

Limitations acknowledged by Cognition itself: screenshots miss quickly disappearing UI (toasts). Models
"lean too heavily on executing JavaScript in the browser to trigger states programmatically instead of
clicking through the UI", meaning the check may not match the real user path.
Testing mode is intended as a "quick sanity check", not a replacement for CI.
https://cognition.com/blog/testing-development , https://docs.devin.ai/work-with-devin/testing-and-recordings

---

## 4. Cursor (cloud agents, Cursor 3.x, Bugbot)

- **Cloud Agents with Computer Use**, 24.02.2026: each agent works in its own VM, tests changes itself
  and attaches to the PR **artifacts: videos, screenshots, log references**. A human can take over the
  remote desktop, try the built software and hand control back to the agent. According to Cursor, ">30% of the PRs we
  merge at Cursor" are created by autonomous cloud agents.
  https://cursor.com/blog/agent-computer-use , https://cursor.com/docs/cloud-agent/capabilities
- Publishing artifacts to GitHub is enabled by the "Allow posting artifacts to GitHub" setting. The URLs are long and
  unguessable, but they **open without authentication**, because GitHub's image proxy requires this.
  https://cursor.com/docs/cloud-agent/capabilities
- A forum complaint (26.02.2026): screenshots and videos did not make it into the PR. A staff reply: posting to GitHub is
  "not yet supported". The feature was released on 26.03 as opt-in. A limitation remains: artifacts render only
  in the description of a PR created by the agent, but not in comments and not in PRs created by humans.
  https://forum.cursor.com/t/cursor-cloud-agents-do-not-post-their-screenshots-or-videos-to-pr/152974
- **Plan Mode**: clarifying questions, then a plan with to-dos and file paths. The plan is stored in markdown (by
  default in the home directory, "Save to workspace" puts it in the repository), it can be edited, then the Build
  button. https://cursor.com/docs/agent/plan-mode
- **Cursor 3** (02.04.2026): Agents Window, a new diffs view (review, stage, commit, PR management).
  https://cursor.com/changelog/3-0 . **Agent Review** works locally, automatically after a task or
  via `/agent-review`, in Quick and Deep modes, and takes `BUGBOT.md` into account.
  https://cursor.com/docs/agent/agent-review
- **Bugbot**: PR comments with the fields Title, Severity (high/medium/low), Description, and the links **Fix in
  Cursor** and **Fix in Web**. The check run takes the values `success` / `neutral` (issues found) /
  `failure` (if fail-on-unresolved is enabled). Rules: `.cursor/BUGBOT.md`, learned rules, which
  are built from reactions, replies and misses marked by humans (`@cursor remember`), and manual rules.
  Autofix starts a cloud agent and puts the fix into a new branch or the current one (no more than 3 attempts).
  https://cursor.com/docs/bugbot , https://cursor.com/blog/bugbot-learning , https://cursor.com/changelog/02-26-26

---

## 5. GitHub Copilot coding agent (now "Copilot cloud agent")

- **Session log**: "Copilot's internal reasoning and the tools it used to understand your repository, make
  changes, and validate its work". The overview shows tokens and duration. The log streams live, opens from the PR
  via the "View session" button, and there is a "Copilot started work" event.
  https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/manage-and-track-agents
- **Every commit message has a link to session logs** "for code review and auditing". Commits
  are signed (Verified), and the human who started the task is listed as co-author.
  https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations
- **PR body**: the agent breaks the issue into a checklist and ticks items as work proceeds
  (https://github.blog/ai-and-ml/github-copilot/assigning-and-completing-issues-with-coding-agent-in-github-copilot/).
  The title and body are updated when responding to feedback
  (https://github.blog/changelog/2025-07-30-copilot-coding-agent-keeps-pull-request-titles-and-bodies-up-to-date/),
  the agent follows the PR template (https://github.blog/changelog/2025-11-05-copilot-coding-agent-now-supports-pull-request-templates/).
- **Screenshots in the PR**: Playwright MCP is enabled by default, and Copilot can "share screenshots of what it has done
  in its pull request" (02.07.2025).
  https://github.blog/changelog/2025-07-02-copilot-coding-agent-now-has-its-own-web-browser/
- **Self-validation before finishing** (28.10.2025): CodeQL, a check of new dependencies against the Advisory DB,
  secret scanning and Copilot code review as a "second opinion". The agent tries to fix what is found and
  describes the result in the PR summary. The set of checks is configurable (03.2026).
  https://github.blog/changelog/2025-10-28-copilot-coding-agent-now-automatically-validates-code-security-and-quality/ ,
  https://github.blog/changelog/2026-03-18-configure-copilot-coding-agents-validation-tools/
- **Agents panel / Agents page / Agents tab** (mission control): a list of sessions, live logs, **steering**
  without stopping the agent (each message consumes AI credits), Stop, Archive, a jump to the PR.
  https://docs.github.com/en/copilot/concepts/agents/cloud-agent/agent-management
- After a session you can ask Copilot Chat "what changed, what was validated, and why", and it answers from
  the session logs. (manage-and-track-agents, see above)
- **Gates**: GitHub Actions do not run until a person with write access clicks "**Approve and run
  workflows**". The agent cannot approve or merge its own PR, and **the person who started the agent cannot approve it
  either**. The agent pushes only to `copilot/*`. Comments from users without write access are not
  passed to the agent. https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations
- GitHub's guide for reviewers of agent PRs (07.05.2026): block any weakening of CI; require
  tests that fail on the old behavior, a rollback plan and an explicit implementation plan.
  https://github.blog/ai-and-ml/generative-ai/agent-pull-requests-are-everywhere-heres-how-to-review-them/

---

## 6. Other products

### Google Jules
- Before code, a **plan approval** is shown: reasoning and steps that can be expanded and commented on
  in the chat. For **auto-approved plans**, since 26.01.2026 a **Planning Critic** works — a second agent that
  critiques and refines the plan before execution. A "9.5% reduction in task failure rates" is claimed.
  https://jules.google/docs/changelog/2026-01-26-1/
- **Critic** (08.2025) checks the final patch in a single pass. It fixes nothing, only flags
  problems and returns the patch to Jules. https://developers.googleblog.com/en/meet-jules-sharpest-critic-and-most-valuable-ally/
- **Activity feed**: steps, output, errors, feedback requests, mini-diffs and inline explanations of changes.
  A full diff editor. **Final summary**: changed files, runtime, lines added/changed, branch name,
  commit message. Publish branch / Publish PR buttons. There is pause.
  https://jules.google/docs/code/ , https://jules.google/docs/running-tasks/
- For frontend Jules sends a **screenshot**, and Playwright is in the base image (07.08.2025). Images
  render directly in the diff viewer (22.08.2025). Jules reacts to comments in the PR (23.09.2025) and
  fixes failing CI itself (19.02.2026). https://jules.google/docs/changelog/

### Replit Agent
- **Task system**: a board with the columns **Drafts / Active / Ready / Done**. Each proposed task has a
  title, description and a detailed plan via "**View plan**", which describes "what it will do and what
  "done" looks like". Buttons "Accept tasks" and "Revise plan". A finished task shows a **work log, test
  results and live preview**, then "Apply changes to main version" or "Dismiss".
  https://docs.replit.com/core-concepts/agent/task-system.md
- **App Testing**: the agent decides itself when to test. Testing runs in a real browser with a visible
  cursor, then the agent produces a summary and fixes what it found. After the run an **interactive video
  replay** with section navigation is available. "Begin take over" is used for login and CAPTCHA; if the human
  does not respond for 10 minutes, Skip fires. Works only for Full Stack JS and Streamlit.
  https://docs.replit.com/core-concepts/agent/app-testing.md
- **Checkpoints**: automatic snapshots of code, conversation context and the DB at milestones, rollback via the history view.
  https://docs.replit.com/core-concepts/agent/checkpoints-and-rollbacks

### Claude Code
- **Cloud (claude.ai/code)**: the `+42 -18` indicator opens a diff view with inline comments that
  go out with the next message. There is "Compare against" and **Create PR** (full, draft, or a compose page with
  ready title and description). A **CI status bar** with **Auto-fix**: the agent subscribes to PR events,
  fixes failed checks and review comments. If a comment is ambiguous, the agent asks the
  human. Replies on GitHub are marked as written by Claude Code. The session can be shared by link.
  https://code.claude.com/docs/en/claude-code-on-the-web
- **Desktop**: a Browser pane with a dev-server preview and **auto-verify**: the agent takes screenshots, inspects the
  DOM, clicks and fills in forms. A diff view with line comments, sent via Cmd+Enter. The
  **Review code** button looks only for high-signal problems. A CI status bar with Auto-fix and Auto-merge. Separate panes
  for plan and tasks. Permission modes Manual / Accept edits / Plan / Auto / Bypass.
  https://code.claude.com/docs/en/desktop
- **Artifacts** (claude.ai/code/artifact): a live HTML page from the session that updates in place and
  keeps versions. The documented scenario is "Walk a reviewer through a pull request with annotated
  diffs" or keeping an "investigation timeline" during a long task. The page is private until shared.
  https://code.claude.com/docs/en/artifacts
- **Code Review** (managed): several agents look for problems, a separate **verification step** checks
  the candidates against the real behavior of the code. Severity: 🔴 Important / 🟡 Nit / 🟣 Pre-existing. For each
  finding, "extended reasoning… how it verified the problem" expands. The check run "Claude Code Review"
  contains a table Severity | File:Line | Issue, annotations in Files changed and a machine-readable result
  (`bughunter-severity`). The conclusion is always **neutral**, meaning the PR is not blocked. Configured via
  `REVIEW.md` and `CLAUDE.md`. https://code.claude.com/docs/en/code-review
- **claude-code-action**: a tracking comment with checkboxes that update as work proceeds
  (`track_progress`), and `use_sticky_comment`.
  https://github.com/anthropics/claude-code-action/blob/main/README.md
- **Hooks** (Stop and others) allow running deterministic checks before finishing. Spotify
  uses this mechanism, see §7. https://code.claude.com/docs/en/hooks-guide

### Factory (Droids)
- **Specification Mode** (Shift+Tab): the Droid writes a spec, the human approves, the spec can be saved to the repo.
  https://docs.factory.ai/cli/user-guides/implementing-large-features
- **Droid Control**: `/verify` checks a behavior claim and delivers a verdict **CONFIRMED / REFUTED /
  INCONCLUSIVE** with evidence. `/demo` records a side-by-side video of the PR (before and after). `/qa-test` runs e2e.
  The result: a step-level pass/fail table with inline evidence, screenshots or text snapshots of the terminal,
  rendered videos. No date given. https://docs.factory.ai/software-factory/droid-control

### Amp (Sourcegraph)
- Review panel (25.10.2025): choosing a commit range, an **AI summary**, a "**tour**" with a recommended reading order of
  files, editable full-file diffs, a commit message.
  https://ampcode.com/news/review
- Agentic Review (18.12.2025): a summary per file and per changeset, a separate review agent with a list of
  actionable improvements that can be passed to the main agent. The Amp team itself calls an open
  question "How do reviews map to threads?". https://ampcode.com/news/agentic-code-review
- **Checks** (04.02.2026): `.agents/checks/*.md` — user invariants scoped by
  directory. For each check a **separate agent** is launched, "a stronger guarantee that each check will
  actually be checked". https://ampcode.com/news/liberating-code-review
- Diffs (16.06.2026): review of any thread's diff on desktop and mobile, duplicate block detection.
  https://ampcode.com/news/diffs

### Linear — Agent Interaction Guidelines and Agent Session
- **AIG** (30.07.2025), six principles: disclose agent identity; inhabit the platform natively; provide
  instant feedback; be clear and transparent about internal state (thinking / waiting / executing /
  complete); respect requests to disengage; "an agent cannot be held accountable" — responsibility
  stays with the human. https://linear.app/developers/aig
- **Agent Session**: the states `pending | active | error | awaitingInput | complete | stale` are derived
  automatically from the latest activity. Activity types: `thought`, `elicitation`, `action` (fields `action`,
  `parameter` and an optional `result`), `response`, `error`, and also `prompt` from the human. `thought` and
  `action` can be made **ephemeral**. **Agent Plan** is a session-level checklist, steps have `content` and
  a status `pending | inProgress | completed | canceled`, updated only as a whole. `externalUrls` —
  signed links to the agent's dashboard. The first activity must arrive within 10 seconds, the response to the
  webhook within 5 seconds. https://linear.app/developers/agent-interaction
- **Signals**: `stop` (human → agent: stop immediately and confirm this via response or
  error), `auth` (an elicitation with "Link account"), `select` (an elicitation with a list of options, free-text
  reply possible). https://linear.app/developers/agent-signals

---

## 7. In-house systems

- **Stripe Minions** (Part 1 and Part 2, February 2026; 1,000, then 1,300+ merged PRs per week): "completely
  minion-produced, human-reviewed". A PR follows the **Stripe PR template**. A local lint (<5 s) as a
  deterministic node of the blueprint works as shift-left. **No more than 2 CI rounds**: the first run,
  automatic application of autofixes, the second run, then handoff to a human even with unfixed failures. The **web UI**
  shows "the decisions and actions the minion took", and an engineer can give "further instructions" before
  review. https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents ,
  https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents-part-2
- **Spotify Honk** (Part 1–3, November–December 2025): deterministic **verifiers** (for example, Maven when
  `pom.xml` is present) return a compact success or failure result. They are available as a tool and
  **run automatically via a stop hook before a PR is opened**: on failure the PR is not opened. On top of
  them works an **LLM judge** (diff + the original prompt), which vetoes roughly 1 in 4 sessions, and about
  half of those get fixed. Evals for the judge have not been built yet. Traces are written to MLflow. PRs go through the usual
  Fleet Management without special formatting.
  https://engineering.atspotify.com/2025/12/feedback-loops-background-coding-agents-part-3 ,
  https://engineering.atspotify.com/2025/11/spotifys-background-coding-agent-part-1 .
  By QCon London (March 2026) the judge was **removed**, because verification steps in the prompt are enough. Also
  a PR inbox and auto-merge for docs appeared. This is a retelling of the talk on InfoQ, a **secondary source**.
  https://www.infoq.com/news/2026/03/spotify-honk-rewrite/
- **Shopify**: the security harness sends a **draft PR with a test proving exploitability**. The verifier
  runs on a different model than the Hunting agent (adversarial review). Unproven findings are rejected or
  downgraded. Credentials, Git and storage are handled by deterministic code.
  https://shopify.engineering/building-an-agentic-harness-that-outlasts-the-model (July 2026).
  **River**: works only in public Slack channels, intermediate findings are published in a thread, session
  logs are stored in Postgres. 3,536 merged PRs in 30 days. https://shopify.engineering/under-the-river (28.05.2026)
- **Uber**: uReview. Generation, then a confidence filter, dedup and suppression of categories based on feedback
  history. Useful / Not Useful buttons, useful >75%, 65% of comments are addressed.
  https://www.uber.com/us/en/blog/ureview/ (12.08.2025). Software Factory: >70% of PRs involve agents,
  quality metrics of managed agents (revert rate, F1, MTTR).
  https://www.uber.com/us/en/blog/efficient-software-factory/ (27.08.2026). The background platform "Minion" —
  **unconfirmed** by an Uber primary source (mentioned only by Pragmatic Engineer:
  https://newsletter.pragmaticengineer.com/p/how-uber-uses-ai-for-development).

---

## 8. Pattern catalog

| Pattern | What it contains | Products | What makes it trustworthy |
|---|---|---|---|
| **Plan / Spec** (before code) | goal, affected files, steps, how it will be verified, open questions | Antigravity Implementation Plan, Devin Interactive Planning, Cursor Plan Mode, Jules plan, Replit View plan, Factory Spec Mode, Claude Code Plan mode | an explicit gate (Proceed, Accept, Build); step-level comments; a plan critic (Jules Planning Critic); the presence of a verification section, that is, a "done" criterion declared in advance (Replit, Antigravity) |
| **Task list / Plan checklist** (live) | steps with statuses | Antigravity Task List, Linear Agent Plan (`pending/inProgress/completed/canceled`), Copilot PR checklist, claude-code-action tracking comment, Replit board | updated as things are actually done; statuses from a fixed set; `canceled` instead of a step silently vanishing |
| **Walkthrough / Summary** (after) | what changed, why, how to verify | Antigravity Walkthrough, Codex summary, Jules summary, Copilot PR body, Devin PR description, Amp AI summary and tour | **citations to logs and files** (Codex); a link to the session log (Devin "Link to Devin run", Copilot — in every commit); without such a binding it remains just model text |
| **Diff organized for reading** | reading order, grouping, move/copy detection, hunk explanation | Devin Review, Amp tour and Diffs, Codex "Last turn", Antigravity Review Changes | a deterministic diff plus model explanations; inline comments go back to the agent |
| **Check results with citations** | tests, linters, CI, security | Codex terminal citations, Copilot self-validation (CodeQL, secret scanning, deps), Stripe CI ≤2 rounds, Spotify verifiers, Replit test results, Bugbot and Claude Code Review check runs | the result is produced by execution, not by the model; a check run in CI; a verifier in a stop hook blocks the PR |
| **Evidence media** | screenshots, video, browser recordings | Antigravity screenshots and recordings, Devin test video and report, Cursor artifacts, Copilot screenshots (Playwright), Codex screenshots, Jules screenshots, Replit replay, Factory `/demo`, Claude Desktop auto-verify | captured from a real run; annotations and chapters; a **list of assertions passed / failed / untested** (Devin); side-by-side before and after (Factory) |
| **Verdict on a claim** | claim, verdict, evidence | Factory `/verify` CONFIRMED / REFUTED / INCONCLUSIVE, Claude Code Review "how it verified" | allows a negative and an inconclusive result; evidence attached |
| **Preview / live result** | a link to the running app or a remote desktop | Replit live preview, Cursor remote desktop takeover, Claude Desktop Browser pane, Codex in-app browser, Devin Browser tab | the human checks it personally and does not depend on the agent's account |
| **Session log / timeline** | all commands, edits, browser | Devin Progress (with time navigation), Copilot session log, Codex task logs, Jules activity feed, Stripe web UI, Linear activities, Shopify River Slack thread | the log is raw and immutable, reachable from the PR and commit |
| **Findings review** | findings with severity | Bugbot, Codex P0/P1, Devin Bug Catcher and Flags, Claude Code Review 🔴🟡🟣, uReview, Amp Checks | severity; a separate verification step; learning from reactions (Bugbot learned rules, uReview); a separate agent per invariant (Amp Checks); does not block merge without explicit configuration |
| **Open questions / Elicitation** | a question to the human, a choice of options | Linear `elicitation` + `select` and `auth`, Antigravity/Cursor clarifying questions, Claude auto-fix "asks you before acting", Devin asks for secrets | a structured question with options; the `awaitingInput` state is visible in the list |
| **Human checklist** | what the human must verify personally | Devin "Review & Testing Checklist for Human" (observed), GitHub guidance: a test that fails before the change, and a rollback plan | explicitly separates "verified by the agent" from "to be verified by the human" |
| **Approval gates** | who approves what | Antigravity Request Review, Copilot "Approve and run workflows" and the ban on self-approval, Replit Apply/Dismiss, Stripe and Spotify: only a human merges | a gate on the platform side, not in the prompt text |
| **Rollback / checkpoints** | return to a state | Replit checkpoints (code, context, DB), Codex revert per hunk, Antigravity undo | deterministic restoration |

General conclusions:
1. Trust rests on linking the model's text to execution artifacts: Codex citations, session links in
   commits at Copilot and Devin, check runs, video. A walkthrough without such links remains a "story".
2. Verification moves into infrastructure: stop-hook verifiers (Spotify), a CI round limit (Stripe),
   self-validation (Copilot), a verification step in review (Claude Code Review). Spotify removed the LLM judge
   as a separate layer in 2026 (per a secondary source).
3. Video proves a lot, but it is easy to abuse. Cognition acknowledges that an agent can set up
   state via JS instead of clicks. The best implementations add an assertions list and a test plan, that is,
   they show what exactly was checked.
4. Almost everywhere feedback goes through inline comments on the artifact (plan, diff, screenshot, page
   element), and the agent receives them on the next turn.
5. Unresolved questions acknowledged by the vendors themselves: how review relates to threads (Amp); the public
   nature of artifact URLs (Cursor); findings do not block merge (Claude Code Review, Bugbot `neutral`), and gating
   has to be built separately.
