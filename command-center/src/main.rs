//! `workbench-cc`: the agent-workbench command center. Every popup key and Herdr action calls one of
//! these subcommands; the popups can do nothing the CLI cannot.

mod actions;
mod card;
mod env;
mod git;
mod github;
mod herdr;
mod lane;
mod model;
mod run;
mod setup;
mod tui;
mod words;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use actions::Expect;
use env::Config;
use model::Options;

#[derive(Parser)]
#[command(
    name = "workbench-cc",
    version,
    about = "The agent-workbench command center for Herdr"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Clone)]
struct View {
    /// Print machine-readable JSON instead of opening the popup.
    #[arg(long)]
    json: bool,
    /// Print one rendered frame of the popup, for example `100x40`.
    #[arg(long, value_name = "WIDTHxHEIGHT")]
    print: Option<String>,
    /// Use only cached GitHub facts.
    #[arg(long)]
    offline: bool,
}

#[derive(Args, Clone)]
struct Target {
    /// The agent's pane; defaults to the pane the owner was in.
    #[arg(long)]
    pane: Option<String>,
    /// Refuse unless the pane still holds this agent session.
    #[arg(long)]
    expect_session: Option<String>,
    /// Refuse unless the pane still works in this directory.
    #[arg(long)]
    expect_cwd: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Everything across projects: what needs you, what changed, every agent, the work.
    Overview {
        #[command(flatten)]
        view: View,
        /// Only the current project.
        #[arg(long)]
        this_project: bool,
        /// Directory whose repository is the current project.
        #[arg(long)]
        current: Option<PathBuf>,
        /// Remember this snapshot as "last looked" (the popup does this when it opens).
        #[arg(long)]
        mark_seen: bool,
    },
    /// The result card of one agent.
    Card {
        #[command(flatten)]
        view: View,
        #[command(flatten)]
        target: Target,
    },
    /// Open something for an agent.
    Open {
        #[command(subcommand)]
        what: Open,
    },
    /// Jump to the agent's pane.
    Focus {
        #[command(flatten)]
        target: Target,
    },
    /// Open one of the popups (used by the plugin's actions).
    Popup { which: PopupKind },
    /// Re-emit the sidebar tags.
    Refresh {
        /// Report every tag even if Herdr already shows it (after a restart).
        #[arg(long)]
        force: bool,
    },
    /// Refresh the tags and print the tab-bar counter.
    TabStatus,
    /// Run the command a split pane was opened with (internal).
    #[command(hide = true)]
    Exec,
    /// Propose key bindings, the tab-bar counter and the sidebar tag, and write them after you agree.
    Configure {
        /// Write without asking.
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        overview_key: Option<String>,
        #[arg(long)]
        card_key: Option<String>,
    },
    /// Remove everything `configure` wrote.
    Unconfigure,
    /// Check Herdr, gh, git and the editor.
    Doctor,
    /// Lanes: one executor per issue in its own worktree and Herdr workspace.
    Lane {
        #[command(subcommand)]
        what: Lane,
    },
}

