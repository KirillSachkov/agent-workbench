# Классические механики для artifact-driven pipeline на GitHub

Дата: 2026-09-25. Источники — первичные, где удалось (спецификации, GitHub Docs, авторы).
Утверждение без прямой проверки помечено «не подтверждено». Термины оставлены на английском.

## 0. Главное для нашего контекста

- Организация `sachkov-inside` на плане **Free**; `platform`, `inside-telegram`, `workspace`,
  `inside-landing` — **public**, `ai-engineering`, `inside-content`, `workshop-cases` — **private**
  (проверено `gh repo list sachkov-inside`, `gh api orgs/sachkov-inside`).
  - Artifact attestations: доступны в public-репозиториях на всех планах; для private/internal нужен
    GitHub Enterprise Cloud; на GHES не поддерживаются
    ([actions/attest README](https://github.com/actions/attest),
    [attest-build-provenance README](https://github.com/actions/attest-build-provenance)).
    Значит, для `platform` и `inside-telegram` работают сейчас, для `ai-engineering` — нет.
  - Merge queue: любой public-репозиторий организации либо private при Enterprise Cloud
    ([Managing a merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)).
  - Free для организаций даёт «full feature set» на public-репозиториях
    ([GitHub's plans](https://docs.github.com/en/get-started/learning-about-github/githubs-plans)),
    т. е. rulesets и required checks на public-репозиториях доступны.
- **Срочно:** с **1 октября 2026** retention policy (по умолчанию 90 дней; public — максимум 90,
  private — до 400) распространяется на **checks, workflow runs и commit statuses**, а не только на
  artifacts и logs. До этого они жили 400+ дней. Retention действует и на данные сторонних
  интеграций ([Managing GitHub Actions settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository#configuring-the-retention-period-for-checks-workflow-runs-commit-statuses-artifacts-and-logs-in-your-repository),
  [changelog 2026-07-17](https://github.blog/changelog/2026-07-17-actions-retention-will-cover-checks-workflow-runs-and-statuses/)).
  Вывод: check runs и statuses — это **gate**, а не архив доказательств. Долговечная запись должна
  жить в git (файл) или в attestation.
- Три слоя, которые стоит разделить: **спецификация** (что должно быть: требования с ID, сценарии,
  контракты), **доказательство** (что проверено: машинный отчёт, привязанный к SHA), **gate** (что
  блокирует merge: required check, ruleset, merge queue).

---

## 1. Требования и трассировка

### 1.1 EARS (Easy Approach to Requirements Syntax)

**Что это.** Ограниченный естественный язык для требований, созданный Alistair Mavin и коллегами
в Rolls-Royce, впервые опубликован в 2009 (RE'09). Клаузы всегда в одном порядке:
`While <precondition>, when <trigger>, the <system> shall <response>`. Правило: ноль или больше
preconditions, ноль или один trigger, одна система, одна или больше реакций
([alistairmavin.com/ears](https://alistairmavin.com/ears/)). Оригинальная статья: Mavin et al., «Easy Approach to Requirements Syntax (EARS)», RE'09 (библиографические детали по странице автора; PDF не открывался).

Шаблоны (там же):

| Шаблон | Синтаксис |
|---|---|
| Ubiquitous | `The <system> shall <response>` |
| State-driven | `While <precondition>, the <system> shall <response>` |
| Event-driven | `When <trigger>, the <system> shall <response>` |
| Optional feature | `Where <feature is included>, the <system> shall <response>` |
| Unwanted behaviour | `If <trigger>, then the <system> shall <response>` |
| Complex | `While …, when …, the <system> shall …` (и сочетания с If/Then) |

**Минимальный пример (наш домен):**

```markdown
- REQ-PAY-003 (event): When the payment provider confirms a payment, the platform shall grant
  course access to the paying user within 60 seconds.
- REQ-PAY-004 (unwanted): If the payment notification signature is invalid, then the platform
  shall reject the notification and shall not change the order state.
- REQ-BOT-010 (state): While the user has no linked account, the Telegram bot shall offer the
  account-linking flow.
```

**Какой сбой предотвращает.** Размытые требования («должно работать быстро»), смешение условия
и реакции, забытые негативные ветки (шаблон If/Then заставляет их писать). Для агентов важно:
EARS проверяется линтером регулярным выражением (ключевые слова, одно `shall`, наличие системы).

**Стоимость на GitHub.** Почти нулевая: шаблон в спецификации + простой lint-скрипт в CI.

**Подводные камни.** EARS не делает требование проверяемым само по себе: «within 60 seconds» —
да, «user-friendly» — нет. Один `shall` на требование, иначе трассировка теряет гранулярность.
EARS описывает поведение системы, не UI-детали; для UI лучше Gherkin-сценарий (раздел 2).

### 1.2 ID требований, RTM и двунаправленная трассировка

**Что это.** У каждого требования стабильный уникальный ID; Requirements Traceability Matrix
(RTM) связывает требование → дизайн → код → тест → результат. Двунаправленность: forward
(каждое требование реализовано и проверено) и backward (каждый код/тест обоснован требованием).
В DO-178C трассы идут system req ↔ high-level req ↔ low-level req ↔ source code, а также
req ↔ test cases ↔ procedures ↔ results; глубина зависит от уровня DAL (для DAL D достаточно
трасс до high-level requirements и тестов) — источники вторичные, сам стандарт RTCA платный
([Parasoft: DO-178C traceability](https://www.parasoft.com/learning-center/do-178c/requirements-traceability/),
[Jama: DO-178C](https://www.jamasoftware.com/requirements-management-guide/aerospace-and-defense/do-178c/)).
ISO 26262-8 (clause 6) требует уникально идентифицированных, версионируемых требований и
двунаправленной трассировки через фазы — по вторичным источникам, текст стандарта не проверен
([Sodius Willert](https://www.sodiuswillert.com/en/blog/maintaining-iso-26262-traceability-across-automotive-suppliers)) —
«не подтверждено» по первоисточнику.

**Механика, которую стоит взять (без сертификационного веса):**
1. ID неизменяем, не переиспользуется; ревизия в ID (`~1`, `~2`) при смысловом изменении.
2. Каждое «нижнее» звено ссылается на «верхнее» (`Covers: REQ-…`), а «верхнее» объявляет, какие
   виды покрытия ему нужны (`Needs: impl, test`).
3. Инструмент в CI строит граф и падает на: uncovered (нет покрытия нужного вида), orphan
   (ссылка на несуществующий ID), outdated (покрытие ссылается на старую ревизию).

**Какой сбой предотвращает.** «Сделали, но не то»; тест без требования; требование, которое
молча выпало при рефакторинге; агент, утверждающий «AC выполнены» без привязки к тесту.

### 1.3 Лёгкие инструменты

**OpenFastTrace (OFT)** — Java, CLI; спецификации в Markdown/RST, теги в коде. ID имеет вид
`<artifact-type>~<name>~<revision>`. В Markdown:

```markdown
`req~pay-grant-access~1`
When the payment provider confirms a payment, the platform shall grant course access.

Needs: impl, utest, itest
```

В коде (TypeScript и Python поддерживаются):

```ts
// [impl->req~pay-grant-access~1]
export async function grantAccess(orderId: string) { … }
```

В Gherkin (`.feature`): тег `@id:scn~user-can-log-in~1` и комментарии `# Covers: req~…~1`,
`# Needs: dsn, itest` перед заголовком сценария. Покрытие с ревизией:
`[impl~~2->dsn~validate-authentication-request~1]`
([OFT input formats](https://github.com/itsallcode/openfasttrace/blob/main/doc/user_guide/reference/input_format_support.md),
[OFT system requirements](https://github.com/itsallcode/openfasttrace/blob/main/doc/spec/system_requirements.md)).
Отчёт помечает `not ok`, включая transitive defects
([console report](https://github.com/itsallcode/openfasttrace/blob/main/doc/user_guide/reference/console_tracing_report.md)).
Код возврата CLI при дефектах трассировки использовать как gate — «не подтверждено» (не читал
раздел CLI exit codes).

**StrictDoc** — Python; свой формат `.sdoc`, экспорт HTML/ReqIF/JSON (экспорт — по докам
проекта, детально не проверял). Пример из user guide:

```text
[REQUIREMENT]
UID: REQ-002
TITLE: Requirement #2's title
STATEMENT: Requirement #2 statement
RELATIONS:
- TYPE: Parent
  VALUE: REQ-001
- TYPE: File
  VALUE: /full/path/file.py
```

В исходниках маркер `@relation(REQ-1, scope=function)` (также `scope=file|class`)
([StrictDoc user guide .sdoc](https://github.com/strictdoc-project/strictdoc/blob/main/docs/strictdoc_01_user_guide.sdoc)).

**Sphinx-needs** — расширение Sphinx: директивы `.. req::` с `:id:`, `:status:`, `:links:`,
экспорт `needs.json`, `needtable`/`needflow`, constraints и regex на ID
([sphinx-needs docs](https://sphinx-needs.readthedocs.io/en/latest/)). Требует Sphinx-сборки
документации — для нашего Markdown-мира тяжеловато.

**Doorstop** — Python; каждое требование — отдельный YAML-файл в git, UID с префиксом (`REQ001`),
документы образуют дерево, есть reviewed/fingerprint для «suspect links», команда `doorstop`
валидирует дерево в CI ([doorstop docs](https://doorstop.readthedocs.io/en/latest/)).

**Сравнение для нас.** Ни один не нативен для TS. OFT ближе всего к нашему процессу (Markdown +
комментарии в TS + Gherkin), но тянет JVM. Для небольшой команды дешевле **свой минимальный
трассировщик на Python в harness**: ID-регекс по `docs/specs/**/*.md`, `// covers: REQ-…` в
тестах, Playwright `annotation`/`tag` с ID (раздел 2.2), JSON-отчёт → gate. OFT — как эталон
семантики (`Needs`/`Covers`/revision/outdated).

**Подводные камни.** Трассировка «ради галочки»: агент ставит `covers:` на тест, который не
проверяет поведение. Лечится только ревью теста и связкой требование → конкретный assertion
(Gherkin-шаг, Playwright-шаг). Ревизии ID: без них изменение смысла требования не инвалидирует
старые покрытия.

---

## 2. Исполняемые спецификации

### 2.1 BDD / Gherkin и Specification by Example

**Что это.** BDD по Cucumber — три практики: Discovery (структурированные разговоры на примерах),
Formulation (запись примеров как структурированной документации), Automation
([Cucumber: BDD](https://cucumber.io/docs/bdd/)). Gherkin — формат `Feature / Rule / Scenario /
Scenario Outline / Examples`, теги `@…` на Feature, Rule, Scenario; `Rule` — одно бизнес-правило
(с Gherkin 6) ([Gherkin reference](https://cucumber.io/docs/gherkin/reference/)).
Specification by Example — книга Gojko Adzic (2011), та же идея: примеры как единый источник для
требований, тестов и «living documentation»
([gojko.net](https://gojko.net/books/specification-by-example/)).

**Пример:**

```gherkin
@id:scn~pay-grant-access~1
# Covers: req~pay-grant-access~1
Feature: Access after payment
  Rule: Confirmed payment grants access
    Scenario: Card payment confirmed
      Given a user with an unpaid order for "AI Engineering"
      When the provider sends a valid CONFIRMED notification
      Then the user sees the course in "My courses" within 60 seconds
```

**Какой сбой предотвращает.** Acceptance criteria, которые нельзя выполнить механически;
расхождение «что обещали» и «что проверили»; агент, интерпретирующий AC свободно.

**Стоимость.** Cucumber-js или `playwright-bdd` (сторонний) — средняя; нужен слой step
definitions. Альтернатива дешевле: Gherkin-подобный текст в issue как AC, а исполняемость — через
Playwright-тест, помеченный тем же ID (2.2).

**Подводные камни.** Step-definition-ад и императивные сценарии («click button X»). Gherkin ценен
на уровне бизнес-правил, не UI-скриптов. Если никто, кроме агентов, не читает `.feature`, польза
Discovery теряется — остаётся только формат.

### 2.2 Playwright-тесты как acceptance evidence

**Механика.** Playwright поддерживает `tag` (строки с `@`) и `annotation` (`{type, description}`),
которые доступны в reporter API и видны в HTML-отчёте; фильтр `--grep @tag`
([Playwright annotations](https://playwright.dev/docs/test-annotations)). JSON reporter пишет
полный результат прогона (`--reporter=json`, `PLAYWRIGHT_JSON_OUTPUT_NAME`)
([Playwright reporters](https://playwright.dev/docs/test-reporters)).

```ts
test('card payment grants access', {
  tag: ['@acceptance'],
  annotation: [{ type: 'covers', description: 'REQ-PAY-003' },
               { type: 'issue', description: 'sachkov-inside/platform#812' }],
}, async ({ page }) => { /* … */ });
```

Скрипт в CI читает `results.json`, собирает `{requirementId → [test, status]}` и сверяет со
списком ID из спецификации задачи. Отсутствующий или упавший ID → check `failure`.

**Предотвращает.** «Все тесты зелёные», но нужный AC вообще не тестировался.
**Стоимость.** Низкая: конвенция + ~100 строк скрипта. **Камни.** Скриншоты/видео — это
иллюстрация, не доказательство; доказательство — assertion. Флейки дают ложные «PASSED/FAILED»:
нужен retry-учёт в отчёте (Playwright помечает flaky).

### 2.3 Контракты между producer и consumer

**Consumer-driven contracts (Pact).** Контракт генерируется тестами consumer'а; provider
проверяет его; результаты хранятся в Pact Broker; «Pact Matrix» версий и `can-i-deploy` решает,
можно ли деплоить версию в окружение (`record-deployment` отмечает, что где стоит)
([Pact docs](https://docs.pact.io/),
[can-i-deploy](https://docs.pact.io/pact_broker/can_i_deploy)). Pact хорош, когда обе стороны
под вашим контролем и активно развиваются; плох для public API, pass-through API/BFF и
ситуаций, где нельзя управлять данными provider'а
([What is Pact good for](https://docs.pact.io/getting_started/what_is_pact_good_for)).

**Schema-first (OpenAPI / JSON Schema).** «Provider contract testing» — проверка, что поведение
provider'а совпадает с документированным контрактом (например OpenAPI); сама по себе не
доказывает, что consumer вызывает API правильно (там же, [Pact docs](https://docs.pact.io/)).
Для CI есть `oasdiff` — diff OpenAPI с уровнями `ERR/WARN/INFO` и `fail-on`
([oasdiff](https://github.com/oasdiff/oasdiff), [oasdiff-action](https://github.com/oasdiff/oasdiff-action)).

**Пример gate:**

```yaml
- uses: oasdiff/oasdiff-action/breaking@<pinned-sha>
  with:
    base: 'origin/main:apps/api/openapi.json'
    revision: 'apps/api/openapi.json'
    fail-on: ERR
```

(точные имена inputs — «не подтверждено», сверить с README action).

**Предотвращает.** Молчаливые breaking changes между `platform` API и `inside-telegram`/web.
**Стоимость.** OpenAPI из NestJS (`@nestjs/swagger`) + oasdiff — низкая; Pact + Broker — средняя
(отдельный сервис или PactFlow). **Для нас:** web и API в одном монорепо — общие TS-типы и
сгенерированный клиент дешевле Pact; между репозиториями (`platform` ↔ `inside-telegram`) —
закоммиченный `openapi.json` как артефакт + oasdiff. **Камни.** Сгенерированный из кода OpenAPI
отражает код, а не намерение; schema-first (правка схемы до кода) даёт больше, но дороже.

---

## 3. Доказательства и provenance, привязанные к коммиту

### 3.1 in-toto attestation framework

**Что это.** Слои: Envelope (DSSE, подпись) → Statement → Predicate. Statement v1
([spec](https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md)):

```jsonc
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [{ "name": "<NAME>", "digest": { "<ALGORITHM>": "<HEX_VALUE>" } }],
  "predicateType": "<URI>",
  "predicate": { … }
}
```

Subject сопоставляется **только по digest**; subjects считаются неизменяемыми. DigestSet
поддерживает `sha256`, `sha512`, …, а также `gitCommit`, `gitTree`, `gitBlob`, `gitTag`
(SHA-1 40 символов или SHA-256 64) ([DigestSet](https://github.com/in-toto/attestation/blob/main/spec/v1/digest_set.md)).
Есть готовый predicate **Test Result** `https://in-toto.io/attestation/test-result/v0.1`:
`result: PASSED|WARNED|FAILED`, `configuration`, `url`, `passedTests`, `warnedTests`,
`failedTests`; subject — протестированные исходные артефакты
([test-result predicate](https://github.com/in-toto/attestation/blob/main/spec/predicates/test-result.md)).

### 3.2 SLSA

**Build track** ([SLSA v1.1 levels](https://slsa.dev/spec/v1.1/levels)):
L1 — provenance существует (предотвращает ошибки релиза, например сборку из коммита, которого
нет в upstream); L2 — подписанная provenance от hosted build platform (предотвращает подмену
после сборки); L3 — hardened builds, изоляция запусков и секретов подписи (предотвращает подмену
во время сборки инсайдером/украденными credentials).

**Source track (SLSA v1.2)** ([source requirements](https://slsa.dev/spec/v1.2/source-requirements)):
L1 version controlled; L2 history & provenance (непрерывная неизменяемая история + source
provenance attestations); L3 continuous technical controls (enforced правила на защищённых ref);
L4 two-party review. Уровень ревизии сообщается **Verification Summary Attestation**
(`https://slsa.dev/verification_summary/v1`) с `subject.digest.gitCommit` и `verifiedLevels`.
Это ровно наш кейс: «ревизия X прошла проверки Y» как подписанный документ на `gitCommit`.

GitHub заявляет: artifact attestations сами по себе дают **SLSA v1.0 Build L2**; reusable
workflow как изолированный builder — путь к **L3**
([GitHub: artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations)).

### 3.3 GitHub artifact attestations

- `actions/attest@v4` — три режима: provenance (без predicate), SBOM (`sbom-path`), **custom**
  (`predicate-type` + `predicate` или `predicate-path`). Subject: `subject-path`,
  `subject-digest` (+`subject-name`), `subject-checksums`. Permissions: `id-token: write`,
  `attestations: write`, `artifact-metadata: write`. Лимиты: predicate ≤ 16 MB, ≤ 1024 subjects.
  `subject-digest` **обязан быть `sha256:…`**; checksums-файлы — sha256 (sha512 в checksum-файле)
  ([actions/attest README](https://github.com/actions/attest)).
- `actions/attest-build-provenance` с v4 — просто обёртка над `actions/attest`
  ([README](https://github.com/actions/attest-build-provenance)).
- Public repo → Sigstore Public Good; bundle хранится у GitHub **и пишется в публично читаемый
  transparency log**. Private → собственный Sigstore GitHub без transparency log
  ([GitHub docs](https://docs.github.com/en/actions/concepts/security/artifact-attestations)).
- REST: `GET /repos/{owner}/{repo}/attestations/{subject_digest}` (`sha256:HEX`, фильтр
  `predicate_type`), `POST /repos/{owner}/{repo}/attestations` принимает Sigstore bundle
  ([REST attestations](https://docs.github.com/en/rest/repos/attestations)).
- `gh attestation verify <file> | oci://…` требует `--owner` или `--repo`; по умолчанию
  предикат `https://slsa.dev/provenance/v1`, иначе `--predicate-type`. Полезные флаги:
  `--signer-workflow`, `--signer-repo`, `--source-digest`, `--source-ref`,
  `--deny-self-hosted-runners`, `--format json`, `--jq`, `--bundle` (offline). Важно: доверять
  можно только `signature.certificate` (из OIDC-токена) и `verifiedTimestamps`;
  `statement.predicate` контролируется workflow и может быть подделан при компрометации
  контекста — отсюда рекомендация «trusted builder» в reusable workflow
  ([gh attestation verify](https://cli.github.com/manual/gh_attestation_verify)).
- GitHub прямо советует **не** подписывать частые сборки «just for automated testing» и отдельные
  файлы исходников/документации; attestations полезны только если их верифицируют
  ([GitHub docs](https://docs.github.com/en/actions/concepts/security/artifact-attestations)).

### 3.4 Check Runs, commit statuses, rulesets, merge queue

- **Check Runs API:** создавать может только GitHub App (OAuth/пользователи — только читать);
  `GITHUB_TOKEN` в Actions — токен GitHub App, `checks: write` разрешает создавать check run
  ([Check runs REST](https://docs.github.com/en/rest/checks/runs),
  [workflow permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)).
  Поля: `name`, `head_sha`, `status`, `conclusion` (`success|failure|neutral|cancelled|skipped|
  timed_out|action_required`), `details_url`, `external_id`, `output{title, summary (Markdown),
  text, annotations, images}`. Annotations — максимум 50 за запрос, дальше дописываются через
  update; в Actions — 10 warning + 10 error на step; title аннотации ≤ 255 символов; до 3
  `actions`-кнопок; в одном check suite до 1000 check runs с одним именем, старые удаляются.
  Лимит длины `summary` (часто цитируют 65535) — «не подтверждено» в текущей версии доков.
- **Job summary** (`$GITHUB_STEP_SUMMARY`): ≤ 1 MiB на step, показывается ≤ 20 summaries на job
  ([workflow commands](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands)).
- **Commit statuses:** `error|failure|pending|success`, `context`, `target_url`, `description`;
  ≤ 1000 статусов на sha+context; нужен push-доступ
  ([statuses REST](https://docs.github.com/en/rest/commits/statuses)).
- **Required status checks в rulesets:** можно указать **expected source** — конкретный GitHub App;
  статус от другого источника не разблокирует merge. Без этого «любой с write-доступом может
  выставить любой статус» ([available rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)).
  Rulesets: до 75 на репозиторий, видны всем с read-доступом, слоятся с branch protection
  ([about rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)).
- **Merge queue:** workflow обязан слушать `merge_group`, иначе required check не придёт и merge
  упадёт; очередь собирает временную ветку `main/pr-N` с base и предыдущими PR
  ([merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)).
- **SHA-ловушка:** на `pull_request` `GITHUB_SHA` — последний merge-коммит ветки
  `refs/pull/N/merge`, а не head PR; head — `github.event.pull_request.head.sha`. На
  `merge_group` — SHA merge group ([events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)).
  Итог: одно и то же «доказательство» существует минимум для трёх разных SHA (head PR,
  merge-ref, merge_group), а после squash в `main` — для четвёртого.
- **Rate limits:** `GITHUB_TOKEN` — 1000 запросов/час на репозиторий; secondary — ≤ 80
  content-generating запросов/мин и ≤ 500/час
  ([REST rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)).

### 3.5 Как сделать «acceptance evidence» attestation для SHA и проверить до merge

Проблема: GitHub хранит и ищет attestations по **sha256 digest файла**, а коммит — это
`gitCommit` (SHA-1). Поэтому subject — сам evidence-файл, а коммит — внутри predicate **и** в
сертификате (поля source repository digest/ref из OIDC, проверяются `--source-digest`,
`--source-ref`). Что `source-digest` в сертификате равен `github.sha` запуска — следует из
логики OIDC claims, экспериментом не проверено («не подтверждено»).

**Workflow (public-репозиторий `platform`):**

```yaml
name: acceptance-evidence
on:
  merge_group:          # gate на итоговом дереве, которое попадёт в main
  pull_request:         # ранняя обратная связь
permissions:
  contents: read
  checks: write
  id-token: write
  attestations: write
  artifact-metadata: write
jobs:
  acceptance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<sha>
      - run: PLAYWRIGHT_JSON_OUTPUT_NAME=pw.json pnpm playwright test --grep @acceptance --reporter=json
      - run: python3 tools/evidence.py --spec docs/specs/PAY-812.md --results pw.json
               --commit "${{ github.sha }}" --out evidence/acceptance.json
        # падает, если какой-то REQ из спеки не покрыт PASSED-тестом
      - if: github.event_name == 'merge_group'
        uses: actions/attest@v4
        with:
          subject-path: evidence/acceptance.json
          predicate-type: https://in-toto.io/attestation/test-result/v0.1
          predicate-path: evidence/predicate.json
      - uses: actions/upload-artifact@<sha>
        with: { name: acceptance-evidence, path: evidence/, retention-days: 90 }
```

**predicate.json (стандартный test-result, наши ID в именах тестов):**

```json
{
  "result": "PASSED",
  "configuration": [{ "name": "playwright.config.ts", "digest": { "gitBlob": "…" } },
                    { "name": "spec", "uri": "git+https://github.com/sachkov-inside/platform@<sha>#docs/specs/PAY-812.md" }],
  "url": "https://github.com/sachkov-inside/platform/actions/runs/<run_id>",
  "passedTests": ["REQ-PAY-003 :: card payment grants access"],
  "warnedTests": [],
  "failedTests": []
}
```

**Проверка:**

```bash
gh attestation verify evidence/acceptance.json \
  --repo sachkov-inside/platform \
  --predicate-type https://in-toto.io/attestation/test-result/v0.1 \
  --signer-workflow sachkov-inside/platform/.github/workflows/acceptance-evidence.yml \
  --source-digest "$SHA" --deny-self-hosted-runners \
  --format json --jq '.[0].verificationResult.statement.predicate.result'
```

**Честная оценка.** Gate до merge дешевле и надёжнее сделать **required check** (conclusion
check run = результат `evidence.py`) с expected source = GitHub Actions; attestation добавляет
не блокировку, а **долговечную, подписанную, внешне проверяемую запись** (переживает retention
checks; для public — ещё и в transparency log). Attestation полезна как аудит «что было принято и
чем доказано», при релизе или для внешнего проверяющего. Подписывать каждый push не нужно
(совет GitHub выше) — только `merge_group`/релиз. Для private-репозиториев на Free недоступно;
замена — evidence-файл в git + commit status/check.

**Камни.** (1) predicate пишет workflow — если агент может менять workflow в том же PR, он может
подделать evidence; лечится `--signer-workflow`, reusable workflow из отдельного репо и CODEOWNERS
на `.github/`. (2) Публичный transparency log: никаких персональных данных/секретов в predicate.
(3) SHA-ловушка из 3.4. (4) «Merge group SHA станет коммитом в `main`» — «не подтверждено»,
зависит от merge method; привязывайте ещё и `gitTree` в predicate.

---

## 4. Assurance cases: GSN, SACM, CAE

**GSN.** Goal Structuring Notation, стандарт сообщества SCSC, текущая версия — **3**
([SCSC GSN standard](https://scsc.uk/gsn-standard)). Элементы: Goal (утверждение), Strategy
(как разложили), Solution (ссылка на evidence), Context, Assumption, Justification; связи
SupportedBy и InContextOf; «undeveloped» для не доказанных веток (состав элементов — по
стандарту v3, в этой сессии PDF не открывался; «не подтверждено» в деталях).

**CAE (Adelard / Bloomfield).** Claims (утверждение, может быть истинным или ложным), Arguments
(почему evidence поддерживает claim), Evidence (артефакты, устанавливающие факты); «CAE building
blocks» — типовые шаблоны декомпозиции (decomposition, substitution, concretion, calculation,
evidence incorporation) ([Adelard CAE](https://www.adelard.com/asce/cae/),
[Bloomfield & Netkachova, Building Blocks](https://openaccess.city.ac.uk/id/eprint/5121/1/BuildingBlocksforAssuranceCases.pdf)).

**SACM.** OMG Structured Assurance Case Metamodel: объединяет argumentation и artifact
metamodel, служит обменным форматом для GSN и CAE. На странице OMG — **2.4 beta**
(сентябрь 2026) и формальные версии до 2.3 ([OMG SACM](https://www.omg.org/spec/SACM/)).

**Маппинг на записи агента («claim → evidence»):**

```yaml
# .evidence/PAY-812.claims.yaml  (append-only, один писатель — CI)
claim: C1
statement: "REQ-PAY-003 выполнено на коммите <sha>"
context: [spec: docs/specs/PAY-812.md@<sha>]
argument: "Сценарий scn~pay-grant-access~1 проходит на merge_group; тест проверяет выдачу доступа"
evidence:
  - kind: test-result
    ref: attestation:<id> | run:<url>
    subject_digest: sha256:…
assumptions: ["провайдер в sandbox эмулирует CONFIRMED так же, как prod"]
status: supported | undeveloped | defeated
```

**Какой сбой предотвращает.** Главная болезнь агентских отчётов — «сделано, всё работает» без
связи с доказательством. CAE заставляет разделить **утверждение**, **рассуждение** и
**доказательство**, а `undeveloped`/`assumptions` делают дыры видимыми владельцу.

**Стоимость.** Формат — низкая (YAML + schema). Полноценный GSN-редактор не нужен.
**Камни.** Assurance case легко превращается в бумажную работу; держите 1 claim на требование и
evidence только машинного происхождения (ссылка на run/attestation), а текст argument — коротким.
Оценка «убедительности» аргумента остаётся за человеком.

---

## 5. Design by Contract, DoR/DoD, fitness functions

**Design by Contract (Meyer, Eiffel).** Preconditions (обязанность клиента), postconditions
(обязанность поставщика), invariants ([Eiffel: DbC](https://www.eiffel.com/values/design-by-contract/introduction/)).
Перенос на pipeline: каждый этап (spec → tickets → implementation → review) — «метод» с
pre/postconditions над артефактами. Пример: precondition `implementation` = issue имеет
валидный spec-блок, все REQ-ID в EARS, есть `Needs`; postcondition = evidence покрывает все
REQ-ID и check `acceptance` = success.

**Definition of Ready / Done.** Scrum Guide: DoD — «формальное описание состояния Increment,
когда он соответствует требуемым мерам качества»; элемент бэклога, отвечающий DoD, становится
Increment ([Scrum Guide 2020](https://scrumguides.org/scrum-guide.html)). DoR в Scrum Guide нет —
это практика команд. Механика: DoR/DoD как **список машинно-проверяемых предикатов** (JSON Schema
issue-блока, наличие ссылок, required checks), а не чеклист в markdown, который агент «отмечает».

```yaml
# harness/gates/dod.yaml
- id: dod.spec-linked      check: pr.body.links.issue != null
- id: dod.reqs-covered     check: evidence.uncovered == []
- id: dod.contract-stable  check: oasdiff.breaking == [] or pr.labels has 'breaking-approved'
- id: dod.owner-approved   check: review.approved_by contains owner   # человек, не агент
```

**Architecture fitness functions.** Из *Building Evolutionary Architectures* (Ford, Parsons, Kua,
Sadalage): объективная автоматическая оценка архитектурной характеристики — тесты, метрики,
мониторы ([Thoughtworks Radar](https://www.thoughtworks.com/radar/techniques/architectural-fitness-function),
[O'Reilly, 2nd ed.](https://www.oreilly.com/library/view/building-evolutionary-architectures/9781492097532/) — страница издателя не открылась, состав авторов 2-го издания «не подтверждено»).
Для нас: dependency-cruiser/eslint-boundaries (web не импортирует backend), лимит размера
бандла, oasdiff, «каждый NestJS-контроллер имеет OpenAPI-описание», p95 времени ответа в
smoke. Хорошо сочетается с MADR-разделом `Confirmation` (6.4): решение → fitness function.

**Предотвращает.** Эрозию архитектуры и «галочки» в DoD. **Стоимость** — низкая, большинство
проверок уже есть как lint/test. **Камни.** Fitness function без владельца и порога → шум; порог
меняется только через ADR.

---

## 6. Жизненный цикл артефактов

**Immutable records + append-only log vs mutable documents.** Event Sourcing (Fowler): состояние
выводится из последовательности событий, которые не переписываются
([Fowler: Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)). Правило для нас:
**спецификация** — mutable документ с версией (правится через PR, ревизия ID растёт);
**evidence, решения о приёмке, attestations** — immutable, каждая запись привязана к SHA и
никогда не редактируется; исправление — новая запись, ссылающаяся на старую (`supersedes`).

**State machine артефакта.** Явные состояния и допустимые переходы, проверяемые CI:

```text
spec:     draft → proposed → approved(owner) → superseded
evidence: pending → passed | failed   (terminal; новый SHA = новая запись)
adr:      proposed → accepted | rejected → deprecated | superseded-by:ADR-NNNN
```

Переход `approved` может делать только владелец (проверка автора события/label через ruleset или
Action). Недопустимый переход → check failure.

**Single-writer ownership.** Принцип Martin Thompson: у каждого элемента состояния один писатель
— устраняет конфликты и гонки ([Single Writer Principle](https://mechanical-sympathy.blogspot.com/2011/09/single-writer-principle.html); страница в этой сессии не открылась — формулировка по памяти, «не подтверждено»).
Перенос: у каждого поля артефакта один владелец записи — agent пишет spec/код, **только CI**
пишет evidence и check conclusions, **только owner** ставит approval. Это же защищает от
агента, который «сам себе выдал доказательство».

**ADR lifecycle.** Nygard (2011): короткий файл, номера последовательны и не переиспользуются,
отменённое решение не удаляется, а помечается superseded; статусы proposed / accepted /
deprecated / superseded; разделы Context, Decision, Status, Consequences
([Nygard](https://www.cognitect.com/blog/2011/11/15/documenting-architecture-decisions)).
MADR 4.0.0 (2024-09-17): front matter `status: proposed | rejected | accepted | deprecated |
superseded by ADR-0123`, `decision-makers`, разделы Considered Options, Decision Outcome,
Consequences, **Confirmation** (как проверить, что решение соблюдается)
([MADR](https://adr.github.io/madr/)). Front matter валидируется JSON Schema в CI; ссылка
`superseded by` проверяется на существование.

**Предотвращает.** Потерю истории решений, «тихие» правки принятых критериев, гонки нескольких
агентов в одном файле. **Камни.** Immutable-записи в git растут; держите их маленькими (JSON) и
ссылайтесь на тяжёлое (отчёты, скриншоты) через artifacts/attestation digest.

---

## 7. GitHub-native хранилища структурированных артефактов

| Хранилище | Что хорошо | Ограничения / риски | Подходит для |
|---|---|---|---|
| **Issue forms** (`.github/ISSUE_TEMPLATE/*.yml`) | Типизированные поля, `validations.required`, dropdown/checkbox | Public preview; ответы превращаются в **markdown в body**, body потом свободно редактируется; для PR не поддерживаются ([issue forms syntax](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms)) | Приём задачи, DoR-поля при создании |
| **Fenced JSON/YAML в body issue/PR** + Action-парсер | Машиночитаемо, видно человеку, валидируется JSON Schema на `issues.edited`/`pull_request` | Нет истории на уровне поля (только история правок body); любой с правом edit меняет; конкурирующие правки агентов | Spec-блок задачи, PR evidence-summary |
| **Projects custom fields** | Типы text/number/date/single-select/iteration; фильтры | До 50 полей на проект, до 50 000 items ([understanding fields](https://docs.github.com/en/issues/planning-and-tracking-with-projects/understanding-fields), [changelog](https://github.blog/changelog/2025-02-26-increased-items-in-github-projects-now-in-public-preview/)); только GraphQL; не версионируются | Статус/стадия, владелец, ссылки |
| **Issue fields (org-level)** | Типизированные поля на всех issues организации; single-select, text, number, date; до 25 на org; видимость Public/Organization only ([issue fields](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/managing-issue-fields-in-your-organization)) | Новая функция, API/доступность на Free — «не подтверждено» | Замена части Project-полей |
| **Файлы в репо + front matter + JSON Schema в CI** | Версионирование, review, привязка к SHA, blame, ruleset/CODEOWNERS | Нужен PR на каждое изменение; merge-конфликты | Спецификации, ADR, claims/evidence-индекс |
| **Check run output / commit status** | Привязан к `head_sha`, gate через required checks, аннотации на строках | Писать может только GitHub App/`GITHUB_TOKEN`; annotations 50/запрос; **retention 90 дней по умолчанию с 01.10.2026**, для public максимум 90 | Gate и отчёт о проверке |
| **Workflow artifacts** | Большие файлы (отчёты, trace, видео), `retention-days` | Retention по умолчанию 90 дней; public 1–90, private 1–400 ([storing workflow data](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/storing-and-sharing-data-from-a-workflow), [retention settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository)); квоты хранения по плану ([Actions limits](https://docs.github.com/en/actions/reference/limits)) | Тяжёлое evidence |
| **Attestations** | Подпись Sigstore, OIDC-идентичность workflow, поиск по digest, для public — transparency log | Public на любом плане, private — только Enterprise Cloud; subject только sha256; predicate ≤ 16 MB; удаление — по жизненному циклу ([manage attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/manage-attestations)); срок хранения у GitHub — «не подтверждено» | Долговечная приёмка на merge/release |
| **git notes** | Метаданные к коммиту без изменения SHA | Не push/fetch по умолчанию (нужны refspec `refs/notes/*`); GitHub UI их не показывает ([Ken Muse](https://www.kenmuse.com/blog/storing-data-in-git-objects-with-notes/)); rulesets защищают ветки/теги, не `refs/notes` — вывод, «не подтверждено» | Не рекомендую как источник истины |

Общие лимиты API: `GITHUB_TOKEN` 1000 req/h/repo; secondary 80 content-creating/min, 500/h
([rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)).
Агенты, массово правящие issues/Project-поля, упираются именно в secondary limits.

---

## 8. Рекомендуемый минимальный toolkit

Принцип: **спецификация — в git, gate — в checks, долговечное доказательство — в git + attestation**.
Issue/Project — навигация и статус, не источник истины для критериев.

1. **Spec-файл на задачу** `docs/specs/<issue>.md` с YAML front matter
   (`id, issue, status, owner, supersedes`) и требованиями в **EARS** с ID `REQ-<AREA>-<NNN>`
   (ревизия при смысловой правке). JSON Schema для front matter + lint EARS (Python в harness).
   Issue form для создания задачи ссылается на этот файл.
2. **Трассировка своим скриптом по семантике OFT**: `Needs` у требования, `covers` в Playwright
   `annotation` / vitest-комментарии; отчёт `uncovered / orphan / outdated` → required check
   `trace`. OFT подключать, только если свой скрипт начнёт расти.
3. **Acceptance evidence** — Playwright JSON → `evidence.py` → check run `acceptance` (summary =
   таблица REQ → тест → статус, annotations на непокрытых REQ). Required, expected source =
   GitHub Actions. Workflow слушает `pull_request` и **`merge_group`**.
4. **Контракты**: закоммиченный `openapi.json` из NestJS + `oasdiff` (`fail-on: ERR`) как required
   check; разрешение breaking — только label владельца. Pact не вводить, пока нет нескольких
   независимо деплоящихся consumer'ов.
5. **Attestation на merge_group** (public `platform`, `inside-telegram`): `actions/attest@v4` с
   predicate `test-result/v0.1` на evidence-файл; `gh attestation verify` с
   `--signer-workflow` и `--source-digest` в релизном шаге. Для private-репозиториев — только
   evidence-файл + check.
6. **Claims-файл по CAE** (`.evidence/<issue>.claims.yaml`), пишет **только CI**; агент в PR
   ссылается на claim ID, а не пересказывает «всё проверено».
7. **Rulesets**: required checks `trace`, `acceptance`, `contract`, `schema`; CODEOWNERS на
   `.github/workflows/**`, `harness/**`, `docs/specs/**` со статусом approved; запрет force-push.
8. **ADR в MADR 4** с обязательным `Confirmation`, ссылающимся на fitness function; front matter
   валидируется, `superseded by` проверяется.
9. **Retention**: до 1 октября 2026 выставить retention по нуждам (public максимум 90 дней) и
   считать checks эфемерными; всё, что нужно дольше, — в git/attestation.

Порядок внедрения по отдаче: (1)+(3)+(7) → (2) → (4) → (8) → (5)+(6).
