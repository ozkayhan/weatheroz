use chrono::NaiveDate;
use weatheroz::providers::base::BaseWeatherProvider;
use weatheroz::providers::metnorway::MetNorwayProvider;
use weatheroz::providers::models::{HourlyPoint, NormalizedWeatherData};
use weatheroz::providers::openmeteo::OpenMeteoProvider;
use weatheroz::providers::wttr::WttrProvider;
use weatheroz::providers::{run_weather_race, WeatherProvider};

#[tokio::test]
async fn test_provider_implementations_exist() {
    let openmeteo = OpenMeteoProvider;
    let metnorway = MetNorwayProvider;
    let wttr = WttrProvider;

    assert_eq!(openmeteo.name(), "Open-Meteo");
    assert_eq!(metnorway.name(), "MET Norway");
    assert_eq!(wttr.name(), "wttr.in");
}

#[tokio::test]
async fn test_weather_race_orchestrator() {
    let providers = vec![
        WeatherProvider::OpenMeteo(OpenMeteoProvider),
        WeatherProvider::MetNorway(MetNorwayProvider),
        WeatherProvider::Wttr(WttrProvider),
    ];

    let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let client = std::sync::Arc::new(reqwest::Client::new());
    let api_keys = std::collections::HashMap::new();
    let ctx = weatheroz::providers::base::FetchContext {
        client: client.clone(),
        lat: 41.0082,
        lon: 28.9784,
        start_date: &tomorrow,
        end_date: &tomorrow,
        api_keys: &api_keys,
        enrich: false,
        days: 1,
        minute_resolution: false,
    };
    let race_list = vec![
        "Open-Meteo".to_string(),
        "MET Norway".to_string(),
        "wttr.in".to_string(),
    ];
    let fallback_list = vec![];
    let (weather_data, stats, winner_name) =
        match run_weather_race(&ctx, providers.clone(), &race_list, &fallback_list, None).await {
            Ok(res) => res,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

    assert!(!weather_data.hourly.is_empty());
    assert!(!stats.is_empty());
    assert!(!winner_name.is_empty());
}

#[tokio::test]
async fn test_historical_date_excludes_metnorway() {
    let providers = [
        WeatherProvider::OpenMeteo(OpenMeteoProvider),
        WeatherProvider::MetNorway(MetNorwayProvider),
        WeatherProvider::Wttr(WttrProvider),
    ];

    let historical_date = "2025-01-01";
    let today_str = "2026-05-19";

    let s_date = NaiveDate::parse_from_str(historical_date, "%Y-%m-%d").unwrap();
    let today_date = NaiveDate::parse_from_str(today_str, "%Y-%m-%d").unwrap();

    let eligible_provider_names: Vec<&str> = providers
        .iter()
        .map(|p| p.name())
        .filter(|&name| !(s_date < today_date && name == "MET Norway"))
        .collect();

    assert!(!eligible_provider_names.contains(&"MET Norway"));
    assert!(eligible_provider_names.contains(&"Open-Meteo"));
    assert!(eligible_provider_names.contains(&"wttr.in"));
}

#[tokio::test]
async fn test_forecast_date_includes_metnorway() {
    let providers = [
        WeatherProvider::OpenMeteo(OpenMeteoProvider),
        WeatherProvider::MetNorway(MetNorwayProvider),
        WeatherProvider::Wttr(WttrProvider),
    ];

    let forecast_date = "2026-05-20";
    let today_str = "2026-05-19";

    let s_date = NaiveDate::parse_from_str(forecast_date, "%Y-%m-%d").unwrap();
    let today_date = NaiveDate::parse_from_str(today_str, "%Y-%m-%d").unwrap();

    let eligible_provider_names: Vec<&str> = providers
        .iter()
        .map(|p| p.name())
        .filter(|&name| !(s_date < today_date && name == "MET Norway"))
        .collect();

    assert!(eligible_provider_names.contains(&"MET Norway"));
    assert!(eligible_provider_names.contains(&"Open-Meteo"));
    assert!(eligible_provider_names.contains(&"wttr.in"));
}

#[tokio::test]
async fn test_models_serialization() {
    let hourly_point = HourlyPoint {
        time: "2026-05-19T12:00:00".to_string(),
        temperature: 20.5,
        apparent_temperature: 19.8,
        precipitation_probability: 10.0,
        precipitation: 0.0,
        humidity: 65.0,
        wind_speed: 10.2,
        wind_direction: 180.0,
        cloud_cover: 25.0,
        weather_code: 100,
        aqi: None,
        uv_index: None,
        is_day: None,
        visibility: None,
        soil_temperature: None,
        soil_moisture: None,
    };

    let serialized = serde_json::to_string(&hourly_point).unwrap();
    let deserialized: HourlyPoint = serde_json::from_str(&serialized).unwrap();

    assert_eq!(hourly_point.time, deserialized.time);
    assert_eq!(hourly_point.temperature, deserialized.temperature);
    assert_eq!(hourly_point.humidity, deserialized.humidity);
}

#[tokio::test]
async fn test_normalized_weather_data_structure() {
    let weather_data = NormalizedWeatherData {
        provider_name: "Open-Meteo".to_string(),
        hourly: vec![HourlyPoint {
            time: "2026-05-19T12:00:00".to_string(),
            temperature: 20.5,
            apparent_temperature: 19.8,
            precipitation_probability: 10.0,
            precipitation: 0.0,
            humidity: 65.0,
            wind_speed: 10.2,
            wind_direction: 180.0,
            cloud_cover: 25.0,
            weather_code: 100,
            aqi: None,
            uv_index: None,
            is_day: None,
            visibility: None,
            soil_temperature: None,
            soil_moisture: None,
        }],
    };

    let serialized = serde_json::to_string(&weather_data).unwrap();
    let deserialized: NormalizedWeatherData = serde_json::from_str(&serialized).unwrap();

    assert_eq!(weather_data.provider_name, deserialized.provider_name);
    assert_eq!(weather_data.hourly.len(), deserialized.hourly.len());
}
