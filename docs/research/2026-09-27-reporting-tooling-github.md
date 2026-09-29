# Tools for owner-readable results for every delivery (GitHub, cheap)

Date: 2026-09-27. Goal: for every delivery, give the owner a durable and understandable card: what
changed, what was run, pass/fail, what was verified, screenshots or video, where to try it.
Sources are primary, URLs are given. Where a fact could not be confirmed, it is marked "unconfirmed".

## 0. On-site checks (read-only)

| Fact | Value | How obtained |
|---|---|---|
| `platform`: visibility / Pages | `public`, `has_pages=false` | `gh api repos/sachkov-inside/platform` |
| Organization plan | `free` | `gh api orgs/sachkov-inside --jq .plan.name` |
| Workflows | `add-to-inside-project`, `ci`, `deploy`, `inside-agent-sessions`, `inside-harness-health`, `nightly-fullstack`, `release`, `workshop-evaluator` + dependabot | `gh api .../actions/workflows` |
| Environments | only `Production` (`deploy.yml`, `workflow_dispatch`, immutable releases `vN`) | `gh api .../environments` |
| Artifact/log retention | `days=90`, `maximum_allowed_days=90` | `gh api .../actions/permissions/artifact-and-log-retention` |
| `docs/evidence` in the working tree | 103 MB, 807 PNG, 82 directories, 0 videos | `du -sh`, `find` |
| `docs/evidence` in git history | 1087 blobs, ~109 MiB raw / ~101 MiB on disk; 86 commits | `git rev-list --objects --all -- docs/evidence` + `cat-file` |
| The whole repository | `size-pack` 185 MiB, `.git` 219 MB | `git count-objects -vH` |
| Playwright now | `reporter: [["list"],["html"]]`, `screenshot: "only-on-failure"`, `trace: "retain-on-failure"`; the report is uploaded with `upload-artifact@v7` with `retention-days: 7` | `apps/web/playwright.config.ts`, `.github/workflows/ci.yml` |
| PR body | "Implementation report" of 5–12 thousand characters (#779, #783, #784) | `gh pr list --state merged` |

Conclusion: evidence screenshots already take up about half of the packed history of `platform`
(~101 of 185 MiB). History cannot be rewritten (force-push is blocked), but new PNGs can stop being
put into git. The Playwright CI report lives for 7 days, so for the owner it effectively does not exist.

### What survives retention from 2026-10-01

From October 1, 2026, GitHub extends Actions retention (90 days by default, maximum 90 for public)
to checks (check suites/runs), workflow runs and commit statuses, including those created by
third-party apps; the change is not retroactive.
https://github.blog/changelog/2026-08-27-actions-retention-will-cover-checks-workflow-runs-and-statuses/
Setting: https://docs.github.com/en/organizations/managing-organization-settings/configuring-the-retention-period-for-github-actions-artifacts-and-logs-in-your-organization

| Medium | Lives past 90 days? |
|---|---|
| PR body, PR/issue comments, review comments | yes (this is issue/PR content, not covered by Actions retention; not mentioned in the announcement) |
| Job summary (`$GITHUB_STEP_SUMMARY`) | no: part of a workflow run → deleted together with the run (an inference from the announcement; there is no direct wording — unconfirmed) |
| Check run output (summary/text/annotations/images), commit statuses | no, ≤90 days |
| Actions artifacts and logs | no, ≤90 days (7 for `platform` now) |
| Deployments / deployment statuses (`environment_url`) | not named in the announcement — unconfirmed |
| Release + assets, GitHub Pages, git, external storage (R2/S3) | yes |

Consequence: everything the owner must be able to find in six months must go into a PR comment/body,
a release or external storage. Checks and summaries are suitable only as an operational view.

---

## 1. GitHub surfaces for a "result card"

### 1.1 Job summary (`$GITHUB_STEP_SUMMARY`)

- What the owner sees: a Markdown page on the run page (Actions → run → Summary). From a PR —
  via "Details" on a check.
- Limits: 1 MiB per step, at most 20 step summaries are displayed per job; GFM; step summaries
  are concatenated into one job summary, jobs are ordered by completion time.
  https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands
- Supports HTML, tables, Mermaid.
  https://github.blog/news-insights/product-news/supercharging-github-actions-with-job-summaries/
- Images: only by an external https URL (proxied through Camo); `data:` URIs are not rendered,
  external links are sometimes rewritten. There is no official specification on images — unconfirmed;
  discussions: https://github.com/orgs/community/discussions/35932 ,
  https://github.com/orgs/community/discussions/60247 . On Camo:
  https://docs.github.com/en/enterprise-cloud@latest/authentication/keeping-your-account-and-data-secure/about-anonymized-urls
- Durability: ≤90 days (see above).
- Snippet:

```yaml
- name: Result summary
  if: ${{ !cancelled() }}
  run: |
    {
      echo "## Check result"
      echo "| Check | Result |"
      echo "|---|---|"
      echo "| unit (Vitest) | ${{ steps.unit.outcome }} |"
      echo "| e2e (Playwright) | ${{ steps.e2e.outcome }} |"
      echo "[HTML report](https://sachkov-inside.github.io/platform-evidence/pr-${{ github.event.pull_request.number }}/)"
    } >> "$GITHUB_STEP_SUMMARY"
```

- Free out of the box: the Vitest reporter `github-actions` itself writes a job summary (statistics,
  flaky) and annotations; options `jobSummary.enabled/outputPath/title/fileLinks`.
  https://vitest.dev/guide/reporters
- Pitfall: the owner does not see the summary in the PR itself, a click is needed; it will disappear after 90 days.

### 1.2 Check run output (Checks API)

- `output` fields: `title` (required), `summary` (required, Markdown), `text` (Markdown), `annotations`
  (up to 50 per request; `message` up to 64 KB, `title` up to 255 characters), `images[]` (`alt`,
  `image_url`, `caption`). https://docs.github.com/en/rest/checks/runs
- The length of `summary`/`text` is 65,535 characters (stated in the dorny/test-reporter README as the
  report limit; the number was not found in the REST docs — unconfirmed).
- What the owner sees: a separate check tab in the PR with Markdown and images, annotations right in
  the "Files changed" diff.
- Durability: ≤90 days. Suitable for pass/fail and annotations, not for an archive.
- Needs `permissions: checks: write`; for PRs from forks the token is read-only (our agents work in
  branches of the main repo, not critical).

### 1.3 Sticky PR comment — the main candidate

- `marocchino/sticky-pull-request-comment@v3`: one comment per `header`, updated on every push;
  inputs `message`/`path`, `recreate`, `append`, `hide_and_recreate`, `only_update`, `delete`;
  `permissions: pull-requests: write`.
  https://github.com/marocchino/sticky-pull-request-comment
- What the owner sees: on the Conversation tab a single card "what changed / what was run /
  result / what was verified / where to try it / images", which remains after merge and after
  retention.
- Snippet:

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
      - uses: actions/download-artifact@v8   # quality/e2e results (JUnit, JSON)
      - run: node scripts/result-card.mjs > result-card.md   # builds Markdown from JUnit/JSON/verification
      - uses: marocchino/sticky-pull-request-comment@v3
        with:
          header: result-card
          path: result-card.md
```

- Comment limit: 65,536 characters (not found in the REST docs — unconfirmed).
- Images in a comment: only links to external URLs (see §5). There is no official API for uploading
  images into a comment.
- Pitfall: in a merge queue (`merge_group`) there is no PR context — publish the card in a
  `pull_request` workflow, not in a queue run.

### 1.4 GitHub Deployments / Environments

- `jobs.<id>.environment: { name, url }`; the URL can be taken from step outputs. Shown on the
  deployments page and in the PR linked to the deployment ("View deployment" button).
  https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
- Free: environments are available only for public repos (yes for `platform` and `inside-telegram`;
  no for private `ai-engineering` and others).
  https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments
- Snippet for a preview:

```yaml
deploy-preview:
  environment:
    name: pr-${{ github.event.pull_request.number }}
    url: ${{ steps.up.outputs.url }}
  steps:
    - id: up
      run: echo "url=https://pr-${{ github.event.pull_request.number }}.preview.example.ru" >> "$GITHUB_OUTPUT"
```

- Alternative without the workflow level: REST `POST /repos/{o}/{r}/deployments` +
  `POST .../deployments/{id}/statuses` with `environment_url`, `log_url`, `transient_environment`.
  https://docs.github.com/en/rest/deployments/deployments
- Pitfall: every `pr-N` environment stays in the list of environments — delete on PR close
  (`DELETE /repos/{o}/{r}/environments/{name}`).

---

## 2. Test reports

### 2.1 Playwright HTML report + trace viewer

- `video`/`trace` modes: `off`, `on`, `retain-on-failure`, `retain-on-first-failure`,
  `retain-on-failure-and-retries`, `on-first-retry`, `on-all-retries`; `screenshot`: `off`, `on`,
  `only-on-failure`. https://playwright.dev/docs/test-use-options
- Video is scaled to 800×800 by default; `video.size`; for a manual context
  `browser.newContext({ recordVideo: { dir } })`, the file appears after `context.close()`.
  There are captions on video: `video.show.actions` (element highlight + action caption) and
  `video.show.test` (test/step name) — convenient for the owner. https://playwright.dev/docs/videos

```ts
// apps/web/playwright.config.ts — a separate "evidence" project for acceptance scenarios
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

- HTML report: `outputFolder`, `open`, `title`, `attachmentsBaseURL` (attachments can be kept in
  external storage); `blob` + `merge-reports` for shards; `junit`; `github` → annotations.
  https://playwright.dev/docs/test-reporters
- The trace viewer is built into the HTML report; `trace.playwright.dev` is a static version, the trace
  is loaded in the browser and is not sent anywhere; it can be opened via `?trace=<https-URL>`, but
  the storage needs CORS. https://playwright.dev/docs/trace-viewer
- The official Playwright recommendation is `upload-artifact` (example with `retention-days: 30`) or
  static hosting (Azure). https://playwright.dev/docs/ci-intro
- A single file can be uploaded without zip (`upload-artifact@v7`, `archive: false`), then an
  image/HTML without external CSS/JS opens directly in the browser; the link requires a login and lives until
  retention expires. https://github.blog/changelog/2026-02-26-github-actions-now-supports-uploading-and-downloading-non-zipped-artifacts/ ,
  https://github.com/actions/upload-artifact
- Pitfall: the HTML report is an SPA with tens of MB of attachments; for Pages a per-PR directory and cleanup are needed.

### 2.2 Publishing reports to GitHub Pages

- Pages limits: site ≤1 GB, source repo recommended ≤1 GB, deploy ≤10 minutes, soft limit
  100 GB/month of traffic and 10 builds/hour.
  https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits
- Free for organizations: Pages only in public repos (not for private), per
  https://docs.github.com/en/get-started/learning-about-github/githubs-plans . A Pages site is always
  public (private access — only Enterprise Cloud; not found on that page — unconfirmed).
- Options:
  - `actions/upload-pages-artifact` + `actions/deploy-pages@v4` (`pages: write`, `id-token: write`,
    `environment: github-pages`). Each deploy publishes one artifact as a whole, so there is no
    per-PR history without building the whole site yourself (an inference; the README does not say —
    unconfirmed).
    https://github.com/actions/deploy-pages
  - The `gh-pages` branch with per-PR directories: `rossjrw/pr-preview-action@v1` puts into
    `pr-preview/pr-N/`, deletes on PR close, writes a sticky comment itself (with a QR code); requires
    the Pages source = branch, not "GitHub Actions"; forks are not supported.
    https://github.com/rossjrw/pr-preview-action
- Recommendation: a separate public repo `sachkov-inside/platform-evidence` with Pages from a branch,
  `pr-<N>/` directories (HTML report, screenshots, video), a cron cleanup of anything older than N days, and
  long-lived key frames in release assets/R2. This way the `platform` git does not grow, and the 1 GB limit is isolated.

### 2.3 Allure Report 3

- Stable, rewritten in TypeScript, plugins `awesome`, `dashboard`, `classic`, quality gates,
  history in `historyPath` (JSONL), `historyLimit`, `variables`.
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

- Pages: the official guide for Allure 3 explicitly warns that the example does not carry history
  between runs; for history they suggest their own storage service (Docker/Cloudflare Workers).
  Allure 2 has `simple-elf/allure-report-action` with `allure_history`.
  https://allurereport.org/docs/guides/github-pages/
- Pros: trends, flaky, categories, attachments (screenshots, video, trace). Cons: a second report
  format next to Playwright HTML, the adapters `allure-playwright`/`allure-vitest`, its own history
  storage. Excessive for a single owner.

### 2.4 JUnit → check run / annotations

- `dorny/test-reporter@v3`: creates a check run from JUnit and others, `use-actions-summary` (default
  `true`), `max-annotations` (default 10, maximum 50), report ≤65,535 bytes; needs
  `checks: write`; for forks — a scheme via `workflow_run`. The list of formats includes `jest-junit`,
  `java-junit`; Vitest/Playwright JUnit are not listed explicitly — compatibility unconfirmed.
  https://github.com/dorny/test-reporter
- For Vitest the built-in `github-actions` reporter is simpler (§1.1), for Playwright — the built-in
  `github` reporter. All of this lives ≤90 days.

### 2.5 Hosted dashboards (brief)

- Currents: from $49/month, 10K results, retention up to 1 year; no permanent free tier (trial only).
  https://currents.dev/pricing
- Argos (see §3) also accepts Playwright traces and failure screenshots.

---

## 3. Visual review with "one-click approve"

| Tool | What the owner sees | Free | Commercial public repo | Pitfalls |
|---|---|---|---|---|
| Chromatic (Storybook) | PR checks "UI Tests" and "UI Review"; in the web UI a diff, Accept/Deny, Approve | Free: 5,000 snapshots/month, Chrome only, visual + interaction tests; UI Review on Free — the pricing page is contradictory (unconfirmed); TurboSnap on Free — "Not included" (unconfirmed) | OSS program "by application"; Starter $179/month, 35,000 snapshots | Needs `fetch-depth: 0`; the docs advise `on: push`; merge queue (`merge_group`) — support not checked |
| Argos | A diff in Argos, a status in the PR, Approve/Reject | Hobby: 5,000 screenshots/month "for personal projects" | OSS sponsorship only "not for commercial use" → not suitable for `platform`; Pro $100/month, 35,000 | Hobby for an org — unconfirmed |
| Percy (BrowserStack) | A diff in Percy, a status in the PR | 5,000 screenshots/month, unlimited users | counted as browser×width | exact paid prices are not published |
| Lost Pixel OSS | CI fails, baselines are updated by a PR | free (OSS) | — | baselines in git = PNGs in the repo again |
| Playwright `toHaveScreenshot` | a failing test + a diff in the HTML report; approve = a commit of updated baselines (`--update-snapshots`) | free | — | PNG in git, dependence on OS/fonts, no approve button |

Sources: https://www.chromatic.com/pricing , https://www.chromatic.com/docs/review/ ,
https://www.chromatic.com/docs/turbosnap/ (copied snapshot = 0.2 billed),
https://www.chromatic.com/docs/github-actions/ , https://argos-ci.com/pricing ,
https://argos-ci.com/docs/learn/billing-and-subscription/open-source.md ,
https://argos-ci.com/docs/reference/playwright.md ,
https://www.browserstack.com/docs/percy/overview/plans-and-billing , https://docs.lost-pixel.com/user-docs

Chromatic snippet:

```yaml
- uses: actions/checkout@v7
  with: { fetch-depth: 0 }
- uses: chromaui/action@<SHA>   # pin the SHA of the current major; the docs allow @latest/@vX
  with:
    projectToken: ${{ secrets.CHROMATIC_PROJECT_TOKEN }}
    exitZeroOnChanges: true      # do not fail CI, wait for the owner's decision in the UI
    autoAcceptChanges: main
    onlyChanged: true            # TurboSnap, if available on the plan
# outputs: buildUrl, storybookUrl, changeCount → into the sticky comment
```

Argos snippet (Playwright):

```ts
reporter: [["@argos-ci/playwright/reporter", createArgosReporterOptions({ uploadToArgos: !!process.env.CI })]],
// in a test: await argosScreenshot(page, "checkout-success");
```

Conclusion: "click-to-approve" is cheaply provided only by Chromatic Free (if Storybook covers the UI). For
a commercial project the Argos OSS programs do not apply. Chromatic's `storybookUrl` is also a
free hosting of the branch's Storybook for the owner.

---

## 4. Preview environments

| Option | For what | Price | URL on the PR | Pitfalls |
|---|---|---|---|---|
| Vercel Preview | Next.js | Hobby — non-commercial only; commercial use requires Pro ($20/dev seat) | a comment + deployment | a web app without a backend is almost useless |
| Netlify Deploy Previews | Next.js | Free plan: the availability of previews is not stated on the page — unconfirmed | status check + comment + deployment | same |
| Render Preview Environments | NestJS + Postgres + web | a Pro workspace is required; the DB in a preview is empty (seed via `initialDeployHook`); `expireAfterDays` | "View deployment" | Pro price — unconfirmed |
| Railway PR environments | the whole stack | Hobby $5/month ($5 usage included), Pro $20; PR envs on Hobby — unconfirmed; there are Bot PR environments (Claude Code and others) | GitHub deployment — unconfirmed | usage billing |
| Fly review apps | the whole stack | usage | `environment.url` in the PR | Postgres/cleanup — you have to write them yourself |
| Coolify (self-hosted) | docker compose on your own VPS | the cost of the VPS | the GitHub App comments the PR URL `{{pr_id}}.{{domain}}` | PR code is executed on the server |
| Your own docker compose on a VPS | like the current stand | the cost of the VPS | `environment: {name, url}` | cleanup, ports, secrets |

Sources: https://vercel.com/docs/limits/fair-use-guidelines , https://vercel.com/pricing ,
https://docs.netlify.com/deploy/deploy-types/deploy-previews/ ,
https://render.com/docs/preview-environments , https://docs.railway.com/guides/environments ,
https://railway.com/pricing , https://docs.fly.io/blueprints/review-apps-guide/ ,
https://coolify.io/docs/applications/ci-cd/github/preview-deploy

Conclusion for `platform`: a backend + PostgreSQL + object storage + T-Bank payments make SaaS
previews expensive and incomplete. Realistic: a per-PR compose stack on the existing VPS (or Coolify on top of it)
triggered by a `preview` label, with a GitHub environment `pr-N` and the URL in a sticky comment; teardown on close.
The free minimum is the `storybookUrl` link from Chromatic or Storybook on Pages.

---

## 5. Media storage outside git

| Medium | Limits/price | Visibility | Durability | Note |
|---|---|---|---|---|
| Git (as now) | file: recommendation 1 MB, hard limit 100 MB; repo on-disk recommended ≤10 GB | as the repo | forever, cannot be deleted without a rewrite | already ~101 MiB of `platform` history |
| Git LFS (Free) | 10 GiB storage + 10 GiB bandwidth/month per account, public counts too; on exceeding without payment — only pointer files / LFS is disabled until the end of the month | as the repo | forever | CI checkout spends bandwidth |
| Release assets | ≤2 GiB per file, ≤1000 assets per release, no limits on total size and traffic | as the repo | forever | `platform` uses immutable releases — evidence needs a separate repo or a pre-release with the tag `evidence-pr-N` |
| A separate branch/repo `evidence` + Pages | Pages ≤1 GB site, 100 GB/month | public | until deleted | rotation is mandatory |
| Cloudflare R2 | free: 10 GB-month, 1M Class A, 10M Class B, egress free; then $0.015/GB-month | public bucket / r2.dev or a custom domain; or presigned | as long as you pay | a secret in Actions is needed; CORS for the trace viewer |
| Actions artifacts | ≤90 days; public — free, private Free — 500 MB | login only | ≤90 days | not an archive |

Sources: https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits ,
https://docs.github.com/en/billing/concepts/product-billing/git-lfs ,
https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases ,
https://developers.cloudflare.com/r2/pricing/ ,
https://docs.github.com/en/billing/concepts/product-billing/github-actions

Uploading images into a PR comment programmatically:
- There is no official API; requests in the community got no response from GitHub, considered a deliberate
  limitation. https://github.com/orgs/community/discussions/28219 ,
  https://github.com/orgs/community/discussions/29993
- The unofficial endpoint `https://uploads.github.com/user-attachments/assets` with a Bearer token
  (described in August 2026, the author himself is unsure of its stability), the `gh-image` extension.
  https://island94.org/2026/08/programmatically-upload-attachments-to-github-issues-pull-requests-comments
  Do not use for the harness: undocumented, may disappear.
- A working path: upload to R2/Pages/a release asset and insert `![alt](https://...)` into a sticky
  comment; GitHub proxies through Camo.

Snippet (a release asset as durable storage in a separate repo):

```bash
gh release create "evidence-pr-$PR" --repo sachkov-inside/platform-evidence \
  --title "Evidence PR #$PR" --notes "Source: sachkov-inside/platform#$PR" --prerelease
gh release upload "evidence-pr-$PR" evidence/*.png evidence/*.webm --repo sachkov-inside/platform-evidence --clobber
# URL: https://github.com/sachkov-inside/platform-evidence/releases/download/evidence-pr-$PR/<file>
```

Check before adopting: whether `releases/download/...` PNGs are displayed inline in a comment (a redirect
to objects.githubusercontent.com; through Camo it usually works — unconfirmed), and whether
`.webm` plays (GitHub's inline player works for videos uploaded via the UI; for external links — only
a link, unconfirmed). Reliable for video: a "▶ video" link + a GIF preview.

---

## 6. Dashboards across many tasks

- GitHub Projects: fields (Status, Iteration, a text "Evidence URL", a single-select "Acceptance"),
  views table/board/roadmap; the Iteration field — any length, breaks, filters `@current`,
  `@previous`, `@next`, grouping by iteration.
  https://docs.github.com/en/issues/planning-and-tracking-with-projects/understanding-fields/about-iteration-fields
- Insights: current charts and historical (Burn up by default; Open/Completed/Closed PR/Not
  planned). Limitations of the historical charts on Free were not found — unconfirmed.
  https://docs.github.com/en/issues/planning-and-tracking-with-projects/viewing-insights-from-your-project/about-insights-for-projects
- Weekly digest: a scheduled workflow (`on: schedule: cron`) → `gh pr list --search "merged:>=$(date -d '7 days ago' +%F)" --json number,title,url`
  + links to sticky comments → an issue/Discussion "Week N" or a job summary. Cheap and durable,
  if written to an issue rather than a summary.
- Release notes: `.github/release.yml` with `changelog.exclude.labels/authors` and
  `categories[].labels`, `"*"` — catch-all. https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes
  `platform` already has immutable releases `v7..v9` from the `release.yml` workflow —
  `gh release create --generate-notes` and PR labels are enough.

```yaml
# .github/release.yml
changelog:
  exclude:
    labels: [skip-release-notes]
    authors: [dependabot]
  categories:
    - title: For participants
      labels: [user-facing]
    - title: Payments and access
      labels: [payments]
    - title: Internal
      labels: ["*"]
```

- Keep a Changelog (Added/Changed/Deprecated/Removed/Fixed/Security, an Unreleased section):
  https://keepachangelog.com/en/1.1.0/ . For an agent-driven flow it is better to generate from PRs than
  to maintain by hand; a manual CHANGELOG duplicates release notes.

---

## 7. Recording the agent's manual verification and CLI demos

- Browser: the same Playwright `recordVideo` (or `video.show` with step captions) in an acceptance
  scenario; the result is `.webm` + `trace.zip`. The "open trace" link =
  `https://trace.playwright.dev/?trace=<public URL of trace.zip>` (the storage needs CORS).
  https://playwright.dev/docs/videos , https://playwright.dev/docs/trace-viewer
- A GIF preview for a comment: `ffmpeg -i video.webm -vf "fps=8,scale=800:-1" preview.gif`
  (inline in Markdown as an image; the video as a link).
- CLI: asciinema records `.cast`; asciinema.org is a free public hosting with visibility
  public/unlisted/private, self-hosting is possible. https://docs.asciinema.org/manual/server/
  `agg demo.cast demo.gif` — a GIF for embedding in a PR. https://docs.asciinema.org/manual/agg/
  The retention policy for unclaimed recordings on asciinema.org — unconfirmed.

---

## 8. Recommended stack (by value/cost ratio)

1. **A sticky "result card" in the PR** (marocchino v3) from machine inputs: JUnit (Vitest,
   Playwright), `docs/verification/*.json`, the list of `@evidence` scenarios, links to media and the stand.
   Cost 0, survives retention, the owner sees everything in one place. Shorten the PR body to "what and
   why", and move pass/fail and checks into the card, which is written by CI, not by the agent from memory.
2. **Media outside git**: a public repo `platform-evidence` (Pages from a branch, `pr-N/` with the HTML report,
   screenshots, video; rotation ~90–180 days) + durable key frames in release assets of the same
   repo (`evidence-pr-N`). Private repos (Pages unavailable on Free) → Cloudflare R2 (10 GB free).
   Stop adding PNGs to `docs/evidence` (leave history alone).
3. **A Playwright `evidence` project**: `screenshot: on`, `video: on` with step captions, `trace: on`
   only for acceptance scenarios; publish the HTML report into `pr-N/`; the `github` reporter and the
   Vitest `github-actions` reporter for an operational view (summary/annotations, ≤90 days).
4. **Release notes** via `.github/release.yml` + labels; a weekly digest in an issue by cron.
   Projects: an "Evidence URL" field and an "Awaiting acceptance" view.
5. **Visual review**: Chromatic Free (5,000 snapshots, Chrome) — only if Storybook already
   covers the key screens; check merge queue compatibility and the actual snapshot consumption.
   Argos/Percy — not until we hit the limit; OSS programs do not apply to the commercial `platform`.
6. **Preview on a PR**: a per-PR docker compose on the existing VPS (or Coolify) with `environment.url` —
   high value for acceptance, but a noticeable cost of work and a security concern (PR code on the server).
   Vercel Hobby is prohibited for commercial use; Render previews require Pro.
7. **Allure 3 / Currents** — postpone: the value (trends, flaky) does not justify a second report format and
   history storage with a single owner.
