# Harness of Harness (HoH) as a harness design: roles, artifacts, configuration, limits

Date: 2026-09-29. Method: the repository `Flesymeb/HarnessOfHarness` was cloned at `main` =
`ae7cc6f` (2026-09-28) and its source read directly; the `hoh-lite` and `gh-pages` branches were
fetched for comparison; repository metadata came from `gh api`; the paper (arXiv 2609.01481) was
read from the PDF with `pdftotext`; the project page text was taken from `index.html` on the
`gh-pages` branch. Anything not confirmed in one of these sources is marked "unconfirmed".

Measured results (benchmark gains, ablations) are already in
[artifact-pipelines-academic §4.7](2026-09-25-artifact-pipelines-academic.md) and are not repeated
here. This note looks at HoH as a harness design to learn from.

Source abbreviations used below:

- **[paper]** — Yan et al., "Harness of Harness: Multi-Day Autonomous Software Development with
  Continual Improvement", arXiv 2609.01481, v1 2026-09-01, preprint, not peer-reviewed
  (https://arxiv.org/abs/2609.01481; the arXiv comment field only links the repo and page).
- **[page]** — https://flesymeb.github.io/HarnessOfHarness/ (`gh-pages` branch, `index.html`).
- **[repo]** — https://github.com/Flesymeb/HarnessOfHarness at commit `ae7cc6f`. File paths below
  are relative to the repository root; `lite/` stands for `hoh-lite/src/gameloop/`.

---

## 1. What "harness of harnesses" means

- The paper defines a *harness* as "the surrounding system that provides tools, manages execution
  and mediates the LLM's interaction with the development environment". HoH "builds upon existing
  harnesses" and organises their runs into planning–coding–testing loops "without modifying their
  implementations" ([paper] §1, §6).
- So the outer harness does not replace the agent loop. It **invokes an unchanged coding-agent CLI
  three times per iteration**, once per role, with different prompts, permissions and output
  contracts: `D_t = Plan_H(S, E_{t−1})`, `A_t = Dev_H(A_{t−1}; S, D_t)`,
  `E_t = Test_H(A_t; S, D_t)` ([paper] App. A.1, eq. 3; Algorithm 1).
- The coordinator is **not an agent**. The HoH-lite docstring reads: "The runtime is not an agent.
  It owns sequencing and auditable receipts while role implementations remain replaceable harness
  invocations" (`lite/core/runtime.py`, lines 1–5). The paper calls it "a deterministic Runtime" that
  "freezes each role's inputs, enforces its permissions, binds evidence to the tested candidate, and
  records the resulting project state" ([paper] Fig. 3 caption).
- The paper describes the design principle as "constrains verifiable outputs rather than
  prescribing agent workflows" ([page] abstract). The Runtime "controls which inputs an invocation
  can access, which tools and write operations it may use, and which output schema it must satisfy",
  and leaves reasoning and tool order to the agent ([paper] §3.4).
- There are two code artefacts. The **HoH system** behind the demos "builds on this foundation with
  more development and debugging tools, reusable skills, and project integrations" and is **not
  published** ([repo] `README.md`, "Code"). **HoH-lite** is "a lightweight, extensible implementation
  of HoH's core workflow" (`hoh-lite/README.md`). The rest of this note describes HoH-lite unless it
  says "paper".

## 2. Roles, stages and control loop

### 2.1 Three fixed roles

| Role | Access (default binding) | Output |
|---|---|---|
| Project Planner | workspace read-only, no live Godot MCP | a prioritisation overlay inserted into a deterministic development document |
| Developer | workspace read-write, Godot MCP read-write | the updated artifact (`/workspace/game`) with replay traces |
| QA Tester | an isolated copy of the candidate, Godot MCP `read-only-runtime` | `visual_playtest_report.md` + `.json` |

Sources: `lite/core/roles.py` (`default_role_bindings`, lines 70–101); `lite/templates/project_planner_prompt.md`;
`lite/adapters/gamecraft_bench/templates/visual_tester_prompt.md`; [paper] Table 4.

The access rules are **validated in code, not just in prompts**. `RoleBinding.__post_init__` rejects
a read-write workspace or read-write MCP for anyone except the Developer, and rejects any live MCP
for the Planner (`lite/core/roles.py`, lines 39–54). The paper calls this the "single-writer
boundary" ([paper] §3.4.2).

### 2.2 The loop as a state machine

- `GameLoopRuntime._validate_transition` allows only Planner → Developer → Tester, then
  Tester → Developer or Tester → Tester. The first role in a loop must be the Planner. A loop cannot
  finish before the Developer stage (`lite/core/runtime.py`, lines 132–163).
- Each role call is appended as an event (`loop_index`, `sequence`, `role`, `status`, `started_at`,
  `duration_seconds`, `details`) to `runtime_receipt.json`. The receipt is rewritten atomically on
  every event (`lite/core/runtime.py`, lines 22–41 and 190–204; atomic write with fsync in
  `lite/core/documents.py`, `write_json`).
- Runs are append-only. An existing receipt makes the runtime refuse to start ("start a new run or
  continue from an archived trial", `lite/core/runtime.py`, lines 58–62). Continuation uses a new run
  ID with `--resume-from` (latest completed valid loop) or `--seed-trial-dir` +
  `--start-loop-index` (`hoh-lite/README.md`; `lite/adapters/gamecraft_bench/runner.py`, lines
  913–919 and 2451–2454).
- **Inner repair loop.** `tester_policy` is `next-loop` (one tester pass per loop) or
  `repair-then-next`. The second runs an *acceptance* tester. If it does not pass and the failure is
  repairable, it runs up to `max_inner_repairs` same-loop Developer repair passes, each followed by a
  new acceptance tester. A final *next-loop* tester then writes the evidence for the next Planner
  (`runner.py`, lines 1008–1020 and 2900–3010; `build_inner_repair_instruction`, lines 2309–2346).
  Infrastructure failures, timeouts and candidate mutation are classed as non-repairable
  (`tester_failure_is_repairable`, lines 429–439).
- The tester has two phases with different output contracts. The **acceptance** phase returns
  `phase`, `status`, `repair_required`, `acceptance_checklist`, `mismatches`, `evidence` and
  `recommendation`, and must not include next-loop goals. The **next_loop** phase returns `status`,
  `remaining_bugs`, `next_loop_goals`, `preserve`, `evidence` and `recommendation`, and must not ask
  for same-loop repair (`lite/adapters/gamecraft_bench/prompts.py`, lines 61–112). On the last
  iteration the acceptance tester audits the whole product, because "nothing can be deferred to a
  nonexistent next loop" (lines 62–71).
- The generic `LoopEngine` (Planner → Developer → Tester → `benchmark.evaluate`) is used only by CI
  tests (`lite/core/loop_engine.py`; `.github/ci/test_hoh_lite.py`, line 17). The real loop is the
  3,436-line `main()` of the GameCraft adapter (`runner.py`). The benchmark-neutral core is therefore
  more aspiration than extracted reality.

## 3. How it wraps agent runtimes

- **Interface.** `HarnessAdapter.prepare(HarnessRequest) -> PreparedHarness` "prepare[s] a role
  process without starting it". The request carries the role binding, workspace, prompt path, output
  path, environment, `json_stream`, `ephemeral`, `sandbox_mode` and `extra_config`. The result is
  `command` (argv), `environment`, `stdin_path`, `output_mode`, `preserved_paths` and `readonly_files`
  (`lite/harnesses/base.py`).
- One process runner handles every harness, with wall and idle timeouts. The docstring says a
  benchmark "cannot decide how Codex, OpenCode, Pi, or another Harness is launched"
  (`lite/core/harness_runner.py`, lines 43–57). Output is normalised per `output_mode` into
  `(text, usage)` (`lite/harnesses/output.py`; `lite/harnesses/runtime.py`).
- **Registered adapters:** `codex`, `deepseek-harness`, `opencode`, `pi`
  (`lite/harnesses/registry.py`, lines 33–44). Examples of how each one maps role policy onto the
  runtime's native mechanism:
  - Codex: `codex exec --ignore-user-config --ignore-rules [--ephemeral] [--json] --sandbox
    read-only|workspace-write -C <ws> --model … -o <out> -c … -`, with the prompt on stdin. The MCP
    server and its read-only mode are injected through `-c mcp_servers.godot_mcp.*`
    (`lite/harnesses/codex.py`).
  - OpenCode: writes a private `opencode.json` under a throwaway `HOME`, sets permission
    `{"*": "deny"}` for the Planner and `{"*": "allow", "task": "deny"}` for the other roles, and
    runs `opencode --pure run --format json`. The adapter is hard-wired to one model
    (`deepseek-v4-pro`) and one provider key (`lite/harnesses/opencode.py`).
  - DeepSeek Harness: pins SDK/runtime versions and SHA-256 hashes of preset files. The official
    minimal preset "uses danger-full-access" and requires an explicit env acknowledgement
    (`lite/harnesses/deepseek.py`, lines 14–36 and 160; `hoh-lite/.env.example`).
- **Claude Code is not supported.** The roadmap lists "More harness & model adapters — bring your
  own coding agent to HoH-lite (e.g., Claude Code)" as open ([repo] `README.md`, "Roadmap").
- Isolation from the user's own agent configuration is deliberate: Codex runs with
  `--ignore-user-config --ignore-rules`, OpenCode and Pi get a throwaway `HOME`. Roles therefore do
  not see the user's global instructions or skills (`codex.py`, lines 21–26; `opencode.py`, lines
  27–43; `lite/harnesses/pi.py`, lines 28–33).
- `gameloop-harness-doctor` checks harness prerequisites (`lite/harnesses/doctor.py`;
  `pyproject.toml` scripts). What exactly it checks was not read in detail (unconfirmed).

## 4. Artifacts and handoffs

### 4.1 Development document = deterministic scaffold + bounded LLM overlay

- The Runtime renders a deterministic scaffold from `lite/templates/development_document.md`: loop
  policy, version context, public task brief, mechanics, content, difficulty, visual and demo-trace
  requirements, and a quality checklist.
- The Planner returns **only** an overlay with a fixed shape: `## Project Planner Priorities` →
  `### Priority Order` (1–3 items) → `### Preservation Gate` → `### Acceptance Gate`, 150–350 words.
  The overlay is inserted under `## Development Focus For This Loop`
  (`lite/templates/project_planner_prompt.md`, lines 54–87).
- **The overlay is validated mechanically:** heading presence, 1–3 numbered priorities, a minimum of
  300 characters, a maximum of one fifth of the scaffold, required document headings, and a
  "template similarity" of at least 0.90. On failure the prompt is re-issued once with the validation
  errors appended. If it fails again, the Runtime **falls back to the deterministic scaffold** and
  records `document_source: deterministic_template` and `fallback_reason` in `planner_status.json`
  (`lite/adapters/gamecraft_bench/planner.py`, lines 27–116 and 250–380). This implements the paper's
  "outputs that violate the required schema trigger a retry" ([paper] §1).
- Output files are versioned: `development_doc_vNNN.md` plus `development_doc_delta.md`
  (`lite/core/documents.py`, `development_doc_filename`; `lite/templates/development_doc_delta.md`).

### 4.2 Evidence: paper schema versus HoH-lite reality

- **Paper.** The evidence bundle is a partition into `verified_records` and `gap_records`. Each
  record has `claim_id`, `claim`, `execution_records[{type, path, observation}]` and `status`; gap
  records also have `player_impact` and `recommended_update`. A `planner_handoff` has
  `preservation_constraints`, `update_targets` and `validation_requirements`. "Source-code presence
  alone is not treated as behavioral verification" ([paper] App. A.4, eqs. 4–6, Listing 1). The paper
  also says the QA report "need not reproduce the mathematical tuple notation verbatim"; "the
  benchmark adapter normalizes" it into E_t (App. A.2, end).
- **HoH-lite.** Neither `planner_handoff` nor `execution_records` appears anywhere in the source
  (`grep` over `hoh-lite/src`). The tester writes `visual_playtest_report.json` with `status ∈
  {pass, partial, fail, blocked}`, `evidence`, `player_impact`, `recommendation` and `owner`
  (`developer` or `pm`), plus the phase fields from §2.2 (`visual_tester_prompt.md`, lines 49–59).
  The Runtime normalises this into:
  - `issues.json`: open issues with `id`, `source`, `severity ∈ {blocker, high, medium, low}`,
    `title`, `impact`, `evidence`, `recommendation`, `claim_id`, built from tester gaps, tester
    next-loop goals and mechanical execution facts (build failed, not evaluated, no demos, runtime
    errors) (`lite/core/evidence.py`, `build_loop_issues`, lines 202–282);
  - `loop_memory.json` with `verified_demo_claims`, `open_issues` and `next_loop_targets` (lines
    430–467);
  - `demo_evidence_matrix.json`, a to-be-filled matrix of required review dimensions and demo
    promises (lines 470–507);
  - `evidence_history.json`, the lossless ledger across loops.
  Loop artifacts are written by `write_loop_artifacts` (`lite/core/prompts.py`, lines 499–522).
- **Parsing is tolerant, not schema-strict.** Status is read from JSON (`status`, `review_status`, or
  `repair_required: false` → pass). If none is found, a regex is run over Markdown
  (`runner.py`, lines 346–416). Verified claims accept a set of "pass-like" strings (`pass`, `ok`,
  `present`, `done`, …) and several key aliases (`lite/core/evidence.py`, lines 33–43 and 308–427).
  There are no JSON Schema files in the repository (`git ls-files | grep -i schema` outside vendored
  code returns nothing).
- **Progressive disclosure to the Planner.** The Planner gets a *bounded* evidence packet: at most 8
  open issues, 4 bugs, 4 gaps, 3 goals and 3 preserve items, each clipped to 240 characters, plus a
  `lossless_artifacts` list pointing to `issues.json`, `evidence_history.json` and
  `development_brief.md`. The code comment gives the reason: "Repeating entire QA reports and the full
  history here makes later planning calls attend to old prose instead of current product gaps"
  (`lite/core/prompts.py`, `build_project_planner_evidence_packet`, lines 177–236). The history
  section is capped at the last 3 loops and 6,000 characters (lines 93–151). This matches the paper's
  "progressive disclosure rather than a dedicated memory module" ([paper] §1).
- **Public-only evidence policy.** Hidden tests, scores, private rubrics and evaluator rationales are
  never fed back into the loop (`lite/core/policy.py`; [paper] App. A.5). This separation matters in
  a benchmark. In agent-workbench the analogue is keeping CI or reviewer verdicts distinct from agent
  self-reports.

### 4.3 The frozen, read-only QA candidate

- Implementation: the candidate game directory (and verifier demos) is **copied** into
  `tester_workspace/`, with symlinks and caches dropped and the public task text re-materialised as
  `public-task/description.md` (`runner.py`, `_prepare_visual_tester_workspace`, lines 525–585). The
  tester runs under a Linux namespace or `bwrap` sandbox that hides private paths. A direct,
  unisolated mode requires explicit env opt-in (`runner.py`, lines 645–790;
  `lite/adapters/gamecraft_bench/local_env.py`, lines 228–262; `hoh-lite/.env.example`).
- Enforcement: `candidate_source_sha256` hashes the authoritative candidate before and after the
  tester run, ignoring `reports/`, `demo_outputs/`, logs and engine caches (`lite/core/integrity.py`).
  A changed hash sets `tester_status = "invalid_candidate_mutation"`, a protocol error that cannot be
  repaired (`runner.py`, lines 1696–1720 and 442–459).
- As noted in the earlier note, the paper argues for freezing but has no ablation of frozen versus
  editable QA.

### 4.4 Process evidence ("did the tester actually run it?")

- A patched, vendored Godot MCP server writes a receipt per call (`call-*.json`). The Runtime
  summarises the receipts into categories (`lite/core/mcp_evidence.py`). When coverage is required,
  both Developer and Tester must show `project`, `lifecycle`, `runtime_state`, `runtime_log`,
  `interactive_input` and `screenshot` calls, and the Developer must close at least one debug cycle.
  Otherwise the tester status becomes `invalid_mcp_coverage` (`mcp_evidence.py`, lines 161–207;
  `runner.py`, lines 1690–1722). The default is `require_developer_mcp_coverage: false`
  (`hoh-lite/configs/gameloop.example.json`).
- The vendored MCP is pinned by upstream commit, file count, SHA-256 and overlay-patch hash
  (`lite/_vendor/godot-mcp.lock.json`).

## 5. Configuration and extension points

- **Run profile (JSON + CLI flags).** `configs/gameloop.example.json` exposes `attempts` (the
  iteration budget), `reasoning_effort`, `developer_backend`, per-role `roles.{planner,developer,tester}`
  with `harness`, `model`, `workspace_access` and `tools`, plus `planner_mode` (`harness` |
  `template`), `llm_tester`, `tester_policy`, `max_inner_repairs`, `external_verifier`,
  `warm_start_latest`, `require_developer_mcp_coverage`, per-role wall and idle timeouts, and
  `domain_policies` (`hoh-lite/configs/gameloop.example.json`). Ready-made profiles per harness–model
  pair live in `hoh-lite/configs/harnesses/*.json`. Unknown role names are rejected
  (`lite/core/roles.py`, `role_bindings_from_config`).
- **Mixed runtimes per role are possible in principle.** Each role has its own `harness` and `model`
  (`roles.py`, lines 104–153). The paper, however, holds one harness–model pair fixed across roles
  ([paper] §3.4, App. A.1). Mixed-role runs are not evaluated (unconfirmed whether they work end to
  end).
- **Role prompts are Markdown templates with `{{slots}}`** (`lite/templates/*.md`,
  `lite/adapters/gamecraft_bench/templates/*.md`), assembled as "[role instruction] ⊕ [public
  specification] ⊕ [preceding evidence] ⊕ [document scaffold] ⊕ [output contract]" ([paper]
  App. A.2). They are package data, not user-overridable files. No override mechanism was found
  (unconfirmed beyond a code read).
