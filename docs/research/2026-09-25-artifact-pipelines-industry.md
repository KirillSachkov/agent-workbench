# Артефакты вместо чата: как ведущие практики coding-агентов строят разработку

Дата исследования: 2026-09-25. Метод: первичные источники (engineering-блоги вендоров, официальная
документация, исходники репозиториев). Репозитории читались на конкретных коммитах (указаны ниже).
Всё, что не удалось подтвердить первичным источником, помечено **«не подтверждено»**.

Ограничения доступа в этой сессии: `openai.com` и `developers.openai.com` напрямую не открывались
(сетевой блок), поэтому пост OpenAI прочитан по копии в Wayback Machine
(`web.archive.org/web/2026id_/https://openai.com/index/harness-engineering/`), а статья про ExecPlans —
по исходнику в `github.com/openai/openai-cookbook` (`articles/codex_exec_plans.md`).
Страница Anthropic «Building effective agents» в этой сессии не загрузилась (обрыв соединения),
поэтому по ней ниже только помеченные «не подтверждено» утверждения.

Главное различие, которое проходит через все разделы: **что проверяет код** (скрипт, схема, hook,
CI, тест) и **что держится только на тексте промпта** (прозой просим модель не трогать поле).
Везде, где это видно по исходникам, это отмечено явно.

---

## 1. Anthropic

### 1.1 «Effective harnesses for long-running agents» (Justin Young, 26 ноября 2025)

Источник: https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents ;
код: https://github.com/anthropics/claude-quickstarts/tree/main/autonomous-coding
(`prompts/initializer_prompt.md`, `prompts/coding_prompt.md`, `agent.py`, `progress.py`, `security.py`).

**(a) Артефакты и поля.**

- `feature_list.json` — массив записей вида (дословно из поста):
  ```json
  {
    "category": "functional",
    "description": "New chat button creates a fresh conversation",
    "steps": ["Navigate to main interface", "Click the 'New Chat' button", "..."],
    "passes": false
  }
  ```
  В демо claude.ai-клона — «over 200 features», все изначально «failing». Промпт инициализатора
  требует: минимум 200 записей, категории `functional` и `style`, «At least 25 tests MUST have 10+
  steps», порядок по приоритету, «ALL tests start with "passes": false».
- `claude-progress.txt` — свободный лог «what agents have done»; по `coding_prompt.md` после каждой
  сессии дописать: что сделано, какие тесты закрыты, найденные проблемы, что делать дальше, статус
  вида «45/200 tests passing».
- `init.sh` — скрипт запуска dev-сервера, чтобы следующая сессия не угадывала, как поднять приложение.
- Git: первый коммит инициализатора («Initial setup: feature_list.json, init.sh, and project structure»),
  далее коммит на каждую фичу с описательным сообщением.
- Скриншоты проверки (`verification/` упомянут в шаблоне commit message в `coding_prompt.md`).

**(b) Жизненный цикл, владение, валидация.**

- Инициализатор — это та же модель и тот же harness, но другой первый промпт (сноска 1 поста:
  «separate agents … only because they have different initial user prompts»). В `agent.py` выбор
  промпта сделан кодом: `is_first_run = not tests_file.exists()` — нет `feature_list.json`, значит
  запускается инициализатор.
- Кто может менять `feature_list.json`: только поле `passes`, только `false → true`.
  В посте: «It is unacceptable to remove or edit tests because this could lead to missing or buggy
  functionality». В промпте инициализатора: «IT IS CATASTROPHIC TO REMOVE OR EDIT FEATURES IN FUTURE
  SESSIONS». В `coding_prompt.md`: «YOU CAN ONLY MODIFY ONE FIELD: "passes"», «NEVER: Remove tests,
  Edit test descriptions, Modify test steps, Combine or consolidate tests, Reorder tests»,
  «ONLY CHANGE "passes" FIELD AFTER VERIFICATION WITH SCREENSHOTS».
- Выбор формата: «we landed on using JSON … the model is less likely to inappropriately change or
  overwrite JSON files compared to Markdown files».
- Протокол старта сессии: `pwd` → прочитать git log и progress → выбрать самую приоритетную
  непройденную фичу → запустить `init.sh` → прогнать базовый e2e-сценарий до новой работы (чтобы
  поймать сломанное состояние от прошлой сессии). Конец сессии: коммит + обновление progress.
- Проверка фичи: браузерная автоматизация через Puppeteer MCP «as a human user would».
- Переживание между контекстами: состояние целиком в файлах и git; новая сессия восстанавливается из
  `claude-progress.txt` + `git log` + `feature_list.json`.

**(c) Код против прозы.** Кодом обеспечено: выбор инициализатора по наличию файла; подсчёт прогресса
(`progress.py`, `count_passing_tests` просто считает `passes: true`); allowlist bash-команд через
security hook (`security.py`, `ALLOWED_COMMANDS`, `bash_security_hook`). **Не обеспечено кодом**:
неизменяемость описаний и шагов, запрет удаления записей, требование скриншотов перед `passes: true`.
Это держится только на тексте промпта; схемы JSON, diff-проверки или hook, запрещающего правку
других полей, в quickstart нет (проверено по исходникам).

**(d) Эффективность.** Количественных метрик в посте нет; есть качественная таблица четырёх
ошибок и их лечения (преждевременное «готово», грязное состояние, фича отмечена без проверки,
время на запуск приложения).

**(e) Ограничения.** В посте: Puppeteer MCP не видит браузерные `alert`-модалки, и такие фичи
выходили более сырыми; открытый вопрос, лучше ли мультиагентная схема; пример оптимизирован под
full-stack веб.

### 1.2 «Harness design for long-running application development» (Prithvi Rajasekaran, 24 марта 2026)

Источник: https://www.anthropic.com/engineering/harness-design-long-running-apps

**(a) Артефакты.** Три агента — planner, generator, evaluator.
- Продуктовая спецификация от planner: из промпта в 1–4 предложения — полный spec. В приложении к
  посту пример («RetroForge»): Overview, Features с нумерацией, в каждой фиче «User Stories: As a user,
  I want to …» и «Project Data Model». Planner намеренно держится уровня продукта и высокоуровневого
  дизайна: ошибки в детальной технической части «would cascade into the downstream implementation».
- **Sprint contract**: перед каждым спринтом generator предлагает, что построит и как это проверить;
  evaluator ревьюит; итерации до согласия. «Communication was handled via files: one agent would write
  a file, another agent would read it and respond either within that file or with a new file».
  Контракты гранулярные: «Sprint 3 alone had 27 criteria covering the level editor».
