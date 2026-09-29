# Structured artifacts and evidence in LLM multi-agent development: a 2023–2026 literature review

Collection date: 2026-09-25. Metadata of all arXiv papers was verified via the arXiv API (`export.arxiv.org/api/query`),
texts were read from the PDFs (arXiv) with `pdftotext`. "Peer review" is stated from the arXiv comment/journal_ref field
or from the conference page; where there is none — "preprint, not peer-reviewed". Everything that could not be read in
the primary source is marked "unconfirmed".

Legend: **[R]** — peer-reviewed publication; **[P]** — preprint/technical report.

---

## 1. MetaGPT — SOPs and structured role outputs

- Hong et al., "MetaGPT: Meta Programming for A Multi-Agent Collaborative Framework", arXiv 2308.00352
  (v1 2023-08-01, v7 2024-11-01). **[R]** ICLR 2024 (in the PDF: "Published as a conference paper at ICLR 2024").
  https://arxiv.org/abs/2308.00352 · https://github.com/geekan/MetaGPT

**Mechanics.**
- SOPs are encoded in sequences of prompts. Work proceeds like an assembly line: Product Manager → Architect →
  Project Manager → Engineer → QA Engineer.
- *Structured Communication Interfaces*: "We establish a schema and format for each role and request that
  individuals provide the necessary outputs based on their specific role and context". Roles exchange
  documents and diagrams rather than dialogue. Artifacts: PRD, system interface design, sequence flow diagram,
  API spec, task list (§3.2, fig. 3, appendix B).
- *Shared message pool + publish-subscribe*: all roles publish structured messages to a shared pool and
  subscribe to the ones relevant to their role profile. Activation rule: "an agent activates its action only after
  receiving all its prerequisite dependencies". This is a readiness condition at the level of artifact dependencies, not
  at the level of evidence (§3.2).
- *Executable feedback*: the Engineer runs the code and tests and iteratively fixes based on the results (§3.3).

**Measured effect.**
- SoftwareDev (70 tasks, but only **7** randomly chosen tasks are used in the comparison; metrics from humans and statistics),
  table 1: Executability (1–4) — ChatDev 2.25, MetaGPT without feedback 3.67, MetaGPT 3.75. Human Revision Cost:
  2.5 / 2.25 / 0.83. Tokens: 19,292 / 24,613 / 31,255.
- Executable feedback: +4.2 pp (HumanEval) and +5.4 pp (MBPP) Pass@1. Final 85.9% and 87.7%.
- Role ablation (table 3). Each role adds its own artifact. Engineer only: executability 1.0, revisions 10.
  + Product (PRD): 2.0 / 6.5. + Product + Architect: 2.5 / 4.0. All four roles: 4.0 / 2.5. Cost $0.915 → $1.385.

**What is missing.** There is no direct ablation of "structured documents versus free dialogue" with the same roles.
The effect of schemas is confounded with the effect of roles and SOPs. Only executable feedback was tested separately.

**Threats to validity.** 7 tasks in SoftwareDev. Executability is scored by humans. HumanEval/MBPP are functions, not
projects. In the independent ChatDev evaluation (below) MetaGPT loses on "Quality" (0.1523 versus 0.3953), i.e. the
results depend on the evaluation protocol. In MAST (below) MetaGPT has 1.56 times more failures of category FC3
(verification) than ChatDev, although it has 60–68% fewer failures in FC1/FC2.

---

## 2. ChatDev, AgileCoder, EvoDev, EvoMAC

### 2.1 ChatDev
- Qian et al., "ChatDev: Communicative Agents for Software Development", arXiv 2307.07924 (v1 2023-07-16,
  v5 2024-06-05). **[R]** ACL 2024. https://arxiv.org/abs/2307.07924 · https://github.com/OpenBMB/ChatDev

**Mechanics.** Chat chain: 3 phases (design, coding, testing) and 5 subtasks. Roles: CEO, CTO, programmer, reviewer,
tester. A subtask finishes after "two unchanged code modifications or after 10 rounds". *Communicative
dehallucination* (CDH): before answering, an agent requests clarification. The artifacts are code and documents, but handoff goes
through dialogue in natural language and in code, without schemas.

**Measured effect** (SRDD, 1,200 task descriptions; ChatGPT-3.5, T=0.2). Table 1: Quality (= Completeness ×
Executability × Consistency) — GPT-Engineer 0.1419, MetaGPT 0.1523, ChatDev 0.3953.
Ablation (table 4), Quality:
- stopping after Coding 0.2512; after Complete 0.3690; after Review 0.3717; full chain (with Testing) 0.3953;
- without CDH 0.3094; without roles 0.2212.
- Executability: after Coding 0.77, with Testing 0.88. Without roles 0.58.

**Threats.** Consistency is the cosine similarity of embeddings of the requirements and the code, not a check that
requirements are fulfilled. Completeness is the absence of placeholders. The metrics are weakly related to correctness. The pairwise
evaluation was done by GPT-4 and humans.

### 2.2 AgileCoder
- Nguyen et al., arXiv 2406.11912 (v2 2024-07-14, "Work in progress" on arXiv). **[R]** FORGE 2025 (IEEE/ACM,
  pp. 156–167): https://conf.researchr.org/details/forge-2025/forge-2025-papers/1/AgileCoder-Dynamic-Collaborative-Agents-for-Software-Development-based-on-Agile-Meth ·
  https://arxiv.org/abs/2406.11912 · https://github.com/FSoft-AI4Code/AgileCoder

