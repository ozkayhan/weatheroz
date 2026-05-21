use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};
use serde::Deserialize;

#[derive(Clone)]
pub struct WeatherbitProvider;

#[derive(Deserialize, Debug)]
struct WeatherbitWeather {
    code: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct WeatherbitHourlyItem {
    timestamp_local: Option<String>,
    temp: Option<f64>,
    app_temp: Option<f64>,
    pop: Option<f64>,
    precip: Option<f64>,
    rh: Option<f64>,
    wind_spd: Option<f64>,
    wind_dir: Option<f64>,
    clouds: Option<f64>,
    weather: Option<WeatherbitWeather>,
    uv: Option<f64>,
    vis: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct WeatherbitResponse {
    data: Option<Vec<WeatherbitHourlyItem>>,
}

impl BaseWeatherProvider for WeatherbitProvider {
    fn name(&self) -> &str {
        "Weatherbit"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("Weatherbit") {
            Some(k) => k,
            None => return Err("Weatherbit API key is missing. Configure it in config.json or set WEATHER_KEY_WEATHERBIT.".into()),
        };

        let url = format!(
            "https://api.weatherbit.io/v2.0/forecast/hourly?lat={}&lon={}&key={}&hours={}",
            ctx.lat,
            ctx.lon,
            key,
            ctx.days * 24
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Weatherbit HTTP error {}", resp.status()).into());
        }

        let raw: WeatherbitResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(data) = raw.data {
            for item in data {
                let time_str = item.timestamp_local.unwrap_or_default(); // "2026-05-21T08:00:00"
                let date_part = time_str.split('T').next().unwrap_or("");
                if date_part < ctx.start_date || date_part > ctx.end_date {
                    continue;
                }

                let temp = item.temp.unwrap_or(0.0);
                let app = item.app_temp.unwrap_or(temp);
                let code = item.weather.and_then(|w| w.code).unwrap_or(800);

                // Map Weatherbit code to WMO
                let wmo = match code {
                    200..=233 => 95,             // Thunderstorm
                    300..=302 | 500..=522 => 63, // Rain
                    600..=623 => 73,             // Snow
                    700..=751 => 45,             // Fog
                    800 => 0,                    // Clear
                    801..=804 => 3,              // Clouds
                    _ => 0,
                };

                points.push(HourlyPoint {
                    time: time_str,
                    temperature: temp,
                    apparent_temperature: app,
                    precipitation_probability: item.pop.unwrap_or(0.0),
                    precipitation: item.precip.unwrap_or(0.0),
                    humidity: item.rh.unwrap_or(0.0),
                    wind_speed: item.wind_spd.unwrap_or(0.0) * 3.6, // m/s to km/h
                    wind_direction: item.wind_dir.unwrap_or(0.0),
                    cloud_cover: item.clouds.unwrap_or(0.0),
                    weather_code: wmo,
                    aqi: None,
                    uv_index: item.uv,
                    is_day: None,
                    visibility: item.vis,
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
