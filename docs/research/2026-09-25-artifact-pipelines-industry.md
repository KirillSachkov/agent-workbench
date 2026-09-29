# Artifacts instead of chat: how leading coding-agent practices structure development

Research date: 2026-09-25. Method: primary sources (vendor engineering blogs, official
documentation, repository sources). Repositories were read at specific commits (listed below).
Anything that could not be confirmed from a primary source is marked **"unconfirmed"**.

Access limits in this session: `openai.com` and `developers.openai.com` could not be opened directly
(network block), so the OpenAI post was read from a Wayback Machine copy
(`web.archive.org/web/2026id_/https://openai.com/index/harness-engineering/`), and the ExecPlans
article from the source in `github.com/openai/openai-cookbook` (`articles/codex_exec_plans.md`).
The Anthropic page "Building effective agents" did not load in this session (connection dropped),
so below it has only claims marked "unconfirmed".

The main distinction running through all sections: **what code checks** (a script, schema, hook,
CI, test) versus **what rests only on prompt text** (asking the model in prose not to touch a field).
Wherever the sources make it visible, this is noted explicitly.

---

## 1. Anthropic

### 1.1 "Effective harnesses for long-running agents" (Justin Young, November 26, 2025)

Source: https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents ;
code: https://github.com/anthropics/claude-quickstarts/tree/main/autonomous-coding
(`prompts/initializer_prompt.md`, `prompts/coding_prompt.md`, `agent.py`, `progress.py`, `security.py`).

**(a) Artifacts and fields.**

- `feature_list.json` — an array of entries of this shape (verbatim from the post):
  ```json
  {
    "category": "functional",
    "description": "New chat button creates a fresh conversation",
    "steps": ["Navigate to main interface", "Click the 'New Chat' button", "..."],
    "passes": false
  }
  ```
  In the claude.ai clone demo — "over 200 features", all initially "failing". The initializer prompt
  requires: at least 200 entries, categories `functional` and `style`, "At least 25 tests MUST have 10+
  steps", ordering by priority, "ALL tests start with "passes": false".
- `claude-progress.txt` — a free-form log of "what agents have done"; per `coding_prompt.md`, after
  each session append: what was done, which tests were closed, problems found, what to do next, a
  status such as "45/200 tests passing".
- `init.sh` — a script that starts the dev server so the next session does not have to guess how to
  bring the app up.
- Git: the initializer's first commit ("Initial setup: feature_list.json, init.sh, and project structure"),
  then one commit per feature with a descriptive message.
- Verification screenshots (`verification/` is mentioned in the commit message template in `coding_prompt.md`).

**(b) Lifecycle, ownership, validation.**

- The initializer is the same model and the same harness, but a different first prompt (footnote 1 of
  the post: "separate agents … only because they have different initial user prompts"). In `agent.py`
  the prompt choice is made by code: `is_first_run = not tests_file.exists()` — no `feature_list.json`
  means the initializer runs.
- Who may change `feature_list.json`: only the `passes` field, only `false → true`.
  In the post: "It is unacceptable to remove or edit tests because this could lead to missing or buggy
  functionality". In the initializer prompt: "IT IS CATASTROPHIC TO REMOVE OR EDIT FEATURES IN FUTURE
  SESSIONS". In `coding_prompt.md`: "YOU CAN ONLY MODIFY ONE FIELD: "passes"", "NEVER: Remove tests,
  Edit test descriptions, Modify test steps, Combine or consolidate tests, Reorder tests",
  "ONLY CHANGE "passes" FIELD AFTER VERIFICATION WITH SCREENSHOTS".
- Format choice: "we landed on using JSON … the model is less likely to inappropriately change or
  overwrite JSON files compared to Markdown files".
- Session start protocol: `pwd` → read the git log and progress → pick the highest-priority
  unpassed feature → run `init.sh` → run a basic e2e scenario before new work (to catch a broken
  state left by the previous session). Session end: commit + progress update.
- Feature verification: browser automation via Puppeteer MCP "as a human user would".
- Surviving across contexts: state lives entirely in files and git; a new session recovers from
  `claude-progress.txt` + `git log` + `feature_list.json`.

**(c) Code versus prose.** Enforced by code: initializer selection by file presence; progress counting
(`progress.py`, `count_passing_tests` simply counts `passes: true`); a bash-command allowlist via a
security hook (`security.py`, `ALLOWED_COMMANDS`, `bash_security_hook`). **Not enforced by code**:
immutability of descriptions and steps, the ban on deleting entries, the requirement of screenshots
before `passes: true`. These rest only on prompt text; the quickstart has no JSON schemas, diff checks,
or a hook that forbids editing other fields (verified from the sources).

**(d) Effectiveness.** The post has no quantitative metrics; there is a qualitative table of four
failures and their remedies (premature "done", dirty state, a feature marked without verification,
time spent starting the app).

**(e) Limitations.** In the post: Puppeteer MCP cannot see browser `alert` modals, and such features
came out rougher; it is an open question whether a multi-agent setup is better; the example is
optimized for full-stack web.

### 1.2 "Harness design for long-running application development" (Prithvi Rajasekaran, March 24, 2026)

Source: https://www.anthropic.com/engineering/harness-design-long-running-apps

**(a) Artifacts.** Three agents — planner, generator, evaluator.
- The product specification from the planner: from a 1–4 sentence prompt to a full spec. In the post's
  appendix, an example ("RetroForge"): Overview, numbered Features, and in each feature "User Stories:
  As a user, I want to …" and "Project Data Model". The planner deliberately stays at the product level
  and high-level design: errors in the detailed technical part "would cascade into the downstream
  implementation".
- **Sprint contract**: before each sprint the generator proposes what it will build and how to verify
  it; the evaluator reviews; iterate until agreement. "Communication was handled via files: one agent
  would write a file, another agent would read it and respond either within that file or with a new
  file". Contracts are granular: "Sprint 3 alone had 27 criteria covering the level editor".
- The evaluator's report: PASS/FAIL per contract criterion with specifics (example: "FAIL — Delete key
  handler at LevelEditor.tsx:892 requires both selection and selectedEntityId …").
