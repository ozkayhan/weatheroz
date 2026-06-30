pub mod accuweather;
pub mod base;
pub mod brightsky;
pub mod envcanada;
pub mod fmi;
pub mod meteostat;
pub mod metnorway;
pub mod models;
pub mod nws;
pub mod openmeteo;
pub mod openweathermap;
pub mod pirateweather;
pub mod smhi;
pub mod tomorrowio;
pub mod visualcrossing;
pub mod weatherapi;
pub mod weatherbit;
pub mod weatherstack;
pub mod wttr;
pub mod yandex;

use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{AQIData, HourlyPoint, NormalizedWeatherData};
use crate::tui::{conditional_sleep, ProviderState, SharedState};
use chrono::NaiveDate;
use futures_util::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone)]
pub enum WeatherProvider {
    OpenMeteo(openmeteo::OpenMeteoProvider),
    MetNorway(metnorway::MetNorwayProvider),
    Wttr(wttr::WttrProvider),
    BrightSky(brightsky::BrightSkyProvider),
    Smhi(smhi::SmhiProvider),
    Fmi(fmi::FmiProvider),
    Nws(nws::NwsProvider),
    Meteostat(meteostat::MeteostatProvider),
    EnvCanada(envcanada::EnvCanadaProvider),
    OpenWeatherMap(openweathermap::OpenWeatherMapProvider),
    WeatherApi(weatherapi::WeatherApiProvider),
    Weatherbit(weatherbit::WeatherbitProvider),
    TomorrowIo(tomorrowio::TomorrowIoProvider),
    VisualCrossing(visualcrossing::VisualCrossingProvider),
    WeatherStack(weatherstack::WeatherStackProvider),
    Yandex(yandex::YandexProvider),
    AccuWeather(accuweather::AccuWeatherProvider),
    PirateWeather(pirateweather::PirateWeatherProvider),
    Mock(mock::MockProvider),
}

pub mod mock {
    use super::models::NormalizedWeatherData;
    use crate::providers::base::{BaseWeatherProvider, FetchContext};

    #[derive(Clone, Debug)]
    pub struct MockProvider {
        pub name: String,
        pub delay_ms: u64,
        pub fail: bool,
        pub temperature: Option<f64>,
    }

    impl BaseWeatherProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn fetch(
            &self,
            _ctx: &FetchContext<'_>,
        ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
            if self.fail {
                return Err("Mock provider error".into());
            }
            let hourly = if let Some(t) = self.temperature {
                use crate::providers::models::HourlyPoint;
                vec![HourlyPoint {
                    time: "2026-05-21T12:00:00Z".to_string(),
                    temperature: t,
                    apparent_temperature: t,
                    precipitation_probability: 0.0,
                    precipitation: 0.0,
                    humidity: 50.0,
                    wind_speed: 10.0,
                    wind_direction: 180.0,
                    cloud_cover: 0.0,
                    weather_code: 0,
                    aqi: None,
                    uv_index: None,
                    is_day: None,
                    visibility: None,
                    soil_temperature: None,
                    soil_moisture: None,
                }]
            } else {
                vec![]
            };
            Ok(NormalizedWeatherData {
                provider_name: self.name.clone(),
                hourly,
            })
        }
    }
}

impl WeatherProvider {
    pub fn name(&self) -> &str {
        match self {
            WeatherProvider::OpenMeteo(p) => p.name(),
            WeatherProvider::MetNorway(p) => p.name(),
            WeatherProvider::Wttr(p) => p.name(),
            WeatherProvider::BrightSky(p) => p.name(),
            WeatherProvider::Smhi(p) => p.name(),
            WeatherProvider::Fmi(p) => p.name(),
            WeatherProvider::Nws(p) => p.name(),
            WeatherProvider::Meteostat(p) => p.name(),
            WeatherProvider::EnvCanada(p) => p.name(),
            WeatherProvider::OpenWeatherMap(p) => p.name(),
            WeatherProvider::WeatherApi(p) => p.name(),
            WeatherProvider::Weatherbit(p) => p.name(),
            WeatherProvider::TomorrowIo(p) => p.name(),
            WeatherProvider::VisualCrossing(p) => p.name(),
            WeatherProvider::WeatherStack(p) => p.name(),
            WeatherProvider::Yandex(p) => p.name(),
            WeatherProvider::AccuWeather(p) => p.name(),
            WeatherProvider::PirateWeather(p) => p.name(),
            WeatherProvider::Mock(p) => p.name(),
        }
    }

