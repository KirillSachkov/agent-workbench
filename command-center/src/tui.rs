//! The Overview and result-card popups: a thin terminal view over the same model the JSON commands
//! print. Every key calls the same functions as a CLI subcommand.

use std::sync::mpsc;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

use crate::actions::{self, Expect};
use crate::card::{self, Card};
use crate::env::Config;
use crate::model::{self, AgentView, Options, Snapshot};

fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Colours by meaning, not by label.
fn tone(phrase: &str) -> Style {
    let colour = match phrase {
        "Waits for you" | "CI failed" => Color::Red,
        "Done, not seen yet"
        | "Finished"
        | "Ready for your acceptance"
        | "CI passed"
        | "PR merged" => Color::Green,
        "Working" | "CI running" | "PR opened" => Color::Yellow,
        _ => return Style::default(),
    };
    Style::default().fg(colour)
}

fn agent_line(a: &AgentView, selected: bool) -> Line<'static> {
    let mut spans = vec![
        Span::raw(if selected { "› " } else { "  " }),
        Span::styled(a.agent.clone(), bold()),
    ];
    if let Some(branch) = &a.branch {
        spans.push(Span::raw(format!("  {branch}")));
    }
    let mut chain = vec![];
    if let Some(task) = &a.task {
        chain.push(format!(
            "#{}{}",
            task.number,
            if task.inferred { " (from branch)" } else { "" }
        ));
    }
    if let Some(pr) = &a.pr {
        chain.push(format!(
            "PR #{}{}",
            pr.number,
            if pr.state == "merged" { " merged" } else { "" }
        ));
        if pr.state != "merged" {
            chain.push(pr.ci_phrase.clone());
        }
    }
    if !chain.is_empty() {
        spans.push(Span::raw(format!("  {}", chain.join(" → "))));
    }
    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        a.status_phrase.clone(),
        tone(&a.status_phrase),
    ));
    let line = Line::from(spans);
    if selected {
        line.style(Style::default().add_modifier(Modifier::REVERSED))
    } else {
        line
    }
}

/// The Overview as lines, plus which line shows which agent.
pub fn overview_lines(
    s: &Snapshot,
    selected: usize,
) -> (Vec<Line<'static>>, Vec<(usize, AgentView)>) {
    let mut lines: Vec<Line<'static>> = vec![];
    let mut agents = vec![];
    let mut push_agent = |lines: &mut Vec<Line<'static>>, a: &AgentView| {
        let is_selected = agents.len() == selected;
        agents.push((lines.len(), a.clone()));
        lines.push(agent_line(a, is_selected));
    };

    lines.push(Line::from(Span::styled(
        format!("Needs you ({})", s.needs_you.len()),
        bold(),
    )));
    if s.needs_you.is_empty() {
        lines.push(Line::styled("  nothing waits for you", dim()));
    }
    for item in &s.needs_you {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(item.phrase.clone(), tone(&item.phrase)),
            Span::raw(format!("  {}", item.text)),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled("Since you last looked", bold())));
    if s.since_last_looked.is_empty() {
        lines.push(Line::styled("  nothing new", dim()));
    }
    for item in &s.since_last_looked {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(item.phrase.clone(), tone(&item.phrase)),
            Span::raw(format!("  {}", item.text)),
        ]));
    }
    for p in &s.projects {
        lines.push(Line::raw(""));
        let mut head = vec![Span::styled(p.name.clone(), bold().fg(Color::Cyan))];
        head.push(Span::styled(
            format!(
                "  {}",
                p.github.clone().unwrap_or_else(|| "git only".into())
            ),
            dim(),
        ));
        if p.stale {
            let note = if p.work.is_some() {
                "  GitHub unreachable, last known facts"
            } else {
                "  GitHub unreachable"
            };
            head.push(Span::styled(note, Style::default().fg(Color::Yellow)));
        }
        lines.push(Line::from(head));
        if p.agents.is_empty() {
            lines.push(Line::styled("  no agents", dim()));
        }
        for a in &p.agents {
            push_agent(&mut lines, a);
        }
        if let Some(w) = &p.work {
            for spec in &w.specs {
                lines.push(Line::raw(format!(
                    "  Spec #{} {} — {}",
                    spec.number, spec.title, spec.progress
                )));
            }
            for item in &w.ready {
                lines.push(Line::from(vec![
                    Span::styled("  Ready now ", Style::default().fg(Color::Green)),
                    Span::raw(format!("#{} {}", item.number, item.title)),
                ]));
            }
            for item in &w.in_flight {
                let pr = item
                    .pr
                    .as_ref()
                    .map(|pr| format!(" → PR #{} {}", pr.number, pr.ci_phrase))
                    .unwrap_or_default();
                lines.push(Line::from(vec![
                    Span::styled("  In flight ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!(
                        "#{} {} ({}){pr}",
                        item.number,
                        item.title,
                        item.assignees.join(", ")
                    )),
                ]));
            }
        }
    }
    if !s.loose.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled("Folders without git", bold())));
        for a in &s.loose {
            push_agent(&mut lines, a);
            let last = lines.len() - 1;
            lines[last]
                .spans
                .push(Span::styled(format!("  {}", a.cwd), dim()));
        }
    }
    (lines, agents)
}

