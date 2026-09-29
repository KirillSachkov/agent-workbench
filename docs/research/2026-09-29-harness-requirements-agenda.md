# Harness requirements: agenda for the grilling session

Date: 2026-09-29. A synthesis, not new research: the open questions from four notes of the same day,
deduplicated and ordered so that earlier answers constrain later ones. Each item links to the
questions it merges:

- **HoH** — [hoh-harness-design §9.3](2026-09-29-hoh-harness-design.md)
- **MP** — [mattpocock-skills-design, open questions](2026-09-29-mattpocock-skills-design.md)
- **FC** — [harness-frameworks-compared §11.4](2026-09-29-harness-frameworks-compared.md)
- **CP** — [cross-runtime-portability §10.4](2026-09-29-cross-runtime-portability.md)

Fixed inputs (not reopened): `docs/brief.md` and `docs/decisions.md` — general and configurable, no
project-specific parts, English, Rust core/CLI/TUI, tracker is GitHub Issues, stage derived from
facts, hooks after the artifact contract, spec as a repo file approved through a PR.

## A. Positioning

1. **Own method or host of methods?** Does agent-workbench define its own pipeline and artifact
   contract only, or can a profile host spec-kit, OpenSpec or BMAD artifacts? (FC10)
2. **Relation to Matt Pocock's suite.** A pinned, unmodified upstream layer updated by reviewed diff,
   or a fork we edit? Do our own skills follow its conventions (user- vs model-invoked, composition)?
   (MP1, MP8)
3. **Default stage shape.** Keep idea → spec → tickets → implementation, or adopt HoH's
   Planner → Developer → QA loop, or keep our stages and add Preservation/Acceptance gates as fields?
   (HoH1)

## B. Pipeline model and configuration

4. **Expressiveness.** Artifact DAG (OpenSpec), fixed loop with typed extension points (GSD) or a
   workflow engine with control flow (spec-kit)? How much of a stage is data and how much skill prose?
   (FC4)
5. **What a profile is.** Stages, checks, skills, standards, tracker settings, domain policy packs?
   Inheritance? Where profiles live (machine, repo, shared repo)? (FC5, HoH6)
6. **How much configuration.** Where the line sits between a profile and "Config is death" (plain
   instructions in `AGENTS.md`). (MP2)
7. **Automatic repair loop.** A bounded retry inside a stage, or every failed check returns to the
   owner? (HoH2)

## C. Artifacts and evidence

8. **Candidate identity.** PR head SHA, tree SHA or content hash; what happens to evidence when the
   head moves after review. (HoH3)
9. **Evidence shape.** Verified/gap partition with required execution records; allowed record types
   (CI run, test log, screenshot, trace). (HoH4)
10. **Carry-over between stages.** Bounded packet in the prompt plus a lossless ledger — and where the
    ledger lives (repo file, issue comments, PR artifacts). (HoH7)
11. **Completion step.** A stage skill, a CLI command or a hook, relative to `implement`,
    `code-review` and `pr`. (MP4)

## D. Gates and verification

12. **Where gates execute.** CLI validators called by skills, native hooks, or CI as the only
    universal gate; does CI come before hooks? (FC6)
13. **Who decides pass.** Can a QA agent's verdict pass a stage when gated by mechanical
    preconditions, or is the final signal always human or CI? (HoH8)
14. **Which human gates become checks** (tickets approved, seams confirmed, PR not merged by agent)
    and which stay prose. (MP5)
15. **Hooks, when they come.** Enforcement or observation; fail-open or fail-closed. (CP6)
16. **Agent in CI.** Only mechanical checks, or also a judgment check through `claude -p` /
    `codex exec` with an output schema? (CP7)

## E. Runtime support

17. **Runtime tiers and proof of support.** Is Claude Code + Codex the tier-1 minimum; when do Gemini,
    OpenCode, Cursor, Copilot, Kimi come; what proves support (session-start injection, hook
    conformance, eval run)? (CP1, FC7, MP6)
18. **Neutral source or native files.** One neutral declaration rendered into native files with a
    drift check (`sync`), or native files shipped directly? (CP2)
19. **Skill mirroring and Windows.** Symlink or copy for `.claude/skills`; is Windows a target? (CP3)
20. **Invocation policy source of truth.** Frontmatter plus generated `openai.yaml`, or a registry.
    (CP4)
21. **Role permissions.** Declared per role (read-only reviewer, single writer) and enforced by
    runtime sandbox flags, hooks or a post-hoc check — given that we do not launch agents. (HoH5)

## F. Distribution and lifecycle

22. **Channel.** Own binary rendering per runtime, plugin marketplaces, a generic skills installer,
    or a binary plus plugins — without the two-path trap. (FC1, CP5)
23. **What is committed to a project.** Generated per-runtime files, nothing, or config plus a lock
    with generation on each machine. (FC2)
24. **Local edits across updates.** Override layer only, refuse on modified files, or three-way
    reapply. (FC3)
25. **Version skew in a team.** A repo-level harness lock with fail-fast checks? (FC11)
26. **Global vs per-project.** What the machine install owns, whether it may write user-level agent
    configs. (MP7, CP10)
27. **Release channels** for our own skills (stable/beta, promotion rules). (MP8)

## G. Tracker

28. **Tracker model.** GitHub only first, or a store adapter from day one; where stage facts and
    owner decisions live. (FC8)
29. **Tracker operations.** Skills call our CLI, or keep prose templates in `docs/agents/`. (MP3)

## H. Ecosystem (can be deferred)

30. **Extensions and trust.** A community catalog at all; discovery-only, re-consent on executable
    changes. (FC9)
31. **MCP.** Ship servers by default or only a place to declare them; secret naming. (CP8)
32. **Telemetry.** None or opt-in. (FC12)
33. **ACP for the command center** as a read-only status source — belongs to #3. (CP9)