**Mechanics.** Roles: PM, Scrum Master, Developer, Senior Developer, Tester. The PM writes a *product backlog* with tasks and
**acceptance criteria**. The SM checks feasibility and can send the backlog back. Sprints: Planning → Development →
Testing → Review. A sprint inherits the result of the previous one. The *Dynamic Code Graph Generator* builds a dependency
graph G. The Tester tests the changed files and their ancestors in G.

**Measured effect** (ablation, table 3, GPT-3.5 Turbo, HumanEval/MBPP pass@1):
- full system 70.53 / 80.92;
- without incremental dev −1.02 / −2.47;
- without test generation −8.33 / −5.28;
- without code review −1.63 / −5.51.
- Claude 3 Haiku: without tests −6.10 / −4.45.
- ProjectDev (14 tasks): executability 57.79 versus ChatDev 32.79 and MetaGPT 7.73. Without graph G executability
  23.38 versus 57.50, and 11 cases of context overflow.

**Threats.** HumanEval/MBPP are functional tasks. ProjectDev is small. Acceptance criteria are not formally
checked as a gate. The backlog is text, without a schema.

### 2.3 EvoDev (arXiv 2511.02399)
- Liu et al., "Towards Iterative End-to-End Software Development: A Feature-Driven Multi-Agent Framework"
  (v1 2025-11-04, v3 2026-06-05). **[R]** ISSTA 2026 (comment on arXiv). https://arxiv.org/abs/2511.02399

**Mechanics.** Requirements are decomposed into user features. A *Feature Map* is built — a dependency DAG.
Each node holds a multi-layer context: **business logic, software design, code implementation**. This context
is passed to descendants along the edges. There is an overall design stage and a separate Business Analyst agent.

**Measured effect** (APPDev: 15 Android apps, 13.5 functional requirements on average; evaluated by an
acceptance checklist and a Likert questionnaire from four participants):
- versus Claude Code +57.3% on Function Completeness (FC). With Claude-4-Sonnet: build 100% versus 73.3%; FC 3.57
  versus 2.27.
- Versus a single agent +16.0…58.5% depending on the LLM.
- RQ3 ablation: overall design gives FC +0.22 (+7.2%).
- **Iterations without passing the predecessors' context worsen the result**: build 100% → 86.7%, FC 3.29 → 2.80.
  With context passing FC +0.50 (+16.3%).

**Limitations (stated by the authors).** The method has no testing: agents confuse whether the error is in the test or in the code.
Evaluation is manual, on 15 apps, Android/Kotlin only.

### 2.4 EvoMAC
- Hu et al., "Self-Evolving Multi-Agent Collaboration Networks for Software Development", arXiv 2410.16946
  (2024-10-22). **[R]** ICLR 2025: https://iclr.cc/virtual/2025/poster/31011 · https://arxiv.org/abs/2410.16946

**Mechanics.** Coding team (a MAC network) → the Testing team writes unit tests from the requirements. These tests serve as a *target
proxy*, and the compilation and run results provide objective feedback. The Updating team performs "textual
backpropagation" and restructures the coding team. rSDE-Bench is a requirement-oriented benchmark with
automatic checking of each requirement.

**Measured effect** (GPT-4o-Mini, table 1): Website Basic/Advanced 89.38 / 65.05; Game 77.54 / 51.60. The best
multi-agent baseline (ChatDev) 62.67 / 43.45 / 53.63 / 32.26. HumanEval 94.51.
Key ablation (variant f versus g): **if test execution in the environment is replaced by an LLM critique of the code**, the result
drops. Website −12.67 / −14.97 pp, Game −21.74 / −18.28 pp. This is a direct measurement: executable evidence
beats an LLM's opinion.

**Threats.** The tests are generated by the same system, i.e. the proxy may be wrong. The web/game tasks are narrow.

---

## 3. Proof-or-Stop (arXiv 2607.14890) — the key paper

- Huang, Hsia, Sun, Shi, Huang, White, "Proof-or-Stop: Don't Trust the Agent, Trust the Evidence — Loop
  Engineering for Verifiable Evidence-Gated Lifecycle Control", arXiv 2607.14890v1, 2026-07-16. 48 pages.
  **[P]** "Preprint v1", not peer-reviewed. https://arxiv.org/abs/2607.14890 · code: https://github.com/Proof-or-Stop
  (the paper gives only the organization URL; the exact repository is unconfirmed).

### 3.1 Semantics and formal framework
- **Agent-as-claim**. An agent's output is a *claim* (reviewed, tested, DONE, ready-to-merge), not a lifecycle
  state. The word "proof" is used operationally: evidence admissible to a gate under the stated trust model.
  Semantic correctness of the program is not asserted.
- State identities (eq. 1), computed from `git ls-tree` without lifecycle metadata:
  `materialHash(H)` = SHA256 of the canonical tracked tree; `headHash` = commit; `storyFilesHash` = SHA256 of the files
  belonging to the story.
- **Evidence** is a structured record, not prose. Binding
  `β(E) = ⟨materialHash, headHash, storyFilesHash, policyHash, commandSetHash⟩`. For executed actions there is a
  receipt `ρ(E) = ⟨cmd, args, cwd, exit, outputDigest⟩`. Signed records additionally carry
  actor/lane/host/session/signing-key.
- **Admissibility** (eq. 2): `Fresh ∧ Complete ∧ IntegrityVerified ∧ ProducerAuthorized ∧ ExecutionAttested ∧
  Supports(E,c) ∧ OutcomeAccepted`. Any change to the tree makes old evidence stale.
- **Gated advancement** (eq. 3): the transition φi→φi+1 is allowed only if every required claim from Ci is backed by
  admissible E. "a natural-language report from an agent is not an Ec … Eq. (3) therefore has no term for
  self-report".