pub fn card_lines(c: &Card) -> Vec<Line<'static>> {
    let a = &c.agent;
    let mut lines = vec![];
    let place = [a.project.clone(), a.branch.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", a.agent), bold()),
        Span::raw(place),
        Span::raw("  "),
        Span::styled(a.status_phrase.clone(), tone(&a.status_phrase)),
    ]));
    if let Some(pr) = &a.pr {
        lines.push(Line::raw(format!("PR #{} {}", pr.number, pr.title)));
    }
    if let Some(task) = &a.task {
        lines.push(Line::raw(format!(
            "Task #{}{}",
            task.number,
            if task.inferred {
                " (from the branch name)"
            } else {
                ""
            }
        )));
    }
    let f = &c.facts;
    let mut facts = vec![];
    if let Some(ci) = &f.ci {
        facts.push(ci.clone());
    }
    if let Some(review) = &f.review {
        facts.push(review.clone());
    }
    if let Some(sha) = &f.head_sha {
        facts.push(format!("head {}", &sha[..sha.len().min(7)]));
    }
    facts.push(format!(
        "{} files +{} −{}",
        f.files_changed, f.additions, f.deletions
    ));
    facts.push(if f.tests_changed {
        "tests changed".into()
    } else {
        "no test changes".into()
    });
    if f.ci_config_changed {
        facts.push("CI configuration changed".into());
    }
    lines.push(Line::raw(facts.join(" · ")));
    if let Some(wt) = &f.worktree {
        lines.push(Line::styled(wt.clone(), dim()));
    }
    if c.stale {
        lines.push(Line::styled(
            "GitHub unreachable: last known facts",
            Style::default().fg(Color::Yellow),
        ));
    }
    let section = |lines: &mut Vec<Line<'static>>, title: &str, text: &str| {
        lines.push(Line::raw(""));
        lines.push(Line::styled(title.to_string(), bold()));
        for l in text.lines() {
            lines.push(Line::raw(format!("  {l}")));
        }
    };
    match c.shape.as_str() {
        "result-card" => {
            if let Some(t) = &c.result {
                section(&mut lines, "Result", t);
            }
            if let Some(t) = &c.needs_you {
                section(&mut lines, "Needs you", t);
            }
            if !c.look_first.is_empty() {
                lines.push(Line::raw(""));
                lines.push(Line::styled("Look first", bold()));
                for (i, lf) in c.look_first.iter().enumerate() {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} ", i + 1), Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{}:{} — {}", lf.path, lf.line, lf.reason)),
                    ]));
                }
            }
            if let Some(t) = &c.how_to_try {
                section(&mut lines, "How to try", t);
            }
            for s in &c.other_sections {
                section(&mut lines, &s.title, &s.text);
            }
        }
        "plain" => section(&mut lines, "Pull request", c.body.as_deref().unwrap_or("")),
        _ => {
            lines.push(Line::raw(""));
            lines.push(Line::styled("No pull request yet. Changed files:", bold()));
            for file in &c.files {
                lines.push(Line::raw(format!(
                    "  {} +{} −{}",
                    file.path, file.additions, file.deletions
                )));
            }
        }
    }
    lines
}

const OVERVIEW_KEYS: &str =
    "↑↓ select · enter card · f go to agent · o PR · p this project · r refresh · q close";
const CARD_KEYS: &str = "1-9 file · d diff · o PR · a app · f go to agent · ↑↓ scroll · q back";

