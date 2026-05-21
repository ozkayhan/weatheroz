use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct NwsProvider;

#[derive(Deserialize, Debug)]
struct NwsPointProperties {
    #[serde(rename = "forecastHourly")]
    forecast_hourly: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NwsPointResponse {
    properties: Option<NwsPointProperties>,
}

#[derive(Deserialize, Debug)]
struct NwsPeriodValue {
    value: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct NwsForecastPeriod {
    #[serde(rename = "startTime")]
    start_time: String,
    temperature: Option<f64>,
    #[serde(rename = "probabilityOfPrecipitation")]
    prob_precip: Option<NwsPeriodValue>,
    #[serde(rename = "windSpeed")]
    wind_speed: Option<String>,
    #[serde(rename = "windDirection")]
    wind_direction: Option<String>,
    #[serde(rename = "shortForecast")]
    #[allow(dead_code)]
    short_forecast: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NwsForecastProperties {
    periods: Option<Vec<NwsForecastPeriod>>,
}

#[derive(Deserialize, Debug)]
struct NwsForecastResponse {
    properties: Option<NwsForecastProperties>,
}

fn parse_wind_speed(wind_str: &str) -> f64 {
    // NWS wind speed can be "10 mph" or "5 to 10 mph"
    let clean: String = wind_str.chars().filter(|c| c.is_digit(10) || *c == ' ').collect();
    let parts: Vec<&str> = clean.split_whitespace().collect();
    if let Some(last) = parts.last() {
        if let Ok(mph) = last.parse::<f64>() {
            return mph * 1.60934; // mph to km/h
        }
    }
    0.0
}

fn parse_wind_direction(dir_str: &str) -> f64 {
    match dir_str {
        "N" => 0.0, "NNE" => 22.5, "NE" => 45.0, "ENE" => 67.5,
        "E" => 90.0, "ESE" => 112.5, "SE" => 135.0, "SSE" => 157.5,
        "S" => 180.0, "SSW" => 202.5, "SW" => 225.0, "WSW" => 247.5,
        "W" => 270.0, "WNW" => 292.5, "NW" => 315.0, "NNW" => 337.5,
        _ => 0.0,
    }
}

impl BaseWeatherProvider for NwsProvider {
    fn name(&self) -> &str {
        "NWS"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        // NWS only works for US coordinates. If it fails or is outside the US, we return an empty result or error.
        let point_url = format!("https://api.weather.gov/points/{:.4},{:.4}", ctx.lat, ctx.lon);
        
        let client = &ctx.client;
        let points_resp = client.get(&point_url)
            .header("User-Agent", "weather-cli/0.1.0 contact@example.com")
            .send()
            .await?;

        if !points_resp.status().is_success() {
            // If outside US, NWS returns 404/500, we'll return empty data so the race continues smoothly
            return Ok(NormalizedWeatherData {
                provider_name: self.name().to_string(),
                hourly: vec![],
            });
        }

        let points_data: NwsPointResponse = points_resp.json().await?;
        let forecast_url = points_data.properties
            .and_then(|p| p.forecast_hourly)
            .ok_or("Failed to parse forecast URL from NWS points response")?;

        let forecast_resp = client.get(&forecast_url)
            .header("User-Agent", "weather-cli/0.1.0 contact@example.com")
            .send()
            .await?;

        if !forecast_resp.status().is_success() {
            return Err(format!("NWS Forecast HTTP error {}", forecast_resp.status()).into());
        }

        let forecast_data: NwsForecastResponse = forecast_resp.json().await?;
        let mut points = Vec::new();

        if let Some(props) = forecast_data.properties {
            if let Some(periods) = props.periods {
                for period in periods {
                    let time_formatted = period.start_time.replace('Z', "").split('+').next().unwrap_or("").to_string();
                    let date_part = time_formatted.split('T').next().unwrap_or("");
                    if date_part < ctx.start_date || date_part > ctx.end_date {
                        continue;
                    }

                    // NWS returns Fahrenheit by default, convert to Celsius
                    let fahrenheit = period.temperature.unwrap_or(32.0);
                    let celsius = (fahrenheit - 32.0) * 5.0 / 9.0;

                    let precip_prob = period.prob_precip.and_then(|v| v.value).unwrap_or(0.0);
                    let wind_sp = parse_wind_speed(period.wind_speed.as_deref().unwrap_or("0"));
                    let wind_dir = parse_wind_direction(period.wind_direction.as_deref().unwrap_or(""));

                    points.push(HourlyPoint {
                        time: time_formatted,
                        temperature: celsius,
                        apparent_temperature: celsius,
                        precipitation_probability: precip_prob,
                        precipitation: 0.0,
                        humidity: 50.0, // NWS hourly does not always include relative humidity directly in compact periods
                        wind_speed: wind_sp,
                        wind_direction: wind_dir,
                        cloud_cover: 0.0,
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
        }

        Ok(NormalizedWeatherData {
            provider_name: self.name().to_string(),
            hourly: points,
        })
    }
}
