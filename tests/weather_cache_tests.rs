#![allow(clippy::await_holding_lock)]

use std::sync::Mutex;
use std::time::SystemTime;
use weatheroz::providers::models::{HourlyPoint, NormalizedWeatherData};
use weatheroz::weather_cache::{get_cached_weather, save_cached_weather};

static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[tokio::test]
async fn test_weather_cache_save_and_retrieve_fresh() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let cache_dir = std::env::temp_dir().join("test_weather_cache_fresh_dir");
    let _ = std::fs::create_dir_all(&cache_dir);
    let cache_path = cache_dir.join("geo_cache.json");
    std::env::set_var("WEATHEROZ_CACHE_PATH", cache_path.to_str().unwrap());

    // Clean up
    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_file(cache_dir.join("weather_cache.json"));

    let dummy_data = NormalizedWeatherData {
        provider_name: "MockProvider".to_string(),
        hourly: vec![HourlyPoint {
            time: "2026-05-21T12:00".to_string(),
            temperature: 23.5,
            apparent_temperature: 24.0,
            precipitation_probability: 10.0,
            precipitation: 0.0,
            humidity: 50.0,
            wind_speed: 12.0,
            wind_direction: 180.0,
            cloud_cover: 20.0,
            weather_code: 0,
            aqi: None,
            uv_index: None,
            is_day: None,
            visibility: None,
            soil_temperature: None,
            soil_moisture: None,
        }],
    };

    save_cached_weather(41.0138, 28.9497, "2026-05-21", "2026-05-21", &dummy_data).await;

    let retrieved = get_cached_weather(41.0138, 28.9497, "2026-05-21", "2026-05-21", None).await;
    assert!(retrieved.is_some());
    let (data, is_fresh, _ts) = retrieved.unwrap();
    assert!(is_fresh);
    assert_eq!(data.provider_name, "MockProvider");
    assert_eq!(data.hourly[0].temperature, 23.5);

    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_file(cache_dir.join("weather_cache.json"));
    let _ = std::fs::remove_dir(&cache_dir);
    std::env::remove_var("WEATHEROZ_CACHE_PATH");
}

#[tokio::test]
async fn test_weather_cache_stale() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let cache_dir = std::env::temp_dir().join("test_weather_cache_stale_dir");
    let _ = std::fs::create_dir_all(&cache_dir);
    let cache_path = cache_dir.join("weather_cache.json");
    std::env::set_var("WEATHEROZ_CACHE_PATH", cache_dir.to_str().unwrap());

    // Clean up
    let _ = std::fs::remove_file(&cache_path);

    let dummy_data = NormalizedWeatherData {
        provider_name: "MockProviderStale".to_string(),
        hourly: vec![],
    };

    // Save with custom timestamp (16 minutes ago)
    let now_secs = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let stale_secs = now_secs - (16.0 * 60.0);

    let key = format!(
        "{:.4}:{:.4}:{}:{}",
        41.0138, 28.9497, "2026-05-21", "2026-05-21"
    );
    let mut cache = std::collections::HashMap::new();

    #[derive(serde::Serialize)]
    struct TestEntry {
        data: NormalizedWeatherData,
        timestamp: f64,
    }
    cache.insert(
        key,
        TestEntry {
            data: dummy_data,
            timestamp: stale_secs,
        },
    );
    std::fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

    let retrieved = get_cached_weather(41.0138, 28.9497, "2026-05-21", "2026-05-21", None).await;
    assert!(
        retrieved.is_some(),
        "retrieved cache is None; expected Some"
    );
    let (data, is_fresh, ts) = retrieved.unwrap();
    assert!(!is_fresh);
    assert_eq!(data.provider_name, "MockProviderStale");
    assert_eq!(ts, stale_secs);

    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_dir(&cache_dir);
    std::env::remove_var("WEATHEROZ_CACHE_PATH");
}