- **Domain policy packs.** Small Markdown checklists (`lite/policies/*.md`: generic, combat-ai,
  movement-camera, …). Each has a size limit of 24,000 characters. They are selected automatically by
  task-ID prefix or by at least two keyword hits in the public task, or listed explicitly. Every
  selection is recorded with the file's SHA-256 in `domain_policy_manifest.json`, under the rule
  "public task requirements remain authoritative" (`lite/core/domain_policy.py`). The pack list is a
  hard-coded tuple, so a new pack needs a code change.
- **New project or task type = a `BenchmarkAdapter`.** It implements `run_cli`, `load_task` →
  `TaskSpec`, `create_context` → `BenchmarkContext`, and `evaluate` → `EvaluationResult`
  (`lite/benchmarks/base.py`). Adapters are registered in `default_benchmark_registry`; the CLI
  dispatches on `--benchmark` (`lite/benchmarks/registry.py`; `lite/cli.py`). Only
  `gamecraft-bench` exists. The paper's FrontierSWE and ProgramBench adapters are not in the repo, and
  "More project types — templates and examples beyond Godot games" is on the roadmap ([repo]
  `README.md`). The Godot and replay specifics (evidence dimensions, asset rules, trace schema) are
  hard-coded in `lite/core/evidence.py` and the templates, even though these sit under `core/`.
