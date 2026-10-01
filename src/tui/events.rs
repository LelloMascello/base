use crate::tui::app::App;
use crate::tui::picker::{PickerKind, PickerState};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

/// What should happen after one round of `handle`, decided by the key that
/// was pressed. `None` is the common case ("I already updated `app`,
/// nothing else to do"); `Attach` is the one case `events.rs` can't resolve
/// on its own: opening the attach picker needs the `Terminal`, which isn't
/// available here (only `tui::run`, which owns it — see `tui/mod.rs`).
pub enum Action {
    None,
    Attach,
}

/// Waits for a keyboard event for at most 100ms and updates `app`
/// accordingly. The timeout is what lets the main loop keep redrawing the
/// screen at regular intervals even without input (useful later for things
/// like a "..." spinner on a running task, not implemented in this dummy
/// version yet).
pub fn handle(app: &mut App) -> io::Result<Action> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(Action::None);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(Action::None);
    };
    // Some terminals send both "key pressed" and "key released" through
    // crossterm: we only care about the press.
    if key.kind != KeyEventKind::Press {
        return Ok(Action::None);
    }

    match key.code {
        KeyCode::Enter => app.submit(),
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Action::Attach);
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.interrupt();
        }
        KeyCode::Char(c) => {
            app.input.push(c);
        }
        _ => {}
    }

    Ok(Action::None)
}

/// What to do after one round of the picker screen: keep showing it, or
/// stop because the user picked something (with the chosen path) or
/// cancelled.
pub enum PickerOutcome {
    Continue,
    Selected(PathBuf),
    Quit,
}

/// Navigation is arrow keys only: ↑/↓ move the selection, → opens the
/// highlighted folder, ← goes up one level. Enter confirms the current
/// choice, Esc cancels — nothing else is bound here on purpose.
pub fn handle_picker(state: &mut PickerState) -> io::Result<PickerOutcome> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(PickerOutcome::Continue);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(PickerOutcome::Continue);
    };
    if key.kind != KeyEventKind::Press {
        return Ok(PickerOutcome::Continue);
    }

    match key.code {
        KeyCode::Up => state.move_selection(-1),
        KeyCode::Down => state.move_selection(1),
        KeyCode::Right => state.enter_selected(), // no-op if the highlighted entry is a file
        KeyCode::Left => state.go_up(),
        KeyCode::Enter => match state.kind {
            // Directory mode: Enter always confirms the folder currently
            // being browsed, regardless of what's highlighted.
            PickerKind::Directory => return Ok(PickerOutcome::Selected(state.current.clone())),
            // Attach mode: Enter picks whatever is highlighted — a file or
            // a folder, without looking inside it (use → to do that instead).
            PickerKind::Attach => {
                if let Some(path) = state.highlighted_path() {
                    return Ok(PickerOutcome::Selected(path));
                }
            }
        },
        KeyCode::Esc => return Ok(PickerOutcome::Quit),
        _ => {}
    }

    Ok(PickerOutcome::Continue)
}