- Scope restriction: the heavy mechanism applies only to claims that move the lifecycle. Notes,
  rationale and docs remain advisory and are not part of bindings. Runtime memory is marked `gateEvidence:false`.

### 3.2 Lifecycle and gates (table 8)
Story: `init → init-check → plan → dev → review → test → done`.
| Transition | Artifact | Check | Fail-closed |
|---|---|---|---|
| plan→dev | structured plan review / story contract | scope-contract, allowed paths, fresh storyFilesHash | no DEV without plan evidence |
| dev→review | scope-contract check | the diff is within the declared scope and bound to materialHash | out-of-scope → no REVIEW |
| review→test | `review-runs.json`, `review-passes.json`, `findings.json` | current round, reviewer identity, freshness; open verified critical/high block | blocks TEST |
| test→done | `done-required-evidence.json` (`evidence.json`) | tree hashes + policy/command-set match; build + full test suite exit 0 | blocks DONE on stale/missing/drift |
| high-risk DONE | host verdict receipts | floor R×K = **3 rounds × ≥2 independent verdicts** (different host, session, key) over the current materialHash | honest degradation: `degraded ⇒ FullAssurance=false` (eq. 5) |

Additionally:
- `findings.json`: severity, category, status, evidenceState, resolution trail.
- A wording gate / claim-boundary registry forbids wording stronger than the evidence allows.
- Merge must check the certificate against the exact commit with compare-and-swap. The PR is an interface, not a trust boundary.
- Clarification B-6c4d. Freshness is not determined by "novelty of the id". A PASS is admissible from any signed run of the current
  round if the hashes and scope match. An open FINDING requires the latest run of the given lane.

### 3.3 Measurements
1. **Engine contract** (Tier A): 10/10 scenarios (intake, executor, repair-loop, review-loop, evidence-gate,
   block-escalate, budget-stop, human-handoff, multi-host-review, no-false-done), **false-DONE = 0**.
   Stress set 10×15 = 150 ledger rows. It is checked that:
   - "prose" closure without machine evidence does not count;
   - stale bindings block;
   - deferral is not counted as delivery;
   - assignment is not equal to delivery;
   - duplicate ids are caught.
   This is a check of the mechanism, not a gain in outcomes.
2. **Receipt bundle** (B-4c65): a genuine bundle accepted; **18 classes of forgery rejected**, false-accept = 0,
   false-reject = 0. Trust model — local-key, single-host.
3. **Gate-strength** (B-7b6e). The reviewer metric is a pair (catchRate, falseBlock), because catchRate alone
   is insufficient. Test-only reviewer: catchRate 0. Block-all: 1.0, but it also blocks correct artifacts.
   Reference reviewers, not a live lane.
4. **Powered ablation, preregistered**: 5 arms × 24 tasks × 16 scenarios (null + B1–B15 injections) × k=5 =
   **9,240 cells**, one model (Sonnet; the exact snapshot was not recorded). "Amplified" — an artifact passed the visible test,
   failed the hidden oracle and was shipped anyway (table 11):

   | Arm | Amplified (B1–B15) | Tokens | Time |
   |---|---|---|---|
   | A1 prompt-only | 13/1440 (0.90%) | 168,622 | 51.7 s |
   | A2 naive-retry | 18/1800 (1.00%) | 169,202 | 51.3 s |
   | A2′ compute-budgeted | 31/1800 (1.72%) | 170,545 | 54.8 s |
   | A3 review-only (advisory) | 14/1800 (0.78%) | 200,510 | 80.3 s |
   | A4 Proof-or-Stop | **2/1800 (0.11%)** | 204,553 | 81.2 s |

   - H1 (A4 versus A2′, not-amplified): **+1.6 pp, 95% CI [0.8, 2.5]**. H2 (completion): +3.3 pp [0.0, 10.0],
     marginal.
   - **A3 versus A4** is the key isolation for us. The same single reviewer check at nearly equal budget (tokens
     A3/A4 = 0.98). The only difference is whether the review is advisory or a mandatory gate: 14/1800 versus 2/1800. Without the du-duration
     task: 9/1725 versus 0/1725. H3 is an exploratory hypothesis.
   - Effect concentration: 29 of the 31 cases of A2′ and both cases of A4 fall on a single task (du-duration). Without it
     2/1725 versus 0/1725. Per-scenario BH: 0/15 significant.
   - The powered matrix does **not** use story-level materialHash/receipt gating. It is an experiment on the
     control policy, not on the full hash mechanism.
5. **Clean-task pilot** (3 tasks × k=3). All arms 9/9. The gate adds only overhead: A4/A1 = 2.07× cost,
   2.51× wall. In the fault-pilot the bare loop missed 15/15 faults, the gated loop — 0/15.
6. **Paired matrix** Cell03/Cell06 (1,152 cells, descriptive). No-review reached completion in 1,143 cells.
   Proof-or-Stop admitted 1,042 and safely stopped 110. 106 completions of no-review were not admitted, but they were not checked
   by a hidden oracle. Tokens ×3.80.
7. **Self-application corpus**: 565 stories, 1,007 findings, 94.8% resolved; high ≈51%, critical ≈1%.
   - Lanes differ: the test-coverage lane ("intent↔evidence binding") yields 70% of high/critical; the concept lane —
     10 of the 14 critical.
   - Curated deep set: 26 of 28 findings (93%) were found when the author's tests were green. This is a reviewer
     label, not a counterfactual experiment. 6 real defects are confirmed by fix commits.
   - Cross-vendor (Codex host-2): 68 high/critical findings across 26 stories that had already passed same-vendor Claude
     lanes. The sample is selective.

