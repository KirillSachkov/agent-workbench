# agent-workbench

A general harness, development pipeline and command center for AI-first development with several
agent runtimes across projects.

## Language

### Harness and pipeline

**Harness**:
The installed instructions, skills, conventions and configuration through which agents work on a
project.
_Avoid_: framework, toolkit

**Artifact**:
A durable output of a stage that a human or a later stage relies on: a specification, plan, delivery
report, evidence or handoff.
_Avoid_: deliverable, document

**Candidate**:
The exact version of the work under review: the commit at the head of its pull request.
_Avoid_: build, snapshot

**Evidence**:
The per-criterion proof that a candidate meets its acceptance and preservation criteria: each
criterion is verified, failed or unverified, and every verified one cites evidence records.
_Avoid_: proof, test results

**Evidence record**:
One trace of execution supporting a criterion — a CI run, a command with its real output, a test, a
screenshot, a recording, a review report or a human note — tied to a candidate.
_Avoid_: attachment, log

**Stale evidence**:
Evidence produced on an earlier candidate than the current one. Kept, but it cannot support
acceptance.
_Avoid_: outdated proof

**Artifact contract**:
The documented shape of each artifact — required sections and worked examples — that skills follow
and that humans and reviewers judge. It is the core of the harness, is not enforced by the harness's
code, and does not depend on which method produced the artifact.
_Avoid_: template, schema

**Method**:
The way artifacts get produced: a chain of skills and instructions that agents follow. Replaceable;
any method whose artifacts follow the artifact contract is acceptable.
_Avoid_: process, workflow, methodology

**Default method**:
The method shipped with the harness and used unless a project chooses otherwise.

**Upstream**:
The external project a forked part of the harness was copied from, recorded by the commit it was
forked at so that later upstream changes can be reviewed and adopted selectively.
_Avoid_: vendor, source repo

**Invocation policy**:
Who may start a skill: only the user by name (user-invoked), or also the model on its own
(model-invoked). A user-invoked skill may call model-invoked skills, never another user-invoked one.
_Avoid_: visibility, trigger mode

**Runtime tier**:
How firmly the harness supports an agent runtime: tier 1 is verified before every release; tier 2
gets the same files without a guarantee and with its limits listed.
_Avoid_: support level, compatibility

### Work shape

**Track**:
The path a task takes through the pipeline, chosen once by its size: small (the task itself states
the intent, one PR), medium (a spec, then tickets) or large (a map of decisions first). Each track is
an artifact graph.
_Avoid_: flow, lane, mode

**Artifact graph**:
The declared pipeline of a track: which artifacts and facts must exist, what each depends on and which
facts mark it done. The harness computes each node's state from facts; it does not run or gate steps.
_Avoid_: workflow, state machine

**Node state**:
The state of one node in an artifact graph, computed from facts: done, ready (all dependencies done)
or blocked.
_Avoid_: status, stage status

**Intent approval**:
The owner's approval, before any code, of what will be done, what must not break and how done will be
recognised.
_Avoid_: sign-off, spec review

**Acceptance**:
The owner's decision that a delivered result is accepted; by default the merge of its pull request.
_Avoid_: approval, sign-off

**Acceptance mode**:
How a task's result is accepted, set at intent approval: by the owner (`human`, the default) or
automatically when the project's CI is green and non-author review has no serious open findings
(`auto`).
_Avoid_: review level, auto-merge

**Acceptance criteria**:
Observable conditions, stated before the work, that show the work is done.
_Avoid_: definition of done, exit criteria

**Repair round**:
One attempt by the agent to fix a failed check or review finding and rerun the checks. Infrastructure
failures do not count.
_Avoid_: retry, iteration

**Convention**:
An agreed way of working written into skills, instructions and examples, followed by agents and
judged by people, not enforced by the harness's code.
_Avoid_: rule, policy, gate

**Preservation criteria**:
Behaviour the work must not break, stated before the work.
_Avoid_: constraints, non-goals

### Configuration

**Managed block**:
The one clearly marked section of a project's `AGENTS.md` that the harness owns and may rewrite; the
rest of the file belongs to the project.
_Avoid_: generated section, harness block

**Built-in default**:
The tracks, skills, conventions and tracker settings the harness ships for software development, used
wherever a project does not configure otherwise.
_Avoid_: preset, template

**Project configuration**:
The one committed file in a project's repository that holds what the harness's code must read to
collect facts and install — artifact graphs, enabled skills, tracker settings — and wins over the
built-in default. Everything else is customised by editing the project's own harness files.
_Avoid_: profile, settings