- Отчёт evaluator: по каждому критерию контракта PASS/FAIL с конкретикой (пример: «FAIL — Delete key
  handler at LevelEditor.tsx:892 requires both selection and selectedEntityId …»).
- Критерии оценки с порогами: «Each criterion had a hard threshold, and if any one fell below it, the
  sprint failed».

**(b) Механика.** Evaluator тестирует живое приложение через Playwright MCP (UI, API, состояние БД).
Evaluator калибровали few-shot примерами. Контекстные сбросы (clean slate + handoff-артефакт) были
нужны для Sonnet 4.5 из-за «context anxiety»; с Opus 4.5 их убрали, с Opus 4.6 убрали и спринты,
evaluator перевели на один проход в конце.

**(c) Код против прозы.** Пороги критериев и проход/провал спринта решает LLM-evaluator (инференс,
не детерминированная проверка). Формат файлов контракта в посте не показан — **не подтверждено**,
есть ли у контракта схема.

**(d) Эффективность.** Solo-прогон: 20 мин / $9; полный harness: 6 ч / $200, качество «immediately
apparent» лучше. V2 (Opus 4.6): 3 ч 50 мин / $124.70, разбивка по фазам в таблице поста.

**(e) Ограничения.** «Out of the box, Claude is a poor QA agent»: находил дефекты и уговаривал себя
их пропустить; тестировал поверхностно. Вывод поста: evaluator «is not a fixed yes-or-no decision» —
окупается только на задачах за пределами того, что модель делает сама; каждую новую модель стоит
заново проверять и снимать неработающие части harness.

### 1.3 «Building a C compiler with a team of parallel Claudes» (5 февраля 2026)

Источник: https://www.anthropic.com/engineering/building-c-compiler

