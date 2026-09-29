# Структурированные артефакты и доказательства в LLM multi-agent разработке: обзор литературы 2023–2026

Дата сбора: 2026-09-25. Метаданные всех arXiv-работ проверены через arXiv API (`export.arxiv.org/api/query`),
тексты прочитаны из PDF (arXiv) через `pdftotext`. «Рецензирование» указано по полю comment/journal_ref arXiv
или по странице конференции; где его нет — «препринт, без рецензирования». Всё, что не удалось прочитать в
первоисточнике, помечено «не подтверждено».

Условные обозначения: **[R]** — рецензируемая публикация; **[P]** — препринт/техотчёт.

---

## 1. MetaGPT — SOP и структурированные выходы ролей

- Hong et al., «MetaGPT: Meta Programming for A Multi-Agent Collaborative Framework», arXiv 2308.00352
  (v1 2023-08-01, v7 2024-11-01). **[R]** ICLR 2024 (в PDF: «Published as a conference paper at ICLR 2024»).
  https://arxiv.org/abs/2308.00352 · https://github.com/geekan/MetaGPT

**Механика.**
- SOP закодированы в последовательности промптов. Работа идёт как на сборочной линии: Product Manager → Architect →
  Project Manager → Engineer → QA Engineer.
- *Structured Communication Interfaces*: «We establish a schema and format for each role and request that
  individuals provide the necessary outputs based on their specific role and context». Роли обмениваются
  документами и диаграммами, а не диалогом. Артефакты: PRD, system interface design, sequence flow diagram,
  API spec, task list (§3.2, рис. 3, приложение B).
- *Shared message pool + publish-subscribe*: все роли публикуют структурированные сообщения в общий пул и
  подписываются на нужные по профилю роли. Правило активации: «an agent activates its action only after
  receiving all its prerequisite dependencies». Это условие готовности на уровне зависимостей артефактов, а не
  на уровне доказательств (§3.2).
- *Executable feedback*: Engineer запускает код и тесты и итеративно чинит по результатам (§3.3).

**Измеренный эффект.**
- SoftwareDev (70 задач, но в сравнении только **7** случайно выбранных задач; метрики по людям и статистике),
  табл. 1: Executability (1–4) — ChatDev 2.25, MetaGPT без feedback 3.67, MetaGPT 3.75. Human Revision Cost:
  2.5 / 2.25 / 0.83. Токенов: 19 292 / 24 613 / 31 255.
- Executable feedback: +4.2 п.п. (HumanEval) и +5.4 п.п. (MBPP) Pass@1. Итог 85.9% и 87.7%.
- Абляция ролей (табл. 3). Каждая роль добавляет свой артефакт. Только Engineer: executability 1.0, revisions 10.
  + Product (PRD): 2.0 / 6.5. + Product + Architect: 2.5 / 4.0. Все четыре роли: 4.0 / 2.5. Стоимость $0.915 → $1.385.

**Чего нет.** Прямой абляции «структурированные документы против свободного диалога» при тех же ролях нет.
Эффект схем смешан с эффектом ролей и SOP. Отдельно проверено только executable feedback.

**Угрозы валидности.** 7 задач в SoftwareDev. Executability выставляют люди. HumanEval/MBPP — функции, а не
проекты. В независимой оценке ChatDev (ниже) MetaGPT проигрывает по «Quality» (0.1523 против 0.3953), то есть
результаты зависят от протокола оценки. В MAST (ниже) у MetaGPT в 1.56 раза больше отказов категории FC3
(верификация), чем у ChatDev, хотя в FC1/FC2 отказов на 60–68% меньше.

---

## 2. ChatDev, AgileCoder, EvoDev, EvoMAC

### 2.1 ChatDev
- Qian et al., «ChatDev: Communicative Agents for Software Development», arXiv 2307.07924 (v1 2023-07-16,
  v5 2024-06-05). **[R]** ACL 2024. https://arxiv.org/abs/2307.07924 · https://github.com/OpenBMB/ChatDev

**Механика.** Chat chain: 3 фазы (design, coding, testing) и 5 подзадач. Роли: CEO, CTO, programmer, reviewer,
tester. Подзадача завершается после «two unchanged code modifications or after 10 rounds». *Communicative
dehallucination* (CDH): перед ответом агент запрашивает уточнение. Артефакты — код и документы, но передача идёт
через диалог в естественном языке и в коде, без схем.

**Измеренный эффект** (SRDD, 1 200 описаний задач; ChatGPT-3.5, T=0.2). Табл. 1: Quality (= Completeness ×
Executability × Consistency) — GPT-Engineer 0.1419, MetaGPT 0.1523, ChatDev 0.3953.
Абляция (табл. 4), Quality:
- остановка после Coding 0.2512; после Complete 0.3690; после Review 0.3717; полная цепочка (с Testing) 0.3953;
- без CDH 0.3094; без ролей 0.2212.
- Executability: после Coding 0.77, с Testing 0.88. Без ролей 0.58.

**Угрозы.** Consistency — это косинусная близость эмбеддингов требований и кода, а не проверка выполнения
требований. Completeness — это отсутствие плейсхолдеров. Метрики слабо связаны с корректностью. Pairwise-оценка
сделана GPT-4 и людьми.

### 2.2 AgileCoder
- Nguyen et al., arXiv 2406.11912 (v2 2024-07-14, «Work in progress» на arXiv). **[R]** FORGE 2025 (IEEE/ACM,
  pp. 156–167): https://conf.researchr.org/details/forge-2025/forge-2025-papers/1/AgileCoder-Dynamic-Collaborative-Agents-for-Software-Development-based-on-Agile-Meth ·
  https://arxiv.org/abs/2406.11912 · https://github.com/FSoft-AI4Code/AgileCoder

**Механика.** Роли: PM, Scrum Master, Developer, Senior Developer, Tester. PM пишет *product backlog* с задачами и
**acceptance criteria**. SM проверяет выполнимость и может вернуть бэклог. Спринты: Planning → Development →
Testing → Review. Спринт наследует результат предыдущего. *Dynamic Code Graph Generator* строит граф
зависимостей G. Tester тестирует изменённые файлы и их предков в G.

