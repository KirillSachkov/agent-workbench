# Локальные следы работы агентов: механизмы и инструменты (2026-09-27)

Задача: локально и автоматически получать читаемые артефакты по каждой сессии агента (Claude Code,
Codex CLI) в git worktrees — какая задача, что сделано, что изменилось, какие проверки и с каким
результатом, где смотреть, что осталось — и связывать их с будущим GitHub PR.

Проверенные локально версии: Claude Code 2.1.283, codex-cli 0.157.1. Форматы файлов сверены по
ключам, без содержимого.

---

## 1. Claude Code

### 1.1 Hooks

Источник: https://code.claude.com/docs/en/hooks

**События (33 шт.)**: SessionStart, Setup, UserPromptSubmit, UserPromptExpansion, PreToolUse,
PermissionRequest, PermissionDenied, PostToolUse, PostToolUseFailure, PostToolBatch, Notification,
MessageDisplay, SubagentStart, SubagentStop, TaskCreated, TaskCompleted, Stop, StopFailure,
TeammateIdle, InstructionsLoaded, ConfigChange, CwdChanged, DirectoryAdded, FileChanged,
WorktreeCreate, WorktreeRemove, PreCompact, PostCompact, PreModelSwitch, PostModelSwitch,
Elicitation, ElicitationResult, SessionEnd.

**Общие поля stdin JSON**: `session_id`, `prompt_id`, `transcript_path`, `cwd`, `scratchpad_dir`,
`permission_mode`, `effort`, `hook_event_name`, у субагентов `agent_id`, `agent_type`.

**Ключевые для «журнала сессии»**:

| Событие | Что даёт | Специфичные поля |
|---|---|---|
| SessionStart | старт/resume/clear/compact/fork; можно вернуть `additionalContext` (например, «задача #123, ветка X») | matcher `startup\|resume\|clear\|compact\|fork` (имя поля источника в выдержке docs расходится: `source`/`how_started` — не подтверждено, проверить на своём hook) |
| PostToolUse / PostToolUseFailure | каждая команда Bash (тесты, lint) с результатом | `tool_name`, `tool_input`, `tool_response`, `tool_use_id` |
| Stop / SubagentStop | конец ответа; итоговое сообщение агента | `last_assistant_message`, `stop_hook_active` |
| TaskCreated / TaskCompleted | задачи внутреннего todo | `task_id`, `task_name` |
| WorktreeCreate / WorktreeRemove | создание/удаление worktree | `worktree_path`, `branch` |
| SessionEnd | финализация журнала | `reason`: `clear\|resume\|logout\|prompt_input_exit\|other` |

**Блокировка**: exit 2 блокирует PreToolUse, UserPromptSubmit, UserPromptExpansion, Stop,
SubagentStop, TeammateIdle, TaskCreated, PreModelSwitch, WorktreeCreate, WorktreeRemove. Для
PostToolUse, Notification, SessionEnd и др. exit 2 не блокирует. Stop с exit 2 = «не заканчивай»,
stderr уходит агенту — так можно заставить агента дописать отчёт до завершения (обязательно
проверять `stop_hook_active`, чтобы не зациклить).

**Типы hooks**: `command`, `http`, `mcp_tool`, `prompt`, `agent`. `async: true` — фоновой запуск;
`asyncRewake: true` — фон, будит Claude при exit 2.

**Таймауты**: command 600 с по умолчанию; **SessionEnd — общий бюджет 1.5 с** (до 60 с, если задать
больший timeout). Значит, тяжёлую генерацию отчёта на SessionEnd не делать: писать инкрементально в
PostToolUse/Stop, а в SessionEnd только финализировать или запускать detached-процесс.

**Где конфигурировать**: `~/.claude/settings.json` (все проекты), `.claude/settings.json` (в repo,
для всех сессий в этом repo и его worktrees), `.claude/settings.local.json` (gitignored), managed
policy, plugin `hooks/hooks.json`, frontmatter skills/agents. Hooks из разных уровней
**сливаются**. Project hooks подчиняются workspace trust. `${CLAUDE_PROJECT_DIR}` — корень, где
стартовала сессия; в worktree он не меняется, текущий каталог брать из `cwd`.

**Ответ**: да, hook из `.claude/settings.json` в repo срабатывает для каждой сессии, открытой в этом
repo (включая worktree, так как файл закоммичен и присутствует в worktree).

Пример (project `.claude/settings.json`):

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command",
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh start" }] }],
    "PostToolUse": [{ "matcher": "Bash", "hooks": [{ "type": "command", "async": true,
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh tool" }] }],
    "Stop": [{ "hooks": [{ "type": "command",
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh stop" }] }],
    "SessionEnd": [{ "hooks": [{ "type": "command",
      "command": "${CLAUDE_PROJECT_DIR}/.claude/hooks/journal.sh end" }] }]
  }
}
```

`journal.sh` читает stdin JSON (`jq`), берёт `session_id`, `cwd`, `git -C "$cwd" branch
--show-current`, `git diff --stat`, и дописывает строку JSONL + Markdown в
`<worktree>/.agent-runs/<session_id>/` (или в общий каталог вне repo).

Зрелость: стабильный документированный API; список событий быстро растёт между версиями.

### 1.2 Transcript-файлы и связь с PR

Источник: https://code.claude.com/docs/en/sessions

- Путь: `~/.claude/projects/<project>/<session-id>.jsonl`, `<project>` = путь cwd с заменой
  не-алфавитно-цифровых символов на `-`. Субагенты: `<session-id>/subagents/agent-<id>.jsonl` +
  `.meta.json` (наблюдено локально).
- Docs прямо предупреждают: **формат внутренний и меняется между версиями**; для скриптов
  использовать `/export`, `claude -p --output-format json|stream-json`, `transcript_path` из hooks,
  Agent SDK.
- Хранение 30 дней по умолчанию: `cleanupPeriodDays`. Для архива — копировать в SessionEnd.
- Локально наблюдаемые типы строк (не документированы): `user`, `assistant`, `system`,
  `attachment`, `ai-title`, `pr-link` (с `prUrl`, `prNumber`, `prRepository`), `file-history-*`,
  `cost-state`; поля `gitBranch`, `cwd`, `totalLinesAdded/Removed`, `totalCostUSD`.
- **Встроенная связь сессия↔PR**: `claude --from-pr <number>` открывает picker, отфильтрованный
  по сессиям, связанным с PR; в picker можно вставить URL PR, чтобы найти создавшую его сессию;
  `Ctrl+W` — все worktrees репозитория, `Ctrl+B` — фильтр по текущей ветке.
- Именование: `claude -n <name>`, `/rename`; иначе генерируемый заголовок. `claude agents --json`
  перечисляет запущенные сессии.
- `/export [file]` — читаемый plain-text транскрипт (с выводом инструментов).
- Суммаризация существующей сессии скриптом:
  `claude -p --resume <session-id> --output-format json "summarize what we changed" | jq -r .result`.

### 1.3 Statusline

Источник: https://code.claude.com/docs/en/statusline

stdin JSON: `session_id`, `session_name`, `transcript_path`, `cwd`, `model`, `workspace`
(`current_dir`, `project_dir`, `git_worktree`), `worktree`, `version`, `cost`
(`total_cost_usd`, `total_lines_added`, …), `context_window`. Обновляется по событиям (debounce
300 мс) + `refreshInterval`. Полезно, чтобы в каждой параллельной вкладке видеть задачу/ветку;
можно дописывать heartbeat в файл, но это побочный канал, не журнал.

```json
{ "statusLine": { "type": "command", "command": "~/.claude/statusline.sh", "refreshInterval": 10 } }
```

### 1.4 OpenTelemetry

Источник: https://code.claude.com/docs/en/monitoring-usage

```bash
export CLAUDE_CODE_ENABLE_TELEMETRY=1
export OTEL_METRICS_EXPORTER=otlp        # otlp|prometheus|console|none
export OTEL_LOGS_EXPORTER=otlp           # otlp|console|none
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
# содержимое по умолчанию скрыто:
export OTEL_LOG_USER_PROMPTS=1 OTEL_LOG_TOOL_DETAILS=1
# трейсы (beta):
export CLAUDE_CODE_ENHANCED_TELEMETRY_BETA=1 OTEL_TRACES_EXPORTER=otlp
```

Метрики: `claude_code.session.count`, `lines_of_code.count`, `pull_request.count`,
`commit.count`, `cost.usage`, `token.usage`, `code_edit_tool.decision`, `active_time.total`.
События: `claude_code.user_prompt`, `api_request`, `assistant_response`, `tool_decision`,
`tool_result`, и др. Атрибуты: `session.id`, `vcs.*` (при `OTEL_METRICS_INCLUDE_REPOSITORY=1`).
Трейсы: spans `claude_code.interaction` → `llm_request`, `tool`.

Что даёт владельцу: счётчики и тайминги по сессиям, но **не читаемый отчёт** — нужен локальный
collector + backend (например, otel-collector → файл/ClickHouse/Grafana). Для соло-владельца это
избыточно по сравнению с hooks; полезно как вторичный канал метрик.

---

## 2. OpenAI Codex CLI

### 2.1 Hooks

Источник: https://developers.openai.com/codex/hooks (редирект на https://learn.chatgpt.com/docs/hooks)

- События: `SessionStart`, `SessionEnd`, `SubagentStart`, `UserPromptSubmit`, `PreToolUse`,
  `PermissionRequest`, `PostToolUse`, `PreCompact`, `PostCompact`, `SubagentStop`, `Stop`,
  `Interrupt`.
- Общие поля: `session_id`, `transcript_path` (может быть null), `cwd`, `hook_event_name`,
  `model`; turn-scoped — `turn_id`; tool — `tool_name`, `tool_input`.
- Где: `~/.codex/hooks.json` или `~/.codex/config.toml`; **project-level
  `<repo>/.codex/hooks.json`** или `.codex/config.toml`; plugins. Включены по умолчанию
  (отключение `[features] hooks = false`). Entire отмечает: hooks включены по умолчанию с
  codex-cli 0.124.0.
- Формат совместим по духу с Claude Code (`matcher` + `hooks[{type:"command",command}]`) — один
  скрипт журнала можно переиспользовать, различая `hook_event_name`/поля.

```json
{ "hooks": { "Stop": [ { "hooks": [ { "type": "command",
  "command": "python3 .codex/hooks/journal.py" } ] } ] } }
```

### 2.2 `notify` (старый механизм)

Источник: https://learn.chatgpt.com/docs/config-file/config-advanced

```toml
notify = ["python3", "/path/to/notify.py"]
```
Только событие `agent-turn-complete`; JSON передаётся **аргументом**, поля `thread-id`,
`turn-id`, `cwd`, `input-messages`, `last-assistant-message`. Годится как fallback.

### 2.3 Session/rollout файлы

- `~/.codex/sessions/YYYY/MM/DD/rollout-<timestamp>-<uuid>.jsonl` (наблюдено локально; docs
  говорят только «sessions and rollout files persisted», путь в docs — не подтверждено).
- Первая строка `session_meta` с полями `id`, `cwd`, `git`, `cli_version`, `source`,
  `parent_thread_id`, `forked_from_id`…; далее `response_item` (message, reasoning,
  function_call/_output, custom_tool_call), `event_msg` (task_started/task_complete, token_count),
  `turn_context`. Формат внутренний.
- `~/.codex/history.jsonl` — история промптов (`[history] persistence`).
- `codex exec --ephemeral` отключает запись.

### 2.4 `codex exec --json` и OTel

Источник: https://learn.chatgpt.com/docs/non-interactive-mode (раздел Codex non-interactive)

- `codex exec --json` → JSONL: `thread.started`, `turn.started`, `turn.completed` (с usage),
  `item.started`, `item.completed` (agent_message, reasoning, command_execution, file_change,
  mcp_tool_call, web_search, plan update), `error`.
- `-o/--output-last-message <path>` — финальное сообщение в файл; `--output-schema <schema.json>`
  — финальный ответ по JSON Schema (удобно для структурированного «отчёта о сессии»).
- `codex exec resume <SESSION_ID>`.
- OTel: `[otel] exporter = "none"|"otlp-http"|"otlp-grpc"`, `log_user_prompt = false`,
  `environment`. События в исходниках: `codex.tool_result` и др.
  (https://github.com/openai/codex/blob/main/codex-rs/otel/src/tool_result.rs); полный список в
  docs — не подтверждено.

---

## 3. Инструменты захвата сессий

### 3.1 Entire CLI (Checkpoints)

Источник: https://github.com/entireio/cli (README), https://entire.io/blog/hello-entire-world

- Что: на каждом `git commit` сохраняет checkpoint: транскрипт, промпты, затронутые файлы,
  токены, tool calls, субагенты; опционально AI-summary («intent, outcome, learnings, friction
  points, open items») — почти ровно то, что нужно владельцу.
- Где: **отдельные refs `refs/entire/checkpoints/<shard>/<id>`** (ID — 26-символьный ULID).
  Старые версии писали в ветку `entire/checkpoints/v1` (по сторонним обзорам; текущий README
  описывает refs). Во время работы — локальная shadow branch (никогда не пушится). Коммиты на
  рабочей ветке не создаются.
- Связь с commit/PR: trailer **`Entire-Checkpoint: <id>`** в сообщении коммита → через PR
  связь идёт автоматически по коммитам.
- Push: checkpoint refs уезжают вместе с `git push` в один выбранный remote
  (`push_sessions`, `checkpoint_push_remote`), либо в отдельный repo (`checkpoint_remote`),
  `--skip-push-sessions` — только локально.
- Агенты: Claude Code, Codex (ставит `.codex/hooks.json`), Cursor, Copilot CLI, OpenCode, Gemini,
  Droid, Antigravity, Pi. Ставит hooks в native-конфиги агентов.
- Команды: `entire enable [--agent claude-code]`, `entire status`, `entire checkpoint
  list|explain [--generate]|search|tokens`, `entire recap`, `entire dispatch`, `entire session
  resume <branch>`, `entire disable`.
- Конфиг: `.entire/settings.json` (shared) + `.entire/settings.local.json`;
  `strategy_options.summarize.enabled`, `summary_generation.provider = claude-code|codex|…`.
- Редакция секретов в транскриптах; но snapshot-ы кода на shadow branch — сырые.
- Лицензия MIT; работает локально/offline; облако entire.io опционально. Зрелость: v0.11.x,
  ежедневные nightly (последняя 2026-09-26), ~5k stars — активно, но API ещё до 1.0.
- Ограничения: summary тратит токены агента; checkpoint появляется только при коммите (работа без
  коммита видна лишь на shadow branch); GitHub UI refs не показывает — читать через CLI/entire.io.

### 3.2 git-ai

Источник: https://github.com/git-ai-project/git-ai, стандарт
https://github.com/git-ai-project/git-ai/blob/main/specs/git_ai_standard_v3.0.0.md

- Что: построчная атрибуция AI-кода (агент, модель, сессия, промпт) → `git ai blame`,
  `git ai stats [--json] <a>..<b>` (% AI-кода, принятые строки, override по моделям).
- Где: **Git Notes** (`git log --show-notes="ai"`); сами промпты/сессии — **вне git** (redacted,
  локально). Без git hooks и без обёртки git.
- Агенты: Claude Code, Codex, Cursor и др.; local-first, без логина. Apache-2.0, v1.7.5
  (2026-09-09), ~2.8k stars. Платный «Git AI for Teams» — опционально.
- Даёт «кто писал строки», но не «что проверено/что осталось». Реализует/поддерживает Agent Trace.

### 3.3 Agent Trace (спецификация)

Источник: https://agent-trace.dev (repo `cursor/agent-trace` — ссылка из сайта, на 2026-09-27
GitHub вернул 404; не подтверждено, переехал ли repo).

- RFC v0.1.0 (янв. 2026), Cursor + Cognition, поддержка Cloudflare, Vercel, Jules, Amp,
  OpenCode, git-ai. CC BY 4.0.
- JSON-запись: `version`, `id`, `timestamp`, `vcs` (тип+revision), `tool`, `files[]` →
  `conversations[]` → `ranges` + `contributor` (human/ai/mixed/unknown), `metadata`.
- **Хранение не определено** (файлы, git notes, БД). Это формат атрибуции, не журнал сессии.
  Полезен как схема для поля «authorship» в своём артефакте, если нужна совместимость.

### 3.4 SpecStory

Источник: https://github.com/specstoryai/getspecstory

- Сохраняет каждую сессию как Markdown в **`<repo>/.specstory/history/`** (local-first);
  облако `cloud.specstory.com` опционально (sync/search/share).
- Запуск через обёртку: `specstory run claude` / `specstory run codex` (brew
  `specstoryai/tap/specstory`). CLI open source (Apache-2.0), v2.15.1 (2026-09-24).
- Плюс: читаемые .md рядом с кодом, можно коммитить. Минус: нет привязки к коммитам/PR, нет
  понятия проверок; нужна обёртка запуска (или поддержка agents через их файлы — детали не
  подтверждены).

### 3.5 claude-code-log

Источник: https://github.com/daaain/claude-code-log

- Конвертер `~/.claude/projects/**.jsonl` → HTML/Markdown, индекс по проектам, per-session
  страницы, токены, TUI, `watch`, `serve --watch`. Экспериментально: `--provider codex
  --session-id <id>`.
- `uvx claude-code-log@latest --open-browser`; `--detail full|high|low|minimal|user-only`,
  `--format md`.
- MIT, v1.6.0 (2026-08-31), активно. Зависит от внутреннего формата транскриптов → может
  ломаться на новых версиях Claude Code.

### 3.6 ccusage

Источник: https://github.com/ccusage/ccusage — `npx ccusage` (daily/session/blocks, JSON),
v20.0.26 (2026-09-27). Только токены/стоимость по сессиям; не журнал работы. Лицензия в GitHub
помечена «Other» (не подтверждено какая).

---

## 4. Git-native связывание

- **Commit trailers**: `git interpret-trailers` (https://git-scm.com/docs/git-interpret-trailers).
  Предложение: `Agent-Session: claude:<session_id>`, `Task: #123`, `Evidence:
  .agent-runs/<id>/report.md`, `Co-Authored-By: …`. Trailers видны в PR (сообщения коммитов),
  переживают rebase/squash (при squash — если сохранить тело). Добавлять можно hook-ом
  `prepare-commit-msg` или инструкцией агенту; Entire уже делает `Entire-Checkpoint:`.
  Поиск: `git log --format='%(trailers:key=Agent-Session,valueonly)'`.
- **Git notes** (https://git-scm.com/docs/git-notes): метаданные без изменения SHA, можно
  дописывать после коммита. Минусы: `refs/notes/*` не пушатся/не фетчатся по умолчанию (нужен
  refspec), при rebase теряются без `notes.rewriteRef`/`notes.rewrite.rebase`, GitHub UI не
  показывает notes. git-ai решает часть этого сам.
- **Отдельные refs** (как Entire): не мешают истории, независимые push/fetch, но невидимы в UI.
- **Ветки**: `agent/<task-id>-<slug>` + worktree с тем же именем → `gitBranch` в транскрипте и
  `Ctrl+B` в picker сразу группируют сессии по задаче.
- **Папка на задачу** в repo (например, `docs/evidence/<task>/`) — видно в PR diff, но засоряет
  историю; альтернатива — вне repo (`~/.agent-runs/<repo>/<branch>/`), а в PR вставлять ссылку/
  выжимку.

## 5. Локальные доказательства проверок без CI

- **Playwright** (https://playwright.dev/docs/test-reporters,
  https://playwright.dev/docs/test-use-options):
  ```ts
  reporter: [['list'], ['html', { outputFolder: 'playwright-report', open: 'never' }],
             ['json', { outputFile: 'test-results/results.json' }]],
  use: { trace: 'retain-on-failure', video: 'retain-on-failure', screenshot: 'only-on-failure' }
  ```
  `npx playwright show-report [folder|zip]` — локальный сервер с трейсами/видео;
  `merge-reports` для blob.
- **Vitest** (https://vitest.dev/guide/reporters): встроенные `json`, `junit`, `html` (статичный
  Vitest UI, без отдельного пакета), `blob`, `minimal` (alias `agent`).
  ```ts
  test: { reporters: ['default', 'json', 'junit'],
          outputFile: { json: '.evidence/vitest.json', junit: '.evidence/junit.xml' } }
  ```
- **Evidence-папка**: hook `PostToolUse(Bash)` ловит команды тестов (`pnpm test`, `playwright`),
  копирует/линкует отчёты в `.agent-runs/<session>/evidence/`; просмотр `python3 -m http.server`
  или `npx serve`. Индексная страница генерируется скриптом из JSON (session, branch, task,
  checks[{cmd, exit, report}], diffstat, open items).
- Агрегаторы: Allure / Monocart (custom reporters Playwright), JUnit-viewers — упомянуты в docs
  Playwright как сторонние; готового «дашборда локальных прогонов агентов» из первичных источников
  не найдено (не подтверждено).

## 6. Вывод для harness

Минимальная связка без внешних сервисов: project hooks (Claude `.claude/settings.json` + Codex
`.codex/hooks.json`) пишут `session.json`/`report.md` в per-session каталог; тестовые раннеры
кладут JSON/HTML-отчёты туда же; коммиты получают trailer `Agent-Session:`; при `gh pr create`
скрипт вставляет выжимку и ссылки в описание PR. Claude Code сам уже связывает сессию с PR
(`--from-pr`). Entire закрывает «транскрипт + summary + привязка к коммиту» готово, git-ai —
построчную атрибуцию; оба open source и работают локально.
