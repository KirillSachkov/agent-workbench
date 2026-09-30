#!/usr/bin/env bash
# Tier-1 runtime smoke check (seam 2 of the harness spec, #1). Installs this harness source into a
# temporary fixture project with `workbench init`, then asks headless Claude Code, Codex and
# OpenCode sessions whether they:
#
#   1. see the project instructions (a codename that only AGENTS.md states),
#   2. list the model-invoked skills,
#   3. do not start a user-invoked skill on their own, and
#   4. run it when called by name,
#
# and records how OpenCode lists a skill it sees in both .claude/skills and .agents/skills.
#
# Runs before a release and on demand, not on every push: it spends real model tokens. Nothing is
# written outside the fixture and its sibling log directory except the report, the release build
# in target/ and whatever session history a runtime keeps on its own. Usage:
#
#   scripts/smoke-tier1.sh [--from <repository>@<ref>] [--runtimes "claude codex opencode"]
#                          [--report <file>]
#
# --from defaults to this repository at its current commit (uncommitted changes are not seen).
set -uo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
from="$root@$(git -C "$root" rev-parse HEAD)"
runtimes="claude codex opencode"
report=""
while [ $# -gt 0 ]; do
  case "$1" in
    --from) from="$2"; shift 2 ;;
    --runtimes) runtimes="$2"; shift 2 ;;
    --report) report="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

codename="ZEPHYR-$RANDOM$RANDOM"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/workbench-smoke.XXXXXX")"
logs="$fixture.logs"
mkdir -p "$logs"

cargo build -q --release -p workbench --manifest-path "$root/Cargo.toml" || exit 1
workbench="$root/target/release/workbench"

# The fixture: a git repository with a codename only AGENTS.md knows, the harness installed into
# it, and two probe skills: one model-invoked, one user-invoked.
(
  cd "$fixture" || exit 1
  git init -q -b main
  printf '# Smoke fixture\n\nThe project codename is %s. When asked for the codename, answer with it exactly.\n' \
    "$codename" > AGENTS.md
  git add -A && git -c user.name=smoke -c user.email=smoke@example.com commit -q -m fixture
  "$workbench" init --from "$from" > "$logs/init.log" 2>&1 || { cat "$logs/init.log"; exit 1; }
  mkdir -p .agents/skills/smoke-model .agents/skills/smoke-user
  cat > .agents/skills/smoke-model/SKILL.md <<'SKILL'
---
name: smoke-model
description: Use when the user asks for the smoke word.
---

Reply with exactly SMOKE-MODEL-OK and nothing else.
SKILL
  cat > .agents/skills/smoke-user/SKILL.md <<'SKILL'
---
name: smoke-user
description: Use when the user asks to greet the fixture.
disable-model-invocation: true
---

Reply with exactly SMOKE-USER-OK and nothing else.
SKILL
  "$workbench" sync >> "$logs/init.log" 2>&1 || { cat "$logs/init.log"; exit 1; }
  git add -A && git -c user.name=smoke -c user.email=smoke@example.com commit -q -m harness
) || exit 1

# ask <runtime> <probe> <prompt>: runs one headless session in the fixture and prints its answer.
# Returns non-zero when the session failed or answered nothing, so a crash never passes a probe.
ask() {
  local runtime="$1" prompt="$3" out="$logs/$1-$2.txt"
  case "$runtime" in
    claude)
      (cd "$fixture" && claude -p "$prompt" --no-session-persistence --permission-mode default \
        --disallowedTools "Bash Edit Write" < /dev/null) > "$out" 2>&1 ;;
    codex)
      codex exec --cd "$fixture" --sandbox read-only --ephemeral --skip-git-repo-check \
        -o "$out.last" "$prompt" < /dev/null > "$out.full" 2>&1
      # Only the final message counts: the full log echoes tool output (AGENTS.md included).
      cat "$out.last" > "$out" 2>/dev/null || : > "$out" ;;
    opencode)
      # A user calls a skill by name in OpenCode as a slash command (`/smoke-user` in the TUI).
      if [ "${prompt#/}" != "$prompt" ]; then
        (cd "$fixture" && opencode run --command "${prompt#/}" < /dev/null) > "$out" 2>&1
      else
        (cd "$fixture" && opencode run "$prompt" < /dev/null) > "$out" 2>&1
      fi ;;
  esac
  local status=$?
  cat "$out"
  [ "$status" = 0 ] && grep -q '[^[:space:]]' "$out"
}

