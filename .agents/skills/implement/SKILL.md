---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

Read the task in full first (body and comments, through `docs/agents/issue-tracker.md`) and any handoff on it. Work on a branch for the task, named by the project's convention when `AGENTS.md` states one. One writer per task: if the task is assigned to someone else, or another worktree already holds its branch, stop and tell the user; otherwise claim the task.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end.

Ambiguity about intent, or a conflict between the task and the spec, goes to the user at once. Do not guess and build on the guess.

Once done, call the Skill tool with "code-review" to review the work. The reviewers are fresh-context subagents; they report findings and do not edit. You fix what they find.

When a check or a review finding fails with a clear signal, fix it and rerun: that is one **repair round**. Stop after the project's limit (`[repair] max_rounds` in `workbench.toml`, 2 by default). Infrastructure failures (network, timeouts, a broken runner) do not count as rounds; report them separately. Never weaken a test or a CI step to make it pass.

Commit your work to the current branch, then call the Skill tool with "pr" to open the pull request with its result card.

If you have to stop before the work is done (the repair limit is reached, the context is running out, or the user switches agent or person), commit what you have, then call the Skill tool with "handoff" instead of "pr".
