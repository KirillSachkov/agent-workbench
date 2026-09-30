---
name: coordinate
description: Read the tracker, pull requests and checks and return a short control brief of the project — current stage, progress of each spec or map, blockers with the unblocking action, work ready now split into parallel lanes, gates waiting for the owner, and the single next step. Read-only; starts nothing. Use when asked where the project stands, what to do next, or what can run in parallel.
---

# Coordinate

Return a **control brief**: one screen that tells the owner where the project stands and what to do next. Rebuild it from facts every time — the tracker, pull requests, checks and git — never from memory of an earlier session.

**Read-only.** This skill creates, edits, labels, comments on, merges and closes nothing, and starts no agent sessions. It proposes; the owner, or a skill the owner started, acts.

## 1. Gather the facts

Use the operations in `docs/agents/issue-tracker.md` for the tracker, and the pull request operations there (or the code host's CLI) for pull requests. Read, do not write:

- **Project rules.** `AGENTS.md`, the project's roadmap or decision log if it has one, and the tracks in `workbench.toml` (`workbench config` prints them resolved).
- **Specs and maps.** Open issues that are specs or `wayfinder:map` maps, their child tickets (sub-issues or the tickets that name them as parent), and the state of each child.
- **Tickets.** Open tasks with their labels (triage role, acceptance mode), assignees, and open blockers.
- **Pull requests.** Open pull requests with their linked issue, draft state, review decision and check rollup; pull requests merged since the last brief if the owner asks what changed.
- **Work in flight.** `git worktree list` and branches named after issues show which tasks already have a writer.

## 2. Derive

- **Stage.** Where the project is in its roadmap or, without one, in its current track: which spec or map is active and which node (intent, spec, tickets, delivery, accepted) it has reached.
- **Progress.** For each open spec or map: children done / total, and what is in flight.
- **Blockers.** Each blocked task with the thing that blocks it and the action that unblocks it — who does what (the owner approves a spec, an agent fixes CI on PR #n, a person answers a question on #m).
- **Ready now.** Open, unblocked, unassigned tasks whose intent is approved (the `ready-for-agent` role), without an open pull request.
- **Lanes.** Split the ready work into **lanes**: one task per lane, each sized for one fresh agent session. Two tasks may run in parallel only when they do not touch the same **shared surface** — the same module or files, root configuration, lockfiles, CI, the database schema, the public interface another task changes. When unsure, sequence them. Propose at most three parallel lanes unless the project says otherwise, and flag a task too large for one session instead of making it a lane.
- **Gates for the owner.** Everything waiting on a person: intents to approve, specs to review, pull requests to check and merge (with their acceptance mode), questions asked in issues or pull requests, `ready-for-human` tasks.
- **Next step.** The one action that unblocks the most, or moves the active stage forward, now.

## 3. Write the brief

```markdown
**Stage:** <roadmap stage or active spec/map, and its node>

**Progress**
- <spec or map #n>: <done>/<total> done, <k> in flight

**Blockers**
- #<n> <title> — blocked by <what>; unblock: <who does what>

**Ready now — lanes**
1. #<n> <title> — <one-line scope>; touches <surface>
2. #<m> <title> — <one-line scope>; touches <surface>
Not in parallel: #<a> and #<b> (both change <surface>).

**Waiting for you**
- PR #<n> (acceptance:human) — result ready to check
- #<m> — question from the agent

**Next step:** <one action>
```

Keep it short: one line per item, links as `#n`, no prose paragraphs. Leave out a section that has nothing in it. Say plainly when a fact could not be read (no access, rate limit) rather than guessing it.

Another skill may use the brief as its input — for example a skill that starts lanes after the owner approves them. The brief itself never starts anything.
