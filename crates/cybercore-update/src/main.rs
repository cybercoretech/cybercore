use cybercore_updater::{Engine, Event, Inspection, Manifest, Task, TaskStatus, ThemeReport};
use crossterm::{
    event::{self, Event as Input, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{
    collections::HashMap,
    env,
    io::{self, stdout},
    path::PathBuf,
    process::Command,
    time::Duration,
};

#[derive(Clone, Copy)]
enum Action {
    Apply,
    Repair,
    Restart,
    Rollback,
    ThemeSync,
}

impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::Apply => "apply selected tasks",
            Self::Repair => "repair/retry selected tasks",
            Self::Restart => "restart selected services",
            Self::Rollback => "rollback selected binaries",
            Self::ThemeSync => "synchronize theme catalogs",
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env::var("CYBERCORE_ROOT").unwrap_or_else(|_| {
        "/home/raven/Devspace/Cybercore/framework/cybercore".into()
    }));
    let engine = Engine::new(
        Manifest::load(root.join("tools/cybercore-consumers.json"))?,
        &root,
    );

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
    let result = run(&mut terminal, &engine, engine.tasks());
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    engine: &Engine,
    tasks: Vec<Task>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut selected = vec![true; tasks.len()];
    let mut cursor = 0usize;
    let mut task_offset = 0u16;
    let mut log_offset = 0u16;
    let mut statuses = HashMap::<String, String>::new();
    let mut inspections = HashMap::<String, Inspection>::new();
    let mut logs = vec![
        "↑/↓ move  Space toggle  A all  N none  I inspect  T themes  G GitHub  O open PR  H history".to_string(),
        "P plan  Enter apply  R repair  S restart  B rollback  U sync themes  Q quit".to_string(),
    ];
    let mut pending: Option<Action> = None;
    let mut first_github_url = None;
    if let Ok(history) = engine.history() {
        logs.push(format!("History loaded: {} recorded operations.", history.len()));
    }
    refresh_inspection(engine, &mut inspections, &mut statuses, &mut logs);

    loop {
        terminal.draw(|frame| {
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(9)])
                .split(frame.area());
            let ready = statuses.values().filter(|s| s.as_str() == "READY").count();
            let attention = statuses.len().saturating_sub(ready);
            let selected_count = selected.iter().filter(|value| **value).count();
            let title = if let Some(action) = pending {
                format!("CYBERCORE // UPDATE CONTROL  |  CONFIRM: {} [Y/N]", action.label())
            } else {
                format!(
                    "CYBERCORE // UPDATE CONTROL  |  {} ready  {} attention  {} selected",
                    ready, attention, selected_count
                )
            };
            frame.render_widget(
                Paragraph::new(title)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(if pending.is_some() {
                        Color::Yellow
                    } else {
                        Color::Cyan
                    }).add_modifier(Modifier::BOLD)),
                vertical[0],
            );

            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
                .split(vertical[1]);
            let task_lines = tasks
                .iter()
                .enumerate()
                .map(|(index, task)| {
                    let marker = if selected[index] { "[x]" } else { "[ ]" };
                    let pointer = if index == cursor { ">" } else { " " };
                    let status = statuses.get(&task.name).map(String::as_str).unwrap_or("UNKNOWN");
                    Line::from(vec![
                        Span::styled(pointer, if index == cursor {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        }),
                        Span::styled(
                            marker,
                            if selected[index] {
                                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        ),
                        Span::raw(" "),
                        Span::styled(
                            format!("{:<27}", task.name),
                            Style::default().fg(if index == cursor {
                                Color::White
                            } else {
                                Color::Gray
                            }),
                        ),
                        Span::styled(status, status_style(status)),
                    ])
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(task_lines)
                    .block(Block::default().title("Fleet tasks").borders(Borders::ALL))
                    .scroll((task_offset, 0)),
                horizontal[0],
            );

            let detail = tasks
                .get(cursor)
                .and_then(|task| inspections.get(&task.name))
                .map(task_detail)
                .unwrap_or_else(|| "No task selected.".into());
            frame.render_widget(
                Paragraph::new(detail)
                    .block(Block::default().title("Task details").borders(Borders::ALL))
                    .wrap(ratatui::widgets::Wrap { trim: false }),
                horizontal[1],
            );

            frame.render_widget(
                Paragraph::new(logs.iter().map(|log| log_line(log)).collect::<Vec<_>>())
                    .block(Block::default().title("Activity / logs  ([ ] scroll, Home/End jump)").borders(Borders::ALL))
                    .scroll((log_offset, 0)),
                vertical[2],
            );
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        if let Input::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') if pending.is_none() => break,
                KeyCode::Esc => pending = None,
                KeyCode::Char('y') if pending.is_some() => {
                    let action = pending.take().expect("pending action");
                    execute_action(
                        engine, action, &tasks, &selected, &mut statuses, &mut logs,
                    );
                }
                KeyCode::Char('n') if pending.is_some() => {
                    pending = None;
                    logs.push("Action cancelled.".into());
                }
                KeyCode::Up if pending.is_none() => {
                    cursor = cursor.saturating_sub(1);
                    task_offset = task_offset.saturating_sub(1);
                }
                KeyCode::Down if pending.is_none() => {
                    cursor = (cursor + 1).min(tasks.len().saturating_sub(1));
                    task_offset = task_offset.saturating_add(1);
                }
                KeyCode::PageUp if pending.is_none() => task_offset = task_offset.saturating_sub(6),
                KeyCode::PageDown if pending.is_none() => task_offset = task_offset.saturating_add(6),
                KeyCode::Char('[') => log_offset = log_offset.saturating_sub(4),
                KeyCode::Char(']') => log_offset = log_offset.saturating_add(4),
                KeyCode::Home => {
                    cursor = 0;
                    task_offset = 0;
                    log_offset = 0;
                }
                KeyCode::End => {
                    cursor = tasks.len().saturating_sub(1);
                    task_offset = cursor.saturating_sub(4) as u16;
                    log_offset = logs.len().saturating_sub(7) as u16;
                }
                KeyCode::Char(' ') if pending.is_none() => selected[cursor] = !selected[cursor],
                KeyCode::Char('a') if pending.is_none() => selected.fill(true),
                KeyCode::Char('n') if pending.is_none() => selected.fill(false),
                KeyCode::Char('i') if pending.is_none() => {
                    refresh_inspection(engine, &mut inspections, &mut statuses, &mut logs);
                }
                KeyCode::Char('g') if pending.is_none() => {
                    github_report(engine, &mut logs, &mut first_github_url)
                }
                KeyCode::Char('t') if pending.is_none() => theme_report(engine, &mut logs),
                KeyCode::Char('o') if pending.is_none() => open_github_pr(&first_github_url, &mut logs),
                KeyCode::Char('h') if pending.is_none() => history_report(engine, &mut logs),
                KeyCode::Char('?') if pending.is_none() => logs.push(
                    "HELP: I inspect | T themes | G GitHub | O open PR | H history | P plan | Enter apply | R repair | S restart | B rollback | U sync themes | Y/N confirm"
                        .into(),
                ),
                KeyCode::Char('p') if pending.is_none() => {
                    plan_tasks(engine, &tasks, &selected, &mut statuses, &mut logs);
                }

                KeyCode::Enter if pending.is_none() => pending = Some(Action::Apply),
                KeyCode::Char('r') if pending.is_none() => pending = Some(Action::Repair),
                KeyCode::Char('s') if pending.is_none() => pending = Some(Action::Restart),
                KeyCode::Char('b') if pending.is_none() => pending = Some(Action::Rollback),
                KeyCode::Char('u') if pending.is_none() => pending = Some(Action::ThemeSync),
                _ => {}
            }
            trim_logs(&mut logs);
            if !matches!(key.code, KeyCode::Char('[') | KeyCode::PageUp | KeyCode::Home) {
                log_offset = logs.len().saturating_sub(7) as u16;
            }
        }
    }
    Ok(())
}

