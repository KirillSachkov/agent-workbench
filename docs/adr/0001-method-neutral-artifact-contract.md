---
status: accepted, amended by ADR 0004
---

# The artifact contract is the core; other methods are not hosted

agent-workbench defines its own artifact contract and stage model and ships a default method, but
does not ship adapters that translate spec-kit, OpenSpec or BMAD artifacts into the contract. A stage
counts when its artifact passes the contract, whatever produced it, so a team may keep writing specs
with another tool as long as the output satisfies the contract. We chose this because the value the
existing frameworks lack is the fact layer — stage derived from tracker and PR facts, owner decisions
as facts, a repo-level harness lock, a CI entry point — not another method; methods churn and get
absorbed by runtimes (Agent OS v3 dropped its spec and task commands), and adapters to fast-moving
formats (spec-kit ships about 52 releases in 90 days; BMAD is mid-migration) would be a permanent
maintenance surface while adding no checks we would not write anyway.

Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §11.2.6, §11.3, §1–3;
`docs/research/2026-09-29-mattpocock-skills-design.md` §2.7. Decision ticket: #6.

## Considered options

- **Own method only.** Rejected: ties the contract to our skills and turns away teams that already
  write specs with another tool, against the brief's "as general and configurable as possible".
- **Host of methods through adapters.** Rejected for the maintenance cost above; revisit only as a
  new effort, not by adding an adapter piecemeal.

## Amendment (ADR 0004)

The contract is a documented shape with examples that skills follow and people judge; the
harness's code does not check it. The method-neutral part of this decision stands.
