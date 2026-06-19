# 🌤️ weatheroz & TUI Dashboard

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/Build-passing-brightgreen.svg)](Cargo.toml)

A blazing-fast, concurrent CLI and interactive TUI application written in Rust. It aggregates, races, caches, and renders real-time weather forecasts and historical data.

---

## 🚀 Key Features

* **🏎️ Parallel Race Orchestrator**: Fetches weather forecasts concurrently from **18 real HTTP providers** (Open-Meteo, MET Norway, wttr.in, Bright Sky, SMHI, FMI, NWS, Meteostat, Environment Canada, OpenWeatherMap, WeatherAPI, Weatherbit, Tomorrow.io, Visual Crossing, WeatherStack, Yandex, AccuWeather, Pirate Weather). The fastest successful response wins the race! Note: Open-Meteo already blends dozens of upstream models (DWD, NOAA/GFS, ECMWF, JMA, MET Norway, GEM…) internally, so a single winner can itself be a multi-model consensus.
* **🖥️ Interactive TUI Dashboard**: A gorgeous terminal user interface powered by `ratatui` and `crossterm`. Features real-time state visualization, asynchronous fetch updates, and a dedicated in-TUI logging panel displaying tracing logs.
* **📦 Smart Cache & Offline Mode**: 
  - **Weather Cache**: Persists data locally to minimize API hits. Stale data (older than 15 mins) automatically triggers a refresh, but acts as a resilient fallback in offline/network-failure modes.
  - **Geocoding Cache**: Remembers coordinates for locations for up to 30 days, avoiding redundant network lookups.
* **📡 Intelligent Location Resolution**: Automatically resolves location using **IP Geolocation** for auto-detection, CLI arguments, or manual fallback input.
* **📊 10 Visual Output Modes**: 
  - `default` (table), `compact` (ASCII card), `inline` (status bar), `json`, `emoji`, `ascii-banner`, `sparkline` (24h trend), `bordered-card`, `markdown`, `html-preview` (browser)
  - Machine-readable JSON output for automated integrations.
  - Flexibly display 24-hour standard projections or full 168-hour weekly reports (`--all-hours`).

---

## 🛠️ Quick Start (< 5 Min)

### Prerequisites
Make sure you have Rust and Cargo installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installation

Install directly from crates.io:
```bash
cargo install weatheroz
```

This puts the `weatheroz` command on your `PATH`.

<details>
<summary>Build from source</summary>

```bash
git clone https://github.com/ozkayhan/weatheroz.git
cd weatheroz
cargo build --release
```

The compiled binary will be available at `./target/release/weatheroz`.
</details>

---

## Usage

### 1. Interactive TUI Mode (Default)
Run without arguments to open the immersive interactive dashboard:
```bash
weatheroz
```
* **Controls**: Press `q` or `Esc` to safely exit.
* Automatically triggers geocoding, starts parallel requests, and draws a layout featuring current temperature, wind speed, precipitation, and live logs.

### 2. Standard CLI Mode
Request weather for a specific city:
```bash
weatheroz Istanbul
```

### 3. Core CLI Flags
- **Location**: positional argument (accepts multiple words, e.g., `"Central Park, NY"`)
- `-m, --mode <MODE>`: Visual output mode (`default`, `compact`, `inline`, `json`, `emoji`, `ascii-banner`, `sparkline`, `bordered-card`, `markdown`, `html-preview`)
- `-f, --from-date <YYYY-MM-DD>`: Start date for queries (default: today)
- `-t, --to-date <YYYY-MM-DD>`: End date for queries (default: today)
- `-d, --days <N>`: Forecast days (up to 40)
- `-e, --enrich`: Enable data enrichment (soil metrics, visibility, UV index, AQI)
- `-a, --all-hours`: Show 168 hours instead of 24
- `-j, --json-output`: Output JSON (shortcut for `-m json`)
- `-v, --verbose`: Show detailed performance logs and race timings
- `--minute`: Request minute-by-minute updates (where supported)
- `--cache-ttl <MINUTES>`: Override cache TTL (0 = disable cache)

### 4. Date-Range Historical and Forecast Queries
Query specific historical or future date ranges:
```bash
weatheroz London --from-date 2026-05-20 --to-date 2026-05-22
```

### 5. Verbose & Parallel Performance Tracing
See the results of the parallel provider race in real time:
```bash
weatheroz "New York" --verbose
```

### 6. Automated JSON Integrations
Output machine-readable raw JSON data:
```bash
weatheroz Tokyo --json-output
```

### 7. Visual Modes
Try different output styles:
```bash
weatheroz Istanbul -m sparkline     # 24-hour trend
weatheroz Istanbul -m html-preview  # Browser dashboard
weatheroz Istanbul -m emoji         # Rich colored output
```

---

## 📁 Project Structure

```
.
├── Cargo.toml            # Package configuration and dependencies
├── src/
│   ├── main.rs           # CLI entry point, logging registration & race loop
│   ├── cli.rs            # Command line argument parser definitions
│   ├── orchestrator.rs   # Core parallel race orchestrator & offline fallback logic
│   ├── weather_cache.rs  # Persistence and local cache operations
│   ├── geocoding/        # IP discovery and geocoding cache modules
│   ├── providers/        # 18 real HTTP weather API integrations
│   ├── shared/           # Models, DTO structures, and shared configuration services
│   ├── tui/              # TUI event loop, engine, rendering dashboard
│   └── output.rs         # 10 visual modes (table, JSON, sparkline, emoji, HTML, etc.)
├── tests/                # Comprehensive integration, cached fallbacks & provider race test suites
└── docs/                 # Detailed system architecture, development, and deployment docs
```

---

## 🤝 Contributing

Contributions are highly welcome! Please read our [CONTRIBUTING.md](CONTRIBUTING.md) to learn about our development process, branching model, conventional commit standards, and testing guidelines.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
All contributors are expected to uphold the [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) standards.
For vulnerability reporting, please review our [SECURITY.md](SECURITY.md).
