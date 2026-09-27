# base
BASE, acronimo che sta per Bounded Agent System Environment è un harness agentico, leggero e progettato per girare da terminale.

```
base/
├── Cargo.toml          # Il file di configurazione di Cargo (dipendenze, versione)
├── README.md           # Il file in cui metteremo la documentazione scritta prima
└── src/
    ├── main.rs         # Entry point: decide se eseguire comandi CLI o avviare la TUI
    │
    ├── cli/            # Gestione dei comandi da terminale (usando la libreria Clap)
    │   ├── mod.rs      # Espone il modulo cli al resto del programma
    │   └── args.rs     # Definisce la logica di `base add orc`, `base add wrk`, ecc.
    │
    ├── tui/            # Tutta l'interfaccia grafica terminale (usando Ratatui)
    │   ├── mod.rs
    │   ├── app.rs      # Lo STATO dell'app (es. cosa c'è scritto nel prompt, agenti attivi)
    │   ├── ui.rs       # Il DISEGNO dell'interfaccia (bordi, colori, layout)
    │   └── events.rs   # Gestione della tastiera (es. intercetta CTRL+F o Invio)
    │
    ├── agent/          # La logica di comunicazione con le IA
    │   ├── mod.rs
    │   ├── api.rs      # Il client HTTP (Reqwest) per parlare con Groq/OpenRouter
    │   ├── orc.rs      # Logica specifica dell'Orchestratore (creazione del piano)
    │   └── worker.rs   # Logica specifica dei Worker (esecuzione dei task)
    │
    ├── sandbox/        # Il sistema di sicurezza (Il "recinto")
    │   ├── mod.rs
    │   └── fs.rs       # Funzioni personalizzate per leggere/scrivere file in sicurezza
    │
    └── config/         # Gestione delle preferenze utente
        ├── mod.rs
        └── store.rs    # Salva e legge le API key e i Worker in un file locale (es. .base.json)
```


```
[user@computer ~]$ base add orc -c orc_config.toml
[+] Orchestrator added successfully. Model set as main coordinator.

[user@computer ~]$ base add wrk -c wrk1_conf.toml
[+] Worker added successfully.

[user@computer ~]$ base list wrk
ID   Type   Endpoint                                      Description
-------------------------------------------------------------------------------------------------------
1    wrk    http://localhost:8081/chat/completions/api    this is a agent to perform face recognition task
2    wrk    https://api.groq.com/openai/v1/chat/...       specialized in python code generation

[user@computer ~]$ base remove wrk 1
[-] Worker 1 removed successfully.

[user@computer ~]$ base remove orc
[-] Orchestrator removed successfully. The main coordinator is now unassigned.

[user@computer ~]$ cd Project_1
[user@computer Project_1]$ base .

--------------------------------------------------
  Alternatively, you can point directly to 
  a folder without entering it:
--------------------------------------------------

[user@computer ~]$ base Project_1

--------------------------------------------------
  Here's a basic mockup of the TUI
--------------------------------------------------

╭────────────── BASE: Bounded Agent System Environment ────────────────╮
│  Working Directory: /home/user/Project_1                   [Sandbox] │
├──────────────────────────────────────────────────────────────────────┤
│ [ORC] Orchestrator: Idle                                             │
│ [WRK] Worker 1: Face recognition task... (Ready)                     │
│                                                                      │
│  > Agent orchestration initialized. Ready for instructions.          │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│ ╭─ Prompt ─────────────────────────────────────────────────────────╮ │
│ │ Write your request here...                                       │ │
│ │                                                                  │ │
│ ╰──────────────────────────────────────────────────────────────────╯ │
│  [Enter] Submit  |  [Ctrl+F] Attach external file  |  [Esc] Quit     │
╰──────────────────────────────────────────────────────────────────────╯
```


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