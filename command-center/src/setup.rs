//! `configure`, `unconfigure` and `doctor`: the only writes to the owner's Herdr configuration are
//! marker-fenced blocks, written after the owner agrees and removed by `unconfigure`.

use std::io::{BufRead, Write};
use std::path::Path;

use toml::Value;

use crate::env::{self, Config, herdr_config_path, plugin_id};
use crate::{herdr, run};

const BEGIN: &str = "# >>> agent-workbench command center (managed by workbench-cc; `workbench-cc unconfigure` removes it) >>>";
const END: &str = "# <<< agent-workbench command center <<<";

/// Herdr's default prefix bindings (`herdr --default-config`, 0.9.1), which stay taken unless the
/// owner rebinds them. herdr-nvim suggests `prefix+e` and `prefix+o`, which are defaults too.
const HERDR_DEFAULTS: &[(&str, &str)] = &[
    ("help", "prefix+?"),
    ("settings", "prefix+s"),
    ("detach", "prefix+q"),
    ("reload_config", "prefix+shift+r"),
    ("open_notification_target", "prefix+o"),
    ("workspace_picker", "prefix+w"),
    ("goto", "prefix+g"),
    ("new_workspace", "prefix+shift+n"),
    ("new_worktree", "prefix+shift+g"),
    ("rename_workspace", "prefix+shift+w"),
    ("close_workspace", "prefix+shift+d"),
    ("new_tab", "prefix+c"),
    ("rename_tab", "prefix+shift+t"),
    ("previous_tab", "prefix+p"),
    ("next_tab", "prefix+n"),
    ("close_tab", "prefix+shift+x"),
    ("rename_pane", "prefix+shift+p"),
    ("edit_scrollback", "prefix+e"),
    ("focus_pane_left", "prefix+h"),
    ("focus_pane_down", "prefix+j"),
    ("focus_pane_up", "prefix+k"),
    ("focus_pane_right", "prefix+l"),
    ("cycle_pane_next", "prefix+tab"),
    ("cycle_pane_previous", "prefix+shift+tab"),
    ("split_vertical", "prefix+v"),
    ("split_horizontal", "prefix+minus"),
    ("close_pane", "prefix+x"),
    ("zoom", "prefix+z"),
    ("resize_mode", "prefix+r"),
    ("toggle_sidebar", "prefix+b"),
];

/// Removes every block we wrote.
fn strip_blocks(text: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if trimmed == BEGIN {
            inside = true;
        } else if trimmed == END && inside {
            inside = false;
        } else if !inside {
            out.push_str(line);
        }
    }
    out
}

fn normalise_key(key: &str) -> String {
    key.trim().to_lowercase().replace(' ', "")
}

/// The keys a `[keys]` value binds: one string or a list of strings.
fn keys_of(value: &Value) -> Vec<String> {
    match value {
        Value::String(key) => vec![normalise_key(key)],
        Value::Array(list) => list
            .iter()
            .filter_map(Value::as_str)
            .map(normalise_key)
            .collect(),
        _ => vec![],
    }
}

/// Every key the configuration binds, with what it is bound to. Plugins cannot declare keys in
/// their manifests, so other plugins' bindings are the `[[keys.command]]` entries.
fn bound_keys(config: &Value) -> Vec<(String, String)> {
    let keys = config.get("keys");
    let mut bound = vec![];
    for (action, default) in HERDR_DEFAULTS {
        match keys.and_then(|k| k.get(*action)) {
            Some(custom) => {
                for key in keys_of(custom) {
                    bound.push((key, format!("Herdr's {action}")));
                }
            }
            None => bound.push((
                normalise_key(default),
                format!("Herdr's {action} (default)"),
            )),
        }
    }
    if let Some(Value::Table(table)) = keys {
        for (action, value) in table {
            if action != "prefix" && !HERDR_DEFAULTS.iter().any(|(a, _)| a == action) {
                for key in keys_of(value) {
                    bound.push((key, format!("Herdr's {action}")));
                }
            }
        }
        if let Some(Value::Array(commands)) = table.get("command") {
            for command in commands {
                if let Some(key) = command.get("key").and_then(Value::as_str) {
                    let what = command
                        .get("command")
                        .and_then(Value::as_str)
                        .unwrap_or("a custom command");
                    bound.push((normalise_key(key), format!("[[keys.command]] {what}")));
                }
            }
        }
    }
    bound
}

fn quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', r"'\''"))
}

fn toml_string(s: &str) -> String {
    Value::String(s.to_string()).to_string()
}

struct Plan {
    text: String,
    notes: Vec<String>,
}

