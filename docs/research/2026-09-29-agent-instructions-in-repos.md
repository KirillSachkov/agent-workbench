# How real repositories organise agent instructions and skills

Date: 2026-09-29. Question from #40 (part of #5, blocks E18 #23): how widely used repositories, not
harness frameworks, organise `AGENTS.md`, `CLAUDE.md` and skills in practice, and which components
harnesses and pipelines beyond the eight already compared tend to contain.

Method.

- **Sample.** 46 widely used repositories across languages and organisations, picked by hand for
  reach: vendor repositories (OpenAI, Anthropic, Microsoft, Google, Vercel), language and runtime
  projects (CPython, Rust, Go, Node, Deno, Bun), frameworks and large applications. The recursive git
  tree of each default branch was read with `gh api repos/<repo>/git/trees/<branch>?recursive=1` on
  2026-09-29, and the commit read is listed per repository in §1.1. Four trees came back truncated
  by the API (rust-lang/rust, microsoft/TypeScript, microsoft/typescript-go, PostHog/posthog), so
  nested-file counts for them are lower bounds.
- **Content.** Root `AGENTS.md` and `CLAUDE.md` blobs were read with `gh api .../git/blobs/<sha>`.
  Symlinks were identified by git mode `120000` and resolved from their blob content.
- **History.** `gh api --paginate repos/<repo>/commits?path=<file>` gave the commit count, the
  distinct authors, the first commit and AI co-author trailers. This endpoint does not follow
  renames, so the "first commit" is sometimes the rename.
- **Ecosystem-wide counts.** `gh api search/code` `total_count` values. GitHub reports these as
  approximate, and whether symlinked paths are indexed is unconfirmed.
- **Tools and studies.** `vercel-labs/skills` was read from a shallow clone at `3694740`. Next.js, Nx,
  Svelte CLI and Codex sources were read with `gh api`. arXiv abstracts came from the arXiv API.
- **§3.** The harness component inventory came from a delegated reading pass over each harness's own
  repository and first-party posts. Its sources are listed in §3.

Citation shorthand: `<owner>/<repo>@<sha>:<path>` means
`https://github.com/<owner>/<repo>/blob/<sha>/<path>`.

Already covered elsewhere and not repeated:

- [cross-runtime-portability](2026-09-29-cross-runtime-portability.md) covers which runtime reads
  which file and path.
- [harness-frameworks-compared](2026-09-29-harness-frameworks-compared.md) covers spec-kit,
  OpenSpec, BMAD, Superpowers, Kiro, GSD, Ruflo and Agent OS.
- [mattpocock-skills-design](2026-09-29-mattpocock-skills-design.md) covers Matt Pocock's skills.

---

## 0. Summary

1. **`AGENTS.md` is the canonical file in most sampled repositories that have one.**
   - Of 46 repositories, 37 have a root `AGENTS.md` or `CLAUDE.md`. The content lives in `AGENTS.md`
     in 28, in `CLAUDE.md` in 8 and in Zed's `.rules` in 1.
   - gemini-cli uses `GEMINI.md`. Eight have no root instruction file. Three of those
     (microsoft/TypeScript, microsoft/typescript-go, django) use `.github/copilot-instructions.md`
     instead.
   - The trend is migration toward `AGENTS.md`: next.js, ruff, n8n, sentry and supabase all renamed
     `CLAUDE.md` to `AGENTS.md` (§1.3).
2. **When both files exist, the bridge is a symlink (8 of 18) or a one-line `@AGENTS.md`
   import (8 of 18).**
   - 13 repositories ship `AGENTS.md` with no `CLAUDE.md` at all.
   - Next.js deleted its `CLAUDE.md` symlink on 2026-09-22 because "Claude Code now reads
     `AGENTS.md` directly".
   - Copying the content into both files was not observed. The only two repositories with separate
     contents hold different material in each.
3. **Maintainers write the files by hand and many contributors edit them.**
   - Examples: PostHog has 169 commits by 40 authors, openai/codex 77 by 22, apache/airflow 69 by
     19. PyTorch's first commit is "A basic CLAUDE.md based on bad things I see claude code doing".
   - Only one of 37 root files keeps the `/init` boilerplate header, and none uses the Codex `/init`
     title.
   - AI co-author trailers appear in the history of 21 of 40 files, so agents help edit them.
   - A median root file is about 9.4 KB and 172 lines. 17 of 37 exceed the 200 lines Claude Code
     recommends, and 2 exceed Codex's default 32 KiB cap.
4. **Nested files are common in monorepos and absent elsewhere.**
   - Nested instruction files show up in 18 of the 37 repositories with a root file. The largest are
     PostHog (55+), opencode (17), n8n (25), airflow (13), grafana (12) and workers-sdk (8).
5. **Language and runtime projects use the file for AI policy, not productivity.**
   - CPython, Node.js and Rust ship an `AGENTS.md` whose main job is to limit what agents may do.
   - Django's `.github/copilot-instructions.md` tells Copilot not to review pull requests at all.
6. **Skills are committed to the repository, in one canonical directory.**
   - 32 of 46 repositories commit skills:
     - `.agents/skills`: 12
     - `.claude/skills`: 10
     - `.github/skills` (all Microsoft): 3
     - a neutral `skills/` or `.ai/skills`: 3
     - `.codex/skills`: 2
     - `.opencode/skills`: 2
   - No sampled repository installs its own skills per machine.
   - Multi-runtime repositories keep one real directory and point the other runtimes at it:
     - a directory symlink (next.js, sentry, PostHog, supabase, vercel/ai, tldraw)
     - per-skill symlinks (airflow)
     - a Claude Code plugin wrapper (n8n, ruff)
     - locally created symlinks from a `make` target (transformers)
7. **`vercel-labs/skills` writes three things into a project.**
   - A canonical copy in `.agents/skills/<name>/`.
   - A relative symlink from each non-universal runtime directory (for Claude Code,
     `.claude/skills/<name>`).
   - A committed `skills-lock.json` recording source, ref, path and a SHA-256 of the files.
   - Next.js is the one sampled repository with that lock file, for a single third-party skill
     whose files are also committed.
8. **Tools that must put text into a project's `AGENTS.md` use a managed block.**
   - Next.js (`next dev`, `create-next-app`) and Nx (`nx configure-ai-agents`) write a marked
     block and update only between the markers.
   - Codex `/init` refuses to touch an existing `AGENTS.md`.
9. **Harness components (19 harnesses, §3).**
   - Recurring: the spec → plan → tasks trio, slash commands, an installer CLI, a loop runner,
     persistent context docs, subagents and an automated review step.
   - Rare: a constitution, the harness's own MCP server, evals of the harness itself, CI
     integration, audit trails, doc-gardening agents.

**Direct answer: should the harness create `AGENTS.md`, or leave it to each project?**

- **The project owns `AGENTS.md`, and the harness should not generate its content.**
  - Every sampled file is hand-written and specific to its project.
  - Generated overviews did not help agents in a controlled study (§1.6).
  - "Init fossilization" is one of the six measured smells.
- **The harness should do three narrow things.**
  1. **Create a minimal skeleton only when no file exists**, never overwriting one, as Codex `/init`
     does.
  2. **Own one clearly marked managed block** that it may rewrite, holding a pointer to its skills
     and pipeline. This is the Next.js and Nx pattern.
  3. **Wire the mechanical bridge.** That means a one-line `@AGENTS.md` in `CLAUDE.md`, or nothing
     where the runtime reads `AGENTS.md` natively, plus skill directory links.
- **Everything else stays the project's:** content, nesting, size, and the canonical file name
  (some projects keep `CLAUDE.md` canonical).
