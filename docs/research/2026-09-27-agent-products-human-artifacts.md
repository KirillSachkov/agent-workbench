# Как coding-agent продукты показывают работу агента человеку-ревьюеру

Состояние на 2026-09-27. Источники первичные, если не оговорено. Пометка «не подтверждено» — утверждение
взято из вторичного источника или из поисковой выдачи, а первичную страницу открыть не удалось.
Колонка «источник данных» везде различает: **исполнение** (артефакт порождён реальным запуском:
запись браузера, лог команды, check run) и **модель** (текст, который написала модель и который может
расходиться с фактом).

---

## 1. Google Antigravity — Artifacts (центральный кейс)

Хронология: запуск 18–20 ноября 2025 (IDE, форк VS Code, Editor + Manager view); Antigravity 2.0 —
Google I/O, 19 мая 2026: отдельное desktop-приложение, Go-CLI вместо Gemini CLI, SDK, Enterprise.
- https://antigravity.google/blog/introducing-google-antigravity (18.11.2025)
- https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/ (20.11.2025)
- https://techcrunch.com/2026/05/19/google-launches-antigravity-2-0-with-an-updated-desktop-app-and-cli-tool-at-io-2026/

### Зачем
Google формулирует проблему прямо: «Delegating work to an agent requires trust, but scrolling through raw
tool calls is tedious». Artifacts позволяют «verify the agent's logic at a glance»; UI показывает
«tool calls grouped within tasks», а не каждое действие.
https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/ ,
https://antigravity.google/blog/introducing-google-antigravity

Определение в docs: artifact — «a structured deliverable created by the agent to accomplish its task and
communicate its progress and thinking to the human user». Перечень типов: rich markdown plans, code diffs,
architecture diagrams, images, browser recordings. Производятся в основном в **Planning Mode** (в Fast
Mode планирования нет). https://antigravity.google/docs/artifacts/ , https://antigravity.google/docs/artifact-review/

### Типы артефактов

| Артефакт | Содержимое | Когда | Где | Источник данных |
|---|---|---|---|---|
| **Task List** | структурированный список шагов (checklist), обновляется по ходу | перед кодом и во время работы | review pane / sidebar | модель |
| **Implementation Plan** (`/plan`) | «technical details on what revisions are necessary and are meant to be reviewed by the user»; task breakdown + «verification checkpoints» | после analysis/discovery и уточняющих вопросов, до изменений | review pane; кнопка **Proceed** в разговоре и в шапке артефакта | модель |
| **Code diffs / Review Changes** | все накопленные за разговор диффы в отдельном editor pane | по ходу | кнопка `Review Changes` в нижней панели Agent panel | исполнение (git diff) |
| **Walkthrough** | «concise summary of the changes»; для браузерных задач — screenshots и screen recordings | после завершения реализации | review pane | текст — модель; медиа — исполнение |
| **Visual Screenshots** | снимок страницы или элемента, снятый browser subagent | автономно или по просьбе | как image artifact | исполнение |
| **Browser Recordings** | видео действий browser subagent; проигрывается в цикле | когда subagent «may choose to generate a recording» | внизу Browser step UI + как recording artifact | исполнение |
| Architecture diagrams (Mermaid) | диаграммы в markdown-артефактах | в плане | desktop; в CLI — Kitty graphics / ASCII / raw | модель |

Источники: https://antigravity.google/docs/implementation-plan , https://antigravity.google/docs/plan/ ,
https://antigravity.google/docs/walkthrough/ , https://antigravity.google/docs/screenshots ,
https://antigravity.google/docs/ide/browser-recordings/ , https://antigravity.google/docs/ide/review-changes-editor/ ,
https://antigravity.google/docs/cli/artifacts/

