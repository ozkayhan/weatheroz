use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct VisualCrossingProvider;

#[derive(Deserialize, Debug)]
struct VcHour {
    datetime: String,
    temp: Option<f64>,
    feelslike: Option<f64>,
    precipprob: Option<f64>,
    precip: Option<f64>,
    humidity: Option<f64>,
    windspeed: Option<f64>,
    winddir: Option<f64>,
    cloudcover: Option<f64>,
    icon: Option<String>,
    uvindex: Option<f64>,
    visibility: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct VcDay {
    datetime: String,
    hours: Option<Vec<VcHour>>,
}

#[derive(Deserialize, Debug)]
struct VcResponse {
    days: Option<Vec<VcDay>>,
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

impl BaseWeatherProvider for VisualCrossingProvider {
    fn name(&self) -> &str {
        "Visual Crossing"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("Visual Crossing") {
            Some(k) => k,
            None => return Err("Visual Crossing API key is missing. Configure it in config.json or set WEATHER_KEY_VISUAL_CROSSING.".into()),
        };

        // Visual Crossing Timeline API endpoint: timeline/lat,lon/start_date/end_date?key=...&unitGroup=metric
        let url = format!(
            "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline/{:.4},{:.4}/{}/{}?key={}&unitGroup=metric&include=hours",
            ctx.lat, ctx.lon, ctx.start_date, ctx.end_date, key
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Visual Crossing HTTP error {}", resp.status()).into());
        }

        let raw: VcResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(days) = raw.days {
            for day in days {
                let date_str = day.datetime; // "2026-05-21"
                if let Some(hours) = day.hours {
                    for hr in hours {
                        let time_formatted = format!("{}T{}", date_str, hr.datetime); // "2026-05-21T08:00:00"
                        let date_part = time_formatted.split('T').next().unwrap_or("");
                        if date_part < ctx.start_date || date_part > ctx.end_date {
                            continue;
                        }

                        let temp = hr.temp.unwrap_or(0.0);
                        let feels = hr.feelslike.unwrap_or(temp);
                        let wmo = icon_to_wmo(hr.icon.as_deref().unwrap_or(""));

                        points.push(HourlyPoint {
                            time: time_formatted,
                            temperature: temp,
                            apparent_temperature: feels,
                            precipitation_probability: hr.precipprob.unwrap_or(0.0),
                            precipitation: hr.precip.unwrap_or(0.0),
                            humidity: hr.humidity.unwrap_or(0.0),
                            wind_speed: hr.windspeed.unwrap_or(0.0),
                            wind_direction: hr.winddir.unwrap_or(0.0),
                            cloud_cover: hr.cloudcover.unwrap_or(0.0),
                            weather_code: wmo,
                            aqi: None,
                            uv_index: hr.uvindex,
                            is_day: None,
                            visibility: hr.visibility,
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
