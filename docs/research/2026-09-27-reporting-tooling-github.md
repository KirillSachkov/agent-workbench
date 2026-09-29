# Инструменты для понятных владельцу результатов по каждой поставке (GitHub, дёшево)

Дата: 2026-09-27. Цель: для каждой поставки дать владельцу долговечную и понятную карточку: что
изменилось, что запускалось, pass/fail, что проверено, скриншоты или видео, где попробовать.
Источники первичные, URL приведены. Где факт не удалось подтвердить, стоит пометка «не подтверждено».

## 0. Проверки на месте (read-only)

| Факт | Значение | Как получено |
|---|---|---|
| `platform`: видимость / Pages | `public`, `has_pages=false` | `gh api repos/sachkov-inside/platform` |
| План организации | `free` | `gh api orgs/sachkov-inside --jq .plan.name` |
| Workflows | `add-to-inside-project`, `ci`, `deploy`, `inside-agent-sessions`, `inside-harness-health`, `nightly-fullstack`, `release`, `workshop-evaluator` + dependabot | `gh api .../actions/workflows` |
| Environments | только `Production` (`deploy.yml`, `workflow_dispatch`, immutable releases `vN`) | `gh api .../environments` |
| Retention артефактов/логов | `days=90`, `maximum_allowed_days=90` | `gh api .../actions/permissions/artifact-and-log-retention` |
| `docs/evidence` в рабочем дереве | 103 MB, 807 PNG, 82 каталога, 0 видео | `du -sh`, `find` |
| `docs/evidence` в истории git | 1087 blobs, ~109 MiB raw / ~101 MiB на диске; 86 коммитов | `git rev-list --objects --all -- docs/evidence` + `cat-file` |
| Весь репозиторий | `size-pack` 185 MiB, `.git` 219 MB | `git count-objects -vH` |
| Playwright сейчас | `reporter: [["list"],["html"]]`, `screenshot: "only-on-failure"`, `trace: "retain-on-failure"`; отчёт выгружается `upload-artifact@v7` с `retention-days: 7` | `apps/web/playwright.config.ts`, `.github/workflows/ci.yml` |
| PR body | «Отчёт о реализации» 5–12 тыс. символов (#779, #783, #784) | `gh pr list --state merged` |

Вывод: скриншоты доказательств уже занимают около половины упакованной истории `platform`
(~101 из 185 MiB). Историю переписывать нельзя (force-push заблокирован), но новые PNG можно
перестать класть в git. CI-отчёт Playwright живёт 7 дней, то есть для владельца его фактически нет.

### Что переживёт retention с 2026-10-01

GitHub с 1 октября 2026 распространяет Actions retention (по умолчанию 90 дней, для public
максимум 90) на checks (check suites/runs), workflow runs и commit statuses, включая созданные
сторонними приложениями; действие не ретроактивное.
https://github.blog/changelog/2026-08-27-actions-retention-will-cover-checks-workflow-runs-and-statuses/
Настройка: https://docs.github.com/en/organizations/managing-organization-settings/configuring-the-retention-period-for-github-actions-artifacts-and-logs-in-your-organization

| Носитель | Живёт после 90 дней? |
|---|---|
| PR body, PR/issue comments, review comments | да (это контент issue/PR, под retention Actions не попадает; в анонсе не упомянуты) |
| Job summary (`$GITHUB_STEP_SUMMARY`) | нет: часть workflow run → удаляется вместе с run (вывод из анонса; прямой формулировки нет — не подтверждено) |
| Check run output (summary/text/annotations/images), commit statuses | нет, ≤90 дней |
| Actions artifacts и logs | нет, ≤90 дней (у `platform` сейчас 7) |
| Deployments / deployment statuses (`environment_url`) | в анонсе не названы — не подтверждено |
| Release + assets, GitHub Pages, git, внешнее хранилище (R2/S3) | да |

Следствие: всё, что владелец должен найти через полгода, должно попасть в PR comment/body, в
релиз или во внешнее хранилище. Checks и summaries годятся только как оперативный вид.

---

## 1. Поверхности GitHub для «карточки результата»

### 1.1 Job summary (`$GITHUB_STEP_SUMMARY`)

- Что видит владелец: Markdown-страницу на странице run (Actions → run → Summary). Из PR —
  через «Details» у check.
- Лимиты: 1 MiB на step, отображается максимум 20 summaries из steps на job; GFM; summaries
  шагов склеиваются в один job summary, jobs упорядочены по времени завершения.
  https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands
- Поддерживает HTML, таблицы, Mermaid.
  https://github.blog/news-insights/product-news/supercharging-github-actions-with-job-summaries/
- Картинки: только по внешнему https-URL (проксируются через Camo); `data:`-URI не рендерятся,
  внешние ссылки иногда переписываются. Официальной спецификации по картинкам нет — не подтверждено;
  обсуждения: https://github.com/orgs/community/discussions/35932 ,
  https://github.com/orgs/community/discussions/60247 . Про Camo:
  https://docs.github.com/en/enterprise-cloud@latest/authentication/keeping-your-account-and-data-secure/about-anonymized-urls
- Долговечность: ≤90 дней (см. выше).
- Сниппет:

```yaml
- name: Result summary
  if: ${{ !cancelled() }}
  run: |
    {
      echo "## Результат проверки"
      echo "| Проверка | Итог |"
      echo "|---|---|"
      echo "| unit (Vitest) | ${{ steps.unit.outcome }} |"
      echo "| e2e (Playwright) | ${{ steps.e2e.outcome }} |"
      echo "[HTML-отчёт](https://sachkov-inside.github.io/platform-evidence/pr-${{ github.event.pull_request.number }}/)"
    } >> "$GITHUB_STEP_SUMMARY"
```

- Бесплатно готовое: Vitest reporter `github-actions` сам пишет job summary (статистика, flaky) и
  annotations; опции `jobSummary.enabled/outputPath/title/fileLinks`.
  https://vitest.dev/guide/reporters
- Подводный камень: владелец не видит summary в самом PR, нужен клик; через 90 дней пропадёт.

### 1.2 Check run output (Checks API)

- Поля `output`: `title` (обяз.), `summary` (обяз., Markdown), `text` (Markdown), `annotations`
  (до 50 за запрос; `message` до 64 KB, `title` до 255 символов), `images[]` (`alt`, `image_url`,
  `caption`). https://docs.github.com/en/rest/checks/runs
- Длина `summary`/`text` — 65 535 символов (указано в README dorny/test-reporter как предел
  отчёта; в REST-доке число не найдено — не подтверждено).
- Что видит владелец: отдельную вкладку check в PR с Markdown и картинками, annotations прямо в
  diff «Files changed».
- Долговечность: ≤90 дней. Подходит для pass/fail и аннотаций, не для архива.
- Нужен `permissions: checks: write`; у PR из форков токен read-only (у нас агенты работают в
  ветках основного repo, не критично).

### 1.3 Sticky PR comment — главный кандидат

- `marocchino/sticky-pull-request-comment@v3`: один комментарий по `header`, обновляется при
  каждом push; входы `message`/`path`, `recreate`, `append`, `hide_and_recreate`, `only_update`,
  `delete`; `permissions: pull-requests: write`.
  https://github.com/marocchino/sticky-pull-request-comment
- Что видит владелец: на вкладке Conversation одну карточку «что изменилось / что прогнали /
  итог / что проверено / где попробовать / картинки», которая остаётся после merge и после
  retention.
- Сниппет:

```yaml
permissions:
  contents: read
  pull-requests: write
jobs:
  result-card:
    needs: [quality, e2e]
    if: ${{ !cancelled() && github.event_name == 'pull_request' }}
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/download-artifact@v8   # результаты quality/e2e (JUnit, JSON)
      - run: node scripts/result-card.mjs > result-card.md   # собирает Markdown из JUnit/JSON/verification
      - uses: marocchino/sticky-pull-request-comment@v3
        with:
          header: result-card
          path: result-card.md
```

- Лимит комментария: 65 536 символов (в доках REST не нашёл — не подтверждено).
- Картинки в комментарии: только ссылки на внешние URL (см. §5). Официального API загрузки
  изображений в комментарий нет.
- Pitfall: в merge queue (`merge_group`) PR-контекста нет — карточку публиковать в
  `pull_request`-workflow, а не в прогоне очереди.

### 1.4 GitHub Deployments / Environments

- `jobs.<id>.environment: { name, url }`; URL можно брать из outputs шага. Отображается на
  странице deployments и в PR, связанном с развёртыванием (кнопка «View deployment»).
  https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
- Free: environments доступны только для public repos (для `platform` и `inside-telegram` — да;
  для private `ai-engineering` и др. — нет).
  https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments
- Сниппет для preview:

```yaml
deploy-preview:
  environment:
    name: pr-${{ github.event.pull_request.number }}
    url: ${{ steps.up.outputs.url }}
  steps:
    - id: up
      run: echo "url=https://pr-${{ github.event.pull_request.number }}.preview.example.ru" >> "$GITHUB_OUTPUT"
```

- Альтернатива без workflow-уровня: REST `POST /repos/{o}/{r}/deployments` +
  `POST .../deployments/{id}/statuses` с `environment_url`, `log_url`, `transient_environment`.
  https://docs.github.com/en/rest/deployments/deployments
- Pitfall: каждое `pr-N` environment остаётся в списке environments — удалять при закрытии PR
  (`DELETE /repos/{o}/{r}/environments/{name}`).

---

## 2. Отчёты тестов

### 2.1 Playwright HTML report + trace viewer

- Режимы `video`/`trace`: `off`, `on`, `retain-on-failure`, `retain-on-first-failure`,
  `retain-on-failure-and-retries`, `on-first-retry`, `on-all-retries`; `screenshot`: `off`, `on`,
  `only-on-failure`. https://playwright.dev/docs/test-use-options
- Видео по умолчанию масштабируется в 800×800; `video.size`; для ручного контекста
  `browser.newContext({ recordVideo: { dir } })`, файл появляется после `context.close()`.
  Есть подписи на видео: `video.show.actions` (подсветка элемента + подпись действия) и
  `video.show.test` (название теста/шага) — удобно владельцу. https://playwright.dev/docs/videos

```ts
// apps/web/playwright.config.ts — отдельный проект «evidence» для приёмочных сценариев
projects: [
  { name: "chromium", use: { screenshot: "only-on-failure", trace: "retain-on-failure" } },
  {
    name: "evidence",
    grep: /@evidence/,
    use: {
      screenshot: "on",
      trace: "on",
      video: {
        mode: "on",
        size: { width: 1280, height: 720 },
        show: { actions: { duration: 500, position: "top-right" }, test: { level: "step", position: "top-left" } },
      },
    },
  },
],
reporter: [["list"], ["html", { open: "never", title: "platform e2e" }], ["junit", { outputFile: "results/e2e.xml" }], ["github"]],
```

- HTML report: `outputFolder`, `open`, `title`, `attachmentsBaseURL` (вложения можно держать во
  внешнем хранилище); `blob` + `merge-reports` для шардов; `junit`; `github` → annotations.
  https://playwright.dev/docs/test-reporters
- Trace viewer встроен в HTML report; `trace.playwright.dev` — статическая версия, трасса
  грузится в браузере и никуда не отправляется; можно открыть по `?trace=<https-URL>`, но нужен
  CORS у хранилища. https://playwright.dev/docs/trace-viewer
- Официальная рекомендация Playwright — `upload-artifact` (пример с `retention-days: 30`) или
  статический хостинг (Azure). https://playwright.dev/docs/ci-intro
- Одиночный файл можно выгрузить без zip (`upload-artifact@v7`, `archive: false`), тогда
  картинка/HTML без внешних CSS/JS открывается прямо в браузере; ссылка требует логина и живёт до
  истечения retention. https://github.blog/changelog/2026-02-26-github-actions-now-supports-uploading-and-downloading-non-zipped-artifacts/ ,
  https://github.com/actions/upload-artifact
- Pitfall: HTML report — SPA с десятками МБ вложений; для Pages нужен per-PR каталог и чистка.

### 2.2 Публикация отчётов на GitHub Pages

- Лимиты Pages: сайт ≤1 GB, source repo рекомендовано ≤1 GB, деплой ≤10 минут, мягкий лимит
  100 GB/мес трафика и 10 builds/час.
  https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits
- Free для организаций: Pages только в public repos (для private — нет), по
  https://docs.github.com/en/get-started/learning-about-github/githubs-plans . Сайт Pages всегда
  публичен (приватный доступ — только Enterprise Cloud; на странице не найдено — не подтверждено).
- Варианты:
  - `actions/upload-pages-artifact` + `actions/deploy-pages@v4` (`pages: write`, `id-token: write`,
    `environment: github-pages`). Каждый деплой публикует один артефакт целиком, то есть истории
    per-PR нет без собственной сборки всего сайта (вывод; в README не сказано — не подтверждено).
    https://github.com/actions/deploy-pages
  - Ветка `gh-pages` с каталогами на PR: `rossjrw/pr-preview-action@v1` кладёт в
    `pr-preview/pr-N/`, удаляет при закрытии PR, сам пишет sticky comment (с QR); требует
    источник Pages = branch, не «GitHub Actions»; форки не поддерживаются.
    https://github.com/rossjrw/pr-preview-action
- Рекомендация: отдельный public repo `sachkov-inside/platform-evidence` с Pages из ветки,
  каталоги `pr-<N>/` (HTML report, screenshots, video), cron-чистка старше N дней, а долгоживущие
  ключевые кадры — в release assets/R2. Так git `platform` не растёт, а 1 GB лимит изолирован.

### 2.3 Allure Report 3

- Стабильный, переписан на TypeScript, плагины `awesome`, `dashboard`, `classic`, quality gates,
  история в `historyPath` (JSONL), `historyLimit`, `variables`.
  https://allurereport.org/docs/v3/ , https://allurereport.org/docs/v3/configure/

```js
// allurerc.mjs
import { defineConfig } from "allure";
export default defineConfig({
  name: "platform",
  output: "./allure-report",
  historyPath: "./history.jsonl",
  qualityGate: { rules: [{ maxFailures: 0, fastFail: true }] },
  plugins: { awesome: { options: { reportLanguage: "ru" } } },
});
```

- Pages: официальный гайд для Allure 3 прямо предупреждает, что пример не переносит историю
  между прогонами; для истории предлагают свой storage service (Docker/Cloudflare Workers).
  Allure 2 имеет `simple-elf/allure-report-action` с `allure_history`.
  https://allurereport.org/docs/guides/github-pages/
- Плюсы: тренды, flaky, категории, вложения (скриншоты, видео, trace). Минусы: второй формат
  отчёта рядом с Playwright HTML, адаптеры `allure-playwright`/`allure-vitest`, своё хранилище
  истории. Для одного владельца избыточно.

### 2.4 JUnit → check run / annotations

- `dorny/test-reporter@v3`: создаёт check run из JUnit и др., `use-actions-summary` (по умолчанию
  `true`), `max-annotations` (по умолчанию 10, максимум 50), отчёт ≤65 535 байт; нужен
  `checks: write`; для форков — схема через `workflow_run`. В списке форматов есть `jest-junit`,
  `java-junit`; Vitest/Playwright JUnit явно не перечислены — совместимость не подтверждена.
  https://github.com/dorny/test-reporter
- Для Vitest проще встроенный `github-actions` reporter (§1.1), для Playwright — встроенный
  `github` reporter. Всё это живёт ≤90 дней.

### 2.5 Hosted dashboards (кратко)

- Currents: от $49/мес, 10K результатов, retention до 1 года; постоянного free нет (только trial).
  https://currents.dev/pricing
- Argos (см. §3) тоже принимает Playwright traces и failure screenshots.

---

## 3. Visual review с «approve в один клик»

| Инструмент | Что видит владелец | Бесплатно | Коммерческий public repo | Pitfalls |
|---|---|---|---|---|
| Chromatic (Storybook) | PR checks «UI Tests» и «UI Review»; в веб-UI diff, Accept/Deny, Approve | Free: 5 000 snapshots/мес, только Chrome, visual + interaction tests; UI Review на Free — страница pricing противоречива (не подтверждено); TurboSnap на Free — «Not included» (не подтверждено) | OSS-программа «по заявке»; Starter $179/мес, 35 000 snapshots | Нужен `fetch-depth: 0`; docs советуют `on: push`; merge queue (`merge_group`) — поддержка не проверена |
| Argos | Diff в Argos, статус в PR, Approve/Reject | Hobby: 5 000 screenshots/мес «для personal projects» | OSS-спонсорство только «not for commercial use» → `platform` не подходит; Pro $100/мес, 35 000 | Hobby для org — не подтверждено |
| Percy (BrowserStack) | Diff в Percy, статус в PR | 5 000 screenshots/мес, unlimited users | считается browser×width | точные цены paid не опубликованы |
| Lost Pixel OSS | CI падает, baselines обновляются PR-ом | бесплатно (OSS) | — | baselines в git = снова PNG в repo |
| Playwright `toHaveScreenshot` | failing test + diff в HTML report; approve = коммит обновлённых baselines (`--update-snapshots`) | бесплатно | — | PNG в git, зависимость от ОС/шрифтов, нет кнопки approve |

Источники: https://www.chromatic.com/pricing , https://www.chromatic.com/docs/review/ ,
https://www.chromatic.com/docs/turbosnap/ (copied snapshot = 0.2 billed),
https://www.chromatic.com/docs/github-actions/ , https://argos-ci.com/pricing ,
https://argos-ci.com/docs/learn/billing-and-subscription/open-source.md ,
https://argos-ci.com/docs/reference/playwright.md ,
https://www.browserstack.com/docs/percy/overview/plans-and-billing , https://docs.lost-pixel.com/user-docs

Chromatic сниппет:

```yaml
- uses: actions/checkout@v7
  with: { fetch-depth: 0 }
- uses: chromaui/action@<SHA>   # закрепить SHA текущего мажора; docs допускают @latest/@vX
  with:
    projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
    exitZeroOnChanges: true      # не валить CI, ждать решения владельца в UI
    autoAcceptChanges: main
    onlyChanged: true            # TurboSnap, если доступен на плане
# outputs: buildUrl, storybookUrl, changeCount → в sticky comment
```

Argos сниппет (Playwright):

```ts
reporter: [["@argos-ci/playwright/reporter", createArgosReporterOptions({ uploadToArgos: !!process.env.CI })]],
// в тесте: await argosScreenshot(page, "checkout-success");
```

Вывод: «click-to-approve» дёшево даёт только Chromatic Free (если Storybook покрывает UI). Для
коммерческого проекта OSS-программы Argos не применимы. `storybookUrl` Chromatic — ещё и
бесплатный хостинг Storybook ветки для владельца.

---

## 4. Preview environments

| Вариант | Для чего | Цена | URL на PR | Pitfalls |
|---|---|---|---|---|
| Vercel Preview | Next.js | Hobby — только non-commercial; коммерция требует Pro ($20/dev seat) | комментарий + deployment | web без backend почти бесполезен |
| Netlify Deploy Previews | Next.js | Free-план: наличие previews на странице не указано — не подтверждено | status check + comment + deployment | то же |
| Render Preview Environments | NestJS + Postgres + web | нужен Pro workspace; БД в preview пустая (seed через `initialDeployHook`); `expireAfterDays` | «View deployment» | цена Pro — не подтверждена |
| Railway PR environments | весь стек | Hobby $5/мес (включено $5 usage), Pro $20; PR envs на Hobby — не подтверждено; есть Bot PR environments (Claude Code и др.) | GitHub deployment — не подтверждено | usage-биллинг |
| Fly review apps | весь стек | usage | `environment.url` в PR | Postgres/cleanup — надо писать самим |
| Coolify (self-hosted) | docker compose на своём VPS | стоимость VPS | GitHub App комментирует PR URL `{{pr_id}}.{{domain}}` | PR-код исполняется на сервере |
| Свой docker compose на VPS | как сейчас стенд | стоимость VPS | `environment: {name, url}` | чистка, порты, секреты |

Источники: https://vercel.com/docs/limits/fair-use-guidelines , https://vercel.com/pricing ,
https://docs.netlify.com/deploy/deploy-types/deploy-previews/ ,
https://render.com/docs/preview-environments , https://docs.railway.com/guides/environments ,
https://railway.com/pricing , https://docs.fly.io/blueprints/review-apps-guide/ ,
https://coolify.io/docs/applications/ci-cd/github/preview-deploy

Вывод для `platform`: backend + PostgreSQL + object storage + платежи Т-Банка делают SaaS-previews
дорогими и неполными. Реалистично: per-PR compose-стек на имеющемся VPS (или Coolify поверх него)
по метке `preview`, с GitHub environment `pr-N` и URL в sticky comment; уничтожение при закрытии.
Бесплатный минимум — ссылка `storybookUrl` от Chromatic или Storybook на Pages.

---

## 5. Хранение медиа вне git

| Носитель | Лимиты/цена | Публичность | Долговечность | Замечание |
|---|---|---|---|---|
| Git (как сейчас) | файл: рекомендация 1 MB, жёсткий лимит 100 MB; repo on-disk рекомендовано ≤10 GB | как repo | вечно, нельзя удалить без rewrite | уже ~101 MiB истории `platform` |
| Git LFS (Free) | 10 GiB storage + 10 GiB bandwidth/мес на аккаунт, public тоже считается; при превышении без оплаты — только pointer-файлы / LFS выключается до конца месяца | как repo | вечно | CI checkout тратит bandwidth |
| Release assets | ≤2 GiB на файл, ≤1000 assets на release, лимитов на общий объём и трафик нет | как repo | вечно | `platform` использует immutable releases — для evidence нужен отдельный repo или pre-release с тегом `evidence-pr-N` |
| Отдельная ветка/repo `evidence` + Pages | Pages ≤1 GB сайт, 100 GB/мес | public | пока не удалено | ротация обязательна |
| Cloudflare R2 | free: 10 GB-мес, 1M Class A, 10M Class B, egress бесплатно; далее $0.015/GB-мес | public bucket / r2.dev или custom domain; или presigned | пока платите | нужен секрет в Actions; CORS для trace viewer |
| Actions artifacts | ≤90 дней; public — бесплатно, private Free — 500 MB | только с логином | ≤90 дней | не архив |

Источники: https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits ,
https://docs.github.com/en/billing/concepts/product-billing/git-lfs ,
https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases ,
https://developers.cloudflare.com/r2/pricing/ ,
https://docs.github.com/en/billing/concepts/product-billing/github-actions

Загрузка картинок в PR comment программно:
- Официального API нет; запросы в community без ответа GitHub, считается сознательным
  ограничением. https://github.com/orgs/community/discussions/28219 ,
  https://github.com/orgs/community/discussions/29993
- Неофициальный endpoint `https://uploads.github.com/user-attachments/assets` с Bearer token
  (описан в августе 2026, автор сам не уверен в стабильности), расширение `gh-image`.
  https://island94.org/2026/08/programmatically-upload-attachments-to-github-issues-pull-requests-comments
  Для harness не использовать: недокументировано, может исчезнуть.
- Рабочий путь: загрузить в R2/Pages/release asset и вставить `![alt](https://...)` в sticky
  comment; GitHub проксирует через Camo.

Сниппет (release asset как долговечное хранилище в отдельном repo):

```bash
gh release create "evidence-pr-$PR" --repo sachkov-inside/platform-evidence \
  --title "Evidence PR #$PR" --notes "Source: sachkov-inside/platform#$PR" --prerelease
gh release upload "evidence-pr-$PR" evidence/*.png evidence/*.webm --repo sachkov-inside/platform-evidence --clobber
# URL: https://github.com/sachkov-inside/platform-evidence/releases/download/evidence-pr-$PR/<file>
```

Проверить до внедрения: отображаются ли `releases/download/...` PNG инлайн в комментарии (редирект
на objects.githubusercontent.com; через Camo обычно работает — не подтверждено), и воспроизводится ли
`.webm` (GitHub инлайн-плеер работает для загруженных через UI видео; для внешних ссылок — только
ссылка, не подтверждено). Надёжно для видео: ссылка «▶ видео» + GIF-превью.

---

## 6. Дашборды по многим задачам

- GitHub Projects: поля (Status, Iteration, текстовое «Evidence URL», single-select «Приёмка»),
  views table/board/roadmap; Iteration field — любая длина, перерывы, фильтры `@current`,
  `@previous`, `@next`, группировка по итерации.
  https://docs.github.com/en/issues/planning-and-tracking-with-projects/understanding-fields/about-iteration-fields
- Insights: current charts и historical (по умолчанию Burn up; Open/Completed/Closed PR/Not
  planned). Ограничения исторических графиков на Free не найдены — не подтверждено.
  https://docs.github.com/en/issues/planning-and-tracking-with-projects/viewing-insights-from-your-project/about-insights-for-projects
- Weekly digest: scheduled workflow (`on: schedule: cron`) → `gh pr list --search "merged:>=$(date -d '7 days ago' +%F)" --json number,title,url`
  + ссылки на sticky comments → issue/Discussion «Неделя N» или job summary. Дёшево и долговечно,
  если писать в issue, а не в summary.
- Release notes: `.github/release.yml` с `changelog.exclude.labels/authors` и
  `categories[].labels`, `"*"` — catch-all. https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes
  У `platform` уже есть immutable releases `v7..v9` от `release.yml` workflow — достаточно
  `gh release create --generate-notes` и меток на PR.

```yaml
# .github/release.yml
changelog:
  exclude:
    labels: [skip-release-notes]
    authors: [dependabot]
  categories:
    - title: Для участников
      labels: [user-facing]
    - title: Оплата и доступ
      labels: [payments]
    - title: Внутреннее
      labels: ["*"]
```

- Keep a Changelog (Added/Changed/Deprecated/Removed/Fixed/Security, раздел Unreleased):
  https://keepachangelog.com/en/1.1.0/ . Для agent-driven потока лучше генерировать из PR, а не
  вести руками; ручной CHANGELOG дублирует release notes.

---

## 7. Запись ручной проверки агента и CLI-демо

- Браузер: тот же Playwright `recordVideo` (или `video.show` с подписями шагов) в сценарии
  приёмки; итог — `.webm` + `trace.zip`. Ссылка «открыть trace» =
  `https://trace.playwright.dev/?trace=<публичный URL trace.zip>` (нужен CORS у хранилища).
  https://playwright.dev/docs/videos , https://playwright.dev/docs/trace-viewer
- GIF-превью для комментария: `ffmpeg -i video.webm -vf "fps=8,scale=800:-1" preview.gif`
  (инлайн в Markdown как картинка; видео — ссылкой).
- CLI: asciinema запись `.cast`; asciinema.org — бесплатный публичный хостинг с visibility
  public/unlisted/private, есть self-host. https://docs.asciinema.org/manual/server/
  `agg demo.cast demo.gif` — GIF для встраивания в PR. https://docs.asciinema.org/manual/agg/
  Политика хранения неприкреплённых (unclaimed) записей на asciinema.org — не подтверждено.

---

## 8. Рекомендуемый стек (по отношению ценность/стоимость)

1. **Sticky «карточка результата» в PR** (marocchino v3) из машинных входов: JUnit (Vitest,
   Playwright), `docs/verification/*.json`, список `@evidence`-сценариев, ссылки на медиа и stand.
   Цена 0, переживает retention, владелец видит всё в одном месте. PR body сократить до «что и
   зачем», а pass/fail и проверки перенести в карточку, которую пишет CI, а не агент по памяти.
2. **Медиа вне git**: public repo `platform-evidence` (Pages из ветки, `pr-N/` с HTML report,
   скриншотами, видео; ротация ~90–180 дней) + долговечные ключевые кадры в release assets того же
   repo (`evidence-pr-N`). Приватные repos (Pages на Free недоступны) → Cloudflare R2 (10 GB free).
   Перестать добавлять PNG в `docs/evidence` (историю не трогать).
3. **Playwright проект `evidence`**: `screenshot: on`, `video: on` с подписями шагов, `trace: on`
   только для приёмочных сценариев; HTML report публиковать в `pr-N/`; `github` reporter и
   Vitest `github-actions` reporter для оперативного вида (summary/annotations, ≤90 дней).
4. **Release notes** через `.github/release.yml` + метки; weekly digest в issue по cron.
   Projects: поле «Evidence URL» и view «Ждёт приёмки».
5. **Visual review**: Chromatic Free (5 000 snapshots, Chrome) — только если Storybook уже
   покрывает ключевые экраны; проверить совместимость с merge queue и реальный расход snapshots.
   Argos/Percy — не раньше, чем упрёмся в лимит; OSS-программы для коммерческого `platform` не
   применимы.
6. **Preview на PR**: per-PR docker compose на имеющемся VPS (или Coolify) с `environment.url` —
   высокая ценность для приёмки, но заметная стоимость работ и безопасность (код PR на сервере).
   Vercel Hobby запрещён для коммерции; Render previews требуют Pro.
7. **Allure 3 / Currents** — отложить: ценность (тренды, flaky) не окупает второй формат отчёта и
   хранилище истории при одном владельце.