    pub fn requires_key(&self) -> Option<&str> {
        match self {
            WeatherProvider::OpenWeatherMap(_) => Some("OpenWeatherMap"),
            WeatherProvider::WeatherApi(_) => Some("WeatherAPI"),
            WeatherProvider::Weatherbit(_) => Some("Weatherbit"),
            WeatherProvider::TomorrowIo(_) => Some("Tomorrow.io"),
            WeatherProvider::VisualCrossing(_) => Some("Visual Crossing"),
            WeatherProvider::WeatherStack(_) => Some("WeatherStack"),
            WeatherProvider::Yandex(_) => Some("Yandex"),
            WeatherProvider::AccuWeather(_) => Some("AccuWeather"),
            WeatherProvider::PirateWeather(_) => Some("Pirate Weather"),
            WeatherProvider::Meteostat(_) => Some("Meteostat"),
            _ => None,
        }
    }

    pub async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            WeatherProvider::OpenMeteo(p) => p.fetch(ctx).await,
            WeatherProvider::MetNorway(p) => p.fetch(ctx).await,
            WeatherProvider::Wttr(p) => p.fetch(ctx).await,
            WeatherProvider::BrightSky(p) => p.fetch(ctx).await,
            WeatherProvider::Smhi(p) => p.fetch(ctx).await,
            WeatherProvider::Fmi(p) => p.fetch(ctx).await,
            WeatherProvider::Nws(p) => p.fetch(ctx).await,
            WeatherProvider::Meteostat(p) => p.fetch(ctx).await,
            WeatherProvider::EnvCanada(p) => p.fetch(ctx).await,
            WeatherProvider::OpenWeatherMap(p) => p.fetch(ctx).await,
            WeatherProvider::WeatherApi(p) => p.fetch(ctx).await,
            WeatherProvider::Weatherbit(p) => p.fetch(ctx).await,
            WeatherProvider::TomorrowIo(p) => p.fetch(ctx).await,
            WeatherProvider::VisualCrossing(p) => p.fetch(ctx).await,
            WeatherProvider::WeatherStack(p) => p.fetch(ctx).await,
            WeatherProvider::Yandex(p) => p.fetch(ctx).await,
            WeatherProvider::AccuWeather(p) => p.fetch(ctx).await,
            WeatherProvider::PirateWeather(p) => p.fetch(ctx).await,
            WeatherProvider::Mock(p) => p.fetch(ctx).await,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderStat {
    pub time_ms: f64,
    pub success: bool,
    pub error: Option<String>,
}

fn blend_weather_data(
    mut successful: Vec<(String, NormalizedWeatherData, f64)>,
) -> (NormalizedWeatherData, String) {
    if successful.is_empty() {
        return (
            NormalizedWeatherData {
                provider_name: "Empty".to_string(),
                hourly: vec![],
            },
            "Empty".to_string(),
        );
    }

    if successful.len() == 1 {
        let (name, data, _) = successful.remove(0);
        return (data, name);
    }

    // Sort successful responses by speed (elapsed time) to know the primary winner
    successful.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let names: Vec<String> = successful.iter().map(|(name, _, _)| name.clone()).collect();
    let blended_name = format!("Consensus Blended ({})", names.join(", "));

    // Group hourly points by their time string chronologically using BTreeMap
    let mut time_map = std::collections::BTreeMap::new();
    for (_, data, _) in &successful {
        for pt in &data.hourly {
            time_map
                .entry(pt.time.clone())
                .or_insert_with(Vec::new)
                .push(pt);
        }
    }

    let mut blended_hourly = Vec::new();
    for (time, pts) in time_map {
        let count = pts.len() as f64;
        if count == 0.0 {
            continue;
        }

        let temperature = pts.iter().map(|p| p.temperature).sum::<f64>() / count;
        let apparent_temperature = pts.iter().map(|p| p.apparent_temperature).sum::<f64>() / count;
        let precipitation_probability =
            pts.iter().map(|p| p.precipitation_probability).sum::<f64>() / count;
        let precipitation = pts.iter().map(|p| p.precipitation).sum::<f64>() / count;
        let humidity = pts.iter().map(|p| p.humidity).sum::<f64>() / count;
        let wind_speed = pts.iter().map(|p| p.wind_speed).sum::<f64>() / count;
        let wind_direction = pts.iter().map(|p| p.wind_direction).sum::<f64>() / count;
        let cloud_cover = pts.iter().map(|p| p.cloud_cover).sum::<f64>() / count;

        // Mode (majority vote) for weather_code
        let mut codes = HashMap::new();
        for pt in &pts {
            *codes.entry(pt.weather_code).or_insert(0) += 1;
        }
        let weather_code = codes
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(code, _)| code)
            .unwrap_or(0);

        // Blending optional uv_index
        let uv_vals: Vec<f64> = pts.iter().filter_map(|p| p.uv_index).collect();
        let uv_index = if uv_vals.is_empty() {
            None
        } else {
            Some(uv_vals.iter().sum::<f64>() / uv_vals.len() as f64)
        };

        // Blending optional visibility
        let vis_vals: Vec<f64> = pts.iter().filter_map(|p| p.visibility).collect();
        let visibility = if vis_vals.is_empty() {
            None
        } else {
            Some(vis_vals.iter().sum::<f64>() / vis_vals.len() as f64)
        };

        // Blending optional soil_temperature
        let soil_t_vals: Vec<f64> = pts.iter().filter_map(|p| p.soil_temperature).collect();
        let soil_temperature = if soil_t_vals.is_empty() {
            None
        } else {
            Some(soil_t_vals.iter().sum::<f64>() / soil_t_vals.len() as f64)
        };

        // Blending optional soil_moisture
        let soil_m_vals: Vec<f64> = pts.iter().filter_map(|p| p.soil_moisture).collect();
        let soil_moisture = if soil_m_vals.is_empty() {
            None
        } else {
            Some(soil_m_vals.iter().sum::<f64>() / soil_m_vals.len() as f64)
        };

        // Blending optional is_day
        let day_vals: Vec<bool> = pts.iter().filter_map(|p| p.is_day).collect();
        let is_day = if day_vals.is_empty() {
            None
        } else {
            let true_count = day_vals.iter().filter(|&&v| v).count();
            Some(true_count * 2 >= day_vals.len())
        };

        // Blending optional AQIData
        let aqi_list: Vec<&AQIData> = pts.iter().filter_map(|p| p.aqi.as_ref()).collect();
        let aqi = if aqi_list.is_empty() {
            None
        } else {
            let aqi_count = aqi_list.len() as f64;
            let co = aqi_list.iter().filter_map(|a| a.co).sum::<f64>() / aqi_count;
            let no2 = aqi_list.iter().filter_map(|a| a.no2).sum::<f64>() / aqi_count;
            let o3 = aqi_list.iter().filter_map(|a| a.o3).sum::<f64>() / aqi_count;
            let so2 = aqi_list.iter().filter_map(|a| a.so2).sum::<f64>() / aqi_count;
            let pm2_5 = aqi_list.iter().filter_map(|a| a.pm2_5).sum::<f64>() / aqi_count;
            let pm10 = aqi_list.iter().filter_map(|a| a.pm10).sum::<f64>() / aqi_count;
            Some(AQIData {
                co: Some(co),
                no2: Some(no2),
                o3: Some(o3),
                so2: Some(so2),
                pm2_5: Some(pm2_5),
                pm10: Some(pm10),
            })
        };

        blended_hourly.push(HourlyPoint {
            time,
            temperature,
            apparent_temperature,
            precipitation_probability,
            precipitation,
            humidity,
            wind_speed,
            wind_direction,
            cloud_cover,
            weather_code,
            aqi,
            uv_index,
            is_day,
            visibility,
            soil_temperature,
            soil_moisture,
        });
    }

