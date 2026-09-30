# Issue tracker: GitHub

Issues and specs for this repo live as GitHub issues. Use the `gh` CLI for all operations. `gh` infers the repository from `git remote -v` when run inside a clone, and fills `{owner}/{repo}` in `gh api` paths the same way.

Pass multi-line text through a file (`--body-file`), not an inline string, so Markdown survives the shell.

## Issues

- **Create**: `gh issue create --title "<title>" --body-file <file> [--label "<label>"]`
- **Read** (body, labels, assignees and comments in one call): `gh issue view <n> --json number,title,state,body,labels,assignees,comments --jq '{number, title, state, body, labels: [.labels[].name], assignees: [.assignees[].login], comments: [.comments[] | {author: .author.login, body}]}'`. Do not combine `--comments` with `--json` or `--jq`; the `comments` field above already carries them.
- **List**: `gh issue list --state open --label "<label>" --json number,title,labels,assignees --jq '[.[] | {number, title, labels: [.labels[].name], assignees: [.assignees[].login]}]'`. Add `body` to `--json` only when you need it; it makes the output long.
- **Comment**: `gh issue comment <n> --body-file <file>`
- **Apply / remove labels**: `gh issue edit <n> --add-label "<label>"` / `gh issue edit <n> --remove-label "<label>"`
- **Claim**: `gh issue edit <n> --add-assignee @me`
- **Close**: `gh issue close <n> --comment "<why>"`

## Blocking edges

Used by `to-tickets`, `wayfinder`, `triage` and `coordinate`. A task is **blocked** while any issue it depends on is open; it is **unblocked** when every blocker is closed.

GitHub's native issue dependencies are the canonical form: they show in the issue's sidebar. The API takes the blocker's numeric **database id**, not its `#number` and not its `node_id`.

- **Database id of an issue**: `gh api repos/{owner}/{repo}/issues/<n> --jq .id`
- **Add an edge** (`<child>` is blocked by `<blocker>`): `gh api --method POST repos/{owner}/{repo}/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-id>`
- **Remove an edge**: `gh api --method DELETE repos/{owner}/{repo}/issues/<child>/dependencies/blocked_by/<blocker-id>`
- **List blockers** (open and closed): `gh api repos/{owner}/{repo}/issues/<n>/dependencies/blocked_by --jq '[.[] | {number, state, title}]'`
- **List what an issue blocks**: `gh api repos/{owner}/{repo}/issues/<n>/dependencies/blocking --jq '[.[] | {number, state, title}]'`
- **Open blockers count** (the live gate): `gh api repos/{owner}/{repo}/issues/<n> --jq .issue_dependencies_summary.blocked_by`. Zero means unblocked. `issue_dependencies_summary` is a field of the REST issue object; it is not a `gh issue view --json` field.

Where dependencies are not available, put a `Blocked by: #<n>, #<n>` line at the top of the blocked issue's body; it is unblocked when every listed issue is closed.

## Parent and child issues

GitHub sub-issues link a spec or map to its tickets. Like dependencies, they take the child's database id.

- **Add a child**: `gh api --method POST repos/{owner}/{repo}/issues/<parent>/sub_issues -F sub_issue_id=<child-id>`
- **Remove a child**: `gh api --method DELETE repos/{owner}/{repo}/issues/<parent>/sub_issue -F sub_issue_id=<child-id>`
- **List children**: `gh api --paginate repos/{owner}/{repo}/issues/<parent>/sub_issues --jq '.[] | {number, state, title}'`. Without `--paginate` only the first 30 come back.
- **Progress**: `gh api repos/{owner}/{repo}/issues/<parent> --jq .sub_issues_summary` gives `completed`, `total` and `percent_completed`.
- **Parent of an issue**: `gh api repos/{owner}/{repo}/issues/<n>/parent --jq '{number, title}'`

Where sub-issues are not enabled, list the children as a task list in the parent's body and put `Part of #<parent>` at the top of each child.

