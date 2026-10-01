//! In the future this file will read and validate the `~/.config/base/*.toml`
//! tree (agents, tools, guardrails — see the project's README). For now
//! `load()` always returns the same fixed data, so the rest of the program
//! (CLI, TUI) already has something to work with.
//!
//! `#[derive(Debug, Clone)]` above each struct asks the compiler to
//! generate the implementation for "printable in debug form" and
//! "duplicable" on its own. In C++ you'd write a copy constructor by hand;
//! here the line above the type is enough.

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub orchestrator: AgentInfo,
    pub workers: Vec<AgentInfo>,
    pub tools: Vec<ToolInfo>,
}

/// STUB: no reading from disk. Once real TOML parsing exists, this
/// signature (`fn load() -> Config`) will stay the same: only what
/// happens inside will change.
pub fn load() -> Config {
    Config {
        orchestrator: AgentInfo {
            name: "orc".to_string(),
            description: "orchestrator".to_string(),
        },
        workers: vec![AgentInfo {
            name: "vision".to_string(),
            description: "face recognition and image analysis".to_string(),
        }],
        tools: vec![
            ToolInfo { name: "fs-read".to_string() },
            ToolInfo { name: "fs-write".to_string() },
            ToolInfo { name: "fs-manage".to_string() },
            ToolInfo { name: "web-search".to_string() },
        ],
    }
}