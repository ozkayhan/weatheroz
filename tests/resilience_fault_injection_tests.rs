use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use weatheroz::providers::base::FetchContext;
use weatheroz::providers::mock::MockProvider;
use weatheroz::providers::{run_weather_race, WeatherProvider};

fn setup_ctx<'a>(api_keys: &'a HashMap<String, String>, date: &'a str) -> FetchContext<'a> {
    FetchContext {
        client: Arc::new(reqwest::Client::new()),
        lat: 41.0082,
        lon: 28.9784,
        start_date: date,
        end_date: date,
        api_keys,
        enrich: false,
        days: 1,
        minute_resolution: false,
    }
}

#[tokio::test]
async fn test_primary_to_fallback_transition_on_failure() {
    // Tests that when all primary race providers fail, the orchestrator
    // gracefully transitions to the fallback group and succeeds.

    let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let providers = vec![
        WeatherProvider::Mock(MockProvider {
            name: "Primary-Fail-1".to_string(),
            delay_ms: 10,
            fail: true,
            temperature: None,
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Primary-Fail-2".to_string(),
            delay_ms: 20,
            fail: true,
            temperature: None,
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Fallback-Success".to_string(),
            delay_ms: 30,
            fail: false,
            temperature: Some(15.0),
        }),
    ];

    let api_keys = HashMap::new();
    let ctx = setup_ctx(&api_keys, &tomorrow);

    let (weather_data, stats, winner_name) = run_weather_race(
        &ctx,
        providers,
        &["Primary-Fail-1".to_string(), "Primary-Fail-2".to_string()],
        &["Fallback-Success".to_string()],
        None,
    )
    .await
    .unwrap();

    // The primary race must have failed, and the fallback succeeded!
    assert_eq!(winner_name, "Fallback-Success");
    assert_eq!(weather_data.provider_name, "Fallback-Success");
    assert_eq!(weather_data.hourly[0].temperature, 15.0);

    // Verify stats for the successful fallback group
    assert!(stats.contains_key("Fallback-Success"));
    assert!(stats.get("Fallback-Success").unwrap().success);
}

#[tokio::test]
async fn test_full_system_failure_bubbles_detailed_error() {
    // Tests that when both primary and fallback races fail, the orchestrator returns
    // a detailed composite error indicating all providers failed.

    let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let providers = vec![
        WeatherProvider::Mock(MockProvider {
            name: "Primary-Fail-1".to_string(),
            delay_ms: 5,
            fail: true,
            temperature: None,
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Fallback-Fail-2".to_string(),
            delay_ms: 5,
            fail: true,
            temperature: None,
        }),
    ];

    let api_keys = HashMap::new();
    let ctx = setup_ctx(&api_keys, &tomorrow);

    let res = run_weather_race(
        &ctx,
        providers,
        &["Primary-Fail-1".to_string()],
        &["Fallback-Fail-2".to_string()],
        None,
    )
    .await;

    assert!(res.is_err());
    let err_msg = res.err().unwrap().to_string();

    // The error message must mention the failure of the fallback group
    assert!(err_msg.contains("Fallback-Fail-2") || err_msg.contains("failed"));
}

#[tokio::test]
async fn test_shared_state_progression_tracking() {
    // Tests that the orchestrator updates SharedState (TUI state) correctly, including:
    // 1. Initial/Progress step updates.
    // 2. Individual provider statuses (running, completed, failed).
    // 3. Completion markers.

    let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let providers = vec![
        WeatherProvider::Mock(MockProvider {
            name: "Mock-Fast-State".to_string(),
            delay_ms: 10,
            fail: false,
            temperature: Some(21.0),
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Mock-Fail-State".to_string(),
            delay_ms: 15,
            fail: true,
            temperature: None,
        }),
    ];

    let shared_state = Arc::new(Mutex::new(weatheroz::tui::ProcessState::new("istanbul".to_string())));

    let api_keys = HashMap::new();
    let ctx = setup_ctx(&api_keys, &tomorrow);

    let (_weather_data, _stats, _winner) = run_weather_race(
        &ctx,
        providers,
        &["Mock-Fast-State".to_string(), "Mock-Fail-State".to_string()],
        &[],
        Some(&shared_state),
    )
    .await
    .unwrap();

    let state_guard = shared_state.lock().unwrap();

    // Invariants for step indicators and global progress
    assert_eq!(state_guard.step_race, "completed");
    assert_eq!(state_guard.step_blending, "completed");
    assert_eq!(state_guard.global_progress, 100);

    // Assert that provider states are correctly captured
    assert!(state_guard.providers.contains_key("Mock-Fast-State"));
    let fast_prov_state = state_guard.providers.get("Mock-Fast-State").unwrap();
    assert_eq!(fast_prov_state.status, "completed");
    assert!(fast_prov_state.time.is_some());
    assert!(fast_prov_state.error.is_none());

    assert!(state_guard.providers.contains_key("Mock-Fail-State"));
    let fail_prov_state = state_guard.providers.get("Mock-Fail-State").unwrap();
    assert_eq!(fail_prov_state.status, "failed");
    assert!(fail_prov_state.error.is_some());
}
