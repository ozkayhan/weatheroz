use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use weatheroz::cli::validate_date;
use weatheroz::providers::base::FetchContext;
use weatheroz::providers::tomorrowio::TomorrowIoProvider;
use weatheroz::providers::{run_weather_race, WeatherProvider};

#[test]
fn test_cli_date_validation_scenarios() {
    // 1. Correct dates
    assert!(validate_date("2026-05-21").is_ok());
    assert!(validate_date("2000-02-29").is_ok()); // Leap year

    // 2. Incorrect dates
    assert!(validate_date("2026-13-01").is_err()); // Month 13
    assert!(validate_date("2026-05-32").is_err()); // Day 32
    assert!(validate_date("2026/05/21").is_err()); // Wrong separator
    assert!(validate_date("2026-05").is_err());    // Missing day
    assert!(validate_date("invalid-date").is_err());
    assert!(validate_date("").is_err());
}

#[tokio::test]
async fn test_provider_skipped_without_api_key() {
    // Tests that when a provider requiring keys (e.g. Tomorrow.io) has no key configured
    // in the context, it is skipped before executing, marked as failed with a missing key message.

    let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let providers = vec![
        WeatherProvider::TomorrowIo(TomorrowIoProvider),
        WeatherProvider::Mock(weatheroz::providers::mock::MockProvider {
            name: "Mock-Backup".to_string(),
            delay_ms: 5,
            fail: false,
            temperature: Some(25.0),
        }),
    ];

    let shared_state = Arc::new(Mutex::new(weatheroz::tui::ProcessState::new("istanbul".to_string())));

    // No Tomorrow.io API key in context
    let api_keys = HashMap::new();
    let ctx = FetchContext {
        client: Arc::new(reqwest::Client::new()),
        lat: 41.0082,
        lon: 28.9784,
        start_date: &tomorrow,
        end_date: &tomorrow,
        api_keys: &api_keys,
        enrich: false,
        days: 1,
        minute_resolution: false,
    };

    let (_weather_data, _stats, winner_name) = run_weather_race(
        &ctx,
        providers,
        &["Tomorrow.io".to_string(), "Mock-Backup".to_string()],
        &[],
        Some(&shared_state),
    )
    .await
    .unwrap();

    // Tomorrow.io should have been skipped, and Mock-Backup won!
    assert_eq!(winner_name, "Mock-Backup");

    let state_guard = shared_state.lock().unwrap();
    assert!(state_guard.providers.contains_key("Tomorrow.io"));
    let tomorrow_state = state_guard.providers.get("Tomorrow.io").unwrap();
    assert_eq!(tomorrow_state.status, "failed");
    assert!(tomorrow_state.error.as_ref().unwrap().contains("Missing API key"));
}

#[tokio::test]
async fn test_provider_executed_with_api_key() {
    // Tests that when a provider requiring keys (e.g. Tomorrow.io) HAS a key configured,
    // the system proceeds to execute it (even if it subsequently fails due to an invalid key/network,
    // which proves it was not filtered out beforehand).

    let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let providers = vec![
        WeatherProvider::TomorrowIo(TomorrowIoProvider),
    ];

    let shared_state = Arc::new(Mutex::new(weatheroz::tui::ProcessState::new("istanbul".to_string())));

    // Tomorrow.io API key is provided
    let mut api_keys = HashMap::new();
    api_keys.insert("Tomorrow.io".to_string(), "dummy_key".to_string());

    let ctx = FetchContext {
        client: Arc::new(reqwest::Client::new()),
        lat: 41.0082,
        lon: 28.9784,
        start_date: &tomorrow,
        end_date: &tomorrow,
        api_keys: &api_keys,
        enrich: false,
        days: 1,
        minute_resolution: false,
    };

    // The race should fail/error due to dummy key on real fetch, rather than being skipped
    let _res = run_weather_race(
        &ctx,
        providers,
        &["Tomorrow.io".to_string()],
        &[],
        Some(&shared_state),
    )
    .await;

    // Check that it was evaluated (evaluated means it wasn't skipped beforehand, so its status in TuiState
    // is "failed" but does NOT have a missing API key error).
    let state_guard = shared_state.lock().unwrap();
    if let Some(t_state) = state_guard.providers.get("Tomorrow.io") {
        assert_eq!(t_state.status, "failed");
        assert!(!t_state.error.as_ref().unwrap().contains("Missing API key"));
    }
}