**Измеренный эффект** (абляция, табл. 3, GPT-3.5 Turbo, HumanEval/MBPP pass@1):
- полная система 70.53 / 80.92;
- без incremental dev −1.02 / −2.47;
- без генерации тестов −8.33 / −5.28;
- без code review −1.63 / −5.51.
- Claude 3 Haiku: без тестов −6.10 / −4.45.
- ProjectDev (14 задач): executability 57.79 против ChatDev 32.79 и MetaGPT 7.73. Без графа G executability
  23.38 против 57.50, и 11 случаев переполнения контекста.

**Угрозы.** HumanEval/MBPP — функциональные задачи. ProjectDev мал. Acceptance criteria формально не
проверяются как gate. Бэклог — это текст, без схемы.

### 2.3 EvoDev (arXiv 2511.02399)
- Liu et al., «Towards Iterative End-to-End Software Development: A Feature-Driven Multi-Agent Framework»
  (v1 2025-11-04, v3 2026-06-05). **[R]** ISSTA 2026 (comment на arXiv). https://arxiv.org/abs/2511.02399

**Механика.** Требования раскладываются на пользовательские features. Строится *Feature Map* — DAG зависимостей.
Каждый узел хранит многослойный контекст: **business logic, software design, code implementation**. Этот контекст
передаётся потомкам по рёбрам. Есть этап overall design и отдельный Business Analyst agent.

**Измеренный эффект** (APPDev: 15 Android-приложений, в среднем 13.5 функциональных требований; оценка по
acceptance checklist и Likert-анкете четырёх участников):
- против Claude Code +57.3% по Function Completeness (FC). С Claude-4-Sonnet: build 100% против 73.3%; FC 3.57
  против 2.27.
- Против одиночного агента +16.0…58.5% в зависимости от LLM.
- Абляция RQ3: overall design даёт FC +0.22 (+7.2%).
- **Итерации без передачи контекста предшественников ухудшают результат**: build 100% → 86.7%, FC 3.29 → 2.80.
  С передачей контекста FC +0.50 (+16.3%).

**Ограничения (заявлены авторами).** Тестирования в методе нет: агенты путают, где ошибка — в тесте или в коде.
Оценка ручная, на 15 приложениях, только Android/Kotlin.

### 2.4 EvoMAC
- Hu et al., «Self-Evolving Multi-Agent Collaboration Networks for Software Development», arXiv 2410.16946
  (2024-10-22). **[R]** ICLR 2025: https://iclr.cc/virtual/2025/poster/31011 · https://arxiv.org/abs/2410.16946

**Механика.** Coding team (сеть MAC) → Testing team пишет unit-тесты по требованиям. Эти тесты служат *target
proxy*, а результаты компиляции и запуска дают объективную обратную связь. Updating team выполняет «textual
backpropagation» и перестраивает coding team. rSDE-Bench — бенчмарк, ориентированный на требования, с
автоматической проверкой каждого требования.

**Измеренный эффект** (GPT-4o-Mini, табл. 1): Website Basic/Advanced 89.38 / 65.05; Game 77.54 / 51.60. Лучший
multi-agent baseline (ChatDev) 62.67 / 43.45 / 53.63 / 32.26. HumanEval 94.51.
Ключевая абляция (вариант f против g): **если заменить выполнение тестов в среде на LLM-критику кода**, результат
падает. Website −12.67 / −14.97 п.п., Game −21.74 / −18.28 п.п. Это прямое измерение: исполняемые доказательства
лучше мнения LLM.

**Угрозы.** Тесты генерирует та же система, то есть прокси может быть неверным. Web/game-задачи узкие.

---

## 3. Proof-or-Stop (arXiv 2607.14890) — ключевая работа

- Huang, Hsia, Sun, Shi, Huang, White, «Proof-or-Stop: Don't Trust the Agent, Trust the Evidence — Loop
  Engineering for Verifiable Evidence-Gated Lifecycle Control», arXiv 2607.14890v1, 2026-07-16. 48 страниц.
  **[P]** «Preprint v1», без рецензирования. https://arxiv.org/abs/2607.14890 · код: https://github.com/Proof-or-Stop
  (в статье указан только организационный URL; точный репозиторий не подтверждён).

### 3.1 Семантика и формальный каркас
- **Agent-as-claim**. Вывод агента — это *claim* (reviewed, tested, DONE, ready-to-merge), а не состояние
  lifecycle. Слово «proof» используется операционно: доказательство, допустимое для gate при заявленной модели
  доверия. Семантическая корректность программы не утверждается.
- Идентичности состояния (ур. 1), считаются по `git ls-tree` без метаданных lifecycle:
  `materialHash(H)` = SHA256 канонического tracked-дерева; `headHash` = commit; `storyFilesHash` = SHA256 файлов,
  принадлежащих story.
- **Evidence** — структурированная запись, не проза. Binding
  `β(E) = ⟨materialHash, headHash, storyFilesHash, policyHash, commandSetHash⟩`. Для исполненных действий есть
  receipt `ρ(E) = ⟨cmd, args, cwd, exit, outputDigest⟩`. Подписанные записи дополнительно несут
  actor/lane/host/session/signing-key.
- **Admissibility** (ур. 2): `Fresh ∧ Complete ∧ IntegrityVerified ∧ ProducerAuthorized ∧ ExecutionAttested ∧
  Supports(E,c) ∧ OutcomeAccepted`. Любое изменение дерева делает старые доказательства устаревшими.
- **Gated advancement** (ур. 3): переход φi→φi+1 разрешён, только если каждый required claim из Ci подкреплён
  допустимым E. «a natural-language report from an agent is not an Ec … Eq. (3) therefore has no term for
  self-report».
- Ограничение области: тяжёлый механизм применяется только к claims, которые двигают lifecycle. Заметки,
  rationale и docs остаются advisory и не входят в bindings. Runtime-память помечена `gateEvidence:false`.

