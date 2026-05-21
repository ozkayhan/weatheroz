use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};
use serde::Deserialize;

#[derive(Clone)]
pub struct BrightSkyProvider;

#[derive(Deserialize, Debug)]
struct BrightSkyWeatherRecord {
    timestamp: String,
    temperature: Option<f64>,
    precipitation: Option<f64>,
    relative_humidity: Option<f64>,
    wind_speed: Option<f64>,
    wind_direction: Option<f64>,
    cloud_cover: Option<f64>,
    icon: Option<String>,
}

#[derive(Deserialize, Debug)]
struct BrightSkyResponse {
    weather: Option<Vec<BrightSkyWeatherRecord>>,
}

fn icon_to_wmo(icon: &str) -> i32 {
    match icon {
        "clear-day" | "clear-night" => 0,
        "partly-cloudy-day" | "partly-cloudy-night" => 2,
        "cloudy" => 3,
        "fog" => 45,
        "rain" => 63,
        "snow" => 73,
        "sleet" => 77,
        "wind" => 1,
        _ => 0,
    }
}

impl BaseWeatherProvider for BrightSkyProvider {
    fn name(&self) -> &str {
        "Bright Sky"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.brightsky.dev/weather?lat={}&lon={}&date={}&last_date={}",
            ctx.lat, ctx.lon, ctx.start_date, ctx.end_date
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Bright Sky HTTP error {}", resp.status()).into());
        }

        let raw: BrightSkyResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(records) = raw.weather {
            for rec in records {
                let time_formatted = rec
                    .timestamp
                    .replace('Z', "")
                    .split('+')
                    .next()
                    .unwrap_or("")
                    .to_string();
                let temp = rec.temperature.unwrap_or(0.0);
                let precip = rec.precipitation.unwrap_or(0.0);
                let hum = rec.relative_humidity.unwrap_or(0.0);
                let wind_sp = rec.wind_speed.unwrap_or(0.0) * 3.6; // convert m/s to km/h
                let wind_dir = rec.wind_direction.unwrap_or(0.0);
                let cloud = rec.cloud_cover.unwrap_or(0.0);
                let wmo = icon_to_wmo(rec.icon.as_deref().unwrap_or(""));

                points.push(HourlyPoint {
                    time: time_formatted,
                    temperature: temp,
                    apparent_temperature: temp,
                    precipitation_probability: if precip > 0.0 { 100.0 } else { 0.0 },
                    precipitation: precip,
                    humidity: hum,
                    wind_speed: wind_sp,
                    wind_direction: wind_dir,
                    cloud_cover: cloud,
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
