use std::collections::HashMap;
use std::sync::Arc;
use chrono::{NaiveDate, Utc};
use crate::cli::Args;
use crate::geocoding::{GeocodedLocation, resolve_location};
use crate::providers::{run_weather_race, WeatherProvider, ProviderStat};
use crate::providers::base::FetchContext;
use crate::providers::openmeteo::OpenMeteoProvider;
use crate::providers::metnorway::MetNorwayProvider;
use crate::providers::wttr::WttrProvider;
use crate::providers::brightsky::BrightSkyProvider;
use crate::providers::smhi::SmhiProvider;
use crate::providers::fmi::FmiProvider;
use crate::providers::nws::NwsProvider;
use crate::providers::meteostat::MeteostatProvider;
use crate::providers::envcanada::EnvCanadaProvider;
use crate::providers::openweathermap::OpenWeatherMapProvider;
use crate::providers::weatherapi::WeatherApiProvider;
use crate::providers::weatherbit::WeatherbitProvider;
use crate::providers::tomorrowio::TomorrowIoProvider;
use crate::providers::visualcrossing::VisualCrossingProvider;
use crate::providers::weatherstack::WeatherStackProvider;
use crate::providers::yandex::YandexProvider;
use crate::providers::accuweather::AccuWeatherProvider;
use crate::providers::pirateweather::PirateWeatherProvider;
use crate::providers::simulated;
use crate::tui::SharedState;
use crate::weather_cache::{get_cached_weather, save_cached_weather};
use crate::providers::models::NormalizedWeatherData;

pub struct OrchestratorResult {
    pub resolved_location: GeocodedLocation,
    pub weather_data: NormalizedWeatherData,
    pub stats: HashMap<String, ProviderStat>,
    pub winner_name: String,
    pub is_offline: bool,
    pub cached_timestamp: Option<f64>,
}

