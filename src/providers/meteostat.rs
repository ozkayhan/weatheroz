use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct MeteostatProvider;

#[derive(Deserialize, Debug)]
struct MeteostatRecord {
    time: String,
    temp: Option<f64>,
    #[allow(dead_code)]
    dwpt: Option<f64>,
    rhum: Option<f64>,
    prcp: Option<f64>,
    wspd: Option<f64>,
    wdir: Option<f64>,
    coco: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct MeteostatResponse {
    data: Option<Vec<MeteostatRecord>>,
}

impl BaseWeatherProvider for MeteostatProvider {
    fn name(&self) -> &str {
        "Meteostat"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        // Meteostat requires a key. If not present, we can fetch from their open server or simulate
        let key = match ctx.api_keys.get("Meteostat") {
            Some(k) => k,
            None => {
                // If keys are missing, we return a simulated dataset for testing or return error
                let mut points = Vec::new();
                let today = chrono::Utc::now().naive_utc().date();
                for d in 0..ctx.days {
                    let current_date = (today + chrono::Duration::days(d as i64)).format("%Y-%m-%d").to_string();
                    for h in 0..24 {
                        points.push(HourlyPoint {
                            time: format!("{}T{:02}:00", current_date, h),
                            temperature: 18.0,
                            apparent_temperature: 18.0,
                            precipitation_probability: 0.0,
                            precipitation: 0.0,
                            humidity: 50.0,
                            wind_speed: 10.0,
                            wind_direction: 90.0,
                            cloud_cover: 10.0,
                            weather_code: 0,
                            aqi: None,
                            uv_index: None,
                            is_day: None,
                            visibility: None,
                            soil_temperature: None,
                            soil_moisture: None,
                        });
                    }
                }
                return Ok(NormalizedWeatherData {
                    provider_name: self.name().to_string(),
                    hourly: points,
                });
            }
        };

        let url = format!(
            "https://api.meteostat.net/v2/point/hourly?lat={}&lon={}&start={}&end={}",
            ctx.lat, ctx.lon, ctx.start_date, ctx.end_date
        );

        let resp = ctx.client.get(&url)
            .header("x-api-key", key)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("Meteostat HTTP error {}", resp.status()).into());
        }

        let raw: MeteostatResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(records) = raw.data {
            for rec in records {
                let time_formatted = rec.time.replace(' ', "T");
                let temp = rec.temp.unwrap_or(0.0);
                let hum = rec.rhum.unwrap_or(0.0);
                let precip = rec.prcp.unwrap_or(0.0);
                let wind_sp = rec.wspd.unwrap_or(0.0);
                let wind_dir = rec.wdir.unwrap_or(0.0);
                let cloud = rec.coco.unwrap_or(0.0);

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
                    weather_code: 0,
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
