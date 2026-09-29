# Human task lists beside agents' trackers

Date: 2026-09-29. Question from #57 (part of the command center map #41): how developers keep a
**human task list** apart from the **tracker agents work in**, and launch agent work from a human
task. The owner's frame: per project, an agents' tracker that follows the harness pipeline (GitHub
Issues, decision G28), a separate list of human tasks, possibly one personal board across all
projects, and "start agent work from this human task" in one step.

Already known and not repeated here: the Herdr plugin landscape and non-Herdr bricks
([command-centers-landscape](2026-09-29-command-centers-landscape.md) §6a–6b), GitHub as a data
source and rate limits, and gh-dash as a UI option
([command-center-build-options](2026-09-29-command-center-build-options.md) §1.4, §2(a)).

Method.

- **GitHub.** docs.github.com pages, the GitHub changelog and the `cli/cli` release notes, fetched
  on 2026-09-29. Some doc pages were read through a fetch-and-summarise tool, so quotes are short and
  may be paraphrased; the page URL is given for each claim. Local `gh` help output (installed
  version 2.88.1; latest release v2.101.0 of 2026-09-15 per `gh api repos/cli/cli/releases/latest`).
  Three behaviours were checked against this repository with read-only `gh api` calls (§1.4, §1.5).
- **Tools.** Repository stats from `gh api repos/<owner>/<repo>` (stars, `pushed_at`, SPDX license)
  on 2026-09-29. Mechanics from each tool's README, docs site or source on the default branch read on
  that date. Nothing was installed or run.
- **SaaS.** Official docs and changelogs of Linear, Todoist and Things. Where only a search-result
  summary of an official page was available, the claim is marked **secondary**. User write-ups are
  marked **secondary**; anything not verified is marked **unconfirmed**.

Citation shorthand: `<owner>/<repo>:<path>` means
`https://github.com/<owner>/<repo>/blob/<default branch>/<path>` as read on 2026-09-29.

## 1. GitHub options

### 1.1 Projects v2 as a personal board

- A project is "an adaptable table, board, and roadmap" created "at the user or organization
  level" ([about projects](https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/about-projects)).
