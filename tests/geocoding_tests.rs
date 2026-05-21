use weather_oz::geocoding::normalize_query;

#[test]
fn test_normalize_query_stripping() {
    let result = normalize_query("Yeni Istanbul");
    assert!(result.contains(&"Istanbul".to_string()));
    assert!(
        result.contains(&"yeni istanbul".to_string())
            || result.contains(&"Yeni Istanbul".to_string())
    );
}

#[test]
fn test_normalize_query_ascii_folding() {
    let result = normalize_query("İstanbul");
    assert!(result.contains(&"Istanbul".to_string()));
}

#[tokio::test]
async fn test_coordinate_precision() {
    use weather_oz::geocoding::resolve_location;
    let client = std::sync::Arc::new(reqwest::Client::new());
    let result = resolve_location(client, "istanbul", None).await;
    assert!(result.is_ok());
    let loc = result.unwrap();
    assert_eq!(loc.latitude, (loc.latitude * 10000.0).round() / 10000.0);
    assert_eq!(loc.longitude, (loc.longitude * 10000.0).round() / 10000.0);
}

#[tokio::test]
async fn test_cache_expiration_ttl() {
    use std::time::SystemTime;
    use weather_oz::geocoding::resolve_location;

    let client = std::sync::Arc::new(reqwest::Client::new());
    let cache_dir = std::env::temp_dir();
    let cache_path = cache_dir.join("test_ttl_geo_cache.json");

    std::env::set_var("WEATHER_OZ_CACHE_PATH", cache_path.to_str().unwrap());
    let _ = std::fs::remove_file(&cache_path);

    let now_secs = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let expired_secs = now_secs - (31.0 * 24.0 * 60.0 * 60.0);
    let expired_content = format!(
        r#"{{"expiredcity":{{"data":{{"name":"ExpiredCity","latitude":40.1234,"longitude":29.1234,"country":"Turkey","admin1":"Bursa"}},"timestamp":{}}}}}"#,
        expired_secs
    );
    std::fs::write(&cache_path, expired_content).unwrap();

    let result_expired = resolve_location(client.clone(), "expiredcity", None).await;
    assert!(result_expired.is_err());

    let valid_secs = now_secs - (1.0 * 24.0 * 60.0 * 60.0);
    let valid_content = format!(
        r#"{{"validcity":{{"data":{{"name":"ValidCity","latitude":40.1234,"longitude":29.1234,"country":"Turkey","admin1":"Bursa"}},"timestamp":{}}}}}"#,
        valid_secs
    );
    std::fs::write(&cache_path, valid_content).unwrap();

    let result_valid = resolve_location(client, "validcity", None).await;
    assert!(result_valid.is_ok());
    let loc = result_valid.unwrap();
    assert_eq!(loc.name, "ValidCity");
    assert_eq!(loc.latitude, 40.1234);

    let _ = std::fs::remove_file(&cache_path);
    std::env::remove_var("WEATHER_OZ_CACHE_PATH");
}

#[tokio::test]
async fn test_resolve_ip_location_success() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use weather_oz::geocoding::resolve_ip_location;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mock_url = format!("http://{}:{}/json", addr.ip(), addr.port());

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0; 1024];
            let _ = socket.read(&mut buf).await;
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\n  \"status\": \"success\",\n  \"country\": \"Turkey\",\n  \"regionName\": \"Istanbul\",\n  \"city\": \"Istanbul\",\n  \"lat\": 41.01382,\n  \"lon\": 28.94978\n}";
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    let client = std::sync::Arc::new(reqwest::Client::new());
    let result = resolve_ip_location(client, Some(&mock_url), None).await;
    assert!(result.is_ok());
    let loc = result.unwrap();
    assert_eq!(loc.name, "Istanbul");
    assert_eq!(loc.country, "Turkey");
    assert_eq!(loc.admin1, "Istanbul");
    assert_eq!(loc.latitude, 41.0138);
    assert_eq!(loc.longitude, 28.9498);
}

#[tokio::test]
async fn test_resolve_ip_location_failure() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use weather_oz::geocoding::resolve_ip_location;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mock_url = format!("http://{}:{}/json", addr.ip(), addr.port());

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0; 1024];
            let _ = socket.read(&mut buf).await;
            let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\n  \"status\": \"fail\",\n  \"message\": \"invalid query\"\n}";
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    let client = std::sync::Arc::new(reqwest::Client::new());
    let result = resolve_ip_location(client, Some(&mock_url), None).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("IP Geolocation failed"));
    assert!(err_msg.contains("status is fail"));
}