- Grading criteria with thresholds: "Each criterion had a hard threshold, and if any one fell below it, the
  sprint failed".

**(b) Mechanics.** The evaluator tests the live application via Playwright MCP (UI, API, DB state).
The evaluator was calibrated with few-shot examples. Context resets (clean slate + handoff artifact)
were needed for Sonnet 4.5 because of "context anxiety"; with Opus 4.5 they were removed, with Opus 4.6
sprints were removed too, and the evaluator was moved to a single pass at the end.

**(c) Code versus prose.** Criterion thresholds and sprint pass/fail are decided by an LLM evaluator
(inference, not a deterministic check). The post does not show the contract file format — it is
**unconfirmed** whether the contract has a schema.

**(d) Effectiveness.** Solo run: 20 min / $9; full harness: 6 h / $200, with quality "immediately
apparent" as better. V2 (Opus 4.6): 3 h 50 min / $124.70, with a per-phase breakdown in the post's table.

**(e) Limitations.** "Out of the box, Claude is a poor QA agent": it found defects and talked itself
into ignoring them; it tested superficially. The post's conclusion: the evaluator "is not a fixed
yes-or-no decision" — it pays off only on tasks beyond what the model does on its own; each new model
should be re-tested and the parts of the harness that no longer work should be removed.

### 1.3 "Building a C compiler with a team of parallel Claudes" (February 5, 2026)

Source: https://www.anthropic.com/engineering/building-c-compiler

- The task-locking artifact: "Claude takes a "lock" on a task by writing a text file to
  current_tasks/ (e.g. … current_tasks/parse_if_statement.txt)". A conflict between two agents is
  resolved by git ("git's synchronization forces the second agent to pick a different one"). After
  the work: pull, merge, push, release the lock.
- State for new sessions: "extensive READMEs and progress files that should be updated frequently".
- The test harness as the main validator: "the task verifier is nearly perfect, otherwise Claude will
  solve the wrong problem"; output is compact, details go to a log; error lines carry `ERROR` on the
  same line so they can be found with grep; a `--fast` mode (a deterministic 1%/10% sample per agent).
  CI was added later so that "new commits can't break existing code"; GCC as an oracle for the Linux kernel.
- Numbers: 16 agents, ~2000 Claude Code sessions, ~$20,000, 100,000 lines of Rust, builds Linux 6.9 on
  x86/ARM/RISC-V, 99% on most compiler test suites.
- A limitation from the post: "it is easy to see tests pass and assume the job is done, when this is rarely
  the case".

### 1.4 Claude Code: subagents, hooks, skills as producers and guards of artifacts

Sources: https://code.claude.com/docs/en/hooks , https://code.claude.com/docs/en/sub-agents ,
https://code.claude.com/docs/en/skills , https://code.claude.com/docs/en/memory

- **Hooks are the only deterministic layer.** Exit code 2 blocks the event: `PreToolUse`
  blocks the tool call; `Stop` — "Prevents Claude from stopping, continues the conversation";
  `SubagentStop` — does not let the subagent finish; `TaskCreated` rolls back task creation;
  `TaskCompleted` — "Prevents the task from being marked as completed"; `PreCompact` blocks
  compaction. `PostToolUse` cannot block (the tool has already run) — it only shows stderr to the
  model. A hook's JSON response is checked against a schema; invalid JSON with a code other than 2 is a
  non-blocking error, and exit 1 does **not** block (an important trap). Hence the "validator-as-hook"
  pattern: a script checks the artifact (schema, links, presence of a field) and returns 2 in
  `Stop`/`TaskCompleted`.
- **Subagents**: frontmatter `name`, `description` (required), `tools`, `disallowedTools`, `model`,
  `permissionMode`, `maxTurns`, `skills`, `hooks`, `memory` (`user|project|local`), `background`,
  `isolation: worktree`. A subagent works in its own context and returns its result as a report;
  a subagent's hooks live only while it runs, and `Stop` inside it becomes `SubagentStop`.
  The documentation has no schema for a subagent's report — the return is text.
- **Skills**: `SKILL.md` with frontmatter (`description`, `disable-model-invocation`, `user-invocable`,
  `allowed-tools`, `disallowed-tools`, `context: fork`, `hooks`, `paths`, `metadata`, and others). A skill
  can register hooks for the rest of the session — this is how a "procedure" gets a deterministic check.
  After auto-compaction, recently invoked skills are re-attached within a token budget.
- **Surviving context**: the root `CLAUDE.md` is re-read from disk after `/compact`
  ("Project-root CLAUDE.md survives compaction"); instructions given only in the conversation are lost.

### 1.5 "The AI-Native SDLC playbook" (claude.com, August 21, 2026)

Source: https://claude.com/blog/the-ai-native-sdlc-playbook

This is the closest analogue found to "Claude Code for product development". A separate "Claude Code
for product development technical report" was **not found** in primary sources — it is **unconfirmed**
that such a document exists (there is a PDF "How Anthropic teams use Claude Code" and a "2026 Agentic
Coding Trends Report"; their content was not checked in this session).

**(a) Artifact chain.** "Each stage ends by writing one to version control (including intent.md,
spec.md, plan.md, the diff and its tests, the PR with its review findings, and the incident record)
and the next stage begins by reading it".
- `intent.md` — the idea author's proto-spec: what is needed, why, under what constraints.
- `spec.md` — requirements + design, in one session, with "areas of concern" flags; committed next to
  `intent.md` ("The file pair records what was asked for and what was decided").
- `plan.md` — an example from the post:
  ```
  # Plan: claims status self-service (from intent.md 2026-06-02)
  ## Files that change
  ## Order of work
  ## Risks
  ## Proof
  ```
- A PR with review findings; an incident record that writes the next `intent.md`.