- Артефакт-блокировка задачи: «Claude takes a "lock" on a task by writing a text file to
  current_tasks/ (e.g. … current_tasks/parse_if_statement.txt)». Конфликт двух агентов разрешает git
  («git's synchronization forces the second agent to pick a different one»). После работы: pull,
  merge, push, снять lock.
- Состояние для новых сессий: «extensive READMEs and progress files that should be updated frequently».
- Тестовый harness как главный валидатор: «the task verifier is nearly perfect, otherwise Claude will
  solve the wrong problem»; вывод компактный, подробности в лог; строки ошибок с `ERROR` на той же
  строке, чтобы находились grep'ом; режим `--fast` (детерминированная 1%/10% выборка на агента).
  Позже добавлен CI, чтобы «new commits can't break existing code»; GCC как оракул для ядра Linux.
- Цифры: 16 агентов, ~2000 сессий Claude Code, ~$20 000, 100 000 строк Rust, собирает Linux 6.9 на
  x86/ARM/RISC-V, 99% на большинстве компиляторных тест-сьютов.
- Ограничение из поста: «it is easy to see tests pass and assume the job is done, when this is rarely
  the case».

### 1.4 Claude Code: subagents, hooks, skills как производители и стражи артефактов

Источники: https://code.claude.com/docs/en/hooks , https://code.claude.com/docs/en/sub-agents ,
https://code.claude.com/docs/en/skills , https://code.claude.com/docs/en/memory

- **Hooks — единственный детерминированный слой.** Exit code 2 блокирует событие: `PreToolUse`
  блокирует вызов инструмента; `Stop` — «Prevents Claude from stopping, continues the conversation»;
  `SubagentStop` — не даёт субагенту завершиться; `TaskCreated` откатывает создание задачи;
  `TaskCompleted` — «Prevents the task from being marked as completed»; `PreCompact` блокирует
  компактизацию. `PostToolUse` блокировать не может (инструмент уже отработал) — только показывает
  stderr модели. JSON-ответ hook'а проверяется схемой; невалидный JSON при коде ≠2 — неблокирующая
  ошибка, а exit 1 **не** блокирует (важная ловушка). Отсюда паттерн «validator-as-hook»: скрипт
  проверяет артефакт (схема, ссылки, наличие поля) и возвращает 2 в `Stop`/`TaskCompleted`.
- **Subagents**: frontmatter `name`, `description` (обязательные), `tools`, `disallowedTools`, `model`,
  `permissionMode`, `maxTurns`, `skills`, `hooks`, `memory` (`user|project|local`), `background`,
  `isolation: worktree`. Субагент работает в собственном контексте и возвращает результат как отчёт;
  hooks субагента живут только пока он работает, а `Stop` в нём превращается в `SubagentStop`.
  Схемы для отчёта субагента в документации нет — возврат это текст.
- **Skills**: `SKILL.md` c frontmatter (`description`, `disable-model-invocation`, `user-invocable`,
  `allowed-tools`, `disallowed-tools`, `context: fork`, `hooks`, `paths`, `metadata` и др.). Skill
  может зарегистрировать hooks на остаток сессии — так «процедура» получает детерминированную проверку.
  После auto-compaction недавно вызванные skills переподключаются в пределах бюджета токенов.
- **Переживание контекста**: корневой `CLAUDE.md` перечитывается с диска после `/compact`
  («Project-root CLAUDE.md survives compaction»); инструкции, данные только в разговоре, теряются.

### 1.5 «The AI-Native SDLC playbook» (claude.com, 21 августа 2026)

Источник: https://claude.com/blog/the-ai-native-sdlc-playbook

Это ближайший найденный аналог «Claude Code для продуктовой разработки». Отдельный «технический
отчёт Claude Code for product development» в первичных источниках **не найден — не подтверждено**,
что такой документ существует (есть PDF «How Anthropic teams use Claude Code» и «2026 Agentic Coding
Trends Report», их содержание в этой сессии не проверялось).

**(a) Цепочка артефактов.** «Each stage ends by writing one to version control (including intent.md,
spec.md, plan.md, the diff and its tests, the PR with its review findings, and the incident record)
and the next stage begins by reading it».
- `intent.md` — прото-спека автора идеи: что нужно, зачем, при каких ограничениях.
- `spec.md` — требования + дизайн, одной сессией, с флагами «areas of concern»; коммитится рядом с
  `intent.md` («The file pair records what was asked for and what was decided»).
- `plan.md` — пример из поста:
  ```
  # Plan: claims status self-service (from intent.md 2026-06-02)
  ## Files that change
  ## Order of work
  ## Risks
  ## Proof
  ```
- PR с review findings; incident record, который пишет следующий `intent.md`.

**(b) Механика.** Коммит артефакта — триггер следующей стадии («An accepted intent.md triggers the
requirements and design pass, an approved spec.md triggers plan mode, a merged PR triggers the
pipeline…»). Plan mode сам не даёт править файлы до принятия плана. «When implementation departs from
the plan, update plan.md in the same commit. Consider using a hook to enforce synchronization». PR
review «checks the eventual diff against» `plan.md`. Для внешних систем: объявить один source of truth
на артефакт; минимальная планка — «All artifacts note the record ID and all legacy records contain the
commit SHA of the markdown file».

**(c) Код против прозы.** Пост прямо разделяет: «A skill is a control, though an advisory one …
A policy that must always hold needs something deterministic behind the skill, such as a hook».
Примеры hooks: блок правки тестовых файлов во время фикса («an agent fixing code must not be able to
weaken the check on that code»), блок правок миграций/инфраструктуры без change ticket, hooks как
approval gates на релизе. Регрессионный прогон конфигурации агента в CI «on any change to CLAUDE.md,
skills or hooks». Синхронизация `plan.md` и diff — только рекомендация (hook «consider»).

**(d) Эффективность.** Цифр нет; предложены метрики из git: время между коммитами `intent.md` и
`spec.md`; число коммитов `spec.md` после первого `plan.md` (переделки требований); доля изменений,
чей diff всё ещё соответствует `plan.md`.

**(e) Ограничения.** Пост признаёт, что в регулируемых организациях source of truth часто остаётся
в Jira/ServiceNow, и markdown-артефакты становятся копиями.

### 1.6 «Building effective agents» (Anthropic)

URL: https://www.anthropic.com/engineering/building-effective-agents . В этой сессии страница не
загрузилась, поэтому содержимое **не подтверждено**. По памяти: паттерны workflow (prompt chaining с
программными проверками-«gate» между шагами, evaluator-optimizer) и принцип опоры на «ground truth
from the environment». Цитировать без повторной проверки не рекомендуется.

---

## 2. OpenAI

### 2.1 «Harness engineering: leveraging Codex in an agent-first world» (Ryan Lopopolo, 11 февраля 2026)

Источник: https://openai.com/index/harness-engineering/ (прочитан через копию Wayback Machine).

**(a) Артефакты и структура.** «Repository knowledge [is] the system of record». `AGENTS.md` ~100
строк — «the table of contents», не энциклопедия. Дерево из поста:
```
AGENTS.md
ARCHITECTURE.md
docs/
├── design-docs/ (index.md, core-beliefs.md, …)
├── exec-plans/ (active/, completed/, tech-debt-tracker.md)
├── generated/ (db-schema.md)
├── product-specs/ (index.md, new-user-onboarding.md, …)
├── references/ (design-system-reference-llms.txt, nixpacks-llms.txt, uv-llms.txt, …)
├── DESIGN.md, FRONTEND.md, PLANS.md, PRODUCT_SENSE.md,
├── QUALITY_SCORE.md, RELIABILITY.md, SECURITY.md
```
Design docs каталогизированы «including verification status»; quality-документ «grades each product
domain and architectural layer, tracking gaps over time». «Plans are treated as first-class artifacts.
Ephemeral lightweight plans are used for small changes, while complex work is captured in execution
plans with progress and decision logs that are checked into the repository».

**(b) Механика.** Провал «одного большого AGENTS.md» описан по пунктам: вытесняет задачу из контекста,
«too much guidance becomes non-guidance», гниёт, «hard to verify … mechanical checks (coverage,
freshness, ownership, cross-links)». Отсюда progressive disclosure. Всё, что не в репозитории
(Google Docs, Slack, головы людей), для агента «doesn't exist». Recurring «doc-gardening» агент ищет
устаревшие документы и открывает fix-up PR. Фоновые задачи Codex «scan for deviations, update
quality grades, and open targeted refactoring pull requests» («garbage collection»).

**(c) Код против прозы.** «We enforce this mechanically. Dedicated linters and CI jobs validate that
the knowledge base is up to date, cross-linked, and structured correctly». Архитектура: слои
`Types → Config → Repo → Service → Runtime → UI`, сквозные вещи только через `Providers`, «enforced
mechanically via custom linters (Codex-generated) and structural tests»; статически проверяются
structured logging, именование схем и типов, лимиты размера файлов. «Because the lints are custom, we
write the error messages to inject remediation instructions into agent context». Правило эскалации:
«When documentation falls short, we promote the rule into code». Конкретные схемы/скрипты проверок
документации в посте **не показаны — не подтверждено**, что именно они проверяют сверх перечисленного.

**(d) Эффективность.** Пять месяцев, «0 lines of manually-written code», ~1 млн строк, ~1500 PR,
3 инженера (позже 7), «3.5 PRs per engineer per day», одиночные прогоны Codex «upwards of six hours».

**(e) Ограничения.** «Minimal blocking merge gates», флейки лечатся перезапуском — «would be
irresponsible in a low-throughput environment». Codex «replicates patterns that already exist …
even uneven or suboptimal ones»; раньше команда тратила каждую пятницу (20% недели) на уборку «AI
slop». Не знают, как архитектурная целостность поведёт себя на горизонте лет. Автономность «should
not be assumed to generalize without similar investment».

### 2.2 ExecPlans / `PLANS.md` (OpenAI Cookbook)

Источник: https://developers.openai.com/cookbook/articles/codex_exec_plans ; исходник
`openai/openai-cookbook/articles/codex_exec_plans.md` (последние правки 2026-01-13…15, Vaibhav Srivastav).

**(a) Поля.** Обязательные «living» секции: `Progress`, `Surprises & Discoveries`, `Decision Log`,
`Outcomes & Retrospective` («These are not optional»). Скелет также содержит `Purpose / Big Picture`,
`Context and Orientation`, `Plan of Work`, `Concrete Steps`, `Validation and Acceptance`,
`Idempotence and Recovery`, `Artifacts and Notes`, `Interfaces and Dependencies`. Форматы записей:
```
- [x] (2025-10-01 13:00Z) Example completed step.
- [ ] Example partially completed step (completed: X; remaining: Y).

- Observation: …
  Evidence: …

- Decision: …
  Rationale: …
  Date/Author: …
```

**(b) Механика.** Подключается секцией в `AGENTS.md`: «When writing complex features or significant
refactors, use an ExecPlan (as described in .agent/PLANS.md) from design to implementation».
Самодостаточность: читатель — «complete beginner … they have only the current working tree and the
single ExecPlan file»; «it should always be possible to restart from _only_ the ExecPlan and no other
work». При реализации не спрашивать пользователя «next steps», обновлять секции в каждой точке
остановки, «commit frequently». Каждый milestone «independently verifiable». При изменении плана
«write a note at the bottom of the plan describing the change and the reason why». Формат строгий:
один fenced-блок `md`, чек-листы разрешены только в `Progress`. Против «кода ради определения»:
«must produce a demonstrably working behavior, not merely code changes to "meet a definition"».

**(c) Код против прозы.** Всё — проза. Валидатора структуры ExecPlan, линтера секций или CI-проверки
в статье нет. (Пост harness engineering говорит про линтеры базы знаний, но связь с ExecPlan там не
раскрыта — **не подтверждено**.)

**(d) Эффективность.** «very similar to one that has enabled Codex to work for more than seven hours
from a single prompt» — единичное утверждение без методики.

**(e) Ограничения.** «ExecPlan» — произвольный термин, «Codex has not been trained on it»;
пользователю предлагают адаптировать секции.

---

## 3. GitHub spec-kit

Источник: https://github.com/github/spec-kit , коммит `adbd62a` (2026-09-24), версия 1.0.11 по
`CHANGELOG.md`. Файлы: `templates/*.md`, `templates/commands/*.md`, `scripts/bash/*`,
`workflows/README.md`, `workflows/ARCHITECTURE.md`, `workflows/speckit/workflow.yml`.

**(a) Артефакты и поля.**
- `.specify/memory/constitution.md` — принципы проекта; футер
  `**Version**: [CONSTITUTION_VERSION] | **Ratified**: … | **Last Amended**: …`. Команда
  constitution требует semver (MAJOR/MINOR/PATCH, PATCH — «Clarifications, wording, typo fixes») и
  «Sync Impact Report as an HTML comment at the top of the constitution file».
- `spec.md` (`templates/spec-template.md`): шапка `Feature Branch: [###-feature-name]`, `Created`,
  `Status: Draft`, `Input`; «User Scenarios & Testing *(mandatory)*» — истории с приоритетом P1/P2/P3,
  `Why this priority`, `Independent Test`, Given/When/Then; «Edge Cases»; «Requirements *(mandatory)*»
  с `FR-001…` («System MUST …»), маркер `[NEEDS CLARIFICATION: …]`; «Key Entities»; «Success Criteria
  *(mandatory)*» с `SC-001…`, «technology-agnostic and measurable»; «Assumptions».
- `plan.md`: `Technical Context` (Language/Version, Primary Dependencies, Testing, Target Platform,
  Performance Goals, Constraints, Scale/Scope — каждое поле может быть `NEEDS CLARIFICATION`);
  «Constitution Check — *GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*»;
  «Complexity Tracking — Fill ONLY if Constitution Check has violations that must be justified».
  Производные файлы: `research.md`, `data-model.md`, `quickstart.md`, `contracts/`.
- `tasks.md`: формат `[ID] [P?] [Story] Description`, например
  `- [ ] T012 [P] [US1] Create [Entity1] model in src/models/[entity1].py`; фазы Setup → Foundational
  → по одной фазе на user story (P1 = MVP) → Polish. `[P]` = параллельно (разные файлы, нет зависимостей).
- `checklists/*.md`: «CRITICAL CONCEPT: Checklists are UNIT TESTS FOR REQUIREMENTS WRITING» — проверяют
  качество требований, а не реализацию. `checklists/requirements.md` — встроенный чек-лист качества
  спеки (ведут `specify`/`clarify`); кастомные — «reviewer-owned», команда checklist «MUST NOT mark
  generated items `[x]`».

**(b) Жизненный цикл, связи, валидация.**
- Команды: `/speckit-constitution → specify → clarify → plan → tasks → analyze → implement`, плюс
  `checklist`, `converge`, `taskstoissues` и расширения (bug-assess/fix/test, assess-*).
- Привязка к фиче: `create-new-feature.sh` создаёт ветку `NNN-short-name` и каталог фичи; активная
  фича резолвится через `SPECIFY_FEATURE` или `.specify/feature.json` (`common.sh`). Привязки
  артефакта к SHA коммита нет (не найдено).
- `specify`: максимум 3 маркера `[NEEDS CLARIFICATION]` («LIMIT: Maximum 3»), самопроверка по
  `checklists/requirements.md` до 3 итераций, затем предупреждение.
- `analyze`: «STRICTLY READ-ONLY», межартефактная проверка `spec.md`/`plan.md`/`tasks.md` против
  конституции. Проходы: дубли, неоднозначность (vague adjectives, незакрытые `TODO`/`???`),
  недоспецификация, противоречие конституции (всегда CRITICAL — «not dilution, reinterpretation, or
  silent ignoring»), coverage gaps («Requirements with zero associated tasks», «Tasks with no mapped
  requirement»), inconsistency (terminology drift). Отчёт — таблица `ID | Category | Severity |
  Location(s) | Summary | Recommendation` + coverage-таблица; до 50 находок.
- `implement`: сначала сканирует `checklists/` и считает `- [ ]`/`- [x]`; при незакрытых пунктах
  «STOP and ask … proceed anyway? (yes/no)»; отмечает выполненные задачи `[X]` в `tasks.md`.
- `converge`: сверяет код с spec/plan/tasks и дописывает недостающую работу новыми задачами в `tasks.md`.
- Новый **workflow engine** (`specify workflow run speckit`): YAML-шаги с `type: gate`
  («Review the generated spec before planning», `options: [approve, reject]`, `on_reject: abort`);
  состояние после каждого шага пишется в `.specify/workflows/runs/{run_id}/state.json`, лог —
  `log.jsonl` (append-only), `specify workflow resume <run_id>` продолжает с места паузы.

**(c) Код против прозы.** Кодом: существование нужных файлов (`check-prerequisites.sh` падает с
`ERROR: plan.md not found`), нумерация фич и веток, резолв активной фичи, workflow engine с
persist/resume и gate-паузами. **Прозой (выполняет LLM)**: весь `analyze` (маппинг задач на требования
делается «by keyword / explicit reference patterns» самой моделью), подсчёт чек-листов в `implement`,
лимит маркеров, соблюдение конституции. Схемы/линтера `spec.md` в репозитории нет (не найдено).

**(d) Эффективность.** Контролируемых данных в репозитории нет (не найдено).

**(e) Критика.** Böckeler (раздел 5.1): «a LOT of markdown files … repetitive … very verbose and
tedious to review»; агент принял описания существующих классов из research за новую спецификацию и
сгенерировал дубликаты; «I'd rather review code than all these markdown files».

---

## 4. AWS Kiro

Источники: https://kiro.dev/docs/specs/ , https://kiro.dev/docs/specs/feature-specs/ ,
https://kiro.dev/docs/specs/bugfix-specs/ , https://kiro.dev/docs/specs/correctness/ (обновлено
2026-08-04), https://kiro.dev/docs/specs/analyze-requirements/ (обновлено 2026-09-02),
https://kiro.dev/docs/specs/best-practices/ , https://kiro.dev/docs/steering/ , https://kiro.dev/docs/hooks/

