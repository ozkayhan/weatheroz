# Overview

`weatheroz` is a command-line and terminal-UI weather application written in
Rust. Given a location (typed, IP-detected, or coordinates), it fetches a
forecast and prints it.

## 30-second mental model

1. **Resolve** the location → lat/lon (`src/geocoding/`, via Open-Meteo
   geocoding API or `ip-api.com` auto-detection).
2. **Check cache** for that lat/lon + date range (`src/weather_cache.rs`). Fresh
   hit → return it, skip the network.
3. **Race** a group of providers concurrently (`src/providers/mod.rs::run_weather_race`).
   The first success starts a 100ms "consensus window"; every provider that
   answers inside it is **blended** (averaged per hour) into one forecast.
   Falls back to a second provider group if the primary group all fail, and to
   stale cache if the network is down entirely.
4. **Render** the result in one of 10 modes (`src/output.rs`) — or drive a live
   `ratatui` dashboard (`src/tui/`) when running interactively.

## What it is / isn't

- **Is:** a single-binary CLI tool (`cargo install weatheroz`). No server, no
  database, no persistent process. State lives in JSON files under
  `~/.cache/weatheroz/` and `~/.config/weatheroz/`.
- **Isn't:** a library you embed (though `src/lib.rs` re-exports the modules), a
  web service, or a multi-user system.

## Two run modes

- **TUI (default)** when stdout is a terminal and neither `--json-output` nor
  `--verbose` is set: full-screen dashboard with per-provider progress and a
  live log panel.
- **Plain CLI** otherwise: prints a table (or the chosen `--mode`) to stdout.

See [architecture.md](architecture.md) for the pipeline in detail and
[code-map.md](code-map.md) to find any piece of it.
