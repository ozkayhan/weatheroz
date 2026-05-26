# Production Deployment Guide

This document describes how to compile, optimize, package, and distribute the Weatheroz & TUI binary for production usage.

---

## ⚡ Production Builds

To compile the application with full optimizations, use Cargo's release profile:

```bash
cargo build --release
```

The resulting optimized executable will be generated at `target/release/weatheroz`.

---

## 🚀 Binary Optimization Options

To produce the smallest and fastest binary sizes, you can configure the `[profile.release]` section in `Cargo.toml`. Standard optimization profiles include:

```toml
[profile.release]
opt-level = 3            # Full compiler speed optimizations
lto = true               # Enable Link Time Optimization (cross-module optimization)
codegen-units = 1        # Reduce parallel compilation blocks to maximize optimization
panic = "abort"          # Remove stack unwinding code to shave binary size (optional)
strip = true             # Strip symbols and debug info from final binary
```

*Note: Enabling LTO and reducing codegen units will increase compilation times, but will produce faster, smaller executables.*

---

## 🖥️ Cross-Compilation

To distribute pre-compiled binary packages for different platforms, you can cross-compile:

### 1. Compile for Linux (from macOS/Windows)
The easiest way is using `cross`, a Docker-based cross-compiling toolchain for Rust:
```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --target x86_64-unknown-linux-gnu --release
```

### 2. Compile for macOS (Apple Silicon & Intel)
Build universal binaries that run natively on both hardware architectures:
```bash
# Add targets
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# Compile release files
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Combine into a single fat binary using lipo
lipo -create \
  target/aarch64-apple-darwin/release/weatheroz \
  target/x86_64-apple-darwin/release/weatheroz \
  -output target/release/weatheroz_universal
```

---

## 📁 Production Cache & Paths

At runtime, the binary manages three main locations:
1. **Cache Folder**:
   - Resolved coordinates and local forecasts are kept in `~/.cache/weatheroz/`.
   - Ensure the user running the application has read/write permission to their `$HOME` directory.
2. **Log File**:
   - Live system events are appended to `~/.cache/weatheroz/weather.log`.
   - In production, you can set up `logrotate` to periodically compress or delete old log lines.
3. **Configuration file**:
   - Looks for an option configuration file in a standard standard JSON config service.
