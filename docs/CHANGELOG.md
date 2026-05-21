# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-05-21

### Added
- **Parallel Race Orchestrator**: Concurrent HTTP queries across `OpenMeteo`, `MetNorway`, and `wttr` weather services, outputting thread detail metrics in verbose modes.
- **Interactive TUI Dashboard**: Implemented Crossterm and Ratatui alternate screens visualizing live current weather grids, precipitations, and operational trace logs on screen.
- **Weather & Geocoding Caching**: Implemented automatic 15-minute validity weather data saves and 30-day geocoding query caches to improve performance and enable off-grid operations.
- **IP Location Auto-Resolution**: IP-based city geocoding to auto-detect device location when no arguments are provided.
- **Customized CLI Formatting**: Detailed `comfy-table` layouts, machine-readable JSON rendering, and options to output 168-hour weekly reports.
- **Developer Kits**: Integrated detailed unit and integration tests covering geocoding, fallbacks, CLI inputs, and race speeds.