- **Reproducibility manifest.** `reproducibility.json` records the Python version, OS, source and
  benchmark git revision plus dirty flag, config SHA-256, role bindings, harness `--version` output
  and Godot version. It records no environment secrets, and CI checks this
  (`lite/core/reproducibility.py`; `.github/ci/test_hoh_lite.py`, line 142).

## 6. Installation and runtime requirements

- Python ≥ 3.12, pip-installable package `gameloop` 0.1.0 with extras `gamecraft-bench`
  (`harbor==0.23.0`) and `deepseek-harness` (`hoh-lite/pyproject.toml`).
- Run requirements: Godot 4, the chosen harness CLI with credentials, an external GameCraft-Bench
  checkout, Node.js/npm, bubblewrap, Xvfb, xdotool and FFmpeg (`hoh-lite/README.md`, "Quick start").
  The sandboxing (namespaces, `bwrap`) and CPU-affinity settings are Linux-specific
  (`hoh-lite/.env.example`; `local_env.py`). No macOS path was found (unconfirmed beyond the code
  read).
- The run command is `gameloop --config configs/harnesses/<profile>.json --bench … --task … --run-id …`,
  after `gameloop-godot-mcp-vendor prepare` (`hoh-lite/README.md`).
- **Operation is batch and unattended**: one process runs T loops. There is no UI. A web UI and a TUI
  console are roadmap items ([repo] `README.md`, "Roadmap").