**(a) Артефакты.** Спека = три файла:
- `requirements.md` (или `bugfix.md`): user stories и acceptance criteria в EARS:
  `WHEN [condition/event] THE SYSTEM SHALL [expected behavior]`. Заявленные свойства: clarity,
  testability («Each requirement can be directly translated into test cases»), traceability.
- `bugfix.md` вместо requirements: `Current Behavior (Defect)`, `Expected Behavior (Correct)`,
  `Unchanged Behavior (Regression Prevention)`.
- `design.md`: архитектура, sequence diagrams, соображения реализации.
- `tasks.md`: дискретные задачи; по Böckeler, задачи «trace back to the requirement numbers».
- Steering: `.kiro/steering/*.md` (стартовые `product.md`, `tech.md`, `structure.md`); frontmatter
  `inclusion: always | fileMatch | manual`; также читается `AGENTS.md` (без inclusion-режимов).
- Hooks: `.kiro/hooks/<id>.json`, поля `trigger` (PascalCase), `matcher` (regex), action `command`
  или агентный промпт.

**(b) Механика.** Два варианта: Requirements-First (Requirements → Design → Tasks) и Design-First
(Design → Requirements → Tasks). Перед design — опциональный **Analyze Requirements**: ищет логические
противоречия, неоднозначности («large files», «fast response times»), конфликтующие ограничения,
неявные допущения, пропущенные edge cases; «takes minutes, not seconds»; вопросы идут в чат, ответы
правят `requirements.md`. Исполнение задач: UI показывает статус in-progress/completed; «run all
tasks» строит граф зависимостей и запускает независимые задачи параллельно. Изменения: правка
requirements/design и затем **Sync Files** в `tasks.md` создаёт задачи под новые требования. Specs
«designed to be version-controlled»; для нескольких команд — git submodules.
**Property-based testing**: Kiro «extracts properties from your EARS-formatted requirements …
generates hundreds or thousands of random test cases»; наведение на property показывает связь с
требованием и задачей; PBT «optional by default»; при падении — shrinking до минимального контрпримера.
Hooks: `Pre Tool Use`, `Prompt Submit`, **`Pre Task Execution` (перед задачей спеки) могут
блокировать**; `Post Task Execution`, `File Save`, `Agent Stop` — нет. Файловые триггеры реагируют
только на изменения агента, не на ручные.