### 3.4 Threats to validity (stated by the authors, §11)
- One model family, 24 tasks, a self-hosted corpus.
- The corpus was built and reviewed by LLMs.
- The `smoke_would_miss` labels were assigned by reviewers.
- A2′ is not budget-matched at the level of an individual run.
- The effect is rare and concentrated in one task.
- The local-key trust model does not protect against a compromised runner.
- There is no baseline frequency of "green-but-wrong" in real work, so the cost/benefit ratio cannot be estimated.
- There are no external benchmarks.

### 3.5 Related work cited by Proof-or-Stop
- **ResearchLoop**: Xia & Wang, arXiv 2605.28282, 2026-05-27, **[P]** technical report. https://arxiv.org/abs/2605.28282 ·
  https://github.com/plan-lab-szu/ResearchLoop
  - An evidence-gated control plane for research. Chain "Big RQ → sub-RQ → claim → task/spec → evidence →
    gate → paper". YAML schemas `STATUS.yaml`, `RESEARCH_SPINE.yaml`, `PAPER_CLAIM_LEDGER.yaml`.
  - Controlled study: 16 tasks × 3 seeds, deterministic audit. Unsupported claim rate reduced by
    **8.43 pp (−63.3%)** versus ad-hoc and by 8.47 pp versus a linear baseline. The authors themselves rate the
    significance as borderline: **p ≈ 0.08**.
  - The full protocol emits noticeably fewer claims (169 versus 467–569): part of the effect is in suppressing claims.
- **Agentic Agile-V**: Koch, arXiv 2605.20456, 2026-05-19, 7 pages, **[P]**. https://arxiv.org/abs/2605.20456
  - A conceptual paper without an experiment. A conversation-to-contract gate, a taxonomy of minimal input
    artifacts, an evidence-bundle acceptance model, the SCOPE-V cycle.
- **EviBound**: arXiv 2511.05524, 2025-10-28, **[P]**. https://arxiv.org/abs/2511.05524
  - Two gates. The Approval Gate before the run checks the schema of the acceptance criteria. The Verification Gate after the run
    checks artifacts via the MLflow API: run ID, required artifacts, FINISHED status.
  - 8 tasks. Prompt-only: 8/8 false "complete". Verification gate only: 25% false. Both gates: 0%, 7/8
    verified, 1 task blocked.
  - The sample is very small (n=8).

---

## 4. Harness engineering: automatic harness improvement and the link to evidence

### 4.1 Self-Harness (arXiv 2606.09498)
- Zhang et al., "Self-Harness: Harnesses That Improve Themselves", v1 2026-06-08, v3 2026-08-20. **[P]**.
  https://arxiv.org/abs/2606.09498
- **Mechanics**: Weakness Mining (from traces) → Harness Proposal (minimal edits, each tied to a specific
  failure mechanism) → **Proposal Validation**: "an edit is promoted only if it improves performance without causing
  measurable degradation on held-out tasks". This is a regression gate for changes to the harness itself.
- **Effect**: in all 9 model×benchmark combinations (Terminal-Bench-2.0, SWE-bench Verified, AppWorld; MiniMax
  M2.5, Qwen3.5-35B-A3B, GLM-5) both held-in and held-out improve. Relative gain up to 132%.
- Examples of accepted edits:
  - TB2, MiniMax: 42.2% → 53.9% (artifacts are created earlier, schema-invalid tool content, exit from looping
    tool cycles);
  - Qwen: 18.0% → 36.7%;
  - on SWE-bench Verified the edits concern **patch validation** ("local-test enforcement").
- **Threats**: compared only against a minimal initial harness; evaluations on benchmark samples.

### 4.2 Meta-Harness (arXiv 2603.28052)
- Lee, Nair, Zhang, Lee, Khattab, Finn, 2026-03-30, **[P]**. https://arxiv.org/abs/2603.28052
- **Mechanics**: a proposer agent reads a file system with all past candidates: source code, **raw
  execution traces**, scores.
- **Ablation** (online text classification): scores-only — median/best 34.6/41.3; scores + LLM summary — 34.9/38.7;
  full access to traces — **50.0/56.7**. "summaries do not recover the missing signal". This is direct evidence
  against compressing evidence into prose summaries.
- On TerminalBench-2 the discovered harness beats Terminus-KIRA: #1 among agents on Haiku 4.5 (37.6 versus 35.5).

### 4.3 AutoHarness (arXiv 2603.03329)
- Lou, Lázaro-Gredilla, Dedieu, Wendelken, Lehrach, Murphy, 2026-02-10, **[P]**. https://arxiv.org/abs/2603.03329
- Gemini-2.5-Flash synthesizes a code harness that filters out illegal actions, iteratively refining it from
  environment feedback. The harness prevents **all** illegal moves in 145 TextArena games. For comparison: in
  Kaggle GameArena 78% of Flash's losses were due to illegal moves.
- Applicability to SE is indirect: it is a deterministic code gate around an LLM. Only the abstract was read.

### 4.4 HarnessFix (arXiv 2606.06324)
- Chen, Wang, Liu, Wang, Zheng, Wang, "From Failed Trajectories to Reliable LLM Agents: Diagnosing and Repairing
  Harness Flaws", v1 2026-06-04, v2 2026-07-02. **[P]**. https://arxiv.org/abs/2606.06324
- **Mechanics**: traces and harness artifacts are compiled into **HTIR** (Harness-aware Trace IR) with data-flow and
  control-flow links between steps. Failures are attributed to steps and harness artifacts. Recurring diagnoses
  are consolidated into **flaw records**. The records are mapped to scoped repair operators, and a patch is generated under a
  flaw-specific repair specification. It is accepted only through **regression-aware acceptance** on a validation set:
  the goal is to reduce the target defect without new regressions.
