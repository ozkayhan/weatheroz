use std::process::Command;

#[tokio::test]
async fn test_cli_normal_output() {
    let output = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stdout.contains("Istanbul"));
    assert!(stdout.contains("Republic of Türkiye"));
    assert!(stdout.contains("Coordinates: 41.0138°, 28.9497°"));
    assert!(stdout.contains("Time"));
    assert!(stdout.contains("Temp"));
    assert!(stderr.is_empty());
    assert!(output.status.success());
}

#[tokio::test]
async fn test_cli_json_output() {
    let output = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .arg("--json-output")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.is_empty());
    assert!(output.status.success());

    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON output");
    assert!(parsed.get("provider_name").is_some());
    assert!(parsed.get("hourly").is_some());
    assert!(parsed.get("resolved_location").is_some());
}

#[tokio::test]
async fn test_cli_verbose_mode() {
    let cache_dir = std::env::temp_dir().join("test_integration_verbose_dir");
    let _ = std::fs::create_dir_all(&cache_dir);
    let temp_cache = cache_dir.join("temp_verbose_geo_cache.json");
    let weather_cache = cache_dir.join("weather_cache.json");

    let output = Command::new("./target/debug/weatheroz")
        .env("WEATHEROZ_CACHE_PATH", temp_cache.to_str().unwrap())
        .arg("istanbul")
        .arg("--verbose")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stdout.contains("Cache Hit") || stdout.contains("Cache Miss"));
    assert!(stdout.contains("Starting parallel race") || stdout.contains("Weather Cache Hit"));
    assert!(stdout.contains("Winner:") || stdout.contains("Weather Cache Hit"));
    assert!(stderr.is_empty());
    assert!(output.status.success());

    let _ = std::fs::remove_file(&temp_cache);
    let _ = std::fs::remove_file(&weather_cache);
    let _ = std::fs::remove_dir(&cache_dir);
}

#[tokio::test]
async fn test_cli_date_validation() {
    let output = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .arg("--from-date=2026-13-01")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Date must be in YYYY-MM-DD format") || stdout.contains("Error:"));
    assert!(!output.status.success());
}

#[tokio::test]
async fn test_cli_all_hours_flag() {
    let tomorrow = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let day_after = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();

    let output_without = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .arg(format!("--from-date={}", tomorrow))
        .arg(format!("--to-date={}", day_after))
        .output()
        .expect("Failed to execute weatheroz");

    let stdout_without = String::from_utf8_lossy(&output_without.stdout);
    let stderr_without = String::from_utf8_lossy(&output_without.stderr);
    if !output_without.status.success() || !stdout_without.contains("Showing first 24 of") {
        panic!("test_cli_all_hours_flag (without --all-hours) failed.\nStatus: {:?}\nStdout: {}\nStderr: {}", output_without.status, stdout_without, stderr_without);
    }

    let output_with = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .arg(format!("--from-date={}", tomorrow))
        .arg(format!("--to-date={}", day_after))
        .arg("--all-hours")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout_with = String::from_utf8_lossy(&output_with.stdout);
    let stderr_with = String::from_utf8_lossy(&output_with.stderr);
    if !output_with.status.success() || stdout_with.contains("Showing first 24 of") {
        panic!("test_cli_all_hours_flag (with --all-hours) failed.\nStatus: {:?}\nStdout: {}\nStderr: {}", output_with.status, stdout_with, stderr_with);
    }
}

#[tokio::test]
async fn test_cli_historical_date() {
    let output = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .arg("--from-date=2025-01-01")
        .arg("--to-date=2025-01-01")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Historical Data"));
}

#[tokio::test]
async fn test_cli_mixed_date_range() {
    // to-date must stay in the future relative to whenever this test runs, so compute it
    // relative to today instead of using a fixed date that eventually falls into the past.
    let to_date = (chrono::Utc::now().naive_utc().date() + chrono::Duration::days(3))
        .format("%Y-%m-%d")
        .to_string();
    let output = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .arg("--from-date=2025-01-01")
        .arg(format!("--to-date={}", to_date))
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Historical + Forecast"));
}

#[tokio::test]
async fn test_cli_invalid_location() {
    let output = Command::new("./target/debug/weatheroz")
        .arg("Xyzzzzz")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Location not found"));
    assert!(!output.status.success());
}

#[tokio::test]
async fn test_cli_cache_hit() {
    let cache_dir = std::env::temp_dir().join("test_integration_cli_cache_hit_dir");
    let _ = std::fs::create_dir_all(&cache_dir);
    let cache_path = cache_dir.join("test_geo_cache.json");
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let cache_content = format!(
        r#"{{"istanbul":{{"data":{{"name":"Istanbul","latitude":41.0138,"longitude":28.9497,"country":"Republic of Türkiye","admin1":"Istanbul"}},"timestamp":{}}}}}"#,
        now_secs
    );
    std::fs::write(&cache_path, cache_content).unwrap();

    let output = Command::new("./target/debug/weatheroz")
        .env("WEATHEROZ_CACHE_PATH", cache_path.to_str().unwrap())
        .arg("istanbul")
        .arg("--verbose")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("Cache Hit"));

    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_dir(&cache_dir);
}

#[tokio::test]
async fn test_cli_cache_miss() {
    let output = Command::new("./target/debug/weatheroz")
        .arg("nonexistentcity12345")
        .output()
        .expect("Failed to execute weatheroz");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Location not found"));
    assert!(!output.status.success());
}

#[tokio::test]
async fn test_tui_dashboard() {
    let output = Command::new("./target/debug/weatheroz")
        .arg("istanbul")
        .arg("--json-output")
        .output()
        .expect("Failed to execute weatheroz");

    assert!(output.status.success());
}
