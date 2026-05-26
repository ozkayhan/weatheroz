use crate::tui::{conditional_sleep, SharedState};
use deunicode::deunicode;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const CACHE_TTL_SECONDS: f64 = 30.0 * 24.0 * 60.0 * 60.0;

const PREFIXES_TO_STRIP: &[&str] = &[
    "eski", "yeni", "new", "old", "upper", "lower", "north", "south", "east", "west",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeocodedLocation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: String,
    pub admin1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: GeocodedLocation,
    pub timestamp: f64,
}

fn get_cache_path() -> PathBuf {
    if let Ok(custom_path) = std::env::var("WEATHEROZ_CACHE_PATH") {
        return PathBuf::from(custom_path);
    }
    let mut path = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    };
    path.push(".cache");
    path.push("weatheroz");
    path.push("geo_cache.json");
    path
}

async fn load_cache_async() -> HashMap<String, CacheEntry> {
    let path = get_cache_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

async fn save_cache_async(cache: &HashMap<String, CacheEntry>) {
    let path = get_cache_path();
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Ok(serialized) = serde_json::to_string_pretty(cache) {
        let _ = tokio::fs::write(path, serialized).await;
    }
}

static GEO_CACHE: OnceLock<RwLock<HashMap<String, CacheEntry>>> = OnceLock::new();
static GEO_CACHE_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

static LAST_GEO_CACHE_PATH: OnceLock<std::sync::Mutex<PathBuf>> = OnceLock::new();

async fn get_geo_cache() -> &'static RwLock<HashMap<String, CacheEntry>> {
    let cache_lock = GEO_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let current_path = get_cache_path();

    let mut path_changed = false;
    let path_mutex = LAST_GEO_CACHE_PATH.get_or_init(|| std::sync::Mutex::new(PathBuf::new()));
    {
        let mut guard = path_mutex.lock().unwrap();
        if *guard != current_path {
            *guard = current_path.clone();
            path_changed = true;
        }
    }

    if path_changed || !GEO_CACHE_LOADED.load(std::sync::atomic::Ordering::Acquire) {
        let mut writer = cache_lock.write().await;
        let disk_cache = load_cache_async().await;
        *writer = disk_cache;
        GEO_CACHE_LOADED.store(true, std::sync::atomic::Ordering::Release);
    }
    cache_lock
}

pub fn normalize_query(query: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let words: Vec<&str> = query.split_whitespace().collect();
    if !words.is_empty() && PREFIXES_TO_STRIP.contains(&words[0].to_lowercase().as_str()) {
        let stripped = words[1..].join(" ");
        if !stripped.is_empty() {
            candidates.push(stripped);
        }
    }
    candidates.push(query.to_string());

    let replaced = query.replace(',', " ");
    let parts: Vec<&str> = replaced
        .split_whitespace()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    for part in parts {
        if !PREFIXES_TO_STRIP.contains(&part.to_lowercase().as_str()) {
            candidates.push(part.to_string());
        }
    }

    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for c in candidates {
        let key = c.to_lowercase().trim().to_string();
        if !seen.contains(&key) {
            seen.insert(key);
            deduped.push(c);
        }
    }

    let mut ascii_candidates = Vec::new();
    for c in &deduped {
        let folded = deunicode(c);
        if folded != *c && !folded.is_empty() {
            ascii_candidates.push(folded);
        }
    }

    let mut result = deduped;
    result.extend(ascii_candidates);
    result
}

#[derive(Deserialize)]
struct GeocodingApiResponse {
    results: Option<Vec<GeocodingApiResult>>,
}

#[derive(Deserialize)]
struct GeocodingApiResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    admin1: Option<String>,
}

async fn fetch_geocoding(
    client: Arc<reqwest::Client>,
    query: &str,
) -> Result<Vec<GeocodingApiResult>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=5&language=en&format=json",
        urlencoding::encode(query)
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(format!("HTTP error {}", resp.status()).into());
    }
    let parsed: GeocodingApiResponse = resp.json().await?;
    Ok(parsed.results.unwrap_or_default())
}