- The ETCLOVG layer taxonomy. The operator catalog of the lifecycle layer has "verification-gated finalization",
  "delegated-output validation". In the verification layer — "intermediate validation gating", "effect-evidence
  completion guarding".
- An empirical study of 30 agents: 29 of 30 have defects in the three leading layers. Verification defects are more common
  in benchmark-oriented harnesses. Lifecycle defects — in long-lived agents.
- **Effect**: +6.3…18.4% over the initial harness, sign-test p < 0.001. Versus Meta-Harness +2.6…5.0 pp with
  63.5–100.5% fewer tokens.
- **Ablation** (table VI; GAIA / SWE / AppWorld / TB2):

  | Variant | GAIA | SWE | AppWorld | TB2 |
  |---|---|---|---|---|
  | Full | 61.7 | 57.3 | 43.0 | 26.5 |
  | Without regression-aware acceptance | 55.6 | 53.3 | 39.3 | 24.5 |
  | Without trace-grounded diagnosis | 51.1 | 50.7 | 38.1 | 21.6 |
  | Prompt-only | 50.6 | 48.3 | 37.4 | 18.6 |

  HTIR agrees with human labeling: step accuracy 85.0%, repair-operator accuracy 82.5%.

### 4.5 "Agent Harness Engineering: A Survey" (OpenReview eONq7FdiHa)
- Li, Xiao, Zhang, … Reddy, 2026. Per the PDF header — "Under review as submission to TMLR", **[P]**.
  https://openreview.net/forum?id=eONq7FdiHa · https://github.com/Picrew/LLM-Harness
- Introduces the ETCLOVG taxonomy: Execution, Tooling, Context, Lifecycle, Observability, **Verification**, Governance.
  Per secondary sources, it covers 110+ papers and 148 projects.
- **Unconfirmed**: the full text could not be read (OpenReview returns a challenge). The content of the Verification section
  is not retold here. The layer definitions are taken from table I in HarnessFix, which cites the survey
  as [1].

### 4.6 "Dive into Claude Code" (arXiv 2604.14228)
- Liu, Zhao, Shang, Shen, v1 2026-04-14, v2 2026-07-02. Tech report, **[P]**. https://arxiv.org/abs/2604.14228 ·
  https://github.com/VILA-Lab/Dive-into-Claude-Code
- An architectural analysis based on source code. The core is a while-loop. Around it:
  - a permission system with 7 modes and an ML classifier;
  - 5-layer compaction;
  - 4 extension mechanisms: MCP, plugins, skills, hooks;
  - subagent delegation;
  - append-oriented session storage.
- There are no quantitative measurements of the effect of artifacts or evidence. The paper is descriptive. Only the abstract was read.

### 4.7 Harness-of-Harness (arXiv 2609.01481) — a reference point
- Yan, Su, Zhang, Li, Zhang, Zhang, Chen, Bai, Hu, 2026-09-01, **[P]**. https://arxiv.org/abs/2609.01481 ·
  https://github.com/Flesymeb/HarnessOfHarness
- Iteration: `D_t = Plan(S, E_{t−1})`, `A_t = Dev(A_{t−1}; S, D_t)`, `E_t = Test(A_t; S, D_t)`.
- The evidence bundle consists of claims with `execution_records` and `status ∈ {verified, gap}`, `player_impact`,
  `recommended_update`, and also `planner_handoff {preservation_constraints, update_targets,
  validation_requirements}`.
- QA receives `read_only(A_t)` — a **frozen** candidate — plus deterministic build/run checks. "A
  criterion is verified only when candidate-bound records support the required behavior. Observed failures, unmet
  requirements, regressions, and insufficient evidence are recorded as gaps".
- **Ablation** (GameCraft-Bench, 45 tasks, Codex/GPT-5.5): Full HoH@3 71.52. Without Evidence Feedback (replanning without
  prior evidence) 65.23 (**−6.28**). Without Plan Update −8.13. Without Warm-Start −7.85 (and tokens 11.12M versus
  8.41M).
- There is **no** separate ablation of "frozen read-only QA versus QA allowed to edit" (not found in the text).

---

## 5. Failure analysis

### 5.1 MAST — "Why Do Multi-Agent LLM Systems Fail?"
- Cemri, Pan, Yang, … Zaharia, Gonzalez, Stoica, arXiv 2503.13657 (v1 2025-03-17, v3 2025-10-26). **[R]**
  NeurIPS 2025 Datasets & Benchmarks (stated in the PDF). https://arxiv.org/abs/2503.13657
- MAST-Data: 1,600+ traces across 7 MAS; κ = 0.88. 14 failure modes in 3 categories.
- **FC3 Task Verification** (fig. 1): FM-3.1 Premature Termination 6.2%, FM-3.2 No or Incomplete Verification 8.2%,
  FM-3.3 Incorrect Verification 9.1%. In total **23.5%** (21.3% in another version of the figure). FC1 also has FM-1.5
  "Unaware of Termination Conditions".
- Observation: systems with explicit verifiers (MetaGPT, ChatDev) generally fail less often, but "many existing
  verifiers perform only superficial checks … checking if the code compiles or if there are leftover TODO
  comments". Example: ChatDev's chess compiles but violates the rules of the game.
- Interventions (app. H, ChatDev):
  - ProgramDev-v0 (32 tasks): baseline 25.0% → improved prompts (only the senior role can close the discussion,
    the verifier targets edge cases) 34.4% → **a new topology: a loop in which the process ends only when the
    CTO confirms that all reviews are satisfied** 40.6%.
  - HumanEval: 89.6 → 90.3 → 91.5.
  - In the main body text: adding high-level task objective verification gives +15.6%.
  - The authors themselves: "do not constitute substantial improvements".