fn draw(frame: &mut Frame, title: &str, lines: &[Line<'static>], scroll: usize, footer: &str) {
    let [top, body, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    frame.render_widget(Paragraph::new(Line::styled(title.to_string(), bold())), top);
    frame.render_widget(
        Paragraph::new(lines.to_vec()).scroll((scroll as u16, 0)),
        body,
    );
    frame.render_widget(
        Paragraph::new(Line::styled(footer.to_string(), dim())),
        bottom,
    );
}

fn overview_title(s: &Snapshot, only_current: bool) -> String {
    format!(
        "Command center — {}",
        if only_current {
            "this project"
        } else {
            "all projects"
        }
    ) + &if s.projects.iter().any(|p| p.stale) {
        "  (GitHub unreachable)".to_string()
    } else {
        String::new()
    }
}

/// Renders one frame to text: the TUI's snapshot tests read this.
pub fn print(width: u16, height: u16, render: impl FnOnce(&mut Frame)) -> String {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height))
        .expect("test backend");
    terminal.draw(render).expect("draw");
    let buffer = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..height {
        let row: String = (0..width)
            .map(|x| buffer[(x, y)].symbol().to_string())
            .collect();
        out.push_str(row.trim_end());
        out.push('\n');
    }
    out
}

pub fn print_overview(s: &Snapshot, only_current: bool, width: u16, height: u16) -> String {
    let (lines, _) = overview_lines(s, 0);
    print(width, height, |f| {
        draw(
            f,
            &overview_title(s, only_current),
            &lines,
            0,
            OVERVIEW_KEYS,
        )
    })
}

pub fn print_card(c: &Card, width: u16, height: u16) -> String {
    print(width, height, |f| {
        draw(f, "Result card", &card_lines(c), 0, CARD_KEYS)
    })
}

enum Next {
    Stay,
    Close,
}

fn expect(a: &AgentView) -> Expect {
    Expect {
        session: a.session.clone(),
        cwd: Some(a.cwd.clone()),
    }
}

/// Handles a card key; `Close` means the action is done and the popup should close.
fn card_key(code: KeyCode, card: &Card, config: &Config, message: &mut String) -> Next {
    let a = &card.agent;
    let result = match code {
        KeyCode::Char(c @ '1'..='9') => actions::verify(&a.pane_id, &expect(a))
            .and_then(|_| actions::open_look_first(card, c as usize - '0' as usize, config))
            .map(|_| Next::Close),
        KeyCode::Char('d') => actions::verify(&a.pane_id, &expect(a))
            .and_then(|_| actions::open_diff(a, config))
            .map(|_| Next::Close),
        KeyCode::Char('f') => actions::verify(&a.pane_id, &expect(a))
            .and_then(|_| actions::focus(a))
            .map(|_| Next::Close),
        KeyCode::Char('o') => actions::open_pr(a).map(|_| Next::Stay),
        KeyCode::Char('a') => actions::open_app(card, config).map(|_| Next::Stay),
        _ => Ok(Next::Stay),
    };
    result.unwrap_or_else(|err| {
        *message = err;
        Next::Stay
    })
}

fn scroll_key(code: KeyCode, scroll: &mut usize, max: usize) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => *scroll = (*scroll + 1).min(max),
        KeyCode::Up | KeyCode::Char('k') => *scroll = scroll.saturating_sub(1),
        KeyCode::PageDown | KeyCode::Char(' ') => *scroll = (*scroll + 10).min(max),
        KeyCode::PageUp => *scroll = scroll.saturating_sub(10),
        _ => {}
    }
}

fn key() -> Option<KeyCode> {
    if !event::poll(Duration::from_millis(200)).ok()? {
        return None;
    }
    match event::read().ok()? {
        Event::Key(k) if k.kind == KeyEventKind::Press => Some(k.code),
        _ => None,
    }
}

fn run_card(terminal: &mut DefaultTerminal, card: &Card, config: &Config) -> Result<Next, String> {
    let lines = card_lines(card);
    let mut scroll = 0;
    let mut message = String::new();
    loop {
        let footer = if message.is_empty() {
            CARD_KEYS.to_string()
        } else {
            message.clone()
        };
        terminal
            .draw(|f| draw(f, "Result card", &lines, scroll, &footer))
            .map_err(|e| e.to_string())?;
        let Some(code) = key() else { continue };
        message.clear();
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Next::Stay),
            _ => {}
        }
        scroll_key(code, &mut scroll, lines.len().saturating_sub(1));
        if let Next::Close = card_key(code, card, config, &mut message) {
            return Ok(Next::Close);
        }
    }
}

