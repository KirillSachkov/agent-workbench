# Stack landscape: what comparable developer and agent tools are built with

Date: 2026-09-29. Method: GitHub REST API (`gh api repos/<repo>` and `gh api repos/<repo>/languages`,
top languages by bytes, star counts on that date) plus `package.json` of web frontends. Each row's
source is the repository URL `https://github.com/<repo>`. Nothing here is taken from memory without a
link.

## 1. Terminal multiplexers and agent workspaces

| Repo | Stars | Languages |
|---|---|---|
| herdrdev/herdr | 41k | Rust 93% |
| zellij-org/zellij | 36k | Rust 98% |
| wezterm/wezterm | 29k | Rust 98% |
| tmux/tmux | 50k | C 72% |
| manaflow-ai/cmux | 27k | Swift 61%, Rust 12%, TS 9% |

Herdr plugins (GitHub topic `herdr-plugin`, top 15 by stars): Rust 6, TypeScript 5, Go 1, Python 1,
Lua 1, Swift 1. The plugins this project plans to use: `persiyanov/herdr-reviewr` (Rust),
`plannotator/herdr-annotate` (Rust), `hhdebb/herdr-radar` (JavaScript). Herdr plugin panes accept any
argv TUI, so the plugin language is free (see `2026-09-29-command-center-build-options.md`, 1.1).

## 2. Coding-agent runtimes

| Repo | Stars | Languages |
|---|---|---|
| openai/codex | 127k | Rust 97% |
| anomalyco/opencode | 211k | TypeScript 74% |
| google-gemini/gemini-cli | 107k | TypeScript 97% |
| earendil-works/pi | 110k | TypeScript 95% |
| aaif-goose/goose | 55k | Rust 72%, TypeScript 23% |
| charmbracelet/crush | 28k | Go 98% |
| Aider-AI/aider | 49k | Python 80% |

## 3. Developer TUIs and CLIs

| Repo | Stars | Languages |
|---|---|---|
| jesseduffield/lazygit | 83k | Go 99% |
| jesseduffield/lazydocker | 53k | Go 98% |
| derailed/k9s | 35k | Go 99% |
| dlvhdr/gh-dash | 13k | Go 99% |
| cli/cli (gh) | 46k | Go 99% |
| nektos/act | 72k | Go 81% |
| sxyazi/yazi | 42k | Rust 94% |
| jj-vcs/jj | 32k | Rust 99% |
| atuinsh/atuin | 32k | Rust 96% |
| astral-sh/uv | 90k | Rust 97% |
| jdx/mise | 34k | Rust 81% |
| casey/just | 36k | Rust 98% |
| go-task/task | 16k | Go 80% |
| evilmartians/lefthook | 9k | Go 87% |
| j178/prek | 9k | Rust 97% |
| pre-commit/pre-commit | 16k | Python 97% |

TUI frameworks: charmbracelet/bubbletea (Go, 45k), vadimdemedes/ink (TypeScript, 40k),
Textualize/textual (Python, 37k), ratatui/ratatui (Rust, 23k).

## 4. Agent command centers and orchestration dashboards

| Repo | Stars | Languages | Web stack |
|---|---|---|---|
| paperclipai/paperclip | 93k | TypeScript 93% | React 19, Vite, TanStack Query, Tailwind 4; Express, Drizzle (`ui/`, `server/package.json`) |
| BloopAI/vibe-kanban | 28k | Rust 50%, TypeScript 46% | React, Vite, TanStack Router and Query, Radix, Tailwind (`packages/local-web/package.json`); shipped through `npx-cli` |
| superset-sh/superset | 15k | TypeScript 85% | Electron, Next, Hono (`package.json`) |
| humanlayer/humanlayer | 12k | TypeScript 59%, Go 33% | — |
| smtg-ai/claude-squad | 9k | Go 89% | TUI only |
| stravu/crystal | 3k | TypeScript 96% | Electron |
| coder/agentapi | 2k | Go 82% | archived |

## 5. Spec, skills and harness tooling

| Repo | Stars | Languages | Distribution |
|---|---|---|---|
| github/spec-kit | 139k | Python 96% | `uvx` |
| Fission-AI/OpenSpec | 71k | TypeScript 98% | npm |
| bmad-code-org/BMAD-METHOD | 54k | Python 63%, HTML 21%, JS 11% | npx |
| mattpocock/skills | 272k | Shell 71%, JavaScript 28% | copied skill files |
| vercel-labs/skills | 33k | TypeScript 99% | `npx skills` |

## 6. Observations

- The harness content itself (pipeline document, skills, artifact templates) is Markdown with
  frontmatter across the whole category; the code around it varies.
- Long-lived terminal infrastructure (multiplexers, Codex, Herdr and most of its plugins) is Rust.
- Developer TUIs built around git and GitHub (gh, gh-dash, lazygit, k9s, claude-squad, crush) are Go
  with Bubble Tea.
- Local agent dashboards converge on React + Vite + TanStack + Tailwind in the browser, with either a
  TypeScript server (Paperclip) or a Rust binary serving it (vibe-kanban).
- Spec and skills installers in the agent ecosystem ship mostly through npm/npx, sometimes `uvx`.
