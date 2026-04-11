# Run Crab Run — Design & Implementation Plan

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     run-crab-run                        │
│                  (binary crate, wires it all)           │
│                                                          │
│  ┌──────────┐  ┌───────────┐  ┌────────────────────┐  │
│  │  API     │  │ Scheduler  │  │  Webhook Receiver   │  │
│  │ (Axum)   │  │ (RRULE)    │  │  (POST /hook/...)   │  │
│  └────┬─────┘  └─────┬─────┘  └────────┬────────────┘  │
│       │              │                │                  │
│       └──────────────┼────────────────┘                  │
│                      ▼                                   │
│              ┌──────────────┐                            │
│              │ Job Executor │ ◄── debounce/coalesce      │
│              └──────┬───────┘                            │
│                     │                                    │
│              ┌──────▼───────┐                             │
│              │   Process    │ ── spawn CLI commands       │
│              │  Supervisor  │ ── collect stdout/stderr   │
│              └──────┬───────┘ ── measure cpu/ram         │
│                     │                                    │
│              ┌──────▼───────┐                             │
│              │   SQLite     │                             │
│              │  (Storage)   │                             │
│              └─────────────┘                             │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │           Leptos UI (Dashboard)                  │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Workspace Structure

```
run_crab_run/
├── Cargo.toml                  # Workspace root
├── config/default.toml         # App configuration
├── crates/
│   ├── rcr-core/               # Models, error types, config
│   │   └── src/
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       └── models/
│   │           ├── config.rs
│   │           ├── job.rs
│   │           ├── mod.rs
│   │           ├── run.rs
│   │           ├── trigger.rs
│   │           └── webhook.rs
│   ├── rcr-db/                 # SQLite via SQLx, migrations
│   │   └── src/
│   │       ├── jobs.rs
│   │       ├── lib.rs
│   │       ├── migrations.rs
│   │       ├── runs.rs
│   │       └── webhooks.rs
│   ├── rcr-runner/             # Execution engine, scheduler, monitoring
│   │   └── src/
│   │       ├── executor.rs     # Process spawner + debounce/coalesce
│   │       ├── lib.rs
│   │       ├── monitor.rs      # CPU/RAM sampling via sysinfo
│   │       └── scheduler.rs    # RRULE-based scheduler
│   ├── rcr-api/                # Axum HTTP server
│   │   └── src/
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       ├── routes/
│   │       │   ├── jobs.rs
│   │       │   ├── mod.rs
│   │       │   ├── runs.rs
│   │       │   ├── trigger.rs
│   │       │   └── webhooks.rs
│   │       └── state.rs
│   ├── rcr-ui/                 # Leptos WASM frontend (scaffolded)
│   │   └── src/
│   │       ├── app.rs
│   │       └── lib.rs
│   └── run-crab-run/           # Binary entry point
│       └── src/
│           └── main.rs
```