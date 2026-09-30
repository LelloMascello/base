# BASE — Bounded Agent System Environment

A lightweight, terminal-based agentic harness that isolates agents inside a sandbox tied to a directory.

## Philosophy

- **The sandbox is real**: agents are confined to the working directory (or the folder specified at launch), not just by convention but through actual read, write, and execute permissions.
- **Config over CLI**: behavior — agents, tools, guardrails — is not built from the command line, but declared in readable files that can be versioned with git and shared across a team.
- **Extremely customizable**: any personal script or HTTP service can become a tool for agents, by describing in a file what it's called, how it runs, and where it's allowed to act.

## CLI

The binary exposes three ways of being used:

```bash
base              # opens BASE on a folder selection screen; the chosen folder becomes the sandbox
base .            # opens BASE with the sandbox rooted in the current directory
base <folder>     # opens BASE with the sandbox rooted in <folder>, without having to enter it
```

Everything else — agents, tools, guardrails — is configured by editing the files in `~/.config/base`, not from the command line.

## Bootstrap

On first launch — or whenever something is missing — BASE generates in `~/.config/base` only the files that don't exist yet, with commented placeholders ready to be filled in. It never overwrites a file the user has already modified.

```
~/.config/base/
├── config.toml
├── agents/
│   ├── orchestrator.toml
│   └── worker.example.toml
├── tools/
│   ├── web-search.toml
│   ├── fs-read.toml
│   ├── fs-write.toml
│   └── fs-manage.toml
└── guardrails/
    └── default.toml
```

### Startup validation

Before showing the TUI, BASE loads and validates the entire configuration tree and prints a summary: loaded agents and tools, and any errors (invalid TOML, missing required field, a tool referenced in a guardrail rule but not defined, a launch folder outside a fixed `scope`). Startup is blocked only if the orchestrator is missing or invalid, or if the requested folder is outside the boundary; tools and workers with errors are simply excluded, with a warning.

## `config.toml` — global settings

```toml
schema_version = 1

[defaults]
log_level = "info"
snapshot_dir = ".base/snapshots"       # relative to the opened sandbox
attachments_dir = ".base/attachments"
```

## Agents — `agents/*.toml`

One file per agent. `kind` distinguishes the orchestrator (exactly one, mandatory) from workers (zero or more).

```toml
# agents/orchestrator.toml
schema_version = 1
kind = "orchestrator"
name = "orc"
endpoint = "http://localhost:8080/v1/chat/completions"
key_env = "BASE_ORC_KEY"     # the key is read from an environment variable, never in plain text in the file
model = "llama-3.1-70b"
```

```toml
# agents/worker.example.toml — template: duplicate and rename it for each new worker
schema_version = 1
kind = "worker"
name = "vision"
description = "face recognition and image analysis"   # the Orchestrator routes tasks based on this field
endpoint = "http://localhost:8081/v1/chat/completions"
key_env = "BASE_VISION_KEY"
model = "llava-1.6"
```

## Tools — `tools/*.toml`

Three kinds of tools, distinguished by the `type` field:

| type | use | example |
|---|---|---|
| `builtin` | functionality implemented in the binary itself (sandbox fs, web search) | the four default tools |
| `script` | local executable (Python, bash, a binary) | a user's personal script |
| `http` | server or API to send requests to | a search service, a remote tool |

Typical scenario — your own Python script used as a tool:

```toml
# tools/format-code.toml
schema_version = 1
type = "script"
name = "format-code"
interpreter = "python3"
path = "~/scripts/format.py"
args = ["--fix"]
timeout_s = 30
```

If the tool is instead a server you make calls to, only the way it's invoked changes:

```toml
# tools/my-remote-tool.toml
schema_version = 1
type = "http"
name = "my-remote-tool"
endpoint = "http://localhost:9000/run"
key_env = "MY_TOOL_KEY"
timeout_s = 15
```

### Default tools (generated at bootstrap)