## 7. Mechanical checks versus LLM judgement; human gates

| Kind | What | Source |
|---|---|---|
| Mechanical | role-permission invariants; legal role transitions; append-only receipts | `roles.py`, `runtime.py` |
| Mechanical | planner overlay shape, size and similarity; retry then deterministic fallback | `planner.py` |
| Mechanical | replay-trace contract validated before a trial can seed the next loop; build status read from the external verifier's public breakdown | `artifacts.py`, `validate_demo_trace_schema` (line 106) and `summarize_breakdown`; `runner.py`, line 315 |
| Instruction only | headless launch check (`godot --headless … --quit-after 5`) is asked of the Developer, not run by the Runtime itself (as far as read) | `extra_instruction_base.md` |
| Mechanical | candidate hash unchanged during QA | `integrity.py`, `runner.py` |
| Mechanical | MCP call coverage from receipts (optional) | `mcp_evidence.py` |
| Mechanical | infrastructure-failure detection kept separate from product failure | `runner.py`, lines 290–323 and 429–459 |
| Mechanical | external verifier and benchmark judge, kept out of the loop's inputs | `runner.py`, `evaluate_candidate_with_verifier`; `policy.py` |
| LLM judgement | choosing ≤ 3 priorities, preservation and acceptance gates | Planner overlay |
| LLM judgement | pass / partial / fail / blocked, gaps, next-loop goals, what to preserve | Tester report |