#[derive(Subcommand)]
enum Lane {
    /// Start an approved lane: a worktree and workspace, the named agent with its brief, the claim.
    Start {
        /// The issue the lane works on.
        issue: u64,
        /// The agent kind Herdr starts (claude, codex, opencode, …).
        #[arg(long)]
        agent: String,
        /// The repository; defaults to the current directory.
        #[arg(long)]
        repo: Option<PathBuf>,
        /// The branch; defaults to `[lanes] branch` in the configuration.
        #[arg(long)]
        branch: Option<String>,
        /// A file with extra instructions for the executor, appended to its brief.
        #[arg(long)]
        brief: Option<PathBuf>,
        /// Start even when the project already runs `[lanes] max` lanes (the owner said so).
        #[arg(long)]
        over_limit: bool,
        #[arg(long)]
        json: bool,
    },
    /// Wait until the lane's agent stops, call the owner with a Herdr notification, report why.
    Watch {
        /// The lane: its agent name, its pane or its issue number.
        lane: String,
        /// Stop watching after this many seconds while the agent still works.
        #[arg(long, value_name = "SECONDS")]
        timeout: Option<u64>,
        /// How long an idle lane gets to start working.
        #[arg(long, value_name = "SECONDS", default_value_t = 120)]
        start_timeout: u64,
        #[arg(long)]
        json: bool,
    },
    /// Running lanes with their issue, agent, status and PR.
    List {
        /// Only the current project.
        #[arg(long)]
        this_project: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum Open {
    /// A file at a line in the editor, beside the agent's pane.
    File {
        #[command(flatten)]
        target: Target,
        /// The card's "look first" item (1-based).
        #[arg(long, conflicts_with = "path")]
        look_first: Option<usize>,
        #[arg(long, required_unless_present = "look_first")]
        path: Option<String>,
        #[arg(long, default_value_t = 1)]
        line: u64,
    },
    /// The branch diff against its base, beside the agent's pane.
    Diff {
        #[command(flatten)]
        target: Target,
    },
    /// The agent's pull request in the browser.
    Pr {
        #[command(flatten)]
        target: Target,
    },
    /// The app address from the card's "How to try".
    App {
        #[command(flatten)]
        target: Target,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum PopupKind {
    Overview,
    Card,
}

fn size(spec: &str) -> Result<(u16, u16), String> {
    let (w, h) = spec.split_once('x').ok_or("expected WIDTHxHEIGHT")?;
    Ok((
        w.parse().map_err(|_| "bad width")?,
        h.parse().map_err(|_| "bad height")?,
    ))
}

fn options(
    config: &Config,
    network: bool,
    current: Option<PathBuf>,
    only_current: bool,
) -> Options {
    Options {
        github: actions::github_settings(config, network),
        current: current.or_else(env::current_dir),
        only_current,
    }
}

fn pane_of(target: &Target) -> Result<String, String> {
    target
        .pane
        .clone()
        .or_else(env::focused_pane)
        .ok_or_else(|| "no pane given (use --pane)".to_string())
}

fn expect_of(target: &Target) -> Expect {
    Expect {
        session: target.expect_session.clone(),
        cwd: target.expect_cwd.clone(),
    }
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn refresh_tags(config: &Config, force: bool) -> Result<model::Snapshot, String> {
    let snapshot = model::build(&options(config, true, None, false))?;
    let lock_path = env::state_dir().join("refresh.lock");
    let _ = std::fs::create_dir_all(env::state_dir());
    let lock = std::fs::File::create(&lock_path).map_err(|e| e.to_string())?;
    if fs2::FileExt::try_lock_exclusive(&lock).is_err() {
        return Ok(snapshot);
    }
    let agents = herdr::agents()?;
    for a in snapshot
        .projects
        .iter()
        .flat_map(|p| &p.agents)
        .chain(&snapshot.loose)
    {
        let shown = agents
            .iter()
            .find(|h| h.pane_id == a.pane_id)
            .and_then(|h| h.tokens.get(herdr::TAG_TOKEN));
        if !force && shown == a.tag.as_ref() {
            continue;
        }
        if a.tag.is_none() && shown.is_none() && !force {
            continue;
        }
        herdr::report_tag(&a.pane_id, a.tag.as_deref())?;
    }
    Ok(snapshot)
}

fn main_inner() -> Result<ExitCode, String> {
    env::extend_path();
    let cli = Cli::parse();
    let config = || env::load_config();
    match cli.command {
        Command::Overview {
            view,
            this_project,
            current,
            mark_seen,
        } => {
            let config = config()?;
            let opts = options(&config, !view.offline, current, this_project);
            if !view.json && view.print.is_none() {
                return tui::overview(opts, &config).map(|_| ExitCode::SUCCESS);
            }
            let snapshot = model::build(&opts)?;
            if mark_seen {
                model::mark_seen(&snapshot)?;
            }
            match view.print {
                Some(spec) => {
                    let (w, h) = size(&spec)?;
                    print!("{}", tui::print_overview(&snapshot, this_project, w, h));
                }
                None => print_json(&snapshot)?,
            }
        }
        Command::Card { view, target } => {
            let config = config()?;
            let opts = options(&config, !view.offline, None, false);
            let card = actions::card_for(&pane_of(&target)?, &expect_of(&target), &opts)?;
            match (&view.print, view.json) {
                (Some(spec), _) => {
                    let (w, h) = size(spec)?;
                    print!("{}", tui::print_card(&card, w, h));
                }
                (None, true) => print_json(&card)?,
                (None, false) => tui::card(card, &config)?,
            }
        }
        Command::Open { what } => {
            let config = config()?;
            let opts = options(&config, true, None, false);
            match what {
                Open::File {
                    target,
                    look_first: Some(n),
                    ..
                } => {
                    let card = actions::card_for(&pane_of(&target)?, &expect_of(&target), &opts)?;
                    actions::open_look_first(&card, n, &config)?;
                }
                Open::File {
                    target, path, line, ..
                } => {
                    let agent = actions::agent_in(&pane_of(&target)?, &expect_of(&target), &opts)?;
                    actions::open_file(&agent, path.as_deref().unwrap_or(""), line, &config)?;
                }
                Open::Diff { target } => {
                    let agent = actions::agent_in(&pane_of(&target)?, &expect_of(&target), &opts)?;
                    actions::open_diff(&agent, &config)?;
                }
                Open::Pr { target } => {
                    let agent = actions::agent_in(&pane_of(&target)?, &expect_of(&target), &opts)?;
                    actions::open_pr(&agent)?;
                }
                Open::App { target } => {
                    let card = actions::card_for(&pane_of(&target)?, &expect_of(&target), &opts)?;
                    actions::open_app(&card, &config)?;
                }
            }
        }
        Command::Focus { target } => {
            let pane = pane_of(&target)?;
            actions::verify(&pane, &expect_of(&target))?;
            herdr::focus_agent(&pane)?;
        }
        Command::Popup { which } => {
            let ctx = env::context();
            let mut vars = vec![];
            if let Some(pane) = env::focused_pane() {
                vars.push(("WB_PANE", pane));
            }
            if let Some(cwd) = ctx.focused_pane_cwd.or(ctx.workspace_cwd) {
                vars.push(("WB_CURRENT", cwd));
            }
            let entry = match which {
                PopupKind::Overview => "overview",
                PopupKind::Card => "card",
            };
            herdr::open_popup(entry, &vars)?;
        }
        Command::Refresh { force } => {
            refresh_tags(&config()?, force)?;
        }
        Command::TabStatus => {
            let snapshot = refresh_tags(&config()?, false)?;
            println!("{}", snapshot.counter.text);
        }
        Command::Exec => actions::exec_from_env()?,
        Command::Configure {
            yes,
            overview_key,
            card_key,
        } => {
            let mut config = config()?;
            if let Some(key) = overview_key {
                config.keys.overview = key;
            }
            if let Some(key) = card_key {
                config.keys.card = key;
            }
            setup::configure(&config, yes)?;
        }
        Command::Unconfigure => setup::unconfigure()?,
        Command::Lane { what } => {
            let config = config()?;
            match what {
                Lane::Start {
                    issue,
                    agent,
                    repo,
                    branch,
                    brief,
                    over_limit,
                    json,
                } => {
                    let started = lane::start(
                        lane::StartArgs {
                            issue,
                            agent,
                            repo,
                            branch,
                            brief,
                            over_limit,
                        },
                        &config,
                    )?;
                    if json {
                        print_json(&started)?;
                    } else {
                        println!(
                            "lane {} {}: #{} in {} ({}), workspace {}{}",
                            started.lane,
                            started.status,
                            started.issue,
                            started.worktree,
                            started.branch,
                            started.workspace_id,
                            match &started.claim_error {
                                Some(err) => format!("; not claimed: {err}"),
                                None => String::new(),
                            }
                        );
                    }
                }
                Lane::Watch {
                    lane: which,
                    timeout,
                    start_timeout,
                    json,
                } => {
                    let opts = options(&config, true, None, false);
                    let watched = lane::watch(
                        lane::WatchArgs {
                            lane: which,
                            timeout,
                            start_timeout,
                        },
                        &config,
                        &opts,
                    )?;
                    if json {
                        print_json(&watched)?;
                    } else {
                        println!("lane {} {}: {}", watched.lane, watched.stop, watched.status);
                    }
                }
                Lane::List { this_project, json } => {
                    let opts = options(&config, true, None, this_project);
                    let list = lane::list(&config, &opts)?;
                    if json {
                        print_json(&list)?;
                    } else {
                        lane::print_list(&list);
                    }
                }
            }
        }
        Command::Doctor => {
            if !setup::doctor(config()) {
                return Ok(ExitCode::FAILURE);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    match main_inner() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("workbench-cc: {err}");
            ExitCode::FAILURE
        }
    }
}
