# Data Model

No database. All types are Rust structs; persistence is JSON files on disk.

## Core weather types (`src/shared/domain/models.rs`)

Re-exported through `src/providers/models.rs`.

```rust
struct NormalizedWeatherData { provider_name: String, hourly: Vec<HourlyPoint> }

struct HourlyPoint {
    time: String,                 // ISO timestamp, the blend/group key
    temperature, apparent_temperature, precipitation_probability, precipitation,
    humidity, wind_speed, wind_direction, cloud_cover: f64,
    weather_code: i32,
    aqi: Option<AQIData>,
    // enriched, optional (provider- and --enrich-dependent):
    uv_index: Option<f64>, is_day: Option<bool>, visibility: Option<f64>,
    soil_temperature: Option<f64>, soil_moisture: Option<f64>,
}

struct AQIData { co, no2, o3, so2, pm2_5, pm10: Option<f64> }
```

Every provider's `fetch` returns `NormalizedWeatherData`; the blender consumes
and produces it; the renderers display it. It's the spine of the app.

## Geocoding (`src/geocoding/mod.rs`)

```rust
struct GeocodedLocation { name, country, admin1: String, latitude, longitude: f64 }
struct CacheEntry { data: GeocodedLocation, timestamp: f64 }  // unix secs
```
Lat/lon are rounded to 4 decimals on resolve (matches the cache key precision).

## Orchestrator result (`src/orchestrator.rs`)

```rust
struct OrchestratorResult {
    resolved_location: GeocodedLocation,
    weather_data: NormalizedWeatherData,
    stats: HashMap<String, ProviderStat>,   // per-provider timing/success
    winner_name: String,                     // provider, "Consensus Blended (…)", or "Cache"
    is_offline: bool,                        // true => served stale cache
    cached_timestamp: Option<f64>,
}
```
`ProviderStat { time_ms: f64, success: bool, error: Option<String> }` (in `providers/mod.rs`).

## Config (`src/shared/domain/config.rs`)

`AppConfig` is serde-(de)serialized from `~/.config/weatheroz/config.json`:
`default_location`, `temperature_unit`, `api_keys: HashMap<String,String>`,
`race_providers`, `fallback_providers`, `cache_ttl_minutes`, `forecast_days`,
`minute_updates`, `enrich_data`. Defaults in `AppConfig::default`. See
[configuration.md](configuration.md).

## On-disk files (all JSON, all under `~/`)

| File | Type serialized | TTL | Written by |
|------|-----------------|-----|------------|
| `.config/weatheroz/config.json` | `AppConfig` | — | `JsonConfigService::save_config` |
| `.cache/weatheroz/geo_cache.json` | `HashMap<String, CacheEntry>` keyed by lowercased query | 30 days | `geocoding/mod.rs` |
| `.cache/weatheroz/weather_cache.json` | `HashMap<String, WeatherCacheEntry>` keyed by `lat:lon:start:end` (4-dp) | 15 min (configurable) | `weather_cache.rs` |
| `.cache/weatheroz/weather.log` | tracing INFO text | append | `main.rs` file layer |

`WeatherCacheEntry { data: NormalizedWeatherData, timestamp: f64 }`. Cache paths
are overridable with `WEATHEROZ_CACHE_PATH` / `WEATHEROZ_CONFIG_PATH`.
