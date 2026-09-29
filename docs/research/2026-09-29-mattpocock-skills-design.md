# Matt Pocock's skills as a harness philosophy and product

Date: 2026-09-29. Method: a clone of https://github.com/mattpocock/skills at HEAD `c55ee46`
(2026-09-18), its tags, `CHANGELOG.md`, pending `.changeset/*.md`, `.agents/` (ADRs and
conventions), `.out-of-scope/`, plugin manifests, scripts and the promoted `SKILL.md` files; the
aihero.dev pages https://www.aihero.dev/skills (and its `skills.md` twin), the five changelog posts
linked from it, three orientation posts, and the published docs page for the setup skill, all fetched
on 2026-09-29; the AI Coding Dictionary repository https://github.com/mattpocock/dictionary-of-ai-coding
(HEAD `ed1ebed`, 2026-09-24); the Sandcastle README https://github.com/mattpocock/sandcastle (last push
2026-06-29); the skills.sh installer README https://github.com/vercel-labs/skills; open issues in
`mattpocock/skills` read with `gh`. Star counts are from `gh api` on 2026-09-29.

Citation shorthand: `repo:<path>` means
`https://github.com/mattpocock/skills/blob/c55ee46073ed923f86ce59a5eb3b6d895095d1b7/<path>`;
`dict:<Term>` means `https://github.com/mattpocock/dictionary-of-ai-coding/blob/ed1ebed/dictionary/<Term>.md`;
`#N` means `https://github.com/mattpocock/skills/issues/N` (or the PR of that number when the
CHANGELOG cites it as a PR).

Scope. `docs/research/2026-09-25-artifact-pipelines-industry.md` §6 already covers the artifacts of
`to-spec`, `to-tickets`, `triage` (agent brief), `wayfinder`, the AFK/HITL labels, the `implement`
completion gap and Ralph. This note does not repeat them. It covers the suite as a harness design and
as a distributed product.

---

## 1. What the product is

- **Size and shape.** 25 promoted skills in two buckets, `engineering/` (18) and `productivity/` (7);
  the plugin ships exactly this set (`repo:.claude-plugin/plugin.json`, version 1.2.3). 14 are
  user-invoked and 11 model-invoked (counted from `disable-model-invocation: true` in each promoted
  `SKILL.md`). Non-promoted buckets: `misc/` (4, "kept around but rarely used"), `in-progress/` (9,
  beta), `deprecated/` (empty) (`repo:skills/*/README.md`).
- **Small files.** `grill-with-docs`, `grill-me` and `wait-what` are 7 lines each, `implement` 15,
  `research` 12, `resolving-merge-conflicts` 14; the largest are `teach` (140), `diagnosing-bugs` (138),
  `wayfinder` (128) (line counts at `c55ee46`). Only two promoted skills carry executable code:
  `repo:skills/engineering/wizard/template.sh` and
  `repo:skills/engineering/diagnosing-bugs/scripts/hitl-loop.template.sh`.
