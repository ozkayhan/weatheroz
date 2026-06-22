# Output Modes & TUI

## CLI render modes (`src/output.rs`)

`Mode` is defined in `src/cli.rs` (variant names map to kebab-case CLI values
via `Mode::as_str`). `main.rs` picks the mode string; `default` uses the table
path directly, everything else goes through `output.rs::render_mode`, which is a
`match mode.to_lowercase()`:

| `--mode` value | What it prints |
|----------------|----------------|
| `default` | comfy-table hourly table (`print_hourly_table`); rows capped at 24 (or 168 with `--all-hours`). Also prints location + date-range headers. |
| `compact` | ASCII-art weather card with temp / condition / wind / humidity / cloud. |
| `inline` | one-line status-bar summary. |
| `json` | machine-readable JSON (also via `-j/--json-output`). |
| `emoji` | rich colored emoji output. |
| `ascii-banner` | large ASCII banner. |
| `sparkline` | 24-hour temperature trend as a sparkline. |
| `bordered-card` | bordered summary card. |
| `markdown` | Markdown-formatted output. |
| `html-preview` | builds an HTML dashboard and opens it in the browser. |

Helper functions in `output.rs`: `print_location_info`, `print_date_range_info`,
`print_offline_warning` (shown when `is_offline`), `print_error_block`,
`get_ascii_art`, and a weather-code → (emoji, description) lookup (WMO codes
0–99, the same codes Open-Meteo uses).

`render_mode` receives the resolved location, hourly data, date range, a
`data_type` (`historical` / `forecast` / `mixed`, computed in `main.rs` from the
dates vs today), offline flag, cached timestamp, and provider name.

## TUI dashboard (`src/tui/`)

Active when `use_tui` (terminal stdout, no `--json-output`/`--verbose`).

- **State:** `ProcessState` (`tui/mod.rs`) in an `Arc<Mutex<…>>` (`SharedState`).
  Tracks `query`, `resolved_location`, `cache_status`, `global_progress`, step
  flags (`step_parsing`/`geocoding`/`race`/`blending`), a `providers`
  map of `ProviderState { status, time, error }`, and `last_log`.
- **Wiring:** `main.rs` spawns the orchestrator on a tokio task and loops:
  lock state → `renderer::draw_dashboard(frame, &state)` → poll crossterm 100ms
  for `q`/`Esc` → check if the task finished. The orchestrator, geocoding, and
  race mutate the shared state as they progress (e.g. setting per-provider
  `running`/`completed`/`failed`).
- **Logging into the UI:** `TuiLoggingLayer` (a `tracing_subscriber::Layer`)
  captures each event's `message` field into `ProcessState.last_log`, shown in
  the dashboard's log panel.
- **`conditional_sleep`:** small deliberate delays (only when a `SharedState`
  exists, i.e. TUI mode) so the animated progress is visible; no-op in plain CLI.
- **Rendering:** `tui/renderer.rs::draw_dashboard` builds the ratatui layout each
  frame from the current `ProcessState`.
