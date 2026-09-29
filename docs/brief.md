# Brief

## Why

A developer who works AI-first runs several agents (Claude Code, Codex and others) in parallel across
different projects. They need to control the development pipeline and see results quickly: which
tasks are in flight, on which branches, which agents are working and at what stage, what each agent
did, which artifacts it left, what has been verified and what is waiting for their decision. Vendor
apps show only their own agent, large platforms such as Paperclip bring their own tracker and a lot
of extra weight, and many command centers have already shut down.

## Outcome

1. **Harness and pipeline.** Tracks of work declared as artifact graphs, with mechanical checks and
   owner checkpoints; a built-in default for software development that each project adapts or
   replaces in its own configuration. General and configurable, not tied to any particular project.
   (Amended 2026-09-29 by decisions A3, B4 and B5 in `docs/decisions.md`; there are no profiles.)
2. **Artifact contract.** Specification, plan, delivery report, evidence, handoff; checked by a
   command, later by hooks and CI.
3. **Workspace.** Agent and Herdr setup: plugins for status, diff review and document review, key
   bindings, and our own plugin that ties them together.
4. **Command center.** A data aggregation core (GitHub: issues, stages, PRs, checks, tracker sessions;
   artifacts; Herdr: live agents; git: branches and worktrees) with two interfaces on top: a TUI in
   Herdr and a web dashboard.
5. **One-command installation** on the developer machine and into a project.

## Principles

- Tasks and stages live in the tracker; the command center shows them and keeps no statuses of its
  own.
- A stage is derived from facts: an artifact, a PR, checks, an owner decision.
- What is mandatory is checked by code; judgment stays with instructions and the human.
- Public interfaces of sources, not internal session files of agents.
- Small existing tools where they are good; our own code where pipeline knowledge is needed.
- As general and configurable as possible: no project-specific rules in this repository.

## Not in scope now

- Product code of the projects where the harness is applied.
- A task tracker of our own instead of GitHub.
- Launching and orchestrating agents instead of Herdr and the runtimes themselves.
- Project-specific configuration (for example Inside); consumers configure the harness in their own
  repositories.

## Sources of ideas

Earlier harness lines are references for ideas and lessons, not code to port.

| Source | What to learn from |
|---|---|
| `inside-engineering` (Inside harness) | Pipeline WORKFLOW, CLI and package lifecycle, tracker and agent sessions, skill adaptations |
| `agent-harness` | Portable installation for several runtimes, `start-project`, the clean Matt Pocock skill set |
| Personal Herdr setup | Plugin that binds a session to a worktree, key bindings and actions |
| Research | `docs/research/` here and the AI Engineering course notes on harnesses |

## Resolved decisions

License, language, order, stack and project-specific parts were decided on 2026-09-29; see
`docs/decisions.md`.
