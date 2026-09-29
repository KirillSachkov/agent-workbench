# Cross-runtime portability: what a universal harness can target today (2026-09-29)

Question: which parts of a development harness (instructions, skills, plugins, hooks, subagents,
tools, CI runs) can be written once and work across agent runtimes, and which need per-runtime
adapters. Input for the requirements grilling of #1 (harness and pipeline).

Method: official docs, specifications and vendor source repositories only, read on 2026-09-29
(WebFetch and `gh api`). Every row carries its source. Anything not confirmed from a primary source
is marked "unconfirmed". Local versions checked with `--version`/`--help`: Claude Code 2.1.284,
codex-cli 0.158.0, opencode 1.18.30, GitHub Copilot CLI 0.0.400 (the latest release is v1.0.89 from
2026-09-28, https://github.com/github/copilot-cli/releases, so the local copy is old), Kimi Code
CLI 0.43.0. Gemini CLI and Cursor CLI are not installed and were checked from docs only.

Runtimes covered: Claude Code (CC), OpenAI Codex CLI, Gemini CLI, OpenCode, Cursor (IDE and CLI),
GitHub Copilot (CLI, cloud agent, VS Code), Kimi Code CLI. Goose, Amp, Windsurf/Devin Desktop and Zed
appear where they were quick to confirm.

Two naming notes:
- Codex docs moved: `developers.openai.com/codex/...` now redirects (308) to `learn.chatgpt.com/docs/...`.
  Links below use the target.
- "Kimi" here means **Kimi Code CLI** (TypeScript, https://github.com/MoonshotAI/kimi-code). The older
  Python `kimi-cli` (https://github.com/MoonshotAI/kimi-cli) is archived. The two have different paths
  (`.kimi-code/` against `.kimi/`).

---

## 1. Summary

1. **AGENTS.md is the de facto instruction file.** Codex, OpenCode, Cursor, Copilot, Kimi, Goose,
   Amp, Windsurf and Zed read it natively. Claude Code has read it natively since v2.1.277, but by
   default **only when no `CLAUDE.md` exists** in the working directory or above. Gemini CLI reads it
   only after `context.fileName` is configured. The standard is governed by the Agentic AI Foundation
   (Linux Foundation) and has no formal spec beyond the site.
2. **The Agent Skills format is portable. The discovery path is not quite.** Every runtime checked
   reads `SKILL.md` directories. `.agents/skills` is read by Codex, Gemini, OpenCode, Cursor, Copilot,
   Kimi, Goose, Amp, Windsurf and Zed, but **not by Claude Code**, which reads only `.claude/skills`.
   Only the six spec frontmatter fields are portable. Invocation policy (model-invoked against
   user-only) uses different fields per runtime.
3. **The Claude plugin format has become a de facto interchange format.** Codex and Copilot CLI
   discover `.claude-plugin/plugin.json` and `marketplace.json`. A neutral **Agent Plugins 1.0.0**
   spec (skills and MCP only; maintainers from Amazon, Cursor, Microsoft, OpenAI and Vercel) is read
   by Codex, Cursor and Copilot CLI. Claude Code does not mention it. Gemini extensions, OpenCode
   plugins and Kimi plugins are separate formats.
4. **Hooks exist in all seven runtimes, and the Claude Code hook contract is the de facto reference.**
   That contract is: a shell command, JSON on stdin, exit 2 to block, and a JSON decision on stdout.
   - Cursor and Copilot CLI read `.claude/settings.json` hooks directly.
   - Codex (`/import`) and Gemini (`gemini hooks migrate --from-claude`) convert them.
   - Kimi mirrors the event names, but its config is user-level TOML only.
   - OpenCode has in-process JS/TS plugin functions and no shell hook contract.
   - Four events are common to every shell-hook runtime: session start, blocking pre-tool, post-tool,
     and stop with forced continuation.
5. **The MCP protocol is portable; MCP configuration is not.** Only Claude Code, Copilot CLI and
   VS Code read a project `.mcp.json` with the `mcpServers` key. Every other runtime needs a generated
   file in its own shape: TOML, `mcp`, `servers`, arrays or a different env syntax. The spec revision
   is 2026-07-28 (stateless).
6. **Headless runs are universal.** Every runtime has `-p`/`exec`/`run` with JSON or JSONL output.
   Only `claude --json-schema` and `codex exec --output-schema` enforce a schema on the final answer.
   Four runtimes have an official GitHub Action. ACP (Agent Client Protocol) is spoken natively by
   Gemini, Copilot, OpenCode, Kimi and Cursor. Claude Code and Codex speak it through adapters.

---

## 2. Project instructions: AGENTS.md

### 2.1 The standard

- Plain Markdown with no required fields. The site's rule is "The closest AGENTS.md to the edited
  file wins; explicit user chat prompts override everything". It defines no size limit and no
  fallback names. https://agents.md
- Stewardship: "AGENTS.md is now stewarded by the Agentic AI Foundation under the Linux Foundation"
  (https://agents.md). AAIF was founded in December 2025 with MCP, goose and AGENTS.md
  (https://www.linuxfoundation.org/press/linux-foundation-announces-the-formation-of-the-agentic-ai-foundation).
  Its current projects: https://aaif.io/projects/.
- Repository: `agentsmd/agents.md` (MIT) holds the README and the site only. There is no normative
  spec for nesting. https://github.com/agentsmd/agents.md

In practice "closest wins" is implemented three different ways: concatenate from root to cwd (with
the closest file last), one file per directory, or first file found only.

### 2.2 Per runtime

| Runtime | Reads AGENTS.md | Nesting and merge | Other names, limits | Source |
|---|---|---|---|---|
| Claude Code | **Partial.** Native since v2.1.277. The default mode `claude-md-or-agents-md` reads `AGENTS.md` and `.claude/AGENTS.md` only if no `CLAUDE.md`, `.claude/CLAUDE.md` or `CLAUDE.local.md` exists in cwd or above. Other modes: `claude-md-and-agents-md`, `claude-md`, `managed-only`. The mode is settable only in user or managed settings (`pluginConfigs["agents-md@builtin"]`) and is ignored in project settings | Root-to-cwd concatenation; subdirectories load lazily on Read; `@path` imports up to 4 hops | Does not read `AGENTS.local.md`, `AGENTS.override.md` or anything under `.agents/`. Files over 4 MiB are skipped, under 200 lines is recommended. The `@AGENTS.md` import in `CLAUDE.md` remains the documented bridge and never double-loads | https://code.claude.com/docs/en/memory |
| Codex | Yes | Global `~/.codex/AGENTS.override.md` or `AGENTS.md`. Project: from the git root down to cwd, at most one file per directory (`AGENTS.override.md` → `AGENTS.md` → fallback names), concatenated with the closer file later | `project_doc_max_bytes = 32768` (total), `project_doc_fallback_filenames` (for example `CLAUDE.md`), `project_root_markers` | https://learn.chatgpt.com/docs/agent-configuration/agents-md ; https://github.com/openai/codex/blob/main/codex-rs/config/defaults.toml |
| Gemini CLI | **Config needed.** The default is `GEMINI.md`; `"context": {"fileName": ["AGENTS.md", ...]}` adds it | All concatenated: global, then cwd and parents up to `.git` or home, then subdirectories (up to 200), then JIT on tool access; `@file.md` imports | `/memory show`, `/memory reload` | https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/gemini-md.md ; https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/configuration.md |
| OpenCode | Yes (primary) | The first name found from cwd up to the worktree wins (`AGENTS.md` → `CLAUDE.md` → legacy `CONTEXT.md`); names are not mixed. Subdirectory files attach on Read | Global `~/.config/opencode/AGENTS.md`, falling back to `~/.claude/CLAUDE.md`. `instructions` globs and URLs in `opencode.json`. `OPENCODE_DISABLE_CLAUDE_CODE*` turns the Claude fallbacks off | https://opencode.ai/docs/rules/ |
| Cursor | Yes (root and nested; the CLI also reads `CLAUDE.md`) | "combined with parent directories, with more specific instructions taking precedence" | `.cursor/rules/*.mdc` with `description`, `globs`, `alwaysApply` | https://cursor.com/docs/context/rules ; https://cursor.com/docs/cli/using |
| Copilot | Yes: cloud agent, CLI, VS Code, JetBrains. Not GitHub.com chat or Visual Studio | Cloud agent: nearest file wins. CLI: concatenates root, cwd and path directories with no defined priority | Also `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`, `.github/instructions/**/*.instructions.md` (`applyTo`). VS Code: `chat.useAgentsMdFile`; nested files behind `chat.useNestedAgentsMdFiles` (experimental, off) | https://docs.github.com/en/copilot/reference/custom-instructions-support ; https://code.visualstudio.com/docs/copilot/customization/custom-instructions |
| Kimi Code | Yes | Project `AGENTS.md` and `.kimi-code/AGENTS.md`, global `~/.kimi-code/AGENTS.md`, generic `~/.agents/AGENTS.md`. Nesting rules unconfirmed | Content injected as `${agents_md}` into the system prompt template | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/agents.md |
| Goose | Yes (`CONTEXT_FILE_NAMES` defaults to `AGENTS.md`, `.goosehints`) | cwd to repo root, nested on access, concatenated | — | https://github.com/aaif-goose/goose/blob/main/documentation/docs/guides/context-engineering/using-goosehints.md |
| Amp | Yes | cwd and parents to `$HOME`; subtree on read | Falls back to `AGENT.md`, `CLAUDE.md` | https://ampcode.com/docs/customize/agents-md |
| Windsurf / Devin Desktop | Yes | Root file is an always-on rule; a subdirectory file becomes a `<dir>/**` glob rule | — | https://docs.devin.ai/desktop/cascade/agents-md |
| Zed | Yes, but **first match only** from a fixed list (`.rules` … `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`) | No merge | — | https://github.com/zed-industries/zed/blob/main/docs/src/ai/instructions.md |

---

## 3. Agent Skills

### 3.1 The spec

Source: https://agentskills.io/specification , repo https://github.com/agentskills/agentskills

- A skill is a directory with `SKILL.md` (YAML frontmatter plus Markdown). Optional directories:
  `scripts/`, `references/`, `assets/`.
- Frontmatter fields:

  | Field | Required | Constraints |
  |---|---|---|
  | `name` | yes | 1–64 chars, `a-z0-9-`, no leading, trailing or double hyphen, must equal the directory name |
  | `description` | yes | 1–1024 chars; what the skill does and when to use it |
  | `license` | no | — |
  | `compatibility` | no | ≤500 chars |
  | `metadata` | no | string→string map |
  | `allowed-tools` | no | **experimental**, space-separated; "support … may vary" |

- Progressive disclosure has three levels: metadata (~100 tokens) at startup, the body on activation
  (<5000 tokens, <500 lines recommended), and resources on demand.
- Validator: `skills-ref validate` ("for demonstration purposes only", README of
  https://github.com/agentskills/agentskills/tree/main/skills-ref).
- Governance: "originally developed by Anthropic, released as an open standard". Changes go through
  GitHub Discussions. It is **not** an AAIF project as of this date (https://agentskills.io/home ;
  https://aaif.io/projects/).
- The spec defines **no discovery paths**. The client-implementation guide calls `.agents/skills/`
  "a widely-adopted convention", notes that some clients also scan `.claude/skills/`, says project
  overrides user, and recommends trust gating and a `disable-model-invocation` opt-out:
  https://github.com/agentskills/agentskills/blob/main/docs/client-implementation/adding-skills-support.mdx

### 3.2 Discovery per runtime

| Runtime | Project paths | User paths | `.agents/skills` | `.claude/skills` | Source |
|---|---|---|---|---|---|
| Claude Code | `.claude/skills` in cwd and parents up to the repo root; nested `<dir>/.claude/skills` (lazy); `--add-dir`; plugins | `~/.claude/skills` | **no** (docs state that nothing under `.agents/` is read for instructions; skills docs list only `.claude/skills`) | yes | https://code.claude.com/docs/en/skills |
| Codex | `.agents/skills` in each directory from the project root to cwd; `.codex/skills` (from source) | `~/.agents/skills`; legacy `$CODEX_HOME/skills`; admin `/etc/codex/skills` | yes | no (not in docs) | https://learn.chatgpt.com/docs/build-skills ; https://github.com/openai/codex/blob/main/codex-rs/ext/skills/src/host_roots.rs |
| Gemini CLI | `.gemini/skills`, alias `.agents/skills` (the alias wins within a tier) | `~/.gemini/skills`, `~/.agents/skills` | yes | no | https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/skills.md |
| OpenCode | `.opencode/skills`, `.claude/skills`, `.agents/skills` from cwd up to the worktree; `skills.paths` | `~/.config/opencode/skills`, `~/.claude/skills`, `~/.agents/skills` | yes | yes | https://opencode.ai/docs/skills/ |
| Cursor | `.agents/skills`, `.cursor/skills`, plus `.claude/skills` and `.codex/skills` for compatibility | the same under `~/` | yes | yes | https://cursor.com/docs/context/skills |
| Copilot | `.github/skills`, `.claude/skills`, `.agents/skills` | `~/.copilot/skills`, `~/.agents/skills` | yes | yes | https://docs.github.com/en/copilot/concepts/agents/about-agent-skills |
| Kimi Code | `.kimi-code/skills`, `.agents/skills` (project root = nearest `.git`); `extra_skill_dirs` | `~/.kimi-code/skills`, `~/.agents/skills` | yes | no (not in docs; the legacy kimi-cli did read it) | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/skills.md |
| Goose | `.agents/skills`, `.goose/skills`, `.claude/skills` | `~/.agents/skills`, `~/.claude/skills` | yes | yes | https://github.com/aaif-goose/goose/blob/main/documentation/docs/guides/context-engineering/using-skills.md |
| Amp | `.agents/skills`, `.claude/skills` | `~/.config/agents/skills`, `~/.agents/skills`, `~/.claude/skills` and others | yes | yes | https://ampcode.com/docs/customize/skills |
| Windsurf / Devin Desktop | `.devin/skills`, `.agents/skills`, `.claude/skills` | several, including `~/.agents/skills` | yes | yes | https://docs.devin.ai/desktop/cascade/skills |
| Zed | `.agents/skills` only | `~/.agents/skills` only | yes | no | https://github.com/zed-industries/zed/blob/main/docs/src/ai/skills.md (docs on `main`; stable release unconfirmed) |

### 3.3 Frontmatter extensions and invocation policy

| Runtime | Beyond the spec | Hide from the model (user-only) | Hide from the user (model-only) | Explicit call | Source |
|---|---|---|---|---|---|
| Claude Code | `when_to_use`, `argument-hint`, `arguments`, `allowed-tools`, `disallowed-tools`, `model`, `effort`, `context: fork`, `agent`, `hooks`, `paths`, `shell`, … Spec fields `license`, `compatibility`, `metadata` are accepted but ignored | `disable-model-invocation: true` | `user-invocable: false` | `/name` | https://code.claude.com/docs/en/skills |
| Codex | Reads only `name`, `description`, `metadata.short-description`; everything else is ignored. The sidecar `agents/openai.yaml` holds `interface.*`, `policy.allow_implicit_invocation`, `policy.products`, `dependencies.tools[]` (MCP only) | `policy.allow_implicit_invocation: false` in `agents/openai.yaml` | no | `$name` | https://learn.chatgpt.com/docs/build-skills ; https://github.com/openai/codex/blob/main/codex-rs/skills/src/assets/samples/skill-creator/references/openai_yaml.md |
| Gemini CLI | Only `name` and `description` are documented; activation through the `activate_skill` tool with user confirmation | `/skills disable` (not a frontmatter field) | no | no per-skill slash command documented | https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/skills.md |
| OpenCode | Spec fields; unknown fields ignored; skills also registered as commands (from source) | `permission` deny/ask on the `skill` tool | no | `/name` (from source) | https://opencode.ai/docs/skills/ |
| Cursor | `paths`, `disable-model-invocation`, `icon`, `color`, `metadata` | `disable-model-invocation` | no | `/name` | https://cursor.com/docs/context/skills |
| Copilot | CLI: `license`, `allowed-tools`. VS Code: `argument-hint`, `user-invocable`, `disable-model-invocation` | VS Code: `disable-model-invocation`; CLI unconfirmed | VS Code: `user-invocable: false` | `/name` | https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/create-skills ; https://code.visualstudio.com/docs/copilot/customization/agent-skills |
| Kimi Code | `type` (`prompt`/`inline`/`flow`), `whenToUse`, `arguments`; flat `.md` skills allowed | `disableModelInvocation` (kebab-case and snake_case also accepted) or `type: flow` | no | `/skill:name` | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/skills.md |

The only cross-runtime invocation field is `disable-model-invocation`. Claude Code, Cursor, VS Code,
Zed (https://github.com/zed-industries/zed/blob/main/docs/src/ai/skills.md) and Kimi read it. Codex
needs a generated `agents/openai.yaml`. Slash syntax differs: `/name`, `$name`, `/skill:name`, `@name`.

---

## 4. Plugins and extensions

### 4.1 Formats

| Runtime | Manifest | Marketplace | What a plugin can bundle | Source |
|---|---|---|---|---|
| Claude Code | `.claude-plugin/plugin.json` (optional; only `name` is required) | `.claude-plugin/marketplace.json` | skills, commands, agents, `hooks/hooks.json`, `.mcp.json`, `.lsp.json`, output styles, monitors and themes (experimental), `bin/`, settings (`agent` and `subagentStatusLine` only), `userConfig`, `dependencies` | https://code.claude.com/docs/en/plugins/manifest-reference ; https://code.claude.com/docs/en/plugins/marketplace-reference |
| Codex | Search order: root `plugin.json` with the Agent Plugins `$schema`, then `.codex-plugin/`, then **`.claude-plugin/`**, then `.cursor-plugin/` | `.agents/plugins/marketplace.json`, **`.claude-plugin/marketplace.json`**, `.cursor-plugin/marketplace.json` | skills, `mcpServers`, apps, hooks, `interface`, commands (converted to skills on install). The parser has no `agents` or LSP fields | https://github.com/openai/codex/blob/main/codex-rs/utils/plugins/src/plugin_namespace.rs ; https://github.com/openai/codex/blob/main/codex-rs/core-plugins/src/marketplace.rs ; https://developers.openai.com/plugins/build/plugins.md |
| Gemini CLI | `gemini-extension.json` (`mcpServers`, `contextFileName`, `excludeTools`, `settings`, …) | none (gallery at geminicli.com/extensions) | `commands/*.toml`, `hooks/hooks.json`, `skills/`, `agents/*.md` (preview), `policies/*.toml`, themes | https://github.com/google-gemini/gemini-cli/blob/main/docs/extensions/reference.md |
| OpenCode | none: a JS/TS module, local (`.opencode/plugins/`) or an npm name in `opencode.json` `"plugin"` | none | event hooks and custom tools (Zod). Bundling agents, commands, skills or MCP is not documented | https://opencode.ai/docs/plugins/ |
| Cursor | `.cursor-plugin/plugin.json` or an Agent Plugins root `plugin.json` | `.cursor-plugin/marketplace.json`; team marketplaces | rules, skills, agents, commands, hooks, `mcp.json` | https://cursor.com/docs/plugins ; https://cursor.com/docs/plugins/building |
| Copilot CLI | An Agent Plugins root `plugin.json` has priority; legacy search `.plugin/`, `plugin.json`, `.github/plugin/`, **`.claude-plugin/`** | the same directories, including **`.claude-plugin/`** | agents (`.agent.md`), skills, commands, hooks, `mcpServers`, `lspServers`, extensions. `${CLAUDE_PLUGIN_ROOT}` and `CLAUDE_PLUGIN_DATA` are honored as aliases | https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-plugin-reference |
| Kimi Code | `kimi.plugin.json` or `.kimi-plugin/plugin.json` | its own JSON (`plugins[]` with `id` and `source`) | skills, agents, commands, hooks, `mcpServers`, system prompt, `sessionStart.skill` | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/plugins.md |

**Agent Plugins 1.0.0** (1.1.0 is a draft). A root `plugin.json` with `$schema` and `name`, a
`skills/` directory and `mcp.json`. It covers only skills and MCP; agents, hooks, commands and rules
are left to vendor `extensions`. The Technical Steering Committee comes from Amazon, Cursor,
Microsoft, OpenAI and Vercel; Anthropic and Google are not listed.
https://github.com/agentplugins/agent-plugins-spec ;
https://github.com/agentplugins/agent-plugins-spec/blob/main/MAINTAINERS.md ; https://agent-plugins.org

### 4.2 Install, update and team enablement

| Runtime | Install and update | A repository can declare plugins for the team | Source |
|---|---|---|---|
| Claude Code | `claude plugin install name@mkt --scope user\|project\|local`, `marketplace add`, `update`, `validate`. Version comes from `version`, else from the marketplace entry, else the commit SHA. Auto-update is on only for official marketplaces | **Partial.** `.claude/settings.json` accepts `extraKnownMarketplaces` (after workspace trust) and `enabledPlugins`. A plugin from an external source is **not auto-installed**: the user sees "enabled in project settings but isn't installed". Relative-path plugins from the marketplace load directly | https://code.claude.com/docs/en/plugins/loading ; https://code.claude.com/docs/en/plugins/org |
| Codex | `codex plugin add PLUGIN@MKT`, `codex plugin marketplace add/upgrade`. Entry policy `INSTALLED_BY_DEFAULT` exists | `.codex/config.toml`: `[plugins."name@mkt"] enabled = true` | https://developers.openai.com/plugins/build/plugins.md |
| Gemini CLI | `gemini extensions install <git-url\|path> [--auto-update]`, `update`, `link` | not documented | https://github.com/google-gemini/gemini-cli/blob/main/docs/extensions/reference.md |
| OpenCode | npm plugins are auto-installed through Bun at startup | **Yes:** `opencode.json` `"plugin"` | https://opencode.ai/docs/plugins/ |
| Cursor | Marketplace UI, team marketplaces | team marketplaces; repository-level declaration unconfirmed | https://cursor.com/docs/plugins |
| Copilot CLI | `copilot plugin install plugin@mkt\|OWNER/REPO\|URL\|path`, `update`, `marketplace add` | **Yes:** `.github/copilot/settings.json` `enabledPlugins` and `extraKnownMarketplaces` ("Declarative plugin auto-install"). It **also reads `.claude/settings.json`** | https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference |
| Kimi Code | `/plugins install <path\|zip\|github>` | **No:** per-user only | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/plugins.md |

---

## 5. Hooks

The Claude Code event list and payload were covered in `2026-09-27-local-session-artifacts.md`. Here
the question is parity.

### 5.1 Contract per runtime

| Runtime | Config (project level) | Handlers | Transport | Block and decision | Maturity | Source |
|---|---|---|---|---|---|---|
| Claude Code | `.claude/settings.json`, `.claude/settings.local.json`, plugin `hooks/hooks.json`, skill and agent frontmatter; user and managed levels | `command`, `http`, `mcp_tool`, `prompt`, `agent` (experimental) | stdin JSON: `session_id`, `transcript_path`, `cwd`, `permission_mode`, `hook_event_name`, `tool_name`, `tool_input` | exit 2 blocks. JSON: `continue`, `decision:"block"`, `hookSpecificOutput.permissionDecision` (allow/deny/ask/defer), `updatedInput`, `additionalContext`. Stop and SubagentStop block, which forces the agent to continue | stable, 33 events | https://code.claude.com/docs/en/hooks |
| Codex | `.codex/hooks.json` or `[hooks]` in `.codex/config.toml`; plugin `hooks/hooks.json`; user `~/.codex/` | `command`, `mcp_tool` (`prompt` and `agent` are parsed but skipped) | stdin JSON: `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `model`, `permission_mode`, `turn_id`, `tool_name`, `tool_input`, `tool_use_id`. JSON schemas published per event | exit 2 blocks; `decision`/`permissionDecision`, `updatedInput`, `additionalContext`. Stop `decision:"block"` forces continuation. PreToolUse `ask` is not supported. PreToolUse covers shell, `apply_patch`, MCP and function tools, but not hosted tools | `hooks` is `Stage::Stable`, on by default (`codex features list` on 0.158.0: `hooks stable true`) | https://learn.chatgpt.com/docs/hooks ; https://github.com/openai/codex/tree/main/codex-rs/hooks/schema/generated ; https://github.com/openai/codex/blob/main/codex-rs/features/src/lib.rs |
| Gemini CLI | `hooks` in `.gemini/settings.json`; user, system and extension levels | `command` | stdin JSON: `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `timestamp`, tool fields. `CLAUDE_PROJECT_DIR` is exported for compatibility | exit 2 blocks; `decision` allow/deny, `hookSpecificOutput.tool_input` rewrites arguments; AfterAgent `deny` forces a retry. Also BeforeModel, AfterModel and BeforeToolSelection | on by default since v0.27.0 | https://geminicli.com/docs/hooks/reference/ ; https://github.com/google-gemini/gemini-cli/releases/tag/v0.27.0 |
| Cursor | `.cursor/hooks.json` (`"version": 1`); user, team (dashboard) and enterprise levels | `command`, `prompt` | stdin JSON: `conversation_id`, `generation_id`, `model`, `hook_event_name`, `workspace_roots`, `transcript_path?` | exit 2 denies; other non-zero codes fail open; `permission`, `updated_input`, `followup_message` (stop and subagentStop, `loop_limit` 5). Separate shell, MCP and read-file events | no beta label; CLI parity unconfirmed | https://cursor.com/docs/agent/hooks |
| OpenCode | `.opencode/plugins/*.{js,ts}` or npm plugins in `opencode.json` | in-process JS/TS functions | function arguments `(input, output)` | `throw` in `tool.execute.before` blocks; mutate `output.args`; `permission.ask` sets a status. **No shell contract, no forced continuation on idle** | stable API, some `experimental.*` hooks | https://opencode.ai/docs/plugins/ ; https://github.com/anomalyco/opencode/blob/dev/packages/plugin/src/index.ts |
| Copilot | CLI: `.github/hooks/*.json`, `.github/copilot/settings(.local).json`, **`.claude/settings(.local).json`**, user, policy and plugin levels. The cloud agent reads only `.github/hooks/*.json` | `command` (`bash`/`powershell`), `http`, `prompt` (CLI only) | stdin JSON. camelCase events use camelCase fields; PascalCase (Claude-style) events use snake_case fields | preToolUse `permissionDecision` plus `modifiedArgs`; agentStop and subagentStop `decision:"block"` force continuation. exit 2 denies only for preToolUse and permissionRequest. preToolUse fails closed, other events fail open | hooks since CLI 0.0.396; agentStop and subagentStop since 0.0.401 (the local 0.0.400 predates them) | https://docs.github.com/en/copilot/reference/hooks-configuration ; https://github.com/github/copilot-cli/blob/main/changelog.md |
| VS Code (Copilot) | `.github/hooks/*.json`, `~/.copilot/hooks/`; Claude settings files when `chat.useClaudeHooks` is on (**matchers ignored**) | `command` | stdin JSON | 8 events | requires `chat.useHooks` and a trusted workspace | https://code.visualstudio.com/docs/copilot/customization/hooks |
| Kimi Code | **user level only**: `[[hooks]]` in `~/.kimi-code/config.toml` with the fields `event`, `matcher`, `command`, `timeout`. A project level is not documented | shell command | stdin JSON (snake_case) | exit 2 or `permissionDecision:"deny"`. Only PreToolUse, Stop and UserPromptSubmit can block; the rest observe only | — | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/hooks.md |

### 5.2 Who accepts the Claude Code hook format

| Runtime | Mechanism | Source |
|---|---|---|
| Cursor | **Reads at runtime.** `.claude/settings(.local).json` and `~/.claude/settings.json` are read when "Include Third-Party Plugins, Skills, and Other Configs" is on (the default). Events and decisions are mapped; Notification, PermissionRequest and Glob are unsupported | https://cursor.com/docs/reference/third-party-hooks |
| Copilot CLI | **Reads at runtime** `.claude/settings(.local).json` and accepts PascalCase names with the nested matcher structure | https://docs.github.com/en/copilot/reference/hooks-configuration |
| VS Code | Reads at runtime behind `chat.useClaudeHooks`, matchers ignored | https://code.visualstudio.com/docs/copilot/customization/hooks |
| Codex | **One-time import** (`/import` from Claude Code or Cursor). Plugins with `.claude-plugin` get `CLAUDE_PLUGIN_ROOT` | https://learn.chatgpt.com/docs/import ; https://github.com/openai/codex/blob/main/codex-rs/external-agent-migration/src/hooks_cla.rs |
| Gemini CLI | **One-time migration**: `gemini hooks migrate --from-claude` maps event names and tool names (Bash→`run_shell_command`, Edit→`replace`) | https://github.com/google-gemini/gemini-cli/blob/main/packages/cli/src/commands/hooks/migrate.ts |
| Kimi Code | Same event names and `permissionDecision`, different (TOML, user-level) config | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/hooks.md |
| OpenCode | none | https://opencode.ai/docs/plugins/ |

### 5.3 Parity table for the events a harness needs

Sources are the "Source" column of 5.1.

| Need | Claude Code | Codex | Gemini CLI | Cursor | OpenCode | Copilot CLI | Kimi Code |
|---|---|---|---|---|---|---|---|
| Session start (+context) | SessionStart | SessionStart | SessionStart | sessionStart | `session.created` event (observe only) | sessionStart | SessionStart (observe) |
| Prompt submit (can block) | UserPromptSubmit | UserPromptSubmit | BeforeAgent | beforeSubmitPrompt | `chat.message` (mutate) | userPromptSubmitted (output handling contradictory, see §10) | UserPromptSubmit |
| Pre-tool, can block | PreToolUse (deny/ask/rewrite) | PreToolUse (deny/rewrite, no ask) | BeforeTool (deny/rewrite) | preToolUse + shell, MCP and read variants | `tool.execute.before` (throw/mutate) | preToolUse (deny/ask/rewrite) | PreToolUse |
| Post-tool | PostToolUse, PostToolUseFailure | PostToolUse | AfterTool | postToolUse, postToolUseFailure | `tool.execute.after` | postToolUse, postToolUseFailure | PostToolUse, PostToolUseFailure |
| Stop with forced continuation | Stop | Stop | AfterAgent (deny = retry) | stop (`followup_message`) | **no** (`session.idle` event only) | agentStop (CLI ≥0.0.401) | Stop |
| Subagent stop | SubagentStop (block) | SubagentStop | no | subagentStop | no | subagentStop | SubagentStop (observe) |
| Pre-compact | PreCompact (block) | PreCompact | PreCompress | preCompact | `experimental.session.compacting` | preCompact (output ignored) | PreCompact (observe) |
| Session end | SessionEnd | SessionEnd | SessionEnd | sessionEnd | no (`session.deleted` is the closest) | sessionEnd | SessionEnd |
| Permission request | PermissionRequest | PermissionRequest | no | no | `permission.ask` | permissionRequest | PermissionRequest (observe) |
| Notification | Notification | no | Notification (ToolPermission) | no | TUI and bus events | notification | Notification |

The common core across the six shell-hook runtimes is: session start, blocking pre-tool, post-tool,
and stop with forced continuation. The seventh, OpenCode, lacks forced continuation and needs a JS
shim. Tool names differ across runtimes (`Bash` against `run_shell_command` against `apply_patch`),
so **matchers are not portable**, even where the event is.

---

## 6. Subagents and custom agents

| Runtime | Location | Format and main fields | Reads `.claude/agents` | Source |
|---|---|---|---|---|
| Claude Code | `.claude/agents/`, `~/.claude/agents/`, plugin `agents/`, `--agents` JSON | MD frontmatter: `name`, `description`, `tools`, `disallowedTools`, `model`, `permissionMode`, `maxTurns`, `skills`, `mcpServers`, `hooks`, `memory`, `isolation: worktree`, … Plugin agents ignore `hooks`, `mcpServers` and `permissionMode` | native | https://code.claude.com/docs/en/sub-agents |
| Codex | `.codex/agents/*.toml`, `~/.codex/agents/` | TOML: `name`, `description`, `developer_instructions`, `model`, `model_reasoning_effort`, `sandbox_mode`, `mcp_servers`, `skills.config` | one-time import only | https://learn.chatgpt.com/docs/agent-configuration/subagents ; https://github.com/openai/codex/blob/main/codex-rs/external-agent-migration/src/subagents.rs |
| Gemini CLI | `.gemini/agents/*.md`, `~/.gemini/agents/`, extensions | MD: `name`, `description`, `kind` (local/remote), `tools`, `mcpServers`, `model`, `temperature`, `max_turns`, `timeout_mins`. Subagents cannot call subagents | not confirmed | https://github.com/google-gemini/gemini-cli/blob/main/docs/core/subagents.md |
| OpenCode | `.opencode/agents/*.md`, `~/.config/opencode/agents/`, `agent` in `opencode.json` | MD: `description`, `mode` (primary/subagent/all), `model`, `prompt`, `permission`, `steps`, `hidden` | no | https://opencode.ai/docs/agents/ |
| Cursor | `.cursor/agents/`, `~/.cursor/agents/` | MD: `name`, `description`, `model`, `readonly`, `is_background` | **yes** (also `.codex/agents/`), `.cursor/` wins | https://cursor.com/docs/subagents |
| Copilot | `.github/agents/*.agent.md` or **`.claude/agents/`** (walks up to the git root), `~/.copilot/agents/` | MD: `name`, `description`, `target`, `tools`, `model`, `disable-model-invocation`, `user-invocable`, `mcp-servers` | **yes** | https://docs.github.com/en/copilot/reference/custom-agents-configuration |
| Kimi Code | `.kimi-code/agents/`, **`.agents/agents/`**, user equivalents | MD: `name`, `description`, `whenToUse`, `tools`, `disallowedTools`, `subagents`, `override`. Foreign fields are ignored, so Claude-style files load if placed in its directories | no (but compatible) | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/customization/agents.md |

Portable across runtimes: a Markdown body plus `name` and `description`. `tools` lists (the names
differ), `model` values and permission fields are runtime-specific.

---

## 7. MCP

### 7.1 Protocol

- Current spec revision **2026-07-28** (previous 2025-11-25). The protocol becomes stateless: no
  `initialize` and no session header; `server/discover` is added; the server-to-client requests
  sampling, elicitation and roots are replaced by the `input_required` pattern; Roots, Sampling and
  Logging are deprecated. https://modelcontextprotocol.io/specification/2026-07-28/changelog
- Transports: stdio and Streamable HTTP (HTTP+SSE deprecated).
  https://modelcontextprotocol.io/specification/2026-07-28/basic/transports
- Authorization: OAuth 2.1 with PRM (RFC 9728) and resource indicators, for HTTP transports only.
  https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization
- Governance: MCP joined AAIF on 2025-12-09.
  https://blog.modelcontextprotocol.io/posts/2025-12-09-mcp-joins-agentic-ai-foundation/
- Runtime adoption of the 2026-07-28 stateless revision: not stated in any runtime's docs
  (unconfirmed).

### 7.2 Configuration per runtime

| Runtime | Project file | Root key and shape | Env syntax, quirks | Source |
|---|---|---|---|---|
| Claude Code | **`.mcp.json`** (plus local and user scopes in `~/.claude.json`, `--mcp-config`) | `mcpServers`; `type` stdio/http/sse/ws | `${VAR}`, `${VAR:-default}`; project servers need approval interactively and load without it in `-p` | https://code.claude.com/docs/en/mcp |
| Codex | `.codex/config.toml` (trusted projects only) | TOML `[mcp_servers.<name>]` | `env`, `env_vars`, `bearer_token_env_var`, `enabled_tools`, per-server tool approval | https://learn.chatgpt.com/docs/extend/mcp?surface=cli |
| Gemini CLI | `.gemini/settings.json` | `mcpServers`; the transport is chosen by field: `command`, `url` (SSE) or `httpUrl` | `$VAR`/`${VAR}` only inside `env`; `trust`, `includeTools`/`excludeTools` | https://geminicli.com/docs/tools/mcp-server/ |
| OpenCode | `opencode.json` | `mcp`; `type: local` with **array** `command` and `environment`, or `type: remote` | `{env:VAR}`, `{file:path}` | https://opencode.ai/docs/mcp-servers/ |
| Cursor | `.cursor/mcp.json` (the CLI shares it) | `mcpServers` | `${env:NAME}`, `${workspaceFolder}`, `envFile` | https://cursor.com/docs/context/mcp |
| Copilot CLI | **`.mcp.json`** and `.github/mcp.json` from cwd up to the git root (trusted folders) | `mcpServers`; **`tools` is required** (`["*"]`) | `$VAR`, `${VAR}`, `${VAR:-d}` | https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-command-reference |
| VS Code | `.vscode/mcp.json` (`servers`) or the "portable" root **`.mcp.json`** (`mcpServers`) | — | `${input:…}` for secrets | https://code.visualstudio.com/docs/copilot/customization/mcp-servers |
| Copilot cloud agent | repository settings on GitHub (not a file) | `mcpServers`, `tools` required | secrets must be prefixed `COPILOT_MCP_`; **tools only**, no remote OAuth servers | https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/configure-mcp-servers |
| Kimi Code | `.kimi-code/mcp.json` (trusted projects only) | `mcpServers` | `bearerTokenEnvVar`, `enabledTools` | https://moonshotai.github.io/kimi-code/en/customization/mcp |

Beyond tools, CC and Cursor document prompts, resources and elicitation; Gemini documents prompts
and resources; the Copilot cloud agent supports tools only. For Codex, OpenCode, Copilot CLI and
Kimi, support beyond tools is unconfirmed (except Codex elicitation approval,
https://learn.chatgpt.com/docs/config-file/config-reference). Sources are the rows above.

---

## 8. Headless and CI

| Runtime | Command | Machine output | Auto-approve and permissions | Final-answer schema | Resume | Official GitHub Action | Source |
|---|---|---|---|---|---|---|---|
| Claude Code | `claude -p` (`--bare` for CI) | `--output-format json\|stream-json` | `--permission-mode …`, `--allowedTools`, `--max-turns`, `--max-budget-usd` | **`--json-schema`** → `structured_output` | `--resume`, `--continue`, `--fork-session` | `anthropics/claude-code-action@v1` | https://code.claude.com/docs/en/headless ; https://code.claude.com/docs/en/github-actions |
| Codex | `codex exec` (read-only sandbox by default) | `--json` (JSONL: `thread.*`, `turn.*`, `item.*`), `-o` last message | `--sandbox read-only\|workspace-write\|danger-full-access`, `--dangerously-bypass-approvals-and-sandbox` | **`--output-schema <file>`** | `codex exec resume` | `openai/codex-action` | https://learn.chatgpt.com/docs/non-interactive-mode ; https://github.com/openai/codex-action/blob/main/action.yml |
| Gemini CLI | `gemini -p` | `-o json\|stream-json`; documented exit codes | `--approval-mode default\|auto_edit\|yolo\|plan`; Policy Engine | no | `--resume` | `google-github-actions/run-gemini-cli` | https://geminicli.com/docs/cli/headless/ ; https://github.com/google-github-actions/run-gemini-cli |
| OpenCode | `opencode run`; `opencode serve` plus an SDK | `--format json` | `--auto`, `OPENCODE_PERMISSION` | SDK only (`json_schema`) | `-c`, `-s`, `--fork` | `anomalyco/opencode/github` | https://opencode.ai/docs/cli/ ; https://opencode.ai/docs/github/ |
| Cursor CLI | `agent -p` | `--output-format json\|stream-json` | `--force`, `--approve-mcps`; `.cursor/cli.json` permissions | no | `--resume` | none (docs recipe with curl install) | https://cursor.com/docs/cli/headless ; https://cursor.com/docs/cli/github-actions |
| Copilot CLI | `copilot -p` | `--output-format json` (JSONL) | `--allow-all-tools` (required), `--allow-tool`/`--deny-tool 'shell(git:*)'` | no | `--resume`, `--continue` | none (docs recipe with npm install) | https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-programmatic-reference |
| Kimi Code | `kimi -p` | `--output-format stream-json` | `auto` permission policy in `-p` plus deny rules | no | `-S`, `-c` | none | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/reference/kimi-command.md |

**ACP (Agent Client Protocol).** JSON-RPC over stdio for editors and hosts to drive agents; schema
v1.23.0 was released 2026-09-18 (https://github.com/agentclientprotocol/agent-client-protocol).

| Runtime | ACP entry point | Source |
|---|---|---|
| Gemini CLI | `gemini --acp`; the CLI reference still shows `--experimental-acp` | https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/acp-mode.md |
| Copilot CLI | `copilot --acp`, preview | https://docs.github.com/en/copilot/reference/copilot-cli-reference/acp-server |
| OpenCode | `opencode acp` | https://opencode.ai/docs/cli/ |
| Kimi Code | `kimi acp` | https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/reference/kimi-acp.md |
| Cursor | `agent acp`, hidden | https://cursor.com/docs/cli/reference/parameters |
| Claude Code | adapter `agentclientprotocol/claude-agent-acp` | https://github.com/agentclientprotocol/claude-agent-acp |
| Codex | adapter `agentclientprotocol/codex-acp`; its own protocol is `codex app-server` | https://github.com/agentclientprotocol/codex-acp |

---

## 9. Master parity table

`yes` = native. `partial` = with configuration or a limitation. `import` = a one-time conversion
command. `no` = not supported or not documented. Sources are in the sections referenced in the
first column.

| Feature | Claude Code | Codex | Gemini CLI | OpenCode | Cursor | Copilot CLI | Kimi Code |
|---|---|---|---|---|---|---|---|
| Reads `AGENTS.md` (§2) | partial (only without `CLAUDE.md`) | yes | partial (`context.fileName`) | yes | yes | yes | yes |
| Skills in `.agents/skills` (§3) | **no** | yes | yes | yes | yes | yes | yes |
| Skills in `.claude/skills` (§3) | yes | no | no | yes | yes | yes | no |
| `disable-model-invocation` honored (§3.3) | yes | no (`openai.yaml`) | no (`/skills disable`) | no (permissions) | yes | VS Code yes, CLI unconfirmed | yes |
| Shell hooks, stdin JSON, exit 2 (§5) | yes | yes | yes | no (JS plugins) | yes | yes | yes (user level only) |
| Project-level hook file in the repo (§5) | `.claude/settings.json` | `.codex/hooks.json` | `.gemini/settings.json` | `.opencode/plugins/` | `.cursor/hooks.json` | `.github/hooks/*.json` | no |
| Reads Claude hook config (§5.2) | native | import | import (migrate) | no | yes | yes | no |
| Reads the Claude plugin format (§4) | native | yes | no | no | unconfirmed | yes | no |
| Reads Agent Plugins 1.0 (§4) | no (not in docs) | yes | no | no | yes | yes | no |
| Repo declares plugins for the team (§4.2) | partial (external plugins need an install) | yes | no | yes | unconfirmed | yes | no |
| Subagent files (§6) | `.claude/agents` | `.codex/agents` TOML | `.gemini/agents` | `.opencode/agents` | `.cursor/agents` + reads `.claude/agents` | `.github/agents` + reads `.claude/agents` | `.agents/agents` |
| Project MCP in root `.mcp.json` (§7) | yes | no | no | no | no | yes | no |
| Headless with JSON output (§8) | yes | yes | yes | yes | yes | yes | yes |
| Schema-enforced final answer (§8) | yes | yes | no | SDK only | no | no | no |
| Official GitHub Action (§8) | yes | yes | yes | yes | no | no | no |
| ACP (§8) | adapter | adapter | yes | yes | yes (hidden) | yes (preview) | yes |

---

## 10. Implications for agent-workbench

### 10.1 Portable core the harness can rely on

1. **`AGENTS.md` as the one canonical instruction file**, kept under 32 KiB (the Codex default
   `project_doc_max_bytes`). Keep a `CLAUDE.md` containing `@AGENTS.md`. Claude Code reads
   `AGENTS.md` natively only when no `CLAUDE.md` exists anywhere up the tree, including parent
   workspace directories, and a project cannot change that mode. The bridge is the documented
   no-double-load path (§2.2). The current bootstrap already does this.
2. **Skills in the Agent Skills format, restricted to spec frontmatter** (`name`, `description`,
   optional `license`, `compatibility`, `metadata`), plus `disable-model-invocation` where a skill
   must be user-only. Canonical location `.agents/skills`, mirrored into `.claude/skills` for Claude
   Code. That covers all seven runtimes. `allowed-tools` is experimental and not portable. The current
   bootstrap (`.harness/skills` exposed through both roots) matches this.
3. **Mechanical checks as plain commands**, for example a harness `check` subcommand run by CI. Every
   runtime can run a shell command in headless mode, so enforcement does not depend on any runtime's
   hook or structured-output feature. This matches the brief's principle that what is mandatory is
   checked by code.
4. **One hook script with a small normalizing shim** that reads stdin JSON and writes either a
   decision or exit 2. Target the four common events: session start, pre-tool block, post-tool, and
   stop with forced continuation. The Claude Code format is read directly by Cursor and Copilot CLI,
   which makes it the natural source format.
5. **MCP servers as the portable tool layer**, declared once in a neutral form. The protocol works in
   every runtime.

### 10.2 What needs per-runtime adapters (generate plus a drift check)

- **Gemini CLI:** `.gemini/settings.json` with `context.fileName: ["AGENTS.md"]`, `hooks` (event and
  tool names renamed) and `mcpServers`.
- **Codex:** `agents/openai.yaml` per user-only skill; `.codex/hooks.json`; `[mcp_servers]` in
  `.codex/config.toml`; `.codex/agents/*.toml` if the harness ships subagents.
- **OpenCode:** a JS plugin in `.opencode/plugins/` that execs the shared hook script and maps
  `tool.execute.before` and after; the `mcp` key in `opencode.json`. Forced continuation is not
  available.
- **Cursor:** `.cursor/mcp.json`. Hooks come from `.claude/settings.json` when the third-party
  setting is on, otherwise `.cursor/hooks.json`.
- **Copilot:** `.mcp.json` with `tools: ["*"]`. `.github/hooks/*.json` if the cloud agent must be
  covered, because it does not read `.claude/`.
- **Kimi Code:** hooks can be installed only in user config, which is a machine-level install step and
  not a repository file; `.kimi-code/mcp.json`.
- **Claude Code:** `.claude/skills` mirror, `.claude/settings.json` hooks, `.mcp.json`.

### 10.3 What cannot be made portable

- **Tool names and matchers** (`Bash`, `run_shell_command`, `apply_patch`, `shell(git:*)`), and with
  them fine-grained permission rules.
- **Decision semantics beyond deny/allow**: `ask`, input rewriting, subagent stop, permission
  request, pre-compact blocking. Each runtime supports a different subset (§5.3).
- **Invocation syntax** (`/name`, `$name`, `/skill:name`, `@name`). Skill text must not hard-code it.
- **Subagent definitions** beyond `name`, `description` and body: model ids, tool lists, sandbox and
  permission modes.
- **Trust and approval models** for project-level hooks, MCP servers and plugins. Every runtime gates
  them differently, and some (Codex config, Kimi MCP) ignore project files until the folder is trusted.
- **Team distribution of plugins.** Only Codex, OpenCode and Copilot auto-enable from a repository
  declaration. Claude Code still requires an install for external plugins; Gemini and Kimi are
  per-user.
- **Schema-validated agent output** exists only in Claude Code and Codex.

### 10.4 Open questions for the grilling session

1. **Runtime tiers.** Which runtimes are tier 1 (adapters tested in CI) and which are best effort?
   Is Claude Code plus Codex the minimum? Are Gemini, OpenCode, Cursor, Copilot and Kimi in scope for
   #1 or later?
2. **Neutral source against native files.** Should the harness own a neutral declaration (skills,
   hooks, MCP, invocation policy) and generate native files with a drift check, or ship native files
   directly and accept the duplication? The generator approach implies a `harness sync` style command
   in the Rust CLI.
3. **Mirroring skills: symlink or copy?** `.agents/skills` → `.claude/skills` by symlink or by copy?
   Committed symlinks break on Windows without `core.symlinks`
   (https://code.claude.com/docs/en/memory). Is Windows a target for teams?
4. **Invocation policy source of truth.** Frontmatter (`disable-model-invocation`) plus a generated
   `openai.yaml`, or a registry file as today (`.harness/skills/REGISTRY.md`)?
5. **Distribution channel.** Copy files into the repository (installer), publish as a Claude-format
   plugin and marketplace (read by Claude Code, Codex and Copilot CLI), publish as Agent Plugins
   (Codex, Cursor, Copilot; not Claude Code), or several? Version pinning and update flow?
6. **Role of hooks.** The 2026-09-27 decision puts hooks after the artifact contract. When they come,
   are they enforcement (block, force continuation until the report exists) or observation only
   (journal for the command center)? Fail-open or fail-closed? Is OpenCode's missing forced
   continuation acceptable?
7. **CI runtime.** Does CI only run mechanical checks, or also an agent (judgment check) through
   `claude -p` or `codex exec` with a schema? If an agent, which one? Only two runtimes can enforce an
   output schema.
8. **MCP in the harness.** Does the harness ship any MCP servers by default, or only a place to
   declare them? How are secrets named across runtimes (for example the Copilot cloud agent's
   `COPILOT_MCP_` prefix)?
9. **ACP and the command center.** Driving agents is out of scope per the brief. Is ACP still useful
   as a read-only status interface for the command center, next to Herdr, `claude agents --json` and
   the Codex app-server?
10. **User-level installs.** Kimi hooks and some plugin installs are per-user only. Does "one-command
    installation on the developer machine" cover writing into user-level agent configs, which is an
    "ask first" boundary today?

---

## 11. Unconfirmed

- Nesting rules for Kimi Code `AGENTS.md` beyond the documented locations.
- Whether the Copilot CLI and cloud agent honor `disable-model-invocation` and `user-invocable`
  (documented for VS Code only).
- A per-skill slash invocation in Gemini CLI.
- Whether Codex requires a trusted project for `.codex/skills` (inferred from source).
- Whether Cursor reads `.claude-plugin/*`, and whether Cursor CLI has parity with the IDE for
  plugins, subagents and hooks.
- Whether Claude Code will adopt Agent Plugins.
- Whether Codex honors `enabledPlugins` from `.claude/settings.json`, and whether Codex plugin
  `hooks/hooks.json` semantics match Claude's. From source, Codex drops plugin `agents/` and LSP
  entries.
- The Copilot `userPromptSubmitted` output: the reference says config-file hook output is dropped,
  while the changelog says `additionalContext` is included.
- The exact PascalCase→camelCase event mapping in Copilot (for example Stop → agentStop).
- The version in which hooks were introduced in Codex, Gemini CLI, Cursor and Kimi Code.
- A project-level hook config for Kimi Code.
- MCP prompts, resources and elicitation support in Codex (except elicitation), OpenCode, Copilot CLI
  and Kimi.
- Runtime adoption of the MCP 2026-07-28 stateless revision.
- The fate of `codex exec --full-auto`: absent from the 0.158.0 `--help`, with no announcement found.
