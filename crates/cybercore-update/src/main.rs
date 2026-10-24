use cybercore_updater::{Engine, Event, Manifest, TaskStatus};
use crossterm::{
    event::{self, Event as Input, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{
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
    let tasks = engine.tasks();

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
    let result = run(&mut terminal, &engine, tasks);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    engine: &Engine,
    tasks: Vec<cybercore_updater::Task>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut selected = vec![true; tasks.len()];
    let mut cursor: usize = 0;
    let mut logs = vec!["Space toggle  A all  N none  I inspect  R repair  Enter apply  Q quit".to_string()];
    append_inspection(engine, &mut logs);
    loop {
        terminal.draw(|frame| {
            let areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(6), Constraint::Length(8)])
                .split(frame.area());
            frame.render_widget(
                Paragraph::new("CYBERCORE // SYSTEM UPDATER")
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                areas[0],
            );
            let items = tasks
                .iter()
                .enumerate()
                .map(|(index, task)| {
                    let marker = if selected[index] { "[x]" } else { "[ ]" };
                    ListItem::new(format!("{} {}", marker, task.name))
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                List::new(items)
                    .block(Block::default().title("Tasks").borders(Borders::ALL))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                areas[1],
            );
            frame.render_widget(
                Paragraph::new(logs.join("\n"))
                    .block(Block::default().title("Activity").borders(Borders::ALL)),
                areas[2],
            );
        })?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        match event::read()? {
            Input::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Up => cursor = cursor.saturating_sub(1),
                KeyCode::Down => cursor = (cursor + 1).min(tasks.len().saturating_sub(1)),
                KeyCode::Char(' ') => selected[cursor] = !selected[cursor],
                KeyCode::Char('a') => selected.fill(true),
                KeyCode::Char('n') => selected.fill(false),
                KeyCode::Enter => {
                    let names = tasks
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| selected[*index])
                        .map(|(_, task)| task.name.clone())
                        .collect::<Vec<_>>();
                    logs.push("Applying selected tasks…".into());
                    engine.run(&names, true, |event: Event| {
                        logs.push(format_event(event));
                        if logs.len() > 7 {
                            logs.remove(0);
                        }
                    });
                }
                KeyCode::Char('i') => {
                    append_inspection(engine, &mut logs);
                }
                KeyCode::Char('r') => {
                    let names = tasks
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| selected[*index])
                        .map(|(_, task)| task.name.clone())
                        .collect::<Vec<_>>();
                    logs.push("Repairing selected tasks…".into());
                    engine.repair(&names, |event: Event| {
                        logs.push(format_event(event));
                        if logs.len() > 7 {
                            logs.remove(0);
                        }
                    });
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}

fn append_inspection(engine: &Engine, logs: &mut Vec<String>) {
    for item in engine.inspect() {
        let message = if item.diagnostics.is_empty() {
            format!("INSPECT {:<22} ready", item.name)
        } else {
            format!(
                "INSPECT {:<22} {}",
                item.name,
                item.diagnostics.join("; ")
            )
        };
        logs.push(message);
    }
    while logs.len() > 7 {
        logs.remove(0);
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
    format!("{status:<7} {} — {}", event.task, event.message)
}
