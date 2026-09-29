# Command center для агентной работы: источники данных, UI-варианты, архитектура

Дата: 2026-09-29. Машина владельца: herdr 0.9.1, Claude Code 2.1.284, codex-cli 0.158.0.
Метод: живые read-only команды на машине + официальная документация. Всё, что не проверено
командой или первичным источником, помечено «не подтверждено».

## 1. Источники данных

### 1.1 Herdr (самый богатый источник live-состояния)

Проверено: `herdr api --help` → подкоманды `snapshot`, `schema`; `herdr api schema --json`
(protocol 22, schema_version 1; схемы `request`, `success_response`, `error_response`, `event`,
`subscription_event`). Из схемы извлечено 129 методов/событий, ключевые:

- Чтение: `session.snapshot`, `workspace.list|get`, `tab.list|get`, `pane.list|get|read|process_info`,
  `agent.list|get|read|explain|wait`, `worktree.list`, `plugin.list|action.list|log.list`,
  `integration.list`, `server.agent_manifests`.
- Действия: `agent.focus|prompt|send_keys|start|rename`, `pane.split|focus|send_text`,
  `workspace.create|focus`, `worktree.create|open|remove`, `notification.show`,
  `plugin.action.invoke`, `plugin.pane.open|focus|close`, `command.invoke`.
- Метаданные от внешних источников: `pane.report_metadata` (title, display_agent, state_labels,
  до 16 `tokens`, `ttl_ms`), `workspace.report_metadata`, `pane.report_agent_session`
  (agent_session_id / path), `agent.view.set` (фильтр/сортировка в сайдбаре агентов).
- События: `events.subscribe` (persistent stream), `events.wait`; типы `workspace.*`, `tab.*`,
  `pane.created|closed|exited|updated|agent_detected|agent_status_changed|output_matched`,
  `worktree.created|opened|removed`, `layout.updated`.

`herdr agent list` на машине вернул 7 агентов с полями: `agent` (claude/codex),
`agent_session.value` (ID сессии рантайма), `agent_status` (`idle|working|blocked|done|unknown`),
`cwd`, `foreground_cwd`, `pane_id`, `tab_id`, `workspace_id`, `terminal_title`, `revision`,
`state_change_seq`, и `tokens` {repo, branch, worktree, base}. Токены пишет существующий
плагин `sachkov.agent-context` (`<личный плагин agent-context>`),
который хранит binding «agent_session → точный git worktree» и уже умеет `review` (Hunk на
merge-base) и `files`. Это готовое ядро связки «агент ↔ ветка ↔ worktree».