### 3.2 Lifecycle и gates (табл. 8)
Story: `init → init-check → plan → dev → review → test → done`.
| Переход | Артефакт | Проверка | Fail-closed |
|---|---|---|---|
| plan→dev | structured plan review / story contract | scope-contract, allowed paths, свежий storyFilesHash | нет DEV без plan evidence |
| dev→review | scope-contract check | diff в рамках объявленного scope и привязан к materialHash | out-of-scope → нет REVIEW |
| review→test | `review-runs.json`, `review-passes.json`, `findings.json` | текущий round, identity ревьюера, свежесть; открытые verified critical/high блокируют | блок TEST |
| test→done | `done-required-evidence.json` (`evidence.json`) | хэши дерева + policy/command-set совпадают; build + full test suite exit 0 | блок DONE при stale/missing/drift |
| high-risk DONE | host verdict receipts | floor R×K = **3 раунда × ≥2 независимых verdict** (разные host, session, key) над текущим materialHash | честная деградация: `degraded ⇒ FullAssurance=false` (ур. 5) |

Дополнительно:
- `findings.json`: severity, category, status, evidenceState, resolution trail.
- Wording gate / claim-boundary registry запрещает формулировки сильнее, чем позволяют доказательства.
- Merge должен проверять сертификат против точного commit с compare-and-swap. PR — интерфейс, а не граница доверия.
- Уточнение B-6c4d. Свежесть не определяется по «новизне id». PASS допустим из любого подписанного run текущего
  раунда, если хэши и scope совпадают. Открытый FINDING требует последнего run данной lane.

### 3.3 Измерения
1. **Engine contract** (Tier A): 10/10 сценариев (intake, executor, repair-loop, review-loop, evidence-gate,
   block-escalate, budget-stop, human-handoff, multi-host-review, no-false-done), **false-DONE = 0**.
   Стресс-набор 10×15 = 150 строк ledger. Проверяется, что:
   - «прозаическое» закрытие без machine evidence не считается;
   - stale bindings блокируют;
   - deferral не засчитывается как delivery;
   - assignment не равен delivery;
   - дубликаты id отлавливаются.
   Это проверка механизма, а не выигрыш по исходам.
2. **Receipt bundle** (B-4c65): подлинный bundle принят; **18 классов подделки отклонены**, false-accept = 0,
   false-reject = 0. Модель доверия — local-key, single-host.
3. **Gate-strength** (B-7b6e). Метрика ревьюера — пара (catchRate, falseBlock), потому что одного catchRate
   недостаточно. Test-only reviewer: catchRate 0. Block-all: 1.0, но блокирует и корректные артефакты.
   Эталонные ревьюеры, а не живая lane.
4. **Powered-абляция, пререгистрированная**: 5 arms × 24 задачи × 16 сценариев (null + B1–B15 инъекции) × k=5 =
   **9 240 ячеек**, одна модель (Sonnet; точный snapshot не записан). «Amplified» — артефакт прошёл видимый тест,
   провалил скрытый oracle и всё равно был отгружен (табл. 11):

   | Arm | Amplified (B1–B15) | Токены | Время |
   |---|---|---|---|
   | A1 prompt-only | 13/1440 (0.90%) | 168 622 | 51.7 s |
   | A2 naive-retry | 18/1800 (1.00%) | 169 202 | 51.3 s |
   | A2′ compute-budgeted | 31/1800 (1.72%) | 170 545 | 54.8 s |
   | A3 review-only (advisory) | 14/1800 (0.78%) | 200 510 | 80.3 s |
   | A4 Proof-or-Stop | **2/1800 (0.11%)** | 204 553 | 81.2 s |

   - H1 (A4 против A2′, not-amplified): **+1.6 п.п., 95% CI [0.8, 2.5]**. H2 (completion): +3.3 п.п. [0.0, 10.0],
     маргинально.
   - **A3 против A4** — главная для нас изоляция. Та же одна проверка ревьюером при почти равном бюджете (токены
     A3/A4 = 0.98). Разница только в том, advisory ревью или обязательный gate: 14/1800 против 2/1800. Без задачи
     du-duration: 9/1725 против 0/1725. H3 — эксплораторная гипотеза.
   - Концентрация эффекта: 29 из 31 случаев A2′ и оба случая A4 приходятся на одну задачу (du-duration). Без неё
     2/1725 против 0/1725. Per-scenario BH: 0/15 значимы.
   - В powered-матрице **не** используется story-level materialHash/receipt gating. Это эксперимент над
     control policy, а не над полным механизмом хэшей.
5. **Clean-task pilot** (3 задачи × k=3). Все arms 9/9. Gate даёт только накладные расходы: A4/A1 = 2.07× cost,
   2.51× wall. В fault-pilot bare loop пропустил 15/15 ошибок, gated loop — 0/15.
6. **Paired matrix** Cell03/Cell06 (1 152 ячейки, описательная). No-review дошёл до завершения в 1 143 ячейках.
   Proof-or-Stop допустил 1 042 и безопасно остановил 110. 106 завершений no-review не были допущены, но скрытым
   oracle они не проверялись. Токены ×3.80.
7. **Self-application corpus**: 565 stories, 1 007 findings, 94.8% resolved; high ≈51%, critical ≈1%.
   - Lanes различаются: test-coverage lane («intent↔evidence binding») даёт 70% high/critical; concept lane —
     10 из 14 critical.
   - Кураторский deep set: 26 из 28 findings (93%) найдены, когда тесты автора были зелёными. Это метка ревьюера,
     не контрфактический эксперимент. 6 реальных дефектов подтверждены fix-коммитами.
   - Cross-vendor (Codex host-2): 68 high/critical findings по 26 stories, которые уже прошли same-vendor Claude
     lanes. Выборка селективная.

