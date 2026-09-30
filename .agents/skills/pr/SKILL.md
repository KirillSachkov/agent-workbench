---
name: pr
description: "Use when opening or updating a pull request: writes the PR body as a result card (result, what needs the owner, where to look first, how to try it, criteria with evidence) and follows the task's acceptance mode."
metadata:
  credits:
    skill: show-me
    author: Dex Horthy
    organisation: Humanlayer
    url: "https://github.com/humanlayer/skills/blob/main/plugins/show-me/skills/show-me/SKILL.md"
---

The PR body is a **result card**: the report the owner checks in one step, on GitHub and in the command center. Write it from the primary sources (the task or spec, read again with body and comments; the diff; and the output of what you ran), not from memory of the session.

The command center reads four sections by their headings: `Result`, `Needs you`, `Look first` and `How to try`. Keep these headings exactly as written. The shape is documented as a convention in the artifact contract; nothing checks it, so an honest card matters more than a full one.

## 1. Gather the facts

- The task: its acceptance criteria and preservation criteria (what must not break), its labels, and the acceptance mode (`acceptance:auto`, `acceptance:human`, or none, which means the project default in `workbench.toml`, `human` unless changed).
- The candidate: push the branch, then take the head commit with `git rev-parse HEAD`. Every evidence record names the commit it was produced on.
- The diff against the base: `git diff <base>...HEAD --stat`, and which of the changed files are tests or CI configuration.
- The review: the findings `code-review` reported, and what you did about each.

## 2. Write the card

~~~markdown
## Result

<one line: what now works, in the task's words>

## Needs you

<decisions or actions only the owner can take, one per line; or "Nothing: ready to merge." / "Nothing: merged under acceptance:auto.">

## Look first

- `path/to/file.ext:42` - <why this place deserves the first look>

## How to try

```sh
<command that shows the change working>
```

<http(s) URL of the running app or page, when there is one>

## Criteria

- **verified**: <acceptance criterion, as the task states it>
  - `test` @ `<sha>`: `<test name>`: failed before the change, passes now.
- **unverified**: <criterion> (<why it has no evidence yet>)

Preservation:

- **verified**: <what must not break>
  - `ci` @ `<sha>`: <check name>: green, <link>

## Changes to tests and CI

<every test or CI file changed, and why; or "None.">

## Not done

<what the task asked for that this PR does not do, and follow-ups; or "Nothing.">

## What changed

<optional: the smallest visual that makes the change clear; see VISUALS.md>

Closes #<issue>
~~~

### Result

One line a person who has not seen the session understands. Say what works now, not what you did.

### Needs you

Decisions the owner must take and actions only a person can do (a secret to add, a design choice left open). Ambiguity you met during the work belongs here if it is still open. Under `acceptance:human` the merge itself is implied; do not list it.

### Look first

One line per place, `path:line - reason`: the riskiest change, the core of the logic, anything surprising. Paths are relative to the repository root and point inside it. Three to five lines is typical; order them by importance.

### How to try

A command the owner can paste to see the change working, and the address of the running app when the change has one. Commands must be safe to run: no deletion, no deploy.

### Criteria and evidence

Every acceptance and preservation criterion of the task gets exactly one status:

- **verified**: at least one evidence record supports it on the current head.
- **failed**: evidence shows it does not hold.
- **unverified**: no evidence. This is the default; never upgrade it by assertion.

An evidence record is `` `<type>` @ `<short sha>`: <reference>: <observation> ``. The types:

| Type | Reference | Observation |
|---|---|---|
| `ci` | the check run name and its URL | its conclusion |
| `command` | the command as run | the relevant lines of its real output |
| `test` | the test's name or `path::name` | for new behaviour: failed before the change, passes after |
| `screenshot` | the image, uploaded to the PR | what it shows |
| `recording` | the recording's link | what it shows |
| `review` | the review report (for example the `code-review` result) | findings left open, if any |
| `manual` | who checked | what they saw |

Command output is pasted from the tool's result, never retyped or paraphrased; keep only the lines that carry the signal and redact secrets.

**Stale evidence.** Evidence binds to the commit it ran on. When you update the card after new commits, keep older records but mark them `(stale)` and do not let them verify a criterion; rerun what supports acceptance on the new head.

### Changes to tests and CI

List every changed test file and CI configuration file with the reason, separately from the product change, so a weakened check is visible. Deleting or loosening an assertion needs a stated reason.

### Not done

Parts of the task that are not delivered, known gaps, follow-up issues. Silence here claims the task is complete.

### What changed

Optional. Pick the smallest visual that makes the change clear: pseudocode, a call tree, a file tree, a diff sketch, Mermaid. See [VISUALS.md](VISUALS.md).

Skip preambles and keep prose brief. Use the project's domain vocabulary from `GLOSSARY.md`.

## 3. Open or update the pull request

Use the pull request operations in `docs/agents/issue-tracker.md` (or the code host's CLI if it has none). Write the body to a file and pass the file, so Markdown is not mangled by the shell. Link the task so the PR closes it on merge (`Closes #<issue>` on GitHub). When the card changes after new commits, update the body in place rather than adding a comment.

## 4. Follow the acceptance mode

- **`acceptance:human`**, or no label and a `human` project default: stop at the open pull request. Do not merge.
- **`acceptance:auto`**, or no label and an `auto` project default: merge only when the project's CI is green on the current head and the non-author review left no serious open findings. A `code-review` run by fresh-context subagents counts as non-author review, as does a review by another agent or a person. Then write "Nothing: merged under acceptance:auto." under `Needs you`. If either condition fails, stop at the open pull request and say which one failed under `Needs you`.

A reviewer's verdict never accepts work by itself, and an agent never sets, removes or changes an acceptance label on its own decision.