**(b) Mechanics.** Committing an artifact triggers the next stage ("An accepted intent.md triggers the
requirements and design pass, an approved spec.md triggers plan mode, a merged PR triggers the
pipeline…"). Plan mode itself does not allow editing files until the plan is accepted. "When implementation departs from
the plan, update plan.md in the same commit. Consider using a hook to enforce synchronization". PR
review "checks the eventual diff against" `plan.md`. For external systems: declare one source of truth
per artifact; the minimum bar — "All artifacts note the record ID and all legacy records contain the
commit SHA of the markdown file".

**(c) Code versus prose.** The post explicitly separates the two: "A skill is a control, though an advisory one …
A policy that must always hold needs something deterministic behind the skill, such as a hook".
Hook examples: blocking edits to test files during a fix ("an agent fixing code must not be able to
weaken the check on that code"), blocking edits to migrations/infrastructure without a change ticket, hooks as
approval gates at release. A regression run of the agent configuration in CI "on any change to CLAUDE.md,
skills or hooks". Syncing `plan.md` with the diff is only a recommendation (the hook is "consider").

**(d) Effectiveness.** No numbers; git-derived metrics are proposed: time between the `intent.md` and
`spec.md` commits; the number of `spec.md` commits after the first `plan.md` (requirement rework); the share of changes
whose diff still matches `plan.md`.

**(e) Limitations.** The post acknowledges that in regulated organizations the source of truth often stays
in Jira/ServiceNow, and markdown artifacts become copies.

### 1.6 "Building effective agents" (Anthropic)

URL: https://www.anthropic.com/engineering/building-effective-agents . The page did not
load in this session, so the content is **unconfirmed**. From memory: workflow patterns (prompt chaining with
programmatic "gate" checks between steps, evaluator-optimizer) and the principle of relying on "ground truth
from the environment". Citing it without re-verification is not recommended.

---

## 2. OpenAI

### 2.1 "Harness engineering: leveraging Codex in an agent-first world" (Ryan Lopopolo, February 11, 2026)

Source: https://openai.com/index/harness-engineering/ (read via a Wayback Machine copy).

**(a) Artifacts and structure.** "Repository knowledge [is] the system of record". `AGENTS.md` of ~100
lines — "the table of contents", not an encyclopedia. The tree from the post:
```
AGENTS.md
ARCHITECTURE.md
docs/
├── design-docs/ (index.md, core-beliefs.md, …)
├── exec-plans/ (active/, completed/, tech-debt-tracker.md)
├── generated/ (db-schema.md)
├── product-specs/ (index.md, new-user-onboarding.md, …)
├── references/ (design-system-reference-llms.txt, nixpacks-llms.txt, uv-llms.txt, …)
├── DESIGN.md, FRONTEND.md, PLANS.md, PRODUCT_SENSE.md,
├── QUALITY_SCORE.md, RELIABILITY.md, SECURITY.md
```
Design docs are cataloged "including verification status"; the quality document "grades each product
domain and architectural layer, tracking gaps over time". "Plans are treated as first-class artifacts.
Ephemeral lightweight plans are used for small changes, while complex work is captured in execution
plans with progress and decision logs that are checked into the repository".

**(b) Mechanics.** The failure of "one big AGENTS.md" is described point by point: it crowds the task out of context,
"too much guidance becomes non-guidance", it rots, it is "hard to verify … mechanical checks (coverage,
freshness, ownership, cross-links)". Hence progressive disclosure. Everything not in the repository
(Google Docs, Slack, people's heads) "doesn't exist" for the agent. A recurring "doc-gardening" agent looks for
stale documents and opens fix-up PRs. Background Codex tasks "scan for deviations, update
quality grades, and open targeted refactoring pull requests" ("garbage collection").

**(c) Code versus prose.** "We enforce this mechanically. Dedicated linters and CI jobs validate that
the knowledge base is up to date, cross-linked, and structured correctly". Architecture: layers
`Types → Config → Repo → Service → Runtime → UI`, cross-cutting concerns only through `Providers`, "enforced
mechanically via custom linters (Codex-generated) and structural tests"; structured logging, schema and
type naming, and file size limits are checked statically. "Because the lints are custom, we
write the error messages to inject remediation instructions into agent context". The escalation rule:
"When documentation falls short, we promote the rule into code". Specific schemas/scripts for documentation
checks are **not shown in the post — unconfirmed** what exactly they check beyond what is listed.

**(d) Effectiveness.** Five months, "0 lines of manually-written code", ~1 million lines, ~1500 PRs,
3 engineers (later 7), "3.5 PRs per engineer per day", single Codex runs "upwards of six hours".

**(e) Limitations.** "Minimal blocking merge gates", flaky tests are handled by rerunning — "would be
irresponsible in a low-throughput environment". Codex "replicates patterns that already exist …
even uneven or suboptimal ones"; the team used to spend every Friday (20% of the week) cleaning up "AI
slop". They do not know how architectural integrity will behave over a horizon of years. Autonomy "should
not be assumed to generalize without similar investment".

### 2.2 ExecPlans / `PLANS.md` (OpenAI Cookbook)

Source: https://developers.openai.com/cookbook/articles/codex_exec_plans ; source file
`openai/openai-cookbook/articles/codex_exec_plans.md` (last edits 2026-01-13…15, Vaibhav Srivastav).

**(a) Fields.** Mandatory "living" sections: `Progress`, `Surprises & Discoveries`, `Decision Log`,
`Outcomes & Retrospective` ("These are not optional"). The skeleton also contains `Purpose / Big Picture`,
`Context and Orientation`, `Plan of Work`, `Concrete Steps`, `Validation and Acceptance`,
`Idempotence and Recovery`, `Artifacts and Notes`, `Interfaces and Dependencies`. Entry formats:
```
- [x] (2025-10-01 13:00Z) Example completed step.
- [ ] Example partially completed step (completed: X; remaining: Y).

- Observation: …
  Evidence: …

- Decision: …
  Rationale: …
  Date/Author: …
```

**(b) Mechanics.** Attached as a section in `AGENTS.md`: "When writing complex features or significant
refactors, use an ExecPlan (as described in .agent/PLANS.md) from design to implementation".
Self-containment: the reader is a "complete beginner … they have only the current working tree and the
single ExecPlan file"; "it should always be possible to restart from _only_ the ExecPlan and no other
work". During implementation, do not ask the user for "next steps", update the sections at every
stopping point, "commit frequently". Each milestone is "independently verifiable". When the plan changes,
"write a note at the bottom of the plan describing the change and the reason why". The format is strict:
a single fenced `md` block, checklists are allowed only in `Progress`. Against "code for the sake of a definition":
"must produce a demonstrably working behavior, not merely code changes to "meet a definition"".

**(c) Code versus prose.** Everything is prose. The article has no ExecPlan structure validator, section
linter, or CI check. (The harness engineering post talks about knowledge-base linters, but the link to
ExecPlan is not spelled out there — **unconfirmed**.)

**(d) Effectiveness.** "very similar to one that has enabled Codex to work for more than seven hours
from a single prompt" — a single claim without a methodology.

**(e) Limitations.** "ExecPlan" is an arbitrary term, "Codex has not been trained on it";
the user is encouraged to adapt the sections.

---

## 3. GitHub spec-kit

Source: https://github.com/github/spec-kit , commit `adbd62a` (2026-09-24), version 1.0.11 per
`CHANGELOG.md`. Files: `templates/*.md`, `templates/commands/*.md`, `scripts/bash/*`,
`workflows/README.md`, `workflows/ARCHITECTURE.md`, `workflows/speckit/workflow.yml`.

**(a) Artifacts and fields.**
- `.specify/memory/constitution.md` — project principles; footer
  `**Version**: [CONSTITUTION_VERSION] | **Ratified**: … | **Last Amended**: …`. The constitution command
  requires semver (MAJOR/MINOR/PATCH, PATCH — "Clarifications, wording, typo fixes") and
  "Sync Impact Report as an HTML comment at the top of the constitution file".
- `spec.md` (`templates/spec-template.md`): header `Feature Branch: [###-feature-name]`, `Created`,
  `Status: Draft`, `Input`; "User Scenarios & Testing *(mandatory)*" — stories with priority P1/P2/P3,
  `Why this priority`, `Independent Test`, Given/When/Then; "Edge Cases"; "Requirements *(mandatory)*"
  with `FR-001…` ("System MUST …"), the marker `[NEEDS CLARIFICATION: …]`; "Key Entities"; "Success Criteria
  *(mandatory)*" with `SC-001…`, "technology-agnostic and measurable"; "Assumptions".
- `plan.md`: `Technical Context` (Language/Version, Primary Dependencies, Testing, Target Platform,
  Performance Goals, Constraints, Scale/Scope — each field may be `NEEDS CLARIFICATION`);
  "Constitution Check — *GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*";
  "Complexity Tracking — Fill ONLY if Constitution Check has violations that must be justified".
  Derived files: `research.md`, `data-model.md`, `quickstart.md`, `contracts/`.
- `tasks.md`: format `[ID] [P?] [Story] Description`, for example
  `- [ ] T012 [P] [US1] Create [Entity1] model in src/models/[entity1].py`; phases Setup → Foundational
  → one phase per user story (P1 = MVP) → Polish. `[P]` = parallel (different files, no dependencies).
- `checklists/*.md`: "CRITICAL CONCEPT: Checklists are UNIT TESTS FOR REQUIREMENTS WRITING" — they check
  the quality of requirements, not the implementation. `checklists/requirements.md` is the built-in spec
  quality checklist (maintained by `specify`/`clarify`); custom ones are "reviewer-owned", and the checklist command "MUST NOT mark
  generated items `[x]`".

**(b) Lifecycle, links, validation.**
- Commands: `/speckit-constitution → specify → clarify → plan → tasks → analyze → implement`, plus
  `checklist`, `converge`, `taskstoissues` and extensions (bug-assess/fix/test, assess-*).
- Binding to a feature: `create-new-feature.sh` creates a branch `NNN-short-name` and the feature directory; the active
  feature is resolved via `SPECIFY_FEATURE` or `.specify/feature.json` (`common.sh`). There is no binding
  of an artifact to a commit SHA (not found).
- `specify`: at most 3 `[NEEDS CLARIFICATION]` markers ("LIMIT: Maximum 3"), a self-check against
  `checklists/requirements.md` for up to 3 iterations, then a warning.
- `analyze`: "STRICTLY READ-ONLY", a cross-artifact check of `spec.md`/`plan.md`/`tasks.md` against the
  constitution. Passes: duplicates, ambiguity (vague adjectives, unresolved `TODO`/`???`),
  underspecification, constitution conflict (always CRITICAL — "not dilution, reinterpretation, or
  silent ignoring"), coverage gaps ("Requirements with zero associated tasks", "Tasks with no mapped
  requirement"), inconsistency (terminology drift). The report is a table `ID | Category | Severity |
  Location(s) | Summary | Recommendation` + a coverage table; up to 50 findings.
- `implement`: first scans `checklists/` and counts `- [ ]`/`- [x]`; with open items
  "STOP and ask … proceed anyway? (yes/no)"; marks completed tasks `[X]` in `tasks.md`.
- `converge`: reconciles the code with spec/plan/tasks and appends missing work as new tasks in `tasks.md`.
- The new **workflow engine** (`specify workflow run speckit`): YAML steps with `type: gate`
  ("Review the generated spec before planning", `options: [approve, reject]`, `on_reject: abort`);
  state is written after each step to `.specify/workflows/runs/{run_id}/state.json`, the log to
  `log.jsonl` (append-only), and `specify workflow resume <run_id>` continues from the pause point.

**(c) Code versus prose.** By code: existence of the required files (`check-prerequisites.sh` fails with
`ERROR: plan.md not found`), numbering of features and branches, resolution of the active feature, the workflow engine with
persist/resume and gate pauses. **By prose (executed by the LLM)**: all of `analyze` (mapping tasks to requirements
is done "by keyword / explicit reference patterns" by the model itself), the checklist counting in `implement`,
the marker limit, adherence to the constitution. The repository has no schema/linter for `spec.md` (not found).

**(d) Effectiveness.** No controlled data in the repository (not found).

**(e) Criticism.** Böckeler (section 5.1): "a LOT of markdown files … repetitive … very verbose and
tedious to review"; the agent took descriptions of existing classes from research as a new specification and
generated duplicates; "I'd rather review code than all these markdown files".

---

## 4. AWS Kiro

Sources: https://kiro.dev/docs/specs/ , https://kiro.dev/docs/specs/feature-specs/ ,
https://kiro.dev/docs/specs/bugfix-specs/ , https://kiro.dev/docs/specs/correctness/ (updated
2026-08-04), https://kiro.dev/docs/specs/analyze-requirements/ (updated 2026-09-02),
https://kiro.dev/docs/specs/best-practices/ , https://kiro.dev/docs/steering/ , https://kiro.dev/docs/hooks/

**(a) Artifacts.** A spec = three files:
- `requirements.md` (or `bugfix.md`): user stories and acceptance criteria in EARS:
  `WHEN [condition/event] THE SYSTEM SHALL [expected behavior]`. Claimed properties: clarity,
  testability ("Each requirement can be directly translated into test cases"), traceability.
- `bugfix.md` instead of requirements: `Current Behavior (Defect)`, `Expected Behavior (Correct)`,
  `Unchanged Behavior (Regression Prevention)`.
- `design.md`: architecture, sequence diagrams, implementation considerations.
- `tasks.md`: discrete tasks; per Böckeler, tasks "trace back to the requirement numbers".
- Steering: `.kiro/steering/*.md` (starter `product.md`, `tech.md`, `structure.md`); frontmatter
  `inclusion: always | fileMatch | manual`; `AGENTS.md` is also read (without inclusion modes).
- Hooks: `.kiro/hooks/<id>.json`, fields `trigger` (PascalCase), `matcher` (regex), an action `command`
  or an agent prompt.

**(b) Mechanics.** Two variants: Requirements-First (Requirements → Design → Tasks) and Design-First
(Design → Requirements → Tasks). Before design there is an optional **Analyze Requirements**: it looks for logical
contradictions, ambiguities ("large files", "fast response times"), conflicting constraints,
implicit assumptions, missed edge cases; "takes minutes, not seconds"; questions go to the chat, and the answers
edit `requirements.md`. Task execution: the UI shows in-progress/completed status; "run all
tasks" builds a dependency graph and runs independent tasks in parallel. Changes: editing
requirements/design and then **Sync Files** in `tasks.md` creates tasks for the new requirements. Specs are
"designed to be version-controlled"; for multiple teams — git submodules.
**Property-based testing**: Kiro "extracts properties from your EARS-formatted requirements …
generates hundreds or thousands of random test cases"; hovering over a property shows the link to the
requirement and the task; PBT is "optional by default"; on failure — shrinking to a minimal counterexample.
Hooks: `Pre Tool Use`, `Prompt Submit`, **`Pre Task Execution` (before a spec task) can
block**; `Post Task Execution`, `File Save`, `Agent Stop` cannot. File triggers react
only to the agent's changes, not to manual ones.

**(c) Code versus prose.** By code: generated property-based tests (real executable tests
tied to a requirement), hooks with blocking, the task dependency graph in the IDE. By prose/LLM: EARS itself
(the documentation has no formal EARS parser or linter — it is **unconfirmed** that one exists),
Analyze Requirements (LLM reasoning), Sync Files.

**(d) Effectiveness.** No quantitative data in the documentation.

**(e) Limitations.** The documentation on PBT: "evidence of correctness, not a proof"; a weak property
will pass with wrong behavior; not everything reduces to properties. Böckeler: on a small bug Kiro is "a
sledgehammer to crack a nut": 4 user stories and 16 acceptance criteria; Kiro is "mostly spec-first", with no
clear strategy for maintaining the spec after the task.

---

## 5. Other methods of 2025–2026

### 5.1 Birgitta Böckeler (Thoughtworks) on martinfowler.com — the critical frame

- "Understanding Spec-Driven-Development: Kiro, spec-kit, and Tessl" (October 15, 2025):
  https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html . Three levels: **spec-first**
  (a spec before code for a task), **spec-anchored** (the spec lives on and is maintained with the feature),
  **spec-as-source** (a human edits only the spec). "All SDD approaches … are spec-first, but not all
  strive to be spec-anchored or spec-as-source". Criticism: excessive for small tasks; reviewing markdown
  is harder than reviewing code; "False sense of control?" — the agent both ignores instructions and "go way overboard
  because it was too eagerly following instructions (e.g. one of the constitution articles)";
  non-determinism of generation even from the same low-level spec; a parallel with MDD, which
  "never took off for business applications".
- "Harness engineering for coding agent users" (April 2, 2026):
  https://martinfowler.com/articles/harness-engineering.html . Guides (feedforward) and sensors
  (feedback); computational (tests, linters, type checkers — "results are reliable") versus
  inferential (LLM review, "more non-deterministic"). Sensors are strong when the signal is optimized for the LLM: "custom
  linter messages that include instructions for the self-correction". The main gap is the **behaviour
  harness**: usually it is a "functional specification" as the guide and an AI-generated test suite as the
  sensor; "puts a lot of faith into the AI-generated tests, that's not good enough yet". "Correctness
  is outside any sensor's remit if the human didn't clearly specify what they wanted".

### 5.2 BMAD Method

Source: https://github.com/bmad-code-org/BMAD-METHOD (HEAD as of 2026-09-25; files
`skills/bmad-preview-ticketing/assets/story-template.md`, `skills/bmad-sprint-planning/scripts/sprint_plan.py`,
`skills/bmad-architecture/scripts/lint_spine.py`, `skills/bmad-architecture/assets/spine-template.md`,
`skills/bmad-build/sync-sprint-status.md`, `skills/bmad-build/compile-epic-context.md`).

**(a) Artifacts.**
- A story/ticket file with YAML frontmatter: `id` (from `tickets.toml`), `type: story`, `title`, `parent`,
  `covers` (ids of the epic's requirements that the story closes), `after` (prerequisites:
  `<sibling id>`, `<epic id>.<entry id>`, `epic-<slug>`), `assignee`, `refined: false`, `hitl: false`,
  `risk`, `estimate`; `status` is written by the build (`draft | ready-for-dev | in-progress | in-review | done |
  blocked`), `tracker_status` — tracker synchronization. Body: `Description`, `Acceptance Criteria`
  (a single `Verify: …` line or numbered Given/When/Then), `Boundaries` ("Must not change: …
  name behavior, not files"), `References`, `Notes` (`Decision:`, `Assumption:`, `Open question:`),
  `Plan` ("Filled in by the coding agent; never sent to a tracker").
- `sprint-status.yaml`: story keys like `3-2-foo`, story statuses `backlog → ready-for-dev →
  in-progress → review → done`, epic `backlog → in-progress → done`, retrospective `optional → done`,
  `action_items`, `last_updated`.
- `ARCHITECTURE-SPINE.md`: decision blocks `### AD-n — {decision}` with fields `Binds`, `Prevents`, `Rule`
  ("stable ascending id (never reused/renumbered)"), a `## Stack` table with versions.
- `epic-<N>-context.md`: epic context for the developer assembled from PRD/architecture/UX (Goal, Stories,
  Requirements & Constraints, Technical Decisions, UX…) — this is the current analogue of document "sharding".
  No separate shard-doc command was found in the current HEAD; sharding of the PRD/architecture is a BMAD v4 trait,
  and in the current version it is **unconfirmed**.

**(b)+(c) Mechanics and code.** This has the most deterministic code among SDD methods:
- `sprint_plan.py` ("Parse epic files and deterministically generate or refresh sprint-status.yaml"):
  subcommands `generate`, `status`, `validate`; atomic write (temp, fsync, `os.replace`) and rollback
  "if post-write validation fails"; a check for "parseable, recognized keys, legal statuses, well-formed
  action_items"; a drift report (`in_sync`, `illegal`, `orphans`); statuses are not downgraded except by
  an explicit repair via `--set`. The separation principle: "The LLM decides *which* files are epics (discovery
  is judgment); this script owns everything after that decision".
- `lint_spine.py`: "LLMs miscount IDs and miss literal placeholders; a grep does not". It checks
  placeholders (`TBD`, `TODO`, "similar to AD-n", unfilled `{token}`), duplicates and non-monotonicity of
  `AD-n`, missing `Binds/Prevents/Rule`, stack rows without a version. Output is JSON, exit is always 0,
  and the decision is made by the Reviewer Gate.
- `sync-sprint-status.md` — a procedure for the model: set the story status, move the epic to
  `in-progress`, do not downgrade a status, update `last_updated`.
- The scripts have unit tests (`scripts/tests/test_*.py`).

**(d)** No effectiveness metrics in the repository (not found). **(e)** The method changes a lot between
versions (legacy v6 statuses `drafted`, `contexted` are normalized by the script), which is itself a risk
for long-lived artifacts.

### 5.3 Tessl

Sources: https://github.com/tesslio/spec-driven-development-tile (`docs/spec-format.md`,
`scripts/validate-specs.sh`, `scripts/check-spec-links.sh`, `.github/workflows/ci.yaml`, `tile.json`
v2.0.1); https://docs.tessl.io/codifying-and-enforcing-your-skill-standards/verifiers-overview.md ;
Böckeler (5.1).

- Spec format: a `*.spec.md` file, YAML frontmatter `name`, `description`, `targets` (paths/globs
  of the described code; "All specs must have at least one target"), a public API block, requirements with
  links to tests `[@test] ../tests/…`.
- Code: `validate-specs.sh` checks the extension and the presence of frontmatter with `name/description/targets`;
  `check-spec-links.sh` — "that [@test] links and targets in .spec.md files point to existing files";
  CI requires a tile version bump. Eval scenarios for the process itself (`evals/spec-drift-after-refactor`,
  `evals/skip-spec-pushback`, `evals/trivial-change-exception`, and others).
- Per Böckeler, Tessl is the only one of the three aiming at spec-anchored and trying spec-as-source;
  code is marked `// GENERATED FROM SPEC - DO NOT EDIT`, tags `@generate`/`@test`.
- The 2026 shift: the former docs.tessl.io pages about spec-driven development return 404; the current
  documentation is about the skills/plugins registry, evals and **verifiers**: "LLM-as-judge check that compares
  committed files with an invariant stored as JSON", scope and level (`info|warn|error`) in
  `tessl.json`, in CI `error` gives a non-zero exit and blocks the PR; "Prefer a linter or test when one can
  enforce the invariant".

### 5.4 Factory.ai

Sources: https://docs.factory.ai/cli/user-guides/specification-mode ,
https://docs.factory.ai/missions/overview , https://docs.factory.ai/missions/planning ,
https://factory.ai/news/using-linters-to-direct-agents (Alvin Sng, September 5, 2025).

- Spec Mode: read-only exploration, then `ExitSpecMode` asks for approval. The option "Save spec as
  Markdown" writes the approved plan to `.factory/docs/YYYY-MM-DD-slug.md`.
- Missions: the plan = features grouped into milestones; "Validation workers run at the end of each
  milestone"; estimate `total runs ≈ #features + 2 * #milestones`; QA by launching the app with a single
  command and writing logs to disk. The storage format of a mission plan is **unconfirmed** in the documentation.
- Linters as law: AGENTS.md explains the "why", rules are encoded in lint with "clear severity,
  autofix, and waiver policies"; the same rules on save, pre-commit, CI, PR bots and in the agent's toolchain;
  "Achieving "lint green" becomes the definition of "Done"". Categories: grep-ability,
  glob-ability, architectural boundaries, documentation signals. Linters as a "migration engine".

### 5.5 Sourcegraph Amp

Sources: https://ampcode.com/news/handoff (2025-10-23), https://ampcode.com/news/neo (May 2026).
Handoff replaced compaction: the user states a goal, Amp generates a draft prompt for the new
thread and a list of relevant files, and the human edits before sending. In 2026 Amp turned back around:
"Compaction now runs automatically when the context window is 90% full … So handoff is out".
Takeaway for the topic: even product handoff artifacts between sessions turned out to depend on the model and
were dropped once models got better at handling compaction.

### 5.6 Cognition / Devin Playbooks

Source: https://docs.devin.ai/product-guides/creating-playbooks . A playbook is "like a custom system
prompt for a repeated task". Sections: outcome, `Procedure` (setup → task → delivery, each step with
an action verb), `Specifications` ("Describe postconditions — what should be true after Devin is
done"), `Advice`, `Forbidden Actions`, `Required from User`. This is a reusable procedure artifact;
the documentation has no mechanical check of postconditions (not found).

### 5.7 Geoffrey Huntley, Ralph (the original source of the loop)

Source: https://ghuntley.com/ralph/ (July 2025). `while :; do cat PROMPT.md | claude-code ; done`;
the loop's artifacts are `specs/*` and `@fix_plan.md` ("a bullet point list sorted in priority"), "One item per
loop"; "phase two: backpressure" — tests/build as back pressure.

---

## 6. Matt Pocock: `mattpocock/skills` and Ralph

Sources: https://github.com/mattpocock/skills (HEAD `c55ee46`, 2026-09-18):
`skills/engineering/to-spec/SKILL.md`, `skills/engineering/to-tickets/SKILL.md`,
`skills/engineering/triage/AGENT-BRIEF.md`, `skills/engineering/wayfinder/SKILL.md`,
`skills/engineering/setup-matt-pocock-skills/triage-labels.md`, `docs/engineering/implement.md`,
`CHANGELOG.md`. Ralph: https://www.aihero.dev/tips-for-ai-coding-with-ralph-wiggum (updated
January 8, 2026; read via the Wayback Machine).

**Renames (important for links).** Per `CHANGELOG.md`: "`to-prd` is renamed to `to-spec`";
"`to-plan` and `to-issues` are merged into one `to-tickets` skill, and `to-issues` is deleted".
The current repository does not mention Ralph (code search: 0 matches).

**(a) Artifacts.**
- Spec (`to-spec`): `Problem Statement`, `Solution`, `User Stories` ("A LONG, numbered list … As an
  <actor>, I want a <feature>, so that <benefit>"), `Implementation Decisions` (modules, interfaces,
  schemas, API contracts; "Do NOT include specific file paths or code snippets"), `Testing Decisions`
  (what counts as a good test, which modules, prior art), `Out of Scope`, `Further Notes`. Published
  to the tracker with the label `ready-for-agent`. Before that, testing "seams" are agreed ("the ideal number
  is one").
- Tickets (`to-tickets`): tracer-bullet vertical slices, "sized to fit in a single fresh context
  window", each with **blocking edges**. Locally: one file per ticket
  `.scratch/<feature-slug>/issues/<NN>-<slug>.md` with fields `What to build`, `Blocked by`,
  `Status: ready-for-agent`, a criteria checklist; in the tracker — an issue with `Parent`, `What to build`,
  `Acceptance criteria`, `Blocked by` and native blocking links. "Work the **frontier**: any ticket
  whose blockers are all done". For broad refactors — expand–contract.
- Agent Brief (`triage/AGENT-BRIEF.md`): a structured comment when moving to
  `ready-for-agent` — "the authoritative specification that an AFK agent will work from. The original
  body and discussion are context: the agent brief is the contract". Fields: `Category`, `Summary`,
  `Current behavior`, `Desired behavior`, `Key interfaces`, `Acceptance criteria`, `Out of scope`.
  The principle "Durability over precision": "Don't reference file paths: they go stale", do not give line numbers;
  describe interfaces, types and behavioral contracts.
- Wayfinder map: a single issue with the label `wayfinder:map` — "an **index**, not a store"; decisions live in
  child tickets (`wayfinder:research|prototype|grilling|task`), a ticket "sized to one 100K token agent
  session"; claim = assigning an assignee before work starts; blocking through the tracker's native links.

**(b) AFK versus HITL.** Labels: `ready-for-agent` ("Fully specified, ready for an AFK agent") versus
`ready-for-human`. In wayfinder: "Every ticket is either **HITL** … or **AFK**"; "A HITL ticket only
resolves through that live exchange; the agent never stands in for the human's side". Per the changelog,
this fixed cases where `/wayfinder` was "grilling itself instead of the human".

**(c) Code versus prose.** Almost everything is prose and native tracker mechanisms (labels, blocking links,
assignee as a lock). Deterministic code in the repository: a guardrail hook for dangerous git commands
(`skills/misc/git-guardrails-claude-code/scripts/block-dangerous-git.sh`) and a dependency-cruiser
config in an experimental skill; there is no spec/ticket validator. The `implement` documentation
honestly admits the gap: "`implement` has no completion step … does not tick the `- [ ]` boxes on the
originating issue. Close the ticket and reconcile the criteria yourself" — and this breaks the frontier,
"If nothing gets closed, nothing ever becomes visibly unblocked". Parallel `/implement` runs in one
checkout led to `commit --amend` on someone else's commit and a lost stash.

**Ralph (aihero.dev).** A `for ((i=1; i<=$1; i++))` loop around `docker sandbox run claude -p
"@some-plan-file.md @progress.txt …"`; instructions: pick the highest-priority task, run
feedback loops, append to `progress.txt`, make a commit, "ONLY WORK ON A SINGLE FEATURE", and on
completion output `<promise>COMPLETE</promise>`. The script (code) checks for this marker in the output and
exits. HITL Ralph (`ralph-once.sh`, "Run once, watch, intervene") versus AFK Ralph ("Run in a loop
with max iterations"; "always cap your iterations … 5-10 … or 30-50"). The PRD is JSON with a `passes` field
modeled on Anthropic's: "The PRD becomes both scope definition and progress tracker". The contents of
`progress.txt`: the task and a link to the PRD item, decisions and reasons, changed files, blockers; "Don't keep
progress.txt forever … It's session-specific". Feedback loops: "The best setup blocks commits unless
everything passes. Ralph can't declare victory if the tests are red". HITL is kept for risky
tasks: "Use HITL Ralph for early architectural decisions — the code from these tasks stays forever".
The phrase about "\"fixed\" tests that now test nothing", found in retellings, was not
found in the article text — unconfirmed.

---

## 7. Summary table: artifact × producer × consumer × validation × storage

| Artifact | Who writes | Who reads | Validation (code / prose) | Storage, commit binding |
|---|---|---|---|---|
| `feature_list.json` (Anthropic) | initializer agent | every coding session; `progress.py` | code: only counting `passes`; immutability is prose; e2e via Puppeteer | file in the repo, a commit per feature |
| `claude-progress.txt` / `progress.txt` (Anthropic, Ralph) | every session, append | next session | none | repo; in Ralph deleted after the sprint |
| `init.sh` | initializer | every session at start | execution (if it breaks, it is visible at once) | repo |
| Sprint contract / QA report (Anthropic 2026) | generator + evaluator | generator, evaluator | LLM evaluator with thresholds + Playwright | files; format not published |
| Lock files `current_tasks/*.txt` (C compiler) | the agent that took the task | other agents | git (push conflict) | bare git upstream |
| `intent.md` → `spec.md` → `plan.md` (Claude SDLC playbook) | idea author / PO+Claude / engineer in plan mode | next stage, PR review | plan mode does not allow edits until approval; hooks for gates; plan↔diff sync is a recommendation | commit = stage trigger; legacy records store the SHA of the markdown file |
| Hooks `Stop`/`TaskCompleted`/`PreToolUse` (Claude Code) | team | runtime | code: exit 2 blocks | `.claude/settings.json`, skills, subagents |
| `AGENTS.md` as table of contents + `docs/` (OpenAI) | Codex, doc-gardening agent | all agents | code: linters and CI for freshness, cross-links, structure; custom architecture linters with remediation messages | repo |
| ExecPlan / `PLANS.md` (OpenAI) | agent | agent (including a "stateless" restart) | prose only | `exec-plans/active` → `completed` |
| `QUALITY_SCORE.md`, `tech-debt-tracker.md` (OpenAI) | background Codex tasks | people, agents | updated by agents; schema not published | repo |
| `constitution.md` (spec-kit) | `/speckit-constitution` | `plan` (gate), `analyze` | semver and Sync Impact Report are prose | `.specify/memory/` |
| `spec.md` / `plan.md` / `tasks.md` (spec-kit) | commands specify/plan/tasks | `analyze`, `implement`, `converge` | code: file presence, branch `NNN-name`, workflow gates + `state.json`; substantive checking is LLM | `specs/NNN-name/`, feature branch |
| `checklists/*.md` (spec-kit) | `checklist`, the reviewer ticks `[x]` | `implement` | checkbox counting is LLM, then a question to the human | feature directory |
| `requirements.md` (EARS) / `bugfix.md` (Kiro) | Kiro + human | design, tasks, PBT | Analyze Requirements (LLM); PBT — executable tests | `.kiro/specs/<feature>/`, git |
| `tasks.md` (Kiro) | Kiro, Sync Files | task executor | dependency graph, status in the UI; a `Pre Task Execution` hook can block | repo |
| Story frontmatter + `sprint-status.yaml` (BMAD) | SM/PM agents, build | dev agent, retro | code: `sprint_plan.py validate`, atomic write, ban on downgrading status | repo, `last_updated` |
| `ARCHITECTURE-SPINE.md` `AD-n` (BMAD) | architect agent | build, reviewer gate | code: `lint_spine.py` (placeholders, ids, fields, versions); semantics — LLM | repo |
| `*.spec.md` with `[@test]` (Tessl) | spec-writer skill | generation, verifiers | code: `validate-specs.sh`, `check-spec-links.sh`; verifiers (LLM judge) in CI | repo, `targets` globs |
| Spec Mode plan / Mission plan (Factory) | Droid | Droid, validation workers | per-milestone validators (LLM + app launch); lint green as Done | `.factory/docs/YYYY-MM-DD-slug.md`; mission format unconfirmed |
| Handoff draft (Amp, 2025; dropped in 2026) | Amp | new thread | the human edits the draft | thread |
| Playbook (Devin) | human | Devin sessions | none (postconditions in prose) | Devin web app |
| Spec / tickets / Agent Brief / wayfinder map (Pocock) | `to-spec`, `to-tickets`, `triage`, `wayfinder` | `/implement`, AFK agent | tracker labels and blocking links; no validator; `implement` does not close the ticket | GitHub/Linear or `.scratch/<feature>/issues/NN-slug.md` |
| `prd.json` + `<promise>COMPLETE</promise>` (Ralph) | human/agent in plan mode; the agent marks `passes` | loop | code: grep for the marker in bash; feedback loops block the commit if so configured | repo |

## 8. Cross-cutting conclusions

1. **What code checks is reliable.** In all sources the strongest guarantees come from executable
   checks: e2e/PBT tests, custom linters with instructional messages, hooks with exit 2, scripts
   like `sprint_plan.py validate` and `lint_spine.py`. Invariants such as "do not edit the feature description", "sync the
   plan with the diff", "tick the checklist" remain prose almost everywhere — and this is exactly where the authors themselves
   record failures (Böckeler, Anthropic's QA agent, Pocock's `implement`).
2. **Separation of "judgment — LLM, bookkeeping — script".** BMAD states it directly ("The LLM decides which
   files are epics … this script owns everything after that decision"); OpenAI — "promote the rule into
   code"; Tessl — "Prefer a linter or test when one can enforce the invariant".
3. **Immutability is rarely enforced mechanically.** Anthropic picks JSON because the model spoils it less often
   than Markdown, but there is no protection in code; spec-kit forbids `checklist` from ticking `[x]` —
   also in prose. A deterministic ban on downgrading status exists in BMAD; blocking test edits during a
   fix is proposed as a hook in the Claude SDLC playbook.
4. **Commit binding is almost everywhere implicit** (the artifact lives in the same repo and commit). Explicit binding
   appears only as a recommendation: the Claude SDLC playbook — the commit SHA of the markdown file in a legacy record,
   updating `plan.md` in the same commit as a deviation from the plan.
5. **Surviving sessions** is solved one way: a file in the repo plus git log (Anthropic, Ralph,
   ExecPlan, OpenAI `exec-plans/`, spec-kit `state.json`). Product handoff mechanisms (Amp) turned out
   temporary and dependent on the model generation; Anthropic also dropped context resets and sprints when
   moving to Opus 4.5/4.6.
6. **Criticism of volume.** A lot of up-front markdown creates review load and a false sense of control
   (Böckeler); vendors' answers are to cut the spec down to decisions and boundaries (Pocock: no file paths,
   BMAD: a single `Verify:` line by default), keep a ticket within the size of one context window, and
   move verification into executable tests.
