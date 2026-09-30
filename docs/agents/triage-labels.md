# Labels

The skills speak in terms of canonical roles. This file maps those roles to the actual label strings used in this repo's issue tracker. Keep it in step with `[tracker.labels]` in `workbench.toml`.

## Triage roles

| Role | Label in our tracker | Meaning |
| --- | --- | --- |
| `needs-triage` | `needs-triage` | Someone needs to look at this and decide what happens next |
| `needs-info` | `needs-info` | Waiting for the reporter to answer a question |
| `ready-for-agent` | `ready-for-agent` | Fully described and approved; an agent can take it |
| `ready-for-human` | `ready-for-human` | Needs a person to do it |
| `wontfix` | `wontfix` | Will not be done |

Labelling a task `ready-for-agent` is the owner's **intent approval** on the small track.

## Acceptance modes

| Mode (key in `workbench.toml`) | Label in our tracker | Meaning |
| --- | --- | --- |
| `acceptance-human` | `acceptance:human` | The owner checks the result and merges it |
| `acceptance-auto` | `acceptance:auto` | The agent may merge once CI is green and review found nothing serious |

A task with neither label is accepted by the project default (`[acceptance] default` in `workbench.toml`; `human` unless changed). Only a person sets or changes an acceptance label.

When a skill mentions a role (e.g. "apply the ready-for-agent triage label"), use the corresponding label string from these tables.

Edit the label column to match whatever vocabulary you actually use.