- **Acceptance is an LLM verdict gated by mechanical preconditions.** `tester_acceptance_passed` is
  true only if the tester's `review_status == "pass"` (`runner.py`, lines 418–427). Mechanical checks
  can invalidate a verdict (mutation, missing MCP coverage, a malformed report), but they cannot
  produce a pass on their own.
- **Human gates: none by design.** The goal is development "without further human guidance or
  intervention" ([paper] §1). In the 70-loop Fusepoint run, "Human involvement was limited to
  restoring network or API availability and did not extend to planning, implementation, debugging,
  testing, or acceptance" ([paper] §5.1). That run used GitHub issues and commits as the issue ledger
  and history: 81 issues recorded, 65 closed and 17 reopened after regressions by loop 70
  ([paper] §5.2). The GitHub-issue integration is part of the unpublished full HoH, not HoH-lite
  (no GitHub API use found in `hoh-lite/src`).

## 8. Limitations and maturity

- **Repository activity** (`gh api repos/Flesymeb/HarnessOfHarness`, 2026-09-29): created
  2026-09-01, last push 2026-09-28, 146 stars, 15 forks, 2 open issues (both questions, no PRs), no
  releases, branches `main`, `hoh-lite` and `gh-pages`. There are 75 commits, all by one account
  (`Flesymeb`). By date: 58 on 09-01/02 (README and assets), 8 on 09-08, then 09-23, 09-24 and 09-28.
  **HoH-lite arrived as one bulk commit** `ec845b7` ("Add HoH-lite release source and
  documentation", 2026-09-24), followed by five commits titled "update" (`git log`).
- **License:** MIT for the paper and demo materials; Apache-2.0 for `hoh-lite/`, with NOTICE and
  third-party notices ([repo] `README.md`, "License"; `hoh-lite/LICENSE`). The GitHub API reports
  MIT for the repository and "TypeScript" as the language, the latter because of the vendored Godot
  MCP server.
- **Code size:** about 14.3k lines of Python in `hoh-lite/src/gameloop` excluding vendored code.
  `runner.py` alone is 3,436 lines (`wc -l`).
- **CI:** compile, `--help`, adapter doctor against a pinned benchmark commit, unit tests of runtime
  contracts, a wheel/sdist leak check, and `scripts/check-public-release`. The last one scans *every
  reachable commit* for machine paths, credential patterns and forbidden directories
  (`.github/workflows/hoh-lite-ci.yml`; `hoh-lite/scripts/check-public-release`). There is no
  end-to-end loop test with a real agent in CI.
- **Design limits relevant to us:**
  - one domain (Godot games, replay traces) is baked into "core";
  - no Claude Code adapter;
  - batch autonomy with no human gate and no tracker integration in the lite version;
  - the evidence contract is prose plus tolerant parsing, not a schema;
  - the paper's typed `planner_handoff` does not exist in the released code;
  - the published results used a fixed harness–model pair ([paper] B.1), and the README warns that
    "the default profile accepts the installed Codex CLI, so check this manifest before comparing a
    run with the paper's results" (`hoh-lite/README.md`);
  - token cost per iteration is high enough that "Efficiency — lower token cost and wall-clock time
    per iteration" is the first roadmap item ([repo] `README.md`; per-invocation token distributions
    in [paper] App. C.5).

