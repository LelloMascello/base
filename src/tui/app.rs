use crate::config::Config;
use std::path::PathBuf;

/// Which phase the conversation is in. In Rust an `enum` isn't just a list
/// of names like in C: each variant can carry data (not needed here), and
/// the compiler forces you to handle *every* case wherever you `match` on
/// it — if you add a variant tomorrow and forget a `match`, the project
/// stops compiling instead of silently misbehaving at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    AwaitingApproval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    You,
    Info,
    Warn,
    Error,
    Agent,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub text: String,
    pub kind: LineKind,
}

pub struct App {
    pub root: PathBuf,
    pub config: Config,
    pub input: String,
    pub log: Vec<LogLine>,
    pub mode: Mode,
    pub should_quit: bool,
}

impl App {
    pub fn new(root: PathBuf, config: Config) -> Self {
        let mut app = Self {
            root,
            config,
            input: String::new(),
            log: Vec::new(),
            mode: Mode::Normal,
            should_quit: false,
        };
        app.push(LineKind::Info, "Agent orchestration initialized. Ready for instructions.");
        app.push(LineKind::Info, "Type a request, or /help for the list of commands.");
        app
    }

    pub fn push(&mut self, kind: LineKind, text: impl Into<String>) {
        self.log.push(LogLine { text: text.into(), kind });
    }

    /// Called when the user presses Enter. Control then returns to
    /// `events.rs`, which redraws the screen afterwards.
    pub fn submit(&mut self) {
        let line = self.input.trim().to_string();
        self.input.clear();
        if line.is_empty() {
            return;
        }
        self.push(LineKind::You, format!("> {line}"));

        if self.mode == Mode::AwaitingApproval {
            self.resolve_approval(&line);
            return;
        }

        match line.strip_prefix('/') {
            Some(cmd) => self.handle_command(cmd),
            None => self.handle_task(&line),
        }
    }

    fn handle_command(&mut self, cmd: &str) {
        match cmd {
            "help" => {
                self.push(LineKind::Info, "/agents   loaded agents and tools");
                self.push(LineKind::Info, "/clear    clears the log");
                self.push(LineKind::Info, "/quit     exits (same as Esc)");
            }
            "agents" => {
                let orc = self.config.orchestrator.clone();
                self.push(LineKind::Agent, format!("[ORC]  {}", orc.name));
                for w in self.config.workers.clone() {
                    self.push(LineKind::Agent, format!("[WRK]  {} · {}", w.name, w.description));
                }
                for t in self.config.tools.clone() {
                    self.push(LineKind::Agent, format!("[TOOL] {}", t.name));
                }
            }
            "clear" => self.log.clear(),
            "quit" => self.should_quit = true,
            other => self.push(LineKind::Error, format!("Unknown command: /{other}")),
        }
    }

    fn handle_task(&mut self, input: &str) {
        if let Some(token) = crate::sandbox::fs::find_path_like_token(input) {
            if crate::sandbox::fs::resolve_within(&self.root, token).is_none() {
                self.push(LineKind::Error, format!("Denied: '{token}' is outside the sandbox ({})", self.root.display()));
                self.push(LineKind::Warn, "Ctrl+A attaches an external file as read-only, if you need one.");
                return;
            }
        }

        let plan = crate::agent::orc::plan(input);
        for line in plan.lines() {
            self.push(LineKind::Agent, line.to_string());
        }
        self.push(LineKind::Warn, "Approve? [y] yes  [n] no");
        self.mode = Mode::AwaitingApproval;
    }

    fn resolve_approval(&mut self, answer: &str) {
        self.mode = Mode::Normal;
        match answer.to_lowercase().as_str() {
            "y" | "yes" => {
                let result = crate::agent::orc::run_approved_plan();
                self.push(LineKind::Info, result);
            }
            _ => self.push(LineKind::Warn, "Plan discarded."),
        }
    }

    /// Called by `tui::run` after the user picked a file or folder in the
    /// selection screen opened with Ctrl+A. Still a deliberate stub: it
    /// only records *which* path was picked, without actually copying it
    /// into `.base/attachments/` — that part is still fake.
    pub fn attach_selected(&mut self, path: std::path::PathBuf) {
        self.push(
            LineKind::Info,
            format!(
                "Selected: {} (stub — this would be copied read-only into {}/.base/attachments/)",
                path.display(),
                self.root.display()
            ),
        );
    }

    /// Called by `tui::run` if the user leaves the attach picker with Esc
    /// without choosing anything.
    pub fn attach_cancelled(&mut self) {
        self.push(LineKind::Info, "Attach cancelled.");
    }

    pub fn interrupt(&mut self) {
        if self.mode == Mode::AwaitingApproval {
            self.mode = Mode::Normal;
            self.push(LineKind::Warn, "Interrupted.");
        } else {
            self.push(LineKind::Info, "Nothing is running.");
        }
    }
}