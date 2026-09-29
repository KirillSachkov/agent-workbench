# Conventions over checks: the harness's code enforces nothing

The pipeline rests on skills, conventions and worked examples, as Matt Pocock's skills do. The
harness's own code only collects facts for the command center (node state) and installs; agents
read facts with `gh` and `git` themselves; it does not validate artifacts and does not block anything. What still
blocks lives outside the harness: the project's own CI and the owner's merge. The owner chose this
over a hybrid (conventions plus a small project-chosen blocking set) because rigid tooling breaks as
models and runtimes change, gets in the way of deliberate deviations, and costs upkeep, while skills
and conventions are easy to edit per project.

This is a deliberate deviation from the research: agents over-report their own work (22–80% of
cases) and rules that live only in prose decay (`docs/research/2026-09-27-reviewing-agent-work-practices.md`
§2, §6; `docs/research/2026-09-25-artifact-pipelines-industry.md` §8;
`docs/research/2026-09-29-mattpocock-skills-design.md` §6). Do not "fix" it by adding validators
without a new owner decision. Decision ticket: #15.

## Consequences

- The artifact contract (ADR 0001) and evidence (C9) are documented shapes, not schemas.
- Human judgment and non-author review carry verification; the result card makes gaps visible
  rather than blocking on them.
- Questions about where gates execute, which human gates become checks and hooks as enforcement
  mostly reduce to "none by the harness".