---

## 9. Implications for agent-workbench

### 9.1 Ideas to adopt

1. **The runtime adapter as a pure "prepare" function.** Model each agent runtime as
   `prepare(role_binding, workspace, prompt, output_path, env) → {argv, env, stdin, output_mode}`
   plus one shared supervisor with wall and idle timeouts (§3). This is the smallest seam that works
   for Codex, OpenCode, Pi and (later) Claude Code. It fits our Rust core without our becoming an
   orchestrator. Note: the brief keeps launching agents out of scope, so the useful part may be the
   *role binding* data model rather than the launcher.
2. **Role permissions as validated data, not prose.** A role binding declares `workspace_access`
   and tool access, and the loader rejects illegal combinations: only the implementer writes; the
   planner and QA are read-only (§2.1). Our stage definitions can carry the same fields, and
   `workbench check` can refuse a profile that grants a reviewer write access.
3. **Single-writer boundary plus candidate hash for QA.** Bind a QA or review artifact to the exact
   candidate. HoH hashes the source tree before and after QA (§4.3). For us the natural identity is
   the git commit or tree SHA of the PR head. Review evidence should carry that SHA, and the contract
   check should mark evidence stale when the head moves. The earlier note lists this binding as
   "argued, not measured", so treat it as cheap hygiene, not a proven win.
