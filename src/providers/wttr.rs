use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct WttrProvider;

#[derive(Deserialize, Debug)]
struct WttrHourlyBlock {
    #[serde(rename = "tempC")]
    temp_c: Option<String>,
    #[serde(rename = "FeelsLikeC")]
    feels_like_c: Option<String>,
    chanceofrain: Option<String>,
    #[serde(rename = "precipMM")]
    precip_mm: Option<String>,
    humidity: Option<String>,
    #[serde(rename = "windspeedKmph")]
    wind_speed_kmh: Option<String>,
    #[serde(rename = "winddirDegree")]
    wind_dir_degree: Option<String>,
    cloudcover: Option<String>,
    #[serde(rename = "weatherCode")]
    weather_code: Option<String>,
}

#[derive(Deserialize, Debug)]
struct WttrDay {
    date: Option<String>,
    hourly: Option<Vec<WttrHourlyBlock>>,
}

#[derive(Deserialize, Debug)]
struct WttrResponse {
    weather: Option<Vec<WttrDay>>,
}

fn wwo_code_to_wmo(wwo_code: i32) -> i32 {
    match wwo_code {
        113 => 0,
        116 => 2,
        119 => 3,
        122 => 3,
        143 => 45,
        176 => 61,
        179 => 71,
        182 => 71,
        185 => 56,
        200 => 95,
        227 => 77,
        230 => 75,
        248 => 45,
        260 => 48,
        263 | 266 => 51,
        281 => 56,
        284 => 57,
        293 | 296 => 61,
        299 | 302 => 63,
        305 | 308 => 65,
        311 => 66,
        314 => 67,
        317 => 71,
        320 => 75,
        323 | 326 => 71,
        329 | 332 => 73,
        335 | 338 => 75,
        350 => 77,
        353 => 80,
        356 => 81,
        359 => 82,
        362 => 85,
        365 => 86,
        368 => 85,
        371 => 86,
        374 | 377 => 77,
        386 | 389 => 95,
        392 => 96,
        395 => 99,
        _ => 0,
    }
}

impl WttrProvider {
    fn normalize(&self, raw_data: WttrResponse, start_date: &str, end_date: &str) -> NormalizedWeatherData {
        let mut points = Vec::new();

        if let Some(days) = raw_data.weather {
            for day in days {
                let day_date = match day.date {
                    Some(d) => d,
                    None => continue,
                };
                if day_date.as_str() < start_date || day_date.as_str() > end_date {
                    continue;
                }

                let hourly_blocks = match day.hourly {
                    Some(h) => h,
                    None => continue,
                };
                if hourly_blocks.is_empty() {
                    continue;
                }

                for h in 0..24 {
                    let block_idx = std::cmp::min(h / 3, hourly_blocks.len() - 1);
                    let block = &hourly_blocks[block_idx];

                    let temp_str = block.temp_c.as_deref().unwrap_or("0");
                    let temp = temp_str.parse::<f64>().unwrap_or(0.0);

                    let feels_str = block.feels_like_c.as_deref().unwrap_or(temp_str);
                    let feels_like = feels_str.parse::<f64>().unwrap_or(temp);

                    let prob_str = block.chanceofrain.as_deref().unwrap_or("0");
                    let precip_prob = prob_str.parse::<f64>().unwrap_or(0.0);

                    let precip_str = block.precip_mm.as_deref().unwrap_or("0.0");
                    let precip = precip_str.parse::<f64>().unwrap_or(0.0);

                    let hum_str = block.humidity.as_deref().unwrap_or("0");
                    let humidity = hum_str.parse::<f64>().unwrap_or(0.0);

                    let wind_sp_str = block.wind_speed_kmh.as_deref().unwrap_or("0");
                    let wind_speed = wind_sp_str.parse::<f64>().unwrap_or(0.0);

                    let wind_dir_str = block.wind_dir_degree.as_deref().unwrap_or("0");
                    let wind_direction = wind_dir_str.parse::<f64>().unwrap_or(0.0);

                    let cloud_str = block.cloudcover.as_deref().unwrap_or("0");
                    let cloud_cover = cloud_str.parse::<f64>().unwrap_or(0.0);

                    let code_str = block.weather_code.as_deref().unwrap_or("113");
                    let wwo_code = code_str.parse::<i32>().unwrap_or(113);
                    let wmo_code = wwo_code_to_wmo(wwo_code);

                    points.push(HourlyPoint {
                        time: format!("{}T{:02}:00", day_date, h),
                        temperature: temp,
                        apparent_temperature: feels_like,
                        precipitation_probability: precip_prob,
                        precipitation: precip,
                        humidity,
                        wind_speed,
                        wind_direction,
                        cloud_cover,
                        weather_code: wmo_code,
                        aqi: None,
                        uv_index: None,
                        is_day: None,
                        visibility: None,
                        soil_temperature: None,
                        soil_moisture: None,
                    });
                }
            }
        }

        NormalizedWeatherData {
            provider_name: "wttr.in".to_string(),
            hourly: points,
        }
    }
}

impl BaseWeatherProvider for WttrProvider {
    fn name(&self) -> &str {
        "wttr.in"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://wttr.in/{},{}?format=j1", ctx.lat, ctx.lon);
        let resp = ctx.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(format!("Failed to reach wttr.in: HTTP {}", resp.status()).into());
        }

        let raw_data: WttrResponse = resp.json().await?;
        Ok(self.normalize(raw_data, ctx.start_date, ctx.end_date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wttr_interpolation_3h_to_1h() {
        let raw = WttrResponse {
            weather: Some(vec![WttrDay {
                date: Some("2026-05-19".to_string()),
                hourly: Some(vec![
                    WttrHourlyBlock {
                        temp_c: Some("15".to_string()),
                        feels_like_c: Some("14".to_string()),
                        chanceofrain: Some("10".to_string()),
                        precip_mm: Some("0.0".to_string()),
                        humidity: Some("70".to_string()),
                        wind_speed_kmh: Some("12".to_string()),
                        wind_dir_degree: Some("45".to_string()),
                        cloudcover: Some("20".to_string()),
                        weather_code: Some("113".to_string()),
                    },
                    WttrHourlyBlock {
                        temp_c: Some("14".to_string()),
                        feels_like_c: Some("13".to_string()),
                        chanceofrain: Some("20".to_string()),
                        precip_mm: Some("0.2".to_string()),
                        humidity: Some("75".to_string()),
                        wind_speed_kmh: Some("11".to_string()),
                        wind_dir_degree: Some("50".to_string()),
                        cloudcover: Some("40".to_string()),
                        weather_code: Some("116".to_string()),
                    },
                ]),
            }]),
        };

        let provider = WttrProvider;
        let normalized = provider.normalize(raw, "2026-05-19", "2026-05-19");
        assert_eq!(normalized.hourly.len(), 24);

        let pt0 = &normalized.hourly[0];
        assert_eq!(pt0.time, "2026-05-19T00:00");
        assert_eq!(pt0.temperature, 15.0);
    }
}
