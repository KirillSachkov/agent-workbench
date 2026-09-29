# Classic mechanics for an artifact-driven pipeline on GitHub

Date: 2026-09-25. Sources are primary where possible (specifications, GitHub Docs, authors).
A claim without direct verification is marked "unconfirmed". Terms are kept in English.

## 0. What matters most for our context

- The `sachkov-inside` organization is on the **Free** plan; `platform`, `inside-telegram`, `workspace`,
  `inside-landing` are **public**, `ai-engineering`, `inside-content`, `workshop-cases` are **private**
  (verified with `gh repo list sachkov-inside`, `gh api orgs/sachkov-inside`).
  - Artifact attestations: available in public repositories on all plans; for private/internal
    GitHub Enterprise Cloud is required; not supported on GHES
    ([actions/attest README](https://github.com/actions/attest),
    [attest-build-provenance README](https://github.com/actions/attest-build-provenance)).
    So they work now for `platform` and `inside-telegram`, but not for `ai-engineering`.
  - Merge queue: any public repository of the organization, or private with Enterprise Cloud
    ([Managing a merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)).
  - Free for organizations gives the "full feature set" on public repositories
    ([GitHub's plans](https://docs.github.com/en/get-started/learning-about-github/githubs-plans)),
    i.e. rulesets and required checks are available on public repositories.
- **Urgent:** starting **1 October 2026** the retention policy (90 days by default; public — at most 90,
  private — up to 400) applies to **checks, workflow runs and commit statuses**, not only to
  artifacts and logs. Before that they lived 400+ days. Retention also applies to data from third-party
  integrations ([Managing GitHub Actions settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository#configuring-the-retention-period-for-checks-workflow-runs-commit-statuses-artifacts-and-logs-in-your-repository),
  [changelog 2026-07-17](https://github.blog/changelog/2026-07-17-actions-retention-will-cover-checks-workflow-runs-and-statuses/)).
  Conclusion: check runs and statuses are a **gate**, not an archive of evidence. The durable record must
  live in git (a file) or in an attestation.
- Three layers worth separating: **specification** (what must be true: requirements with IDs, scenarios,
  contracts), **evidence** (what was verified: a machine report tied to a SHA), **gate** (what
  blocks merge: required check, ruleset, merge queue).

---

## 1. Requirements and traceability

### 1.1 EARS (Easy Approach to Requirements Syntax)

**What it is.** A constrained natural language for requirements, created by Alistair Mavin and colleagues
at Rolls-Royce, first published in 2009 (RE'09). Clauses always come in the same order:
`While <precondition>, when <trigger>, the <system> shall <response>`. Rule: zero or more
preconditions, zero or one trigger, one system, one or more responses
([alistairmavin.com/ears](https://alistairmavin.com/ears/)). Original paper: Mavin et al., "Easy Approach to Requirements Syntax (EARS)", RE'09 (bibliographic details from the author's page; the PDF was not opened).

Templates (same source):

| Template | Syntax |
|---|---|
| Ubiquitous | `The <system> shall <response>` |
| State-driven | `While <precondition>, the <system> shall <response>` |
| Event-driven | `When <trigger>, the <system> shall <response>` |
| Optional feature | `Where <feature is included>, the <system> shall <response>` |
| Unwanted behaviour | `If <trigger>, then the <system> shall <response>` |
| Complex | `While …, when …, the <system> shall …` (and combinations with If/Then) |

**Minimal example (our domain):**

```markdown
- REQ-PAY-003 (event): When the payment provider confirms a payment, the platform shall grant
  course access to the paying user within 60 seconds.
- REQ-PAY-004 (unwanted): If the payment notification signature is invalid, then the platform
  shall reject the notification and shall not change the order state.
- REQ-BOT-010 (state): While the user has no linked account, the Telegram bot shall offer the
  account-linking flow.
```

**What failure it prevents.** Vague requirements ("should work fast"), mixing of condition
and response, forgotten negative branches (the If/Then template forces you to write them). For agents it matters that
EARS can be checked by a regex linter (keywords, a single `shall`, presence of a system).

**Cost on GitHub.** Nearly zero: a template in the spec + a simple lint script in CI.

**Pitfalls.** EARS does not make a requirement verifiable by itself: "within 60 seconds" is
verifiable, "user-friendly" is not. One `shall` per requirement, otherwise traceability loses granularity.
EARS describes system behavior, not UI details; for UI a Gherkin scenario is better (section 2).

### 1.2 Requirement IDs, RTM and bidirectional traceability

**What it is.** Every requirement has a stable unique ID; a Requirements Traceability Matrix
(RTM) links requirement → design → code → test → result. Bidirectionality: forward
(every requirement is implemented and verified) and backward (every piece of code/test is justified by a requirement).
In DO-178C the traces run system req ↔ high-level req ↔ low-level req ↔ source code, and also
req ↔ test cases ↔ procedures ↔ results; depth depends on the DAL level (for DAL D, traces to
high-level requirements and tests are enough) — the sources are secondary, the RTCA standard itself is paid
([Parasoft: DO-178C traceability](https://www.parasoft.com/learning-center/do-178c/requirements-traceability/),
[Jama: DO-178C](https://www.jamasoftware.com/requirements-management-guide/aerospace-and-defense/do-178c/)).
ISO 26262-8 (clause 6) requires uniquely identified, versioned requirements and
bidirectional traceability across phases — per secondary sources, the text of the standard was not checked
([Sodius Willert](https://www.sodiuswillert.com/en/blog/maintaining-iso-26262-traceability-across-automotive-suppliers)) —
"unconfirmed" against the primary source.

**Mechanics worth adopting (without the certification weight):**
1. An ID is immutable and never reused; the ID carries a revision (`~1`, `~2`) when the meaning changes.
2. Every "lower" link references the "upper" one (`Covers: REQ-…`), and the "upper" one declares which
   kinds of coverage it needs (`Needs: impl, test`).
3. A tool in CI builds the graph and fails on: uncovered (no coverage of a required kind), orphan
   (a reference to a nonexistent ID), outdated (coverage references an old revision).

**What failure it prevents.** "Built, but the wrong thing"; a test without a requirement; a requirement that
silently dropped out during refactoring; an agent claiming "AC met" without binding it to a test.

### 1.3 Lightweight tools

**OpenFastTrace (OFT)** — Java, CLI; specifications in Markdown/RST, tags in code. The ID has the form
`<artifact-type>~<name>~<revision>`. In Markdown:

```markdown
`req~pay-grant-access~1`
When the payment provider confirms a payment, the platform shall grant course access.

Needs: impl, utest, itest
```

In code (TypeScript and Python are supported):

```ts
// [impl->req~pay-grant-access~1]
export async function grantAccess(orderId: string) { … }
```

In Gherkin (`.feature`): the tag `@id:scn~user-can-log-in~1` and the comments `# Covers: req~…~1`,
`# Needs: dsn, itest` before the scenario heading. Coverage with a revision:
`[impl~~2->dsn~validate-authentication-request~1]`
([OFT input formats](https://github.com/itsallcode/openfasttrace/blob/main/doc/user_guide/reference/input_format_support.md),
[OFT system requirements](https://github.com/itsallcode/openfasttrace/blob/main/doc/spec/system_requirements.md)).
The report marks `not ok`, including transitive defects
([console report](https://github.com/itsallcode/openfasttrace/blob/main/doc/user_guide/reference/console_tracing_report.md)).
Using the CLI exit code on traceability defects as a gate — "unconfirmed" (I did not read the
CLI exit codes section).

**StrictDoc** — Python; its own `.sdoc` format, export to HTML/ReqIF/JSON (export per the project
docs, not checked in detail). Example from the user guide:

```text
[REQUIREMENT]
UID: REQ-002
TITLE: Requirement #2's title
STATEMENT: Requirement #2 statement
RELATIONS:
- TYPE: Parent
  VALUE: REQ-001
- TYPE: File
  VALUE: /full/path/file.py
```

In source files the marker is `@relation(REQ-1, scope=function)` (also `scope=file|class`)
([StrictDoc user guide .sdoc](https://github.com/strictdoc-project/strictdoc/blob/main/docs/strictdoc_01_user_guide.sdoc)).

**Sphinx-needs** — a Sphinx extension: `.. req::` directives with `:id:`, `:status:`, `:links:`,
`needs.json` export, `needtable`/`needflow`, constraints and regexes on IDs
([sphinx-needs docs](https://sphinx-needs.readthedocs.io/en/latest/)). It requires a Sphinx build
of the documentation — somewhat heavy for our Markdown world.

**Doorstop** — Python; every requirement is a separate YAML file in git, a UID with a prefix (`REQ001`),
documents form a tree, there is reviewed/fingerprint for "suspect links", and the `doorstop` command
validates the tree in CI ([doorstop docs](https://doorstop.readthedocs.io/en/latest/)).

**Comparison for us.** None is native to TS. OFT is closest to our process (Markdown +
comments in TS + Gherkin), but it drags in a JVM. For a small team it is cheaper to write **a minimal
tracer of our own in Python in the harness**: an ID regex over `docs/specs/**/*.md`, `// covers: REQ-…` in
tests, Playwright `annotation`/`tag` with the ID (section 2.2), a JSON report → gate. OFT serves as the reference
for semantics (`Needs`/`Covers`/revision/outdated).

**Pitfalls.** Traceability for show: an agent puts `covers:` on a test that does not verify
the behavior. It is cured only by reviewing the test and by linking the requirement to a specific assertion
(a Gherkin step, a Playwright step). ID revisions: without them a change in the meaning of a requirement does not invalidate
old coverage.

---

## 2. Executable specifications

### 2.1 BDD / Gherkin and Specification by Example

**What it is.** BDD per Cucumber consists of three practices: Discovery (structured conversations around examples),
Formulation (writing examples down as structured documentation), Automation
([Cucumber: BDD](https://cucumber.io/docs/bdd/)). Gherkin is the format `Feature / Rule / Scenario /
Scenario Outline / Examples`, with `@…` tags on Feature, Rule, Scenario; `Rule` is a single business rule
(since Gherkin 6) ([Gherkin reference](https://cucumber.io/docs/gherkin/reference/)).
Specification by Example is a book by Gojko Adzic (2011), the same idea: examples as a single source for
requirements, tests and "living documentation"
([gojko.net](https://gojko.net/books/specification-by-example/)).

**Example:**

```gherkin
@id:scn~pay-grant-access~1
# Covers: req~pay-grant-access~1
Feature: Access after payment
  Rule: Confirmed payment grants access
    Scenario: Card payment confirmed
      Given a user with an unpaid order for "AI Engineering"
      When the provider sends a valid CONFIRMED notification
      Then the user sees the course in "My courses" within 60 seconds
```

**What failure it prevents.** Acceptance criteria that cannot be executed mechanically;
divergence between "what was promised" and "what was verified"; an agent interpreting AC freely.

**Cost.** Cucumber-js or `playwright-bdd` (third-party) — medium; a layer of step
definitions is needed. A cheaper alternative: Gherkin-like text in the issue as AC, with executability via a
Playwright test tagged with the same ID (2.2).

**Pitfalls.** Step-definition hell and imperative scenarios ("click button X"). Gherkin is valuable
at the level of business rules, not UI scripts. If nobody but agents reads the `.feature` files, the benefit of
Discovery is lost — only the format remains.

### 2.2 Playwright tests as acceptance evidence

**Mechanics.** Playwright supports `tag` (strings with `@`) and `annotation` (`{type, description}`),
which are available in the reporter API and visible in the HTML report; the filter is `--grep @tag`
([Playwright annotations](https://playwright.dev/docs/test-annotations)). The JSON reporter writes
the full run result (`--reporter=json`, `PLAYWRIGHT_JSON_OUTPUT_NAME`)
([Playwright reporters](https://playwright.dev/docs/test-reporters)).

```ts
test('card payment grants access', {
  tag: ['@acceptance'],
  annotation: [{ type: 'covers', description: 'REQ-PAY-003' },
               { type: 'issue', description: 'sachkov-inside/platform#812' }],
}, async ({ page }) => { /* … */ });
```

A script in CI reads `results.json`, builds `{requirementId → [test, status]}` and reconciles it with the
list of IDs from the task's spec. A missing or failed ID → check `failure`.

**Prevents.** "All tests are green", but the needed AC was not tested at all.
**Cost.** Low: a convention + ~100 lines of script. **Pitfalls.** Screenshots/video are an
illustration, not proof; the proof is the assertion. Flaky tests produce false "PASSED/FAILED":
retry accounting is needed in the report (Playwright marks flaky).

### 2.3 Contracts between producer and consumer

**Consumer-driven contracts (Pact).** The contract is generated by the consumer's tests; the provider
verifies it; results are stored in a Pact Broker; the "Pact Matrix" of versions and `can-i-deploy` decide
whether a version can be deployed to an environment (`record-deployment` records what is deployed where)
([Pact docs](https://docs.pact.io/),
[can-i-deploy](https://docs.pact.io/pact_broker/can_i_deploy)). Pact is good when both sides
are under your control and actively developed; bad for public APIs, pass-through APIs/BFFs and
situations where you cannot control the provider's data
([What is Pact good for](https://docs.pact.io/getting_started/what_is_pact_good_for)).

**Schema-first (OpenAPI / JSON Schema).** "Provider contract testing" checks that the provider's
behavior matches the documented contract (for example OpenAPI); by itself it does not
prove that the consumer calls the API correctly (same source, [Pact docs](https://docs.pact.io/)).
For CI there is `oasdiff` — an OpenAPI diff with `ERR/WARN/INFO` levels and `fail-on`
([oasdiff](https://github.com/oasdiff/oasdiff), [oasdiff-action](https://github.com/oasdiff/oasdiff-action)).

**Gate example:**

```yaml
- uses: oasdiff/oasdiff-action/breaking@<pinned-sha>
  with:
    base: 'origin/main:apps/api/openapi.json'
    revision: 'apps/api/openapi.json'
    fail-on: ERR
```

(the exact input names are "unconfirmed", check against the action's README).

**Prevents.** Silent breaking changes between the `platform` API and `inside-telegram`/web.
**Cost.** OpenAPI from NestJS (`@nestjs/swagger`) + oasdiff — low; Pact + Broker — medium
(a separate service or PactFlow). **For us:** web and API in one monorepo — shared TS types and a
generated client are cheaper than Pact; between repositories (`platform` ↔ `inside-telegram`) —
a committed `openapi.json` as an artifact + oasdiff. **Pitfalls.** OpenAPI generated from code
reflects the code, not the intent; schema-first (editing the schema before the code) gives more, but costs more.

---

## 3. Evidence and provenance tied to a commit

### 3.1 in-toto attestation framework

**What it is.** Layers: Envelope (DSSE, signature) → Statement → Predicate. Statement v1
([spec](https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md)):

```jsonc
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [{ "name": "<NAME>", "digest": { "<ALGORITHM>": "<HEX_VALUE>" } }],
  "predicateType": "<URI>",
  "predicate": { … }
}
```

A subject is matched **by digest only**; subjects are considered immutable. DigestSet
supports `sha256`, `sha512`, …, and also `gitCommit`, `gitTree`, `gitBlob`, `gitTag`
(SHA-1 of 40 characters or SHA-256 of 64) ([DigestSet](https://github.com/in-toto/attestation/blob/main/spec/v1/digest_set.md)).
There is a ready-made **Test Result** predicate `https://in-toto.io/attestation/test-result/v0.1`:
`result: PASSED|WARNED|FAILED`, `configuration`, `url`, `passedTests`, `warnedTests`,
`failedTests`; the subject is the tested source artifacts
([test-result predicate](https://github.com/in-toto/attestation/blob/main/spec/predicates/test-result.md)).

### 3.2 SLSA

**Build track** ([SLSA v1.1 levels](https://slsa.dev/spec/v1.1/levels)):
L1 — provenance exists (prevents release mistakes, for example building from a commit that
is not in upstream); L2 — signed provenance from a hosted build platform (prevents tampering
after the build); L3 — hardened builds, isolation of runs and signing secrets (prevents tampering
during the build by an insider/stolen credentials).

**Source track (SLSA v1.2)** ([source requirements](https://slsa.dev/spec/v1.2/source-requirements)):
L1 version controlled; L2 history & provenance (continuous immutable history + source
provenance attestations); L3 continuous technical controls (enforced rules on protected refs);
L4 two-party review. The revision's level is reported by a **Verification Summary Attestation**
(`https://slsa.dev/verification_summary/v1`) with `subject.digest.gitCommit` and `verifiedLevels`.
This is exactly our case: "revision X passed checks Y" as a signed document on a `gitCommit`.

GitHub states: artifact attestations by themselves provide **SLSA v1.0 Build L2**; a reusable
workflow as an isolated builder is the path to **L3**
([GitHub: artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations)).

### 3.3 GitHub artifact attestations

- `actions/attest@v4` — three modes: provenance (no predicate), SBOM (`sbom-path`), **custom**
  (`predicate-type` + `predicate` or `predicate-path`). Subject: `subject-path`,
  `subject-digest` (+`subject-name`), `subject-checksums`. Permissions: `id-token: write`,
  `attestations: write`, `artifact-metadata: write`. Limits: predicate ≤ 16 MB, ≤ 1024 subjects.
  `subject-digest` **must be `sha256:…`**; checksum files are sha256 (sha512 in a checksum file)
  ([actions/attest README](https://github.com/actions/attest)).
- `actions/attest-build-provenance` since v4 is just a wrapper over `actions/attest`
  ([README](https://github.com/actions/attest-build-provenance)).
- Public repo → Sigstore Public Good; the bundle is stored at GitHub **and written to a publicly readable
  transparency log**. Private → GitHub's own Sigstore without a transparency log
  ([GitHub docs](https://docs.github.com/en/actions/concepts/security/artifact-attestations)).
- REST: `GET /repos/{owner}/{repo}/attestations/{subject_digest}` (`sha256:HEX`, filter
  `predicate_type`), `POST /repos/{owner}/{repo}/attestations` accepts a Sigstore bundle
  ([REST attestations](https://docs.github.com/en/rest/repos/attestations)).
- `gh attestation verify <file> | oci://…` requires `--owner` or `--repo`; the default
  predicate is `https://slsa.dev/provenance/v1`, otherwise `--predicate-type`. Useful flags:
  `--signer-workflow`, `--signer-repo`, `--source-digest`, `--source-ref`,
  `--deny-self-hosted-runners`, `--format json`, `--jq`, `--bundle` (offline). Important: only
  `signature.certificate` (from the OIDC token) and `verifiedTimestamps` can be trusted;
  `statement.predicate` is controlled by the workflow and can be forged if the context is
  compromised — hence the "trusted builder" recommendation in a reusable workflow
  ([gh attestation verify](https://cli.github.com/manual/gh_attestation_verify)).
- GitHub explicitly advises **not** signing frequent builds "just for automated testing" and individual
  source/documentation files; attestations are useful only if they are verified
  ([GitHub docs](https://docs.github.com/en/actions/concepts/security/artifact-attestations)).

### 3.4 Check Runs, commit statuses, rulesets, merge queue

- **Check Runs API:** only a GitHub App can create them (OAuth/users can only read);
  `GITHUB_TOKEN` in Actions is a GitHub App token, and `checks: write` allows creating a check run
  ([Check runs REST](https://docs.github.com/en/rest/checks/runs),
  [workflow permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)).
  Fields: `name`, `head_sha`, `status`, `conclusion` (`success|failure|neutral|cancelled|skipped|
  timed_out|action_required`), `details_url`, `external_id`, `output{title, summary (Markdown),
  text, annotations, images}`. Annotations — at most 50 per request, the rest are appended via
  update; in Actions — 10 warning + 10 error per step; an annotation title ≤ 255 characters; up to 3
  `actions` buttons; up to 1000 check runs with the same name in one check suite, older ones are deleted.
  The `summary` length limit (often quoted as 65535) is "unconfirmed" in the current version of the docs.
- **Job summary** (`$GITHUB_STEP_SUMMARY`): ≤ 1 MiB per step, at most 20 summaries per job are shown
  ([workflow commands](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands)).
- **Commit statuses:** `error|failure|pending|success`, `context`, `target_url`, `description`;
  ≤ 1000 statuses per sha+context; push access is required
  ([statuses REST](https://docs.github.com/en/rest/commits/statuses)).
- **Required status checks in rulesets:** you can specify an **expected source** — a specific GitHub App;
  a status from a different source does not unblock the merge. Without this "anyone with write access can
  set any status" ([available rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)).
  Rulesets: up to 75 per repository, visible to everyone with read access, layered with branch protection
  ([about rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)).
- **Merge queue:** the workflow must listen to `merge_group`, otherwise the required check will not arrive and the merge
  will fail; the queue builds a temporary branch `main/pr-N` with the base and the preceding PRs
  ([merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)).
- **SHA trap:** on `pull_request`, `GITHUB_SHA` is the latest merge commit of the branch
  `refs/pull/N/merge`, not the PR head; the head is `github.event.pull_request.head.sha`. On
  `merge_group` — the SHA of the merge group ([events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)).
  Result: the same "evidence" exists for at least three different SHAs (PR head,
  merge-ref, merge_group), and after a squash into `main` — for a fourth.
- **Rate limits:** `GITHUB_TOKEN` — 1000 requests/hour per repository; secondary — ≤ 80
  content-generating requests/min and ≤ 500/hour
  ([REST rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)).

### 3.5 How to make an "acceptance evidence" attestation for a SHA and verify it before merge

Problem: GitHub stores and looks up attestations by the file's **sha256 digest**, while a commit is a
`gitCommit` (SHA-1). So the subject is the evidence file itself, and the commit goes inside the predicate **and** into
the certificate (the source repository digest/ref fields from OIDC, checked with `--source-digest`,
`--source-ref`). That `source-digest` in the certificate equals the run's `github.sha` follows from the
logic of the OIDC claims and was not verified experimentally ("unconfirmed").

**Workflow (public repository `platform`):**

```yaml
name: acceptance-evidence
on:
  merge_group:          # gate on the resulting tree that will land in main
  pull_request:         # early feedback
permissions:
  contents: read
  checks: write
  id-token: write
  attestations: write
  artifact-metadata: write
jobs:
  acceptance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<sha>
      - run: PLAYWRIGHT_JSON_OUTPUT_NAME=pw.json pnpm playwright test --grep @acceptance --reporter=json
      - run: python3 tools/evidence.py --spec docs/specs/PAY-812.md --results pw.json
               --commit "${{ github.sha }}" --out evidence/acceptance.json
        # fails if any REQ from the spec is not covered by a PASSED test
      - if: github.event_name == 'merge_group'
        uses: actions/attest@v4
        with:
          subject-path: evidence/acceptance.json
          predicate-type: https://in-toto.io/attestation/test-result/v0.1
          predicate-path: evidence/predicate.json
      - uses: actions/upload-artifact@<sha>
        with: { name: acceptance-evidence, path: evidence/, retention-days: 90 }
```

**predicate.json (standard test-result, our IDs in test names):**

```json
{
  "result": "PASSED",
  "configuration": [{ "name": "playwright.config.ts", "digest": { "gitBlob": "…" } },
                    { "name": "spec", "uri": "git+https://github.com/sachkov-inside/platform@<sha>#docs/specs/PAY-812.md" }],
  "url": "https://github.com/sachkov-inside/platform/actions/runs/<run_id>",
  "passedTests": ["REQ-PAY-003 :: card payment grants access"],
  "warnedTests": [],
  "failedTests": []
}
```

**Verification:**

```bash
gh attestation verify evidence/acceptance.json \
  --repo sachkov-inside/platform \
  --predicate-type https://in-toto.io/attestation/test-result/v0.1 \
  --signer-workflow sachkov-inside/platform/.github/workflows/acceptance-evidence.yml \
  --source-digest "$SHA" --deny-self-hosted-runners \
  --format json --jq '.[0].verificationResult.statement.predicate.result'
```

**Honest assessment.** A pre-merge gate is cheaper and more reliable to build as a **required check** (the conclusion of the
check run = the result of `evidence.py`) with expected source = GitHub Actions; an attestation adds
not blocking but a **durable, signed, externally verifiable record** (it outlives check
retention; for public repositories it is also in the transparency log). An attestation is useful as an audit of "what was accepted and
what proved it", at release time or for an external verifier. There is no need to sign every push
(GitHub's advice above) — only `merge_group`/release. Not available for private repositories on Free;
the replacement is an evidence file in git + a commit status/check.

**Pitfalls.** (1) The predicate is written by the workflow — if an agent can change the workflow in the same PR, it can
forge the evidence; cured by `--signer-workflow`, a reusable workflow from a separate repo, and CODEOWNERS
on `.github/`. (2) The public transparency log: no personal data/secrets in the predicate.
(3) The SHA trap from 3.4. (4) "The merge group SHA will become a commit in `main`" — "unconfirmed",
depends on the merge method; also bind `gitTree` in the predicate.

---

## 4. Assurance cases: GSN, SACM, CAE

**GSN.** Goal Structuring Notation, an SCSC community standard, current version — **3**
([SCSC GSN standard](https://scsc.uk/gsn-standard)). Elements: Goal (a claim), Strategy
(how it was decomposed), Solution (a reference to evidence), Context, Assumption, Justification; the relations
SupportedBy and InContextOf; "undeveloped" for unproven branches (the set of elements is per the
v3 standard, the PDF was not opened in this session; "unconfirmed" in the details).

**CAE (Adelard / Bloomfield).** Claims (an assertion that may be true or false), Arguments
(why the evidence supports the claim), Evidence (artifacts that establish facts); "CAE building
blocks" are typical decomposition templates (decomposition, substitution, concretion, calculation,
evidence incorporation) ([Adelard CAE](https://www.adelard.com/asce/cae/),
[Bloomfield & Netkachova, Building Blocks](https://openaccess.city.ac.uk/id/eprint/5121/1/BuildingBlocksforAssuranceCases.pdf)).

**SACM.** The OMG Structured Assurance Case Metamodel: combines an argumentation and an artifact
metamodel, serves as an interchange format for GSN and CAE. The OMG page lists **2.4 beta**
(September 2026) and formal versions up to 2.3 ([OMG SACM](https://www.omg.org/spec/SACM/)).

**Mapping to agent records ("claim → evidence"):**

```yaml
# .evidence/PAY-812.claims.yaml  (append-only, one writer — CI)
claim: C1
statement: "REQ-PAY-003 is satisfied at commit <sha>"
context: [spec: docs/specs/PAY-812.md@<sha>]
argument: "Scenario scn~pay-grant-access~1 passes on merge_group; the test verifies that access is granted"
evidence:
  - kind: test-result
    ref: attestation:<id> | run:<url>
    subject_digest: sha256:…
assumptions: ["the provider in the sandbox emulates CONFIRMED the same way as prod"]
status: supported | undeveloped | defeated
```

**What failure it prevents.** The main disease of agent reports is "done, everything works" with no
link to evidence. CAE forces separating the **claim**, the **argument** and the
**evidence**, and `undeveloped`/`assumptions` make the gaps visible to the owner.

**Cost.** The format is low-cost (YAML + schema). A full GSN editor is not needed.
**Pitfalls.** An assurance case easily turns into paperwork; keep 1 claim per requirement and
evidence only of machine origin (a reference to a run/attestation), and keep the argument text short.
Judging how convincing the argument is remains with a human.

---

## 5. Design by Contract, DoR/DoD, fitness functions

**Design by Contract (Meyer, Eiffel).** Preconditions (the client's obligation), postconditions
(the supplier's obligation), invariants ([Eiffel: DbC](https://www.eiffel.com/values/design-by-contract/introduction/)).
Transfer to the pipeline: each stage (spec → tickets → implementation → review) is a "method" with
pre/postconditions over artifacts. Example: the precondition for `implementation` = the issue has a
valid spec block, all REQ-IDs are in EARS, `Needs` is present; the postcondition = the evidence covers all
REQ-IDs and the check `acceptance` = success.

**Definition of Ready / Done.** Scrum Guide: the DoD is "a formal description of the state of the Increment
when it meets the quality measures required"; a backlog item that meets the DoD becomes an
Increment ([Scrum Guide 2020](https://scrumguides.org/scrum-guide.html)). The Scrum Guide has no DoR —
it is a team practice. Mechanics: DoR/DoD as a **list of machine-checkable predicates** (JSON Schema
of the issue block, presence of links, required checks), not a markdown checklist that the agent "ticks".

```yaml
# harness/gates/dod.yaml
- id: dod.spec-linked      check: pr.body.links.issue != null
- id: dod.reqs-covered     check: evidence.uncovered == []
- id: dod.contract-stable  check: oasdiff.breaking == [] or pr.labels has 'breaking-approved'
- id: dod.owner-approved   check: review.approved_by contains owner   # a human, not an agent
```

**Architecture fitness functions.** From *Building Evolutionary Architectures* (Ford, Parsons, Kua,
Sadalage): an objective automated assessment of an architectural characteristic — tests, metrics,
monitors ([Thoughtworks Radar](https://www.thoughtworks.com/radar/techniques/architectural-fitness-function),
[O'Reilly, 2nd ed.](https://www.oreilly.com/library/view/building-evolutionary-architectures/9781492097532/) — the publisher's page did not open, the author list of the 2nd edition is "unconfirmed").
For us: dependency-cruiser/eslint-boundaries (web does not import backend), a bundle
size limit, oasdiff, "every NestJS controller has an OpenAPI description", p95 response time in
smoke. It combines well with the MADR `Confirmation` section (6.4): decision → fitness function.

**Prevents.** Architecture erosion and box-ticking in the DoD. **Cost** — low, most
checks already exist as lint/test. **Pitfalls.** A fitness function without an owner and a threshold → noise; the threshold
changes only through an ADR.

---

## 6. Artifact lifecycle

**Immutable records + append-only log vs mutable documents.** Event Sourcing (Fowler): state
is derived from a sequence of events that are not rewritten
([Fowler: Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)). Rule for us:
the **specification** is a mutable document with a version (edited via PR, the ID revision grows);
**evidence, acceptance decisions, attestations** are immutable, each record is tied to a SHA and
is never edited; a correction is a new record referencing the old one (`supersedes`).

**Artifact state machine.** Explicit states and permitted transitions, checked by CI:

```text
spec:     draft → proposed → approved(owner) → superseded
evidence: pending → passed | failed   (terminal; a new SHA = a new record)
adr:      proposed → accepted | rejected → deprecated | superseded-by:ADR-NNNN
```

Only the owner can make the `approved` transition (checking the event author/label via a ruleset or
an Action). An invalid transition → check failure.

**Single-writer ownership.** Martin Thompson's principle: each piece of state has one writer
— it eliminates conflicts and races ([Single Writer Principle](https://mechanical-sympathy.blogspot.com/2011/09/single-writer-principle.html); the page did not open in this session — the wording is from memory, "unconfirmed").
Transfer: each artifact field has one write owner — the agent writes spec/code, **only CI**
writes evidence and check conclusions, **only the owner** sets approval. This also protects against
an agent that "issues itself the evidence".

**ADR lifecycle.** Nygard (2011): a short file, numbers are sequential and never reused,
a reversed decision is not deleted but marked superseded; statuses proposed / accepted /
deprecated / superseded; sections Context, Decision, Status, Consequences
([Nygard](https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions)).
MADR 4.0.0 (2024-09-17): front matter `status: proposed | rejected | accepted | deprecated |
superseded by ADR-0123`, `decision-makers`, sections Considered Options, Decision Outcome,
Consequences, **Confirmation** (how to verify the decision is being followed)
([MADR](https://adr.github.io/madr/)). The front matter is validated with JSON Schema in CI; the `superseded by`
reference is checked for existence.

**Prevents.** Loss of decision history, "silent" edits to accepted criteria, races of several
agents in one file. **Pitfalls.** Immutable records in git accumulate; keep them small (JSON) and
reference heavy things (reports, screenshots) via artifacts/attestation digest.

---

## 7. GitHub-native stores for structured artifacts

| Store | What is good | Limitations / risks | Suited for |
|---|---|---|---|
| **Issue forms** (`.github/ISSUE_TEMPLATE/*.yml`) | Typed fields, `validations.required`, dropdown/checkbox | Public preview; answers become **markdown in the body**, and the body can then be freely edited; not supported for PRs ([issue forms syntax](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms)) | Task intake, DoR fields at creation |
| **Fenced JSON/YAML in the issue/PR body** + an Action parser | Machine-readable, visible to humans, validated with JSON Schema on `issues.edited`/`pull_request` | No field-level history (only the body edit history); anyone with edit rights can change it; competing edits by agents | The task's spec block, PR evidence summary |
| **Projects custom fields** | Types text/number/date/single-select/iteration; filters | Up to 50 fields per project, up to 50,000 items ([understanding fields](https://docs.github.com/en/issues/planning-and-tracking-with-projects/understanding-fields), [changelog](https://github.blog/changelog/2025-02-26-increased-items-in-github-projects-now-in-public-preview/)); GraphQL only; not versioned | Status/stage, owner, links |
| **Issue fields (org-level)** | Typed fields on all issues of the organization; single-select, text, number, date; up to 25 per org; visibility Public/Organization only ([issue fields](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/managing-issue-fields-in-your-organization)) | A new feature, API/availability on Free — "unconfirmed" | Replacing some Project fields |
| **Files in the repo + front matter + JSON Schema in CI** | Versioning, review, binding to a SHA, blame, ruleset/CODEOWNERS | A PR is needed for every change; merge conflicts | Specifications, ADRs, claims/evidence index |
| **Check run output / commit status** | Tied to `head_sha`, gate via required checks, annotations on lines | Only a GitHub App/`GITHUB_TOKEN` can write; annotations 50/request; **retention 90 days by default from 01.10.2026**, for public at most 90 | Gate and verification report |
| **Workflow artifacts** | Large files (reports, trace, video), `retention-days` | Default retention 90 days; public 1–90, private 1–400 ([storing workflow data](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/storing-and-sharing-data-from-a-workflow), [retention settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)); storage quotas by plan ([Actions limits](https://docs.github.com/en/actions/reference/limits)) | Heavy evidence |
| **Attestations** | Sigstore signature, OIDC identity of the workflow, lookup by digest, transparency log for public | Public on any plan, private — only Enterprise Cloud; subject only sha256; predicate ≤ 16 MB; deletion follows the lifecycle ([manage attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/manage-attestations)); GitHub's retention period — "unconfirmed" | Durable acceptance at merge/release |
| **git notes** | Metadata for a commit without changing the SHA | Not pushed/fetched by default (refspec `refs/notes/*` needed); the GitHub UI does not show them ([Ken Muse](https://www.kenmuse.com/blog/storing-data-in-git-objects-with-notes/)); rulesets protect branches/tags, not `refs/notes` — an inference, "unconfirmed" | I do not recommend as a source of truth |

General API limits: `GITHUB_TOKEN` 1000 req/h/repo; secondary 80 content-creating/min, 500/h
([rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)).
Agents that mass-edit issues/Project fields run into precisely the secondary limits.

---

## 8. Recommended minimal toolkit

Principle: **the specification lives in git, the gate in checks, durable evidence in git + attestation**.
Issue/Project are navigation and status, not the source of truth for criteria.

1. **A spec file per task** `docs/specs/<issue>.md` with YAML front matter
   (`id, issue, status, owner, supersedes`) and requirements in **EARS** with IDs `REQ-<AREA>-<NNN>`
   (a revision on a change of meaning). A JSON Schema for the front matter + an EARS lint (Python in the harness).
   The issue form for creating a task references this file.
2. **Traceability with our own script following OFT semantics**: `Needs` on a requirement, `covers` in Playwright
   `annotation` / vitest comments; an `uncovered / orphan / outdated` report → required check
   `trace`. Adopt OFT only if our own script starts to grow.
3. **Acceptance evidence** — Playwright JSON → `evidence.py` → check run `acceptance` (summary =
   a REQ → test → status table, annotations on uncovered REQs). Required, expected source =
   GitHub Actions. The workflow listens to `pull_request` and **`merge_group`**.
4. **Contracts**: a committed `openapi.json` from NestJS + `oasdiff` (`fail-on: ERR`) as a required
   check; permission for a breaking change — only via the owner's label. Do not introduce Pact until there are several
   independently deployed consumers.
5. **Attestation on merge_group** (public `platform`, `inside-telegram`): `actions/attest@v4` with
   predicate `test-result/v0.1` on the evidence file; `gh attestation verify` with
   `--signer-workflow` and `--source-digest` in the release step. For private repositories — only the
   evidence file + check.
6. **A claims file following CAE** (`.evidence/<issue>.claims.yaml`), written **only by CI**; the agent in the PR
   references the claim ID rather than retelling "everything was checked".
7. **Rulesets**: required checks `trace`, `acceptance`, `contract`, `schema`; CODEOWNERS on
   `.github/workflows/**`, `harness/**`, `docs/specs/**` with status approved; force-push forbidden.
8. **ADRs in MADR 4** with a mandatory `Confirmation` referencing a fitness function; front matter
   is validated, `superseded by` is checked.
9. **Retention**: before 1 October 2026 set retention as needed (public at most 90 days) and
   treat checks as ephemeral; anything needed longer goes in git/attestation.

Order of adoption by return: (1)+(3)+(7) → (2) → (4) → (8) → (5)+(6).
