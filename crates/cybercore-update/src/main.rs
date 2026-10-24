use cybercore_updater::{Engine, Event, Inspection, Manifest, Task, TaskStatus};
use crossterm::{
    event::{self, Event as Input, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{
    collections::HashMap,
    env,
    io::{self, stdout},
    path::PathBuf,
    time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env::var("CYBERCORE_ROOT").unwrap_or_else(|_| {
        "/home/raven/Devspace/Cybercore/framework/cybercore".into()
    }));
    let manifest_path = root.join("tools/cybercore-consumers.json");
    let engine = Engine::new(Manifest::load(&manifest_path)?, &root);

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
    let mut statuses = HashMap::new();
    let mut logs = vec!["↑/↓ move  Space toggle  A all  N none  I inspect".to_string()];
    logs.push("P plan  Enter apply  R repair  S restart  B rollback  Q quit".to_string());
    refresh_inspection(engine, &mut statuses, &mut logs);

    loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(10)])
                .split(frame.area());
            frame.render_widget(
                Paragraph::new("CYBERCORE // SYSTEM UPDATER  |  LOCAL OPERATOR MODE")
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                areas[0],
            );

            let task_lines = tasks
                .iter()
                .enumerate()
                .map(|(index, task)| {
                    let marker = if selected[index] { "[x]" } else { "[ ]" };
                    let pointer = if index == cursor { ">" } else { " " };
                    let status = statuses
                        .get(&task.name)
                        .map(String::as_str)
                        .unwrap_or("UNKNOWN");
                    format!("{pointer} {marker} {:<28} {status}", task.name)
                })
                .collect::<Vec<_>>()
                .join("\n");
            frame.render_widget(
                Paragraph::new(task_lines)
                    .block(Block::default().title("Tasks").borders(Borders::ALL))
                    .scroll((task_offset, 0)),
                areas[1],
            );

            let activity = logs.join("\n");
            frame.render_widget(
                Paragraph::new(activity)
                    .block(Block::default().title("Activity").borders(Borders::ALL))
                    .scroll((log_offset, 0)),
                areas[2],
            );
        })?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        if let Input::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Up => {
                    cursor = cursor.saturating_sub(1);
                    task_offset = task_offset.saturating_sub(1);
                }
                KeyCode::Down => {
                    cursor = (cursor + 1).min(tasks.len().saturating_sub(1));
                    task_offset = task_offset.saturating_add(1);
                }
                KeyCode::PageUp => {
                    task_offset = task_offset.saturating_sub(6);
                }
                KeyCode::PageDown => {
                    task_offset = task_offset.saturating_add(6);
                }
                KeyCode::Home => {
                    cursor = 0;
                    task_offset = 0;
                }
                KeyCode::End => {
                    cursor = tasks.len().saturating_sub(1);
                    task_offset = cursor.saturating_sub(4) as u16;
                }
                KeyCode::Char(' ') => selected[cursor] = !selected[cursor],
                KeyCode::Char('a') => selected.fill(true),
                KeyCode::Char('n') => selected.fill(false),
                KeyCode::Char('i') => refresh_inspection(engine, &mut statuses, &mut logs),
                KeyCode::Char('p') => {
                    run_tasks(engine, &tasks, &selected, false, &mut statuses, &mut logs)
                }
                KeyCode::Enter => {
                    run_tasks(engine, &tasks, &selected, true, &mut statuses, &mut logs)
                }
                KeyCode::Char('r') => {
                    let names = selected_names(&tasks, &selected);
                    logs.push("Repairing selected tasks…".into());
                    let results = engine.repair(&names, |event| logs.push(format_event(event)));
                    record_results(results, &mut statuses, &mut logs);
                }
                KeyCode::Char('s') => {
                    let names = selected_names(&tasks, &selected);
                    logs.push("Restarting selected user services…".into());
                    let results = engine.restart_services(&names, |event| logs.push(format_event(event)));
                    record_results(results, &mut statuses, &mut logs);
                }
                KeyCode::Char('b') => {
                    let names = selected_names(&tasks, &selected);
                    logs.push("Rolling back selected binaries…".into());
                    let results = engine.rollback(&names, |event| logs.push(format_event(event)));
                    record_results(results, &mut statuses, &mut logs);
                }
                _ => {}
            }
            trim_logs(&mut logs);
            log_offset = logs.len().saturating_sub(8) as u16;
        }
    }
    Ok(())
}

fn run_tasks(
    engine: &Engine,
    tasks: &[Task],
    selected: &[bool],
    apply: bool,
    statuses: &mut HashMap<String, String>,
    logs: &mut Vec<String>,
) {
    let names = selected_names(tasks, selected);
    if names.is_empty() {
        logs.push("No tasks selected.".into());
        return;
    }
    logs.push(if apply {
        "Applying selected tasks…".into()
    } else {
        "Planning selected tasks…".into()
    });
    let results = engine.run(&names, apply, |event| logs.push(format_event(event)));
    record_results(results, statuses, logs);
}

fn selected_names(tasks: &[Task], selected: &[bool]) -> Vec<String> {
    tasks
        .iter()
        .enumerate()
        .filter(|(index, _)| selected[*index])
        .map(|(_, task)| task.name.clone())
        .collect()
}

fn refresh_inspection(
    engine: &Engine,
    statuses: &mut HashMap<String, String>,
    logs: &mut Vec<String>,
) {
    let inspection = engine.inspect();
    statuses.clear();
    let attention = inspection
        .iter()
        .filter(|item| !item.diagnostics.is_empty())
        .count();
    for item in inspection {
        let status = if item.diagnostics.is_empty() {
            "READY".to_string()
        } else {
            "ATTENTION".to_string()
        };
        statuses.insert(item.name.clone(), status.clone());
        logs.push(format_inspection(&item, &status));
    }
    logs.push(format!(
        "Inspection complete: {} ready, {} attention required.",
        statuses.values().filter(|status| status.as_str() == "READY").count(),
        attention
    ));
}

fn format_inspection(item: &Inspection, status: &str) -> String {
    if item.diagnostics.is_empty() {
        format!("INSPECT {:<28} {status}", item.name)
    } else {
        format!(
            "INSPECT {:<28} {status}: {}",
            item.name,
            item.diagnostics.join("; ")
        )
    }
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
    if logs.len() > 200 {
        let remove = logs.len() - 200;
        logs.drain(0..remove);
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