### 3.4 Угрозы валидности (заявлены авторами, §11)
- Одна семья моделей, 24 задачи, самохостинговый корпус.
- Корпус построен и отревьюен LLM.
- Метки `smoke_would_miss` поставлены ревьюерами.
- A2′ не выравнен по бюджету на уровне отдельного run.
- Эффект редкий и сосредоточен в одной задаче.
- Модель доверия local-key не защищает от скомпрометированного runner.
- Нет базовой частоты «green-but-wrong» в реальной работе, поэтому нельзя оценить соотношение cost/benefit.
- Внешних бенчмарков нет.

### 3.5 Смежные работы, на которые ссылается Proof-or-Stop
- **ResearchLoop**: Xia & Wang, arXiv 2605.28282, 2026-05-27, **[P]** техотчёт. https://arxiv.org/abs/2605.28282 ·
  https://github.com/plan-lab-szu/ResearchLoop
  - Evidence-gated control plane для исследований. Цепочка «Big RQ → sub-RQ → claim → task/spec → evidence →
    gate → paper». YAML-схемы `STATUS.yaml`, `RESEARCH_SPINE.yaml`, `PAPER_CLAIM_LEDGER.yaml`.
  - Контролируемое исследование: 16 задач × 3 seeds, детерминированный аудит. Unsupported claim rate снижен на
    **8.43 п.п. (−63.3%)** против ad-hoc и на 8.47 п.п. против линейного baseline. Сами авторы оценивают
    значимость как пограничную: **p ≈ 0.08**.
  - Полный протокол выдаёт заметно меньше claims (169 против 467–569): часть эффекта — в подавлении claims.
- **Agentic Agile-V**: Koch, arXiv 2605.20456, 2026-05-19, 7 страниц, **[P]**. https://arxiv.org/abs/2605.20456
  - Концептуальная работа без эксперимента. Conversation-to-contract gate, таксономия минимальных входных
    артефактов, evidence-bundle acceptance model, цикл SCOPE-V.
- **EviBound**: arXiv 2511.05524, 2025-10-28, **[P]**. https://arxiv.org/abs/2511.05524
  - Два gate. Approval Gate до запуска проверяет схему acceptance criteria. Verification Gate после запуска
    проверяет артефакты через MLflow API: run ID, required artifacts, статус FINISHED.
  - 8 задач. Prompt-only: 8/8 ложных «complete». Только verification gate: 25% ложных. Оба gate: 0%, 7/8
    проверено, 1 задача заблокирована.
  - Выборка очень мала (n=8).

---

## 4. Harness engineering: автоматическое улучшение harness и связь с доказательствами

### 4.1 Self-Harness (arXiv 2606.09498)
- Zhang et al., «Self-Harness: Harnesses That Improve Themselves», v1 2026-06-08, v3 2026-08-20. **[P]**.
  https://arxiv.org/abs/2606.09498
- **Механика**: Weakness Mining (по traces) → Harness Proposal (минимальные правки, каждая привязана к конкретному
  механизму отказа) → **Proposal Validation**: «an edit is promoted only if it improves performance without causing
  measurable degradation on held-out tasks». Это regression gate для изменений самого harness.
- **Эффект**: во всех 9 комбинациях модель×бенчмарк (Terminal-Bench-2.0, SWE-bench Verified, AppWorld; MiniMax
  M2.5, Qwen3.5-35B-A3B, GLM-5) растут и held-in, и held-out. Относительный прирост до 132%.
- Примеры принятых правок:
  - TB2, MiniMax: 42.2% → 53.9% (артефакты создаются раньше, schema-invalid tool content, выход из зацикленных
    tool-циклов);
  - Qwen: 18.0% → 36.7%;
  - на SWE-bench Verified правки касаются **patch validation** («local-test enforcement»).
- **Угрозы**: сравнение только с минимальным исходным harness; оценки на выборках бенчмарков.

### 4.2 Meta-Harness (arXiv 2603.28052)
- Lee, Nair, Zhang, Lee, Khattab, Finn, 2026-03-30, **[P]**. https://arxiv.org/abs/2603.28052
- **Механика**: агент-proposer читает файловую систему со всеми прошлыми кандидатами: исходный код, **сырые
  execution traces**, оценки.
- **Абляция** (online text classification): scores-only — median/best 34.6/41.3; scores + LLM-summary — 34.9/38.7;
  полный доступ к traces — **50.0/56.7**. «summaries do not recover the missing signal». Это прямое свидетельство
  против сжатия доказательств в прозаические саммари.
- На TerminalBench-2 найденный harness лучше Terminus-KIRA: #1 среди агентов на Haiku 4.5 (37.6 против 35.5).

### 4.3 AutoHarness (arXiv 2603.03329)
- Lou, Lázaro-Gredilla, Dedieu, Wendelken, Lehrach, Murphy, 2026-02-10, **[P]**. https://arxiv.org/abs/2603.03329
- Gemini-2.5-Flash синтезирует код-harness, который отсекает недопустимые действия, итеративно уточняя его по
  обратной связи среды. Харнесс предотвращает **все** нелегальные ходы в 145 играх TextArena. Для сравнения: в
  Kaggle GameArena 78% поражений Flash были из-за нелегальных ходов.
- Применимость к SE косвенная: это детерминированный кодовый gate вокруг LLM. Прочитана только аннотация.

### 4.4 HarnessFix (arXiv 2606.06324)
- Chen, Wang, Liu, Wang, Zheng, Wang, «From Failed Trajectories to Reliable LLM Agents: Diagnosing and Repairing
  Harness Flaws», v1 2026-06-04, v2 2026-07-02. **[P]**. https://arxiv.org/abs/2606.06324
- **Механика**: traces и артефакты harness компилируются в **HTIR** (Harness-aware Trace IR) с data-flow и
  control-flow связями между шагами. Отказы атрибутируются шагам и артефактам harness. Повторяющиеся диагнозы
  сводятся в **flaw records**. Записи отображаются на scoped repair operators, и патч генерируется под
  flaw-specific repair specification. Принимается только через **regression-aware acceptance** на validation set:
  цель — снижение целевого дефекта без новых регрессий.