    (
        NormalizedWeatherData {
            provider_name: blended_name.clone(),
            hourly: blended_hourly,
        },
        blended_name,
    )
}

pub async fn run_weather_race(
    ctx: &FetchContext<'_>,
    providers: Vec<WeatherProvider>,
    race_list: &[String],
    fallback_list: &[String],
    state: Option<&SharedState>,
) -> Result<
    (NormalizedWeatherData, HashMap<String, ProviderStat>, String),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let today = chrono::Utc::now().naive_utc().date();
    let s = NaiveDate::parse_from_str(ctx.start_date, "%Y-%m-%d")?;

    let is_historical = s < today;

    // Helper to filter and run a set of providers in parallel
    let run_race_group = |group_providers: Vec<WeatherProvider>| async move {
        let mut eligible = Vec::new();
        for p in group_providers {
            // MET Norway doesn't support historical queries
            if is_historical && p.name() == "MET Norway" {
                if let Some(s_state) = state {
                    let mut guard = s_state.lock().unwrap();
                    guard.providers.insert(
                        "MET Norway".to_string(),
                        ProviderState {
                            status: "failed".to_string(),
                            time: None,
                            error: Some("Excluded for historical query".to_string()),
                        },
                    );
                }
                continue;
            }

            // Skip providers that require an API key which is not configured
            if let Some(key_name) = p.requires_key() {
                if !ctx.api_keys.contains_key(key_name) {
                    if let Some(s_state) = state {
                        let mut guard = s_state.lock().unwrap();
                        guard.providers.insert(
                            p.name().to_string(),
                            ProviderState {
                                status: "failed".to_string(),
                                time: None,
                                error: Some(format!("Missing API key '{}'", key_name)),
                            },
                        );
                    }
                    continue;
                }
            }

            eligible.push(p);
        }

        if eligible.is_empty() {
            return Err("No eligible providers in this race group.".to_string());
        }

        if let Some(s_state) = state {
            let mut guard = s_state.lock().unwrap();
            guard.step_race = "running".to_string();
            guard.global_progress = 60;
            for p in &eligible {
                guard.providers.insert(
                    p.name().to_string(),
                    ProviderState {
                        status: "running".to_string(),
                        time: None,
                        error: None,
                    },
                );
            }
        }

        tracing::info!("📡 Launching parallel weather race group...");
        conditional_sleep(state, 150).await;

        let mut group_stats = HashMap::new();
        let mut successful_responses = Vec::new();
        let mut first_success_time: Option<f64> = None;

        let mut futures = FuturesUnordered::new();
        for p in eligible.clone() {
            let p_name = p.name().to_string();
            futures.push(async move {
                let start_time = Instant::now();
                let res = p.fetch(ctx).await;
                let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;
                (p_name, res, elapsed)
            });
        }

        let mut deadline = None;

        loop {
            tokio::select! {
                res_opt = futures.next() => {
                    match res_opt {
                        Some((p_name, res, elapsed)) => {
                            match res {
                                Ok(data) => {
                                    group_stats.insert(p_name.clone(), ProviderStat {
                                        time_ms: elapsed,
                                        success: true,
                                        error: None,
                                    });
                                    if let Some(s_state) = state {
                                        let mut guard = s_state.lock().unwrap();
                                        guard.providers.insert(p_name.clone(), ProviderState {
                                            status: "completed".to_string(),
                                            time: Some(elapsed),
                                            error: None,
                                        });
                                    }

                                    if first_success_time.is_none() {
                                        tracing::info!("🏆 {} won the race group in {:.1}ms!", p_name, elapsed);
                                        first_success_time = Some(elapsed);

                                        // Optimize: if there is only 1 eligible provider, we can break immediately!
                                        if eligible.len() == 1 {
                                            successful_responses.push((p_name, data, elapsed));
                                            break;
                                        }

                                        // Start the 100ms consensus window deadline
                                        deadline = Some(Box::pin(tokio::time::sleep(std::time::Duration::from_millis(100))));
                                    }
                                    successful_responses.push((p_name, data, elapsed));
                                }
                                Err(e) => {
                                    let err_str = e.to_string();
                                    group_stats.insert(p_name.clone(), ProviderStat {
                                        time_ms: elapsed,
                                        success: false,
                                        error: Some(err_str.clone()),
                                    });
                                    if let Some(s_state) = state {
                                        let mut guard = s_state.lock().unwrap();
                                        guard.providers.insert(p_name.clone(), ProviderState {
                                            status: "failed".to_string(),
                                            time: Some(elapsed),
                                            error: Some(err_str.clone()),
                                        });
                                    }
                                    tracing::warn!("⚠ {} failed: {}", p_name, err_str);
                                }
                            }
                        }
                        None => {
                            // Stream is empty
                            break;
                        }
                    }
                }
                _ = async {
                    if let Some(ref mut d) = deadline {
                        d.await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    tracing::info!("⏳ Consensus window of 100ms expired. Breaking early to short-circuit slow providers.");
                    break;
                }
            }
        }

        if !successful_responses.is_empty() {
            let (final_data, winner_name) = blend_weather_data(successful_responses);
            Ok((final_data, group_stats, winner_name))
        } else {
            Err(format!(
                "All providers in group failed. Errors:\n{}",
                group_stats
                    .iter()
                    .filter_map(|(n, s)| s.error.as_ref().map(|e| format!("{}: {}", n, e)))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }
    };

    // Filter providers into main group and fallback group
    let mut main_providers = Vec::new();
    let mut fallback_providers = Vec::new();

    for p in &providers {
        let name = p.name();
        if race_list
            .iter()
            .any(|r| r.to_lowercase() == name.to_lowercase())
        {
            main_providers.push(p.clone());
        } else if fallback_list
            .iter()
            .any(|f| f.to_lowercase() == name.to_lowercase())
        {
            fallback_providers.push(p.clone());
        }
    }

    // If main_providers filter left nothing, use all providers as main group
    if main_providers.is_empty() {
        main_providers = providers.clone();
    }

    tracing::info!("🏎 Running primary weather race group...");
    match run_race_group(main_providers).await {
        Ok(result) => {
            if let Some(s_state) = state {
                let mut guard = s_state.lock().unwrap();
                guard.step_race = "completed".to_string();
                guard.step_blending = "completed".to_string();
                guard.global_progress = 100;
            }
            Ok(result)
        }
        Err(e) => {
            tracing::warn!("❌ Primary weather race failed: {}", e);
            if fallback_providers.is_empty() {
                return Err(format!(
                    "Primary race failed, and no fallback providers are configured. Error: {}",
                    e
                )
                .into());
            }

            tracing::info!("🛡 Primary race failed. Initiating fallback weather race group...");
            match run_race_group(fallback_providers).await {
                Ok(result) => {
                    if let Some(s_state) = state {
                        let mut guard = s_state.lock().unwrap();
                        guard.step_race = "completed".to_string();
                        guard.step_blending = "completed".to_string();
                        guard.global_progress = 100;
                    }
                    Ok(result)
                }
                Err(f_err) => {
                    if let Some(s_state) = state {
                        let mut guard = s_state.lock().unwrap();
                        guard.step_race = "failed".to_string();
                    }
                    Err(format!("Both primary and fallback races failed.\nPrimary errors: {}\nFallback errors: {}", e, f_err).into())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockProvider;
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_race_fastest_provider_wins() {
        let client = Arc::new(reqwest::Client::new());
        let providers = vec![
            WeatherProvider::Mock(MockProvider {
                name: "Mock-Slow-1".to_string(),
                delay_ms: 300,
                fail: false,
                temperature: None,
            }),
            WeatherProvider::Mock(MockProvider {
                name: "Mock-Fast-2".to_string(),
                delay_ms: 50,
                fail: false,
                temperature: None,
            }),
        ];

        let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        let api_keys = HashMap::new();
        let ctx = FetchContext {
            client,
            lat: 41.0082,
            lon: 28.9784,
            start_date: &tomorrow,
            end_date: &tomorrow,
            api_keys: &api_keys,
            enrich: false,
            days: 1,
            minute_resolution: false,
            timezone: None,
        };

        let (weather_data, stats, winner_name) = run_weather_race(
            &ctx,
            providers,
            &["Mock-Slow-1".to_string(), "Mock-Fast-2".to_string()],
            &[],
            None,
        )
        .await
        .unwrap();

        assert_eq!(winner_name, "Mock-Fast-2");
        assert_eq!(weather_data.provider_name, "Mock-Fast-2");
        assert!(stats.contains_key("Mock-Fast-2"));
    }
}
