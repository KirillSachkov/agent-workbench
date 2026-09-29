# Harness and pipeline frameworks compared as products

Date: 2026-09-29. Method: shallow clones of each repository at the commits listed below, the
repositories' own docs and source, official documentation sites where the product is closed
(Kiro), and the GitHub REST API (`gh api repos/<repo>`, `/releases`, `/commits?since=`) for
maturity numbers on 2026-09-29. Every claim carries a source link; anything that could not be
confirmed is marked **"unconfirmed"**.

Scope. The earlier note [artifact-pipelines-industry](2026-09-25-artifact-pipelines-industry.md)
covers the *artifacts* of spec-kit, Kiro, BMAD and Tessl (fields, lifecycle, what code checks). This
note does not repeat them. It looks at the frameworks **as products someone installs, updates,
configures and runs across several agent runtimes**: installation and update, multi-runtime
rendering, configurability, mechanical enforcement, tracker integration and maturity.

Frameworks and the commits read:

| Framework | Repository | Commit / tag read |
|---|---|---|
| spec-kit | `github/spec-kit` | `8d3f64c` (2026-09-28), release v1.0.12 |
| OpenSpec | `Fission-AI/OpenSpec` | `d4e1c77` (2026-09-28), tag `@fission-ai/openspec@1.13.2` |
| BMAD Method | `bmad-code-org/BMAD-METHOD` | `main` `1cbcfa2` (2026-09-28, unreleased 6.13-next); branch `V6.12` `9ee3dfd` for the npm installer |
| Superpowers | `obra/superpowers` | `8ca22db` (2026-09-25), tag v6.4.2 |
| Kiro | `kirodotdev/Kiro` (issue tracker only) + https://kiro.dev/docs/ | `bfe7ff3` (2026-08-27) |
| GSD (Get Shit Done) | `open-gsd/gsd-core` (successor of archived `gsd-build/get-shit-done`) | `d8533f6` (2026-09-28), tag v1.15.0 |
| Ruflo (formerly claude-flow) | `ruvnet/ruflo` | `fce8e6d` (2026-09-28), tag v3.48.0 |
| Agent OS | `buildermethods/agent-os` | `475b0ca` (2026-08-29), tag v3.0.0 |

Citation shorthand: a path such as `SK:docs/upgrade.md` means
`https://github.com/<repo>/blob/<commit>/<path>` with the repo and commit from the section header.

Selection of the "other" frameworks. `gh search repos "spec-driven development" --sort stars`
(2026-09-29) and the names suggested in the task. GSD was picked as the largest pure harness
(64k stars on the archived original, active successor); Ruflo as the largest "agent harness"
positioned as orchestration rather than method; Agent OS as the one with an explicit profile
system. Also seen but not analysed in depth: Google's Conductor, spec-kitty, Pimzino
spec-workflow-mcp (section 10).

---

## 0. Maturity at a glance (gh api, 2026-09-29)

| Repo | Stars | Forks | License | Created | Latest release | Releases since 2026-06-29 | Commits since 2026-08-30 | Open issues + PRs |
|---|---|---|---|---|---|---|---|---|
| github/spec-kit | 139,321 | 12,481 | MIT | 2025-08-21 | v1.0.12, 2026-09-25 | 52 | 215 | 268 |
| Fission-AI/OpenSpec | 70,622 | 4,844 | MIT | 2025-08-05 | 1.13.2, 2026-09-23 | 11 | 107 | 224 |
| bmad-code-org/BMAD-METHOD | 53,608 | 6,036 | MIT + trademark notice (GitHub reports NOASSERTION) | 2025-04-13 | v6.12.0, 2026-09-04 | 3 | 183 | 43 |
| obra/superpowers | 292,620 | 26,196 | MIT | 2025-10-09 | v6.4.2, 2026-09-25 | 6 | 2 (releases land as squashed commits) | 278 |
| kirodotdev/Kiro | 4,336 | 322 | proprietary product; tracker repo has no license | 2025-06-17 | no GitHub releases; see changelog | n/a | 0 | 1,345 |
| open-gsd/gsd-core | 9,969 | 714 | MIT | 2026-05-22 | v1.15.0, 2026-09-26 | 17 | 487 | 185 |
| gsd-build/get-shit-done (archived) | 64,438 | 5,441 | MIT | — | archived 2026-05-31 | — | — | — |
| ruvnet/ruflo | 73,465 | 8,725 | MIT | 2025-06-02 | v3.48.0, 2026-09-28 | ≥100 (all 100 fetched fall after 2026-07-04) | 371 | 1,015 |
| buildermethods/agent-os | 5,456 | 832 | MIT | 2025-07-16 | v3.0.0, 2026-01-20 | 0 | 0 | 2 |

