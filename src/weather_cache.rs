use crate::providers::models::NormalizedWeatherData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const WEATHER_CACHE_TTL_SECONDS: f64 = 15.0 * 60.0; // default 15 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherCacheEntry {
    pub data: NormalizedWeatherData,
    pub timestamp: f64,
}

fn get_weather_cache_path() -> PathBuf {
    if let Ok(custom_path) = std::env::var("WEATHER_OZ_CACHE_PATH") {
        let path = PathBuf::from(custom_path);
        if path.extension().is_some() {
            if let Some(parent) = path.parent() {
                return parent.join("weather_cache.json");
            }
        }
        return path.join("weather_cache.json");
    }
    let mut path = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    };
    path.push(".cache");
    path.push("weather_oz");
    path.push("weather_cache.json");
    path
}

async fn load_weather_cache_async() -> HashMap<String, WeatherCacheEntry> {
    let path = get_weather_cache_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

async fn save_weather_cache_async(cache: &HashMap<String, WeatherCacheEntry>) {
    let path = get_weather_cache_path();
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    if let Ok(serialized) = serde_json::to_string_pretty(cache) {
        let _ = tokio::fs::write(path, serialized).await;
    }
}

static WEATHER_CACHE: OnceLock<RwLock<HashMap<String, WeatherCacheEntry>>> = OnceLock::new();
static WEATHER_CACHE_LOADED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

static LAST_WEATHER_CACHE_PATH: OnceLock<std::sync::Mutex<PathBuf>> = OnceLock::new();

async fn get_weather_cache() -> &'static RwLock<HashMap<String, WeatherCacheEntry>> {
    let cache_lock = WEATHER_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let current_path = get_weather_cache_path();

    let mut path_changed = false;
    let path_mutex = LAST_WEATHER_CACHE_PATH.get_or_init(|| std::sync::Mutex::new(PathBuf::new()));
    {
        let mut guard = path_mutex.lock().unwrap();
        if *guard != current_path {
            *guard = current_path.clone();
            path_changed = true;
        }
    }

    if path_changed || !WEATHER_CACHE_LOADED.load(std::sync::atomic::Ordering::Acquire) {
        let mut writer = cache_lock.write().await;
        let disk_cache = load_weather_cache_async().await;
        *writer = disk_cache;
        WEATHER_CACHE_LOADED.store(true, std::sync::atomic::Ordering::Release);
    }
    cache_lock
}

fn make_cache_key(lat: f64, lon: f64, start_date: &str, end_date: &str) -> String {
    format!(
        "{:.4}:{:.4}:{}:{}",
        lat,
        lon,
        start_date.trim(),
        end_date.trim()
    )
}

/// Retrieves a weather cache entry.
/// Returns `Some((data, is_fresh, timestamp))` if the cache entry exists, where `is_fresh` is true if the entry is within the specified TTL.
pub async fn get_cached_weather(
    lat: f64,
    lon: f64,
    start_date: &str,
    end_date: &str,
    ttl_seconds: Option<f64>,
) -> Option<(NormalizedWeatherData, bool, f64)> {
    let ttl = ttl_seconds.unwrap_or(WEATHER_CACHE_TTL_SECONDS);
    let key = make_cache_key(lat, lon, start_date, end_date);
    let cache_lock = get_weather_cache().await;
    {
        let reader = cache_lock.read().await;
        if let Some(entry) = reader.get(&key) {
            let now_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();

            let age = now_secs - entry.timestamp;
            let is_fresh = age >= 0.0 && age < ttl;

            return Some((entry.data.clone(), is_fresh, entry.timestamp));
        }
    }

    // Try reloading from disk on memory miss
    {
        let disk_cache = load_weather_cache_async().await;
        let mut writer = cache_lock.write().await;
        *writer = disk_cache;
    }

    // Recheck after disk reload
    {
        let reader = cache_lock.read().await;
        let entry = reader.get(&key)?;
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        let age = now_secs - entry.timestamp;
        let is_fresh = age >= 0.0 && age < ttl;

        Some((entry.data.clone(), is_fresh, entry.timestamp))
    }
}

/// Saves weather data to the cache.
pub async fn save_cached_weather(
    lat: f64,
    lon: f64,
    start_date: &str,
    end_date: &str,
    data: &NormalizedWeatherData,
) {
    let key = make_cache_key(lat, lon, start_date, end_date);
    let cache_lock = get_weather_cache().await;
    let mut writer = cache_lock.write().await;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    writer.insert(
        key,
        WeatherCacheEntry {
            data: data.clone(),
            timestamp: now_secs,
        },
    );

    let cache_clone = writer.clone();
    tokio::spawn(async move {
        save_weather_cache_async(&cache_clone).await;
    });
}