4. **A deterministic scaffold with a small, shape-validated LLM overlay.** The runtime owns the
   document skeleton. The agent fills a bounded section with fixed headings (priorities ≤ 3, a
   Preservation Gate, an Acceptance Gate). The check validates the shape, retries once with the
   errors, then falls back and records why (§4.1). This maps directly onto our "artifact contract
   checked by a command": frontmatter plus required headings, checked by JSON Schema and a heading
   linter.
5. **Preservation Gate and Acceptance Gate as mandatory plan sections.** Every plan names what must
   not regress and the smallest end-to-end validation (§4.1). This is a concrete, checkable form of
   our "exit condition" per stage.
6. **Verified versus gap evidence, where "code exists" is not evidence.** Each claim has a status in
   {verified, gap} and cites execution records ({type, path, observation}). Unsupported claims default
   to gap ([paper] Listing 1). This is a good shape for our evidence artifact. We should implement it
   as an actual schema, which HoH-lite did not (§4.2).
7. **Two-level memory: a bounded packet in the prompt, a lossless ledger on disk.** Give the next
   stage a clipped summary plus explicit pointers to the full files (§4.2). This fits "a stage is
   derived from facts": the command center reads the ledger, and agents get the bounded view.
8. **Process receipts as evidence that checks were actually run.** HoH counts MCP call categories
   to reject a "review" that never launched the product (§4.4). A lightweight equivalent for us: the
   evidence artifact must reference command outputs (test log, CI run ID) that the check can resolve.
