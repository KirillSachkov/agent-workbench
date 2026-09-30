//! Running external programs (herdr, git, gh, the editor) with captured output and a timeout.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use wait_timeout::ChildExt;

pub struct Output {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    fn failed(message: String) -> Self {
        Output {
            ok: false,
            stdout: String::new(),
            stderr: message,
        }
    }

    /// The first line of stderr, or a generic note, for error messages.
    pub fn error_line(&self) -> String {
        let line = self
            .stderr
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim();
        if line.is_empty() {
            "command failed".to_string()
        } else {
            line.to_string()
        }
    }
}

/// Runs `program` with `args`, never through a shell.
pub fn run(program: &str, args: &[&str], cwd: Option<&Path>, timeout: Option<Duration>) -> Output {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => return Output::failed(format!("cannot run {program}: {err}")),
    };
    let mut out = child.stdout.take().expect("piped stdout");
    let mut err = child.stderr.take().expect("piped stderr");
    let out_reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = out.read_to_end(&mut buf);
        buf
    });
    let err_reader = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = err.read_to_end(&mut buf);
        buf
    });
    let status = match timeout {
        Some(limit) => match child.wait_timeout(limit) {
            Ok(Some(status)) => Some(status),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                None
            }
        },
        None => child.wait().ok(),
    };
    let stdout = String::from_utf8_lossy(&out_reader.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&err_reader.join().unwrap_or_default()).into_owned();
    match status {
        Some(status) => Output {
            ok: status.success(),
            stdout,
            stderr,
        },
        None => Output::failed(format!("{program} timed out")),
    }
}

/// Starts `argv` detached from our terminal and does not wait for it.
pub fn spawn_detached(argv: &[String]) -> Result<(), String> {
    let (program, args) = argv.split_first().ok_or("empty command")?;
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("cannot run {program}: {err}"))
}

/// Finds `program` on PATH (or returns it when it is a path that exists).
pub fn which(program: &str) -> Option<std::path::PathBuf> {
    if program.contains('/') {
        let path = std::path::PathBuf::from(program);
        return path.is_file().then_some(path);
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(program))
        .find(|p| p.is_file())
}