- A skill that teaches how to edit `AGENTS.md` (PostHog's `editing-agents-md`) is a better harness
  contribution than generated text.

---

## 1. `AGENTS.md` and `CLAUDE.md` in popular repositories

### 1.1 The sample

Legend:

- **Root file:** where the root instructions actually live, with bytes / lines.
- **Bridge:** how the other name reaches that content.
- **Nested:** instruction files below the root, excluding vendored code and test fixtures.
- **Skills:** the committed skill directory, with the count of `SKILL.md` directories.
- **Sha:** the commit read. Links are `https://github.com/<repo>/tree/<sha>`.

| Repository | Stars | Root file | Bridge | Nested | Skills (canonical → other runtimes) | Sha |
|---|---|---|---|---|---|---|
| openai/codex | 127k | `AGENTS.md` 22,397 / 320 | no `CLAUDE.md` | 1 | `.codex/skills` (11) | `0b1b78a4f1` |
| openai/openai-agents-python | 30k | `AGENTS.md` 38,241 / 214 | `CLAUDE.md` → `AGENTS.md` symlink | 0 | `.agents/skills` (14), plus `.agents/references/` | `7b27134b2d` |
| anthropics/claude-code | 149k | none | — | 0 | only `.claude/commands` (3, issue triage) | `dec92bc87a` |
| anthropics/claude-agent-sdk-python | 8k | `CLAUDE.md` 683 / 27 | no `AGENTS.md` | 0 | `.claude/skills` (1), `.claude/agents`, `.claude/commands` | `37422c21d8` |
| anthropics/anthropic-sdk-python | 4k | `CLAUDE.md` 31,559 / 368 | no `AGENTS.md` | 0 | none | `a7285e919a` |
| vercel/next.js | 143k | `AGENTS.md` 29,546 / 527 | none (symlink removed, #99043) | `.github/AGENTS.md` + eval fixtures | `.agents/skills` (20) → `.claude/skills` dir symlink; `skills-lock.json` | `b829c6189b` |
| vercel/ai | 27k | `AGENTS.md` 13,420 / 320 | none at root; nested `CLAUDE.md` = `@AGENTS.md` | 2 | `skills/` → `.agents/skills`, `.claude/skills` dir symlinks | `0116f28486` |
| microsoft/vscode | 193k | `AGENTS.md` 271 B pointer to `.github/copilot-instructions.md` | none | 7 | `.github/skills` (47), `.agents/skills` (1) | `318df148eb` |
| microsoft/TypeScript | 111k | none; `.github/copilot-instructions.md` | — | truncated | `.github/skills` (4) | `673a5f17d7` |
| microsoft/typescript-go | 26k | none; `.github/copilot-instructions.md` | — | truncated | `.github/skills` (1), `.github/agents` (3) | `89d5d5b284` |
| microsoft/playwright | 97k | `CLAUDE.md` 7,890 / 164 | no `AGENTS.md` | 0 | `.claude/skills` (5) | `d67c16ec4e` |
| google-gemini/gemini-cli | 107k | `GEMINI.md` | — | 7 `GEMINI.md` | `.gemini/commands` (TOML) | `fe6350238c` |
| react/react (was facebook/react) | 251k | `CLAUDE.md` 359 / 13 | no `AGENTS.md` | `compiler/CLAUDE.md` | `.claude/skills` (7) | `7c6ac13e19` |
| rust-lang/rust | 119k | `AGENTS.md` 13,435 / 258 (LLM policy) | `CLAUDE.md` = `@AGENTS.md` | rust-analyzer (`AGENTS.md` → `CLAUDE.md`) | none | `ea6bb45b74` |
| python/cpython | 77k | `AGENTS.md` 603 / 16 (policy pointer) | none | 0 | none | `dc0b1f8dfb` |
| astral-sh/uv | 90k | `AGENTS.md` 2,432 / 33 | none | 0 | `.codex/skills` (1) | `73b640e8b5` |
| astral-sh/ruff | 50k | `AGENTS.md` 18,631 / 218 | `CLAUDE.md` = `@AGENTS.md` | 0 | `.agents/skills` (4) → Claude via a directory-marketplace plugin | `0d7e203568` |
| denoland/deno | 109k | `CLAUDE.md` 11,866 / 433 | no `AGENTS.md` | 0 | `.claude/skills` (6) | `1b48a20f85` |
| oven-sh/bun | 96k | `CLAUDE.md` 17,614 / 240 | `AGENTS.md` → `CLAUDE.md` symlink (root + 3 nested) | 10 | `.claude/skills` (8), `.claude/commands`, `.claude/hooks` | `9f70da0741` |
| supabase/supabase | 111k | `AGENTS.md` 6,814 / 75 | `CLAUDE.md` = `@AGENTS.md` (root + 3 nested) | 3 | `.agents/skills` (22) → `.claude/skills` dir symlink | `5b18a5d1af` |
| langchain-ai/langchain | 147k | `AGENTS.md` 20,580 / 390 | none | 0 | none | `aaf25d0abd` |
| huggingface/transformers | 167k | `.ai/AGENTS.md` 3,287 / 40 | `AGENTS.md` and `CLAUDE.md` → `.ai/AGENTS.md` | 0 | `.ai/skills` → local symlinks created by `make` | `da4bd0dca1` |
| pytorch/pytorch | 104k | `CLAUDE.md` 15,226 / 343 | `AGENTS.md` → `CLAUDE.md` symlink | `torch/_dynamo/CLAUDE.md` | `.claude/skills` (18), `.claude/hooks` | `71d9ed283f` |
| kubernetes/kubernetes | 128k | `AGENTS.md` 1,584 / 38 | none | vendored only | none | `36702c3835` |
| golang/go | 139k | none | — | 0 | none | `522ebc8370` |
| n8n-io/n8n | 206k | `AGENTS.md` 20,892 / 408 | `CLAUDE.md` = `@AGENTS.md` | 15 `AGENTS.md`, 10 `CLAUDE.md` | `.agents/skills` (24) → Claude plugin `.claude/plugins/n8n` with per-skill symlinks | `55fd9daa08` |
| anomalyco/opencode (was sst/opencode) | 211k | `AGENTS.md` 8,748 / 161 | none | 17 | `.opencode/skills` (2) | `7945de2089` |
| zed-industries/zed | 91k | `.rules` 12,461 / 190 | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md` → `.rules` | 1 (`docs/AGENTS.md`) | `.agents/skills` (4) | `adfcaa122a` |
| getsentry/sentry | 45k | `AGENTS.md` 6,589 / 129 | `CLAUDE.md` = `@AGENTS.md` (root + 3 nested) | 4 | `.agents/skills` (30) → `.claude/skills` dir symlink | `7ca2189492` |
| PostHog/posthog | 40k | `AGENTS.md` 43,183 / 326 | `CLAUDE.md` → `AGENTS.md` symlink (root + nested) | 55+ (truncated) | `.agents/skills` (108) → `.claude/skills` dir symlink; `.claude/agents` (11), `.claude/hooks` | `99a0f7a090` |
| cloudflare/workers-sdk | 5k | `AGENTS.md` 9,450 / 157 | `CLAUDE.md`: `/init` header + "See @AGENTS.md" | 8 | `.opencode/skills` (1) | `485cfb3abf` |
| withastro/astro | 63k | `AGENTS.md` 8,903 / 165 | none | 0 | `.agents/skills` (10), `.agents/evals` | `6ce6ae97c4` |
| sveltejs/svelte | 88k | `AGENTS.md` 590 / 11 | none | 0 | `.agents/skills` (1) | `020242d6be` |
| nodejs/node | 122k | `AGENTS.md` 3,083 / 64 (policy) | none | 0 | none | `1f26576a3f` |
| electron/electron | 123k | `CLAUDE.md` 13,960 / 342 | no `AGENTS.md` | 2 | `.claude/skills` (5) | `2454be571a` |
| home-assistant/core | 91k | `AGENTS.md` 5,556 / 56 | `CLAUDE.md` → `AGENTS.md` symlink | 0 | `.claude/skills` (7) ← `.gemini/skills` symlink; `.claude/agents` | `4c971107fd` |
| django/django | 91k | none; `.github/copilot-instructions.md` refuses reviews | — | 0 | none | `9332b163a6` |
| rails/rails | 59k | `AGENTS.md` 6,905 / 201 | none | 0 | none | `7e1cb3bfa5` |
| biomejs/biome | 26k | `AGENTS.md` 4,521 / 82 | `CLAUDE.md` → `CONTRIBUTING.md` symlink | 0 | `.claude/skills` (11) with a catalog `README.md` | `034366def4` |
| tauri-apps/tauri | 111k | none | — | 0 | none | `c9a3cb892e` |
| shadcn-ui/ui | 125k | none at root | — | 2 | none | `db2db460a2` |
| grafana/grafana | 77k | `AGENTS.md` 9,987 / 172 | `CLAUDE.md` = `@AGENTS.md` | 12 | `.claude/skills` (3) | `e95d104245` |
| apache/airflow | 47k | `AGENTS.md` 22,493 / 250 | `CLAUDE.md` → `AGENTS.md` symlink | 13 | `.agents/skills` (8) → per-skill symlinks in `.claude/skills` (6), `.github/skills` (4) | `d776f003b7` |
| tldraw/tldraw | 51k | `AGENTS.md` 13,439 / 228 | `CLAUDE.md` = `@AGENTS.md` | 0 | `skills/` → `.agents/`, `.claude/`, `.cursor/skills` dir symlinks | `da7162e13b` |
| excalidraw/excalidraw | 133k | `AGENTS.md` 373 / 4 and `CLAUDE.md` 1,295 / 34 | separate contents | 0 | none | `5a406e5187` |
| microsoft/semantic-kernel | 29k | none | — | 0 | none | `c5868e5e3b` |

Stars come from `gh api repos/<repo>` on 2026-09-29. Two repositories redirect: `facebook/react`
now resolves to `react/react` and `sst/opencode` to `anomalyco/opencode`. Specific facts cited below:

- The symlink targets, e.g. `CLAUDE.md -> AGENTS.md` in openai-agents-python, airflow,
  home-assistant and PostHog, come from the blobs of the git-mode `120000` entries at the listed
  commits.
- Bun's `AGENTS.md -> CLAUDE.md` and PyTorch's `AGENTS.md -> CLAUDE.md` come from the same source,
  as do Zed's `AGENTS.md`, `CLAUDE.md` and `GEMINI.md` → `.rules` (#29014, "Initial `.rules` file
  for agent with symlinks for other rules file paths").

### 1.2 Counts from the sample

- **Presence.** 37 of 46 have a root `AGENTS.md` or `CLAUDE.md`, and gemini-cli has `GEMINI.md`.
  Eight have none: anthropics/claude-code, microsoft/TypeScript, microsoft/typescript-go, golang/go,
  django, tauri, shadcn-ui and microsoft/semantic-kernel. A root `AGENTS.md` (file or symlink) exists
  in 31 and a root `CLAUDE.md` in 24.
- **Canonical file.**
  - `AGENTS.md` in 28, counting transformers' `.ai/AGENTS.md`.
  - `CLAUDE.md` in 8: the Anthropic SDKs, playwright, react, deno, electron, bun, pytorch.
  - `.rules` in 1 (Zed).
  - Every `CLAUDE.md`-canonical repository except bun and pytorch has no `AGENTS.md` at all.
- **Bridge in the 18 repositories that have both names at the root.**
  - Symlink, 8 of 18:
    - `CLAUDE.md` → `AGENTS.md`: 4
    - both → a third file: 2 (transformers, zed)
    - `AGENTS.md` → `CLAUDE.md`: 2 (bun, pytorch)
  - A `CLAUDE.md` holding a one-line `@AGENTS.md` import (11 bytes), 8 of 18: rust, ruff,
    supabase, n8n, sentry, grafana, tldraw, workers-sdk. The workers-sdk file keeps the `/init`
    header above the import.
  - Different content in each file, 2 of 18: excalidraw, and biome, whose `CLAUDE.md` is a
    symlink to `CONTRIBUTING.md`.
- **No bridge.** 13 repositories have `AGENTS.md` and no `CLAUDE.md`: codex, next.js, vercel/ai,
  vscode, cpython, uv, langchain, kubernetes, opencode, astro, svelte, node, rails. Claude Code
  reads `AGENTS.md` natively only when no `CLAUDE.md` exists, and has done so since v2.1.277
  ([cross-runtime-portability §2.2](2026-09-29-cross-runtime-portability.md)). These repositories
  rely on that, or do not target Claude Code.
- **Size.**
  - Across the 37 canonical root files, the median is 9,450 bytes and 172 lines. The largest is
    PostHog at 43,183 bytes.
  - 6 files are under 1 KB. They are pointers or policies: vscode, cpython, svelte, react,
    claude-agent-sdk-python, excalidraw.
  - 17 of 37 exceed 200 lines, the size Claude Code docs recommend staying under.
  - 2 exceed Codex's default `project_doc_max_bytes = 32768`: openai-agents-python at 38,241 and
    PostHog at 43,183. Codex truncates past that cap
    ([cross-runtime-portability §2.2](2026-09-29-cross-runtime-portability.md)). One of the two is
    OpenAI's own SDK.
- **Nesting.** Nested instruction files exist in 18 of the 37: codex, next.js, vercel/ai, vscode,
  rust (rust-analyzer), react, bun, supabase, pytorch, n8n, opencode, zed, sentry, PostHog,
  workers-sdk, electron, grafana, airflow. In practice nesting follows monorepo package boundaries,
  for example `packages/wrangler/AGENTS.md` and `apps/studio/AGENTS.md`. Where a nested `CLAUDE.md`
  exists beside a nested `AGENTS.md`, it uses the same bridge as the root.
- **Structure.** Most files have one H1 and 5–15 H2 sections. The usual sections:
  - commands (build, test, lint)
  - repository map or architecture
  - code conventions
  - testing
  - commits and pull requests
  - more and more often, a "Skills" section that indexes the skill directory (supabase, tldraw,
    biome, sentry, n8n)

  This matches the published studies in §1.6.

### 1.3 Who writes and maintains them

History of the canonical file, from `gh api repos/<repo>/commits?path=<file>`:

| Repository | File | Commits | Authors | First commit (date, author, message) | Commits with AI trailer |
|---|---|---|---|---|---|
| PostHog/posthog | `AGENTS.md` | 169 | 40 | 2025-09-23 webjunkie "Add AGENTS.md, edit CLAUDE.md" | 30 |
| openai/codex | `AGENTS.md` | 77 | 22 | 2025-05-11 bolinfest "feat: add support for AGENTS.md in Rust CLI" | 2 |
| apache/airflow | `AGENTS.md` | 69 | 19 | 2025-05-20 kaxil "Add `AGENT.md` for using OpenAI Codex" | 9 |
| oven-sh/bun | `CLAUDE.md` | 50 | 6 | 2025-06-24 Jarred-Sumner | 4 |
| n8n-io/n8n | `AGENTS.md` | 46 | 25 | 2025-11-25 "Move to AGENTS.md file with agent instructions" | 18 |
| openai/openai-agents-python | `AGENTS.md` | 45 | 5 | 2025-05-18 "Create AGENTS.md" | 1 |
| pytorch/pytorch | `CLAUDE.md` | 41 | 11 | 2025-09-04 ezyang "A basic CLAUDE.md based on bad things I see claude code doing" | 3 |
| vercel/next.js | `AGENTS.md` | 38 | 16 | 2026-01-05 "Rename CLAUDE.md to AGENTS.md with symlink" | 2 |
| anomalyco/opencode | `AGENTS.md` | 38 | 6 | 2025-07-15 thdxr | 2 |
| microsoft/vscode | `.github/copilot-instructions.md` | 38 | 15 | 2024-11-28 "Add Copilot instructions" | 14 |
| getsentry/sentry | `AGENTS.md` | 36 | 18 | 2025-10-30 "Consolidate AI rules into AGENTS.md files" | 6 |
| astral-sh/ruff | `AGENTS.md` | 35 | 9 | 2026-03-07 carljm "CLAUDE.md -> AGENTS.md" | 0 |
| langchain-ai/langchain | `AGENTS.md` | 32 | 8 | 2025-09-24 "add `AGENTS.md`" | 0 |
| home-assistant/core | `AGENTS.md` | 29 | 14 | 2025-10-05 | 7 |
| google-gemini/gemini-cli | `GEMINI.md` | 29 | 15 | 2025-05-15 "Add GEMINI.md for project conventions" | 0 |
| zed-industries/zed | `.rules` | 22 | 13 | 2025-04-17 mgsloan | 1 |
| grafana/grafana | `AGENTS.md` | 20 | 14 | 2025-10-01 | 4 |
| vercel/ai | `AGENTS.md` | 19 | 7 | 2026-01-05 lgrammel | 2 |
| cloudflare/workers-sdk | `AGENTS.md` | 17 | 9 | 2025-12-19 | 0 |
| biomejs/biome | `AGENTS.md` | 16 | 2 | 2025-11-03 | 0 |
| withastro/astro | `AGENTS.md` | 15 | 6 | 2026-02-11 FredKSchott "Experiment: Automatic triage for new GitHub issues" | 0 |
| electron/electron | `CLAUDE.md` | 13 | 5 | 2025-12-19 "add CLAUDE.md and Chromium Upgrade claude skill" | 4 |
| tldraw/tldraw | `AGENTS.md` | 13 | 5 | 2026-04-20 | 2 |
| microsoft/playwright | `CLAUDE.md` | 12 | 2 | 2026-03-10 pavelfeldman | 0 |
| rust-lang/rust | `AGENTS.md` | 9 | 2 | 2026-08-12 jyn514 "Add an AGENTS.md that enforces the LLM policy" | 0 |
| astral-sh/uv | `AGENTS.md` | 7 | 4 | 2026-04-07 charliermarsh | 0 |
| huggingface/transformers | `.ai/AGENTS.md` | 6 | 3 | 2026-03-18 "Centralize AI agent templates in `.ai`" | 1 |
| denoland/deno | `CLAUDE.md` | 5 | 4 | 2025-11-06 | 1 |
| rails/rails | `AGENTS.md` | 5 | 5 | 2025-10-23 rafaelfranca | 0 |
| kubernetes/kubernetes | `AGENTS.md` | 4 | 2 | 2026-03-24 | 0 |
| anthropics/claude-agent-sdk-python | `CLAUDE.md` | 4 | 2 | 2025-07-20 | 0 |
| anthropics/anthropic-sdk-python | `CLAUDE.md` | 3 | 2 (incl. `claude[bot]`) | 2026-09-18 | 2 |
| microsoft/vscode | `AGENTS.md` | 3 | 2 | 2025-12-01 bpasero | 1 |
| supabase/supabase | `AGENTS.md` | 2 | 2 | 2026-09-03 "make agent instructions agent-agnostic" | 0 |
| react/react | `CLAUDE.md` | 2 | 2 | 2026-01-24 rickhanlonii "[repo] init claude config" | 0 |
| sveltejs/svelte | `AGENTS.md` | 2 | 1 | 2026-04-02 Rich-Harris | 0 |
| nodejs/node | `AGENTS.md` | 2 | 2 | 2026-08-22 jasnell "meta: add a root-level AGENTS.md" | 0 |
| excalidraw/excalidraw | `AGENTS.md` / `CLAUDE.md` | 2 / 1 | 2 / 1 | 2026-08-26 / 2025-05-25 | 0 |
| python/cpython | `AGENTS.md` | 1 | 1 | 2026-09-03 "Add an `AGENTS.md` pointing to our policy" | 0 |

"AI trailer" counts commit messages matching `Co-authored-by: Claude|Codex|Copilot|Cursor`,
"Generated with [Claude" or a Claude Code or Codex URL. Squash merges can drop trailers, so the
counts are lower bounds.

Observations:

- **Authorship.** Core maintainers start the file. Examples: ezyang (PyTorch), jasnell (Node),
  Rich-Harris (Svelte), rafaelfranca (Rails), charliermarsh (uv), carljm (ruff), FredKSchott
  (Astro), Jarred-Sumner (Bun). In large projects many contributors then edit it. Agents
  co-author edits in 21 of 40 histories, most in PostHog (30) and n8n (18).
- **Generation.**
  - Only `cloudflare/workers-sdk`'s `CLAUDE.md` keeps the `/init` sentence "This file provides
    guidance to Claude Code (claude.ai/code) when working with code in this repository."
  - The Codex `/init` prompt asks for a document titled "Repository Guidelines" of "200-400
    words" and says "If it does [exist], do not overwrite or modify it"
    (`openai/codex@0b1b78a4f1:codex-rs/tui/assets/prompt_for_init_command.md`). No sampled root
    file carries that title.
  - Whatever tools produced the first drafts, the files in use have been rewritten by hand.
- **Migration toward one neutral file.**
  - next.js #88105 (2026-01-05) renamed `CLAUDE.md` to `AGENTS.md` and symlinked back "to ensure
    claude code can still work as it's currently not supporting AGENTS.md".
  - next.js #99043 (2026-09-22) removed the symlink because "Claude Code now reads `AGENTS.md`
    directly" (https://github.com/vercel/next.js/pull/88105 ,
    https://github.com/vercel/next.js/pull/99043).
  - supabase #49941 (2026-09-03) made "every `CLAUDE.md` … a one-line `@AGENTS.md` import", moved
    all skills to `.agents/skills` with `.claude/skills` as "a single symlink to it", and deleted
    `.cursor/` and `.github/instructions/` (8 files) because they duplicated the skills
    (https://github.com/supabase/supabase/pull/49941).
  - ruff (#23791 "CLAUDE.md -> AGENTS.md"), sentry ("Consolidate AI rules into AGENTS.md files")
    and n8n ("Move to AGENTS.md file") show the same move in their first commit messages above.

### 1.4 Instruction files as AI policy

Some of the most widely used language and runtime projects use the file mainly to restrict agents:

- **Node.js.** `AGENTS.md` says "New Node.js contributors should avoid using AI agents to interact
  with the project". It requires `Assisted-by: <agent name>` and a human `Signed-off-by`, and
  forbids agent `Co-authored-by`, unsupervised PRs and pushes. It lists sanctions
  (`nodejs/node@1f26576a3f:AGENTS.md`).
- **Rust.** `AGENTS.md` enforces the project's LLM usage policy with "When a gate fails …
  **STOP** that work" and a personal-use exemption that requires an explicit user statement
  (`rust-lang/rust@ea6bb45b74:AGENTS.md`).
- **CPython.** `AGENTS.md` is a 16-line pointer to the devguide AI policy
  (`python/cpython@dc0b1f8dfb:AGENTS.md`).
- **Django.** Django has no `AGENTS.md`. Its `.github/copilot-instructions.md` (`applyTo` its
  source types) tells Copilot not to review and to output a fixed refusal
  (`django/django@9332b163a6:.github/copilot-instructions.md`).
- **Others with an AI policy or disclosure section:** home-assistant ("AI policy"), kubernetes,
  airflow, PostHog, openai-agents-python, biome, playwright and pytorch, from a text search of the
  fetched files.

### 1.5 Neighbouring agent configuration in the same repositories

- **Hooks.** `.claude/hooks` exists in ruff (session start), bun (`pre-bash-guard.js`,
  `post-edit-format.js`), pytorch (PR-review write restriction and validators, a time budget) and
  PostHog (environment setup).
- **Subagents.** `.claude/agents` exists in PostHog (11, for example `code-reviewer.md`,
  `test-writer.md`), home-assistant (`raise-pull-request.md`) and claude-agent-sdk-python.
- **Committed `.claude/settings.json`.** Present in about a dozen repositories, for example ruff,
  bun, supabase, sentry, PostHog, electron, biome, n8n, tldraw, vercel/ai and react.
- **`.agents/` beyond skills.**
  - n8n keeps `.agents/review-rules/` with backend, db-migrations, frontend and qa-dx rules.
  - openai-agents-python keeps `.agents/references/` with lifecycle notes.
  - PostHog keeps `.agents/owners.yaml` and `.agents/security.md`.
  - next.js and astro keep `.agents/evals/` (`skills.eval.ts`).
  - transformers keeps `.ai/review-rules.md`.
- **Enforcement.** PostHog adds a path-scoped Claude rule, `.claude/rules/agents-md.md`, that
  makes agents invoke an `editing-agents-md` skill before changing any `AGENTS.md` or `CLAUDE.md`.
  That skill sets a ladder: linter, then pre-commit, then skill, then an `AGENTS.md` rule "as a
  last resort". It also carries a size budget ("under 200 lines") and the six measured smells
  (`PostHog/posthog@99a0f7a090:.claude/rules/agents-md.md`,
  `PostHog/posthog@99a0f7a090:.agents/skills/editing-agents-md/SKILL.md`).

### 1.6 What the published studies measured

- **2,303 context files from 1,925 repositories.** The files "evolve like configuration code
  through frequent, small additions". The most common content is test procedures (75.9%),
  implementation details (70.8%) and architecture (68.1%). Security (14.8%) and performance (14.5%)
  are rare. "Agent READMEs: An Empirical Study of Context Files for Agentic Coding",
  https://arxiv.org/abs/2511.12884
- **253 `CLAUDE.md` files from 242 repositories.** "Shallow hierarchies with one main heading and
  several subsections", "dominated by operational commands, technical implementation notes, and
  high-level architecture". https://arxiv.org/abs/2509.14744
- **Six configuration smells across 100 popular repositories.** Lint Leakage 62%, Context Bloat
  42%, Skill Leakage 35%, and more. https://arxiv.org/abs/2606.15828. The other three names and
  percentages (Conflicting Instructions 28%, Init Fossilization 24%, Blind References 16%) are
  quoted from PostHog's skill, not verified in the paper body.
- **Effect on task success.** Context files, LLM-generated and developer-written alike, "do not
  generally improve task success rates, while increasing inference cost by over 20%". "Repository
  overviews … are not helpful", but instructions in the files "are well followed".
  https://arxiv.org/abs/2602.11988
- **Effect on runtime.** With `AGENTS.md`, median runtime was 28.64% lower and output tokens
  16.58% lower on 124 PRs. https://arxiv.org/abs/2601.20404
- **File structure.** Size, position, architecture and contradictions in adjacent files had "no
  detectable contrast" on adherence across 1,650 Claude Code sessions.
  https://arxiv.org/abs/2605.10039

---

## 2. Skills in repositories

### 2.1 Where skills live

From the sample (§1.1), 32 of 46 commit skills:

| Canonical location | Repositories |
|---|---|
| `.agents/skills` (12) | openai-agents-python, next.js, vscode (1 skill), ruff, supabase, n8n, zed, sentry, PostHog, astro, svelte, airflow |
| `.claude/skills` (10) | claude-agent-sdk-python, playwright, react, deno, bun, pytorch, electron, home-assistant, biome, grafana |
| `.github/skills` (3) | vscode (47), microsoft/TypeScript, microsoft/typescript-go |
| neutral dir (3) | vercel/ai `skills/`, tldraw `skills/`, transformers `.ai/skills` |
| `.codex/skills` (2) | openai/codex, astral-sh/uv |
| `.opencode/skills` (2) | opencode, cloudflare/workers-sdk |

GitHub-wide code search on 2026-09-29 gave approximate `total_count` values. They count files,
not repositories:

| Query | Count |
|---|---|
| `SKILL.md` under `.claude/skills` | ~421k |
| `SKILL.md` under `.agents/skills` | ~358k |
| `SKILL.md` under `.github/skills` | ~97k |
| `SKILL.md` under `.cursor/skills` | ~53k |
| `SKILL.md` under `.opencode/skills` | ~46k |
| `SKILL.md` under `.codex/skills` | ~40k |
| `filename:AGENTS.md` | ~1.12M |
| `filename:CLAUDE.md` | ~791k |
| `filename:copilot-instructions.md` | ~161k |
| `filename:GEMINI.md` | ~67k |
| `CLAUDE.md` containing `@AGENTS.md` | ~49k |
| `filename:skills-lock.json` | ~48k |
| `.claude-plugin/plugin.json` | ~26k |

Symlinked copies are probably not indexed (unconfirmed), so these counts mostly reflect canonical
locations.

In popular repositories the skills are task procedures for contributors, not general methodology.
Examples:

- **Pull requests and releases:** `create-pr`, `backport-pr`, `pr-status-triage` (next.js);
  `babysit-pr`, `codex-pr-body` (codex); `release-candidate-prep` (openai-agents-python).
- **Migrations:** `generate-migration`, `django-models` (sentry); `clickhouse-migrations` (PostHog).
- **Testing:** `writing-bundler-tests` (bun); `studio-e2e-tests` (supabase).
- **Upgrades:** `electron-chromium-upgrade` (electron); `update-v8-version` (codex).
- **Triage:** `issue-triage` (deno); `triaging-issues` (pytorch).
- **Review:** `code-review` plus four `code-review-*` skills (codex); `ha-review` (home-assistant);
  `human-like-code-review` (n8n).

Planning or spec methodology skills are rare: n8n's `spec-driven-development` and openai-agents-python's
`implementation-strategy`, `implementation-kickoff` and `implementation-final-review` are the
exceptions seen.

### 2.2 One set, several runtimes

Claude Code does not read `.agents/skills`
([cross-runtime-portability §3.2](2026-09-29-cross-runtime-portability.md)). So every repository
that keeps skills in `.agents/skills` and cares about Claude Code adds a link. Four patterns were
observed:

1. **Directory symlink** `.claude/skills` → `../.agents/skills`: next.js, sentry, PostHog, supabase.
   A variant keeps the canonical copy in a neutral top-level directory and symlinks every runtime to
   it:
   - vercel/ai links `.agents/skills` and `.claude/skills` → `../skills`.
   - tldraw also links `.cursor/skills`. Its `AGENTS.md` says "Keep `skills/` as the source of
     truth … Do not duplicate skill content for different agents; add compatibility pointers or
     symlinks instead" (`tldraw/tldraw@da7162e13b:AGENTS.md`).
   - home-assistant goes the other way, with `.gemini/skills` → `../.claude/skills`.
2. **Per-skill symlinks.** Airflow links selected skills, for example
   `.claude/skills/aip-user-stories` → `../../.agents/skills/aip-user-stories` (6 in `.claude/skills`,
   4 in `.github/skills`). This lets each runtime expose a different subset.
3. **A Claude Code plugin as the wrapper.**
   - **n8n** keeps shared skills in `.agents/skills`, and Claude Code consumes them "through
     symlinks in `.claude/plugins/n8n/skills/`; OpenCode reads `.agents/skills/` directly". The
     plugin exists "to get the `n8n:` namespace prefix … standalone `.claude/skills/` entries cannot
     be namespaced". `pnpm sync:skill-links` and `check:skill-links` maintain the links, and a
     committed settings file registers the plugin directory as a marketplace
     (`n8n-io/n8n@55fd9daa08:AGENTS.md`, `.claude/plugins/n8n/README.md`, `.claude/settings.json`).
   - **ruff** turns `.agents/` itself into a plugin (`.agents/.claude-plugin/plugin.json`,
     `marketplace.json`) and enables it from `.claude/settings.json` through
     `extraKnownMarketplaces: {source: "directory", path: ".agents"}` plus
     `enabledPlugins: {"ty-skills@ruff-agent-skills": true}` (`astral-sh/ruff@0d7e203568:.claude/settings.json`).
4. **Links created on each machine.** transformers commits `.ai/skills` and `.ai/AGENTS.md`. Its
   `Makefile` targets run `ln -snf ../.ai/skills .agents/skills` and
   `ln -snf ../.ai/skills .claude/skills`, and `clean-ai` removes them
   (`huggingface/transformers@da4bd0dca1:Makefile`). This is the only sampled case where a
   per-machine step is needed, and it creates only links, not content.

Cost of symlinks: n8n warns that on Windows, without `core.symlinks` and Developer Mode or WSL,
"git writes them as plain text stubs and Claude Code fails to load the affected skills", and ships
a check script for that (`n8n-io/n8n@55fd9daa08:.claude/plugins/n8n/README.md`).

Copying the same skill into several runtime directories was not observed in the sample. The
only repositories with two real skill directories hold different skills in each: vscode's
`.github/skills` against `.agents/skills`, and n8n's Claude-only overrides.

### 2.3 Committed against installed per machine

- **In-repo skills are committed.** All 32 repositories commit their own skills, and none installs
  them per machine.
- **Third-party skills are committed too, with a lock.** next.js has `skills-lock.json` for one
  third-party skill, `gh-stack` from `github/gh-stack`, with `computedHash`. The skill's files are
  also committed under `.agents/skills/gh-stack/` (`vercel/next.js@b829c6189b:skills-lock.json`).
- **Plugins install per machine from committed settings.** Enabling a plugin from committed
  `.claude/settings.json` is a hybrid: the pointer is committed and the content installs per
  machine after workspace trust. Svelte's `sv add ai-tools` documents exactly that for its official
  plugin: "enabled through a committed `.claude/settings.json` … then it installs automatically"
  (`sveltejs/cli@5af9b97115:documentation/docs/30-add-ons/01-ai-tools.md`).

### 2.4 What `vercel-labs/skills` writes into a project

Read at `vercel-labs/skills@3694740352` (32,757 stars, MIT):

- **Canonical copy.** In project scope (the default) each skill is copied to `.agents/skills/<name>/`
  (`src/constants.ts`: `UNIVERSAL_SKILLS_DIR = '.agents/skills'`; `src/installer.ts`, "Canonical
  location: .agents/skills/<skill-name>").
- **Agent links.** For each selected agent whose project path is not `.agents/skills` (for Claude
  Code, `.claude/skills/`), the installer creates a **relative** symlink to the canonical copy. On
  Windows it creates a junction instead (`src/installer.ts` `createSymlink`). 23 listed agents (14 rows
  of the table) already use `.agents/skills` as their project path (Codex, Cursor, Gemini CLI, GitHub Copilot,
  OpenCode, Amp, Zed and others), so they need no link. Claude Code, AiderDesk, Augment and some
  others need one (README "Supported agents" table).
- **Copy mode.** `--copy` writes independent copies instead ("Use when symlinks aren't supported").
- **Scope.** Project scope is described as "Committed with your project, shared with team", and
  global scope (`-g`) as "Available across all projects".
- **Project lock.** `skills-lock.json` in the project root holds per skill `source`, `sourceUrl`,
  `ref`, `sourceType`, `skillPath` and a SHA-256 `computedHash` of the files on disk. It is
  "intentionally minimal and timestamp-free to minimize merge conflicts", is "meant to be checked
  into version control", and is sorted alphabetically (`src/local-lock.ts`).
- **Global lock.** `~/.agents/.skill-lock.json`, or `$XDG_STATE_HOME/skills/.skill-lock.json`
  (`src/skill-lock.ts`).
- **Restore and sync.** `npx skills experimental_install` restores from `skills-lock.json` into
  `.agents/skills/` only. `experimental_sync` syncs skills shipped in `node_modules` into agent
  directories (`src/install.ts`, `src/cli.ts`).
- **Telemetry** is on by default, with `DISABLE_TELEMETRY` or `DO_NOT_TRACK` to opt out (README).

### 2.5 Frameworks that write agent files into consumer projects

Libraries ship agent configuration into their users' projects. How they do it is a precedent for a
harness:

- **Next.js.**
  - `create-next-app` creates the agent files. When `next dev` detects a coding agent, it
    "creates or updates the managed instructions" in `AGENTS.md` between
    `<!-- BEGIN:nextjs-agent-rules -->` and `<!-- END:nextjs-agent-rules -->`.
  - "Next.js updates only the content between the agent rules markers and preserves the rest of
    `AGENTS.md`." The `agentRules` config option can disable it
    (`vercel/next.js@b829c6189b:docs/01-app/02-guides/ai-agents.mdx`).
  - Since #99043 it no longer creates or updates `CLAUDE.md`.
- **Nx.** `nx configure-ai-agents` writes `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.mcp.json`,
  `opencode.json` and `.codex/config.toml` for the selected agents. It manages its text between
  `<!-- nx configuration start-->` and `<!-- nx configuration end-->`, with "Leave the start & end
  comments to automatically receive updates" (`nrwl/nx@a11ea49fd3:packages/nx/src/ai/constants.ts`).
- **Svelte CLI.** `sv add ai-tools` offers the official plugin, or individual tools per client: an
  MCP config, an `AGENTS.md`-style README for agents, skills and subagents
  (`sveltejs/cli@5af9b97115:documentation/docs/30-add-ons/01-ai-tools.md`).
- **Codex `/init`** only creates `AGENTS.md` when none exists (§1.3).

---

## 3. Harness and pipeline components beyond the eight compared

Nineteen harnesses were counted: the eight from
[harness-frameworks-compared](2026-09-29-harness-frameworks-compared.md), plus eleven read for this
note. Stars are from `gh api` on 2026-09-29, and links are pinned to that day's HEAD.

### 3.1 The eleven

- **Google Conductor** — `gemini-cli-extensions/conductor` (3,750★).
  - A skills-based plugin for Antigravity and Claude Code. It has no CLI of its own and uses a
    Context → Spec & Plan → Implement loop.
  - Context docs: `conductor/product.md`, `tech-stack.md`, `workflow.md`, `code_styleguides/`,
    and `index.md` as "Single Source of Truth for all tools". It does not edit `AGENTS.md`.
  - Tracks hold `spec.md` and a `plan.md` that doubles as the task list, with commit SHAs appended
    to finished tasks.
  - Review is a skill. One commit per task, `git notes` summaries, and revert by track, phase or
    task.
  - Approve/Revise loops for every doc, and manual verification per phase.
  - No hooks, MCP server, tracker or evals.
  - Sources:
    - https://github.com/gemini-cli-extensions/conductor/blob/6e8f9a860bcdd6a2c423473c12e745200688c633/README.md
    - https://github.com/gemini-cli-extensions/conductor/blob/6e8f9a860bcdd6a2c423473c12e745200688c633/skills/conductor-setup/SKILL.md
- **spec-kitty** — `spec-kitty/spec-kitty` (1,650★, `main` on a 4.0.0rc line).
  - A Python CLI with the loop spec → plan → tasks → next → review → accept → merge.
  - A "charter" (constitution) plus doctrine packs.
  - Artifacts live in `kitty-specs/`, and work packages come from `wps.yaml`.
  - Lanes (planned → in_progress → for_review → approved → done), one git worktree per work
    package under `.worktrees/`, and a local kanban dashboard.
  - 17 agent surfaces. Codex and some others get `.agents/skills`.
  - `upgrade` with a manifest, retrospectives, a pre-commit guard and a Claude Code session hook.
  - "Auto-merge" means merging multiple dependency branches into a temporary base before a
    worktree is created. It is not automatic PR merging.
  - The tracker connector design names Jira, Linear and GitHub, but whether it works in the
    local-only product is unconfirmed.
  - Sources:
    - https://github.com/spec-kitty/spec-kitty/blob/5b3bd9cf7b3feb16729c12c9ee8bc6af8fd39131/README.md
    - https://github.com/spec-kitty/spec-kitty/blob/5b3bd9cf7b3feb16729c12c9ee8bc6af8fd39131/docs/api/supported-agents.md
    - https://github.com/spec-kitty/spec-kitty/blob/5b3bd9cf7b3feb16729c12c9ee8bc6af8fd39131/docs/adr/1.x/2026-01-23-4-auto-merge-multi-parent-dependencies.md
- **spec-workflow-mcp** — `Pimzino/spec-workflow-mcp` (4,297★).
  - An MCP server (tools and prompts) with steering docs and a Requirements → Design → Tasks flow
    under `.spec-workflow/`.
  - Dashboard approvals, implementation logs, a `tasks.md` validator, a web kanban dashboard and a
    VS Code extension.
  - Worktree support is unconfirmed (only test names suggest it).
  - Source: https://github.com/Pimzino/spec-workflow-mcp/blob/d38e82eaa8a6a2f5480285fa1feb8cf8378c86e0/README.md
- **Anthropic, product practices** — `anthropics/claude-code` (148,562★), `anthropics/skills`
  (178,952★), `anthropics/claude-code-action` (9,233★), and the best-practices docs
  (https://code.claude.com/docs/en/best-practices).
  - Practices: `CLAUDE.md` via `/init` kept short; skills, subagents, hooks and plugins;
    Explore → Plan → Implement → Commit; "give Claude a way to verify its work"; adversarial review
    by a subagent; worktrees; `claude -p` for CI.
  - Bundled plugins:
    - `feature-dev`: 7 phases, explorer, architect and reviewer agents, and user approval before
      implementation.
    - `code-review`: parallel review agents plus validation subagents.
    - Also `pr-review-toolkit`, `ralph-wiggum` (a Stop-hook loop) and `hookify`.
  - `skill-creator` ships an eval runner and grader.
  - Sources:
    - https://github.com/anthropics/claude-code/blob/dec92bc87ab6fe9c7be0fcba1f97966f902dd243/plugins/feature-dev/commands/feature-dev.md
    - https://github.com/anthropics/skills/tree/8a1541c4a3ffa5a20a5a91de0dcf3f0bab1d1ef4/skills/skill-creator/scripts
- **Anthropic, long-running harness posts.**
  - "Effective harnesses for long-running agents" (2025-11-26):
    - An initializer agent, then coding sessions.
    - `feature_list.json` with `passes`, because "the model is less likely to inappropriately
      change or overwrite JSON files compared to Markdown files".
    - `claude-progress.txt`, `init.sh` and a commit per feature.
    - A fixed start-of-session routine and end-to-end verification.
  - "Harness design for long-running application development" (2026-03-24):
    - Planner, generator and evaluator roles, with sprint contracts and file-based handoffs.
    - Every component "encodes an assumption about what the model can't do on its own".
  - Sources: https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents ,
    https://www.anthropic.com/engineering/harness-design-long-running-apps
- **OpenAI, "Harness engineering"** (2026-02-11), https://openai.com/index/harness-engineering/.
  The site returned 403 to scripted fetches, so it was read in a browser.
  - `AGENTS.md` as "the table of contents" rather than "the encyclopedia", about 100 lines.
  - `docs/` as the system of record: design docs, `core-beliefs.md`, execution plans (active and
    completed), a tech-debt tracker and quality scores.
  - Execution plans with required `Progress`, `Surprises & Discoveries`, `Decision Log` and
    `Outcomes & Retrospective` sections (https://developers.openai.com/cookbook/articles/codex_exec_plans).
  - Custom linters and structural tests whose messages carry fix instructions for the agent.
  - A recurring "doc-gardening" agent and background cleanup tasks.
  - Agent-to-agent review with minimal blocking merge gates.
  - The app booted per worktree with its own observability stack.
  - OpenAI's own repository keeps `.codex/skills` with a `code-review` family (§1.1). `openai/skills`
    is marked deprecated in favour of `openai/plugins`
    (https://github.com/openai/skills/blob/49f948faa9258a0c61caceaf225e179651397431/README.md).
- **compound-engineering** — `EveryInc/compound-engineering-plugin` (25,323★).
  - The loop is brainstorm → plan → work → simplify → code-review → compound. "`/ce-compound`
    writes learnings that the next `/ce-brainstorm` and `/ce-plan` read as grounding".
  - Artifacts live in `docs/brainstorms/`, `docs/plans/` and `docs/solutions/`.
  - 37 skills and 17 reviewer personas.
  - `/lfg` runs the loop through PR and CI watching, and does not merge unless granted.
  - Worktree, handoff and PR-babysitting skills, and `.compound-engineering/config.yaml`.
  - Plugin manifests for 7 hosts plus converters, and per-host skill evals.
  - Source: https://github.com/EveryInc/compound-engineering-plugin/blob/b27637b062d168fd877bd6b54a1bdfc959762421/README.md
- **HumanLayer** — `humanlayer/humanlayer` (11,622★; the README says the code is "pretty much all
  deprecated").
  - 27 `.claude/commands`: research, plan, implement and validate; handoffs; worktrees; Linear;
    "ralph" autonomous runs.
  - 6 locator and analyzer subagents.
  - Plans in `thoughts/shared/plans/` with "Automated Verification" and "Manual Verification"
    criteria per phase and a pause for human confirmation.
  - A `thoughts/` directory synced to a separate repository by git hooks.
  - Source: https://github.com/humanlayer/humanlayer/blob/99abe673498cf8bdcd5f989aebe9406a27185b3b/.claude/commands/create_plan.md
- **Task Master** — `eyaltoledano/claude-task-master` (28,111★).
  - A CLI (`parse-prd`, `next`, `expand`) and an MCP server.
  - `.taskmaster/tasks.json` and a PRD.
  - Rule profiles for 15 editors, Kiro hooks, and a VS Code kanban.
  - Source: https://github.com/eyaltoledano/claude-task-master/blob/c0c98d367c55296bfe69e65680625b6db437af02/README.md
- **AI-DLC** — `awslabs/aidlc-workflows` (4,903★).
  - An installer plus an `aidlc` CLI, with 5 phases and 33 stages.
  - 14 agents and 11 workflow profiles.
  - 19 hook scripts, including a state-transition guard and a plan-approval guard.
  - "Sensor" validators and human approval gates.
  - A 107-event audit trail and learned rules.
  - Per-harness emitters for Claude, Codex, Copilot, Cursor, Kiro and opencode.
  - The state file name is unconfirmed.
  - Source: https://github.com/awslabs/aidlc-workflows/blob/0c2ba3af2b08901244cf5202b4e17039b144c135/README.md
- **Cline Memory Bank.** Markdown only: `projectbrief.md`, `productContext.md`, `activeContext.md`,
  `systemPatterns.md`, `techContext.md`, `progress.md`, loaded through custom instructions.
  Source: https://docs.cline.bot/prompting/cline-memory-bank

Excluded: `humanlayer/12-factor-agents` (principles for building LLM software, not a coding
harness), `github/awesome-copilot` (a catalog) and Tessl (not read).

### 3.2 Frequency

A component is counted only when the harness ships or prescribes it, as confirmed from the sources
above or from the baseline note. Partial matches are not counted. Cells the baseline note does not
state were left out, so baseline harnesses may be undercounted. "Own MCP server" means the harness
ships a server; using another server does not count.

Abbreviations: SK spec-kit, OS OpenSpec, BM BMAD, SP Superpowers, KI Kiro, GSD, RF Ruflo, AOS Agent
OS, CO Conductor, SKY spec-kitty, SWM spec-workflow-mcp, ANT-P Anthropic product practices, ANT-LR
Anthropic long-running posts, OAI OpenAI harness engineering, CE compound-engineering, HL
HumanLayer, TM Task Master, AID AI-DLC, CMB Cline Memory Bank.

| Component | Count (of 19) | Where |
|---|---|---|
| Spec artifact | 10 | OS, KI, CO, SKY, SWM, ANT-LR, OAI, CE, TM, AID |
| Installer or CLI | 10 | SK, OS, BM, GSD, RF, SKY, CE, HL, TM, AID |
| Loop runner or workflow engine | 10 | SK, GSD, RF, SKY, ANT-P, ANT-LR, OAI, CE, HL, AID |
| Plan artifact | 9 | KI, CO, SKY, SWM, ANT-P, OAI, CE, HL, AID |
| Slash commands | 9 | SK, OS, AOS, CO, SKY, ANT-P, CE, HL, AID |
| State or progress file | 9 | GSD, CO, SKY, SWM, ANT-LR, OAI, TM, AID, CMB |
| Persistent context or steering docs | 8 | KI, AOS, CO, SKY, SWM, OAI, AID, CMB |
| Role agents or subagents | 8 | BM, RF, ANT-P, ANT-LR, OAI, CE, HL, AID |
| Skills | 8 | SK, OS, SP, CO, SKY, ANT-P, OAI, CE |
| Runtime hooks | 8 | SK, SP, KI, GSD, SKY, ANT-P, TM, AID |
| Automated review step | 8 | CO, SKY, ANT-P, ANT-LR, OAI, CE, HL, AID |
| Task list or tickets | 7 | KI, CO, SKY, SWM, ANT-LR, HL, TM |
| Approval gates | 7 | SK, CO, SKY, SWM, ANT-P, HL, AID |
| Git discipline (commit per task, PR, revert) | 7 | CO, SKY, ANT-P, ANT-LR, OAI, CE, HL |
| Validators, linters, structural tests | 6 | OS, GSD, SKY, SWM, OAI, AID |
| Multi-runtime rendering | 6 | SK, OS, SKY, CE, TM, AID |
| Memory or learning capture | 6 | RF, SKY, OAI, CE, AID, CMB |
| Instruction file generation or editing | 5 | ANT-P, OAI, TM, AID, CMB |
| Worktree or parallel isolation | 5 | SKY, ANT-P, OAI, CE, HL |
| Host plugin packaging | 5 | SP, CO, SWM, ANT-P, CE |
| Own extension or pack registry | 5 | SK, BM, KI, RF, SKY |
| Tracker integration | 4 | BM, SKY (design; unconfirmed in practice), ANT-P (GitHub Action), HL (Linear) |
| Dashboard or UI | 4 | SKY, SWM, HL, TM |
| Update mechanism (manifest, upgrade) | 4 | SK, GSD, SKY, AID |
| Constitution or principles | 3 | SK, SKY, OAI |
| Own MCP server | 3 | RF, SWM, TM |
| Evals of the harness itself | 3 | SP, ANT-P, CE |
| CI integration | 3 | ANT-P, OAI, CE |
| Research step or artifact | 3 | SKY, HL, TM |
| Session handoff docs | 3 | ANT-LR, CE, HL |
| Runtime or browser self-verification | 3 | ANT-LR, OAI, CE |
| Audit trail | 2 | SKY, AID |
| Environment bootstrap per session or worktree | 2 | ANT-LR, OAI |
| Recurring doc-gardening or cleanup agents | 1 | OAI |
| Agent-queryable observability stack | 1 | OAI |

**Recurring (in about half or more).**

- A spec → plan → task-list chain.
- An installer or CLI.
- A loop runner.
- Slash commands.
- A state or progress file.
- Persistent context docs.
- Subagents.
- Skills.
- Hooks.
- An automated review step.

**Rare (3 or fewer).**

- A constitution.
- The harness's own MCP server.
- Evals of the harness itself.
- CI integration.
- A research artifact.
- Handoff docs.
- Browser self-verification.
- Audit trails.
- Per-worktree environments.
- Doc-gardening agents.

### 3.3 Harnesses against real repositories

The popular repositories in §1–2 hold a different subset of these components than the harnesses:

- **What they have:** an instruction file (37/46) and committed task skills (32/46).
- **What some have:** committed settings, hooks (4), subagents (3), review rules (n8n,
  transformers) and skill evals (next.js, astro).
- **What none has:** a spec → plan → tasks pipeline in the repository. The closest are n8n's
  `spec-driven-development` skill and sentry's `.claude/plans/` placeholder.
- **Where the review step lives:** in skills (codex, home-assistant, n8n, deno).

---

## 4. Implications for agent-workbench (input for E18 #23)

- **Neutral source against native files.** Real repositories already converge on "neutral source
  plus links": `AGENTS.md` plus `.agents/skills`, with `.claude/skills` as a symlink or plugin
  wrapper. This is what `vercel-labs/skills` automates. A harness that follows the same layout
  matches what users already see in next.js, sentry, supabase and PostHog.
- **Links and Windows.** Pick one link strategy per repository:
  - A directory symlink is the simplest.
  - Per-skill links allow a different subset per runtime.
  - A plugin wrapper gives Claude namespacing.

  Windows needs `core.symlinks` or copy mode. n8n and `vercel-labs/skills` both deal with this.
- **Ownership of `AGENTS.md`.** The file belongs to the project. The harness should:
  - Create a skeleton only when the file is absent.
  - Own one marked block, as Next.js and Nx do.
  - Keep the `CLAUDE.md` bridge to one line, or none where the runtime reads `AGENTS.md` natively.
  - Warn at Codex's 32 KiB cap and Claude Code's 200-line guidance.
- **What the pipeline keeps and adds.** The harness pipeline itself (spec → plan → tasks, review,
  gates) is what popular repositories lack. It is the harness's value, and it should arrive as
  skills that sit beside the repository's own task skills without renaming them. Rare components
  seen in the strongest setups are candidates rather than defaults: harness evals, audit trails,
  doc-gardening.