fn github_report(engine: &Engine, logs: &mut Vec<String>, first_url: &mut Option<String>) {
    logs.push("Collecting read-only GitHub repository and PR status…".into());
    *first_url = None;
    for repo in engine.github_report() {
        let dirty = if repo.dirty { "DIRTY" } else { "CLEAN" };
        logs.push(format!("GITHUB {} [{}] branch={}", repo.repository, dirty, repo.branch));
        if let Some(error) = repo.error {
            logs.push(format!("GITHUB {} error: {}", repo.repository, error));
        }

        if repo.pull_requests.is_empty() {
            logs.push(format!("GITHUB {} open PRs: none", repo.repository));
        } else {
            logs.push(format!(
                "GITHUB {} open PRs: {}",
                repo.repository,
                repo.pull_requests.len()
            ));
            for pull_request in repo.pull_requests {
                if first_url.is_none() {
                    *first_url = Some(pull_request.url.clone());
                }
                let draft = if pull_request.draft { "[DRAFT] " } else { "" };
                let reviewers = if pull_request.review_requests.is_empty() {
                    "none".into()
                } else {
                    pull_request.review_requests.join(", ")
                };
                logs.push(format!(
                    "#{number} {draft}{title} by {author} reviewers={reviewers} — {url}",
                    number = pull_request.number,
                    title = pull_request.title,
                    author = pull_request.author,
                    url = pull_request.url
                ));
            }
        }
    }
}