## Pull requests

Used by `pr`, `code-review` and `coordinate`.

- **Open**: `gh pr create --base <base> --title "<title>" --body-file <file>` (add `--draft` for work in progress). Put `Closes #<issue>` in the body so the merge closes the task.
- **Update the body**: `gh pr edit <n> --body-file <file>`
- **Read**: `gh pr view <n> --json number,url,state,isDraft,headRefName,headRefOid,reviewDecision,statusCheckRollup,closingIssuesReferences,body`
- **Checks on the head**: `gh pr checks <n>` (exits non-zero while a check fails or is pending)
- **List open**: `gh pr list --state open --json number,title,headRefName,isDraft,reviewDecision,statusCheckRollup,closingIssuesReferences --jq '[.[] | {number, title, branch: .headRefName, draft: .isDraft, review: .reviewDecision, checks: ([.statusCheckRollup[]? | .conclusion // .state] | unique), closes: [.closingIssuesReferences[].number]}]'`
- **Diff**: `gh pr diff <n>`
- **Comment**: `gh pr comment <n> --body-file <file>`
- **Merge** (only under `acceptance:auto`, see the `pr` skill): `gh pr merge <n> --squash` (or the merge method the project uses)

## Pull requests as a triage surface

**PRs as a request surface: no.** _(Set to `yes` if this repo treats external PRs as feature requests; the `triage` skill reads this flag.)_

When set to `yes`, PRs run through the same labels and states as issues, using the `gh pr` equivalents:

- **Read a PR**: the **Read** command under Pull requests, plus `gh pr diff <n>`.
- **List external PRs for triage**: `gh pr list --state open --json number,title,body,labels,author,authorAssociation --jq '[.[] | select(.authorAssociation == "CONTRIBUTOR" or .authorAssociation == "FIRST_TIME_CONTRIBUTOR" or .authorAssociation == "NONE")]'`
- **Comment / label / close**: `gh pr comment`, `gh pr edit --add-label` / `--remove-label`, `gh pr close`.

GitHub shares one number space across issues and PRs, so a bare `#42` may be either — resolve with `gh pr view 42` and fall back to `gh issue view 42`.

## When a skill says "publish to the issue tracker"

Create a GitHub issue.

## When a skill says "fetch the relevant ticket"

Use the **Read** command under Issues: it returns the body and every comment.

## Ready work

Open, unblocked tasks whose intent is approved: `gh api --paginate "repos/{owner}/{repo}/issues?state=open&labels=<ready-for-agent label>" --jq '.[] | select(.pull_request == null) | {number, title, blocked_by: .issue_dependencies_summary.blocked_by, assignees: [.assignees[].login], labels: [.labels[].name]}'`. A task is ready when `blocked_by` is 0 and nobody is assigned.

## Wayfinding operations

Used by `wayfinder`. The **map** is a single issue with **child** issues as tickets.

- **Map**: a single issue labelled `wayfinder:map`, holding the Notes / Decisions-so-far / Fog body. `gh issue create --label wayfinder:map --title "<title>" --body-file <file>`.
- **Child ticket**: an issue linked to the map as a sub-issue (see Parent and child issues). Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Once claimed, the ticket is assigned to the driving dev.
- **Blocking**: native dependencies, as in Blocking edges.
- **Frontier query**: the map's open, unblocked, unclaimed children, in map order: `gh api --paginate repos/{owner}/{repo}/issues/<map>/sub_issues --jq '.[] | select(.state == "open" and .issue_dependencies_summary.blocked_by == 0 and (.assignees | length) == 0) | {number, title}'`. The first one wins.
- **Claim**: `gh issue edit <n> --add-assignee @me` — the session's first write.
- **Resolve**: `gh issue comment <n> --body-file <file>` with the answer, then `gh issue close <n>`, then append a context pointer (gist + link) to the map's Decisions-so-far.