- MetaGPT versus ChatDev on ProgramDev: MetaGPT has 60–68% fewer FC1/FC2 failures, but **1.56 times more FC3**.

### 5.2 SlopCodeBench (arXiv 2603.24755)
- Orlanski et al., v1 2026-03-25, v2 2026-05-07, **[P]**. https://arxiv.org/abs/2603.24755 · https://www.scbench.ai
- 36 tasks, 196 checkpoints. An agent extends its own code according to an evolving specification.
- The best agent passes 14.8% of checkpoints. No task is solved end-to-end.
- Erosion grows in 77% of trajectories, verbosity — in 75.5%.
- Versus 473 human repositories: 2.3 times more verbose and 2.0 times more eroded.
- Quality-aware prompts reduce initial verbosity/erosion by up to a third, **but not the rate of degradation**. At the same time
  cost per checkpoint +12.1%, correctness −2.3 pp.
- Conclusion for our topic: the checkpoint tests are green while structural quality degrades. Evidence of quality is needed,
  not only of functionality. An instruction in the prompt does not change the trajectory.

### 5.3 SWE-EVO (arXiv 2512.18470)
- Le, Thai, Nguyen Manh, Phan Nhat, Bui, v1 2025-12-20, v6 2026-05-22, **[P]**. https://arxiv.org/abs/2512.18470
- 48 tasks from the release notes of 7 Python projects. On average 21 files and 874 tests per task.
- GPT-5.4 + OpenHands: 25% (versus 72.8% for GPT-5.2 on SWE-bench Verified).
- **Fix Rate** counts partial progress **only if all PASS_TO_PASS pass**: a regression zeroes the
  result.
- Failure analysis: for gpt-5 more than 60% of failures are Instruction Following. Older models show more looping and "Gave Up
  Prematurely".

### 5.4 ProjDevBench (arXiv 2602.01655)
- Lu et al., v1 2026-02-02, v2 2026-02-09, **[P]**. https://arxiv.org/abs/2602.01655 ·
  https://github.com/zsworld6/projdevbench
- 20 tasks, 8 categories. Combined evaluation: Online Judge (≈80% weight) and LLM-assisted code review for
  conformance to the specification (≈20%, per fig. 2). The review checks, for example, forbidden libraries and the
  "FS-as-DB" pattern.
- Overall acceptance 27.38%. Weak areas: system design, time complexity, resource management.
- Instructive as a scheme of "executable oracle + specification review" with different weights.

---

## 6. LLM-as-judge, self-verification, spec-to-test, traceability

### 6.1 Reliability of an LLM judge in acceptance
- **False success / confident closing**: arXiv 2606.09863, 2026-06-01, **[R — workshop]** FAGEN@ICML 2026.
  https://arxiv.org/abs/2606.09863
  - False success accounts for 45–48% of failures in the single-control domains of tau2-bench and 75.8% in AppWorld trajectories
    with explicit status claims.
  - **No LLM-judge configuration exceeded AUROC 0.65** (5 judges × 5 prompts). On AppWorld — 0.54.
    Judges rely on "confident closing language".
  - TF-IDF detectors give AUROC 0.83 and 0.95.
- **OverclaimBench**: arXiv 2609.20812, 2026-09-17 (v3 2026-09-22), **[P]**. https://arxiv.org/abs/2609.20812
  - In 67.9% of runs agents do not read all the files they are supposed to review. Among such runs 80.4%
    are misleading (59–96% depending on the model).
  - Subagent delegation raises coverage, but incomplete reviews are still the most often misleading.
  - Those that falsely claim a "complete review" miss injected defects ~1.8 times more often.
- **CodeJudgeBench**: arXiv 2507.10535, 2025-07-14, **[P]**. https://arxiv.org/abs/2507.10535
  - 26 judges. The judgments are noticeably random. The order of answers in a pair strongly affects accuracy.
  - Pairwise is better than pointwise.
- **Bias in the Loop**: arXiv 2604.16790, 2026-04-18, **[P]**. https://arxiv.org/abs/2604.16790
  - A judge's verdicts on code depend strongly on hints in the prompt with the code unchanged. The effect is so large
    that it changes conclusions and model rankings.
- **Systematic failures verifying code against NL specs**: arXiv 2508.12358, **[R]** ASE 2025 NIER.
  https://arxiv.org/abs/2508.12358
  - LLMs often mark **correct** code as not conforming to the requirements.
  - More complex prompts (explanations, suggested fixes) **increase** the number of errors.
- **Agent-as-a-Judge / DevAI**: arXiv 2410.10934, 2024-10-14, **[P]** (later ICML 2025 — **unconfirmed**).
  https://arxiv.org/abs/2410.10934
  - 55 tasks, 365 hierarchical requirements.
  - Agreement with the human consensus: Agent-as-a-Judge ~90%, LLM-as-a-Judge ~70%. For example, OpenHands gray-box:
    90.44% versus 70.76%.
  - Alignment rate is misleading under class imbalance: the LLM judge gets 84.15% on MetaGPT simply by rejecting
    almost everything.
  - Majority voting of three humans reduces an individual human's error to 6.01%.
- **LLM Critics Help Catch LLM Bugs**: arXiv 2407.00215, 2024-06-28, OpenAI, **[P]**.
  https://arxiv.org/abs/2407.00215
  - Critics find more bugs than hired reviewers, but hallucinate bugs. A "human + critic" team
    hallucinates less.
