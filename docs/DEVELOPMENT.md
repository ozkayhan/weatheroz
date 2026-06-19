# Developer Guide

This document outlines the local setup, tooling, and testing instructions for developers working on the weatheroz & TUI codebase.

---

## 💻 Environment Setup

### Prerequisites
Make sure you have installed:
- **Rust Toolchain**: `rustup` is highly recommended to manage toolchains.
- **Cargo dependencies**: Standard libraries and networking elements are fetched automatically.

---

## 🛠️ Typical Workflow

### 1. Build and Run
- Compile in debug mode for rapid iterations:
  ```bash
  cargo build
  ```
- Run the local CLI or TUI directly:
  ```bash
  # Launch TUI dashboard (requires interactive terminal)
  cargo run
  
  # Run in verbose CLI mode
  cargo run -- --location "London" --verbose
  
  # Run for a date range in JSON format
  cargo run -- --location "Paris" --from-date 2026-05-20 --to-date 2026-05-22 --json-output
  ```

### 2. Code Quality & Linting
We enforce strict style standards:
- Always run the formatter before checking in files:
  ```bash
  cargo fmt --all
  ```
- Run the compiler lints and clippy checks:
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
  Ensure all warnings are resolved; warnings will cause CI pipeline builds to fail.

---

## 🧪 Testing

We maintain a comprehensive test suite across units, integrations, cache architectures, and provider race behaviors:

### Running Tests
- **Run all tests**:
  ```bash
  cargo test
  ```
- **Run unit tests only**:
  ```bash
  cargo test --lib
  ```
- **Run specific integration suites**:
  ```bash
  # Geocoding caches and coordinate precision
  cargo test --test geocoding_tests
  
  # Orchestrator race, CLI flags, and layouts
  cargo test --test integration_tests
  
  # Cache saving, retrieval, and offline fallback fallbacks
  cargo test --test weather_cache_tests
  ```

---

## 📝 Debugging & Logging

The application utilizes `tracing` to capture and stream operational logs.

### Tracing Log Files
Logs are written in standard formats to:
- **macOS/Linux**: `~/.cache/weatheroz/weather.log`
- **Fallback**: `./.cache/weatheroz/weather.log`

You can tail this file during development to inspect parallel requests, geocoding lookups, and cache resolutions:
```bash
tail -f ~/.cache/weatheroz/weather.log
```

### Environment Filters
When running the application outside of TUI mode, you can control console log details using the `RUST_LOG` environment variable:
```bash
RUST_LOG=info cargo run -- --location "Berlin"
```