**(c) Код против прозы.** Кодом: сгенерированные property-based тесты (реальные исполняемые тесты,
привязанные к требованию), hooks с блокировкой, граф зависимостей задач в IDE. Прозой/LLM: сам EARS
(формального парсера или линтера EARS в документации нет — **не подтверждено**, что он существует),
Analyze Requirements (LLM-рассуждение), Sync Files.

**(d) Эффективность.** Количественных данных в документации нет.

**(e) Ограничения.** Документация о PBT: «evidence of correctness, not a proof»; слабое свойство
пройдёт при неверном поведении; не всё сводится к свойствам. Böckeler: на мелком баге Kiro — «a
sledgehammer to crack a nut»: 4 user stories и 16 acceptance criteria; Kiro «mostly spec-first», без
ясной стратегии сопровождения спеки после задачи.

---

## 5. Другие методы 2025–2026

### 5.1 Birgitta Böckeler (Thoughtworks) на martinfowler.com — критическая рамка

- «Understanding Spec-Driven-Development: Kiro, spec-kit, and Tessl» (15 октября 2025):
  https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html . Три уровня: **spec-first**
  (спека до кода для задачи), **spec-anchored** (спека живёт и сопровождается вместе с фичей),
  **spec-as-source** (человек правит только спеку). «All SDD approaches … are spec-first, but not all
  strive to be spec-anchored or spec-as-source». Критика: избыточность для малых задач; ревью markdown
  тяжелее ревью кода; «False sense of control?» — агент и игнорирует инструкции, и «go way overboard
  because it was too eagerly following instructions (e.g. one of the constitution articles)»;
  недетерминизм генерации даже из одной и той же низкоуровневой спеки; параллель с MDD, который
  «never took off for business applications».
- «Harness engineering for coding agent users» (2 апреля 2026):
  https://martinfowler.com/articles/harness-engineering.html . Guides (feedforward) и sensors
  (feedback); computational (тесты, линтеры, тайпчекеры — «results are reliable») против inferential
  (LLM-review, «more non-deterministic»). Сенсоры сильны, когда сигнал оптимизирован для LLM: «custom
  linter messages that include instructions for the self-correction». Главная дыра — **behaviour
  harness**: обычно это «функциональная спецификация» как guide и AI-сгенерированный тест-сьют как
  sensor; «puts a lot of faith into the AI-generated tests, that's not good enough yet». «Correctness
  is outside any sensor's remit if the human didn't clearly specify what they wanted».

### 5.2 BMAD Method

Источник: https://github.com/bmad-code-org/BMAD-METHOD (HEAD на 2026-09-25; файлы
`skills/bmad-preview-ticketing/assets/story-template.md`, `skills/bmad-sprint-planning/scripts/sprint_plan.py`,
`skills/bmad-architecture/scripts/lint_spine.py`, `skills/bmad-architecture/assets/spine-template.md`,
`skills/bmad-build/sync-sprint-status.md`, `skills/bmad-build/compile-epic-context.md`).

**(a) Артефакты.**
- Story/ticket файл с YAML frontmatter: `id` (из `tickets.toml`), `type: story`, `title`, `parent`,
  `covers` (id требований эпика, которые история закрывает), `after` (пререквизиты:
  `<sibling id>`, `<epic id>.<entry id>`, `epic-<slug>`), `assignee`, `refined: false`, `hitl: false`,
  `risk`, `estimate`; `status` пишет сборка (`draft | ready-for-dev | in-progress | in-review | done |
  blocked`), `tracker_status` — синхронизация с трекером. Тело: `Description`, `Acceptance Criteria`
  (одна строка `Verify: …` либо нумерованные Given/When/Then), `Boundaries` («Must not change: …
  name behavior, not files»), `References`, `Notes` (`Decision:`, `Assumption:`, `Open question:`),
  `Plan` («Filled in by the coding agent; never sent to a tracker»).
