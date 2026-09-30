//! `workbench`: installs a harness source into a project and keeps it in step.

mod agents_md;
mod config;
mod files;
mod guard;
mod init;
mod lock;
mod skills;
mod source;
mod sync;
mod update;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "workbench", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install a harness source into the current project.
    Init {
        /// Harness source as `<repository>@<ref>`: a local path, a git URL or `owner/repo` on
        /// GitHub; without `@<ref>` the source's default branch is used.
        #[arg(long, value_name = "REPO@REF")]
        from: String,
    },
    /// Regenerate derived files: skill links, Codex and OpenCode files, the registry and the
    /// managed block of AGENTS.md. Reports stale files; never fails because of them.
    Sync,
    /// Bring a new harness version in on a new branch, merging project edits three-way.
    Update {
        /// Ref to update to; defaults to the newest release tag, or the locked branch's head.
        #[arg(long, value_name = "REF")]
        to: Option<String>,
    },
    /// Print the resolved configuration: the built-in default overlaid by workbench.toml.
    Config,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let project = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => return fail(e.into()),
    };
    let result = run(cli.command, project);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(e),
    }
}

fn run(command: Command, project: PathBuf) -> anyhow::Result<()> {
    match command {
        Command::Init { from } => init::run(&project, &from),
        Command::Sync => sync::run(&project),
        Command::Update { to } => update::run(&project, to.as_deref()),
        Command::Config => config::print(&project),
    }
}

fn fail(error: anyhow::Error) -> ExitCode {
    eprintln!("error: {error:#}");
    ExitCode::FAILURE
}
