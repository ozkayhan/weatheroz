# Code Map

Start here to find where to change something. All paths relative to repo root.

## Entry points
- `src/main.rs` — binary entry (`#[tokio::main] async fn main`). Arg parsing,
  tracing setup, location resolution, TUI event loop, final render dispatch.
- `src/lib.rs` — library root; just `pub mod` declarations re-exporting every
  module (so the crate is usable as a lib and by `tests/`).

## Source modules (`src/`)

| Path | Responsibility |
|------|----------------|
| `cli.rs` | `Args` (clap derive), `Mode` enum + `as_str()`, `validate_date`, help styling. **Change CLI flags here.** |
| `orchestrator.rs` | `run_orchestrator` + `OrchestratorResult`. Wires geocoding → cache → race → save. |
| `providers/mod.rs` | `WeatherProvider` enum, `ProviderStat`, `run_weather_race`, `blend_weather_data`, the `mock` test provider, race unit test. **Core race/blend logic.** |
| `providers/base.rs` | `FetchContext<'a>` and the `BaseWeatherProvider` trait (`name`, `async fetch`). |
| `providers/models.rs` | One re-export line → `shared/domain/models`. |
| `providers/<name>.rs` | One file per provider (18 of them). See [providers.md](providers.md). |
| `geocoding/mod.rs` | `GeocodedLocation`, `resolve_location` (name→coords, Open-Meteo geocoding), `resolve_ip_location` (ip-api.com), `normalize_query`, 30-day geo cache. |
| `weather_cache.rs` | `get_cached_weather` / `save_cached_weather`, 15-min TTL weather cache keyed by `lat:lon:start:end`. |
| `output.rs` | All CLI rendering: `print_hourly_table`, `print_location_info`, `print_date_range_info`, `print_offline_warning`, `print_error_block`, and `render_mode` (the 10-mode `match`). |
| `tui/mod.rs` | `ProcessState`, `ProviderState`, `SharedState` type, `TuiLoggingLayer`, `conditional_sleep`. |
| `tui/renderer.rs` | `draw_dashboard` — the ratatui layout drawn each frame. |
| `shared/domain/config.rs` | `AppConfig`, `ConfigService` trait, `JsonConfigService` (file + env loading). |
| `shared/domain/models.rs` | `HourlyPoint`, `AQIData`, `NormalizedWeatherData` — the canonical data types. |
| `shared/domain/mod.rs`, `shared/mod.rs` | module wiring only. |

## "Where do I go to change X?"
- **Add/rename a CLI flag** → `src/cli.rs` (and read it in `main.rs`/`orchestrator.rs`).
- **Add an output mode** → add a `Mode` variant + `as_str` in `cli.rs`, add a
  match arm in `output.rs::render_mode`.
- **Add a weather provider** → new `src/providers/<name>.rs`, register in
  `providers/mod.rs` (enum + `name`/`requires_key`/`fetch` arms + `mod`).
  Full recipe in [providers.md](providers.md).
- **Change race/fallback defaults** → `orchestrator.rs` (`default_race`,
  `default_fallback`) and `config.rs` (`AppConfig::default`).
- **Change blending math** → `providers/mod.rs::blend_weather_data`.
- **Change a cache TTL / file location** → `weather_cache.rs` (15 min),
  `geocoding/mod.rs` (30 days), `config.rs` paths.
- **Change the TUI layout** → `tui/renderer.rs`.

## Tests (`tests/`)
`geocoding_tests.rs`, `integration_tests.rs`, `logic_v2_tests.rs`,
`provider_tests.rs`, `weather_cache_tests.rs`. See [testing.md](testing.md).

## Docs & meta (not code)
- `README.md` — user-facing readme. `principles.md` — project principles.
- `docs/` — human prose docs (`ARCHITECTURE.md`, `DEPLOYMENT.md`,
  `DEVELOPMENT.md`, `CHANGELOG.md`, `cli_usage_guide.md`, `api_docs/`). These are
  hand-maintained narrative docs; `valid-docs/` (this folder) is the
  code-grounded map.
- `docsjson/docs.json` — currently `{}` (empty placeholder).
- `.agents/`, `.github/`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`,
  `LICENSE` — contributor/OSS scaffolding.