pub async fn resolve_location(
    client: Arc<reqwest::Client>,
    query: &str,
    state: Option<&SharedState>,
) -> Result<GeocodedLocation, Box<dyn std::error::Error + Send + Sync>> {
    let query_key = query.to_lowercase().trim().to_string();

    if let Some(s) = state {
        let mut guard = s.lock().unwrap();
        guard.step_geocoding = "running".to_string();
        guard.global_progress = 10;
    }
    tracing::info!("Querying cache for '{}'...", query);
    conditional_sleep(state, 150).await;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    let cache_lock = get_geo_cache().await;
    {
        let reader = cache_lock.read().await;
        if let Some(entry) = reader.get(&query_key) {
            if now_secs - entry.timestamp < CACHE_TTL_SECONDS {
                if let Some(s) = state {
                    let mut guard = s.lock().unwrap();
                    guard.cache_status = "hit".to_string();
                    guard.resolved_location =
                        Some(format!("{}, {}", entry.data.name, entry.data.country));
                    guard.global_progress = 40;
                }
                tracing::info!("⚡ Cache Hit: '{}' loaded!", query);
                conditional_sleep(state, 150).await;
                if let Some(s) = state {
                    let mut guard = s.lock().unwrap();
                    guard.step_geocoding = "completed".to_string();
                    guard.global_progress = 50;
                }
                conditional_sleep(state, 100).await;
                return Ok(entry.data.clone());
            }
        }
    }

    // Try reloading from disk if not found or expired in memory
    {
        let disk_cache = load_cache_async().await;
        let mut writer = cache_lock.write().await;
        *writer = disk_cache;
    }

    // Recheck after disk reload
    {
        let reader = cache_lock.read().await;
        if let Some(entry) = reader.get(&query_key) {
            if now_secs - entry.timestamp < CACHE_TTL_SECONDS {
                if let Some(s) = state {
                    let mut guard = s.lock().unwrap();
                    guard.cache_status = "hit".to_string();
                    guard.resolved_location =
                        Some(format!("{}, {}", entry.data.name, entry.data.country));
                    guard.global_progress = 40;
                }
                tracing::info!("⚡ Cache Hit: '{}' loaded!", query);
                conditional_sleep(state, 150).await;
                if let Some(s) = state {
                    let mut guard = s.lock().unwrap();
                    guard.step_geocoding = "completed".to_string();
                    guard.global_progress = 50;
                }
                conditional_sleep(state, 100).await;
                return Ok(entry.data.clone());
            }
        }
    }

    if let Some(s) = state {
        let mut guard = s.lock().unwrap();
        guard.cache_status = "miss".to_string();
        guard.global_progress = 25;
    }
    tracing::info!("🔍 Cache Miss: Querying Geocoding API for '{}'...", query);
    conditional_sleep(state, 150).await;

    let candidates = normalize_query(query);
    for candidate in candidates {
        tracing::info!("📡 Querying Geocoding API for '{}'...", candidate);
        conditional_sleep(state, 100).await;

        match fetch_geocoding(client.clone(), &candidate).await {
            Ok(results) => {
                if !results.is_empty() {
                    let best = &results[0];
                    let resolved = GeocodedLocation {
                        name: best.name.clone(),
                        latitude: (best.latitude * 10000.0).round() / 10000.0,
                        longitude: (best.longitude * 10000.0).round() / 10000.0,
                        country: best.country.clone().unwrap_or_default(),
                        admin1: best.admin1.clone().unwrap_or_default(),
                    };

                    {
                        let mut writer = cache_lock.write().await;
                        writer.insert(
                            query_key.clone(),
                            CacheEntry {
                                data: resolved.clone(),
                                timestamp: now_secs,
                            },
                        );
                        let cache_clone = writer.clone();
                        tokio::spawn(async move {
                            save_cache_async(&cache_clone).await;
                        });
                    }

                    if let Some(s) = state {
                        let mut guard = s.lock().unwrap();
                        guard.resolved_location =
                            Some(format!("{}, {}", resolved.name, resolved.country));
                        guard.global_progress = 45;
                    }
                    tracing::info!(
                        "✅ Location resolved: {}, {}",
                        resolved.name,
                        resolved.country
                    );
                    conditional_sleep(state, 150).await;

                    if let Some(s) = state {
                        let mut guard = s.lock().unwrap();
                        guard.step_geocoding = "completed".to_string();
                        guard.global_progress = 50;
                    }
                    conditional_sleep(state, 100).await;

                    return Ok(resolved);
                }
            }
            Err(e) => {
                if let Some(s) = state {
                    let mut guard = s.lock().unwrap();
                    guard.step_geocoding = "failed".to_string();
                }
                tracing::error!("❌ Geocoding Error: {}", e);
                conditional_sleep(state, 150).await;
                return Err(e);
            }
        }
    }

    if let Some(s) = state {
        let mut guard = s.lock().unwrap();
        guard.step_geocoding = "failed".to_string();
    }
    tracing::error!("❌ Location not found: '{}'", query);
    conditional_sleep(state, 150).await;
    Err(format!("Location not found: {}", query).into())
}

#[derive(Deserialize, Debug)]
struct IpApiResponse {
    status: String,
    country: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    city: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
}

pub async fn resolve_ip_location(
    client: Arc<reqwest::Client>,
    url_override: Option<&str>,
    state: Option<&SharedState>,
) -> Result<GeocodedLocation, Box<dyn std::error::Error + Send + Sync>> {
    let url = url_override.unwrap_or("http://ip-api.com/json/");

    if let Some(s) = state {
        let mut guard = s.lock().unwrap();
        guard.step_geocoding = "running".to_string();
        guard.global_progress = 10;
    }
    tracing::info!("📡 Detecting location via IP address...");

    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(format!("IP Geolocation API error: HTTP {}", resp.status()).into());
    }

    let parsed: IpApiResponse = resp.json().await?;
    if parsed.status != "success" {
        return Err(format!("IP Geolocation failed: status is {}", parsed.status).into());
    }

    let lat = parsed.lat.ok_or("Missing latitude in IP API response")?;
    let lon = parsed.lon.ok_or("Missing longitude in IP API response")?;

    let resolved = GeocodedLocation {
        name: parsed
            .city
            .clone()
            .unwrap_or_else(|| "Unknown City".to_string()),
        latitude: (lat * 10000.0).round() / 10000.0,
        longitude: (lon * 10000.0).round() / 10000.0,
        country: parsed.country.clone().unwrap_or_default(),
        admin1: parsed.region_name.clone().unwrap_or_default(),
    };

    if let Some(s) = state {
        let mut guard = s.lock().unwrap();
        guard.resolved_location = Some(format!("{}, {}", resolved.name, resolved.country));
        guard.global_progress = 45;
        guard.step_geocoding = "completed".to_string();
    }
    tracing::info!(
        "✅ IP Location resolved: {}, {}",
        resolved.name,
        resolved.country
    );

    Ok(resolved)
}
