use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};
use serde::Deserialize;

#[derive(Clone)]
pub struct WeatherStackProvider;

#[derive(Deserialize, Debug)]
struct WeatherStackCurrent {
    temperature: Option<f64>,
    feelslike: Option<f64>,
    humidity: Option<f64>,
    wind_speed: Option<f64>,
    wind_degree: Option<f64>,
    cloudcover: Option<f64>,
    weather_code: Option<i32>,
    uv_index: Option<f64>,
    visibility: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct WeatherStackResponse {
    current: Option<WeatherStackCurrent>,
}

impl BaseWeatherProvider for WeatherStackProvider {
    fn name(&self) -> &str {
        "WeatherStack"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("WeatherStack") {
            Some(k) => k,
            None => return Err("WeatherStack API key is missing. Configure it in config.json or set WEATHER_KEY_WEATHERSTACK.".into()),
        };

        // WeatherStack free API only provides current weather.
        let url = format!(
            "http://api.weatherstack.com/current?access_key={}&query={},{}",
            key, ctx.lat, ctx.lon
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("WeatherStack HTTP error {}", resp.status()).into());
        }

        let raw: WeatherStackResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(curr) = raw.current {
            let temp = curr.temperature.unwrap_or(0.0);
            let feels = curr.feelslike.unwrap_or(temp);
            let humidity = curr.humidity.unwrap_or(0.0);
            let wind_sp = curr.wind_speed.unwrap_or(0.0);
            let wind_dir = curr.wind_degree.unwrap_or(0.0);
            let cloud = curr.cloudcover.unwrap_or(0.0);
            let code = curr.weather_code.unwrap_or(113);

            // Map WeatherStack codes to WMO
            let wmo = match code {
                113 => 0,        // Clear
                116 => 2,        // Partly cloudy
                119 | 122 => 3,  // Cloudy/Overcast
                143 | 248 => 45, // Fog
                293..=308 => 63, // Rain
                323..=338 => 73, // Snow
                386..=395 => 95, // Thunder
                _ => 0,
            };

            // Since WeatherStack only gives current weather, we map it to 24 hours of the start date
            let today = chrono::Utc::now().naive_utc().date();
            let start =
                chrono::NaiveDate::parse_from_str(ctx.start_date, "%Y-%m-%d").unwrap_or(today);
            for h in 0..24 {
                points.push(HourlyPoint {
                    time: format!("{}T{:02}:00:00", start.format("%Y-%m-%d"), h),
                    temperature: temp,
                    apparent_temperature: feels,
                    precipitation_probability: 0.0,
                    precipitation: 0.0,
                    humidity,
                    wind_speed: wind_sp,
                    wind_direction: wind_dir,
                    cloud_cover: cloud,
                    weather_code: wmo,
                    aqi: None,
                    uv_index: curr.uv_index,
                    is_day: None,
                    visibility: curr.visibility,
                    soil_temperature: None,
                    soil_moisture: None,
                });
            }
        }

        Ok(NormalizedWeatherData {
            provider_name: self.name().to_string(),
            hourly: points,
        })
    }
}
