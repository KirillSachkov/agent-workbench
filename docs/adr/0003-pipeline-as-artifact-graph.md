# The pipeline is a declared artifact graph, not a workflow engine

Each track is data: nodes for the artifacts and facts that must exist, their dependencies, the checks
each node must pass and where it lives. The harness computes every node's state (done, ready,
blocked) from facts in the repository, tracker, PRs and CI, as OpenSpec's `status --json` does; it
never runs steps and stores no state of its own. We chose this because process is chosen per task
(decision A3), a stage must be derived from facts, and launching agents is not ours to do: a fixed
loop with typed extension points (GSD) would impose one process, and a workflow engine with
`if`/`while`/fan-out and persisted run state (spec-kit) would make the harness an orchestrator.
Ready nodes also form a ready-made queue for any external orchestrator.

Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §1.3, §2.3–2.4, §6.3, §11.1.5.
Decision ticket: #9.

## Consequences

- Conditional rules (for example "a risky change needs a second reviewer") are expressed as
  properties of a track or profile, not as control flow.
- Per-node instructions for agents live in skills; after ADR 0004 and decision C11 skills do not
  depend on the CLI.