```toml
# tools/web-search.toml
schema_version = 1
type = "builtin"
name = "web-search"
handler = "core.web_search"
key_env = "BASE_SEARCH_KEY"   # optional, depends on the configured provider
```

```toml
# tools/fs-read.toml
schema_version = 1
type = "builtin"
name = "fs-read"
handler = "core.fs.read"          # reading existing files and folders (cat, ls, tree)
```

```toml
# tools/fs-write.toml
schema_version = 1
type = "builtin"
name = "fs-write"
handler = "core.fs.write"         # modifying the content of an already existing file
```

```toml
# tools/fs-manage.toml
schema_version = 1
type = "builtin"
name = "fs-manage"
handler = "core.fs.manage"        # creating, rm, mv, cp of files and folders
```

## Guardrails — `guardrails/*.toml`

**Whitelist** model: a tool exists as soon as its file appears in `tools/`, but it can't be called by any agent until it appears in at least one guardrail rule. Zero rules for a tool = the tool is invisible in that sandbox — not "asks for confirmation" but simply absent. Multiple guardrail files can coexist — a personal one, a team one versioned with git — applied in order: the last matching rule wins, which also allows exceptions (`permission = "deny"`) that restrict an otherwise open area.

```toml
# guardrails/default.toml
schema_version = 1

[defaults]
scope = "workdir"        # the boundary is the folder passed to `base .` / `base <folder>`: it changes every session

[[mount]]
name = "manuals"
path = "~/Documents/language-manuals"   # always visible, regardless of which folder opens the session

[[rule]]
tool = "fs-read"
paths = ["**"]           # the whole sandbox
permission = "auto"

[[rule]]
tool = "fs-write"
paths = ["**"]
permission = "confirm"   # modifying existing files: asks for confirmation

[[rule]]
tool = "fs-manage"
paths = ["**"]
permission = "confirm"   # create/rm/mv/cp remain destructive: they ask for confirmation

[[rule]]
tool = "web-search"
permission = "auto"      # doesn't touch the filesystem, "paths" doesn't apply

[[rule]]                 # the mount is accessible only because a rule references it
tool = "fs-read"
mount = "manuals"
permission = "auto"
# no rule with mount = "manuals" for fs-write or fs-manage: there they remain unavailable

[[rule]]                 # exception: a normally readable area, excluded
tool = "fs-read"
paths = ["secrets/**"]
permission = "deny"
```

### The boundary (`scope`) and the two ways to widen it

`scope` accepts two forms:

- **`"workdir"`** (default): the boundary is the folder passed to `base .` / `base <folder>`, different in each session.
- **a fixed path** (e.g. `"~"`, or an absolute path): the boundary is always that one, regardless of where `base` is launched. In this mode `base <folder>` must resolve inside that fixed path — otherwise startup is refused — and it only sets the initial folder in the TUI, not the boundary.

```toml
[defaults]
scope = "~"   # fixed boundary: always the entire home, wherever BASE is opened
```

Any request pointing outside the active boundary (`../`, absolute paths outside the root) is denied regardless of the rules above. Besides a wider fixed boundary, there are two ways to bring something in from outside, for different needs:

- **`Ctrl+F` (attach)** — a single external file, for that session: a read-only copy is placed inside `.base/attachments/`. It disappears with the session and must be repeated each time.
- **`[[mount]]` in guardrails** — an entire folder, always present, in every future session, independent of the active boundary (also useful for bringing in a path that a fixed `scope` wouldn't cover anyway, e.g. a shared folder outside the home).

A mount, by itself, grants nothing: like a tool with no rules, it is defined but inert until at least one `[[rule]]` references it with `mount = "<name>"` in place of `paths`. This is also the mechanism for deciding **which tools are usable in that mount**: in the example above only `fs-read` has a rule with `mount = "manuals"`, so there you can read but not write or create/delete.

Widening `scope` or adding a mount changes only the *where*: the *what* is still filtered by the `permission` rules — `fs-write` and `fs-manage` remain `confirm` even with `scope = "~"`, unless you change those rules too.

### Why `fs-read`, `fs-write`, `fs-manage` and `web-search` remain four separate files

Disabling a capability must be an isolated action, not an edit inside a shared block. Combining reading and writing in a single tool would have the same problem as a monolithic `fs` with all operations together: whitelisting reading as `auto` would bring writing along as `auto` too, with no way to separate them. For a set of agents meant only to read folders and produce reports, there are two levels, depending on how definitive the restriction needs to be:

- **Soft** (applies to that configuration, tool still installed): remove the `fs-write`, `fs-manage` and `web-search` rules from the guardrail file. The tools remain defined in `tools/`, but no agent can call them anymore.
- **Hard** (applies to the entire installation): delete `tools/fs-write.toml`, `tools/fs-manage.toml` and `tools/web-search.toml`. Since they are independent files, it's an `rm`, not an edit.

`fs-read` alone, with `fs-write`, `fs-manage` and `web-search` absent or not whitelisted, is exactly the set for a "reads and reports" agent: no writing, no deletion, no network access.

To extend a tool to all working directories with execute permission — the scenario of a personal Python script — two steps are enough:

1. Define the tool in `tools/<name>.toml` (`type = "script"`, file path, execution command).
2. Add a rule in a guardrail file with `paths = ["**"]` and the desired `permission`.

## TUI

Launching `base` without arguments, the first screen is folder selection: you navigate between subfolders, choose the one to use as the sandbox, and from there you enter the same interface you'd get with `base .` / `base <folder>` — only how you got to the folder changes, not what happens afterward.

Status panel per agent (Orchestrator, workers, tools). Plan proposed by the Orchestrator with approval (`[y] approve  [n] discard  [e] edit`), `+/-` diff on changes, snapshots and `/undo`, confirmation for every non-`auto` tool execution. Slash commands: `/plan`, `/agents`, `/perm`, `/attach`, `/undo`, `/clear`, `/quit`. `Esc` interrupts the current task, `Ctrl+D` closes BASE.

## Architecture

- **`config`**: at bootstrap generates missing files in `~/.config/base`; at every startup loads and validates the entire `*.toml` tree, with `schema_version` to handle future migrations.
- **`cli`**: minimal parsing, a single optional argument — the sandbox path.
- **`sandbox/fs`**: implementation behind the builtin tools `fs-read`, `fs-write`, `fs-manage`. Path canonicalization and symlink blocking are hardcoded, not configurable from files: `scope`, mounts and rules decide where and what is allowed inside this boundary, they don't replace it.

## Open / roadmap

- Hot-reload of configuration files while BASE is already open (file watcher).
- `/test <name>` in the TUI, to validate an agent or tool without restarting the session.
- Automatic migrations when `schema_version` changes.
- Ready-made guardrail presets (e.g. "read-only reporter") to activate with a flag, as an alternative to manually deleting tool files.
- A minimal `.toml` editor inside the demo, to try the "edit config → see the effect in the TUI" workflow without leaving the fake terminal.


```
# agent_config.toml

[agent]
name = "my_agent"
role = "orchestrator" # Can be "orchestrator" or "worker"
endpoint = "https://openrouter.ai/api/v1/chat/completions"
api_key = "sk-or-v1-124qwerty..."
model = "meta-llama/llama-3.1-70b-instruct"

# (Required for workers, ignored for orchestrators) 
# The Orchestrator reads this to know what tasks to route to this specific agent.
description = "Analyzes project structures and delegates tasks to specialized workers."

system_prompt = "You are the orchestrator. Output your plan as a JSON array."

[parameters]
temperature = 0.2
top_p = 0.9
max_tokens = 4096
context_window = 128000
# "json_object" forces valid JSON output (crucial for Orchestrator). "text" is standard for workers.
response_format = "json_object" 

[headers]
# Optional: Required by some providers like OpenRouter, ignored by Groq/Local if left empty.
"HTTP-Referer" = "https://github.com/yourusername/base"
"X-Title" = "BASE Terminal App"
```