fn plan(original: &str, config: &Config) -> Result<Plan, String> {
    let base = strip_blocks(original);
    let parsed: Value =
        toml::from_str(&base).map_err(|e| format!("cannot read the Herdr configuration: {e}"))?;
    let bound = bound_keys(&parsed);
    let plugin = plugin_id();
    let wanted = [
        (
            &config.keys.overview,
            format!("{plugin}.overview"),
            "command center: overview",
        ),
        (
            &config.keys.card,
            format!("{plugin}.card"),
            "command center: result card of this agent",
        ),
    ];
    let mut conflicts = vec![];
    for (key, _, _) in &wanted {
        if let Some((_, owner)) = bound.iter().find(|(k, _)| *k == normalise_key(key)) {
            conflicts.push(format!("{key} is already bound to {owner}"));
        }
    }
    if normalise_key(&config.keys.overview) == normalise_key(&config.keys.card) {
        conflicts.push(format!(
            "the overview and card keys are both {}",
            config.keys.overview
        ));
    }
    if !conflicts.is_empty() {
        return Err(format!(
            "refusing to write: {}. Choose other keys with --overview-key / --card-key or [keys] in {}.",
            conflicts.join("; "),
            env::config_path().display()
        ));
    }

    let exe = std::env::current_exe()
        .map(|p| env::canonical(&p))
        .map_err(|e| e.to_string())?;
    let status_entry = format!(
        "{{ type = \"command\", command = {}, interval_seconds = 15, timeout_seconds = 10 }}",
        toml_string(&format!("{} tab-status", quote(&exe)))
    );
    let ui = parsed.get("ui");
    let mut notes = vec![];
    let mut tail = vec![];
    for (key, action, description) in &wanted {
        tail.push(format!(
            "[[keys.command]]\nkey = {}\ntype = \"plugin_action\"\ncommand = {}\ndescription = {}\n",
            toml_string(key),
            toml_string(action),
            toml_string(description)
        ));
    }
    let mut ui_block = None;
    if ui.and_then(|u| u.get("tab_bar_right")).is_some() {
        notes.push(format!(
            "[ui] tab_bar_right is yours; add this entry to it yourself: {status_entry}"
        ));
    } else if base.lines().any(|line| line.trim() == "[ui]") {
        ui_block = Some(format!("tab_bar_right = [{status_entry}]\n"));
    } else {
        tail.push(format!("[ui]\ntab_bar_right = [{status_entry}]\n"));
    }
    let agents = ui
        .and_then(|u| u.get("sidebar"))
        .and_then(|s| s.get("agents"));
    if agents.is_some() {
        notes.push(format!("[ui.sidebar.agents] is yours; add \"$wb_tag\" to one of its rows to see the tag (token {}).", herdr::TAG_TOKEN));
    } else {
        tail.push(format!(
            "[ui.sidebar.agents]\nrows = [[\"state_icon\", \"machine\", \"workspace\", \"tab\"], [\"agent\", \"${}\"]]\n",
            herdr::TAG_TOKEN
        ));
    }

    let mut text = String::new();
    let mut ui_done = ui_block.is_none();
    for line in base.split_inclusive('\n') {
        text.push_str(line);
        if !ui_done && line.trim() == "[ui]" {
            if !line.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!(
                "{BEGIN}\n{}{END}\n",
                ui_block.as_deref().unwrap_or("")
            ));
            ui_done = true;
        }
    }
    if !ui_done {
        return Err("cannot find the [ui] table header to add the tab-bar entry; add it yourself or move [ui] to its own line".into());
    }
    let owned = text.trim_end();
    let separator = if owned.is_empty() { "" } else { "\n\n" };
    let text = format!("{owned}{separator}{BEGIN}\n{}{END}\n", tail.join("\n"));

    let check: Value = toml::from_str(&text).map_err(|e| {
        format!("the change would break the Herdr configuration, nothing written: {e}")
    })?;
    let has_keys = check
        .get("keys")
        .and_then(|k| k.get("command"))
        .and_then(Value::as_array)
        .is_some_and(|c| {
            c.iter()
                .filter(|c| {
                    c.get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|s| s.starts_with(&plugin))
                })
                .count()
                == 2
        });
    if !has_keys {
        return Err("the change would not add the key bindings, nothing written".into());
    }
    Ok(Plan { text, notes })
}

