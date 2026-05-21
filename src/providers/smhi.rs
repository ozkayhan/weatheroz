use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};
use serde::Deserialize;

#[derive(Clone)]
pub struct SmhiProvider;

#[derive(Deserialize, Debug)]
struct SmhiParameter {
    name: String,
    values: Vec<f64>,
}

#[derive(Deserialize, Debug)]
struct SmhiTimeSeries {
    #[serde(rename = "validTime")]
    valid_time: String,
    parameters: Vec<SmhiParameter>,
}

#[derive(Deserialize, Debug)]
struct SmhiResponse {
    #[serde(rename = "timeSeries")]
    time_series: Option<Vec<SmhiTimeSeries>>,
}

impl BaseWeatherProvider for SmhiProvider {
    fn name(&self) -> &str {
        "SMHI"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        // SMHI requires longitude and latitude rounded to 6 decimal places
        let lat_rounded = (ctx.lat * 1000000.0).round() / 1000000.0;
        let lon_rounded = (ctx.lon * 1000000.0).round() / 1000000.0;

        let url = format!(
            "https://opendata-download-metfcst.smhi.se/api/category/pmp3g/version/2/geotype/point/lon/{}/lat/{}/data.json",
            lon_rounded, lat_rounded
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("SMHI HTTP error {}", resp.status()).into());
        }

        let raw: SmhiResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(ts_list) = raw.time_series {
            for ts in ts_list {
                let time_formatted = ts.valid_time.replace('Z', "").replace('T', " ");
                let date_part = time_formatted.split(' ').next().unwrap_or("");
                if date_part < ctx.start_date || date_part > ctx.end_date {
                    continue;
                }

                let mut temp = 0.0;
                let mut precip = 0.0;
                let mut hum = 0.0;
                let mut wind_sp = 0.0;
                let mut wind_dir = 0.0;
                let mut cloud = 0.0;
                let mut wmo = 0;

                for param in ts.parameters {
                    if param.values.is_empty() {
                        continue;
                    }
                    match param.name.as_str() {
                        "t" => temp = param.values[0],                // Temperature Celsius
                        "r" => hum = param.values[0],                 // Relative Humidity %
                        "ws" => wind_sp = param.values[0] * 3.6,      // wind speed m/s to km/h
                        "wd" => wind_dir = param.values[0],           // wind direction degree
                        "tcc_mean" => cloud = param.values[0] * 12.5, // octas to % (approx)
                        "pmean" => precip = param.values[0],          // precipitation mean mm/h
                        "Wsymb2" => wmo = param.values[0] as i32,     // Weather symbol (1-27)
                        _ => {}
                    }
                }

                // Map SMHI symbols (1-27) to WMO codes roughly
                let wmo_mapped = match wmo {
                    1..=2 => 0,    // Clear
                    3..=4 => 2,    // Partly cloudy
                    5..=6 => 3,    // Cloudy
                    7 => 45,       // Fog
                    8..=10 => 51,  // Light rain
                    18..=20 => 61, // Rain
                    21..=24 => 71, // Snow
                    _ => 0,
                };

                points.push(HourlyPoint {
                    time: time_formatted.replace(' ', "T"),
                    temperature: temp,
                    apparent_temperature: temp,
                    precipitation_probability: if precip > 0.0 { 100.0 } else { 0.0 },
                    precipitation: precip,
                    humidity: hum,
                    wind_speed: wind_sp,
                    wind_direction: wind_dir,
                    cloud_cover: cloud,
                    weather_code: wmo_mapped,
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