Документация (https://raw.githubusercontent.com/herdrdev/herdr/v0.9.1/docs/next/website/src/content/docs/socket-api.mdx):
NDJSON по unix socket `~/.config/herdr/herdr.sock`; `session.snapshot` — «one-time bootstrap snapshot
for clients that keep their own local runtime cache»; подписки «do not replay events retained
before that point» → паттерн: открыть подписку, затем взять snapshot, затем применять события.
Совместимость: «Client and server builds do not need to match», методы объявляются сервером,
неподдерживаемые дают обычную ошибку. Это публичный, версионированный контракт (schema + protocol).
`done` = «idle and not yet seen».

Статус агентов надёжен, потому что установлены официальные интеграции (`herdr integration status`:
claude v10, codex v8, copilot, kimi, opencode — через hooks рантаймов).

Плагины (https://raw.githubusercontent.com/herdrdev/herdr/v0.9.1/docs/next/website/src/content/docs/plugins.mdx):
директория с `herdr-plugin.toml`: `actions`, `panes` (любой argv-TUI; placement overlay/popup/split/
tab/zoomed), event hooks, link handlers; env `HERDR_SOCKET_PATH`, `HERDR_PLUGIN_STATE_DIR`,
`HERDR_PANE_ID` и др.; `plugin install` (GitHub) / `plugin link` (локально). «Native non-terminal
plugin UI are not part of plugin v1» — то есть UI плагина только терминальный.

### 1.2 Claude Code

- `claude agents --json` (проверено): массив активных сессий, interactive и background:
  `sessionId`, `pid`, `cwd`, `kind`, `name`, `status` (`busy|idle`) или `state` (`blocked` для bg),
  `startedAt`; `--all` добавляет завершённые bg. Документированный скриптовый интерфейс
  (https://code.claude.com/docs/en/sessions — «identifies the session in listings of running sessions,
  such as agent view and `claude agents --json`»).
- Hooks (https://code.claude.com/docs/en/hooks): SessionStart/End, UserPromptSubmit, Stop,
  StopFailure, Notification (matcher `permission_prompt`, `idle_prompt`, `agent_needs_input`,
  `agent_completed`…), PermissionRequest, Subagent*, TaskCreated/Completed, CwdChanged,
  WorktreeCreate/Remove и др. Общие поля: `session_id`, `transcript_path`, `cwd`. Тип `http` —
  POST JSON на локальный endpoint: чистый способ стримить события в собственный индекс без файлов.
- Транскрипты `~/.claude/projects/<project>/<session-id>.jsonl` — **внутренний формат**:
  «The entry format is internal to Claude Code and changes between versions» (там же). Не парсить.
- `claude --from-pr <n>` — picker сессий, связанных с PR (сессия, создавшая PR, находится по URL).
  Годится как действие «открыть сессию по PR», но не как источник данных.
- Headless: `claude -p --output-format json|stream-json` (`system/init`, `assistant`, `result` с
  `session_id`, cost) (https://code.claude.com/docs/en/headless) — для запуска review/summary из UI,
  например `claude -p --resume <id> --output-format json "summarize"`.
- OpenTelemetry (https://code.claude.com/docs/en/monitoring-usage): `CLAUDE_CODE_ENABLE_TELEMETRY=1`,
  OTLP/Prometheus; метрики `session.count`, `cost.usage`, `token.usage`, `pull_request.count`,
  `commit.count`; события `user_prompt`, `tool_result`… с `session.id`. Полезно для стоимости и
  активности, избыточно для статуса (статус уже есть в Herdr).

### 1.3 Codex

- `codex exec --json`: JSONL `thread.started{thread_id}` … `turn.completed{usage}` (вторичный
  источник https://takopi.dev/reference/runners/codex/exec-json-cheatsheet/; официальный — не подтверждено).
- App-server (https://learn.chatgpt.com/docs/app-server): JSON-RPC 2.0 по stdio / unix socket
  (stable) / WebSocket (experimental); методы `thread/list|read|resume|start`, `turn/start|interrupt`,
  `review/start`; уведомления `turn/started|completed`, `item/*`. Схема генерируется
  `codex app-server generate-json-schema|generate-ts` под конкретную версию. На машине уже есть
  общий daemon (`~/.codex/app-server-daemon`, `codex agents` «Browse all agent sessions on the shared
  local app-server daemon»). Команда `codex app-server` помечена `[experimental]` в CLI — считать
  контракт стабильным только для thread/turn-ядра.
- Hooks (https://developers.openai.com/codex/hooks): SessionStart, PreToolUse, PermissionRequest,
  PostToolUse, UserPromptSubmit, Subagent*, Stop (полный список версии 0.150+ — по вторичному источнику,
  не подтверждено).
- Локальные sqlite (`~/.codex/state_5.sqlite`, `thread_history_1.sqlite`, rollouts в
  `~/.codex/sessions/`) — **внутренние**, не использовать.

### 1.4 GitHub

- Projects v2 (проверено `gh project field-list`): Developer Pipeline — Status {Inbox, Blocked, Ready,
  In progress, Review, Acceptance, Done}, Area, Priority, Linked pull requests, Parent issue,
  Sub-issues progress, Repository, Reviewers. Human Backlog — Status {Todo, In Progress, Done}, Priority.
  Статусы уже проецирует `inside_tracker.py` (см. `docs/agents/tracker-automation.md`), а
  владение сессией — trusted issue comment от `tracker_sessions.py start` (receipt с session id и
  веткой). Это готовый канонический join «issue ↔ session ↔ branch».
- Чтение: GraphQL `organization.projectV2(number).items(first:100){fieldValues, content{Issue|PullRequest}}`;
  PR: `statusCheckRollup`, `reviewDecision`, `isDraft`; поиск `gh search prs --owner sachkov-inside`.
- Лимиты (проверено `gh api rate_limit`): core 5000/ч, graphql 5000 пунктов/ч. Опрос Projects раз в
  60–120 с на несколько сотен items укладывается с большим запасом. Условные REST-запросы с ETag
  (304) не расходуют primary limit (https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api).
- Webhooks: `projects_v2_item` — **только org webhooks, public preview, «subject to change»**
  (https://docs.github.com/en/webhooks/webhook-events-and-payloads#projects_v2_item). Для локального
  приложения нужен публичный endpoint или relay (`gh webhook forward` — dev-инструмент). Для
  соло-машины polling проще и честнее; webhooks — только если появится серверная часть.

### 1.5 Git и harness

- `git worktree list --porcelain` — стабильный машинный формат (проверено на platform: worktrees
  в `inside/worktrees/*` и чужие, например `<каталог другого инструмента>`).
  `herdr worktree list` даёт то же + `open_workspace_id`.
- `~/Work/projects.json` — реестр 18 проектов (`id`, `path`, `remote`, `stacks`, `status`).
- `harness/bin/inside-harness` — install/update/diff/health/rollback; своего JSON-вывода статуса
  нет (не подтверждено, что есть `--json`). Артефакты pipeline-стадий (spec/tickets) живут в issues.

### Итог по стабильности

| Источник | Контракт | Использовать для |
|---|---|---|
| Herdr socket API + events | публичный, versioned schema | live-агенты, статус, фокус, действия |
| `claude agents --json`, hooks (`http`) | публичный | Claude-сессии вне Herdr, события |
| Codex app-server thread/turn | stable ядро, остальное experimental | Codex-сессии, запуск review |
| GitHub GraphQL/REST, `gh` | публичный | задачи, стадия, PR, checks, review |
| `git worktree --porcelain` | стабильный | ветки/worktrees |
| Транскрипты `.jsonl`, codex sqlite | **внутренний** | не использовать |
| Projects webhooks | public preview | не сейчас |

## 2. UI-варианты

**(a) GitHub Projects views + saved searches + gh-dash.** Усилия: часы. Поддержка: почти ноль.
Презентабельность: высокая (Projects — веб, GitHub Mobile). Действия: gh-dash keybindings c
шаблонами `{{.RepoName}} {{.PrNumber}} {{.HeadRefName}} {{.RepoPath}}`, `repoPaths` мапит репо
в локальный путь (https://gh-dash.dev/configuration/keybindings/). Минус: не видит агентов, worktrees и
Herdr; Projects-поля в gh-dash не поддерживаются (не подтверждено). Пример:

```yaml
# ~/.config/gh-dash/config.yml
prSections:
  - title: Ждут моего review
    filters: is:open org:sachkov-inside review-requested:@me
  - title: Мои агенты (draft/open)
    filters: is:open org:sachkov-inside author:@me
issuesSections:
  - title: Ready for agent
    filters: is:open org:sachkov-inside label:ready-for-agent
  - title: Owner gate
    filters: is:open org:sachkov-inside label:ready-for-human
repoPaths:
  sachkov-inside/platform: ~/Work/Products/inside/repositories/platform
  sachkov-inside/*: ~/Work/Products/inside/repositories/*
keybindings:
  prs:
    - key: R
      name: review в Claude
      command: cd {{.RepoPath}} && claude --from-pr {{.PrNumber}}
    - key: v
      name: approve
      command: gh pr review --repo {{.RepoName}} --approve {{.PrNumber}}
```

**(b) TUI как Herdr plugin pane (Textual / Bubble Tea / Ratatui).** Усилия: 2–5 дней для v1.
Поддержка: средняя, но в одном языке с harness (Python → Textual). Презентабельность: хорошая для
демо в терминале, не для ссылки коллеге. Действия нативные: `agent.focus`, `plugin.action.invoke`
(agent-context review), `pane.split` + `claude --resume`, `gh pr view --web`. Живёт там же, где
агенты; подписка на события Herdr даёт мгновенный статус.

**(c) Локальный web (FastAPI + HTMX/SSE или SvelteKit).** Усилия: 1–2 недели. Поддержка: выше
(фронтенд, сборка). Презентабельность: лучшая, можно показать экран/скриншот, при желании
опубликовать read-only снимок. Действия: через backend вызывает те же CLI/socket; фокус терминала —
`agent.focus` через Herdr, открытие PR — ссылка. Риск: localhost-сервер с правом слать ввод агентам
нужно защищать (bind 127.0.0.1, токен).

**(d) Desktop Tauri.** Усилия: (c) + упаковка/подпись. Выгода над (c) — menubar, нативные уведомления.
Для соло-разработчика не окупается сейчас.

**(e) Backstage / Port.** Backstage — тяжёлый Node-монорепо для org-каталогов; Port — SaaS
(данные уходят наружу). Оба не знают про локальные агенты; избыточно для одного человека.

**(f) Raycast extension.** Усилия: 1–2 дня (TypeScript). Хорош как launcher поверх готового индекса:
«найти задачу/агента → фокус/открыть PR». Не заменяет обзорный экран. Имеет смысл как второй клиент
одного read-model.

## 3. Архитектура без хаков

1. **Один read-model**, построенный адаптерами, каждый читает только публичный контракт:
   `herdr` (snapshot + events.subscribe), `claude` (`agents --json`, опционально http-hook),
   `codex` (app-server `thread/list` или только статус из Herdr), `github` (GraphQL Projects + PR
   rollup, polling с курсором `updatedAt`), `git` (`worktree list --porcelain` по `projects.json`),
   `tracker` (session receipts из issue comments, уже в формате harness).
2. **Ключи join'а**: `session_id` (Herdr `agent_session.value` = Claude/Codex id), `worktree path`
   (agent-context binding), `branch` → `issue #` (конвенция `feat/<n>-slug`), issue → Project item →
   Status (= pipeline stage), PR `Closes #n`. Нечёткие связи помечать как «выведено», не как факт.
3. **Событие + опрос**: Herdr — события (локально, дёшево); GitHub — polling 60–120 с + ETag;
   git — по событиям `worktree.*` Herdr и раз в N минут. Webhooks откладываются.
4. **Где состояние**: истина остаётся в источниках (GitHub — задачи и стадии; Herdr — live;
   git — ветки). Индекс — выбрасываемый кэш (SQLite в `~/.local/state/<tool>/`), пересобирается
   с нуля. Своих статусов задач не заводить — иначе второй tracker (запрещено `AGENTS.md` Work).
5. **Runtime-agnostic**: ядро оперирует понятием `AgentSession{runtime, id, status, cwd, worktree}`;
   Herdr уже нормализует статусы для 5+ рантаймов, поэтому основной адаптер статуса — Herdr, а
   runtime-адаптеры только добавляют то, чего нет в Herdr (имя сессии, bg-сессии Claude).
6. **Действия = вызовы существующих CLI**, не имитация ввода: `herdr agent focus`,
   `herdr plugin action invoke sachkov.agent-context review`, `gh pr view --web`,
   `claude --resume <id>` / `codex resume <id>` в новом pane, `gh pr review --approve` (owner gate —
   только явным действием владельца).
7. **Устанавливаемость**: ядро — Python-пакет с CLI `… snapshot --json`; UI-слой — Herdr plugin
   (`herdr-plugin.toml` с pane и actions). Конфиг проектов — из `projects.json`, орг/Projects — из
   harness (`docs/agents/issue-tracker.md` уже задаёт номера Projects). Решение для владельца: где
   живёт код. Инструмент личный и межпроектный → естественно рядом с `sachkov-agent-context` в
   каталог личных плагинов или отдельный repo; в `inside-engineering` package — только
   если он должен ставиться в каждый Inside-repo (скорее нет).

## 4. Что переиспользовать

- **sachkov-agent-context** (уже есть): binding session→worktree, токены repo/branch/base, action review.
- **gh-dash** (https://github.com/dlvhdr/gh-dash): GitHub-часть уже сегодня, custom keybindings.
- **lazygit**, **Hunk**, **octo.nvim** (https://github.com/pwntester/octo.nvim) — review внутри
  терминала/Neovim; запускать как действие, не встраивать.
- **GitHub Mobile / Projects web** — презентабельный read-only вид для других людей.
- `gh` extensions: `gh-dash`, `gh webhook forward` (dev-relay), `gh project` (встроено).
- Open-source dashboards для заимствования идей/кода (не проверялись детально): agent-deck
  (https://github.com/asheshgoplani/agent-deck), claudecodeui (https://github.com/siteboon/claudecodeui),
  списки https://github.com/andyrewlee/awesome-agent-orchestrators и
  https://www.augmentcode.com/tools/open-source-agent-orchestrators. Большинство сами владеют
  запуском агентов (tmux) — конфликтует с Herdr; брать UI-идеи, не рантайм.
- Linear: даёт красивый UI и GitHub-sync, но это второй tracker → противоречит правилу «tracker
  проекта единственный источник статуса». Не рекомендую.

## 5. Рекомендация

**Архитектура:** Python-ядро «read-model + адаптеры» (Herdr socket/events, `claude agents --json`,
GitHub GraphQL polling, `git worktree --porcelain`, tracker receipts) → SQLite-кэш → два клиента:
(1) Textual TUI как Herdr plugin pane (основной, с действиями), (2) позже — FastAPI+HTMX read-only
страница для показа другим. GitHub Projects остаются источником истины о задачах и стадиях;
gh-dash закрывает GitHub-срез без кода.

**Минимальная первая версия (≈2–3 дня):**
1. День 0: конфиг gh-dash выше (review-queue, ready-for-agent, owner gate) — ноль кода.
2. CLI `cc snapshot --json`: join `herdr agent list` + `claude agents --json` + `git worktree list`
   по `projects.json` + Developer Pipeline items (Status, Linked PR, checks rollup). Ключ: worktree/branch
   → issue #. Без демона, polling по запуску.
3. Textual pane в Herdr-плагине: три таблицы — «Агенты» (runtime, статус, repo/branch, issue, stage),
   «Задачи по проектам» (Ready/In progress/Review/Acceptance), «Worktrees без агента/без PR».
   Действия: Enter → `herdr agent focus`; `r` → agent-context review; `o` → `gh pr view --web`;
   `s` → новый pane с `claude --resume`.
4. Критерий готовности: для каждого живого агента видны задача, стадия и PR без ручного поиска;
   каждое действие — вызов публичного CLI/API; удаление кэша ничего не теряет.

Следующие шаги после v1: подписка `events.subscribe` вместо опроса Herdr; Claude `http`-hook для
Notification/Stop; web read-only вид; Raycast-launcher поверх `cc snapshot --json`.
