---
name: handoff
description: Write a handoff as a comment on the task's issue so another session or person can continue. Use when a session must stop with its work unfinished (context running out, repair limit reached, switching agent or person), or when the user asks for a handoff.
argument-hint: "What will the next session be used for?"
---

Write a handoff so a fresh agent, or a person, can continue the work from the tracker alone. The handoff is a comment on the task's issue, posted with the "comment on an issue" operation in `docs/agents/issue-tracker.md` — not a file in a temporary directory, and not a message in this chat.

Before writing, make the state reachable: commit the work in progress to the task's branch and push it, so every link in the handoff resolves for the next reader.

If the work has no issue, post the handoff on its pull request. With neither, ask the user where it should go.

If the user passed arguments, treat them as a description of what the next session will focus on and tailor the handoff to it.

## Shape

```markdown
## Handoff: <what the task is, in a few words> (<YYYY-MM-DD>)

**Goal.** What the task must achieve, in one or two sentences. Link the spec or the acceptance criteria instead of restating them.

**State.** Where the work stands: the branch and its head commit, the pull request if one exists, what is built and what is not.

**Verified.** What was checked and how — the command, test or review and its outcome — and on which commit. Anything not checked is not listed here.

**Next step.** The single next action, specific enough to start without asking.

**Done when.** The criterion that ends the task.

**Links.** The issue, spec, pull request, commits, files at a line, research notes, decisions.

**Suggested skills.** Which skills the next session should call the Skill tool for.
```

## Rules

- Links, not copies. Do not duplicate what already lives in an artifact (spec, ADR, issue, commit, diff); point at it by URL or `path:line`.
- Write only what the next reader needs and cannot get from the artifacts: the reasoning behind choices, dead ends already tried, the one fact that is easy to miss.
- Redact sensitive information: API keys, passwords, tokens, personal data.
- A handoff does not change the task's labels or close anything.
