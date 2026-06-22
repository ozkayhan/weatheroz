# Configuration

Config loading lives in `src/shared/domain/config.rs`
(`JsonConfigService::load_config`). Order: read the JSON file if present (bad/
missing → `AppConfig::default()`), then apply env-var overrides.

## Config file

Path: `$WEATHEROZ_CONFIG_PATH`, else `~/.config/weatheroz/config.json`.
Shape = `AppConfig` (see [data-model.md](data-model.md)). Defaults:

| Field | Default |
|-------|---------|
| `default_location` | `None` |
| `temperature_unit` | `"celsius"` |
| `api_keys` | `{}` |
| `race_providers` | `["Open-Meteo", "MET Norway", "wttr.in"]` |
| `fallback_providers` | `["Bright Sky"]` |
| `cache_ttl_minutes` | `15` |
| `forecast_days` | `7` |
| `minute_updates` | `false` |
| `enrich_data` | `false` |

## Environment variables

Scalar overrides (applied after the file is read):

| Var | Overrides |
|-----|-----------|
| `WEATHER_DEFAULT_LOCATION` | `default_location` |
| `WEATHER_TEMPERATURE_UNIT` | `temperature_unit` |
| `WEATHER_CACHE_TTL_MINUTES` | `cache_ttl_minutes` (parsed u64) |
| `WEATHER_FORECAST_DAYS` | `forecast_days` (parsed u32) |
| `WEATHER_MINUTE_UPDATES` | `minute_updates` (`true`/`1`) |
| `WEATHER_ENRICH_DATA` | `enrich_data` (`true`/`1`) |

API keys: any env var named `WEATHER_KEY_<NAME>` becomes an `api_keys` entry —
the name is uppercase-with-`_`, converted to `-` (e.g.
`WEATHER_KEY_OPEN_WEATHER_MAP`). The key the race looks up is the string from a
provider's `requires_key()` (e.g. `OpenWeatherMap`, `WeatherAPI`,
`Pirate Weather`) — match the casing/spelling accordingly when using config
`api_keys`.

Path/behavior vars (read elsewhere, not in `AppConfig`):

| Var | Effect | Read in |
|-----|--------|---------|
| `WEATHEROZ_CONFIG_PATH` | override config file path | `config.rs` |
| `WEATHEROZ_CACHE_PATH` | override geo/weather cache path | `geocoding/mod.rs`, `weather_cache.rs`, `main.rs` |
| `HOME` | base for default `~/.config` and `~/.cache` paths | several |
| `RUST_LOG` (+ default-env filter) | stderr tracing level (plain CLI only) | `main.rs` |

## CLI flags (`src/cli.rs`)

Flags override config per-run: positional `location`, `-f/--from-date`,
`-t/--to-date`, `-a/--all-hours`, `-j/--json-output`, `-v/--verbose`,
`-m/--mode`, `-e/--enrich`, `-d/--days`, `--minute`, `--cache-ttl` (0 disables
cache), `--completions <shell>` (prints completion script and exits). Resolution
precedence in `orchestrator.rs`: CLI flag → config value → hard-coded default.

## Files written at runtime
See [data-model.md](data-model.md) — geo cache, weather cache, and a log file
under `~/.cache/weatheroz/`.
