# Releasing

How this repository releases the harness and the `workbench` binary. The command center releases
on its own tags (ADR 0006).

## Channels

- **Stable: semver tags** `vX.Y.Z` of this repository. A project pins one in its harness lock
  (`workbench init --from KirillSachkov/agent-workbench@v0.1.0`); `workbench update` offers the
  newest stable tag above it.
- **`@main`: a recorded choice.** A project may follow `main` instead
  (`--from KirillSachkov/agent-workbench@main`); the lock then records `main` and the commit, and
  `workbench update` brings the newest `main`. Choose it deliberately; it gets changes before any
  release notes exist.
- **Beta skills** live in [`beta/`](../beta/README.md) and are installed only when a project lists
  them under `[skills] beta` in its `workbench.toml`.

## Versions

One version covers the harness content and the binary: `version` in `harness.toml` and in
`crates/workbench/Cargo.toml` move together. `min_workbench` in `harness.toml` is the oldest binary
that can install the release; raise it when the content needs a newer binary.

- **Major:** a change that breaks projects on update (a removed skill without replacement, a lock
  or configuration format change).
- **Minor:** new skills, promoted beta skills, renames, new commands.
- **Patch:** fixes and wording.

## Steps

1. **Content is ready on `main`.** Every rename and removal since the last release is in
   `[changes]` of `harness.toml` (the map is cumulative: never delete an entry). Every forked skill
   has its `[upstream.<skill>]` entry.
2. **Changelog.** Add a section to `CHANGELOG.md`: added, changed, renamed and removed skills,
   binary changes, and anything a project must do by hand.
3. **Versions.** Set `version` in `harness.toml` and `crates/workbench/Cargo.toml`, and
   `min_workbench` if needed.
4. **Checks.** `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`.
5. **Tier-1 smoke check.** `scripts/smoke-tier1.sh --report docs/smoke/<date>-tier1.md` must pass
   for Claude Code, Codex and OpenCode. Commit the report and link it from the changelog.
6. **Owner approval.** The owner approves the release; nothing is tagged without it.
7. **Tag and publish.** Tag the approved commit on `main` and push the tag:
   `git tag -a vX.Y.Z -m "agent-workbench X.Y.Z" && git push origin vX.Y.Z`. Create the GitHub
   release from the tag with the changelog section as its notes:
   `gh release create vX.Y.Z --title "agent-workbench X.Y.Z" --notes-file <section>`.
8. **Check the install.** In a clean environment: `cargo install --locked --git
   https://github.com/KirillSachkov/agent-workbench --tag vX.Y.Z workbench`, then
   `workbench init --from KirillSachkov/agent-workbench@vX.Y.Z` in a scratch git repository.

## Promoting a beta skill

A convention, not a check: the skill has been used in a real project, its description and the
router (`ask-matt`) are updated, it moves from `beta/` to `skills/`, and the changelog records the
promotion.

## Adopting upstream changes

The forked skills track `mattpocock/skills` (ADR 0002). To adopt an upstream change: review it,
merge it three-way into our copy (base: the upstream file before the change; theirs: after; ours:
our skill), keep our intentional differences, and name the adopted commits in the comment above
the `[upstream.*]` entries of `harness.toml` and in the changelog. The recorded commit stays the
fork point until the whole skill is rebased onto a newer upstream commit.