fn theme_report(engine: &Engine, logs: &mut Vec<String>) {
    let report = engine.theme_report();
    append_theme_report(&report, logs);
}

fn append_theme_report(report: &ThemeReport, logs: &mut Vec<String>) {
    logs.push(format!(
        "THEMES canonical={} builtins={} target={} managed-local={}",
        report.canonical_sources,
        report.framework_builtins,
        report.target_total,
        report.managed_local
    ));
    logs.push(format!(
        "THEMES missing-managed={} extra-managed={}",
        report.missing_managed.len(),
        report.extra_managed.len()
    ));
    if !report.missing_managed.is_empty() {
        logs.push(format!("THEMES missing: {}", report.missing_managed.join(", ")));
    }
    if !report.extra_managed.is_empty() {
        logs.push(format!("THEMES extra: {}", report.extra_managed.join(", ")));
    }
}

fn open_github_pr(url: &Option<String>, logs: &mut Vec<String>) {
    let Some(url) = url else {
        logs.push("No open PR URL is available; press G first.".into());
        return;
    };
    match Command::new("xdg-open").arg(url).spawn() {
        Ok(_) => logs.push(format!("Opened PR in browser: {url}")),
        Err(error) => logs.push(format!("Could not open PR in browser: {error}")),
    }
}

