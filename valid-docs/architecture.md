# Architecture

A single binary. `main` orchestrates a linear pipeline; the only concurrency is
the provider race and a couple of fire-and-forget cache writes.

## Top-level flow (`src/main.rs`)

1. **Parse args** (`Args::parse()`, `src/cli.rs`). If `--completions <shell>`,
   print the completion script and exit.
2. **Load config** (`JsonConfigService::load_config`, `src/shared/domain/config.rs`)
   — JSON file + env-var overrides, falls back to `AppConfig::default()`.
3. **Validate** `--from-date` / `--to-date` (`YYYY-MM-DD`).
4. **Decide TUI vs plain:** `use_tui = !json_output && !verbose && stdout.is_terminal()`.
5. **Set up tracing** (`src/main.rs::main`): up to three layers — a file layer
   (`~/.cache/weatheroz/weather.log`, INFO), a stderr layer (only in non-TUI,
   gated by `RUST_LOG`/`EnvFilter`), and a `TuiLoggingLayer` that feeds
   `ProcessState.last_log` (only in TUI).
6. **Build one shared `reqwest::Client`** (6s request timeout, 3s connect
   timeout), wrapped in `Arc`.
7. **Resolve the location query:** CLI positional args, else IP auto-detection
   (`resolve_ip_location`), else interactive stdin prompt.
8. **Run the orchestrator** (`src/orchestrator.rs::run_orchestrator`). In TUI
   mode it runs on a `tokio::spawn`ed task while the main thread draws the
   dashboard and polls for `q`/`Esc`.
9. **Render** the `OrchestratorResult` via `src/output.rs` (table for `default`,
   else `render_mode`).

## The orchestrator (`src/orchestrator.rs::run_orchestrator`)

Returns `OrchestratorResult { resolved_location, weather_data, stats,
winner_name, is_offline, cached_timestamp }`.

1. Compute the date range (`--days` / config `forecast_days` / explicit
   from/to; default 7 days).
2. Resolve location (reuse `initial_location` from IP detection, else
   `geocoding::resolve_location`).
3. **Cache check** (`weather_cache::get_cached_weather`) keyed by
   `lat:lon:start:end`. Fresh → return immediately with `winner_name = "Cache"`.
   Stale → kept aside as `offline_data` for the network-failure path. TTL from
   `--cache-ttl` / config / default 15 min; `0` disables the cache.
4. Build the list of all 18 `WeatherProvider` variants and a `FetchContext`
   (client, lat/lon, dates, api_keys, enrich, days, minute_resolution).
5. **Run the race** (`providers::run_weather_race`) with a `race_list` (primary
   group, default `Open-Meteo, MET Norway, wttr.in`) and a `fallback_list`
   (default `Bright Sky`).
6. On success → `save_cached_weather` (fire-and-forget `tokio::spawn`) and
   return. On failure → fall back to stale `offline_data` (`is_offline = true`),
   else propagate the error.

## The race + blend (`src/providers/mod.rs`)

`run_weather_race`:

- Splits the provider list into **main** and **fallback** groups by matching
  names against `race_list` / `fallback_list` (case-insensitive). If no name
  matches the main group, *all* providers become the main group.
- `run_race_group` filters out ineligible providers: MET Norway for historical
  queries, and any provider whose `requires_key()` API key is absent from
  config. Eligible providers run via `FuturesUnordered`.
- **Consensus window:** the first successful response starts a 100ms
  `tokio::time::sleep` deadline (`tokio::select!`). Every success that lands
  before the deadline (or before the stream drains) is collected. Single
  eligible provider → short-circuits immediately.
- `blend_weather_data` merges the collected responses: hourly points grouped by
  timestamp (`BTreeMap`), numeric fields **averaged**, `weather_code` by
  **majority vote**, optional fields (uv/visibility/soil/aqi) averaged over
  whoever reported them, `is_day` by majority. One response → returned as-is;
  many → `provider_name = "Consensus Blended (a, b, …)"`.
- Returns `(NormalizedWeatherData, HashMap<String, ProviderStat>, winner_name)`.
  Primary group failure triggers the fallback group; both failing is an error.

### Why these choices
- **Race + 100ms window, not "fastest wins":** the fastest response still wins
  latency-wise, but the short window cheaply folds in any near-simultaneous
  responses for a consensus average without waiting on slow providers.
- **Blend by averaging:** smooths single-provider outliers. Note Open-Meteo is
  itself a multi-model blend upstream, so even a lone winner is a consensus.
- **Stale-cache fallback:** makes the tool usable offline / during API outages.
- **One shared `reqwest::Client`:** connection pool reuse across the race.

## State & side effects

- **No database.** Persistence is three JSON files under `~/.cache/weatheroz/`
  (geocoding cache, weather cache) and `~/.config/weatheroz/config.json`, plus a
  log file. See [configuration.md](configuration.md) and [data-model.md](data-model.md).
- **In-memory caches** are process-global `OnceLock<RwLock<HashMap…>>`
  (`geocoding/mod.rs`, `weather_cache.rs`) that lazily load from disk and reload
  if the cache path env var changes (the path-change check exists mainly so
  tests can point at temp dirs).
- **TUI state** is a single `Arc<Mutex<ProcessState>>` (`src/tui/mod.rs`) written
  by the orchestrator/geocoding/race and read by the renderer each frame.