- `sprint-status.yaml`: ключи историй вида `3-2-foo`, статусы story `backlog → ready-for-dev →
  in-progress → review → done`, epic `backlog → in-progress → done`, retrospective `optional → done`,
  `action_items`, `last_updated`.
- `ARCHITECTURE-SPINE.md`: блоки решений `### AD-n — {decision}` с полями `Binds`, `Prevents`, `Rule`
  («stable ascending id (never reused/renumbered)»), таблица `## Stack` с версиями.
- `epic-<N>-context.md`: собранный из PRD/архитектуры/UX контекст эпика для разработчика (Goal, Stories,
  Requirements & Constraints, Technical Decisions, UX…) — это нынешний аналог «шардинга» документов.
  Отдельной команды shard-doc в текущем HEAD не найдено; шардинг PRD/архитектуры — черта BMAD v4,
  в текущей версии **не подтверждено**.

**(b)+(c) Механика и код.** Здесь больше всего детерминированного кода среди SDD-методов:
- `sprint_plan.py` («Parse epic files and deterministically generate or refresh sprint-status.yaml»):
  подкоманды `generate`, `status`, `validate`; атомарная запись (temp, fsync, `os.replace`) и откат,
  «if post-write validation fails»; проверка «parseable, recognized keys, legal statuses, well-formed
  action_items»; отчёт о дрейфе (`in_sync`, `illegal`, `orphans`); статусы не понижаются, кроме
  явного repair через `--set`. Принцип разделения: «The LLM decides *which* files are epics (discovery
  is judgment); this script owns everything after that decision».
- `lint_spine.py`: «LLMs miscount IDs and miss literal placeholders; a grep does not». Проверяет
  плейсхолдеры (`TBD`, `TODO`, «similar to AD-n», незаполненные `{token}`), дубли и немонотонность
  `AD-n`, отсутствие `Binds/Prevents/Rule`, строки стека без версии. Выход — JSON, exit всегда 0,
  решение принимает Reviewer Gate.
- `sync-sprint-status.md` — процедура для модели: выставить статус истории, поднять эпик в
  `in-progress`, не понижать статус, обновить `last_updated`.
- У скриптов есть unit-тесты (`scripts/tests/test_*.py`).

**(d)** Метрик эффективности в репозитории нет (не найдено). **(e)** Метод сильно меняется между
версиями (legacy-статусы v6 `drafted`, `contexted` нормализуются скриптом), что само по себе риск
для долгоживущих артефактов.

### 5.3 Tessl

Источники: https://github.com/tesslio/spec-driven-development-tile (`docs/spec-format.md`,
`scripts/validate-specs.sh`, `scripts/check-spec-links.sh`, `.github/workflows/ci.yaml`, `tile.json`
v2.0.1); https://docs.tessl.io/codifying-and-enforcing-your-skill-standards/verifiers-overview.md ;
Böckeler (5.1).

- Формат спеки: файл `*.spec.md`, YAML frontmatter `name`, `description`, `targets` (пути/глобы
  описываемого кода; «All specs must have at least one target»), блок публичного API, требования со
  ссылками на тесты `[@test] ../tests/…`.
- Код: `validate-specs.sh` проверяет расширение, наличие frontmatter с `name/description/targets`;
  `check-spec-links.sh` — «that [@test] links and targets in .spec.md files point to existing files»;
  CI требует bump версии tile. Evals-сценарии для самого процесса (`evals/spec-drift-after-refactor`,
  `evals/skip-spec-pushback`, `evals/trivial-change-exception` и др.).
- По Böckeler, Tessl — единственный из трёх, кто целится в spec-anchored и пробует spec-as-source;
  код помечается `// GENERATED FROM SPEC - DO NOT EDIT`, теги `@generate`/`@test`.
- Сдвиг 2026: прежние страницы docs.tessl.io про spec-driven development отдают 404; актуальная
  документация — про skills/plugins registry, evals и **verifiers**: «LLM-as-judge check that compares
  committed files with an invariant stored as JSON», область и уровень (`info|warn|error`) в
  `tessl.json`, в CI `error` даёт non-zero exit и блокирует PR; «Prefer a linter or test when one can
  enforce the invariant».

### 5.4 Factory.ai

Источники: https://docs.factory.ai/cli/user-guides/specification-mode ,
https://docs.factory.ai/missions/overview , https://docs.factory.ai/missions/planning ,
https://factory.ai/news/using-linters-to-direct-agents (Alvin Sng, 5 сентября 2025).

- Spec Mode: read-only исследование, затем `ExitSpecMode` запрашивает одобрение. Опция «Save spec as
  Markdown» пишет одобренный план в `.factory/docs/YYYY-MM-DD-slug.md`.
- Missions: план = features, сгруппированные в milestones; «Validation workers run at the end of each
  milestone»; оценка `total runs ≈ #features + 2 * #milestones`; QA через запуск приложения одной
  командой и логи на диск. Формат хранения плана миссии в документации **не подтверждён**.
- Линтеры как закон: AGENTS.md объясняет «зачем», правила кодируются в lint с «clear severity,
  autofix, and waiver policies»; одни и те же правила на save, pre-commit, CI, PR bots и в toolchain
  агента; «Achieving "lint green" becomes the definition of "Done"». Категории: grep-ability,
  glob-ability, архитектурные границы, documentation signals. Линтеры как «migration engine».

### 5.5 Sourcegraph Amp

Источники: https://ampcode.com/news/handoff (2025-10-23), https://ampcode.com/news/neo (май 2026).
Handoff заменял компактизацию: пользователь задаёт цель, Amp генерирует черновик промпта нового
треда и список релевантных файлов, человек правит перед отправкой. В 2026 Amp развернулся обратно:
«Compaction now runs automatically when the context window is 90% full … So handoff is out».
Вывод для темы: даже продуктовые handoff-артефакты между сессиями оказались зависимыми от модели и
были отменены, когда модели стали лучше переносить компактизацию.

### 5.6 Cognition / Devin Playbooks

Источник: https://docs.devin.ai/product-guides/creating-playbooks . Playbook — «like a custom system
prompt for a repeated task». Секции: outcome, `Procedure` (setup → задача → delivery, каждый шаг с
глаголом действия), `Specifications` («Describe postconditions — what should be true after Devin is
done»), `Advice`, `Forbidden Actions`, `Required from User`. Это переиспользуемый артефакт-процедура;
механической проверки postconditions в документации нет (не найдено).

### 5.7 Geoffrey Huntley, Ralph (первоисточник цикла)

