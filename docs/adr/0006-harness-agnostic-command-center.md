# The command center works without our harness and shows more as it finds more facts

The command center is a separate part from the harness: it shows any folder where an agent runs and
adds detail level by level from the facts it finds — a folder with an agent (Herdr status), a git
repository (branches, worktrees, diffs), a GitHub repository (PRs, CI, reviews, linked issues,
labels), and a repository whose harness follows our documented conventions (stages, criteria and
evidence, the spec → tickets tree). A project with another harness, or none, simply shows fewer
details; another harness reaches the top level by following the conventions, with no adapter and
no registration. We chose this over a command center that works only with our harness because the
owner runs agents in many repositories with different harnesses and wants one place for all of
them, and because conventions (ADR 0004) keep the seam cheap. Decision ticket: #49.

## Consequences

- A "project" is a git repository found from where agents run, plus projects pinned in local
  configuration; nothing has to be installed into a repository for it to appear.
- The conventions the top level reads (the result card in the PR, labels, artifact graphs) are part
  of the artifact contract, not of the command center's code.
