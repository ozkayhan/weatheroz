use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_location: Option<String>,
    pub temperature_unit: String,
    pub api_keys: HashMap<String, String>,
    pub race_providers: Option<Vec<String>>,
    pub fallback_providers: Option<Vec<String>>,
    pub cache_ttl_minutes: Option<u64>,
    pub forecast_days: Option<u32>,
    pub minute_updates: Option<bool>,
    pub enrich_data: Option<bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_location: None,
            temperature_unit: "celsius".to_string(),
            api_keys: HashMap::new(),
            race_providers: Some(vec![
                "Open-Meteo".to_string(),
                "MET Norway".to_string(),
                "wttr.in".to_string(),
            ]),
            fallback_providers: Some(vec!["Bright Sky".to_string()]),
            cache_ttl_minutes: Some(15),
            forecast_days: Some(7),
            minute_updates: Some(false),
            enrich_data: Some(false),
        }
    }
}

#[async_trait]
pub trait ConfigService: Send + Sync {
    async fn load_config(&self) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>>;
    async fn save_config(
        &self,
        config: &AppConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct JsonConfigService {
    config_path: PathBuf,
}

impl Default for JsonConfigService {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonConfigService {
    pub fn new() -> Self {
        Self {
            config_path: Self::get_default_path(),
        }
    }

    fn get_default_path() -> PathBuf {
        if let Ok(custom_path) = std::env::var("WEATHER_OZ_CONFIG_PATH") {
            return PathBuf::from(custom_path);
        }
        let mut path = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from(".")
        };
        path.push(".config");
        path.push("weather_oz");
        path.push("config.json");
        path
    }
}

#[async_trait]
impl ConfigService for JsonConfigService {
    async fn load_config(&self) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
        let mut config = if self.config_path.exists() {
            match tokio::fs::read_to_string(&self.config_path).await {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => AppConfig::default(),
            }
        } else {
            AppConfig::default()
        };

        // Apply environment overrides
        if let Ok(loc) = std::env::var("WEATHER_DEFAULT_LOCATION") {
            config.default_location = Some(loc);
        }
        if let Ok(unit) = std::env::var("WEATHER_TEMPERATURE_UNIT") {
            config.temperature_unit = unit;
        }
        if let Ok(ttl) = std::env::var("WEATHER_CACHE_TTL_MINUTES") {
            if let Ok(parsed) = ttl.parse::<u64>() {
                config.cache_ttl_minutes = Some(parsed);
            }
        }
        if let Ok(days) = std::env::var("WEATHER_FORECAST_DAYS") {
            if let Ok(parsed) = days.parse::<u32>() {
                config.forecast_days = Some(parsed);
            }
        }
        if let Ok(min_up) = std::env::var("WEATHER_MINUTE_UPDATES") {
            config.minute_updates = Some(min_up == "true" || min_up == "1");
        }
        if let Ok(enrich) = std::env::var("WEATHER_ENRICH_DATA") {
            config.enrich_data = Some(enrich == "true" || enrich == "1");
        }

        // Look for environment variable keys, e.g. WEATHER_KEY_OPENWEATHERMAP
        for (key, val) in std::env::vars() {
            if key.starts_with("WEATHER_KEY_") {
                let provider_name = key.trim_start_matches("WEATHER_KEY_").replace('_', "-");
                config.api_keys.insert(provider_name, val);
            }
        }

        Ok(config)
    }

    async fn save_config(
        &self,
        config: &AppConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let serialized = serde_json::to_string_pretty(config)?;
        tokio::fs::write(&self.config_path, serialized).await?;
        Ok(())
    }
}