Source for every row: `https://api.github.com/repos/<repo>` plus `/releases?per_page=100` and
`/commits?since=2026-08-30T00:00:00Z` (paginated). The BMAD license text is MIT with an appended
"TRADEMARK NOTICE" (https://github.com/bmad-code-org/BMAD-METHOD/blob/1cbcfa2/LICENSE).

---

## 1. GitHub spec-kit

Base: `SK:` = https://github.com/github/spec-kit/blob/8d3f64c/

### 1.1 Installation and update

- CLI `specify-cli`, Python ≥ 3.11 with uv or pipx; Git only for the git extension
  (`SK:pyproject.toml`, `SK:docs/installation.md`).
- Recommended install is pinned to a tag:
  `uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@vX.Y.Z`; PyPI and
  one-shot `uvx` also work, and there is an air-gapped guide (`SK:docs/installation.md`).
- Project setup: `specify init <name> --integration <key>`, with `--script sh|ps|py`,
  `--non-interactive`, `--ignore-agent-tools` for CI (`SK:docs/installation.md`).
- In the project: `.specify/` with `memory/constitution.md`, `scripts/`, `templates/`,
  `init-options.json`, `integration.json`, `integrations/<key>.manifest.json`, `extensions.yml`,
  `extensions/<id>/`, `presets/<id>/`, `workflows/<id>/`, `workflows/runs/<run_id>/` and a hook
  dispatcher `events.py` (`SK:src/specify_cli/_init_options.py`,
  `SK:src/specify_cli/integrations/manifest.py`, `SK:src/specify_cli/events/__init__.py`); plus the
  generated commands or skills in the agent directory, e.g. `.claude/skills/speckit-*`
  (`SK:docs/reference/integrations.md`).
- On the machine: the uv/pipx tool, optional `~/.specify/auth.json` and user-level catalog files
  `~/.specify/*-catalogs.yml` (`SK:docs/reference/authentication.md`,
  `SK:docs/reference/integrations.md`).
- Upgrade: `specify self check` / `specify self upgrade [--tag vX.Y.Z] [--dry-run]`; it detects the
  install method and upgrades only uv-tool and pipx installs. Project files:
  `specify integration upgrade <key>` then `specify extension update` (`SK:docs/upgrade.md`).
- **Local edits survive through a hash manifest.** Every installed file is recorded with SHA-256
  (`SK:src/specify_cli/integrations/manifest.py#L129-L162`); upgrade stops on a modified managed
  file unless `--force`; shared scripts and templates are refreshed only if they still match the
  last managed copy; uninstall keeps modified files; `specs/` and the constitution are never
  touched (`SK:docs/upgrade.md`, `SK:docs/reference/integrations.md`).
- Exception: `specify preset update` removes and re-adds the preset, with no staging or rollback
  (`SK:docs/reference/presets.md`).

### 1.2 Multi-runtime

- About 43 integrations: 42 in the built-in catalog plus `generic` with `--commands-dir`; the
  community integration catalog is empty (`SK:integrations/catalog.json`,
  `SK:integrations/catalog.community.json`).
- Registry: one Python package per agent under `src/specify_cli/integrations/<key>/`; rendering by
  base classes `MarkdownIntegration`, `TomlIntegration` (Gemini, Tabnine), `YamlIntegration` (Goose
  recipes) and `SkillsIntegration` (`speckit-<name>/SKILL.md`); a typical subclass sets three class
  attributes (`SK:src/specify_cli/integrations/base.py`).
- What differs per runtime: target directory, file format, invocation (`/speckit-x`, `$speckit-x`
  for Codex, `/skill:speckit-x` for Kimi), skills vs commands mode, and quirks such as a prose
  fallback for Kiro CLI, which does not substitute `$ARGUMENTS` (`SK:docs/reference/integrations.md`).
- Several integrations can be installed at once with one default, each declaring
  `multi_install_safe`; `specify integration use <key>` switches the default. **Extensions and
  presets are materialised only for the default integration**, so a multi-agent repository is not
  symmetric (`SK:docs/reference/integrations.md` "Install", "Use").
- **Agent-native hooks from canonical events.** `session_start`, `pre_tool_use`, `post_tool_use`,
  `session_end`, `user_prompt_submit`, `stop` are rendered into each agent's own hook file
  (`.claude/settings.json`, `.codex/config.toml`, `.cursor/hooks.json`,
  `.github/hooks/speckit.json`, `opencode.json`, and others), and each integration declares how
  hook output is fed back as context (`SK:src/specify_cli/integrations/claude/__init__.py#L57-L66`,
  `SK:src/specify_cli/integrations/codex/__init__.py#L36-L45`,
  `SK:src/specify_cli/integrations/base.py#L1036-L1050`). Precedence: built-in defaults →
  extension `events:` → `.specify/integration-events.yml` (`SK:src/specify_cli/events/__init__.py`).
- The `agent-context` extension maintains CLAUDE.md, AGENTS.md, GEMINI.md via markers
  (`SK:extensions/agent-context/README.md`).

### 1.3 Configurability

- Core commands are fixed templates (`SK:templates/commands/`). On top sits a YAML **workflow
  engine** with step types `command`, `prompt`, `shell`, `init`, `slot`, `gate`, `if`, `switch`,
  `while`, `do-while`, `fan-out`, `fan-in`; a step may choose its own integration and model;
  `specify workflow run … --json` and `resume` (`SK:docs/reference/workflows.md`).
- **Overlays and slots**: `.specify/workflows/overlays/<id>/*.yml` edit a workflow's step list and
  survive workflow updates; named `type: slot` steps are extension points skipped until an overlay
  fills them (`SK:docs/reference/workflows.md` "Workflow Overlays", "Workflow slots").
- Four primitives plus bundles: **extensions** (`extension.yml` with `requires.speckit_version`,
  `provides.commands`, `config`, `hooks` — `SK:extensions/git/extension.yml`), **presets**
  (override templates/commands/scripts with replace, prepend, append or wrap), **workflows**,
  **integrations**, and **bundles** that pin a role-based stack (`SK:docs/reference/bundles.md`).
- File resolution stack: `.specify/templates/overrides/` → presets by priority → extensions by
  priority → core (`SK:docs/reference/presets.md` "File Resolution"). Extension config merges
  `extension.yml` defaults → `<ext>-config.yml` (committed) → `<ext>-config.local.yml` (gitignored) →
  `SPECKIT_<EXT>_*` env (`SK:docs/reference/extensions.md`).
- Catalogs on 2026-09-29: 4 official + 176 community extensions, 2 + 40 presets, 3 + 2 workflows,
  2 bundles (`SK:extensions/catalog.community.json` and siblings). The community catalog is
  `install_allowed: false` (discovery only) — an explicit trust split
  (`SK:docs/reference/extensions.md` "Trust model").

### 1.4 Mechanical enforcement

- **Extension hooks (`before_plan`, `after_implement`) are executed by the LLM, not by code**: the
  command template tells the agent to read `.specify/extensions.yml` and run them, adding "Emitting
  the block alone does not run the hook"; `condition` fields are not evaluated, hooks with one are
  skipped (`SK:templates/commands/plan.md#L27-L58`, `SK:docs/reference/extensions.md`).
- Code-level pieces: workflow gates that pause a run (`verdict_input` lets CI decide a gate),
  persisted `state.json`/`log.jsonl`, `specify workflow validate`, `specify integration status
  --json` (exit 1, reports modified or missing managed files, meant for CI), rendered agent-native
  hooks, and the prerequisite scripts from the earlier note (`SK:docs/reference/workflows.md`,
  `SK:docs/reference/integrations.md`). `shell` steps are not sandboxed and `requires` is advisory.
- No content validator in core; the community fills the gap (`ci-guard`, `gates` — "Deterministic
  quality enforcement … agent hooks, git checks, and CI pipelines with one policy",
  `arch-governance`) (`SK:extensions/catalog.community.json`).

### 1.5 Tracker integration

- Core: `/speckit.taskstoissues` turns `tasks.md` into GitHub issues through the GitHub MCP server,
  executed by the LLM, deduplicating on the `T\d{3,}` id and refusing non-GitHub remotes
  (`SK:templates/commands/taskstoissues.md`).
- Community extensions: `github-issues`, `gh-triage`, `jira`, `jira-sync`, `linear`,
  `azure-devops`, and the `maqa` family for GitHub Projects v2, Jira, Linear, ADO
  (`SK:extensions/catalog.community.json`). Quality of these is unconfirmed.

---

## 2. Fission-AI OpenSpec

Base: `OS:` = https://github.com/Fission-AI/OpenSpec/blob/d4e1c77/

### 2.1 Installation and update

- Node ≥ 20.19.0; `npm install -g @fission-ai/openspec@latest` (pnpm, bun, yarn, Nix too)
  (`OS:package.json`, `OS:docs/installation.md`). The docs also ship a copy-paste "install with your
  AI assistant" prompt with explicit stop points (`OS:docs/installation.md`).
- Setup: `openspec init [--tools a,b|all|none] [--profile core|custom] [--force]`; creates
  `openspec/{specs,changes,config.yaml}` and per-tool skills and/or commands, e.g.
  `.claude/skills/openspec-*`, `.claude/commands/opsx/*.md`, `.agents/skills/` (`OS:docs/cli.md`).
- On the machine: global config in `$XDG_CONFIG_HOME/openspec/` with `profile`, `delivery`,
  `workflows` keys (`OS:src/core/global-config.ts#L6-L67`).
- Upgrade: `openspec update` regenerates files; it first checks npm for a newer CLI (≤ 1.5 s,
  skipped under CI or `DO_NOT_TRACK`) and offers the install command (`OS:docs/cli.md`).
- Pinning only through npm (`@<version>`); no project lock beyond a `generatedBy` version stamp
  inside generated skills (`OS:docs/cli.md`).
- **Local edits are overwritten by design.** Skills are rewritten when their stamp differs from the
  CLI; command files are compared by content and any edit counts as drift; the docs say to keep
  your own instructions elsewhere. An `.openspec-target` marker records who owns the shared
  `.agents/skills/` directory (`OS:docs/cli.md`, `OS:docs/supported-tools.md`).

### 2.2 Multi-runtime

- 40 tool ids in `AI_TOOLS`, including a vendor-neutral `agents` target writing to `.agents/skills/`
  (`OS:src/core/config.ts#L41`); 31 command adapters, one per tool
  (`OS:src/core/command-generation/adapters/`).
- Delivery mode `skills | commands | both`; command files use each tool's format (`.toml` Gemini,
  `.prompt` Continue, `.prompt.md` Copilot/Kiro); some tools (Codex, Zed, Kimi) are skills-only
  (`OS:docs/supported-tools.md`).
- OpenSpec **stopped writing AGENTS.md** and strips its old marker blocks
  (`OS:docs/supported-tools.md`). No agent-native hook generation was found in `src/`.

### 2.3 Configurability

- **The pipeline is a schema**: `openspec/schemas/<name>/schema.yaml` declares artifacts with `id`,
  `generates`, `template`, `instruction`, `requires` (a DAG) and an `apply` section
  (`requires`, `tracks: tasks.md`); `openspec schema fork spec-driven my-workflow`; resolution
  `--schema` → the change's `.openspec.yaml` → `config.yaml` → default `spec-driven`
  (`OS:docs/customization.md`).
- Stated philosophy "Actions, Not Phases": fluid actions instead of phase gates
  (`OS:docs/workflows.md`, `OS:docs/opsx.md#L322-L351`).
- `openspec/config.yaml`: `schema`, `context` (injected into every artifact prompt),
  `rules.<artifactId>[]`, `operations.apply|archive.guidance[]` (`OS:docs/customization.md`).
- Profiles: `core` (propose, explore, apply, update, sync, archive) and `custom` (pick extra
  workflows), stored in *global* config (`OS:docs/supported-tools.md`, `OS:docs/cli.md`).
- The workflow prompts themselves are hard-coded TypeScript templates; no plugin mechanism
  (`OS:src/core/templates/workflows/`). Community extension is schemas only (5 listed, copied by
  hand, no catalog) (`OS:docs/customization.md` "Community Schemas").
- "Stores" (beta): separate planning repositories registered per machine for cross-repo work
  (`OS:docs/stores-beta/user-guide.md`).

### 2.4 Mechanical enforcement

