use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct WeatherApiProvider;

#[derive(Deserialize, Debug)]
struct WaHour {
    time: String,
    temp_c: Option<f64>,
    feelslike_c: Option<f64>,
    chance_of_rain: Option<f64>,
    precip_mm: Option<f64>,
    humidity: Option<f64>,
    wind_kph: Option<f64>,
    wind_degree: Option<f64>,
    cloud: Option<f64>,
    condition: Option<WaCondition>,
    uv: Option<f64>,
    vis_km: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct WaCondition {
    code: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct WaForecastDay {
    hour: Option<Vec<WaHour>>,
}

#[derive(Deserialize, Debug)]
struct WaForecast {
    forecastday: Option<Vec<WaForecastDay>>,
}

#[derive(Deserialize, Debug)]
struct WaResponse {
    forecast: Option<WaForecast>,
}

impl BaseWeatherProvider for WeatherApiProvider {
    fn name(&self) -> &str {
        "WeatherAPI"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("WeatherAPI") {
            Some(k) => k,
            None => return Err("WeatherAPI key is missing. Configure it in config.json or set WEATHER_KEY_WEATHERAPI.".into()),
        };

        let days = std::cmp::min(ctx.days, 14); // WeatherAPI free key supports up to 14 days
        let url = format!(
            "https://api.weatherapi.com/v1/forecast.json?key={}&q={},{}&days={}",
            key, ctx.lat, ctx.lon, days
        );

        let resp = ctx.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("WeatherAPI HTTP error {}", resp.status()).into());
        }

        let raw: WaResponse = resp.json().await?;
        let mut points = Vec::new();

        if let Some(forecast) = raw.forecast {
            if let Some(forecastdays) = forecast.forecastday {
                for fday in forecastdays {
                    if let Some(hours) = fday.hour {
                        for hr in hours {
                            let time_formatted = hr.time.replace(' ', "T");
                            let date_part = time_formatted.split('T').next().unwrap_or("");
                            if date_part < ctx.start_date || date_part > ctx.end_date {
                                continue;
                            }

                            let temp = hr.temp_c.unwrap_or(0.0);
                            let feels = hr.feelslike_c.unwrap_or(temp);
                            let code = hr.condition.and_then(|c| c.code).unwrap_or(1000);

                            // Map WeatherAPI condition code to WMO
                            let wmo = match code {
                                1000 => 0,        // Sunny/Clear
                                1003 => 2,        // Partly cloudy
                                1006 | 1009 => 3, // Cloudy/Overcast
                                1030 | 1135 => 45, // Mist/Fog
                                1063 | 1180..=1201 => 61, // Patchy/light rain
                                1066 | 1210..=1225 => 73, // Snow
                                1087 | 1273..=1282 => 95, // Thunder
                                _ => 0,
                            };

                            points.push(HourlyPoint {
                                time: time_formatted,
                                temperature: temp,
                                apparent_temperature: feels,
                                precipitation_probability: hr.chance_of_rain.unwrap_or(0.0),
                                precipitation: hr.precip_mm.unwrap_or(0.0),
                                humidity: hr.humidity.unwrap_or(0.0),
                                wind_speed: hr.wind_kph.unwrap_or(0.0),
                                wind_direction: hr.wind_degree.unwrap_or(0.0),
                                cloud_cover: hr.cloud.unwrap_or(0.0),
                                weather_code: wmo,
                                aqi: None,
                                uv_index: hr.uv,
                                is_day: None,
                                visibility: hr.vis_km,
                                soil_temperature: None,
                                soil_moisture: None,
                            });
                        }
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
