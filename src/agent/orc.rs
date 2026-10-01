//! In the future this is where we'll really call the agent's endpoint (see
//! `config::AgentInfo`: it already has a `name`, only the URL/key are
//! missing, which will arrive once `config::load()` really parses the TOML).
//!
//! These functions' signatures are already the "final" ones: they take in
//! whatever they need, return a `String` with the reply. Only the body
//! will change, once they stop being stubs.

/// Asks the orchestrator for a plan for the given prompt.
///
/// STUB: ignores `_prompt` and always returns the same text. The real call
/// will use `_prompt` to build the HTTP request to `endpoint`.
pub fn plan(_prompt: &str) -> String {
    "Proposed plan:\n 1. read the files involved\n 2. apply the change\n 3. run the tests"
        .to_string()
}

/// Runs the plan after the user has approved it.
///
/// STUB: no real network call or tool execution.
pub fn run_approved_plan() -> String {
    "Done (stub) — no real change was applied. \
     This is where the real call to the agent and its tools will go."
        .to_string()
}