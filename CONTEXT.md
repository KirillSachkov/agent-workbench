# agent-workbench

A general harness, development pipeline and command center for AI-first development with several
agent runtimes across projects.

## Language

### Harness and pipeline

**Harness**:
The installed instructions, skills, configuration and checks through which agents work on a project.
_Avoid_: framework, toolkit

**Artifact**:
A durable output of a stage that a human or a later stage relies on: a specification, plan, delivery
report, evidence or handoff.
_Avoid_: deliverable, document

**Artifact contract**:
The rules an artifact must satisfy for its stage to count, checked by code. It is the core of the
harness and does not depend on which method produced the artifact.
_Avoid_: template, format

**Method**:
The way artifacts get produced: a chain of skills and instructions that agents follow. Replaceable;
any method whose artifacts pass the artifact contract is acceptable.
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
