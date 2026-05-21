use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct PirateWeatherProvider;

#[derive(Deserialize, Debug)]
struct PirateDataPoint {
    time: Option<i64>,
    temperature: Option<f64>,
    #[serde(rename = "apparentTemperature")]
    apparent_temp: Option<f64>,
    #[serde(rename = "precipProbability")]
    precip_prob: Option<f64>,
    #[serde(rename = "precipIntensity")]
    precip_intensity: Option<f64>,
    humidity: Option<f64>,
    #[serde(rename = "windSpeed")]
    wind_speed: Option<f64>,
    #[serde(rename = "windBearing")]
    wind_bearing: Option<f64>,
    #[serde(rename = "cloudCover")]
    cloud_cover: Option<f64>,
    icon: Option<String>,
    #[serde(rename = "uvIndex")]
    uv: Option<f64>,
    visibility: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct PirateHourlyBlock {
    data: Option<Vec<PirateDataPoint>>,
}

#[derive(Deserialize, Debug)]
struct PirateResponse {
    hourly: Option<PirateHourlyBlock>,
}

fn icon_to_wmo(icon: &str) -> i32 {
    match icon {
        "clear-day" | "clear-night" => 0,
        "partly-cloudy-day" | "partly-cloudy-night" => 2,
        "cloudy" => 3,
        "fog" => 45,
        "wind" => 1,
        "rain" => 63,
        "snow" => 73,
        "sleet" => 77,
        _ => 0,
    }
}

impl BaseWeatherProvider for PirateWeatherProvider {
    fn name(&self) -> &str {
        "Pirate Weather"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("Pirate Weather") {
            Some(k) => k,
            None => return Err("Pirate Weather API key is missing. Configure it in config.json or set WEATHER_KEY_PIRATE_WEATHER.".into()),
        };

        let url = format!(
            "https://api.pirateweather.net/forecast/{}/{},{}?units=si",
            key, ctx.lat, ctx.lon
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Pirate Weather HTTP error {}", resp.status()).into());
        }

        let raw: PirateResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(hourly) = raw.hourly {
            if let Some(data) = hourly.data {
                for item in data {
                    let ts = item.time.unwrap_or(0);
                    let naive = chrono::DateTime::from_timestamp(ts, 0).map(|dt_utc| dt_utc.naive_utc()).unwrap_or_default();
                    let time_formatted = naive.format("%Y-%m-%dT%H:%M:%S").to_string();
                    let date_part = time_formatted.split('T').next().unwrap_or("");
                    if date_part < ctx.start_date || date_part > ctx.end_date {
                        continue;
                    }

                    let temp = item.temperature.unwrap_or(0.0);
                    let app = item.apparent_temp.unwrap_or(temp);
                    let wmo = icon_to_wmo(item.icon.as_deref().unwrap_or(""));

                    points.push(HourlyPoint {
                        time: time_formatted,
                        temperature: temp,
                        apparent_temperature: app,
                        precipitation_probability: item.precip_prob.unwrap_or(0.0) * 100.0, // 0..1 to %
                        precipitation: item.precip_intensity.unwrap_or(0.0),
                        humidity: item.humidity.unwrap_or(0.0) * 100.0, // 0..1 to %
                        wind_speed: item.wind_speed.unwrap_or(0.0) * 3.6, // m/s to km/h
                        wind_direction: item.wind_bearing.unwrap_or(0.0),
                        cloud_cover: item.cloud_cover.unwrap_or(0.0) * 100.0, // 0..1 to %
                        weather_code: wmo,
                        aqi: None,
                        uv_index: item.uv,
                        is_day: None,
                        visibility: item.visibility,
                        soil_temperature: None,
                        soil_moisture: None,
                    });
                }
            }
        }

        Ok(NormalizedWeatherData {
            provider_name: self.name().to_string(),
            hourly: points,
        })
    }
}
