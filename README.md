# base
BASE, acronimo che sta per Bounded Agent System Environment è un harness agentico, leggero e progettato per girare da terminale.


## comandi:
```
[user@computer ~]$ base add orc -s 'http://localhost:8080/chat/completions/api' -k '124qwerty...'
[+] Orchestrator added successfully. Model set as main coordinator.

[user@computer ~]$ base add wrk -s 'http://localhost:8081/chat/completions/api' -k '124qwerty...' -d 'this is a agent to perform face recognition task'
[+] Worker added successfully. The Orchestrator will use this description to route tasks.

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
