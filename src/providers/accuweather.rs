use serde::Deserialize;
use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

#[derive(Clone)]
pub struct AccuWeatherProvider;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct AccuLocationResponse {
    Key: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct AccuTempValue {
    Value: Option<f64>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct AccuHourlyItem {
    DateTime: String,
    Temperature: Option<AccuTempValue>,
    RealFeelTemperature: Option<AccuTempValue>,
    RelativeHumidity: Option<f64>,
    Wind: Option<AccuWind>,
    CloudCover: Option<f64>,
    WeatherIcon: Option<i32>,
    UVIndex: Option<f64>,
    Visibility: Option<AccuTempValue>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct AccuWind {
    Speed: Option<AccuTempValue>,
    Direction: Option<AccuWindDirection>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct AccuWindDirection {
    Degrees: Option<f64>,
}

impl BaseWeatherProvider for AccuWeatherProvider {
    fn name(&self) -> &str {
        "AccuWeather"
    }

    async fn fetch(
        &self,
        ctx: &FetchContext<'_>,
    ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
        let key = match ctx.api_keys.get("AccuWeather") {
            Some(k) => k,
            None => return Err("AccuWeather API key is missing. Configure it in config.json or set WEATHER_KEY_ACCUWEATHER.".into()),
        };

        // AccuWeather requires resolving lat/lon to a Location Key first
        let geocode_url = format!(
            "http://dataservice.accuweather.com/locations/v1/cities/geoposition/search?apikey={}&q={},{:}",
            key, ctx.lat, ctx.lon
        );

        let loc_resp = ctx.client.get(&geocode_url).send().await?;
        if !loc_resp.status().is_success() {
            return Err(format!("AccuWeather Geocode HTTP error {}", loc_resp.status()).into());
        }

        let loc_data: AccuLocationResponse = loc_resp.json().await?;
        let loc_key = loc_data.Key.ok_or("Failed to retrieve AccuWeather Location Key")?;

        // Fetch 12 Hours forecast
        let forecast_url = format!(
            "http://dataservice.accuweather.com/forecasts/v1/hourly/12hour/{}?apikey={}&details=true&metric=true",
            loc_key, key
        );

        let forecast_resp = ctx.client.get(&forecast_url).send().await?;
        if !forecast_resp.status().is_success() {
            return Err(format!("AccuWeather Forecast HTTP error {}", forecast_resp.status()).into());
        }

        let raw: Vec<AccuHourlyItem> = forecast_resp.json().await?;
        let mut points = Vec::new();

        for item in raw {
            let time_formatted = item.DateTime.replace('Z', "").split('+').next().unwrap_or("").to_string();
            let date_part = time_formatted.split('T').next().unwrap_or("");
            if date_part < ctx.start_date || date_part > ctx.end_date {
                continue;
            }

            let temp = item.Temperature.and_then(|t| t.Value).unwrap_or(0.0);
            let feels = item.RealFeelTemperature.and_then(|t| t.Value).unwrap_or(temp);
            let icon = item.WeatherIcon.unwrap_or(1);

            // Map AccuWeather Icon IDs to WMO
            let wmo = match icon {
                1..=5 => 0,     // Clear/Sunny
                6..=10 => 2,    // Clouds
                11 => 45,       // Fog
                12..=18 => 63,  // Rain
                19..=29 => 73,  // Snow
                _ => 0,
            };

            let wind_spd = item.Wind.as_ref().and_then(|w| w.Speed.as_ref().and_then(|s| s.Value)).unwrap_or(0.0);
            let wind_dir = item.Wind.as_ref().and_then(|w| w.Direction.as_ref().and_then(|d| d.Degrees)).unwrap_or(0.0);
            let vis = item.Visibility.and_then(|v| v.Value);

            points.push(HourlyPoint {
                time: time_formatted,
                temperature: temp,
                apparent_temperature: feels,
                precipitation_probability: 0.0,
                precipitation: 0.0,
                humidity: item.RelativeHumidity.unwrap_or(50.0),
                wind_speed: wind_spd,
                wind_direction: wind_dir,
                cloud_cover: item.CloudCover.unwrap_or(0.0),
                weather_code: wmo,
                aqi: None,
                uv_index: item.UVIndex,
                is_day: None,
                visibility: vis,
                soil_temperature: None,
                soil_moisture: None,
            });
        }

        Ok(NormalizedWeatherData {
            provider_name: self.name().to_string(),
            hourly: points,
        })
    }
}