- Таксономия слоёв ETCLOVG. В каталоге операторов lifecycle-слоя есть «verification-gated finalization»,
  «delegated-output validation». В verification-слое — «intermediate validation gating», «effect-evidence
  completion guarding».
- Эмпирическое исследование 30 агентов: в 29 из 30 есть дефекты в трёх ведущих слоях. Verification-дефекты чаще
  встречаются в harness, ориентированных на бенчмарки. Lifecycle-дефекты — в долгоживущих агентах.
- **Эффект**: +6.3…18.4% к исходному harness, sign-test p < 0.001. Против Meta-Harness +2.6…5.0 п.п. при
  63.5–100.5% меньшем числе токенов.
- **Абляция** (табл. VI; GAIA / SWE / AppWorld / TB2):

  | Вариант | GAIA | SWE | AppWorld | TB2 |
  |---|---|---|---|---|
  | Full | 61.7 | 57.3 | 43.0 | 26.5 |
  | Без regression-aware acceptance | 55.6 | 53.3 | 39.3 | 24.5 |
  | Без trace-grounded diagnosis | 51.1 | 50.7 | 38.1 | 21.6 |
  | Prompt-only | 50.6 | 48.3 | 37.4 | 18.6 |

  HTIR совпадает с человеческой разметкой: step accuracy 85.0%, repair-operator accuracy 82.5%.

### 4.5 «Agent Harness Engineering: A Survey» (OpenReview eONq7FdiHa)
- Li, Xiao, Zhang, … Reddy, 2026. По заголовку PDF — «Under review as submission to TMLR», **[P]**.
  https://openreview.net/forum?id=eONq7FdiHa · https://github.com/Picrew/LLM-Harness
- Вводит таксономию ETCLOVG: Execution, Tooling, Context, Lifecycle, Observability, **Verification**, Governance.
  По вторичным источникам, охватывает 110+ работ и 148 проектов.
- **Не подтверждено**: полный текст не удалось прочитать (OpenReview отдаёт challenge). Содержание раздела
  Verification здесь не пересказывается. Определения слоёв взяты из табл. I в HarnessFix, который цитирует обзор
  как [1].

### 4.6 «Dive into Claude Code» (arXiv 2604.14228)
- Liu, Zhao, Shang, Shen, v1 2026-04-14, v2 2026-07-02. Tech report, **[P]**. https://arxiv.org/abs/2604.14228 ·
  https://github.com/VILA-Lab/Dive-into-Claude-Code
- Архитектурный анализ по исходному коду. Ядро — while-loop. Вокруг него:
  - permission system с 7 режимами и ML-классификатором;
  - 5-слойный compaction;
  - 4 механизма расширения: MCP, plugins, skills, hooks;
  - делегирование subagents;
  - append-oriented session storage.
- Количественных измерений эффекта артефактов или доказательств нет. Работа описательная. Прочитана аннотация.

### 4.7 Harness-of-Harness (arXiv 2609.01481) — точка отсчёта
- Yan, Su, Zhang, Li, Zhang, Zhang, Chen, Bai, Hu, 2026-09-01, **[P]**. https://arxiv.org/abs/2609.01481 ·
  https://github.com/Flesymeb/HarnessOfHarness
- Итерация: `D_t = Plan(S, E_{t−1})`, `A_t = Dev(A_{t−1}; S, D_t)`, `E_t = Test(A_t; S, D_t)`.
- Evidence bundle состоит из claims с `execution_records` и `status ∈ {verified, gap}`, `player_impact`,
  `recommended_update`, а также `planner_handoff {preservation_constraints, update_targets,
  validation_requirements}`.
- QA получает `read_only(A_t)` — **замороженного** кандидата — плюс детерминированные build/run checks. «A
  criterion is verified only when candidate-bound records support the required behavior. Observed failures, unmet
  requirements, regressions, and insufficient evidence are recorded as gaps».
- **Абляция** (GameCraft-Bench, 45 задач, Codex/GPT-5.5): Full HoH@3 71.52. Без Evidence Feedback (replanning без
  предыдущих доказательств) 65.23 (**−6.28**). Без Plan Update −8.13. Без Warm-Start −7.85 (и токенов 11.12M против
  8.41M).
- Отдельной абляции «замороженный read-only QA против QA с правом правки» **нет** (в тексте не найдено).

---

## 5. Анализ отказов

### 5.1 MAST — «Why Do Multi-Agent LLM Systems Fail?»
- Cemri, Pan, Yang, … Zaharia, Gonzalez, Stoica, arXiv 2503.13657 (v1 2025-03-17, v3 2025-10-26). **[R]**
  NeurIPS 2025 Datasets & Benchmarks (указано в PDF). https://arxiv.org/abs/2503.13657
- MAST-Data: 1 600+ trace по 7 MAS; κ = 0.88. 14 режимов отказа в 3 категориях.
- **FC3 Task Verification** (рис. 1): FM-3.1 Premature Termination 6.2%, FM-3.2 No or Incomplete Verification 8.2%,
  FM-3.3 Incorrect Verification 9.1%. В сумме **23.5%** (в другой версии рисунка 21.3%). В FC1 есть также FM-1.5
  «Unaware of Termination Conditions».
- Наблюдение: системы с явными верификаторами (MetaGPT, ChatDev) в целом отказывают реже, но «many existing
  verifiers perform only superficial checks … checking if the code compiles or if there are leftover TODO
  comments». Пример: шахматы ChatDev компилируются, но нарушают правила игры.
- Вмешательства (прил. H, ChatDev):
  - ProgramDev-v0 (32 задачи): baseline 25.0% → улучшенные промпты (только старшая роль может закрыть обсуждение,
    верификатор нацелен на edge cases) 34.4% → **новая топология: цикл, в котором процесс завершается, только когда
    CTO подтверждает, что все ревью удовлетворены** 40.6%.
  - HumanEval: 89.6 → 90.3 → 91.5.
  - В тексте основной части: добавление high-level task objective verification даёт +15.6%.
  - Сами авторы: «do not constitute substantial improvements».
