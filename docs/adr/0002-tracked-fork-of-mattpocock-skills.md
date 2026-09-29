# Matt Pocock's skills are a tracked fork, not a subscription

The default method builds on `mattpocock/skills` (MIT) as a fork we edit, not as a pinned unmodified
upstream layer. Each forked skill records the upstream commit it was forked at; upstream changes are
reviewed and adopted selectively. The owner chose this to adapt the skills freely to the harness —
for example tracker operations and completion steps — while still picking up upstream improvements.
Upstream offers only "subscribe or fork" and no overlay mechanism, so editing is only possible in a
fork.

Evidence: `docs/research/2026-09-29-mattpocock-skills-design.md` §3, §5, §7. Decision ticket: #7.

## Considered options

- **Subscription: pinned, unmodified upstream, customised only through `docs/agents/*`,
  `CONTEXT.md` and `AGENTS.md`.** Recommended in the session for lower merge cost; rejected by the
  owner because it limits how far the skills can be adapted.

## Consequences

- Upstream renames arrive without aliases (`to-prd` → `to-spec`, `to-issues` → `to-tickets`), so
  every adoption is a reviewed merge, not an automatic update.
- The harness lock must record the upstream commit per forked skill; today's lock records only the
  installer revision, which is why the current copy's provenance had to be recovered by diffing.
