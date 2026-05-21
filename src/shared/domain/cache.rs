use async_trait::async_trait;
use crate::shared::domain::models::NormalizedWeatherData;

#[async_trait]
pub trait CacheService: Send + Sync {
    /// Tries to fetch cached data for the coordinates.
    async fn fetch_cached_weather(
        &self, 
        lat: f64, 
        lon: f64, 
        start_date: &str, 
        end_date: &str
    ) -> Option<(NormalizedWeatherData, bool, f64)>;

    /// Saves freshly fetched data into the cache.
    async fn write_cache_weather(
        &self, 
        lat: f64, 
        lon: f64, 
        start_date: &str, 
        end_date: &str, 
        data: &NormalizedWeatherData
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
