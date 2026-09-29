# Research 5: «командный центр» для параллельных coding agents (состояние на 2026-09-29)

Метод: метаданные GitHub через `gh api` (звёзды, последний push, лицензия, архивность) на 2026-09-29;
README и официальные страницы продуктов; для коммерческих продуктов — официальные блоги/доки, где
удалось; вторичные источники помечены. «не подтверждено» = не удалось проверить по первоисточнику.

## 1. Paperclip (paperclipai/paperclip)

Источник: https://github.com/paperclipai/paperclip (README), https://docs.paperclip.ing

- **Что это.** «Open-source orchestration for teams of AI agents… If OpenClaw is an employee,
  Paperclip is the company.» Node.js server + React UI. Позиционирование — «Manage business goals,
  not pull requests»: цель компании → org chart агентов (CEO, CTO, инженеры…) → бюджеты → heartbeat.
- **Модель данных.** Company (multi-org, полная изоляция) → Goals → Projects → Issues (связи с
  company/project/goal/parent, blocker-зависимости, comments, documents, attachments, work products,
  labels, inbox state) ; Agents (роль, title, reporting line, permissions, budget); Heartbeat runs
  (DB-очередь wakeup, coalescing, budget check, workspace resolution, secret injection, skill loading,
  structured logs, cost events, session state); Approvals/execution policies; Routines (cron/webhook/API
  → каждое срабатывание создаёт issue); Activity log (immutable audit); Plugins (out-of-process
  workers, UI contributions); Secrets; Company export/import.
- **Как запускает агентов.** Adapters: Claude Code, Codex, Cursor/Gemini/bash CLI, OpenCode,
  HTTP/webhook (OpenClaw), внешние adapter-плагины. Агенты не «живут» в терминале — они просыпаются
  по heartbeat/событию (назначение задачи, @-mention), делают atomic checkout задачи и работают.
  Workspaces: project workspace, isolated execution workspaces (git worktrees, operator branches),
  runtime services (dev servers, preview URLs). Sandboxes: e2b, Cloudflare, Daytona, Modal, Novita,
  self-hosted Kubernetes (roadmap: ✅).
- **UI.** Dashboard, task manager (issues/inbox), org chart, costs/budgets, approvals, activity,
  skills studio, evals; mobile-ready UI.
- **Артефакты/ревью.** «Artifacts & Work Products» ✅, «Deep Planning (revisioned plans, plan
  approvals)» ✅, review gates, verify from diffs/screenshots/tests. Но прямо: «Not a code review
  tool. Paperclip orchestrates work, not pull requests. Bring your own review process.»
- **Трекер.** Собственный ticket system. «Bring-your-own-ticket-system (Asana / Linear / Jira as
  on-ramps)» — ⚪ roadmap, т.е. **синхронизации с GitHub Issues/Projects нет** (по README).
- **Бюджеты.** Token/cost tracking по company/agent/project/goal/issue/provider/model; hard stop,
  пауза агентов и отмена очереди при перерасходе.
- **Стек/хостинг.** TypeScript (основной), немного Rust; embedded PostgreSQL локально либо свой
  Postgres; Node.js 24.11+, pnpm 9.15+; `npx paperclipai onboard`; режимы local_trusted / lan / tailnet.
- **Лицензия/активность.** MIT; создан 2026-03-02; ~93k звёзд; push 2026-09-29; релиз v2026.916.1
  (2026-09-21); ~6000 открытых issues+PR — очень высокий шум/скорость изменений.