- `openspec validate [--all|--changes|--specs] [--strict] [--json]` checks structure in code:
  required headers, SHALL/MUST per requirement, "Why" length 50–1000, zero-delta changes fail,
  MODIFIED requirements checked against the main specs; exit 1 on failure
  (`OS:docs/cli.md`, `OS:src/core/validation/constants.ts#L6-L63`). `--archived` fails if an
  archived change has unticked tasks; the docs suggest pre-commit.
- `openspec status --json` computes the artifact DAG with `done | ready | blocked | skipped`;
  `openspec archive` merges spec deltas in code (`OS:docs/agent-contract.md` §4.4, §4.9).
- **A documented agent contract**: one JSON document per call on stdout, a shared diagnostic
  envelope `{severity, code, message, target, fix}`, and `openspec instructions <artifact> --json`
  returning template, dependencies, rules and context at runtime, so instructions are not frozen
  into generated files (`OS:docs/agent-contract.md` §1–2, §4.5–4.6). The CLI splits human-only and
  agent-compatible commands (`OS:docs/cli.md`).
- Content quality (`/opsx:verify`) is LLM judgment. A community schema documents the limit: "OpenSpec
  only checks that artifacts exist, so enforce the gate with your own CI or hook"
  (`OS:docs/customization.md`). No CI template ships.

### 2.5 Tracker integration

- None. Local Markdown under `openspec/`; OpenSpec "never touches git"; branch and PR flow is a
  convention (`OS:docs/team-workflow.md`).

### 2.6 Other

- Telemetry is **on by default** (anonymous PostHog via `edge.openspec.dev`), off with
  `OPENSPEC_TELEMETRY=0`, `DO_NOT_TRACK=1` or CI (`OS:src/telemetry/index.ts`).

---

## 3. BMAD Method

Base: `B:` = https://github.com/bmad-code-org/BMAD-METHOD/blob/1cbcfa2/ ;
`B612:` = https://github.com/bmad-code-org/BMAD-METHOD/blob/9ee3dfd/ (branch `V6.12`).

BMAD is mid-migration: "The 6.12 npm installer is maintained separately on `V6.12`" (`B:AGENTS.md`),
while `main` installs through the third-party Skills CLI or plugin marketplaces. Docs on `main` are
out of sync (`B:docs/customize/add-modules.md` still documents `npx bmad-method install`).

### 3.1 Installation and update

- **v6.12 (npm installer).** `npx bmad-method install` with `--modules`, `--tools`, `--set`,
  `--action install|update|quick-update`, `--channel`, `--pin <spec>` and more
  (`B612:tools/installer/commands/install.js#L13-L48`); Node ≥ 20.12. Installs into `_bmad/` with
  `_bmad/_config/manifest.yaml` (modules, versions, sources). Channels for modules: `stable` (highest
  semver tag), `next` (main HEAD), `pinned`; the manifest stores `sha` and `registryApprovedSha`
  (`B612:tools/installer/modules/channel-resolver.js`,
  `B612:tools/installer/core/manifest-generator.js#L354-L369`). npm dist-tags include a `rollback`
  tag at 4.39.0 (`npm view bmad-method dist-tags`, 2026-09-29).
- **Edits in v6.12**: `_config/files-manifest.csv` holds SHA-256 per file; on update, unknown files
  are treated as custom and restored, changed files are overwritten and the user's version is saved
  as `<file>.bak` (`B612:tools/installer/core/manifest-generator.js#L671-L742`,
  `B612:tools/installer/core/installer.js#L561-L612`).