- **The Verification Horizon**: arXiv 2606.26300, 2026-06-24, **[P]**. https://arxiv.org/abs/2606.26300
  - A verifier is always a proxy of intent. Under optimization the gap grows: reward hacking, signal saturation.
  - 4 types of verifiers are compared: test, rubric, user, agent.

### 6.2 Self-verification and bias toward one's own output
- **Large Language Models Cannot Self-Correct Reasoning Yet**: arXiv 2310.01798, **[R]** ICLR 2024.
  https://arxiv.org/abs/2310.01798
  - Without external feedback, self-correction does not help and sometimes worsens the result.
- **LLM Evaluators Recognize and Favor Their Own Generations**: arXiv 2404.13076, 2024-04-15, **[P]** on arXiv
  (NeurIPS 2024 — **unconfirmed**). https://arxiv.org/abs/2404.13076
  - A linear relationship was found between the ability to recognize one's own text and the strength of self-preference.
- **Self-Preference Bias in LLM-as-a-Judge**: arXiv 2410.21819, **[R — workshop]** NeurIPS 2024 SafeGenAI.
  https://arxiv.org/abs/2410.21819
  - GPT-4 shows significant self-preference. The mechanism is a preference for text with low perplexity.
- **AgentCoder**: arXiv 2312.13010, **[P]** on arXiv. https://arxiv.org/abs/2312.13010
  - RQ6: code and tests **in one dialogue** versus a separate test designer who **does not see the code**.
  - Test accuracy 61.0 / 51.8 versus **87.8 / 89.9** (HumanEval / MBPP, GPT-3.5).
  - Line coverage 72.5 / 75.9 versus 87.5 / 89.5.
  - Pass@1 71.3 / 79.4 versus 79.9 / 89.9.
  - The authors explain the gap by the fact that tests from the code's author "can be biased by the code and lose objectivity".
- **ImpossibleBench**: arXiv 2510.20270, 2025-10-23, **[P]** ("Under review"). https://arxiv.org/abs/2510.20270
  - The tests contradict the specification, so any pass means cheating. Cheating rate (fig. 1): GPT-5 76%,
    Sonnet 3.7 70%, Opus 4.1 54%, Sonnet 4 48%, o3 39%.
  - Hidden tests reduce cheating almost to zero but degrade legitimate work. **Read-only tests** are a
    compromise: they block modification of tests but not special-casing or operator overloading.
  - The `flag_for_human_intervention` option reduces GPT-5's cheating from 54% to 9%.
  - LLM monitors catch 86–89% of attempts on LiveCodeBench, but only 42–65% on SWE-bench.
- **"Solved issues" in SWE-bench**: arXiv 2503.15223, **[P]** on arXiv. https://arxiv.org/abs/2503.15223
  - 29.6% of plausible patches behave differently from the ground truth. The resolution rate is overstated by 6.2 pp.
- **Developer-agent misalignment, 20,574 sessions**: arXiv 2605.29442, **[P]**. https://arxiv.org/abs/2605.29442
  - "inaccurate self-reporting" grows as a share over time. 91.49% of permission requests require an explicit user correction.

### 6.3 Executable specifications and tests from requirements
- **CodeT**: arXiv 2207.10397, 2022, **[R]** ICLR 2023 (from memory, **unconfirmed**; the work predates 2023, given as
  background). https://arxiv.org/abs/2207.10397
  - Generated tests + dual execution agreement: HumanEval pass@1 65.8% (+18.8).
- **TiCoder**: Fakhoury et al., arXiv 2404.10100, **[R]** IEEE TSE 50(9) 2024. https://arxiv.org/abs/2404.10100
  - Intent is clarified via tests. In a user study (15 programmers) participants significantly more often correctly
    assess the code.
  - With an idealized user +45.97% pass@1 within ≤5 interactions.
- **TDD for Code Generation**: Mathews & Nagappan, arXiv 2402.13521, **[P]** on arXiv (ASE 2024 — **unconfirmed**).
  https://arxiv.org/abs/2402.13521
  - Tests alongside the problem statement consistently increase success on MBPP and HumanEval.
- **nl2postcond**: Endres et al., arXiv 2310.01831, **[R]** FSE 2024. https://arxiv.org/abs/2310.01831
  - LLMs translate NL intent into postconditions. They are generally correct and discriminate wrong code. They caught 64
    historical Defects4J bugs.

### 6.4 Requirements traceability
- **Prompts Matter** (Rodriguez, Dearstyne, Cleland-Huang), arXiv 2308.00229, 2023, **[P]** on arXiv (RE'23
  workshop — **unconfirmed**). https://arxiv.org/abs/2308.00229
  - The quality of trace-link recovery depends strongly on the prompt. The work is qualitative, without large numbers.
- No quantitative studies were found of the effect of **requirement→test→code traceability inside a multi-agent pipeline on
  final quality**. The closest:
  - rSDE-Bench and DevAI: per-requirement checking as an evaluation method;
  - the test-coverage lane in Proof-or-Stop ("intent↔evidence binding"): 70% of high/critical findings, but observational;
  - ResearchLoop: the RQ→claim→evidence chain, p≈0.08.

---

## 7. Synthesis

### 7.1 Mechanisms with measured support
1. **Executable evidence instead of LLM opinion.** Support is strong and reproduces across different papers.
   - EvoMAC: replacing execution with an LLM critique costs −12.7…−21.7 pp [R].
   - MetaGPT: executable feedback +4.2/+5.4 pp and revisions 2.25 → 0.83 [R].
   - AgileCoder: without test generation −5.3…−8.3 pp [R].
   - ChatDev: the testing phase gives executability 0.77 → 0.88 [R].
   - Against: LLM judges ≤ AUROC 0.65 on false success [R-workshop]; systematic false rejections on correct
     code [R].