- **Reach.** 271,544 stars and 22,861 forks on 2026-09-29 (`gh api repos/mattpocock/skills`); the v1
  post claimed "over 4.2 million downloads and 135,000 GitHub stars" in June
  (https://www.aihero.dev/skills/skills-changelog-v1-announcement). 535 open issues.
- **Self-description.** "Approaches like GSD, BMAD, and Spec-Kit try to help by owning the process.
  But while doing so, they take away your control and make bugs in the process hard to resolve. These
  skills are designed to be small, easy to adapt, and composable. They work with any model."
  (`repo:README.md`). The site pitch: "Skills are plain files, not a lock-in platform"; "Skills form a
  chain. Each one's output is the next one's input, so the whole workflow gets better as you tune
  single steps" (https://www.aihero.dev/skills).

## 2. Stated design philosophy

### 2.1 Small, composable, owned by the user

- Each skill encodes "one good habit (grilling a plan, writing a spec, reviewing a diff), so the agent
  runs it the same way every time" (https://www.aihero.dev/skills).
- "Skills don't have to be long to be impactful. You just need to choose the right words at the right
  time" (https://www.aihero.dev/5-agent-skills-i-use-every-day, updated 2026-03-16).
- The skills are framed as fixes for four failure modes: misalignment (grilling), verbosity (shared
  language in `CONTEXT.md`), code that does not work (feedback loops, `tdd`, `diagnosing-bugs`), and a
  ball of mud (deep modules, `improve-codebase-architecture`) (`repo:README.md`, "Why These Skills
  Exist"). The codebase, not the prompt, is named as "the biggest influence on AI's output"
  (https://www.aihero.dev/how-to-make-codebases-ai-agents-love).
- `implement` is deliberately thin: it "mostly relies on the agent's priors and what your agents.md
  file teaches it. I almost didn't create a skill for this because it's so simple, but people kept
  asking 'what's the flow?'"
  (https://www.aihero.dev/skills/skills-changelog-v1-1-wayfinder-to-spec-to-tickets-grilling-improvements).

### 2.2 User-invoked versus model-invoked

- The one axis that splits the suite is who can invoke a skill. **User-invoked** skills are reachable
  only by the human typing the name, "their job is to orchestrate". **Model-invoked** skills "hold the
  reusable discipline". "A user-invoked skill may invoke model-invoked skills, but never another
  user-invoked one" (`repo:README.md`, "Reference"; `repo:.agents/invocation.md`).
- The test for keeping a skill model-invoked: "could the model usefully reach for this autonomously?
  (Reuse is the reason to extract a skill, not the test.)" (`repo:.agents/invocation.md`).
- The cost model behind it is two budgets: **context load** (always-loaded descriptions) and
  **cognitive load** (the human as the index of what exists). "Pick model-invocation only when the
  agent must reach the skill on its own, or another skill must"
  (`repo:skills/productivity/writing-for-agents/SKILL.md`, `.../SKILL-MECHANICS.md`).
- The v1 release made this split structural and reported "a 63% reduction in token cost for skill
  descriptions" from `disable-model-invocation: true`
  (https://www.aihero.dev/skills/skills-changelog-v1-announcement). The 63% figure is the author's own
  measurement; the method is not published (unconfirmed).
- A **router skill** (`ask-matt`) cures the cognitive load that user-invoked skills create: "one
  user-invoked skill that names the others and when to reach for each ... It can only hint, never fire
  them" (`repo:skills/productivity/writing-for-agents/SKILL-MECHANICS.md`). Keeping the router accurate
  is a repository rule: "a router that lies" is the failure named in `repo:CLAUDE.md`.
- Shared reference lives inside the model-invoked skill that owns it; two user-invoked skills that need
  the same reference must push it into "a plain file outside the skill system"
  (`repo:skills/productivity/writing-for-agents/SKILL-MECHANICS.md`).

### 2.3 Flows

- A **flow** is "a path through the skills": one main flow (`grill-with-docs` → optional prototype
  detour → `to-spec` → `to-tickets` → `implement`, which drives `tdd` and closes with `code-review`),
  two on-ramps (`triage`, `diagnosing-bugs`) plus `wayfinder` for "a huge, foggy effort", upkeep
  (`improve-codebase-architecture`), a vocabulary layer underneath (`domain-modeling`,
  `codebase-design`), and standalones (`repo:skills/engineering/ask-matt/SKILL.md`).
- The flow is advice, not an engine: nothing sequences the skills except the human and the router.
  `wayfinder` was explicitly kept as "a situational on-ramp, not the new main entry flow ... crowning
  wayfinder as the default spine is a v2-sized move" (`repo:CHANGELOG.md`, 1.1.0, commit `0557d57`).
- The site groups the same set by when you reach for it: Getting Started, Main Flow, Shaping, Upkeep,
  Productivity, Reference (https://www.aihero.dev/skills).

### 2.4 Context hygiene and the smart zone

- Keep grilling, spec and tickets "in one unbroken context window"; each `/implement` "starts fresh,
  working from the ticket". The limit is the smart zone, "~150k tokens on state-of-the-art models"
  (`repo:skills/engineering/ask-matt/SKILL.md`). The figure moved from ~120k to ~150k in 1.2.0
  (`repo:CHANGELOG.md`, `77d207e`).
- The dictionary definition: quality falls off "long before" the window limit; "Plan around the smart
  zone, not the window"; "Doing one task per session gives each task the sharpest part of the session"
  (`dict:Smart zone`).
- Phase boundaries have five ordered options: continue, `/clear`, `/handoff`, subagent, `/compact`;
  continue is ruled out first because it "keeps the conversation as a primary source rather than a
  summary of one", and `/compact` is "the default, not the first reach"; `/handoff` is narrow (new
  harness, new directory, colleague, mid-phase fork) (`repo:skills/engineering/ask-matt/PHASE-BOUNDARIES.md`;
  `repo:CHANGELOG.md`, 1.2.0).
- Tickets are sized to "a single fresh context window" and wayfinder decision tickets to "one 100K
  token agent session" (covered in the industry note §6; not repeated).

### 2.5 AFK agents and human-in-the-loop

- AFK is "the throughput multiplier of AI coding"; its characteristic failure is "hours of finished,
  confident work built on a wrong call made in the first ten minutes". The prescription: resolve
  ambiguity before (grilling, spec), let automated checks and review stand in during, and end in
  "a PR, not changes already merged" after; "AFK doesn't remove human review; it defers all of it to
  the end" (`dict:AFK`).
- HITL suits work that is "ambiguous, irreversible, or where you'd struggle to review the finished
  result"; grilling and prototyping are "in-the-loop by nature, because your reactions are the input"
  (`dict:Human-in-the-loop`).
- A **software factory** is sessions started by triggers (an issue labelled `ready-for-agent`, cron,
  CI failure, another session finishing); "Deciding which of those decisions stay human is the main
  design question". A **dark factory** is one where no human reviews the output (`dict:Software factory`,
  `dict:Dark factory`).
- In the skills this shows up as: grilling splits **facts** (the agent looks them up, via subagents)
  from **decisions** (always put to the human), added after reports of models "grilling itself"
  (`repo:CHANGELOG.md`, 1.1.0, `639df6e`); `tdd` became "reference material only" so it can be handed
  to an AFK agent (v1.1 post); `wizard` is model-invoked so the agent reaches for it "the moment it
  hits a step only a human can perform" (`repo:CHANGELOG.md`, 1.2.0, `b3376f8`).
- Sandcastle is the author's separate TypeScript library for running AFK agents: `sandcastle.run()`
  in Docker, Podman, Vercel or no sandbox, a branch strategy (`head`, `merge-to-head`, `branch`),
  `maxIterations`, and the Ralph-style completion signal `<promise>COMPLETE</promise>`; `init` offers
  templates `simple-loop`, `sequential-reviewer`, `parallel-planner`, `parallel-planner-with-review`
  and agents `claude-code`, `pi`, `codex`, `cursor`, `opencode`, `copilot`
  (https://github.com/mattpocock/sandcastle README). The skills repository does not depend on it; the
  link is conceptual (the `ready-for-agent` label is the hand-off point in both).

### 2.6 Writing discipline as the core technology

`writing-for-agents` is the suite's theory of how instructions steer a model
(`repo:skills/productivity/writing-for-agents/SKILL.md`):

- **Context pointer**: a description or an `AGENTS.md` line; "The pointer's wording, not its target,
  decides when the agent reaches the material".
- **Leading words**: pretrained concepts (_tracer bullets_, _fog of war_, _red_, _tight_) that anchor
  behaviour in few tokens. The same idea drove the Fowler smell baseline in `code-review` ("the agent
  already knows about code smells ... All you need to do is invoke the idea", v1.1 post).
- **Negation** steers toward the forbidden behaviour, so prompt the positive.
- **Completion criteria** must be clear and demanding, against "premature completion".
- **Pruning**: single source of truth; the environment (`package.json` scripts, config, `--help`) is a
  source of truth and a doc that restates it is a **cache**; hunt **no-ops** and **sediment**.

### 2.7 What the author deliberately does not do

| Refusal | Evidence |
|---|---|
| Own the process | "take away your control" (`repo:README.md`) |
| Per-user configuration of skill behaviour | "skills stay opinionated: *'Config is death.'* Preferences belong in your `CLAUDE.md`" (https://www.aihero.dev/skills-setup-matt-pocock-skills, source `repo:docs/engineering/setup-matt-pocock-skills.md`) |
| A user-level (global) config mode | "no user-level mode exists. Every repo carries its own `docs/agents/`" (same page) |
| Niche issue-tracker backends | each backend "hard-codes a CLI shape into the skills ... permanent maintenance surface" (`repo:.out-of-scope/mainstream-issue-trackers-only.md`) |
| A cap on grilling questions | "natural-language steering is the intended control surface, not a numeric limit" (`repo:.out-of-scope/question-limits.md`) |
| A verify/check mode for setup | "run `/setup-matt-pocock-skills` and tell it to verify" (`repo:.out-of-scope/setup-skill-verify-mode.md`) |
| A native Codex plugin (for now) | deferred, §3 (`repo:.agents/adr/0002-ship-as-a-claude-code-plugin.md`) |
| Creating tracker labels | the triage mapping "does not run `gh label create`" (setup docs page) |
| Aliases for renamed skills | "Reinstall under the new name. There is no alias" (v1.2 post) |

## 3. Distribution, installation and versioning

- **Two exclusive routes.** "The Claude Code plugin installs the whole set as a managed, read-only
  bundle that updates when I ship, so you subscribe rather than fork. skills.sh copies editable skill
  files into your project ... Pick one: installing both leaves you with every skill twice"
  (`repo:README.md`). The install wording has a single source, `repo:.agents/install-block.md`, which
  README and changesets copy verbatim; docs pages carry no install commands because the site renders
  an install widget.
- **Plugin.** `claude plugins install mattpocock-skills`, listed in Claude Code's official marketplace
  since 2026-08-05; official marketplaces auto-update. The listing pins a commit sha, so "a release
  reaches installed users when that pin moves, not the moment we tag"; the repo's own
  `marketplace.json` is kept only as a fallback for forks and unreleased commits
  (`repo:.agents/adr/0002-ship-as-a-claude-code-plugin.md`, "Update, 2026-08-05"). The plugin ships only
  the promoted set, listed explicitly in `plugin.json`'s `skills` array.
- **skills.sh.** `npx skills@latest add mattpocock/skills` (whole set, interactive pick of skills and
  agents) or `--skill=<name>`; update with `npx skills update`. "Nothing updates behind your back"
  (`repo:README.md`). The installer supports project scope (`./<agent>/skills/`, committed) or global
  (`-g`), symlink (recommended, one canonical copy) or `--copy`, and roughly 50 agents; Codex maps to
  `.agents/skills/` (https://github.com/vercel-labs/skills README). The site recommends this route
  first and lists "Claude Code · Cursor · Codex · Copilot" (https://www.aihero.dev/skills).
- **Codex.** A native Codex plugin is deferred because `.codex-plugin/plugin.json` accepts `skills` only
  as a single path and Codex "drops symlinks" when it copies a plugin into its cache, so a curated
  subset of a bucketed repo cannot be expressed (`repo:.agents/adr/0002-ship-as-a-claude-code-plugin.md`).
  Instead every skill carries `agents/openai.yaml` beside `SKILL.md` with Codex UI metadata and, for
  user-invoked skills, `policy.allow_implicit_invocation: false`, "so the set works in both harnesses
  without generated copies"; `AGENTS.md` is a symlink to `CLAUDE.md` in the skills repo
  (`repo:CHANGELOG.md`, 1.2.0, `697d4ce`). Parity bugs followed: `writing-for-agents` was invisible to
  Codex implicit invocation until 1.2.2 (`4aaccb5`); Claude-specific tool and agent-type names were
  removed from subagent instructions in 1.2.3 "so the step is followable on Codex and other harnesses"
  (`14bfbbd`); #693 reports that Claude Code coordinator mode drops user-invoked skills.
- **Channels.** `in-progress/` is "a beta channel, published on purpose, installable one skill at a
  time through skills.sh", excluded from the plugin and docs (`repo:skills/in-progress/README.md`;
  `repo:CHANGELOG.md`, 1.2.0, `c66bdee`). Maintainers use `repo:scripts/link-skills.sh`, which symlinks
  every non-`misc`, non-`deprecated` skill into `~/.claude/skills` and `~/.agents/skills`.
- **Versioning.** Changesets with a GitHub release workflow that opens a "version skills" PR and tags
  (`repo:.github/workflows/release.yml`, `repo:.changeset/config.json`); `npm run version` copies the
  `package.json` version into `plugin.json`, and `--check` fails on drift
  (`repo:scripts/sync-plugin-version.mjs`); `claude plugin validate . --strict` is required "after
  touching either manifest" (`repo:CLAUDE.md`), as a written rule rather than a CI step. Tags: v1.0.0
  and v1.0.1 on 2026-06-17, v1.1.0 on 2026-07-08, v1.2.0 and v1.2.2 on 2026-08-05, v1.2.3 on 2026-08-06.
  Since v1.2.3 there are 54 commits and 11 pending changesets on `main`, so plugin users (pinned
  release) and skills.sh users (who pull from the repository) currently run different text. That
  skills.sh installs from the default branch rather than the latest tag is inferred from the README's
  "pull my latest changes" wording (unconfirmed).
- **Breaking changes are renames.** `diagnose` → `diagnosing-bugs` (1.0.0), `to-prd` → `to-spec`,
  `to-plan` + `to-issues` → `to-tickets` (1.1.0), `writing-great-skills` → `writing-for-agents` (1.2.0).
  The v1.1 migration instruction was manual: "The skills installer won't pick these up automatically
  ... go through your skills folder and make sure no old skills are lingering" (v1.1 post). The site's
  own `skills.md` still lists `/to-prd`, `/to-issues` and `/domain-model` as featured skills
  (https://www.aihero.dev/skills.md, fetched 2026-09-29), a live example of name drift.
- **Docs as product surface.** Every promoted skill has `docs/<bucket>/<name>.md`, published at
  `https://aihero.dev/skills-<name>`, with a fixed frame: What it does (with the "defining
  constraint"), When to reach for it, optional Prerequisites, Common questions (sourced from real
  questions, "the count stays honest to the evidence"), It's working if (checkable "without opening
  `SKILL.md`"), Where it fits (`repo:.agents/writing-docs.md`).

## 4. Per-repo configuration: one skill set, many projects

- **Mechanism.** `setup-matt-pocock-skills` is user-invoked and "a prompt-driven skill, not a
  deterministic script. Explore, present what you found, confirm with the user, then write"
  (`repo:skills/engineering/setup-matt-pocock-skills/SKILL.md`). It asks at most three things: issue
  tracker (proposed from `git remote`), triage labels (one recommended-yes question, only if `triage`
  is installed), domain doc layout (single-context by default, multi-context only on monorepo
  signals).
- **Output.** `docs/agents/issue-tracker.md`, `docs/agents/domain.md`, optionally
  `docs/agents/triage-labels.md`, plus an `## Agent skills` block of one-line pointers in whichever of
  `CLAUDE.md`/`AGENTS.md` already exists. "Those files are the only thing that varies between repos.
  The skills themselves are identical everywhere" (published docs page). The skills resolve the tracker
  through that block, never by a literal path; `wayfinder` hard-coding `docs/agents/issue-tracker.md`
  was a bug fixed in 1.1.0 (`repo:CHANGELOG.md`, `d869d45`).
- **Tracker abstraction.** Seed templates for GitHub (`gh`), GitLab (`glab`) and local Markdown under
  `.scratch/<feature>/`, plus "Other": the user describes the workflow in a paragraph and "downstream
  skills follow the prose". Each template answers fixed questions: what "publish to the issue tracker"
  and "fetch the relevant ticket" mean, and a "Wayfinding operations" section (map, child ticket,
  blocking, frontier query, claim, resolve) (`repo:skills/engineering/setup-matt-pocock-skills/issue-tracker-*.md`).
  The README says "GitHub, Linear, or local files" while the skill offers GitHub, GitLab, local and
  Other (`repo:README.md` versus the skill), another drift.
- **Triage labels** are a mapping table from five canonical roles (`needs-triage`, `needs-info`,
  `ready-for-agent`, `ready-for-human`, `wontfix`) to the tracker's real strings
  (`repo:skills/engineering/setup-matt-pocock-skills/triage-labels.md`); the domain term is defined in
  `repo:CONTEXT.md` ("Triage role").
- **Domain docs.** `domain.md` tells every skill to read `CONTEXT.md` (or `CONTEXT-MAP.md` → per-context
  files) and relevant ADRs, to "proceed silently" if absent, to use glossary terms, and to flag ADR
  conflicts explicitly; `domain-modeling` creates the files lazily
  (`repo:skills/engineering/setup-matt-pocock-skills/domain.md`). ADR criteria: hard to reverse,
  surprising without context, the result of a real trade-off
  (https://www.aihero.dev/skills/skills-changelog-ubiquitous-language-grill-with-docs).
- **Hard versus soft dependency on config.** `to-spec`, `to-tickets`, `triage` carry an explicit "run
  `/setup-matt-pocock-skills` if not" line because without the mapping "output is wrong, not just
  fuzzy"; `tdd`, `diagnosing-bugs`, `improve-codebase-architecture` only refer to "the project's
  domain glossary" in vague prose and degrade gracefully
  (`repo:.agents/adr/0001-explicit-setup-pointer-only-for-hard-dependencies.md`).
- **Where the prose adapter breaks.** Open issues show the cost of expressing CLI operations as prose
  templates: `gh issue view --comments` combined with `jq` is rejected by `gh` (#733, which says
  it was first reported in #142 and closed without the fix), `--comments` output omits the issue body (#964, #678),
  `issue_dependencies_summary` is not a valid `gh issue view --json` field (#1118), GitLab commands are
  wrong (#635), the GitHub blocking recipe sits in the wayfinder section so `to-tickets` cannot find it
  (#855). Detection also fails: on the plugin channel the setup skill cannot see that `triage` is
  installed, because user-invoked skills are absent from the model's skill list by design (#812). File
  choice is by existence, not by runtime, so a leftover `CLAUDE.md` hides the block from Codex (#558;
  acknowledged as a "known gap" on the published docs page).
- **Re-running.** The skill's closing message says re-running is only needed to switch trackers; the
  docs page says templates change between versions so older `docs/agents/*.md` "can go stale", and
  re-running is "the cheap fix" (published docs page). There is no version stamp in the generated files
  (per the templates at `c55ee46`).
- **Comparison inside the same author's work.** Sandcastle's `init` is a CLI with the same three kinds
  of choices (sandbox, tracker, template) but "Every interactive prompt has a paired `--flag`", it
  "fails fast" without a TTY, refuses to overwrite `.sandcastle/`, and its Custom tracker is scaffolded
  "in a deliberately broken-until-configured state plus a `.sandcastle/SETUP_ISSUE_TRACKER.md` prompt
  you feed to your coding agent" (https://github.com/mattpocock/sandcastle README): deterministic
  scaffolding with an agent prompt only for the open-ended part.

## 5. Extensibility and composition

- **Fork versus subscribe** is the extensibility model: "Hack around with them. Make them your own"
  (`repo:README.md`); skills.sh copies are owned files, the plugin is read-only. There is no overlay or
  patch mechanism between the two.
- **Preferences go into instruction files, not skill config.** The documented opt-out from grilling in
  rounds back to one question at a time is "a line in your global CLAUDE.md" (`repo:CHANGELOG.md`,
  1.2.0, `a4b2009`); per-user preferences "belong in your `CLAUDE.md` as plain instructions"
  (setup docs page).
- **Cross-skill calls.** Operative dependencies are written as "Call the Skill tool with
  \"grilling\"", one skill per call, never a `../other-skill/FILE.md` link; the phrasing was
  standardized in #878 because a bare `/skill` mention "does not reliably cause it to load", then
  corrected in #880 because six call sites tried to reach user-invoked skills, which the invariant
  forbids (`repo:.agents/invocation.md`; `repo:.changeset/skill-tool-invocation-terminology.md`,
  `repo:.changeset/user-invoked-skill-invocation.md`).
- **No dependency manifest.** 1.0.0 introduced breaking dependencies (`grill-with-docs` needs
  `grilling` and `domain-modeling`; `tdd` needs `codebase-design`) with only a CHANGELOG note ("you must
  install them too") (`repo:CHANGELOG.md`, 1.0.0, `47bde84`). Users merging the skills into their own
  sets ask for a dependency graph (#1084, #775).
- **Composition with project-specific material.** `code-review` reads the repo's own
  `CODING_STANDARDS.md`/`CONTRIBUTING.md`; "A documented repo standard always wins" over the Fowler
  baseline (`repo:skills/engineering/code-review/SKILL.md`). The in-progress `retro` skill says
  `CLAUDE.md`/`AGENTS.md` "should be used incredibly sparingly, usually only for navigation pointers",
  that coding standards belong to the review agent (least context pressure), and that skills are for
  docs whose description should be in context or for user-invoked commands
  (`repo:skills/in-progress/retro/SKILL.md`).
- **The repository dogfoods the approach.** It keeps its own `CONTEXT.md`, `.agents/adr/`,
  `.out-of-scope/` (the knowledge base `triage` writes rejected requests into and reads for prior
  rejections, `repo:skills/engineering/triage/SKILL.md`), and a `CLAUDE.md` of invariants (promoted set
  ↔ README ↔ `plugin.json`, docs page per promoted skill, router re-sync, no em-dashes).

## 6. What is checked mechanically, what is judgment, where humans gate

**Mechanical (code) in or around the suite:**

- `scripts/sync-plugin-version.mjs --check` (version parity) and the changesets release PR.
- `wizard`'s fixed library above the `STAGES` marker, verified "with `bash -n` and `shellcheck`"
  (`repo:CHANGELOG.md`, 1.2.0, `b3376f8`).
- `code-review` fails fast on `git rev-parse <fixed-point>` and an empty diff before dispatching
  subagents (`repo:skills/engineering/code-review/SKILL.md`).
- `misc/git-guardrails-claude-code` (a hook that blocks dangerous git commands) and
  `misc/setup-pre-commit` exist but are not promoted (`repo:skills/misc/README.md`).
- Nothing checks the skill files themselves in CI: the only workflow is the release job. The
  em-dash sweep left unquoted colons in six `description` fields, which made the YAML invalid, "so
  `skills.sh` skipped all six during discovery" until #911 (`repo:.changeset/fix-yaml-frontmatter-colons.md`).

**Judgment (prose) everywhere else**, by design: the setup skill, tracker operations, the flow itself,
the router's coverage and the docs-page rules are all written conventions. The direction of travel is
to move mechanical rules out of prose: `retro` now classifies a coding-standards finding as
"mechanical" (fixed syntactic pattern, banned API, import shape, file location) which "gets a
deterministic check, full stop", reserving `CODING_STANDARDS.md` for "genuine judgement calls", and a
repo with no pre-commit hook and no CI lint/typecheck/test job "is itself a finding"
(`repo:skills/in-progress/retro/SKILL.md`; `repo:.changeset/retro-deterministic-checks.md`, 2026-09-15).
The dictionary makes the same split: an automated check is "Pass/fail, no judgement"; "green checks
mean the asserted properties hold, not that the code is right" (`dict:Automated check`).

**Human gates written into skills:**

| Skill | Gate |
|---|---|
| `grilling` | "Do not act on it until the user confirms you have reached a shared understanding"; decisions are always the user's |
| `tdd` | "write down the seams under test and confirm them with the user. No test is written at an unconfirmed seam" |
| `to-tickets` | "Iterate until the user approves the breakdown", then publish |
| `setup-matt-pocock-skills` | show drafts, "Let them edit before writing" |
| `triage` | "Recommend ... Wait for direction"; confirm role changes before acting |
| `wizard` | stage list confirmed before the script is written; `confirm` before irreversible steps |
| `wayfinder` | HITL tickets resolve only "through that live exchange" (industry note §6) |
| `implement-spec` (beta) | ends by marking the PR "ready for review", no merge |

Sources: the respective `repo:skills/**/SKILL.md` at `c55ee46`. All gates are prose instructions to
the model; none is enforced by a hook or a tracker state.

## 7. Evolution, February to September 2026

Commits per month: Feb 10, Mar 11, Apr 42, May 36, Jun 62, Jul 182, Aug 114, Sep 15 (to 2026-09-18)
(`git log` at `c55ee46`).

| When | Change | Why, per the author |
|---|---|---|
| 2026-02-03 | Initial commit | — |
| 2026-04-30 | Buckets (productivity, engineering, misc, deprecated, personal); `ubiquitous-language` → `grill-with-docs` with `CONTEXT.md` and ADRs; `setup-matt-pocock-skills` makes skills tracker-agnostic | "Skills say 'publish the issues to the issue tracker', but how does it know which one?" (https://www.aihero.dev/skills/skills-changelog-ubiquitous-language-grill-with-docs) |
| 2026-05-11 | `handoff`, `prototype`; supporting info wrapped in XML tags to lower its "loudness" | `grill-with-docs` was "too eager to implement" (https://www.aihero.dev/skills/skills-changelog-handoff-prototype-review-and-writing) |
| 2026-06-17 v1.0.0 | User-/model-invoked taxonomy, shared `grilling`, `domain-modeling`, `codebase-design`; `ask-matt` router | token cost of descriptions; one source for shared discipline (v1 post) |
| 2026-07-08 v1.1.0 | `to-spec`, `to-tickets`, `implement`, `code-review` with Fowler baseline, `wayfinder`, `research`; `tdd` reference-only; grilling confirmation gate and facts/decisions split | "what's the main flow?"; models jumping to implementation or grilling themselves (v1.1 post) |
| 2026-08-05 v1.2.0 | Claude Code plugin in the official marketplace; Codex `openai.yaml` on every skill; docs site; grilling in rounds; `prototype` as one HTML file kept on a `prototype/<name>` branch; six skills removed, `personal/` bucket dropped | "subscribe ... rather than a fork" demand; Codex parity (ADR 0002; v1.2 post) |
| 2026-08-06 v1.2.3 | Harness-neutral subagent wording; `diagnosing-bugs` redacts secrets | followable on Codex (`14bfbbd`); secrets in pasted output (`efce423`) |
| 2026-08-15 | "Call the Skill tool" convention, then its fix for user-invoked targets | hit rate of cross-skill calls; invariant violations (#878, #880) |
| 2026-08-19 | Em-dash sweep; YAML quoting fix | house style; broke skills.sh discovery (#905, #911) |
| 2026-08-21 | `implement-spec` (beta): tickets as a task graph, implementer subagents in background worktrees across the frontier, a merger subagent, one PR | parallel AFK within one spec (`repo:.changeset/add-implement-spec-skill.md`) |
| 2026-08-24 → 09-15 | `retro` (beta), then "mechanical findings → deterministic checks" | environment over instructions (`repo:.changeset/retro-deterministic-checks.md`) |
| 2026-09-17 | `pr` (beta): PR body from the primary source (issue/spec), not the diff; before/after evidence; one-way/two-way door call | fast human review (`repo:.changeset/add-pr-skill.md`) |

Themes: consolidation through renames without aliases; harness neutrality (Codex metadata, neutral
wording, `AGENTS.md` symlink); token economy of always-loaded text; and, most recently, moving toward
parallel AFK execution (`implement-spec`) with a human-review-shaped exit (`pr`), plus deterministic
checks as the preferred home for mechanical rules (`retro`).

**This repository's copy.** The 25 skills under `.harness/skills/` are byte-identical to upstream
between `068b6e0` and `9c9f36c` (2026-08-15 to 2026-08-17) and differ from HEAD `c55ee46` by the later
patches (em-dash sweep, YAML quoting, `wait-what` `CONTEXT-MAP.md`, grilling separators) (directory
diff against `git archive` of each commit). `.harness/harness.lock` records the installer's
`source_revision`, not the upstream `mattpocock/skills` commit, so this provenance had to be recovered
by diffing.

## 8. Commercial and course context, where it explains design

- The skills are the top of a funnel: the README's first link is a newsletter ("~60,000 other devs");
  the site offers a seven-lesson email course and links cohorts "AI Coding for Real Engineers" and
  "Claude Code for Real Engineers" (`repo:README.md`; https://www.aihero.dev/skills). This explains the
  investment in human-facing docs pages, `ask-matt` ("Instead of asking in Discord ... the skill
  itself teaches you", v1 post), and changelog posts per release.
- Docs pages draw "Common questions" from the author's wiki of audience questions and from issues,
  and must "Never name the author" so the page is "a technical document, not a record of who said
  what" (`repo:.agents/writing-docs.md`). Several attributions survived that pass (#870); the
  setup page still quotes "Config is death" without a primary link (unconfirmed where it was said).
- The AI Coding Dictionary is the house vocabulary; docs link the first use of each term to it
  (`repo:.agents/writing-docs.md`). It is where the AFK, HITL, smart zone and factory concepts are
  defined, so it doubles as the suite's glossary.
- `personal/` skills (`edit-article`, `obsidian-vault`) were removed as "tied to my own machine"
  (`repo:CHANGELOG.md`, 1.2.0, `c66bdee`): once the set became a product, personal skills left it.

---

## Implications for agent-workbench

### Adopt

1. **Invocation as a declared, runtime-compiled property.** Keep one field per skill (user-only or
   model-invocable) and emit each runtime's native form (Claude `disable-model-invocation`, Codex
   `policy.allow_implicit_invocation`). Evidence: the split is the suite's primary axis (§2.2); the
   parity bugs in 1.2.0 and 1.2.2 show it breaks when maintained by hand (§3). Our `REGISTRY.md`
   already has an Invocation column; make it the source, not a copy.
2. **Config as pointers plus a few files.** One `## Agent skills`-style block of one-line pointers in
   the instruction file, detailed config in `docs/agents/*.md`, and skills that resolve config through
   the pointer, never through a literal path (§4, `d869d45`). This matches our "thin `CLAUDE.md` bridge
   to `AGENTS.md`" layout.
3. **Canonical roles mapped to project strings.** Our stage vocabulary (idea, spec, tickets,
   implementation, and their states) should be canonical names with a per-project mapping table, the
   way triage roles are (§4).
4. **Hard versus soft dependencies on configuration**, stated per skill (ADR 0001, §4).
5. **Stable and beta channels** (`in-progress/`), explicit promotion invariants, and a changeset-based
   release with a version-parity check (§3).
6. **"It's working if" per skill or stage**: observable signals a human can check without reading the
   skill (§3, `repo:.agents/writing-docs.md`). This is a ready format for our judgment checks.
7. **An `.out-of-scope/` knowledge base and ADRs for the harness itself** (§5), so rejected requests and
   design trade-offs are not re-litigated.

### Avoid

1. **Prose adapters for mechanical operations.** The tracker templates are the suite's main bug source
   (#733, #964, #1118, #635, #855; §4). Our brief says "What is mandatory is checked by code"; tracker
   reads, label changes, blocking edges and frontier queries belong in the Rust CLI, with prose only for
   the open-ended "Other" case (the Sandcastle `init` pattern, §4).
2. **Detecting installation state through the model's context** (#812). The installer knows what it
   installed; it should write that to a lock the skills (and our CLI) can read.
3. **Choosing the instruction file by existence rather than by runtime** (#558). Our installer should
   write `AGENTS.md` as canonical and bridge each runtime explicitly.
4. **Renames without migration.** Four breaking renames each required a manual cleanup (§3). Our
   installer needs a rename map and removal of retired skills.
5. **Two unsynchronized channels.** Plugin users sit on a pinned release while repository users get
   54 unreleased commits (§3). One channel with explicit version pins per project avoids that.
6. **Losing upstream provenance.** Our lock does not record the `mattpocock/skills` commit (§7); a
   vendored upstream must be pinned by its own sha.

### Extend

1. **A completion step and stage exit conditions checked by a command.** `implement` has no completion
   step (industry note §6) and all human gates are prose (§6). Our pipeline adds what the suite
   deliberately leaves out: artifacts and exit conditions validated by the CLI and later hooks/CI.
2. **A declared dependency graph** between skills, validated at install time (#1084, #775, the 1.0.0
   breaking note; §5).
3. **Lint for skill packages**: frontmatter validity (#911), invocation parity across runtimes, router
   coverage of every user-invocable skill, manifest ↔ registry parity. Each of these is a written rule
   in `repo:CLAUDE.md` today (§3, §5).
4. **Project profiles beyond three questions.** The setup skill configures tracker, labels and docs
   layout only (§4); we need stages, checks and gates per project. The tension with "Config is death"
   (§2.7) is real and should be decided, not inherited.
5. **Parallel AFK surfaces.** `implement-spec` (worktrees, frontier, merger subagent, one PR) and the
   factory definitions (§2.5, §7) describe exactly the state our command center has to show; the brief
   keeps launching agents out of scope, so we observe these runs rather than drive them.

### Open questions for the grilling session

1. **Subscribe or fork?** Do we treat `mattpocock/skills` as a pinned, unmodified upstream layer
   (update by reviewed diff) or as a fork we edit? Evidence: the author offers only these two modes
   and no overlay (§3, §5); our copy already lags HEAD (§7).
2. **How much configuration?** Where is the line between our per-project profile and "Config is death"
   (preferences as plain instructions in `AGENTS.md`)? (§2.7, §4)
3. **Tracker operations: CLI or prose?** Do our stage skills call our CLI for tracker work, keeping
   Matt's skills on prose templates, or do we rewrite `docs/agents/issue-tracker.md` to point skills at
   CLI commands? (§4)
4. **Where does our completion step live** relative to `implement`, `code-review` and the beta `pr`
   skill: a separate stage skill, a CLI command, or a hook? (§6, §7)
5. **Which human gates become mandatory checks** (for example seams confirmed, tickets approved,
   PR-not-merge) and which stay prose? (§6)
6. **Runtime matrix.** Claude Code and Codex are first-class upstream; skills.sh covers ~50 agents
   through `.agents/skills`. Which runtimes do we test, and do we ship native plugins or files only?
   (§3)
7. **Global versus per-repo configuration.** Upstream has no user-level mode (§2.7); our developer
   machine installation (brief outcome 5) implies one. What is global, what is per project?
8. **Beta channel and promotion rules for our own skills**, and whether our skills follow the
   user-invoked/model-invoked and "Call the Skill tool" conventions so they compose with the suite.
   (§2.2, §5)
