# Testing

Run with `cargo test`. Tests split into unit tests (inline) and integration
tests (`tests/`).

## Unit tests
- `src/providers/mod.rs` (`#[cfg(test)] mod tests`) — `test_race_fastest_provider_wins`
  uses the `mock::MockProvider` (delay/fail/temperature knobs) to verify the
  fastest mock wins the race. The `Mock` variant of `WeatherProvider` exists
  solely for these tests.

## Integration tests (`tests/`)

These use `weatheroz` as a library (via `src/lib.rs`) or shell out to the built
binary. Each file:

| File | Covers |
|------|--------|
| `provider_tests.rs` | race orchestration, MET-Norway historical exclusion vs forecast inclusion, model (de)serialization, `NormalizedWeatherData` shape. Uses mock providers. |
| `logic_v2_tests.rs` | the 100ms short-circuit/consensus-window timing, consensus blending correctness, resilience when API keys are missing. |
| `geocoding_tests.rs` | query normalization / geocoding behavior. |
| `weather_cache_tests.rs` | weather cache get/save, freshness/TTL logic. |
| `integration_tests.rs` | **end-to-end against the real binary** — runs `./target/debug/weatheroz <args>` and asserts on stdout (e.g. expects "Istanbul", coordinates, JSON output, verbose race output, date validation, cache hit/miss, TUI). |

## Gotchas
- `integration_tests.rs` invokes `./target/debug/weatheroz`, so **build first**
  (`cargo build`) and they assume live network + the Open-Meteo geocoding/forecast
  APIs reachable (they assert on real Istanbul data). Expect failures offline.
- Cache-dependent tests point `WEATHEROZ_CACHE_PATH` at temp locations; the
  in-memory caches' path-change detection (`geocoding/mod.rs`,
  `weather_cache.rs`) exists so those redirects take effect within one process.
