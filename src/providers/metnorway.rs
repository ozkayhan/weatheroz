use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};
use serde::Deserialize;

#[derive(Clone)]
pub struct MetNorwayProvider;

#[derive(Deserialize, Debug)]
struct MetInstantDetails {
    air_temperature: Option<f64>,
    relative_humidity: Option<f64>,
    wind_speed: Option<f64>,
    wind_from_direction: Option<f64>,
    cloud_area_fraction: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct MetInstant {
    details: Option<MetInstantDetails>,
}

#[derive(Deserialize, Debug)]
struct MetNext1Details {
    precipitation_amount: Option<f64>,
    probability_of_precipitation: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct MetNext1Summary {
    symbol_code: Option<String>,
}

#[derive(Deserialize, Debug)]
struct MetNext1 {
    details: Option<MetNext1Details>,
    summary: Option<MetNext1Summary>,
}

#[derive(Deserialize, Debug)]
struct MetData {
    instant: Option<MetInstant>,
    next_1_hours: Option<MetNext1>,
}

#[derive(Deserialize, Debug)]
struct MetTimeSeriesEntry {
    time: String,
    data: Option<MetData>,
}

#[derive(Deserialize, Debug)]
struct MetProperties {
    timeseries: Vec<MetTimeSeriesEntry>,
}

#[derive(Deserialize, Debug)]
struct MetResponse {
    properties: Option<MetProperties>,
}

fn symbol_code_to_wmo(symbol: &str) -> i32 {
    if symbol.is_empty() {
        return 0;
    }
    let base = symbol.split('_').next().unwrap_or("").to_lowercase();
    match base.as_str() {
        "clearsky" => 0,
        "fair" => 1,
        "partlycloudy" => 2,
        "cloudy" => 3,
        "fog" => 45,
        "lightrainshowers" => 80,
        "rainshowers" => 81,
        "heavyrainshowers" => 82,
        "lightrainshowersandthunder" | "rainshowersandthunder" | "heavyrainshowersandthunder" => 95,
        "lightsleetshowers" => 85,
        "sleetshowers" | "heavysleetshowers" => 86,
        "lightsleetshowersandthunder"
        | "sleetshowersandthunder"
        | "heavysleetshowersandthunder" => 95,
        "lightsnowshowers" => 85,
        "snowshowers" | "heavysnowshowers" => 86,
        "lightsnowshowersandthunder" | "snowshowersandthunder" | "heavysnowshowersandthunder" => 95,
        "lightrain" => 61,
        "rain" => 63,
        "heavyrain" => 65,
        "lightrainandthunder" | "rainandthunder" | "heavyrainandthunder" => 95,
        "lightsleet" => 71,
        "sleet" => 73,
        "heavysleet" => 75,
        "lightsleetandthunder" | "sleetandthunder" | "heavysleetandthunder" => 95,
        "lightsnow" => 71,
        "snow" => 73,
        "heavysnow" => 75,
        "lightsnowandthunder" | "snowandthunder" | "heavysnowandthunder" => 95,
        _ => 0,
    }
}

impl MetNorwayProvider {
    fn normalize(
        &self,
        raw_data: MetResponse,
        start_date: &str,
        end_date: &str,
    ) -> NormalizedWeatherData {
        let mut points = Vec::new();

        if let Some(props) = raw_data.properties {
            for entry in props.timeseries {
                let time_str = entry.time;
                if time_str.len() < 10 {
                    continue;
                }
                let date_part = &time_str[0..10];
                if date_part < start_date || date_part > end_date {
                    continue;
                }

                let data = match entry.data {
                    Some(d) => d,
                    None => continue,
                };

                let instant_details =
                    data.instant
                        .and_then(|i| i.details)
                        .unwrap_or(MetInstantDetails {
                            air_temperature: None,
                            relative_humidity: None,
                            wind_speed: None,
                            wind_from_direction: None,
                            cloud_area_fraction: None,
                        });

                let next_1 = data.next_1_hours.unwrap_or(MetNext1 {
                    details: None,
                    summary: None,
                });

                let next_1_details = next_1.details.unwrap_or(MetNext1Details {
                    precipitation_amount: None,
                    probability_of_precipitation: None,
                });

                let symbol = next_1
                    .summary
                    .and_then(|s| s.symbol_code)
                    .unwrap_or_default();
                let wmo_code = symbol_code_to_wmo(&symbol);

                let temp = instant_details.air_temperature.unwrap_or(0.0);
                let humidity = instant_details.relative_humidity.unwrap_or(0.0);
                let wind_speed_ms = instant_details.wind_speed.unwrap_or(0.0);
                let wind_speed_kmh = wind_speed_ms * 3.6;
                let wind_dir = instant_details.wind_from_direction.unwrap_or(0.0);
                let cloud_cover = instant_details.cloud_area_fraction.unwrap_or(0.0);
                let precip = next_1_details.precipitation_amount.unwrap_or(0.0);
                let precip_prob = next_1_details.probability_of_precipitation.unwrap_or(0.0);

                points.push(HourlyPoint {
                    time: time_str.replace('Z', ""),
                    temperature: temp,
                    apparent_temperature: temp,
                    precipitation_probability: precip_prob,
                    precipitation: precip,
                    humidity,
                    wind_speed: (wind_speed_kmh * 100.0).round() / 100.0,
                    wind_direction: wind_dir,
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

        NormalizedWeatherData {
            provider_name: "MET Norway".to_string(),
            hourly: points,
        }
    }
}

impl BaseWeatherProvider for MetNorwayProvider {
    fn name(&self) -> &str {
        "MET Norway"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={}&lon={}",
            ctx.lat, ctx.lon
        );

        let resp = ctx
            .client
            .get(&url)
            .header("User-Agent", "weather-cli/0.1.0 contact@example.com")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("Failed to reach MET Norway: HTTP {}", resp.status()).into());
        }

        let raw_data: MetResponse = resp.json().await?;
        Ok(self.normalize(raw_data, ctx.start_date, ctx.end_date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metnorway_wind_speed_conversion() {
        let entry = MetTimeSeriesEntry {
            time: "2026-05-19T00:00:00Z".to_string(),
            data: Some(MetData {
                instant: Some(MetInstant {
                    details: Some(MetInstantDetails {
                        air_temperature: Some(15.2),
                        relative_humidity: Some(70.0),
                        wind_speed: Some(3.33), // 3.33 m/s * 3.6 = 11.988 km/h -> 11.99
                        wind_from_direction: Some(45.0),
                        cloud_area_fraction: Some(20.0),
                    }),
                }),
                next_1_hours: Some(MetNext1 {
                    details: Some(MetNext1Details {
                        precipitation_amount: Some(0.0),
                        probability_of_precipitation: Some(10.0),
                    }),
                    summary: Some(MetNext1Summary {
                        symbol_code: Some("clearsky_day".to_string()),
                    }),
                }),
            }),
        };

        let raw = MetResponse {
            properties: Some(MetProperties {
                timeseries: vec![entry],
            }),
        };

        let provider = MetNorwayProvider;
        let normalized = provider.normalize(raw, "2026-05-19", "2026-05-19");
        assert_eq!(normalized.hourly.len(), 1);
        let pt = &normalized.hourly[0];
        assert_eq!(pt.temperature, 15.2);
        assert_eq!(pt.wind_speed, 11.99);
    }
}