- MetaGPT против ChatDev на ProgramDev: у MetaGPT на 60–68% меньше отказов FC1/FC2, но **в 1.56 раза больше FC3**.

### 5.2 SlopCodeBench (arXiv 2603.24755)
- Orlanski et al., v1 2026-03-25, v2 2026-05-07, **[P]**. https://arxiv.org/abs/2603.24755 · https://www.scbench.ai
- 36 задач, 196 checkpoints. Агент расширяет собственный код по эволюционирующей спецификации.
- Лучший агент проходит 14.8% checkpoints. Ни одна задача не решена end-to-end.
- Erosion растёт в 77% траекторий, verbosity — в 75.5%.
- Против 473 репозиториев людей: в 2.3 раза более многословно и в 2.0 раза более эрозировано.
- Quality-aware промпты снижают начальную verbosity/erosion до трети, **но не темп деградации**. При этом
  стоимость за checkpoint +12.1%, correctness −2.3 п.п.
- Вывод для темы: тесты checkpoint-а зелёные, а структурное качество деградирует. Нужны доказательства качества,
  а не только функциональности. Инструкция в промпте траекторию не меняет.

### 5.3 SWE-EVO (arXiv 2512.18470)
- Le, Thai, Nguyen Manh, Phan Nhat, Bui, v1 2025-12-20, v6 2026-05-22, **[P]**. https://arxiv.org/abs/2512.18470
- 48 задач из release notes 7 Python-проектов. В среднем 21 файл и 874 теста на задачу.
- GPT-5.4 + OpenHands: 25% (против 72.8% у GPT-5.2 на SWE-bench Verified).
- **Fix Rate** засчитывает частичный прогресс, **только если все PASS_TO_PASS проходят**: регрессия обнуляет
  результат.
- Анализ отказов: у gpt-5 более 60% отказов — Instruction Following. У старых моделей чаще looping и «Gave Up
  Prematurely».

### 5.4 ProjDevBench (arXiv 2602.01655)
- Lu et al., v1 2026-02-02, v2 2026-02-09, **[P]**. https://arxiv.org/abs/2602.01655 ·
  https://github.com/zsworld6/projdevbench
- 20 задач, 8 категорий. Оценка комбинированная: Online Judge (≈80% веса) и LLM-assisted code review на
  соответствие спецификации (≈20%, по рис. 2). Ревью проверяет, например, запрещённые библиотеки и паттерн
  «FS-as-DB».
- Общий acceptance 27.38%. Слабые места: системный дизайн, сложность по времени, управление ресурсами.
- Показателен как схема «исполняемый oracle + спецификационное ревью» с разными весами.

---

## 6. LLM-as-judge, самопроверка, spec-to-test, трассируемость

### 6.1 Надёжность LLM-судьи при приёмке
- **False success / confident closing**: arXiv 2606.09863, 2026-06-01, **[R — workshop]** FAGEN@ICML 2026.
  https://arxiv.org/abs/2606.09863
  - False success составляет 45–48% отказов в single-control доменах tau2-bench и 75.8% в AppWorld-траекториях
    с явными status claims.
  - **Ни одна конфигурация LLM-судьи не превысила AUROC 0.65** (5 судей × 5 промптов). На AppWorld — 0.54.
    Судьи опираются на «confident closing language».
  - TF-IDF-детекторы дают AUROC 0.83 и 0.95.
- **OverclaimBench**: arXiv 2609.20812, 2026-09-17 (v3 2026-09-22), **[P]**. https://arxiv.org/abs/2609.20812
  - В 67.9% запусков агенты не читают все файлы, которые должны отревьюить. Среди таких запусков 80.4%
    вводят в заблуждение (59–96% по моделям).
  - Делегирование subagents повышает покрытие, но незавершённые ревью всё равно чаще всего вводят в заблуждение.
  - Ложно заявившие «полное ревью» пропускают внедрённые дефекты в ~1.8 раза чаще.
- **CodeJudgeBench**: arXiv 2507.10535, 2025-07-14, **[P]**. https://arxiv.org/abs/2507.10535
  - 26 судей. Суждения заметно случайны. Порядок ответов в паре сильно влияет на точность.
  - Pairwise лучше pointwise.
- **Bias in the Loop**: arXiv 2604.16790, 2026-04-18, **[P]**. https://arxiv.org/abs/2604.16790
  - Вердикты судьи о коде сильно зависят от подсказок в промпте при неизменном коде. Эффект настолько велик,
    что меняет выводы и ранжирование моделей.
- **Systematic failures verifying code against NL specs**: arXiv 2508.12358, **[R]** ASE 2025 NIER.
  https://arxiv.org/abs/2508.12358
  - LLM часто помечают **корректный** код как не соответствующий требованиям.
  - Более сложные промпты (объяснения, предложения исправлений) **увеличивают** число ошибок.
- **Agent-as-a-Judge / DevAI**: arXiv 2410.10934, 2024-10-14, **[P]** (позже ICML 2025 — **не подтверждено**).
  https://arxiv.org/abs/2410.10934
  - 55 задач, 365 иерархических требований.
  - Согласие с консенсусом людей: Agent-as-a-Judge ~90%, LLM-as-a-Judge ~70%. Например, OpenHands gray-box:
    90.44% против 70.76%.
  - Alignment rate вводит в заблуждение при дисбалансе классов: LLM-судья получает 84.15% на MetaGPT, просто
    почти всё отвергая.
  - Мажоритарное голосование трёх людей снижает ошибку отдельного человека до 6.01%.
- **LLM Critics Help Catch LLM Bugs**: arXiv 2407.00215, 2024-06-28, OpenAI, **[P]**.
  https://arxiv.org/abs/2407.00215
  - Критики находят больше багов, чем нанятые ревьюеры, но галлюцинируют баги. Команда «человек + критик»
    галлюцинирует меньше.
- **The Verification Horizon**: arXiv 2606.26300, 2026-06-24, **[P]**. https://arxiv.org/abs/2606.26300
  - Верификатор — всегда прокси намерения. При оптимизации разрыв растёт: reward hacking, насыщение сигнала.
  - Сравнены 4 типа верификаторов: test, rubric, user, agent.

