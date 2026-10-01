pub mod app;
mod events;
pub mod picker;
mod ui;

pub use app::App;
pub use picker::PickerKind;

use ratatui::{backend::Backend, Terminal};
use std::io;
use std::path::PathBuf;

/// The main loop: draw, wait for an event, update the state, repeat.
/// `B: Backend` is generic — in this project it will always be
/// `CrosstermBackend`, but writing it this way would in theory allow
/// swapping it out (handy for tests, with a fake backend that never
/// touches the real terminal).
///
/// The only event `events::handle` can't resolve on its own is
/// `Action::Attach`: it needs the `Terminal`, which only this function
/// owns, to open the attach picker screen *inside the same loop*.
pub fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        match events::handle(app)? {
            events::Action::None => {}
            events::Action::Attach => {
                let start = home_dir_or(&app.root);
                match run_picker(terminal, start, PickerKind::Attach)? {
                    Some(path) => app.attach_selected(path),
                    None => app.attach_cancelled(),
                }
            }
        }
    }
    Ok(())
}

/// The picker screen's loop, in folder mode (`base` with no arguments, from
/// `main.rs`) or Attach mode (Ctrl+A inside the sandbox, from `run` above).
/// Returns `Some(path)` if the user picked something, `None` if they
/// cancelled (Esc).
pub fn run_picker<B: Backend>(
    terminal: &mut Terminal<B>,
    start: PathBuf,
    kind: PickerKind,
) -> io::Result<Option<PathBuf>> {
    let mut state = picker::PickerState::new(start, kind);
    loop {
        terminal.draw(|frame| ui::draw_picker(frame, &state))?;
        match events::handle_picker(&mut state)? {
            events::PickerOutcome::Continue => {}
            events::PickerOutcome::Selected(path) => return Ok(Some(path)),
            events::PickerOutcome::Quit => return Ok(None),
        }
    }
}

/// Starting point for picking an external file: the user's home folder,
/// since Ctrl+A exists precisely to reach something *outside* the sandbox.
/// `root` (the sandbox itself) is only a fallback if `$HOME` can't be read
/// in this environment.
fn home_dir_or(root: &std::path::Path) -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.to_path_buf())
}