9. **Append-only run receipts and immutable run IDs; resume from the last completed stage** (§2.2).
   This matches our "no statuses of our own": stage state is reconstructed from receipts and artifacts.
10. **Separate infrastructure failure from product failure.** Transport and timeout errors are not
    fed back as QA findings and do not consume repair budget (§2.2, §7).
11. **A reproducibility manifest per run** with runtime versions, config hash and role bindings, and
    no secrets (§5). This is cheap to emit and useful when comparing agents across projects.
12. **Public-release hygiene in CI**: scan all reachable history for personal paths and credential
    patterns (§8). This directly serves our "public repo, no personal paths" rule.

### 9.2 Ideas to avoid

1. **Domain knowledge hard-coded in "core".** HoH-lite's `core/evidence.py` and templates are full of
   Godot and replay specifics, and the "generic" `LoopEngine` is unused by the real runner (§2.2, §5).
   Our profiles must hold all project-kind knowledge as data (stage lists, checks, templates), with
   the core staying truly generic, as the brief requires.
2. **Tolerant parsing instead of a contract.** Key aliases, "pass-like" strings and regex over
   Markdown (§4.2) make the evidence contract whatever the model happened to write. Our contract
   should be strict JSON Schema with explicit errors, which is what the retry loop then feeds back.
3. **Unattended batch autonomy with no human gate.** HoH's goal (zero human intervention, §7) is the
   opposite of our pipeline, where the owner starts each stage and decides merges. Adopt the loop's
   *artifacts*, not its autonomy.
4. **Prompt templates shipped as package data with no override path** (§5). Our role instructions and
   templates must be overridable per project (profile → project override).
5. **Isolating roles from the user's agent config by default** (`--ignore-user-config`, throwaway
   `HOME`, §3). This fits a benchmark but would strip project skills and `AGENTS.md` from real work.
   If we isolate at all, it should be opt-in per role.
6. **A single bulk-committed codebase with a 3.4k-line runner** (§8). It is a reference for ideas, not
   a dependency.

### 9.3 Open questions for the grilling session

1. Is the **Planner → Developer → QA** triple, with its two nested loops (inner repair with a cap,
   outer next iteration), the right default shape for our stages? Or do we keep idea → spec → tickets →
   implementation and add the Preservation and Acceptance Gates as required fields of the spec and
   plan artifacts? (Evidence: §2.2, §4.1.)
2. Should a stage definition include a **bounded automatic repair loop** (`max_inner_repairs`), or is
   every failed QA a stop that returns to the owner? (§2.2 versus our "owner starts each stage".)
3. What is our **candidate identity** for evidence binding: PR head SHA, tree SHA, or a content hash
   like HoH's? What should happen to evidence when the head moves after review? (§4.3; the earlier
   note says the binding's value is unmeasured.)
4. Does our evidence artifact adopt the **verified/gap partition** with `execution_records` as a
   required field? Which record types do we allow (CI run, test log, screenshot, trace)? (§4.2.)
5. Do we want **role-level permission declarations** in profiles (read-only reviewer, single
   writer), given that we do not launch agents ourselves? Who would enforce them: runtime-native
   sandbox flags, hooks, or only the post-hoc check? (§2.1, §3.)
6. Should profiles carry **domain policy packs** (small checklists selected by project kind or
   keywords, hash-recorded in a manifest), or is per-project `AGENTS.md` enough? (§5.)
7. How much of the previous iteration's evidence enters the next stage's prompt? Do we copy HoH's
   **bounded packet plus lossless ledger** split, and where does the ledger live: repo file, issue
   comments, or PR artifacts? (§4.2.)
8. Does "LLM verdict gated by mechanical preconditions" match our principle "what is mandatory is
   checked by code; judgement stays with the human"? Or must the final pass/fail for a stage always
   be a human or CI signal, never a QA agent's status field? (§7.)