pub fn configure(config: &Config, yes: bool) -> Result<(), String> {
    let path = herdr_config_path();
    let original = std::fs::read_to_string(&path).unwrap_or_default();
    let plan = plan(&original, config)?;
    let mut out = std::io::stdout();
    let _ = writeln!(
        out,
        "workbench-cc configure will change {}:",
        path.display()
    );
    let _ = writeln!(
        out,
        "  key {}  opens the command center overview",
        config.keys.overview
    );
    let _ = writeln!(
        out,
        "  key {}  opens the result card of the focused agent",
        config.keys.card
    );
    let _ = writeln!(
        out,
        "  the tab bar shows how many things need you; the agents sidebar shows the ${} tag",
        herdr::TAG_TOKEN
    );
    for note in &plan.notes {
        let _ = writeln!(out, "  note: {note}");
    }
    let _ = writeln!(
        out,
        "Everything is written between these markers and `workbench-cc unconfigure` removes it:\n  {BEGIN}\n  {END}"
    );
    if !yes {
        let _ = write!(out, "Write these changes? [y/N] ");
        let _ = out.flush();
        let mut answer = String::new();
        let _ = std::io::stdin().lock().read_line(&mut answer);
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            let _ = writeln!(out, "Nothing written.");
            return Ok(());
        }
    }
    write_config(&path, &original, &plan.text)?;
    let _ = writeln!(
        out,
        "Written. A backup of the previous file is next to it with the suffix .workbench-cc.bak."
    );
    reload();
    Ok(())
}

pub fn unconfigure() -> Result<(), String> {
    let path = herdr_config_path();
    let original = std::fs::read_to_string(&path).unwrap_or_default();
    let stripped = strip_blocks(&original);
    if stripped == original {
        println!("Nothing to remove from {}.", path.display());
        return Ok(());
    }
    let cleaned = stripped.replace("\n\n\n", "\n\n");
    let text = if cleaned.trim().is_empty() {
        String::new()
    } else {
        format!("{}\n", cleaned.trim_end())
    };
    write_config(&path, &original, &text)?;
    println!(
        "Removed the command center's blocks from {}.",
        path.display()
    );
    reload();
    Ok(())
}

/// Writes through a symlinked dotfile rather than replacing the link, after a backup.
fn write_config(path: &Path, original: &str, text: &str) -> Result<(), String> {
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if let Some(dir) = target.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if !original.is_empty() {
        let backup = format!("{}.workbench-cc.bak", target.display());
        std::fs::write(&backup, original)
            .map_err(|e| format!("cannot write the backup {backup}: {e}"))?;
    }
    std::fs::write(&target, text).map_err(|e| format!("cannot write {}: {e}", target.display()))
}

fn reload() {
    match herdr::reload_config() {
        Ok(()) => println!("Herdr reloaded its configuration."),
        Err(err) => println!(
            "Herdr did not reload its configuration ({err}); run `herdr server reload-config`."
        ),
    }
}

/// The oldest Herdr version this plugin is built against (`min_herdr_version` in the manifest).
const MIN_HERDR: (u64, u64, u64) = (0, 9, 1);

fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v
        .trim_start_matches('v')
        .split(['.', '-'])
        .map(|p| p.parse::<u64>().ok());
    Some((
        parts.next()??,
        parts.next()??,
        parts.next().flatten().unwrap_or(0),
    ))
}

pub fn doctor(config: Result<Config, String>) -> bool {
    let mut ok = true;
    let mut report = |good: bool, what: &str, detail: String| {
        ok &= good;
        println!("{} {what}: {detail}", if good { "ok  " } else { "FAIL" });
    };
    match herdr::version() {
        Ok(v) => match parse_version(&v) {
            Some(parsed) if parsed >= MIN_HERDR => report(true, "herdr", v),
            _ => report(
                false,
                "herdr",
                format!(
                    "{v} is older than {}.{}.{}",
                    MIN_HERDR.0, MIN_HERDR.1, MIN_HERDR.2
                ),
            ),
        },
        Err(err) => report(false, "herdr", err),
    }
    let git = run::run("git", &["--version"], None, None);
    report(
        git.ok,
        "git",
        if git.ok {
            git.stdout.trim().to_string()
        } else {
            git.error_line()
        },
    );
    let gh = run::run("gh", &["--version"], None, None);
    if gh.ok {
        report(
            true,
            "gh",
            gh.stdout.lines().next().unwrap_or("").trim().to_string(),
        );
        let auth = run::run(
            "gh",
            &["auth", "status"],
            None,
            Some(std::time::Duration::from_secs(20)),
        );
        report(
            auth.ok,
            "gh auth",
            if auth.ok {
                "logged in".into()
            } else {
                format!("{} (run `gh auth login`)", auth.error_line())
            },
        );
    } else {
        report(false, "gh", gh.error_line());
    }
    match config {
        Ok(config) => {
            report(true, "config", env::config_path().display().to_string());
            for (what, template) in [
                ("editor", &config.editor),
                ("diff", &config.diff),
                ("browser", &config.browser),
            ] {
                let program = template.first().cloned().unwrap_or_default();
                match run::which(&program) {
                    Some(path) => report(true, what, path.display().to_string()),
                    None => report(false, what, format!("{program} not found on PATH")),
                }
            }
        }
        Err(err) => report(false, "config", err),
    }
    ok
}
