---
name: setup-harness
description: Configure this repo for the harness's skills — set up its issue tracker operations, labels (triage roles and acceptance modes) and domain doc layout. Run once before first use of the engineering skills, or to switch trackers.
disable-model-invocation: true
---

# Setup Harness

Scaffold the per-repo configuration that the engineering skills assume:

- **Issue tracker** — where issues live and the exact commands for each operation (GitHub by default; GitLab and local markdown are also supported out of the box)
- **Labels** — the strings for the five triage roles and the two acceptance modes, created on the tracker
- **Domain docs** — where `CONTEXT.md` and ADRs live, and the consumer rules for reading them

The `workbench` binary already wrote the skills, the managed block in `AGENTS.md`, the `CLAUDE.md` bridge and `workbench.toml`. This skill writes what needs judgment: `docs/agents/*` and the labels.

This is a prompt-driven skill, not a deterministic script. Explore, present what you found, confirm with the user, then write.

## Process

### 1. Explore

Look at the current repo to understand its starting state. Read whatever exists; don't assume:

- `git remote -v` — is this a GitHub or GitLab repo? Which one?
- `AGENTS.md` — does it exist, does it carry the workbench managed block, and is there already an `## Agent skills` section? `CLAUDE.md` — does it carry the `@AGENTS.md` bridge line?
- `workbench.toml` — the tracker kind and label strings it declares (`workbench config` prints the resolved values)
- `CONTEXT.md` and `CONTEXT-MAP.md` at the repo root
- `docs/adr/` and any `src/*/docs/adr/` directories
- `docs/agents/` — does this skill's prior output already exist?
- `.scratch/` — sign that a local-markdown issue tracker convention is already in use
- On GitHub, the labels that already exist: `gh label list --json name,color,description`
- Monorepo signals — a `pnpm-workspace.yaml`, a `workspaces` field in `package.json`, a Cargo workspace with many members, or a populated `packages/*` with its own `src/`. Their absence means single-context, which is almost every repo.

If the managed block or the bridge is missing, tell the user to run `workbench sync`; do not write them by hand.

### 2. Present findings and ask

Summarise what's present and what's missing. Then take the sections in order — one section, one answer, then the next.

Lead each section with the recommended answer so the user can accept it in a word. Give a one-line explainer only when the choice genuinely branches; skip a section entirely when exploration already settled it.

**Section A — Issue tracker.**

> Explainer: The "issue tracker" is where issues live for this repo. Skills like `to-tickets`, `triage`, `to-spec`, `pr` and `coordinate` read from and write to it — they need the exact commands to create, read and link issues and pull requests. Pick the place you actually track work for this repo.

If a `git remote` points at GitHub, propose GitHub. If it points at GitLab (`gitlab.com` or a self-hosted host), propose GitLab. Otherwise (or if the user prefers), offer:

- **GitHub** — issues live in the repo's GitHub Issues (uses the `gh` CLI)
- **GitLab** — issues live in the repo's GitLab Issues (uses the [`glab`](https://gitlab.com/gitlab-org/cli) CLI)
- **Local markdown** — issues live as files under `.scratch/<feature>/` in this repo (good for solo projects or repos without a remote)
- **Other** (Jira, Linear, etc.) — ask the user to describe the workflow in one paragraph; record it as prose with the same sections as the templates (issues, blocking edges, parent and child, pull requests)

Record the choice in `docs/agents/issue-tracker.md`, and set `[tracker] kind` in `workbench.toml` when it differs from the file's current value. The GitHub and GitLab templates carry a "PRs as a request surface" flag, defaulted **off** — leave it off and don't raise it.

**Section B — Labels.** Ask exactly one question:

> Do you want to keep the default labels? (recommended: **yes**)

The defaults are the five triage roles, each label string equal to its role name (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`), and the two acceptance modes as `acceptance:human` and `acceptance:auto`. Only if the user says no — usually because the tracker already uses other names — collect the overrides, so skills apply existing labels instead of creating duplicates.

**Section C — Domain docs.** Default to **single-context** — one `CONTEXT.md` + `docs/adr/` at the repo root. This fits almost every repo; write it without asking.

Offer **multi-context** — a root `CONTEXT-MAP.md` pointing to per-context `CONTEXT.md` files — only when exploration found monorepo signals. Then confirm which layout they want.

### 3. Confirm and edit

Show the user a draft of:

- The `## Agent skills` section for `AGENTS.md`
- The contents of `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md` and `docs/agents/domain.md`
- The labels that will be created on the tracker, and any that already exist with a different colour or description

Let them edit before writing.

### 4. Write

**`AGENTS.md`.** Add the `## Agent skills` section outside the workbench managed block. If the section already exists, update it in place rather than appending a duplicate; don't touch the surrounding sections. Write instructions into `AGENTS.md`, never into `CLAUDE.md`, which stays a one-line bridge.

```markdown
## Agent skills

### Issue tracker

[one-line summary of where issues are tracked]. See `docs/agents/issue-tracker.md`.

### Labels

[one-line summary: the triage roles and acceptance modes]. See `docs/agents/triage-labels.md`.

### Domain docs

[one-line summary of layout — "single-context" or "multi-context"]. See `docs/agents/domain.md`.
```

**`docs/agents/`.** Write the files from the seed templates in this skill folder, filling in the choices:

- [issue-tracker-github.md](./issue-tracker-github.md) — GitHub issue tracker
- [issue-tracker-gitlab.md](./issue-tracker-gitlab.md) — GitLab issue tracker
- [issue-tracker-local.md](./issue-tracker-local.md) — local-markdown issue tracker
- [triage-labels.md](./triage-labels.md) — label mapping
- [domain.md](./domain.md) — domain doc consumer rules + layout

For "other" issue trackers, write `docs/agents/issue-tracker.md` from scratch using the user's description.

When labels were overridden, write the same strings into `[tracker.labels]` in `workbench.toml`, so the harness's code and the skills agree.

**Labels on the tracker.**

- **GitHub** with the default labels: run [labels-github.sh](./labels-github.sh) from the repository root. It creates missing labels and leaves existing ones alone; pass `--force` only if the user agreed to rewrite existing colours and descriptions. With overrides, run one `gh label create "<label>" --color <hex> --description "<text>"` per label instead, taking colours and descriptions from the script.
- **GitLab**: `glab label create --name "<label>" --color "#<hex>" --description "<text>"` per label, with the same colours and descriptions.
- **Local markdown**: nothing to create; the strings go on `Status:` lines.

### 5. Done

Tell the user the setup is complete and which skills now read these files. Mention they can edit `docs/agents/*.md` directly later — re-running this skill is only necessary to switch issue trackers or restart from scratch.