Источник: https://ghuntley.com/ralph/ (июль 2025). `while :; do cat PROMPT.md | claude-code ; done`;
артефакты цикла — `specs/*` и `@fix_plan.md` («a bullet point list sorted in priority»), «One item per
loop»; «phase two: backpressure» — тесты/сборка как обратное давление.

---

## 6. Matt Pocock: `mattpocock/skills` и Ralph

Источники: https://github.com/mattpocock/skills (HEAD `c55ee46`, 2026-09-18):
`skills/engineering/to-spec/SKILL.md`, `skills/engineering/to-tickets/SKILL.md`,
`skills/engineering/triage/AGENT-BRIEF.md`, `skills/engineering/wayfinder/SKILL.md`,
`skills/engineering/setup-matt-pocock-skills/triage-labels.md`, `docs/engineering/implement.md`,
`CHANGELOG.md`. Ralph: https://www.aihero.dev/tips-for-ai-coding-with-ralph-wiggum (обновлено
8 января 2026; прочитано через Wayback Machine).

**Переименования (важно для ссылок).** По `CHANGELOG.md`: «`to-prd` is renamed to `to-spec`»;
«`to-plan` and `to-issues` are merged into one `to-tickets` skill, and `to-issues` is deleted».
В текущем репозитории Ralph не упоминается (поиск по коду: 0 совпадений).

**(a) Артефакты.**
- Spec (`to-spec`): `Problem Statement`, `Solution`, `User Stories` («A LONG, numbered list … As an
  <actor>, I want a <feature>, so that <benefit>»), `Implementation Decisions` (модули, интерфейсы,
  схемы, API-контракты; «Do NOT include specific file paths or code snippets»), `Testing Decisions`
  (что считать хорошим тестом, какие модули, prior art), `Out of Scope`, `Further Notes`. Публикуется
  в трекер с меткой `ready-for-agent`. Перед этим согласуются «seams» тестирования («the ideal number
  is one»).
- Tickets (`to-tickets`): tracer-bullet вертикальные срезы, «sized to fit in a single fresh context
  window», у каждого **blocking edges**. Локально: по одному файлу на тикет
  `.scratch/<feature-slug>/issues/<NN>-<slug>.md` с полями `What to build`, `Blocked by`,
  `Status: ready-for-agent`, чек-лист критериев; в трекере — issue с `Parent`, `What to build`,
  `Acceptance criteria`, `Blocked by` и нативными blocking-связями. «Work the **frontier**: any ticket
  whose blockers are all done». Для широких рефакторингов — expand–contract.
- Agent Brief (`triage/AGENT-BRIEF.md`): структурированный комментарий при переводе в
  `ready-for-agent` — «the authoritative specification that an AFK agent will work from. The original
  body and discussion are context: the agent brief is the contract». Поля: `Category`, `Summary`,
  `Current behavior`, `Desired behavior`, `Key interfaces`, `Acceptance criteria`, `Out of scope`.
  Принцип «Durability over precision»: «Don't reference file paths: they go stale», не указывать номера
  строк; описывать интерфейсы, типы и поведенческие контракты.
- Wayfinder map: одно issue с меткой `wayfinder:map` — «an **index**, not a store»; решения живут в
  дочерних тикетах (`wayfinder:research|prototype|grilling|task`), тикет «sized to one 100K token agent
  session»; claim = назначение assignee до начала работы; блокировки — нативными связями трекера.

**(b) AFK против HITL.** Метки: `ready-for-agent` («Fully specified, ready for an AFK agent») против
`ready-for-human`. В wayfinder: «Every ticket is either **HITL** … or **AFK**»; «A HITL ticket only
resolves through that live exchange; the agent never stands in for the human's side». По changelog,
это исправило случаи, когда `/wayfinder` «grilling itself instead of the human».

**(c) Код против прозы.** Почти всё — проза и нативные механизмы трекера (метки, blocking-связи,
assignee как lock). Детерминированный код в репозитории: guardrail-hook для опасных git-команд
(`skills/misc/git-guardrails-claude-code/scripts/block-dangerous-git.sh`) и конфиг
dependency-cruiser в экспериментальном skill; валидатора spec/ticket нет. Документация `implement`
честно признаёт разрыв: «`implement` has no completion step … does not tick the `- [ ]` boxes on the
originating issue. Close the ticket and reconcile the criteria yourself» — и это ломает frontier,
«If nothing gets closed, nothing ever becomes visibly unblocked». Параллельные `/implement` в одном
checkout приводили к `commit --amend` на чужой коммит и пропаже stash.

**Ralph (aihero.dev).** Цикл `for ((i=1; i<=$1; i++))` вокруг `docker sandbox run claude -p
"@some-plan-file.md @progress.txt …"`; инструкции: выбрать самую приоритетную задачу, прогнать
feedback loops, дописать `progress.txt`, сделать коммит, «ONLY WORK ON A SINGLE FEATURE», при
завершении вывести `<promise>COMPLETE</promise>`. Скрипт (код) проверяет этот маркер в выводе и
выходит. HITL Ralph (`ralph-once.sh`, «Run once, watch, intervene») против AFK Ralph («Run in a loop
with max iterations»; «always cap your iterations … 5-10 … or 30-50»). PRD — JSON с полем `passes`
по образцу Anthropic: «The PRD becomes both scope definition and progress tracker». Состав
`progress.txt`: задача и ссылка на PRD-item, решения и причины, изменённые файлы, блокеры; «Don't keep
progress.txt forever … It's session-specific». Feedback loops: «The best setup blocks commits unless
everything passes. Ralph can't declare victory if the tests are red». HITL оставлен для рискованных
задач: «Use HITL Ralph for early architectural decisions — the code from these tasks stays forever».
Фраза про «"fixed" tests that now test nothing», встречающаяся в пересказах, в тексте статьи не
найдена — не подтверждено.

---

## 7. Сводная таблица: артефакт × производитель × потребитель × валидация × хранение