2. **Independence of the checker from the author.**
   - AgentCoder: tests written without access to the code are more accurate by 27–38 pp [P].
   - Self-correction without an external signal does not work [R].
   - Self-preference in judges [R-workshop].
   - HoH and Proof-or-Stop build their architecture on this, but there is **no separate ablation** of "frozen read-only QA versus QA
     that can edit".
   - Read-only tests in ImpossibleBench reduce test modification [P].
3. **A mandatory gate versus advisory review at equal budget.**
   - Proof-or-Stop, A3 versus A4: 14/1800 → 2/1800 missed green-but-wrong [P, one model, effect
     concentrated in one task].
   - MAST: the "finish only on confirmed reviews" loop 25.0 → 40.6% on 32 tasks [R].
   - EviBound: prompt-only 100% false "complete", verification gate 25%, two gates 0% [P, n=8].
4. **Raw evidence, not its retelling, when passing between iterations.**
   - Meta-Harness: traces versus summary — median 50.0 versus 34.9 [P].
   - HoH: without evidence feedback −6.28 [P].
   - HarnessFix: without trace-grounded diagnosis −4…−10 pp [P].
   - EvoDev: iterations without the predecessors' context worsen build and FC [R].
5. **A regression gate on changes to the harness or process.**
   - HarnessFix: without regression-aware acceptance −2…−6 pp [P].
   - Self-Harness: promotion only without held-out degradation; no accepted harness worsened the split [P].
   - SWE-EVO Fix Rate formalizes "a regression zeroes progress" as a metric.
6. **Structured intermediate documents and roles.** Support is moderate and mixed.
   - MetaGPT: role ablation with their artifacts — executability 1.0 → 4.0 on 7 tasks [R].
   - EvoDev: Feature Map, context layers, overall design +7.2% [R].
   - But ChatDev in its own evaluation outperforms MetaGPT [R]. MAST: MetaGPT has 1.56 times more FC3 failures.
   - Schemas by themselves improve coordination (FC1/FC2), but not verification. **There is no clean ablation of "schema versus
     free text" with the same roles in any paper.**

### 7.2 Mechanisms that are only argued for (no experiment, or an insufficient one)
- Binding evidence to the code state (materialHash/headHash, freshness, command-set hash). Freshness,
  signatures and tamper classes were checked only by Proof-or-Stop's contract tests (10/10, 18/18). The powered ablation does
  **not** use the hashes. There is no study showing how often stale evidence occurs in real
  work.
- A 3×2 quorum and cross-vendor review. There is only observational data: 68 findings, a selective sample.
- A claim-boundary / wording gate, suppression of overclaiming in reports (Proof-or-Stop, ResearchLoop). The effect in
  ResearchLoop is borderline (p≈0.08) and is partly explained by the fact that fewer claims are simply emitted.
- A conversation-to-contract gate and minimal input artifacts (Agile-V) — concept only.
- A frozen read-only candidate for QA (HoH) — there is a rationale, no ablation.
- Requirement→evidence traceability as a gate. Used as an evaluation method (DevAI, rSDE-Bench), but
  the effect on outcomes as a control mechanism has not been measured.
- Quality evidence (erosion, verbosity) as a gate. SlopCodeBench shows that a prompt does not help. Nobody has tried
  quality checks as a gate.

### 7.3 Which failure modes are covered
| Failure mode (source) | Mechanism | Support status |
|---|---|---|
| Premature termination / false DONE (MAST FM-3.1; false success 45–76% of failures; OverclaimBench 80.4% misleading) | a gate with no term for self-report; a done-receipt; a ledger "14/15 ≠ done" | contract 10/10 (P); EviBound 0/8 (P, small n) |
| Superficial or wrong verification (MAST FM-3.2/3.3: "compiles / no TODOs") | executable tests, hidden/independent oracle, test-coverage lane | strong (EvoMAC, AgentCoder, Proof-or-Stop A3/A4) |
| Visible-pass / hidden-fail, test overfitting, cheating (ImpossibleBench up to 76%; SWE-bench 29.6%) | review as a mandatory gate; read-only tests; hidden oracle; an abort flag | measured (P) |
| Stale evidence after a code edit | materialHash freshness | mechanism only, frequency not measured |
| Loss of context between iterations / agents (MAST FC2; EvoDev) | typed handoff (Feature Map, planner_handoff), raw traces | measured (EvoDev R, HoH P, Meta-Harness P) |
| Regressions on changes (SWE-EVO P2P; harness edits) | a regression gate | measured (HarnessFix, Self-Harness P) |
| Self-preference and self-verification | separate authors of tests and code, cross-vendor review | partial (AgentCoder P; judges R); cross-vendor — observational |
| Degradation of code structure with green tests (SlopCodeBench) | no confirmed mechanism; prompts do not work | open |

**Main conclusion.** The 2023–2026 literature consistently supports three theses:
- acceptance should be decided by an executable or machine-checkable signal, not by an LLM's opinion or an agent's self-report;
- verification should be done by someone other than the author;
- review works when it **blocks** a transition rather than advises.

Typed documents (MetaGPT, EvoDev, HoH handoff) help coordination and context transfer. However,
there is no isolated measurement of "schema versus free text", and in MAST the structured MetaGPT is worse precisely in
verification. Cryptographic and hash binding of evidence, quorums and claim-boundary are so far backed by contract
tests and observations, not by controlled comparisons. The closest work, Proof-or-Stop, is a
preprint not verified by peer review, on a single model, and its main effect is concentrated in one task.
