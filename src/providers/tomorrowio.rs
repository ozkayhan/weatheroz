use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct TomorrowIoProvider;

#[derive(Deserialize, Debug)]
struct TomorrowValues {
    temperature: Option<f64>,
    #[serde(rename = "temperatureApparent")]
    temp_app: Option<f64>,
    #[serde(rename = "precipitationProbability")]
    precip_prob: Option<f64>,
    #[serde(rename = "precipitationIntensity")]
    precip_intensity: Option<f64>,
    humidity: Option<f64>,
    #[serde(rename = "windSpeed")]
    wind_speed: Option<f64>,
    #[serde(rename = "windDirection")]
    wind_dir: Option<f64>,
    #[serde(rename = "cloudCover")]
    cloud_cover: Option<f64>,
    #[serde(rename = "weatherCode")]
    weather_code: Option<i32>,
    #[serde(rename = "uvIndex")]
    uv: Option<f64>,
    visibility: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct TomorrowInterval {
    time: String,
    values: Option<TomorrowValues>,
}

#[derive(Deserialize, Debug)]
struct TomorrowTimeline {
    hourly: Option<Vec<TomorrowInterval>>,
}

#[derive(Deserialize, Debug)]
struct TomorrowResponse {
    timelines: Option<TomorrowTimeline>,
}

fn tomorrow_code_to_wmo(code: i32) -> i32 {
    match code {
        1000 => 0,        // Clear
        1100..=1102 => 2, // Partly cloudy
        1001 => 3,        // Cloudy
        2000..=2100 => 45, // Fog
        4000..=4201 => 63, // Rain
        5000..=5101 => 73, // Snow
        8000 => 95,       // Thunder
        _ => 0,
    }
}

impl BaseWeatherProvider for TomorrowIoProvider {
    fn name(&self) -> &str {
        "Tomorrow.io"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("Tomorrow.io") {
            Some(k) => k,
            None => return Err("Tomorrow.io API key is missing. Configure it in config.json or set WEATHER_KEY_TOMORROW_IO.".into()),
        };

        let url = format!(
            "https://api.tomorrow.io/v4/weather/forecast?location={:.4},{:.4}&apikey={}&timesteps=1h",
            ctx.lat, ctx.lon, key
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Tomorrow.io HTTP error {}", resp.status()).into());
        }

        let raw: TomorrowResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(timelines) = raw.timelines {
            if let Some(hourly) = timelines.hourly {
                for item in hourly {
                    let time_formatted = item.time.replace('Z', "").split('+').next().unwrap_or("").to_string();
                    let date_part = time_formatted.split('T').next().unwrap_or("");
                    if date_part < ctx.start_date || date_part > ctx.end_date {
                        continue;
                    }

                    if let Some(vals) = item.values {
                        let temp = vals.temperature.unwrap_or(0.0);
                        let app = vals.temp_app.unwrap_or(temp);
                        let code = vals.weather_code.unwrap_or(1000);
                        let wmo = tomorrow_code_to_wmo(code);

                        points.push(HourlyPoint {
                            time: time_formatted,
                            temperature: temp,
                            apparent_temperature: app,
                            precipitation_probability: vals.precip_prob.unwrap_or(0.0),
                            precipitation: vals.precip_intensity.unwrap_or(0.0),
                            humidity: vals.humidity.unwrap_or(0.0),
                            wind_speed: vals.wind_speed.unwrap_or(0.0) * 3.6, // m/s to km/h
                            wind_direction: vals.wind_dir.unwrap_or(0.0),
                            cloud_cover: vals.cloud_cover.unwrap_or(0.0),
                            weather_code: wmo,
                            aqi: None,
                            uv_index: vals.uv,
                            is_day: None,
                            visibility: vals.visibility,
                            soil_temperature: None,
                            soil_moisture: None,
                        });
                    }
                }
            }
        }

        Ok(NormalizedWeatherData {
            provider_name: self.name().to_string(),
            hourly: points,
        })
    }
}
