# Providers

Each provider fetches a forecast from one upstream API and normalizes it to
`NormalizedWeatherData`. All live in `src/providers/`.

## The contract (`src/providers/base.rs`)

```rust
pub trait BaseWeatherProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch(&self, ctx: &FetchContext<'_>)
        -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>>;
}
```

`FetchContext<'a>` carries everything a fetch needs: `client: Arc<reqwest::Client>`,
`lat`, `lon`, `start_date`/`end_date` (`&str`, `YYYY-MM-DD`), `api_keys`,
`enrich`, `days`, `minute_resolution`.

## The dispatch enum (`src/providers/mod.rs`)

Providers are **not** boxed trait objects — they're a `WeatherProvider` enum
(one variant per provider, plus `Mock` for tests). Three `match` blocks forward
to the inner type: `name()`, `requires_key()`, and `fetch()`. This keeps the
race loop on concrete `Clone` types without `dyn`/`async-trait`.

`requires_key()` returns `Some(key_name)` for providers that need an API key;
the race skips them when that key isn't in `config.api_keys`.

## The 18 providers

| File | `name()` | API key? | Notes |
|------|----------|----------|-------|
| `openmeteo.rs` | `Open-Meteo` | no | forecast + archive endpoints, enriched & minutely vars; default primary |
| `metnorway.rs` | `MET Norway` | no | forecast only — **excluded for historical** queries; default primary |
| `wttr.rs` | `wttr.in` | no | default primary |
| `brightsky.rs` | `Bright Sky` | no | default fallback (DWD data) |
| `smhi.rs` | `SMHI` | no | Sweden |
| `fmi.rs` | `FMI (Finland)` | no | Finland |
| `nws.rs` | `NWS` | no | US National Weather Service |
| `meteostat.rs` | `Meteostat` | **yes** | |
| `envcanada.rs` | `Environment Canada` | no | |
| `openweathermap.rs` | `OpenWeatherMap` | **yes** | |
| `weatherapi.rs` | `WeatherAPI` | **yes** | |
| `weatherbit.rs` | `Weatherbit` | **yes** | |
| `tomorrowio.rs` | `Tomorrow.io` | **yes** | |
| `visualcrossing.rs` | `Visual Crossing` | **yes** | |
| `weatherstack.rs` | `WeatherStack` | **yes** | |
| `yandex.rs` | `Yandex` | **yes** | |
| `accuweather.rs` | `AccuWeather` | **yes** | |
| `pirateweather.rs` | `Pirate Weather` | **yes** | |

The keyed providers are registered in the enum but never enter the default race
(default `race_list` is the three free providers + `Bright Sky` fallback); they
activate only if you put their name in `race_providers`/`fallback_providers` and
supply the matching key. Key name = the value `requires_key()` returns (e.g.
`OpenWeatherMap`), set via config `api_keys` or `WEATHER_KEY_*` env var — see
[configuration.md](configuration.md).

## Provider file shape (example: `openmeteo.rs`)

Typical structure: `const` URL bases and query-var strings → private
`#[derive(Deserialize)]` structs mirroring the API JSON → a `Provider` unit
struct → `impl BaseWeatherProvider` with `name()` and `fetch()` that builds the
URL, calls `ctx.client.get(...)`, parses, and maps into `Vec<HourlyPoint>`.

## Adding a provider

1. Create `src/providers/<name>.rs` with a unit struct implementing
   `BaseWeatherProvider`. Map the upstream response into `NormalizedWeatherData`.
2. In `providers/mod.rs`: add `pub mod <name>;`, add a `WeatherProvider` enum
   variant, and add match arms in `name()`, `fetch()`, and (if it needs a key)
   `requires_key()`.
3. Add the variant to the `providers` vec in `orchestrator.rs::run_orchestrator`.
4. To race it by default, add its `name()` to `default_race`/`default_fallback`
   in `orchestrator.rs` and `AppConfig::default` in `config.rs`.
