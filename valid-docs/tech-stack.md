# Tech Stack

- **Language:** Rust, edition 2021. Binary crate `weatheroz` (also exposes a
  library via `src/lib.rs`).
- **Async runtime:** `tokio` 1 (`features = ["full"]`). `main` is `#[tokio::main]`.
- **Build profile:** `[profile.release]` uses `opt-level=3`, `lto=true`,
  `codegen-units=1`, `strip=true` (small, optimized binary for `cargo install`).

## Dependencies (`Cargo.toml`)

| Crate | Version | Used for |
|-------|---------|----------|
| `tokio` | 1 (full) | async runtime, `tokio::select!`, `RwLock`, fs |
| `reqwest` | 0.12 (`json`, `rustls-tls`, no default features) | all HTTP calls |
| `serde` / `serde_json` | 1.0 | (de)serialize API responses, config, caches |
| `clap` / `clap_complete` | 4.0 / 4.5 | CLI arg parsing + shell completions |
| `chrono` | 0.4 (`serde`) | date parsing/arithmetic, today's date |
| `deunicode` | 1.3 | ASCII-fold location queries for geocoding fallback |
| `ratatui` | 0.26 | TUI dashboard rendering |
| `crossterm` | 0.27 | terminal control, key events for the TUI |
| `comfy-table` | 7.1 | table rendering in CLI output modes |
| `futures-util` | 0.3 | `FuturesUnordered` for the provider race |
| `urlencoding` | 2.1 | encode location query into geocoding URL |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 (`fmt`,`registry`,`env-filter`) | logging to file, stderr, and the in-TUI log panel |

## Tooling

- **Build/run:** `cargo build`, `cargo build --release`, `cargo run -- <args>`.
- **Test:** `cargo test` (unit tests in `src/providers/mod.rs`, integration
  tests in `tests/`). See [testing.md](testing.md).
- **CI:** `.github/workflows/rust.yml` (GitHub Actions for Rust).
- **Install:** published to crates.io as `weatheroz` → `cargo install weatheroz`.

No async-trait, no derive_more, no error-handling crate — errors are
`Box<dyn std::error::Error + Send + Sync>`, and the provider trait uses native
`async fn` in traits (`#[allow(async_fn_in_trait)]`).
