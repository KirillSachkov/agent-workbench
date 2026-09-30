#!/usr/bin/env bash
# Creates the harness's labels in the current GitHub repository. Colours carry meaning: yellow waits
# for a decision, purple waits for someone else, green is for an agent (light green: the agent may
# merge), blue is for a person, white is closed out. Descriptions are plain language, as people see
# them on GitHub.
#
# Without --force an existing label is left untouched (gh reports it and the script goes on); with
# --force its colour and description are updated to the ones below.
set -uo pipefail

force=()
[ "${1:-}" = "--force" ] && force=(--force)

label() {
  gh label create "$1" --color "$2" --description "$3" ${force[@]+"${force[@]}"} \
    || echo "kept existing label: $1"
}

label "needs-triage"     "fbca04" "Someone needs to look at this and decide what happens next"
label "needs-info"       "d4c5f9" "Waiting for the reporter to answer a question"
label "ready-for-agent"  "0e8a16" "Fully described and approved; an agent can take it"
label "ready-for-human"  "1d76db" "Needs a person to do it"
label "wontfix"          "ffffff" "Will not be done"
label "acceptance:human" "0052cc" "The owner checks the result and merges it"
label "acceptance:auto"  "c2e0c6" "The agent may merge once CI is green and review found nothing serious"
