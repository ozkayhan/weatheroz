use std::sync::Arc;
use std::collections::HashMap;
use crate::providers::models::NormalizedWeatherData;

pub struct FetchContext<'a> {
    pub client: Arc<reqwest::Client>,
    pub lat: f64,
    pub lon: f64,
    pub start_date: &'a str,
    pub end_date: &'a str,
    pub api_keys: &'a HashMap<String, String>,
    pub enrich: bool,
    pub days: u32,
    pub minute_resolution: bool,
}

#[allow(async_fn_in_trait)]
pub trait BaseWeatherProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>>;
}