- **main.** `npx skills add bmad-code-org/BMAD-METHOD --skill bmad …`, or
  `/plugin marketplace add bmad-code-org/bmad-plugins`, or the Codex marketplace; then `bmad setup`,
  which needs `uv` and Python 3.11+ ("Rendered skills have no interpreter fallback")
  (`B:docs/start/install-bmad.md`, `B:CHANGELOG.md` v6.11.0). The Skills CLI is
  `vercel-labs/skills` (32,730 stars, MIT, "Supports OpenCode, Claude Code, Codex, Cursor, and 75
  more", symlink-or-copy install to project or `-g` global, `npx skills update`) —
  https://github.com/vercel-labs/skills (README at `3694740`).
- Re-running `bmad setup` updates: checks module versions, runs `npx skills update`, asks new config
  questions, **moves `_bmad/custom/` files of renamed skills, offers to delete retired skills**, then
  offers migrations (`B:skills/bmad/references/setup.md`). Each module is described by a
  `bmod.toml` with `version`, `update_source = "github:…"`, `required_skills` with versions and
  `renamed`/`removed` lists — "A retired name is never reused" (`B:skills/bmod-method/bmod.toml`).
- **Edits on main never go into shipped files**: overrides live in `_bmad/custom/` ("Updates do not
  touch your files"); each skill's `customize.toml` is the schema that updates overwrite
  (`B:docs/customize/customize-bmad.md`). Rendered skills publish content-addressed snapshots to
  `_bmad/render/` with a hash manifest (`B:CHANGELOG.md` v6.11.0).

### 3.2 Multi-runtime

- v6.12 registry `platform-codes.yaml`: 48 platforms, each just a `target_dir` and
  `global_target_dir` for skills (e.g. `.claude/skills`, `.agents/skills` shared by codex, amp,
  auggie…) with `preferred`/`suspended` flags; "Multiple platforms may share the same target_dir …
  `.agents/skills/` cross-tool standard" (`B612:tools/installer/ide/platform-codes.yaml`).
- **Rendering is uniform**: the same `SKILL.md` directories are copied to every tool; no per-runtime
  command files. On `main`, targeting is delegated entirely to the Skills CLI and marketplaces
  (`B:docs/start/install-bmad.md`). The plugins repo carries `.claude-plugin/marketplace.json`,
  `.agents/plugins/marketplace.json` and `.codex-plugin/plugin.json` files
  (https://github.com/bmad-code-org/bmad-plugins at `d009608`).
- No hooks, no SessionStart bootstrap: discovery relies on the host's native skill mechanism
  (search of `B:skills/`). Web bundles package workflows as Gemini Gems and Custom GPTs
  (`B:tools/bundle_web_bundles.py`).

### 3.3 Configurability

- Not fixed stages but a sized path: planning tools "are independent tools, not stages"; intent →
  `bmad-spec` → `bmad-build` if it fits one session, otherwise `bmad-ticket` then one build per
  story (`B:docs/plan/choose-a-planning-path.md`); "Build decides how much ceremony a change needs
  after investigating it" (`B:CHANGELOG.md` v6.12).
- **Layered TOML config**: `_bmad/config.toml` → `config.user.toml` → `custom/config.toml` →
  `custom/config.user.toml`; per skill `customize.toml` → `_bmad/custom/{skill}.toml` (team,
  committed) → `{skill}.user.toml` (personal, gitignored). Merge by value shape: scalars replace,
  tables deep-merge, arrays of tables keyed by `code`/`id` merge by key, other arrays append; an
  override cannot delete a base item (`B:docs/customize/customize-bmad.md`).
- Customizable surfaces include `persistent_facts`, `activation_steps_prepend/append`,
  `on_complete`, menus, template paths and `[[workflow.review_layers]]` (add, replace or disable a
  reviewer, including an external tool) (`B:docs/customize/customize-bmad.md`). Resolution is
  inspectable: `uv run _bmad/scripts/resolve_customization.py --skill … --key agent` prints JSON.
- Modules (official `bmb`, `cis`, `gds`, `tea`, `loop`; community from any Git URL with `@ref`, with
  an "UNVERIFIED MODULE" warning) (`B:docs/customize/add-modules.md`); a community registry repo
  https://github.com/bmad-code-org/bmad-plugins-marketplace.

### 3.4 Mechanical enforcement

- Judgment vs bookkeeping split with `uv` scripts: `tickets.py` (find, next, `mark <ref> <status>`),
  `lint_spine.py`, `setup.py`, `render_skill.py` (`B:skills/*/scripts/`); renderers exit 1 on a
  missing config key (`B:CHANGELOG.md` v6.11.0). Build Auto halts with machine-readable reasons
  and "never" marks a ticket done — only a person or orchestrator does
  (`B:docs/build/autonomous-development-loops.md`).
- Readiness gate in sprint planning (PASS/CONCERNS/FAIL) is LLM-judged (`B:CHANGELOG.md` v6.11.0).
- **No hooks**: "Hooks are not integrated yet, so nothing syncs on its own"
  (`B:docs/plan/set-up-the-ticket-tree.md#L128`). No CI template for consumers. BMAD's own CI runs
  deterministic skill and manifest validators (`B:.github/workflows/quality.yaml`,
  `B:tools/skill-validator.md`); repository policy: "Do not write automated tests for LLM output"
  (`B:AGENTS.md`).

### 3.5 Tracker integration

- `bmad-ticket` asks where tickets live and records it in `_bmad/custom/ticketing-store-config.toml`:
  Repo (default, Markdown), GitHub Issues (sub-issues and blocked-by), Jira, Linear, Notion, Trello.
  The Markdown files stay the working copy, tracker status is mirrored as `tracker_status`, sync
  happens only when the skill runs, and trackers "still need a lot of testing"
  (`B:docs/plan/set-up-the-ticket-tree.md`). `tickets.py mark` refuses to run on a tracker store
  (`B:docs/build/autonomous-development-loops.md#L56`).

---

## 4. obra/superpowers

Base: `S:` = https://github.com/obra/superpowers/blob/8ca22db/

### 4.1 Installation and update

- **Only through each host's native plugin system**, one install per host: e.g.
  `/plugin install superpowers@claude-plugins-official`,
  `gemini extensions install https://github.com/obra/superpowers`,
  `copilot plugin install superpowers@superpowers-marketplace`, `pi install git:…`, the Codex
  marketplace, an OpenCode config entry (`S:README.md` "Installation", `S:.opencode/INSTALL.md`).
- **Nothing is installed into the project**; the project footprint is only the artifacts it writes
  (`docs/superpowers/specs/…-design.md`, `docs/superpowers/plans/…md`, worktrees)
  (`S:skills/brainstorming/SKILL.md`, `S:skills/writing-plans/SKILL.md`).
- Updates are "somewhat coding-agent dependent, but are often automatic" (`S:README.md`
  "Updating"). One version string lives in 11 manifests kept in sync by `scripts/bump-version.sh`
  (`S:.version-bump.json`); users pin only through the host's plugin manager.
- Local edits: none supported; personal skills go into the runtime's own skills directory, and
  user instructions in CLAUDE.md/AGENTS.md "take precedence over skills"
  (`S:skills/using-superpowers/SKILL.md`, `S:skills/writing-skills/SKILL.md`).

### 4.2 Multi-runtime

- 16 hosts: Claude Code, Antigravity, Codex app and CLI, Cursor, Devin CLI, Factory Droid, Gemini
  CLI, Copilot CLI, Grok Build, Kimi Code, OpenCode, Pi, Qwen Code, Hermes, Muse (`S:README.md`).
- One repo holds per-host manifests: `.claude-plugin/`, `.codex-plugin/`, `.cursor-plugin/`,
  `.devin-plugin/`, `.opencode/plugins/`, `.pi/extensions/`, `gemini-extension.json`,
  `.agents/plugins/marketplace.json`, and others (repo root at `8ca22db`).
- **Porting architecture** (`S:docs/porting-to-a-new-harness.md`): skills are harness-agnostic and
  "name actions, not tools"; a per-host tool map in
  `skills/using-superpowers/references/<harness>-tools.md`; a per-host bootstrap injects
  `using-superpowers/SKILL.md` at session start — "The bootstrap is the entire integration". A host
  without automatic session-start injection is "not supported"; rule: "Never edit the user's files".
- Bootstrap implementations: Claude Code `SessionStart` hook (`startup|clear|compact`) running one
  script that detects the host from environment variables (`S:hooks/hooks.json`,
  `S:hooks/session-start`); Gemini via `contextFileName: GEMINI.md` (`S:gemini-extension.json`);
  Hermes lacks a post-compaction hook, so the bootstrap is lost in long sessions (`S:README.md`).

### 4.3 Configurability

- **Fixed pipeline in prose**: brainstorming → using-git-worktrees → writing-plans →
  subagent-driven-development / executing-plans → test-driven-development → requesting-code-review →
  finishing-a-development-branch; "Mandatory workflows, not suggestions" (`S:README.md`).
- No config file, profiles or presets; output locations change only by "User preferences" in prose
  (`S:skills/writing-plans/SKILL.md`). Extension = write your own skills; domain-specific ones
  "belong in a standalone plugin" (`S:README.md` "Contributing"). The project reports a 94% PR
  rejection rate (`S:AGENTS.md`).

### 4.4 Mechanical enforcement

- In-session enforcement is prose pressure: "If you think there is even a 1% chance a skill might
  apply … you ABSOLUTELY MUST invoke the skill" (`S:skills/using-superpowers/SKILL.md`); "NO
  PRODUCTION CODE WITHOUT A FAILING TEST FIRST" (`S:skills/test-driven-development/SKILL.md`); "NO
  COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE"
  (`S:skills/verification-before-completion/SKILL.md`).
- The only hook is SessionStart context injection; no blocking hooks, no CI or validators for
  consumers (`S:hooks/hooks.json`).
- The skills themselves are tested offline: infrastructure tests per host in `tests/`, and
  behavioural evals in a separate harness that drives real Claude Code, Codex and Gemini CLI
  sessions in tmux with an LLM verifier plus deterministic post-checks (`S:docs/testing.md`,
  https://github.com/prime-radiant-inc/superpowers-evals).

### 4.5 Tracker integration

- None; `finishing-a-development-branch` offers merge, PR, keep or discard (`S:README.md`).

---

## 5. Kiro (AWS)

Sources: https://kiro.dev/docs/ (pages cited inline), https://kiro.dev/changelog/ ,
https://github.com/kirodotdev/Kiro/blob/bfe7ff3/README.md . Kiro is a closed product; the GitHub repo
is "the public issue and feedback tracker for Kiro".

### 5.1 Installation and update

- IDE: downloaded installer for macOS, Windows, Linux; "downloads updates automatically in the
  background"; older releases can be downloaded to downgrade; no documented pinning
  (https://kiro.dev/docs/getting-started/installation/).
- CLI: `curl -fsSL https://cli.kiro.dev/install | bash` (https://kiro.dev/docs/cli/headless/); user
  config in `~/.kiro/`, `kiro-cli doctor` (https://kiro.dev/docs/cli/setup/).
- Project files, all under `.kiro/`: `steering/*.md`, `hooks/*.json`, `agents/<name>.json|.md`,
  `skills/`, `settings/mcp.json`, `specs/<feature>/`, with global mirrors in `~/.kiro/`
  (https://kiro.dev/docs/steering/ , /hooks/ , /custom-agents/ , /skills/ ,
  /mcp/configuration/). Kiro writes no managed files into the project, so there is no update-merge
  problem (inferred; unconfirmed).
- Organisation distribution: global steering can be "pushed to user's PCs via MDM solutions or
  Group Policies, or downloaded … from a central repository" (https://kiro.dev/docs/steering/).

### 5.2 Multi-runtime

- Kiro surfaces only (IDE, CLI, Web, Mobile, Crew); "One unified agent harness powers every Kiro
  surface" (tracker README). Since the 2026-08-03 "one agent" change, specs run in CLI and web too;
  agents, hooks and Cedar permissions are shared; clients talk to the harness over the Agent Client
  Protocol (https://kiro.dev/blog/one-agent/).
- **Portability is inbound only**: reads `AGENTS.md` (always included, no inclusion modes), Agent
  Skills `SKILL.md`, Powers in the Agent Plugins format (`plugin.json`, `skills/`, `mcp.json`), MCP
  (https://kiro.dev/docs/steering/ , /skills/ , /powers/). It generates nothing for other runtimes.

### 5.3 Configurability

- **Pipeline hardcoded**: Feature spec (Requirements-First, Design-First, or Quick Spec without
  approval gates) and Bugfix spec; three fixed phases; no custom spec types or templates documented
  (https://kiro.dev/docs/specs/).
- Extension happens in context and tools: steering inclusion `always | fileMatch | manual | auto`
  with live `#[[file:…]]` refs; custom agents (`tools`, `permissions`, `prompt`, `model`, `hooks`,
  `includeMcpJson`, …); Powers activated by task keywords from a registry, GitHub URL or folder
  (https://kiro.dev/docs/steering/ , /custom-agents/ , /powers/).
- Precedence is explicit: workspace over global for steering; agent `mcpServers` > workspace >
  global for MCP, merged (https://kiro.dev/docs/mcp/configuration/). No per-project-kind profiles
  found.

### 5.4 Mechanical enforcement

- Hooks: 11 triggers in `{"version":"v1","hooks":[…]}`; only `PromptSubmit`, `PreToolUse`,
  `PreTaskExecution` can block (non-zero exit); `AgentStop`, `PostToolUse`, file events,
  `PostTaskExecution` cannot (https://kiro.dev/docs/hooks/). So there is no blocking "stop gate".
- Headless CI: `kiro-cli chat --no-interactive` with `KIRO_API_KEY`, `--trust-tools`, `--agent`,
  `--output-format stream-json`, documented exit codes (https://kiro.dev/docs/cli/headless/).
- No validator for spec or steering files; gates are UI approvals between phases
  (https://kiro.dev/docs/specs/).

### 5.5 Tracker integration

- Kiro Web + GitHub: the `kiro` label or a `/kiro` comment assigns an issue to the agent, which
  branches, opens and updates PRs and reacts to Actions feedback
  (https://kiro.dev/docs/autonomous-agent/github/). Jira/Confluence only as MCP context (marketing
  text; native sync unconfirmed). Specs themselves are local files.

### 5.6 Maturity

- Proprietary ("AWS Content" under the AWS Customer Agreement, https://kiro.dev/license/); pricing
  Free to $200 per user per month (https://kiro.dev/pricing/); very high cadence — IDE 1.1, CLI
  2.22–2.24, Crew 0.7 between 2026-09-14 and 09-28 (https://kiro.dev/changelog/).

---

## 6. GSD — "Get Shit Done", now GSD Core

Base: `G:` = https://github.com/open-gsd/gsd-core/blob/d8533f6/

Lineage. `gsd-build/get-shit-done` (64k stars) is archived and its README says "GSD Has Moved…
continues as GSD Core" at `open-gsd/gsd-core`
(https://github.com/gsd-build/get-shit-done/blob/bdcaab2/README.md). `gsd-build/gsd-2` redirects to
GSD Pi, which is a separate standalone coding agent with its own TUI, not a plugin for other runtimes
(https://github.com/open-gsd/gsd-pi/blob/4d64f42/README.md). The npm package was renamed from
`get-shit-done-cc` to `@opengsd/gsd-core` and the version counter reset; leftover old installs
poisoned a shared update cache, so the installer now detects and removes them
(`G:docs/cleanup-get-shit-done-cc.md`). Why the project changed organisations is unconfirmed.

What it is: a fixed per-phase loop Discuss → Plan → Execute (parallel waves in fresh subagent
contexts) → Verify → Ship, with state in `.planning/` (`G:README.md#L24-L39`,
`G:docs/reference/planning-artifacts.md`).

### 6.1 Installation and update

- `npx @opengsd/gsd-core@latest`, interactive runtime and scope choice, or flags such as
  `--claude --global`, `--codex --local` (`G:docs/how-to/install-on-your-runtime.md#L20-L36`).
  Runtime requirement is inconsistent: the doc says "Node.js 18+" (same file, L5) while
  `package.json` has `engines.node >=24.0.0` (`G:package.json#L59`).
- **The installer is mandatory**: sources are authored in Claude Code format and converted per
  runtime — "Do not copy files from `agents/` or `commands/` directly"
  (`G:docs/how-to/install-on-your-runtime.md#L11-L16`).
- Global install goes to the runtime's config home (`~/.claude/`, `~/.codex/skills/gsd-*/SKILL.md`
  plus `~/.codex/agents/gsd-*.toml`); local install to the project's runtime dir plus `gsd-core/`
  with workflows, templates and the `gsd-tools.cjs` CLI; project state in `.planning/`
  (`G:docs/how-to/install-on-your-runtime.md`, `G:package.json`, `G:docs/CONFIGURATION.md#L9-L11`).
- A second path, `claude plugin install gsd-core`, uses a different command namespace
  (`/gsd-core:<cmd>`), cannot apply install-time config and still needs `gsd-tools` and `node` on
  PATH (`G:docs/how-to/install-on-your-runtime.md#L65-L100`).
- Update: `/gsd-update` with `--next`/`--rc`, `--sync` (align several runtime roots) and `--reapply`
  (`G:commands/gsd/update.md`).
- **The most complete edit-survival scheme found**: `gsd-file-manifest.json` with SHA-256 per file;
  on reinstall, modified files go to `gsd-local-patches/`, pristine copies to `gsd-pristine/`, and
  `--reapply` performs a three-way merge of pristine, user and new versions
  (`G:bin/install.js#L9849-L9857`, `G:docs/manual-update.md#L61`).
- Cost: `bin/install.js` is 14,922 lines (local count at `d8533f6`).

### 6.2 Multi-runtime

- Sections for Claude Code, OpenCode, Kilo, Codex, Kimi, Copilot, Cursor, Windsurf/Devin, Cline,
  CodeBuddy, Qwen, Augment, Antigravity, Trae, ZCode, pi (`G:docs/how-to/install-on-your-runtime.md`).
- **The runtime registry is data**: one `capabilities/<runtime>/capability.json` per runtime with
  `role: "runtime"`, a support tier (tier 1: claude, codex, antigravity, vscode), `configHome`,
  `artifactLayout` (skill and agent destinations with a named converter such as
  `convertClaudeCommandToCodexSkill`), `commandStyle`, `hooksSurface`, `hookEvents`, dispatch limits,
  `frontmatterDialect` and `unsupportedFeatures` (`G:capabilities/codex/capability.json`).
- **Explicit degradation**: Claude Code gets ~14 hooks (SessionStart, Pre/PostToolUse, SubagentStop,
  Stop, PreCompact, FileChanged); Codex gets only a SessionStart update check because context
  monitoring relies on Claude's statusline; OpenCode has no hook surface, so GSD ships a JS plugin
  bridging OpenCode's event bus to the same hook scripts
  (`G:docs/how-to/install-on-your-runtime.md#L44-L58`, `#L126-L130`, `#L172-L184`).
- Runtime churn: Gemini CLI was removed as a runtime in 1.8.0 and as a reviewer lane in 1.15.0; GSD
  gives Google's change of free-tier terms as the reason (GSD's claim, unconfirmed); CI fails if a
  retired runtime is named as live in shipped Markdown (`G:CHANGELOG.md#L26-L32`, `#L96`).
- Cost: each workflow opens with a shell preamble probing ~16 runtime config homes to find
  `gsd-tools.cjs` (`G:gsd-core/workflows/inbox.md`); the changelog records cross-runtime leaks such as
  Codex agents keeping `@~/.claude/...` includes (`G:CHANGELOG.md` 1.15.0).

### 6.3 Configurability

- **Fixed loop, closed extension points.** 12 "loop extension points" (`discuss:pre/post`,
  `plan:pre/post`, `execute:pre`, `execute:wave:pre/post`, `execute:post`, `verify:pre/post`,
  `ship:pre`, …). A capability manifest contributes `steps` (topologically ordered by
  `produces`/`consumes`), `contributions` (prompt fragments injected into a named agent role) and
  `gates` (`blocking`, `onError: skip|halt`); a build step rejects hook kinds that a point does not
  dispatch (`G:docs/reference/capability-manifest.md#L72-L165`). Built-in features are capabilities
  too, e.g. `tdd` owns `workflow.tdd_mode`, injects a planner fragment at `plan:pre` and adds a
  non-blocking gate at `execute:post` (`G:capabilities/tdd/capability.json`).
- Per-project `.planning/config.json`: `mode` (`interactive | yolo`), `granularity`,
  `model_profile` and per-phase models, ~40 `workflow.*` toggles (research, plan_check, verifier,
  tdd_mode, use_worktrees, code_review, build_command, test_command, …), `gates.*`, `git.*`,
  `hooks.*` (`G:docs/CONFIGURATION.md#L13-L80`, `#L216-L243`); a `FileChanged` hook hot-reloads it.
- Presets: `context_profile` dev/research/review and a documented table of setting combinations
  (Prototyping / Normal / Production); no project-kind presets (`G:docs/CONFIGURATION.md#L243`,
  `#L643-L651`).
- Third-party capabilities from `~/.gsd/capabilities/<id>/` and `<project>/.gsd/capabilities/<id>/`;
  first-party wins, collisions rejected, incompatible `engines.gsd` skipped; **gates from a failed
  overlay fail open with a warning**; `capabilities.strict_known_registries` allowlist, `auto_update`
  off by default, and **re-consent whenever the set of executable surfaces (hooks, commands, MCP
  servers) changes** (`G:docs/CONFIGURATION.md#L1024-L1043`, `#L1187-L1220`).

### 6.4 Mechanical enforcement

- `gsd-tools.cjs` with 20 modules; validators `verify plan-structure | phase-completeness |
  references | commits | artifacts | key-links` and `validate consistency | health [--repair]`
  returning coded diagnostics `{code, message, fix, repairable}` (`G:docs/CLI-TOOLS.md#L7-L20`,
  `#L769-L825`).
- A **blocking decision-coverage gate**: every `D-NN` decision in CONTEXT.md must appear in a plan's
  must-haves, objective or tasks, or the phase is not marked planned; the verify-phase counterpart
  only warns (`G:docs/CONFIGURATION.md#L1385-L1412`). Confirmation gates apply only in interactive
  mode; `yolo` auto-approves (`G:docs/CONFIGURATION.md#L1348-L1360`).
- Hooks: the workflow guard is advisory for edits, with one fail-closed block on `git add -f` in
  agent/worktree branches (`G:hooks/gsd-workflow-guard.js#L7-L14`); a Conventional Commits check
  exits 2, only when `hooks.community: true` (`G:hooks/gsd-validate-commit.sh#L1-L9`); plus
  read-before-edit, secret-read, worktree-path guards and a context monitor.
- No CI template for consumer projects (none found); discuss/plan/verify content and UAT are LLM
  judgment.

### 6.5 Tracker integration

- None native: the issue-driven guide is "documentation only. No new commands, no daemon, no
  tracker integration" (`G:docs/issue-driven-orchestration.md#L1-L21`). Tracking is local
  (`ROADMAP.md`, `STATE.md`, `BACKLOG.md`); `gh` is used ad hoc for PRs (`/gsd-ship`) and inbox
  triage (`G:gsd-core/workflows/ship.md#L371`, `G:gsd-core/workflows/inbox.md`).

---

## 7. Ruflo (formerly claude-flow)

Base: `R:` = https://github.com/ruvnet/ruflo/blob/fce8e6d/

What it is. Self-described "agent meta-harness for Claude Code and Codex" with "100+ specialized
agents, coordinated swarms, self-learning memory, federation" (`R:README.md`). In code it is an npm
CLI plus an MCP server; the package is still named `claude-flow` 3.48.0 (`R:package.json#L1-L16`).
The README's performance and accuracy figures ("89% accuracy" routing, "1.3×–1953×" vs other
frameworks) are **vendor claims**; the README links a `verification.md` that does not exist at
`fce8e6d` (`R:README.md#L392`, checked with `find`). `docs/STATUS.md` is pinned at 3.10.2 while
HEAD is 3.48.0.

### 7.1 Installation and update

- Two paths with different surfaces: Claude Code plugins (`/plugin marketplace add ruvnet/ruflo`,
  33 plugins; no workspace files, no hooks) or the CLI `npx ruflo@latest init [wizard]` /
  `npm i -g ruflo`, which writes `.claude/`, `.claude-flow/`, `CLAUDE.md`, `.mcp.json`,
  `.swarm/memory.db` (`R:README.md`, `R:v3/@claude-flow/cli/src/init/executor.ts#L161-L167`).
  Node ≥ 20; ~340 MB default install (`R:docs/USERGUIDE.md#L548-L556`).
- Init presets `DEFAULT | MINIMAL | FULL` plus per-component, per-hook and per-skill-group booleans
  (`R:v3/@claude-flow/cli/src/init/types.ts#L12-L135`).
- Update: `init upgrade` **force-overwrites "critical helpers"**; `--add-missing` copies files only
  when absent, so customisations survive only by never being updated; no manifest or merge
  (`R:v3/@claude-flow/cli/src/init/executor.ts#L574`, `#L767`). An existing `.claude/settings.json`
  is merged (hooks only if absent, permission rules appended, pre-existing hooks risk-scanned)
  (`R:v3/@claude-flow/cli/src/init/executor.ts#L900-L955`). Installed helper bytes are covered by a
  signed manifest (`R:v3/@claude-flow/cli/src/init/helper-signing.ts`).

### 7.2 Multi-runtime

- Claude Code and Codex only: `init --codex` writes AGENTS.md, `.agents/skills/`, `config.toml` and
  registers MCP via `codex mcp add`; `init --dual` sets up both, with Claude as "ORCHESTRATOR" and
  Codex as "EXECUTOR" (`R:docs/USERGUIDE.md#L586-L660`,
  `R:v3/@claude-flow/codex/src/generators/`). No Gemini CLI, OpenCode or Cursor adapter found in
  `init`.

### 7.3 Configurability, enforcement, tracker

- **No defined development pipeline**: workflow templates are hardcoded in the CLI (development,
  research, testing, security-audit, code-review, refactoring, sparc, custom); a
  `workflow run -f ./workflow.yaml` is advertised but its schema is unconfirmed
  (`R:v3/@claude-flow/cli/src/commands/workflow.ts#L11-L21`). SPARC is an optional plugin
  (`R:plugins/ruflo-sparc/`).
- ~10 Claude Code hooks call `.claude/helpers/hook-handler.cjs` for routing, metrics and "learning"
  and are non-blocking by design (`R:v3/@claude-flow/cli/src/init/settings-generator.ts#L284-L465`).
  **The one "block" uses the wrong exit code**: `pre-bash` prints `[BLOCKED]` and calls
  `process.exit(1)` (`R:.claude/helpers/hook-handler.cjs#L444-L458`), but Claude Code blocks only on
  exit 2 (industry note §1.4). No artifact validators found.
- Tracker: local claims in `.claude-flow/claims/claims.json`; the GitHub sync is a stub returning
  `[]` with the comment "This would integrate with GitHub API"
  (`R:v3/@claude-flow/cli/src/services/claim-service.ts#L664-L667`); MCP `github-tools` shells out to
  `gh` for listings.

---

## 8. Agent OS (Builder Methods)

Base: `AO:` = https://github.com/buildermethods/agent-os/blob/475b0ca/ ; docs
https://buildermethods.com/agent-os

### 8.1 Installation and update

- The installation page asks for an email before showing commands
  (https://buildermethods.com/agent-os/installation). Base install lives in `~/agent-os/`
  (`commands/`, `profiles/<name>/standards/`, `scripts/`, `config.yml`)
  (https://buildermethods.com/agent-os/file-structure). Bash only (`AO:scripts/*.sh`).
- Project install: `~/agent-os/scripts/project-install.sh [--profile <name>] [--commands-only]`
  copies standards along the profile chain into `agent-os/standards/`, regenerates
  `agent-os/standards/index.yml` and copies commands into `.claude/commands/agent-os/`
  (`AO:scripts/project-install.sh`).
- **Update is manual**: `rm -rf ~/agent-os && git clone … ~/agent-os && rm -rf ~/agent-os/.git`,
  backing up `profiles/` by hand; "If you've customized any commands or scripts, back those up
  separately and merge your changes after updating" (https://buildermethods.com/agent-os/updating).
  Re-install asks "(y/N)" before overwriting standards; `--commands-only` overwrites commands
  unconditionally (`AO:scripts/project-install.sh`). No lock or pin beyond `version:` in
  `config.yml` (`AO:config.yml`).

### 8.2 Multi-runtime

- v3 installs only for Claude Code (`AO:scripts/project-install.sh`); other tools "reference the
  command file with `@`" (https://buildermethods.com/agent-os/adaptability), while commands assume
  Claude Code features — `shape-spec` "must be run in plan mode", "Always use AskUserQuestion tool"
  (`AO:commands/agent-os/shape-spec.md`).

### 8.3 Configurability

- Five hardcoded commands in v3: `discover-standards`, `inject-standards`, `index-standards`,
  `plan-product`, `shape-spec`. **v3 (2026-01-20) removed spec writing, task breakdown and
  orchestration** from v2.1, deferring them to runtimes' plan mode and todo lists
  (`AO:CHANGELOG.md`).
- **Profiles with inheritance**: named directories of standards; `profiles: <name>: inherits_from:
  <parent>` in `~/agent-os/config.yml`; the chain is resolved with cycle (`CIRCULAR:`) and missing
  (`NOTFOUND:`) detection, child files win (`AO:config.yml`, `AO:scripts/common-functions.sh`,
  https://buildermethods.com/agent-os/profiles). Suggested axes: stack (`rails`, `nextjs`), client,
  context.
- **Reverse flow**: `sync-to-profile.sh [--profile|--new-profile <name>] [--all] [--overwrite]`
  pushes project standards back into a profile with conflict checks and backups
  (`AO:scripts/sync-to-profile.sh`).
- No plugin mechanism; feature PRs are "rarely accepted into core" (`AO:.github/CONTRIBUTING.md`).

### 8.4 Enforcement and tracker

- No hooks, validators or CI for consumers; the only "gate" is a prose check that plan mode is on
  (`AO:commands/agent-os/shape-spec.md`). No tracker integration: specs are local files in
  `agent-os/specs/` (`AO:commands/agent-os/shape-spec.md`).

---

## 9. Comparison tables

Sources are the per-framework sections above; cells summarise them.

### 9.1 Distribution, update and runtimes

| | Install | Project footprint | Update and local edits | Runtimes | Rendering per runtime |
|---|---|---|---|---|---|
| spec-kit | `uv tool install specify-cli` (Python ≥ 3.11), pinned to a git tag; `specify init` | `.specify/` + generated commands/skills + native hook files | `specify self upgrade`; `integration upgrade`; **SHA-256 manifest, refuses on modified files** unless `--force` | ~43 | Python integration classes → Markdown / TOML / YAML / skills; canonical events → native hook configs |
| OpenSpec | `npm i -g @fission-ai/openspec` (Node ≥ 20.19) | `openspec/` + skills/commands per tool | `openspec update`; **generated files overwritten by design** | 40 | 31 adapters; skills, commands or both; no hooks, no AGENTS.md |
| BMAD | v6.12: `npx bmad-method install`; main: `npx skills add` or plugin marketplaces, then `bmad setup` (uv, Python 3.11+) | `_bmad/` (config, custom, render), skills in tool dir | v6.12: hash manifest + `.bak`; main: overrides in `_bmad/custom/` never touched, `bmod.toml` rename/remove lists | 48 platform dirs | Same `SKILL.md` copied everywhere; runtime targeting delegated to Skills CLI / marketplaces |
| Superpowers | Each host's plugin manager, once per host | Nothing managed; only artifacts it writes | Host plugin updates, often automatic; no project edits supported | 16 | Per-host manifest + per-host tool map + session-start bootstrap |
| Kiro | IDE installer / `curl … \| bash` CLI, auto-update | `.kiro/` user content only | Binary updates; no managed project files | Kiro surfaces only | Reads AGENTS.md, Agent Skills, Agent Plugins, MCP; emits nothing |
| GSD Core | `npx @opengsd/gsd-core@latest` (Node ≥ 24 per `package.json`) or a Claude plugin | runtime dir + `gsd-core/` + `.planning/` | `/gsd-update`; **SHA-256 manifest + pristine copy + three-way `--reapply`** | ~16 (tiered) | Declarative `capability.json` per runtime with converters and `unsupportedFeatures` |
| Ruflo | `npx ruflo@latest init` or Claude plugins | `.claude/`, `.claude-flow/`, `CLAUDE.md`, `.mcp.json`, memory DB | `init upgrade` force-overwrites helpers; `--add-missing`; settings merged | 2 (Claude Code, Codex) | Generators for CLAUDE.md vs AGENTS.md/config.toml |
| Agent OS | Email-gated docs; bash scripts in `~/agent-os` | `agent-os/` + `.claude/commands/agent-os/` | `rm -rf` + re-clone, manual backup | 1 installed (Claude Code); others by `@`-reference | None |

### 9.2 Configurability, enforcement, tracker, maturity

| | Pipeline definition | Per-project config and profiles | Extensions | Mechanical enforcement | Tracker | Stars / cadence (2026-09-29) |
|---|---|---|---|---|---|---|
| spec-kit | Fixed commands + YAML workflow engine (gates, loops, fan-out, resume) with overlays and slots | Layered overrides; extension config committed/local/env; bundles | Extensions, presets, workflows, integrations, bundles; 176 community extensions (discovery-only) | Workflow gates and state in code, `integration status --json`, native hooks; **extension hooks executed by the LLM** | GitHub issues via MCP (LLM); Jira/Linear/ADO community | 139k; 52 releases in 90 days |
| OpenSpec | **Schema DAG** of artifacts (`schema.yaml`), "actions, not phases" | `config.yaml` (context, per-artifact rules); `core`/`custom` profiles (global) | Custom schemas only, copied by hand | `openspec validate --strict --json`, `status --json`, archive merge in code; JSON agent contract | None | 71k; 11 releases in 90 days |
| BMAD | Sized path (spec → build or tickets → build); tools not stages | 4-layer TOML, team vs `.user` files, keyed merge | Modules from registry or Git URL | `uv` scripts for bookkeeping; no hooks, no consumer CI | **Store choice**: repo, GitHub Issues, Jira, Linear, Notion, Trello (mirror, sync on skill run) | 54k; 3 releases in 90 days, 183 commits/30 d |
| Superpowers | Fixed skill chain in prose | None | Own skills / separate plugins | Session-start injection only; offline evals of skills | None | 293k; 6 releases in 90 days |
| Kiro | Hardcoded 3-phase spec types | Steering inclusion modes, custom agents, precedence agent > workspace > global | Powers (Agent Plugins format) | Hooks, 3 blocking triggers, no blocking stop; headless CLI for CI | GitHub label/comment assigns issues to Kiro Web | proprietary; several releases per week |
| GSD Core | Fixed 5-step loop + **12 typed extension points** | `.planning/config.json` (~40 toggles, modes, model profiles), setting combinations | Capability manifests (local/user dirs, registry allowlist, re-consent) | `gsd-tools` validators with coded diagnostics, blocking decision-coverage gate, guard hooks | Local files; tracker "documentation only" | 10k (+64k archived); 17 releases in 90 days, 487 commits/30 d |
| Ruflo | No dev pipeline; hardcoded workflow templates, SPARC plugin | Init component flags and presets | 33 Claude Code plugins | Non-blocking hooks; the one block uses exit 1 | Local claims; GitHub sync stub | 73k; ≥100 releases in 90 days |
| Agent OS | 5 hardcoded commands (v3 dropped spec/tasks/orchestration) | **Profiles with inheritance**, push-back sync | None | None | None | 5.5k; no release since 2026-01-20 |

---

## 10. Also seen (not analysed in depth)

- **Conductor** (`gemini-cli-extensions/conductor`, 3,750 stars, Apache-2.0, `VERSION` 0.3.0, commit
  `6e8f9a8`): one skills tree packaged as a plugin (`plugin.json`, `.claude-plugin/marketplace.json`),
  installed with `agy plugins install …` for Antigravity or `/plugin marketplace add …` for Claude
  Code, or symlinked into a workspace-level `.agents/plugins/conductor`; a host "UX adapter" rule
  (`rules/conductor_antigravity.md`) tells the agent to use the native `ask_question` modal when the
  tool exists and fall back to numbered text choices otherwise
  (https://github.com/gemini-cli-extensions/conductor/blob/6e8f9a8/README.md). Its artifacts are
  `conductor/product.md`, `tech-stack.md`, `workflow.md`, `tracks/<id>/{spec,plan}.md`,
  `metadata.json` (same README).
- **vercel-labs/skills** (32,730 stars, MIT): the cross-agent skill installer BMAD now depends on;
  project or `-g` global scope, symlink to one canonical copy or copy, `npx skills update`, "75 more"
  agents beyond OpenCode, Claude Code, Codex, Cursor (https://github.com/vercel-labs/skills , README
  at `3694740`).
- **spec-kitty** (1,650 stars, MIT) — "Kanban dashboard, git worktrees, auto-merge" per the repo
  description; **Pimzino/spec-workflow-mcp** (4,297 stars, GPL-3.0) — an MCP server with a web
  dashboard and VS Code extension per its description. Both only from `gh api` descriptions;
  unconfirmed beyond that.

---

## 11. Implications for agent-workbench

### 11.1 Patterns to adopt

1. **Hash manifest of every managed file**, with a clear policy on a modified file. spec-kit refuses
   and asks for `--force` (§1.1); GSD keeps a pristine copy and offers a three-way reapply (§6.1);
   BMAD v6.12 overwrote and saved `.bak` (§3.1). Combined with the next item, this answers "how
   local edits survive updates".
2. **Customisation lives in a separate layer that updates never touch.** BMAD `_bmad/custom/*.toml`
   with team and `.user` (gitignored) files and keyed merge (§3.3); spec-kit
   `.specify/templates/overrides/` and workflow overlays with named slots (§1.3). This is safer than
   merging into managed files and matches our "team" requirement (decisions, 2026-09-29).
3. **Runtime adapters as data, one source corpus.** GSD `capability.json` per runtime with layout,
   converter, hook surface and `unsupportedFeatures` (§6.2); BMAD's `platform-codes.yaml` (§3.2);
   spec-kit's three-attribute integration classes (§1.2). Degradation is explicit per runtime
   instead of silent.
4. **Canonical hook events rendered into each runtime's native hook config**, including how each
   runtime feeds hook output back (spec-kit §1.2). This is the only framework that turns one gate
   into Claude, Codex, Cursor, Copilot and OpenCode hooks.
5. **Pipeline as declared data with dependency edges.** OpenSpec's artifact DAG with `requires` and
   `status --json` (§2.3–2.4); GSD's closed set of typed extension points with `blocking` gates and
   build-time checks (§6.3); spec-kit's workflow engine with persisted `state.json` and `resume`
   (§1.3). Our brief's "stage = input, artifact, mechanical check, judgment check, role, exit
   condition" maps naturally onto OpenSpec-style artifact nodes plus GSD-style typed gates.
6. **An agent-facing CLI contract**: JSON on stdout, one diagnostic envelope
   (`{severity, code, message, target, fix}` in OpenSpec §2.4, `{code, message, fix, repairable}` in
   GSD §6.4), human-only vs agent-safe commands, and instructions served at runtime rather than
   frozen into generated files (`openspec instructions --json`).
7. **Layered configuration with explicit precedence**: defaults → committed team file → gitignored
   local file → environment (spec-kit extension config §1.3, BMAD §3.3), with a command that prints
   the resolved value (BMAD `resolve_customization.py`).
8. **Profiles with inheritance** for project kinds, with cycle detection and child-wins overlay, and
   a reverse "promote to profile" flow (Agent OS §8.3).
9. **Rename and retirement bookkeeping** for skills: `renamed`/`removed` lists and "a retired name is
   never reused" (BMAD `bmod.toml`, §3.1).
10. **Extension trust model**: community catalog discovery-only (spec-kit §1.3), host allowlist and
    re-consent when executable surfaces change, `engines` ranges (GSD §6.3).
11. **Porting acceptance test per runtime**: Superpowers declares a runtime supported only if it can
    inject context automatically at session start and skills "name actions, not tools" with a
    per-runtime tool map (§4.2). Its cross-runtime behavioural evals are the only direct test that
    a workflow actually fires in each runtime (§4.4).

### 11.2 Anti-patterns

1. **"Hooks" that the model is asked to run.** spec-kit's extension hooks are prose instructions
   ("Emitting the block alone does not run the hook") with unevaluated conditions (§1.4). Anything
   called a gate must be executed by code.
2. **Untested blocking semantics.** Ruflo's only block exits 1, which Claude Code does not treat as
   blocking (§7.3); Kiro has no blocking stop trigger at all (§5.4). Hook rendering needs per-runtime
   conformance tests, not just generation.
3. **Two install paths with different surfaces.** GSD's plugin path has a different namespace and
   ignores install-time config (§6.1); Ruflo's plugin path has no hooks while the CLI installs
   everything (§7.1); BMAD's migration left docs describing both (§3). One source of truth for "what
   is installed" is needed even if several channels exist.
4. **Overwrite-on-update or manual re-clone.** OpenSpec overwrites by design (§2.1), Ruflo
   force-overwrites helpers (§7.1), Agent OS asks for `rm -rf` and manual backups (§8.1).
5. **Asymmetric multi-runtime.** spec-kit materialises extensions and presets only for the default
   integration (§1.2), so a team with Claude and Codex users gets different behaviour.
6. **Hardcoding stages that runtimes later absorb.** Agent OS v3 dropped its spec, task and
   orchestration commands because plan modes and todo lists absorbed them (§8.3); runtime churn also
   removed Gemini CLI from GSD (§6.2). Keep durable parts (artifact contract, checks, tracker facts)
   separate from runtime-specific orchestration.
7. **Mandatory workflow enforced only by prompt pressure** (Superpowers "ABSOLUTELY MUST", §4.4) and
   **no configurability** at all; works for one opinionated method, not for "profiles for different
   kinds of projects".
8. **Opt-out telemetry** (OpenSpec §2.6) and email-gated install docs (Agent OS §8.1) — both
   friction points for a public, team-oriented tool.
9. **Installer mass.** GSD's installer is ~15k lines and each workflow probes ~16 config homes
   (§6.1–6.2); a compiled single binary with an explicit per-project state file can avoid runtime
   discovery in every prompt.

### 11.3 Gaps none of them fill

1. **Stage derived from tracker and PR facts.** No framework reads stage from issues, PRs, checks and
   owner decisions. BMAD mirrors a `tracker_status` and syncs only when a skill runs (§3.5);
   spec-kit pushes tasks to issues one-way through the LLM (§1.5); GSD's tracker support is
   "documentation only" (§6.5); OpenSpec, Superpowers, Agent OS have none; Kiro's label-assign flow is
   closest but proprietary and single-runtime (§5.5). This is exactly our brief's principle "a stage
   is derived from facts".
2. **A CI template for consumer projects.** None of the eight ships one; the checks exist as CLI
   commands (OpenSpec, GSD, spec-kit `integration status`) or community extensions (spec-kit
   `ci-guard`, `gates`). Our "checked by a command, later by hooks and CI" needs a first-party CI
   entry point.
3. **Profiles that bundle stages and checks, not just standards.** Agent OS profiles carry standards
   only (§8.3); GSD has setting combinations but no project-kind presets (§6.3); OpenSpec has
   workflow-selection profiles stored globally (§2.3); spec-kit bundles pin a stack of extensions
   (§1.3) — the closest, but not a project-kind profile.
4. **A project-level lock of the harness version itself.** Pinning is per machine (uv tag, npm
   version, host plugin manager); version ranges exist only for extensions (`requires.speckit_version`
   §1.3, `engines.gsd` §6.3). A team cannot declare "this repo uses harness X.Y" and have every
   member's agent fail fast on skew (not found in any of the eight; unconfirmed absence for Kiro).
5. **Cross-runtime conformance proof.** Only Superpowers tests behaviour in real runtimes, and only
   offline (§4.4); no framework checks that a rendered hook actually blocks in each runtime.
6. **Owner decisions as first-class facts.** spec-kit gate verdicts live in local run state (§1.4);
   GSD confirmation gates vanish in `yolo` mode (§6.4); none records an approval where the tracker
   or PR can see it.

### 11.4 Open questions for the grilling session

1. **Distribution channel.** Own Rust binary that renders per runtime (spec-kit, GSD, OpenSpec), native
   plugin marketplaces only (Superpowers), a generic skills installer (BMAD via `vercel-labs/skills`),
   or a binary plus plugins — and how to avoid the two-path trap (§11.2.3)?
2. **What is committed to the project repo?** Generated per-runtime files (spec-kit, OpenSpec), nothing
   (Superpowers), or only config plus a lock with generation on each machine? This decides how a team
   member with a different runtime gets the same pipeline (§11.2.5).
3. **Edit survival policy.** Never edit managed files and use override layers only (BMAD main), refuse
   on modified files (spec-kit), or three-way reapply (GSD)?
4. **Pipeline expressiveness.** Artifact DAG (OpenSpec), fixed loop with typed extension points (GSD),
   or a workflow engine with control flow (spec-kit)? How much of the stage definition is data versus
   skill prose?
5. **What is a profile?** Stages, checks, skills, standards, tracker settings? Is inheritance needed
   (Agent OS), and where do profiles live (machine, repo, a shared repo)?
6. **Where gates execute.** CLI validators called by skills, native agent hooks (blocking semantics
   differ per runtime, §5.4, §7.3), or CI as the only universal gate? The 2026-09-27 decision puts
   hooks after the artifact contract; does CI come before hooks?
7. **Runtime tiers and acceptance test.** Which runtimes are tier 1 (GSD tiers, §6.2), and what proves
   support — session-start injection (Superpowers), blocking hook conformance, or an eval run?
8. **Tracker model.** GitHub only first, or a store adapter from day one (BMAD's store choice, §3.5)?
   Where exactly do stage facts and owner decisions live (§11.3.1, §11.3.6)?
9. **Extensions and trust.** Is there a community catalog at all? If yes, discovery-only by default
   and re-consent when executable surfaces change (spec-kit, GSD)?
10. **Coexistence with other methods.** Should agent-workbench host spec-kit, OpenSpec or BMAD
    artifacts as a profile (spec-kit's GitHub description: "Toolkit to help you get started with SDD or any other process"), or define
    its own artifact contract only?
11. **Version skew in a team.** Is a repo-level harness lock with fail-fast checks required
    (§11.3.4)?
12. **Telemetry.** None, or opt-in only (§11.2.8)?
