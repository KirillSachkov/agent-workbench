# The harness is a forkable repository, updated by three-way merge

A person or team customises the harness by forking agent-workbench (or creating a repository from it
as a template) and editing anything; upstream changes reach the fork through plain git merges. Our
CLI delivers a harness source — agent-workbench by default, or a fork — into projects
(`workbench init --from <repo>@<ref>`), records it in the harness lock, and `workbench update` merges
a new version three-way against the recorded base, on its own branch and pull request, leaving
conflicts as ordinary git conflicts. We chose this over BMAD's override layer (which forbids editing
managed files, against decision B6) and spec-kit's refusal on modified files (which makes local
edits a dead end), because editing files is the intended way to customise and git already solves
merging a fork; it also replaces a profile system (B5). The same model applies one level up, where
agent-workbench tracks its fork of Matt Pocock's skills (ADR 0002).

Evidence: `docs/research/2026-09-29-harness-frameworks-compared.md` §3.1, §6.1, §11.1.1–2, §11.1.9.
Decision ticket: #29.

## Consequences

- Each release carries a rename/removal map so retired skills disappear from projects instead of
  lingering (the upstream rename problem, `docs/research/2026-09-29-mattpocock-skills-design.md` §3).
- The harness lock must record the source repository and ref, plus the hashes of written files, to
  serve as the merge base.
