//! Plain words for states and labels: the Overview reads as status for a person, not raw labels.

pub fn status(status: &str) -> &'static str {
    match status {
        "blocked" => "Waits for you",
        "done" => "Done, not seen yet",
        "working" => "Working",
        "idle" => "Idle",
        _ => "Unknown",
    }
}

pub fn ci(ci: &str) -> &'static str {
    match ci {
        "success" => "CI passed",
        "failure" => "CI failed",
        "pending" => "CI running",
        _ => "no CI",
    }
}

/// A short mark for the narrow sidebar.
pub fn ci_mark(ci: &str) -> &'static str {
    match ci {
        "success" => "✓",
        "failure" => "✗",
        "pending" => "…",
        _ => "",
    }
}

pub fn review(decision: Option<&str>) -> &'static str {
    match decision {
        Some("APPROVED") => "approved",
        Some("CHANGES_REQUESTED") => "changes requested",
        Some("REVIEW_REQUIRED") => "review required",
        _ => "no review",
    }
}

/// The phrase for a label; labels the harness does not define keep their own name.
pub fn label(label: &str) -> String {
    let phrase = match label {
        "needs-triage" => "Needs triage",
        "needs-info" => "Waits for information",
        "ready-for-agent" => "Ready for an agent",
        "ready-for-human" => "Ready for you",
        "wontfix" => "Won't be done",
        "acceptance:human" => "Acceptance: you",
        "acceptance:auto" => "Acceptance: automatic",
        "wayfinder:map" => "Map",
        "wayfinder:grilling" => "Decision to talk through",
        "wayfinder:research" => "Research",
        "wayfinder:prototype" => "Prototype",
        "wayfinder:task" => "Map task",
        other => {
            return match other.strip_prefix("wayfinder:") {
                Some(rest) => format!("Map: {rest}"),
                None => other.to_string(),
            };
        }
    };
    phrase.to_string()
}
