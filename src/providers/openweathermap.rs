use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};
use serde::Deserialize;

#[derive(Clone)]
pub struct OpenWeatherMapProvider;

#[derive(Deserialize, Debug)]
struct OwmTemp {
    temp: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct OwmWeather {
    id: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct OwmForecastItem {
    dt: Option<i64>,
    main: Option<OwmTemp>,
    weather: Option<Vec<OwmWeather>>,
}

#[derive(Deserialize, Debug)]
struct OwmForecastResponse {
    list: Option<Vec<OwmForecastItem>>,
}

impl BaseWeatherProvider for OpenWeatherMapProvider {
    fn name(&self) -> &str {
        "OpenWeatherMap"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("OpenWeatherMap") {
            Some(k) => k,
            None => return Err("OpenWeatherMap API key is missing. Configure it in config.json or set WEATHER_KEY_OPENWEATHERMAP.".into()),
        };

        // Call OpenWeatherMap 5-day / 3-hour forecast
        let url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?lat={}&lon={}&appid={}&units=metric",
            ctx.lat, ctx.lon, key
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("OpenWeatherMap HTTP error {}", resp.status()).into());
        }

        let raw: OwmForecastResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(list) = raw.list {
            for item in list {
                let dt = item.dt.unwrap_or(0);
                let naive = chrono::DateTime::from_timestamp(dt, 0)
                    .map(|dt_utc| dt_utc.naive_utc())
                    .unwrap_or_default();
                let time_str = naive.format("%Y-%m-%dT%H:%M:%S").to_string();
                let date_part = time_str.split('T').next().unwrap_or("");
                if date_part < ctx.start_date || date_part > ctx.end_date {
                    continue;
                }

                let temp = item.main.and_then(|m| m.temp).unwrap_or(0.0);
                let code = item
                    .weather
                    .and_then(|w| w.first().and_then(|x| x.id))
                    .unwrap_or(800);

                // Map OWM codes (2xx, 3xx, 5xx, 6xx, 7xx, 8xx) to WMO codes
                let wmo = match code {
                    200..=232 => 95, // Thunderstorm
                    300..=321 => 51, // Drizzle
                    500..=531 => 63, // Rain
                    600..=622 => 73, // Snow
                    701..=781 => 45, // Fog/Mist
                    800 => 0,        // Clear
                    801..=804 => 3,  // Cloudy
                    _ => 0,
                };

                points.push(HourlyPoint {
                    time: time_str,
                    temperature: temp,
                    apparent_temperature: temp,
                    precipitation_probability: 0.0,
                    precipitation: 0.0,
                    humidity: 60.0,
                    wind_speed: 10.0,
                    wind_direction: 180.0,
                    cloud_cover: 50.0,
                    weather_code: wmo,
                    aqi: None,
                    uv_index: None,
                    is_day: None,
                    visibility: None,
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
