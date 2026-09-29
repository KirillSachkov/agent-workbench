# Research 4 — где и как смотреть артефакты стадий (review surfaces)

Дата: 2026-09-27. Источники — официальная документация, changelog, репозитории; вторичные помечены.
«не подтверждено» — не удалось проверить по первичному источнику в рамках этого прогона.

Вопрос: владелец не смотрит агентов вживую, а принимает готовые артефакты стадий (spec, ticket
breakdown, delivery report с тестами/скриншотами/видео, handoff). Нужна поверхность, которая
работает везде, где стоит harness, для Claude Code и Codex, локально до PR и позже на GitHub.

---

## 1. Vendor-native поверхности

### 1.1 Google Antigravity — Artifacts
- **Что видит владелец:** Task list, Implementation Plan (markdown), Walkthrough (сводка изменений,
  как проверить), code diffs, скриншоты, browser recordings.
  https://antigravity.google/docs/artifacts/ , https://antigravity.google/docs/walkthrough/ ,
  https://antigravity.google/docs/ide/implementation-plan/
- **Feedback:** inline-комментарии в стиле Google Docs на plan / task list / diff / walkthrough;
  «provide inline text feedback to steer the agent… before it modifies any local files». Агент
  получает комментарии в той же conversation после явного «proceed». Вторично:
  https://atamel.dev/posts/2025/12-10_antigravity_provide_feedback/
