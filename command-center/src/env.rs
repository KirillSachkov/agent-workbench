//! Where the plugin keeps its files, its configuration, and the Herdr context it was started in.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The plugin id in `herdr-plugin.toml`.
pub const PLUGIN_ID: &str = "agent-workbench.command-center";

pub fn plugin_id() -> String {
    std::env::var("HERDR_PLUGIN_ID").unwrap_or_else(|_| PLUGIN_ID.to_string())
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Display state (the "last looked" snapshot, the GitHub cache). Never task state.
pub fn state_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("HERDR_PLUGIN_STATE_DIR") {
        return PathBuf::from(dir);
    }
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"));
    base.join("agent-workbench-cc")
}

pub fn config_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("HERDR_PLUGIN_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"));
    base.join("agent-workbench-cc")
}

/// The owner's Herdr configuration file.
pub fn herdr_config_path() -> PathBuf {
    match std::env::var_os("HERDR_CONFIG_PATH") {
        Some(path) => PathBuf::from(path),
        None => home().join(".config/herdr/config.toml"),
    }
}

/// A Herdr server not started from a login shell gives plugins a minimal PATH, so the usual tool
/// locations are appended (never prepended: what the owner's PATH finds first still wins).
pub fn extend_path() {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::env::split_paths(&current).collect();
    for extra in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"] {
        let extra = PathBuf::from(extra);
        if !paths.contains(&extra) {
            paths.push(extra);
        }
    }
    for extra in [home().join(".local/bin"), home().join(".cargo/bin")] {
        if !paths.contains(&extra) {
            paths.push(extra);
        }
    }
    if let Ok(joined) = std::env::join_paths(paths) {
        // SAFETY: called once at start-up, before any thread is spawned.
        unsafe { std::env::set_var("PATH", joined) };
    }
}

/// What Herdr passes to plugin commands in `HERDR_PLUGIN_CONTEXT_JSON`.
#[derive(Debug, Default, Deserialize)]
pub struct Context {
    pub focused_pane_id: Option<String>,
    pub focused_pane_cwd: Option<String>,
    pub workspace_cwd: Option<String>,
}

pub fn context() -> Context {
    std::env::var("HERDR_PLUGIN_CONTEXT_JSON")
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

/// The pane the owner was looking at: set by our own action (`WB_PANE`), else Herdr's context.
pub fn focused_pane() -> Option<String> {
    std::env::var("WB_PANE")
        .ok()
        .filter(|p| !p.is_empty())
        .or_else(|| context().focused_pane_id)
        .or_else(|| std::env::var("HERDR_PANE_ID").ok())
}

/// The directory whose repository is "the current project".
pub fn current_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("WB_CURRENT")
        && !dir.is_empty()
    {
        return Some(PathBuf::from(dir));
    }
    let ctx = context();
    ctx.focused_pane_cwd
        .or(ctx.workspace_cwd)
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
}

pub fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `config.toml` in the plugin's config directory. Every field is optional.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Opens a file at a line; `{path}` and `{line}` are replaced.
    pub editor: Vec<String>,
    /// Opens the branch diff against its base; `{base}` is replaced.
    pub diff: Vec<String>,
    /// Opens a URL; `{url}` is replaced.
    pub browser: Vec<String>,
    /// How long GitHub facts are reused before they are fetched again.
    pub github_refresh_seconds: u64,
    pub github_timeout_seconds: u64,
    pub keys: Keys,
    pub lanes: Lanes,
}

/// How `lane start` starts executors.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Lanes {
    /// Lanes that may run at once in one project; `lane start --over-limit` overrides it.
    pub max: usize,
    /// The branch of a new lane; `{issue}` and `{slug}` (from the issue title) are replaced.
    pub branch: String,
    /// Per agent kind, the arguments it starts with; `{prompt}` is the one-line first prompt.
    /// Merged over the built-in templates, so setting one runtime keeps the others.
    pub args: std::collections::BTreeMap<String, Vec<String>>,
}

impl Default for Lanes {
    fn default() -> Self {
        Lanes {
            max: 2,
            branch: "feat/{issue}-{slug}".into(),
            args: Default::default(),
        }
    }
}

impl Lanes {
    /// The launch arguments for `kind`: configured, else built in for the runtimes we know.
    pub fn args_for(&self, kind: &str) -> Option<Vec<String>> {
        if let Some(args) = self.args.get(kind) {
            return Some(args.clone());
        }
        let builtin: &[&str] = match kind {
            "claude" | "codex" => &["{prompt}"],
            "opencode" => &["--prompt", "{prompt}"],
            _ => return None,
        };
        Some(builtin.iter().map(|s| s.to_string()).collect())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Keys {
    pub overview: String,
    pub card: String,
}

impl Default for Keys {
    fn default() -> Self {
        Keys {
            overview: "prefix+i".into(),
            card: "prefix+u".into(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let browser = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        Config {
            editor: vec![
                "nvim".into(),
                "+{line}".into(),
                "--".into(),
                "{path}".into(),
            ],
            diff: vec!["nvim".into(), "-c".into(), "CodeDiff {base}...".into()],
            browser: vec![browser.into(), "{url}".into()],
            github_refresh_seconds: 60,
            github_timeout_seconds: 20,
            keys: Keys::default(),
            lanes: Lanes::default(),
        }
    }
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load_config() -> Result<Config, String> {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).map_err(|err| format!("{}: {err}", path.display())),
        Err(_) => Ok(Config::default()),
    }
}

/// Fills `{name}` placeholders in each argument of a command template.
pub fn fill(template: &[String], values: &[(&str, &str)]) -> Vec<String> {
    template
        .iter()
        .map(|arg| {
            values.iter().fold(arg.clone(), |acc, (name, value)| {
                acc.replace(&format!("{{{name}}}"), value)
            })
        })
        .collect()
}