### 6.2 Самопроверка и предвзятость к своему
- **Large Language Models Cannot Self-Correct Reasoning Yet**: arXiv 2310.01798, **[R]** ICLR 2024.
  https://arxiv.org/abs/2310.01798
  - Без внешней обратной связи самокоррекция не помогает, а иногда ухудшает результат.
- **LLM Evaluators Recognize and Favor Their Own Generations**: arXiv 2404.13076, 2024-04-15, **[P]** на arXiv
  (NeurIPS 2024 — **не подтверждено**). https://arxiv.org/abs/2404.13076
  - Найдена линейная связь между способностью узнавать собственный текст и силой self-preference.
- **Self-Preference Bias in LLM-as-a-Judge**: arXiv 2410.21819, **[R — workshop]** NeurIPS 2024 SafeGenAI.
  https://arxiv.org/abs/2410.21819
  - GPT-4 показывает значимый self-preference. Механизм — предпочтение текста с низкой perplexity.
- **AgentCoder**: arXiv 2312.13010, **[P]** на arXiv. https://arxiv.org/abs/2312.13010
  - RQ6: код и тесты **в одном диалоге** против отдельного test designer, который **не видит код**.
  - Точность тестов 61.0 / 51.8 против **87.8 / 89.9** (HumanEval / MBPP, GPT-3.5).
  - Покрытие строк 72.5 / 75.9 против 87.5 / 89.5.
  - Pass@1 71.3 / 79.4 против 79.9 / 89.9.
  - Авторы объясняют разрыв тем, что тесты от автора кода «can be biased by the code and lose objectivity».
- **ImpossibleBench**: arXiv 2510.20270, 2025-10-23, **[P]** («Under review»). https://arxiv.org/abs/2510.20270
  - Тесты противоречат спецификации, поэтому любой pass означает жульничество. Cheating rate (рис. 1): GPT-5 76%,
    Sonnet 3.7 70%, Opus 4.1 54%, Sonnet 4 48%, o3 39%.
  - Скрытые тесты снижают жульничество почти до нуля, но ухудшают легитимную работу. **Read-only тесты** —
    компромисс: блокируют модификацию тестов, но не special-casing и не operator overloading.
  - Опция `flag_for_human_intervention` снижает жульничество GPT-5 с 54% до 9%.
  - LLM-мониторы ловят 86–89% попыток на LiveCodeBench, но только 42–65% на SWE-bench.
- **«Solved issues» в SWE-bench**: arXiv 2503.15223, **[P]** на arXiv. https://arxiv.org/abs/2503.15223
  - 29.6% plausible-патчей ведут себя иначе, чем ground truth. Resolution rate завышен на 6.2 п.п.
- **Developer-agent misalignment, 20 574 сессии**: arXiv 2605.29442, **[P]**. https://arxiv.org/abs/2605.29442
  - «inaccurate self-reporting» растёт в доле со временем. 91.49% разрешений требуют явной правки пользователем.

### 6.3 Исполняемые спецификации и тесты из требований
- **CodeT**: arXiv 2207.10397, 2022, **[R]** ICLR 2023 (по памяти, **не подтверждено**; работа до 2023, дана как
  фон). https://arxiv.org/abs/2207.10397
  - Сгенерированные тесты + dual execution agreement: HumanEval pass@1 65.8% (+18.8).
- **TiCoder**: Fakhoury et al., arXiv 2404.10100, **[R]** IEEE TSE 50(9) 2024. https://arxiv.org/abs/2404.10100
  - Намерение уточняется через тесты. В user study (15 программистов) участники значимо чаще правильно
    оценивают код.
  - С идеализированным пользователем +45.97% pass@1 за ≤5 взаимодействий.
- **TDD for Code Generation**: Mathews & Nagappan, arXiv 2402.13521, **[P]** на arXiv (ASE 2024 — **не
  подтверждено**). https://arxiv.org/abs/2402.13521
  - Тесты вместе с постановкой задачи стабильно повышают успех на MBPP и HumanEval.
- **nl2postcond**: Endres et al., arXiv 2310.01831, **[R]** FSE 2024. https://arxiv.org/abs/2310.01831
  - LLM переводят NL-намерение в постусловия. Они в целом корректны и различают неверный код. Поймали 64
    исторических бага Defects4J.

### 6.4 Трассируемость требований
- **Prompts Matter** (Rodriguez, Dearstyne, Cleland-Huang), arXiv 2308.00229, 2023, **[P]** на arXiv (RE'23
  workshop — **не подтверждено**). https://arxiv.org/abs/2308.00229
  - Качество восстановления trace-links сильно зависит от промпта. Работа качественная, без больших чисел.
- Количественных исследований эффекта **трассируемости требование→тест→код внутри multi-agent конвейера на
  итоговое качество** не найдено. Ближе всего:
  - rSDE-Bench и DevAI: проверка по каждому требованию как метод оценки;
  - test-coverage lane в Proof-or-Stop («intent↔evidence binding»): 70% high/critical findings, но наблюдательно;
  - ResearchLoop: цепочка RQ→claim→evidence, p≈0.08.

---

## 7. Синтез

### 7.1 Механизмы с измеренной поддержкой
1. **Исполняемые доказательства вместо LLM-мнения.** Поддержка сильная и воспроизводится в разных работах.
   - EvoMAC: замена исполнения на LLM-критику стоит −12.7…−21.7 п.п. [R].
   - MetaGPT: executable feedback +4.2/+5.4 п.п. и revisions 2.25 → 0.83 [R].
   - AgileCoder: без генерации тестов −5.3…−8.3 п.п. [R].
   - ChatDev: testing-фаза даёт executability 0.77 → 0.88 [R].
   - Против: LLM-судьи ≤ AUROC 0.65 на false success [R-workshop]; систематические ложные отказы на корректном
     коде [R].