pub async fn run_orchestrator(
    args: &Args,
    config: &crate::shared::domain::config::AppConfig,
    client: Arc<reqwest::Client>,
    location_query: String,
    initial_location: Option<GeocodedLocation>,
    state: Option<&SharedState>,
) -> Result<OrchestratorResult, Box<dyn std::error::Error + Send + Sync>> {
    let today_str = Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();
    let start_date = args.from_date.clone().unwrap_or_else(|| today_str.clone());

    // Compute days count and date range
    let days_count = args.days.or(config.forecast_days).unwrap_or(7);
    let end_date = if let Some(to_date) = &args.to_date {
        to_date.clone()
    } else if args.days.is_some() || config.forecast_days.is_some() || args.from_date.is_none() {
        let s_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap_or_else(|_| Utc::now().naive_utc().date());
        let computed = s_date + chrono::Duration::days(days_count as i64 - 1);
        computed.format("%Y-%m-%d").to_string()
    } else {
        start_date.clone()
    };

    // 1. Resolve Location
    let resolved = if let Some(loc) = initial_location {
        if let Some(s) = state {
            let mut guard = s.lock().unwrap();
            guard.resolved_location = Some(format!("{}, {}", loc.name, loc.country));
            guard.cache_status = "miss".to_string();
            guard.step_geocoding = "completed".to_string();
            guard.global_progress = 50;
        }
        loc
    } else {
        resolve_location(client.clone(), &location_query, state).await?
    };

    // 2. Check Caches
    let mut cache_hit_data = None;
    let mut offline_data = None;

    let ttl_minutes = args.cache_ttl.or(config.cache_ttl_minutes).unwrap_or(15);
    let ttl_seconds = (ttl_minutes as f64) * 60.0;

    if ttl_minutes > 0 {
        match get_cached_weather(resolved.latitude, resolved.longitude, &start_date, &end_date, Some(ttl_seconds)).await {
            Some((data, is_fresh, timestamp)) => {
                if is_fresh {
                    cache_hit_data = Some(data);
                } else {
                    offline_data = Some((data, timestamp));
                }
            }
            None => {}
        }
    }

    if let Some(data) = cache_hit_data {
        if let Some(s) = state {
            let mut guard = s.lock().unwrap();
            guard.cache_status = "hit".to_string();
            guard.step_race = "completed".to_string();
            guard.step_blending = "completed".to_string();
            guard.global_progress = 100;
            for (_, p_state) in guard.providers.iter_mut() {
                p_state.status = "completed".to_string();
                p_state.time = Some(0.0);
            }
        }
        tracing::info!("⚡ Cache Hit: Loaded fresh weather data from local cache.");

        let mut stats = HashMap::new();
        stats.insert("Cache".to_string(), ProviderStat {
            time_ms: 0.0,
            success: true,
            error: None,
        });

        return Ok(OrchestratorResult {
            resolved_location: resolved,
            weather_data: data,
            stats,
            winner_name: "Cache".to_string(),
            is_offline: false,
            cached_timestamp: None,
        });
    }

    // 3. Define and register all 30 providers
    let providers = vec![
        WeatherProvider::OpenMeteo(OpenMeteoProvider),
        WeatherProvider::MetNorway(MetNorwayProvider),
        WeatherProvider::Wttr(WttrProvider),
        WeatherProvider::BrightSky(BrightSkyProvider),
        WeatherProvider::Smhi(SmhiProvider),
        WeatherProvider::Fmi(FmiProvider),
        WeatherProvider::Nws(NwsProvider),
        WeatherProvider::Meteostat(MeteostatProvider),
        WeatherProvider::EnvCanada(EnvCanadaProvider),
        WeatherProvider::OpenWeatherMap(OpenWeatherMapProvider),
        WeatherProvider::WeatherApi(WeatherApiProvider),
        WeatherProvider::Weatherbit(WeatherbitProvider),
        WeatherProvider::TomorrowIo(TomorrowIoProvider),
        WeatherProvider::VisualCrossing(VisualCrossingProvider),
        WeatherProvider::WeatherStack(WeatherStackProvider),
        WeatherProvider::Yandex(YandexProvider),
        WeatherProvider::AccuWeather(AccuWeatherProvider),
        WeatherProvider::PirateWeather(PirateWeatherProvider),
        WeatherProvider::AerisWeather(simulated::AerisWeatherProvider),
        WeatherProvider::StormGlass(simulated::StormGlassProvider),
        WeatherProvider::MeteoBlue(simulated::MeteoBlueProvider),
        WeatherProvider::Climacell(simulated::ClimacellProvider),
        WeatherProvider::Ambee(simulated::AmbeeProvider),
        WeatherProvider::OpenUv(simulated::OpenUvProvider),
        WeatherProvider::Oikolab(simulated::OikolabProvider),
        WeatherProvider::Weatherzone(simulated::WeatherzoneProvider),
        WeatherProvider::Aemet(simulated::AemetProvider),
        WeatherProvider::MeteoFrance(simulated::MeteoFranceProvider),
        WeatherProvider::SmhiHistorical(simulated::SmhiHistoricalProvider),
        WeatherProvider::Jma(simulated::JmaProvider),
    ];

    // 4. Run Weather Provider Race with Primary and Fallback tiers
    let ctx = FetchContext {
        client: client.clone(),
        lat: resolved.latitude,
        lon: resolved.longitude,
        start_date: &start_date,
        end_date: &end_date,
        api_keys: &config.api_keys,
        enrich: args.enrich || config.enrich_data.unwrap_or(false),
        days: days_count,
        minute_resolution: args.minute || config.minute_updates.unwrap_or(false),
    };

    let default_race = vec![
        "Open-Meteo".to_string(),
        "MET Norway".to_string(),
        "wttr.in".to_string(),
    ];
    let default_fallback = vec!["Bright Sky".to_string()];

    let race_list = config.race_providers.as_ref().unwrap_or(&default_race);
    let fallback_list = config.fallback_providers.as_ref().unwrap_or(&default_fallback);

    match run_weather_race(
        &ctx,
        providers,
        race_list,
        fallback_list,
        state,
    ).await {
        Ok((weather_data, stats, winner)) => {
            save_cached_weather(resolved.latitude, resolved.longitude, &start_date, &end_date, &weather_data).await;
            Ok(OrchestratorResult {
                resolved_location: resolved,
                weather_data,
                stats,
                winner_name: winner,
                is_offline: false,
                cached_timestamp: None,
            })
        }
        Err(e) => {
            if let Some((stale_data, timestamp)) = offline_data {
                if let Some(s) = state {
                    let mut guard = s.lock().unwrap();
                    guard.cache_status = "hit".to_string();
                    guard.step_race = "completed".to_string();
                    guard.step_blending = "completed".to_string();
                    guard.global_progress = 100;
                    for (_, p_state) in guard.providers.iter_mut() {
                        p_state.status = "completed".to_string();
                        p_state.time = Some(0.0);
                    }
                }
                tracing::info!("⚡ Offline Mode: Failed to load online data. Reverting to local cache.");

                let mut stats = HashMap::new();
                stats.insert("Cache".to_string(), ProviderStat {
                    time_ms: 0.0,
                    success: true,
                    error: None,
                });

                Ok(OrchestratorResult {
                    resolved_location: resolved,
                    weather_data: stale_data,
                    stats,
                    winner_name: "Cache".to_string(),
                    is_offline: true,
                    cached_timestamp: Some(timestamp),
                })
            } else {
                Err(e)
            }
        }
    }
}