fn history_report(engine: &Engine, logs: &mut Vec<String>) {
    match engine.history() {
        Ok(history) if history.is_empty() => logs.push("HISTORY no recorded operations.".into()),
        Ok(history) => {
            logs.push(format!("HISTORY {} recorded operations:", history.len()));
            for entry in history.iter().rev().take(10) {
                let summary = entry
                    .results
                    .iter()
                    .map(|(name, status)| format!("{name}={status}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                logs.push(format!("HISTORY {} — {}", entry.operation, summary));
            }
        }
        Err(error) => logs.push(format!("HISTORY failed to load: {error}")),
    }
}

fn task_detail(item: &Inspection) -> String {
    let service = item.service.as_deref().unwrap_or("none");
    let binary = item
        .installed_binary
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "sync task".into());
    let diagnostics = if item.diagnostics.is_empty() {
        "none".into()
    } else {
        item.diagnostics.join("\n")
    };
    format!(
        "SOURCE\n{}\n\nSOURCE EXISTS  {}\nWORKTREE CLEAN {}\nSERVICE        {}\nSERVICE ACTIVE {}\nBINARY         {}\n\nDIAGNOSTICS\n{}",
        item.source.display(),
        yes_no(item.source_exists),
        yes_no(item.worktree_clean),
        service,
        item.service_active.map(yes_no).unwrap_or("n/a"),
        binary,
        diagnostics
    )
}

fn status_style(status: &str) -> Style {
    let color = match status {
        "READY" | "OK" | "SUCCEEDED" => Color::Green,
        "RUNNING" | "PLANNED" => Color::Yellow,
        "ATTENTION" | "BLOCKED" => Color::Red,
        "FAILED" => Color::Red,
        _ => Color::DarkGray,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

fn log_line(log: &str) -> Line<'static> {
    let (prefix, rest) = log.split_once(' ').unwrap_or((log, ""));
    let style = match prefix {
        "OK" | "READY" => status_style("OK"),
        "RUN" | "RUNNING" | "PLANNED" | "PLAN" | "INSPECT" => {
            Style::default().fg(Color::Yellow)
        }
        "BLOCKED" | "FAILED" | "ATTENTION" => status_style("FAILED"),
        "HELP:" => Style::default().fg(Color::Magenta),
        _ => Style::default().fg(Color::Gray),
    };
    Line::from(vec![
        Span::styled(prefix.to_string(), style),
        Span::raw(if rest.is_empty() { "" } else { " " }),
        Span::styled(rest.to_string(), Style::default().fg(Color::Gray)),
    ])
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn plan_tasks(
    engine: &Engine,
    tasks: &[Task],
    selected: &[bool],
    statuses: &mut HashMap<String, String>,
    logs: &mut Vec<String>,
) {
    let names = selected_names(tasks, selected);
    if names.is_empty() {
        logs.push("No tasks selected.".into());
        return;
    }
    logs.push("Planning selected tasks (no changes will be made)…".into());
    let results = engine.run(&names, false, |event| logs.push(format_event(event)));
    record_results(results.clone(), statuses, logs);
    if let Err(error) = engine.record_history("plan", &results) {
        logs.push(format!("HISTORY failed to save plan: {error}"));
    }
}

fn execute_action(
    engine: &Engine,
    action: Action,
    tasks: &[Task],
    selected: &[bool],
    statuses: &mut HashMap<String, String>,
    logs: &mut Vec<String>,
) {
    let names = if matches!(action, Action::ThemeSync) {
        vec![
            "cyberdeck-hub-themes".to_string(),
            "dwcode-favicons-themes".to_string(),
            "dwcode-themes".to_string(),
        ]
    } else {
        selected_names(tasks, selected)
    };
    if names.is_empty() {
        logs.push("No tasks selected.".into());
        return;
    }
    logs.push(format!("Executing {}…", action.label()));
    let results = match action {
        Action::Apply => engine.run(&names, true, |event| logs.push(format_event(event))),
        Action::Repair => engine.repair(&names, |event| logs.push(format_event(event))),
        Action::Restart => engine.restart_services(&names, |event| logs.push(format_event(event))),
        Action::Rollback => engine.rollback(&names, |event| logs.push(format_event(event))),
        Action::ThemeSync => engine.run(&names, true, |event| logs.push(format_event(event))),
    };
    record_results(results.clone(), statuses, logs);
    if let Err(error) = engine.record_history(action.label(), &results) {
        logs.push(format!("HISTORY failed to save operation: {error}"));
    }
}

fn selected_names(tasks: &[Task], selected: &[bool]) -> Vec<String> {
    tasks.iter().enumerate().filter(|(i, _)| selected[*i]).map(|(_, task)| task.name.clone()).collect()
}

fn refresh_inspection(
    engine: &Engine,
    inspections: &mut HashMap<String, Inspection>,
    statuses: &mut HashMap<String, String>,
    logs: &mut Vec<String>,
) {
    let items = engine.inspect();
    inspections.clear();
    statuses.clear();
    let mut attention = 0;
    for item in items {
        let status = if item.diagnostics.is_empty() { "READY" } else { attention += 1; "ATTENTION" };
        statuses.insert(item.name.clone(), status.into());
        logs.push(format!("INSPECT {:<27} {status}", item.name));
        inspections.insert(item.name.clone(), item);
    }
    logs.push(format!(
        "Inspection complete: {} ready, {} attention required.",
        statuses.len().saturating_sub(attention),
        attention
    ));
}

fn record_results(
    results: Vec<(String, TaskStatus)>,
    statuses: &mut HashMap<String, String>,
    logs: &mut Vec<String>,
) {
    for (name, status) in results {
        let label = match status {
            TaskStatus::Succeeded => "OK",
            TaskStatus::Blocked => "BLOCKED",
            TaskStatus::Failed => "FAILED",
            TaskStatus::Planned => "PLANNED",
            TaskStatus::Running => "RUNNING",
        };
        statuses.insert(name.clone(), label.into());
        logs.push(format!("{label:<8} {name}"));
    }
}

fn trim_logs(logs: &mut Vec<String>) {
    if logs.len() > 300 {
        logs.drain(0..logs.len() - 300);
    }
}

fn format_event(event: Event) -> String {
    let status = match event.status {
        TaskStatus::Planned => "PLAN",
        TaskStatus::Running => "RUN",
        TaskStatus::Succeeded => "OK",
        TaskStatus::Blocked => "BLOCKED",
        TaskStatus::Failed => "FAILED",
    };
    format!("{status:<8} {} — {}", event.task, event.message)
}