- **Хранение/версии:** markdown-файлы в `~/.gemini/antigravity/brain/<conversation-id>/`
  (`task.md`, `implementation_plan.md`, `walkthrough.md` + `.metadata.json`) — вне репозитория,
  привязаны к conversation. Источник вторичный (https://kerrick.blog/posts/2026/a-power-user-guide-for-google-antigravity/ ,
  https://github.com/michaelw9999/antigravity-cli ) — официально **не подтверждено**.
- **Claude/Codex:** нет — только агент Antigravity. Для этого контекста — образец UX, не поверхность.
- **Риски:** lock-in в IDE, артефакты вне git, обратная связь живёт только в сессии.

### 1.2 Claude — Artifacts из Claude Code (claude.ai/code/artifacts)
Официально: https://code.claude.com/docs/en/artifacts
- **Что видит:** живая HTML- или Markdown-страница на приватном URL claude.ai; `.md` рендерится
  как стилизованный документ с подсветкой кода. Галерея — https://claude.ai/code/artifacts ;
  `/artifacts` в CLI (v2.1.208+) — список, открыть, прикрепить к сессии.
- **Версии:** каждый publish — версия; в Share выбирается, какую версию видят зрители. Другая
  сессия обновляет artifact по URL (или через `/artifacts`), иначе создаёт новый.
- **Feedback → агент:** комментарии только у artifact, расшаренного **внутри организации**
  (Team/Enterprise), Claude Code v2.1.221+. «Send to Claude» или `@claude` активирует тред;
  сессия, которая опубликовала artifact, «watches that artifact for comments for as long as the
  session runs» (v2.1.228+); автоответ/автоправка зависят от permission mode, лимит 60 комментариев
  в час. После конца сессии — только по запросу «прочитай комментарии на URL». На Pro/Max шаринг
  только публичной ссылкой, а у публичного artifact комментарии отключены → **для Pro/Max
  комментарии фактически недоступны**. Известный баг: комментарий «отправлен», но не доходит —
  https://github.com/anthropics/claude-code/issues/92618
- **Медиа:** по публичной доке — одна self-contained страница ≤16 MiB, внешние картинки
  блокирует CSP, изображения только data URI, relative links не работают. В runtime этой сессии
  Artifact tool уже умеет `files` (многофайловые artifacts) и asset store (image/video/PDF, ≤15 MB
  на файл) — в публичной доке не отражено, похоже на gated rollout (**не подтверждено** как GA).
- **Local vs cloud:** только cloud (Anthropic infra), нужен claude.ai login; не работает с API key,
  Bedrock/Vertex, ZDR/HIPAA; по умолчанию выключено в Agent SDK и GitHub Action.
- **Codex:** нет (Codex не публикует; прочитать через WebFetch тоже нельзя — нужна авторизация).
- **Прочее:** `/design` — canvas-артефакт (v2.1.265+). Retention настраивается админом
  (Team/Enterprise). Compliance API для выгрузки.
- **Claude Docs** (beta с 2026-09-16): rich-text документы в claude.ai, комментарии, совместное
  редактирование, экспорт в Word/PDF/Google Docs/markdown; хранятся во вкладке Artifacts.
  Вторично: https://www.computerworld.com/article/4223177/anthropic-tries-to-make-claude-stickier-with-launch-of-docs-and-slides.html ,
  https://www.engadget.com/2259938/anthropics-claude-can-now-create-editable-documents-for-you-cowork-chat-together/ .
  В этой сессии Claude Code есть Claude Docs connector (MCP `claude_ai_Claude_Docs`) → Claude Code
  может создавать/читать docs и отвечать на комментарии. Официальной страницы в code.claude.com
  не нашёл — **не подтверждено** как поддерживаемый путь для Claude Code вообще.
- **Claude Code desktop:** preview запущенного приложения, visual diff с inline-комментариями,
  «Review code», мониторинг PR/CI с auto-fix (2026-02-20):
  https://claude.com/blog/preview-review-and-merge-with-claude-code ; Code Review на PR:
  https://code.claude.com/docs/en/code-review
- **Оценка:** лучший «презентационный» слой для Claude (красиво, версии, ссылка, телефон),
  но не source of truth: cloud-only, только Claude, комментарии требуют Team/Enterprise,
  привязка к живой сессии.

### 1.3 OpenAI Codex (app / cloud / CLI)
- **Review pane (app):** показывает git-состояние (Unstaged/Staged/Commit/Branch/Last turn),
  stage/revert по hunk, inline-комментарии на строках: «Codex treats inline comments as review
  guidance» — уходят в тот же thread. PR-контекст и комментарии ревьюеров из GitHub в sidebar.
  https://learn.chatgpt.com/docs/code-review?surface=app
- **Artifacts viewer:** превью документов, презентаций, таблиц, PDF, HTML (интерактивно + source);
  можно «point to a specific part of a file and tell ChatGPT what to change» — аннотация уходит в
  thread. CLI превью не имеет. https://learn.chatgpt.com/docs/artifacts-viewer
  Про markdown-рендер как отдельный тип — **не подтверждено**.
- **In-app browser + Annotate mode:** клик по элементу страницы, заметка + скриншот элемента → в
  чат агенту. https://developers.openai.com/codex/appshots (Appshots), вторично:
  https://x.com/kr0der/status/2047510880741364205
- **Codex cloud (web):** summary + diff задачи, «citations of terminal logs and test outputs»,
  скриншот результата прикладывается к задаче и к GitHub PR.
  https://developers.openai.com/codex/cloud , https://openai.com/index/introducing-codex/ ,
  https://openai.com/index/work-with-codex-from-anywhere/
- **Task sidebar** (plan, sources, artifacts, summary) — Platform 26.415, по вторичному источнику
  https://codex.danielvaughan.com/2026/04/17/codex-app-workspace-pr-review-task-sidebar-artifact-viewer/
- **Оценка:** сильная review-поверхность для Codex-сессий, но только для них, и комментарии живут
  в thread. Claude туда не попадает.

## 2. GitHub как поверхность

- **Markdown:** рендер в issues/PR/файлах; prose diff (rich diff) для `.md`:
  https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files
  **Ограничение:** в rich diff нельзя оставлять inline-комментарии — только в source diff
  (https://github.com/orgs/community/discussions/186730). Обход — сторонние browser extensions
  (https://github.com/sabbour/md-review-extension , https://github.com/chienyuanchang/rich-diff-comments ),
  они постят обычный review comment на исходную строку.
- **Feedback → агент:** pull request review comments и issue comments читаются через `gh`
  (`gh pr view --comments`, `gh api repos/{o}/{r}/pulls/{n}/comments`, `gh issue view --comments`).
  Автоматической доставки в локальную сессию нет — агент тянет по команде/на старте стадии.
  Claude Code desktop умеет мониторить PR/CI; Codex app показывает PR-комментарии в sidebar.
- **Медиа:** картинки и видео (mp4/mov) прикладываются к комментариям и описанию PR через
  drag-and-drop (лимиты по размеру зависят от плана — **не подтверждено** в этом прогоне);
  файлы в репозитории рендерятся как изображения.
- **Agent HQ / Mission Control / Agents panel:** назначать задачи Copilot, Claude, Codex
  (облачные агенты GitHub), смотреть session logs, steer, переходить в PR.
  https://github.blog/changelog/2025-10-28-a-mission-control-to-assign-steer-and-track-copilot-coding-agent-tasks/ ,
  https://github.blog/changelog/2026-02-04-claude-and-codex-are-now-available-in-public-preview-on-github/ ,
  https://github.blog/changelog/2026-02-26-claude-and-codex-now-available-for-copilot-business-pro-users/
  Это облачные агенты с GitHub-стороны (нужна подписка Copilot), не локальные Claude Code / Codex
  с harness; для «смотреть результат локальной сессии» не подходит.
- **GitHub Pages / Projects:** Pages — публикация статического сайта из docs (для приватного
  репозитория видимость Pages ограничивается только на Enterprise Cloud — **не подтверждено**,
  проверить при необходимости). Projects — доска статусов, не место чтения артефакта.
- **Оценка:** лучший durable/audit слой, одинаков для обоих агентов, но только после push/PR;
  чтение длинной spec в source diff неудобно; комментарии в агента — pull, не push.

## 3. Локальные генераторы сайтов и md-viewers

| Инструмент | Что даёт | Минусы для этого случая |
|---|---|---|
| Astro Starlight https://starlight.astro.build | папка md/mdx + front matter, dev server с live reload, поиск, компоненты (можно собрать index/board из front matter через content collections) | Node-зависимость, нет комментариев |
| VitePress https://vitepress.dev | быстрый dev server, data loaders (`createContentLoader`) для индекса по front matter | нет комментариев |
| Material for MkDocs https://squidfunk.github.io/mkdocs-material/ | Python, зрелый | **end of life 2026-11-05**, развитие ушло в Zensical https://zensical.org (вторично: https://fpgmaas.com/blog/collapse-of-mkdocs/ ) |
| Docusaurus https://docusaurus.io | React, версии документации | тяжёлый для solo |
| Quarto https://quarto.org | md + вычисляемые отчёты, `quarto preview` | тяжеловат, акцент на науку |

Общее: нет встроенного канала комментариев обратно агенту (можно Giscus → GitHub Discussions
https://giscus.app , но это уже GitHub). Видео — обычный `<video>`/markdown-ссылка на файл в папке.

Лёгкие viewers: Obsidian (vault поверх `docs/`, backlinks, картинки/видео, plugins; комментариев
агенту нет) https://obsidian.md ; Typora; VS Code markdown preview (встроен, есть в Codex IDE
extension и Claude Code IDE-интеграции); terminal — `glow` https://github.com/charmbracelet/glow ,
`frogmouth` https://github.com/Textualize/frogmouth (без картинок/видео).

**Специализированные annotate-инструменты (самая релевантная находка):**
- **Plannotator** https://github.com/backnotprop/plannotator , https://plannotator.ai — визуальное
  аннотирование plan / любого `.md` / папки / URL / code diff, `/plannotator-annotate spec.md`,
  `/plannotator-review`; фидбек возвращается в сессию агента (для Claude Code — hook на
  ExitPlanMode и slash commands). Поддержка: Claude Code, Codex, Copilot CLI, Gemini CLI, OpenCode,
  Amp, Pi и др. Local-first, без телеметрии; share-ссылки с AES-256-GCM. Apache-2.0/MIT, ~9k stars.
  Про картинки/видео в документе — **не подтверждено**.
- **plannotator-tui** https://github.com/plannotator/plannotator-tui — аннотирование markdown в
  терминале (select, comment, looks-good, delete → отправить агенту).
- **herdr-annotate** https://github.com/plannotator/herdr-annotate — то же внутри Herdr (у
  владельца Herdr уже есть): комментарии к тексту терминала, markdown-документам и ответам агента,
  отправка прямо в pane агента. README не удалось открыть (socket closed), описание по выдаче
  поиска — детали **не подтверждено**.

## 4. GUI/дашборды поверх агентских сессий (проверено, существует ли)

| Продукт | Статус (2026-09) | Per-task артефакты | Комментарий обратно агенту | Claude/Codex |
|---|---|---|---|---|
| Vibe Kanban https://github.com/BloopAI/vibe-kanban | **sunsetting** (баннер в README), Apache-2.0, self-host | kanban issues, workspace (branch+terminal+dev server), diff, встроенный browser preview | inline-комментарии на diff → агенту | оба + ещё 8 |
| Conductor https://www.conductor.build/docs/ | активен, macOS only, Pro $50/мес | workspace, diff viewer, checks, PR/merge | да (детали **не подтверждено**) | Claude Code, Codex, Cursor, OpenCode |
| Nimbalyst (ex-Crystal) https://github.com/nimbalyst/nimbalyst ; Crystal deprecated https://github.com/stravu/crystal | активен, MIT, mac/win/linux + iOS | WYSIWYG markdown, mockups, Mermaid, session kanban, red/green diff правок агента, «plain files on disk in your git repo» | accept/reject правок; комментарии — **не подтверждено** | Claude Code, Codex |
| Sculptor (Imbue) https://imbue.com/product/sculptor , https://docs.imbue.com/changelog | beta, бесплатно, Mac, Docker-контейнеры | Pairing Mode, merge review UI, suggestions | — | Claude Code (Codex — **не подтверждено**) |
| CloudCLI / claudecodeui https://github.com/siteboon/claudecodeui | активен (вторично) | web/мобильный UI сессий, файлы, git | чат в сессию | Claude Code, Codex, Cursor CLI |
| opcode (ex-Claudia) https://github.com/winfunc/opcode | без коммитов с 10.2025 (вторично) | GUI сессий | — | Claude Code |
| Terragon https://www.terragonlabs.com | **закрыт 2026-02-09** | — | — | — |
| GitHub Agent HQ | см. §2 | session logs → PR | steer в сессии | облачные Claude/Codex GitHub |

Вывод: все живые GUI — это «кабина пилота» для параллельных сессий (worktree + diff), а не
поверхность приёмки артефактов стадий. Ни один не читает ваш harness-формат (spec / delivery
report) как первоклассный объект; lock-in в их модель workspace. Наиболее близкий по духу —
Nimbalyst (файлы в репо, markdown WYSIWYG), но это замена IDE.

## 5. CLI/TUI review

- `gh dash` https://github.com/dlvhdr/gh-dash — дашборд PR/issues в терминале, glamour-рендер
  markdown, комментарий из preview pane (`c`, Ctrl+d), Checks tab. Хорош для «что ждёт меня»,
  плох для длинной spec и медиа.
- `lazygit` — git-операции, не приёмка.
- Custom TUI (Textual) — дёшево для списка задач/статусов, но картинки/видео в терминале
  фактически нет (частично kitty/iTerm graphics protocols).
- Итог: TUI — навигатор и «inbox», не место чтения spec и просмотра видео.

## 6. Механика обратной связи и handoff

### Как комментарий владельца попадает в следующую сессию
| Канал | Доставка | Переживает сессию? | Агент-агностичен? |
|---|---|---|---|
| Файл в репо (комментарии в `review.md`/inline-блоки в spec, коммит) | агент читает на старте стадии | да, в git | да |
| Plannotator / plannotator-tui / herdr-annotate | push прямо в текущую сессию | нет (если не сохранить в файл) | да (много агентов) |
| GitHub PR review / issue comments | pull через `gh` | да | да |
| Claude artifact comments | push в сессию-публикатора, пока она жива; позже pull по URL | тред в claude.ai | только Claude, только Team/Enterprise |
| Claude Docs comments | через Docs connector | да, в claude.ai | только Claude |
| Codex review pane / artifacts viewer / Annotate | в thread | в истории thread | только Codex |
| Antigravity comments | в conversation | в brain-папке | только Antigravity |
| MCP-сервер над папкой ревью | pull tool-call | как хранилище | да, если MCP есть у обоих (есть) |

### Форматы handoff между сессиями
- **OpenAI ExecPlans** (PLANS.md): self-contained living document; обязательные разделы Progress,
  Surprises & Discoveries, Decision Log, Outcomes & Retrospective; «restart from only the ExecPlan».
  https://github.com/openai/openai-cookbook/blob/main/articles/codex_exec_plans.md
- **Anthropic long-running harness:** `feature_list.json` + `claude-progress.txt` + git log,
  каждая сессия восстанавливает контекст из файлов, одна фича за сессию, e2e через браузер.
  https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- **Matt Pocock `handoff` skill:** компактный документ в OS temp dir; уже записанное (spec, plan,
  ADR, issues, commits, diffs) — ссылкой, не копией; раздел suggested skills; redaction секретов.
  https://github.com/mattpocock/skills/blob/main/skills/productivity/handoff/SKILL.md ;
  известная слабость — теряет тонкие решения долгих сессий (issue #186).
- **Amp Handoff:** генерирует черновик prompt + список файлов для нового thread, человек правит
  перед отправкой. https://ampcode.com/news/handoff . Утверждение, что в Amp «Neo» handoff убран в
  пользу compaction, — только вторичный источник; https://ampcode.com/news/neo не открылся —
  **не подтверждено**.
- **Что работает лучше:** файл в репо (git), самодостаточный, с разделами «цель / состояние /
  решения / что проверено (с доказательствами) / следующий шаг / критерий завершения», ссылки
  вместо копий, плюс машиночитаемый front matter (status, issue, pr, head SHA, evidence paths).
  Это пересечение ExecPlan + progress file + Pocock; совпадает с правилом Work AGENTS.md
  («не передавай только историю чата»).

---

## Сравнительная таблица

Шкала: ++ отлично, + да, ± частично, − нет.

| Поверхность | Читабельность spec | Медиа (PNG/MP4) | Feedback → агент | Версии | Локально до PR | Claude | Codex | Setup | Зрелость | Lock-in |
|---|---|---|---|---|---|---|---|---|---|---|
| Файлы в репо + VS Code/Obsidian preview | + | + | ± (правка файла) | ++ git | ++ | + | + | ~0 | ++ | нет |
| Plannotator / herdr-annotate | ++ | ± | ++ push в сессию | ± | ++ | + | + | низкий | + (молодой) | низкий, OSS |
| Локальный Starlight/VitePress | ++ (индекс/доска) | ++ | − | ++ git | ++ | + | + | средний | ++ | нет |
| Claude Artifacts / Docs | ++ | + (data URI; assets gated) | + только Team/Ent., живая сессия | + | − cloud | + | − | низкий | + (новое, баги) | высокий |
| Codex app panes / cloud | + | + (скриншоты) | ++ в thread | ± | + (app) | − | + | низкий | + | высокий |
| Antigravity Artifacts | ++ | ++ | ++ | ± | + | − | − | средний | + | высокий |
| GitHub PR/Issues | ± (source diff) / + (render) | + | ± pull через gh | ++ | − | + | + | ~0 | ++ | средний |
| Agent GUI (Conductor, Nimbalyst, Vibe Kanban) | ± | ± | + на diff | ± | + | + | + | средний | ± (Vibe Kanban закрывается) | средний |
| gh dash / TUI | − | − | ± | — | ± | + | + | низкий | ++ | нет |

## Рекомендация для этого контекста (слоями)

1. **Source of truth — файлы в репозитории, агент-агностичный формат.** Для каждой стадии —
   markdown с front matter: `spec.md`, `breakdown.md`, `delivery.md` (что сделано, какие проверки
   прошли с командами и итогом, head SHA, где попробовать — URL стенда/команда, список evidence),
   `handoff.md` по схеме ExecPlan/progress. Медиа — `docs/evidence/<issue>/*.png|mp4` (в проекте
   уже есть конвенция docs/evidence). Обратная связь владельца тоже файлом (`review.md` рядом или
   блоки `> [!owner]` в spec) — так её прочитает любая следующая сессия, Claude или Codex.
2. **Локальный рендер для чтения — одна команда harness.** `inside-harness review <issue>` (или
   аналог): собирает статичную HTML-страницу из md + evidence (картинки, `<video>`), открывает в
   браузере; индекс-страница по front matter = «доска приёмки» без Projects. Самый дешёвый вариант —
   без фреймворка (python-markdown + шаблон); если нужен поиск/навигация — Starlight или VitePress
   в dev-режиме над `docs/`. Не брать Material for MkDocs (EOL 2026-11-05).
3. **Канал комментариев в живую сессию — Plannotator / herdr-annotate** (оба агента, local-first,
   владелец уже на Herdr). Правило harness: аннотации после отправки агент сохраняет в `review.md`,
   чтобы они пережили сессию.
4. **GitHub — durable слой после push.** Spec — в PR как `.md` (комментарии на строки в source
   diff; при желании extension для rich diff); delivery report — описание PR / комментарий со
   скриншотами и видео; агенты читают feedback через `gh`. Projects — только статусы.
5. **Vendor-слои — опционально, как удобство, не как канон.** Claude Artifact/Docs — красивая
   презентация и ссылка на телефон, когда сессия Claude; помнить: комментарии только Team/Enterprise,
   cloud, только Claude, retention и баг #92618. Codex app review pane / Annotate — когда сессия
   Codex. Результат этих каналов агент обязан переносить в файлы слоя 1.

Не рекомендуется: строить процесс вокруг Agent GUI (Vibe Kanban закрывается, Terragon закрыт,
Conductor — macOS + платный + свой workspace-формат), вокруг Antigravity (не ваши агенты), или
вокруг одного vendor-канала комментариев.