Разделы Implementation Plan. Официальные docs точных заголовков не дают. Сторонние воспроизведения
формата `implementation_plan.md` называют разделы Goal Description, User Review Required, Open Questions,
Proposed Changes (файлы: create/modify/delete) и Verification Plan; для отчёта используют `walkthrough.md`
(https://github.com/mohmaedeslam00116/cline/issues/42). **Не подтверждено** официальной документацией.
Официально подтверждено только одно: план содержит task breakdown и «verification checkpoints»
(https://antigravity.google/docs/plan/).

### Обратная связь и одобрение
- **Комментарии как в Google Docs** на текстовых артефактах и «select-and-comment feedback on screenshots».
  Обратная связь попадает в работу агента «without interruption».
  https://antigravity.google/blog/introducing-google-antigravity
- В плане можно оставлять inline-комментарии на отдельных шагах и просить изменить объём работы. Дальше
  есть два пути: **Proceed** или переключатель **Review**, который собирает все комментарии и отправляет
  их агенту. Агент либо переделывает план и снова просит review, либо начинает работу.
  https://antigravity.google/docs/implementation-plan , https://antigravity.google/docs/plan/
- На diffs в Review Changes тоже можно оставлять inline-комментарии, и агент их получает.
  https://antigravity.google/docs/ide/review-changes-editor/
- Практический пример Google Developer Advocate: комментарий к плану («FastAPI instead of Flask»), к
  task list («more detailed verification instructions»), к коду, к скриншоту в walkthrough («color theme
  from blue to orange»). Комментарии нужно явно отправить (submit), и они влияют на последующие шаги.
  https://atamel.dev/posts/2025/12-10_antigravity_provide_feedback/ (10.12.2025)
- **Artifact Review Policy**: `Request Review` — рекомендуемый и стандартный режим: агент всегда
  останавливается на плане или диффе и ждёт явного одобрения. `Always Proceed` — агент не
  останавливается. https://antigravity.google/docs/artifact-review/
- Отдельно существуют permission presets (Default / Request Review / Turbo) и правила Deny > Ask > Allow
  для `command(...)`, `read_url`, `execute_url`. Для браузера по умолчанию действует Ask.
  https://antigravity.google/docs/permissions/
- **CLI (2.0)**: `ctrl+r` открывает Artifact Picker. Файлы разделены на «Actionable Code Files»
  (код, конфиги, планы — нужно approve) и «Media Drawer» (PNG/JPG/WebP/SVG/MP4/WebM). Клавиши: `y`
  approve, `n` reject, `Shift+A`/`Shift+R` — массовые действия, `p` — превью, `c` — построчный
  комментарий в detail viewer (отметка 💬), `m` — режим Mermaid. Статус-бар подсказывает
  «/artifact to review». https://antigravity.google/docs/cli/artifacts/

### Ограничения
- Walkthrough пишет модель. Его текст не привязан к логам так, как в Codex, и официальные docs не
  описывают citations. Доказательной силой обладают только screenshots и recordings.
  https://antigravity.google/docs/walkthrough/
- Browser recording создаётся, когда subagent «may choose» его сделать, то есть не гарантирован.
  Docs не описывают формат записи и комментарии к видео (комментарии к screenshots описаны).
  https://antigravity.google/docs/ide/browser-recordings/
- Весь цикл review работает только в Planning Mode. Режим `Always Proceed` фактически отключает gate.
  https://antigravity.google/docs/artifact-review/
- Публичных первичных жалоб (например, «walkthrough утверждает то, чего не было») я не нашёл:
  **не подтверждено**.

---

## 2. OpenAI Codex

### Cloud tasks
- С запуска (май 2025) Codex «trained to provide verifiable evidence of its actions through citations of
  terminal logs and files». После задачи пользователь видит **diff view** и «comprehensive log of actions».
  Citations ведут на изменённые файлы и на выполненные терминальные команды, чтобы можно было «verify the
  outcomes of terminal commands, such as tests». System card, 16.05.2025:
  https://cdn.openai.com/pdf/8df7697b-c1b2-4222-be00-1fd3298f351d/codex_system_card.pdf
- Там же риск «**falsely claim to have completed a task**»: в ранних тестах Codex на невыполнимых задачах
  часто заявлял, что всё сделано. Меры: RL-штраф за «results inconsistent with its actions» и награда за
  признание ограничений. Доля корректных признаний «couldn't complete» выросла с 0.15 до 0.85; также
  помогает «User transparency and diff reviews». (тот же PDF, §2.3)
- По выдаче поиска: в summary используются file citations для изменений кода, а terminal citations — в
  секции Testing. Страница openai.com вернула 403, поэтому **не подтверждено** прямым чтением.
  https://openai.com/index/introducing-codex/
- Интерфейс cloud: «watch the task logs», «inspect the summary and diff», «open a pull request»,
  follow-up. В списке задач видны repo, branch, статистика `+31−1` и статус merge.
  https://learn.chatgpt.com/docs/cloud
- Upgrades (сентябрь 2025): Codex в облаке может «spin up its own browser, look at what it built, iterate,
  and attach a screenshot of the result to the task and GitHub PR». Взято из выдачи поиска, openai.com
  отдал 403, поэтому **частично не подтверждено**. https://openai.com/index/introducing-upgrades-to-codex/

### Codex app / review pane (2026)
- `/review`: сравнение с base branch, uncommitted changes, отдельный commit или custom instructions. Review
  pane показывает Unstaged / Staged / Commit / Branch / **Last turn** (только последние правки агента).
  Inline-комментарий добавляется через `+` на строке. Stage и Revert доступны для всего диффа, файла или
  hunk. При наличии `gh` рядом видны PR context и reviewer feedback.
  https://learn.chatgpt.com/docs/code-review?surface=app
- Встроенный браузер: annotation mode с комментариями к элементам и областям страницы. Агент делает
  screenshots, чтобы проверить отрисовку, а человек сравнивает страницу «alongside the code diff».
  https://learn.chatgpt.com/docs/browser
- Changelog сентября 2026: «agent command center» (группировка по модели, токены и оценка расхода),
  recaps с отдельным блоком «next actions», Mermaid прямо в ответах. https://learn.chatgpt.com/docs/changelog

### Codex code review (GitHub)
- Запуск через `@codex review` (реакция 👀) или Automatic reviews. Показываются только **P0/P1**. Правила
  берутся из `AGENTS.md`: в одном описании секция называется `## Code Review Rules`, во вторичных
  источниках — `## Review guidelines`. После review можно написать `@codex fix the P1 issue`, и Codex
  запустит cloud-задачу, которая отправит исправление в ветку.
  https://learn.chatgpt.com/docs/third-party/github.md
- Заявлено, что review «matches the stated intent of a PR to the actual diff» и «executes code and tests to
  validate behavior». Утверждение из выдачи поиска: **не подтверждено** прямым чтением.
  https://openai.com/index/introducing-upgrades-to-codex/

### Ограничения
Citations и summary пишет модель, но они ссылаются на реальные логи, и это главная защита. Сама OpenAI
признаёт риск ложного заявления о завершении задачи (system card).

---

## 3. Devin (Cognition)

| Артефакт | Содержимое | Источник данных |
|---|---|---|
| **Interactive Planning** (Devin 2.0, 03.04.2025) | за секунды: relevant files, findings, preliminary plan; план можно изменить до автономной работы | модель — https://cognition.com/blog/devin-2 |
| **Progress tab** | единая лента: shell-команды, правки кода, действия в браузере; клик по шагу открывает детали; по истории команд можно перемещаться во времени (будущие шаги серые) | исполнение — https://docs.devin.ai/work-with-devin/devin-session-tools |
| Shell / IDE / Browser | история команд с выводом, VS Code в реальном времени, браузер или desktop; есть takeover (read-only или writable) | исполнение — там же |
| **Test plan** (testing mode) | «single most important end-to-end flow», шаги, основанные на реальных code paths; план отправляют человеку | модель — https://docs.devin.ai/work-with-devin/testing-and-recordings |
| **Test recording** | запись экрана с аннотациями ключевых моментов, auto-zoom и сжатием простоев; приходит вложением в сообщение (webapp/Slack) | исполнение + аннотации агента — там же |
| **Test report** | «labeled screenshots from key moments»; видеоплеер с **chapters** и хронологическим списком **assertions: passed / failed / untested** | исполнение + разметка агента — https://cognition.com/blog/testing-development (29.05.2026) |
| **PR description** | наблюдаемый шаблон: Summary / Problem / Solution / **Review & Testing Checklist for Human** / Notes / **Link to Devin run** | модель; шаблон виден в реальном PR https://github.com/BerriAI/litellm/pull/43437 , официально не описан (**не подтверждено** как стандарт) |
| **Devin Review** | PR, перегруппированный логически, а не по алфавиту, с объяснением каждого hunk; распознаёт move/copy; Bug Catcher: red (probable bug) / yellow (warning) / gray (FYI); Flags (Investigate / Informational); security с CWE; вкладки Changes / Bugs / Flags / Description / Discussion / Commits; chat с контекстом кодовой базы; Auto-Fix | модель — https://docs.devin.ai/work-with-devin/devin-review , https://cognition.com/blog/devin-review |

Обратная связь: testing mode запускается кнопкой «Test the app» или настройкой «Pre-approve testing».
Правки из chat в Devin Review применяются как commit. Триггеры Devin Review: `/devin review` или
auto-review на open, push и ready-for-review. Для stacked PRs есть индикаторы готовности каждого слоя.
https://docs.devin.ai/work-with-devin/devin-review

Ограничения, которые признаёт сама Cognition: screenshots пропускают быстро исчезающий UI (toasts). Модели
«lean too heavily on executing JavaScript in the browser to trigger states programmatically instead of
clicking through the UI», то есть проверка может не соответствовать реальному пользовательскому пути.
Testing mode задуман как «quick sanity check», а не замена CI.
https://cognition.com/blog/testing-development , https://docs.devin.ai/work-with-devin/testing-and-recordings

---

## 4. Cursor (cloud agents, Cursor 3.x, Bugbot)

- **Cloud Agents with Computer Use**, 24.02.2026: каждый агент работает в своей VM, сам тестирует изменения
  и прикладывает к PR **artifacts: videos, screenshots, log references**. Человек может взять управление
  remote desktop, пощупать собранное ПО и вернуть управление агенту. По словам Cursor, «>30% of the PRs we
  merge at Cursor» создают автономные cloud-агенты.
  https://cursor.com/blog/agent-computer-use , https://cursor.com/docs/cloud-agent/capabilities
- Публикация artifacts в GitHub включается настройкой «Allow posting artifacts to GitHub». URL длинные и
  неугадываемые, но **открываются без аутентификации**, потому что этого требует image proxy GitHub.
  https://cursor.com/docs/cloud-agent/capabilities
- Жалоба из форума (26.02.2026): screenshots и видео не попадали в PR. Ответ сотрудника: постинг в GitHub
  «not yet supported». Функцию выпустили 26.03 как opt-in. Остаётся ограничение: artifacts рендерятся только
  в описании PR, созданного агентом, но не в комментариях и не в PR, созданных людьми.
  https://forum.cursor.com/t/cursor-cloud-agents-do-not-post-their-screenshots-or-videos-to-pr/152974
- **Plan Mode**: уточняющие вопросы, затем план с to-dos и путями файлов. План хранится в markdown (по
  умолчанию в home, «Save to workspace» кладёт его в репозиторий), его можно редактировать, дальше кнопка
  Build. https://cursor.com/docs/agent/plan-mode
- **Cursor 3** (02.04.2026): Agents Window, новый diffs view (review, stage, commit, управление PR).
  https://cursor.com/changelog/3-0 . **Agent Review** работает локально, автоматически после задачи или
  через `/agent-review`, в режимах Quick и Deep, и учитывает `BUGBOT.md`.
  https://cursor.com/docs/agent/agent-review
- **Bugbot**: комментарии в PR с полями Title, Severity (high/medium/low), Description, ссылками **Fix in
  Cursor** и **Fix in Web**. Check run принимает значения `success` / `neutral` (найдены проблемы) /
  `failure` (если включён fail-on-unresolved). Правила: `.cursor/BUGBOT.md`, learned rules, которые
  строятся из реакций, ответов и пропусков, отмеченных людьми (`@cursor remember`), и manual rules.
  Autofix запускает cloud agent и кладёт исправление в новую ветку или в текущую (не больше 3 попыток).
  https://cursor.com/docs/bugbot , https://cursor.com/blog/bugbot-learning , https://cursor.com/changelog/02-26-26

---

## 5. GitHub Copilot coding agent (сейчас «Copilot cloud agent»)

- **Session log**: «Copilot's internal reasoning and the tools it used to understand your repository, make
  changes, and validate its work». В overview видны токены и длительность. Лог стримится вживую, из PR
  открывается кнопкой «View session», есть событие «Copilot started work».
  https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/manage-and-track-agents
- **В сообщении каждого commit есть ссылка на session logs** «for code review and auditing». Commits
  подписаны (Verified), а соавтором указан человек, запустивший задачу.
  https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations
- **PR body**: агент раскладывает issue в checklist и отмечает пункты по мере работы
  (https://github.blog/ai-and-ml/github-copilot/assigning-and-completing-issues-with-coding-agent-in-github-copilot/).
  Title и body обновляются при ответах на feedback
  (https://github.blog/changelog/2025-07-30-copilot-coding-agent-keeps-pull-request-titles-and-bodies-up-to-date/),
  агент следует PR template (https://github.blog/changelog/2025-11-05-copilot-coding-agent-now-supports-pull-request-templates/).
- **Screenshots в PR**: Playwright MCP включён по умолчанию, и Copilot «share screenshots of what it has done
  in its pull request» (02.07.2025).
  https://github.blog/changelog/2025-07-02-copilot-coding-agent-now-has-its-own-web-browser/
- **Self-validation перед завершением** (28.10.2025): CodeQL, проверка новых зависимостей по Advisory DB,
  secret scanning и Copilot code review как «second opinion». Найденное агент пытается исправить, а
  результат описывает в PR summary. Набор проверок настраивается (03.2026).
  https://github.blog/changelog/2025-10-28-copilot-coding-agent-now-automatically-validates-code-security-and-quality/ ,
  https://github.blog/changelog/2026-03-18-configure-copilot-coding-agents-validation-tools/
- **Agents panel / Agents page / Agents tab** (mission control): список сессий, живые логи, **steering**
  без остановки агента (каждое сообщение расходует AI credits), Stop, Archive, переход в PR.
  https://docs.github.com/en/copilot/concepts/agents/cloud-agent/agent-management
- После сессии можно спросить Copilot Chat «what changed, what was validated, and why», и он ответит по
  session logs. (manage-and-track-agents, см. выше)
- **Gates**: GitHub Actions не запускаются, пока человек с write-доступом не нажмёт «**Approve and run
  workflows**». Агент не может approve или merge свой PR, и **тот, кто запустил агента, тоже не может его
  approve**. Агент пушит только в `copilot/*`. Комментарии пользователей без write-доступа агенту не
  передаются. https://docs.github.com/en/copilot/concepts/agents/cloud-agent/risks-and-mitigations
- Руководство GitHub для ревьюеров агентных PR (07.05.2026): блокировать любое ослабление CI; требовать
  тесты, которые падают на старом поведении, rollback plan и явный implementation plan.
  https://github.blog/ai-and-ml/generative-ai/agent-pull-requests-are-everywhere-heres-how-to-review-them/

---

## 6. Остальные продукты

### Google Jules
- Перед кодом показывается **plan approval**: reasoning и шаги, которые можно раскрыть и откомментировать
  в чате. Для **auto-approved plans** с 26.01.2026 работает **Planning Critic** — второй агент, который
  критикует и уточняет план до выполнения. Заявлено «9.5% reduction in task failure rates».
  https://jules.google/docs/changelog/2026-01-26-1/
- **Critic** (08.2025) проверяет итоговый патч в один проход. Он ничего не исправляет, только помечает
  проблемы и возвращает патч Jules. https://developers.googleblog.com/en/meet-jules-sharpest-critic-and-most-valuable-ally/
- **Activity feed**: шаги, вывод, ошибки, запросы feedback, мини-диффы и inline-объяснения изменений.
  Полный diff editor. **Итоговый summary**: изменённые файлы, runtime, строки added/changed, branch name,
  commit message. Кнопки Publish branch / Publish PR. Есть pause.
  https://jules.google/docs/code/ , https://jules.google/docs/running-tasks/
- Для frontend Jules отправляет **screenshot**, в base image есть Playwright (07.08.2025). Изображения
  рендерятся прямо в diff viewer (22.08.2025). Jules реагирует на комментарии в PR (23.09.2025) и сам
  чинит упавший CI (19.02.2026). https://jules.google/docs/changelog/

### Replit Agent
- **Task system**: board со столбцами **Drafts / Active / Ready / Done**. У каждой предложенной задачи есть
  title, description и детальный план через «**View plan**», в котором описано, «what it will do and what
  "done" looks like». Кнопки «Accept tasks» и «Revise plan». Готовая задача показывает **work log, test
  results и live preview**, дальше «Apply changes to main version» или «Dismiss».
  https://docs.replit.com/core-concepts/agent/task-system.md
- **App Testing**: агент сам решает, когда тестировать. Тестирование идёт в настоящем браузере с видимым
  курсором, затем агент выдаёт summary и исправляет найденное. После прогона доступен **interactive video
  replay** с навигацией по секциям. «Begin take over» используется для логина и CAPTCHA; если человек
  10 минут не отвечает, срабатывает Skip. Работает только для Full Stack JS и Streamlit.
  https://docs.replit.com/core-concepts/agent/app-testing.md
- **Checkpoints**: автоматические снимки кода, контекста разговора и БД на вехах, откат через history view.
  https://docs.replit.com/core-concepts/agent/checkpoints-and-rollbacks

### Claude Code
- **Cloud (claude.ai/code)**: индикатор `+42 -18` открывает diff view с inline-комментариями, которые
  уходят со следующим сообщением. Есть «Compare against» и **Create PR** (full, draft или страница compose с
  готовыми title и description). **CI status bar** с **Auto-fix**: агент подписывается на события PR,
  исправляет упавшие checks и review-комментарии. Если комментарий неоднозначен, агент спрашивает
  человека. Ответы в GitHub помечаются как написанные Claude Code. Сессию можно расшарить ссылкой.
  https://code.claude.com/docs/en/claude-code-on-the-web
- **Desktop**: Browser pane с preview dev-сервера и **auto-verify**: агент делает screenshots, инспектирует
  DOM, кликает и заполняет формы. Diff view с построчными комментариями, отправка через Cmd+Enter. Кнопка
  **Review code** ищет только high-signal проблемы. CI status bar с Auto-fix и Auto-merge. Отдельные panes
  для plan и tasks. Permission modes Manual / Accept edits / Plan / Auto / Bypass.
  https://code.claude.com/docs/en/desktop
- **Artifacts** (claude.ai/code/artifact): живая HTML-страница из сессии, которая обновляется на месте и
  хранит версии. Документированный сценарий — «Walk a reviewer through a pull request with annotated
  diffs» или вести «investigation timeline» по ходу длинной задачи. Страница private, пока ею не поделились.
  https://code.claude.com/docs/en/artifacts
- **Code Review** (managed): несколько агентов ищут проблемы, отдельный **verification step** проверяет
  кандидатов на реальном поведении кода. Severity: 🔴 Important / 🟡 Nit / 🟣 Pre-existing. У каждой
  находки раскрывается «extended reasoning… how it verified the problem». Check run «Claude Code Review»
  содержит таблицу Severity | File:Line | Issue, annotations в Files changed и машиночитаемый итог
  (`bughunter-severity`). Conclusion всегда **neutral**, то есть PR не блокируется. Настраивается через
  `REVIEW.md` и `CLAUDE.md`. https://code.claude.com/docs/en/code-review
- **claude-code-action**: трекинг-комментарий с чекбоксами, которые обновляются по мере работы
  (`track_progress`), и `use_sticky_comment`.
  https://github.com/anthropics/claude-code-action/blob/main/README.md
- **Hooks** (Stop и другие) позволяют запускать детерминированные проверки перед завершением. Spotify
  использует этот механизм, см. §7. https://code.claude.com/docs/en/hooks-guide

### Factory (Droids)
- **Specification Mode** (Shift+Tab): Droid пишет spec, человек одобряет, spec можно сохранить в репо.
  https://docs.factory.ai/cli/user-guides/implementing-large-features
- **Droid Control**: `/verify` проверяет behavior claim и выносит вердикт **CONFIRMED / REFUTED /
  INCONCLUSIVE** с evidence. `/demo` записывает side-by-side видео PR (до и после). `/qa-test` прогоняет e2e.
  Результат: step-level pass/fail table с inline evidence, screenshots или text snapshots терминала,
  отрендеренные видео. Дата не указана. https://docs.factory.ai/software-factory/droid-control

### Amp (Sourcegraph)
- Review panel (25.10.2025): выбор commit range, **AI summary**, «**tour**» с рекомендуемым порядком чтения
  файлов, редактируемые full-file diffs, commit message.
  https://ampcode.com/news/review
- Agentic Review (18.12.2025): summary по каждому файлу и по changeset, отдельный review agent со списком
  actionable improvements, которые можно передать основному агенту. Команда Amp прямо называет открытым
  вопрос «How do reviews map to threads?». https://ampcode.com/news/agentic-code-review
- **Checks** (04.02.2026): `.agents/checks/*.md` — пользовательские инварианты с областью действия по
  каталогу. Для каждого check запускается **отдельный агент**, «a stronger guarantee that each check will
  actually be checked». https://ampcode.com/news/liberating-code-review
- Diffs (16.06.2026): review диффа любого thread на desktop и mobile, duplicate block detection.
  https://ampcode.com/news/diffs

### Linear — Agent Interaction Guidelines и Agent Session
- **AIG** (30.07.2025), шесть принципов: disclose agent identity; inhabit the platform natively; provide
  instant feedback; be clear and transparent about internal state (thinking / waiting / executing /
  complete); respect requests to disengage; «an agent cannot be held accountable» — ответственность
  остаётся у человека. https://linear.app/developers/aig
- **Agent Session**: состояния `pending | active | error | awaitingInput | complete | stale` выводятся
  автоматически из последней activity. Типы activity: `thought`, `elicitation`, `action` (поля `action`,
  `parameter` и опциональный `result`), `response`, `error`, а также `prompt` от человека. `thought` и
  `action` можно сделать **ephemeral**. **Agent Plan** — чеклист уровня сессии, у шагов есть `content` и
  статус `pending | inProgress | completed | canceled`, обновляется только целиком. `externalUrls` —
  подписанные ссылки на dashboard агента. Первая activity должна прийти в течение 10 секунд, ответ на
  webhook — в течение 5 секунд. https://linear.app/developers/agent-interaction
- **Signals**: `stop` (человек → агент: немедленно остановиться и подтвердить это через response или
  error), `auth` (elicitation с «Link account»), `select` (elicitation со списком вариантов, можно ответить
  свободным текстом). https://linear.app/developers/agent-signals

---

## 7. In-house системы

- **Stripe Minions** (Part 1 и Part 2, февраль 2026; 1 000, затем 1 300+ merged PR в неделю): «completely
  minion-produced, human-reviewed». PR оформляется по **Stripe PR template**. Локальный lint (<5 с) как
  детерминированный узел blueprint работает как shift-left. **Не больше 2 раундов CI**: первый прогон,
  автоприменение autofixes, второй прогон, затем передача человеку даже с непочиненными падениями. **Web UI**
  показывает «the decisions and actions the minion took», инженер может дать «further instructions» до
  review. https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents ,
  https://stripe.dev/blog/minions-stripes-one-shot-end-to-end-coding-agents-part-2
- **Spotify Honk** (Part 1–3, ноябрь–декабрь 2025): детерминированные **verifiers** (например, Maven при
  наличии `pom.xml`) возвращают сжатый результат success или failure. Они доступны как tool и
  **автоматически запускаются через stop hook перед открытием PR**: при провале PR не открывается. Поверх
  них работает **LLM-judge** (diff + исходный prompt), который ветирует примерно 1 из 4 сессий, и примерно
  половина из них исправляется. Для judge ещё не сделали evals. Traces пишутся в MLflow. PR идут через
  обычный Fleet Management без особого оформления.
  https://engineering.atspotify.com/2025/12/feedback-loops-background-coding-agents-part-3 ,
  https://engineering.atspotify.com/2025/11/spotifys-background-coding-agent-part-1 .
  К QCon London (март 2026) judge **убрали**, потому что хватает verification steps в prompt. Также
  появились PR inbox и auto-merge для docs. Это пересказ доклада в InfoQ, **вторичный источник**.
  https://www.infoq.com/news/2026/03/spotify-honk-rewrite/
- **Shopify**: security harness присылает **draft PR с тестом, доказывающим эксплуатируемость**. Verifier
  работает на другой модели, чем Hunting agent (adversarial review). Недоказанные находки отклоняются или
  понижаются. Credentials, Git и storage обслуживает детерминированный код.
  https://shopify.engineering/building-an-agentic-harness-that-outlasts-the-model (июль 2026).
  **River**: работает только в публичных Slack-каналах, промежуточные находки публикуются в thread, session
  logs хранятся в Postgres. За 30 дней 3 536 merged PR. https://shopify.engineering/under-the-river (28.05.2026)
- **Uber**: uReview. Генерация, затем confidence-фильтр, dedup и подавление категорий по истории
  feedback. Кнопки Useful / Not Useful, useful >75%, исправляется 65% комментариев.
  https://www.uber.com/us/en/blog/ureview/ (12.08.2025). Software Factory: >70% PR с участием агентов,
  метрики качества managed agents (revert rate, F1, MTTR).
  https://www.uber.com/us/en/blog/efficient-software-factory/ (27.08.2026). Фоновая платформа «Minion» —
  **не подтверждено** первичным источником Uber (упоминание только у Pragmatic Engineer:
  https://newsletter.pragmaticengineer.com/p/how-uber-uses-ai-for-development).

---

## 8. Каталог паттернов

| Паттерн | Что содержит | Продукты | Что делает его доверенным |
|---|---|---|---|
| **Plan / Spec** (до кода) | цель, затрагиваемые файлы, шаги, как будет проверено, открытые вопросы | Antigravity Implementation Plan, Devin Interactive Planning, Cursor Plan Mode, Jules plan, Replit View plan, Factory Spec Mode, Claude Code Plan mode | явный gate (Proceed, Accept, Build); комментарии на уровне шага; критик плана (Jules Planning Critic); наличие раздела верификации, то есть заранее заявленного критерия «done» (Replit, Antigravity) |
| **Task list / Plan checklist** (живой) | шаги со статусами | Antigravity Task List, Linear Agent Plan (`pending/inProgress/completed/canceled`), Copilot PR checklist, claude-code-action tracking comment, Replit board | обновление по факту выполнения; статусы из фиксированного набора; `canceled` вместо тихого исчезновения шага |
| **Walkthrough / Summary** (после) | что изменено, зачем, как проверить | Antigravity Walkthrough, Codex summary, Jules summary, Copilot PR body, Devin PR description, Amp AI summary и tour | **citations на логи и файлы** (Codex); ссылка на session log (Devin «Link to Devin run», Copilot — в каждом commit); без привязки остаётся просто текстом модели |
| **Diff, организованный для чтения** | порядок чтения, группировка, move/copy detection, объяснение hunk | Devin Review, Amp tour и Diffs, Codex «Last turn», Antigravity Review Changes | детерминированный diff плюс объяснения модели; inline-комментарии возвращаются агенту |
| **Check results с citations** | тесты, линтеры, CI, security | Codex terminal citations, Copilot self-validation (CodeQL, secret scanning, deps), Stripe CI ≤2 раунда, Spotify verifiers, Replit test results, Bugbot и Claude Code Review check runs | результат порождён исполнением, а не моделью; check run в CI; verifier в stop hook блокирует PR |
| **Evidence media** | screenshots, видео, записи браузера | Antigravity screenshots и recordings, Devin test video и report, Cursor artifacts, Copilot screenshots (Playwright), Codex screenshots, Jules screenshots, Replit replay, Factory `/demo`, Claude Desktop auto-verify | снято с реального запуска; аннотации и chapters; **список assertions passed / failed / untested** (Devin); side-by-side до и после (Factory) |
| **Verdict на claim** | утверждение, вердикт, evidence | Factory `/verify` CONFIRMED / REFUTED / INCONCLUSIVE, Claude Code Review «how it verified» | допускает отрицательный и неопределённый результат; evidence приложено |
| **Preview / live result** | ссылка на работающее приложение или remote desktop | Replit live preview, Cursor remote desktop takeover, Claude Desktop Browser pane, Codex in-app browser, Devin Browser tab | человек проверяет сам и не зависит от рассказа агента |
| **Session log / timeline** | все команды, правки, браузер | Devin Progress (с переходами во времени), Copilot session log, Codex task logs, Jules activity feed, Stripe web UI, Linear activities, Shopify River Slack thread | лог сырой и неизменяемый, с доступом из PR и commit |
| **Findings review** | находки с severity | Bugbot, Codex P0/P1, Devin Bug Catcher и Flags, Claude Code Review 🔴🟡🟣, uReview, Amp Checks | severity; отдельный verification step; обучение на реакциях (Bugbot learned rules, uReview); отдельный агент на каждый инвариант (Amp Checks); не блокирует merge без явной настройки |
| **Open questions / Elicitation** | вопрос человеку, выбор вариантов | Linear `elicitation` + `select` и `auth`, Antigravity/Cursor clarifying questions, Claude auto-fix «asks you before acting», Devin просит secrets | структурированный вопрос с вариантами; состояние `awaitingInput` видно в списке |
| **Human checklist** | что человек должен проверить сам | Devin «Review & Testing Checklist for Human» (наблюдаемо), GitHub guidance: тест, падающий до изменения, и rollback plan | явно отделяет «проверено агентом» от «проверить человеку» |
| **Approval gates** | кто и что одобряет | Antigravity Request Review, Copilot «Approve and run workflows» и запрет самоодобрения, Replit Apply/Dismiss, Stripe и Spotify: только человек мержит | gate на стороне платформы, а не в тексте prompt |
| **Rollback / checkpoints** | возврат к состоянию | Replit checkpoints (код, контекст, БД), Codex revert по hunk, Antigravity undo | детерминированное восстановление |

Общие выводы:
1. Доверие держится на связи текста модели с артефактами исполнения: citations у Codex, ссылки на
   сессию в commit у Copilot и Devin, check runs, видео. Walkthrough без таких связей остаётся «рассказом».
2. Верификация смещается в инфраструктуру: stop-hook verifiers (Spotify), лимит раундов CI (Stripe),
   self-validation (Copilot), verification step в review (Claude Code Review). LLM-judge как отдельный слой
   Spotify в 2026 убрал (по данным вторичного источника).
3. Видео доказывает многое, но им легко злоупотребить. Cognition признаёт, что агент может выставлять
   состояние через JS вместо кликов. Лучшие реализации добавляют assertions list и test plan, то есть
   показывают, что именно проверялось.
4. Почти везде обратная связь идёт через inline-комментарии к артефакту (план, diff, screenshot, элемент
   страницы), и агент получает их в следующем ходе.
5. Нерешённые вопросы, которые признают сами вендоры: как review соотносится с threads (Amp); публичность
   URL artifacts (Cursor); findings не блокируют merge (Claude Code Review, Bugbot `neutral`), а gating
   нужно собирать самостоятельно.