#[test]
fn test_cli_weather_cache_hit() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let cache_dir = std::env::temp_dir().join("test_weather_cache_cli_hit_dir");
    let _ = std::fs::create_dir_all(&cache_dir);
    let geo_cache_path = cache_dir.join("geo_cache.json");
    let weather_cache_path = cache_dir.join("weather_cache.json");

    let now_secs = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    // 1. Write geo cache
    let geo_cache_content = format!(
        r#"{{"cachecity":{{"data":{{"name":"CacheCity","latitude":45.1234,"longitude":12.1234,"country":"Italy","admin1":"Veneto"}},"timestamp":{}}}}}"#,
        now_secs
    );
    std::fs::write(&geo_cache_path, geo_cache_content).unwrap();

    // 2. Write weather cache
    let today_str = chrono::Utc::now()
        .naive_utc()
        .date()
        .format("%Y-%m-%d")
        .to_string();
    let key = format!("{:.4}:{:.4}:{}:{}", 45.1234, 12.1234, today_str, today_str);
    let weather_cache_content = format!(
        r#"{{"{}":{{"data":{{"provider_name":"CachedProvider","hourly":[]}},"timestamp":{}}}}}"#,
        key, now_secs
    );
    std::fs::write(&weather_cache_path, weather_cache_content).unwrap();

    // Run weatheroz in verbose mode
    let output = std::process::Command::new("./target/debug/weatheroz")
        .env("WEATHEROZ_CACHE_PATH", geo_cache_path.to_str().unwrap())
        .arg("cachecity")
        .arg("--verbose")
        .arg("-d")
        .arg("1")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("--- test_cli_weather_cache_hit stdout ---\n{}", stdout);
    println!("--- test_cli_weather_cache_hit stderr ---\n{}", stderr);

    assert!(output.status.success());
    assert!(stdout.contains("Weather Cache Hit"));
    assert!(stdout.contains("CacheCity"));
    assert!(stdout.contains("Italy"));

    let _ = std::fs::remove_file(&geo_cache_path);
    let _ = std::fs::remove_file(&weather_cache_path);
    let _ = std::fs::remove_dir(&cache_dir);
}

#[test]
fn test_cli_offline_fallback() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let cache_dir = std::env::temp_dir().join("test_weather_cache_cli_fallback_dir");
    let _ = std::fs::create_dir_all(&cache_dir);
    let geo_cache_path = cache_dir.join("geo_cache.json");
    let weather_cache_path = cache_dir.join("weather_cache.json");

    let now_secs = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    // 1. Write geo cache
    let geo_cache_content = format!(
        r#"{{"offlinecity":{{"data":{{"name":"OfflineCity","latitude":99.0,"longitude":199.0,"country":"Ocean","admin1":"Deep"}},"timestamp":{}}}}}"#,
        now_secs
    );
    std::fs::write(&geo_cache_path, geo_cache_content).unwrap();

    // 2. Write weather cache with stale timestamp (16 minutes ago)
    let stale_secs = now_secs - (16.0 * 60.0);
    let today_str = chrono::Utc::now()
        .naive_utc()
        .date()
        .format("%Y-%m-%d")
        .to_string();
    let key = format!("{:.4}:{:.4}:{}:{}", 99.0, 199.0, today_str, today_str);
    let weather_cache_content = format!(
        r#"{{"{}":{{"data":{{"provider_name":"StaleProvider","hourly":[]}},"timestamp":{}}}}}"#,
        key, stale_secs
    );
    std::fs::write(&weather_cache_path, weather_cache_content).unwrap();

    // Run weatheroz in verbose mode
    let output = std::process::Command::new("./target/debug/weatheroz")
        .env("WEATHEROZ_CACHE_PATH", geo_cache_path.to_str().unwrap())
        .arg("offlinecity")
        .arg("--verbose")
        .arg("-d")
        .arg("1")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("--- test_cli_offline_fallback stdout ---\n{}", stdout);
    println!("--- test_cli_offline_fallback stderr ---\n{}", stderr);

    assert!(output.status.success());
    assert!(stdout.contains("Offline Mode") || stdout.contains("OfflineCity"));

    let _ = std::fs::remove_file(&geo_cache_path);
    let _ = std::fs::remove_file(&weather_cache_path);
    let _ = std::fs::remove_dir(&cache_dir);
}
