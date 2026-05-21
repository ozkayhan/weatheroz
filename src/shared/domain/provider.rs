use crate::shared::domain::models::NormalizedWeatherData;
use async_trait::async_trait;

#[async_trait]
pub trait WeatherProvider: Send + Sync {
    fn provider_name(&self) -> &str;

    async fn fetch_weather(
        &self,
        client: std::sync::Arc<reqwest::Client>,
        lat: f64,
        lon: f64,
        start_date: &str,
        end_date: &str,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>>;
}
