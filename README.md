# 🌤️ Weather OZ & TUI Dashboard

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/Build-passing-brightgreen.svg)](Cargo.toml)

A blazing-fast, concurrent CLI and interactive TUI application written in Rust. It aggregates, races, caches, and renders real-time weather forecasts and historical data.

---

## 🚀 Key Features

* **🏎️ Parallel Race Orchestrator**: Fetches weather forecasts concurrently from multiple API providers (**Open-Meteo**, **MET Norway**, and **wttr.in**). The fastest successful response wins the race!
* **🖥️ Interactive TUI Dashboard**: A gorgeous terminal user interface powered by `ratatui` and `crossterm`. Features real-time state visualization, asynchronous fetch updates, and a dedicated in-TUI logging panel displaying tracing logs.
* **📦 Smart Cache & Offline Mode**: 
  - **Weather Cache**: Persists data locally to minimize API hits. Stale data (older than 15 mins) automatically triggers a refresh, but acts as a resilient fallback in offline/network-failure modes.
  - **Geocoding Cache**: Remembers coordinates for locations for up to 30 days, avoiding redundant network lookups.
* **📡 Intelligent Location Resolution**: Automatically resolves location using **IP Geolocation** for auto-detection, CLI arguments, or manual fallback input.
* **📊 Comprehensive Layouts**: 
  - Beautiful tabular representation using `comfy-table`.
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
Clone the repository and build the binary in release mode:
```bash
git clone https://github.com/ozkayhan/weather.git
cd weather
cargo build --release
```

The compiled binary will be available at `./target/release/weather_oz`.

---

## Usage

### 1. Interactive TUI Mode (Default)
Run without disabling terminals to open the immersive interactive dashboard:
```bash
./target/release/weather_oz
```
* **Controls**: Press `q` or `Esc` to safely exit the Alternate Screen.
* Automatically triggers geocoding, starts parallel requests, and draws a layout featuring current temperature, wind speed, precipitation, and live logs.

### 2. Standard CLI Mode
Request weather for a specific city:
```bash
./target/release/weather_oz --location "Istanbul"
```

### 3. Date-Range Historical and Forecast Queries
Query specific historical or future date ranges:
```bash
./target/release/weather_oz --location "London" --from-date 2026-05-20 --to-date 2026-05-22
```

### 4. Verbose & Parallel Performance Tracing
See the results of the parallel provider race and cache hits/misses in real time:
```bash
./target/release/weather_oz --location "New York" --verbose
```
*Output snippet:*
```
🔍 Cache Miss: New York
🚀 Starting parallel race for: Open-Meteo, MET Norway, wttr.in

🏁 Thread Details:
  - MET Norway: SUCCESS (124.5ms)
  - Open-Meteo: SUCCESS (89.2ms)
  - wttr.in: SUCCESS (210.4ms)

🏆 Winner: Open-Meteo (89.2ms)
```

### 5. Automated JSON Integrations
Output machine-readable raw JSON data:
```bash
./target/release/weather_oz --location "Tokyo" --json-output
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
│   ├── providers/        # API integrations (OpenMeteo, MetNorway, wttr.in)
│   ├── shared/           # Models, DTO structures, and shared configuration services
│   ├── tui/              # TUI event loop, engine, rendering dashboard
│   └── output.rs         # Formatted tables, JSON outputs, error renderers
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