- **Зрелость/риски.** Критические уязвимости, опубликованы 2026-08-05: CVE-2026-41679 (CVSS 10,
  RCE через импорт агентов на сетевых инсталляциях), DNS rebinding в local_trusted (9.6), отсутствие
  auth на части API (8.3); исправлено в v2026.416.0
  (https://thehackernews.com/2026/08/paperclip-ai-flaws-let-attackers-run.html). Открытые PR про
  redaction секретов в логах запусков, «40+ hour outage» из-за UUID-валидации checkout (issue #4060).
- **Почему тяжело для соло-разработчика с GitHub-пайплайном.**
  1. Своя модель работы (компания/цели/org chart/heartbeat), а не ваш pipeline idea→spec→tickets→
     implementation→review→acceptance; второй tracker параллельно GitHub Issues/Projects.
  2. Агенты — автономные «сотрудники» по расписанию; у вас — интерактивные сессии в Herdr под
     контролем владельца. Heartbeat = LLM-вызов, расход трудно прогнозировать (обзор eesel:
     https://www.eesel.ai/blog/paperclip-ai-review — вторичный).
  3. Не code review tool — ревью PR всё равно в GitHub.
  4. Эксплуатация: сервер + Postgres + security-патчи быстро меняющегося проекта.
  5. Сам README: «If you have one agent, you probably don't need Paperclip».

## 2. Ландшафт инструментов

### 2a. Локальные desktop/TUI «workspace managers» (worktree на задачу, BYO CLI agent)

| Инструмент | Статус (GitHub на 2026-09-29) | Суть |
|---|---|---|
| **Conductor** (conductor.build) | коммерч., закрытый; YC S24, Series A $22M (вторичн.) | Mac-app; Claude Code, Codex, Cursor, OpenCode в изолированных workspaces (branch + worktree + terminals + diff), review diff → PR → merge → archive. Локально бесплатно, свои подписки. Главная страница теперь «Run a team of coding agents in the cloud» — облачные workspaces (детали не подтверждено). Win/Linux нет. https://www.conductor.build/docs/ |
| **Agent Orchestrator (AO)** Untrivial-ai/agent-orchestrator (ранее ComposioHQ) | Apache-2.0, 12.5k★, push 09-29, v0.13.1 (09-26); anonymous telemetry | Desktop+daemon+CLI; worker session на задачу (branch/worktree), **project orchestrator-агент** планирует и раздаёт задачи; **Kanban, где позиция карточки выводится из фактов session/PR/CI/review**; открыть worker: терминал, changed files, PR summary, reviews, preview, отправить CI/review feedback тому же агенту. Tracker adapters в коде: GitHub, GitLab. 25+ harness. https://github.com/Untrivial-ai/agent-orchestrator |
| **Emdash** generalaction/emdash (YC W26) | Apache-2.0, 5.9k★, push 09-29 | Worktree на агента, локально и по SSH; **задачи из Linear, GitHub, Jira, GitLab, Asana…**; diff, PR, CI checks, merge; ставит lifecycle hooks в агенты для статуса/уведомлений/resume. https://github.com/generalaction/emdash |
| **Superset** superset-sh/superset | **Elastic License 2.0** (source-available), 14.7k★, push 09-29 | «agentic IDE»: worktree+terminals, diff viewer, browser preview с портами на worktree, PR review → отправка строк агенту, «Pages» для отчётов с комментариями. https://github.com/superset-sh/superset |
| **Nimbalyst** (ex Crystal) nimbalyst/nimbalyst | MIT, 1.8k★, push 09-28. Crystal (stravu/crystal, 3.1k★) deprecated 2026-02 | Visual workspace: сессии в worktree, **session kanban**, task tracking в markdown-файлах в репо, WYSIWYG red/green diff по документам/мокапам/диаграммам, iOS companion с push. https://github.com/nimbalyst/nimbalyst |
| **Sculptor** (Imbue) imbue-ai/sculptor | MIT, 233★, push 09-28, «experimental research preview» | Workspaces (worktree), chat, Changes, PR tracking. |
| **Xum** (coder/xum, ex coder/mux) | AGPL-3.0, 2k★, push 09-29 | Собственный agent loop; worktree/remote runtimes; code review, agent status sidebar, costs, mermaid-планы. |
| **Vibe Kanban** BloopAI/vibe-kanban | Apache-2.0, 28k★; **компания Bloop закрыта 2026-04-10**, cloud (issues, projects, orgs) удалён; последний релиз 2026-04-24; push 09-19 (community) | Kanban issues → workspaces, inline-комментарии к diff → агенту, preview, 10+ агентов, PR. https://www.vibekanban.com/blog/shutdown |
| **Claude Squad** smtg-ai/claude-squad | AGPL-3.0, 8.5k★, push 2026-08-20 | TUI поверх tmux+worktree; Claude Code, Codex, OpenCode, Amp. |
| **parallel-code** johannesjo | MIT, 1k★ | Claude/Codex/Gemini бок о бок, worktree на каждого. |
| **Claude Code Agent Farm** | 919★, push 09-21 | Скрипт-фреймворк для 20+ Claude Code в tmux, lock-координация. Для sweep-задач, не для pipeline. |

### 2b. Терминальные среды (сессии как примитив) — ближе всего к текущему стеку владельца

- **Herdr** herdrdev/herdr — Apache-2.0, 41k★, push 09-29; у владельца стоит `herdr 0.9.1`.
  Persistent background server, статус агентов working/blocked/idle, multi-machine по SSH,
  socket API = CLI (`herdr api snapshot|schema`, `herdr agent list|get|read|prompt|wait`,
  `herdr worktree`, `herdr notification`), плагины, web client. https://herdr.dev/ .
  Экосистема: kcosr/herdr-web (143★), umutciloglu/herdr-session-manager (3★, v0.1 — поиск по всем
  сессиям Claude/Codex + agent-to-agent сообщения).
- **cmux** manaflow-ai/cmux — 27k★, push 09-29; Ghostty-based macOS terminal: вертикальные вкладки
  с branch, PR status/номером, портами, последним уведомлением; CLI+socket API; встроенный браузер.
  Лицензия: README «GPL», LICENSE в GitHub = NOASSERTION (не подтверждено).
- **Warp** (Oz cloud agents, «Warp Factories» live 2026-08 — вторичн., не подтверждено), оборачивает
  Claude Code/Codex/Gemini/OpenCode. https://docs.warp.dev/changelog/2026/

### 2c. Удалённый доступ / мобильные клиенты к локальным сессиям

- **Happy** slopus/happy — MIT, 24k★, push 09-28; mobile/web клиент Codex и Claude Code, E2E encryption.
- **CloudCLI / claudecodeui** siteboon — AGPL-3.0, 13.8k★, push 09-28; web UI для Claude Code,
  OpenCode, Cursor CLI, Codex.
- **opcode** (ex Claudia) winfunc/opcode — AGPL-3.0, 22k★, push 09-18; GUI для Claude Code, custom agents.
- **Omnara** — **пивот**: теперь «open-source alternative to Claude Managed Agents» (durable agents,
  sandboxes, Postgres state), Apache-2.0, 2.9k★. Больше не «командный центр для Claude Code» в
  исходном смысле. https://www.omnara.com/

### 2d. Облачные/вендорские панели

- **Codex app** (macOS, с 2026-02-02; OpenAI называет «command center for agents»): projects →
  threads, worktree на thread, review pane (inline diff, stage, revert, request changes),
  automations (local/worktree). Только Codex. https://openai.com/index/introducing-the-codex-app/
- **Claude Code desktop** (редизайн 2026-04-14): sidebar со всеми сессиями, фильтр по статусу/проекту/
  окружению, worktree на сессию (`.claude/worktrees/`), терминал, редактор, routines. Только Claude.
  https://claude.com/blog/claude-code-desktop-redesign , https://code.claude.com/docs/en/desktop
- **GitHub Agent HQ / Mission Control** — public preview Claude и Codex с 2026-02-04 для Copilot
  Pro+/Enterprise; задачи из issues (Assignees), Agents tab, VS Code; session logs «View session»;
  draft PR и итерации по review; 1 premium request на сессию. Облако (Actions), **родной GitHub
  tracker**. https://github.blog/changelog/2026-02-04-claude-and-codex-are-now-available-in-public-preview-on-github/
- **Cursor Cloud Agents** — VM в облаке (или self-hosted Enterprise), cursor.com/agents + Agents
  Window, триггеры Slack/GitHub/Linear/webhooks, видео-артефакты. https://cursor.com/blog/cloud-agents
- **Linear Agents** — не дашборд, а **модель данных**: AgentSession (states: working / awaiting
  input / error / complete), AgentActivity (thought, action, elicitation, response, error), plan как
  полный массив шагов. Хороший образец протокола статуса. https://linear.app/developers/agent-interaction
- **Devin** (release notes 2026 существуют: https://docs.devin.ai/release-notes/2026), **Factory**
  (Droids, Missions), **Google Jules** (вторичн.: нет changelog с 2026-03 — не подтверждено), **Amp**
  — все живы, но это свой агент + свой облачный UI; Claude Code/Codex CLI не оркестрируют (кроме Warp).

### 2e. Self-hosted control planes (класса Paperclip)

- **builderz-labs/mission-control** — MIT, 6.3k★, alpha; tasks inbox → execution → review →
  receipts, spend, fleet по runtime (OpenClaw, Claude Code, Codex), REST/OpenAPI, MCP, SSE; SQLite.
  README честно: «one agent on one machine already stays understandable from its native CLI».
- **Omnara** (после пивота) — инфраструктура для durable agents, не UI для разработчика.

### Умершие / свернувшиеся и почему

- **Terragon Labs** — закрыт 2026-02-09, «weren't able to reach the level of traction»; OSS-снимок
  terragon-labs/terragon-oss без поддержки. https://docs.terragonlabs.com/docs/resources/shutdown
- **Vibe Kanban / Bloop** — 2026-04-10, «vast majority are free users… couldn't find a business
  model»; cloud-часть удалена, локальная — community, по HN «stopped improving… annoying bugs».
- **Crystal** → переименован/заменён Nimbalyst (2026-02).
- **Omnara** — пивот из мобильного пульта Claude Code в инфраструктуру managed agents.
- Причина общая: вендоры (Codex app, Claude Code desktop, GitHub Agent HQ, Cursor) встроили
  «список сессий + worktree + diff» бесплатно; обёртки без своей ценности сверх этого не выжили.

## 3. Паттерны и боли пользователей

Боли (первоисточники — HN):
- **Узкое место — ревью, а не генерация**: «8 parallel cards means 8x the diffs to read»; «30 minutes of
  planning and 30 minutes of implementation… is too big to review» (HN, Kanbots, ~май 2026 по id —
  дата оценочная) https://news.ycombinator.com/item?id=48239413
- «What does the kanban interface add here?» — доска без связи с реальными фактами (PR/CI) не нужна.
- Нужен свой ключ и интеграция с GitHub при уходе сервиса (Terragon) — lock-in облачных оркестраторов
  https://news.ycombinator.com/item?id=46589735
- Рынок: «handoff» (один промпт → агент) vs «pipeline» (детерминированные шаги с human gates);
  коммерческие продукты почти все handoff, внутренние платформы крупных компаний — pipeline
  (Jack Kora, 2026-06-29, вторичн.: https://jackkora.com/p/mapping-the-ai-coding-orchestrator).
  Пайплайн владельца — это «pipeline»-класс, которого на рынке для соло почти нет.

Каталог функций хорошего командного центра (информационная архитектура):
1. **Портфель**: проекты → открытые задачи по стадиям pipeline (из трекера, а не своя БД).
2. **Карточка задачи = join фактов**: issue + стадия + ветка/worktree + сессии агентов (runtime,
   машина, статус) + PR + CI + review + артефакты стадии. Статус выводится из фактов (как AO), а не
   двигается руками.
3. **Живой статус агентов**: working / waiting-for-input / blocked / idle / error (Herdr, Linear
   AgentSession) + уведомление «нужен человек» + jump-to-pane.
4. **Лента «что сделал агент»**: transcript/summary, tool actions, изменённые файлы, стоимость.
5. **Артефакты стадий**: spec, план, delivery report, evidence (скриншоты/видео), handoff — рендер
   прямо в карточке.
6. **Ревью**: diff с комментариями → отправка обратно тому же агенту (Superset, Vibe Kanban, AO).
7. **Gates**: явные действия владельца «принять стадию / запустить следующую» с записью в трекер.
8. **Recent / quick-jump**: последние задачи, командная палитра, deep links в терминал/GitHub.
9. **Запуск**: «новая сессия по задаче» = создать worktree + ветку + агента с контекстом задачи.
10. **Открытые данные/API**: всё состояние в git/GitHub/файлах, UI — только проекция (Nimbalyst
    «plain files on disk»), чтобы пережить смерть инструмента.

## 4. Сравнительная таблица

| Инструмент | Для кого | Claude Code / Codex | Где | Источник задач | Worktree | Ревью/артефакты | API | Лицензия/цена | Активность | Lock-in |
|---|---|---|---|---|---|---|---|---|---|---|
| Paperclip | «AI-компании», автономные агенты | да/да (+Cursor, OpenCode, HTTP) | self-host server+Postgres | свой ticket system (BYO tracker — roadmap) | да | work products, plans, approvals; не PR-review | REST, plugins | MIT | 93k★, очень активен, CVE 10.0 в 2026-08 | высокий (своя модель) |
| Agent Orchestrator | разработчик/команда | да/да, 25+ | local desktop+daemon | GitHub, GitLab + orchestrator | да | PR, CI, reviews, preview в карточке | CLI, plugins | Apache-2.0, telemetry | 12.5k★, релиз 09-26 | низкий-средний |
| Emdash | разработчик | да/да | local + SSH | Linear, GitHub, Jira, GitLab… | да | diff, PR, CI, merge | — (не подтверждено) | Apache-2.0 | 5.9k★, активен | низкий |
| Conductor | разработчик на Mac | да/да (+Cursor, OpenCode) | local (+cloud — не подтв.) | свой ввод; Linear/GitHub — не подтв. | да | diff, PR, merge | — | закрытый, free local | активен, $22M | средний |
| Superset | разработчик | да/да | local | свой | да | diff, PR-feedback, preview, Pages | — | ELv2 | 14.7k★ | средний |
| Nimbalyst | разработчик, visual | да/да (+OpenCode) | local desktop + iOS | markdown-трекеры в репо | да | red/green diff, session kanban | MCP | MIT | 1.8k★ | низкий |
| Vibe Kanban | разработчик | да/да, 10+ | local | свой kanban | да | inline diff comments, preview | MCP | Apache-2.0 | компания закрыта 04-2026 | риск заброшенности |
| Herdr | терминальные power users | да/да, 25+ | local + SSH | нет | helpers | нет | socket API, plugins, web | Apache-2.0 | 41k★, активен | низкий |
| cmux | терминал на Mac | да/да | local | нет | нет | PR status в sidebar | CLI/socket | GPL (не подтв.) | 27k★ | низкий |
| Codex app | пользователи Codex | нет/да | local + cloud | свой | да | review pane | — | подписка OpenAI | активен | вендорный |
| Claude Code desktop | пользователи Claude | да/нет | local + cloud | свой | да | diff, терминал | — | подписка Anthropic | активен | вендорный |
| GitHub Agent HQ | команды на GitHub | да/да (+Copilot) | cloud (Actions) | **GitHub Issues** | ветки/PR | session log, draft PR | GitHub API | Copilot Pro+/Ent | preview с 02-2026 | средний (GitHub уже используется) |
| Cursor Cloud Agents | команды Cursor | свой агент | cloud VM | GitHub, Linear, Slack | ветки/PR | video artifacts | API | платно | активен | высокий |
| mission-control | self-host ops | да/да | self-host | свой | — | runs, review, receipts | REST/MCP/SSE | MIT, alpha | 6.3k★ | средний |
| Devin/Factory/Jules/Amp | команды | свои агенты | cloud | GitHub/Linear/Slack | облачно | свои UI | API | платно | живы | высокий |

## 5. Оценка вариантов для владельца

Ключевой вывод: готового инструмента, который (а) берёт задачи и стадии из **GitHub Issues/Projects**,
(б) знает **ваш версионированный pipeline и артефакты стадий**, (в) видит **живые Herdr-сессии
Claude Code и Codex** на нескольких репо, — нет. Рынок делится на «workspace managers» (worktree+diff+PR,
без pipeline) и «control planes» (своя БД, автономные агенты, свой трекер).

- **Adopt Paperclip** — не рекомендуется: второй трекер, модель «автономной компании» вместо
  owner-driven stages, не code review tool, тяжёлая эксплуатация и свежие критические CVE.
- **Adopt ближайший готовый: Agent Orchestrator (AO).** Ближе всего по идее: GitHub tracker adapter,
  карточки выводятся из фактов session/PR/CI/review, Claude Code+Codex, worktree, Apache-2.0.
  Пробелы: не знает ваших стадий (idea→spec→tickets…), артефактов (spec, delivery report, evidence,
  handoff) и owner gates; хочет сам запускать workers (конфликт с Herdr как местом жизни сессий);
  молод (v0.13), telemetry. Альтернатива того же класса — Emdash (шире трекеры, SSH, hooks).
- **Combine (рекомендуемо как первый шаг):** оставить Herdr runtime (+ herdr-web для обзора сессий),
  GitHub Projects как канон стадий (поле Stage/Status), GitHub PR как место ревью и delivery report,
  а один из workspace managers (AO/Emdash/Conductor) — опционально для diff/PR-итераций. Минус: 2–3
  окна, нет единой карточки «задача ↔ сессии ↔ артефакты».
- **Build thin (рекомендуемо, если нужен презентабельный единый центр):** read-mostly web-проекция
  без своей БД-истины: источники = GitHub GraphQL (Issues/Projects/PR/checks), `git worktree list` по
  репо из реестра, `herdr api snapshot` / `herdr agent list|read|wait` (статус, jump), файлы
  артефактов в ветке/PR (spec, report, evidence), transcripts Claude/Codex. Действия = тонкие команды:
  «открыть сессию по issue» (herdr worktree + agent start), «перевести стадию» (поле Project +
  комментарий), «отправить review-замечание агенту» (`herdr agent prompt`). Модель статуса взять у
  Linear AgentSession. Harness — источник определения стадий/артефактов (схема в репо), UI её только
  рендерит. Это «pipeline»-класс, которого на рынке для соло нет; риск — поддержка своего кода, но
  без собственной БД и с GitHub/Herdr как истиной lock-in и цена отказа минимальны.

## 6. Уточнение владельца: небольшие компонуемые инструменты (приоритет)

Владельцу не нужен продукт уровня Paperclip. Ниже — кирпичи, которые складываются вокруг уже
установленного Herdr (`herdr 0.9.1`), `gh` и небольшого своего веб-интерфейса.

### 6a. Платформа Herdr-плагинов (основа сборки)

Плагин = `herdr-plugin.toml` (id, version, min_herdr_version) + любой исполняемый код (Bash, JS,
Rust, Lua…). Умеет: actions (клавиши, `herdr plugin action invoke`), свои терминальные панели
(overlay/popup/split/tab), обработчики событий Herdr, link handlers (ctrl-click по ссылке → action),
доступ к CLI/socket API и env (`HERDR_PLUGIN_STATE_DIR`, ID pane/workspace). Без sandbox.
Реестр: https://herdr.dev/plugins — автоиндекс GitHub-репо с topic `herdr-plugin`
(https://assets.herdr.dev/plugins/index.json: 1375 плагинов на 2026-09-29). Гайд:
https://flaviocopes.com/herdr-plugins/ (вторичный, автор известен). Сам API: `herdr api schema|snapshot`,
`herdr agent list|get|read|prompt|wait|explain`, `herdr worktree`, `herdr notification`.

Звёзды/даты — из реестра Herdr на 2026-09-29; все ниже MIT, если не указано иное (проверено для
reviewr, radar, projects, roamgate, herdr-web).

| Плагин / инструмент | ★ / push | Что даёт | Роль в центре |
|---|---|---|---|
| **eliasstravik/herdr-projects** | 510 / 09-28 (создан 09-18) | coordinator-разговор + worker threads в своих worktree/branch; sidebar «what needs you», строка `review · PR #4`, `~40%`; агенты сами репортят прогресс; ticker следит за PR | ближе всего к «оркестратору внутри Herdr»; забрать модель «thread = задача», self-reported progress |
| **eliasstravik/herdr-agent-progress** | 31 / 09-15 | агент сам пишет прогресс/активность в sidebar | протокол статуса стадии от агента |
| **hhdebb/herdr-radar** | 102 / 09-28 | sidebar: working / ждёт вас / done-до-просмотра / 3 уровня idle, группировка по проекту, worktree под репо | готовый «кто где и кто ждёт меня» |
| **persiyanov/herdr-reviewr** | 790 / 09-23 | review-панель: diff (uncommitted / branch / last turn / commits), комментарии к строкам → агенту, read-only PR view, markdown preview | ревью в терминале, «last-turn diff» = «что агент сделал» |
| **jhochenbaum/herdr-hunk-diff** | 133 / 09-27 | ревью в hunk + inline-комментарии обратно агенту | альтернатива reviewr |
| **plannotator/herdr-annotate** | 580 / 09-27 | аннотировать ответы агента и документы (spec/план) → обратно агенту | ревью spec/плана на стадиях до кода |
| **tomasvarga/herdr-pickr** | 20 / 07-13 | ctrl-click по PR-ссылке → выбрать ревьюер-тул, опционально AI first pass | маршрутизация ревью |
| **wyattjoh/herdr-plugin-gh-pr** | 22 / 07-16 | статус PR ветки фокусного pane в sidebar | связка pane ↔ PR |
| **bredebjorhovd/herdr-board** | 1 / 08-14 | TUI-доска: GitHub issues/PR (+Linear) → dispatch в pane; BLOCKED/WORKING/READY/REVIEW/FAILED; PR review доставляется агенту-автору | **концептуально ближе всего к нужному**, но 1★ — брать как образец/форк, не как зависимость |
| nelsonPires5/herdr-board | 162 / 09-29 | kanban, карточки = промпты в видимые pane | своя доска, не GitHub |
| thanhdat77/herdr-navigator | 175 / 09-24 | fuzzy-переход к workspace/agent/project/session/action | quick-jump |
| andrewchng/herdr-sessionizer | 49 / 09-26 | открыть проект/worktree + TOML-раскладка tabs/panes/команд | лаунчер «сессия по задаче» |
| devashish2203/herdr-worktrunk | 164 / 09-21 | интеграция Worktrunk | worktree на задачу |
| tdi/herdr-worktree-setup | 27 / 09-11 | setup при создании worktree (.env, mise, direnv, deps) | готовность worktree |
| cloudmanic/herdr-plus (Go) | 340 / 09-04 | Projects + Quick Actions | проекты/быстрые действия |
| aemrebarut/herdr-dagr | 88 / 08-23 | swarm как live DAG: attempts, review gates, evidence | идея визуализации стадий/гейтов |
| deimantasnork/captains-deck | 29 / 09-28 | read-only flow kanban | образец read-only доски |
| furkankly/zoetrope | 956 / 09-15 | сессия Claude Code/Codex как live flow graph, терминал или браузер | «что делал агент» |
| nicosuave/memex | 234 / 09-22 | поиск по транскриптам Claude/Codex/…, resume, токены | история сессий |
| Davidcreador/herdr-token-dashboard | 23 / 09-14 | токены по pane | расход |
| **Веб/мобильные клиенты Herdr**: powerfooI/roamgate (256, TS; терминалы, агенты, файлы и diff-аннотации, desktop+mobile), kcosr/herdr-web (143, MIT; использует приватные API Herdr — хрупко), devswha/herdr-web-ui (35), 0cv/herdr-mobile-relay (261, Go; approve с телефона, push) | | | готовый веб-слой к сессиям; roamgate — кандидат на основу/форк |

Экосистема очень молодая (большинство создано 06–09.2026), у многих 1 автор — выбирать 2–4 плагина,
закреплять версии, критичную логику держать у себя.

### 6b. Не-Herdr кирпичи

- **gh-dash** dlvhdr/gh-dash — MIT, 12.6k★, push 09-22: TUI по PR/issues с секциями per-repo (YAML),
  **custom actions** (например, «запустить Herdr-сессию по этому issue»). https://github.com/dlvhdr/gh-dash
- **Worktrunk** max-sixty/worktrunk — 8.5k★, push 09-29 (Rust; лицензия в API NOASSERTION — не
  подтверждено): `wt switch/create/remove`, пути по шаблону, hooks. https://worktrunk.dev
- **ghzinga** osolmaz/ghzinga — 87★: кликабельный TUI одного issue/PR.
- **Agent Sessions** jazzyalex/agent-sessions — MIT, 882★: macOS app — поиск по локальным сессиям
  Codex/Claude/…, resume, quota/стоимость на сессию. **claude-code-log** daaain — 1.2k★: JSONL→HTML/MD
  транскрипта (для evidence/delivery report). **ccusage** — 18.8k★: расход.
- **coder/agentapi** — 1.5k★, push 09-13: HTTP API поверх Claude Code/Codex/… (альтернатива, если
  когда-нибудь уходить от терминала).
- **TUI-менеджеры сессий (альтернативы Herdr, не нужны при Herdr, но источники идей)**:
  agent-of-empires (MIT, 3.3k★; TUI+web, worktree/container), agent-deck (970★), ccmanager (1.25k★),
  amux (162★), seshagy (20★; tmux+herdr, zoxide-лаунчер).
- **Linear AgentSession** как схема статуса (не продукт): states + activity types + plan.
- **tsk** smarzban/tsk (151★): терминальный трекер «TUI для вас, CLI для агентов» — идея, но у
  владельца трекер GitHub.

### 6c. Что забрать у крупных продуктов (кратко)

| Продукт | Забрать идею / часть |
|---|---|
| Paperclip | atomic checkout задачи (одна задача — одна активная сессия); goal ancestry в брифе агента; routines → issue; immutable activity log; run = структурированный лог + cost event |
| Agent Orchestrator | **позиция карточки выводится из фактов** (session, PR, CI, review), а не двигается руками; «отправить CI/review feedback тому же агенту»; tracker adapter интерфейс (backend/internal/ports/tracker.go, Apache-2.0 — можно читать/заимствовать) |
| Emdash | установка lifecycle hooks в агенты для статуса/уведомлений/resume; задачи из GitHub/Linear |
| Conductor | жизненный цикл workspace: create → review → PR → merge → **archive**; «shared context folder» на workspace |
| Superset | превью dev server с портами на worktree; «Pages» — отчёт с комментариями, который агент переделывает по той же ссылке (аналог delivery report) |
| Nimbalyst | всё состояние — plain files в репо; session kanban со связью сессия ↔ файлы |
| Vibe Kanban | inline-комментарии к diff → агенту; урок: облачная часть без бизнес-модели умерла, локальная пережила |
| Codex app / Claude Code desktop | sidebar сессий с фильтром по статусу/проекту; review pane с revert/stage; automations |
| GitHub Agent HQ | задача = issue, запуск через Assignees, лог сессии привязан к PR — модель, совместимая с вашим трекером |
| Linear | AgentSession/AgentActivity как словарь статусов и «elicitation» (агент ждёт человека) |
| mission-control | «completion receipt» — итоговая квитанция задачи (что исполнено, что прошло ревью) |
| Cursor Cloud Agents | видео-артефакт как доказательство работы |

### 6d. Пересмотренная рекомендация (с учётом уточнения)

**Combine + тонкий свой слой, без платформы.**
1. Runtime и живой статус: Herdr + `herdr-radar` (кто ждёт) + `herdr-navigator` (прыжки).
2. Лаунчер «сессия по issue»: Worktrunk (+ `herdr-worktrunk`, `herdr-worktree-setup`) +
   `herdr-sessionizer`-раскладка; вызов из **gh-dash custom action** или своего Herdr-action
   `start-issue <repo>#<n>` → worktree + ветка + агент с брифом из issue/spec.
3. Ревью: `herdr-reviewr` (diff/last-turn/PR, комментарии → агенту), `herdr-annotate` для spec/плана;
   финальное ревью и delivery report — в GitHub PR.
4. «Портфель и стадии»: маленький свой веб-интерфейс (read-mostly, без своей БД): GitHub GraphQL
   (Project field Stage, issues, PR, checks) × `git worktree list` по реестру × `herdr api snapshot`
   × артефакты в ветке/PR. Действия — вызовы `gh` и `herdr` (перевод стадии, `agent prompt`,
   focus). Это та часть, которой нет ни у кого, и она знает ваш harness.
5. Как отправную точку для веба рассмотреть форк **roamgate** (TS, MIT; уже умеет терминалы, агентов,
   diff) или только позаимствовать его клиент к Herdr API; для доски — идеи `bredebjorhovd/herdr-board`
   и derived-status AO.
6. Протокол статуса от агента: skill/hook в harness пишет прогресс (как `herdr-agent-progress`) и
   ссылки на артефакты стадии в issue/PR-комментарий; центр только читает.

Риски: молодость и bus factor Herdr-плагинов (закреплять версии, минимум зависимостей); kcosr/herdr-web
использует приватные API Herdr; своё веб-приложение — поддержка, но при GitHub/Herdr/git как единственных
источниках истины его можно выбросить без потери данных.

Ограничения исследования: коммерческие продукты (Conductor cloud, Warp Factories, Jules) частично
по вторичным источникам; AO docs по плагинам не открылись (404), трекеры определены по дереву кода;
даты HN-тредов оценены по id.