# How each runtime calls a skill by name.
by_name() {
  case "$1" in
    claude) echo "/smoke-user" ;;
    codex) echo "\$smoke-user" ;;
    opencode) echo "/smoke-user" ;;
  esac
}

pass() { [ "$1" = 0 ] && echo pass || echo FAIL; }

rows=""
failed=0
for runtime in $runtimes; do
  if ! command -v "$runtime" > /dev/null; then
    rows="$rows| $runtime | not installed | | | | |\n"
    failed=1
    continue
  fi
  version="$("$runtime" --version 2>/dev/null | head -1 | tr '|' '/')"

  a1="$(ask "$runtime" instructions "What is this project's codename? Answer with the codename only.")"
  grep -q "$codename" <<< "$a1"; r1=$?

  a2="$(ask "$runtime" list "List the names of every skill available to you in this project, one per line, with no other text.")"
  grep -q "smoke-model" <<< "$a2" && grep -q "coordinate" <<< "$a2"; r2=$?
  user_listed=no
  grep -q "smoke-user" <<< "$a2" && user_listed=yes

  # The skill's only effect is a reply that is exactly SMOKE-USER-OK; a mention inside an
  # explanation does not count as running it.
  if a3="$(ask "$runtime" unprompted "Please greet the fixture.")"; then
    ! grep -qx "[[:space:]]*SMOKE-USER-OK[[:space:]]*" <<< "$a3"; r3=$?
  else
    r3=1
  fi

  a4="$(ask "$runtime" by-name "$(by_name "$runtime")")"
  grep -qx "[[:space:]]*SMOKE-USER-OK[[:space:]]*" <<< "$a4"; r4=$?

  note="user-invoked skill listed: $user_listed"
  if [ "$runtime" = opencode ]; then
    # OpenCode reads both .claude/skills and .agents/skills; count how often it registers a skill.
    # Written to a file first: piped, the long output is cut short.
    (cd "$fixture" && opencode debug skill > "$logs/opencode-skills.json" 2>/dev/null)
    copies="$(grep -c '"name": "smoke-model"' "$logs/opencode-skills.json" 2>/dev/null || echo "unknown number of")"
    note="$note; smoke-model registered $copies time(s), though reachable through both .claude/skills (a directory link) and .agents/skills"
  fi
  rows="$rows| $runtime ($version) | $(pass $r1) | $(pass $r2) | $(pass $r3) | $(pass $r4) | $note |\n"
  for r in $r1 $r2 $r3 $r4; do [ "$r" = 0 ] || failed=1; done
done

# The report names no local paths: it may be committed to a public repository.
case "$from" in
  "$root"@*) source_line="this repository at ${from##*@}" ;;
  *) source_line="$from" ;;
esac
table="# Tier-1 smoke check

- Date: $(date -u +%Y-%m-%d)
- Harness source: \`$source_line\`
- Fixture: a fresh git repository; \`workbench init\`, then probe skills \`smoke-model\` (model-invoked) and \`smoke-user\` (user-invoked) and \`workbench sync\`.

| Runtime | Sees instructions | Lists model-invoked skills | Does not start a user-invoked skill on its own | Runs it when called by name | Notes |
|---|---|---|---|---|---|
$(printf '%b' "$rows")"

echo "$table"
[ -n "$report" ] && echo "$table" > "$report"
echo
echo "fixture kept at $fixture, answers of every session in $logs (delete both when done)"
exit "$failed"
