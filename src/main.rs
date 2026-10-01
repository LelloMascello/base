mod agent;
mod cli;
mod config;
mod sandbox;
mod tui;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::Backend, backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let args = cli::Args::parse();

    // If the folder is already given on the command line (`base .` /
    // `base <folder>`), validate it RIGHT AWAY, before even touching the
    // terminal: an error goes to stderr with exit code 1, like any other
    // well-behaved CLI command, without having to enter and then leave the
    // alternate screen first.
    let root_from_cli = match args.path {
        Some(path) => {
            let resolved = path.canonicalize().unwrap_or(path);
            if !resolved.is_dir() {
                eprintln!("base: {} is not a folder", resolved.display());
                std::process::exit(1);
            }
            Some(resolved)
        }
        None => None, // will be picked from the selection screen, inside the TUI
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_session(&mut terminal, root_from_cli);

    // Whatever happens above, the terminal always gets restored:
    // otherwise the user ends up with their shell stuck in raw mode after
    // a crash.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// With a folder already decided (from the CLI), opens the sandbox right
/// away. Otherwise shows the selection screen first: if the user cancels
/// there, the program exits with no errors, having never opened a sandbox.
fn run_session<B: Backend>(terminal: &mut Terminal<B>, root_from_cli: Option<PathBuf>) -> io::Result<()> {
    let root = match root_from_cli {
        Some(path) => path,
        None => {
            let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            match tui::run_picker(terminal, start, tui::PickerKind::Directory)? {
                Some(path) => path,
                None => return Ok(()),
            }
        }
    };

    let config = config::load();
    let mut app = tui::App::new(root, config);
    tui::run(terminal, &mut app)
}