| Артефакт | Кто пишет | Кто читает | Валидация (код / проза) | Хранение, привязка к коммиту |
|---|---|---|---|---|
| `feature_list.json` (Anthropic) | агент-инициализатор | каждая coding-сессия; `progress.py` | код: только подсчёт `passes`; неизменяемость — проза; e2e через Puppeteer | файл в репо, коммит на каждую фичу |
| `claude-progress.txt` / `progress.txt` (Anthropic, Ralph) | каждая сессия, append | следующая сессия | нет | репо; у Ralph удаляется после спринта |
| `init.sh` | инициализатор | каждая сессия на старте | исполнение (сломался — видно сразу) | репо |
| Sprint contract / QA-отчёт (Anthropic 2026) | generator + evaluator | generator, evaluator | LLM-evaluator с порогами + Playwright | файлы; формат не опубликован |
| Lock-файлы `current_tasks/*.txt` (C compiler) | агент, взявший задачу | другие агенты | git (конфликт push) | bare git upstream |
| `intent.md` → `spec.md` → `plan.md` (Claude SDLC playbook) | автор идеи / PO+Claude / инженер в plan mode | следующая стадия, PR review | plan mode не даёт править до одобрения; hooks для гейтов; синхронизация plan↔diff — рекомендация | коммит = триггер стадии; legacy-записи хранят SHA markdown-файла |
| Hooks `Stop`/`TaskCompleted`/`PreToolUse` (Claude Code) | команда | runtime | код: exit 2 блокирует | `.claude/settings.json`, skills, subagents |
| `AGENTS.md` как оглавление + `docs/` (OpenAI) | Codex, doc-gardening агент | все агенты | код: линтеры и CI на свежесть, кросс-ссылки, структуру; кастомные линтеры архитектуры с remediation-сообщениями | репо |
| ExecPlan / `PLANS.md` (OpenAI) | агент | агент (включая «stateless» перезапуск) | только проза | `exec-plans/active` → `completed` |
| `QUALITY_SCORE.md`, `tech-debt-tracker.md` (OpenAI) | фоновые задачи Codex | люди, агенты | обновляются агентами; схема не опубликована | репо |
| `constitution.md` (spec-kit) | `/speckit-constitution` | `plan` (gate), `analyze` | semver и Sync Impact Report — проза | `.specify/memory/` |
| `spec.md` / `plan.md` / `tasks.md` (spec-kit) | команды specify/plan/tasks | `analyze`, `implement`, `converge` | код: наличие файлов, ветка `NNN-name`, workflow gates + `state.json`; содержательная проверка — LLM | `specs/NNN-name/`, ветка фичи |
| `checklists/*.md` (spec-kit) | `checklist`, ревьюер ставит `[x]` | `implement` | подсчёт чекбоксов — LLM, затем вопрос человеку | каталог фичи |
| `requirements.md` (EARS) / `bugfix.md` (Kiro) | Kiro + человек | design, tasks, PBT | Analyze Requirements (LLM); PBT — исполняемые тесты | `.kiro/specs/<feature>/`, git |
| `tasks.md` (Kiro) | Kiro, Sync Files | исполнитель задач | граф зависимостей, статус в UI; `Pre Task Execution` hook может блокировать | репо |
| Story frontmatter + `sprint-status.yaml` (BMAD) | SM/PM-агенты, build | dev-агент, ретро | код: `sprint_plan.py validate`, атомарная запись, запрет понижения статуса | репо, `last_updated` |
| `ARCHITECTURE-SPINE.md` `AD-n` (BMAD) | architect-агент | build, reviewer gate | код: `lint_spine.py` (плейсхолдеры, id, поля, версии); семантика — LLM | репо |
| `*.spec.md` с `[@test]` (Tessl) | spec-writer skill | генерация, verifiers | код: `validate-specs.sh`, `check-spec-links.sh`; verifiers (LLM-judge) в CI | репо, `targets` глобы |
| Spec Mode plan / Mission plan (Factory) | Droid | Droid, validation workers | валидаторы по milestone (LLM + запуск приложения); lint green как Done | `.factory/docs/YYYY-MM-DD-slug.md`; формат миссии не подтверждён |
| Handoff draft (Amp, 2025; отменён в 2026) | Amp | новый тред | человек правит черновик | тред |
| Playbook (Devin) | человек | сессии Devin | нет (postconditions прозой) | Devin web app |
| Spec / tickets / Agent Brief / wayfinder map (Pocock) | `to-spec`, `to-tickets`, `triage`, `wayfinder` | `/implement`, AFK-агент | метки и blocking-связи трекера; валидатора нет; `implement` не закрывает тикет | GitHub/Linear или `.scratch/<feature>/issues/NN-slug.md` |
| `prd.json` + `<promise>COMPLETE</promise>` (Ralph) | человек/агент в plan mode; агент отмечает `passes` | цикл | код: grep маркера в bash; feedback loops блокируют коммит, если так настроено | репо |

## 8. Сквозные выводы

1. **Надёжно то, что проверяет код.** Во всех источниках самые сильные гарантии дают исполняемые
   проверки: e2e/PBT-тесты, кастомные линтеры с сообщениями-инструкциями, hooks с exit 2, скрипты
   вроде `sprint_plan.py validate` и `lint_spine.py`. Инварианты «не редактируй описание фичи», «синхронизируй
   план с diff», «отметь чек-лист» почти везде остаются прозой — и именно здесь авторы сами фиксируют
   сбои (Böckeler, Anthropic QA-агент, Pocock `implement`).
2. **Разделение «суждение — LLM, учёт — скрипт».** BMAD формулирует это прямо («The LLM decides which
   files are epics … this script owns everything after that decision»); OpenAI — «promote the rule into
   code»; Tessl — «Prefer a linter or test when one can enforce the invariant».
3. **Неизменяемость редко обеспечена механически.** Anthropic выбирает JSON, потому что модель реже
   портит его, чем Markdown, но защиты в коде нет; spec-kit запрещает `checklist` ставить `[x]` —
   тоже прозой. Детерминированный запрет понижения статуса есть у BMAD; блок правки тестов во время
   фикса предлагается как hook в Claude SDLC playbook.
4. **Привязка к коммиту почти везде неявная** (артефакт в том же репо и коммите). Явная привязка
   встречается только как рекомендация: Claude SDLC playbook — commit SHA markdown-файла в legacy-записи,
   обновление `plan.md` в том же коммите, что и отклонение от плана.
5. **Переживание сессий** решается одним способом: файл в репо плюс git log (Anthropic, Ralph,
   ExecPlan, OpenAI `exec-plans/`, spec-kit `state.json`). Продуктовые механизмы handoff (Amp) оказались
   временными и зависят от поколения моделей; Anthropic тоже снял context resets и спринты при
   переходе на Opus 4.5/4.6.
6. **Критика объёма.** Много up-front markdown даёт ревью-нагрузку и ложное чувство контроля
   (Böckeler); ответы вендоров — сокращать спеку до решений и границ (Pocock: без путей к файлам,
   BMAD: одна строка `Verify:` по умолчанию), держать тикет в размере одного контекстного окна и
   переносить проверку в исполняемые тесты.