- A project can hold issues and pull requests "from any organization" plus draft issues, so a
  user-owned project can collect issues from repositories of several owners
  ([adding items](https://docs.github.com/en/issues/planning-and-tracking-with-projects/managing-items-in-your-project/adding-items-to-your-project)).
- Limit: 50,000 items per project (active and archived); the earlier limit was 1,200
  ([changelog 2025-04-09](https://github.blog/changelog/2025-04-09-evolving-github-issues-and-projects/)).
- Fields: date, number, single select, text, iteration; up to 50 fields. Layouts: table, board,
  roadmap ([about projects](https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/about-projects)).
- Built-in workflows: item added → Status Todo, issue or PR closed → Done, PR merged → Done,
  auto-archive, auto-add by filter
  ([built-in automations](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-built-in-automations)).
  Each auto-add workflow targets one repository; N repositories need N workflows or adds by hand or
  API ([adding items automatically](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/adding-items-automatically)).
- "Parent issue" and "Sub-issue progress" fields group and filter by parent
  ([parent and sub-issue fields](https://docs.github.com/en/issues/planning-and-tracking-with-projects/understanding-fields/about-parent-issue-and-sub-issue-progress-fields));
  the hierarchy view in Projects went GA on 2026-03-19
  ([changelog](https://github.blog/changelog/2026-03-19-hierarchy-view-in-github-projects-is-now-generally-available/)).
- Visibility: a project is public or private; "Only people with access to a private repository will
  be able to view project items from that private repository"
  ([visibility](https://docs.github.com/en/issues/planning-and-tracking-with-projects/managing-your-project/managing-visibility-of-your-projects)).
- Webhooks `projects_v2` and `projects_v2_item` are listed for organizations only
  ([webhook events](https://docs.github.com/en/webhooks/webhook-events-and-payloads)); a user-owned
  project has no webhook, so a reader polls (inference).

### 1.2 Draft issues

- "Draft issues exist only in your project". They have a title, body, assignees and project fields;
  repository, labels and milestone need conversion to an issue, which asks for a destination
  repository ([adding items](https://docs.github.com/en/issues/planning-and-tracking-with-projects/managing-items-in-your-project/adding-items-to-your-project),
  [converting drafts](https://docs.github.com/en/issues/planning-and-tracking-with-projects/managing-items-in-your-project/converting-draft-issues-to-issues)).
- The GraphQL schema has `addProjectV2DraftIssue`, `updateProjectV2DraftIssue` and
  `convertProjectV2DraftIssueItemToIssue`; `ProjectV2ItemType` is `ISSUE`, `PULL_REQUEST`,
  `DRAFT_ISSUE` or `REDACTED` ([projects API](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)).
- Whether a draft can be a parent or sub-issue: **unconfirmed**. The sub-issue API takes issue ids
  and a draft has no repository (§1.5), so it looks impossible, but no page says so.
- Whether a Copilot or third-party agent can be assigned to a draft: **unconfirmed**; assignment is
  documented for repository issues (§1.8).

### 1.3 `gh project` and the API

- `gh project` needs the `project` token scope and offers `list`, `view`, `field-list`,
  `field-create`, `item-list`, `item-add`, `item-create` (draft), `item-edit`, `item-archive`,
  `item-delete`, `link` and others (local `gh project --help`;
  [manual](https://cli.github.com/manual/gh_project)).
- `item-list` has `--format json`, `--jq`, `--limit` (default 30) and `--query` in the Projects
  filter syntax, for example `"assignee:@me -status:Done"` (local `gh project item-list --help`).
- `item-edit` sets one field value per call by field id, and edits a draft's title and body
  ([manual](https://cli.github.com/manual/gh_project_item-edit)).
- `updateProjectV2ItemFieldValue` covers text, number, date, single select and iteration only;
  assignees, labels and milestone change on the issue itself; "You cannot add and update an item in
  the same call" ([projects API](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)).
- `gh search issues` filters by `--owner`, `--assignee`, `--label` and `--project owner/number`
  (local `gh search issues --help`).

### 1.4 Issue types

- Issue types are managed per organization, up to 25, defaults task, bug and feature
  ([managing issue types](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/managing-issue-types-in-an-organization));
  GA on 2025-04-09 ([changelog](https://github.blog/changelog/2025-04-09-evolving-github-issues-and-projects/)).
- For this user-owned repository, GraphQL `repository.issueTypes` returns `null` (checked
  2026-09-29). A GitHub staff answer says types are organization-only (**secondary**,
  [community discussion](https://github.com/orgs/community/discussions/175785)). So a type split
  between human and agent tasks needs an organization-owned repository.
- `gh` gained `--type` on `issue create/edit` and type filtering in v2.94.0
  ([release notes](https://github.com/cli/cli/releases/tag/v2.94.0)).

### 1.5 Sub-issues and dependencies

- Up to 100 sub-issues per parent and eight levels of nesting
  ([adding sub-issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/adding-sub-issues));
  GA on 2025-04-09.
- Cross-repository is allowed within one owner: "the sub-issue must belong to the same repository
  owner as the parent issue" ([REST sub-issues](https://docs.github.com/en/rest/issues/sub-issues)).
  A parent in a user's repository cannot have sub-issues in an organization's repository.
- Sub-issues work on user-owned repositories: in this repository #57 has parent #41 and #41 has 14
  sub-issues (`gh api .../issues/57/parent`, `.../issues/41/sub_issues`, 2026-09-29). This
  contradicts the community thread in §1.4 for sub-issues.
- Whether a private-repository parent may hold a public-repository sub-issue (or the reverse), and
  what a viewer without access sees: **unconfirmed**.
- "Blocked by" / "blocking" dependencies are GA with up to 50 issues per relationship type
  ([changelog 2025-08-21](https://github.blog/changelog/2025-08-21-dependencies-on-issues/));
  cross-owner behaviour **unconfirmed**.
- `gh` v2.94.0 added `--parent`, `--add-sub-issue`, `--blocked-by`, `--blocking` and JSON fields for
  them ([release notes](https://github.com/cli/cli/releases/tag/v2.94.0);
  [changelog 2026-06-10](https://github.blog/changelog/2026-06-10-manage-sub-issues-types-and-dependencies-from-github-cli/)).

### 1.6 Other links between a task and its agent work

- Closing keywords (`Fixes owner/repo#100`) link a PR to an issue across repositories and close it
  when the PR merges into the default branch; the Development sidebar links branches and PRs by hand
  ([linking a PR](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue)).
- `gh issue develop` creates a linked branch, optionally in another repository (`--branch-repo`)
  ([manual](https://cli.github.com/manual/gh_issue_develop)).
- Tasklist blocks were retired in favour of sub-issues (**unconfirmed**: seen only in a search
  summary of a changelog entry).
- A plain mention (`owner/repo#N`) creates a cross-reference in the timeline of the mentioned issue
  (general GitHub behaviour; no page fetched, **unconfirmed** here).

### 1.7 Separate repository or a split in one repository

- GitHub publishes no guidance on a separate repository versus a label or type split (none found).
- Transfer between repositories works only "between repositories owned by the same user or
  organization account", and a private-repository issue cannot move to a public repository
  ([transferring an issue](https://docs.github.com/en/issues/tracking-your-work-with-issues/administering-issues/transferring-an-issue-to-another-repository)).
- In a public repository every issue is public, so personal human tasks there are public too
  (general GitHub behaviour, **unconfirmed** against a page).

### 1.8 Launching agent work on GitHub

- Assigning an issue to Copilot sends "the issue title, description, any comments that currently
  exist, and any additional instructions you provide"; later comments are not seen, follow-ups go on
  its PR ([use cloud agent](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/cloud-agent/use-cloud-agent-on-github)).
- Claude and Codex became assignable through the Assignees menu in public preview on 2026-02-04
  ([changelog](https://github.blog/changelog/2026-02-04-claude-and-codex-are-now-available-in-public-preview-on-github/));
  current plans and models are listed in
  [third-party agents](https://docs.github.com/en/copilot/concepts/agents/about-third-party-agents).
  These run in GitHub's cloud, not in the owner's terminal. Assignment from a Projects board item:
  **unconfirmed**.
- `anthropics/claude-code-action` (MIT, 9,237★) starts on an `@claude` comment, an assignee or a
  label (`assignee_trigger`, `label_trigger`) in GitHub Actions
  (`anthropics/claude-code-action:docs/usage.md`); the exact issue context it passes is
  **unconfirmed**. The opencode GitHub agent (`/opencode` in a comment) receives the title, body and
  all comments ([docs](https://opencode.ai/docs/github/)).

## 2. Other trackers used for the human side

### 2.1 File-based and terminal trackers

| Tool | ★ / last push / license | Storage | Human UI | Agent interface | Scope |
|---|---|---|---|---|---|
| [smarzban/tsk](https://github.com/smarzban/tsk) | 153 / 2026-09-29 / MIT | one JSON file per user (`~/.tsk/tsk.json`) | TUI, Herdr keys | CLI with `--json`, `tsk guide`, installed skill | per-repo projects plus a cross-project "desk" |
| [MrLesk/Backlog.md](https://github.com/MrLesk/Backlog.md) | 6,892 / 2026-09-28 / MIT | one Markdown file per task in `backlog/` | CLI, TUI board, local web kanban | MCP, `--json`, `backlog instructions` | one repository |
| [steveyegge/beads](https://github.com/steveyegge/beads) | 27,512 / 2026-09-29 / MIT | Dolt database in `.beads/`, JSONL export | CLI; TUI and web are community | `--json`, MCP, `bd setup claude\|codex` | one repository; separate planning repo possible |
| [hmans/beans](https://github.com/hmans/beans) | 950 / 2026-04-06 / Apache-2.0 | Markdown files in `.beans/` | CLI, TUI | GraphQL, agent hooks | one repository |
| [wedow/ticket](https://github.com/wedow/ticket) | 904 / 2026-03-16 / MIT | Markdown with frontmatter in `.tickets/` | bash CLI | a note in `AGENTS.md` | one repository |
| [tasksmd/tasks.md](https://github.com/tasksmd/tasks.md) | 11 / 2026-09-29 / MIT | one `TASKS.md` | CLI | MCP server | one repository; can sync from trackers |
| [todotxt/todo.txt-cli](https://github.com/todotxt/todo.txt-cli) | 6,180 / 2026-09-06 / GPL-3.0 | one line per task ([format](https://github.com/todotxt/todo.txt)) | shell CLI | plain text only | per user |
| [obsidian-tasks](https://github.com/obsidian-tasks-group/obsidian-tasks) | 4,034 / 2026-09-25 / MIT | checkbox lines in vault notes | Obsidian | official Obsidian CLI `tasks format=json` ([help](https://obsidian.md/help/cli)) | per vault |
| [taskwarrior](https://github.com/GothenburgBitFactory/taskwarrior) + [bugwarrior](https://github.com/GothenburgBitFactory/bugwarrior) | 6,089 + 820 | local database; bugwarrior pulls GitHub, Linear, Todoist issues one way | CLI | none stated | per user |
| [naggie/dstask](https://github.com/naggie/dstask) | 1,206 / 2026-05-11 / MIT | Markdown note per task, git-synced | CLI | none stated | per user; imports GitHub issues |
| [git-bug/git-bug](https://github.com/git-bug/git-bug) | 10,670 / 2026-09-29 / GPL-3.0 | git objects, no files in the tree | CLI, TUI, web | GraphQL | one repository; GitHub bridge |
| [eyaltoledano/claude-task-master](https://github.com/eyaltoledano/claude-task-master) | 28,111 / 2026-04-28 / MIT + Commons Clause | `.taskmaster/tasks.json` | CLI | MCP | agent-first, one repository |

Notes, each from the tool's README unless linked otherwise:

- tsk calls itself "a terminal task board for you and your agents: one shared queue, a TUI for you,
  a CLI for them". Hand-off today is copying the task number into an agent conversation; "assign a
  task to an agent, start implementation from tsk, and run the work in Herdr worktrees" is listed
  under "Coming soon". Statuses are open, ready, started, blocked, review, done; no GitHub sync is
  documented ([CLI docs](https://gettsk.sh/docs/cli/), [storage](https://gettsk.sh/docs/storage/)).
- Backlog.md is "for managing project collaboration between humans and AI Agents in a git
  ecosystem"; no GitHub sync is documented.
- beads is "a memory upgrade for your coding agent", i.e. agent-first rather than a human list.
- Obsidian's official CLI became generally available in 1.12.4 on 2026-02-27 and needs the app
  running ([changelog](https://obsidian.md/changelog/2026-02-27-desktop-v1.12.4/),
  [help](https://obsidian.md/help/cli)).
- A developer's account of sharing one `todo.txt` between themselves and agents instead of a SaaS
  board (**secondary**,
  [dev.to](https://dev.to/hideyukimori/a-todotxt-shared-by-a-human-and-ai-agents-why-plain-text-beat-a-saas-board-for-my-workflow-5dbh)).

### 2.2 SaaS with integrations

- **Linear.** Official MCP server at `https://mcp.linear.app/mcp`, with a read-only endpoint
  ([docs](https://linear.app/docs/mcp)). GitHub integration syncs issues one or two ways and links
  PRs by branch name or magic words (**secondary**, search summary of
  [github-integration](https://linear.app/docs/github-integration)). No official CLI; the community
  [schpet/linear-cli](https://github.com/schpet/linear-cli) (978★, ISC, pushed 2026-09-23) is
  "agent friendly". Delegating an issue to an agent creates an `AgentSession`; the `created` event
  carries `promptContext` ("issue details, comments, and guidance"), `previousComments` and
  `guidance` configured per workspace or team ([agent interaction](https://linear.app/developers/agent-interaction)).
  Since 2026-02-26 an issue can be opened in Claude Code, Codex, Conductor, Cursor, Copilot,
  OpenCode, Replit, v0 or Zed with a prefilled prompt holding "the issue ID and all relevant context:
  description, comments, updates, linked references, and images", from team-editable templates
  ([changelog](https://linear.app/changelog/2026-02-26-deeplink-to-ai-coding-tools)).
- **Todoist.** Official MCP server ([Doist/todoist-ai](https://github.com/Doist/todoist-ai)),
  official CLI `td` with `--json` and installable agent skills
  ([Doist/todoist-cli](https://github.com/Doist/todoist-cli)), REST API v1
  ([developer docs](https://developer.todoist.com/api/v1/)). No official GitHub integration found;
  no launch-an-agent feature found.
- **Things 3.** Official URL scheme (`add`, `update` with a token, `show`, `search`, `json`) and
  AppleScript on macOS ([URL scheme](https://culturedcode.com/things/support/articles/2803573/),
  [AppleScript](https://culturedcode.com/things/support/articles/4562654/)). MCP servers and GitHub
  links are community-only (for example
  [rossshannon/Things3-MCP](https://github.com/rossshannon/Things3-MCP), **secondary**).

## 3. What "start this task" does

| Tool | ★ / push / license | Launch mechanics | Context the agent gets | Where state lives | Write-back to the tracker |
|---|---|---|---|---|---|
| [bredebjorhovd/herdr-board](https://github.com/bredebjorhovd/herdr-board) | 1 / 2026-08-14 / none | route by repo or label → `herdr` worktree → pane → agent → first prompt (`src/dispatch.rs`) | template vars title, identifier, body, url, branch, workspace, worktree; default "You are working on: {title} ({identifier}) … Open a pull request when done." No comments or labels | SQLite plus live attempt; row state "derived on every read from upstream state plus the live attempt" | comments on dispatch and outcome, closes on done; `[github] writeback = false` turns it off |
| [nelsonPires5/herdr-board](https://github.com/nelsonPires5/herdr-board) | 162 / 2026-09-29 / none | card moved to an automatic column runs an agent in a `card-<id>` tab | card title and prompt, column system prompt, `BOARD_*` env | own SQLite under the user's data directory | none; not a GitHub tracker |
| [eliasstravik/herdr-projects](https://github.com/eliasstravik/herdr-projects) | 517 / 2026-09-28 / MIT | coordinator agent starts a thread agent per task in its own worktree | "a brief with the project's goal, your standing instructions, the project's memory and its task" | Markdown and TOML folders per project | follows PRs; merged PR resolves the thread |
| [a2u/herdr-jira](https://github.com/a2u/herdr-jira) | 13 | `d` delegates a Jira issue to a running or new agent via `herdr agent prompt` | template `{key} {summary} {description} {url} {status} {assignee}` | Jira | can change Jira status |
| [talent-factory/herdr-linear](https://github.com/talent-factory/herdr-linear) | 5 | Enter starts an agent in a new tab and injects an implement prompt | the Linear issue | Linear | sets "In Progress", composes comments |
| [dlvhdr/gh-dash](https://github.com/dlvhdr/gh-dash) | 12,572 / 2026-09-22 / MIT | a key runs a user-written shell command; nothing is launched by default | issue templates `RepoName`, `RepoPath`, `IssueNumber`, `IssueTitle`, `Author` (`internal/tui/modelUtils.go`); body and comments must be fetched by the command | none; reads GitHub live | only what the command does |
| Copilot / Claude / Codex on GitHub | n/a | assign the issue; cloud agent opens a branch and PR | title, description, existing comments, extra instructions (§1.8) | GitHub | its own PR |
| [Untrivial-ai/agent-orchestrator](https://github.com/Untrivial-ai/agent-orchestrator) (ex-ComposioHQ, redirect **unconfirmed**) | 12,526 / 2026-09-29 / Apache-2.0 | worker session in its own branch and worktree | tracker `Get` "used by spawn-bootstrap to hydrate the agent prompt" (`backend/internal/ports/tracker.go`) | local daemon with SQLite; card position derived from session, PR, CI and review facts | none: GitHub adapter is read-only in v1 (`backend/internal/adapters/tracker/github/doc.go`) |
| [BloopAI/vibe-kanban](https://github.com/BloopAI/vibe-kanban) | 28,220 / 2026-09-19 / Apache-2.0 | workspace = branch, terminal, dev server for an agent | **unconfirmed** | local; cloud shut down 30 days after 2026-04-10 ([shutdown](https://www.vibekanban.com/blog/shutdown)) | PRs; issue sync **unconfirmed** |
| [generalaction/emdash](https://github.com/generalaction/emdash) | 5,879 / 2026-09-29 / Apache-2.0 | worktree and branch per agent; issues from GitHub, Linear, Jira and others "sent into an agent" | **unconfirmed** | local SQLite | **unconfirmed** |
| Linear agents and deeplinks | n/a | delegate (webhook to a hosted agent) or deeplink (opens a local tool) | `promptContext`; deeplink prompt with description, comments, updates, links, images (§2.2) | Linear | agent activities in Linear |

The common shape: create a worktree and branch, start an agent CLI in a pane, send a prompt built
from the task's title and body (Copilot, Linear and opencode add comments; herdr-board does not).
Write-back ranges from none (Agent Orchestrator v1, gh-dash) to comment-and-close (herdr-board).

## 4. Cross-project personal boards

- **GitHub-native.** A user-owned project can hold issues from many repositories and owners (§1.1);
  auto-add is one repository per workflow. The Issues dashboard lists issues you created, are
  assigned or mentioned in across repositories, with up to 25 saved views from search filters
  ([viewing all issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/viewing-all-of-your-issues-and-pull-requests)).
  `gh status` summarises assigned issues and PRs, review requests and mentions across subscribed
  repositories (local `gh status --help`).
- **gh-dash.** Sections are search filters, so `user:` or `org:` gives a multi-repository view and
  `repoPaths` maps repositories to checkouts
  (`dlvhdr/gh-dash:docs/src/content/docs/configuration/searching.mdx`, `repo-paths.mdx`). Projects
  v2 items are not shown: [#915](https://github.com/dlvhdr/gh-dash/issues/915) is open (support
  **unconfirmed**).
- **Derived boards.** bredebjorhovd/herdr-board shows issues of many repositories on one board and
  keeps no status of its own beyond the live attempt; Agent Orchestrator derives card positions from
  facts; captains-deck is read-only over live agent state
  (`deimantasnork/captains-deck:README.md`); herdr-radar groups agents by project and "writes only
  the display tokens the sidebar shows" (`hhdebb/herdr-radar:README.md`); herdr-pr-board is a
  cross-repository PR view from `gh` searches (`cdowell09/herdr-pr-board:README.md`).
- **Boards with their own store.** tsk (desk plus projects in one JSON file), nelsonPires5/herdr-board,
  herdr-projects, emdash and vibe-kanban keep task state outside the forge. They are a second source
  of truth for any task that also exists as an issue.

## 5. Comparison of options for the human list

| Option | Personal board across projects | Private | Links to agents' issues | Terminal and agent access | Second store |
|---|---|---|---|---|---|
| A. Draft items in a private user-owned project | yes, drafts and issues from any owner side by side | yes (project private) | drafts cannot be parents (**unconfirmed**); link by a project field, a mention in the body, or convert to an issue | `gh project item-list --format json`, `item-create`; no Projects support in gh-dash | no, GitHub |
| B. A private human-tasks repository per owner | via a user project or search | yes | sub-issues and closing keywords across repositories of the same owner only | `gh issue`, gh-dash sections | no, GitHub |
| C. Label or type split in the agents' repository | via search or a project | no in a public repository | sub-issues, same repository | as B | no, GitHub; types need an organization |
| D. Terminal tracker (tsk, Backlog.md, todo.txt, beans) | tsk has a desk; others are per repository or per user | yes (local) | none built in; by reference in text | TUI for the human, CLI or MCP for agents | yes |
| E. Linear, Todoist or Things | yes | yes | Linear syncs with GitHub Issues (**secondary**); others none | Linear MCP and deeplinks; Todoist CLI and MCP; Things URL scheme | yes |

## 6. Implications for the command center

Facts and options only; decisions are the owner's.

1. **GitHub can hold both lists without a second store.** A private user-owned project with draft
   items is a human list, and the same project can show the agents' issues from public repositories
   (§1.1, §1.2). This keeps the brief's "no tracker of our own" and G28's "facts from GitHub only".
   The costs: drafts carry no labels and cannot be parents, user projects send no webhooks (polling
   only), and `gh project` edits fields one at a time.
2. **Linking a human task to agent work depends on ownership.** Sub-issues and issue transfer work
   only within one owner (§1.5, §1.7). If the human task is a repository issue under the same owner
   as the agents' repository, "spawned" agent issues can be its sub-issues and the command center
   reads the edge from GitHub. Across owners, or from a draft, the link has to be a mention or a
   project field, which the command center would parse by convention.
3. **A type split needs an organization.** Issue types are unavailable in user-owned repositories
   (§1.4); in one public repository, labels are the only split and the human tasks are public.
4. **Terminal-first human trackers exist but add a store.** tsk fits Herdr closely (a board key, a
   cross-project desk, a CLI for agents) and plans agent launch, but keeps tasks in a local JSON file
   with no GitHub sync (§2.1). Using it means the command center either reads a second source or
   ignores human tasks.
5. **"Start this task" is the same five steps everywhere** (§3): worktree, branch, agent, prompt from
   title and body, optional write-back. Options that stay within A4 and the brief's exclusion of
   launching agents:
   - **a.** hand-off only: copy a task reference or prompt for the owner to paste (tsk today);
   - **b.** a user-configured command bound to a key, with the task's fields as template variables
     (gh-dash), so the command center runs no agent logic of its own;
   - **c.** hand the task to Herdr or a Herdr plugin (bredebjorhovd/herdr-board dispatch, herdr-jira
     and herdr-linear via `herdr agent prompt`);
   - **d.** GitHub assignment to Copilot, Claude or Codex, which runs in the cloud, not in Herdr;
   - **e.** Linear-style deeplink: open a runtime with a prefilled prompt from a template.
   Where the command center's thin action ends and "launching agents" begins is the open question
   already listed on #41.
6. **Derived state is proven in practice.** herdr-board and Agent Orchestrator derive card positions
   from the tracker, PRs and live sessions rather than storing them (§3, §4), which matches the
   brief's principle that the command center keeps no statuses of its own.
7. **The context an agent receives varies.** Title and body is the floor; Copilot, opencode and
   Linear add comments, and Linear adds team guidance through templates. Under G28 the harness's
   skills already read the issue through `docs/agents/issue-tracker.md`, so an issue number may be
   enough context for a harness-aware agent (inference).

## Limits

- Draft-issue parenting, agent assignment from Projects items or drafts, private-parent/public-child
  sub-issues and cross-owner dependencies are unconfirmed.
- Some doc pages were read through a summarising fetch tool; Linear's GitHub integration details and
  the Copilot context quote come partly from search summaries.
- Star counts and push dates are a snapshot of 2026-09-29; several tools are weeks old and have one
  author.
- Conductor, superset's prompt contents, vibe-kanban's context and emdash's write-back were not
  verified.
