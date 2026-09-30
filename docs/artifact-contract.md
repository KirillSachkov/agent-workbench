# Artifact contract

The documented shapes of the artifacts the pipeline produces. Skills follow them and people and
reviewers judge them; the harness's code checks none of them (ADR 0004). Any method whose
artifacts follow these shapes is acceptable (ADR 0001). Each shape's worked template lives in the
skill that writes it; this page is the index and the reference for tools that read the artifacts.

The command center reads some of these shapes to show a project at its highest **detail level**:
"a repository whose harness follows the documented conventions" (ADR 0006). Those parts are marked
**read by tools**; keep them stable, and change them only together with their readers.

## Task intent (small track)

Where: the task's issue. Written by `triage` (the agent brief) or by a person.

- **Intent** — what should happen after the work, from the user's side.
- **Preservation criteria** — behaviour that must keep working.
- **Acceptance criteria** — observable conditions, each checkable on its own.
- **Out of scope.**

Intent approval is the owner's `ready-for-agent` label. Template: `skills/triage/AGENT-BRIEF.md`.

## Spec (medium track)

Where: an issue, or a file approved through its pull request. Written by `to-spec`.

Problem, solution, user stories, acceptance criteria, preservation criteria, implementation
decisions, testing decisions, out of scope. Template: `skills/to-spec/SKILL.md`.

## Ticket

Where: one issue per ticket, linked to its spec as a child, with blocking edges to the tickets it
waits for. Written by `to-tickets`. Self-contained: what to build, acceptance criteria,
preservation criteria, blocked by. Template: `skills/to-tickets/SKILL.md`; tracker operations in
`docs/agents/issue-tracker.md`, section "Blocking edges".

## Labels (read by tools)

Triage roles `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`;
acceptance modes `acceptance:human` (the default, also when no label is set) and
`acceptance:auto`; wayfinder maps and tickets `wayfinder:map`, `wayfinder:<type>`. A project may
map the roles to other strings in `docs/agents/triage-labels.md` and `[tracker.labels]` of
`workbench.toml`. Template: `skills/setup-harness/triage-labels.md`.

## Result card (read by tools)

Where: the body of the pull request that delivers a task. Written by `pr`, which `implement` calls
after `code-review`. Template and rules: `skills/pr/SKILL.md`.

The command center reads four sections by heading, at any heading level, case-insensitive, with an
optional trailing colon:

| Heading | Content | How it is read |
|---|---|---|
| `Result` | the result in one line | shown as the card's title line |
| `Needs you` | decisions and actions only the owner can take | shown under "needs you" |
| `Look first` | one list item per place: `path:line — reason` | each line becomes a key that opens the file at the line |
| `How to try` | a command, and the app's address when there is one | the first `http(s)` address is the app |

`Look first` lines: a list item (`-`, `*`, `+` or `1.`), the path optionally in backticks, a line
number, then `—`, `–`, `-` or `--` and the reason. Paths are relative to the repository root and
stay inside it; other lines are ignored. A body without any of the four headings is shown whole.

The other sections are for people and are shown as they are: `Criteria` (every acceptance and
preservation criterion **verified**, **failed** or **unverified**, with evidence records),
`Changes to tests and CI`, `Not done`, and the optional `What changed`. The body links its task
(`Closes #<n>` on GitHub).

### Evidence records

One record supports one criterion: `` `<type>` @ `<short sha>` — <reference>: <observation> ``.
Types: `ci`, `command`, `test`, `screenshot`, `recording`, `review`, `manual`. A record is bound to
the commit it ran on; after the pull request's head moves, older records are marked `(stale)` and
no longer verify a criterion. A criterion without a record on the current head is unverified.

## Handoff

Where: a comment on the task's issue (or its pull request when there is no issue). Written by
`handoff` when a session stops with work unfinished. Sections: goal, state, verified, next step,
done when, links, suggested skills — links instead of copies. Template: `skills/handoff/SKILL.md`.