pub fn card(card: Card, config: &Config) -> Result<(), String> {
    let mut terminal = ratatui::init();
    let result = run_card(&mut terminal, &card, config);
    ratatui::restore();
    result.map(|_| ())
}

pub fn overview(mut opts: Options, config: &Config) -> Result<(), String> {
    opts.github.network = false;
    let mut snapshot = model::build(&opts)?;
    let mut terminal = ratatui::init();
    let result = run_overview(&mut terminal, &mut snapshot, &mut opts, config);
    ratatui::restore();
    result
}

fn refresh(opts: &Options) -> mpsc::Receiver<Result<Snapshot, String>> {
    let (tx, rx) = mpsc::channel();
    let opts = Options {
        github: crate::github::Settings {
            network: true,
            ..opts.github.clone()
        },
        current: opts.current.clone(),
        only_current: opts.only_current,
    };
    std::thread::spawn(move || {
        let _ = tx.send(model::build(&opts));
    });
    rx
}

fn run_overview(
    terminal: &mut DefaultTerminal,
    snapshot: &mut Snapshot,
    opts: &mut Options,
    config: &Config,
) -> Result<(), String> {
    let mut pending = Some(refresh(opts));
    let mut marked = false;
    let mut selected = 0usize;
    let mut scroll = 0usize;
    let mut message = String::new();
    let result = loop {
        if let Some(rx) = &pending
            && let Ok(fresh) = rx.try_recv()
        {
            pending = None;
            match fresh {
                Ok(fresh) => {
                    *snapshot = fresh;
                    if !marked {
                        let _ = model::mark_seen(snapshot);
                        marked = true;
                    }
                }
                Err(err) => message = err,
            }
        }
        let (lines, agents) = overview_lines(snapshot, selected);
        if let Some((line, _)) = agents.get(selected) {
            let height = terminal
                .size()
                .map(|s| s.height.saturating_sub(2) as usize)
                .unwrap_or(20)
                .max(1);
            if *line < scroll {
                scroll = *line;
            } else if *line >= scroll + height {
                scroll = line + 1 - height;
            }
        }
        let footer = if !message.is_empty() {
            message.clone()
        } else if pending.is_some() {
            format!("refreshing…  {OVERVIEW_KEYS}")
        } else {
            OVERVIEW_KEYS.to_string()
        };
        let title = overview_title(snapshot, opts.only_current);
        terminal
            .draw(|f| draw(f, &title, &lines, scroll, &footer))
            .map_err(|e| e.to_string())?;
        let Some(code) = key() else { continue };
        message.clear();
        let current = agents.get(selected).map(|(_, a)| a.clone());
        match code {
            KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1).min(agents.len().saturating_sub(1))
            }
            KeyCode::Up | KeyCode::Char('k') => selected = selected.saturating_sub(1),
            KeyCode::Char('r') => pending = Some(refresh(opts)),
            KeyCode::Char('p') => {
                opts.only_current = !opts.only_current;
                selected = 0;
                pending = Some(refresh(opts));
            }
            KeyCode::Char('o') => {
                if let Some(a) = &current
                    && let Err(err) = actions::open_pr(a)
                {
                    message = err;
                }
            }
            KeyCode::Char('f') => {
                if let Some(a) = &current {
                    match actions::verify(&a.pane_id, &expect(a)).and_then(|_| actions::focus(a)) {
                        Ok(()) => break Ok(()),
                        Err(err) => message = err,
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(a) = current {
                    terminal
                        .draw(|f| draw(f, "Result card", &[Line::raw("Loading the card…")], 0, ""))
                        .map_err(|e| e.to_string())?;
                    let card = card::build(
                        a,
                        &crate::github::Settings {
                            network: true,
                            ..opts.github.clone()
                        },
                    );
                    if let Next::Close = run_card(terminal, &card, config)? {
                        break Ok(());
                    }
                }
            }
            _ => {}
        }
    };
    if !marked {
        let _ = model::mark_seen(snapshot);
    }
    result
}
