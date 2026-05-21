use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AQIData {
    pub co: Option<f64>,
    pub no2: Option<f64>,
    pub o3: Option<f64>,
    pub so2: Option<f64>,
    pub pm2_5: Option<f64>,
    pub pm10: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HourlyPoint {
    pub time: String,
    pub temperature: f64,
    pub apparent_temperature: f64,
    pub precipitation_probability: f64,
    pub precipitation: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub wind_direction: f64,
    pub cloud_cover: f64,
    pub weather_code: i32,
    pub aqi: Option<AQIData>,
    // Enriched fields (Optional for backward compatibility and provider availability)
    pub uv_index: Option<f64>,
    pub is_day: Option<bool>,
    pub visibility: Option<f64>,
    pub soil_temperature: Option<f64>,
    pub soil_moisture: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedWeatherData {
    pub provider_name: String,
    pub hourly: Vec<HourlyPoint>,
}
