# valid-docs — weatheroz

> **Synced to:** `1cb1b26` — 2026-07-01
> Regenerate/refresh with the `update-docs` skill. These docs describe the
> codebase at the commit above; if HEAD has moved, re-run to reconcile.

Read this folder before exploring the source — it maps the whole codebase.

`weatheroz` is a Rust CLI + TUI that races ~18 real HTTP weather providers in
parallel, blends the fast winners into a consensus forecast, caches it, and
renders it in 10 output modes.

## Documents
- [overview.md](overview.md) — what this project is and the 30-second model
- [tech-stack.md](tech-stack.md) — languages, crates, build/test/run tooling
- [architecture.md](architecture.md) — components, the race/blend pipeline, design decisions
- [code-map.md](code-map.md) — where everything lives (start here to navigate)
- [providers.md](providers.md) — the provider trait, the enum, all 18 providers, how to add one
- [data-model.md](data-model.md) — core structs, config, cache file formats
- [output-modes.md](output-modes.md) — the 10 render modes and the TUI dashboard
- [configuration.md](configuration.md) — config file, env vars, cache/log paths
- [testing.md](testing.md) — test layout and how to run them
