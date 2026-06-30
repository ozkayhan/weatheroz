use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use weatheroz::providers::base::FetchContext;
use weatheroz::providers::mock::MockProvider;
use weatheroz::providers::{run_weather_race, WeatherProvider};

#[tokio::test]
async fn test_short_circuit_performance() {
    // HYPOTHESIS 1: Under the old logic, a single slow provider would block the entire parallel race group
    // from finishing, making the total race time equal to the slowest provider's response time (e.g. 1000ms).
    // Under V2 logic (Short-circuiting), the race should complete and return as soon as the first provider succeeds,
    // making the total race time equal to the fastest provider's response time (~50ms), ignoring the slow one.

    let client = Arc::new(reqwest::Client::new());
    let providers = vec![
        WeatherProvider::Mock(MockProvider {
            name: "Mock-Fast-V2".to_string(),
            delay_ms: 50,
            fail: false,
            temperature: None,
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Mock-Slow-V2".to_string(),
            delay_ms: 1000, // 1 second delay
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

    let start = Instant::now();
    let (weather_data, _stats, winner_name) = run_weather_race(
        &ctx,
        providers,
        &["Mock-Fast-V2".to_string(), "Mock-Slow-V2".to_string()],
        &[],
        None,
    )
    .await
    .unwrap();
    let elapsed = start.elapsed();

    println!(
        "Short-circuit race completed in {} ms. Winner: {}",
        elapsed.as_millis(),
        winner_name
    );

    // Old implementation takes > 1000ms because it waits for the slow mock provider.
    // New V2 short-circuiting should complete in less than 300ms!
    assert!(
        elapsed.as_millis() < 300,
        "Short-circuit test FAILED! Elapsed time was {}ms, which is too slow (expected < 300ms)",
        elapsed.as_millis()
    );

    // Check that we got the correct winner
    assert_eq!(winner_name, "Mock-Fast-V2");
    assert_eq!(weather_data.provider_name, "Mock-Fast-V2");
}

#[tokio::test]
async fn test_consensus_blending_correctness() {
    // HYPOTHESIS 2: Under the old logic, we simply take the absolute fastest provider's results.
    // In V2, we introduce "Smart Blending & Consensus" where we wait for a tiny "consensus window"
    // (e.g. 100ms grace period after the first successful response) and blend the weather data
    // of all successful providers that returned in that window, creating a highly accurate combined forecast.
    //
    // Mock-1: finishes at 40ms, temperature = 20.0
    // Mock-2: finishes at 70ms, temperature = 22.0
    // Mock-3: finishes at 90ms, temperature = 21.0
    // Mock-Slow: finishes at 1000ms, temperature = 25.0 (should be excluded from blending because it's too slow)
    //
    // Blended temperature should be (20.0 + 22.0 + 21.0) / 3.0 = 21.0.

    let client = Arc::new(reqwest::Client::new());
    let providers = vec![
        WeatherProvider::Mock(MockProvider {
            name: "Mock-1".to_string(),
            delay_ms: 40,
            fail: false,
            temperature: Some(20.0),
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Mock-2".to_string(),
            delay_ms: 70,
            fail: false,
            temperature: Some(22.0),
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Mock-3".to_string(),
            delay_ms: 90,
            fail: false,
            temperature: Some(21.0),
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Mock-Slow".to_string(),
            delay_ms: 1000,
            fail: false,
            temperature: Some(25.0),
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

    let start = Instant::now();
    let (weather_data, stats, winner_name) = run_weather_race(
        &ctx,
        providers,
        &[
            "Mock-1".to_string(),
            "Mock-2".to_string(),
            "Mock-3".to_string(),
            "Mock-Slow".to_string(),
        ],
        &[],
        None,
    )
    .await
    .unwrap();
    let elapsed = start.elapsed();

    println!(
        "Blending race completed in {} ms. Winner name: {}",
        elapsed.as_millis(),
        winner_name
    );

    // Speed check: Consensus window starts at 40ms and ends at 40 + 100 = 140ms.
    // It should complete in less than 300ms, not waiting for the 1000ms slow provider.
    assert!(
        elapsed.as_millis() < 300,
        "Blending race was too slow: took {}ms",
        elapsed.as_millis()
    );

    // Winner name should represent the blended status, e.g., containing "Consensus Blended"
    assert!(
        winner_name.contains("Blended") || winner_name.contains("Consensus"),
        "Expected winner name to denote blending, got: {}",
        winner_name
    );

    // Weather data should have blended temperature: 21.0 Celsius!
    assert!(
        !weather_data.hourly.is_empty(),
        "Hourly weather points are empty"
    );
    let temp = weather_data.hourly[0].temperature;
    assert!(
        (temp - 21.0).abs() < 0.01,
        "Expected blended temperature of 21.0, but got {}",
        temp
    );

    // The statistics should track all participants
    assert!(stats.contains_key("Mock-1"));
    assert!(stats.contains_key("Mock-2"));
    assert!(stats.contains_key("Mock-3"));
}

#[tokio::test]
async fn test_resilience_missing_keys() {
    // HYPOTHESIS 3: Under the old logic, a provider that requires keys but lacks them would be executed
    // and would immediately return an error. In V2, we skip providers that require api keys but have none configured
    // before starting the race, preventing unnecessary error logs and keeping UI stats clean.
    // Also, if one provider fails (e.g. Mock-Fail-V2), the race should successfully fall back and use
    // other successful providers in parallel.

    let client = Arc::new(reqwest::Client::new());
    let providers = vec![
        WeatherProvider::Mock(MockProvider {
            name: "Mock-Fail-V2".to_string(),
            delay_ms: 10,
            fail: true,
            temperature: None,
        }),
        WeatherProvider::Mock(MockProvider {
            name: "Mock-Success-V2".to_string(),
            delay_ms: 50,
            fail: false,
            temperature: Some(18.0),
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
        &["Mock-Fail-V2".to_string(), "Mock-Success-V2".to_string()],
        &[],
        None,
    )
    .await
    .unwrap();

    // The race should succeed by falling back to Mock-Success-V2, even though Mock-Fail-V2 failed first.
    assert!(winner_name.contains("Success") || winner_name.contains("Blended"));
    assert!(stats.contains_key("Mock-Fail-V2"));
    assert!(!stats.get("Mock-Fail-V2").unwrap().success);
    assert!(stats.get("Mock-Success-V2").unwrap().success);
    assert_eq!(weather_data.hourly[0].temperature, 18.0);
}
