use crate::tui::app::{App, LineKind, Mode};
use crate::tui::picker::{PickerKind, PickerState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Called once per frame by `tui::run`. `Frame` represents the screen at
/// this exact instant: nothing is drawn incrementally like in a retained
/// GUI, everything is redescribed every time — the model almost every TUI
/// works with (ncurses included, in C).
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.size();

    let agent_rows = 1 + app.config.workers.len() as u16 + app.config.tools.len() as u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),              // title
            Constraint::Length(agent_rows + 2), // agent panel (+ borders)
            Constraint::Min(3),                 // log
            Constraint::Length(3),              // prompt
            Constraint::Length(1),              // key hints
        ])
        .split(area);

    draw_title(frame, chunks[0], app);
    draw_agents(frame, chunks[1], app);
    draw_log(frame, chunks[2], app);
    draw_prompt(frame, chunks[3], app);
    draw_hints(frame, chunks[4], app);
}

fn draw_title(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        " BASE — {}  [Sandbox]",
        app.root.display()
    );
    frame.render_widget(
        Paragraph::new(title).style(Style::default().add_modifier(Modifier::BOLD)),
        area,
    );
}

fn draw_agents(frame: &mut Frame, area: Rect, app: &App) {
    let mut items: Vec<ListItem> = Vec::new();

    items.push(ListItem::new(Line::from(vec![
        Span::styled("[ORC] ", Style::default().fg(Color::Magenta)),
        Span::raw(format!("{}: Idle", app.config.orchestrator.name)),
    ])));

    for w in &app.config.workers {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("[WRK] ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{} · {}: Ready", w.name, w.description)),
        ])));
    }

    for t in &app.config.tools {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("[TOOL] ", Style::default().fg(Color::LightMagenta)),
            Span::raw(format!("{}: Ready", t.name)),
        ])));
    }

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Agents"));
    frame.render_widget(list, area);
}

fn draw_log(frame: &mut Frame, area: Rect, app: &App) {
    let inner_height = area.height.saturating_sub(2) as usize; // -2 for the borders
    let visible = app.log.iter().rev().take(inner_height.max(1)).rev();

    let lines: Vec<Line> = visible
        .map(|entry| {
            let style = match entry.kind {
                LineKind::You => Style::default().fg(Color::Blue),
                LineKind::Info => Style::default().fg(Color::DarkGray),
                LineKind::Warn => Style::default().fg(Color::Yellow),
                LineKind::Error => Style::default().fg(Color::Red),
                LineKind::Agent => Style::default().fg(Color::Cyan),
            };
            Line::from(Span::styled(entry.text.clone(), style))
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn draw_prompt(frame: &mut Frame, area: Rect, app: &App) {
    let label = match app.mode {
        Mode::Normal => "Prompt",
        Mode::AwaitingApproval => "Prompt — answer y/n",
    };
    let text = format!("> {}", app.input);
    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(label)),
        area,
    );
}

fn draw_hints(frame: &mut Frame, area: Rect, app: &App) {
    let hint = match app.mode {
        Mode::Normal => "[Enter] Send  [Ctrl+A] Attach  [Ctrl+C] Interrupt  [Esc] Quit",
        Mode::AwaitingApproval => "[y] Approve  [n] Discard  [Ctrl+C] Cancel",
    };
    let _ = app; // the hint doesn't depend on other state yet, but the signature stays ready for that
    frame.render_widget(Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)), area);
}

/// Draws the folder/file picker screen (`base` with no arguments, or
/// Ctrl+A inside the sandbox). Unlike `draw`, the state here is
/// `PickerState`, not `App`: these are two distinct screens with distinct
/// data — `tui::run_picker` decides which one to call, frame after frame.
pub fn draw_picker(frame: &mut Frame, state: &PickerState) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title with the current path
            Constraint::Min(3),    // entry list
            Constraint::Length(1), // optional error message
            Constraint::Length(1), // key hints
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(format!(" {} — {}", picker_title(state.kind), state.current.display()))
            .style(Style::default().add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    let items: Vec<ListItem> = if state.entries.is_empty() {
        let text = match state.kind {
            PickerKind::Directory => "  (no subfolders here)",
            PickerKind::Attach => "  (empty folder)",
        };
        vec![ListItem::new(text).style(Style::default().fg(Color::DarkGray))]
    } else {
        state
            .entries
            .iter()
            .map(|entry| {
                let name = entry
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| entry.path.display().to_string());
                let icon = if entry.is_dir { "📁" } else { "📄" };
                ListItem::new(format!("{icon} {name}"))
            })
            .collect()
    };

    // `List` alone only draws the rows; to make the highlight follow
    // `state.selected` you also need to pass a mutable `ListState` — a
    // "stateful widget" in ratatui, distinct from "stateless" widgets like
    // `Paragraph` used elsewhere in this file.
    let mut list_state = ListState::default();
    if !state.entries.is_empty() {
        list_state.select(Some(state.selected));
    }
    let list_title = match state.kind {
        PickerKind::Directory => "Subfolders",
        PickerKind::Attach => "Folders and files",
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let message = state.message.clone().unwrap_or_default();
    frame.render_widget(
        Paragraph::new(message).style(Style::default().fg(Color::Red)),
        chunks[2],
    );

    let hint = match state.kind {
        PickerKind::Directory => "[↑/↓] move  [→] open  [←] up  [Enter] use this folder  [Esc] cancel",
        PickerKind::Attach => "[↑/↓] move  [→] open  [←] up  [Enter] select  [Esc] cancel",
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        chunks[3],
    );
}

fn picker_title(kind: PickerKind) -> &'static str {
    match kind {
        PickerKind::Directory => "Choose a folder",
        PickerKind::Attach => "Choose a file or folder to attach",
    }
}