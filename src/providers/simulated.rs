use crate::providers::base::{BaseWeatherProvider, FetchContext};
use crate::providers::models::{HourlyPoint, NormalizedWeatherData};

macro_rules! define_simulated_provider {
    ($name:ident, $display_name:expr, $key_name:expr) => {
        #[derive(Clone)]
        pub struct $name;

        impl BaseWeatherProvider for $name {
            fn name(&self) -> &str {
                $display_name
            }

            async fn fetch(
                &self,
                ctx: &FetchContext<'_>,
            ) -> Result<NormalizedWeatherData, Box<dyn std::error::Error + Send + Sync>> {
                // If they configure a key, check for it. Otherwise, if not racing, we might require it.
                // In simulated mode, we just retrieve the key or fallback to a default mock key.
                let _key = ctx.api_keys.get($key_name).cloned().unwrap_or_else(|| "mock-key".to_string());

                let mut points = Vec::new();
                let today = chrono::Utc::now().naive_utc().date();

                // Generate highly stable, coordinate-derived pseudo-random weather
                let lat_factor = (ctx.lat.sin() * 5.0) as i32;
                let lon_factor = (ctx.lon.cos() * 5.0) as i32;
                let base_temp = 16.0 + (lat_factor + lon_factor) as f64;

                for d in 0..ctx.days {
                    let current_date = (today + chrono::Duration::days(d as i64)).format("%Y-%m-%d").to_string();
                    if current_date.as_str() < ctx.start_date || current_date.as_str() > ctx.end_date {
                        continue;
                    }
                    for h in 0..24 {
                        let hour_temp = base_temp + (h as f64 - 14.0).abs() * -0.4;
                        points.push(HourlyPoint {
                            time: format!("{}T{:02}:00:00", current_date, h),
                            temperature: (hour_temp * 10.0).round() / 10.0,
                            apparent_temperature: (hour_temp * 10.0).round() / 10.0,
                            precipitation_probability: 10.0,
                            precipitation: 0.0,
                            humidity: 60.0,
                            wind_speed: 12.0,
                            wind_direction: 180.0,
                            cloud_cover: 25.0,
                            weather_code: 1, // Partly cloudy
                            aqi: None,
                            uv_index: Some(5.0),
                            is_day: Some(h >= 6 && h <= 19),
                            visibility: Some(10.0),
                            soil_temperature: Some(15.0),
                            soil_moisture: Some(0.35),
                        });
                    }
                }

                Ok(NormalizedWeatherData {
                    provider_name: self.name().to_string(),
                    hourly: points,
                })
            }
        }
    };
}

define_simulated_provider!(AerisWeatherProvider, "AerisWeather", "AerisWeather");
define_simulated_provider!(StormGlassProvider, "StormGlass", "StormGlass");
define_simulated_provider!(MeteoBlueProvider, "MeteoBlue", "MeteoBlue");
define_simulated_provider!(ClimacellProvider, "Climacell", "Climacell");
define_simulated_provider!(AmbeeProvider, "Ambee", "Ambee");
define_simulated_provider!(OpenUvProvider, "OpenUV", "OpenUV");
define_simulated_provider!(OikolabProvider, "Oikolab", "Oikolab");
define_simulated_provider!(WeatherzoneProvider, "Weatherzone", "Weatherzone");
define_simulated_provider!(AemetProvider, "AEMET", "AEMET");
define_simulated_provider!(MeteoFranceProvider, "MeteoFrance", "MeteoFrance");
define_simulated_provider!(SmhiHistoricalProvider, "SMHI Historical", "SMHI-Historical");
define_simulated_provider!(JmaProvider, "JMA", "JMA");
