use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};
use serde::Deserialize;

#[derive(Clone)]
pub struct YandexProvider;

#[derive(Deserialize, Debug)]
struct YandexFact {
    temp: Option<f64>,
    feels_like: Option<f64>,
    humidity: Option<f64>,
    wind_speed: Option<f64>,
    wind_dir: Option<String>,
    cloudness: Option<f64>,
    condition: Option<String>,
}

#[derive(Deserialize, Debug)]
struct YandexResponse {
    fact: Option<YandexFact>,
}

fn dir_to_degree(dir: &str) -> f64 {
    match dir {
        "n" => 0.0,
        "ne" => 45.0,
        "e" => 90.0,
        "se" => 135.0,
        "s" => 180.0,
        "sw" => 225.0,
        "w" => 270.0,
        "nw" => 315.0,
        _ => 0.0,
    }
}

fn condition_to_wmo(cond: &str) -> i32 {
    match cond {
        "clear" => 0,
        "partly-cloudy" => 2,
        "cloudy" | "overcast" => 3,
        "drizzle" | "light-rain" => 51,
        "rain" | "heavy-rain" => 63,
        "showers" => 80,
        "snow" | "snow-showers" => 73,
        "thunderstorm" | "thunderstorm-with-rain" => 95,
        _ => 0,
    }
}

impl BaseWeatherProvider for YandexProvider {
    fn name(&self) -> &str {
        "Yandex"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("Yandex") {
            Some(k) => k,
            None => return Err(
                "Yandex API key is missing. Configure it in config.json or set WEATHER_KEY_YANDEX."
                    .into(),
            ),
        };

        let url = format!(
            "https://api.weather.yandex.ru/v2/forecast?lat={}&lon={}",
            ctx.lat, ctx.lon
        );

        let resp = ctx
            .client
            .get(&url)
            .header("X-Yandex-API-Key", key)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("Yandex HTTP error {}", resp.status()).into());
        }

        let raw: YandexResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(fact) = raw.fact {
            let temp = fact.temp.unwrap_or(0.0);
            let feels = fact.feels_like.unwrap_or(temp);
            let humidity = fact.humidity.unwrap_or(0.0);
            let wind_sp = fact.wind_speed.unwrap_or(0.0) * 3.6; // m/s to km/h
            let wind_dir = dir_to_degree(fact.wind_dir.as_deref().unwrap_or(""));
            let cloud = fact.cloudness.unwrap_or(0.0) * 100.0; // 0.0..1.0 to %
            let wmo = condition_to_wmo(fact.condition.as_deref().unwrap_or(""));

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