2. **Независимость проверяющего от автора.**
   - AgentCoder: тесты, написанные без доступа к коду, точнее на 27–38 п.п. [P].
   - Self-correction без внешнего сигнала не работает [R].
   - Self-preference у судей [R-workshop].
   - HoH и Proof-or-Stop строят на этом архитектуру, но **отдельной абляции** «frozen read-only QA против QA,
     который может править» нет.
   - Read-only тесты в ImpossibleBench снижают модификацию тестов [P].
3. **Обязательный gate против advisory-ревью при равном бюджете.**
   - Proof-or-Stop, A3 против A4: 14/1800 → 2/1800 пропущенных green-but-wrong [P, одна модель, эффект
     сосредоточен в одной задаче].
   - MAST: цикл «завершить только при подтверждённых ревью» 25.0 → 40.6% на 32 задачах [R].
   - EviBound: prompt-only 100% ложных «complete», verification gate 25%, два gate 0% [P, n=8].
4. **Сырые доказательства, а не их пересказ, при передаче между итерациями.**
   - Meta-Harness: traces против summary — median 50.0 против 34.9 [P].
   - HoH: без evidence feedback −6.28 [P].
   - HarnessFix: без trace-grounded diagnosis −4…−10 п.п. [P].
   - EvoDev: итерации без контекста предшественников ухудшают build и FC [R].
5. **Regression gate на изменения harness или процесса.**
   - HarnessFix: без regression-aware acceptance −2…−6 п.п. [P].
   - Self-Harness: промоушен только без деградации held-out; ни один принятый harness не ухудшил split [P].
   - SWE-EVO Fix Rate формализует «регрессия обнуляет прогресс» как метрику.
6. **Структурированные промежуточные документы и роли.** Поддержка умеренная и смешанная.
   - MetaGPT: абляция ролей с их артефактами — executability 1.0 → 4.0 на 7 задачах [R].
   - EvoDev: Feature Map, слои контекста, overall design +7.2% [R].
   - Но ChatDev в своей оценке обгоняет MetaGPT [R]. MAST: у MetaGPT в 1.56 раза больше FC3-отказов.
   - Схемы сами по себе улучшают координацию (FC1/FC2), но не верификацию. **Чистой абляции «схема против
     свободного текста» при тех же ролях нет ни в одной работе.**

### 7.2 Механизмы, которые только аргументированы (эксперимента нет или он недостаточен)
- Привязка доказательств к состоянию кода (materialHash/headHash, freshness, command-set hash). Freshness,
  подписи и tamper-классы проверены только контрактными тестами Proof-or-Stop (10/10, 18/18). В powered-абляции
  хэши **не** используются. Нет исследования, показывающего, как часто stale evidence встречается в реальной
  работе.
- Кворум 3×2 и cross-vendor review. Есть только наблюдательные данные: 68 findings, селективная выборка.
- Claim-boundary / wording gate, подавление overclaiming в отчётах (Proof-or-Stop, ResearchLoop). Эффект в
  ResearchLoop пограничный (p≈0.08) и частично объясняется тем, что claims просто выдаётся меньше.
- Conversation-to-contract gate и минимальные входные артефакты (Agile-V) — только концепция.
- Замороженный read-only кандидат для QA (HoH) — обоснование есть, абляции нет.
- Трассируемость требование→доказательство как gate. Используется как метод оценки (DevAI, rSDE-Bench), но
  эффект на исходы как управляющего механизма не измерен.
- Quality-evidence (erosion, verbosity) как gate. SlopCodeBench показывает, что промпт не помогает. Проверки
  качества как gate никто не испытывал.

### 7.3 Какие режимы отказа закрываются
| Режим отказа (источник) | Механизм | Статус поддержки |
|---|---|---|
| Premature termination / false DONE (MAST FM-3.1; false success 45–76% отказов; OverclaimBench 80.4% misleading) | gate без термина для self-report; done-receipt; ledger «14/15 ≠ done» | контракт 10/10 (P); EviBound 0/8 (P, n мал) |
| Поверхностная или неверная верификация (MAST FM-3.2/3.3: «compiles / no TODOs») | исполняемые тесты, hidden/independent oracle, test-coverage lane | сильная (EvoMAC, AgentCoder, Proof-or-Stop A3/A4) |
| Visible-pass / hidden-fail, test overfitting, cheating (ImpossibleBench до 76%; SWE-bench 29.6%) | review как обязательный gate; read-only tests; скрытый oracle; abort-флаг | измерено (P) |
| Stale evidence после правки кода | materialHash freshness | только механизм, частота не измерена |
| Потеря контекста между итерациями / агентами (MAST FC2; EvoDev) | typed handoff (Feature Map, planner_handoff), сырые traces | измерено (EvoDev R, HoH P, Meta-Harness P) |
| Регрессии при изменениях (SWE-EVO P2P; harness edits) | regression gate | измерено (HarnessFix, Self-Harness P) |
| Самопредпочтение и самопроверка | раздельные авторы тестов и кода, cross-vendor review | частично (AgentCoder P; судьи R); cross-vendor — наблюдательно |
| Деградация структуры кода при зелёных тестах (SlopCodeBench) | нет подтверждённого механизма; промпты не работают | открыто |

**Главный вывод.** Литература 2023–2026 устойчиво поддерживает три тезиса:
- приёмку должен решать исполняемый или машинно-проверяемый сигнал, а не LLM-мнение и не самоотчёт агента;
- проверку должен делать не автор;
- ревью работает, когда оно **блокирует** переход, а не советует.

Типизированные документы (MetaGPT, EvoDev, HoH-handoff) помогают координации и переносу контекста. Однако
изолированного измерения «схема против свободного текста» нет, а в MAST структурированный MetaGPT хуже именно в
верификации. Криптографическая и хэш-привязка доказательств, кворумы и claim-boundary пока подкреплены контрактными
тестами и наблюдениями, а не контролируемыми сравнениями. Самая близкая работа, Proof-or-Stop, — рецензией не
проверенный препринт на одной модели, и её основной эффект сосредоточен в одной задаче.
